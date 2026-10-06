use super::*;
struct Fixture {
    dir: tempfile::TempDir,
    repo: Repo,
    cancel: CancellationToken,
    state: Arc<Mutex<Registry>>,
    editor: Arc<Mutex<crate::editor::Registry>>,
    bare: PathBuf,
}
impl Fixture {
    fn new() -> Self {
        let dir = tempfile::tempdir().unwrap();
        let cancel = CancellationToken::new();
        let executable = git_process::executable(dir.path()).unwrap();
        let run = |args: &[&str]| {
            let o = git_process::run(
                &executable,
                dir.path(),
                &args.iter().map(|s| s.to_string()).collect::<Vec<_>>(),
                &cancel,
                false,
            )
            .unwrap();
            assert_eq!(o.code, 0, "{}", String::from_utf8_lossy(&o.stderr));
        };
        run(&["init", "-b", "main"]);
        run(&["config", "user.name", "Fixture"]);
        run(&["config", "user.email", "fixture@example.invalid"]);
        run(&["config", "core.autocrlf", "false"]);
        let bare = tempfile::tempdir().unwrap().keep();
        run(&["init", "--bare", bare.to_str().unwrap()]);
        run(&["remote", "add", "origin", bare.to_str().unwrap()]);
        run(&["config", "branch.main.remote", "origin"]);
        run(&["config", "branch.main.merge", "refs/heads/main"]);
        let repo = Repo::discover(dir.path().to_str().unwrap(), &cancel).unwrap();
        Self {
            dir,
            repo,
            cancel,
            state: Arc::new(Mutex::new(Registry::default())),
            editor: Arc::new(Mutex::new(crate::editor::Registry::default())),
            bare,
        }
    }
    fn commit(&self, text: &str) {
        std::fs::write(self.dir.path().join("a.txt"), text).unwrap();
        self.repo.command(&["add", "a.txt"], &self.cancel).unwrap();
        self.repo
            .command(&["commit", "-m", "fixture"], &self.cancel)
            .unwrap();
    }
    fn review(&self, op: Mutation) -> Value {
        let s = self.repo.status(&self.cancel).unwrap();
        self.repo
            .review(
                s["revision"].as_str().unwrap(),
                op,
                &self.state,
                &self.cancel,
            )
            .unwrap()
    }
    fn apply(&self, r: &Value) -> Result<Value, String> {
        self.repo.apply(
            r["token"].as_str().unwrap(),
            &self.state,
            &self.editor,
            &self.cancel,
        )
    }
    fn act(&self, op: Mutation) -> Result<Value, String> {
        self.apply(&self.review(op))
    }
    fn push() -> Mutation {
        Mutation::Push {
            remote: "origin".into(),
            branch: "refs/heads/main".into(),
        }
    }
    fn pull() -> Mutation {
        Mutation::Pull {
            remote: "origin".into(),
            branch: "refs/heads/main".into(),
        }
    }
    fn remote_commit(&self, text: &str) {
        let peer = self.dir.path().join("peer");
        if !peer.exists() {
            self.repo
                .command(
                    &[
                        "clone",
                        "--branch",
                        "main",
                        self.bare.to_str().unwrap(),
                        peer.to_str().unwrap(),
                    ],
                    &self.cancel,
                )
                .unwrap();
            std::fs::write(self.repo.git.join("info/exclude"), "peer/\n").unwrap();
        }
        let git = |args: &[&str]| {
            let o = git_process::run(
                &self.repo.executable,
                &peer,
                &args.iter().map(|s| s.to_string()).collect::<Vec<_>>(),
                &self.cancel,
                false,
            )
            .unwrap();
            assert_eq!(o.code, 0, "{}", String::from_utf8_lossy(&o.stderr));
        };
        git(&["config", "user.name", "Peer"]);
        git(&["config", "user.email", "peer@example.invalid"]);
        std::fs::write(peer.join("a.txt"), text).unwrap();
        git(&["add", "a.txt"]);
        git(&["commit", "-m", "peer"]);
        git(&["push", "origin", "main"]);
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        std::fs::remove_dir_all(&self.bare).unwrap();
    }
}
#[test]
fn reviewed_push_fetch_and_fast_forward_pull_use_exact_local_remote() {
    let f = Fixture::new();
    f.commit("base\n");
    f.act(Fixture::push()).unwrap();
    assert_eq!(
        f.repo
            .remote_head("origin", "refs/heads/main", &f.cancel)
            .unwrap()
            .as_deref(),
        f.repo.status(&f.cancel).unwrap()["head"].as_str()
    );
    f.remote_commit("remote\n");
    f.act(Mutation::Fetch {
        remote: "origin".into(),
    })
    .unwrap();
    assert_eq!(
        std::fs::read_to_string(f.dir.path().join("a.txt")).unwrap(),
        "base\n"
    );
    assert_eq!(f.repo.remote_state(&f.cancel).unwrap()["behind"], 1);
    f.act(Fixture::pull()).unwrap();
    assert_eq!(
        std::fs::read_to_string(f.dir.path().join("a.txt")).unwrap(),
        "remote\n"
    );
}
#[test]
fn changed_remote_and_divergence_refuse_without_overwriting_local_work() {
    let f = Fixture::new();
    f.commit("base\n");
    f.act(Fixture::push()).unwrap();
    f.commit("local\n");
    let r = f.review(Fixture::push());
    f.remote_commit("peer\n");
    assert!(f.apply(&r).unwrap_err().contains("Remote ref changed"));
    let before = f.repo.status(&f.cancel).unwrap()["head"].clone();
    assert!(f.act(Fixture::pull()).unwrap_err().contains("diverge"));
    assert_eq!(f.repo.status(&f.cancel).unwrap()["head"], before);
    assert_eq!(
        std::fs::read_to_string(f.dir.path().join("a.txt")).unwrap(),
        "local\n"
    );
}
#[test]
fn uncertain_push_reconciles_completed_ref_before_another_review() {
    let f = Fixture::new();
    f.commit("base\n");
    let r = f.review(Fixture::push());
    let basis = f
        .state
        .lock()
        .unwrap()
        .reviews
        .values()
        .next()
        .unwrap()
        .remote_basis
        .clone()
        .unwrap();
    f.apply(&r).unwrap();
    f.state.lock().unwrap().uncertain.insert(f.repo.id(), basis);
    let s = f.repo.status(&f.cancel).unwrap();
    let err = f
        .repo
        .review(
            s["revision"].as_str().unwrap(),
            Fixture::push(),
            &f.state,
            &f.cancel,
        )
        .unwrap_err();
    assert!(err.contains("was not repeated"));
    assert!(f.state.lock().unwrap().uncertain.is_empty());
}
