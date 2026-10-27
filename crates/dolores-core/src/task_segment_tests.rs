use crate::*;
use std::sync::{
    atomic::{AtomicBool, AtomicUsize, Ordering},
    Arc,
};
use tokio::sync::mpsc;
use tokio_util::sync::CancellationToken;

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
        m: &[AgentMessage],
        _: &[ToolSpec],
        _: CancellationToken,
    ) -> Result<AgentTurn, String> {
        let n = self.0.fetch_add(1, Ordering::SeqCst);
        if n >= 24 {
            assert!(m
                .iter()
                .any(|m| m.content.contains("Preserve correction 世界")));
        }
        Ok(AgentTurn {
            content: String::new(),
            calls: vec![ToolCall {
                id: format!("call-{n}"),
                name: "read_text_file".into(),
                arguments: format!("{{\"path\":\"{n}\"}}"),
            }],
            usage: None,
            output_limit: false,
        })
    }
}
struct Tool(AtomicUsize);
#[async_trait::async_trait]
impl ToolPlugin for Tool {
    fn spec(&self) -> ToolSpec {
        ToolSpec {
            name: "read_text_file".into(),
            description: "Read".into(),
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
        Ok("Evidence".into())
    }
}
struct Host {
    stop: bool,
    fail: bool,
    pending: AtomicBool,
    saves: AtomicUsize,
}
#[async_trait::async_trait]
impl ToolApproval for Host {
    fn task_segments_available(&self) -> bool {
        true
    }
    fn context_recovery_available(&self) -> bool {
        true
    }
    async fn compact_context(
        &self,
        _: &[AgentMessage],
        _: CancellationToken,
    ) -> Result<Option<context_recovery::ContextCheckpoint>, String> {
        panic!("Evidence pagination must not compact a mostly empty model context")
    }
    fn take_steering(&self) -> Result<Vec<SteeringInput>, String> {
        Ok(if self.pending.swap(false, Ordering::SeqCst) {
            vec![SteeringInput {
                text: "Preserve correction 世界".into(),
                skills: vec![],
            }]
        } else {
            vec![]
        })
    }
    async fn save_task_segment(
        &self,
        _: &AgentSummary,
        _: &[AgentMessage],
        c: CancellationToken,
    ) -> Result<(String, u32), String> {
        self.pending.store(true, Ordering::SeqCst);
        let n = self.saves.fetch_add(1, Ordering::SeqCst) + 1;
        if self.stop {
            c.cancel();
        }
        if self.fail {
            return Err("Fixture checkpoint storage failed".into());
        }
        Ok(("fixture-run".into(), n as u32))
    }
    async fn authorize(&self, _: &ToolRequest, _: CancellationToken) -> Result<bool, String> {
        Ok(true)
    }
}
#[tokio::test]
async fn rolling_honors_cumulative_caps_stop_storage_failure_and_mid_maintenance_steering() {
    for (stop, fail) in [(false, false), (true, false), (false, true)] {
        let provider = Worker(AtomicUsize::new(0));
        let tool = Arc::new(Tool(AtomicUsize::new(0)));
        let host = Host {
            stop,
            fail,
            pending: AtomicBool::new(false),
            saves: AtomicUsize::new(0),
        };
        let budget = TaskBudget {
            model_calls: Some(60),
            tool_calls: Some(100),
            ..Default::default()
        };
        let shared = Arc::new(SharedTaskBudget::new(budget));
        let (tx, _rx) = mpsc::channel(1024);
        let result = run_agent_with_shared_budget(
            &provider,
            preview_context(vec![], "Original task").unwrap(),
            &[tool.clone() as Arc<dyn ToolPlugin>],
            &host,
            tx,
            CancellationToken::new(),
            budget,
            shared.clone(),
            false,
        )
        .await;
        if stop {
            assert!(result.err().unwrap().contains("stopped"));
            assert_eq!(provider.0.load(Ordering::SeqCst), 24);
        } else if fail {
            let reply = result.unwrap();
            assert_eq!(reply.pause, Some(PauseReason::Checkpoint));
            assert!(reply.answer.contains("storage failed"));
            assert_eq!(provider.0.load(Ordering::SeqCst), 24);
            assert_eq!(reply.summary.tools.len(), 24);
        } else {
            let reply = result.unwrap();
            assert!(reply.pause.is_some());
            assert_eq!(shared.usage().model_calls, 60);
            assert_eq!(reply.summary.model_calls, 60);
            assert!(reply.summary.archive.is_some());
        }
        assert!(host.saves.load(Ordering::SeqCst) > 0);
    }
}
