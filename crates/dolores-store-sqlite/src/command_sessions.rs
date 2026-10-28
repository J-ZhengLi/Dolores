use super::*;
use dolores_core::{
    command_sessions::{
        OutputPage, ReceiptPage, SessionReceipt, SessionRecord, LIST_PAGE, OUTPUT_PAGE,
    },
    RunSnapshot,
};

fn run(conn: &Connection, task: &str, id: &str) -> Result<RunSnapshot, String> {
    let raw: Option<String> = conn
        .query_row(
            "SELECT data FROM runs WHERE id=?1 AND thread=?2",
            params![id, task],
            |r| r.get(0),
        )
        .optional()
        .map_err(storage_error)?;
    serde_json::from_str(&raw.ok_or("Command run belongs to another chat or is unavailable.")?)
        .map_err(storage_error)
}
pub(super) fn save(store: &SqliteStore, record: &SessionRecord) -> Result<(), String> {
    record.validate()?;
    let r = &record.receipt;
    let data = serde_json::to_string(r).map_err(storage_error)?;
    let mut conn = store.lock()?;
    let tx = conn
        .transaction_with_behavior(rusqlite::TransactionBehavior::Immediate)
        .map_err(storage_error)?;
    let owner = run(&tx, &r.task, &r.run)?;
    if owner.state.terminal() && r.live() {
        return Err("Command owner ended. Save only its final inspection receipt; no process can be reattached.".into());
    }
    let old: Option<(i64, String, Vec<u8>)> = tx
        .query_row(
            "SELECT revision,data,output FROM command_sessions WHERE run=?1 AND id=?2",
            params![r.run, r.id],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
        )
        .optional()
        .map_err(storage_error)?;
    if let Some((revision, old_data, output)) = old {
        if revision as u64 > record.revision {
            return Ok(());
        }
        if revision as u64 == record.revision {
            return if old_data == data && output == record.output {
                Ok(())
            } else {
                Err("Command evidence revision changed. Inspect the saved receipt before continuing.".into())
            };
        }
        let old: SessionReceipt = serde_json::from_str(&old_data).map_err(storage_error)?;
        if old.task != r.task
            || old.run != r.run
            || old.id != r.id
            || old.start_call != r.start_call
            || old.program != r.program
            || old.execution != r.execution
            || old.timeout_seconds != r.timeout_seconds
            || old.capture_bytes != r.capture_bytes
            || !record.output.starts_with(&output)
            || !old.live() && r.state != old.state
            || old.cleanup_complete && !r.cleanup_complete
            || old.truncated && !r.truncated
            || old.output_error && !r.output_error
            || old.lossy_utf8 && !r.lossy_utf8
            || r.input_revision < old.input_revision
            || r.input_bytes < old.input_bytes
        {
            return Err("Command evidence cannot replace its identity, discard output or reopen a finished session.".into());
        }
        tx.execute(
            "UPDATE command_sessions SET revision=?3,data=?4,output=?5 WHERE run=?1 AND id=?2",
            params![r.run, r.id, record.revision as i64, data, record.output],
        )
        .map_err(storage_error)?;
    } else {
        let count: i64 = tx
            .query_row(
                "SELECT count(*) FROM command_sessions WHERE run=?1",
                [&r.run],
                |r| r.get(0),
            )
            .map_err(storage_error)?;
        let limit = owner
            .effective_settings
            .as_ref()
            .map(|s| s.task)
            .unwrap_or_default()
            .tool_limit();
        if count >= limit as i64 {
            return Err("This run reached its retained command-receipt limit. Inspect existing work before starting a new run.".into());
        }
        tx.execute(
            "INSERT INTO command_sessions(run,id,revision,data,output) VALUES(?1,?2,?3,?4,?5)",
            params![r.run, r.id, record.revision as i64, data, record.output],
        )
        .map_err(storage_error)?;
    }
    tx.commit().map_err(storage_error)
}

pub(super) fn list(
    store: &SqliteStore,
    task: &str,
    owner: Option<&str>,
    before: Option<i64>,
) -> Result<ReceiptPage, String> {
    let conn = store.lock()?;
    let exists: bool = conn
        .query_row(
            "SELECT EXISTS(SELECT 1 FROM sessions WHERE id=?1)",
            [task],
            |r| r.get(0),
        )
        .map_err(storage_error)?;
    if !exists {
        return Err(
            "Chat unavailable. Retained command receipts cannot be shared across chats.".into(),
        );
    }
    if let Some(owner) = owner {
        run(&conn, task, owner)?;
    }
    let mut stmt=conn.prepare("SELECT c.rowid,c.data FROM command_sessions c JOIN runs r ON r.id=c.run WHERE r.thread=?1 AND (?2 IS NULL OR c.run=?2) AND (?3 IS NULL OR c.rowid<?3) ORDER BY c.rowid DESC LIMIT ?4").map_err(storage_error)?;
    let mut rows = stmt
        .query(params![task, owner, before, (LIST_PAGE + 1) as i64])
        .map_err(storage_error)?;
    let mut items = Vec::new();
    let mut last = None;
    let mut more = false;
    while let Some(row) = rows.next().map_err(storage_error)? {
        if items.len() == LIST_PAGE {
            more = true;
            break;
        }
        last = Some(row.get::<_, i64>(0).map_err(storage_error)?);
        let raw: String = row.get(1).map_err(storage_error)?;
        items.push(serde_json::from_str(&raw).map_err(storage_error)?);
    }
    Ok(ReceiptPage {
        items,
        next_cursor: if more { last } else { None },
    })
}
pub(super) fn output(
    store: &SqliteStore,
    task: &str,
    owner: &str,
    id: &str,
    offset: usize,
) -> Result<OutputPage, String> {
    let conn = store.lock()?;
    run(&conn, task, owner)?;
    if offset > dolores_core::command_sessions::MAX_OUTPUT {
        return Err("Output cursor exceeds the retained capture limit. Read from offset 0.".into());
    }
    let row:Option<(String,usize,Vec<u8>)>=conn.query_row("SELECT data,length(output),substr(output,?3,?4) FROM command_sessions WHERE run=?1 AND id=?2",params![owner,id,offset+1,OUTPUT_PAGE],|r|Ok((r.get(0)?,r.get(1)?,r.get(2)?))).optional().map_err(storage_error)?;
    let (raw, total, bytes) =
        row.ok_or("Command receipt unavailable. No process was reattached or repeated.")?;
    if offset > total {
        return Err(
            "Output cursor is beyond retained bytes. Read from offset 0; no command was repeated."
                .into(),
        );
    }
    Ok(OutputPage {
        receipt: serde_json::from_str(&raw).map_err(storage_error)?,
        offset,
        next_offset: offset + bytes.len(),
        has_more: offset + bytes.len() < total,
        output: String::from_utf8_lossy(&bytes).into_owned(),
        lossy_utf8: std::str::from_utf8(&bytes).is_err(),
    })
}
pub(super) fn recover(store: &SqliteStore) -> Result<(), String> {
    let mut conn = store.lock()?;
    let tx = conn
        .transaction_with_behavior(rusqlite::TransactionBehavior::Immediate)
        .map_err(storage_error)?;
    let rows: Vec<(String, String, i64, String)> = {
        let mut stmt=tx.prepare("SELECT run,id,revision,data FROM command_sessions WHERE json_extract(data,'$.state') IN ('starting','running','stopping')").map_err(storage_error)?;
        let values = stmt
            .query_map([], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?)))
            .map_err(storage_error)?
            .collect::<Result<_, _>>()
            .map_err(storage_error)?;
        values
    };
    for (run, id, revision, raw) in rows {
        let mut r: SessionReceipt = serde_json::from_str(&raw).map_err(storage_error)?;
        r.state = "lost".into();
        r.cleanup_complete = false;
        r.stdin_closing = true;
        r.stdin_closed = false;
        r.message=Some("Dolores restarted before a final command receipt. Inspect retained output, running processes and file effects before a new explicit command. Input may have occurred; nothing was reattached or repeated.".into());
        if let Some(input) = &mut r.last_input {
            if input.state == "pending" {
                input.state = "uncertain".into();
            }
        }
        r.validate()?;
        let next = revision
            .checked_add(1)
            .ok_or("Command receipt revision exhausted. Prior evidence remains.")?;
        tx.execute(
            "UPDATE command_sessions SET revision=?3,data=?4 WHERE run=?1 AND id=?2",
            params![
                run,
                id,
                next,
                serde_json::to_string(&r).map_err(storage_error)?
            ],
        )
        .map_err(storage_error)?;
    }
    tx.commit().map_err(storage_error)
}

#[cfg(test)]
mod tests {
    use super::*;
    const RUN: &str = "11111111-1111-4111-8111-111111111111";
    fn setup(store: &SqliteStore) {
        store.create("chat").unwrap();
        store.create("other").unwrap();
        store
            .begin_run(&RunSnapshot {
                id: RUN.into(),
                thread: "chat".into(),
                parent_run: None,
                segments: 1,
                model: "fixture".into(),
                settings: Default::default(),
                input: "public fixture".into(),
                state: dolores_core::RunState::Prepared,
                sequence: 0,
                created_at: 0,
                build: "fixture".into(),
                tools: vec!["command_session".into()],
                extensions: vec![],
                effective_settings: None,
            })
            .unwrap();
    }
    fn record(id: &str) -> SessionRecord {
        SessionRecord {
            revision: 0,
            output: vec![],
            receipt: SessionReceipt {
                id: id.into(),
                task: "chat".into(),
                run: RUN.into(),
                start_call: format!("start-{id}"),
                program: "node".into(),
                execution: "native; not sandboxed".into(),
                state: "starting".into(),
                exit_code: None,
                timeout_seconds: 30,
                capture_bytes: 16384,
                output_bytes: 0,
                truncated: false,
                local_log: None,
                output_retained: false,
                log_error: false,
                output_error: false,
                lossy_utf8: false,
                cleanup_complete: false,
                input_revision: 0,
                input_bytes: 0,
                stdin_closed: false,
                stdin_closing: false,
                last_input: None,
                message: None,
            },
        }
    }
    #[test]
    fn multi_hour_unicode_executable_receipt_survives_restart_without_reattachment() {
        let store = SqliteStore::open(Path::new(":memory:")).unwrap();
        setup(&store);
        let mut r = record("long-command");
        r.receipt.program = format!("C:/fixture/中文/{}/python.exe", "public-folder/".repeat(8));
        r.receipt.timeout_seconds = 7200;
        store.save_command_session(&r).unwrap();
        let saved = store.command_sessions("chat", Some(RUN), None).unwrap();
        assert_eq!(saved.items[0], r.receipt);
        store.recover_command_sessions().unwrap();
        let recovered = store.command_sessions("chat", Some(RUN), None).unwrap();
        assert_eq!(recovered.items[0].state, "lost");
        assert_eq!(recovered.items[0].timeout_seconds, 7200);
        assert!(!recovered.items[0].live());
        r.receipt.timeout_seconds = dolores_core::command_sessions::MAX_SESSION_SECONDS + 1;
        assert!(store.save_command_session(&r).is_err());
    }
    #[test]
    fn export_keeps_raw_command_output_and_ownership_even_without_a_completed_reply() {
        let store = SqliteStore::open(Path::new(":memory:")).unwrap();
        setup(&store);
        let mut r = record("export-one");
        r.output = b"````\n<script>\xff".to_vec();
        r.receipt.output_bytes = r.output.len();
        store.save_command_session(&r).unwrap();
        let mut bytes = vec![];
        assert_eq!(
            store
                .export_conversation("chat", dolores_core::ExportFormat::Json, &mut bytes)
                .unwrap(),
            0
        );
        let value: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(
            value["commandSessions"][0]["rawBytes"],
            serde_json::json!(r.output)
        );
        assert_eq!(value["commandSessions"][0]["receipt"]["id"], "export-one");
        bytes.clear();
        store
            .export_conversation("other", dolores_core::ExportFormat::Json, &mut bytes)
            .unwrap();
        let other: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(other["commandSessions"], serde_json::json!([]));
        bytes.clear();
        store
            .export_conversation("chat", dolores_core::ExportFormat::Markdown, &mut bytes)
            .unwrap();
        let text = String::from_utf8(bytes).unwrap();
        assert!(text.contains("Command session (untrusted output)"));
        assert!(text.contains("export-one") && text.contains("rawBytes"));
    }
    #[test]
    fn incremental_output_is_append_only_stale_saves_are_harmless_and_failures_recover() {
        let store = SqliteStore::open(Path::new(":memory:")).unwrap();
        setup(&store);
        let mut r = record("command-one");
        store.save_command_session(&r).unwrap();
        let initial = r.clone();
        r.revision = 1;
        r.receipt.state = "running".into();
        r.output = vec![b'a'; OUTPUT_PAGE + 1];
        r.output.push(255);
        r.receipt.output_bytes = r.output.len();
        store.save_command_session(&r).unwrap();
        store.save_command_session(&initial).unwrap();
        let page = store
            .command_session_output("chat", RUN, &r.receipt.id, 0)
            .unwrap();
        assert!(page.has_more);
        assert_eq!(page.next_offset, OUTPUT_PAGE);
        assert_eq!(page.output.len(), OUTPUT_PAGE);
        let page = store
            .command_session_output("chat", RUN, &r.receipt.id, page.next_offset)
            .unwrap();
        assert!(!page.has_more);
        assert!(page.lossy_utf8);
        assert_eq!(page.output, "a�");
        let mut conflicting = r.clone();
        conflicting.receipt.message = Some("changed same revision".into());
        assert!(store.save_command_session(&conflicting).is_err());
        let mut shrinking = r.clone();
        shrinking.revision += 1;
        shrinking.output.clear();
        shrinking.receipt.output_bytes = 0;
        assert!(store.save_command_session(&shrinking).is_err());
        r.revision = 2;
        r.receipt.state = "completed".into();
        r.receipt.exit_code = Some(0);
        r.receipt.cleanup_complete = true;
        store.lock().unwrap().execute_batch("CREATE TRIGGER reject_command BEFORE UPDATE ON command_sessions BEGIN SELECT RAISE(ABORT,'fixture save unavailable'); END;").unwrap();
        assert!(store.save_command_session(&r).is_err());
        assert_eq!(
            store
                .command_sessions("chat", Some(RUN), None)
                .unwrap()
                .items[0]
                .state,
            "running"
        );
        store
            .lock()
            .unwrap()
            .execute_batch("DROP TRIGGER reject_command")
            .unwrap();
        store.save_command_session(&r).unwrap();
        r.revision = 3;
        r.receipt.state = "running".into();
        r.receipt.cleanup_complete = false;
        assert!(store.save_command_session(&r).is_err());
        assert_eq!(
            store
                .command_sessions("chat", Some(RUN), None)
                .unwrap()
                .items[0]
                .state,
            "completed"
        );
        assert!(store
            .command_session_output("other", RUN, "command-one", 0)
            .is_err());
        assert!(store
            .command_session_output("chat", RUN, "command-one", 20000)
            .is_err());
    }
    #[test]
    fn restart_retains_output_marks_pending_input_uncertain_and_never_reopens_owner() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("history.sqlite3");
        {
            let store = SqliteStore::open(&path).unwrap();
            setup(&store);
            let mut r = record("command-one");
            r.output = b"retained before crash".to_vec();
            r.receipt.output_bytes = r.output.len();
            r.receipt.input_revision = 1;
            r.receipt.input_bytes = 4;
            r.receipt.last_input = Some(dolores_core::command_sessions::InputReceipt {
                call_id: "input-one".into(),
                revision: 1,
                state: "pending".into(),
                bytes: 4,
                written_bytes: 0,
                eof: false,
            });
            store.save_command_session(&r).unwrap();
        }
        let store = SqliteStore::open(&path).unwrap();
        store.recover_command_sessions().unwrap();
        let page = store
            .command_session_output("chat", RUN, "command-one", 0)
            .unwrap();
        assert_eq!(page.output, "retained before crash");
        assert_eq!(page.receipt.state, "lost");
        assert!(!page.receipt.cleanup_complete);
        assert_eq!(page.receipt.last_input.as_ref().unwrap().state, "uncertain");
        store.recover_command_sessions().unwrap();
        assert_eq!(
            store
                .command_session_output("chat", RUN, "command-one", 0)
                .unwrap()
                .receipt,
            page.receipt
        );
        assert!(page
            .receipt
            .message
            .unwrap()
            .contains("nothing was reattached or repeated"));
    }
    #[test]
    fn migration_paging_and_chat_deletion_preserve_other_history() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("history.sqlite3");
        {
            let store = SqliteStore::open(&path).unwrap();
            setup(&store);
            store.commit_turn("chat", "question", "answer").unwrap();
            store.save_draft("chat", "unsent draft").unwrap();
            store
                .lock()
                .unwrap()
                .execute_batch("DROP TABLE command_sessions;PRAGMA user_version=42")
                .unwrap();
        }
        let store = SqliteStore::open(&path).unwrap();
        assert_eq!(store.read_draft("chat").unwrap(), "unsent draft");
        assert_eq!(store.messages("chat").unwrap().len(), 2);
        for i in 0..17 {
            store
                .save_command_session(&record(&format!("command-{i}")))
                .unwrap();
        }
        let page = store.command_sessions("chat", None, None).unwrap();
        assert_eq!(page.items.len(), LIST_PAGE);
        assert!(page.next_cursor.is_some());
        let rest = store
            .command_sessions("chat", None, page.next_cursor)
            .unwrap();
        assert_eq!(rest.items.len(), 1);
        assert!(rest.next_cursor.is_none());
        assert!(store.command_sessions("other", Some(RUN), None).is_err());
        store.delete("chat").unwrap();
        let count: i64 = store
            .lock()
            .unwrap()
            .query_row("SELECT count(*) FROM command_sessions", [], |r| r.get(0))
            .unwrap();
        assert_eq!(count, 0);
        assert_eq!(store.list().unwrap()[0].id, "other");
    }
}
