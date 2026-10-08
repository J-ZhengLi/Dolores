use super::*;
fn setup() -> (tempfile::TempDir, Repo, CancellationToken) {
    let dir = tempfile::tempdir().unwrap();
    let cancel = CancellationToken::new();
    let git = git_process::executable(dir.path()).unwrap();
    for args in [
        vec!["init"],
        vec!["config", "user.name", "Fixture"],
        vec!["config", "user.email", "fixture@example.invalid"],
    ] {
        assert_eq!(
            git_process::run(
                &git,
                dir.path(),
                &args.iter().map(|s| s.to_string()).collect::<Vec<_>>(),
                &cancel,
                false
            )
            .unwrap()
            .code,
            0
        );
    }
    let repo = Repo::discover(dir.path().to_str().unwrap(), &cancel).unwrap();
    (dir, repo, cancel)
}
fn act(
    repo: &Repo,
    state: &Arc<Mutex<Registry>>,
    editor: &Arc<Mutex<crate::editor::Registry>>,
    cancel: &CancellationToken,
    op: Mutation,
) -> Result<Value, String> {
    let status = repo.status(cancel)?;
    let review = repo.review(status["revision"].as_str().unwrap(), op, state, cancel)?;
    repo.apply(review["token"].as_str().unwrap(), state, editor, cancel)
}
#[test]
fn selected_saved_paths_and_hooks_preserve_unrelated_work_and_commit_drafts() {
    let (dir, repo, cancel) = setup();
    let state = Arc::new(Mutex::new(Registry::default()));
    let editor = Arc::new(Mutex::new(crate::editor::Registry::default()));
    std::fs::write(dir.path().join("a.txt"), "a\n").unwrap();
    std::fs::write(dir.path().join("b.txt"), "b\n").unwrap();
    act(
        &repo,
        &state,
        &editor,
        &cancel,
        Mutation::Stage {
            paths: vec!["a.txt".into()],
        },
    )
    .unwrap();
    assert!(repo.status(&cancel).unwrap()["entries"]
        .as_array()
        .unwrap()
        .iter()
        .any(|e| e["path"] == "b.txt" && e["index"] == "?"));
    act(
        &repo,
        &state,
        &editor,
        &cancel,
        Mutation::Unstage {
            paths: vec!["a.txt".into()],
        },
    )
    .unwrap();
    assert!(dir.path().join("a.txt").exists());
    act(
        &repo,
        &state,
        &editor,
        &cancel,
        Mutation::Stage {
            paths: vec!["a.txt".into()],
        },
    )
    .unwrap();
    let hook = repo.git.join("hooks/pre-commit");
    std::fs::write(
        &hook,
        "#!/bin/sh\necho 'fixture hook refused' >&2\nexit 1\n",
    )
    .unwrap();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&hook, std::fs::Permissions::from_mode(0o755)).unwrap();
    }
    let refusal = act(
        &repo,
        &state,
        &editor,
        &cancel,
        Mutation::Commit {
            message: "Retained fixture message".into(),
        },
    )
    .unwrap_err();
    assert!(refusal.contains("hook refused"));
    assert!(repo.status(&cancel).unwrap()["head"].is_null());
    std::fs::remove_file(hook).unwrap();
    let result = act(
        &repo,
        &state,
        &editor,
        &cancel,
        Mutation::Commit {
            message: "Retained fixture message".into(),
        },
    )
    .unwrap();
    assert_eq!(result["completed"], true);
    assert_eq!(
        std::fs::read_to_string(dir.path().join("b.txt")).unwrap(),
        "b\n"
    );
    assert!(repo.command(&["show", "HEAD:b.txt"], &cancel).is_err());
    assert_eq!(
        author_identity("Fixture Person <f@example.invalid> 100 +0800"),
        "Fixture Person <f@example.invalid>"
    );
}
#[test]
fn stale_saved_bytes_consume_review_without_staging_new_content() {
    let (dir, repo, cancel) = setup();
    let state = Arc::new(Mutex::new(Registry::default()));
    let editor = Arc::new(Mutex::new(crate::editor::Registry::default()));
    std::fs::write(dir.path().join("a.txt"), "reviewed").unwrap();
    let status = repo.status(&cancel).unwrap();
    let preview = repo
        .review(
            status["revision"].as_str().unwrap(),
            Mutation::Stage {
                paths: vec!["a.txt".into()],
            },
            &state,
            &cancel,
        )
        .unwrap();
    std::fs::write(dir.path().join("a.txt"), "new outside bytes").unwrap();
    let token = preview["token"].as_str().unwrap();
    assert!(repo
        .apply(token, &state, &editor, &cancel)
        .unwrap_err()
        .contains("basis changed"));
    assert!(repo.apply(token, &state, &editor, &cancel).is_err());
    assert_eq!(repo.status(&cancel).unwrap()["entries"][0]["index"], "?");
}

#[test]
fn all_index_actions_cover_many_large_new_modified_deleted_and_renamed_files() {
    let (dir, repo, cancel) = setup();
    let state = Arc::new(Mutex::new(Registry::default()));
    let editor = Arc::new(Mutex::new(crate::editor::Registry::default()));
    for name in ["modified.txt", "deleted.txt", "rename.txt"] {
        std::fs::write(dir.path().join(name), "base\n").unwrap();
    }
    repo.command(&["add", "."], &cancel).unwrap();
    repo.command(&["commit", "-m", "base"], &cancel).unwrap();
    std::fs::write(dir.path().join("modified.txt"), "changed\n").unwrap();
    std::fs::remove_file(dir.path().join("deleted.txt")).unwrap();
    repo.command(&["mv", "rename.txt", "renamed.txt"], &cancel)
        .unwrap();
    std::fs::write(dir.path().join("renamed.txt"), "base\nnew line\n").unwrap();
    for i in 0..20 {
        std::fs::write(dir.path().join(format!("new-{i}.txt")), "new\n").unwrap();
    }
    std::fs::write(dir.path().join("large.txt"), "large line\n".repeat(60_000)).unwrap();
    act(&repo, &state, &editor, &cancel, Mutation::StageAll).unwrap();
    let status = repo.status(&cancel).unwrap();
    assert!(status["entries"]
        .as_array()
        .unwrap()
        .iter()
        .all(|e| e["worktree"] == " " && e["index"] != "?"));
    act(&repo, &state, &editor, &cancel, Mutation::UnstageAll).unwrap();
    assert!(repo
        .command(&["diff", "--cached", "--name-only"], &cancel)
        .unwrap()
        .is_empty());
    assert!(dir.path().join("large.txt").exists());
    assert!(!dir.path().join("deleted.txt").exists());
    assert!(dir.path().join("renamed.txt").exists());
}

#[test]
fn automatic_commit_review_does_not_stage_until_apply_and_preserves_staged_selection() {
    let (dir, repo, cancel) = setup();
    let state = Arc::new(Mutex::new(Registry::default()));
    let editor = Arc::new(Mutex::new(crate::editor::Registry::default()));
    std::fs::write(dir.path().join("a.txt"), "saved a\n").unwrap();
    std::fs::write(dir.path().join("b.txt"), "saved b\n").unwrap();
    let status = repo.status(&cancel).unwrap();
    let review = repo
        .review(
            status["revision"].as_str().unwrap(),
            Mutation::CommitAll {
                message: "automatic initial commit".into(),
            },
            &state,
            &cancel,
        )
        .unwrap();
    assert_eq!(review["autoStage"], true);
    assert!(review["patch"].as_str().unwrap().contains("+saved a"));
    assert!(review["patch"].as_str().unwrap().contains("+saved b"));
    assert_eq!(
        repo.status(&cancel).unwrap()["revision"],
        status["revision"]
    );
    repo.apply(review["token"].as_str().unwrap(), &state, &editor, &cancel)
        .unwrap();
    assert_eq!(
        repo.command(&["show", "HEAD:b.txt"], &cancel).unwrap(),
        b"saved b\n"
    );
    std::fs::write(dir.path().join("a.txt"), "selected a\n").unwrap();
    repo.command(&["add", "a.txt"], &cancel).unwrap();
    std::fs::write(dir.path().join("a.txt"), "later working a\n").unwrap();
    std::fs::write(dir.path().join("b.txt"), "working b\n").unwrap();
    let status = repo.status(&cancel).unwrap();
    let review = repo
        .review(
            status["revision"].as_str().unwrap(),
            Mutation::CommitAll {
                message: "only selected index".into(),
            },
            &state,
            &cancel,
        )
        .unwrap();
    assert_eq!(review["autoStage"], false);
    assert_eq!(review["paths"], json!(["a.txt"]));
    repo.apply(review["token"].as_str().unwrap(), &state, &editor, &cancel)
        .unwrap();
    assert_eq!(
        repo.command(&["show", "HEAD:a.txt"], &cancel).unwrap(),
        b"selected a\n"
    );
    assert_eq!(
        repo.command(&["show", "HEAD:b.txt"], &cancel).unwrap(),
        b"saved b\n"
    );
    assert_eq!(
        std::fs::read(dir.path().join("a.txt")).unwrap(),
        b"later working a\n"
    );
}

#[test]
fn automatic_commit_stale_review_and_hook_failure_recover_without_losing_work() {
    let (dir, repo, cancel) = setup();
    let state = Arc::new(Mutex::new(Registry::default()));
    let editor = Arc::new(Mutex::new(crate::editor::Registry::default()));
    std::fs::write(dir.path().join("a.txt"), "reviewed\n").unwrap();
    let status = repo.status(&cancel).unwrap();
    let review = repo
        .review(
            status["revision"].as_str().unwrap(),
            Mutation::CommitAll {
                message: "keep draft".into(),
            },
            &state,
            &cancel,
        )
        .unwrap();
    std::fs::write(dir.path().join("a.txt"), "new outside bytes\n").unwrap();
    assert!(repo
        .apply(review["token"].as_str().unwrap(), &state, &editor, &cancel)
        .unwrap_err()
        .contains("basis changed"));
    assert!(repo
        .command(&["diff", "--cached", "--name-only"], &cancel)
        .unwrap()
        .is_empty());
    let hook = repo.git.join("hooks/pre-commit");
    std::fs::write(
        &hook,
        "#!/bin/sh\necho 'automatic fixture refused' >&2\nexit 1\n",
    )
    .unwrap();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&hook, std::fs::Permissions::from_mode(0o755)).unwrap();
    }
    let error = act(
        &repo,
        &state,
        &editor,
        &cancel,
        Mutation::CommitAll {
            message: "keep draft".into(),
        },
    )
    .unwrap_err();
    assert!(error.contains("automatic fixture refused"));
    assert!(repo.status(&cancel).unwrap()["head"].is_null());
    assert_eq!(
        repo.command(&["show", ":a.txt"], &cancel).unwrap(),
        b"new outside bytes\n"
    );
    std::fs::remove_file(hook).unwrap();
    act(
        &repo,
        &state,
        &editor,
        &cancel,
        Mutation::CommitAll {
            message: "keep draft".into(),
        },
    )
    .unwrap();
    assert_eq!(
        repo.command(&["show", "HEAD:a.txt"], &cancel).unwrap(),
        b"new outside bytes\n"
    );
}
