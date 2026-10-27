use super::{skills, storage_error};
use dolores_core::{ProjectSkill, SkillSelection};
use rusqlite::{params, Connection, OptionalExtension};

pub(super) fn available(c: &Connection, s: &str) -> Result<Vec<ProjectSkill>, String> {
    let exists: bool = c
        .query_row(
            "SELECT EXISTS(SELECT 1 FROM sessions WHERE id=?1)",
            [s],
            |r| r.get(0),
        )
        .map_err(storage_error)?;
    if !exists {
        return Err("Chat no longer exists.".into());
    }
    let root: Option<String> = c
        .query_row(
            "SELECT root FROM session_workspaces WHERE session_id=?1",
            [s],
            |r| r.get(0),
        )
        .optional()
        .map_err(storage_error)?
        .flatten();
    let mut saved = root.map_or(Ok(vec![]), |root| skills::read(c, &root))?;
    saved.extend(skills::read(c, skills::GLOBAL_ROOT)?);
    Ok(saved)
}
pub(super) fn read(c: &Connection, s: &str) -> Result<Vec<SkillSelection>, String> {
    let data: Option<String> = c
        .query_row(
            "SELECT data FROM composer_skills WHERE session=?1",
            [s],
            |r| r.get(0),
        )
        .optional()
        .map_err(storage_error)?;
    let selections: Vec<SkillSelection> = data
        .map(|v| serde_json::from_str(&v).map_err(storage_error))
        .transpose()?
        .unwrap_or_default();
    dolores_core::validate_skill_selections(&selections)?;
    Ok(selections)
}
pub(super) fn save(
    c: &mut Connection,
    s: &str,
    selections: &[SkillSelection],
) -> Result<(), String> {
    dolores_core::validate_skill_selections(selections)?;
    let tx = c.transaction().map_err(storage_error)?;
    let saved = available(&tx, s)?;
    let old = read(&tx, s)?;
    // Removing stale selections must remain possible, one chip at a time.
    if !selections.iter().all(|r| old.contains(r)) {
        let retained: Vec<_> = selections.iter().filter(|s| !s.file).cloned().collect();
        // File references are validated against the configured filesystem by the
        // host before save/admission. This layer never reads ambient directories.
        dolores_core::resolve_skill_selections(&saved, &retained)?;
    }
    tx.execute("INSERT INTO composer_skills VALUES(?1,?2) ON CONFLICT(session) DO UPDATE SET data=excluded.data",params![s,serde_json::to_string(selections).map_err(storage_error)?]).map_err(storage_error)?;
    tx.commit().map_err(storage_error)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::SqliteStore;
    use dolores_core::{
        harness_task::{InputEdit, InputMode},
        SessionStore, SkillDocument,
    };
    fn document(text: &str) -> SkillDocument {
        SkillDocument {
            name: "precise-format".into(),
            description: "Structure exports".into(),
            text: text.into(),
        }
    }
    #[test]
    fn selections_reopen_and_queue_snapshot_survives_later_composer_changes() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("skills.db");
        let store = SqliteStore::open(&path).unwrap();
        store.create("chat").unwrap();
        store
            .commit_turn("chat", "Legacy 世界", "Retained")
            .unwrap();
        store
            .lock()
            .unwrap()
            .execute_batch("DROP TABLE composer_skills; PRAGMA user_version=39;")
            .unwrap();
        drop(store);
        let store = SqliteStore::open(&path).unwrap();
        assert_eq!(store.messages("chat").unwrap()[0].content, "Legacy 世界");
        let skill = store
            .activate_global_skill(&document("EXACT_V1"), None, None)
            .unwrap();
        let selected = vec![SkillSelection::from_skill(&skill)];
        store.save_composer_skills("chat", &selected).unwrap();
        store
            .edit_task_inputs(
                "chat",
                &InputEdit::Enqueue {
                    id: "input".into(),
                    text: "Hello".into(),
                    mode: InputMode::Queue,
                },
            )
            .unwrap();
        store.save_composer_skills("chat", &[]).unwrap();
        drop(store);
        let store = SqliteStore::open(&path).unwrap();
        assert!(store.composer_skills("chat").unwrap().is_empty());
        assert_eq!(
            store.task_inputs("chat").unwrap().entries[0].selected_skills,
            selected
        );
        store.delete("chat").unwrap();
        assert!(store.composer_skills("chat").unwrap().is_empty());
    }
    #[test]
    fn failed_save_and_stale_enqueue_preserve_draft_and_allow_stale_removal() {
        let store = SqliteStore::open(std::path::Path::new(":memory:")).unwrap();
        store.create("chat").unwrap();
        let skill = store
            .activate_global_skill(&document("EXACT_V1"), None, None)
            .unwrap();
        let selected = vec![SkillSelection::from_skill(&skill)];
        store.save_draft("chat", "Unsaved thought").unwrap();
        store.lock().unwrap().execute_batch("CREATE TRIGGER reject_selection BEFORE INSERT ON composer_skills BEGIN SELECT RAISE(ABORT,'fixture'); END;").unwrap();
        assert!(store.save_composer_skills("chat", &selected).is_err());
        assert!(store.composer_skills("chat").unwrap().is_empty());
        assert_eq!(store.saved_draft("chat").unwrap(), "Unsaved thought");
        store
            .lock()
            .unwrap()
            .execute_batch("DROP TRIGGER reject_selection;")
            .unwrap();
        store.save_composer_skills("chat", &selected).unwrap();
        store
            .activate_global_skill(&document("EXACT_V2"), Some(1), None)
            .unwrap();
        assert!(store
            .edit_task_inputs(
                "chat",
                &InputEdit::Enqueue {
                    id: "stale".into(),
                    text: "Unsaved thought".into(),
                    mode: InputMode::Steer
                }
            )
            .is_err());
        assert_eq!(store.saved_draft("chat").unwrap(), "Unsaved thought");
        assert!(store.task_inputs("chat").unwrap().entries.is_empty());
        store.save_composer_skills("chat", &[]).unwrap();
        assert!(store.save_composer_skills("chat", &selected).is_err());
    }
}
