use super::{storage_error, SqliteStore};
use dolores_core::WorkspaceInstructions;
use rusqlite::{params, OptionalExtension};

impl SqliteStore {
    pub(super) fn read_instructions(
        &self,
        root: &str,
    ) -> Result<Option<WorkspaceInstructions>, String> {
        let data: Option<String> = self
            .lock()?
            .query_row(
                "SELECT data FROM workspace_instructions WHERE root=?1",
                [root],
                |r| r.get(0),
            )
            .optional()
            .map_err(storage_error)?;
        let value = data.map(|data| serde_json::from_str::<WorkspaceInstructions>(&data))
            .transpose().map_err(|_| "Saved workspace instructions could not be read. Disable them and review AGENTS.md again.")?;
        if let Some(value) = &value {
            value.validate()?;
        }
        Ok(value)
    }
    pub(super) fn save_instructions(
        &self,
        root: &str,
        value: Option<&WorkspaceInstructions>,
    ) -> Result<(), String> {
        if !std::path::Path::new(root).is_absolute() {
            return Err("Instructions need an absolute working folder.".into());
        }
        if let Some(value) = value {
            value.validate()?;
            let data = serde_json::to_string(value).map_err(storage_error)?;
            self.lock()?.execute("INSERT INTO workspace_instructions(root,data) VALUES(?1,?2) ON CONFLICT(root) DO UPDATE SET data=excluded.data", params![root,data]).map_err(storage_error)?;
        } else {
            self.lock()?
                .execute("DELETE FROM workspace_instructions WHERE root=?1", [root])
                .map_err(storage_error)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use dolores_core::{InstructionSource, SessionStore};
    #[test]
    fn snapshots_are_folder_scoped_atomic_and_restored_without_history_changes() {
        let temp = tempfile::tempdir().unwrap();
        let db = temp.path().join("state.db");
        let root = temp.path().to_str().unwrap();
        let value = WorkspaceInstructions {
            provenance: InstructionSource {
                source: "AGENTS.md".into(),
                revision: "12345678-1234-1234-1234-123456789abc".into(),
                approved_at: 1,
                text_bytes: 5,
            },
            text: "hello".into(),
        };
        {
            let store = SqliteStore::open(&db).unwrap();
            store.create("chat").unwrap();
            store.commit_turn("chat", "question", "answer").unwrap();
            store
                .save_workspace_instructions(root, Some(&value))
                .unwrap();
            assert!(store
                .workspace_instructions(&format!("{root}/other"))
                .unwrap()
                .is_none());
            store.lock().unwrap().execute_batch("CREATE TRIGGER refuse_instruction_update BEFORE UPDATE ON workspace_instructions BEGIN SELECT RAISE(ABORT,'fixture'); END;").unwrap();
            let mut replacement = value.clone();
            replacement.text = "world".into();
            assert!(store
                .save_workspace_instructions(root, Some(&replacement))
                .is_err());
            assert_eq!(
                store.workspace_instructions(root).unwrap(),
                Some(value.clone())
            );
        }
        let store = SqliteStore::open(&db).unwrap();
        assert_eq!(store.workspace_instructions(root).unwrap(), Some(value));
        assert_eq!(store.messages("chat").unwrap().len(), 2);
        store.delete("chat").unwrap();
        assert!(store.workspace_instructions(root).unwrap().is_some());
        store.save_workspace_instructions(root, None).unwrap();
        assert!(store.workspace_instructions(root).unwrap().is_none());
    }
}
