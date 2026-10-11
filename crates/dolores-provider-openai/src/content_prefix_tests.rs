use super::*;
use dolores_core::ReasoningControl;

fn wire(parts: &[&str]) -> String {
    let mut frames = parts
        .iter()
        .map(|part| {
            format!(
                "data: {}\n\n",
                json!({"choices":[{"index":0,"delta":{"content":part},"finish_reason":null}]})
            )
        })
        .collect::<String>();
    frames.push_str(&format!("data: {}\n\ndata: [DONE]\n\n", json!({"choices":[{"index":0,"delta":{},"finish_reason":"stop"}],"usage":{"prompt_tokens":12,"completion_tokens":8,"total_tokens":20}})));
    format!("HTTP/1.1 200 OK\r\nContent-Type: text/event-stream\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{frames}", frames.len())
}

#[tokio::test]
async fn thinking_off_normalizes_fragmented_prefix_on_all_response_paths() {
    let parts = [
        "<",
        "/thi",
        "nk>",
        "{\"description\":\"蓝色 square; literal </think> inside\"}",
    ];
    let raw = parts.concat();
    let clean = parts[3];
    for reasoning in [
        ReasoningControl::DeepseekThinkingOff,
        ReasoningControl::ProviderDefault,
    ] {
        let response = json!({"choices":[{"message":{"content":raw},"finish_reason":"stop"}],"usage":{"prompt_tokens":12,"completion_tokens":8,"total_tokens":20}}).to_string();
        let json_wire = format!("HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{response}", response.len());
        let (url, server) =
            crate::tests::sequence_server(vec![wire(&parts), wire(&parts), json_wire]).await;
        let provider = OpenAiProvider::with_settings(
            &ConnectionPreferences {
                base_url: url,
                model: "public-fixture".into(),
            },
            String::new(),
            RequestSettings {
                reasoning,
                ..Default::default()
            },
        )
        .unwrap();
        let expected = if reasoning == ReasoningControl::DeepseekThinkingOff {
            clean
        } else {
            &raw
        };
        let (tx, mut rx) = mpsc::channel(16);
        let usage = provider
            .stream_with_usage(vec![], tx, CancellationToken::new())
            .await
            .unwrap();
        let mut text = String::new();
        while let Some(part) = rx.recv().await {
            text.push_str(&part);
        }
        assert_eq!(text, expected);
        assert_eq!(usage.unwrap().total_tokens, Some(20));
        let (tx, mut rx) = mpsc::channel(16);
        let turn = provider
            .stream_tool_turn(&[], &[], tx, CancellationToken::new())
            .await
            .unwrap();
        text.clear();
        while let Some(part) = rx.recv().await {
            text.push_str(&part);
        }
        assert_eq!(text, expected);
        assert_eq!(turn.content, text);
        assert!(turn.calls.is_empty());
        let turn = provider
            .tool_turn(&[], &[], CancellationToken::new())
            .await
            .unwrap();
        assert_eq!(turn.content, expected);
        assert_eq!(turn.usage.unwrap().total_tokens, Some(20));
        server.await.unwrap();
    }
}
