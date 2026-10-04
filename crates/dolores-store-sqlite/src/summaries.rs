use super::{now, storage_error, SqliteStore};
use dolores_core::{
    Message, Role, SessionSummary, SummaryBatch, SummaryMessage, SummarySource, HISTORY_LIMIT,
};
use rusqlite::{params, Connection, OptionalExtension};

const STALE: &str = "Summary or source messages changed. Refresh and review again.";
fn read(conn: &Connection, session: &str) -> Result<Option<SessionSummary>, String> {
    let exists: bool = conn
        .query_row(
            "SELECT EXISTS(SELECT 1 FROM sessions WHERE id=?1)",
            [session],
            |r| r.get(0),
        )
        .map_err(storage_error)?;
    if !exists {
        return Err("Conversation no longer exists.".into());
    }
    let data: Option<String> = conn
        .query_row(
            "SELECT data FROM session_summaries WHERE session_id=?1",
            [session],
            |r| r.get(0),
        )
        .optional()
        .map_err(storage_error)?;
    data.map(|data| {
        let summary: SessionSummary = serde_json::from_str(&data).map_err(storage_error)?;
        summary.validate()?;
        Ok(summary)
    })
    .transpose()
}
fn batch(
    conn: &Connection,
    session: &str,
    previous: Option<SessionSummary>,
    through: i64,
) -> Result<SummaryBatch, String> {
    let after = previous
        .as_ref()
        .map_or(0, |s| s.provenance.covered_through);
    let mut statement=conn.prepare("SELECT id,role,content FROM messages WHERE session_id=?1 AND id>?2 AND id<=?3 ORDER BY id LIMIT 42").map_err(storage_error)?;
    let mut rows = statement
        .query(params![session, after, through])
        .map_err(storage_error)?;
    let mut messages = Vec::new();
    let mut bytes = 0;
    let mut has_more = false;
    while let Some(user) = rows.next().map_err(storage_error)? {
        let user_role: String = user.get(1).map_err(storage_error)?;
        let user = SummaryMessage {
            id: user.get(0).map_err(storage_error)?,
            role: Role::User,
            content: user.get(2).map_err(storage_error)?,
        };
        // Validate stored roles separately, rather than inferring them from IDs.
        let assistant = rows.next().map_err(storage_error)?.ok_or(STALE)?;
        let assistant_role: String = assistant.get(1).map_err(storage_error)?;
        if user_role != "user" || assistant_role != "assistant" {
            return Err(STALE.into());
        }
        let assistant = SummaryMessage {
            id: assistant.get(0).map_err(storage_error)?,
            role: Role::Assistant,
            content: assistant.get(2).map_err(storage_error)?,
        };
        let next_bytes = bytes + user.content.len() + assistant.content.len();
        if messages.len() == 40 || next_bytes > dolores_core::MAX_SUMMARY_SOURCE_BYTES {
            has_more = true;
            break;
        }
        bytes = next_bytes;
        messages.extend([user, assistant]);
    }
    if messages.is_empty() && has_more {
        return Err("The next conversation turn exceeds the 144-KiB summary source limit.".into());
    }
    Ok(SummaryBatch {
        previous,
        messages,
        has_more,
    })
}
fn write(conn: &Connection, session: &str, value: &SessionSummary) -> Result<(), String> {
    value.validate()?;
    conn.execute("INSERT INTO session_summaries(session_id,data) VALUES(?1,?2) ON CONFLICT(session_id) DO UPDATE SET data=excluded.data",params![session,serde_json::to_string(value).map_err(storage_error)?]).map_err(storage_error)?;
    Ok(())
}
impl SqliteStore {
    pub(super) fn read_summary(&self, session: &str) -> Result<Option<SessionSummary>, String> {
        let conn = self.lock()?;
        read(&conn, session)
    }
    pub(super) fn summary_batch(&self, session: &str) -> Result<SummaryBatch, String> {
        let mut conn = self.lock()?;
        let tx = conn.transaction().map_err(storage_error)?;
        let previous = read(&tx, session)?;
        batch(&tx, session, previous, i64::MAX)
    }
    pub(super) fn write_summary(
        &self,
        session: &str,
        review: &SummaryBatch,
        text: &str,
        model: &str,
    ) -> Result<SessionSummary, String> {
        review.validate()?;
        dolores_core::validate_summary(text)?;
        let mut conn = self.lock()?;
        let tx = conn.transaction().map_err(storage_error)?;
        let previous = read(&tx, session)?;
        if previous != review.previous {
            return Err(STALE.into());
        }
        let through = review.messages.last().unwrap().id;
        let current = batch(&tx, session, previous.clone(), through)?;
        if current.messages != review.messages {
            return Err(STALE.into());
        }
        let provenance = SummarySource {
            revision: previous
                .as_ref()
                .map_or(Some(1), |s| s.provenance.revision.checked_add(1))
                .ok_or(STALE)?,
            covered_through: through,
            covered_turns: previous
                .as_ref()
                .map_or(0, |s| s.provenance.covered_turns)
                .checked_add((review.messages.len() / 2) as u64)
                .ok_or(STALE)?,
            model: model.into(),
            updated_at: now(),
        };
        let value = SessionSummary {
            text: text.into(),
            provenance,
        };
        write(&tx, session, &value)?;
        tx.commit().map_err(storage_error)?;
        Ok(value)
    }
    pub(super) fn correct_summary(
        &self,
        session: &str,
        revision: u32,
        text: &str,
    ) -> Result<SessionSummary, String> {
        dolores_core::validate_summary(text)?;
        let mut conn = self.lock()?;
        let tx = conn.transaction().map_err(storage_error)?;
        let mut value = read(&tx, session)?.ok_or(STALE)?;
        if value.provenance.revision != revision {
            return Err(STALE.into());
        }
        value.provenance.revision = revision.checked_add(1).ok_or(STALE)?;
        value.provenance.updated_at = now();
        value.text = text.into();
        write(&tx, session, &value)?;
        tx.commit().map_err(storage_error)?;
        Ok(value)
    }
    pub(super) fn remove_summary(&self, session: &str, revision: u32) -> Result<(), String> {
        let mut conn = self.lock()?;
        let tx = conn.transaction().map_err(storage_error)?;
        if read(&tx, session)?.ok_or(STALE)?.provenance.revision != revision {
            return Err(STALE.into());
        }
        tx.execute(
            "DELETE FROM session_summaries WHERE session_id=?1",
            [session],
        )
        .map_err(storage_error)?;
        tx.commit().map_err(storage_error)?;
        Ok(())
    }
    pub(super) fn summary_history(
        &self,
        session: &str,
    ) -> Result<dolores_core::SummaryHistory, String> {
        let mut conn = self.lock()?;
        let tx = conn.transaction().map_err(storage_error)?;
        let summary = read(&tx, session)?;
        let through = summary.as_ref().map_or(0, |s| s.provenance.covered_through);
        let count: u64 = tx
            .query_row(
                "SELECT COUNT(*) FROM messages WHERE session_id=?1",
                [session],
                |r| r.get(0),
            )
            .map_err(storage_error)?;
        if !count.is_multiple_of(2) {
            return Err(STALE.into());
        }
        let mut query=tx.prepare("SELECT role,content FROM (SELECT id,role,content FROM messages WHERE session_id=?1 AND id>?2 ORDER BY id DESC LIMIT ?3) ORDER BY id").map_err(storage_error)?;
        let messages = query
            .query_map(params![session, through, HISTORY_LIMIT], |r| {
                let role: String = r.get(0)?;
                Ok(Message {
                    role: match role.as_str() {
                        "user" => Role::User,
                        "assistant" => Role::Assistant,
                        _ => return Err(rusqlite::Error::InvalidQuery),
                    },
                    content: r.get(1)?,
                })
            })
            .map_err(storage_error)?
            .collect::<Result<Vec<_>, _>>()
            .map_err(storage_error)?;
        Ok((messages, Some(count / 2), summary))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use dolores_core::SessionStore;
    #[test]
    fn contiguous_summary_extension_correction_failure_restart_and_delete_preserve_history() {
        let temp = tempfile::tempdir().unwrap();
        let db = temp.path().join("state.db");
        let store = SqliteStore::open(&db).unwrap();
        store.create("chat").unwrap();
        store.create("other").unwrap();
        for n in 0..22 {
            store
                .commit_turn("chat", &format!("Goal {n}"), &format!("Outcome {n}"))
                .unwrap();
        }
        let reviewed = store.review_summary_batch("chat").unwrap();
        assert_eq!(reviewed.messages.len(), 40);
        assert!(reviewed.has_more);
        assert!(store
            .save_session_summary("other", &reviewed, "Goals 0–19.", "fixture")
            .is_err());
        let saved = store
            .save_session_summary("chat", &reviewed, "Goals 0–19.", "fixture")
            .unwrap();
        assert_eq!(saved.provenance.covered_turns, 20);
        assert!(store
            .save_session_summary("chat", &reviewed, "Replay", "fixture")
            .is_err());
        let (history, count, _) = store.summary_context_history("chat").unwrap();
        assert_eq!(history.len(), 4);
        assert_eq!(history[0].content, "Goal 20");
        assert_eq!(count, Some(22));
        let next = store.review_summary_batch("chat").unwrap();
        assert_eq!(next.messages.len(), 4);
        assert!(!next.has_more);
        let connection = Connection::open(&db).unwrap();
        connection.execute_batch("CREATE TRIGGER fail_summary BEFORE UPDATE ON session_summaries BEGIN SELECT RAISE(ABORT,'fixture'); END;").unwrap();
        assert!(store
            .save_session_summary("chat", &next, "All goals.", "fixture")
            .is_err());
        assert_eq!(store.session_summary("chat").unwrap().unwrap(), saved);
        connection
            .execute_batch("DROP TRIGGER fail_summary")
            .unwrap();
        let corrected = store
            .correct_session_summary("chat", 1, "Corrected old goals.")
            .unwrap();
        assert_eq!(
            corrected.provenance.covered_through,
            saved.provenance.covered_through
        );
        assert!(store
            .save_session_summary("chat", &next, "Stale prior summary.", "fixture")
            .is_err());
        let next = store.review_summary_batch("chat").unwrap();
        connection
            .execute(
                "UPDATE messages SET content='changed' WHERE id=?1",
                [next.messages[0].id],
            )
            .unwrap();
        assert!(store
            .save_session_summary("chat", &next, "Stale source.", "fixture")
            .is_err());
        let next = store.review_summary_batch("chat").unwrap();
        let saved = store
            .save_session_summary("chat", &next, "All reviewed goals.", "fixture")
            .unwrap();
        assert_eq!(saved.provenance.covered_turns, 22);
        assert!(store.summary_context_history("chat").unwrap().0.is_empty());
        drop(store);
        let store = SqliteStore::open(&db).unwrap();
        assert_eq!(store.session_summary("chat").unwrap(), Some(saved.clone()));
        assert!(store.session_summary("other").unwrap().is_none());
        assert!(store.delete_session_summary("chat", 1).is_err());
        store
            .delete_session_summary("chat", saved.provenance.revision)
            .unwrap();
        assert_eq!(
            store
                .messages_page("chat", None, false, 80)
                .unwrap()
                .items
                .len(),
            44
        );
        let batch = store.review_summary_batch("chat").unwrap();
        store
            .save_session_summary("chat", &batch, "Restarted summary.", "fixture")
            .unwrap();
        store.delete("chat").unwrap();
        assert!(store.session_summary("chat").is_err());
        let version: i64 = connection
            .query_row("PRAGMA user_version", [], |r| r.get(0))
            .unwrap();
        assert_eq!(version, 21);
    }
}
