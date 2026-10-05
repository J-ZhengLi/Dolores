use super::*;
use dolores_core::KnowledgeState;
use rusqlite::OptionalExtension;
pub(super) fn read(conn: &Connection, root: &str) -> Result<KnowledgeState, String> {
    let value: Option<String> = conn
        .query_row(
            "SELECT data FROM project_knowledge WHERE root=?1",
            [root],
            |r| r.get(0),
        )
        .optional()
        .map_err(storage_error)?;
    let state = value
        .map(|v| serde_json::from_str::<KnowledgeState>(&v).map_err(storage_error))
        .transpose()?
        .unwrap_or_default();
    state.validate()?;
    Ok(state)
}
impl SqliteStore {
    pub(super) fn read_knowledge(&self, root: &str) -> Result<KnowledgeState, String> {
        let conn = self.lock()?;
        read(&conn, root)
    }
    pub(super) fn write_knowledge(
        &self,
        root: &str,
        revision: u32,
        state: &KnowledgeState,
    ) -> Result<KnowledgeState, String> {
        if !std::path::Path::new(root).is_absolute() || state.revision != revision {
            return Err("Knowledge scope/revision is invalid.".into());
        }
        state.validate()?;
        let mut conn = self.lock()?;
        let tx = conn.transaction().map_err(storage_error)?;
        if read(&tx, root)?.revision != revision {
            return Err("Project knowledge changed. Refresh and review again.".into());
        }
        let mut next = state.clone();
        next.revision = revision
            .checked_add(1)
            .ok_or("Knowledge revision exhausted.")?;
        tx.execute("INSERT INTO project_knowledge(root,data) VALUES(?1,?2) ON CONFLICT(root) DO UPDATE SET data=excluded.data",params![root,serde_json::to_string(&next).map_err(storage_error)?]).map_err(storage_error)?;
        tx.commit().map_err(storage_error)?;
        Ok(next)
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn opt_in_revision_failed_write_and_restart_preserve_knowledge() {
        let folder = tempfile::tempdir().unwrap();
        let path = folder.path().join("state.db");
        let root = folder.path().to_str().unwrap();
        let store = SqliteStore::open(&path).unwrap();
        let mut state = store.knowledge(root).unwrap();
        assert!(!state.learning && !state.share_feedback);
        state.learning = true;
        let saved = store.save_knowledge(root, 0, &state).unwrap();
        assert!(store.save_knowledge(root, 0, &state).is_err());
        store.lock().unwrap().execute_batch("CREATE TRIGGER reject_knowledge BEFORE UPDATE ON project_knowledge BEGIN SELECT RAISE(ABORT,'fixture'); END;").unwrap();
        let mut changed = saved.clone();
        changed.learning = false;
        assert!(store.save_knowledge(root, 1, &changed).is_err());
        assert_eq!(store.knowledge(root).unwrap(), saved);
        drop(store);
        assert_eq!(
            SqliteStore::open(&path).unwrap().knowledge(root).unwrap(),
            saved
        );
    }
}
