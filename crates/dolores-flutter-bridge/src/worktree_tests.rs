use super::*;
use crate::{git_process, Command};
use dolores_core::{SessionStore, SessionWorkspace, WorkspaceKind};
use dolores_store_sqlite::SqliteStore;
use std::path::{Path, PathBuf};

#[path = "worktree_archive_tests.rs"]
mod archive_tests;
#[path = "worktree_transfer_tests.rs"]
mod transfer_tests;

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
        self.complete_for(&self.source, request)
    }
    fn complete_for(&self, session: &str, request: Request) -> Result<Value, String> {
        let value = self.engine.call(Command::Worktrees {
            session: session.into(),
            request,
        })?;
        let id = value["job"].as_str().unwrap().to_owned();
        self.wait_for_job(session, &id)
    }
    fn wait_for_job(&self, session: &str, id: &str) -> Result<Value, String> {
        let started = Instant::now();
        loop {
            let value = self.engine.call(Command::Worktrees {
                session: session.into(),
                request: Request::Poll {
                    job: id.to_string(),
                },
            })?;
            if value["done"] == true {
                if let Some(error) = value["error"].as_str() {
                    return Err(error.into());
                }
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
fn managed_worktree_transfer_snapshot_preserves_binary_crlf_and_the_original_index() {
    let f = Fixture::new();
    let record = f.create();
    let root = Path::new(&record.root);
    let scratch = tempfile::tempdir_in(f._dir.path()).unwrap();
    let git_dir = Path::new(record.identity.as_ref().unwrap().git_directory.as_str());
    let before = std::fs::read(git_dir.join("index")).unwrap();
    std::fs::write(
        root.join("note.txt"),
        "unsaved staging-independent text\r\n",
    )
    .unwrap();
    std::fs::write(root.join("new 世界.bin"), [0, 1, 2, 0xff, 0xfe]).unwrap();
    std::fs::remove_file(root.join("filtered.txt")).unwrap();
    git(
        root,
        &[
            "config",
            "filter.fixture.process",
            "DOLORES_MUST_NOT_EXECUTE_FILTER",
        ],
    );
    git(root, &["config", "filter.fixture.required", "true"]);
    let cancel = CancellationToken::new();
    let mut git = worktree_git::Git::new(root).unwrap();
    git.materialization_options(scratch.path(), &cancel)
        .unwrap();
    let first = crate::worktree_snapshot::capture(&git, root, scratch.path(), &cancel).unwrap();
    assert_ne!(first.tree, first.head);
    assert_eq!(std::fs::read(git_dir.join("index")).unwrap(), before);
    let names = git
        .line(&["ls-tree", "-r", "--name-only", &first.tree], &cancel)
        .unwrap();
    assert!(names.contains("new 世界.bin"));
    assert!(!names.contains("filtered.txt"));
    let paths = vec![
        "note.txt".into(),
        "new 世界.bin".into(),
        "filtered.txt".into(),
    ];
    let retained = scratch.path().join("raw");
    let raw = crate::worktree_snapshot::retain_raw(root, &paths, &retained).unwrap();
    assert_eq!(
        std::fs::read(retained.join("0.bin")).unwrap(),
        b"unsaved staging-independent text\r\n"
    );
    assert_eq!(
        std::fs::read(retained.join("1.bin")).unwrap(),
        [0, 1, 2, 0xff, 0xfe]
    );
    assert!(!raw[2].exists);
    assert!(retained.join("manifest.json").is_file());
    assert!(crate::worktree_snapshot::retain_raw(root, &paths, &retained).is_err());
    let second = crate::worktree_snapshot::capture(&git, root, scratch.path(), &cancel).unwrap();
    assert_eq!(first, second);
    std::fs::write(root.join("note.txt"), "newer saved work").unwrap();
    assert_ne!(
        first.tree,
        crate::worktree_snapshot::capture(&git, root, scratch.path(), &cancel)
            .unwrap()
            .tree
    );
    assert_eq!(std::fs::read(f.root.join("note.txt")).unwrap(), b"base\n");
}

#[test]
fn managed_worktree_transfer_snapshot_refuses_bad_paths_stopped_capture_and_occupied_backups() {
    let f = Fixture::new();
    let record = f.create();
    let root = Path::new(&record.root);
    let scratch = tempfile::tempdir_in(f._dir.path()).unwrap();
    for path in [
        "../escape",
        ".git/config",
        "C:/absolute",
        "a\\alias",
        "a//b",
        "a/./b",
    ] {
        assert!(crate::worktree_snapshot::path(path).is_err());
    }
    let cancel = CancellationToken::new();
    cancel.cancel();
    let git = worktree_git::Git::new(root).unwrap();
    assert!(crate::worktree_snapshot::capture(&git, root, scratch.path(), &cancel).is_err());
    let target = scratch.path().join("occupied");
    std::fs::write(&target, b"foreign work").unwrap();
    assert!(crate::worktree_snapshot::retain_raw(root, &["note.txt".into()], &target).is_err());
    assert_eq!(std::fs::read(&target).unwrap(), b"foreign work");
    assert_eq!(std::fs::read(root.join("note.txt")).unwrap(), b"base\n");
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
fn managed_worktree_external_registration_invalidates_review_without_touching_either_checkout() {
    let f = Fixture::new();
    let review = f.review();
    let external = f._dir.path().join("external");
    git(
        &f.root,
        &[
            "worktree",
            "add",
            "--detach",
            external.to_str().unwrap(),
            "HEAD",
        ],
    );
    std::fs::write(external.join("note.txt"), "external dirty work\n").unwrap();
    let index = std::fs::read(f.root.join(".git/index")).unwrap();
    let error = f
        .complete(Request::Create {
            token: review["token"].as_str().unwrap().into(),
        })
        .unwrap_err();
    assert!(error.contains("Registered worktrees changed"), "{error}");
    assert!(f.store.worktrees(None).unwrap().items.is_empty());
    assert!(!Path::new(review["record"]["root"].as_str().unwrap()).exists());
    assert_eq!(
        std::fs::read_to_string(external.join("note.txt")).unwrap(),
        "external dirty work\n"
    );
    assert_eq!(std::fs::read(f.root.join(".git/index")).unwrap(), index);
    let created = f.create();
    assert_eq!(created.state, WorktreeState::Ready);
    assert_eq!(
        std::fs::read_to_string(external.join("note.txt")).unwrap(),
        "external dirty work\n"
    );
}

#[test]
fn managed_worktree_common_reservation_covers_later_external_checkouts_and_existing_git_jobs() {
    let f = Fixture::new();
    let review = f.review();
    let roots = {
        let registry = f.engine.worktrees.lock().unwrap();
        registry
            .reviews
            .get(review["token"].as_str().unwrap())
            .unwrap()
            .roots
            .clone()
    };
    let external = f._dir.path().join("external");
    git(
        &f.root,
        &[
            "worktree",
            "add",
            "--detach",
            external.to_str().unwrap(),
            "HEAD",
        ],
    );
    std::fs::create_dir(external.join("nested")).unwrap();
    f.store
        .create_workspace_session(
            "external",
            &SessionWorkspace {
                kind: WorkspaceKind::Project,
                root: Some(external.to_str().unwrap().into()),
            },
        )
        .unwrap();
    {
        let mut registry = f.engine.git.lock().unwrap();
        registry
            .active
            .insert(external.to_str().unwrap().into(), "fixture-git-job".into());
        assert!(registry
            .reserve_worktree("fixture-management", &roots)
            .is_err());
        registry.active.clear();
        registry
            .reserve_worktree("fixture-management", &roots)
            .unwrap();
        assert!(registry.mutation_overlaps(&external));
        assert!(registry.mutation_overlaps(&external.join("nested")));
        let unrelated = f._dir.path().join("unrelated");
        std::fs::create_dir(&unrelated).unwrap();
        assert!(!registry.mutation_overlaps(&unrelated));
    }
    let error = f
        .engine
        .call(Command::Git {
            session: "external".into(),
            request: crate::source_control::Request::Status,
        })
        .unwrap_err();
    assert!(error.contains("Worktree management"), "{error}");
    #[cfg(windows)]
    {
        let error = f
            .engine
            .call(Command::Terminal {
                request: crate::terminal::Request::Create {
                    session: Some("external".into()),
                    home: false,
                    shell: Some("cmd".into()),
                },
            })
            .unwrap_err();
        assert!(error.contains("repository operation"), "{error}");
    }
    let original_pointer = std::fs::read(external.join(".git")).unwrap();
    std::fs::write(external.join(".git"), "malformed pointer").unwrap();
    assert!(f.engine.git.lock().unwrap().mutation_overlaps(&external));
    std::fs::write(external.join(".git"), original_pointer).unwrap();
    f.engine
        .git
        .lock()
        .unwrap()
        .release_worktree("fixture-management");
    assert!(!f.engine.git.lock().unwrap().mutation_overlaps(&external));
}

#[test]
fn managed_worktree_active_external_run_blocks_creation_until_explicit_release() {
    let f = Fixture::new();
    let review = f.review();
    let external = f._dir.path().join("external");
    git(
        &f.root,
        &[
            "worktree",
            "add",
            "--detach",
            external.to_str().unwrap(),
            "HEAD",
        ],
    );
    let run = crate::Run {
        thread: Some("external".into()),
        id: 901,
        cancel: CancellationToken::new(),
        events: tokio::sync::mpsc::channel(1).1,
        approvals: Arc::new(Mutex::new(None)),
    };
    f.engine
        .active
        .lock()
        .unwrap()
        .reserve_primary(run, external.to_string_lossy().into_owned())
        .unwrap();
    let error = f
        .call(Request::Create {
            token: review["token"].as_str().unwrap().into(),
        })
        .unwrap_err();
    assert!(error.contains("Finish or stop tasks"), "{error}");
    assert!(f.store.worktrees(None).unwrap().items.is_empty());
    let pointer = std::fs::read(external.join(".git")).unwrap();
    std::fs::write(external.join(".git"), "malformed pointer").unwrap();
    let error = f
        .call(Request::Create {
            token: review["token"].as_str().unwrap().into(),
        })
        .unwrap_err();
    assert!(error.contains("malformed"), "{error}");
    std::fs::write(external.join(".git"), pointer).unwrap();
    f.engine.active.lock().unwrap().remove(901);
    // The original review remains available, but registration changes must still
    // refuse before the saved intent. A fresh explicit review recovers.
    assert!(f
        .complete(Request::Create {
            token: review["token"].as_str().unwrap().into()
        })
        .unwrap_err()
        .contains("Registered worktrees changed"));
    assert_eq!(f.create().state, WorktreeState::Ready);
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
        let inspected = f
            .engine
            .call(Command::Worktrees {
                session: id.into(),
                request: Request::Inspect,
            })
            .unwrap();
        assert_eq!(inspected["session"], record.session);
        assert_eq!(inspected["root"], record.root);
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
#[test]
fn managed_worktree_editor_git_and_terminal_use_the_bound_checkout() {
    let f = Fixture::new();
    let record = f.create();
    let root = Path::new(&record.root);
    let command = |request: Value| {
        f.engine.call(
            serde_json::from_value::<Command>(
                json!({"command":"editor","session":record.session,"request":request}),
            )
            .unwrap(),
        )
    };
    let workspace = command(json!({"action":"workspace"})).unwrap();
    let project = workspace["project"].as_str().unwrap();
    let document = command(json!({"action":"open","project":project,"path":"note.txt"})).unwrap();
    let id = document["document"].as_str().unwrap();
    command(json!({"action":"edit","project":project,"document":id,"version":0,"edits":[{"start":0,"end":0,"text":"owned edit\n"}]})).unwrap();
    command(json!({"action":"save","project":project,"document":id,"version":1})).unwrap();
    let change = f
        .store
        .begin_change(&dolores_core::ChangeDraft {
            root: record.root.clone(),
            session: record.session.clone(),
            target: "note.txt".into(),
            before: "base\n".into(),
            after: "owned edit\nbase\n".into(),
            before_exists: true,
            after_exists: true,
            reverts: None,
        })
        .unwrap();
    f.store.finish_change(change, true).unwrap();
    let revert = f.engine.preview_revert(&record.session, change).unwrap();
    let token = revert["token"].as_str().unwrap();
    f.engine
        .git
        .lock()
        .unwrap()
        .reserve_worktree("fixture-lifecycle", &[root.to_path_buf()])
        .unwrap();
    assert!(f
        .engine
        .preview_revert(&record.session, change)
        .unwrap_err()
        .contains("repository management"));
    assert!(f
        .engine
        .apply_revert(&record.session, token)
        .unwrap_err()
        .contains("repository management"));
    f.engine
        .git
        .lock()
        .unwrap()
        .release_worktree("fixture-lifecycle");
    let view = command(json!({"action":"workspace"})).unwrap();
    let version = view["documents"][0]["version"].as_u64().unwrap();
    command(json!({"action":"edit","project":project,"document":id,"version":version,"edits":[{"start":0,"end":0,"text":"unsaved draft\n"}]})).unwrap();
    assert!(f
        .engine
        .apply_revert(&record.session, token)
        .unwrap_err()
        .contains("unsaved editors"));
    assert_eq!(
        std::fs::read_to_string(root.join("note.txt")).unwrap(),
        "owned edit\nbase\n"
    );
    let job = f
        .engine
        .call(Command::Git {
            session: record.session.clone(),
            request: crate::source_control::Request::Status,
        })
        .unwrap();
    let started = Instant::now();
    loop {
        let result = f
            .engine
            .call(Command::Git {
                session: record.session.clone(),
                request: crate::source_control::Request::Poll {
                    job: job["job"].as_str().unwrap().into(),
                },
            })
            .unwrap();
        if result["done"] == true {
            assert_eq!(
                crate::source_control::path_key(Path::new(
                    result["value"]["root"].as_str().unwrap()
                )),
                crate::source_control::path_key(root)
            );
            assert_eq!(result["value"]["entries"][0]["path"], "note.txt");
            break;
        }
        assert!(started.elapsed() < Duration::from_secs(15));
        std::thread::sleep(Duration::from_millis(10));
    }
    let terminal = f
        .engine
        .call(Command::Terminal {
            request: crate::terminal::Request::Create {
                session: Some(record.session.clone()),
                home: false,
                shell: Some("cmd".into()),
            },
        })
        .unwrap();
    let id = terminal["id"].as_str().unwrap().to_owned();
    let result = (|| -> Result<(), String> {
        assert_eq!(
            crate::source_control::path_key(Path::new(terminal["cwd"].as_str().unwrap())),
            crate::source_control::path_key(root)
        );
        f.engine.call(Command::Terminal {
            request: crate::terminal::Request::Input {
                id: id.clone(),
                text: "echo MANAGED_COMMAND>command-location.txt\r\n".into(),
            },
        })?;
        let started = Instant::now();
        while std::fs::read_to_string(root.join("command-location.txt"))
            .ok()
            .is_none_or(|text| text.trim() != "MANAGED_COMMAND")
        {
            let page = f.engine.call(Command::Terminal {
                request: crate::terminal::Request::Poll { id: id.clone() },
            })?;
            use base64::Engine as _;
            let bytes = base64::engine::general_purpose::STANDARD
                .decode(page["bytes"].as_str().unwrap())
                .unwrap();
            if bytes.windows(4).any(|chunk| chunk == b"\x1b[6n") {
                f.engine.call(Command::Terminal {
                    request: crate::terminal::Request::Input {
                        id: id.clone(),
                        text: "\x1b[1;1R".into(),
                    },
                })?;
            }
            if started.elapsed() > Duration::from_secs(15) {
                return Err("Bound terminal did not create its fixture file.".into());
            }
            std::thread::sleep(Duration::from_millis(10));
        }
        assert_eq!(
            std::fs::read_to_string(root.join("command-location.txt"))
                .unwrap()
                .trim(),
            "MANAGED_COMMAND"
        );
        assert!(std::fs::rename(root, root.with_extension("moved")).is_err());
        Ok(())
    })();
    // Always stop only this owned fixture process, including assertion-free errors.
    f.engine
        .call(Command::Terminal {
            request: crate::terminal::Request::Stop { id: id.clone() },
        })
        .unwrap();
    f.engine
        .call(Command::Terminal {
            request: crate::terminal::Request::Close { id },
        })
        .unwrap();
    result.unwrap();
    assert_eq!(
        std::fs::read_to_string(f.root.join("note.txt")).unwrap(),
        "base\n"
    );
    assert!(!f.root.join("command-location.txt").exists());
}
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
    let nested = Path::new(&first.root).join("nested");
    std::fs::create_dir(&nested).unwrap();
    f.store
        .create_workspace_session(
            "shared",
            &SessionWorkspace {
                kind: WorkspaceKind::Project,
                root: Some(nested.to_string_lossy().into_owned()),
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
    {
        let active = f.engine.active.lock().unwrap();
        assert_eq!(active.primary_scope(101), active.primary_scope(103));
        assert_ne!(active.primary_scope(101), active.primary_scope(102));
    }
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
