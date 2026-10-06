use super::*;
#[cfg(test)]
#[path = "git_mutation_tests.rs"]
mod tests;
use serde::{Deserialize, Serialize};
use std::{
    io::Write,
    time::{Duration, Instant},
};
fn author_identity(value: &str) -> String {
    value
        .trim()
        .rsplitn(3, ' ')
        .nth(2)
        .unwrap_or(value.trim())
        .to_string()
}

#[derive(Clone, Deserialize, Serialize)]
#[serde(tag = "kind", rename_all = "camelCase", deny_unknown_fields)]
pub(crate) enum Mutation {
    Stage { paths: Vec<String> },
    Unstage { paths: Vec<String> },
    Commit { message: String },
}
pub(super) struct Review {
    pub token: String,
    pub root: PathBuf,
    revision: String,
    operation: Mutation,
    paths: Vec<String>,
    created: Instant,
    author: Option<String>,
}
impl Repo {
    pub(super) fn review(
        &self,
        revision: &str,
        operation: Mutation,
        registry: &Arc<Mutex<Registry>>,
        cancel: &CancellationToken,
    ) -> Result<Value, String> {
        let status = self.revision(revision, cancel)?;
        let entries = status["entries"].as_array().unwrap();
        let (paths, author, patch) = match &operation {
            Mutation::Stage { paths } | Mutation::Unstage { paths } => {
                if paths.is_empty() || paths.len() > 16 {
                    return Err("Choose one to sixteen saved paths.".into());
                }
                let mut expanded = paths.clone();
                for path in paths {
                    self.path(path)?;
                    let entry = entries
                        .iter()
                        .find(|e| e["path"] == path.as_str())
                        .ok_or("Path is no longer in Changes/Staged. Refresh.")?;
                    if let Some(old) = entry["oldPath"].as_str() {
                        self.path(old)?;
                        if !expanded.iter().any(|p| p == old) {
                            expanded.push(old.into());
                        }
                    }
                }
                let mut preview = String::new();
                for path in paths {
                    let diff = self.diff(
                        revision,
                        path,
                        if matches!(&operation, Mutation::Stage { .. }) {
                            "working"
                        } else {
                            "staged"
                        },
                        None,
                        cancel,
                    )?;
                    preview.push_str(diff["patch"].as_str().unwrap());
                    if preview.len() > 256 * 1024 {
                        return Err(
                            "Review diff exceeds 256 KiB. Review fewer paths or use external Git."
                                .into(),
                        );
                    }
                }
                (expanded, None, preview)
            }
            Mutation::Commit { message } => {
                if message.trim().is_empty() || message.len() > 8192 || message.contains('\0') {
                    return Err("Write a nonempty commit message within 8 KiB.".into());
                }
                if entries.iter().any(|e| e["conflict"] == true) {
                    return Err(
                        "Resolve and save conflicts, then stage deliberately before committing."
                            .into(),
                    );
                }
                let paths = entries
                    .iter()
                    .filter(|e| e["index"] != " " && e["index"] != "?")
                    .map(|e| e["path"].as_str().unwrap().to_string())
                    .collect::<Vec<_>>();
                if paths.is_empty() {
                    return Err(
                        "There are no staged changes. Stage selected saved files first.".into(),
                    );
                }
                for path in &paths {
                    self.path(path)?;
                }
                let author = String::from_utf8(self.command(&["var", "GIT_AUTHOR_IDENT"], cancel)?)
                    .map_err(|_| "Git author is not UTF-8.")?;
                let patch = self.command(
                    &[
                        "diff",
                        "--cached",
                        "--no-ext-diff",
                        "--no-textconv",
                        "--no-color",
                        "--full-index",
                        "--binary",
                        "--",
                    ],
                    cancel,
                )?;
                if patch.len() > 256 * 1024 {
                    return Err(
                        "Staged review exceeds 256 KiB. Use smaller commits or external Git."
                            .into(),
                    );
                }
                (
                    paths,
                    Some(author_identity(&author)),
                    text(&patch)?.to_string(),
                )
            }
        };
        self.revision(revision, cancel)?;
        let token = uuid::Uuid::new_v4().to_string();
        let id = self.id();
        let view = json!({"token":token,"repo":id,"root":self.root,"revision":revision,"operation":operation,"paths":paths,"author":author,"patch":patch,"notice":"Uses saved files and the Git index. Configured Git hooks remain enabled and run with your OS permissions."});
        let mut state = registry
            .lock()
            .map_err(|_| "Git reviews are unavailable.")?;
        state
            .reviews
            .retain(|_, r| r.created.elapsed() < Duration::from_secs(120));
        if !state.reviews.contains_key(&id) && state.reviews.len() >= 8 {
            return Err(
                "Eight Git reviews are retained. Close a review before opening another.".into(),
            );
        }
        state.reviews.insert(
            id,
            Review {
                token,
                root: self.root.clone(),
                revision: revision.into(),
                operation,
                paths,
                created: Instant::now(),
                author,
            },
        );
        Ok(view)
    }
    pub(super) fn apply(
        &self,
        token: &str,
        registry: &Arc<Mutex<Registry>>,
        editor: &Arc<Mutex<crate::editor::Registry>>,
        cancel: &CancellationToken,
    ) -> Result<Value, String> {
        let id = self.id();
        let review = {
            let mut state = registry.lock().map_err(|_| "Git reviews unavailable.")?;
            if !state.reviews.get(&id).is_some_and(|r| r.token == token) {
                return Err("Git review is no longer current. Review again.".into());
            }
            state.reviews.remove(&id).unwrap()
        };
        if review.created.elapsed() > Duration::from_secs(120) || review.root != self.root {
            return Err("Git review expired or worktree changed. Review again.".into());
        }
        editor
            .lock()
            .map_err(|_| "Editor state is unavailable.")?
            .ensure_git_clean(&self.root, &review.paths)?;
        let before = self.revision(&review.revision, cancel)?;
        if let Some(author) = &review.author {
            if author_identity(text(&self.command(&["var", "GIT_AUTHOR_IDENT"], cancel)?)?)
                != *author
            {
                return Err(
                    "Configured author changed. Review the commit again; draft remains.".into(),
                );
            }
        }
        let mut args: Vec<String> = match &review.operation {
            Mutation::Stage { .. } => vec!["add".into(), "--".into()],
            Mutation::Unstage { .. } => {
                if before["head"].is_null() {
                    vec![
                        "rm".into(),
                        "--cached".into(),
                        "--ignore-unmatch".into(),
                        "--".into(),
                    ]
                } else {
                    vec![
                        "restore".into(),
                        "--staged".into(),
                        "--source=HEAD".into(),
                        "--".into(),
                    ]
                }
            }
            Mutation::Commit { .. } => vec![],
        };
        let mut message = None;
        if let Mutation::Commit { message: text } = &review.operation {
            let mut file = tempfile::NamedTempFile::new()
                .map_err(|_| "Commit message could not be prepared. Draft remains.")?;
            file.write_all(text.as_bytes())
                .map_err(|_| "Commit message could not be written.")?;
            file.flush()
                .map_err(|_| "Commit message could not be flushed.")?;
            args = vec![
                "commit".into(),
                "--file".into(),
                file.path().to_string_lossy().into_owned(),
                "--cleanup=verbatim".into(),
            ];
            message = Some(file);
        } else {
            args.extend(review.paths.clone());
        }
        let outcome = git_process::run(&self.executable, &self.root, &args, cancel, false);
        drop(message);
        let after = self.status(&CancellationToken::new());
        let committed = matches!(&review.operation, Mutation::Commit { .. })
            && after.as_ref().is_ok_and(|s| s["head"] != before["head"]);
        let warning=match outcome {
            Ok(output) if output.code==0=>None,
            Ok(output) if committed=>Some(format!("HEAD changed despite Git exit {}. Inspect the new commit before repeating anything.",output.code)),
            Ok(output)=>return Err(format!("Git refused the operation ({}): {}. Commit draft and any existing index changes remain; Refresh before reviewing again.",output.code,String::from_utf8_lossy(&output.stderr))),
            Err(error) if committed=>Some(format!("{error} HEAD changed; inspect the completed commit before retrying.")),
            Err(error)=>return Err(error),
        };
        Ok(
            json!({"completed":true,"operation":review.operation,"status":after.as_ref().ok(),"warning":warning.or_else(||after.err().map(|e|format!("Action completed; repository refresh failed: {e}")))}),
        )
    }
}
