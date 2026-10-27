use super::storage_error;
use dolores_core::harness_task::*;
use rusqlite::{params, Connection, OptionalExtension};

fn exists(c: &Connection, s: &str) -> Result<(), String> {
    let found: bool = c
        .query_row(
            "SELECT EXISTS(SELECT 1 FROM sessions WHERE id=?1)",
            [s],
            |r| r.get(0),
        )
        .map_err(storage_error)?;
    if found {
        Ok(())
    } else {
        Err("Chat no longer exists.".into())
    }
}

fn entries(c: &Connection, s: &str, pending_only: bool) -> Result<Vec<TaskInput>, String> {
    let sql = if pending_only {
        "SELECT data FROM task_inputs WHERE session=?1 AND state='pending' ORDER BY position"
    } else {
        // Retain all receipts on disk; return unresolved + 50 latest terminal receipts.
        "SELECT data FROM (SELECT position,data FROM task_inputs WHERE session=?1 AND (state IN ('pending','delivering') OR (state='uncertain' AND json_extract(data,'$.inspected')=0)) UNION ALL SELECT position,data FROM (SELECT position,data FROM task_inputs WHERE session=?1 AND (state IN ('applied','cancelled','failed') OR (state='uncertain' AND json_extract(data,'$.inspected')=1)) ORDER BY position DESC LIMIT 50)) ORDER BY position"
    };
    let mut q = c.prepare(sql).map_err(storage_error)?;
    let result = q
        .query_map([s], |r| r.get::<_, String>(0))
        .map_err(storage_error)?
        .map(|r| serde_json::from_str(&r.map_err(storage_error)?).map_err(storage_error))
        .collect();
    result
}

pub(super) fn read(c: &Connection, s: &str) -> Result<TaskInputs, String> {
    exists(c, s)?;
    let mut state = c
        .query_row(
            "SELECT revision,suspended,recovery FROM task_input_state WHERE session=?1",
            [s],
            |r| {
                Ok(TaskInputs {
                    revision: r.get(0)?,
                    suspended: r.get(1)?,
                    recovery_required: r.get(2)?,
                    entries: vec![],
                })
            },
        )
        .optional()
        .map_err(storage_error)?
        .unwrap_or_default();
    state.entries = entries(c, s, false)?;
    Ok(state)
}

fn text(value: &str) -> Result<(), String> {
    if value.trim().is_empty()
        || value.len() > dolores_core::MAX_INPUT_BYTES
        || value.contains('\0')
    {
        Err("Instruction must contain text within 16 KiB and no NUL. Shorten it; your draft remains.".into())
    } else {
        Ok(())
    }
}
fn save(c: &Connection, s: &str, v: &TaskInput) -> Result<(), String> {
    c.execute("INSERT INTO task_inputs(id,session,position,state,data) VALUES(?1,?2,?3,?4,?5) ON CONFLICT(id) DO UPDATE SET position=excluded.position,state=excluded.state,data=excluded.data",params![v.id,s,v.position,serde_json::to_value(v.state).map_err(storage_error)?.as_str().unwrap(),serde_json::to_string(v).map_err(storage_error)?]).map_err(storage_error)?;
    Ok(())
}
fn bump(c: &Connection, s: &str, state: &TaskInputs) -> Result<(), String> {
    let revision = next_revision(state.revision, state.revision)?;
    c.execute("INSERT INTO task_input_state VALUES(?1,?2,?3,?4) ON CONFLICT(session) DO UPDATE SET revision=excluded.revision,suspended=excluded.suspended,recovery=excluded.recovery",params![s,revision,state.suspended,state.recovery_required]).map_err(storage_error)?;
    Ok(())
}
fn find(c: &Connection, s: &str, id: &str) -> Result<TaskInput, String> {
    let data: String = c
        .query_row(
            "SELECT data FROM task_inputs WHERE session=?1 AND id=?2",
            params![s, id],
            |r| r.get(0),
        )
        .map_err(|_| "Instruction no longer exists. Refresh the queue.".to_string())?;
    serde_json::from_str(&data).map_err(storage_error)
}

pub(super) fn edit(c: &mut Connection, s: &str, e: &InputEdit) -> Result<TaskInputs, String> {
    let tx = c.transaction().map_err(storage_error)?;
    edit_in_transaction(&tx, s, e, true)?;
    tx.commit().map_err(storage_error)?;
    read(c, s)
}

// Review submission shares this exact queue admission and snapshot path. The
// caller owns the encompassing transaction, including finding delivery records.
pub(super) fn edit_in_transaction(
    tx: &Connection,
    s: &str,
    e: &InputEdit,
    consume_draft: bool,
) -> Result<TaskInputs, String> {
    let mut state = read(tx, s)?;
    match e {
        InputEdit::Acknowledge { id, revision } => {
            let mut v = find(tx, s, id)?;
            if v.state != InputState::Uncertain {
                return Err(
                    "Only uncertain delivery needs acknowledgement. Refresh its receipt.".into(),
                );
            }
            v.revision = next_revision(v.revision, *revision)?;
            v.inspected = true;
            save(tx, s, &v)?;
        }
        InputEdit::Enqueue {
            id,
            text: body,
            mode,
        } => {
            text(body)?;
            if id.is_empty()
                || id.len() > 128
                || !id
                    .bytes()
                    .all(|b| b.is_ascii_alphanumeric() || b"-_".contains(&b))
            {
                return Err("Invalid instruction identity.".into());
            }
            if let Ok(old) = find(tx, s, id) {
                if old.text == *body && old.mode == *mode {
                    return Ok(state);
                }
                return Err(
                    "Instruction identity already used. Refresh before submitting new text.".into(),
                );
            }
            let used: bool = tx
                .query_row(
                    "SELECT EXISTS(SELECT 1 FROM task_inputs WHERE id=?1)",
                    [id],
                    |r| r.get(0),
                )
                .map_err(storage_error)?;
            if used {
                return Err("Instruction identity already used.".into());
            }
            if state
                .entries
                .iter()
                .filter(|v| {
                    !v.inspected
                        && matches!(
                            v.state,
                            InputState::Pending | InputState::Delivering | InputState::Uncertain
                        )
                })
                .count()
                >= MAX_PENDING_INPUTS
            {
                return Err("Instruction queue is full. Finish, cancel or inspect pending work; your draft remains.".into());
            }
            // Text-only queue in EH01. Never consume or accidentally share another draft's attachments.
            if !super::attachments::draft(tx, s)?.is_empty() {
                return Err("Queued instructions currently support text. Remove draft attachments or wait to send them; your draft remains.".into());
            }
            let position: u64 = tx
                .query_row(
                    "SELECT COALESCE(MAX(position),0)+1 FROM task_inputs WHERE session=?1",
                    [s],
                    |r| r.get(0),
                )
                .map_err(storage_error)?;
            save(
                tx,
                s,
                &TaskInput {
                    id: id.clone(),
                    revision: 1,
                    position,
                    mode: *mode,
                    text: body.clone(),
                    state: InputState::Pending,
                    run: None,
                    inspected: false,
                    planning: super::planning::read(tx, s)?,
                    selected_skills: {
                        let selections = super::composer_skills::read(tx, s)?;
                        dolores_core::resolve_skill_selections(
                            &super::composer_skills::available(tx, s)?,
                            &selections
                                .iter()
                                .filter(|s| !s.file)
                                .cloned()
                                .collect::<Vec<_>>(),
                        )?;
                        selections
                    },
                },
            )?;
            if consume_draft {
                tx.execute(
                    "DELETE FROM session_drafts WHERE session_id=?1 AND text=?2",
                    params![s, body],
                )
                .map_err(storage_error)?;
            }
        }
        InputEdit::Edit {
            id,
            revision,
            text: body,
        } => {
            text(body)?;
            let mut v = find(tx, s, id)?;
            if !v.state.editable() {
                return Err("Instruction is already being delivered. Inspect its receipt.".into());
            }
            v.revision = next_revision(v.revision, *revision)?;
            v.text = body.clone();
            save(tx, s, &v)?;
        }
        InputEdit::Cancel { id, revision } => {
            let mut v = find(tx, s, id)?;
            v.revision = next_revision(v.revision, *revision)?;
            v.state = v.state.transition(InputState::Cancelled)?;
            save(tx, s, &v)?;
        }
        InputEdit::Reorder { revision, ids } => {
            next_revision(state.revision, *revision)?;
            let pending = entries(tx, s, true)?;
            let set: std::collections::BTreeSet<_> = ids.iter().collect();
            if pending.len() != ids.len()
                || set.len() != ids.len()
                || pending.iter().any(|v| !set.contains(&v.id))
            {
                return Err("Queue changed. Refresh before reordering.".into());
            }
            let positions: Vec<_> = pending.iter().map(|v| v.position).collect();
            for (i, id) in ids.iter().enumerate() {
                let mut v = find(tx, s, id)?;
                v.position = positions[i];
                v.revision = next_revision(v.revision, v.revision)?;
                save(tx, s, &v)?;
            }
        }
        InputEdit::Resume { revision } => {
            next_revision(state.revision, *revision)?;
            state.suspended = false;
            state.recovery_required = false;
        }
    }
    bump(tx, s, &state)?;
    read(tx, s)
}

pub(super) fn claim(
    c: &mut Connection,
    s: &str,
    id: &str,
    revision: u32,
    run: &str,
) -> Result<TaskInput, String> {
    let tx = c.transaction().map_err(storage_error)?;
    let state = read(&tx, s)?;
    if state.suspended || state.recovery_required {
        return Err("Task is paused. Inspect pending inputs and Resume explicitly.".into());
    }
    let mut v = find(&tx, s, id)?;
    v.revision = next_revision(v.revision, revision)?;
    v.state = v.state.transition(InputState::Delivering)?;
    v.run = Some(run.into());
    save(&tx, s, &v)?;
    bump(&tx, s, &state)?;
    tx.commit().map_err(storage_error)?;
    Ok(v)
}
pub(super) fn finish(
    c: &mut Connection,
    s: &str,
    id: &str,
    next: InputState,
) -> Result<(), String> {
    let tx = c.transaction().map_err(storage_error)?;
    let state = read(&tx, s)?;
    let mut v = find(&tx, s, id)?;
    v.revision = next_revision(v.revision, v.revision)?;
    v.state = v.state.transition(next)?;
    save(&tx, s, &v)?;
    bump(&tx, s, &state)?;
    tx.commit().map_err(storage_error)
}
pub(super) fn suspend(c: &mut Connection, s: &str) -> Result<(), String> {
    let tx = c.transaction().map_err(storage_error)?;
    let mut state = read(&tx, s)?;
    state.suspended = true;
    bump(&tx, s, &state)?;
    tx.commit().map_err(storage_error)
}
pub(super) fn recover(c: &mut Connection) -> Result<(), String> {
    let tx = c.transaction().map_err(storage_error)?;
    let sessions: Vec<String> = {
        let mut q = tx
            .prepare("SELECT session FROM task_input_state")
            .map_err(storage_error)?;
        let values = q
            .query_map([], |r| r.get(0))
            .map_err(storage_error)?
            .collect::<Result<_, _>>()
            .map_err(storage_error)?;
        values
    };
    for s in sessions {
        let mut state = read(&tx, &s)?;
        if state
            .entries
            .iter()
            .any(|v| matches!(v.state, InputState::Pending | InputState::Delivering))
        {
            for v in &state.entries {
                if v.state == InputState::Delivering {
                    let mut v = v.clone();
                    v.state = v.state.after_restart();
                    v.revision = next_revision(v.revision, v.revision)?;
                    save(&tx, &s, &v)?;
                }
            }
            state.recovery_required = true;
            state.suspended = true;
            bump(&tx, &s, &state)?;
        }
    }
    tx.commit().map_err(storage_error)
}

#[cfg(test)]
mod tests {
    use super::super::SqliteStore;
    use super::*;
    use dolores_core::SessionStore;
    fn enqueue(id: &str) -> InputEdit {
        InputEdit::Enqueue {
            id: id.into(),
            text: format!("public instruction {id}"),
            mode: InputMode::Queue,
        }
    }
    #[test]
    fn queue_reorder_stale_edit_and_failed_save_preserve_draft_and_receipts() {
        let store = SqliteStore::open(std::path::Path::new(":memory:")).unwrap();
        store.create("chat").unwrap();
        store.save_draft("chat", "public instruction a").unwrap();
        let a = store.edit_task_inputs("chat", &enqueue("a")).unwrap();
        assert_eq!(store.saved_draft("chat").unwrap(), "");
        assert_eq!(store.edit_task_inputs("chat", &enqueue("a")).unwrap(), a);
        let b = store.edit_task_inputs("chat", &enqueue("b")).unwrap();
        let reordered = store
            .edit_task_inputs(
                "chat",
                &InputEdit::Reorder {
                    revision: b.revision,
                    ids: vec!["b".into(), "a".into()],
                },
            )
            .unwrap();
        assert_eq!(reordered.entries[0].id, "b");
        assert!(store
            .edit_task_inputs(
                "chat",
                &InputEdit::Edit {
                    id: "a".into(),
                    revision: 1,
                    text: "stale".into()
                }
            )
            .is_err());
        store.save_draft("chat", "public instruction c").unwrap();
        store.lock().unwrap().execute_batch("CREATE TRIGGER reject_input BEFORE INSERT ON task_inputs BEGIN SELECT RAISE(ABORT,'fixture'); END;").unwrap();
        assert!(store.edit_task_inputs("chat", &enqueue("c")).is_err());
        assert_eq!(store.saved_draft("chat").unwrap(), "public instruction c");
        assert_eq!(store.task_inputs("chat").unwrap(), reordered);
    }
    #[test]
    fn restart_marks_uncertain_never_replays_and_delete_cascades() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("task.db");
        let store = SqliteStore::open(&path).unwrap();
        store.create("chat").unwrap();
        store.edit_task_inputs("chat", &enqueue("a")).unwrap();
        store.edit_task_inputs("chat", &enqueue("b")).unwrap();
        store.claim_task_input("chat", "a", 1, "run").unwrap();
        drop(store);
        let store = SqliteStore::open(&path).unwrap();
        store.recover_task_inputs().unwrap();
        let state = store.task_inputs("chat").unwrap();
        assert!(state.suspended && state.recovery_required);
        assert_eq!(state.entries[0].state, InputState::Uncertain);
        assert_eq!(state.entries[1].state, InputState::Pending);
        assert!(store.claim_task_input("chat", "b", 1, "next").is_err());
        store
            .edit_task_inputs(
                "chat",
                &InputEdit::Resume {
                    revision: state.revision,
                },
            )
            .unwrap();
        store.claim_task_input("chat", "b", 1, "next").unwrap();
        store
            .finish_task_input("chat", "b", InputState::Applied)
            .unwrap();
        assert!(store.claim_task_input("chat", "a", 2, "replay").is_err());
        assert!(store.claim_task_input("chat", "b", 3, "replay").is_err());
        store.delete("chat").unwrap();
        let count: i64 = store
            .lock()
            .unwrap()
            .query_row("SELECT COUNT(*) FROM task_inputs", [], |r| r.get(0))
            .unwrap();
        assert_eq!(count, 0);
    }

    #[test]
    fn v38_migration_capacity_and_uncertainty_acknowledgement_keep_history() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("legacy.db");
        let store = SqliteStore::open(&path).unwrap();
        store.create("chat").unwrap();
        store
            .commit_turn("chat", "Legacy 世界", "Exact reply")
            .unwrap();
        store.save_draft("chat", "retained draft").unwrap();
        let before = store.messages("chat").unwrap();
        store
            .lock()
            .unwrap()
            .execute_batch(
                "DROP TABLE task_inputs;DROP TABLE task_input_state;PRAGMA user_version=38;",
            )
            .unwrap();
        drop(store);
        let store = SqliteStore::open(&path).unwrap();
        assert_eq!(store.messages("chat").unwrap(), before);
        assert_eq!(store.saved_draft("chat").unwrap(), "retained draft");
        for n in 0..MAX_PENDING_INPUTS {
            store
                .edit_task_inputs("chat", &enqueue(&format!("i{n}")))
                .unwrap();
        }
        assert!(store
            .edit_task_inputs("chat", &enqueue("overflow"))
            .unwrap_err()
            .contains("queue is full"));
        store
            .claim_task_input("chat", "i0", 1, "interrupted")
            .unwrap();
        store.recover_task_inputs().unwrap();
        let state = store.task_inputs("chat").unwrap();
        let uncertain = state.entries.iter().find(|v| v.id == "i0").unwrap();
        store
            .edit_task_inputs(
                "chat",
                &InputEdit::Acknowledge {
                    id: uncertain.id.clone(),
                    revision: uncertain.revision,
                },
            )
            .unwrap();
        store
            .edit_task_inputs("chat", &enqueue("overflow"))
            .unwrap();
        let mut output = vec![];
        store
            .export_conversation("chat", dolores_core::ExportFormat::Json, &mut output)
            .unwrap();
        let json: serde_json::Value = serde_json::from_slice(&output).unwrap();
        assert_eq!(json["instructions"].as_array().unwrap().len(), 9);
        assert_eq!(json["instructions"][0]["state"], "uncertain");
        assert_eq!(json["instructions"][0]["inspected"], true);
        assert_eq!(store.saved_draft("chat").unwrap(), "retained draft");
    }
}
