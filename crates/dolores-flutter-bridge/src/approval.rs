use crate::{forward, stopped};
use async_trait::async_trait;
use dolores_core::{ToolApproval, ToolRequest};
use serde_json::{json, Value};
use std::sync::{Arc, Mutex};
use tokio::sync::{mpsc, oneshot};
use tokio_util::sync::CancellationToken;

pub struct PendingApproval {
    pub call_id: String,
    pub reply: oneshot::Sender<bool>,
}
pub type ApprovalSlot = Arc<Mutex<Option<PendingApproval>>>;
pub struct RunApproval {
    pub policy: Option<Arc<crate::permissions::PermissionGuard>>,
    pub id: u64,
    pub pending: ApprovalSlot,
    pub output: mpsc::Sender<Value>,
    pub log: Option<Arc<crate::run_journal::RunLog>>,
}
struct ClearPending(ApprovalSlot);
impl Drop for ClearPending {
    fn drop(&mut self) {
        if let Ok(mut pending) = self.0.lock() {
            pending.take();
        }
    }
}
#[async_trait]
impl ToolApproval for RunApproval {
    fn is_waiting(&self) -> bool {
        self.pending.lock().map(|p| p.is_some()).unwrap_or(false)
    }
    async fn recheck(&self, _: &ToolRequest, cancel: CancellationToken) -> Result<(), String> {
        if cancel.is_cancelled() {
            return Err(stopped());
        }
        if let Some(policy) = &self.policy {
            policy.recheck()?;
        }
        Ok(())
    }
    async fn authorize(
        &self,
        request: &ToolRequest,
        cancel: CancellationToken,
    ) -> Result<bool, String> {
        self.recheck(request, cancel.clone()).await?;
        // This host-owned request only saves a pause. Actual sharing/authority
        // is decided in the local chooser, never in a model tool invocation.
        if matches!(request.name.as_str(), "request_desktop_access" | "schedule_task") {
            return Ok(true);
        }
        if let Some(policy) = &self.policy {
            if policy.automatic(request)? {
                if let Some(log) = &self.log {
                    log.record(None,"approvalAutomatic",json!({"callId":request.call_id,"name":request.name,"target":request.target,"mode":policy.policy.mode,"revision":policy.revision})).await?;
                }
                return Ok(true);
            }
        }
        if let Some(log) = &self.log {
            log.record(
                Some(dolores_core::RunState::WaitingForApproval),
                "approvalRequested",
                json!({"callId":request.call_id,"name":request.name,"target":request.target,"command":request.command.as_ref().map(|c|&c.invocation)}),
            )
            .await?;
        }
        let (reply, decision) = oneshot::channel();
        // UI decision identities must not collide when providers reuse call IDs.
        let approval_id = format!("{}:{}", self.id, uuid::Uuid::new_v4());
        {
            let mut pending = self
                .pending
                .lock()
                .map_err(|_| "Tool approval is unavailable.")?;
            if pending.is_some() {
                return Err("Another tool decision is pending.".into());
            }
            *pending = Some(PendingApproval {
                call_id: approval_id.clone(),
                reply,
            });
        }
        let _clear = ClearPending(self.pending.clone());
        let mut reviewed = json!(request);
        reviewed["callId"] = json!(approval_id);
        forward(
            &self.output,
            json!({"type":"toolApproval","id":self.id,"request":reviewed}),
            &cancel,
        )
        .await?;
        let allowed = tokio::select! { biased;
            _ = cancel.cancelled() => Err(stopped()),
            result = decision => result.map_err(|_| "Tool approval was closed.".into()),
        }?;
        self.recheck(request, cancel.clone()).await?;
        if let Some(log) = &self.log {
            log.record(
                Some(dolores_core::RunState::Running),
                "approvalDecided",
                json!({"callId":request.call_id,"allow":allowed}),
            )
            .await?;
        }
        Ok(allowed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[tokio::test]
    async fn decisions_are_single_use_and_pending_permissions_clear_on_stop_or_timeout() {
        for mode in 0..3 {
            let pending = Arc::new(Mutex::new(None));
            let (output, mut events) = mpsc::channel(4);
            let approval = Arc::new(RunApproval {
                policy: None,
                id: 7,
                pending: pending.clone(),
                output,
                log: None,
            });
            let cancel = CancellationToken::new();
            let token = cancel.clone();
            let task = tokio::spawn(async move {
                let request = ToolRequest {
                    call_id: "one".into(),
                    name: "read_text_file".into(),
                    target: "readme.txt".into(),
                    query: None,
                    diff: None,
                    command: None,
                    mcp: None,
                };
                tokio::time::timeout(
                    std::time::Duration::from_millis(150),
                    approval.authorize(&request, token),
                )
                .await
            });
            let event = events.recv().await.unwrap();
            assert_eq!(event["type"], "toolApproval");
            assert!(event["request"]["callId"].as_str().unwrap().starts_with("7:"));
            assert_ne!(event["request"]["callId"], "one");
            match mode {
                0 => pending
                    .lock()
                    .unwrap()
                    .take()
                    .unwrap()
                    .reply
                    .send(true)
                    .unwrap(),
                1 => cancel.cancel(),
                _ => {}
            }
            let result = task.await.unwrap();
            if mode == 0 {
                assert!(result.unwrap().unwrap());
            } else if mode == 1 {
                assert!(result.unwrap().is_err());
            } else {
                assert!(result.is_err());
            }
            assert!(pending.lock().unwrap().is_none());
        }
    }
}
