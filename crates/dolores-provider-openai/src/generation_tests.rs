use super::*;
use dolores_core::{AgentMessage, ReasoningControl};

fn response(body: &str, stream: bool) -> String {
    format!(
        "HTTP/1.1 200 OK\r\nContent-Type: {}\r\nConnection: close\r\n\r\n{body}",
        if stream {
            "text/event-stream"
        } else {
            "application/json"
        }
    )
}
#[tokio::test]
async fn explicit_controls_reach_chat_streamed_and_strict_tool_requests() {
    use ReasoningControl::*;
    let stream="data: {\"choices\":[{\"delta\":{\"content\":\"done\"},\"finish_reason\":\"stop\"}]}\n\ndata: [DONE]\n\n";
    let strict = r#"{"choices":[{"message":{"content":"done"},"finish_reason":"stop"}]}"#;
    for reasoning in [
        ProviderDefault,
        DeepseekThinkingOff,
        OpenaiLow,
        OpenaiMedium,
        OpenaiHigh,
    ] {
        let (base_url, server) = tests::sequence_server(vec![
            response(stream, true),
            response(stream, true),
            response(strict, false),
        ])
        .await;
        let provider = OpenAiProvider::with_settings(
            &ConnectionPreferences {
                base_url,
                model: "fixture".into(),
            },
            String::new(),
            RequestSettings {
                max_output_tokens: 4096,
                timeout_seconds: 8,
                reasoning,
            },
        )
        .unwrap();
        let (tx, _rx) = mpsc::channel(32);
        provider
            .stream(vec![], tx, CancellationToken::new())
            .await
            .unwrap();
        let messages = vec![AgentMessage {
            parts: vec![],
            role: "user".into(),
            content: "work".into(),
            calls: vec![],
            call_id: None,
        }];
        let (tx, _rx) = mpsc::channel(32);
        provider
            .stream_tool_turn(&messages, &[], tx, CancellationToken::new())
            .await
            .unwrap();
        provider
            .tool_turn(&messages, &[], CancellationToken::new())
            .await
            .unwrap();
        let requests = server.await.unwrap();
        let mut count = 0;
        for body in requests.lines().filter(|line| line.starts_with('{')) {
            let value: Value = serde_json::from_str(body).unwrap();
            count += 1;
            match reasoning {
                ProviderDefault => {
                    assert!(value.get("thinking").is_none());
                    assert!(value.get("reasoning_effort").is_none());
                    assert_eq!(value["max_tokens"], 4096);
                }
                DeepseekThinkingOff => {
                    assert_eq!(value["thinking"], json!({"type":"disabled"}));
                    assert!(value.get("reasoning_effort").is_none());
                    assert_eq!(value["max_tokens"], 4096);
                }
                _ => {
                    assert!(value.get("max_tokens").is_none());
                    assert!(value.get("thinking").is_none());
                    assert_eq!(value["max_completion_tokens"], 4096);
                    assert_eq!(
                        value["reasoning_effort"],
                        match reasoning {
                            OpenaiLow => "low",
                            OpenaiMedium => "medium",
                            _ => "high",
                        }
                    );
                }
            }
        }
        assert_eq!(count, 3);
    }
}
#[tokio::test]
async fn rejected_reasoning_never_retries_or_echoes_provider_body() {
    let error = r#"{"error":{"param":"thinking","message":"unsupported thinking and stream_options PRIVATE_FIXTURE"}}"#;
    for tools in [false, true] {
        let rejected = format!(
            "HTTP/1.1 400 Bad Request\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{error}",
            error.len()
        );
        let (base_url, server) = tests::sequence_server(vec![rejected]).await;
        let provider = OpenAiProvider::with_settings(
            &ConnectionPreferences {
                base_url,
                model: "fixture".into(),
            },
            String::new(),
            RequestSettings {
                reasoning: ReasoningControl::DeepseekThinkingOff,
                ..Default::default()
            },
        )
        .unwrap();
        let (tx, _rx) = mpsc::channel(32);
        let result = if tools {
            provider
                .stream_tool_turn(&[], &[], tx, CancellationToken::new())
                .await
                .map(|_| ())
        } else {
            provider.stream(vec![], tx, CancellationToken::new()).await
        };
        let message = result.unwrap_err();
        assert!(message.starts_with("Model rejected generation settings."));
        assert!(!message.contains("PRIVATE_FIXTURE"));
        assert_eq!(server.await.unwrap().matches("POST ").count(), 1);
    }
}
