use crate::{storage_error, SqliteStore};
use dolores_core::{
    RunEvent, RunEventPage, RunSegment, RunSnapshot, RunState, MAX_RUN_EVENT_BYTES,
};
use rusqlite::{params, OptionalExtension};
use serde_json::Value;

impl SqliteStore {
    pub(crate) fn read_run_events_page(
        &self,
        thread: &str,
        id: &str,
        after: u32,
    ) -> Result<RunEventPage, String> {
        let conn = self.lock()?;
        let raw: String = conn
            .query_row(
                "SELECT data FROM runs WHERE id=?1 AND thread=?2",
                params![id, thread],
                |r| r.get(0),
            )
            .optional()
            .map_err(storage_error)?
            .ok_or("Run belongs to another chat or is unavailable.")?;
        let run: RunSnapshot = serde_json::from_str(&raw).map_err(storage_error)?;
        let mut stmt = conn.prepare("SELECT data FROM run_events WHERE run_id=?1 AND sequence>?2 ORDER BY sequence LIMIT 129").map_err(storage_error)?;
        let mut items = Vec::new();
        let mut bytes = 0;
        let mut more = false;
        for row in stmt
            .query_map(params![id, after], |r| r.get::<_, String>(0))
            .map_err(storage_error)?
        {
            let raw = row.map_err(storage_error)?;
            if !items.is_empty() && (items.len() >= 128 || bytes + raw.len() > 256 * 1024) {
                more = true;
                break;
            }
            bytes += raw.len();
            items.push(serde_json::from_str::<RunEvent>(&raw).map_err(storage_error)?);
        }
        let next = if more {
            items.last().map(|e| e.sequence)
        } else {
            None
        };
        Ok(RunEventPage {
            items,
            next,
            total: run.sequence,
        })
    }

    /// Complete evidence and the next safe context commit in one transaction.
    /// This seals storage only; restart never dispatches the saved context.
    pub(crate) fn write_run_segment(
        &self,
        id: &str,
        expected: u32,
        records: &[Value],
        checkpoint: &Value,
    ) -> Result<(u32, RunSegment), String> {
        let checkpoint = serde_json::to_string(checkpoint).map_err(storage_error)?;
        if checkpoint.len() > MAX_RUN_EVENT_BYTES || records.len() > 512 {
            return Err(
                "Task checkpoint exceeds its record allowance. Saved progress remains.".into(),
            );
        }
        let mut encoded = Vec::with_capacity(records.len());
        let mut bytes = 0;
        for record in records {
            let data = serde_json::to_string(record).map_err(storage_error)?;
            bytes += data.len();
            if data.len() > MAX_RUN_EVENT_BYTES || bytes > 8 * 1024 * 1024 {
                return Err(
                    "Task segment evidence exceeds its record allowance. Saved progress remains."
                        .into(),
                );
            }
            encoded.push(data);
        }
        let mut conn = self.lock()?;
        let tx = conn
            .transaction_with_behavior(rusqlite::TransactionBehavior::Immediate)
            .map_err(storage_error)?;
        let raw: String = tx
            .query_row("SELECT data FROM runs WHERE id=?1", [id], |r| r.get(0))
            .map_err(storage_error)?;
        let mut run: RunSnapshot = serde_json::from_str(&raw).map_err(storage_error)?;
        if run.sequence != expected || run.state != RunState::Running {
            return Err("Task checkpoint owner changed or is no longer running.".into());
        }
        let ordinal: u32 = tx
            .query_row(
                "SELECT COALESCE(MAX(ordinal),0)+1 FROM run_segments WHERE run_id=?1",
                [id],
                |r| r.get(0),
            )
            .map_err(storage_error)?;
        for data in encoded {
            run.sequence = run
                .sequence
                .checked_add(1)
                .ok_or("Task sequence exhausted.")?;
            let event = RunEvent {
                sequence: run.sequence,
                kind: "agentEvidence".into(),
                state: run.state.clone(),
                data: serde_json::from_str(&data).map_err(storage_error)?,
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
        }
        run.sequence = run
            .sequence
            .checked_add(1)
            .ok_or("Task sequence exhausted.")?;
        let marker = RunEvent {
            sequence: run.sequence,
            kind: "taskCheckpoint".into(),
            state: run.state.clone(),
            data: serde_json::json!({"ordinal":ordinal,"checkpoint":serde_json::from_str::<Value>(&checkpoint).map_err(storage_error)?}),
        };
        tx.execute(
            "INSERT INTO run_events(run_id,sequence,data) VALUES(?1,?2,?3)",
            params![
                id,
                run.sequence,
                serde_json::to_string(&marker).map_err(storage_error)?
            ],
        )
        .map_err(storage_error)?;
        let segment = RunSegment {
            ordinal,
            through_sequence: run.sequence,
            checkpoint: serde_json::from_str(&checkpoint).map_err(storage_error)?,
        };
        tx.execute("INSERT INTO run_segments(run_id,ordinal,through_sequence,checkpoint) VALUES(?1,?2,?3,?4)", params![id,ordinal,run.sequence,checkpoint]).map_err(storage_error)?;
        tx.execute(
            "UPDATE runs SET data=?2 WHERE id=?1",
            params![id, serde_json::to_string(&run).map_err(storage_error)?],
        )
        .map_err(storage_error)?;
        tx.commit().map_err(storage_error)?;
        Ok((run.sequence, segment))
    }
}
