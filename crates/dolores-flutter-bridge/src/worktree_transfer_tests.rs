use super::*;

#[test]
fn managed_worktree_transfer_selected_binary_create_delete_preserves_both_indexes_and_dirty_work() {
    let f = Fixture::new();
    let source = f.create();
    let root = Path::new(&source.root);
    std::fs::write(root.join("note.txt"), "selected source change\n").unwrap();
    std::fs::write(root.join("new 世界.bin"), [0, 1, 0xff, 0, 0xfe]).unwrap();
    std::fs::remove_file(root.join("filtered.txt")).unwrap();
    std::fs::write(root.join("not-selected.txt"), "keep only in source").unwrap();
    std::fs::write(f.root.join("unrelated.txt"), "staged destination work\n").unwrap();
    git(&f.root, &["add", "unrelated.txt"]);
    std::fs::write(
        f.root.join("untracked.txt"),
        "retain destination untracked work",
    )
    .unwrap();
    let original_index = std::fs::read(f.root.join(".git/index")).unwrap();
    let source_index =
        std::fs::read(Path::new(&source.identity.as_ref().unwrap().git_directory).join("index"))
            .unwrap();
    let comparison = f
        .complete_for(&source.session, Request::CompareTransfer)
        .unwrap();
    assert!(comparison["paths"]
        .as_array()
        .unwrap()
        .iter()
        .any(|v| v == "new 世界.bin"));
    let review = f
        .complete_for(
            &source.session,
            Request::ReviewTransfer {
                destination: f.source.clone(),
                paths: vec![
                    "note.txt".into(),
                    "new 世界.bin".into(),
                    "filtered.txt".into(),
                ],
            },
        )
        .unwrap();
    assert_eq!(review["receipt"]["applicable"], true, "{review}");
    assert_eq!(
        std::fs::read_to_string(f.root.join("note.txt")).unwrap(),
        "base\n"
    );
    let token = review["token"].as_str().unwrap().to_string();
    let other = f.create();
    assert!(f
        .complete_for(
            &other.session,
            Request::ApplyTransfer {
                token: token.clone()
            }
        )
        .unwrap_err()
        .contains("another chat"));
    let result = f
        .complete_for(
            &source.session,
            Request::ApplyTransfer {
                token: token.clone(),
            },
        )
        .unwrap();
    assert_eq!(result["applied"], true, "{result}");
    assert!(f
        .complete_for(&source.session, Request::ApplyTransfer { token })
        .is_err());
    assert_eq!(
        std::fs::read_to_string(f.root.join("note.txt")).unwrap(),
        "selected source change\n"
    );
    assert_eq!(
        std::fs::read(f.root.join("new 世界.bin")).unwrap(),
        [0, 1, 0xff, 0, 0xfe]
    );
    assert!(!f.root.join("filtered.txt").exists());
    assert!(!f.root.join("not-selected.txt").exists());
    assert_eq!(
        std::fs::read_to_string(f.root.join("untracked.txt")).unwrap(),
        "retain destination untracked work"
    );
    assert_eq!(
        std::fs::read(f.root.join(".git/index")).unwrap(),
        original_index
    );
    assert_eq!(
        std::fs::read(Path::new(&source.identity.as_ref().unwrap().git_directory).join("index"))
            .unwrap(),
        source_index
    );
    assert_eq!(
        std::fs::read_to_string(root.join("not-selected.txt")).unwrap(),
        "keep only in source"
    );
    let directory = Path::new(review["directory"].as_str().unwrap());
    assert!(directory.join("intent.json").is_file());
    assert!(directory.join("result.json").is_file());
    assert_eq!(
        std::fs::read(directory.join("destination-files/0.bin")).unwrap(),
        b"base\n"
    );
    let listing = f
        .engine
        .call(Command::Worktrees {
            session: source.session.clone(),
            request: Request::ListTransfers,
        })
        .unwrap();
    assert_eq!(listing["items"][0]["result"]["state"], "applied");
    let retained_ref = format!(
        "refs/dolores/transfers/{}/source-files",
        review["receipt"]["id"].as_str().unwrap()
    );
    assert_eq!(
        git(&f.root, &["rev-parse", &retained_ref]),
        review["receipt"]["sourceSnapshot"]["tree"]
    );
}

#[test]
fn managed_worktree_transfer_conflict_keeps_versions_and_recovery_never_replays() {
    let f = Fixture::new();
    let source = f.create();
    std::fs::write(
        Path::new(&source.root).join("note.txt"),
        "source conflict\n",
    )
    .unwrap();
    std::fs::write(f.root.join("note.txt"), "destination conflict\n").unwrap();
    let review = f
        .complete_for(
            &source.session,
            Request::ReviewTransfer {
                destination: f.source.clone(),
                paths: vec!["note.txt".into()],
            },
        )
        .unwrap();
    assert_eq!(review["receipt"]["applicable"], false);
    let directory = Path::new(review["directory"].as_str().unwrap());
    assert_eq!(
        std::fs::read(directory.join("source-files/0.bin")).unwrap(),
        b"source conflict\n"
    );
    assert_eq!(
        std::fs::read(directory.join("destination-files/0.bin")).unwrap(),
        b"destination conflict\n"
    );
    assert!(f
        .complete_for(
            &source.session,
            Request::ApplyTransfer {
                token: review["token"].as_str().unwrap().into()
            }
        )
        .unwrap_err()
        .contains("conflicts"));
    assert!(!directory.join("intent.json").exists());
    assert_eq!(
        std::fs::read_to_string(f.root.join("note.txt")).unwrap(),
        "destination conflict\n"
    );
    std::fs::write(f.root.join("note.txt"), "base\n").unwrap();
    let fresh = f
        .complete_for(
            &source.session,
            Request::ReviewTransfer {
                destination: f.source.clone(),
                paths: vec!["note.txt".into()],
            },
        )
        .unwrap();
    let folder = Path::new(fresh["directory"].as_str().unwrap());
    // Deterministic intent-save failure prevents publication, preserving the patch
    // and both versions. The existing artifact is never overwritten.
    std::fs::write(
        folder.join("intent.json"),
        "{\"fixture\":\"publication unknown\"}",
    )
    .unwrap();
    assert!(f
        .complete_for(
            &source.session,
            Request::ApplyTransfer {
                token: fresh["token"].as_str().unwrap().into()
            }
        )
        .unwrap_err()
        .contains("evidence could not be created"));
    assert_eq!(
        std::fs::read_to_string(f.root.join("note.txt")).unwrap(),
        "base\n"
    );
    let reopened = Engine::new(
        f.store.clone(),
        Arc::new(crate::connection::testing::MemoryCredentials::default()),
    )
    .unwrap();
    let list = crate::worktree_transfer::list(folder.parent().unwrap(), &source.session).unwrap();
    assert!(list["items"]
        .as_array()
        .unwrap()
        .iter()
        .any(|v| v["result"]["state"] == "uncertain"));
    assert_eq!(
        std::fs::read_to_string(f.root.join("note.txt")).unwrap(),
        "base\n"
    );
    assert!(reopened
        .call(Command::Worktrees {
            session: source.session.clone(),
            request: Request::ApplyTransfer {
                token: fresh["token"].as_str().unwrap().into()
            }
        })
        .is_err());
    let final_review = f
        .complete_for(
            &source.session,
            Request::ReviewTransfer {
                destination: f.source.clone(),
                paths: vec!["note.txt".into()],
            },
        )
        .unwrap();
    assert_eq!(
        f.complete_for(
            &source.session,
            Request::ApplyTransfer {
                token: final_review["token"].as_str().unwrap().into()
            }
        )
        .unwrap()["applied"],
        true
    );
}

#[test]
fn managed_worktree_transfer_stale_files_modified_patch_and_cancel_keep_destination() {
    let f = Fixture::new();
    let source = f.create();
    let paths = vec!["note.txt".into()];
    std::fs::write(Path::new(&source.root).join("note.txt"), "chosen change\n").unwrap();
    let review = f
        .complete_for(
            &source.session,
            Request::ReviewTransfer {
                destination: f.source.clone(),
                paths: paths.clone(),
            },
        )
        .unwrap();
    std::fs::write(f.root.join("note.txt"), "newer destination work\n").unwrap();
    assert!(f
        .complete_for(
            &source.session,
            Request::ApplyTransfer {
                token: review["token"].as_str().unwrap().into()
            }
        )
        .unwrap_err()
        .contains("changed after review"));
    assert_eq!(
        std::fs::read_to_string(f.root.join("note.txt")).unwrap(),
        "newer destination work\n"
    );
    std::fs::write(f.root.join("note.txt"), "base\n").unwrap();
    let review = f
        .complete_for(
            &source.session,
            Request::ReviewTransfer {
                destination: f.source.clone(),
                paths: paths.clone(),
            },
        )
        .unwrap();
    std::fs::write(
        Path::new(review["directory"].as_str().unwrap()).join("selected.patch"),
        "modified patch",
    )
    .unwrap();
    assert!(f
        .complete_for(
            &source.session,
            Request::ApplyTransfer {
                token: review["token"].as_str().unwrap().into()
            }
        )
        .unwrap_err()
        .contains("patch changed"));
    let review = f
        .complete_for(
            &source.session,
            Request::ReviewTransfer {
                destination: f.source.clone(),
                paths,
            },
        )
        .unwrap();
    let mut registry = f.engine.worktrees.lock().unwrap();
    let pending = registry
        .transfers
        .remove(review["token"].as_str().unwrap())
        .unwrap();
    drop(registry);
    let stopped = CancellationToken::new();
    stopped.cancel();
    assert!(crate::worktree_transfer::apply(f.store.as_ref(), pending, &stopped).is_err());
    assert_eq!(
        std::fs::read_to_string(f.root.join("note.txt")).unwrap(),
        "base\n"
    );
    assert!(!Path::new(review["directory"].as_str().unwrap())
        .join("intent.json")
        .exists());
}

#[test]
fn managed_worktree_transfer_result_save_failure_retains_completed_effect_and_raw_recovery() {
    let f = Fixture::new();
    let source = f.create();
    std::fs::write(
        Path::new(&source.root).join("note.txt"),
        "retained completed effect\n",
    )
    .unwrap();
    let review = f
        .complete_for(
            &source.session,
            Request::ReviewTransfer {
                destination: f.source.clone(),
                paths: vec!["note.txt".into()],
            },
        )
        .unwrap();
    let directory = Path::new(review["directory"].as_str().unwrap());
    std::fs::write(directory.join("result.json"), "{}").unwrap();
    let error = f
        .complete_for(
            &source.session,
            Request::ApplyTransfer {
                token: review["token"].as_str().unwrap().into(),
            },
        )
        .unwrap_err();
    assert!(error.contains("Publication may have finished"), "{error}");
    assert_eq!(
        std::fs::read_to_string(f.root.join("note.txt")).unwrap(),
        "retained completed effect\n"
    );
    assert_eq!(
        std::fs::read(directory.join("destination-files/0.bin")).unwrap(),
        b"base\n"
    );
    assert!(directory.join("intent.json").is_file());
    assert!(
        crate::worktree_transfer::list(directory.parent().unwrap(), &source.session)
            .unwrap_err()
            .contains("malformed")
    );
    assert!(f
        .complete_for(
            &source.session,
            Request::ApplyTransfer {
                token: review["token"].as_str().unwrap().into()
            }
        )
        .is_err());
}

#[test]
fn managed_worktree_transfer_changed_backup_and_snapshot_limit_refuse_before_publication() {
    let f = Fixture::new();
    let source = f.create();
    let root = Path::new(&source.root);
    std::fs::write(root.join("note.txt"), "source change\n").unwrap();
    let review = f
        .complete_for(
            &source.session,
            Request::ReviewTransfer {
                destination: f.source.clone(),
                paths: vec!["note.txt".into()],
            },
        )
        .unwrap();
    std::fs::write(
        Path::new(review["directory"].as_str().unwrap()).join("destination-files/0.bin"),
        "altered backup",
    )
    .unwrap();
    let error = f
        .complete_for(
            &source.session,
            Request::ApplyTransfer {
                token: review["token"].as_str().unwrap().into(),
            },
        )
        .unwrap_err();
    assert!(error.contains("backup changed"), "{error}");
    assert_eq!(
        std::fs::read_to_string(f.root.join("note.txt")).unwrap(),
        "base\n"
    );
    let huge = root.join("too-large.bin");
    std::fs::File::create(&huge)
        .unwrap()
        .set_len(65 * 1024 * 1024)
        .unwrap();
    let error = f
        .complete_for(&source.session, Request::CompareTransfer)
        .unwrap_err();
    assert!(error.contains("64 MiB"), "{error}");
    assert_eq!(huge.metadata().unwrap().len(), 65 * 1024 * 1024);
    std::fs::remove_file(huge).unwrap();
    assert!(f
        .complete_for(&source.session, Request::CompareTransfer)
        .is_ok());
}

#[test]
fn managed_worktree_transfer_committed_change_merges_nonoverlapping_dirty_destination() {
    let f = Fixture::new();
    let lines = (0..40).map(|i| format!("line {i}\n")).collect::<String>();
    std::fs::write(f.root.join("note.txt"), &lines).unwrap();
    git(&f.root, &["add", "note.txt"]);
    git(&f.root, &["commit", "-m", "Public multiline base"]);
    let source = f.create();
    let root = Path::new(&source.root);
    std::fs::write(
        root.join("note.txt"),
        lines.replace("line 1\n", "source line 1\n"),
    )
    .unwrap();
    git(root, &["add", "note.txt"]);
    git(root, &["commit", "-m", "Public source task"]);
    std::fs::write(
        f.root.join("note.txt"),
        lines.replace("line 39\n", "destination line 39\n"),
    )
    .unwrap();
    let index = std::fs::read(f.root.join(".git/index")).unwrap();
    let review = f
        .complete_for(
            &source.session,
            Request::ReviewTransfer {
                destination: f.source.clone(),
                paths: vec!["note.txt".into()],
            },
        )
        .unwrap();
    assert_eq!(review["receipt"]["applicable"], true, "{review}");
    assert_eq!(
        f.complete_for(
            &source.session,
            Request::ApplyTransfer {
                token: review["token"].as_str().unwrap().into()
            }
        )
        .unwrap()["applied"],
        true
    );
    let merged = std::fs::read_to_string(f.root.join("note.txt")).unwrap();
    assert!(merged.contains("source line 1\n"));
    assert!(merged.contains("destination line 39\n"));
    assert_eq!(std::fs::read(f.root.join(".git/index")).unwrap(), index);
    assert!(!std::fs::read_to_string(root.join("note.txt"))
        .unwrap()
        .contains("destination line 39\n"));
}
