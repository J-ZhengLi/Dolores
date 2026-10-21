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
fn capacity_uses_tighter_bound_and_recovery_preserves_literals_without_calls() {
    let mut m = vec![
        message("system", "Host goal/constraints"),
        message("user", "Original objective 世界"),
        message("assistant", &"x".repeat(80000)),
        message("user", "Keep this correction"),
    ];
    let c = context_capacity(&m, &[], Some(1000000), RequestSettings::default(), true).unwrap();
    assert!(c.pressure_percent >= 60);
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
struct Ledger(AtomicUsize);
impl BudgetLedger for Ledger {
    fn reserve(&self, models: usize, _: usize) -> Result<bool, String> {
        Ok(self.0.fetch_add(models, Ordering::SeqCst) < 2)
    }
}
struct Fixture(AtomicUsize);
#[async_trait::async_trait]
impl ModelProvider for Fixture {
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
            assert!(result.err().unwrap().contains("malformed"));
        } else {
            assert_eq!(result.unwrap().answer, "Verified continuation");
        }
    }
}
