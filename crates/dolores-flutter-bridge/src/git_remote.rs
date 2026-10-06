use super::*;
#[derive(Clone)]
pub(in crate::source_control) struct Basis {
    remote: String,
    branch: String,
    head: String,
    remote_head: Option<String>,
}
impl Repo {
    pub(in crate::source_control) fn remote_state(
        &self,
        cancel: &CancellationToken,
    ) -> Result<Value, String> {
        let remotes = text(&self.command(&["remote"], cancel)?)?
            .lines()
            .take(32)
            .map(str::to_string)
            .collect::<Vec<_>>();
        let branch = self
            .command(&["symbolic-ref", "--quiet", "--short", "HEAD"], cancel)
            .ok()
            .and_then(|b| String::from_utf8(b).ok())
            .map(|s| s.trim().to_string());
        let get = |key: &str| {
            self.command(&["config", "--get", key], cancel)
                .ok()
                .and_then(|b| String::from_utf8(b).ok())
                .map(|s| s.trim().to_string())
        };
        let remote = branch
            .as_ref()
            .and_then(|b| get(&format!("branch.{b}.remote")))
            .filter(|r| r != "." && remotes.contains(r));
        let target = branch
            .as_ref()
            .and_then(|b| get(&format!("branch.{b}.merge")));
        let counts = self
            .command(
                &["rev-list", "--left-right", "--count", "HEAD...@{upstream}"],
                cancel,
            )
            .ok()
            .and_then(|b| String::from_utf8(b).ok())
            .and_then(|s| {
                let n = s
                    .split_whitespace()
                    .map(str::parse::<u64>)
                    .collect::<Result<Vec<_>, _>>()
                    .ok()?;
                if n.len() == 2 {
                    Some(n)
                } else {
                    None
                }
            });
        Ok(
            json!({"remotes":remotes,"remote":remote,"branch":target,"ahead":counts.as_ref().map(|v|v[0]),"behind":counts.as_ref().map(|v|v[1])}),
        )
    }
    fn remote_target(
        &self,
        remote: &str,
        branch: &str,
        cancel: &CancellationToken,
    ) -> Result<(), String> {
        if remote.starts_with('-')
            || !self.remote_state(cancel)?["remotes"]
                .as_array()
                .unwrap()
                .iter()
                .any(|r| r == remote)
        {
            return Err("Choose a configured remote. Set credentials and tracking with external Git, then Refresh.".into());
        }
        if !branch.is_empty() {
            if !branch.starts_with("refs/heads/") {
                return Err("Remote target must be a branch ref.".into());
            }
            self.command(&["check-ref-format", branch], cancel)?;
        }
        Ok(())
    }
    fn remote_run(
        &self,
        args: &[&str],
        cancel: &CancellationToken,
    ) -> Result<git_process::Output, String> {
        let output = git_process::run(
            &self.executable,
            &self.root,
            &args.iter().map(|s| s.to_string()).collect::<Vec<_>>(),
            cancel,
            true,
        )?;
        if output.code != 0 {
            return Err("Remote Git refused or could not complete the request. Check authentication, network and branch divergence using external Git; Refresh and review again. Effects may remain.".into());
        }
        // Remote/helper diagnostics may contain credential-bearing URLs. Do not export them.
        Ok(git_process::Output {
            code: 0,
            stdout: output.stdout,
            stderr: vec![],
        })
    }
    fn remote_head(
        &self,
        remote: &str,
        branch: &str,
        cancel: &CancellationToken,
    ) -> Result<Option<String>, String> {
        let out = self.remote_run(&["ls-remote", "--refs", remote, branch], cancel)?;
        let raw = text(&out.stdout)?;
        if raw.trim().is_empty() {
            return Ok(None);
        }
        let mut lines = raw.lines();
        let (oid, name) = lines
            .next()
            .unwrap()
            .split_once('\t')
            .ok_or("Malformed remote ref. Inspect external Git.")?;
        if lines.next().is_some()
            || name != branch
            || ![40, 64].contains(&oid.len())
            || !oid.bytes().all(|b| b.is_ascii_hexdigit())
        {
            return Err("Remote ref is ambiguous. Inspect external Git.".into());
        }
        Ok(Some(oid.to_string()))
    }
    pub(super) fn review_remote(
        &self,
        status: &Value,
        op: &Mutation,
        registry: &Arc<Mutex<Registry>>,
        cancel: &CancellationToken,
    ) -> Result<(String, Basis), String> {
        let (remote, branch) = match op {
            Mutation::Fetch { remote } => (remote.as_str(), ""),
            Mutation::Pull { remote, branch } | Mutation::Push { remote, branch } => {
                (remote.as_str(), branch.as_str())
            }
            _ => return Err("Unsupported remote action.".into()),
        };
        self.remote_target(remote, branch, cancel)?;
        // Reconcile an earlier uncertain push even if the next action targets a different ref.
        let uncertain = registry
            .lock()
            .map_err(|_| "Git state unavailable.")?
            .uncertain
            .get(&self.id())
            .cloned();
        if let Some(old) = uncertain {
            self.remote_target(&old.remote, &old.branch, cancel)?;
            let current = self.remote_head(&old.remote, &old.branch, cancel)?;
            registry
                .lock()
                .map_err(|_| "Git state unavailable.")?
                .uncertain
                .remove(&self.id());
            if current.as_deref() == Some(&old.head) {
                return Err("The earlier uncertain push reached its reviewed HEAD. Refresh to inspect it; it was not repeated.".into());
            }
        }
        let head = status["head"].as_str().unwrap_or("").to_string();
        if !matches!(op, Mutation::Fetch { .. }) && head.is_empty() {
            return Err("Commit first, then review a remote branch action.".into());
        }
        if matches!(op, Mutation::Pull { .. }) && !status["entries"].as_array().unwrap().is_empty()
        {
            return Err(
                "Save and commit or selectively stash changes before fast-forward Pull.".into(),
            );
        }
        let remote_head = if branch.is_empty() {
            None
        } else {
            self.remote_head(remote, branch, cancel)?
        };
        if matches!(op, Mutation::Push { .. }) && remote_head.as_deref() == Some(&head) {
            return Err("Remote branch already has this HEAD. Refresh; no push is needed.".into());
        }
        if matches!(op, Mutation::Pull { .. }) && remote_head.is_none() {
            return Err("Remote branch is absent. Check tracking with external Git.".into());
        }
        let verb = match op {
            Mutation::Fetch { .. } => {
                "Fetch configured remote refs; saved files and HEAD stay unchanged"
            }
            Mutation::Pull { .. } => {
                "Fetch this branch and fast-forward HEAD only; divergence is refused"
            }
            _ => "Publish this exact local HEAD to this branch, with no force push",
        };
        Ok((format!("{verb}.\nRemote: {remote}\nTarget: {branch}\nLocal HEAD: {head}\nRemote ref: {}\nConfigured credential helpers and Git hooks may run with your OS permissions. No interactive credential prompt. Stop/deadline requires ref reconciliation before retry.",remote_head.as_deref().unwrap_or("absent")),Basis{remote:remote.into(),branch:branch.into(),head,remote_head}))
    }
    pub(super) fn apply_remote(
        &self,
        review: &Review,
        registry: &Arc<Mutex<Registry>>,
        cancel: &CancellationToken,
    ) -> Result<git_process::Output, String> {
        let b = review
            .remote_basis
            .as_ref()
            .ok_or("Remote review missing.")?;
        self.remote_target(&b.remote, &b.branch, cancel)?;
        if !b.branch.is_empty() && self.remote_head(&b.remote, &b.branch, cancel)? != b.remote_head
        {
            return Err("Remote ref changed since review. Refresh and review its new state; no local mutation or push was performed.".into());
        }
        match &review.operation {
            Mutation::Fetch { .. } => self.remote_run(
                &["fetch", "--no-recurse-submodules", "--", &b.remote],
                cancel,
            ),
            Mutation::Pull { .. } => {
                self.remote_run(
                    &[
                        "fetch",
                        "--no-recurse-submodules",
                        "--",
                        &b.remote,
                        &b.branch,
                    ],
                    cancel,
                )?;
                let fetched =
                    text(&self.command(&["rev-parse", "--verify", "FETCH_HEAD"], cancel)?)?
                        .trim()
                        .to_string();
                if Some(&fetched) != b.remote_head.as_ref() {
                    return Err(
                        "Fetched ref differs from review. No merge was performed; Refresh.".into(),
                    );
                }
                self.command(&["merge-base","--is-ancestor",&b.head,&fetched],cancel).map_err(|_|"Pull would diverge. No merge was performed. Inspect external Git or review your local commits.")?;
                self.remote_run(&["merge", "--ff-only", "--", &fetched], cancel)
            }
            Mutation::Push { .. } => {
                {
                    let mut state = registry.lock().map_err(|_| "Git state unavailable.")?;
                    if state.uncertain.len() >= 8 && !state.uncertain.contains_key(&self.id()) {
                        return Err("Eight uncertain push results are retained. Reconcile an earlier remote before pushing another repository.".into());
                    }
                    state.uncertain.insert(self.id(), b.clone());
                }
                let result = self.remote_run(
                    &[
                        "push",
                        "--porcelain",
                        "--no-recurse-submodules",
                        "--",
                        &b.remote,
                        &format!("{}:{}", b.head, b.branch),
                    ],
                    cancel,
                );
                if result.is_ok() {
                    registry
                        .lock()
                        .map_err(|_| "Git state unavailable.")?
                        .uncertain
                        .remove(&self.id());
                }
                result
            }
            _ => Err("Unsupported remote action.".into()),
        }
    }
}

#[cfg(test)]
#[path = "git_remote_tests.rs"]
mod tests;
