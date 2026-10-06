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

const MAX_IDLE_WIRE_BYTES: usize = 2 * 1024 * 1024;
const MAX_AGENT_FRAME_BYTES: usize = 256 * 1024;
const INVALID: &str = "Model returned an invalid streamed tool response.";

fn thinking_preview(reasoning: &str) -> String {
    const LIMIT: usize = 16 * 1024;
    const OMITTED: &str = "[Earlier thinking omitted]\n\n";
    if reasoning.len() <= LIMIT {
        return reasoning.to_owned();
    }
    let mut start = reasoning.len() - (LIMIT - OMITTED.len());
    while !reasoning.is_char_boundary(start) {
        start += 1;
    }
    format!("{OMITTED}{}", &reasoning[start..])
}

#[derive(Default)]
struct PartialCall {
    id: String,
    name: String,
    arguments: String,
    function: bool,
}
#[derive(Default)]
struct Assembly {
    reasoning: Option<String>,
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
    fn payload_bytes(&self) -> usize {
        self.content.len()
            + self.reasoning.as_ref().map_or(0, String::len)
            + self
                .calls
                .values()
                .map(|c| c.id.len() + c.name.len() + c.arguments.len())
                .sum::<usize>()
    }
    fn activity(&self) -> dolores_core::ModelActivity {
        use dolores_core::ModelActivity;
        if !self.calls.is_empty() {
            ModelActivity::ToolArguments
        } else if !self.content.is_empty() {
            ModelActivity::Responding
        } else if self.reasoning.as_ref().is_some_and(|r| !r.is_empty()) {
            ModelActivity::Reasoning
        } else {
            ModelActivity::Waiting
        }
    }
    // Text and thinking use separate channels. Partial names/arguments never become
    // a tool request, and no provider text can become an approval decision.
    fn push(&mut self, value: Value) -> Result<String, String> {
        if value.get("error").is_some() {
            return Err("Provider reported an error while streaming.".into());
        }
        if let Some(usage) = reported_usage(&value) {
            self.usage = Some(usage);
        }
        let choices = value["choices"].as_array().ok_or(INVALID)?;
        if let Some(reasoning) = value
            .pointer("/choices/0/delta/reasoning_content")
            .filter(|v| !v.is_null())
        {
            append(
                self.reasoning.get_or_insert_with(String::new),
                reasoning,
                super::agent::MAX_REASONING_BYTES,
            )?;
        }
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
        self.request_stream_tool_turn_with_activity(messages, tools, output, None, None, cancel)
            .await
    }
    pub(crate) async fn request_stream_tool_turn_with_activity(
        &self,
        messages: &[AgentMessage],
        tools: &[ToolSpec],
        output: mpsc::Sender<String>,
        activity: Option<mpsc::Sender<dolores_core::ModelActivity>>,
        thinking: Option<mpsc::Sender<String>>,
        cancel: CancellationToken,
    ) -> Result<AgentTurn, String> {
        tokio::select! { biased;
            _ = cancel.cancelled() => Err("Response stopped.".into()),
            result = self.stream_agent_request(messages, tools, output, activity, thinking, cancel.clone()) => result,
        }
    }
    async fn stream_agent_request(
        &self,
        messages: &[AgentMessage],
        tools: &[ToolSpec],
        output: mpsc::Sender<String>,
        activity: Option<mpsc::Sender<dolores_core::ModelActivity>>,
        thinking: Option<mpsc::Sender<String>>,
        cancel: CancellationToken,
    ) -> Result<AgentTurn, String> {
        let mut wire_limit = MAX_CONTEXT_BYTES
            + if messages
                .iter()
                .any(|m| m.parts.iter().any(|p| p.is_image()))
            {
                12 * 1024 * 1024
            } else {
                0
            };
        let messages = self.wire_agent_messages(messages)?;
        wire_limit += super::agent::reasoning_wire_bytes(&messages);
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
        let connection_deadline =
            tokio::time::Instant::now() + Duration::from_secs(self.settings.timeout_seconds.into());
        let mut response = loop {
            let mut request = self.client.post(self.endpoint.clone()).json(&body);
            if !self.api_key.is_empty() {
                request = request.bearer_auth(&self.api_key);
            }
            let mut response = tokio::time::timeout_at(connection_deadline, request.send()).await.map_err(|_| "Model connection stalled before a response. Check the provider or try again explicitly.")?.map_err(|_| {
                "Could not reach the model. Check the endpoint and whether the server is running."
            })?;
            if include_usage && tokio::time::timeout_at(connection_deadline, usage_option_rejected(&mut response, &cancel)).await.map_err(|_| "Model connection stalled before a response. Check the provider or try again explicitly.")?? {
                include_usage = false;
                body.as_object_mut().unwrap().remove("stream_options");
                continue;
            }
            break response;
        };
        if !response.status().is_success() {
            tokio::time::timeout_at(connection_deadline, crate::check_context_limit(&mut response, &cancel)).await.map_err(|_| "Model connection stalled before a response. Check the provider or try again explicitly.")??;
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
        let mut idle_wire_bytes = 0usize;
        let allowance = Duration::from_secs(self.settings.timeout_seconds.into());
        let mut deadline = tokio::time::Instant::now() + allowance;
        let mut preview_bytes = 0;
        loop {
            if tokio::time::Instant::now() >= deadline {
                return Err("Model stream stalled without new response data. Progress is retained; continue explicitly or check the provider.".into());
            }
            let next = tokio::time::timeout_at(deadline, stream.next()).await.map_err(|_| "Model stream stalled without new response data. Progress is retained; continue explicitly or check the provider.")?;
            let Some(chunk) = next else { break };
            let chunk =
                chunk.map_err(|_| "Connection interrupted before the response finished.")?;
            idle_wire_bytes = idle_wire_bytes.saturating_add(chunk.len());
            let previous_payload = assembly.payload_bytes();
            let mut done = false;
            for data in decoder.push(&chunk)? {
                // Event count reflects provider fragmentation, not generated work.
                // Bound fields and inactivity, not the total time of an active stream.
                if data.len() > MAX_AGENT_FRAME_BYTES {
                    return Err("Provider sent an oversized stream event (256 KiB maximum). No incomplete tool call was executed. Try a smaller generated file or check the provider's streaming support.".into());
                }
                if data == "[DONE]" {
                    done = true;
                    break;
                }
                let value = serde_json::from_str(&data)
                    .map_err(|_| "Provider returned a malformed stream.")?;
                let text = assembly.push(value)?;
                let phase = assembly.activity();
                if assembly.payload_bytes() > previous_payload {
                    if let Some(activity) = &activity {
                        // These pulses report actual decoded progress, never heartbeats.
                        let _ = activity.try_send(phase);
                    }
                    if let (Some(thinking), Some(reasoning)) = (&thinking, &assembly.reasoning) {
                        // Latest bounded snapshot: a slow observer cannot lose a delta
                        // and cannot block the provider. Full protocol text stays private.
                        let end = reasoning.len();
                        if end > preview_bytes
                            && (preview_bytes == 0
                                || end - preview_bytes >= 128
                                || phase != dolores_core::ModelActivity::Reasoning)
                            && thinking.try_send(thinking_preview(reasoning)).is_ok()
                        {
                            preview_bytes = end;
                        }
                    }
                }
                if !text.is_empty() {
                    tokio::select! { biased;
                        _ = cancel.cancelled() => return Err("Response stopped.".into()),
                        result = tokio::time::timeout(allowance, output.send(text)) => result.map_err(|_| "Conversation delivery stalled. Progress is retained; reopen the chat before continuing.")?.map_err(|_| "Conversation window closed.")?,
                    }
                }
            }
            if assembly.payload_bytes() > previous_payload {
                idle_wire_bytes = 0;
                deadline = tokio::time::Instant::now() + allowance;
            }
            if idle_wire_bytes > MAX_IDLE_WIRE_BYTES {
                return Err("Provider stream sent 2 MiB without new response data. Stop and check the provider or try again explicitly. No incomplete tool call was executed.".into());
            }
            if done {
                let reasoning = assembly.reasoning.take();
                let turn = assembly.complete()?;
                self.remember_reasoning(&turn, reasoning)?;
                return Ok(turn);
            }
            // Keep deadline and Stop responsive even with continuously ready metadata.
            tokio::task::yield_now().await;
        }
        if !decoder.buffer.is_empty() {
            return Err("Connection ended before the tool response finished.".into());
        }
        let reasoning = assembly.reasoning.take();
        let turn = assembly.complete()?;
        self.remember_reasoning(&turn, reasoning)?;
        Ok(turn)
    }
}

#[cfg(test)]
#[path = "agent_stream_tests.rs"]
mod tests;
