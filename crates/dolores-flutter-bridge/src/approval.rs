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
    pub id: u64,
    pub pending: ApprovalSlot,
    pub output: mpsc::Sender<Value>,
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
    async fn authorize(
        &self,
        request: &ToolRequest,
        cancel: CancellationToken,
    ) -> Result<bool, String> {
        let (reply, decision) = oneshot::channel();
        {
            let mut pending = self
                .pending
                .lock()
                .map_err(|_| "Tool approval is unavailable.")?;
            if pending.is_some() {
                return Err("Another tool decision is pending.".into());
            }
            *pending = Some(PendingApproval {
                call_id: request.call_id.clone(),
                reply,
            });
        }
        let _clear = ClearPending(self.pending.clone());
        forward(
            &self.output,
            json!({"type":"toolApproval","id":self.id,"request":request}),
            &cancel,
        )
        .await?;
        tokio::select! { biased;
            _ = cancel.cancelled() => Err(stopped()),
            result = decision => result.map_err(|_| "Tool approval was closed.".into()),
        }
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
                id: 7,
                pending: pending.clone(),
                output,
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
            assert_eq!(events.recv().await.unwrap()["type"], "toolApproval");
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
