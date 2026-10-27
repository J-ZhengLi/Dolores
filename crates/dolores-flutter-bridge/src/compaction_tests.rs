use super::*;
use dolores_core::{Message, PluginDescriptor};
#[test]
fn in_run_summary_failure_saves_completed_tool_work_instead_of_a_send_error() {
    use dolores_core::{
        AgentMessage, AgentTurn, ToolApproval, ToolCall, ToolPlugin, ToolRequest, ToolSpec,
    };
    use std::sync::atomic::{AtomicUsize, Ordering};
    struct Worker(AtomicUsize);
    #[async_trait::async_trait]
    impl ModelProvider for Worker {
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
            _: &[AgentMessage],
            _: &[ToolSpec],
            _: CancellationToken,
        ) -> Result<AgentTurn, String> {
            let n = self.0.fetch_add(1, Ordering::SeqCst);
            assert!(n < 12, "The fixture must reach context compaction");
            Ok(AgentTurn {
                content: format!("Inspection segment {n}"),
                calls: vec![ToolCall {
                    id: format!("read-{n}"),
                    name: "read_text_file".into(),
                    arguments: "{\"path\":\"evidence.txt\"}".into(),
                }],
                usage: None,
                output_limit: false,
            })
        }
    }
    struct SummaryLimit;
    #[async_trait::async_trait]
    impl ModelProvider for SummaryLimit {
        fn descriptor(&self) -> PluginDescriptor {
            PluginDescriptor {
                id: "summary-fixture",
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
            Err(dolores_provider_openai::OUTPUT_LIMIT_ERROR.into())
        }
    }
    struct Evidence(AtomicUsize);
    #[async_trait::async_trait]
    impl ToolPlugin for Evidence {
        fn spec(&self) -> ToolSpec {
            ToolSpec {
                name: "read_text_file".into(),
                description: "Read public fixture evidence".into(),
                parameters: json!({"type":"object"}),
            }
        }
        fn prepare(&self, call: &ToolCall) -> Result<ToolRequest, String> {
            Ok(ToolRequest {
                call_id: call.id.clone(),
                name: call.name.clone(),
                target: "evidence.txt".into(),
                query: None,
                diff: None,
                command: None,
                mcp: None,
            })
        }
        async fn invoke(&self, _: &ToolRequest, _: CancellationToken) -> Result<String, String> {
            self.0.fetch_add(1, Ordering::SeqCst);
            Ok("public evidence ".repeat(1000))
        }
    }
    struct Allow;
    #[async_trait::async_trait]
    impl ToolApproval for Allow {
        async fn authorize(&self, _: &ToolRequest, _: CancellationToken) -> Result<bool, String> {
            Ok(true)
        }
    }
    let store = Arc::new(SqliteStore::open(std::path::Path::new(":memory:")).unwrap());
    store.create("chat").unwrap();
    let run = dolores_core::RunSnapshot {
        id: "11111111-1111-4111-8111-111111111111".into(),
        thread: "chat".into(),
        model: "fixture".into(),
        input: "Inspect evidence".into(),
        settings: Default::default(),
        state: dolores_core::RunState::Prepared,
        sequence: 0,
        created_at: 0,
        build: "fixture".into(),
        tools: vec![],
        extensions: vec![],
        effective_settings: None,
        parent_run: None,
        segments: 1,
    };
    store.begin_run(&run).unwrap();
    let log = run_journal::RunLog::new(store.clone(), run.id.clone());
    let evidence = Arc::new(Evidence(AtomicUsize::new(0)));
    let approval = Arc::new(goals::RecoveryApproval {
        inner: Arc::new(Allow),
        provider: Some(Arc::new(SummaryLimit)),
        log: log.clone(),
    });
    let runtime = Runtime::new().unwrap();
    let (output, _events) = mpsc::channel(256);
    let result = runtime.block_on(execute(
        store.clone(),
        Arc::new(Worker(AtomicUsize::new(0))),
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
            input: "Inspect evidence".into(),
            model: "fixture".into(),
            settings: None,
            tools: vec![evidence.clone()],
            approval: Some(approval),
        },
        CancellationToken::new(),
        &output,
    ));
    assert!(
        result.is_ok(),
        "{}",
        result.as_ref().err().cloned().unwrap_or_default()
    );
    let messages = store.messages_page("chat", None, false, 2).unwrap().items;
    let metadata = messages.last().unwrap().metadata.as_ref().unwrap();
    assert_eq!(
        metadata.paused.as_ref().unwrap().reason,
        dolores_core::PauseReason::Checkpoint
    );
    let count = evidence.0.load(Ordering::SeqCst);
    assert!(count > 0);
    assert_eq!(metadata.agent.as_ref().unwrap().tools.len(), count);
    assert_eq!(metadata.paused.as_ref().unwrap().receipts.len(), count);
    assert!(store
        .run_events("chat", &run.id)
        .unwrap()
        .iter()
        .any(|e| e.kind == "contextRecoveryFailed"));
    struct Finish;
    #[async_trait::async_trait]
    impl ModelProvider for Finish {
        fn descriptor(&self) -> PluginDescriptor {
            PluginDescriptor {
                id: "continue-fixture",
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
            messages: &[AgentMessage],
            _: &[ToolSpec],
            _: CancellationToken,
        ) -> Result<AgentTurn, String> {
            assert!(messages.iter().any(|m| m.content.contains("previews")));
            assert!(!messages
                .iter()
                .any(|m| m.content.contains(&"public evidence ".repeat(1000))));
            Ok(AgentTurn {
                content: "Continued from retained evidence without repeating reads.".into(),
                calls: vec![],
                usage: None,
                output_limit: false,
            })
        }
    }
    let continued = runtime.block_on(execute(
        store.clone(),
        Arc::new(Finish),
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
            log: None,
            compaction_provider: None,
            compacted: false,
            resume_run: None,
            permissions: Default::default(),
            task: Default::default(),
            interaction: Default::default(),
            continuation: Some(messages.last().unwrap().id),
            id: 2,
            session: Some("chat".into()),
            input: continuation::INPUT.into(),
            model: "fixture".into(),
            settings: None,
            tools: vec![evidence.clone()],
            approval: Some(Arc::new(Allow)),
        },
        CancellationToken::new(),
        &output,
    ));
    assert!(
        continued.is_ok(),
        "{}",
        continued.as_ref().err().cloned().unwrap_or_default()
    );
    assert_eq!(
        evidence.0.load(Ordering::SeqCst),
        count,
        "Continue must not replay completed reads"
    );
    let saved = store.messages_page("chat", None, false, 50).unwrap().items;
    let original = saved
        .iter()
        .find(|m| m.id == messages.last().unwrap().id)
        .unwrap();
    assert_eq!(original.metadata.as_ref().unwrap().paused, metadata.paused);
}
struct Summarizer {
    calls: std::sync::atomic::AtomicU32,
    invalid: bool,
}
#[async_trait::async_trait]
impl ModelProvider for Summarizer {
    fn descriptor(&self) -> PluginDescriptor {
        PluginDescriptor {
            id: "fixture",
            kind: "provider",
            api_version: 1,
        }
    }
    fn request_settings(&self) -> Option<RequestSettings> {
        Some(RequestSettings {
            max_output_tokens: Some(128),
            timeout_seconds: 30,
            ..Default::default()
        })
    }
    fn context_window_tokens(&self) -> Option<u32> {
        Some(4096)
    }
    async fn stream(
        &self,
        messages: Vec<Message>,
        output: mpsc::Sender<String>,
        _: CancellationToken,
    ) -> Result<(), String> {
        self.calls.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        let summary = messages[0]
            .content
            .starts_with("You draft a session summary");
        let text = if summary && self.invalid {
            "x".repeat(8193)
        } else if summary {
            "Decisions: retain Unicode. Unresolved: finish tests.".into()
        } else {
            assert!(messages[0].content.contains("Original goal: Build parser"));
            "Task reply".into()
        };
        output.send(text).await.map_err(|_| "Closed".into())
    }
}
#[test]
fn compaction_is_automatic_once_charged_and_failure_preserves_summary_history_and_draft() {
    for invalid in [false, true] {
        let store = Arc::new(SqliteStore::open(std::path::Path::new(":memory:")).unwrap());
        store.create("chat").unwrap();
        for n in 0..30 {
            store
                .commit_turn(
                    "chat",
                    &format!("Build parser goal {n}. {}", "u".repeat(380)),
                    &"a".repeat(400),
                )
                .unwrap();
        }
        store.save_draft("chat", "continue").unwrap();
        assert!(store.auto_compact("chat").unwrap());
        let provider = Arc::new(Summarizer {
            calls: std::sync::atomic::AtomicU32::new(0),
            invalid,
        });
        let runtime = Runtime::new().unwrap();
        let (output, mut events) = mpsc::channel(32);
        let result = runtime.block_on(execute(
            store.clone(),
            provider.clone(),
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
                log: None,
                compaction_provider: Some(provider.clone()),
                compacted: false,
                resume_run: None,
                permissions: Default::default(),
                task: Default::default(),
                interaction: Default::default(),
                continuation: None,
                id: 1,
                session: Some("chat".into()),
                input: "continue".into(),
                model: "fixture".into(),
                settings: provider.request_settings(),
                tools: vec![],
                approval: None,
            },
            CancellationToken::new(),
            &output,
        ));
        if invalid {
            assert!(result.unwrap_err().contains("Last valid summary"));
            assert!(store.session_summary("chat").unwrap().is_none());
            assert_eq!(store.context_history("chat").unwrap().1, Some(30));
            assert_eq!(store.saved_draft("chat").unwrap(), "continue");
            assert_eq!(provider.calls.load(std::sync::atomic::Ordering::SeqCst), 1);
        } else {
            assert!(result.is_ok(), "{result:?}");
            assert_eq!(provider.calls.load(std::sync::atomic::Ordering::SeqCst), 2);
            let saved = store.session_summary("chat").unwrap().unwrap();
            assert!(saved.text.starts_with("Original goal: Build parser"));
            assert!(saved.provenance.covered_turns > 0);
            assert_eq!(store.context_history("chat").unwrap().1, Some(31));
            let mut remaining = None;
            while let Ok(e) = events.try_recv() {
                if e["type"] == "started" {
                    remaining = e["taskBudget"]["modelCalls"].as_u64();
                }
            }
            assert_eq!(remaining, Some(63));
        }
    }
}
