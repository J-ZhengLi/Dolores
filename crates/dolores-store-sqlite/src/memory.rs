use super::{now, storage_error, SqliteStore};
use dolores_core::{MemoryDraft, MemoryPreference, MemoryScope, MAX_PREFERENCES_PER_SCOPE};
use rusqlite::{params, OptionalExtension};

const CONFLICT: &str =
    "Memory preference changed or was deleted. Cancel, refresh and review it again.";
fn scope_key(root: Option<&str>) -> Result<&str, String> {
    match root {
        Some(root) if !std::path::Path::new(root).is_absolute() => {
            Err("Memory folder scope is invalid.".into())
        }
        _ => Ok(root.unwrap_or("")),
    }
}
fn decode(data: String) -> Result<MemoryPreference, String> {
    let value: MemoryPreference = serde_json::from_str(&data)
        .map_err(|_| "Saved memory preference could not be read. Review Memory before sending.")?;
    value.validate()?;
    Ok(value)
}
impl SqliteStore {
    pub(super) fn read_memory(&self, root: Option<&str>) -> Result<Vec<MemoryPreference>, String> {
        let root = scope_key(root)?;
        let conn = self.lock()?;
        let mut query = conn.prepare("SELECT root,data FROM memory_preferences WHERE root='' OR (root=?1 AND ?1!='') ORDER BY root,id LIMIT 25").map_err(storage_error)?;
        let rows = query
            .query_map([root], |row| {
                Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
            })
            .map_err(storage_error)?;
        let mut values = Vec::new();
        for row in rows {
            let (scope, data) = row.map_err(storage_error)?;
            let value = decode(data)?;
            if value.scope
                != if scope.is_empty() {
                    MemoryScope::All
                } else {
                    MemoryScope::Folder
                }
            {
                return Err("Saved memory preference scope is invalid.".into());
            }
            values.push(value);
        }
        if values.len() > MAX_PREFERENCES_PER_SCOPE * 2 {
            return Err("Saved memory preferences exceed the local entry limit.".into());
        }
        if [MemoryScope::All, MemoryScope::Folder].iter().any(|scope| {
            values.iter().filter(|p| p.scope == *scope).count() > MAX_PREFERENCES_PER_SCOPE
        }) {
            return Err("Saved memory preferences exceed the local scope limit.".into());
        }
        values.sort_by(|a, b| {
            b.updated_at
                .cmp(&a.updated_at)
                .then_with(|| a.id.cmp(&b.id))
        });
        Ok(values)
    }
    pub(super) fn write_memory(
        &self,
        root: Option<&str>,
        draft: &MemoryDraft,
    ) -> Result<MemoryPreference, String> {
        let root = scope_key(root)?;
        draft.validate()?;
        let mut conn = self.lock()?;
        let tx = conn.transaction().map_err(storage_error)?;
        let previous: Option<String> = tx
            .query_row(
                "SELECT data FROM memory_preferences WHERE id=?1 AND root=?2",
                params![draft.id, root],
                |r| r.get(0),
            )
            .optional()
            .map_err(storage_error)?;
        let timestamp = now();
        let (revision, created_at) = match (previous, draft.revision) {
            (None, None) => {
                let count: usize = tx
                    .query_row(
                        "SELECT count(*) FROM memory_preferences WHERE root=?1",
                        [root],
                        |r| r.get(0),
                    )
                    .map_err(storage_error)?;
                if count >= MAX_PREFERENCES_PER_SCOPE {
                    return Err("Memory has reached its 12-preference limit for this scope. Delete an unused preference first.".into());
                }
                (1, timestamp)
            }
            (Some(data), Some(expected)) => {
                let previous = decode(data)?;
                if previous.revision != expected {
                    return Err(CONFLICT.into());
                }
                (
                    expected.checked_add(1).ok_or(CONFLICT)?,
                    previous.created_at,
                )
            }
            _ => return Err(CONFLICT.into()),
        };
        let value = MemoryPreference {
            id: draft.id.clone(),
            revision,
            title: draft.title.clone(),
            text: draft.text.clone(),
            scope: if root.is_empty() {
                MemoryScope::All
            } else {
                MemoryScope::Folder
            },
            source: "user".into(),
            enabled: draft.enabled,
            created_at,
            updated_at: timestamp,
        };
        let data = serde_json::to_string(&value).map_err(storage_error)?;
        if draft.revision.is_some() {
            tx.execute(
                "UPDATE memory_preferences SET data=?1 WHERE id=?2 AND root=?3",
                params![data, value.id, root],
            )
            .map_err(storage_error)?;
        } else {
            tx.execute(
                "INSERT INTO memory_preferences(id,root,data) VALUES(?1,?2,?3)",
                params![value.id, root, data],
            )
            .map_err(storage_error)?;
        }
        tx.commit().map_err(storage_error)?;
        Ok(value)
    }
    pub(super) fn remove_memory(
        &self,
        root: Option<&str>,
        id: &str,
        revision: u32,
    ) -> Result<(), String> {
        let root = scope_key(root)?;
        if !dolores_core::valid_memory_id(id) || revision == 0 {
            return Err("Memory preference identity is invalid.".into());
        }
        let mut conn = self.lock()?;
        let tx = conn.transaction().map_err(storage_error)?;
        let data: String = tx
            .query_row(
                "SELECT data FROM memory_preferences WHERE id=?1 AND root=?2",
                params![id, root],
                |r| r.get(0),
            )
            .optional()
            .map_err(storage_error)?
            .ok_or(CONFLICT)?;
        if decode(data)?.revision != revision {
            return Err(CONFLICT.into());
        }
        tx.execute(
            "DELETE FROM memory_preferences WHERE id=?1 AND root=?2",
            params![id, root],
        )
        .map_err(storage_error)?;
        tx.commit().map_err(storage_error)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use dolores_core::SessionStore;
    fn draft(id: &str) -> MemoryDraft {
        MemoryDraft {
            id: id.into(),
            revision: None,
            title: "Response style".into(),
            text: "Prefer concise examples. 世界".into(),
            enabled: true,
        }
    }
    #[test]
    fn revisions_scope_atomic_failures_deletion_restart_and_history_stay_consistent() {
        let temp = tempfile::tempdir().unwrap();
        let db = temp.path().join("state.db");
        let root = temp.path().to_str().unwrap();
        let store = SqliteStore::open(&db).unwrap();
        store.create("chat").unwrap();
        store.commit_turn("chat", "hello", "answer").unwrap();
        let all = store.save_memory_preference(None, &draft("all")).unwrap();
        let folder = store
            .save_memory_preference(Some(root), &draft("folder"))
            .unwrap();
        assert_eq!(store.memory_preferences(None).unwrap(), vec![all.clone()]);
        assert_eq!(store.memory_preferences(Some(root)).unwrap().len(), 2);
        let mut corrected = draft("all");
        corrected.revision = Some(all.revision);
        corrected.text = "Prefer one example.".into();
        corrected.enabled = false;
        let corrected = store.save_memory_preference(None, &corrected).unwrap();
        assert_eq!(corrected.revision, 2);
        assert_eq!(corrected.created_at, all.created_at);
        let stale = MemoryDraft {
            revision: Some(1),
            ..draft("all")
        };
        assert!(store.save_memory_preference(None, &stale).is_err());
        assert!(store.delete_memory_preference(None, "all", 1).is_err());
        assert!(store
            .delete_memory_preference(None, "folder", folder.revision)
            .is_err());
        assert!(store
            .save_memory_preference(
                Some(root),
                &MemoryDraft {
                    revision: Some(2),
                    ..draft("all")
                }
            )
            .is_err());
        store.lock().unwrap().execute_batch("CREATE TRIGGER fail_memory_update BEFORE UPDATE ON memory_preferences BEGIN SELECT RAISE(ABORT,'fixture'); END; CREATE TRIGGER fail_memory_delete BEFORE DELETE ON memory_preferences BEGIN SELECT RAISE(ABORT,'fixture'); END;").unwrap();
        assert!(store
            .save_memory_preference(
                None,
                &MemoryDraft {
                    revision: Some(2),
                    ..draft("all")
                }
            )
            .is_err());
        assert!(store.delete_memory_preference(None, "all", 2).is_err());
        assert_eq!(
            store.memory_preferences(None).unwrap(),
            vec![corrected.clone()]
        );
        store
            .lock()
            .unwrap()
            .execute_batch("DROP TRIGGER fail_memory_update; DROP TRIGGER fail_memory_delete;")
            .unwrap();
        drop(store);
        let store = SqliteStore::open(&db).unwrap();
        assert_eq!(store.memory_preferences(None).unwrap(), vec![corrected]);
        assert_eq!(store.messages("chat").unwrap().len(), 2);
        store.delete("chat").unwrap();
        assert_eq!(store.memory_preferences(Some(root)).unwrap().len(), 2);
        store.delete_memory_preference(None, "all", 2).unwrap();
        assert!(store.memory_preferences(None).unwrap().is_empty());
        assert_eq!(store.memory_preferences(Some(root)).unwrap(), vec![folder]);
    }
    #[test]
    fn scope_creation_caps_and_invalid_text_cannot_modify_other_records() {
        let store = SqliteStore::open(std::path::Path::new(":memory:")).unwrap();
        for n in 0..12 {
            store
                .save_memory_preference(None, &draft(&format!("entry-{n}")))
                .unwrap();
        }
        assert!(store
            .save_memory_preference(None, &draft("excess"))
            .is_err());
        assert!(store
            .save_memory_preference(
                None,
                &MemoryDraft {
                    text: "bad\0text".into(),
                    ..draft("bad")
                }
            )
            .is_err());
        assert!(store
            .save_memory_preference(Some("relative"), &draft("bad"))
            .is_err());
        assert_eq!(store.memory_preferences(None).unwrap().len(), 12);
        store.delete_memory_preference(None, "entry-0", 1).unwrap();
        store
            .save_memory_preference(None, &draft("replacement"))
            .unwrap();
        assert_eq!(store.memory_preferences(None).unwrap().len(), 12);
    }
}
