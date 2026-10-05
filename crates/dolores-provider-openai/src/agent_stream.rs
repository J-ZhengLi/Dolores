use crate::{reported_usage, usage_option_rejected, OpenAiProvider, SseDecoder};
use dolores_core::{
    validate_call, AgentMessage, AgentTurn, TokenUsage, ToolCall, ToolSpec, MAX_CONTEXT_BYTES,
    MAX_FILE_ARGUMENT_BYTES, MAX_OUTPUT_BYTES, MAX_TOOL_CALLS,
};
use futures_util::StreamExt;
use serde_json::{json, Value};
use std::{collections::BTreeMap, time::Duration};
use tokio::sync::mpsc;
use tokio_util::sync::CancellationToken;

const MAX_WIRE_BYTES: usize = 2 * 1024 * 1024;
const MAX_AGENT_FRAME_BYTES: usize = 256 * 1024;
const MAX_FRAMES: usize = 4096;
const INVALID: &str = "Model returned an invalid streamed tool response.";

#[derive(Default)]
struct PartialCall {
    id: String,
    name: String,
    arguments: String,
    function: bool,
}
#[derive(Default)]
struct Assembly {
    content: String,
    calls: BTreeMap<usize, PartialCall>,
    finish: Option<String>,
    usage: Option<TokenUsage>,
}
fn append(target: &mut String, value: &Value, limit: usize) -> Result<(), String> {
    if value.is_null() {
        return Ok(());
    }
    let text = value.as_str().ok_or(INVALID)?;
    if target.len() + text.len() > limit {
        return Err("Streamed tool field exceeds its byte limit.".into());
    }
    target.push_str(text);
    Ok(())
}
impl Assembly {
    // Only model text leaves the adapter. Partial names/arguments never become
    // a tool request, and no provider text can become an approval decision.
    fn push(&mut self, value: Value) -> Result<String, String> {
        if value.get("error").is_some() {
            return Err("Provider reported an error while streaming.".into());
        }
        if let Some(usage) = reported_usage(&value) {
            self.usage = Some(usage);
        }
        let choices = value["choices"].as_array().ok_or(INVALID)?;
        if choices.is_empty() {
            return Ok(String::new());
        }
        if choices.len() != 1 {
            return Err(INVALID.into());
        }
        let choice = &choices[0];
        if !choice["index"].is_null() && choice["index"] != 0 {
            return Err(INVALID.into());
        }
        let delta = choice["delta"].as_object().ok_or(INVALID)?;
        if delta.get("function_call").is_some_and(|v| !v.is_null()) {
            return Err("Model requested a legacy tool format. Check model support.".into());
        }
        let text = match delta.get("content") {
            None | Some(Value::Null) => "",
            Some(value) => value.as_str().ok_or(INVALID)?,
        };
        let has_calls = delta.get("tool_calls").is_some_and(|v| !v.is_null());
        if self.finish.is_some()
            && (!text.is_empty() || has_calls || !choice["finish_reason"].is_null())
        {
            return Err("Model sent response data after finishing.".into());
        }
        if self.content.len() + text.len() > MAX_OUTPUT_BYTES {
            return Err("Response exceeds the 128 KiB limit.".into());
        }
        self.content.push_str(text);
        if let Some(calls) = delta.get("tool_calls").filter(|v| !v.is_null()) {
            let calls = calls.as_array().ok_or(INVALID)?;
            if calls.len() > MAX_TOOL_CALLS {
                return Err("Model returned too many tool calls.".into());
            }
            for part in calls {
                let index = part["index"].as_u64().ok_or(INVALID)?;
                if index >= MAX_TOOL_CALLS as u64 {
                    return Err("Model returned an invalid tool index.".into());
                }
                let call = self.calls.entry(index as usize).or_default();
                if !part["type"].is_null() {
                    if part["type"] != "function" {
                        return Err("Model requested an unsupported tool type.".into());
                    }
                    call.function = true;
                }
                append(&mut call.id, &part["id"], 128)?;
                if let Some(function) = part.get("function").filter(|v| !v.is_null()) {
                    let function = function.as_object().ok_or(INVALID)?;
                    append(
                        &mut call.name,
                        function.get("name").unwrap_or(&Value::Null),
                        128,
                    )?;
                    append(
                        &mut call.arguments,
                        function.get("arguments").unwrap_or(&Value::Null),
                        MAX_FILE_ARGUMENT_BYTES,
                    )?;
                }
            }
        }
        if let Some(reason) = choice["finish_reason"].as_str() {
            if !matches!(reason, "stop" | "tool_calls" | "length") {
                return Err("Model finished without a complete tool response.".into());
            }
            self.finish = Some(reason.to_owned());
        } else if !choice["finish_reason"].is_null() {
            return Err(INVALID.into());
        }
        Ok(text.to_owned())
    }
    fn complete(self) -> Result<AgentTurn, String> {
        if self.finish.as_deref() == Some("length") {
            // Never publish incomplete/pending tool calls as executable work.
            return Ok(AgentTurn {
                content: self.content,
                calls: vec![],
                usage: self.usage,
                output_limit: true,
            });
        }
        let expected = if self.calls.is_empty() {
            "stop"
        } else {
            "tool_calls"
        };
        if self.finish.as_deref() != Some(expected) {
            return Err("Connection ended before the tool response finished.".into());
        }
        let mut calls = Vec::new();
        for (expected_index, (index, part)) in self.calls.into_iter().enumerate() {
            if index != expected_index || !part.function {
                return Err(INVALID.into());
            }
            let call = ToolCall {
                id: part.id,
                name: part.name,
                arguments: part.arguments,
            };
            validate_call(&call)?;
            if !serde_json::from_str::<Value>(&call.arguments).is_ok_and(|v| v.is_object()) {
                return Err("Model returned incomplete or invalid tool arguments.".into());
            }
            if calls.iter().any(|prior: &ToolCall| prior.id == call.id) {
                return Err("Model reused a tool call ID.".into());
            }
            calls.push(call);
        }
        Ok(AgentTurn {
            output_limit: false,
            content: self.content,
            calls,
            usage: self.usage,
        })
    }
}

impl OpenAiProvider {
    pub(crate) async fn request_stream_tool_turn(
        &self,
        messages: &[AgentMessage],
        tools: &[ToolSpec],
        output: mpsc::Sender<String>,
        cancel: CancellationToken,
    ) -> Result<AgentTurn, String> {
        tokio::select! { biased;
            _ = cancel.cancelled() => Err("Response stopped.".into()),
            result = tokio::time::timeout(Duration::from_secs(self.settings.timeout_seconds.into()),
                self.stream_agent_request(messages, tools, output, cancel.clone())) =>
                result.map_err(|_| "Model request timed out. Adjust the request timeout or try again.")?,
        }
    }
    async fn stream_agent_request(
        &self,
        messages: &[AgentMessage],
        tools: &[ToolSpec],
        output: mpsc::Sender<String>,
        cancel: CancellationToken,
    ) -> Result<AgentTurn, String> {
        let wire_limit = MAX_CONTEXT_BYTES
            + if messages
                .iter()
                .any(|m| m.parts.iter().any(|p| p.is_image()))
            {
                12 * 1024 * 1024
            } else {
                0
            };
        let messages = self.wire_agent_messages(messages)?;
        if serde_json::to_vec(&messages)
            .map_err(|_| "Could not prepare tool request.")?
            .len()
            > wire_limit
        {
            return Err("Tool context exceeds the 128 KiB limit.".into());
        }
        let specs: Vec<_> = tools.iter().map(|s| json!({"type":"function","function":{"name":s.name,"description":s.description,"parameters":s.parameters}})).collect();
        let mut body = json!({"model":self.model,"messages":messages,"tools":specs,"tool_choice":"auto","parallel_tool_calls":false,"stream":true,"max_tokens":self.settings.max_output_tokens,"stream_options":{"include_usage":true}});
        self.apply_generation_settings(&mut body);
        let mut include_usage = true;
        let mut response = loop {
            let mut request = self.client.post(self.endpoint.clone()).json(&body);
            if !self.api_key.is_empty() {
                request = request.bearer_auth(&self.api_key);
            }
            let mut response = request.send().await.map_err(|_| {
                "Could not reach the model. Check the endpoint and whether the server is running."
            })?;
            if include_usage && usage_option_rejected(&mut response, &cancel).await? {
                include_usage = false;
                body.as_object_mut().unwrap().remove("stream_options");
                continue;
            }
            break response;
        };
        if !response.status().is_success() {
            crate::check_context_limit(&mut response, &cancel).await?;
            return Err(match response.status().as_u16() {
                401 | 403 => "Model access denied. Check your API key and permissions.".into(),
                429 => "Model rate limit reached. Try again later.".into(),
                status => format!("Tool request failed (HTTP {status}). Start a side chat or check model support."),
            });
        }
        if !response
            .headers()
            .get(reqwest::header::CONTENT_TYPE)
            .and_then(|v| v.to_str().ok())
            .is_some_and(|v| v.starts_with("text/event-stream"))
        {
            return Err("Model did not return a streamed tool response. Start a side chat or check model support.".into());
        }
        let mut stream = response.bytes_stream();
        let mut decoder = SseDecoder::default();
        let mut assembly = Assembly::default();
        let mut wire_bytes = 0;
        let mut frames = 0;
        while let Some(chunk) = stream.next().await {
            let chunk =
                chunk.map_err(|_| "Connection interrupted before the response finished.")?;
            wire_bytes += chunk.len();
            if wire_bytes > MAX_WIRE_BYTES {
                return Err("Streamed tool response exceeds the 2 MiB wire limit.".into());
            }
            for data in decoder.push(&chunk)? {
                frames += 1;
                if frames > MAX_FRAMES || data.len() > MAX_AGENT_FRAME_BYTES {
                    return Err("Streamed tool response exceeds its frame limit.".into());
                }
                if data == "[DONE]" {
                    return assembly.complete();
                }
                let value = serde_json::from_str(&data)
                    .map_err(|_| "Provider returned a malformed stream.")?;
                let text = assembly.push(value)?;
                if !text.is_empty() {
                    tokio::select! { biased;
                        _ = cancel.cancelled() => return Err("Response stopped.".into()),
                        result = output.send(text) => result.map_err(|_| "Conversation window closed.")?,
                    }
                }
            }
        }
        if !decoder.buffer.is_empty() {
            return Err("Connection ended before the tool response finished.".into());
        }
        assembly.complete()
    }
}

#[cfg(test)]
#[path = "agent_stream_tests.rs"]
mod tests;
