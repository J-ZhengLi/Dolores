use dolores_core::*;
use std::{
    collections::VecDeque,
    sync::{Arc, Mutex},
};
use tokio_util::sync::CancellationToken;
pub(super) struct Model(pub Mutex<VecDeque<ToolCall>>);
#[async_trait::async_trait]
impl ModelProvider for Model {
    fn request_settings(&self) -> Option<RequestSettings> {
        Some(RequestSettings::default())
    }
    fn descriptor(&self) -> PluginDescriptor {
        PluginDescriptor {
            id: "compat-fixture",
            kind: "provider",
            api_version: 1,
        }
    }
    async fn stream(
        &self,
        _: Vec<Message>,
        _: tokio::sync::mpsc::Sender<String>,
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
        if let Some(call) = self.0.lock().unwrap().pop_front() {
            Ok(AgentTurn {
                content: String::new(),
                calls: vec![call],
                usage: None,
                output_limit: false,
            })
        } else {
            assert!(m.iter().any(|m| m.role == "tool"));
            Ok(AgentTurn {
                content: "Verified fixture".into(),
                calls: vec![],
                usage: None,
                output_limit: false,
            })
        }
    }
}
pub(super) struct Allow;
#[async_trait::async_trait]
impl ToolApproval for Allow {
    async fn authorize(&self, _: &ToolRequest, _: CancellationToken) -> Result<bool, String> {
        Ok(true)
    }
}
pub(super) async fn run(tools: Vec<Arc<dyn ToolPlugin>>, calls: Vec<ToolCall>) -> AgentReply {
    let model = Model(Mutex::new(calls.into()));
    let (tx, mut rx) = tokio::sync::mpsc::channel(32);
    let drain = tokio::spawn(async move { while rx.recv().await.is_some() {} });
    let result = run_agent(
        &model,
        preview_context(vec![], "Verify configured fixture").unwrap(),
        &tools,
        &Allow,
        tx,
        CancellationToken::new(),
    )
    .await
    .unwrap();
    drain.await.unwrap();
    result
}
