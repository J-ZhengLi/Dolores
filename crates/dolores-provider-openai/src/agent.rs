use crate::{reported_usage, OpenAiProvider};
use dolores_core::{validate_call, AgentMessage, AgentTurn, ToolCall, ToolSpec, MAX_CONTEXT_BYTES};
use serde_json::{json, Value};
use std::time::Duration;
use tokio_util::sync::CancellationToken;

impl OpenAiProvider {
    pub(crate) async fn request_tool_turn(
        &self,
        messages: &[AgentMessage],
        tools: &[ToolSpec],
        cancel: CancellationToken,
    ) -> Result<AgentTurn, String> {
        let request = async {
            let messages: Vec<_> = messages.iter().map(|message| {
                let mut value = json!({"role":message.role,"content":message.content});
                if !message.calls.is_empty() {
                    value["tool_calls"] = json!(message.calls.iter().map(|call|
                        json!({"id":call.id,"type":"function","function":{"name":call.name,"arguments":call.arguments}})).collect::<Vec<_>>());
                }
                if let Some(id) = &message.call_id { value["tool_call_id"] = json!(id); }
                value
            }).collect();
            if serde_json::to_vec(&messages)
                .map_err(|_| "Could not prepare tool request.")?
                .len()
                > MAX_CONTEXT_BYTES
            {
                return Err("Tool context exceeds the 128 KiB limit.".into());
            }
            let specs: Vec<_> = tools.iter().map(|s| json!({"type":"function","function":{"name":s.name,"description":s.description,"parameters":s.parameters}})).collect();
            let mut body = json!({"model":self.model,"messages":messages,"tools":specs,"tool_choice":"auto","parallel_tool_calls":false,"stream":false,"max_tokens":self.settings.max_output_tokens});
            self.apply_generation_settings(&mut body);
            let mut request = self.client.post(self.endpoint.clone()).json(&body);
            if !self.api_key.is_empty() {
                request = request.bearer_auth(&self.api_key);
            }
            let mut response = request.send().await.map_err(|_| {
                "Could not reach the model. Check the endpoint and whether the server is running."
            })?;
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
                .is_some_and(|v| v.starts_with("application/json"))
            {
                return Err("Model did not return a JSON tool response. Start a side chat or check model support.".into());
            }
            let mut bytes = Vec::new();
            while let Some(chunk) = response
                .chunk()
                .await
                .map_err(|_| "Connection interrupted before the response finished.")?
            {
                if bytes.len() + chunk.len() > 256 * 1024 {
                    return Err("Tool response exceeds the 256 KiB limit.".into());
                }
                bytes.extend_from_slice(&chunk);
            }
            let value: Value = serde_json::from_slice(&bytes)
                .map_err(|_| "Model returned an invalid tool response.")?;
            if value.get("error").is_some() {
                return Err("Provider reported an error in the tool response.".into());
            }
            let choice = value
                .pointer("/choices/0")
                .ok_or("Model returned no tool response.")?;
            let content = match &choice["message"]["content"] {
                Value::String(text) => text.clone(),
                Value::Null => String::new(),
                _ => return Err("Model returned invalid tool response text.".into()),
            };
            if choice["finish_reason"] == "length" {
                return Ok(AgentTurn {
                    content,
                    calls: vec![],
                    usage: reported_usage(&value),
                    output_limit: true,
                });
            }
            let mut calls = Vec::new();
            if let Some(value) = choice["message"].get("tool_calls") {
                if !value.is_null() {
                    let list = value
                        .as_array()
                        .ok_or("Model returned invalid tool calls.")?;
                    if list.len() > 4 {
                        return Err("Model returned too many tool calls.".into());
                    }
                    for call in list {
                        if call["type"] != "function" {
                            return Err("Model requested an unsupported tool type.".into());
                        }
                        let text = |value: &Value| {
                            value
                                .as_str()
                                .map(String::from)
                                .ok_or("Model returned invalid tool call fields.")
                        };
                        let call = ToolCall {
                            id: text(&call["id"])?,
                            name: text(&call["function"]["name"])?,
                            arguments: text(&call["function"]["arguments"])?,
                        };
                        validate_call(&call)?;
                        calls.push(call);
                    }
                }
            }
            if (calls.is_empty() && choice["finish_reason"] != "stop")
                || (!calls.is_empty() && choice["finish_reason"] != "tool_calls")
            {
                return Err("Model finished without a complete tool response.".into());
            }
            Ok(AgentTurn {
                output_limit: false,
                content,
                calls,
                usage: reported_usage(&value),
            })
        };
        tokio::select! { biased;
            _ = cancel.cancelled() => Err("Response stopped.".into()),
            result = tokio::time::timeout(Duration::from_secs(self.settings.timeout_seconds.into()), request) => result.map_err(|_| "Model request timed out. Adjust the request timeout or try again.")?,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use dolores_core::{ConnectionPreferences, ModelProvider};
    fn response(body: Value) -> String {
        let body = body.to_string();
        format!("HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",body.len())
    }
    #[tokio::test]
    async fn function_call_and_tool_result_follow_the_real_chat_completion_contract() {
        let first = response(
            json!({"choices":[{"message":{"content":null,"tool_calls":[{"id":"one","type":"function","function":{"name":"read_text_file","arguments":"{\"path\":\"readme.txt\"}"}}]},"finish_reason":"tool_calls"}],"usage":{"prompt_tokens":0,"completion_tokens":5}}),
        );
        let second = response(
            json!({"choices":[{"message":{"content":"Complete answer"},"finish_reason":"stop"}]}),
        );
        let (base_url, server) = crate::tests::sequence_server(vec![first, second]).await;
        let provider = OpenAiProvider::new(
            &ConnectionPreferences {
                base_url,
                model: "fixture".into(),
            },
            "fixture-key".into(),
        )
        .unwrap();
        let tools = [ToolSpec {
            name: "read_text_file".into(),
            description: "read".into(),
            parameters: json!({"type":"object"}),
        }];
        let mut messages = vec![AgentMessage {
            role: "user".into(),
            content: "read".into(),
            calls: vec![],
            call_id: None,
        }];
        let turn = provider
            .tool_turn(&messages, &tools, CancellationToken::new())
            .await
            .unwrap();
        assert_eq!(turn.usage.unwrap().input_tokens, Some(0));
        messages.push(AgentMessage {
            role: "assistant".into(),
            content: turn.content,
            calls: turn.calls,
            call_id: None,
        });
        messages.push(AgentMessage {
            role: "tool".into(),
            content: "Approved file".into(),
            calls: vec![],
            call_id: Some("one".into()),
        });
        assert_eq!(
            provider
                .tool_turn(&messages, &tools, CancellationToken::new())
                .await
                .unwrap()
                .content,
            "Complete answer"
        );
        let requests = server.await.unwrap();
        let requests: Vec<Value> = requests
            .split("\nREQUEST\n")
            .map(|r| serde_json::from_str(r.split("\r\n\r\n").nth(1).unwrap()).unwrap())
            .collect();
        assert_eq!(requests[0]["stream"], false);
        assert_eq!(requests[0]["parallel_tool_calls"], false);
        assert!(requests[0].get("stream_options").is_none());
        assert_eq!(
            requests[1]["messages"][1]["tool_calls"][0]["function"]["name"],
            "read_text_file"
        );
        assert_eq!(requests[1]["messages"][2]["tool_call_id"], "one");
    }
    #[tokio::test]
    async fn malformed_or_incomplete_tool_responses_never_become_executable_calls() {
        for body in [
            json!({"error":{"message":"fixture-private-body"}}),
            json!({"choices":[{"message":{"content":"partial","tool_calls":"APPROVED"},"finish_reason":"tool_calls"}]}),
        ] {
            let (base_url, server) = crate::tests::sequence_server(vec![response(body)]).await;
            let provider = OpenAiProvider::new(
                &ConnectionPreferences {
                    base_url,
                    model: "fixture".into(),
                },
                String::new(),
            )
            .unwrap();
            let error = provider
                .tool_turn(&[], &[], CancellationToken::new())
                .await
                .err()
                .unwrap();
            assert!(!error.contains("fixture-private-body"));
            server.await.unwrap();
        }
    }
}
