use super::*;
fn names(bytes: &[u8]) -> Result<Vec<String>, String> {
    bytes
        .split(|b| *b == 0)
        .filter(|b| !b.is_empty())
        .map(|b| text(b).map(str::to_string))
        .collect()
}

pub(in crate::source_control) fn hunks(
    patch: &str,
    revision: &str,
    path: &str,
    basis: &str,
) -> Vec<Value> {
    if !patch.starts_with("diff --git ")
        || patch.matches("diff --git ").count() != 1
        || [
            "new file mode ",
            "deleted file mode ",
            "rename from ",
            "rename to ",
            "old mode ",
            "new mode ",
            "GIT binary patch",
        ]
        .iter()
        .any(|s| patch.contains(s))
    {
        return vec![];
    }
    let mut prefix = String::new();
    let mut sections = Vec::<String>::new();
    for line in patch.split_inclusive('\n') {
        if line.starts_with("@@ ") {
            sections.push(String::new());
        }
        if let Some(last) = sections.last_mut() {
            last.push_str(line);
        } else {
            prefix.push_str(line);
        }
    }
    sections.iter().take(32).map(|section|json!({"id":hash(format!("{revision}\0{path}\0{basis}\0{section}").as_bytes()),"header":section.lines().next().unwrap_or(""),"patch":format!("{prefix}{section}")})).collect()
}
impl Repo {
    pub(in crate::source_control) fn local_state(
        &self,
        cancel: &CancellationToken,
    ) -> Result<Value, String> {
        let branches = self.command(
            &[
                "for-each-ref",
                "--count=50",
                "--format=%(refname:short)%00%(objectname)",
                "refs/heads",
            ],
            cancel,
        )?;
        let branches = text(&branches)?
            .lines()
            .map(|l| {
                let (name, id) = l.split_once('\0').ok_or("Malformed Git branch record.")?;
                Ok(json!({"name":name,"id":id}))
            })
            .collect::<Result<Vec<Value>, String>>()?;
        let raw = self.command(
            &[
                "stash",
                "list",
                "-z",
                "--max-count=30",
                "--format=%H%x00%gd%x00%gs",
            ],
            cancel,
        )?;
        let mut fields = raw.split(|b| *b == 0).collect::<Vec<_>>();
        if fields.last().is_some_and(|s| s.is_empty()) {
            fields.pop();
        }
        if fields.len() % 3 != 0 {
            return Err("Malformed stash records. Refresh.".into());
        }
        let stashes = fields
            .chunks(3)
            .map(|r| Ok(json!({"id":text(r[0])?,"ref":text(r[1])?,"subject":text(r[2])?})))
            .collect::<Result<Vec<Value>, String>>()?;
        Ok(json!({"branches":branches,"stashes":stashes}))
    }
    fn validate_branch(&self, name: &str, cancel: &CancellationToken) -> Result<(), String> {
        if name.starts_with('-') || name.len() > 256 || name.contains('\0') {
            return Err("Choose a valid branch name within 256 bytes.".into());
        }
        self.command(&["check-ref-format", "--branch", name], cancel)?;
        Ok(())
    }
    fn stash(&self, id: &str, cancel: &CancellationToken) -> Result<(), String> {
        if !self.local_state(cancel)?["stashes"]
            .as_array()
            .unwrap()
            .iter()
            .any(|s| s["id"] == id)
        {
            return Err(
                "Stash changed or is outside the first 30 entries. Refresh or use external Git."
                    .into(),
            );
        }
        Ok(())
    }
    pub(super) fn review_local(
        &self,
        revision: &str,
        status: &Value,
        op: &Mutation,
        cancel: &CancellationToken,
    ) -> Result<(Vec<String>, Option<String>, String), String> {
        let entries = status["entries"].as_array().unwrap();
        let result = match op {
            Mutation::Hunks { path, staged, ids } => {
                if ids.is_empty() || ids.len() > 32 {
                    return Err("Choose one to 32 displayed hunks.".into());
                }
                let basis = if *staged { "staged" } else { "working" };
                let diff = self.diff(revision, path, basis, None, cancel)?;
                if diff["reason"].is_string() || diff["conflict"] == true {
                    return Err("This comparison does not support hunk actions. Use reviewed whole-file staging after resolving conflicts.".into());
                }
                let sections = hunks(diff["patch"].as_str().unwrap(), revision, path, basis);
                let mut patch = String::new();
                if ids
                    .iter()
                    .any(|id| !sections.iter().any(|s| s["id"] == id.as_str()))
                {
                    return Err("Hunk basis changed. Refresh this diff and review again.".into());
                }
                for section in sections
                    .iter()
                    .filter(|s| ids.iter().any(|id| s["id"] == id.as_str()))
                {
                    let value = section["patch"].as_str().unwrap();
                    if patch.is_empty() {
                        patch.push_str(value);
                    } else {
                        let start = value.find("@@ ").ok_or("Invalid hunk.")?;
                        patch.push_str(&value[start..]);
                    }
                }
                if patch.len() > 64 * 1024 {
                    return Err("Selected patch exceeds 64 KiB. Choose fewer hunks.".into());
                }
                (vec![path.clone()], None, patch)
            }
            Mutation::Discard { path } => {
                self.path(path)?;
                self.command(&["ls-files", "--error-unmatch", "--", path], cancel)?;
                let diff = self.diff(revision, path, "working", None, cancel)?;
                (
                    vec![path.clone()],
                    None,
                    format!(
                        "Restore saved working file from Index. Staged content remains.\n{}",
                        diff["patch"]
                    ),
                )
            }
            Mutation::StashCreate { paths, message } => {
                if status["head"].is_null()
                    || paths.is_empty()
                    || paths.len() > 16
                    || message.len() > 512
                    || message.contains(['\0', '\n'])
                {
                    return Err("Stash needs an existing commit, one to sixteen tracked paths and a short one-line message.".into());
                }
                let mut patch=String::from("Stash only these tracked saved paths, including their staged changes. Untracked files remain.\n");
                for path in paths {
                    self.path(path)?;
                    let entry = entries
                        .iter()
                        .find(|e| e["path"] == path.as_str())
                        .ok_or("Stash path is no longer changed. Refresh.")?;
                    if entry["index"] == "?" || entry["conflict"] == true {
                        return Err(
                            "Resolve conflicts; untracked files are not included in this stash."
                                .into(),
                        );
                    }
                    if entry["oldPath"].is_string() {
                        return Err(
                            "Stash renamed paths using external Git to review both names together."
                                .into(),
                        );
                    }
                    for basis in ["staged", "working"] {
                        let diff = self.diff(revision, path, basis, None, cancel)?;
                        patch.push_str(diff["patch"].as_str().unwrap());
                    }
                }
                (paths.clone(), None, patch)
            }
            Mutation::StashApply { stash, pop } => {
                self.stash(stash, cancel)?;
                let raw = self.command(
                    &[
                        "stash",
                        "show",
                        "--include-untracked",
                        "--name-only",
                        "-z",
                        stash,
                    ],
                    cancel,
                )?;
                let paths = names(&raw)?;
                for path in &paths {
                    self.path(path)?;
                }
                if paths.len() > 2000 {
                    return Err("Stash exceeds the path bound. Use external Git.".into());
                }
                let patch = self.command(
                    &[
                        "stash",
                        "show",
                        "--include-untracked",
                        "--binary",
                        "--no-ext-diff",
                        "--no-textconv",
                        stash,
                    ],
                    cancel,
                )?;
                (vec![],None,format!("{} exact stash {stash}, restoring its index and working changes. Conflicts retain the stash; inspect and resolve before any retry.\n{}\n{}",if *pop{"Apply then drop"}else{"Apply"},paths.join("\n"),text(&patch)?))
            }
            Mutation::BranchCreate { name } | Mutation::BranchSwitch { name } => {
                self.validate_branch(name, cancel)?;
                if matches!(op, Mutation::BranchSwitch { .. }) {
                    self.command(
                        &["rev-parse", "--verify", &format!("refs/heads/{name}")],
                        cancel,
                    )?;
                    if entries.iter().any(|e| e["index"] != "?") {
                        return Err("Save and commit or selectively stash tracked changes before switching branches. Untracked files remain; Git refuses an overwrite.".into());
                    }
                }
                (
                    vec![],
                    None,
                    format!(
                        "{} branch {name}. No forced checkout or automatic stash.",
                        if matches!(op, Mutation::BranchCreate { .. }) {
                            "Create"
                        } else {
                            "Switch to"
                        }
                    ),
                )
            }
            Mutation::Revert { commit } => {
                if !entries.is_empty() {
                    return Err("Commit or selectively stash changes before reverting a commit. No files were changed.".into());
                }
                let details = self.commit_files(commit, cancel)?;
                let paths = details["files"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(|v| v.as_str().unwrap().to_string())
                    .collect::<Vec<_>>();
                for path in &paths {
                    self.path(path)?;
                }
                let patch = self.command(
                    &[
                        "show",
                        "--format=",
                        "--no-ext-diff",
                        "--no-textconv",
                        "--binary",
                        "--reverse",
                        commit,
                        "--",
                    ],
                    cancel,
                )?;
                (paths,None,format!("Create a new commit reversing {commit}; keep its original commit and history. On conflict, resolve saved files then finish with external Git, or Abort here.\n{}",text(&patch)?))
            }
            Mutation::RevertAbort => {
                if status["reverting"] != true {
                    return Err("There is no conflicted revert to abort.".into());
                }
                (vec![],None,"Abort the conflicted revert and restore its pre-revert state. Unsaved editors must first be saved or closed.".into())
            }
            _ => return Err("Unsupported local Git action.".into()),
        };
        if result.2.len() > 256 * 1024 {
            return Err("Git review exceeds 256 KiB. Use a smaller selection.".into());
        }
        Ok(result)
    }
    pub(super) fn apply_local(
        &self,
        review: &Review,
        cancel: &CancellationToken,
    ) -> Result<git_process::Output, String> {
        let run = |args: &[&str]| {
            git_process::run(
                &self.executable,
                &self.root,
                &args.iter().map(|s| s.to_string()).collect::<Vec<_>>(),
                cancel,
                false,
            )
        };
        match &review.operation {
            Mutation::Hunks { staged, .. } => {
                let mut file =
                    tempfile::NamedTempFile::new().map_err(|_| "Patch could not be prepared.")?;
                file.write_all(review.patch.as_bytes())
                    .map_err(|_| "Patch could not be written.")?;
                file.flush().map_err(|_| "Patch could not be flushed.")?;
                let path = file.path().to_string_lossy().to_string();
                let mut args = vec!["apply", "--cached"];
                if *staged {
                    args.push("--reverse");
                }
                args.push("--check");
                args.push(&path);
                let check = run(&args)?;
                if check.code != 0 {
                    return Ok(check);
                }
                args.retain(|a| *a != "--check");
                run(&args)
            }
            Mutation::Discard { path } => run(&["restore", "--worktree", "--", path]),
            Mutation::StashCreate { paths, message } => {
                let mut args = vec!["stash", "push", "-m", message, "--"];
                args.extend(paths.iter().map(String::as_str));
                run(&args)
            }
            Mutation::StashApply { stash, pop } => {
                self.stash(stash, cancel)?;
                let applied = run(&["stash", "apply", "--index", stash])?;
                if applied.code != 0 || !*pop {
                    return Ok(applied);
                }
                let state = self.local_state(cancel)?;
                let item = state["stashes"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .find(|v| v["id"] == stash.as_str());
                let Some(item) = item else {
                    return Ok(git_process::Output {
                        code: 0,
                        stdout: applied.stdout,
                        stderr:
                            b"Stash applied, but its ref changed; no stash was dropped. Refresh."
                                .to_vec(),
                    });
                };
                let dropped = run(&["stash", "drop", item["ref"].as_str().unwrap()])?;
                if dropped.code != 0 {
                    return Ok(git_process::Output{code:0,stdout:applied.stdout,stderr:b"Stash applied; dropping its retained ref failed. Inspect before retrying.".to_vec()});
                }
                Ok(applied)
            }
            Mutation::BranchCreate { name } => run(&["branch", "--", name]),
            Mutation::BranchSwitch { name } => run(&["switch", "--no-guess", name]),
            Mutation::Revert { commit } => run(&["revert", "--no-edit", commit]),
            Mutation::RevertAbort => run(&["revert", "--abort"]),
            _ => Err("Unsupported Git operation.".into()),
        }
    }
}

#[cfg(test)]
#[path = "git_local_tests.rs"]
mod tests;
