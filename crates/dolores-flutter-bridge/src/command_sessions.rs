use crate::{blocking, permissions::PermissionGuard, run_journal::RunLog, Engine};
use dolores_core::{SessionStore, ToolRecord};
use dolores_tools_command::sessions::{CommandSessions, SessionEvidence, SessionRecord};
use serde::Deserialize;
use serde_json::{json, Value};
use std::{
    collections::HashMap,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, Mutex,
    },
};
use tokio_util::sync::CancellationToken;

struct Evidence {
    store: Arc<dyn SessionStore>,
    revisions: Mutex<HashMap<String, u64>>,
}
impl SessionEvidence for Evidence {
    fn save(&self, record: &SessionRecord) -> Result<(), String> {
        record.validate()?;
        let mut revisions = self
            .revisions
            .lock()
            .map_err(|_| "Command evidence unavailable.")?;
        if revisions
            .get(&record.receipt.id)
            .is_some_and(|r| *r >= record.revision)
        {
            return Ok(());
        }
        self.store.save_command_session(record)?;
        revisions.insert(record.receipt.id.clone(), record.revision);
        Ok(())
    }
}
pub(super) fn evidence(store: Arc<dyn SessionStore>) -> Arc<dyn SessionEvidence> {
    Arc::new(Evidence {
        store,
        revisions: Mutex::new(HashMap::new()),
    })
}
pub(super) struct Live {
    pub commands: Arc<CommandSessions>,
    pub task: String,
    pub run: String,
    pub log: Arc<RunLog>,
    pub policy: Arc<PermissionGuard>,
    pub owner: CancellationToken,
    finished: AtomicBool,
    retained: AtomicBool,
    warning: Mutex<Option<String>>,
    created: std::time::Instant,
}
impl Live {
    pub fn new(
        commands: Arc<CommandSessions>,
        task: String,
        run: String,
        log: Arc<RunLog>,
        policy: Arc<PermissionGuard>,
        owner: CancellationToken,
    ) -> Arc<Self> {
        Arc::new(Self {
            commands,
            task,
            run,
            log,
            policy,
            owner,
            finished: AtomicBool::new(false),
            retained: AtomicBool::new(false),
            warning: Mutex::new(None),
            created: std::time::Instant::now(),
        })
    }
    fn save(&self) -> Result<(), String> {
        let result = self.commands.persist();
        *self.warning.lock().unwrap_or_else(|e| e.into_inner()) = result.as_ref().err().cloned();
        result
    }
    pub async fn finish(self: &Arc<Self>) -> Result<Vec<ToolRecord>, String> {
        let this = self.clone();
        let result=blocking(move||{
            this.commands.finish()?;
            this.finished.store(true,Ordering::SeqCst);
            this.save()?;
            this.commands.snapshots()?.into_iter().map(|s|Ok(ToolRecord {
                call_id:format!("host-end-{}",s.receipt.id),name:"command_session".into(),target:s.receipt.id.clone(),status:"completed".into(),
                content:json!({"session":s.receipt,"source":"host run-end cleanup; not a model invocation","untrusted":true}).to_string(),
                command:None,query:Some("{\"operation\":\"host_run_end\"}".into()),diff:None,mcp:None,parts:vec![],
            })).collect()
        }).await;
        result
    }
    pub async fn observe(self: Arc<Self>, stop: CancellationToken) {
        loop {
            tokio::select! {biased; _=stop.cancelled()=>break,_=self.owner.cancelled()=>break,
            _=tokio::time::sleep(std::time::Duration::from_secs(1))=>{}}
            let this = self.clone();
            if blocking(move || this.save()).await.is_err() {
                self.owner.cancel();
                break;
            }
        }
    }
}
#[derive(Default)]
pub(super) struct Registry(Mutex<HashMap<String, Arc<Live>>>);
impl Registry {
    pub fn admit(&self) -> Result<(), String> {
        if self
            .0
            .lock()
            .map_err(|_| "Command supervision unavailable.")?
            .values()
            .filter(|v| v.retained.load(Ordering::SeqCst))
            .count()
            >= 4
        {
            return Err("Four runs retain unsaved command evidence. Open Chat details → Commands, inspect output and Retry saving before starting more project work. Your draft remains.".into());
        }
        Ok(())
    }
    pub fn register(&self, live: Arc<Live>) {
        self.0
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .insert(live.run.clone(), live);
    }
    pub fn settle(&self, live: &Live, saved: bool) {
        if saved {
            self.0
                .lock()
                .unwrap_or_else(|e| e.into_inner())
                .remove(&live.run);
        } else {
            live.retained.store(true, Ordering::SeqCst);
        }
    }
    pub fn remove_task(&self, task: &str) {
        self.0
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .retain(|_, v| v.task != task);
    }
    fn get(&self, task: &str, run: &str) -> Result<Option<Arc<Live>>, String> {
        let found = self
            .0
            .lock()
            .map_err(|_| "Command supervision unavailable.")?
            .get(run)
            .cloned();
        if found.as_ref().is_some_and(|v| v.task != task) {
            return Err("Command run belongs to another chat. Nothing was sent or stopped.".into());
        }
        Ok(found)
    }
}
/// Unexpected future destruction must release the run's owned native jobs.
pub(super) struct Lifetime {
    pub live: Option<Arc<Live>>,
    pub armed: bool,
}
impl Drop for Lifetime {
    fn drop(&mut self) {
        if self.armed {
            if let Some(live) = &self.live {
                live.owner.cancel();
                let _ = live.commands.finish();
                let _ = live.save();
                live.finished.store(true, Ordering::SeqCst);
                live.retained.store(true, Ordering::SeqCst);
            }
        }
    }
}
#[derive(Deserialize)]
#[serde(tag = "operation", rename_all = "camelCase", deny_unknown_fields)]
pub(super) enum Request {
    RecoverIsolation,
    List {
        run: Option<String>,
        cursor: Option<i64>,
    },
    Read {
        run: String,
        id: String,
        #[serde(default)]
        offset: usize,
    },
    Input {
        run: String,
        id: String,
        revision: u64,
        text: String,
        #[serde(default)]
        eof: bool,
    },
    Interrupt {
        run: String,
        id: String,
    },
    Close {
        run: String,
        id: String,
    },
    Save {
        run: String,
    },
}
impl Engine {
    pub(super) fn command_session_call(
        &self,
        task: &str,
        request: Request,
    ) -> Result<Value, String> {
        match request {
            Request::RecoverIsolation => self.recover_isolation(),
            Request::List { run, cursor } => {
                // Unsaved live/finished evidence remains inspectable even when a
                // database failure prevented its first row from being committed.
                let mut live: Vec<_> = self
                    .command_sessions
                    .0
                    .lock()
                    .map_err(|_| "Command supervision unavailable.")?
                    .values()
                    .filter(|v| v.task == task && run.as_ref().is_none_or(|r| r == &v.run))
                    .cloned()
                    .collect();
                live.sort_by_key(|v| std::cmp::Reverse(v.created));
                let mut value = match self.store.command_sessions(task, run.as_deref(), cursor) {
                    Ok(page) => json!(page),
                    Err(error) if !live.is_empty() => {
                        json!({"items":[],"nextCursor":null,"historyError":error})
                    }
                    Err(error) => return Err(error),
                };
                let mut warnings = vec![];
                let owners: Vec<_> = live
                    .iter()
                    .map(|v| json!({"run":v.run,"unsaved":v.retained.load(Ordering::SeqCst)}))
                    .collect();
                if cursor.is_none() {
                    for owner in &live {
                        if let Some(w) = owner
                            .warning
                            .lock()
                            .unwrap_or_else(|e| e.into_inner())
                            .clone()
                        {
                            warnings.push(json!({"run":owner.run,"message":w}));
                        }
                    }
                }
                value["liveItems"] = json!(if cursor.is_none() {
                    live.first()
                        .map(|v| v.commands.receipts())
                        .transpose()?
                        .unwrap_or_default()
                } else {
                    vec![]
                });
                value["retainedRuns"] = json!(owners);
                value["storageWarnings"] = json!(warnings);
                value["isolation"] = self.isolation_status();
                Ok(value)
            }
            Request::Read { run, id, offset } => {
                if let Some(live) = self.command_sessions.get(task, &run)? {
                    if live.commands.receipts()?.iter().any(|r| r.id == id) {
                        let page = live.commands.inspect(&id, offset)?;
                        return Ok(
                            json!({"receipt":page["session"],"offset":page["offset"],"nextOffset":page["nextOffset"],"hasMore":page["hasMore"],"output":page["output"],"lossyUtf8":page["lossyUtf8"]}),
                        );
                    }
                }
                Ok(json!(self
                    .store
                    .command_session_output(task, &run, &id, offset)?))
            }
            Request::Input {
                run,
                id,
                revision,
                text,
                eof,
            } => {
                let live=self.command_sessions.get(task,&run)?.ok_or("Saved session cannot accept input. Inspect its output; no process was reattached.")?;
                live.policy.recheck()?;
                if text.len() > 4096 || text.is_empty() && !eof {
                    return Err("Input must contain 1–4096 UTF-8 bytes, or explicitly close stdin. Nothing was sent.".into());
                }
                let call = format!("human-{}", uuid::Uuid::new_v4());
                self.runtime.block_on(live.log.record(None,"toolIntent",json!({"callId":call,"name":"command_session","target":id,"arguments":{"operation":"input","session":id,"revision":revision,"text":text,"eof":eof},"source":"explicit human stdin Send","certainty":"Input may have occurred until its receipt is inspected."})))?;
                let value = live.commands.send_input(&id, revision, call, text, eof)?;
                live.save()?;
                Ok(value)
            }
            Request::Interrupt { run, id } => self.command_session_stop(task, &run, &id, false),
            Request::Close { run, id } => self.command_session_stop(task, &run, &id, true),
            Request::Save { run } => {
                let live = self
                    .command_sessions
                    .get(task, &run)?
                    .ok_or("No unsaved command owner remains. Refresh saved receipts.")?;
                live.save()?;
                if live.finished.load(Ordering::SeqCst) {
                    self.command_sessions.settle(&live, true);
                }
                Ok(json!({"saved":true}))
            }
        }
    }
    fn isolation_status(&self) -> Value {
        #[cfg(windows)]
        {
            let count = self
                .isolation_host
                .as_ref()
                .map(|h| h.pending_cleanup())
                .transpose();
            let error = count.as_ref().err().cloned().or_else(|| {
                self.isolation_error
                    .lock()
                    .unwrap_or_else(|e| e.into_inner())
                    .clone()
            });
            json!({"available":self.isolation_host.is_some(),"pendingCleanup":count.ok().flatten().unwrap_or(0),"error":error})
        }
        #[cfg(not(windows))]
        {
            json!({"available":false,"pendingCleanup":0,"error":"Isolation is supported on Windows only."})
        }
    }
    fn recover_isolation(&self) -> Result<Value, String> {
        #[cfg(windows)]
        {
            let host = self.isolation_host.as_ref().ok_or(
                "Isolation host unavailable. Restart Dolores to reopen its private cleanup state.",
            )?;
            let result = host.recover();
            *self
                .isolation_error
                .lock()
                .unwrap_or_else(|e| e.into_inner()) = result.as_ref().err().cloned();
            result?;
            Ok(self.isolation_status())
        }
        #[cfg(not(windows))]
        {
            Err("Isolation is supported on Windows only.".into())
        }
    }
    fn command_session_stop(
        &self,
        task: &str,
        run: &str,
        id: &str,
        close: bool,
    ) -> Result<Value, String> {
        let live=self.command_sessions.get(task,run)?.ok_or("Saved session has no live handle. Inspect retained output; nothing was reattached or repeated.")?;
        let value = live.commands.request_stop(id, close)?;
        live.save()?;
        Ok(value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use dolores_core::{
        RunSnapshot, RunState, SessionWorkspace, ToolCall, ToolPlugin, WorkspaceKind,
    };
    #[cfg(windows)]
    #[test]
    fn cleanup_is_explicit_and_retains_unexpected_files_without_replaying_work() {
        let mut f = fixture();
        let state = f._dir.path().join("isolation-state");
        f.engine.isolation_host =
            Some(dolores_tools_command::isolation::Host::open(&state).unwrap());
        let orphan = state.join(uuid::Uuid::new_v4().to_string());
        std::fs::create_dir(&orphan).unwrap();
        std::fs::write(orphan.join("revoked"), "").unwrap();
        std::fs::write(orphan.join("keep.txt"), "PUBLIC_RETAINED").unwrap();
        let list = || {
            f.engine.command_session_call(
                &f.live.task,
                Request::List {
                    run: None,
                    cursor: None,
                },
            )
        };
        assert_eq!(list().unwrap()["isolation"]["pendingCleanup"], 1);
        assert!(orphan.join("keep.txt").exists());
        assert!(f
            .engine
            .command_session_call(&f.live.task, Request::RecoverIsolation)
            .unwrap_err()
            .contains("Unexpected file"));
        assert_eq!(
            std::fs::read_to_string(orphan.join("keep.txt")).unwrap(),
            "PUBLIC_RETAINED"
        );
        std::fs::remove_file(orphan.join("keep.txt")).unwrap();
        let recovered = f
            .engine
            .command_session_call(&f.live.task, Request::RecoverIsolation)
            .unwrap();
        assert_eq!(recovered["pendingCleanup"], 0);
        assert!(recovered["error"].is_null());
        assert!(f.live.commands.receipts().unwrap().is_empty());
        assert_eq!(
            f.store.saved_draft(&f.live.task).unwrap(),
            "Keep the chat draft"
        );
    }
    struct Fixture {
        engine: Engine,
        store: Arc<dolores_store_sqlite::SqliteStore>,
        live: Arc<Live>,
        _dir: tempfile::TempDir,
    }
    fn fixture() -> Fixture {
        let dir = tempfile::tempdir().unwrap();
        let store = Arc::new(
            dolores_store_sqlite::SqliteStore::open(&dir.path().join("history.sqlite3")).unwrap(),
        );
        let task = uuid::Uuid::new_v4().to_string();
        let run = uuid::Uuid::new_v4().to_string();
        store
            .create_workspace_session(
                &task,
                &SessionWorkspace {
                    kind: WorkspaceKind::Project,
                    root: Some(dir.path().to_string_lossy().into_owned()),
                },
            )
            .unwrap();
        store.create("other-chat").unwrap();
        store.save_draft(&task, "Keep the chat draft").unwrap();
        let engine = Engine::new(
            store.clone(),
            Arc::new(crate::connection::testing::MemoryCredentials::default()),
        )
        .unwrap();
        store
            .begin_run(&RunSnapshot {
                id: run.clone(),
                thread: task.clone(),
                parent_run: None,
                segments: 1,
                model: "fixture".into(),
                settings: Default::default(),
                input: "public command fixture".into(),
                state: RunState::Prepared,
                sequence: 0,
                created_at: 0,
                build: "fixture".into(),
                tools: vec!["command_session".into()],
                extensions: vec![],
                effective_settings: None,
            })
            .unwrap();
        let log = RunLog::new(store.clone(), run.clone());
        engine
            .runtime
            .block_on(log.record(Some(RunState::Running), "started", json!({})))
            .unwrap();
        let owner = CancellationToken::new();
        let commands = Arc::new(
            CommandSessions::new(dir.path(), task.clone(), run.clone(), owner.clone())
                .unwrap()
                .with_evidence(evidence(store.clone())),
        );
        let policy = Arc::new(PermissionGuard {
            store: store.clone(),
            session: task.clone(),
            revision: 0,
            policy: Default::default(),
        });
        let live = Live::new(commands, task, run, log, policy, owner);
        engine.command_sessions.register(live.clone());
        Fixture {
            engine,
            store,
            live,
            _dir: dir,
        }
    }
    fn start(f: &Fixture) -> String {
        let tool = f
            .engine
            .extension_registry(Some(&f.live.task))
            .unwrap()
            .pin_tool(f.live.commands.clone())
            .unwrap();
        let logged = crate::run_journal::LoggedTool {
            inner: tool,
            log: f.live.log.clone(),
            policy: Some(f.live.policy.clone()),
        };
        let request=logged.prepare(&ToolCall {id:"start-public".into(),name:"command_session".into(),arguments:json!({"operation":"start","program":"node","args":["-e","process.stdin.on('data',b=>{process.stdout.write(b);require('fs').appendFileSync('received',b)});console.log('ready')"]}).to_string()}).unwrap();
        let value: Value = serde_json::from_str(
            &f.engine
                .runtime
                .block_on(logged.invoke(&request, f.live.owner.clone()))
                .unwrap(),
        )
        .unwrap();
        value["session"]["id"].as_str().unwrap().into()
    }
    fn read(f: &Fixture, id: &str) -> Value {
        f.engine
            .call(crate::Command::SessionControl {
                session: f.live.task.clone(),
                request: Request::Read {
                    run: f.live.run.clone(),
                    id: id.into(),
                    offset: 0,
                },
            })
            .unwrap()
    }
    fn until(f: &Fixture, id: &str, check: impl Fn(&Value) -> bool) -> Value {
        let end = std::time::Instant::now() + std::time::Duration::from_secs(6);
        loop {
            let value = read(f, id);
            if check(&value) {
                return value;
            }
            assert!(std::time::Instant::now() < end, "{value}");
            std::thread::sleep(std::time::Duration::from_millis(10));
        }
    }
    #[test]
    fn human_input_uses_exact_owner_and_persisted_output_survives_handle_release() {
        let f = fixture();
        let id = start(&f);
        until(&f, &id, |v| v["output"].as_str().unwrap().contains("ready"));
        let wrong = f.engine.call(crate::Command::SessionControl {
            session: "other-chat".into(),
            request: Request::Input {
                run: f.live.run.clone(),
                id: id.clone(),
                revision: 0,
                text: "wrong".into(),
                eof: false,
            },
        });
        assert!(wrong.unwrap_err().contains("another chat"));
        f.engine
            .call(crate::Command::SessionControl {
                session: f.live.task.clone(),
                request: Request::Input {
                    run: f.live.run.clone(),
                    id: id.clone(),
                    revision: 0,
                    text: "public 世界\n".into(),
                    eof: false,
                },
            })
            .unwrap();
        until(&f, &id, |v| {
            v["output"].as_str().unwrap().contains("public 世界")
                // Echo precedes the fixture's append. Pipe delivery/output is
                // not an acknowledgement of the later filesystem effect.
                && std::fs::read_to_string(f._dir.path().join("received"))
                    .is_ok_and(|text| text == "public 世界\n")
        });
        assert_eq!(
            std::fs::read_to_string(f._dir.path().join("received")).unwrap(),
            "public 世界\n"
        );
        let stale = f.engine.call(crate::Command::SessionControl {
            session: f.live.task.clone(),
            request: Request::Input {
                run: f.live.run.clone(),
                id: id.clone(),
                revision: 0,
                text: "duplicate".into(),
                eof: false,
            },
        });
        assert!(stale.unwrap_err().contains("Input changed"));
        f.engine
            .call(crate::Command::SessionControl {
                session: f.live.task.clone(),
                request: Request::Interrupt {
                    run: f.live.run.clone(),
                    id: id.clone(),
                },
            })
            .unwrap();
        until(&f, &id, |v| v["receipt"]["cleanupComplete"] == true);
        let ended = f.engine.runtime.block_on(f.live.finish()).unwrap();
        assert_eq!(ended.len(), 1);
        f.engine.command_sessions.settle(&f.live, true);
        let saved = read(&f, &id);
        assert_eq!(saved["receipt"]["state"], "interrupted");
        assert!(saved["output"].as_str().unwrap().contains("public 世界"));
        assert!(f
            .engine
            .call(crate::Command::SessionControl {
                session: f.live.task.clone(),
                request: Request::Input {
                    run: f.live.run.clone(),
                    id: id.clone(),
                    revision: 1,
                    text: "after end".into(),
                    eof: false
                }
            })
            .is_err());
        assert_eq!(
            f.store.saved_draft(&f.live.task).unwrap(),
            "Keep the chat draft"
        );
        assert!(f
            .store
            .run_events(&f.live.task, &f.live.run)
            .unwrap()
            .iter()
            .any(|e| e.kind == "toolIntent" && e.data["source"] == "explicit human stdin Send"));
    }
    #[test]
    fn failed_evidence_stops_owner_keeps_memory_output_and_explicit_save_recovers() {
        let f = fixture();
        let id = start(&f);
        until(&f, &id, |v| v["output"].as_str().unwrap().contains("ready"));
        f.live.save().unwrap();
        rusqlite::Connection::open(f._dir.path().join("history.sqlite3")).unwrap().execute_batch("CREATE TRIGGER reject_command BEFORE UPDATE ON command_sessions BEGIN SELECT RAISE(ABORT,'fixture evidence unavailable');END;").unwrap();
        let result = f.engine.call(crate::Command::SessionControl {
            session: f.live.task.clone(),
            request: Request::Input {
                run: f.live.run.clone(),
                id: id.clone(),
                revision: 0,
                text: "public uncertain input\n".into(),
                eof: false,
            },
        });
        assert!(result.unwrap_err().contains("evidence could not be saved"));
        assert!(f.live.owner.is_cancelled());
        until(&f, &id, |v| v["receipt"]["cleanupComplete"] == true);
        assert!(f.engine.runtime.block_on(f.live.finish()).is_err());
        f.engine.command_sessions.settle(&f.live, false);
        let list = f
            .engine
            .command_session_call(
                &f.live.task,
                Request::List {
                    run: Some(f.live.run.clone()),
                    cursor: None,
                },
            )
            .unwrap();
        assert!(!list["storageWarnings"].as_array().unwrap().is_empty());
        assert!(read(&f, &id)["output"].as_str().unwrap().contains("ready"));
        rusqlite::Connection::open(f._dir.path().join("history.sqlite3"))
            .unwrap()
            .execute_batch("DROP TRIGGER reject_command")
            .unwrap();
        f.engine
            .command_session_call(
                &f.live.task,
                Request::Save {
                    run: f.live.run.clone(),
                },
            )
            .unwrap();
        assert!(read(&f, &id)["output"].as_str().unwrap().contains("ready"));
        assert!(f
            .engine
            .command_sessions
            .get(&f.live.task, &f.live.run)
            .unwrap()
            .is_none());
    }
}
