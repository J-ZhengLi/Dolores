use super::*;

#[test]
fn managed_worktree_archive_restores_saved_file_directory_transitions() {
    let f = Fixture::new();
    std::fs::write(f.root.join("file-to-folder"), "original file\n").unwrap();
    std::fs::create_dir(f.root.join("folder-to-file")).unwrap();
    std::fs::write(f.root.join("folder-to-file/child.txt"), "original child\n").unwrap();
    git(&f.root, &["add", "."]);
    git(&f.root, &["commit", "-m", "Public path transition fixture"]);
    let source = f.create();
    let root = Path::new(&source.root);
    std::fs::remove_file(root.join("file-to-folder")).unwrap();
    std::fs::create_dir(root.join("file-to-folder")).unwrap();
    std::fs::write(root.join("file-to-folder/new.txt"), "retained child\n").unwrap();
    std::fs::remove_file(root.join("folder-to-file/child.txt")).unwrap();
    std::fs::remove_dir(root.join("folder-to-file")).unwrap();
    std::fs::write(root.join("folder-to-file"), "retained file\n").unwrap();
    let review = f
        .complete_for(&source.session, Request::ReviewArchive)
        .unwrap();
    f.complete_for(
        &source.session,
        Request::Archive {
            token: review["token"].as_str().unwrap().into(),
        },
    )
    .unwrap();
    let review = f
        .complete_for(&source.session, Request::ReviewRestore)
        .unwrap();
    f.complete_for(
        &source.session,
        Request::Restore {
            token: review["token"].as_str().unwrap().into(),
        },
    )
    .unwrap();
    assert_eq!(
        std::fs::read_to_string(root.join("file-to-folder/new.txt")).unwrap(),
        "retained child\n"
    );
    assert_eq!(
        std::fs::read_to_string(root.join("folder-to-file")).unwrap(),
        "retained file\n"
    );
    assert!(f.root.join("file-to-folder").is_file());
    assert!(f.root.join("folder-to-file").is_dir());
}

#[cfg(windows)]
#[test]
fn managed_worktree_archive_waits_for_terminals_language_pins_and_preserves_failed_setup_files() {
    let f = Fixture::new();
    let source = f.create();
    let root = Path::new(&source.root);
    let terminal = f
        .engine
        .call(Command::Terminal {
            request: crate::terminal::Request::Create {
                session: Some(source.session.clone()),
                home: false,
                shell: Some("cmd".into()),
            },
        })
        .unwrap();
    let id = terminal["id"].as_str().unwrap().to_string();
    assert!(f
        .complete_for(&source.session, Request::ReviewArchive)
        .unwrap_err()
        .contains("terminals"));
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
    {
        let pins = f.engine.pin_worktree(&source.session).unwrap().unwrap();
        let mut registry = f.engine.languages.lock().unwrap();
        registry.retain_checkout(source.root.clone(), pins);
        assert!(registry.checkout_pinned(root));
        assert!(!registry.checkout_pinned(&root.with_extension("another")));
    }
    assert!(f
        .complete_for(&source.session, Request::ReviewArchive)
        .unwrap_err()
        .contains("language services"));
    f.engine.languages.lock().unwrap().stop_all().unwrap();
    use dolores_core::ToolPlugin;
    let tool = dolores_tools_command::RunCommand::new(root).unwrap();
    let request=tool.prepare(&dolores_core::ToolCall {id:"failed-public-setup".into(),name:"run_command".into(),arguments:json!({"program":"node","args":["-e","require('fs').writeFileSync('setup-output.txt','FAILED_SETUP_RETAINED');process.exit(7)"]}).to_string()}).unwrap();
    let setup = f
        .engine
        .runtime
        .block_on(tool.invoke(&request, CancellationToken::new()))
        .unwrap();
    let setup: Value = serde_json::from_str(&setup).unwrap();
    assert_eq!(setup["exitCode"], 7, "{setup}");
    assert_eq!(
        std::fs::read_to_string(root.join("setup-output.txt"))
            .unwrap()
            .trim(),
        "FAILED_SETUP_RETAINED"
    );
    let review = f
        .complete_for(&source.session, Request::ReviewArchive)
        .unwrap();
    f.complete_for(
        &source.session,
        Request::Archive {
            token: review["token"].as_str().unwrap().into(),
        },
    )
    .unwrap();
    let restore = f
        .complete_for(&source.session, Request::ReviewRestore)
        .unwrap();
    f.complete_for(
        &source.session,
        Request::Restore {
            token: restore["token"].as_str().unwrap().into(),
        },
    )
    .unwrap();
    assert_eq!(
        std::fs::read_to_string(root.join("setup-output.txt"))
            .unwrap()
            .trim(),
        "FAILED_SETUP_RETAINED"
    );
    assert!(!f.root.join("setup-output.txt").exists());
}

#[test]
fn managed_worktree_archive_storage_full_and_snapshot_storage_failure_keep_checkout_and_staging() {
    let f = Fixture::new();
    let source = f.create();
    let root = Path::new(&source.root);
    std::fs::write(
        root.join("retained.txt"),
        "retained despite storage failure\n",
    )
    .unwrap();
    let index =
        std::fs::read(Path::new(&source.identity.as_ref().unwrap().git_directory).join("index"))
            .unwrap();
    let db = rusqlite::Connection::open(f._dir.path().join("data/state.db")).unwrap();
    db.execute_batch("CREATE TRIGGER fail_archive_intent BEFORE UPDATE ON managed_worktrees WHEN NEW.state='archiving' BEGIN SELECT RAISE(ABORT,'database or disk is full'); END;").unwrap();
    let review = f
        .complete_for(&source.session, Request::ReviewArchive)
        .unwrap();
    assert!(f
        .complete_for(
            &source.session,
            Request::Archive {
                token: review["token"].as_str().unwrap().into()
            }
        )
        .is_err());
    assert_eq!(
        f.store.worktree(&source.session).unwrap().unwrap().state,
        WorktreeState::Ready
    );
    assert_eq!(
        std::fs::read_to_string(root.join("retained.txt")).unwrap(),
        "retained despite storage failure\n"
    );
    assert_eq!(
        std::fs::read(Path::new(&source.identity.as_ref().unwrap().git_directory).join("index"))
            .unwrap(),
        index
    );
    db.execute_batch("DROP TRIGGER fail_archive_intent;")
        .unwrap();
    let evidence = Path::new(review["directory"].as_str().unwrap());
    let backup = evidence.join("files/0.bin");
    let original = std::fs::read(&backup).unwrap();
    std::fs::write(&backup, "changed archive bytes").unwrap();
    assert!(crate::worktree_archive::load(
        &source,
        review["snapshot"].as_str().unwrap(),
        evidence,
        &CancellationToken::new()
    )
    .err()
    .unwrap()
    .contains("archive bytes changed"));
    std::fs::write(&backup, original).unwrap();
    let huge = root.join("ignored-large.bin");
    std::fs::write(root.join(".gitignore"), "ignored-large.bin\n").unwrap();
    std::fs::File::create(&huge)
        .unwrap()
        .set_len(65 * 1024 * 1024)
        .unwrap();
    let error = f
        .complete_for(&source.session, Request::ReviewArchive)
        .unwrap_err();
    assert!(error.contains("64 MiB"), "{error}");
    assert!(root.is_dir());
    assert_eq!(huge.metadata().unwrap().len(), 65 * 1024 * 1024);
    std::fs::remove_file(huge).unwrap();
    let review = f
        .complete_for(&source.session, Request::ReviewArchive)
        .unwrap();
    assert_eq!(
        f.complete_for(
            &source.session,
            Request::Archive {
                token: review["token"].as_str().unwrap().into()
            }
        )
        .unwrap()["record"]["state"],
        "archived"
    );
}

#[test]
fn managed_worktree_archive_repeated_cycles_and_wrong_owned_parent_refuse_without_original_removal()
{
    let f = Fixture::new();
    let source = f.create();
    let root = Path::new(&source.root);
    let wrong = f._dir.path().join("wrong-parent");
    std::fs::create_dir(&wrong).unwrap();
    assert!(crate::worktree_archive::owned_target(&source, &wrong)
        .err()
        .unwrap()
        .contains("outside"));
    for cycle in 0..2 {
        std::fs::write(root.join("cycle.txt"), format!("cycle {cycle}\n")).unwrap();
        let review = f
            .complete_for(&source.session, Request::ReviewArchive)
            .unwrap();
        f.complete_for(
            &source.session,
            Request::Archive {
                token: review["token"].as_str().unwrap().into(),
            },
        )
        .unwrap();
        let restore = f
            .complete_for(&source.session, Request::ReviewRestore)
            .unwrap();
        f.complete_for(
            &source.session,
            Request::Restore {
                token: restore["token"].as_str().unwrap().into(),
            },
        )
        .unwrap();
        assert_eq!(
            std::fs::read_to_string(root.join("cycle.txt")).unwrap(),
            format!("cycle {cycle}\n")
        );
        assert_eq!(
            std::fs::read_to_string(f.root.join("note.txt")).unwrap(),
            "base\n"
        );
    }
}

#[cfg(windows)]
#[test]
fn managed_worktree_archive_partial_restore_refuses_foreign_bytes_then_explicit_resume_skips_completed_work(
) {
    use std::os::windows::fs::OpenOptionsExt;
    use windows_sys::Win32::Storage::FileSystem::FILE_SHARE_READ;
    let f = Fixture::new();
    let source = f.create();
    let root = Path::new(&source.root);
    std::fs::write(root.join("aaa-first.txt"), "completed file\n").unwrap();
    std::fs::write(root.join("note.txt"), "restored original\n").unwrap();
    let review = f
        .complete_for(&source.session, Request::ReviewArchive)
        .unwrap();
    f.complete_for(
        &source.session,
        Request::Archive {
            token: review["token"].as_str().unwrap().into(),
        },
    )
    .unwrap();
    let review = f
        .complete_for(&source.session, Request::ReviewRestore)
        .unwrap();
    let job = f
        .engine
        .call(Command::Worktrees {
            session: source.session.clone(),
            request: Request::Restore {
                token: review["token"].as_str().unwrap().into(),
            },
        })
        .unwrap();
    let until = Instant::now() + Duration::from_secs(15);
    let locked = loop {
        if let Ok(file) = std::fs::File::options()
            .read(true)
            .share_mode(FILE_SHARE_READ)
            .open(root.join("note.txt"))
        {
            break file;
        }
        assert!(
            Instant::now() < until,
            "Restore did not materialize its public base file in time"
        );
        std::thread::sleep(Duration::from_millis(1));
    };
    let error = f
        .wait_for_job(&source.session, job["job"].as_str().unwrap())
        .unwrap_err();
    assert!(error.contains("publish restored bytes"), "{error}");
    drop(locked);
    assert_eq!(
        std::fs::read_to_string(root.join("aaa-first.txt")).unwrap(),
        "completed file\n"
    );
    let fixed = std::time::SystemTime::UNIX_EPOCH + Duration::from_secs(1_600_000_000);
    std::fs::File::options()
        .write(true)
        .open(root.join("aaa-first.txt"))
        .unwrap()
        .set_times(std::fs::FileTimes::new().set_modified(fixed))
        .unwrap();
    std::fs::write(root.join("note.txt"), "foreign recovery version\n").unwrap();
    let saved = f.store.worktree(&source.session).unwrap().unwrap();
    assert_eq!(saved.recovery_from, Some(WorktreeState::Restoring));
    let inspected = f
        .complete_for(
            &source.session,
            Request::InspectLifecycle {
                revision: saved.revision,
            },
        )
        .unwrap();
    assert_eq!(inspected["record"]["state"], "recovery");
    let review = f
        .complete_for(&source.session, Request::ReviewRestore)
        .unwrap();
    let error = f
        .complete_for(
            &source.session,
            Request::Restore {
                token: review["token"].as_str().unwrap().into(),
            },
        )
        .unwrap_err();
    assert!(error.contains("Restore target changed"), "{error}");
    assert_eq!(
        std::fs::read_to_string(root.join("note.txt")).unwrap(),
        "foreign recovery version\n"
    );
    let foreign = f._dir.path().join("retained-foreign-note.txt");
    std::fs::rename(root.join("note.txt"), &foreign).unwrap();
    std::fs::write(root.join("note.txt"), "base\n").unwrap();
    let review = f
        .complete_for(&source.session, Request::ReviewRestore)
        .unwrap();
    let value = f
        .complete_for(
            &source.session,
            Request::Restore {
                token: review["token"].as_str().unwrap().into(),
            },
        )
        .unwrap();
    assert_eq!(value["record"]["state"], "ready");
    assert_eq!(
        std::fs::read_to_string(root.join("note.txt")).unwrap(),
        "restored original\n"
    );
    assert_eq!(
        std::fs::metadata(root.join("aaa-first.txt"))
            .unwrap()
            .modified()
            .unwrap(),
        fixed,
        "Completed file bytes must be skipped, not rewritten"
    );
    assert_eq!(
        std::fs::read_to_string(foreign).unwrap(),
        "foreign recovery version\n"
    );
}

#[test]
fn managed_worktree_archive_restore_failed_ready_save_reconciles_without_recopying() {
    let f = Fixture::new();
    let source = f.create();
    let root = Path::new(&source.root);
    std::fs::write(root.join("note.txt"), "complete restore\n").unwrap();
    let review = f
        .complete_for(&source.session, Request::ReviewArchive)
        .unwrap();
    f.complete_for(
        &source.session,
        Request::Archive {
            token: review["token"].as_str().unwrap().into(),
        },
    )
    .unwrap();
    let review = f
        .complete_for(&source.session, Request::ReviewRestore)
        .unwrap();
    let db = rusqlite::Connection::open(f._dir.path().join("data/state.db")).unwrap();
    db.execute_batch("CREATE TRIGGER fail_restored_ready BEFORE UPDATE ON managed_worktrees WHEN NEW.state='ready' BEGIN SELECT RAISE(ABORT,'fixture final Ready save failure'); END;").unwrap();
    assert!(f
        .complete_for(
            &source.session,
            Request::Restore {
                token: review["token"].as_str().unwrap().into()
            }
        )
        .is_err());
    assert_eq!(
        std::fs::read_to_string(root.join("note.txt")).unwrap(),
        "complete restore\n"
    );
    let before = std::fs::metadata(root.join("note.txt"))
        .unwrap()
        .modified()
        .unwrap();
    let saved = f.store.worktree(&source.session).unwrap().unwrap();
    assert_eq!(saved.recovery_from, Some(WorktreeState::Restoring));
    db.execute_batch("DROP TRIGGER fail_restored_ready;")
        .unwrap();
    let value = f
        .complete_for(
            &source.session,
            Request::InspectLifecycle {
                revision: saved.revision,
            },
        )
        .unwrap();
    assert_eq!(value["record"]["state"], "ready");
    assert_eq!(
        std::fs::metadata(root.join("note.txt"))
            .unwrap()
            .modified()
            .unwrap(),
        before
    );
    f.engine.require_worktree_ready(&source.session).unwrap();
}

#[test]
fn managed_worktree_archive_restore_retains_staging_binary_ignored_and_empty_directories() {
    let f = Fixture::new();
    let source = f.create();
    let root = Path::new(&source.root);
    std::fs::write(root.join("note.txt"), "staged work\n").unwrap();
    git(root, &["add", "note.txt"]);
    std::fs::write(root.join("note.txt"), "unstaged work 世界\r\n").unwrap();
    std::fs::write(root.join(".gitignore"), "ignored/\n").unwrap();
    std::fs::create_dir(root.join("ignored")).unwrap();
    std::fs::create_dir(root.join("empty")).unwrap();
    std::fs::write(root.join("ignored/cache.bin"), [0, 0xff, 0, 1, 0xfe]).unwrap();
    std::fs::write(root.join("untracked.txt"), "untracked preserved\n").unwrap();
    std::fs::remove_file(root.join("filtered.txt")).unwrap();
    let mut permissions = std::fs::metadata(root.join("ignored/cache.bin"))
        .unwrap()
        .permissions();
    permissions.set_readonly(true);
    std::fs::set_permissions(root.join("ignored/cache.bin"), permissions).unwrap();
    let index_path = Path::new(&source.identity.as_ref().unwrap().git_directory).join("index");
    let index = std::fs::read(&index_path).unwrap();
    let source_index = std::fs::read(f.root.join(".git/index")).unwrap();
    let review = f
        .complete_for(&source.session, Request::ReviewArchive)
        .unwrap();
    assert_eq!(review["kind"], "archive");
    assert!(root.is_dir());
    let wrong = f
        .call(Request::Archive {
            token: review["token"].as_str().unwrap().into(),
        })
        .unwrap_err();
    assert!(
        wrong.contains("owning managed") || wrong.contains("another chat"),
        "{wrong}"
    );
    let result = f
        .complete_for(
            &source.session,
            Request::Archive {
                token: review["token"].as_str().unwrap().into(),
            },
        )
        .unwrap();
    assert_eq!(result["record"]["state"], "archived", "{result}");
    assert!(!root.exists());
    assert!(f.engine.require_worktree_ready(&source.session).is_err());
    let archived = f.store.worktree(&source.session).unwrap().unwrap();
    let snapshot = archived.snapshot.as_deref().unwrap();
    assert_eq!(
        git(
            &f.root,
            &[
                "rev-parse",
                &crate::worktree_archive::archive_ref(&source, snapshot)
            ]
        ),
        snapshot
    );
    let restore = f
        .complete_for(&source.session, Request::ReviewRestore)
        .unwrap();
    assert_eq!(restore["kind"], "restore");
    assert!(!root.exists());
    let restored = f
        .complete_for(
            &source.session,
            Request::Restore {
                token: restore["token"].as_str().unwrap().into(),
            },
        )
        .unwrap();
    assert_eq!(restored["record"]["state"], "ready", "{restored}");
    f.engine.require_worktree_ready(&source.session).unwrap();
    assert_eq!(
        std::fs::read(root.join("note.txt")).unwrap(),
        "unstaged work 世界\r\n".as_bytes()
    );
    assert_eq!(
        std::fs::read(root.join("ignored/cache.bin")).unwrap(),
        [0, 0xff, 0, 1, 0xfe]
    );
    assert!(std::fs::metadata(root.join("ignored/cache.bin"))
        .unwrap()
        .permissions()
        .readonly());
    assert_eq!(
        std::fs::read_to_string(root.join("untracked.txt")).unwrap(),
        "untracked preserved\n"
    );
    assert!(root.join("empty").is_dir());
    assert!(!root.join("filtered.txt").exists());
    let record = f.store.worktree(&source.session).unwrap().unwrap();
    assert_eq!(
        std::fs::read(Path::new(&record.identity.unwrap().git_directory).join("index")).unwrap(),
        index
    );
    assert_eq!(
        std::fs::read(f.root.join(".git/index")).unwrap(),
        source_index
    );
    assert_eq!(
        std::fs::read_to_string(f.root.join("note.txt")).unwrap(),
        "base\n"
    );
    assert!(git(root, &["diff", "--cached"]).contains("staged work"));
    assert!(git(root, &["diff"]).contains("unstaged work"));
}

#[test]
fn managed_worktree_archive_changed_review_occupied_restore_and_missing_root_keep_foreign_work() {
    let f = Fixture::new();
    let source = f.create();
    let root = Path::new(&source.root);
    let review = f
        .complete_for(&source.session, Request::ReviewArchive)
        .unwrap();
    std::fs::write(root.join("newer.txt"), "retain newer work").unwrap();
    let error = f
        .complete_for(
            &source.session,
            Request::Archive {
                token: review["token"].as_str().unwrap().into(),
            },
        )
        .unwrap_err();
    assert!(error.contains("changed"), "{error}");
    assert_eq!(
        std::fs::read_to_string(root.join("newer.txt")).unwrap(),
        "retain newer work"
    );
    assert_eq!(
        f.store.worktree(&source.session).unwrap().unwrap().state,
        WorktreeState::Ready
    );
    let review = f
        .complete_for(&source.session, Request::ReviewArchive)
        .unwrap();
    f.complete_for(
        &source.session,
        Request::Archive {
            token: review["token"].as_str().unwrap().into(),
        },
    )
    .unwrap();
    let restore = f
        .complete_for(&source.session, Request::ReviewRestore)
        .unwrap();
    std::fs::create_dir(root).unwrap();
    std::fs::write(root.join("foreign.txt"), "foreign version").unwrap();
    assert!(f
        .complete_for(
            &source.session,
            Request::Restore {
                token: restore["token"].as_str().unwrap().into()
            }
        )
        .unwrap_err()
        .contains("occupied"));
    assert_eq!(
        std::fs::read_to_string(root.join("foreign.txt")).unwrap(),
        "foreign version"
    );
    let retained = root.with_extension("foreign-retained");
    std::fs::rename(root, &retained).unwrap();
    let restore = f
        .complete_for(&source.session, Request::ReviewRestore)
        .unwrap();
    f.complete_for(
        &source.session,
        Request::Restore {
            token: restore["token"].as_str().unwrap().into(),
        },
    )
    .unwrap();
    assert_eq!(
        std::fs::read_to_string(retained.join("foreign.txt")).unwrap(),
        "foreign version"
    );
    assert_eq!(
        std::fs::read_to_string(root.join("newer.txt")).unwrap(),
        "retain newer work"
    );
    let moved = root.with_extension("missing-retained");
    std::fs::rename(root, &moved).unwrap();
    assert!(f
        .complete_for(&source.session, Request::ReviewArchive)
        .is_err());
    assert_eq!(
        std::fs::read_to_string(moved.join("newer.txt")).unwrap(),
        "retain newer work"
    );
}

#[test]
fn managed_worktree_archive_failed_final_save_inspects_without_repeating_removal() {
    let f = Fixture::new();
    let source = f.create();
    let root = Path::new(&source.root);
    std::fs::write(root.join("manual-setup.txt"), "failed setup output remains").unwrap();
    let review = f
        .complete_for(&source.session, Request::ReviewArchive)
        .unwrap();
    let db = rusqlite::Connection::open(f._dir.path().join("data/state.db")).unwrap();
    db.execute_batch("CREATE TRIGGER fail_archived BEFORE UPDATE ON managed_worktrees WHEN NEW.state='archived' BEGIN SELECT RAISE(ABORT,'fixture final storage failure'); END;").unwrap();
    let error = f
        .complete_for(
            &source.session,
            Request::Archive {
                token: review["token"].as_str().unwrap().into(),
            },
        )
        .unwrap_err();
    assert!(error.contains("snapshot"), "{error}");
    assert!(!root.exists());
    let saved = f.store.worktree(&source.session).unwrap().unwrap();
    assert_eq!(saved.state, WorktreeState::Recovery);
    assert_eq!(saved.recovery_from, Some(WorktreeState::Archiving));
    db.execute_batch("DROP TRIGGER fail_archived;").unwrap();
    let inspected = f
        .complete_for(
            &source.session,
            Request::InspectLifecycle {
                revision: saved.revision,
            },
        )
        .unwrap();
    assert_eq!(inspected["record"]["state"], "archived");
    assert!(!root.exists());
    let restore = f
        .complete_for(&source.session, Request::ReviewRestore)
        .unwrap();
    f.complete_for(
        &source.session,
        Request::Restore {
            token: restore["token"].as_str().unwrap().into(),
        },
    )
    .unwrap();
    assert_eq!(
        std::fs::read_to_string(root.join("manual-setup.txt")).unwrap(),
        "failed setup output remains"
    );
}
