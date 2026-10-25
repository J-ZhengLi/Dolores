use super::*;
use dolores_store_sqlite::SqliteStore;
fn fixture() -> (tempfile::TempDir, Arc<SqliteStore>, Engine, String) {
    let (directory, store, root) = store_fixture();
    let engine = Engine::new(
        store.clone(),
        Arc::new(crate::connection::testing::MemoryCredentials::default()),
    )
    .unwrap();
    (directory, store, engine, root)
}
fn store_fixture() -> (tempfile::TempDir, Arc<SqliteStore>, String) {
    let directory = tempfile::tempdir().unwrap();
    let root = directory
        .path()
        .canonicalize()
        .unwrap()
        .to_str()
        .unwrap()
        .to_owned();
    let store = Arc::new(SqliteStore::open(&directory.path().join("state.db")).unwrap());
    store
        .create_workspace_session(
            "work",
            &dolores_core::SessionWorkspace {
                kind: dolores_core::WorkspaceKind::Project,
                root: Some(root.clone()),
            },
        )
        .unwrap();
    (directory, store, root)
}
fn hook(event: Event, mode: Mode) -> Hook {
    Hook {
        name: "check".into(),
        event,
        mode,
        program: "node".into(),
        script: "hook.cjs".into(),
        args: vec![],
        timeout_seconds: 1,
    }
}
#[test]
fn review_pins_script_and_failed_save_stale_identity_and_disable_recover() {
    let (directory, store, engine, root) = fixture();
    std::fs::write(
        directory.path().join("hook.cjs"),
        "throw Error('must not run during review')",
    )
    .unwrap();
    let active = RunCoordinator::default();
    let review = engine
        .hooks_call(
            "work",
            Request::Review {
                revision: 0,
                hooks: vec![hook(Event::BeforeTool, Mode::Blocking)],
            },
            &active,
        )
        .unwrap();
    assert!(store.lifecycle_hooks(&root).unwrap().hooks.is_empty());
    let token = review["token"].as_str().unwrap().to_owned();
    let db = rusqlite::Connection::open(directory.path().join("state.db")).unwrap();
    db.execute_batch("CREATE TRIGGER refuse_hook BEFORE INSERT ON lifecycle_hooks BEGIN SELECT RAISE(ABORT,'public fixture'); END;").unwrap();
    assert!(engine
        .hooks_call(
            "work",
            Request::Enable {
                token: token.clone()
            },
            &active
        )
        .is_err());
    assert!(engine.hook_review.lock().unwrap().is_some());
    db.execute_batch("DROP TRIGGER refuse_hook").unwrap();
    std::fs::write(directory.path().join("hook.cjs"), "console.log('changed')").unwrap();
    assert!(engine
        .hooks_call("work", Request::Enable { token }, &active)
        .is_err());
    let review = engine
        .hooks_call(
            "work",
            Request::Review {
                revision: 0,
                hooks: vec![hook(Event::BeforeTool, Mode::Blocking)],
            },
            &active,
        )
        .unwrap();
    let saved = engine
        .hooks_call(
            "work",
            Request::Enable {
                token: review["token"].as_str().unwrap().into(),
            },
            &active,
        )
        .unwrap();
    assert_eq!(saved["configuration"]["revision"], 1);
    // Native workspace roots are plain Windows paths, while hook scope is
    // canonical (verbatim on Windows). Startup must use the same scope resolver.
    let plain = directory.path().to_str().unwrap().to_owned();
    store
        .create_workspace_session(
            "plain",
            &dolores_core::SessionWorkspace {
                kind: dolores_core::WorkspaceKind::Project,
                root: Some(plain),
            },
        )
        .unwrap();
    let (output, _events) = tokio::sync::mpsc::channel(1);
    assert!(Runtime::for_session(
        store.as_ref(),
        "plain",
        RunLog::new(store.clone(), "unused".into()),
        Arc::new(Allow),
        output,
        1
    )
    .unwrap()
    .is_some());
    let stale = engine
        .hooks_call(
            "work",
            Request::Review {
                revision: 1,
                hooks: vec![hook(Event::BeforeTool, Mode::Blocking)],
            },
            &active,
        )
        .unwrap();
    engine.hook_review.lock().unwrap().as_mut().unwrap().created =
        Instant::now() - Duration::from_secs(301);
    assert!(engine
        .hooks_call(
            "work",
            Request::Enable {
                token: stale["token"].as_str().unwrap().into()
            },
            &active
        )
        .is_err());
    let mut busy = RunCoordinator::default();
    let (_output, events) = tokio::sync::mpsc::channel(1);
    busy.reserve(crate::Run {
        thread: None,
        id: 99,
        cancel: CancellationToken::new(),
        events,
        approvals: Arc::default(),
    })
    .unwrap();
    assert!(engine
        .hooks_call("work", Request::Disable { revision: 1 }, &busy)
        .is_err());
    assert!(engine.hooks_call("work", Request::View, &busy).is_ok());
    std::fs::remove_file(directory.path().join("hook.cjs")).unwrap();
    engine
        .hooks_call("work", Request::Disable { revision: 1 }, &active)
        .unwrap();
    assert!(store.lifecycle_hooks(&root).unwrap().hooks.is_empty());
}

struct Allow;
#[tokio::test]
async fn child_hook_receipts_share_serial_execution_and_parent_allowance() {
    let (directory, store, root) = store_fixture();
    std::fs::write(directory.path().join("hook.cjs"),"const fs=require('fs');const lock=fs.openSync('hook.lock','wx');setTimeout(()=>{fs.closeSync(lock);fs.unlinkSync('hook.lock');console.log(JSON.stringify({allow:true,message:'PUBLIC_SERIAL'}));},10);").unwrap();
    let saved = inspect(Path::new(&root), hook(Event::BeforeTool, Mode::Blocking)).unwrap();
    let (hooks, _events, run) = runtime(store.clone(), &root, vec![saved]);
    for _ in 0..12 {
        let (a, b) = tokio::join!(
            hooks.fire_owned(
                Event::BeforeTool,
                Some("read_text_file"),
                None,
                Some("first"),
                CancellationToken::new()
            ),
            hooks.fire_owned(
                Event::BeforeTool,
                Some("read_text_file"),
                None,
                Some("second"),
                CancellationToken::new()
            )
        );
        a.unwrap();
        b.unwrap();
    }
    assert!(hooks
        .fire(Event::BeforeTool, None, None, CancellationToken::new())
        .await
        .is_err());
    let events = store.run_events("work", &run).unwrap();
    assert_eq!(events.iter().filter(|e| e.kind == "hookIntent").count(), 24);
    assert!(events
        .iter()
        .filter(|e| e.kind == "hookIntent")
        .all(|e| e.data["childId"].is_string()));
}
#[async_trait::async_trait]
impl ToolApproval for Allow {
    async fn authorize(&self, _: &ToolRequest, _: CancellationToken) -> Result<bool, String> {
        Ok(true)
    }
}
fn runtime(
    store: Arc<SqliteStore>,
    root: &str,
    hooks: Vec<ReviewedHook>,
) -> (Arc<Runtime>, tokio::sync::mpsc::Receiver<Value>, String) {
    let id = uuid::Uuid::new_v4().to_string();
    store
        .begin_run(&dolores_core::RunSnapshot {
            parent_run: None,
            segments: 1,
            id: id.clone(),
            thread: "work".into(),
            model: "fixture".into(),
            settings: Default::default(),
            input: "public hook fixture".into(),
            state: dolores_core::RunState::Prepared,
            sequence: 0,
            created_at: 1,
            build: "fixture".into(),
            tools: vec![],
            extensions: vec![],
            effective_settings: None,
        })
        .unwrap();
    let log = RunLog::new(store, id.clone());
    let (output, events) = tokio::sync::mpsc::channel(64);
    (
        Runtime::new(
            root.into(),
            Configuration { revision: 1, hooks },
            log,
            Arc::new(Allow),
            output,
            1,
        )
        .unwrap()
        .unwrap(),
        events,
        id,
    )
}
struct Effect(Arc<std::sync::atomic::AtomicUsize>);
#[async_trait::async_trait]
impl ToolPlugin for Effect {
    fn spec(&self) -> dolores_core::ToolSpec {
        dolores_core::ToolSpec {
            name: "effect".into(),
            description: "public fixture".into(),
            parameters: json!({}),
        }
    }
    fn prepare(&self, c: &ToolCall) -> Result<ToolRequest, String> {
        Ok(ToolRequest {
            call_id: c.id.clone(),
            name: "effect".into(),
            target: "public marker".into(),
            query: None,
            diff: None,
            command: None,
            mcp: None,
        })
    }
    async fn invoke(&self, _: &ToolRequest, _: CancellationToken) -> Result<String, String> {
        self.0.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        Ok("PUBLIC_TOOL_RESULT".into())
    }
}
#[tokio::test]
async fn blocking_refusal_prevents_effect_and_observation_failure_preserves_result() {
    let (directory, store, root) = store_fixture();
    let path = directory.path().join("hook.cjs");
    std::fs::write(
        &path,
        "console.log(JSON.stringify({allow:false,message:'PUBLIC_CHECK_REFUSED'}))",
    )
    .unwrap();
    let saved = inspect(Path::new(&root), hook(Event::BeforeTool, Mode::Blocking)).unwrap();
    let (hooks, mut events, run) = runtime(store.clone(), &root, vec![saved]);
    let calls = Arc::new(std::sync::atomic::AtomicUsize::new(0));
    let plugin = HookedTool {
        inner: Arc::new(Effect(calls.clone())),
        hooks,
    };
    let request = plugin
        .prepare(&ToolCall {
            id: "one".into(),
            name: "effect".into(),
            arguments: "{}".into(),
        })
        .unwrap();
    assert!(plugin
        .invoke(&request, CancellationToken::new())
        .await
        .err()
        .unwrap()
        .contains("refused"));
    assert_eq!(calls.load(std::sync::atomic::Ordering::SeqCst), 0);
    assert_eq!(events.try_recv().unwrap()["receipt"]["status"], "blocked");
    let receipts = store.run_events("work", &run).unwrap();
    assert_eq!(
        receipts.iter().filter(|r| r.kind == "hookIntent").count(),
        1
    );
    std::fs::write(&path, "console.log('malformed output')").unwrap();
    let saved = inspect(Path::new(&root), hook(Event::AfterTool, Mode::Observe)).unwrap();
    let (hooks, mut events, _) = runtime(store.clone(), &root, vec![saved]);
    let plugin = HookedTool {
        inner: Arc::new(Effect(calls.clone())),
        hooks,
    };
    assert_eq!(
        plugin
            .invoke(&request, CancellationToken::new())
            .await
            .unwrap(),
        "PUBLIC_TOOL_RESULT"
    );
    assert_eq!(events.try_recv().unwrap()["receipt"]["status"], "error");
    assert_eq!(calls.load(std::sync::atomic::Ordering::SeqCst), 1);
}
#[tokio::test]
async fn timeout_output_flood_bad_json_and_changed_scripts_refuse_without_replay() {
    let (directory, store, root) = store_fixture();
    let path = directory.path().join("hook.cjs");
    for (source, change) in [
        ("console.log('not JSON')", false),
        ("process.stdout.write('x'.repeat(10000))", false),
        (
            "console.log(JSON.stringify({message:'missing allow'}))",
            false,
        ),
        (
            "console.log(JSON.stringify({allow:true,message:'ok',capabilities:['all']}))",
            false,
        ),
        ("setInterval(()=>{},100)", false),
        (
            "console.log(JSON.stringify({allow:true,message:'ok'}))",
            true,
        ),
    ] {
        std::fs::write(&path, source).unwrap();
        let saved = inspect(Path::new(&root), hook(Event::BeforeTool, Mode::Blocking)).unwrap();
        let (hooks, mut events, run) = runtime(store.clone(), &root, vec![saved]);
        if change {
            std::fs::write(&path, "throw Error('changed')").unwrap();
        }
        assert!(hooks
            .fire(
                Event::BeforeTool,
                Some("effect"),
                None,
                CancellationToken::new()
            )
            .await
            .is_err());
        assert_eq!(events.try_recv().unwrap()["receipt"]["status"], "error");
        assert_eq!(
            store
                .run_events("work", &run)
                .unwrap()
                .iter()
                .filter(|r| r.kind == "hookIntent")
                .count(),
            usize::from(!change)
        );
    }
}
#[tokio::test]
async fn stop_cleans_owned_command_and_budget_and_compaction_are_bounded() {
    let (directory, store, root) = store_fixture();
    let path = directory.path().join("hook.cjs");
    std::fs::write(
        &path,
        "require('fs').writeFileSync('hook-started.txt','PUBLIC');setInterval(()=>{},100)",
    )
    .unwrap();
    let saved = inspect(Path::new(&root), hook(Event::TaskStart, Mode::Blocking)).unwrap();
    let (hooks, _events, _) = runtime(store.clone(), &root, vec![saved]);
    let cancel = CancellationToken::new();
    let token = cancel.clone();
    let task = tokio::spawn(async move { hooks.fire(Event::TaskStart, None, None, token).await });
    tokio::time::timeout(Duration::from_secs(5), async {
        while !directory.path().join("hook-started.txt").exists() {
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
    })
    .await
    .unwrap();
    cancel.cancel();
    assert!(task.await.unwrap().is_err());
    std::fs::write(
        &path,
        "console.log(JSON.stringify({allow:true,message:'ok'}))",
    )
    .unwrap();
    let saved = inspect(
        Path::new(&root),
        hook(Event::BeforeCompaction, Mode::Blocking),
    )
    .unwrap();
    let (hooks, _events, run) = runtime(store.clone(), &root, vec![saved]);
    for _ in 0..24 {
        hooks
            .fire(
                Event::BeforeCompaction,
                None,
                None,
                CancellationToken::new(),
            )
            .await
            .unwrap();
    }
    assert!(hooks
        .fire(
            Event::BeforeCompaction,
            None,
            None,
            CancellationToken::new()
        )
        .await
        .is_err());
    assert_eq!(
        store
            .run_events("work", &run)
            .unwrap()
            .iter()
            .filter(|r| r.kind == "hookIntent")
            .count(),
        24
    );
    // Saved state remains independent of run receipts; reopening is inspection only.
    assert!(store.lifecycle_hooks(&root).unwrap().hooks.is_empty());
}

struct Compact(Arc<std::sync::atomic::AtomicUsize>);
#[async_trait::async_trait]
impl ToolApproval for Compact {
    fn context_recovery_available(&self) -> bool {
        true
    }
    async fn authorize(&self, _: &ToolRequest, _: CancellationToken) -> Result<bool, String> {
        Ok(true)
    }
    async fn compact_context(
        &self,
        _: &[dolores_core::AgentMessage],
        _: CancellationToken,
    ) -> Result<Option<dolores_core::context_recovery::ContextCheckpoint>, String> {
        self.0.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        Ok(None)
    }
}
#[tokio::test]
async fn compaction_gate_and_cumulative_time_refuse_before_work_but_observers_cannot_deny() {
    let (directory, store, root) = store_fixture();
    let path = directory.path().join("hook.cjs");
    std::fs::write(
        &path,
        "console.log(JSON.stringify({allow:false,message:'no'}))",
    )
    .unwrap();
    let saved = inspect(
        Path::new(&root),
        hook(Event::BeforeCompaction, Mode::Blocking),
    )
    .unwrap();
    let (hooks, _events, _) = runtime(store.clone(), &root, vec![saved]);
    let calls = Arc::new(std::sync::atomic::AtomicUsize::new(0));
    let approval = HookedApproval {
        inner: Arc::new(Compact(calls.clone())),
        hooks,
    };
    assert!(approval
        .compact_context(&[], CancellationToken::new())
        .await
        .is_err());
    assert_eq!(calls.load(std::sync::atomic::Ordering::SeqCst), 0);
    let mut observer = hook(Event::BeforeCompaction, Mode::Observe);
    observer.timeout_seconds = 10;
    let saved = inspect(Path::new(&root), observer).unwrap();
    let (hooks, mut events, run) = runtime(store.clone(), &root, vec![saved]);
    for _ in 0..4 {
        hooks
            .fire(
                Event::BeforeCompaction,
                None,
                None,
                CancellationToken::new(),
            )
            .await
            .unwrap();
    }
    assert_eq!(
        store
            .run_events("work", &run)
            .unwrap()
            .iter()
            .filter(|r| r.kind == "hookIntent")
            .count(),
        3
    );
    for _ in 0..3 {
        assert_eq!(events.try_recv().unwrap()["receipt"]["status"], "completed");
    }
    assert_eq!(events.try_recv().unwrap()["receipt"]["status"], "error");
}
