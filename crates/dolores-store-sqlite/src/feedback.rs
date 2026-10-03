use super::{now, storage_error, SqliteStore};
use dolores_core::{FeedbackDraft, TaskFeedback, TurnMetadata};
use rusqlite::{params, OptionalExtension};

impl SqliteStore {
    pub(super) fn write_task_feedback(
        &self,
        session: &str,
        draft: &FeedbackDraft,
    ) -> Result<TaskFeedback, String> {
        draft.validate()?;
        let mut connection = self.lock()?;
        let tx = connection
            .transaction_with_behavior(rusqlite::TransactionBehavior::Immediate)
            .map_err(storage_error)?;
        let (content, metadata): (String, Option<String>) = tx.query_row(
            "SELECT content,(SELECT data FROM turn_metadata WHERE message_id=messages.id) FROM messages WHERE id=?1 AND session_id=?2 AND role='assistant'",
            params![draft.message_id,session], |row| Ok((row.get(0)?, row.get(1)?)))
            .optional().map_err(storage_error)?.ok_or("Reply no longer exists. Close and refresh this chat.")?;
        let metadata: Option<TurnMetadata> = metadata
            .map(|v| serde_json::from_str(&v))
            .transpose()
            .map_err(storage_error)?;
        if content != draft.expected_content || metadata != draft.expected_metadata {
            return Err("Reply changed. Close and refresh before giving feedback.".into());
        }
        let prior: Option<String> = tx
            .query_row(
                "SELECT data FROM task_feedback WHERE message_id=?1",
                [draft.message_id],
                |row| row.get(0),
            )
            .optional()
            .map_err(storage_error)?;
        let prior: Option<TaskFeedback> = prior
            .map(|v| serde_json::from_str(&v))
            .transpose()
            .map_err(storage_error)?;
        if prior.as_ref().map_or(0, |v| v.revision) != draft.revision {
            return Err("Feedback changed elsewhere. Close and reopen to review it.".into());
        }
        let feedback = TaskFeedback {
            revision: draft
                .revision
                .checked_add(1)
                .ok_or("Feedback revision limit reached.")?,
            updated_at: now(),
            outcome: draft.outcome,
            note: draft.note.clone(),
        };
        tx.execute("INSERT INTO task_feedback(message_id,data) VALUES(?1,?2) ON CONFLICT(message_id) DO UPDATE SET data=excluded.data", params![draft.message_id,serde_json::to_string(&feedback).map_err(storage_error)?]).map_err(storage_error)?;
        tx.commit().map_err(storage_error)?;
        Ok(feedback)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use dolores_core::{ExportFormat, SessionStore, TaskOutcome};
    fn draft(id: i64) -> FeedbackDraft {
        FeedbackDraft {
            message_id: id,
            expected_content: "model claims success".into(),
            expected_metadata: None,
            revision: 0,
            outcome: Some(TaskOutcome::NeedsWork),
            note: "Actual task failed 世界".into(),
        }
    }
    #[test]
    fn feedback_is_local_revisioned_exported_and_cascades_after_restart() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("data.db");
        let store = SqliteStore::open(&path).unwrap();
        store.create("task").unwrap();
        store
            .commit_turn("task", "request", "model claims success")
            .unwrap();
        let id = store.messages_page("task", None, false, 80).unwrap().items[1].id;
        let before = store.messages("task").unwrap();
        let mut d = draft(id);
        let first = store.save_task_feedback("task", &d).unwrap();
        assert_eq!(first.revision, 1);
        assert_eq!(store.messages("task").unwrap(), before);
        assert!(store.save_task_feedback("task", &d).is_err());
        d.revision = 1;
        d.outcome = Some(TaskOutcome::Worked);
        let second = store.save_task_feedback("task", &d).unwrap();
        drop(store);
        let store = SqliteStore::open(&path).unwrap();
        assert_eq!(
            store.messages_page("task", None, false, 80).unwrap().items[1]
                .feedback
                .as_ref(),
            Some(&second)
        );
        for format in [ExportFormat::Json, ExportFormat::Markdown] {
            let mut out = vec![];
            store.export_conversation("task", format, &mut out).unwrap();
            assert!(String::from_utf8(out)
                .unwrap()
                .contains("Actual task failed 世界"));
        }
        d.revision = 2;
        d.outcome = None;
        d.note.clear();
        assert_eq!(store.save_task_feedback("task", &d).unwrap().revision, 3);
        assert!(store.save_task_feedback("task", &d).is_err());
        store.delete("task").unwrap();
        assert_eq!(
            store
                .lock()
                .unwrap()
                .query_row::<i64, _, _>("SELECT count(*) FROM task_feedback", [], |r| r.get(0))
                .unwrap(),
            0
        );
    }
    #[test]
    fn wrong_source_scope_invalid_notes_and_failed_writes_preserve_feedback() {
        let store = SqliteStore::open(std::path::Path::new(":memory:")).unwrap();
        store.create("task").unwrap();
        store.create("other").unwrap();
        store
            .commit_turn("task", "request", "model claims success")
            .unwrap();
        let items = store.messages_page("task", None, false, 80).unwrap().items;
        let d = draft(items[1].id);
        assert!(store.save_task_feedback("other", &d).is_err());
        let mut bad = d.clone();
        bad.message_id = items[0].id;
        assert!(store.save_task_feedback("task", &bad).is_err());
        bad = d.clone();
        bad.expected_content = "stale".into();
        assert!(store.save_task_feedback("task", &bad).is_err());
        for note in [
            "x".repeat(2049),
            "bad\0note".into(),
            "sk-123456789012345678901234".into(),
        ] {
            bad = d.clone();
            bad.note = note;
            assert!(store.save_task_feedback("task", &bad).is_err());
        }
        let first = store.save_task_feedback("task", &d).unwrap();
        store.lock().unwrap().execute_batch("CREATE TRIGGER refuse_feedback BEFORE UPDATE ON task_feedback BEGIN SELECT RAISE(ABORT,'fixture'); END;").unwrap();
        bad = d.clone();
        bad.revision = 1;
        bad.outcome = Some(TaskOutcome::Worked);
        assert!(store.save_task_feedback("task", &bad).is_err());
        assert_eq!(
            store.messages_page("task", None, false, 80).unwrap().items[1]
                .feedback
                .as_ref(),
            Some(&first)
        );
    }
}
