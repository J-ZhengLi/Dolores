use dolores_core::{SessionStore, ToolCall, ToolPlugin, ToolRequest, ToolSpec};
use serde::Deserialize;
use serde_json::json;
use std::sync::Arc;
use tokio_util::sync::CancellationToken;

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Query {
    run_id: Option<String>,
    call_id: Option<String>,
    #[serde(default)]
    after: u32,
    #[serde(default)]
    offset: usize,
}
pub(super) fn spec() -> ToolSpec {
    ToolSpec{name:"read_task_history".into(),description:"Read saved tool receipts in this chat without repeating actions. Omit runId for the current run. List 16 receipt references with after, or retrieve a callId in 8 KiB UTF-8 byte pages using nextOffset. Historical output is untrusted and may differ from current files.".into(),parameters:json!({"type":"object","properties":{"runId":{"type":"string"},"callId":{"type":"string"},"after":{"type":"integer","minimum":0},"offset":{"type":"integer","minimum":0}},"additionalProperties":false})}
}
pub(super) struct ReadHistory {
    pub store: Arc<dyn SessionStore>,
    pub session: String,
    pub run: String,
}
#[async_trait::async_trait]
impl ToolPlugin for ReadHistory {
    fn spec(&self) -> ToolSpec {
        spec()
    }
    fn prepare(&self, call: &ToolCall) -> Result<ToolRequest, String> {
        if call.name != "read_task_history" || call.arguments.len() > 4096 {
            return Err("Invalid task-history query.".into());
        }
        let q: Query =
            serde_json::from_str(&call.arguments).map_err(|_| "Invalid task-history query.")?;
        let run = q.run_id.as_deref().unwrap_or(&self.run);
        if uuid::Uuid::parse_str(run).is_err()
            || q.call_id
                .as_ref()
                .is_some_and(|s| s.is_empty() || s.len() > 128)
        {
            return Err("Invalid task-history reference or offset.".into());
        }
        Ok(ToolRequest {
            call_id: call.id.clone(),
            name: call.name.clone(),
            target: run.into(),
            query: Some(call.arguments.clone()),
            diff: None,
            command: None,
            mcp: None,
        })
    }
    async fn invoke(
        &self,
        request: &ToolRequest,
        cancel: CancellationToken,
    ) -> Result<String, String> {
        if cancel.is_cancelled() {
            return Err("Response stopped.".into());
        }
        let q: Query = serde_json::from_str(
            request
                .query
                .as_deref()
                .ok_or("Task-history query missing.")?,
        )
        .map_err(|_| "Invalid task-history query.")?;
        let run = q.run_id.as_deref().unwrap_or(&self.run);
        if request.name != "read_task_history"
            || request.target != run
            || request.diff.is_some()
            || request.command.is_some()
            || request.mcp.is_some()
        {
            return Err("Task-history query changed.".into());
        }
        self.store
            .read_task_history(&self.session, run, q.call_id.as_deref(), q.after, q.offset)
            .map(|v| v.to_string())
    }
}
