use crate::{git_process, Engine};
use serde::Deserialize;
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
};
use tokio_util::sync::CancellationToken;
#[path = "git_diff.rs"]
mod diff;
#[path = "git_mutation.rs"]
mod mutation;

#[derive(Deserialize)]
#[serde(tag = "action", rename_all = "camelCase", deny_unknown_fields)]
pub(crate) enum Request {
    Status,
    Diff {
        repo: String,
        revision: String,
        path: String,
        basis: String,
        commit: Option<String>,
    },
    History {
        repo: String,
        head: String,
        cursor: usize,
    },
    CommitFiles {
        repo: String,
        commit: String,
    },
    Poll {
        job: String,
    },
    Cancel {
        job: String,
    },
    Review {
        repo: String,
        revision: String,
        operation: mutation::Mutation,
    },
    Apply {
        repo: String,
        token: String,
    },
    DiscardReview {
        repo: String,
        token: String,
    },
}
struct Job {
    session: String,
    cancel: CancellationToken,
    result: Option<Result<Value, String>>,
}
#[derive(Default)]
pub(crate) struct Registry {
    jobs: BTreeMap<String, Job>,
    pub(super) active: BTreeMap<String, String>,
    closed: bool,
    lanes: BTreeMap<String, Arc<Mutex<()>>>,
    reviews: BTreeMap<String, mutation::Review>,
    mutations: BTreeMap<String, PathBuf>,
}
impl Registry {
    pub fn mutation_overlaps(&self, root: &Path) -> bool {
        self.mutations
            .values()
            .any(|r| root.starts_with(r) || r.starts_with(root))
    }
    pub fn review_root(&self, id: &str) -> Option<PathBuf> {
        self.reviews.get(id).map(|r| r.root.clone())
    }
    pub fn stop(&self) {
        for job in self.jobs.values() {
            job.cancel.cancel();
        }
    }
}
#[derive(Clone)]
struct Repo {
    project: PathBuf,
    root: PathBuf,
    git: PathBuf,
    executable: PathBuf,
}
impl Repo {
    fn command(&self, args: &[&str], cancel: &CancellationToken) -> Result<Vec<u8>, String> {
        let output = git_process::run(
            &self.executable,
            &self.root,
            &args.iter().map(|s| s.to_string()).collect::<Vec<_>>(),
            cancel,
            false,
        )?;
        if output.code != 0 {
            return Err(format!(
                "Git failed ({}): {}. Refresh after checking repository state.",
                output.code,
                String::from_utf8_lossy(&output.stderr)
            ));
        }
        Ok(output.stdout)
    }
    fn discover(project: &str, cancel: &CancellationToken) -> Result<Self, String> {
        let project = Path::new(project).canonicalize().map_err(|_| {
            "Project folder is unavailable. Return to Home and choose an existing folder."
        })?;
        let executable = git_process::executable(&project)?;
        let mut repo = Self {
            root: project.clone(),
            project,
            git: PathBuf::new(),
            executable,
        };
        let root=repo.command(&["rev-parse","--show-toplevel"],cancel).map_err(|_|"This folder is not a Git worktree. Open an existing repository from Home, or initialize it deliberately using external Git.")?;
        repo.root = PathBuf::from(text(&root)?.trim())
            .canonicalize()
            .map_err(|_| "Git worktree root is unavailable.")?;
        let git = repo.command(&["rev-parse", "--absolute-git-dir"], cancel)?;
        repo.git = PathBuf::from(text(&git)?.trim())
            .canonicalize()
            .map_err(|_| "Git metadata is unavailable.")?;
        Ok(repo)
    }
    fn id(&self) -> String {
        hash(format!("{}\0{}", self.root.display(), self.git.display()).as_bytes())
    }
    fn status(&self, cancel: &CancellationToken) -> Result<Value, String> {
        let raw = self.command(
            &["status", "--porcelain=v1", "-z", "--untracked-files=all"],
            cancel,
        )?;
        let entries = parse_status(&raw)?;
        let head = self
            .command(&["rev-parse", "--verify", "HEAD"], cancel)
            .ok()
            .map(|s| String::from_utf8_lossy(&s).trim().to_string());
        let branch = self
            .command(&["symbolic-ref", "--quiet", "--short", "HEAD"], cancel)
            .ok()
            .map(|s| String::from_utf8_lossy(&s).trim().to_string());
        let mut digest = Sha256::new();
        digest.update(&raw);
        digest.update(head.as_deref().unwrap_or("unborn").as_bytes());
        digest.update(self.command(&["config", "--null", "--list", "--show-origin"], cancel)?);
        let mut total = 0u64;
        for entry in &entries {
            let path = entry["path"].as_str().ok_or("Invalid Git path.")?;
            match self.path(path) {
                Ok(file) if file.is_file() => {
                    let bytes = file
                        .metadata()
                        .map_err(|_| "Changed file is unavailable. Refresh.")?
                        .len();
                    total += bytes;
                    if bytes > 16 * 1024 * 1024 || total > 32 * 1024 * 1024 {
                        return Err("Changed files exceed the 16 MiB file / 32 MiB revision limit. Use external Git, then Refresh.".into());
                    }
                    digest.update(
                        std::fs::read(file).map_err(|_| "Changed file is unavailable. Refresh.")?,
                    );
                }
                Ok(_) => digest.update(b"missing-or-directory"),
                Err(_) => digest.update(b"restricted-alias"),
            }
        }
        let index = self.git.join("index");
        if index.exists() {
            if index.metadata().map_err(|_| "Index is unavailable.")?.len() > 16 * 1024 * 1024 {
                return Err("Index exceeds the 16 MiB initial limit. Use external Git.".into());
            }
            digest.update(std::fs::read(index).map_err(|_| "Index is unavailable.")?);
        }
        Ok(
            json!({"repo":self.id(),"root":self.root,"projectRoot":self.project,"head":head,"branch":branch,"entries":entries,"revision":format!("{:x}",digest.finalize())}),
        )
    }
    fn path(&self, path: &str) -> Result<PathBuf, String> {
        if path.is_empty()
            || path.len() > 4096
            || path.contains(['\\', '\0'])
            || Path::new(path).is_absolute()
        {
            return Err("Unsupported Git path. No file was changed.".into());
        }
        let mut file = self.root.clone();
        for part in path.split('/') {
            if part.is_empty()
                || part == "."
                || part == ".."
                || part.eq_ignore_ascii_case(".git")
                || (cfg!(windows) && part.contains(':'))
            {
                return Err("Git path is outside the worktree or is metadata.".into());
            }
            file.push(part);
            if let Ok(meta) = std::fs::symlink_metadata(&file) {
                #[cfg(windows)]
                let alias = {
                    use std::os::windows::fs::MetadataExt;
                    meta.file_attributes() & 0x400 != 0
                };
                #[cfg(not(windows))]
                let alias = false;
                if meta.file_type().is_symlink() || alias {
                    return Err("Linked paths are read-only in Source Control. Use external Git deliberately.".into());
                }
            }
        }
        Ok(file)
    }
    fn check(&self, id: &str) -> Result<(), String> {
        if id != self.id() {
            Err("Repository changed. Return to its Home conversation and Refresh.".into())
        } else {
            Ok(())
        }
    }
    fn revision(&self, revision: &str, cancel: &CancellationToken) -> Result<Value, String> {
        let status = self.status(cancel)?;
        if status["revision"] != revision {
            return Err(
                "Git basis changed. Refresh and review the new state; no action was performed."
                    .into(),
            );
        }
        Ok(status)
    }
}
fn text(bytes: &[u8]) -> Result<&str, String> {
    std::str::from_utf8(bytes).map_err(|_|"Git returned a filename or text not encoded as UTF-8. Inspect it using external Git; no path was changed.".into())
}
fn hash(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}
fn parse_status(raw: &[u8]) -> Result<Vec<Value>, String> {
    let mut records = raw.split(|b| *b == 0).filter(|r| !r.is_empty());
    let mut result = vec![];
    while let Some(record) = records.next() {
        if record.len() < 4 || record[2] != b' ' {
            return Err("Malformed Git status. Refresh; no action was performed.".into());
        }
        let xy = text(&record[..2])?;
        if !xy.bytes().all(|b| b" MADRCUT?!".contains(&b)) {
            return Err("Malformed Git status codes. Refresh.".into());
        }
        let path = text(&record[3..])?;
        let old = if xy.contains(['R', 'C']) {
            Some(text(
                records.next().ok_or("Incomplete Git rename record.")?,
            )?)
        } else {
            None
        };
        result.push(json!({"path":path,"oldPath":old,"index":&xy[..1],"worktree":&xy[1..],"conflict":matches!(xy,"DD"|"AU"|"UD"|"UA"|"DU"|"AA"|"UU")}));
        if result.len() > 2000 {
            return Err("More than 2,000 changes. Use external Git to reduce the set, then Refresh; no partial status was accepted.".into());
        }
    }
    Ok(result)
}
impl Engine {
    pub(crate) fn stop_git(&self) -> Result<(), String> {
        {
            let mut state = self
                .git
                .lock()
                .map_err(|_| "Source Control is unavailable.")?;
            state.closed = true;
            state.stop();
        }
        let start = std::time::Instant::now();
        while !self
            .git
            .lock()
            .map_err(|_| "Source Control is unavailable.")?
            .active
            .is_empty()
        {
            if start.elapsed() > std::time::Duration::from_secs(2) {
                return Err(
                    "Git cleanup has not finished. Keep Dolores open and retry Close.".into(),
                );
            }
            std::thread::sleep(std::time::Duration::from_millis(5));
        }
        Ok(())
    }
    pub(crate) fn git_call(&self, session: &str, request: Request) -> Result<Value, String> {
        let mut state = self
            .git
            .lock()
            .map_err(|_| "Source Control is unavailable.")?;
        if state.closed {
            return Err("Source Control is closing. Restart Dolores before new operations.".into());
        }
        match request {
            Request::Poll { job } => {
                let value = state
                    .jobs
                    .get_mut(&job)
                    .filter(|j| j.session == session)
                    .ok_or("Git job belongs to another conversation or has expired.")?;
                if let Some(result) = value.result.take() {
                    state.jobs.remove(&job);
                    result.map(|value| json!({"done":true,"value":value}))
                } else {
                    Ok(json!({"done":false}))
                }
            }
            Request::Cancel { job } => {
                state
                    .jobs
                    .get(&job)
                    .filter(|j| j.session == session)
                    .ok_or("Git job is no longer owned by this conversation.")?
                    .cancel
                    .cancel();
                Ok(Value::Null)
            }
            operation => {
                let root = self
                    .store
                    .workspace(session)?
                    .root
                    .ok_or("Select a project conversation on Home or open a folder.")?;
                if state.jobs.len() >= 8
                    || state.active.len() >= 2
                    || state.active.contains_key(&root)
                {
                    return Err(
                        "Source Control is busy. Wait for its current operation, then Retry."
                            .into(),
                    );
                }
                let id = uuid::Uuid::new_v4().to_string();
                if let Request::Apply { repo, .. } = &operation {
                    let review = state
                        .reviews
                        .get(repo)
                        .ok_or("Git review expired. Review again.")?;
                    let mutation_root = review.root.clone();
                    state.mutations.insert(id.clone(), mutation_root);
                }
                let cancel = CancellationToken::new();
                state.active.insert(root.clone(), id.clone());
                state.jobs.insert(
                    id.clone(),
                    Job {
                        session: session.into(),
                        cancel: cancel.clone(),
                        result: None,
                    },
                );
                let registry = self.git.clone();
                let editor = self.editor.clone();
                let owned = id.clone();
                std::thread::spawn(move || {
                    let result = Repo::discover(&root, &cancel).and_then(|repo| {
                        let lane = {
                            let mut state = registry
                                .lock()
                                .map_err(|_| "Source Control is unavailable.")?;
                            state.lanes.retain(|_, lane| Arc::strong_count(lane) > 1);
                            state.lanes.entry(repo.id()).or_default().clone()
                        };
                        let _guard = lane
                            .try_lock()
                            .map_err(|_| "This worktree is busy. Wait, then Refresh.")?;
                        match operation {
                            Request::Status => repo.status(&cancel),
                            Request::Diff {
                                repo: id,
                                revision,
                                path,
                                basis,
                                commit,
                            } => {
                                repo.check(&id)?;
                                repo.diff(&revision, &path, &basis, commit.as_deref(), &cancel)
                            }
                            Request::History {
                                repo: id,
                                head,
                                cursor,
                            } => {
                                repo.check(&id)?;
                                repo.history(&head, cursor, &cancel)
                            }
                            Request::CommitFiles { repo: id, commit } => {
                                repo.check(&id)?;
                                repo.commit_files(&commit, &cancel)
                            }
                            Request::Review {
                                repo: id,
                                revision,
                                operation,
                            } => {
                                repo.check(&id)?;
                                repo.review(&revision, operation, &registry, &cancel)
                            }
                            Request::Apply { repo: id, token } => {
                                repo.check(&id)?;
                                repo.apply(&token, &registry, &editor, &cancel)
                            }
                            Request::DiscardReview { repo: id, token } => {
                                repo.check(&id)?;
                                let mut state =
                                    registry.lock().map_err(|_| "Git reviews unavailable.")?;
                                if state.reviews.get(&id).is_some_and(|r| r.token == token) {
                                    state.reviews.remove(&id);
                                }
                                Ok(Value::Null)
                            }
                            _ => unreachable!(),
                        }
                    });
                    if let Ok(mut state) = registry.lock() {
                        state.active.remove(&root);
                        state.mutations.remove(&owned);
                        if let Some(job) = state.jobs.get_mut(&owned) {
                            job.result = Some(result);
                        }
                    }
                });
                Ok(json!({"job":id}))
            }
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn nul_paths_and_rename_pairs_are_exact() {
        let rows = parse_status(
            b" M space \xe4\xb8\xad.txt\0R  new.txt\0old.txt\0?? -option.txt\0UU conflict\0",
        )
        .unwrap();
        assert_eq!(rows[0]["path"], "space 中.txt");
        assert_eq!(rows[1]["oldPath"], "old.txt");
        assert_eq!(rows[2]["path"], "-option.txt");
        assert_eq!(rows[3]["conflict"], true);
        assert!(parse_status(b"R  new\0").is_err());
        assert!(parse_status(b"?? \xff\0").is_err());
    }
    #[test]
    fn real_repository_and_missing_repo_have_distinct_recovery() {
        let dir = tempfile::tempdir().unwrap();
        let cancel = CancellationToken::new();
        assert!(Repo::discover(dir.path().to_str().unwrap(), &cancel)
            .err()
            .unwrap()
            .contains("not a Git"));
        let git = git_process::executable(dir.path()).unwrap();
        git_process::run(&git, dir.path(), &["init".into()], &cancel, false).unwrap();
        std::fs::write(dir.path().join("space 中.txt"), "hello").unwrap();
        let repo = Repo::discover(dir.path().to_str().unwrap(), &cancel).unwrap();
        let state = repo.status(&cancel).unwrap();
        assert_eq!(state["entries"][0]["path"], "space 中.txt");
        assert!(state["head"].is_null());
        cancel.cancel();
        assert!(repo.status(&cancel).is_err());
    }
}
