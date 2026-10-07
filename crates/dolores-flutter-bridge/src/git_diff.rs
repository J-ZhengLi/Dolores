use super::*;
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
        let value = self.collect_diff(revision, path, basis, commit, None, cancel)?;
        if !value["next"].is_null() {
            return Err("This action's full review exceeds 256 KiB. All change pages remain viewable in Source Control.".into());
        }
        Ok(value)
    }

    pub(super) fn diff_page(
        &self,
        revision: &str,
        path: &str,
        basis: &str,
        commit: Option<&str>,
        page: (usize, Option<&str>),
        cancel: &CancellationToken,
    ) -> Result<Value, String> {
        self.collect_diff(revision, path, basis, commit, Some(page), cancel)
    }

    fn collect_diff(
        &self,
        revision: &str,
        path: &str,
        basis: &str,
        commit: Option<&str>,
        page: Option<(usize, Option<&str>)>,
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
        let (left_label, right_label) = match basis {
            "working" => ("Index".to_string(), "Saved working tree".to_string()),
            "staged" => ("HEAD".to_string(), "Index".to_string()),
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
                (
                    details["parents"][0]
                        .as_str()
                        .map(|s| s[..8].to_string())
                        .unwrap_or("Empty tree".into()),
                    id[..8].into(),
                )
            }
            _ => return Err("Unsupported Git comparison basis.".into()),
        };
        let untracked = basis == "working" && entry.is_some_and(|e| e["index"] == "?");
        let mut args = vec![
            if basis == "commit" { "show" } else { "diff" },
            "--no-ext-diff",
            "--no-textconv",
            "--no-color",
            "--full-index",
            "--unified=3",
        ];
        if basis == "staged" {
            args.push("--cached");
        }
        if basis == "commit" {
            args.extend(["--format=", commit.unwrap()]);
        }
        if untracked {
            args.push("--no-index");
        }
        args.push("--");
        if untracked {
            args.push("/dev/null");
        }
        args.push(path);
        if old_path != path {
            args.push(old_path);
        }
        let (cursor, expected) = page.unwrap_or((0, None));
        if cursor > 0 && expected.is_none() {
            return Err("Diff page needs its original fingerprint. Refresh the comparison.".into());
        }
        let mut collector = diff_page::Collector::new(
            cursor,
            if page.is_some() {
                diff_page::PAGE_ROWS
            } else {
                usize::MAX
            },
        );
        let mut stderr = Vec::new();
        let code = git_process::run_stream(
            &self.executable,
            &self.root,
            &args.iter().map(|s| s.to_string()).collect::<Vec<_>>(),
            cancel,
            false,
            |pipe, bytes| {
                if pipe == 0 {
                    collector.push(bytes);
                } else {
                    if stderr.len() + bytes.len() > 512 * 1024 {
                        return Err(
                            "Git diagnostics exceeded their bound. Refresh before retrying.".into(),
                        );
                    }
                    stderr.extend_from_slice(bytes);
                }
                Ok(())
            },
        )?;
        if code != 0 && !(untracked && code == 1) {
            return Err(format!(
                "Git diff failed ({code}): {}. Refresh before retrying.",
                String::from_utf8_lossy(&stderr)
            ));
        }
        let mut value = collector.finish()?;
        if expected.is_some_and(|expected| value["digest"] != expected) {
            return Err("These changes differ from the displayed page. Refresh this comparison before continuing.".into());
        }
        self.revision(revision, cancel)?;
        let complete = cursor == 0 && value["next"].is_null();
        value["hunks"] = json!(if complete && basis != "commit" {
            mutation::local::hunks(value["patch"].as_str().unwrap(), revision, path, basis)
        } else {
            vec![]
        });
        value["path"] = json!(path);
        value["basis"] = json!(basis);
        value["revision"] = json!(revision);
        value["leftLabel"] = json!(left_label);
        value["rightLabel"] = json!(right_label);
        value["reason"] = Value::Null;
        value["historical"] = json!(basis == "commit");
        value["commit"] = json!(commit);
        value["conflict"] = json!(entry.is_some_and(|e| e["conflict"] == true));
        Ok(value)
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
    fn large_saved_files_render_their_small_changes_in_dolores() {
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
            assert_eq!(out.code, 0);
        };
        invoke(&["init"]);
        invoke(&["config", "user.name", "Fixture"]);
        invoke(&["config", "user.email", "fixture@example.invalid"]);
        let context = "unchanged context\n".repeat(1_050_000);
        std::fs::write(
            dir.path().join("large.txt"),
            format!("old value\n{context}"),
        )
        .unwrap();
        invoke(&["add", "large.txt"]);
        invoke(&["commit", "-m", "large base"]);
        std::fs::write(
            dir.path().join("large.txt"),
            format!("new value\n{context}"),
        )
        .unwrap();
        let repo = Repo::discover(dir.path().to_str().unwrap(), &cancel).unwrap();
        let status = repo.status(&cancel).unwrap();
        let diff = repo
            .diff(
                status["revision"].as_str().unwrap(),
                "large.txt",
                "working",
                None,
                &cancel,
            )
            .unwrap();
        assert!(diff["reason"].is_null(), "{}", diff["reason"]);
        assert!(diff["patch"].as_str().unwrap().contains("+new value"));
    }
    #[test]
    fn large_patch_pages_preserve_content_and_refuse_changed_continuations() {
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
            assert_eq!(out.code, 0);
        };
        invoke(&["init"]);
        invoke(&["config", "user.name", "Fixture"]);
        invoke(&["config", "user.email", "fixture@example.invalid"]);
        let old = (0..30000)
            .map(|i| format!("old content {i}\n"))
            .collect::<String>();
        let new = old.replace("old content", "new content");
        std::fs::write(dir.path().join("pages.txt"), &old).unwrap();
        invoke(&["add", "pages.txt"]);
        invoke(&["commit", "-m", "page base"]);
        std::fs::write(dir.path().join("pages.txt"), &new).unwrap();
        let repo = Repo::discover(dir.path().to_str().unwrap(), &cancel).unwrap();
        let status = repo.status(&cancel).unwrap();
        let revision = status["revision"].as_str().unwrap();
        let first = repo
            .diff_page(revision, "pages.txt", "working", None, (0, None), &cancel)
            .unwrap();
        assert_eq!(first["rows"].as_array().unwrap().len(), 256);
        assert!(first["totalRows"].as_u64().unwrap() > 60000);
        let digest = first["digest"].as_str().unwrap();
        let end = first["totalRows"].as_u64().unwrap() as usize - 32;
        let last = repo
            .diff_page(
                revision,
                "pages.txt",
                "working",
                None,
                (end, Some(digest)),
                &cancel,
            )
            .unwrap();
        assert!(last["next"].is_null());
        assert!(last["patch"]
            .as_str()
            .unwrap()
            .contains("+new content 29999"));
        assert!(repo
            .diff_page(
                revision,
                "pages.txt",
                "working",
                None,
                (end, Some("wrong fingerprint")),
                &cancel
            )
            .unwrap_err()
            .contains("differ from"));
        std::fs::write(dir.path().join("pages.txt"), "changed during viewing\n").unwrap();
        assert!(repo
            .diff_page(
                revision,
                "pages.txt",
                "working",
                None,
                (end, Some(digest)),
                &cancel
            )
            .unwrap_err()
            .contains("basis changed"));
        // No-index displays an untracked file's additions inside Dolores too.
        std::fs::write(dir.path().join("new.txt"), "new untracked content\n").unwrap();
        let fresh = repo.status(&cancel).unwrap();
        let added = repo
            .diff_page(
                fresh["revision"].as_str().unwrap(),
                "new.txt",
                "working",
                None,
                (0, None),
                &cancel,
            )
            .unwrap();
        assert!(added["patch"]
            .as_str()
            .unwrap()
            .contains("+new untracked content"));
    }
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
    }
}
