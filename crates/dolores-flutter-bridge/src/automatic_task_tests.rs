use super::*;
use dolores_core::{
    AgentMessage, AgentTurn, Message, PluginDescriptor, ToolApproval, ToolCall, ToolPlugin,
    ToolRequest, ToolSpec,
};
use std::sync::atomic::{AtomicUsize, Ordering};

struct LongTask {
    calls: AtomicUsize,
    finish: usize,
    window: u32,
}
#[async_trait::async_trait]
impl ModelProvider for LongTask {
    fn context_window_tokens(&self) -> Option<u32> {
        Some(self.window)
    }
    fn descriptor(&self) -> PluginDescriptor {
        PluginDescriptor {
            id: "fixture",
            kind: "provider",
            api_version: 1,
        }
    }
    async fn stream(
        &self,
        _: Vec<Message>,
        _: mpsc::Sender<String>,
        _: CancellationToken,
    ) -> Result<(), String> {
        unreachable!()
    }
    async fn tool_turn(
        &self,
        m: &[AgentMessage],
        _: &[ToolSpec],
        _: CancellationToken,
    ) -> Result<AgentTurn, String> {
        let n = self.calls.fetch_add(1, Ordering::SeqCst);
        assert!(m
            .iter()
            .any(|m| m.content.contains("Original objective 世界")));
        assert!(dolores_core::estimate_agent_tokens(m, &[]).unwrap() < self.window as u64);
        Ok(AgentTurn {
            content: if n >= self.finish {
                "Verified task finished".into()
            } else {
                format!("Reading evidence {n}")
            },
            calls: if n >= self.finish {
                vec![]
            } else {
                vec![ToolCall {
                    id: format!("step-{n}"),
                    name: "read_text_file".into(),
                    arguments: format!("{{\"path\":\"public-{n}.txt\"}}"),
                }]
            },
            usage: None,
            output_limit: false,
        })
    }
}
struct Summary {
    calls: AtomicUsize,
    text: &'static str,
}
#[async_trait::async_trait]
impl ModelProvider for Summary {
    fn descriptor(&self) -> PluginDescriptor {
        PluginDescriptor {
            id: "summary",
            kind: "provider",
            api_version: 1,
        }
    }
    async fn stream(
        &self,
        _: Vec<Message>,
        tx: mpsc::Sender<String>,
        _: CancellationToken,
    ) -> Result<(), String> {
        self.calls.fetch_add(1, Ordering::SeqCst);
        tx.send(self.text.into()).await.map_err(|_| "Closed".into())
    }
}
struct Evidence {
    effects: AtomicUsize,
}
#[async_trait::async_trait]
impl ToolPlugin for Evidence {
    fn spec(&self) -> ToolSpec {
        ToolSpec {
            name: "read_text_file".into(),
            description: "Public evidence".into(),
            parameters: json!({}),
        }
    }
    fn prepare(&self, c: &ToolCall) -> Result<ToolRequest, String> {
        Ok(ToolRequest {
            call_id: c.id.clone(),
            name: c.name.clone(),
            target: c.arguments.clone(),
            query: None,
            diff: None,
            command: None,
            mcp: None,
        })
    }
    async fn invoke(&self, r: &ToolRequest, _: CancellationToken) -> Result<String, String> {
        self.effects.fetch_add(1, Ordering::SeqCst);
        Ok(format!(
            "{} {}",
            r.call_id,
            "Public evidence 世界. ".repeat(500)
        ))
    }
}
struct Allow;
#[async_trait::async_trait]
impl ToolApproval for Allow {
    async fn authorize(&self, _: &ToolRequest, _: CancellationToken) -> Result<bool, String> {
        Ok(true)
    }
}

#[test]
fn one_start_crosses_call_tool_compaction_and_journal_windows_without_continue() {
    run_long_task(
        800,
        r#"{"decisions":["Preserve original instructions"],"progress":["Evidence saved"],"remaining":["Continue reading and verify"]}"#,
    );
}

#[test]
fn complete_model_summary_format_does_not_pause_a_running_task() {
    for text in [
        "## Decisions\nPreserve original instructions.\n## Progress\nEvidence saved.\n## Remaining\nContinue reading and verify.",
        "```json\n{\"decisions\":[\"Preserve original instructions\"],\"progress\":[\"Evidence saved\"],\"remaining\":[\"Continue reading and verify\"]}\n```",
        r#"{"decisions": "Preserve original instructions", "progress": "Evidence saved", "remaining": ["Continue reading and verify"], "notes": "Historical evidence only"}"#,
    ] {
        run_long_task(80, text);
    }
}

fn run_long_task(count: usize, text: &'static str) {
    run_task_in_window(count, text, 40000, true);
}

#[test]
fn evidence_pagination_does_not_summarize_a_roomy_model_window() {
    run_task_in_window(80, "Unused summary fixture", 500000, false);
}

fn run_task_in_window(count: usize, text: &'static str, window: u32, compacts: bool) {
    let store = Arc::new(SqliteStore::open(std::path::Path::new(":memory:")).unwrap());
    store.create("chat").unwrap();
    store.save_draft("chat", "Keep my unrelated draft").unwrap();
    let run = dolores_core::RunSnapshot {
        parent_run: None,
        segments: 1,
        id: "11111111-1111-4111-8111-111111111111".into(),
        thread: "chat".into(),
        model: "fixture".into(),
        settings: Default::default(),
        input: "Original objective 世界".into(),
        state: dolores_core::RunState::Prepared,
        sequence: 0,
        created_at: 0,
        build: "fixture".into(),
        tools: vec![],
        extensions: vec![],
        effective_settings: None,
    };
    store.begin_run(&run).unwrap();
    let log = run_journal::RunLog::new(store.clone(), run.id.clone());
    let tool = Arc::new(Evidence {
        effects: AtomicUsize::new(0),
    });
    let summary = Arc::new(Summary {
        calls: AtomicUsize::new(0),
        text,
    });
    let approval = Arc::new(goals::RecoveryApproval {
        inner: Arc::new(Allow),
        provider: Some(summary.clone()),
        log: log.clone(),
    });
    let logged = Arc::new(run_journal::LoggedTool {
        inner: tool.clone(),
        log: log.clone(),
        policy: None,
    });
    let runtime = Runtime::new().unwrap();
    let maintenance = runtime.block_on(async {
        log.record(Some(dolores_core::RunState::Running), "started", json!({}))
            .await
            .unwrap();
        let (output, mut events) = mpsc::channel::<Value>(32);
        let drain = tokio::spawn(async move {
            let mut maintenance = 0;
            while let Some(e) = events.recv().await {
                if e["type"] == "taskMaintenance" {
                    maintenance += 1;
                }
            }
            maintenance
        });
        execute(
            store.clone(),
            Arc::new(LongTask {
                calls: AtomicUsize::new(0),
                finish: count,
                window,
            }),
            TurnRequest {
                available_skills: None,
                hooks: None,
                skill_resources: None,
                command_sessions: None,
                plan_execution: false,
                planning: Default::default(),
                goal: None,
                selected_skills: vec![],
                delegation: None,
                log: Some(log),
                compaction_provider: None,
                compacted: false,
                resume_run: None,
                permissions: Default::default(),
                task: Default::default(),
                interaction: Default::default(),
                continuation: None,
                id: 1,
                session: Some("chat".into()),
                input: "Original objective 世界".into(),
                model: "fixture".into(),
                settings: None,
                tools: vec![logged],
                approval: Some(approval),
            },
            CancellationToken::new(),
            &output,
        )
        .await
        .unwrap();
        drop(output);
        drain.await.unwrap()
    });
    let messages = store.messages_page("chat", None, false, 2).unwrap().items;
    assert_eq!(messages.len(), 2);
    let metadata = messages.last().unwrap().metadata.as_ref().unwrap();
    assert!(
        metadata.paused.is_none(),
        "A completed tool-free summary must not pause for its formatting: {}",
        messages.last().unwrap().content
    );
    assert_eq!(tool.effects.load(Ordering::SeqCst), count);
    if compacts {
        assert!(summary.calls.load(Ordering::SeqCst) > 2);
        assert!(maintenance > 4);
    } else {
        assert_eq!(summary.calls.load(Ordering::SeqCst), 0);
        assert_eq!(maintenance, 0);
    }
    let agent = metadata.agent.as_ref().unwrap();
    assert_eq!(
        agent.archive.as_ref().unwrap().tool_count + agent.tools.len(),
        count
    );
    assert!(agent.tools.len() < 32);
    assert_eq!(store.runs("chat").unwrap().len(), 1);
    assert_eq!(store.runs("chat").unwrap()[0].segments, 1);
    let mut cursor = 0;
    let mut receipts = std::collections::HashSet::new();
    let mut count = 0;
    loop {
        let page = store.run_events_page("chat", &run.id, cursor).unwrap();
        for e in &page.items {
            count += 1;
            if e.kind == "toolResult" {
                assert!(receipts.insert(e.data["callId"].as_str().unwrap().to_owned()));
                assert!(e.data["content"]
                    .as_str()
                    .unwrap()
                    .contains("Public evidence 世界"));
            }
        }
        if let Some(next) = page.next {
            cursor = next;
        } else {
            break;
        }
    }
    assert_eq!(receipts.len(), tool.effects.load(Ordering::SeqCst));
    if receipts.len() == 800 {
        assert!(count > dolores_core::MAX_RUN_EVENTS);
    }
}
