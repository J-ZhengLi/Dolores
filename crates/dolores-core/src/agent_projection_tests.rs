use crate::*;
use async_trait::async_trait;
use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc, Mutex,
};
use tokio::sync::mpsc;
use tokio_util::sync::CancellationToken;

struct Provider(AtomicUsize, Mutex<String>);
#[async_trait]
impl ModelProvider for Provider {
    fn descriptor(&self) -> PluginDescriptor {
        PluginDescriptor {
            id: "projection-fixture",
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
        if n == 0 {
            *self.1.lock().unwrap() = m[0].content.clone();
        } else {
            assert_eq!(
                m[0].content,
                *self.1.lock().unwrap(),
                "Changing counters must not invalidate the system prefix"
            );
        }
        assert!(m[1].content.contains("Keep exact task constraints 世界"));
        if n == 8 {
            let tools = m.iter().filter(|v| v.role == "tool").collect::<Vec<_>>();
            assert!(
                tools[0].content.len() < 3000,
                "Older successful evidence must not be resent in full"
            );
            assert!(
                tools.last().unwrap().content.len() > 12000,
                "Recent evidence must remain complete"
            );
            assert_eq!(tools.len(), 8);
            for t in tools {
                assert!(m
                    .iter()
                    .any(|v| v.calls.iter().any(|c| Some(&c.id) == t.call_id.as_ref())));
            }
        }
        Ok(AgentTurn {
            content: if n == 8 {
                "Verified".into()
            } else {
                String::new()
            },
            calls: if n == 8 {
                vec![]
            } else {
                vec![ToolCall {
                    id: format!("read-{n}"),
                    name: "read_text_file".into(),
                    arguments: format!(r#"{{"path":"source-{n}.rs"}}"#),
                }]
            },
            usage: None,
            output_limit: false,
        })
    }
}
struct Read;
#[async_trait]
impl ToolPlugin for Read {
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
    async fn invoke(&self, r: &ToolRequest, _: CancellationToken) -> Result<String, String> {
        Ok(format!("{}:{}:tail", r.call_id, "界".repeat(4500)))
    }
}
struct Allow;
#[async_trait]
impl ToolApproval for Allow {
    async fn authorize(&self, _: &ToolRequest, _: CancellationToken) -> Result<bool, String> {
        Ok(true)
    }
}

#[tokio::test]
async fn agent_projects_older_evidence_and_retains_complete_receipts() {
    let (events, _rx) = mpsc::channel(256);
    let reply = run_agent_with_budget(
        &Provider(AtomicUsize::new(0), Mutex::new(String::new())),
        vec![
            Message {
                role: Role::System,
                content: "Stable host".into(),
                parts: vec![],
            },
            Message {
                role: Role::User,
                content: "Keep exact task constraints 世界".into(),
                parts: vec![],
            },
        ],
        &[Arc::new(Read)],
        &Allow,
        events,
        CancellationToken::new(),
        TaskBudget {
            model_calls: Some(12),
            tool_calls: Some(12),
            ..Default::default()
        },
    )
    .await
    .unwrap();
    assert_eq!(reply.pause, None);
    assert_eq!(reply.summary.tools.len(), 8);
    assert!(reply.summary.tools.iter().all(|t| t.content.len() > 12000));
}
