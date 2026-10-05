use crate::{blocking, Run};
use async_trait::async_trait;
use dolores_core::{RunState, SessionStore, ToolCall, ToolPlugin, ToolRequest, ToolSpec};
use serde_json::{json, Value};
use std::{
    collections::BTreeMap,
    fs::{File, OpenOptions},
    path::Path,
    sync::{Arc, Mutex},
};
use tokio_util::sync::CancellationToken;

/// The execution allowance remains one. IDs own cancellation, queues and decisions.
#[derive(Default)]
pub(super) struct RunCoordinator {
    runs: BTreeMap<u64, Run>,
    closed: bool,
}
impl RunCoordinator {
    pub fn reserve(&mut self, run: Run) -> Result<(), String> {
        if self.closed || !self.runs.is_empty() {
            return Err("Stop the current response first. Only one execution is allowed.".into());
        }
        self.runs.insert(run.id, run);
        Ok(())
    }
    pub fn is_some(&self) -> bool {
        !self.runs.is_empty()
    }
    pub fn as_ref(&self) -> Option<&Run> {
        self.runs.values().next()
    }
    pub fn as_mut(&mut self) -> Option<&mut Run> {
        self.runs.values_mut().next()
    }
    pub fn take(&mut self) -> Option<Run> {
        self.runs.pop_first().map(|(_, r)| r)
    }
    pub fn remove(&mut self, id: u64) {
        self.runs.remove(&id);
    }
    pub fn close(&mut self) {
        self.closed = true;
    }
    pub fn closed(&self) -> bool {
        self.closed
    }
}
pub(super) fn lock_directory(directory: &Path) -> Result<File, String> {
    let file = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(false)
        .open(directory.join("dolores.lock"))
        .map_err(|_| "Could not open the application data lock.")?;
    file.try_lock().map_err(|_|"Another Dolores instance owns this data location. Close that instance and restart this one.")?;
    Ok(file)
}
pub(super) struct RunLog {
    store: Arc<dyn SessionStore>,
    pub id: String,
    sequence: Mutex<u32>,
}
impl RunLog {
    pub fn new(store: Arc<dyn SessionStore>, id: String) -> Arc<Self> {
        Arc::new(Self {
            store,
            id,
            sequence: Mutex::new(0),
        })
    }
    pub async fn record(
        self: &Arc<Self>,
        state: Option<RunState>,
        kind: &str,
        data: Value,
    ) -> Result<(), String> {
        let this = self.clone();
        let kind = kind.to_owned();
        blocking(move || {
            let mut sequence = this
                .sequence
                .lock()
                .map_err(|_| "Run evidence is unavailable.")?;
            *sequence = this
                .store
                .append_run_event(&this.id, *sequence, state, &kind, &data)?;
            Ok(())
        })
        .await
    }
}
pub(super) struct LoggedTool {
    pub inner: Arc<dyn ToolPlugin>,
    pub log: Arc<RunLog>,
    pub policy: Option<Arc<crate::permissions::PermissionGuard>>,
}
#[async_trait]
impl ToolPlugin for LoggedTool {
    fn image_results(&self) -> Vec<dolores_core::AttachmentRef> {
        self.inner.image_results()
    }
    fn spec(&self) -> ToolSpec {
        self.inner.spec()
    }
    fn prepare(&self, call: &ToolCall) -> Result<ToolRequest, String> {
        self.inner.prepare(call)
    }
    async fn invoke(
        &self,
        request: &ToolRequest,
        cancel: CancellationToken,
    ) -> Result<String, String> {
        self.log.record(None,"toolIntent",json!({"callId":request.call_id,"name":request.name,"target":request.target,"certainty":"Effect outcome is unknown until a result is recorded."})).await?;
        if cancel.is_cancelled() {
            return Err(crate::stopped());
        }
        if let Some(policy) = &self.policy {
            policy.recheck()?;
        }
        let result = self.inner.invoke(request, cancel).await;
        let content = match &result {
            Ok(text) | Err(text) if text.len() <= dolores_core::MAX_TOOL_BYTES => {
                Some(text.as_str())
            }
            _ => None,
        };
        self.log.record(None,"toolResult",json!({"callId":request.call_id,"name":request.name,"target":request.target,"returned":result.is_ok(),"content":content,"parts":if result.is_ok(){self.inner.image_results()}else{vec![]},"oversized":content.is_none(),"note":"Returned errors do not establish that external effects were undone. Inspect Changes or command receipts."})).await.map_err(|_|"An operation returned but its run evidence could not be saved. Inspect Changes and external effects before retrying.")?;
        result
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn data_lock_refuses_a_second_owner_and_recovers_after_release() {
        let dir = tempfile::tempdir().unwrap();
        let first = lock_directory(dir.path()).unwrap();
        assert!(lock_directory(dir.path())
            .unwrap_err()
            .contains("Another Dolores"));
        drop(first);
        assert!(lock_directory(dir.path()).is_ok());
    }
    #[test]
    fn coordinator_reserves_one_owner_and_rejects_after_shutdown() {
        fn run(id: u64) -> Run {
            Run {
                thread: None,
                id,
                cancel: CancellationToken::new(),
                events: tokio::sync::mpsc::channel(1).1,
                approvals: Arc::new(Mutex::new(None)),
            }
        }
        let mut coordinator = RunCoordinator::default();
        coordinator.reserve(run(1)).unwrap();
        assert!(coordinator.reserve(run(2)).is_err());
        coordinator.remove(2);
        assert!(coordinator.is_some());
        coordinator.remove(1);
        coordinator.reserve(run(2)).unwrap();
        coordinator.take();
        coordinator.close();
        assert!(coordinator.reserve(run(3)).is_err());
    }

    #[test]
    fn read_only_other_chat_inspection_and_wrong_id_stop_leave_owner_running() {
        let store =
            Arc::new(dolores_store_sqlite::SqliteStore::open(Path::new(":memory:")).unwrap());
        store.create("other").unwrap();
        let engine = crate::Engine::new(
            store,
            Arc::new(crate::connection::testing::MemoryCredentials::default()),
        )
        .unwrap();
        let cancel = CancellationToken::new();
        engine
            .active
            .lock()
            .unwrap()
            .reserve(Run {
                thread: None,
                id: 7,
                cancel: cancel.clone(),
                events: tokio::sync::mpsc::channel(1).1,
                approvals: Arc::new(Mutex::new(None)),
            })
            .unwrap();
        assert!(engine
            .call(crate::Command::Runs {
                session: "other".into()
            })
            .is_ok());
        assert!(engine
            .call(crate::Command::MessagesPage {
                session: "other".into(),
                cursor: None,
                newer: false
            })
            .is_ok());
        assert!(engine
            .call(crate::Command::Delete {
                session: "other".into()
            })
            .is_err());
        engine.call(crate::Command::Cancel { id: 8 }).unwrap();
        assert!(!cancel.is_cancelled());
        engine.call(crate::Command::Cancel { id: 7 }).unwrap();
        assert!(cancel.is_cancelled());
    }
}
