use super::*;
use dolores_core::prompt_branches::{Create, Folder};
use std::path::Path;

#[cfg(test)]
#[path = "prompt_branch_tests.rs"]
mod tests;

#[derive(Deserialize)]
#[serde(tag = "action", rename_all = "camelCase", deny_unknown_fields)]
pub(super) enum Request {
    Info,
    Preview { message: i64 },
    Create { request: Create },
    Archive { offset: usize },
    Target { target: String, revision: u32 },
}

impl Engine {
    pub(super) fn prompt_branch_call(
        &self,
        session: &str,
        request: Request,
        active: &run_journal::RunCoordinator,
    ) -> Result<Value, String> {
        match request {
            Request::Info => Ok(json!(self.store.prompt_branch(session)?)),
            Request::Preview { message } => {
                Ok(json!(self.store.preview_prompt_branch(session, message)?))
            }
            Request::Archive { offset } => {
                Ok(json!(self.store.prompt_branch_archive(session, offset)?))
            }
            Request::Target { target, revision } => {
                let record = self
                    .store
                    .prompt_branch_target(session, &target, revision)?;
                self.check_prompt_branch_target(&record, active)?;
                let _pins = self.pin_worktree(&target)?.ok_or("Managed checkout identity is unavailable. Inspect Worktrees; your prompt remains.")?;
                Ok(json!(record))
            }
            Request::Create { request } => {
                if let Some(saved) = self.store.prompt_branch(&request.id)? {
                    if saved.origin.parent != session || saved.request != request {
                        return Err("Branch identity already used. Inspect the saved branch; no draft was replaced.".into());
                    }
                    return Ok(json!(self.store.create_prompt_branch(session, &request)?));
                }
                if request.folder == Folder::Shared {
                    let workspace = self.store.workspace(session)?;
                    let root = workspace.root.ok_or("Original chat has no working folder. Choose No working folder; your edit remains.")?;
                    crate::workspace::canonical_folder(Path::new(&root)).map_err(|e|format!("The current shared folder is unavailable: {e} Choose No working folder or restore the original folder; your edit remains."))?;
                }
                let _pins = if request.folder == Folder::Managed {
                    let record = self.store.prompt_branch_target(
                        session,
                        &request.id,
                        request
                            .target_revision
                            .ok_or("Choose the current worktree revision.")?,
                    )?;
                    self.check_prompt_branch_target(&record, active)?;
                    Some(self.pin_worktree(&request.id)?.ok_or("Managed checkout identity is unavailable. Inspect Worktrees; your prompt remains.")?)
                } else {
                    None
                };
                Ok(json!(self.store.create_prompt_branch(session, &request)?))
            }
        }
    }
    fn check_prompt_branch_target(
        &self,
        record: &dolores_core::worktrees::ManagedWorktree,
        active: &run_journal::RunCoordinator,
    ) -> Result<(), String> {
        if active.has_session(&record.session) {
            return Err("The selected worktree has an active task. Keep it and choose a new empty managed worktree.".into());
        }
        let root = Path::new(&record.root);
        if self
            .terminals
            .lock()
            .map_err(|_| "Terminal ownership is unavailable.")?
            .checkout_pinned(root)
            || self
                .languages
                .lock()
                .map_err(|_| "Language ownership is unavailable.")?
                .checkout_pinned(root)
        {
            return Err("The selected worktree has terminal or language-service activity. Keep it and choose a new empty managed worktree.".into());
        }
        self.editor
            .lock()
            .map_err(|_| "Editor ownership is unavailable.")?
            .ensure_git_clean(root, &[])?;
        // Engine's owner lock remains held through target admission and attachment.
        // Reuse the management overlap check; no Git process is started here.
        let mut git = self
            .git
            .lock()
            .map_err(|_| "Repository ownership is unavailable.")?;
        let check = format!("prompt-branch-target-{}", record.session);
        git.reserve_worktree(
            &check,
            &[
                record.root.clone().into(),
                record.common_directory.clone().into(),
            ],
        )?;
        git.release_worktree(&check);
        Ok(())
    }
}
