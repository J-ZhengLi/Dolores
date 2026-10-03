use super::*;
use dolores_core::{ConnectionPreferences, ModelProvider, RequestSettings};

fn delta(value: Value, finish: Value) -> Value {
    json!({"choices":[{"index":0,"delta":value,"finish_reason":finish}]})
}
fn call(index: usize, id: &str, name: &str, arguments: &str) -> Value {
    json!({"index":index,"id":id,"type":"function","function":{"name":name,"arguments":arguments}})
}
fn wire(frames: &[Value], done: bool) -> String {
    let mut wire = frames
        .iter()
        .map(|v| format!("data: {v}\r\n\r\n"))
        .collect::<String>();
    if done {
        wire.push_str("data: [DONE]\n\n");
    }
    format!("HTTP/1.1 200 OK\r\nContent-Type: text/event-stream\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{wire}", wire.len())
}
fn provider(base_url: String) -> OpenAiProvider {
    OpenAiProvider::new(
        &ConnectionPreferences {
            base_url,
            model: "fixture".into(),
        },
        String::new(),
    )
    .unwrap()
}

#[tokio::test]
async fn output_limit_keeps_final_text_usage_and_never_returns_partial_calls() {
    for call_parts in [
        Value::Null,
        json!([call(0, "unfinished", "create_text_file", "{\"path\":")]),
    ] {
        let frames = [
            delta(
                json!({"content":"saved 世界", "tool_calls":call_parts}),
                json!("length"),
            ),
            json!({"choices":[],"usage":{"prompt_tokens":11,"completion_tokens":64}}),
        ];
        let (endpoint, server) = crate::tests::sequence_server(vec![wire(&frames, true)]).await;
        let (output, mut receiver) = mpsc::channel(32);
        let turn = provider(endpoint)
            .stream_tool_turn(&[], &[], output, CancellationToken::new())
            .await
            .unwrap();
        assert!(turn.output_limit);
        assert!(turn.calls.is_empty());
        assert_eq!(turn.content, "saved 世界");
        assert_eq!(receiver.recv().await.unwrap(), turn.content);
        assert_eq!(turn.usage.unwrap().output_tokens, Some(64));
        server.await.unwrap();
    }
}

#[test]
fn fragmented_unicode_and_interleaved_calls_assemble_by_index() {
    let frames = [
        delta(
            json!({"content":"I will read 世界. ","tool_calls":[call(1,"two","search_text","{\"query\":\""),call(0,"one","read_text_file","{\"path\":")]}),
            Value::Null,
        ),
        delta(
            json!({"tool_calls":[{"index":0,"function":{"arguments":"\"readme.txt\"}"}},{"index":1,"function":{"arguments":"世界\",\"path\":\".\"}"}}]}),
            json!("tool_calls"),
        ),
        json!({"choices":[],"usage":{"prompt_tokens":0,"completion_tokens":5}}),
    ];
    let data = frames
        .iter()
        .map(|v| format!("data: {v}\r\n\r\n"))
        .collect::<String>();
    let mut decoder = SseDecoder::default();
    let mut assembly = Assembly::default();
    let mut text = String::new();
    for byte in data.bytes() {
        for frame in decoder.push(&[byte]).unwrap() {
            text.push_str(
                &assembly
                    .push(serde_json::from_str(&frame).unwrap())
                    .unwrap(),
            );
        }
    }
    let turn = assembly.complete().unwrap();
    assert_eq!(turn.content, text);
    assert_eq!(turn.calls[0].id, "one");
    assert_eq!(turn.calls[1].arguments, r#"{"query":"世界","path":"."}"#);
    assert_eq!(turn.usage.unwrap().input_tokens, Some(0));
}

#[test]
fn incomplete_malformed_and_over_budget_calls_never_complete() {
    for frames in [
        vec![delta(
            json!({"tool_calls":[call(0,"one","read_text_file","{\"path\":")] }),
            Value::Null,
        )],
        vec![delta(
            json!({"tool_calls":[call(0,"one","read_text_file","{")] }),
            json!("tool_calls"),
        )],
        vec![delta(
            json!({"tool_calls":[call(4,"one","read_text_file","{}")] }),
            json!("tool_calls"),
        )],
        vec![delta(
            json!({"tool_calls":[call(1,"one","read_text_file","{}")] }),
            json!("tool_calls"),
        )],
        vec![delta(
            json!({"tool_calls":[call(0,"one","read_text_file",&"x".repeat(4097))] }),
            json!("tool_calls"),
        )],
        vec![delta(
            json!({"tool_calls":[call(0,&"x".repeat(129),"read_text_file","{}")] }),
            json!("tool_calls"),
        )],
        vec![delta(
            json!({"tool_calls":[call(0,"one","read_text_file","{}"),call(1,"one","read_text_file","{}")] }),
            json!("tool_calls"),
        )],
        vec![delta(
            json!({"tool_calls":[call(0,"one","read_text_file","{}")] }),
            json!("stop"),
        )],
        vec![delta(
            json!({"content":"x".repeat(MAX_OUTPUT_BYTES + 1)}),
            json!("stop"),
        )],
        vec![
            delta(json!({"content":"complete"}), json!("stop")),
            delta(json!({"content":"late"}), Value::Null),
        ],
        vec![json!({"error":{"message":"fixture-private-body"}})],
    ] {
        let mut assembly = Assembly::default();
        let result = frames
            .into_iter()
            .try_for_each(|v| assembly.push(v).map(|_| ()));
        let error = result
            .and_then(|_| assembly.complete().map(|_| ()))
            .unwrap_err();
        assert!(!error.contains("fixture-private-body"));
    }
    assert!(Assembly::default().complete().is_err());
}

#[tokio::test]
async fn http_stream_preserves_usage_and_the_real_tool_message_contract() {
    let first = wire(
        &[
            delta(
                json!({"content":"Let me inspect it.","tool_calls":[call(0,"one","read_text_file","{\"path\":")]}),
                Value::Null,
            ),
            delta(
                json!({"tool_calls":[{"index":0,"function":{"arguments":"\"readme.txt\"}"}}]}),
                json!("tool_calls"),
            ),
            json!({"choices":[],"usage":{"prompt_tokens":0,"completion_tokens":5}}),
        ],
        true,
    );
    let second = wire(
        &[
            delta(json!({"content":"Complete "}), Value::Null),
            delta(json!({"content":"answer 世界"}), json!("stop")),
            json!({"choices":[],"usage":{"prompt_tokens":30,"completion_tokens":8}}),
            json!({"choices":[],"usage":{"prompt_tokens":30,"completion_tokens":8}}),
        ],
        true,
    );
    let (base_url, server) = crate::tests::sequence_server(vec![first, second]).await;
    let provider = provider(base_url);
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
    let (sender, mut receiver) = mpsc::channel(32);
    let turn = provider
        .stream_tool_turn(&messages, &tools, sender, CancellationToken::new())
        .await
        .unwrap();
    assert_eq!(receiver.recv().await.unwrap(), "Let me inspect it.");
    assert!(receiver.recv().await.is_none());
    assert_eq!(turn.usage.unwrap().input_tokens, Some(0));
    messages.push(AgentMessage {
        role: "assistant".into(),
        content: turn.content,
        calls: turn.calls,
        call_id: None,
    });
    messages.push(AgentMessage {
        role: "tool".into(),
        content: "Approved text".into(),
        calls: vec![],
        call_id: Some("one".into()),
    });
    let (sender, mut receiver) = mpsc::channel(32);
    let final_turn = provider
        .stream_tool_turn(&messages, &tools, sender, CancellationToken::new())
        .await
        .unwrap();
    let mut text = String::new();
    while let Some(delta) = receiver.recv().await {
        text.push_str(&delta);
    }
    assert_eq!(text, final_turn.content);
    assert_eq!(text, "Complete answer 世界");
    assert_eq!(final_turn.usage.unwrap().output_tokens, Some(8));
    let requests = server.await.unwrap();
    let requests: Vec<Value> = requests
        .split("\nREQUEST\n")
        .map(|r| serde_json::from_str(r.split("\r\n\r\n").nth(1).unwrap()).unwrap())
        .collect();
    assert_eq!(requests[0]["stream"], true);
    assert_eq!(requests[0]["stream_options"]["include_usage"], true);
    assert_eq!(requests[0]["parallel_tool_calls"], false);
    assert_eq!(requests[1]["messages"][1]["content"], "Let me inspect it.");
    assert_eq!(requests[1]["messages"][2]["tool_call_id"], "one");
}

#[tokio::test]
async fn incomplete_stream_and_done_without_finish_do_not_return_calls() {
    for done in [false, true] {
        let response = wire(
            &[delta(
                json!({"content":"partial","tool_calls":[call(0,"one","read_text_file","{\"path\":")]}),
                Value::Null,
            )],
            done,
        );
        let (base_url, server) = crate::tests::sequence_server(vec![response]).await;
        let (sender, _receiver) = mpsc::channel(8);
        let error = provider(base_url)
            .stream_tool_turn(&[], &[], sender, CancellationToken::new())
            .await
            .err()
            .unwrap();
        assert!(error.contains("before the tool response finished"));
        server.await.unwrap();
    }
}

#[tokio::test]
async fn unsupported_usage_retries_only_before_stream_and_keeps_tools_enabled() {
    let rejected =
        "{\"error\":{\"param\":\"stream_options\",\"message\":\"Unsupported stream_options\"}}";
    let rejection = format!("HTTP/1.1 400 Bad Request\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{rejected}",rejected.len());
    let (base_url, server) = crate::tests::sequence_server(vec![
        rejection,
        wire(&[delta(json!({"content":"complete"}), json!("stop"))], true),
    ])
    .await;
    let (sender, _receiver) = mpsc::channel(8);
    let turn = provider(base_url)
        .stream_tool_turn(&[], &[], sender, CancellationToken::new())
        .await
        .unwrap();
    assert_eq!(turn.content, "complete");
    assert!(turn.usage.is_none());
    let requests = server.await.unwrap();
    let requests: Vec<Value> = requests
        .split("\nREQUEST\n")
        .map(|r| serde_json::from_str(r.split("\r\n\r\n").nth(1).unwrap()).unwrap())
        .collect();
    assert!(requests[1].get("stream_options").is_none());
    assert_eq!(requests[1]["stream"], true);
    assert!(requests[1].get("tools").is_some());
}

#[tokio::test]
async fn stream_delivery_is_cancelable_and_under_the_configured_deadline() {
    for stop in [false, true] {
        let (base_url, server) = crate::tests::sequence_server(vec![wire(
            &[
                delta(json!({"content":"one"}), Value::Null),
                delta(json!({"content":"two"}), json!("stop")),
            ],
            true,
        )])
        .await;
        let provider = OpenAiProvider::with_settings(
            &ConnectionPreferences {
                base_url,
                model: "fixture".into(),
            },
            String::new(),
            RequestSettings {
                max_output_tokens: 2048,
                timeout_seconds: 1,
            },
        )
        .unwrap();
        let (sender, _receiver) = mpsc::channel(1);
        let cancel = CancellationToken::new();
        let token = cancel.clone();
        let timer = tokio::spawn(async move {
            if stop {
                tokio::time::sleep(Duration::from_millis(30)).await;
                token.cancel();
            }
        });
        let error = provider
            .stream_tool_turn(&[], &[], sender, cancel)
            .await
            .err()
            .unwrap();
        assert!(error.contains(if stop { "stopped" } else { "timed out" }));
        timer.await.unwrap();
        server.await.unwrap();
    }
}

#[tokio::test]
async fn generic_rejections_and_stream_errors_do_not_retry_or_fall_back_to_plain_chat() {
    for response in [
        "HTTP/1.1 400 Bad Request\r\nContent-Length: 0\r\nConnection: close\r\n\r\n".into(),
        "HTTP/1.1 401 Unauthorized\r\nContent-Length: 0\r\nConnection: close\r\n\r\n".into(),
        "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: 2\r\nConnection: close\r\n\r\n{}".into(),
        wire(&[delta(json!({"content":"partial"}),Value::Null),json!({"error":{"message":"fixture-private-body"}})],true),
    ] {
        let (base_url, server) = crate::tests::sequence_server(vec![response]).await;
        let (sender, _receiver) = mpsc::channel(8);
        let error = provider(base_url).stream_tool_turn(&[],&[],sender,CancellationToken::new()).await.err().unwrap();
        assert!(!error.contains("fixture-private-body"));
        assert_eq!(server.await.unwrap().split("\nREQUEST\n").count(),1);
    }
}

#[tokio::test]
async fn stream_wire_frame_and_event_budgets_are_enforced() {
    let normal = delta(json!({}), Value::Null);
    for (frames, expected) in [
        (vec![normal; MAX_FRAMES + 1], "frame limit"),
        (
            vec![json!({"choices":[],"padding":"x".repeat(MAX_AGENT_FRAME_BYTES+1)})],
            "frame limit",
        ),
        // Many tiny incomplete chunks of a comment cannot allocate forever.
        (
            vec![json!({"choices":[],"padding":"x".repeat(200*1024)}); 11],
            "wire limit",
        ),
    ] {
        let (base_url, server) = crate::tests::sequence_server(vec![wire(&frames, true)]).await;
        let (sender, _receiver) = mpsc::channel(8);
        let error = provider(base_url)
            .stream_tool_turn(&[], &[], sender, CancellationToken::new())
            .await
            .err()
            .unwrap();
        assert!(error.contains(expected), "{error}");
        server.await.unwrap();
    }
}
