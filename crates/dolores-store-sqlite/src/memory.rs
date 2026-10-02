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
    pub(super) fn read_memory_sources(
        &self,
        session: &str,
    ) -> Result<dolores_core::HistoryPage<dolores_core::MemoryMessage>, String> {
        let conn = self.lock()?;
        if !conn
            .query_row(
                "SELECT EXISTS(SELECT 1 FROM sessions WHERE id=?1)",
                [session],
                |r| r.get::<_, bool>(0),
            )
            .map_err(storage_error)?
        {
            return Err("Conversation no longer exists.".into());
        }
        let mut query = conn.prepare("SELECT m.id,m.content FROM messages m WHERE m.session_id=?1 AND m.role='user' AND EXISTS(SELECT 1 FROM messages a WHERE a.session_id=m.session_id AND a.id=m.id+1 AND a.role='assistant') ORDER BY m.id DESC LIMIT 21").map_err(storage_error)?;
        let mut items = query
            .query_map([session], |r| {
                Ok(dolores_core::MemoryMessage {
                    message_id: r.get(0)?,
                    text: r.get(1)?,
                })
            })
            .map_err(storage_error)?
            .collect::<Result<Vec<_>, _>>()
            .map_err(storage_error)?;
        let has_older = items.len() > 20;
        items.truncate(20);
        Ok(dolores_core::HistoryPage {
            items,
            has_older,
            has_newer: false,
        })
    }
    pub(super) fn read_memory_source(
        &self,
        session: &str,
        id: i64,
    ) -> Result<Option<dolores_core::MemoryMessage>, String> {
        self.lock()?.query_row("SELECT m.id,m.content FROM messages m WHERE m.session_id=?1 AND m.id=?2 AND m.role='user' AND EXISTS(SELECT 1 FROM messages a WHERE a.session_id=m.session_id AND a.id=m.id+1 AND a.role='assistant')", params![session,id], |r| Ok(dolores_core::MemoryMessage {message_id:r.get(0)?,text:r.get(1)?})).optional().map_err(storage_error)
    }
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
        self.write_memory_with_sources(root, draft, None)
    }
    pub(super) fn write_suggested_memory(
        &self,
        root: Option<&str>,
        draft: &MemoryDraft,
        sources: &[dolores_core::MemoryMessage],
    ) -> Result<MemoryPreference, String> {
        self.write_memory_with_sources(root, draft, Some(sources))
    }
    fn write_memory_with_sources(
        &self,
        root: Option<&str>,
        draft: &MemoryDraft,
        sources: Option<&[dolores_core::MemoryMessage]>,
    ) -> Result<MemoryPreference, String> {
        let root = scope_key(root)?;
        draft.validate()?;
        let mut conn = self.lock()?;
        let tx = conn.transaction().map_err(storage_error)?;
        match (&draft.origin, sources) {
            (None, None) => {}
            (Some(origin), Some(sources)) if draft.revision.is_none() => {
                dolores_core::memory_suggestion_prompt(sources)?;
                if !sources
                    .iter()
                    .any(|s| s.message_id == origin.message_id && s.text.contains(&origin.quote))
                {
                    return Err("Memory source reference changed. Review this chat again.".into());
                }
                for source in sources {
                    let text:Option<String> = tx.query_row("SELECT m.content FROM messages m WHERE m.session_id=?1 AND m.id=?2 AND m.role='user' AND EXISTS(SELECT 1 FROM messages a WHERE a.session_id=m.session_id AND a.id=m.id+1 AND a.role='assistant')",params![origin.session,source.message_id],|r|r.get(0)).optional().map_err(storage_error)?;
                    if text.as_deref() != Some(source.text.as_str()) {
                        return Err(
                            "Memory source reference changed. Review this chat again.".into()
                        );
                    }
                }
            }
            _ => {
                return Err(
                    "Memory source reference is invalid. Review the suggestion again.".into(),
                )
            }
        }
        let previous: Option<String> = tx
            .query_row(
                "SELECT data FROM memory_preferences WHERE id=?1 AND root=?2",
                params![draft.id, root],
                |r| r.get(0),
            )
            .optional()
            .map_err(storage_error)?;
        let timestamp = now();
        let (revision, created_at, origin) = match (previous, draft.revision) {
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
                (1, timestamp, draft.origin.clone())
            }
            (Some(data), Some(expected)) => {
                if draft.origin.is_some() {
                    return Err("Memory source cannot be replaced during editing.".into());
                }
                let previous = decode(data)?;
                if previous.revision != expected {
                    return Err(CONFLICT.into());
                }
                (
                    expected.checked_add(1).ok_or(CONFLICT)?,
                    previous.created_at,
                    previous.origin,
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
            source: if origin.is_some() {
                "conversation"
            } else {
                "user"
            }
            .into(),
            enabled: draft.enabled,
            created_at,
            updated_at: timestamp,
            origin,
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
            origin: None,
        }
    }
    #[test]
    fn suggested_sources_are_atomic_scoped_and_preserved_through_edits_and_restart() {
        let temp = tempfile::tempdir().unwrap();
        let db = temp.path().join("state.db");
        let store = SqliteStore::open(&db).unwrap();
        store.create("chat").unwrap();
        store.create("other").unwrap();
        for n in 0..21 {
            store
                .commit_turn(
                    "chat",
                    &format!("I prefer concise examples {n}."),
                    "Assistant suggestion is not a source.",
                )
                .unwrap();
        }
        let page = store.memory_source_messages("chat").unwrap();
        assert_eq!(page.items.len(), 20);
        assert!(page.has_older);
        let source = page.items[0].clone();
        assert!(store
            .memory_source_message("other", source.message_id)
            .unwrap()
            .is_none());
        assert!(store
            .memory_source_message("chat", source.message_id + 1)
            .unwrap()
            .is_none());
        let mut suggested = draft("reviewed");
        suggested.origin = Some(dolores_core::MemoryOrigin {
            session: "chat".into(),
            message_id: source.message_id,
            quote: "I prefer concise examples".into(),
            model: "fixture".into(),
            reviewed_at: 1,
        });
        assert!(store.save_memory_preference(None, &suggested).is_err());
        let mut stale = source.clone();
        stale.text.push_str("changed");
        assert!(store
            .save_suggested_memory_preference(None, &suggested, &[stale])
            .is_err());
        let saved = store
            .save_suggested_memory_preference(None, &suggested, std::slice::from_ref(&source))
            .unwrap();
        assert_eq!(saved.source, "conversation");
        let corrected = MemoryDraft {
            revision: Some(1),
            text: "Prefer short replies.".into(),
            ..draft("reviewed")
        };
        let saved = store.save_memory_preference(None, &corrected).unwrap();
        assert_eq!(saved.origin, suggested.origin);
        assert_eq!(saved.revision, 2);
        drop(store);
        let store = SqliteStore::open(&db).unwrap();
        assert_eq!(store.memory_preferences(None).unwrap()[0], saved);
        store.delete("chat").unwrap();
        assert!(store
            .memory_source_message("chat", source.message_id)
            .unwrap()
            .is_none());
        assert_eq!(
            store.memory_preferences(None).unwrap()[0].origin,
            suggested.origin
        );
        suggested.id = "after-deletion".into();
        assert!(store
            .save_suggested_memory_preference(None, &suggested, &[source])
            .is_err());
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
