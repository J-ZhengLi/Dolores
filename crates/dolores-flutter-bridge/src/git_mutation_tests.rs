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
