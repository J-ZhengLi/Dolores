use super::*;
use crate::Command;
use dolores_core::{
    harness_task::{InputMode, InputState},
    SessionWorkspace, WorkspaceKind,
};
use dolores_store_sqlite::SqliteStore;

struct Fixture {
    _dir: tempfile::TempDir,
    root: PathBuf,
    store: Arc<SqliteStore>,
    engine: Engine,
}

#[test]
fn committed_finding_status_releases_only_its_own_terminal_job_without_replaying() {
    let f = Fixture::new();
    let basis = f.comparison();
    let start = f
        .engine
        .call(Command::Git {
            session: "chat".into(),
            request: super::super::Request::Finding {
                operation: Box::new(Operation::Capture {
                    id: "lost-result".into(),
                    comparison: basis,
                    cursor: 0,
                    side_by_side: false,
                    side: Side::Right,
                    line: 1,
                    text: "Retained after transport loss".into(),
                }),
            },
        })
        .unwrap();
    let id = start["job"].as_str().unwrap();
    assert!(f
        .engine
        .call(Command::Git {
            session: "other".into(),
            request: super::super::Request::ReleaseFindingJob { job: id.into() }
        })
        .is_err());
    let began = std::time::Instant::now();
    loop {
        let saved = f
            .engine
            .call(Command::ReviewFindings {
                session: "chat".into(),
                request: Request::Get {
                    id: "lost-result".into(),
                },
            })
            .unwrap();
        if !saved.is_null() {
            let released = f
                .engine
                .call(Command::Git {
                    session: "chat".into(),
                    request: super::super::Request::ReleaseFindingJob { job: id.into() },
                })
                .unwrap();
            if released["released"] == true {
                break;
            }
        }
        assert!(began.elapsed().as_secs() < 25);
        std::thread::sleep(std::time::Duration::from_millis(10));
    }
    assert!(f
        .engine
        .call(Command::Git {
            session: "chat".into(),
            request: super::super::Request::Poll { job: id.into() }
        })
        .is_err());
    assert_eq!(
        f.store.review_findings("chat", None).unwrap().items.len(),
        1
    );
    assert!(f.store.task_inputs("chat").unwrap().entries.is_empty());
}

#[test]
fn split_page_capture_refuses_long_or_undisplayed_lines_and_cancelled_work() {
    let f = Fixture::new();
    let basis = f.comparison();
    let value = f
        .operation(
            "chat",
            Operation::Capture {
                id: "split".into(),
                comparison: basis,
                cursor: 0,
                side_by_side: true,
                side: Side::Right,
                line: 1,
                text: "Inspect the split comparison".into(),
            },
        )
        .unwrap();
    assert_eq!(value["finding"]["anchor"]["excerpt"], "new");
    std::fs::write(f.root.join("a.txt"), format!("{}\n", "x".repeat(5000))).unwrap();
    let basis = f.comparison();
    assert!(f
        .capture("long", basis.clone(), Side::Right, 1)
        .unwrap_err()
        .contains("2 KiB"));
    let repo = Repo::discover(f.root.to_str().unwrap(), &CancellationToken::new()).unwrap();
    let cancel = CancellationToken::new();
    cancel.cancel();
    assert!(execute(
        &repo,
        f.store.as_ref(),
        "chat",
        &Operation::Capture {
            id: "stopped".into(),
            comparison: basis,
            cursor: 0,
            side_by_side: false,
            side: Side::Left,
            line: 1,
            text: "Retain this note".into()
        },
        &cancel
    )
    .unwrap_err()
    .contains("stopped"));
    assert!(f.store.review_finding("chat", "stopped").unwrap().is_none());
    assert_eq!(
        f.store.review_findings("chat", None).unwrap().items.len(),
        1
    );
}
impl Fixture {
    fn new() -> Self {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path().join("public-project");
        std::fs::create_dir(&root).unwrap();
        let git = git_process::executable(&root).unwrap();
        for args in [
            vec!["init", "-b", "main"],
            vec!["config", "user.name", "Fixture"],
            vec!["config", "user.email", "fixture@example.invalid"],
            vec!["config", "core.autocrlf", "false"],
            vec!["config", "commit.gpgSign", "false"],
        ] {
            assert_eq!(
                git_process::run(
                    &git,
                    &root,
                    &args.iter().map(|s| s.to_string()).collect::<Vec<_>>(),
                    &CancellationToken::new(),
                    false
                )
                .unwrap()
                .code,
                0
            );
        }
        std::fs::write(root.join("a.txt"), "old\n").unwrap();
        for args in [
            vec!["add", "."],
            vec!["commit", "-m", "Public review fixture"],
        ] {
            assert_eq!(
                git_process::run(
                    &git,
                    &root,
                    &args.iter().map(|s| s.to_string()).collect::<Vec<_>>(),
                    &CancellationToken::new(),
                    false
                )
                .unwrap()
                .code,
                0
            );
        }
        let store = Arc::new(SqliteStore::open(&dir.path().join("state.db")).unwrap());
        for id in ["chat", "other"] {
            store
                .create_workspace_session(
                    id,
                    &SessionWorkspace {
                        kind: WorkspaceKind::Project,
                        root: Some(crate::workspace::canonical_folder(&root).unwrap()),
                    },
                )
                .unwrap();
        }
        store.save_draft("chat", "Keep unsent 世界").unwrap();
        let engine = Engine::new(
            store.clone(),
            Arc::new(crate::connection::testing::MemoryCredentials::default()),
        )
        .unwrap();
        std::fs::write(root.join("a.txt"), "new\n").unwrap();
        Self {
            _dir: dir,
            root,
            store,
            engine,
        }
    }
    fn comparison(&self) -> Comparison {
        let cancel = CancellationToken::new();
        let repo = Repo::discover(self.root.to_str().unwrap(), &cancel).unwrap();
        let status = repo.status(&cancel).unwrap();
        let diff = repo
            .diff_page(
                status["revision"].as_str().unwrap(),
                "a.txt",
                "working",
                None,
                (0, None),
                &cancel,
            )
            .unwrap();
        Comparison {
            root: repo.root.to_string_lossy().into_owned(),
            repository: status["repo"].as_str().unwrap().into(),
            path: "a.txt".into(),
            basis: "working".into(),
            commit: None,
            revision: status["revision"].as_str().unwrap().into(),
            digest: diff["digest"].as_str().unwrap().into(),
        }
    }
    fn operation(&self, session: &str, operation: Operation) -> Result<Value, String> {
        let started = self.engine.call(Command::Git {
            session: session.into(),
            request: super::super::Request::Finding {
                operation: Box::new(operation),
            },
        })?;
        self.finish(session, started["job"].as_str().unwrap())
    }
    fn finish(&self, session: &str, id: &str) -> Result<Value, String> {
        let start = std::time::Instant::now();
        loop {
            let result = self.engine.call(Command::Git {
                session: session.into(),
                request: super::super::Request::Poll { job: id.into() },
            })?;
            if result["done"] == true {
                return if let Some(error) = result["error"].as_str() {
                    Err(error.into())
                } else {
                    Ok(result["value"].clone())
                };
            }
            assert!(start.elapsed().as_secs() < 25);
            std::thread::sleep(std::time::Duration::from_millis(10));
        }
    }
    fn capture(
        &self,
        id: &str,
        comparison: Comparison,
        side: Side,
        line: u32,
    ) -> Result<Value, String> {
        self.operation(
            "chat",
            Operation::Capture {
                id: id.into(),
                comparison,
                cursor: 0,
                side_by_side: false,
                side,
                line,
                text: "Handle empty input".into(),
            },
        )
    }
}

#[test]
fn review_capture_queue_and_later_resolution_use_host_rows_without_editing_files_or_draft() {
    let f = Fixture::new();
    let initial = f.comparison();
    let before = std::fs::read(f.root.join(".git/index")).unwrap();
    let captured = f.capture("right", initial.clone(), Side::Right, 1).unwrap();
    assert_eq!(captured["finding"]["anchor"]["excerpt"], "new");
    let left = f.capture("left", initial.clone(), Side::Left, 1).unwrap();
    assert_eq!(left["finding"]["anchor"]["excerpt"], "old");
    assert!(f
        .capture("outside", initial.clone(), Side::Right, 99)
        .unwrap_err()
        .contains("outside"));
    let selected = Submission {
        id: "repair-selection".into(),
        mode: InputMode::Queue,
        selected: vec![Selection {
            id: "right".into(),
            revision: 0,
        }],
    };
    let sent = f
        .operation(
            "chat",
            Operation::Submit {
                submission: selected.clone(),
            },
        )
        .unwrap();
    assert_eq!(sent["inputs"]["entries"].as_array().unwrap().len(), 1);
    assert_eq!(f.store.saved_draft("chat").unwrap(), "Keep unsent 世界");
    assert!(f
        .store
        .review_finding("chat", "right")
        .unwrap()
        .unwrap()
        .resolution
        .is_none());
    f.store
        .claim_task_input("chat", &selected.id, 1, "fixture-run")
        .unwrap();
    f.store
        .finish_task_input("chat", &selected.id, InputState::Applied)
        .unwrap();
    std::fs::write(f.root.join("a.txt"), "repaired empty input\n").unwrap();
    let later = f.comparison();
    let retry = f
        .operation(
            "chat",
            Operation::Submit {
                submission: selected.clone(),
            },
        )
        .unwrap();
    assert_eq!(retry["inputs"]["entries"].as_array().unwrap().len(), 1);
    assert_eq!(retry["retained"], true);
    assert!(f
        .operation(
            "other",
            Operation::Submit {
                submission: selected
            }
        )
        .is_err());
    let resolved = f
        .operation(
            "chat",
            Operation::Resolve {
                id: "right".into(),
                revision: 1,
                comparison: later.clone(),
                note: "Verified public empty-input test".into(),
            },
        )
        .unwrap();
    assert_eq!(
        resolved["finding"]["resolution"]["comparison"]["digest"],
        later.digest
    );
    let retry = f
        .operation(
            "chat",
            Operation::Resolve {
                id: "right".into(),
                revision: 1,
                comparison: later,
                note: "Verified public empty-input test".into(),
            },
        )
        .unwrap();
    assert_eq!(retry["retained"], true);
    assert_eq!(std::fs::read(f.root.join(".git/index")).unwrap(), before);
    assert_eq!(
        std::fs::read_to_string(f.root.join("a.txt")).unwrap(),
        "repaired empty input\n"
    );
}

#[test]
fn review_stale_locations_wrong_checkout_changed_digest_and_failed_storage_keep_notes() {
    let f = Fixture::new();
    let initial = f.comparison();
    f.capture("retained", initial.clone(), Side::Right, 1)
        .unwrap();
    let mut wrong = initial.clone();
    wrong.root = f._dir.path().join("another").to_string_lossy().into_owned();
    assert!(f
        .capture("wrong", wrong, Side::Right, 1)
        .unwrap_err()
        .contains("different checkout"));
    let mut changed = initial.clone();
    changed.digest = "0".repeat(64);
    assert!(f
        .capture("digest", changed, Side::Right, 1)
        .unwrap_err()
        .contains("differ"));
    std::fs::write(f.root.join("a.txt"), "newer unrelated work\n").unwrap();
    assert!(f
        .capture("stale", initial.clone(), Side::Right, 1)
        .unwrap_err()
        .contains("changed"));
    let error = f
        .operation(
            "chat",
            Operation::Submit {
                submission: Submission {
                    id: "stale-submit".into(),
                    mode: InputMode::Steer,
                    selected: vec![Selection {
                        id: "retained".into(),
                        revision: 0,
                    }],
                },
            },
        )
        .unwrap_err();
    assert!(error.contains("stale"), "{error}");
    assert!(f.store.task_inputs("chat").unwrap().entries.is_empty());
    assert_eq!(
        f.store
            .review_finding("chat", "retained")
            .unwrap()
            .unwrap()
            .anchor
            .comparison,
        initial
    );
    let db = rusqlite::Connection::open(f._dir.path().join("state.db")).unwrap();
    db.execute_batch("CREATE TRIGGER refuse_review BEFORE INSERT ON review_findings BEGIN SELECT RAISE(ABORT,'fixture save failure'); END;").unwrap();
    let latest = f.comparison();
    assert!(f
        .capture("failed-save", latest.clone(), Side::Right, 1)
        .is_err());
    assert!(f
        .store
        .review_finding("chat", "failed-save")
        .unwrap()
        .is_none());
    db.execute_batch("DROP TRIGGER refuse_review;").unwrap();
    assert!(f.capture("failed-save", latest, Side::Right, 1).is_ok());
    assert_eq!(f.store.saved_draft("chat").unwrap(), "Keep unsent 世界");
    assert_eq!(
        std::fs::read_to_string(f.root.join("a.txt")).unwrap(),
        "newer unrelated work\n"
    );
    // Local inspection/removal is independent of a working Git checkout.
    let moved = f._dir.path().join("retained-project");
    std::fs::rename(&f.root, &moved).unwrap();
    let list = f
        .engine
        .call(Command::ReviewFindings {
            session: "chat".into(),
            request: Request::List { cursor: None },
        })
        .unwrap();
    assert_eq!(list["items"].as_array().unwrap().len(), 2);
    f.engine
        .call(Command::ReviewFindings {
            session: "chat".into(),
            request: Request::Remove {
                id: "failed-save".into(),
                revision: 0,
            },
        })
        .unwrap();
    assert!(moved.join("a.txt").is_file());
}
