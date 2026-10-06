use super::*;
struct Fixture {
    dir: tempfile::TempDir,
    repo: Repo,
    cancel: CancellationToken,
    registry: Arc<Mutex<Registry>>,
    editor: Arc<Mutex<crate::editor::Registry>>,
}
impl Fixture {
    fn new() -> Self {
        let dir = tempfile::tempdir().unwrap();
        let cancel = CancellationToken::new();
        let executable = git_process::executable(dir.path()).unwrap();
        for args in [
            vec!["init"],
            vec!["config", "user.name", "Fixture"],
            vec!["config", "user.email", "fixture@example.invalid"],
            vec!["config", "core.autocrlf", "false"],
        ] {
            let out = git_process::run(
                &executable,
                dir.path(),
                &args.iter().map(|s| s.to_string()).collect::<Vec<_>>(),
                &cancel,
                false,
            )
            .unwrap();
            assert_eq!(out.code, 0);
        }
        let repo = Repo::discover(dir.path().to_str().unwrap(), &cancel).unwrap();
        Self {
            dir,
            repo,
            cancel,
            registry: Arc::new(Mutex::new(Registry::default())),
            editor: Arc::new(Mutex::new(crate::editor::Registry::default())),
        }
    }
    fn write(&self, path: &str, text: &str) {
        std::fs::write(self.dir.path().join(path), text).unwrap();
    }
    fn git(&self, args: &[&str]) -> Vec<u8> {
        self.repo.command(args, &self.cancel).unwrap()
    }
    fn act(&self, op: Mutation) -> Result<Value, String> {
        let s = self.repo.status(&self.cancel)?;
        let r = self.repo.review(
            s["revision"].as_str().unwrap(),
            op,
            &self.registry,
            &self.cancel,
        )?;
        self.repo.apply(
            r["token"].as_str().unwrap(),
            &self.registry,
            &self.editor,
            &self.cancel,
        )
    }
    fn base(&self) {
        self.write("a.txt", "base\n");
        self.git(&["add", "."]);
        self.git(&["commit", "-m", "base"]);
    }
}
#[test]
fn selected_hunks_are_host_derived_and_stale_ids_refuse() {
    let f = Fixture::new();
    let base = (0..30).map(|n| format!("line{n}\n")).collect::<String>();
    f.write("a.txt", &base);
    f.git(&["add", "."]);
    f.git(&["commit", "-m", "base"]);
    let changed = base
        .replace("line1\n", "changed1\n")
        .replace("line25\n", "changed25\n");
    f.write("a.txt", &changed);
    let s = f.repo.status(&f.cancel).unwrap();
    let d = f
        .repo
        .diff(
            s["revision"].as_str().unwrap(),
            "a.txt",
            "working",
            None,
            &f.cancel,
        )
        .unwrap();
    assert_eq!(d["hunks"].as_array().unwrap().len(), 2);
    let id = d["hunks"][0]["id"].as_str().unwrap().to_string();
    f.act(Mutation::Hunks {
        path: "a.txt".into(),
        staged: false,
        ids: vec![id.clone()],
    })
    .unwrap();
    let index = String::from_utf8(f.git(&["show", ":a.txt"])).unwrap();
    assert!(index.contains("changed1"));
    assert!(!index.contains("changed25"));
    assert!(f
        .act(Mutation::Hunks {
            path: "a.txt".into(),
            staged: false,
            ids: vec![id]
        })
        .unwrap_err()
        .contains("basis changed"));
    let s = f.repo.status(&f.cancel).unwrap();
    let d = f
        .repo
        .diff(
            s["revision"].as_str().unwrap(),
            "a.txt",
            "staged",
            None,
            &f.cancel,
        )
        .unwrap();
    f.act(Mutation::Hunks {
        path: "a.txt".into(),
        staged: true,
        ids: vec![d["hunks"][0]["id"].as_str().unwrap().into()],
    })
    .unwrap();
    assert_eq!(String::from_utf8(f.git(&["show", ":a.txt"])).unwrap(), base);
    assert_eq!(
        std::fs::read_to_string(f.dir.path().join("a.txt")).unwrap(),
        changed
    );
}
#[test]
fn selected_stash_branch_discard_and_revert_preserve_unrelated_work() {
    let f = Fixture::new();
    f.base();
    f.write("a.txt", "stash\n");
    f.write("untracked.txt", "keep\n");
    f.act(Mutation::StashCreate {
        paths: vec!["a.txt".into()],
        message: "selected".into(),
    })
    .unwrap();
    assert_eq!(
        std::fs::read_to_string(f.dir.path().join("a.txt")).unwrap(),
        "base\n"
    );
    assert!(f.dir.path().join("untracked.txt").exists());
    let stash = f.repo.local_state(&f.cancel).unwrap()["stashes"][0]["id"]
        .as_str()
        .unwrap()
        .to_string();
    f.act(Mutation::StashApply { stash, pop: true }).unwrap();
    assert!(f.repo.local_state(&f.cancel).unwrap()["stashes"]
        .as_array()
        .unwrap()
        .is_empty());
    assert!(f
        .act(Mutation::BranchSwitch {
            name: "missing".into()
        })
        .is_err());
    f.act(Mutation::Discard {
        path: "a.txt".into(),
    })
    .unwrap();
    f.act(Mutation::BranchCreate {
        name: "other".into(),
    })
    .unwrap();
    f.act(Mutation::BranchSwitch {
        name: "other".into(),
    })
    .unwrap();
    assert_eq!(f.repo.status(&f.cancel).unwrap()["branch"], "other");
    f.write("a.txt", "new\n");
    f.git(&["add", "a.txt"]);
    f.git(&["commit", "-m", "change"]);
    let commit = String::from_utf8(f.git(&["rev-parse", "HEAD"]))
        .unwrap()
        .trim()
        .to_string();
    std::fs::remove_file(f.dir.path().join("untracked.txt")).unwrap();
    f.act(Mutation::Revert {
        commit: commit.clone(),
    })
    .unwrap();
    assert_ne!(f.repo.status(&f.cancel).unwrap()["head"], commit);
    assert_eq!(
        std::fs::read_to_string(f.dir.path().join("a.txt")).unwrap(),
        "base\n"
    );
}
#[test]
fn conflicting_pop_retains_exact_stash_and_conflicting_revert_can_abort() {
    let f = Fixture::new();
    f.base();
    f.write("a.txt", "stashed\n");
    f.act(Mutation::StashCreate {
        paths: vec!["a.txt".into()],
        message: "keep on conflict".into(),
    })
    .unwrap();
    let stash = f.repo.local_state(&f.cancel).unwrap()["stashes"][0]["id"]
        .as_str()
        .unwrap()
        .to_string();
    f.write("a.txt", "changed\n");
    f.git(&["add", "."]);
    f.git(&["commit", "-m", "changed"]);
    assert!(f
        .act(Mutation::StashApply {
            stash: stash.clone(),
            pop: true
        })
        .is_err());
    assert_eq!(
        f.repo.local_state(&f.cancel).unwrap()["stashes"][0]["id"],
        stash
    );
    f.git(&[
        "restore",
        "--source=HEAD",
        "--staged",
        "--worktree",
        "a.txt",
    ]);
    let old = String::from_utf8(f.git(&["rev-parse", "HEAD"]))
        .unwrap()
        .trim()
        .to_string();
    f.write("a.txt", "later\n");
    f.git(&["add", "."]);
    f.git(&["commit", "-m", "later"]);
    let before = f.repo.status(&f.cancel).unwrap()["head"].clone();
    assert!(f.act(Mutation::Revert { commit: old }).is_err());
    assert_eq!(f.repo.status(&f.cancel).unwrap()["reverting"], true);
    f.act(Mutation::RevertAbort).unwrap();
    assert_eq!(f.repo.status(&f.cancel).unwrap()["head"], before);
    assert_eq!(
        std::fs::read_to_string(f.dir.path().join("a.txt")).unwrap(),
        "later\n"
    );
}
