use super::*;
#[derive(Default)]
struct Side {
    value: String,
    reason: Option<String>,
}
impl Side {
    fn bytes(bytes: Vec<u8>) -> Self {
        if bytes.len() > 256 * 1024 {
            return Self {
                reason: Some("Text exceeds 256 KiB; inspect it using external Git.".into()),
                ..Self::default()
            };
        }
        match String::from_utf8(bytes) {
            Ok(value) if !value.contains('\0') && value.lines().all(|line|line.len()<=8192)=>Self{value,reason:None},
            _=>Self{reason:Some("Binary, unsupported encoding or long physical lines; no editable diff is offered.".into()),..Self::default()},
        }
    }
}
impl Repo {
    fn commit_id(&self, id: &str, cancel: &CancellationToken) -> Result<String, String> {
        if !matches!(id.len(), 40 | 64) || !id.bytes().all(|b| b.is_ascii_hexdigit()) {
            return Err("Choose an exact commit from repository history.".into());
        }
        let value = self.command(
            &["rev-parse", "--verify", &format!("{id}^{{commit}}")],
            cancel,
        )?;
        let result = text(&value)?.trim();
        if result != id {
            return Err("Commit basis changed. Reload history.".into());
        }
        Ok(result.into())
    }
    fn side(
        &self,
        spec: Option<&str>,
        path: &str,
        working: bool,
        cancel: &CancellationToken,
    ) -> Result<Side, String> {
        if working {
            let file = self.path(path)?;
            if !file.exists() {
                return Ok(Side::default());
            }
            if !file.is_file() {
                return Ok(Side {
                    reason: Some("Submodule or directory; inspect using external Git.".into()),
                    ..Side::default()
                });
            }
            if file.metadata().map_err(|_| "File unavailable.")?.len() > 256 * 1024 {
                return Ok(Side {
                    reason: Some("File exceeds 256 KiB; use external Git for this diff.".into()),
                    ..Side::default()
                });
            }
            return Ok(Side::bytes(read_bounded(&file, 256 * 1024)?));
        }
        let Some(spec) = spec else {
            return Ok(Side::default());
        };
        // Probe the exact tree/index entry first; transport/corruption errors must
        // not silently masquerade as an absent file.
        let entry = if spec.starts_with(':') {
            self.command(&["ls-files", "--stage", "-z", "--", path], cancel)?
        } else {
            let (commit, _) = spec.split_once(':').ok_or("Invalid blob basis.")?;
            self.command(&["ls-tree", "-z", commit, "--", path], cancel)?
        };
        if entry.is_empty() {
            return Ok(Side::default());
        }
        if spec.starts_with(':')
            && !entry
                .split(|b| *b == 0)
                .any(|r| r.windows(3).any(|s| s == b" 0\t"))
        {
            return Ok(Side {
                reason: Some(
                    "Unmerged index; inspect and resolve the saved conflict before staging.".into(),
                ),
                ..Side::default()
            });
        }
        let size = self.command(&["cat-file", "-s", spec], cancel)?;
        if text(&size)?
            .trim()
            .parse::<usize>()
            .map_err(|_| "Invalid Git blob size.")?
            > 256 * 1024
        {
            return Ok(Side {
                reason: Some("Git blob exceeds 256 KiB; use external Git.".into()),
                ..Side::default()
            });
        }
        Ok(Side::bytes(self.command(&["show", spec], cancel)?))
    }
    pub(super) fn commit_files(
        &self,
        commit: &str,
        cancel: &CancellationToken,
    ) -> Result<Value, String> {
        self.commit_id(commit, cancel)?;
        let parent = self.command(&["rev-list", "--parents", "-n", "1", commit], cancel)?;
        let parents = text(&parent)?
            .split_whitespace()
            .skip(1)
            .map(str::to_string)
            .collect::<Vec<_>>();
        if parents.len() > 1 {
            return Err("Merge commit comparisons are not supported in this initial history view. Choose a non-merge commit.".into());
        }
        let raw = self.command(
            &[
                "diff-tree",
                "--root",
                "--no-commit-id",
                "--name-only",
                "-r",
                "-z",
                commit,
                "--",
            ],
            cancel,
        )?;
        let files = raw
            .split(|b| *b == 0)
            .filter(|b| !b.is_empty())
            .map(|b| text(b).map(str::to_string))
            .collect::<Result<Vec<_>, _>>()?;
        if files.len() > 2000 {
            return Err("Commit exceeds 2,000 changed paths. Inspect with external Git.".into());
        }
        Ok(json!({"commit":commit,"parents":parents,"files":files}))
    }
    pub(super) fn diff(
        &self,
        revision: &str,
        path: &str,
        basis: &str,
        commit: Option<&str>,
        cancel: &CancellationToken,
    ) -> Result<Value, String> {
        self.path(path)?;
        let status = self.revision(revision, cancel)?;
        let entry = status["entries"]
            .as_array()
            .unwrap()
            .iter()
            .find(|e| e["path"] == path);
        let old_path = entry.and_then(|e| e["oldPath"].as_str()).unwrap_or(path);
        self.path(old_path)?;
        let (left_spec, right_spec, left_label, right_label, working) = match basis {
            "working" => (
                Some(format!(":{path}")),
                None,
                "Index".to_string(),
                "Saved working tree".to_string(),
                true,
            ),
            "staged" => (
                status["head"].as_str().map(|h| format!("{h}:{old_path}")),
                Some(format!(":{path}")),
                "HEAD".into(),
                "Index".into(),
                false,
            ),
            "commit" => {
                let id = commit.ok_or("Choose a history commit.")?;
                let details = self.commit_files(id, cancel)?;
                if !details["files"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .any(|v| v == path)
                {
                    return Err("Path does not belong to the selected commit.".into());
                }
                let parent = details["parents"][0].as_str();
                (
                    parent.map(|p| format!("{p}:{path}")),
                    Some(format!("{id}:{path}")),
                    parent
                        .map(|s| s[..8].to_string())
                        .unwrap_or("Empty tree".into()),
                    id[..8].into(),
                    false,
                )
            }
            _ => return Err("Unsupported Git comparison basis.".into()),
        };
        let left = self.side(left_spec.as_deref(), path, false, cancel)?;
        let right = self.side(right_spec.as_deref(), path, working, cancel)?;
        let mut reason = left.reason.or(right.reason);
        let mut args = vec![
            if basis == "commit" { "show" } else { "diff" },
            "--no-ext-diff",
            "--no-textconv",
            "--no-color",
            "--full-index",
            "--binary",
            "--unified=3",
        ];
        let staged = basis == "staged";
        if staged {
            args.push("--cached");
        }
        if basis == "commit" {
            args.push("--format=");
            args.push(commit.unwrap());
        }
        args.push("--");
        args.push(path);
        if old_path != path {
            args.push(old_path);
        }
        let patch = if reason.is_none() {
            match self.command(&args, cancel) {
                Ok(bytes) if bytes.len() <= 256 * 1024 => text(&bytes)?.to_string(),
                Ok(_) => {
                    reason = Some("Patch exceeds 256 KiB. Use external Git.".into());
                    String::new()
                }
                Err(e) => {
                    reason = Some(e);
                    String::new()
                }
            }
        } else {
            String::new()
        };
        self.revision(revision, cancel)?;
        Ok(
            json!({"path":path,"basis":basis,"revision":revision,"leftLabel":left_label,"rightLabel":right_label,"left":left.value,"right":right.value,"hunks":if basis=="commit" {vec![]} else {mutation::local::hunks(&patch,revision,path,basis)},"patch":patch,"reason":reason,"historical":basis=="commit","commit":commit,"conflict":entry.is_some_and(|e|e["conflict"]==true)}),
        )
    }
    pub(super) fn history(
        &self,
        head: &str,
        cursor: usize,
        cancel: &CancellationToken,
    ) -> Result<Value, String> {
        self.commit_id(head, cancel)?;
        if cursor > 10000 {
            return Err("History cursor exceeds 10,000. Refine with external Git.".into());
        }
        let raw = self.command(
            &[
                "log",
                "-z",
                "--max-count=31",
                &format!("--skip={cursor}"),
                "--format=%H%x00%P%x00%an%x00%aI%x00%s",
                head,
                "--",
            ],
            cancel,
        )?;
        let mut fields = raw.split(|b| *b == 0).collect::<Vec<_>>();
        if fields.last().is_some_and(|b| b.is_empty()) {
            fields.pop();
        }
        if fields.len() % 5 != 0 {
            return Err("Malformed history; partial results refused.".into());
        }
        let all=fields.chunks(5).map(|r|Ok(json!({"id":text(r[0])?,"parents":text(r[1])?,"author":text(r[2])?,"date":text(r[3])?,"subject":text(r[4])?}))).collect::<Result<Vec<Value>,String>>()?;
        Ok(
            json!({"head":head,"items":all.iter().take(30).collect::<Vec<_>>(),"next":if all.len()>30{Some(cursor+30)}else{None}}),
        )
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn saved_working_index_and_commit_bases_stay_distinct_and_stale_reads_refuse() {
        let dir = tempfile::tempdir().unwrap();
        let cancel = CancellationToken::new();
        let git = git_process::executable(dir.path()).unwrap();
        let invoke = |args: &[&str]| {
            let out = git_process::run(
                &git,
                dir.path(),
                &args.iter().map(|s| s.to_string()).collect::<Vec<_>>(),
                &cancel,
                false,
            )
            .unwrap();
            assert_eq!(out.code, 0, "{}", String::from_utf8_lossy(&out.stderr));
        };
        invoke(&["init"]);
        invoke(&["config", "user.name", "Fixture"]);
        invoke(&["config", "user.email", "fixture@example.invalid"]);
        std::fs::write(dir.path().join("same.txt"), "base\n").unwrap();
        invoke(&["add", "--", "same.txt"]);
        invoke(&["commit", "-m", "base"]);
        std::fs::write(dir.path().join("same.txt"), "index\n").unwrap();
        invoke(&["add", "--", "same.txt"]);
        std::fs::write(dir.path().join("same.txt"), "working\n").unwrap();
        let repo = Repo::discover(dir.path().to_str().unwrap(), &cancel).unwrap();
        let status = repo.status(&cancel).unwrap();
        let revision = status["revision"].as_str().unwrap();
        let diff = repo
            .diff(revision, "same.txt", "working", None, &cancel)
            .unwrap();
        assert_eq!(diff["left"], "index\n");
        assert_eq!(diff["right"], "working\n");
        let diff = repo
            .diff(revision, "same.txt", "staged", None, &cancel)
            .unwrap();
        assert_eq!(diff["left"], "base\n");
        assert_eq!(diff["right"], "index\n");
        let head = status["head"].as_str().unwrap();
        let history = repo.history(head, 0, &cancel).unwrap();
        assert_eq!(history["items"][0]["subject"], "base");
        let historical = repo
            .diff(revision, "same.txt", "commit", Some(head), &cancel)
            .unwrap();
        assert_eq!(historical["left"], "");
        assert_eq!(historical["right"], "base\n");
        std::fs::write(dir.path().join("same.txt"), "new outside bytes\n").unwrap();
        assert!(repo
            .diff(revision, "same.txt", "working", None, &cancel)
            .unwrap_err()
            .contains("basis changed"));
        assert!(repo.path("../escape").is_err());
        assert!(repo.path(".git/config").is_err());
        assert!(Side::bytes(vec![0, 255]).reason.is_some());
        assert!(Side::bytes(vec![b'x'; 8193]).reason.is_some());
    }
}
