use super::*;
use dolores_core::{ConnectionPreferences, ModelProvider, RequestSettings};

#[tokio::test]
async fn long_reasoning_is_preserved_for_followup_without_entering_public_text() {
    let reasoning = "thinking ".repeat(22000);
    let frames: Vec<_> = reasoning
        .as_bytes()
        .chunks(4096)
        .map(|part| {
            delta(
                json!({"reasoning_content":std::str::from_utf8(part).unwrap()}),
                Value::Null,
            )
        })
        .chain([delta(
            json!({"tool_calls":[call(0,"one","read_text_file","{\"path\":\"a.txt\"}")]}),
            json!("tool_calls"),
        )])
        .collect();
    let (base_url, server) = crate::tests::sequence_server(vec![
        wire(&frames, true),
        wire(&[delta(json!({"content":"done"}), json!("stop"))], true),
    ])
    .await;
    let provider = provider(base_url);
    let initial = [AgentMessage {
        role: "user".into(),
        content: "Read".into(),
        parts: vec![],
        calls: vec![],
        call_id: None,
    }];
    let (tx, mut rx) = mpsc::channel(8);
    let turn = provider
        .stream_tool_turn(&initial, &[], tx, CancellationToken::new())
        .await
        .unwrap();
    assert!(rx.recv().await.is_none());
    let followup = [
        initial[0].clone(),
        AgentMessage {
            role: "assistant".into(),
            content: turn.content,
            parts: vec![],
            calls: turn.calls,
            call_id: None,
        },
        AgentMessage {
            role: "tool".into(),
            content: "text".into(),
            parts: vec![],
            calls: vec![],
            call_id: Some("one".into()),
        },
    ];
    let (tx, _rx) = mpsc::channel(8);
    provider
        .stream_tool_turn(&followup, &[], tx, CancellationToken::new())
        .await
        .unwrap();
    let requests = server.await.unwrap();
    let bodies: Vec<Value> = requests
        .lines()
        .filter(|l| l.starts_with('{'))
        .map(|l| serde_json::from_str(l).unwrap())
        .collect();
    assert_eq!(bodies[1]["messages"][1]["reasoning_content"], reasoning);
    assert!(bodies[1].get("max_tokens").is_none());
}

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
async fn active_reasoning_stream_outlives_the_inactivity_allowance() {
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let endpoint = format!("http://{}/v1", listener.local_addr().unwrap());
    let server = tokio::spawn(async move {
        let (mut socket, _) = listener.accept().await.unwrap();
        let mut request = Vec::new();
        loop {
            let mut buf = [0; 4096];
            let n = socket.read(&mut buf).await.unwrap();
            request.extend_from_slice(&buf[..n]);
            if let Some(i) = request.windows(4).position(|w| w == b"\r\n\r\n") {
                let length = String::from_utf8_lossy(&request[..i])
                    .lines()
                    .find_map(|l| {
                        l.to_lowercase()
                            .strip_prefix("content-length: ")
                            .map(|v| v.parse::<usize>().unwrap())
                    })
                    .unwrap_or(0);
                if request.len() >= i + 4 + length {
                    break;
                }
            }
        }
        socket
            .write_all(
                b"HTTP/1.1 200 OK\r\nContent-Type: text/event-stream\r\nConnection: close\r\n\r\n",
            )
            .await
            .unwrap();
        for frame in [
            delta(json!({"reasoning_content":"Plan the scene."}), Value::Null),
            delta(
                json!({"reasoning_content":" Check keyboard input."}),
                Value::Null,
            ),
            delta(json!({"content":"Done"}), json!("stop")),
        ] {
            if socket
                .write_all(format!("data: {frame}\n\n").as_bytes())
                .await
                .is_err()
            {
                return;
            }
            tokio::time::sleep(Duration::from_millis(600)).await;
        }
        let _ = socket.write_all(b"data: [DONE]\n\n").await;
    });
    let model = OpenAiProvider::with_settings(
        &ConnectionPreferences {
            base_url: endpoint,
            model: "fixture".into(),
        },
        String::new(),
        RequestSettings {
            timeout_seconds: 1,
            ..Default::default()
        },
    )
    .unwrap();
    let (text, mut texts) = mpsc::channel(8);
    let (activity, _activities) = mpsc::channel(8);
    let (thinking, mut thoughts) = mpsc::channel(8);
    let result = model
        .stream_tool_turn_with_thinking(
            &[],
            &[],
            text,
            activity,
            thinking,
            CancellationToken::new(),
        )
        .await;
    server.await.unwrap();
    assert_eq!(
        result.unwrap().content,
        "Done",
        "Progress must reset inactivity, rather than spend an absolute response deadline"
    );
    assert_eq!(texts.recv().await.as_deref(), Some("Done"));
    let mut preview = String::new();
    while let Some(text) = thoughts.recv().await {
        preview = text;
    }
    assert_eq!(preview, "Plan the scene. Check keyboard input.");
}

#[tokio::test]
async fn tool_reasoning_is_returned_exactly_but_never_emitted_and_new_runs_clear_it() {
    let secret = "PRIVATE_REASONING_SENTINEL 世界";
    let response = wire(
        &[
            delta(json!({"reasoning_content":secret}), Value::Null),
            delta(
                json!({"content":"Inspect file","tool_calls":[call(0,"read-1","read_text_file",r#"{"path":"game.html"}"#)]}),
                json!("tool_calls"),
            ),
        ],
        true,
    );
    let (url, _) = crate::tests::sequence_server(vec![response]).await;
    let provider = provider(url);
    let input = vec![dolores_core::AgentMessage {
        role: "user".into(),
        content: "Inspect".into(),
        parts: vec![],
        calls: vec![],
        call_id: None,
    }];
    let (tx, mut rx) = mpsc::channel(8);
    let turn = provider
        .stream_tool_turn(&input, &[], tx, CancellationToken::new())
        .await
        .unwrap();
    assert_eq!(rx.recv().await.unwrap(), "Inspect file");
    assert!(rx.recv().await.is_none());
    let message = dolores_core::AgentMessage {
        role: "assistant".into(),
        content: turn.content.clone(),
        parts: vec![],
        calls: turn.calls.clone(),
        call_id: None,
    };
    let wire = provider
        .wire_agent_messages(std::slice::from_ref(&message))
        .unwrap();
    assert_eq!(wire[0]["reasoning_content"], secret);
    assert!(!json!({"content":turn.content,"calls":turn.calls})
        .to_string()
        .contains(secret));
    let mut changed = message.clone();
    changed.calls[0].arguments = r#"{"path":"different.html"}"#.into();
    assert!(provider.wire_agent_messages(&[changed]).unwrap()[0]
        .get("reasoning_content")
        .is_none());
    provider.wire_agent_messages(&input).unwrap();
    assert!(provider.wire_agent_messages(&[message]).unwrap()[0]
        .get("reasoning_content")
        .is_none());
}

#[tokio::test]
async fn reasoning_activity_arrives_before_completion_without_private_text_and_stop_is_prompt() {
    use dolores_core::ModelActivity;
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let endpoint = format!("http://{}/v1", listener.local_addr().unwrap());
    let (release, gate) = tokio::sync::oneshot::channel::<()>();
    let server = tokio::spawn(async move {
        let (mut socket, _) = listener.accept().await.unwrap();
        let mut bytes = vec![];
        loop {
            let mut buf = [0; 4096];
            let n = socket.read(&mut buf).await.unwrap();
            bytes.extend_from_slice(&buf[..n]);
            if let Some(i) = bytes.windows(4).position(|p| p == b"\r\n\r\n") {
                let headers = String::from_utf8_lossy(&bytes[..i]);
                let length = headers
                    .lines()
                    .find_map(|s| {
                        s.to_lowercase()
                            .strip_prefix("content-length: ")
                            .map(|n| n.parse::<usize>().unwrap())
                    })
                    .unwrap_or(0);
                if bytes.len() >= i + 4 + length {
                    break;
                }
            }
        }
        socket
            .write_all(
                b"HTTP/1.1 200 OK\r\nContent-Type: text/event-stream\r\nConnection: close\r\n\r\n",
            )
            .await
            .unwrap();
        socket
            .write_all(
                format!(
                    "data: {}\n\n",
                    delta(
                        json!({"reasoning_content":"PRIVATE_REASONING_SENTINEL"}),
                        Value::Null
                    )
                )
                .as_bytes(),
            )
            .await
            .unwrap();
        let _ = gate.await;
    });
    let (text, mut texts) = mpsc::channel(8);
    let (activity, mut activities) = mpsc::channel(8);
    let cancel = CancellationToken::new();
    let stopped = cancel.clone();
    let task = tokio::spawn(async move {
        provider(endpoint)
            .stream_tool_turn_with_activity(&[], &[], text, activity, stopped)
            .await
    });
    assert_eq!(
        tokio::time::timeout(Duration::from_secs(2), activities.recv())
            .await
            .unwrap(),
        Some(ModelActivity::Reasoning)
    );
    assert!(texts.try_recv().is_err());
    assert!(!task.is_finished());
    cancel.cancel();
    assert!(tokio::time::timeout(Duration::from_secs(1), task)
        .await
        .unwrap()
        .unwrap()
        .err()
        .unwrap()
        .contains("stopped"));
    let _ = release.send(());
    server.await.unwrap();
}

#[tokio::test]
async fn bounded_reasoning_and_html_survive_large_fragmented_transport() {
    // Normal one-character deltas repeat provider metadata thousands of times.
    // Their HTTP size must not become a second, smaller output allowance.
    let mut frames = Vec::new();
    for _ in 0..12_000 {
        let mut frame = delta(json!({"reasoning_content":"x"}), Value::Null);
        frame["id"] = json!("chatcmpl-fragmented-provider-response");
        frame["object"] = json!("chat.completion.chunk");
        frame["model"] = json!("reasoning-model-with-small-token-deltas");
        frame["created"] = json!(1_700_000_000);
        frames.push(frame);
    }
    let arguments = json!({"path":"game.html","content":"<!doctype html><canvas></canvas><script>let x=1;</script>"}).to_string();
    frames.push(delta(
        json!({"tool_calls":[call(0,"game","create_text_file",&arguments)]}),
        json!("tool_calls"),
    ));
    let response = wire(&frames, true);
    assert!(response.len() > 2 * 1024 * 1024);
    let (url, server) = crate::tests::sequence_server(vec![response]).await;
    let (tx, _rx) = mpsc::channel(8);
    let turn = provider(url)
        .stream_tool_turn(&[], &[], tx, CancellationToken::new())
        .await
        .expect(
            "valid bounded reasoning and a small complete HTML call must survive metadata overhead",
        );
    assert_eq!(turn.calls[0].arguments, arguments);
    assert!(turn.content.is_empty());
    server.await.unwrap();
}

#[tokio::test]
async fn valid_html_tool_arguments_survive_many_small_stream_events() {
    let html = format!(
        "<!doctype html><html><body><script>{}</script></body></html>",
        "let x=1;\n".repeat(650)
    );
    assert!(html.len() < dolores_core::MAX_TOOL_BYTES);
    let arguments = json!({"path":"game.html","content":html}).to_string();
    let mut frames = vec![delta(
        json!({"tool_calls":[call(0,"html-file","create_text_file","")]}),
        Value::Null,
    )];
    for character in arguments.chars() {
        frames.push(delta(
            json!({"tool_calls":[{"index":0,"function":{"arguments":character.to_string()}}]}),
            Value::Null,
        ));
    }
    frames.push(delta(json!({}), json!("tool_calls")));
    frames.push(json!({"choices":[],"usage":{"completion_tokens":6000}}));
    assert!(frames.len() > 4096);
    let response = wire(&frames, true);
    assert!(response.len() < 2 * 1024 * 1024);
    let (base_url, server) = crate::tests::sequence_server(vec![response]).await;
    let (sender, _receiver) = mpsc::channel(8);
    let result = provider(base_url)
        .stream_tool_turn(&[], &[], sender, CancellationToken::new())
        .await;
    let requests = server.await.unwrap();
    assert_eq!(requests.split("\nREQUEST\n").count(), 1);
    let turn = result
        .expect("a bounded HTML file must not fail because its provider uses small SSE events");
    assert_eq!(turn.calls.len(), 1);
    assert_eq!(turn.calls[0].arguments, arguments);
    assert_eq!(turn.usage.unwrap().output_tokens, Some(6000));
}

#[test]
fn large_fragmented_file_json_survives_but_other_tools_and_overflow_remain_bounded() {
    let arguments = json!({"path":"main.js","content":"世界\\\"\r\n".repeat(900)}).to_string();
    assert!(arguments.len() > 4096);
    for name in [
        "create_text_file",
        "edit_text_file",
        "run_command",
        "mcp_tool_test_echo",
    ] {
        let mut assembly = Assembly::default();
        assembly
            .push(delta(
                json!({"tool_calls":[call(0,"one",name,"")]}),
                Value::Null,
            ))
            .unwrap();
        let mut part = String::new();
        for c in arguments.chars() {
            part.push(c);
            if part.len() >= 73 {
                assembly
                    .push(delta(
                        json!({"tool_calls":[{"index":0,"function":{"arguments":part}}]}),
                        Value::Null,
                    ))
                    .unwrap();
                part.clear();
            }
        }
        assembly
            .push(delta(
                json!({"tool_calls":[{"index":0,"function":{"arguments":part}}]}),
                json!("tool_calls"),
            ))
            .unwrap();
        let result = assembly.complete();
        if matches!(name, "create_text_file" | "edit_text_file" | "run_command") {
            assert_eq!(result.unwrap().calls[0].arguments, arguments);
        } else {
            assert!(result.is_err());
        }
    }
    let mut assembly = Assembly::default();
    assert!(assembly.push(delta(json!({"tool_calls":[call(0,"one","create_text_file",&"x".repeat(MAX_FILE_ARGUMENT_BYTES+1))]}),Value::Null)).is_err());
    let mut assembly = Assembly::default();
    assembly
        .push(delta(
            json!({"tool_calls":[call(0,"one","run_command",&"x".repeat(32*1024+1))]}),
            json!("tool_calls"),
        ))
        .unwrap();
    assert!(assembly.complete().is_err());
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
        parts: vec![],
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
        parts: vec![],
        role: "assistant".into(),
        content: turn.content,
        calls: turn.calls,
        call_id: None,
    });
    messages.push(AgentMessage {
        parts: vec![],
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
                max_output_tokens: Some(2048),
                timeout_seconds: 1,
                reasoning: Default::default(),
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
        assert!(error.contains(if stop { "stopped" } else { "delivery stalled" }));
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
async fn idle_stream_and_individual_frame_budgets_are_enforced() {
    for (frames, expected) in [
        (
            vec![json!({"choices":[],"padding":"x".repeat(MAX_AGENT_FRAME_BYTES+1)})],
            "oversized stream event",
        ),
        // Padding cannot evade the no-progress limit, even if DONE arrives
        // in the same network chunk that crosses the allowance.
        (
            vec![json!({"choices":[],"padding":"x".repeat(200*1024)}); 11],
            "without new response data",
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
#[test]
fn long_thinking_preview_keeps_current_progress_and_unicode_within_its_bound() {
    let start = "opening thought";
    assert_eq!(thinking_preview(start), start);
    let reasoning = format!("{start}{}latest plan", "思考".repeat(5000));
    let preview = thinking_preview(&reasoning);
    assert!(preview.len() <= 16 * 1024);
    assert!(preview.starts_with("[Earlier thinking omitted]"));
    assert!(!preview.contains(start));
    assert!(preview.ends_with("latest plan"));
    let next = thinking_preview(&format!("{reasoning} now implementing"));
    assert!(next.ends_with("now implementing"));
    assert_ne!(preview, next);
}
