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

/// One app owner, with ID-addressed primary runs and exclusive maintenance work.
#[derive(Default)]
pub(super) struct RunCoordinator {
    runs: BTreeMap<u64, Run>,
    closed: bool,
    primary: BTreeMap<u64, String>,
    admission: Arc<crate::run_admission::Admission>,
}
impl RunCoordinator {
    pub fn has_session(&self, session: &str) -> bool {
        self.runs
            .values()
            .any(|run| run.thread.as_deref() == Some(session))
    }
    pub fn client_id(&self, session: &str) -> Option<u64> {
        self.runs
            .values()
            .find(|run| run.thread.as_deref() == Some(session))
            .map(|run| run.id)
    }
    pub fn has_repo(&self, root: &std::path::Path) -> bool {
        self.primary.values().any(|scope| {
            crate::source_control::within(std::path::Path::new(scope), root)
                || crate::source_control::within(root, std::path::Path::new(scope))
        }) || self.primary.len() != self.runs.len()
    }
    pub fn has_repository(&self, common: &Path) -> Result<bool, String> {
        if self.primary.len() != self.runs.len() {
            return Ok(true);
        }
        for scope in self.primary.values() {
            if crate::repository_scope::matches(Path::new(scope), common)? {
                return Ok(true);
            }
        }
        Ok(false)
    }
    #[cfg(test)]
    pub fn primary_scope(&self, id: u64) -> Option<&str> {
        self.primary.get(&id).map(String::as_str)
    }
    pub fn reserve(&mut self, run: Run) -> Result<(), String> {
        if self.closed || !self.runs.is_empty() {
            return Err("Stop the current response first. Only one execution is allowed.".into());
        }
        self.runs.insert(run.id, run);
        Ok(())
    }
    pub fn reserve_primary(
        &mut self,
        run: Run,
        scope: String,
    ) -> Result<Arc<crate::run_admission::Admission>, String> {
        self.check_primary(run.id, run.thread.as_deref())?;
        self.admission.insert(run.id, scope.clone())?;
        self.primary.insert(run.id, scope);
        self.runs.insert(run.id, run);
        Ok(self.admission.clone())
    }
    pub fn check_primary(&self, id: u64, thread: Option<&str>) -> Result<(), String> {
        if self.closed || self.primary.len() != self.runs.len() {
            return Err("Finish the current maintenance operation before starting a task.".into());
        }
        if self.runs.contains_key(&id) || self.runs.values().any(|r| r.thread.as_deref() == thread)
        {
            return Err(
                "This conversation already owns a task. Stop it before sending another.".into(),
            );
        }
        if self.runs.len() >= crate::run_admission::MAX_PRIMARY + crate::run_admission::MAX_QUEUED {
            return Err(
                "The run queue is full. Stop or cancel a task before sending; your draft remains."
                    .into(),
            );
        }
        Ok(())
    }
    pub fn get(&self, id: u64) -> Option<&Run> {
        self.runs.get(&id)
    }
    pub fn get_mut(&mut self, id: u64) -> Option<&mut Run> {
        self.runs.get_mut(&id)
    }
    pub fn cancel_session(&self, session: &str) {
        for run in self
            .runs
            .values()
            .filter(|r| r.thread.as_deref() == Some(session))
        {
            run.cancel.cancel();
        }
    }
    pub fn is_some(&self) -> bool {
        !self.runs.is_empty()
    }
    #[cfg(test)]
    pub fn as_mut(&mut self) -> Option<&mut Run> {
        self.runs.values_mut().next()
    }
    pub fn take(&mut self) -> Option<Run> {
        let (id, run) = self.runs.pop_first()?;
        self.primary.remove(&id);
        self.admission.remove(id);
        Some(run)
    }
    pub fn remove(&mut self, id: u64) {
        self.runs.remove(&id);
        self.primary.remove(&id);
        self.admission.remove(id);
    }
    pub fn close(&mut self) {
        self.closed = true;
        self.admission.close();
        for run in self.runs.values() {
            run.cancel.cancel();
        }
    }
    pub fn check_scope(&self, id: u64, scope: &str) -> Result<(), String> {
        self.admission.check(id, scope)
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
    pub async fn seal(
        self: &Arc<Self>,
        records: Vec<Value>,
        checkpoint: Value,
    ) -> Result<dolores_core::RunSegment, String> {
        let this = self.clone();
        blocking(move || {
            let mut sequence = this
                .sequence
                .lock()
                .map_err(|_| "Run evidence is unavailable.")?;
            let (next, segment) =
                this.store
                    .seal_run_segment(&this.id, *sequence, &records, &checkpoint)?;
            *sequence = next;
            Ok(segment)
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
    fn discard(&self, request: &ToolRequest) {
        self.inner.discard(request);
    }
    async fn invoke(
        &self,
        request: &ToolRequest,
        cancel: CancellationToken,
    ) -> Result<String, String> {
        self.log.record(None,"toolIntent",json!({"callId":request.call_id,"name":request.name,"target":request.target,"arguments":if matches!(request.name.as_str(),"desktop_control"|"command_session") {request.query.clone()}else{None},"certainty":"Effect outcome is unknown until a result is recorded."})).await?;
        if cancel.is_cancelled() {
            return Err(crate::stopped());
        }
        if let Some(policy) = &self.policy {
            policy.recheck()?;
        }
        let mut result = self.inner.invoke(request, cancel).await;
        let original = result.as_ref().map_or_else(|s| s.as_str(), |s| s.as_str());
        let original_bytes = original.len();
        let fragmented = original_bytes > dolores_core::MAX_TOOL_BYTES;
        if fragmented {
            let mut offset = 0;
            while offset < original.len() {
                let mut end = (offset + 16 * 1024).min(original.len());
                while !original.is_char_boundary(end) {
                    end -= 1;
                }
                self.log.record(None,"toolResultFragment",json!({"callId":request.call_id,"offset":offset,"totalBytes":original.len(),"text":&original[offset..end]})).await.map_err(|_|"An operation returned but its complete output could not be saved. Inspect effects before retrying.")?;
                offset = end;
            }
            let mut preview = serde_json::from_str::<Value>(original)
                .ok()
                .filter(Value::is_object)
                .unwrap_or_else(|| json!({"preview":text_prefix(original,2048)}));
            for field in ["text", "stdout", "stderr", "output"] {
                if let Some(text) = preview[field].as_str().filter(|s| s.len() > 2048) {
                    preview[field] = json!(text_prefix(text, 2048));
                }
            }
            preview["sourceCallId"] = json!(request.call_id);
            preview["originalBytes"] = json!(original_bytes);
            preview["previewTruncated"] = json!(true);
            preview["historyNote"]=json!("Complete output is saved. Use read_task_history with this sourceCallId as callId and nextOffset; do not rerun actions to recover omitted output.");
            let mut encoded = preview.to_string();
            if encoded.len() > dolores_core::MAX_TOOL_BYTES {
                encoded=json!({"sourceCallId":request.call_id,"originalBytes":original_bytes,"previewTruncated":true,"preview":text_prefix(original,2048),"historyNote":"Complete output is saved; retrieve it with read_task_history by callId."}).to_string();
            }
            result = if result.is_ok() {
                Ok(encoded)
            } else {
                Err(encoded)
            };
        }
        let content = match &result {
            Ok(text) | Err(text) if text.len() <= dolores_core::MAX_TOOL_BYTES => {
                Some(text.as_str())
            }
            _ => None,
        };
        self.log.record(None,"toolResult",json!({"callId":request.call_id,"name":request.name,"target":request.target,"returned":result.is_ok(),"content":content,"originalBytes":original_bytes,"fragmented":fragmented,"parts":if result.is_ok(){self.inner.image_results()}else{vec![]},"oversized":false,"note":"Returned errors do not establish that external effects were undone. Inspect Changes or command receipts."})).await.map_err(|_|"An operation returned but its run evidence could not be saved. Inspect Changes and external effects before retrying.")?;
        result
    }
}
fn text_prefix(text: &str, limit: usize) -> &str {
    let mut end = text.len().min(limit);
    while !text.is_char_boundary(end) {
        end -= 1;
    }
    &text[..end]
}
#[cfg(test)]
mod tests {
    use super::*;
    #[tokio::test]
    async fn oversized_tool_output_is_saved_once_and_exactly_retrievable_without_reexecution() {
        use dolores_core::ToolCall;
        struct Large;
        #[async_trait]
        impl ToolPlugin for Large {
            fn spec(&self) -> ToolSpec {
                ToolSpec {
                    name: "read_text_file".into(),
                    description: "Public fixture".into(),
                    parameters: json!({}),
                }
            }
            fn prepare(&self, _: &ToolCall) -> Result<ToolRequest, String> {
                unreachable!()
            }
            async fn invoke(
                &self,
                _: &ToolRequest,
                _: CancellationToken,
            ) -> Result<String, String> {
                Ok("Public 世界 \\\"\n".repeat(12000))
            }
        }
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("evidence.db");
        let store = Arc::new(dolores_store_sqlite::SqliteStore::open(&path).unwrap());
        store.create("chat").unwrap();
        let id = "11111111-1111-4111-8111-111111111111";
        let snapshot = dolores_core::RunSnapshot {
            parent_run: None,
            segments: 1,
            id: id.into(),
            thread: "chat".into(),
            model: "fixture".into(),
            settings: Default::default(),
            input: "Public fixture".into(),
            state: RunState::Prepared,
            sequence: 0,
            created_at: 0,
            build: "fixture".into(),
            tools: vec![],
            extensions: vec![],
            effective_settings: None,
        };
        store.begin_run(&snapshot).unwrap();
        let log = RunLog::new(store.clone(), id.into());
        log.record(Some(RunState::Running), "started", json!({}))
            .await
            .unwrap();
        let tool = LoggedTool {
            inner: Arc::new(Large),
            log,
            policy: None,
        };
        let request = ToolRequest {
            call_id: "large-read".into(),
            name: "read_text_file".into(),
            target: "public.txt".into(),
            query: None,
            diff: None,
            command: None,
            mcp: None,
        };
        let preview = tool
            .invoke(&request, CancellationToken::new())
            .await
            .unwrap();
        assert!(preview.len() <= dolores_core::MAX_TOOL_BYTES);
        assert_eq!(
            serde_json::from_str::<Value>(&preview).unwrap()["previewTruncated"],
            true
        );
        let reopened = dolores_store_sqlite::SqliteStore::open(&path).unwrap();
        let mut content = String::new();
        let mut offset = 0;
        loop {
            let page = reopened
                .read_task_history("chat", id, Some("large-read"), 0, offset)
                .unwrap();
            content.push_str(page["content"].as_str().unwrap());
            let next = page["nextOffset"].as_u64().unwrap() as usize;
            assert!(next > offset);
            offset = next;
            if page["hasMore"] == false {
                break;
            }
        }
        assert_eq!(content, "Public 世界 \\\"\n".repeat(12000));
        let events = reopened.run_events("chat", id).unwrap();
        assert_eq!(events.iter().filter(|e| e.kind == "toolIntent").count(), 1);
        assert_eq!(events.iter().filter(|e| e.kind == "toolResult").count(), 1);
        assert!(reopened
            .read_task_history("another", id, Some("large-read"), 0, 0)
            .is_err());
    }
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
    #[test]
    fn multiple_primary_owners_route_stop_and_poll_by_id_and_shutdown_cancels_all() {
        let store =
            Arc::new(dolores_store_sqlite::SqliteStore::open(Path::new(":memory:")).unwrap());
        store.create("A").unwrap();
        store.create("B").unwrap();
        let engine = crate::Engine::new(
            store,
            Arc::new(crate::connection::testing::MemoryCredentials::default()),
        )
        .unwrap();
        let a = CancellationToken::new();
        let b = CancellationToken::new();
        for (id, thread, cancel) in [(11, "A", a.clone()), (22, "B", b.clone())] {
            let (out, events) = tokio::sync::mpsc::channel(2);
            out.try_send(json!({"type":"delta","id":id,"text":thread}))
                .unwrap();
            engine
                .active
                .lock()
                .unwrap()
                .reserve_primary(
                    Run {
                        id,
                        thread: Some(thread.into()),
                        cancel,
                        events,
                        approvals: Arc::new(Mutex::new(None)),
                    },
                    thread.into(),
                )
                .unwrap();
        }
        assert_eq!(
            engine.call(crate::Command::Poll { id: 22 }).unwrap()[0]["text"],
            "B"
        );
        engine.call(crate::Command::Cancel { id: 22 }).unwrap();
        assert!(!a.is_cancelled());
        assert!(b.is_cancelled());
        assert_eq!(
            engine.call(crate::Command::Poll { id: 11 }).unwrap()[0]["text"],
            "A"
        );
        engine.call(crate::Command::Shutdown).unwrap();
        assert!(a.is_cancelled());
    }
}
