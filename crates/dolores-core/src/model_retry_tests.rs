use crate::*;
use async_trait::async_trait;
use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
};
use tokio::sync::mpsc;
use tokio_util::sync::CancellationToken;

struct Worker {
    calls: AtomicUsize,
    failures: usize,
    retry: bool,
}
#[async_trait]
impl ModelProvider for Worker {
    fn descriptor(&self) -> PluginDescriptor {
        PluginDescriptor {
            id: "fixture",
            kind: "provider",
            api_version: 1,
        }
    }
    fn model_retry_delay(&self, e: &str) -> Option<std::time::Duration> {
        (self.retry && e == "Transient fixture generation failure")
            .then_some(std::time::Duration::from_millis(1))
    }
    async fn stream(
        &self,
        _: Vec<Message>,
        _: mpsc::Sender<String>,
        _: CancellationToken,
    ) -> Result<(), String> {
        unreachable!()
    }
    async fn stream_tool_turn(
        &self,
        m: &[AgentMessage],
        _: &[ToolSpec],
        out: mpsc::Sender<String>,
        _: CancellationToken,
    ) -> Result<AgentTurn, String> {
        let n = self.calls.fetch_add(1, Ordering::SeqCst);
        assert!(
            !m.iter().any(|v| v.content.contains("Unfinished proposal")),
            "Failed response must not become executable context"
        );
        if n > 0 && n <= self.failures {
            assert_eq!(m.iter().filter(|v| v.role == "tool").count(), 1);
            out.send("Unfinished proposal".into()).await.unwrap();
            return Err("Transient fixture generation failure".into());
        }
        let content = if n == 0 {
            "One action"
        } else {
            "Verified complete"
        };
        out.send(content.into()).await.unwrap();
        Ok(AgentTurn {
            content: content.into(),
            calls: if n == 0 {
                vec![ToolCall {
                    id: "completed-once".into(),
                    name: "fixture_action".into(),
                    arguments: "{}".into(),
                }]
            } else {
                vec![]
            },
            usage: None,
            output_limit: false,
        })
    }
}
struct Effect(AtomicUsize);
#[async_trait]
impl ToolPlugin for Effect {
    fn spec(&self) -> ToolSpec {
        ToolSpec {
            name: "fixture_action".into(),
            description: "Public fixture".into(),
            parameters: serde_json::json!({}),
        }
    }
    fn prepare(&self, c: &ToolCall) -> Result<ToolRequest, String> {
        Ok(ToolRequest {
            call_id: c.id.clone(),
            name: c.name.clone(),
            target: "public".into(),
            query: None,
            diff: None,
            command: None,
            mcp: None,
        })
    }
    async fn invoke(&self, _: &ToolRequest, _: CancellationToken) -> Result<String, String> {
        self.0.fetch_add(1, Ordering::SeqCst);
        Ok("Completed effect evidence".into())
    }
}
struct Allow;
#[async_trait]
impl ToolApproval for Allow {
    async fn authorize(&self, _: &ToolRequest, _: CancellationToken) -> Result<bool, String> {
        Ok(true)
    }
}
fn context() -> Vec<Message> {
    vec![
        Message {
            role: Role::System,
            content: "Host instructions".into(),
            parts: vec![],
        },
        Message {
            role: Role::User,
            content: "Original task 世界".into(),
            parts: vec![],
        },
    ]
}

#[tokio::test]
async fn generation_retry_preserves_effects_partial_evidence_and_cumulative_attempts() {
    for (failures, cap, expected_calls, paused) in [
        (1, None, 3, false),
        (9, None, 4, true),
        (9, Some(3), 3, true),
    ] {
        let p = Worker {
            calls: AtomicUsize::new(0),
            failures,
            retry: true,
        };
        let effect = Arc::new(Effect(AtomicUsize::new(0)));
        let budget = TaskBudget {
            model_calls: cap,
            ..Default::default()
        };
        let shared = Arc::new(SharedTaskBudget::new(budget));
        let (events, mut rx) = mpsc::channel(256);
        let reply = run_agent_with_shared_budget(
            &p,
            context(),
            &[effect.clone() as Arc<dyn ToolPlugin>],
            &Allow,
            events,
            CancellationToken::new(),
            budget,
            shared.clone(),
            false,
        )
        .await
        .unwrap();
        assert_eq!(p.calls.load(Ordering::SeqCst), expected_calls);
        assert_eq!(shared.usage().model_calls, expected_calls);
        assert_eq!(reply.summary.model_calls, expected_calls);
        assert_eq!(
            effect.0.load(Ordering::SeqCst),
            1,
            "Completed action must never be replayed"
        );
        assert_eq!(reply.summary.tools.len(), 1);
        assert_eq!(reply.pause.is_some(), paused);
        if paused {
            assert!(reply.answer.contains("known totals are incomplete"));
        }
        assert!(reply
            .summary
            .steps
            .iter()
            .any(|s| s.text.contains("interrupted before tool dispatch")));
        let mut retries = 0;
        while let Ok(e) = rx.try_recv() {
            if matches!(e, AgentEvent::ModelRetry { .. }) {
                retries += 1;
            }
        }
        assert!(retries <= 2);
        assert!(retries > 0);
    }
}
#[tokio::test]
async fn stop_in_retry_wait_prevents_the_next_generation() {
    let p = Worker {
        calls: AtomicUsize::new(0),
        failures: 9,
        retry: true,
    };
    let effect = Arc::new(Effect(AtomicUsize::new(0)));
    let cancel = CancellationToken::new();
    let stop = cancel.clone();
    let (events, mut rx) = mpsc::channel(256);
    let listener = tokio::spawn(async move {
        while let Some(e) = rx.recv().await {
            if matches!(e, AgentEvent::ModelRetry { .. }) {
                stop.cancel();
                break;
            }
        }
    });
    assert!(run_agent(
        &p,
        context(),
        &[effect.clone() as Arc<dyn ToolPlugin>],
        &Allow,
        events,
        cancel
    )
    .await
    .err()
    .unwrap()
    .contains("stopped"));
    listener.await.unwrap();
    assert_eq!(p.calls.load(Ordering::SeqCst), 2);
    assert_eq!(effect.0.load(Ordering::SeqCst), 1);
}
#[tokio::test]
async fn unknown_adapter_failure_does_not_opt_into_retry() {
    let p = Worker {
        calls: AtomicUsize::new(0),
        failures: 9,
        retry: false,
    };
    let effect = Arc::new(Effect(AtomicUsize::new(0)));
    let (events, _receiver) = mpsc::channel(256);
    let result = run_agent(
        &p,
        context(),
        &[effect],
        &Allow,
        events,
        CancellationToken::new(),
    )
    .await;
    // Keep the consumer alive, as a real host does.
    assert!(result.is_err());
    assert_eq!(p.calls.load(Ordering::SeqCst), 2);
}
