use crate::{storage_error, SqliteStore};
use dolores_core::{RunEvent, RunSnapshot, RunState, MAX_RUN_EVENTS, MAX_RUN_EVENT_BYTES};
use rusqlite::{params, OptionalExtension};

impl SqliteStore {
    pub(crate) fn insert_run(&self, run: &RunSnapshot) -> Result<(), String> {
        run.settings.validate()?;
        if run.state != RunState::Prepared
            || run.sequence != 0
            || run.id.len() != 36
            || run.input.len() > dolores_core::MAX_INPUT_BYTES
            || run.tools.len() > 9
        {
            return Err("Run snapshot is invalid.".into());
        }
        let mut conn = self.lock()?;
        let tx = conn
            .transaction_with_behavior(rusqlite::TransactionBehavior::Immediate)
            .map_err(storage_error)?;
        let count: u32 = tx
            .query_row(
                "SELECT count(*) FROM runs WHERE thread=?1",
                [&run.thread],
                |r| r.get(0),
            )
            .map_err(storage_error)?;
        if count >= 100 {
            return Err("This chat has 100 retained runs. Start a new chat to continue without deleting evidence.".into());
        }
        tx.execute(
            "INSERT INTO runs(id,thread,data) VALUES(?1,?2,?3)",
            params![
                run.id,
                run.thread,
                serde_json::to_string(run).map_err(storage_error)?
            ],
        )
        .map_err(storage_error)?;
        tx.commit().map_err(storage_error)
    }
    pub(crate) fn append_event(
        &self,
        id: &str,
        expected: u32,
        state: Option<RunState>,
        kind: &str,
        data: &serde_json::Value,
    ) -> Result<u32, String> {
        if kind.is_empty()
            || kind.len() > 64
            || serde_json::to_vec(data).map_err(storage_error)?.len() > MAX_RUN_EVENT_BYTES
        {
            return Err("Run evidence exceeds its bounded record limit.".into());
        }
        let mut conn = self.lock()?;
        let tx = conn
            .transaction_with_behavior(rusqlite::TransactionBehavior::Immediate)
            .map_err(storage_error)?;
        let raw: String = tx
            .query_row("SELECT data FROM runs WHERE id=?1", [id], |r| r.get(0))
            .map_err(storage_error)?;
        let mut run: RunSnapshot = serde_json::from_str(&raw).map_err(storage_error)?;
        if run.sequence != expected || run.state.terminal() {
            return Err("Run evidence changed or is already complete.".into());
        }
        if state
            .as_ref()
            .is_some_and(|next| !run.state.can_transition(next))
        {
            return Err("Run state transition is invalid.".into());
        }
        // A terminal marker has a reserved slot, so exhaustion remains inspectable.
        if expected >= MAX_RUN_EVENTS && !state.as_ref().is_some_and(RunState::terminal) {
            return Err(
                "Run evidence allowance exhausted. Inspect existing progress before continuing."
                    .into(),
            );
        }
        run.sequence += 1;
        if let Some(state) = state {
            run.state = state;
        }
        let event = RunEvent {
            sequence: run.sequence,
            kind: kind.into(),
            state: run.state.clone(),
            data: data.clone(),
        };
        tx.execute(
            "INSERT INTO run_events(run_id,sequence,data) VALUES(?1,?2,?3)",
            params![
                id,
                run.sequence,
                serde_json::to_string(&event).map_err(storage_error)?
            ],
        )
        .map_err(storage_error)?;
        tx.execute(
            "UPDATE runs SET data=?2 WHERE id=?1",
            params![id, serde_json::to_string(&run).map_err(storage_error)?],
        )
        .map_err(storage_error)?;
        tx.commit().map_err(storage_error)?;
        Ok(run.sequence)
    }
    pub(crate) fn read_runs(&self, thread: &str) -> Result<Vec<RunSnapshot>, String> {
        let conn = self.lock()?;
        let mut stmt = conn
            .prepare("SELECT data FROM runs WHERE thread=?1 ORDER BY rowid DESC LIMIT 20")
            .map_err(storage_error)?;
        let rows = stmt
            .query_map([thread], |r| r.get::<_, String>(0))
            .map_err(storage_error)?;
        rows.map(|r| serde_json::from_str(&r.map_err(storage_error)?).map_err(storage_error))
            .collect()
    }
    pub(crate) fn read_run_events(&self, thread: &str, id: &str) -> Result<Vec<RunEvent>, String> {
        let conn = self.lock()?;
        let present: Option<String> = conn
            .query_row(
                "SELECT id FROM runs WHERE id=?1 AND thread=?2",
                params![id, thread],
                |r| r.get(0),
            )
            .optional()
            .map_err(storage_error)?;
        if present.is_none() {
            return Err("Run belongs to another chat or is unavailable.".into());
        }
        let mut stmt = conn
            .prepare("SELECT data FROM run_events WHERE run_id=?1 ORDER BY sequence LIMIT 65")
            .map_err(storage_error)?;
        let rows = stmt
            .query_map([id], |r| r.get::<_, String>(0))
            .map_err(storage_error)?;
        rows.map(|r| serde_json::from_str(&r.map_err(storage_error)?).map_err(storage_error))
            .collect()
    }
    pub(crate) fn mark_interrupted(&self) -> Result<(), String> {
        let mut conn = self.lock()?;
        let tx = conn
            .transaction_with_behavior(rusqlite::TransactionBehavior::Immediate)
            .map_err(storage_error)?;
        let rows: Vec<(String, String)> = {
            let mut stmt=tx.prepare("SELECT id,data FROM runs WHERE json_extract(data,'$.state') IN ('prepared','running','waitingForApproval')").map_err(storage_error)?;
            let values = stmt
                .query_map([], |r| Ok((r.get(0)?, r.get(1)?)))
                .map_err(storage_error)?
                .collect::<Result<Vec<_>, _>>()
                .map_err(storage_error)?;
            values
        };
        for (id, raw) in rows {
            let mut run: RunSnapshot = serde_json::from_str(&raw).map_err(storage_error)?;
            run.state = RunState::Interrupted;
            // Reserved restart marker also works when the normal event allowance was exhausted.
            run.sequence += 1;
            let event = RunEvent {
                sequence: run.sequence,
                kind: "restart".into(),
                state: RunState::Interrupted,
                data: serde_json::json!({"message":"Execution was interrupted. Inspect effects and Changes before retrying; no operation was replayed."}),
            };
            tx.execute(
                "INSERT INTO run_events(run_id,sequence,data) VALUES(?1,?2,?3)",
                params![
                    id,
                    run.sequence,
                    serde_json::to_string(&event).map_err(storage_error)?
                ],
            )
            .map_err(storage_error)?;
            tx.execute(
                "UPDATE runs SET data=?2 WHERE id=?1",
                params![id, serde_json::to_string(&run).map_err(storage_error)?],
            )
            .map_err(storage_error)?;
        }
        tx.commit().map_err(storage_error)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use dolores_core::SessionStore;
    #[test]
    fn event_exhaustion_retains_terminal_marker_and_invalid_transitions_are_atomic() {
        let store = SqliteStore::open(std::path::Path::new(":memory:")).unwrap();
        store.create("chat").unwrap();
        let run = RunSnapshot {
            id: "11111111-1111-4111-8111-111111111111".into(),
            thread: "chat".into(),
            model: "fixture".into(),
            settings: Default::default(),
            input: "bounded work".into(),
            state: RunState::Prepared,
            sequence: 0,
            created_at: 0,
            build: "fixture".into(),
            tools: vec![],
            extensions: vec![],
        };
        store.begin_run(&run).unwrap();
        assert!(store
            .append_run_event(
                &run.id,
                0,
                Some(RunState::WaitingForApproval),
                "invalid",
                &serde_json::json!({})
            )
            .is_err());
        store
            .append_run_event(
                &run.id,
                0,
                Some(RunState::Running),
                "started",
                &serde_json::json!({}),
            )
            .unwrap();
        for n in 1..MAX_RUN_EVENTS {
            store
                .append_run_event(&run.id, n, None, "evidence", &serde_json::json!({}))
                .unwrap();
        }
        assert!(store
            .append_run_event(
                &run.id,
                MAX_RUN_EVENTS,
                None,
                "overflow",
                &serde_json::json!({})
            )
            .is_err());
        store
            .append_run_event(
                &run.id,
                MAX_RUN_EVENTS,
                Some(RunState::Failed),
                "finished",
                &serde_json::json!({"message":"Inspect saved evidence before retrying"}),
            )
            .unwrap();
        assert_eq!(store.run_events("chat", &run.id).unwrap().len(), 65);
        store.delete("chat").unwrap();
        assert!(store.run_events("chat", &run.id).is_err());
    }
    #[test]
    fn run_events_are_atomic_scoped_and_restart_never_replays_pending_effects() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("state.db");
        let store = SqliteStore::open(&path).unwrap();
        store.create("thread").unwrap();
        store.create("other").unwrap();
        let run = RunSnapshot {
            id: "11111111-1111-4111-8111-111111111111".into(),
            thread: "thread".into(),
            model: "fixture".into(),
            settings: Default::default(),
            input: "read a file".into(),
            state: RunState::Prepared,
            sequence: 0,
            created_at: 0,
            build: "fixture".into(),
            tools: vec!["read_text_file".into()],
            extensions: vec![],
        };
        store.begin_run(&run).unwrap();
        store
            .append_run_event(
                &run.id,
                0,
                Some(RunState::Running),
                "started",
                &serde_json::json!({}),
            )
            .unwrap();
        store
            .append_run_event(
                &run.id,
                1,
                None,
                "toolIntent",
                &serde_json::json!({"callId":"one","name":"edit_text_file"}),
            )
            .unwrap();
        assert!(store
            .append_run_event(&run.id, 1, None, "stale", &serde_json::json!({}))
            .is_err());
        assert!(store.run_events("other", &run.id).is_err());
        drop(store);
        let reopened = SqliteStore::open(&path).unwrap();
        reopened.interrupt_runs().unwrap();
        let saved = reopened.runs("thread").unwrap();
        assert_eq!(saved[0].state, RunState::Interrupted);
        let events = reopened.run_events("thread", &run.id).unwrap();
        assert_eq!(events.len(), 3);
        assert_eq!(events[1].kind, "toolIntent");
        assert_eq!(events[2].kind, "restart");
        reopened.interrupt_runs().unwrap();
        assert_eq!(reopened.run_events("thread", &run.id).unwrap().len(), 3);
        assert!(reopened
            .append_run_event(&run.id, 3, None, "replay", &serde_json::json!({}))
            .is_err());
        assert!(reopened
            .messages_page("thread", None, false, 80)
            .unwrap()
            .items
            .is_empty());
    }
}
