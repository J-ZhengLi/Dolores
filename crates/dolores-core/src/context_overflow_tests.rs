use crate::*;
use async_trait::async_trait;
use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc, Mutex,
};
use tokio::sync::mpsc;
use tokio_util::sync::CancellationToken;

struct Provider {
    calls: AtomicUsize,
    repeated: bool,
    empty: bool,
    rejected_tokens: Mutex<u64>,
}
#[async_trait]
impl ModelProvider for Provider {
    fn descriptor(&self) -> PluginDescriptor {
        PluginDescriptor {
            id: "overflow",
            kind: "provider",
            api_version: 1,
        }
    }
    fn context_window_tokens(&self) -> Option<u32> {
        Some(500000)
    }
    fn context_overflow(&self, error: &str) -> bool {
        error == "Confirmed fixture context overflow"
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
        t: &[ToolSpec],
        _: CancellationToken,
    ) -> Result<AgentTurn, String> {
        let n = self.calls.fetch_add(1, Ordering::SeqCst);
        assert!(m
            .iter()
            .any(|m| m.content.contains("Exact original objective 世界")));
        let tokens = estimate_agent_tokens(m, t).unwrap();
        if self.empty || n == 1 {
            *self.rejected_tokens.lock().unwrap() = tokens;
            return Err("Confirmed fixture context overflow".into());
        }
        if n > 1 {
            assert!(
                tokens < *self.rejected_tokens.lock().unwrap(),
                "Never retry an unreduced rejected envelope"
            );
            if self.repeated {
                return Err("Confirmed fixture context overflow".into());
            }
        }
        Ok(AgentTurn {
            content: if n == 0 {
                String::new()
            } else {
                "Verified complete".into()
            },
            calls: if n == 0 {
                (0..8)
                    .map(|i| ToolCall {
                        id: format!("read-{i}"),
                        name: "read_text_file".into(),
                        arguments: format!(r#"{{"path":"file-{i}"}}"#),
                    })
                    .collect()
            } else {
                vec![]
            },
            usage: None,
            output_limit: false,
        })
    }
}
struct Read(AtomicUsize);
#[async_trait]
impl ToolPlugin for Read {
    fn spec(&self) -> ToolSpec {
        ToolSpec {
            name: "read_text_file".into(),
            description: "Read public evidence".into(),
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
    async fn invoke(&self, r: &ToolRequest, _: CancellationToken) -> Result<String, String> {
        self.0.fetch_add(1, Ordering::SeqCst);
        Ok(format!(
            "{} {}",
            r.call_id,
            "Public evidence 世界. ".repeat(650)
        ))
    }
}
struct Approval(AtomicUsize);
#[async_trait]
impl ToolApproval for Approval {
    fn context_recovery_available(&self) -> bool {
        true
    }
    async fn compact_context(
        &self,
        _: &[AgentMessage],
        _: CancellationToken,
    ) -> Result<Option<context_recovery::ContextCheckpoint>, String> {
        self.0.fetch_add(1, Ordering::SeqCst);
        Ok(Some(context_recovery::ContextCheckpoint {
            decisions: vec!["Preserve scope".into()],
            progress: vec!["Evidence saved".into()],
            remaining: vec!["Verify result".into()],
        }))
    }
    async fn authorize(&self, _: &ToolRequest, _: CancellationToken) -> Result<bool, String> {
        Ok(true)
    }
}
#[tokio::test]
async fn confirmed_context_rejection_compacts_once_before_a_smaller_retry() {
    for (repeated, empty) in [(false, false), (true, false), (false, true)] {
        let provider = Provider {
            calls: AtomicUsize::new(0),
            repeated,
            empty,
            rejected_tokens: Mutex::new(0),
        };
        let tool = Arc::new(Read(AtomicUsize::new(0)));
        let approval = Approval(AtomicUsize::new(0));
        let (events, _rx) = mpsc::channel(256);
        let plugins = [tool.clone() as Arc<dyn ToolPlugin>];
        let reply = run_agent(
            &provider,
            preview_context(vec![], "Exact original objective 世界").unwrap(),
            &plugins,
            &approval,
            events,
            CancellationToken::new(),
        )
        .await
        .unwrap();
        assert_eq!(approval.0.load(Ordering::SeqCst), 1);
        assert_eq!(
            tool.0.load(Ordering::SeqCst),
            if empty { 0 } else { 8 },
            "Completed effects must never replay"
        );
        assert_eq!(
            provider.calls.load(Ordering::SeqCst),
            if empty { 1 } else { 3 }
        );
        assert_eq!(reply.pause.is_some(), repeated || empty);
        assert_eq!(reply.summary.model_calls, if empty { 2 } else { 4 });
        if !repeated && !empty {
            assert_eq!(reply.answer, "Verified complete");
        }
    }
}
