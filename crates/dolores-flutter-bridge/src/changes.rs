use super::Engine;
use dolores_core::{ChangeJournal, ChangeSnapshot, WorkspaceJournal};
use dolores_tools_fs::{RemoveCreatedPlan, RevertPlan};
use serde_json::{json, Value};
use std::{
    path::Path,
    sync::Arc,
    time::{Duration, Instant},
};

pub(super) struct PendingRevert {
    token: String,
    session: String,
    root: String,
    change_id: i64,
    created: Instant,
    plan: ReversePlan,
}

enum ReversePlan {
    Restore(RevertPlan),
    Remove(RemoveCreatedPlan),
}
impl ReversePlan {
    fn diff(&self) -> &str {
        match self {
            Self::Restore(plan) => plan.diff(),
            Self::Remove(plan) => plan.diff(),
        }
    }
    fn operation(&self) -> &str {
        match self {
            Self::Restore(_) => "restore",
            Self::Remove(_) => "remove",
        }
    }
    fn apply(self, journal: Arc<dyn ChangeJournal>) -> Result<String, String> {
        match self {
            Self::Restore(plan) => plan.apply(journal),
            Self::Remove(plan) => plan.apply(journal),
        }
    }
}

impl Engine {
    fn change_root(&self, session: &str) -> Result<String, String> {
        self.store
            .workspace(session)?
            .root
            .ok_or("Side chats do not have file changes.".into())
    }
    fn scoped_change(&self, session: &str, id: i64) -> Result<ChangeSnapshot, String> {
        let root = self.change_root(session)?;
        let entry = self.store.change_snapshot(id)?;
        if entry.root != root {
            return Err("This change belongs to another working folder.".into());
        }
        Ok(entry)
    }
    pub(super) fn clear_revert(&self) -> Result<(), String> {
        self.revert
            .lock()
            .map_err(|_| "Revert is unavailable.")?
            .take();
        Ok(())
    }
    pub(super) fn changes_page(&self, session: &str, cursor: Option<i64>) -> Result<Value, String> {
        Ok(json!(self
            .store
            .changes_page(&self.change_root(session)?, cursor)?))
    }
    pub(super) fn change_details(&self, session: &str, id: i64) -> Result<Value, String> {
        let entry = self.scoped_change(session, id)?;
        Ok(
            json!({"change":entry.change,"diff":dolores_tools_fs::file_change_diff(
                entry.change.before_exists.then_some(entry.before.as_str()),
                entry.change.after_exists.then_some(entry.after.as_str()))}),
        )
    }
    pub(super) fn preview_revert(&self, session: &str, id: i64) -> Result<Value, String> {
        self.clear_revert()?;
        let entry = self.scoped_change(session, id)?;
        if entry.change.reverts.is_some()
            || !matches!(entry.change.status.as_str(), "pending" | "applied")
        {
            return Err("This change can no longer be reverted.".into());
        }
        let plan = match (entry.change.before_exists, entry.change.after_exists) {
            (true, true) => ReversePlan::Restore(RevertPlan::preview(
                Path::new(&entry.root),
                &entry.change.target,
                &entry.after,
                &entry.before,
            )?),
            (false, true) => ReversePlan::Remove(RemoveCreatedPlan::preview(
                Path::new(&entry.root),
                &entry.change.target,
                &entry.after,
            )?),
            _ => return Err("This change can no longer be reverted.".into()),
        };
        let token = uuid::Uuid::new_v4().to_string();
        let response = json!({"token":token,"target":entry.change.target,"diff":plan.diff(),"operation":plan.operation()});
        *self.revert.lock().map_err(|_| "Revert is unavailable.")? = Some(PendingRevert {
            token,
            session: session.into(),
            root: entry.root,
            change_id: id,
            created: Instant::now(),
            plan,
        });
        Ok(response)
    }
    pub(super) fn cancel_revert(&self, token: &str) -> Result<Value, String> {
        let mut slot = self.revert.lock().map_err(|_| "Revert is unavailable.")?;
        if slot.as_ref().is_some_and(|p| p.token == token) {
            slot.take();
        }
        Ok(Value::Null)
    }
    pub(super) fn apply_revert(&self, session: &str, token: &str) -> Result<Value, String> {
        let pending = self
            .revert
            .lock()
            .map_err(|_| "Revert is unavailable.")?
            .take()
            .ok_or("Revert preview expired. Review it again.")?;
        if pending.token != token
            || pending.session != session
            || pending.created.elapsed() > Duration::from_secs(300)
        {
            return Err("Revert preview expired. Review it again.".into());
        }
        let root = self.change_root(session)?;
        if root != pending.root {
            return Err("Working folder changed. Review the revert again.".into());
        }
        let journal = Arc::new(WorkspaceJournal {
            store: self.store.clone(),
            root,
            session: session.into(),
            reverts: Some(pending.change_id),
        });
        let result = pending.plan.apply(journal)?;
        serde_json::from_str(&result).map_err(|_| "Revert result could not be read.".into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Command, Run};
    use dolores_core::{ChangeDraft, SessionStore, SessionWorkspace, WorkspaceKind};
    use dolores_store_sqlite::SqliteStore;

    fn setup(root: &Path) -> (Engine, Arc<SqliteStore>, i64) {
        let store = Arc::new(SqliteStore::open(Path::new(":memory:")).unwrap());
        let root = crate::workspace::canonical_folder(root).unwrap();
        store
            .create_workspace_session(
                "chat",
                &SessionWorkspace {
                    kind: WorkspaceKind::Project,
                    root: Some(root.clone()),
                },
            )
            .unwrap();
        let id = store
            .begin_change(&ChangeDraft {
                root,
                session: "chat".into(),
                target: "note".into(),
                before: "before 世界\r\n".into(),
                after: "after 世界\r\n".into(),
                before_exists: true,
                after_exists: true,
                reverts: None,
            })
            .unwrap();
        store.finish_change(id, true).unwrap();
        let engine = Engine::new(
            store.clone(),
            Arc::new(crate::connection::testing::MemoryCredentials::default()),
        )
        .unwrap();
        (engine, store, id)
    }
    #[test]
    fn scoped_preview_cancel_wrong_token_and_expiry_do_not_write() {
        let dir = tempfile::tempdir().unwrap();
        let file = dir.path().join("note");
        std::fs::write(&file, "after 世界\r\n").unwrap();
        let (engine, store, id) = setup(dir.path());
        store.create("side").unwrap();
        store
            .create_workspace_session(
                "other",
                &SessionWorkspace {
                    kind: WorkspaceKind::Project,
                    root: Some("another folder".into()),
                },
            )
            .unwrap();
        assert!(engine.change_details("other", id).is_err());
        assert!(engine.changes_page("side", None).is_err());
        let detail = engine.change_details("chat", id).unwrap().to_string();
        assert!(
            !detail.contains("root") && !detail.contains(&dir.path().to_string_lossy().to_string())
        );
        let p = engine.preview_revert("chat", id).unwrap();
        assert!(p["diff"].as_str().unwrap().contains("+before 世界\r\n"));
        assert_eq!(std::fs::read_to_string(&file).unwrap(), "after 世界\r\n");
        engine.cancel_revert(p["token"].as_str().unwrap()).unwrap();
        assert!(engine
            .apply_revert("chat", p["token"].as_str().unwrap())
            .is_err());
        engine.preview_revert("chat", id).unwrap();
        assert!(engine.apply_revert("chat", "wrong").is_err());
        let p = engine.preview_revert("chat", id).unwrap();
        engine.revert.lock().unwrap().as_mut().unwrap().created =
            Instant::now() - Duration::from_secs(301);
        assert!(engine
            .apply_revert("chat", p["token"].as_str().unwrap())
            .is_err());
        let p = engine.preview_revert("chat", id).unwrap();
        assert!(engine
            .apply_revert("other", p["token"].as_str().unwrap())
            .is_err());
        assert_eq!(std::fs::read_to_string(&file).unwrap(), "after 世界\r\n");
        assert_eq!(
            store
                .changes_page(&store.workspace("chat").unwrap().root.unwrap(), None)
                .unwrap()
                .items
                .len(),
            1
        );
    }
    #[test]
    fn revert_publication_conflict_and_generation_exclusion_then_single_use_success() {
        let dir = tempfile::tempdir().unwrap();
        let file = dir.path().join("note");
        std::fs::write(&file, "after 世界\r\n").unwrap();
        let (engine, store, id) = setup(dir.path());
        let root = store.workspace("chat").unwrap().root.unwrap();
        let p = engine.preview_revert("chat", id).unwrap();
        std::fs::write(&file, "external").unwrap();
        assert!(engine
            .apply_revert("chat", p["token"].as_str().unwrap())
            .is_err());
        assert_eq!(std::fs::read_to_string(&file).unwrap(), "external");
        std::fs::write(&file, "after 世界\r\n").unwrap();
        let (_, events) = tokio::sync::mpsc::channel(1);
        engine
            .active
            .lock()
            .unwrap()
            .reserve(Run {
                id: 1,
                cancel: tokio_util::sync::CancellationToken::new(),
                events,
                approvals: Arc::new(std::sync::Mutex::new(None)),
            })
            .unwrap();
        assert!(engine
            .call(Command::ChangesPage {
                session: "chat".into(),
                cursor: None,
            })
            .is_ok());
        for command in [
            Command::PreviewRevert {
                session: "chat".into(),
                change_id: id,
            },
            Command::ApplyRevert {
                session: "chat".into(),
                token: "unknown".into(),
            },
        ] {
            assert!(engine.call(command).is_err());
        }
        engine.active.lock().unwrap().take();
        let p = engine.preview_revert("chat", id).unwrap();
        let token = p["token"].as_str().unwrap();
        assert_eq!(engine.apply_revert("chat", token).unwrap()["applied"], true);
        assert!(engine.apply_revert("chat", token).is_err());
        assert!(engine.preview_revert("chat", id).is_err());
        assert_eq!(std::fs::read_to_string(file).unwrap(), "before 世界\r\n");
        let page = store.changes_page(&root, None).unwrap();
        assert_eq!(page.items.len(), 2);
        assert_eq!(page.items[0].reverts, Some(id));
        assert_eq!(page.items[1].status, "reverted");
    }

    #[test]
    fn created_empty_file_removal_is_scoped_reviewed_cancelable_and_single_use() {
        for content in ["", "new 世界\r\n"] {
            let dir = tempfile::tempdir().unwrap();
            let (engine, store, _) = setup(dir.path());
            let root = store.workspace("chat").unwrap().root.unwrap();
            let id = store
                .begin_change(&ChangeDraft {
                    root,
                    session: "chat".into(),
                    target: "new".into(),
                    before: String::new(),
                    after: content.into(),
                    before_exists: false,
                    after_exists: true,
                    reverts: None,
                })
                .unwrap();
            store.finish_change(id, true).unwrap();
            let file = dir.path().join("new");
            std::fs::write(&file, content).unwrap();
            let detail = engine.change_details("chat", id).unwrap();
            assert_eq!(detail["change"]["beforeExists"], false);
            assert!(detail["diff"]
                .as_str()
                .unwrap()
                .starts_with("--- /dev/null"));
            let preview = engine.preview_revert("chat", id).unwrap();
            assert_eq!(preview["operation"], "remove");
            assert!(preview["diff"].as_str().unwrap().contains("+++ /dev/null"));
            assert!(file.exists());
            engine
                .cancel_revert(preview["token"].as_str().unwrap())
                .unwrap();
            assert!(engine
                .apply_revert("chat", preview["token"].as_str().unwrap())
                .is_err());
            let preview = engine.preview_revert("chat", id).unwrap();
            std::fs::write(&file, "external").unwrap();
            assert!(engine
                .apply_revert("chat", preview["token"].as_str().unwrap())
                .is_err());
            assert_eq!(std::fs::read_to_string(&file).unwrap(), "external");
            std::fs::write(&file, content).unwrap();
            let preview = engine.preview_revert("chat", id).unwrap();
            let result = engine
                .apply_revert("chat", preview["token"].as_str().unwrap())
                .unwrap();
            assert_eq!(result["removed"], true);
            assert!(!file.exists());
            assert!(engine.preview_revert("chat", id).is_err());
            let removal = store
                .change_snapshot(result["changeId"].as_i64().unwrap())
                .unwrap()
                .change;
            assert!(removal.before_exists && !removal.after_exists && removal.reverts == Some(id));
            assert!(engine.preview_revert("chat", removal.id).is_err());
        }
    }
}
