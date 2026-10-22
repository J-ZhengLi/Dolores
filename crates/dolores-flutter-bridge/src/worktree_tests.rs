use super::*;
use crate::{git_process, Command};
use dolores_core::{SessionStore, SessionWorkspace, WorkspaceKind};
use dolores_store_sqlite::SqliteStore;
use std::path::{Path, PathBuf};

struct Fixture {
    _dir: tempfile::TempDir,
    root: PathBuf,
    store: Arc<SqliteStore>,
    engine: Engine,
    source: String,
}

fn git(root: &Path, args: &[&str]) -> String {
    let exe = git_process::executable(root).unwrap();
    let result = git_process::run(
        &exe,
        root,
        &args.iter().map(|v| v.to_string()).collect::<Vec<_>>(),
        &CancellationToken::new(),
        false,
    )
    .unwrap();
    assert_eq!(
        result.code,
        0,
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    String::from_utf8(result.stdout).unwrap().trim().into()
}

impl Fixture {
    fn new() -> Self {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path().join("project");
        std::fs::create_dir(&root).unwrap();
        git(&root, &["init", "-b", "main"]);
        git(&root, &["config", "user.name", "Fixture"]);
        git(&root, &["config", "user.email", "fixture@example.invalid"]);
        git(&root, &["config", "commit.gpgSign", "false"]);
        git(&root, &["config", "core.autocrlf", "false"]);
        git(
            &root,
            &[
                "config",
                "core.hooksPath",
                root.join(".git/hooks").to_str().unwrap(),
            ],
        );
        std::fs::write(root.join("note.txt"), "base\n").unwrap();
        std::fs::write(root.join("filtered.txt"), "original filter input\n").unwrap();
        std::fs::write(root.join(".gitattributes"), "filtered.txt filter=fixture\n").unwrap();
        git(&root, &["add", "."]);
        git(&root, &["commit", "-m", "Public fixture"]);
        let data = dir.path().join("data");
        std::fs::create_dir(&data).unwrap();
        let store = Arc::new(SqliteStore::open(&data.join("state.db")).unwrap());
        let source = uuid::Uuid::new_v4().to_string();
        store
            .create_workspace_session(
                &source,
                &SessionWorkspace {
                    kind: WorkspaceKind::Project,
                    root: Some(crate::workspace::canonical_folder(&root).unwrap()),
                },
            )
            .unwrap();
        let mut engine = Engine::new(
            store.clone(),
            Arc::new(crate::connection::testing::MemoryCredentials::default()),
        )
        .unwrap();
        engine.workspace_directory = Some(data.join("workspaces"));
        Self {
            _dir: dir,
            root,
            store,
            engine,
            source,
        }
    }
    fn call(&self, request: Request) -> Result<Value, String> {
        self.engine.call(Command::Worktrees {
            session: self.source.clone(),
            request,
        })
    }
    fn complete(&self, request: Request) -> Result<Value, String> {
        let value = self.call(request)?;
        let id = value["job"].as_str().unwrap().to_owned();
        let started = Instant::now();
        loop {
            let value = self.call(Request::Poll { job: id.clone() })?;
            if value["done"] == true {
                return Ok(value["value"].clone());
            }
            assert!(
                started.elapsed() < Duration::from_secs(45),
                "worktree fixture job did not finish"
            );
            std::thread::sleep(Duration::from_millis(10));
        }
    }
    fn review(&self) -> Value {
        self.complete(Request::ReviewCreate {
            base: "main".into(),
            setup: WorktreeSetup::Manual,
        })
        .unwrap()
    }
    fn create(&self) -> ManagedWorktree {
        let review = self.review();
        serde_json::from_value(
            self.complete(Request::Create {
                token: review["token"].as_str().unwrap().into(),
            })
            .unwrap()["record"]
                .clone(),
        )
        .unwrap()
    }
}

#[test]
fn managed_worktree_creation_uses_reviewed_commit_and_preserves_original_dirty_work() {
    let f = Fixture::new();
    std::fs::write(f.root.join("note.txt"), "staged source\n").unwrap();
    git(&f.root, &["add", "note.txt"]);
    std::fs::write(f.root.join("note.txt"), "unstaged source\n").unwrap();
    std::fs::write(f.root.join("untracked.txt"), "keep\n").unwrap();
    let index = std::fs::read(f.root.join(".git/index")).unwrap();
    let head = git(&f.root, &["rev-parse", "HEAD"]);
    let first = f.create();
    let second = f.create();
    assert_ne!(first.root, second.root);
    assert_eq!(first.state, WorktreeState::Ready);
    assert_eq!(git(Path::new(&first.root), &["rev-parse", "HEAD"]), head);
    assert_eq!(
        std::fs::read_to_string(Path::new(&first.root).join("note.txt")).unwrap(),
        "base\n"
    );
    std::fs::write(Path::new(&first.root).join("note.txt"), "first task only\n").unwrap();
    assert_eq!(
        std::fs::read_to_string(Path::new(&second.root).join("note.txt")).unwrap(),
        "base\n"
    );
    assert_eq!(
        std::fs::read_to_string(f.root.join("note.txt")).unwrap(),
        "unstaged source\n"
    );
    assert_eq!(std::fs::read(f.root.join(".git/index")).unwrap(), index);
    assert_eq!(
        std::fs::read_to_string(f.root.join("untracked.txt")).unwrap(),
        "keep\n"
    );
    assert_eq!(
        f.store.workspace(&first.session).unwrap().root,
        Some(first.root.clone())
    );
    f.engine.require_worktree_ready(&first.session).unwrap();
    assert!(git(&f.root, &["worktree", "list", "--porcelain"])
        .contains("locked Dolores managed checkout"));
}

#[test]
fn managed_worktree_checkout_skips_repository_hooks_and_filters() {
    let f = Fixture::new();
    let marker = f.root.join("hook-ran.txt");
    let hook = format!(
        "#!/bin/sh\nprintf ran > '{}'\n",
        marker.to_string_lossy().replace('\\', "/")
    );
    std::fs::write(f.root.join(".git/hooks/post-checkout"), hook).unwrap();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(
            f.root.join(".git/hooks/post-checkout"),
            std::fs::Permissions::from_mode(0o755),
        )
        .unwrap();
    }
    git(
        &f.root,
        &[
            "config",
            "filter.fixture.process",
            "DOLORES_MUST_NOT_EXECUTE_FILTER",
        ],
    );
    git(&f.root, &["config", "filter.fixture.required", "true"]);
    let record = f.create();
    assert!(!marker.exists());
    assert_eq!(
        std::fs::read_to_string(Path::new(&record.root).join("filtered.txt")).unwrap(),
        "original filter input\n"
    );
}

#[test]
fn managed_worktree_changed_config_and_wrong_chat_cannot_consume_authority() {
    let f = Fixture::new();
    let review = f.review();
    let token = review["token"].as_str().unwrap().to_owned();
    let error = f
        .engine
        .call(Command::Worktrees {
            session: "wrong-chat".into(),
            request: Request::Create {
                token: token.clone(),
            },
        })
        .unwrap_err();
    assert!(error.contains("another chat"));
    git(&f.root, &["config", "core.abbrev", "12"]);
    let error = f
        .complete(Request::Create {
            token: token.clone(),
        })
        .unwrap_err();
    assert!(error.contains("configuration changed"), "{error}");
    assert!(!Path::new(review["record"]["root"].as_str().unwrap()).exists());
    assert!(f.store.worktrees(None).unwrap().items.is_empty());
    assert!(f.call(Request::Create { token }).is_err());
}

#[test]
fn managed_worktree_existing_destination_and_replaced_root_keep_both_versions() {
    let f = Fixture::new();
    let review = f.review();
    let target = Path::new(review["record"]["root"].as_str().unwrap());
    std::fs::create_dir(target).unwrap();
    std::fs::write(target.join("foreign.txt"), "retain foreign data").unwrap();
    let error = f
        .complete(Request::Create {
            token: review["token"].as_str().unwrap().into(),
        })
        .unwrap_err();
    assert!(error.contains("not overwritten"), "{error}");
    let retained = f
        .store
        .worktree(review["record"]["session"].as_str().unwrap())
        .unwrap()
        .unwrap();
    assert_eq!(retained.state, WorktreeState::Recovery);
    assert!(f.engine.require_worktree_ready(&retained.session).is_err());
    assert_eq!(
        std::fs::read_to_string(target.join("foreign.txt")).unwrap(),
        "retain foreign data"
    );
    let record = f.create();
    let saved = PathBuf::from(format!("{}-retained", record.root));
    std::fs::rename(&record.root, &saved).unwrap();
    std::fs::create_dir(&record.root).unwrap();
    std::fs::write(Path::new(&record.root).join("foreign.txt"), "replacement").unwrap();
    assert!(f
        .engine
        .require_worktree_ready(&record.session)
        .unwrap_err()
        .contains("replaced"));
    assert!(saved.join("note.txt").exists());
    assert!(Path::new(&record.root).join("foreign.txt").exists());
}

#[test]
fn managed_worktree_cancelled_creation_never_executes_or_replays_git() {
    let f = Fixture::new();
    let review = f.review();
    let token = review["token"].as_str().unwrap();
    let registry = f.engine.worktrees.lock().unwrap();
    let review = registry.reviews.get(token).unwrap();
    let cancel = CancellationToken::new();
    cancel.cancel();
    assert!(worktree_git::create(f.store.as_ref(), review, &cancel).is_err());
    assert!(!Path::new(&review.record.root).exists());
    assert!(f.store.worktree(&review.record.session).unwrap().is_none());
}

#[test]
fn managed_worktree_failed_ready_save_recovers_existing_checkout_without_repeating_setup() {
    let f = Fixture::new();
    let db = rusqlite::Connection::open(f._dir.path().join("data/state.db")).unwrap();
    db.execute_batch("CREATE TRIGGER fail_ready BEFORE UPDATE ON managed_worktrees WHEN NEW.state='ready' BEGIN SELECT RAISE(ABORT,'fixture storage failure'); END;").unwrap();
    let review = f.review();
    let owner = review["record"]["session"].as_str().unwrap();
    assert!(f
        .complete(Request::Create {
            token: review["token"].as_str().unwrap().into()
        })
        .is_err());
    let saved = f.store.worktree(owner).unwrap().unwrap();
    assert_eq!(saved.state, WorktreeState::Recovery);
    assert!(Path::new(&saved.root).join(".git").is_file());
    std::fs::write(
        Path::new(&saved.root).join("retained.txt"),
        "manual work after interrupted save",
    )
    .unwrap();
    let index = std::fs::read(
        Path::new(&saved.common_directory)
            .join("worktrees")
            .join(&saved.id)
            .join("index"),
    )
    .unwrap();
    db.execute_batch("DROP TRIGGER fail_ready;").unwrap();
    let reopened = Engine::new(
        f.store.clone(),
        Arc::new(crate::connection::testing::MemoryCredentials::default()),
    )
    .unwrap();
    assert!(reopened.require_worktree_ready(owner).is_err());
    let value = reopened
        .call(Command::Worktrees {
            session: owner.into(),
            request: Request::ReconcileCreation {
                revision: saved.revision,
            },
        })
        .unwrap();
    let job = value["job"].as_str().unwrap();
    let started = Instant::now();
    loop {
        let value = reopened
            .call(Command::Worktrees {
                session: owner.into(),
                request: Request::Poll { job: job.into() },
            })
            .unwrap();
        if value["done"] == true {
            break;
        }
        assert!(started.elapsed() < Duration::from_secs(10));
        std::thread::sleep(Duration::from_millis(10));
    }
    reopened.require_worktree_ready(owner).unwrap();
    assert_eq!(
        std::fs::read_to_string(Path::new(&saved.root).join("retained.txt")).unwrap(),
        "manual work after interrupted save"
    );
    assert_eq!(
        std::fs::read(
            Path::new(&saved.common_directory)
                .join("worktrees")
                .join(&saved.id)
                .join("index")
        )
        .unwrap(),
        index
    );
    assert_eq!(
        git(&f.root, &["worktree", "list", "--porcelain"])
            .matches("worktree ")
            .count(),
        2
    );
}

#[test]
fn managed_worktree_git_pointer_redirection_and_repository_jobs_are_refused() {
    let f = Fixture::new();
    let first = f.create();
    let second = f.create();
    let original = std::fs::read(Path::new(&first.root).join(".git")).unwrap();
    let redirected = std::fs::read(Path::new(&second.root).join(".git")).unwrap();
    std::fs::write(Path::new(&first.root).join(".git"), redirected).unwrap();
    assert!(f.engine.require_worktree_ready(&first.session).is_err());
    std::fs::write(Path::new(&first.root).join(".git"), original).unwrap();
    f.engine.require_worktree_ready(&first.session).unwrap();
    let mut git = f.engine.git.lock().unwrap();
    git.reserve_worktree(
        "fixture-owned-operation",
        &[PathBuf::from(&first.root), f.root.clone()],
    )
    .unwrap();
    assert!(git.mutation_overlaps(Path::new(&first.root)));
    assert!(git
        .reserve_worktree("conflict", std::slice::from_ref(&f.root))
        .is_err());
    drop(git);
    let refused = f
        .engine
        .call(Command::Git {
            session: first.session.clone(),
            request: crate::source_control::Request::Status,
        })
        .unwrap_err();
    assert!(refused.contains("Worktree management"), "{refused}");
    f.engine
        .git
        .lock()
        .unwrap()
        .release_worktree("fixture-owned-operation");
    assert!(!f
        .engine
        .git
        .lock()
        .unwrap()
        .mutation_overlaps(Path::new(&first.root)));
}

#[cfg(windows)]
#[test]
fn managed_worktree_pins_shared_and_nested_chats_and_keeps_editor_recovery_usable() {
    let f = Fixture::new();
    let record = f.create();
    let root = Path::new(&record.root);
    std::fs::create_dir(root.join("nested")).unwrap();
    for (id, path) in [
        ("shared", root.to_path_buf()),
        ("nested", root.join("nested")),
    ] {
        f.store
            .create_workspace_session(
                id,
                &SessionWorkspace {
                    kind: WorkspaceKind::Project,
                    root: Some(path.to_string_lossy().into_owned()),
                },
            )
            .unwrap();
        let pins = f.engine.pin_worktree(id).unwrap();
        assert!(pins.is_some());
        assert!(std::fs::rename(root, root.with_extension("moved")).is_err());
        assert!(std::fs::write(root.join(".git"), "gitdir: elsewhere").is_err());
        drop(pins);
    }
    let call = |request: Value| {
        f.engine.call(
            serde_json::from_value::<Command>(
                json!({"command":"editor","session":record.session,"request":request}),
            )
            .unwrap(),
        )
    };
    let workspace = call(json!({"action":"workspace"})).unwrap();
    let project = workspace["project"].as_str().unwrap();
    let document = call(json!({"action":"open","project":project,"path":"note.txt"})).unwrap();
    let id = document["document"].as_str().unwrap();
    call(json!({"action":"edit","project":project,"document":id,"version":0,"edits":[{"start":0,"end":0,"text":"unsaved draft\n"}]})).unwrap();
    let moved = root.with_extension("retained");
    std::fs::rename(root, &moved).unwrap();
    assert!(f.engine.pin_worktree("shared").is_err());
    assert!(f.engine.pin_worktree("nested").is_err());
    let view = call(json!({"action":"workspace"})).unwrap();
    assert!(view["documents"][0]["text"]
        .as_str()
        .unwrap()
        .contains("unsaved draft"));
    call(json!({"action":"checkpoint","project":project})).unwrap();
    assert!(f
        .store
        .editor_state(&record.root)
        .unwrap()
        .to_string()
        .contains("unsaved draft"));
    assert!(call(json!({"action":"save","project":project,"document":id,"version":1})).is_err());
    assert_eq!(
        std::fs::read_to_string(moved.join("note.txt")).unwrap(),
        "base\n"
    );
    assert!(f
        .engine
        .call(Command::Terminal {
            request: crate::terminal::Request::Create {
                session: Some("shared".into()),
                home: false,
                shell: None
            }
        })
        .is_err());
    assert!(f
        .engine
        .call(Command::Language {
            request: crate::language::Request::Feature {
                session: "shared".into(),
                document: id.into(),
                version: 1,
                feature: "diagnostics".into(),
                position: json!({}),
                name: String::new()
            }
        })
        .is_err());
}

#[cfg(windows)]
struct WaitingProvider(std::sync::atomic::AtomicUsize);
#[cfg(windows)]
#[async_trait::async_trait]
impl dolores_core::ModelProvider for WaitingProvider {
    fn request_settings(&self) -> Option<dolores_core::RequestSettings> {
        Some(dolores_core::RequestSettings {
            max_output_tokens: Some(128),
            ..Default::default()
        })
    }
    fn descriptor(&self) -> dolores_core::PluginDescriptor {
        dolores_core::PluginDescriptor {
            id: "worktree-fixture",
            kind: "provider",
            api_version: 1,
        }
    }
    async fn stream(
        &self,
        _: Vec<dolores_core::Message>,
        _: tokio::sync::mpsc::Sender<String>,
        cancel: CancellationToken,
    ) -> Result<(), String> {
        self.0.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        cancel.cancelled().await;
        Err("Stopped fixture request.".into())
    }
    async fn stream_tool_turn(
        &self,
        _: &[dolores_core::AgentMessage],
        _: &[dolores_core::ToolSpec],
        output: tokio::sync::mpsc::Sender<String>,
        cancel: CancellationToken,
    ) -> Result<dolores_core::AgentTurn, String> {
        self.stream(vec![], output, cancel).await?;
        unreachable!()
    }
}

#[cfg(windows)]
#[test]
fn managed_worktree_two_runs_are_independent_and_queued_shared_root_stays_pinned() {
    let f = Fixture::new();
    let first = f.create();
    let second = f.create();
    f.store
        .create_workspace_session(
            "shared",
            &SessionWorkspace {
                kind: WorkspaceKind::Project,
                root: Some(first.root.clone()),
            },
        )
        .unwrap();
    f.engine
        .call(Command::Configure {
            preferences: dolores_core::ConnectionPreferences {
                base_url: "http://127.0.0.1:9/v1".into(),
                model: "fixture".into(),
            },
            api_key: Some(String::new()),
            remember: false,
            enabled_models: None,
            model_contexts: None,
        })
        .unwrap();
    f.store
        .save_request_settings(&dolores_core::RequestSettings {
            max_output_tokens: Some(128),
            ..Default::default()
        })
        .unwrap();
    let provider = Arc::new(WaitingProvider(std::sync::atomic::AtomicUsize::new(0)));
    f.engine.connection.lock().unwrap().provider = Some(provider.clone());
    for (id, session) in [
        (101, first.session.as_str()),
        (102, second.session.as_str()),
        (103, "shared"),
    ] {
        f.engine
            .call(Command::Start {
                desktop_handoff: false,
                desktop_capture: None,
                desktop_grant: None,
                desktop_reconciled: false,
                observation_model: None,
                resume_run: None,
                continuation: None,
                id,
                session: Some(session.into()),
                input: "Wait for fixture stop.".into(),
                workspace: None,
            })
            .unwrap();
    }
    let started = Instant::now();
    while provider.0.load(std::sync::atomic::Ordering::SeqCst) < 2 {
        assert!(started.elapsed() < Duration::from_secs(5));
        for id in [101, 102, 103] {
            f.engine.call(Command::Poll { id }).unwrap();
        }
        std::thread::sleep(Duration::from_millis(10));
    }
    assert_eq!(provider.0.load(std::sync::atomic::Ordering::SeqCst), 2);
    assert!(std::fs::rename(&first.root, format!("{}-moved", first.root)).is_err());
    f.engine.call(Command::Cancel { id: 101 }).unwrap();
    while provider.0.load(std::sync::atomic::Ordering::SeqCst) < 3 {
        assert!(started.elapsed() < Duration::from_secs(10));
        for id in [101, 102, 103] {
            f.engine.call(Command::Poll { id }).unwrap();
        }
        std::thread::sleep(Duration::from_millis(10));
    }
    assert!(std::fs::rename(&first.root, format!("{}-moved", first.root)).is_err());
    for id in [102, 103] {
        f.engine.call(Command::Cancel { id }).unwrap();
    }
    loop {
        for id in [101, 102, 103] {
            f.engine.call(Command::Poll { id }).unwrap();
        }
        if !f.engine.active.lock().unwrap().is_some() {
            break;
        }
        assert!(started.elapsed() < Duration::from_secs(15));
        std::thread::sleep(Duration::from_millis(10));
    }
    std::fs::rename(&first.root, format!("{}-moved", first.root)).unwrap();
    assert_eq!(
        std::fs::read_to_string(f.root.join("note.txt")).unwrap(),
        "base\n"
    );
}
