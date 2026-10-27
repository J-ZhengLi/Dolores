use crate::context_recovery::*;
use crate::*;
use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
};
use tokio::sync::mpsc;
use tokio_util::sync::CancellationToken;
fn message(role: &str, text: &str) -> AgentMessage {
    AgentMessage {
        role: role.into(),
        content: text.into(),
        parts: vec![],
        calls: vec![],
        call_id: None,
    }
}
#[test]
fn large_model_window_does_not_compact_at_a_small_local_byte_threshold() {
    let messages = vec![
        message("system", "Host instructions"),
        message("user", "Original objective"),
        message("assistant", &"x".repeat(110000)),
    ];
    let capacity = context_capacity(
        &messages,
        &[],
        Some(500000),
        RequestSettings::default(),
        true,
    )
    .unwrap();
    assert!(
        capacity.pressure_percent < 10,
        "Premature compaction at {}%: {} estimated tokens in a 500000-token window",
        capacity.pressure_percent,
        capacity.estimated_tokens
    );
}

#[test]
fn capacity_uses_model_allowance_and_recovery_preserves_literals_without_calls() {
    let mut m = vec![
        message("system", "Host goal/constraints"),
        message("user", "Original objective 世界"),
        message("assistant", &"x".repeat(80000)),
        message("user", "Keep this correction"),
    ];
    let c = context_capacity(&m, &[], Some(1000000), RequestSettings::default(), true).unwrap();
    assert!(c.pressure_percent < 10);
    assert!(capacity_notice(&c).contains("Do not skip validation"));
    let checkpoint=ContextCheckpoint::parse(r#"{"decisions":["Preserve files"],"progress":["Created artifact"],"remaining":["Verify content"]}"#).unwrap();
    m[2].calls.push(ToolCall {
        id: "old-effect".into(),
        name: "create_text_file".into(),
        arguments: "{}".into(),
    });
    let restored = restored_context(&m, &checkpoint, &[], "Host goal/constraints").unwrap();
    assert!(restored[1].content.contains("Original objective 世界"));
    assert!(restored[1].content.contains("Keep this correction"));
    assert!(restored
        .iter()
        .all(|m| m.calls.is_empty() && m.call_id.is_none()));
    assert!(ContextCheckpoint::parse("```json\n{}\n```").is_err());
    assert!(ContextCheckpoint::parse(
        r#"{"decisions":[],"progress":[],"remaining":[],"approve":true}"#
    )
    .is_err());
}
#[test]
fn complete_checkpoint_json_does_not_fail_on_soft_summary_shape_limits() {
    let source = serde_json::json!({"decisions":["Public decision. ".repeat(90)], "progress":(0..34).map(|n| format!("Completed step {n}")).collect::<Vec<_>>(), "remaining":[]});
    let checkpoint = ContextCheckpoint::parse(&source.to_string()).unwrap();
    assert!(
        !checkpoint.remaining.is_empty(),
        "An empty model list must become a host verification reminder, not a completion claim"
    );
    assert!(checkpoint.decisions[0].contains("Public decision"));
    assert_eq!(checkpoint.progress.len(), 34);
    assert!(ContextCheckpoint::parse(
        r#"{"decisions":[],"progress":[],"remaining":[],"approve":true}"#
    )
    .is_err());
}
#[test]
fn model_summaries_are_format_agnostic_but_not_unbounded_or_empty() {
    let prose = "## Progress\nCreated the parser 世界.\n## Remaining\nRun verification.";
    let c = ContextCheckpoint::from_model_summary(prose).unwrap();
    assert_eq!(c.progress, [prose]);
    assert!(c.remaining[0].contains("Verify"));
    let json = r#"{"decisions":["Keep literal constraints"],"progress":["Read source"],"remaining":["Verify"]}"#;
    let c = ContextCheckpoint::from_model_summary(&format!("```json\n{json}\n```\n")).unwrap();
    assert_eq!(c.decisions, ["Keep literal constraints"]);
    let arbitrary = r#"{"approve":true,"progress":"Historical claim"}"#;
    assert_eq!(
        ContextCheckpoint::from_model_summary(arbitrary)
            .unwrap()
            .progress,
        [arbitrary]
    );
    for bad in [
        String::new(),
        " \n".into(),
        "x".repeat(8193),
        "text\0text".into(),
    ] {
        assert!(ContextCheckpoint::from_model_summary(&bad).is_err());
    }
}
#[test]
fn restoration_keeps_user_images_and_labels_the_latest_tool_observation() {
    let image = |name: &str| AttachmentRef {
        digest: name.repeat(64),
        name: format!("{name}.png"),
        mime: "image/png".into(),
        bytes: 3,
    };
    let mut original = message("user", "Exact objective 世界");
    original.parts.push(image("a"));
    let receipt: ToolRecord=serde_json::from_value(serde_json::json!({"callId":"screen","name":"computer","target":"fixture","status":"completed","content":"Observed","parts":[image("b")]})).unwrap();
    let checkpoint = ContextCheckpoint {
        decisions: vec![],
        progress: vec![],
        remaining: vec!["Verify".into()],
    };
    let restored = restored_context(
        &[original.clone()],
        &checkpoint,
        &[receipt],
        "Keep constraints",
    )
    .unwrap();
    assert_eq!(restored[1].parts, original.parts);
    assert_eq!(restored[3].parts, vec![image("b")]);
    assert!(restored[3].content.contains("historical"));
    assert!(restored[1].content.contains("Exact objective 世界"));
}
struct Ledger(AtomicUsize);
impl BudgetLedger for Ledger {
    fn reserve(&self, models: usize, _: usize) -> Result<bool, String> {
        Ok(self.0.fetch_add(models, Ordering::SeqCst) < 2)
    }
}
struct Fixture(AtomicUsize);
#[async_trait::async_trait]
impl ModelProvider for Fixture {
    fn context_window_tokens(&self) -> Option<u32> {
        Some(45000)
    }
    fn model_retry_delay(&self, error: &str) -> Option<std::time::Duration> {
        (error == "Connection ended before the response finished.")
            .then_some(std::time::Duration::from_millis(1))
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
        self.0.fetch_add(1, Ordering::SeqCst);
        assert!(m.iter().any(|m| m.content.contains("Original objective")));
        assert!(!m.iter().any(|m| m.content.contains("OLD_BULK")));
        assert!(m[0].content.contains("Preserve constraints"));
        Ok(AgentTurn {
            content: "Verified continuation".into(),
            calls: vec![],
            usage: None,
            output_limit: false,
        })
    }
}
struct Approval {
    calls: AtomicUsize,
    malformed: bool,
}
#[async_trait::async_trait]
impl ToolApproval for Approval {
    fn context_recovery_available(&self) -> bool {
        true
    }
    async fn compact_context(
        &self,
        _: &[AgentMessage],
        _: CancellationToken,
    ) -> Result<Option<ContextCheckpoint>, String> {
        self.calls.fetch_add(1, Ordering::SeqCst);
        Ok(Some(ContextCheckpoint::parse(if self.malformed {
            "broken"
        } else {
            r#"{"decisions":["Preserve constraints"],"progress":["Retain receipts"],"remaining":["Verify"]}"#
        })?))
    }
    async fn authorize(&self, _: &ToolRequest, _: CancellationToken) -> Result<bool, String> {
        unreachable!()
    }
}
struct UnusedTool;

#[tokio::test]
async fn compaction_retries_only_generation_and_charges_every_attempt() {
    struct Transient {
        calls: AtomicUsize,
        failures: usize,
        stop: bool,
    }
    #[async_trait::async_trait]
    impl ToolApproval for Transient {
        fn context_recovery_available(&self) -> bool {
            true
        }
        async fn compact_context(
            &self,
            _: &[AgentMessage],
            cancel: CancellationToken,
        ) -> Result<Option<ContextCheckpoint>, String> {
            let n = self.calls.fetch_add(1, Ordering::SeqCst);
            if self.stop {
                cancel.cancel();
            }
            if n < self.failures {
                return Err("Connection ended before the response finished.".into());
            }
            Ok(Some(ContextCheckpoint {
                decisions: vec![],
                progress: vec!["Retain all receipts".into()],
                remaining: vec!["Verify".into()],
            }))
        }
        async fn authorize(&self, _: &ToolRequest, _: CancellationToken) -> Result<bool, String> {
            panic!("Compaction must not dispatch a tool")
        }
    }
    for (failures, cap, stop, expected, completes) in [
        (1, None, false, 2, true),
        (8, None, false, 3, false),
        (8, Some(2), false, 2, false),
        (8, None, true, 1, false),
    ] {
        let host = Transient {
            calls: AtomicUsize::new(0),
            failures,
            stop,
        };
        let provider = Fixture(AtomicUsize::new(0));
        let mut context = preview_context(vec![], "Original objective").unwrap();
        context[0].content.push_str("Preserve constraints");
        context.insert(
            1,
            Message {
                role: Role::User,
                content: "Historical input".into(),
                parts: vec![],
            },
        );
        context.insert(
            2,
            Message {
                role: Role::Assistant,
                content: "OLD_BULK".repeat(14000),
                parts: vec![],
            },
        );
        let budget = TaskBudget {
            model_calls: cap,
            ..Default::default()
        };
        let shared = Arc::new(SharedTaskBudget::new(budget));
        let (events, _rx) = mpsc::channel(128);
        let result = run_agent_with_shared_budget(
            &provider,
            context,
            &[Arc::new(UnusedTool)],
            &host,
            events,
            CancellationToken::new(),
            budget,
            shared.clone(),
            false,
        )
        .await;
        assert_eq!(host.calls.load(Ordering::SeqCst), expected);
        assert_eq!(provider.0.load(Ordering::SeqCst), usize::from(completes));
        assert_eq!(
            shared.usage().model_calls,
            expected + usize::from(completes)
        );
        if stop {
            assert!(result.err().unwrap().contains("stopped"));
        } else {
            let reply = result.unwrap();
            assert_eq!(reply.summary.model_calls, expected + usize::from(completes));
            assert_eq!(reply.pause.is_none(), completes);
        }
    }
}

#[async_trait::async_trait]
impl ToolPlugin for UnusedTool {
    fn spec(&self) -> ToolSpec {
        ToolSpec {
            name: "unused".into(),
            description: "Fixture".into(),
            parameters: serde_json::json!({"type":"object"}),
        }
    }
    fn prepare(&self, _: &ToolCall) -> Result<ToolRequest, String> {
        unreachable!()
    }
    async fn invoke(&self, _: &ToolRequest, _: CancellationToken) -> Result<String, String> {
        unreachable!()
    }
}

#[tokio::test]
async fn long_task_compacts_more_than_twice_and_retains_every_receipt() {
    struct LongTask(AtomicUsize);
    #[async_trait::async_trait]
    impl ModelProvider for LongTask {
        fn context_window_tokens(&self) -> Option<u32> {
            Some(40000)
        }
        fn descriptor(&self) -> PluginDescriptor {
            PluginDescriptor {
                id: "long-task",
                kind: "provider",
                api_version: 1,
            }
        }
        fn continuation_pressure_percent(&self) -> u64 {
            if self.0.load(Ordering::SeqCst) == 1 {
                100
            } else {
                0
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
            let n = self.0.fetch_add(1, Ordering::SeqCst);
            if n == 1 {
                assert!(
                    messages
                        .iter()
                        .any(|m| m.content.contains("Host context checkpoint")),
                    "Private protocol pressure should compact before public messages fill up"
                );
            }
            assert!(messages
                .iter()
                .any(|m| m.content.contains("Original objective")));
            Ok(AgentTurn {
                content: if n == 28 {
                    "Task complete".into()
                } else {
                    String::new()
                },
                calls: if n == 28 {
                    vec![]
                } else {
                    vec![ToolCall {
                        id: format!("read-{n}"),
                        name: "read_text_file".into(),
                        arguments: format!("{{\"path\":\"file-{n}\"}}"),
                    }]
                },
                usage: None,
                output_limit: false,
            })
        }
    }
    struct LargeTool(AtomicUsize);
    #[async_trait::async_trait]
    impl ToolPlugin for LargeTool {
        fn spec(&self) -> ToolSpec {
            ToolSpec {
                name: "read_text_file".into(),
                description: "Read fixture".into(),
                parameters: serde_json::json!({}),
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
        async fn invoke(&self, _: &ToolRequest, _: CancellationToken) -> Result<String, String> {
            self.0.fetch_add(1, Ordering::SeqCst);
            Ok("Public fixture evidence. ".repeat(650))
        }
    }
    struct Compact(AtomicUsize);
    #[async_trait::async_trait]
    impl ToolApproval for Compact {
        fn context_recovery_available(&self) -> bool {
            true
        }
        async fn compact_context(
            &self,
            _: &[AgentMessage],
            _: CancellationToken,
        ) -> Result<Option<ContextCheckpoint>, String> {
            self.0.fetch_add(1, Ordering::SeqCst);
            Ok(Some(ContextCheckpoint {
                decisions: vec!["Keep original task".into()],
                progress: vec!["Read files".into()],
                remaining: vec!["Finish checking".into()],
            }))
        }
        async fn authorize(&self, _: &ToolRequest, _: CancellationToken) -> Result<bool, String> {
            Ok(true)
        }
    }
    let tool = Arc::new(LargeTool(AtomicUsize::new(0)));
    let approval = Compact(AtomicUsize::new(0));
    let provider = LongTask(AtomicUsize::new(0));
    let (tx, _rx) = mpsc::channel(1024);
    let reply = run_agent(
        &provider,
        preview_context(vec![], "Original objective").unwrap(),
        &[tool.clone() as Arc<dyn ToolPlugin>],
        &approval,
        tx,
        CancellationToken::new(),
    )
    .await
    .unwrap();
    assert_eq!(
        reply.pause, None,
        "Long tasks must compact again, not stop: {}",
        reply.answer
    );
    assert!(approval.0.load(Ordering::SeqCst) > 2);
    assert_eq!(
        reply.summary.model_calls,
        29 + approval.0.load(Ordering::SeqCst)
    );
    assert_eq!(tool.0.load(Ordering::SeqCst), 28);
    assert_eq!(reply.summary.tools.len(), 28);
    assert!(reply
        .summary
        .tools
        .iter()
        .all(|t| t.content == "Public fixture evidence. ".repeat(650)));
}
#[tokio::test]
async fn compaction_output_failure_saves_a_checkpoint_instead_of_failing_the_task() {
    struct Capped;
    #[async_trait::async_trait]
    impl ToolApproval for Capped {
        fn context_recovery_available(&self) -> bool {
            true
        }
        async fn compact_context(
            &self,
            _: &[AgentMessage],
            _: CancellationToken,
        ) -> Result<Option<ContextCheckpoint>, String> {
            Err("Response reached the model output limit. Try a shorter request.".into())
        }
        async fn authorize(&self, _: &ToolRequest, _: CancellationToken) -> Result<bool, String> {
            unreachable!()
        }
    }
    let mut context = preview_context(vec![], "Original objective").unwrap();
    context.insert(
        1,
        Message {
            role: Role::User,
            content: "Historical input".into(),
            parts: vec![],
        },
    );
    context.insert(
        2,
        Message {
            role: Role::Assistant,
            content: "OLD_BULK".repeat(14000),
            parts: vec![],
        },
    );
    let fixture = Fixture(AtomicUsize::new(0));
    let (tx, _rx) = mpsc::channel(64);
    let reply = run_agent_with_shared_budget(
        &fixture,
        context,
        &[Arc::new(UnusedTool)],
        &Capped,
        tx,
        CancellationToken::new(),
        TaskBudget::default(),
        Arc::new(SharedTaskBudget::new(TaskBudget::default())),
        false,
    )
    .await;
    assert!(
        reply.is_ok(),
        "A failed internal summary must not discard task progress: {}",
        reply.as_ref().err().cloned().unwrap_or_default()
    );
    let reply = reply.unwrap();
    assert_eq!(reply.pause, Some(PauseReason::Checkpoint));
    assert!(reply.answer.contains("compaction"));
    assert!(reply.answer.contains("saved"));
    assert_eq!(fixture.0.load(Ordering::SeqCst), 0);
}
#[tokio::test]
async fn recovery_reserves_attempts_keeps_goal_and_refuses_malformed_summary_before_provider() {
    for malformed in [false, true] {
        let approval = Approval {
            calls: AtomicUsize::new(0),
            malformed,
        };
        let fixture = Fixture(AtomicUsize::new(0));
        let ledger = Arc::new(Ledger(AtomicUsize::new(0)));
        let mut context = preview_context(vec![], "Original objective").unwrap();
        context[0].content.push_str("Preserve constraints");
        // Whole historical turns fit initially but cross the early byte threshold.
        context.insert(
            1,
            Message {
                role: Role::User,
                content: "Historical input".into(),
                parts: vec![],
            },
        );
        context.insert(
            2,
            Message {
                role: Role::Assistant,
                content: "OLD_BULK".repeat(14000),
                parts: vec![],
            },
        );
        let (tx, _rx) = mpsc::channel(64);
        let result = run_agent_with_shared_budget(
            &fixture,
            context,
            &[Arc::new(UnusedTool)],
            &approval,
            tx,
            CancellationToken::new(),
            TaskBudget::default(),
            Arc::new(SharedTaskBudget::with_ledger(
                TaskBudget::default(),
                ledger.clone(),
            )),
            false,
        )
        .await;
        assert_eq!(
            approval.calls.load(Ordering::SeqCst),
            1,
            "{}",
            result.as_ref().err().cloned().unwrap_or_default()
        );
        assert_eq!(fixture.0.load(Ordering::SeqCst), usize::from(!malformed));
        assert_eq!(
            ledger.0.load(Ordering::SeqCst),
            if malformed { 1 } else { 2 }
        );
        if malformed {
            let reply = result.unwrap();
            assert_eq!(reply.pause, Some(PauseReason::Checkpoint));
            assert!(reply.answer.contains("malformed"));
        } else {
            assert_eq!(result.unwrap().answer, "Verified continuation");
        }
    }
}
