use super::{storage_error, SqliteStore};
use dolores_core::{ComparisonStatus, ComparisonSummary, ContextComparison, HistoryPage};
use rusqlite::{params, OptionalExtension};

impl SqliteStore {
    pub(super) fn insert_comparison(
        &self,
        run: &ContextComparison,
    ) -> Result<ContextComparison, String> {
        run.validate()?;
        if run.id != 0
            || run.revision != 0
            || !run.results.is_empty()
            || run.status != ComparisonStatus::Running
        {
            return Err("Start with a fresh comparison.".into());
        }
        let mut db = self.lock()?;
        let tx = db
            .transaction_with_behavior(rusqlite::TransactionBehavior::Immediate)
            .map_err(storage_error)?;
        let count: i64 = tx
            .query_row(
                "SELECT count(*) FROM context_comparisons WHERE session_id=?1",
                [&run.session],
                |r| r.get(0),
            )
            .map_err(storage_error)?;
        if count >= 30 {
            return Err("This chat has 30 comparisons. Export and delete an older comparison before starting another.".into());
        }
        let mut saved = run.clone();
        tx.execute(
            "INSERT INTO context_comparisons(session_id,data) VALUES(?1,?2)",
            params![run.session, "{}"],
        )
        .map_err(storage_error)?;
        saved.id = tx.last_insert_rowid();
        saved.revision = 1;
        tx.execute(
            "UPDATE context_comparisons SET data=?1 WHERE id=?2",
            params![
                serde_json::to_string(&saved).map_err(storage_error)?,
                saved.id
            ],
        )
        .map_err(storage_error)?;
        tx.commit().map_err(storage_error)?;
        Ok(saved)
    }
    pub(super) fn advance_comparison(
        &self,
        run: &ContextComparison,
    ) -> Result<ContextComparison, String> {
        run.validate()?;
        let mut db = self.lock()?;
        let tx = db
            .transaction_with_behavior(rusqlite::TransactionBehavior::Immediate)
            .map_err(storage_error)?;
        let text: String = tx
            .query_row(
                "SELECT data FROM context_comparisons WHERE id=?1 AND session_id=?2",
                params![run.id, run.session],
                |r| r.get(0),
            )
            .optional()
            .map_err(storage_error)?
            .ok_or("Comparison no longer exists.")?;
        let prior: ContextComparison = serde_json::from_str(&text).map_err(storage_error)?;
        if prior.revision != run.revision
            || prior.status != ComparisonStatus::Running
            || !prior.same_request(run)
            || !run.results.starts_with(&prior.results)
            || run.results.len() > prior.results.len() + 1
        {
            return Err("Comparison changed. Completed evidence was not overwritten; start a new comparison.".into());
        }
        let mut saved = run.clone();
        saved.revision = saved
            .revision
            .checked_add(1)
            .ok_or("Comparison revision limit reached.")?;
        tx.execute(
            "UPDATE context_comparisons SET data=?1 WHERE id=?2",
            params![
                serde_json::to_string(&saved).map_err(storage_error)?,
                saved.id
            ],
        )
        .map_err(storage_error)?;
        tx.commit().map_err(storage_error)?;
        Ok(saved)
    }
    pub(super) fn read_comparison(
        &self,
        session: &str,
        id: i64,
    ) -> Result<ContextComparison, String> {
        let text: String = self
            .lock()?
            .query_row(
                "SELECT data FROM context_comparisons WHERE id=?1 AND session_id=?2",
                params![id, session],
                |r| r.get(0),
            )
            .optional()
            .map_err(storage_error)?
            .ok_or("Comparison no longer exists.")?;
        let run: ContextComparison = serde_json::from_str(&text).map_err(storage_error)?;
        run.validate()?;
        Ok(run)
    }
    pub(super) fn read_comparisons(
        &self,
        session: &str,
        cursor: Option<i64>,
    ) -> Result<HistoryPage<ComparisonSummary>, String> {
        if cursor.is_some_and(|c| c <= 0) {
            return Err("Invalid comparison cursor.".into());
        }
        let db = self.lock()?;
        let mut statement = db.prepare("SELECT data FROM context_comparisons WHERE session_id=?1 AND (?2 IS NULL OR id<?2) ORDER BY id DESC LIMIT 11").map_err(storage_error)?;
        let texts = statement
            .query_map(params![session, cursor], |r| r.get::<_, String>(0))
            .map_err(storage_error)?
            .collect::<Result<Vec<_>, _>>()
            .map_err(storage_error)?;
        let has_older = texts.len() > 10;
        let items = texts
            .into_iter()
            .take(10)
            .map(|text| {
                let r: ContextComparison = serde_json::from_str(&text).map_err(storage_error)?;
                r.validate()?;
                Ok(r.summary())
            })
            .collect::<Result<Vec<_>, String>>()?;
        Ok(HistoryPage {
            items,
            has_older,
            has_newer: cursor.is_some(),
        })
    }
    pub(super) fn remove_comparison(
        &self,
        session: &str,
        id: i64,
        revision: u32,
    ) -> Result<(), String> {
        let db = self.lock()?;
        let changed = db.execute("DELETE FROM context_comparisons WHERE id=?1 AND session_id=?2 AND json_extract(data,'$.revision')=?3", params![id,session,revision]).map_err(storage_error)?;
        if changed != 1 {
            return Err("Comparison changed. Refresh before deleting.".into());
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use dolores_core::{
        ComparisonDraft, ComparisonOutcome, ComparisonResult, RequestSettings, SessionStore,
    };
    fn run() -> ContextComparison {
        let draft: ComparisonDraft=serde_json::from_value(serde_json::json!({"title":"Frozen test","baseline":{"label":"Before","text":"","source":null},"candidate":{"label":"After","text":"Include PASS","source":null},"trials":[{"prompt":"Respond briefly","required":["PASS"],"forbidden":["FAIL"]}]})).unwrap();
        let (baseline_messages, candidate_messages) = draft.prompts().unwrap();
        ContextComparison {
            id: 0,
            session: "task".into(),
            revision: 0,
            created_at: 42,
            model: "fixture".into(),
            settings: RequestSettings {
                max_output_tokens: 512,
                timeout_seconds: 10,
                ..Default::default()
            },
            context_window_tokens: None,
            draft,
            baseline_messages,
            candidate_messages,
            results: vec![],
            status: ComparisonStatus::Running,
        }
    }
    fn result(text: &str) -> ComparisonResult {
        ComparisonResult {
            output: text.into(),
            usage: None,
            elapsed_ms: 42,
            outcome: ComparisonOutcome::Completed,
            detail: None,
        }
    }
    #[test]
    fn frozen_progress_restarts_exports_and_refuses_rewriting_or_stale_deletion() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("comparison.db");
        let store = SqliteStore::open(&path).unwrap();
        store.create("task").unwrap();
        store.commit_turn("task", "original", "reply").unwrap();
        let original = store.messages("task").unwrap();
        let mut saved = store.create_comparison(&run()).unwrap();
        saved.results.push(result("ordinary"));
        saved = store.update_comparison(&saved).unwrap();
        let mut tampered = saved.clone();
        tampered.results[0].output = "changed".into();
        assert!(store.update_comparison(&tampered).is_err());
        tampered = saved.clone();
        tampered.model = "different".into();
        assert!(store.update_comparison(&tampered).is_err());
        drop(store);
        let store = SqliteStore::open(&path).unwrap();
        assert_eq!(store.comparison("task", saved.id).unwrap(), saved);
        assert!(!saved.summary().improved);
        assert!(store.comparison("wrong", saved.id).is_err());
        let stale = saved.clone();
        saved.results.push(result("PASS"));
        saved.status = ComparisonStatus::Completed;
        saved = store.update_comparison(&saved).unwrap();
        assert!(saved.summary().improved);
        assert!(store.update_comparison(&stale).is_err());
        for format in [
            dolores_core::ExportFormat::Json,
            dolores_core::ExportFormat::Markdown,
        ] {
            let mut out = vec![];
            store.export_conversation("task", format, &mut out).unwrap();
            assert!(String::from_utf8(out).unwrap().contains("Include PASS"));
        }
        assert_eq!(store.messages("task").unwrap(), original);
        assert!(store
            .delete_comparison("task", saved.id, stale.revision)
            .is_err());
        store
            .delete_comparison("task", saved.id, saved.revision)
            .unwrap();
        let new = store.create_comparison(&run()).unwrap();
        assert!(new.id > saved.id);
        store.delete("task").unwrap();
        assert!(store.comparison("task", new.id).is_err());
    }
    #[test]
    fn capacity_paging_and_failed_progress_keep_previous_evidence() {
        let store = SqliteStore::open(std::path::Path::new(":memory:")).unwrap();
        store.create("task").unwrap();
        store.create("other").unwrap();
        for _ in 0..30 {
            store.create_comparison(&run()).unwrap();
        }
        assert!(store.create_comparison(&run()).is_err());
        let mut other = run();
        other.session = "other".into();
        store.create_comparison(&other).unwrap();
        let first = store.comparisons_page("task", None).unwrap();
        assert_eq!(first.items.len(), 10);
        assert!(first.has_older);
        let next = store
            .comparisons_page("task", Some(first.items.last().unwrap().id))
            .unwrap();
        assert_eq!(next.items.len(), 10);
        assert!(first.items.last().unwrap().id > next.items[0].id);
        let id = first.items[0].id;
        let saved = store.comparison("task", id).unwrap();
        store.lock().unwrap().execute_batch("CREATE TRIGGER refuse_progress BEFORE UPDATE ON context_comparisons BEGIN SELECT RAISE(ABORT,'fixture'); END;").unwrap();
        let mut proposed = saved.clone();
        proposed.results.push(result("ordinary"));
        assert!(store.update_comparison(&proposed).is_err());
        assert_eq!(store.comparison("task", id).unwrap(), saved);
        store
            .lock()
            .unwrap()
            .execute_batch("DROP TRIGGER refuse_progress")
            .unwrap();
        assert_eq!(store.update_comparison(&proposed).unwrap().results.len(), 1);
    }
}
