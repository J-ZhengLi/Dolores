use crate::*;
use async_trait::async_trait;
use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
};
use tokio::sync::mpsc;
use tokio_util::sync::CancellationToken;

struct Provider(AtomicUsize);
#[async_trait]
impl ModelProvider for Provider {
    fn descriptor(&self) -> PluginDescriptor {
        PluginDescriptor {
            id: "catalogue",
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
        specs: &[ToolSpec],
        _: CancellationToken,
    ) -> Result<AgentTurn, String> {
        let n = self.0.fetch_add(1, Ordering::SeqCst);
        assert_eq!(specs.len(), if n == 0 || n == 3 { 1 } else { 2 });
        if n == 1 || n == 2 {
            assert!(specs.iter().any(|s| s.name == "optional_9"));
        }
        Ok(AgentTurn {
            content: if n == 3 {
                "Complete".into()
            } else {
                String::new()
            },
            calls: match n {
                0 => vec![ToolCall {
                    id: "discover".into(),
                    name: "discover_tools".into(),
                    arguments: r#"{"names":["optional_9"]}"#.into(),
                }],
                1 => vec![ToolCall {
                    id: "effect".into(),
                    name: "optional_9".into(),
                    arguments: "{}".into(),
                }],
                2 => vec![ToolCall {
                    id: "unload".into(),
                    name: "discover_tools".into(),
                    arguments: r#"{"unload":["optional_9"]}"#.into(),
                }],
                _ => vec![],
            },
            usage: None,
            output_limit: false,
        })
    }
}
struct Plugin(usize, Arc<AtomicUsize>);
#[async_trait]
impl ToolPlugin for Plugin {
    fn spec(&self) -> ToolSpec {
        ToolSpec {
            name: format!("optional_{}", self.0),
            description: "Optional tool".into(),
            parameters: serde_json::json!({}),
        }
    }
    fn prepare(&self, c: &ToolCall) -> Result<ToolRequest, String> {
        Ok(ToolRequest {
            call_id: c.id.clone(),
            name: c.name.clone(),
            target: "fixture".into(),
            query: None,
            diff: None,
            command: None,
            mcp: None,
        })
    }
    async fn invoke(&self, _: &ToolRequest, _: CancellationToken) -> Result<String, String> {
        self.1.fetch_add(1, Ordering::SeqCst);
        Ok("Completed fixture".into())
    }
}
struct Approval(AtomicUsize);
#[async_trait]
impl ToolApproval for Approval {
    async fn authorize(&self, _: &ToolRequest, _: CancellationToken) -> Result<bool, String> {
        self.0.fetch_add(1, Ordering::SeqCst);
        Ok(true)
    }
}
#[tokio::test]
async fn agent_loads_deferred_tools_without_executing_or_approving_them() {
    let effects = Arc::new(AtomicUsize::new(0));
    let plugins = (0..10)
        .map(|i| Arc::new(Plugin(i, effects.clone())) as Arc<dyn ToolPlugin>)
        .collect::<Vec<_>>();
    let approval = Approval(AtomicUsize::new(0));
    let (events, _rx) = mpsc::channel(128);
    let reply = run_agent(
        &Provider(AtomicUsize::new(0)),
        preview_context(vec![], "Use the optional fixture tool").unwrap(),
        &plugins,
        &approval,
        events,
        CancellationToken::new(),
    )
    .await
    .unwrap();
    assert_eq!(effects.load(Ordering::SeqCst), 1);
    assert_eq!(approval.0.load(Ordering::SeqCst), 1);
    assert_eq!(reply.summary.tools.len(), 3);
    assert_eq!(reply.summary.tools[0].name, "discover_tools");
    assert_eq!(reply.pause, None);
}
