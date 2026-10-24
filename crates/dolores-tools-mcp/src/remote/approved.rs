//! Frozen remote targets participate in the ordinary one-use approval path.
use super::{credentials, headers, RemoteClient, MODERN};
use async_trait::async_trait;
use dolores_core::{
    CredentialStore, McpCallPreview, McpConnection, McpRemoteAuthentication, ToolCall, ToolPlugin,
    ToolRequest, ToolSpec,
};
use serde_json::{json, Value};
use std::{
    collections::HashMap,
    path::PathBuf,
    sync::{Arc, Mutex},
    time::Duration,
};
use tokio::time::Instant;
use tokio_util::sync::CancellationToken;

struct RemotePlugin {
    root: PathBuf,
    connection: McpConnection,
    index: usize,
    vault: Option<Arc<dyn CredentialStore>>,
    plans: Mutex<HashMap<String, ToolRequest>>,
}
pub(crate) fn plugins(
    root: PathBuf,
    connection: McpConnection,
    vault: Option<Arc<dyn CredentialStore>>,
) -> Result<Vec<Arc<dyn ToolPlugin>>, String> {
    connection.validate()?;
    if connection.remote.is_none() {
        return Err("Choose a reviewed remote MCP endpoint.".into());
    }
    Ok((0..connection.tools.len())
        .map(|index| {
            Arc::new(RemotePlugin {
                root: root.clone(),
                connection: connection.clone(),
                index,
                vault: vault.clone(),
                plans: Mutex::new(HashMap::new()),
            }) as Arc<dyn ToolPlugin>
        })
        .collect())
}
#[async_trait]
impl ToolPlugin for RemotePlugin {
    fn spec(&self) -> ToolSpec {
        self.connection.specs().remove(self.index)
    }
    fn prepare(&self, call: &ToolCall) -> Result<ToolRequest, String> {
        dolores_core::validate_call(call)?;
        if call.name != self.spec().name {
            return Err("MCP tool alias is unavailable.".into());
        }
        let arguments: Value =
            serde_json::from_str(&call.arguments).map_err(|_| "MCP arguments must be JSON.")?;
        if !arguments.is_object() || call.arguments.contains('\0') || call.arguments.len() > 4096 {
            return Err("Remote MCP arguments must be a JSON object within 4 KiB.".into());
        }
        let target = self
            .connection
            .remote
            .as_ref()
            .ok_or("Remote MCP target is missing.")?;
        let tool = &self.connection.tools[self.index];
        if self.connection.protocol_version == MODERN {
            headers::tool_headers(&tool.name, &tool.input_schema, &arguments)
                .map_err(|e| e.to_string())?;
        }
        let request = ToolRequest {
            call_id: call.id.clone(),
            name: call.name.clone(),
            target: format!("{} / {}", target.label, tool.name),
            query: None,
            diff: None,
            command: None,
            mcp: Some(Box::new(McpCallPreview {
                connection_id: self.connection.id.clone(),
                server: target.label.clone(),
                tool: tool.name.clone(),
                arguments: arguments.to_string(),
                revision: self.connection.revision,
                isolation: None,
                remote_endpoint: Some(target.endpoint.clone()),
                credential_names: self
                    .connection
                    .credentials
                    .iter()
                    .map(|b| b.name.clone())
                    .collect(),
            })),
        };
        let mut plans = self
            .plans
            .lock()
            .map_err(|_| "MCP approval is unavailable.")?;
        if plans.len() >= dolores_core::MAX_TOOL_CALLS || plans.contains_key(&call.id) {
            return Err("MCP approval limit reached.".into());
        }
        plans.insert(call.id.clone(), request.clone());
        Ok(request)
    }
    fn discard(&self, request: &ToolRequest) {
        if let Ok(mut plans) = self.plans.lock() {
            plans.remove(&request.call_id);
        }
    }
    async fn invoke(
        &self,
        request: &ToolRequest,
        cancel: CancellationToken,
    ) -> Result<String, String> {
        let plan = self
            .plans
            .lock()
            .map_err(|_| "MCP approval is unavailable.")?
            .remove(&request.call_id)
            .ok_or("MCP approval expired. Request a fresh tool call.")?;
        if plan != *request {
            return Err("MCP call changed after approval. Nothing was sent.".into());
        }
        let cancel = cancel.child_token();
        let _guard = crate::CancelOnDrop(cancel.clone());
        let deadline = Instant::now() + Duration::from_secs(30);
        let (root, connection, vault) = (
            self.root.clone(),
            self.connection.clone(),
            self.vault.clone(),
        );
        let credential = tokio::select! {
            biased;
            _=cancel.cancelled()=>return Err(super::fail(super::Kind::Cancelled).to_string()),
            _=tokio::time::sleep_until(deadline)=>return Err(super::fail(super::Kind::Deadline).to_string()),
            result=async move { tokio::task::spawn_blocking(move||{
                if root.canonicalize().ok().as_ref()!=Some(&root){return Err("Working folder changed. Review the MCP connection again.".into());}
                match connection.remote.as_ref().ok_or("Remote MCP target is missing.")?.authentication {
                    McpRemoteAuthentication::None=>Ok(None),
                    McpRemoteAuthentication::Bearer=>credentials::resolve_bearer(&root,&connection,vault.as_ref().ok_or(crate::credentials::VAULT_ERROR)?.as_ref()).map(Some),
                }
            }).await }=>result.map_err(|_|"MCP credential worker failed.")??,
        };
        let target = self
            .connection
            .remote
            .as_ref()
            .ok_or("Remote MCP target is missing.")?;
        let mut client = RemoteClient::open(
            &target.endpoint,
            credential.as_ref().map(|c| c.expose()),
            Some(&self.connection.protocol_version),
            cancel.clone(),
            deadline,
        )
        .await
        .map_err(|e| e.to_string())?;
        let outcome=async {
            let catalog=client.list_tools().await.map_err(|e|e.to_string())?;
            let server=client.server();
            if server.name.as_deref().unwrap_or(&target.label)!=self.connection.server_name || server.version.as_deref().unwrap_or("")!=self.connection.server_version
                || self.connection.tools.iter().any(|t|!catalog.contains(t)) {
                return Err("Reviewed remote MCP metadata changed. Inspect and review the endpoint again; no tool call was sent.".into());
            }
            let arguments=serde_json::from_str(&plan.mcp.ok_or("MCP approval has no arguments.")?.arguments).map_err(|_|"MCP arguments are invalid.")?;
            let result=client.call_tool(&self.connection.tools[self.index],arguments).await.map_err(dispatched_error)?;
            let (text,is_error)=crate::text_content(&result).map_err(dispatched_error)?;
            Ok::<_,String>(json!({"text":text,"isError":is_error}))
        }.await;
        let cleanup = client.close().await;
        let mut result = outcome?;
        if cleanup.is_err() {
            result["cleanupWarning"]=json!("The tool result was retained, but remote session cleanup was not confirmed. Inspect connection health; do not repeat the tool to clean up.");
        }
        let encoded = result.to_string();
        if encoded.len() > dolores_core::MAX_TOOL_BYTES {
            return Err(dispatched_error(format!(
                "MCP result exceeds its {}-byte encoded limit.",
                dolores_core::MAX_TOOL_BYTES
            )));
        }
        Ok(encoded)
    }
}
fn dispatched_error(error: impl std::fmt::Display) -> String {
    format!("{error} External effects may remain. Inspect the remote operation before preparing another call; do not repeat it just to recover its output.")
}
