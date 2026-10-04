use super::{storage_error, SqliteStore};
use rusqlite::{params, OptionalExtension};
impl SqliteStore {
    pub(super) fn read_draft(&self, id: &str) -> Result<String, String> {
        let db = self.lock()?;
        let present: bool = db
            .query_row(
                "SELECT EXISTS(SELECT 1 FROM sessions WHERE id=?1)",
                [id],
                |r| r.get(0),
            )
            .map_err(storage_error)?;
        if !present {
            return Err("Chat is unavailable.".into());
        }
        Ok(db
            .query_row(
                "SELECT text FROM session_drafts WHERE session_id=?1",
                [id],
                |r| r.get(0),
            )
            .optional()
            .map_err(storage_error)?
            .unwrap_or_default())
    }
    pub(super) fn write_draft(&self, id: &str, text: &str) -> Result<(), String> {
        if text.len() > dolores_core::MAX_INPUT_BYTES || text.contains('\0') {
            return Err("Draft exceeds the message allowance. Shorten it; the previous saved draft remains.".into());
        }
        self.lock()?.execute("INSERT INTO session_drafts(session_id,text) VALUES(?1,?2) ON CONFLICT(session_id) DO UPDATE SET text=excluded.text",params![id,text]).map_err(storage_error)?;
        Ok(())
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use dolores_core::SessionStore;
    #[test]
    fn restart_invalid_save_conditional_clear_and_delete_preserve_drafts() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("state.db");
        let store = SqliteStore::open(&path).unwrap();
        store.create("one").unwrap();
        store.create("two").unwrap();
        store.save_draft("one", "世界 draft").unwrap();
        assert!(store
            .save_draft("one", &"x".repeat(dolores_core::MAX_INPUT_BYTES + 1))
            .is_err());
        drop(store);
        let store = SqliteStore::open(&path).unwrap();
        assert_eq!(store.saved_draft("one").unwrap(), "世界 draft");
        assert_eq!(store.saved_draft("two").unwrap(), "");
        store.clear_draft_if("one", "old").unwrap();
        assert_eq!(store.saved_draft("one").unwrap(), "世界 draft");
        store.delete("one").unwrap();
        assert!(store.saved_draft("one").is_err());
    }
}
