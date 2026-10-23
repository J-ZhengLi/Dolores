use super::{storage_error, SqliteStore};
use dolores_core::{harness_task::InputEdit, review_findings::*};
use rusqlite::{params, Connection, OptionalExtension, TransactionBehavior};

fn exists(conn: &Connection, session: &str) -> Result<(), String> {
    if !conn
        .query_row(
            "SELECT EXISTS(SELECT 1 FROM sessions WHERE id=?1)",
            [session],
            |r| r.get::<_, bool>(0),
        )
        .map_err(storage_error)?
    {
        return Err("Review chat is unavailable. Retained files were not changed.".into());
    }
    Ok(())
}

fn decode(raw: &str) -> Result<Finding, String> {
    let value: Finding = serde_json::from_str(raw).map_err(storage_error)?;
    value.validate()?;
    Ok(value)
}

fn find(conn: &Connection, session: &str, id: &str) -> Result<Finding, String> {
    let raw: String = conn
        .query_row(
            "SELECT data FROM review_findings WHERE session=?1 AND id=?2",
            params![session, id],
            |r| r.get(0),
        )
        .map_err(|_| "Finding no longer exists in this chat. Refresh before retrying.")?;
    decode(&raw)
}

pub(super) fn read_one(
    store: &SqliteStore,
    session: &str,
    id: &str,
) -> Result<Option<Finding>, String> {
    identity(id)?;
    let conn = store.lock()?;
    exists(&conn, session)?;
    let raw: Option<String> = conn
        .query_row(
            "SELECT data FROM review_findings WHERE session=?1 AND id=?2",
            params![session, id],
            |r| r.get(0),
        )
        .optional()
        .map_err(storage_error)?;
    raw.as_deref().map(decode).transpose()
}

pub(super) fn read_submission(
    store: &SqliteStore,
    session: &str,
    id: &str,
) -> Result<Option<Submission>, String> {
    identity(id)?;
    let conn = store.lock()?;
    exists(&conn, session)?;
    let row: Option<(String, String)> = conn
        .query_row(
            "SELECT selection,mode FROM review_submissions WHERE session=?1 AND id=?2",
            params![session, id],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )
        .optional()
        .map_err(storage_error)?;
    row.map(|(selection, mode)| {
        Ok(Submission {
            id: id.into(),
            selected: serde_json::from_str(&selection).map_err(storage_error)?,
            mode: serde_json::from_str(&mode).map_err(storage_error)?,
        })
    })
    .transpose()
}

fn update(conn: &Connection, session: &str, old: &Finding, new: &Finding) -> Result<(), String> {
    new.validate()?;
    if new.anchor != old.anchor
        || new.id != old.id
        || old.revision.checked_add(1) != Some(new.revision)
    {
        return Err(
            "Finding provenance cannot be replaced. Add a finding on the current diff.".into(),
        );
    }
    let changed = conn.execute("UPDATE review_findings SET revision=?4,data=?5 WHERE session=?1 AND id=?2 AND revision=?3", params![session,old.id,old.revision,new.revision,serde_json::to_string(new).map_err(storage_error)?]).map_err(storage_error)?;
    if changed != 1 {
        return Err("Finding changed. Refresh before retrying.".into());
    }
    Ok(())
}

pub(super) fn page(
    store: &SqliteStore,
    session: &str,
    cursor: Option<i64>,
) -> Result<Page, String> {
    let conn = store.lock()?;
    exists(&conn, session)?;
    if cursor.is_some_and(|v| v <= 0) {
        return Err("Invalid finding cursor. Refresh the list.".into());
    }
    let mut query = conn.prepare("SELECT rowid,data FROM review_findings WHERE session=?1 AND (?2 IS NULL OR rowid<?2) ORDER BY rowid DESC LIMIT ?3").map_err(storage_error)?;
    let rows = query
        .query_map(params![session, cursor, (PAGE_SIZE + 1) as i64], |r| {
            Ok((r.get::<_, i64>(0)?, r.get::<_, String>(1)?))
        })
        .map_err(storage_error)?
        .collect::<Result<Vec<_>, _>>()
        .map_err(storage_error)?;
    let more = rows.len() > PAGE_SIZE;
    let retained: Vec<_> = rows.into_iter().take(PAGE_SIZE).collect();
    let next_cursor = if more {
        retained.last().map(|r| r.0)
    } else {
        None
    };
    Ok(Page {
        items: retained
            .into_iter()
            .map(|r| decode(&r.1))
            .collect::<Result<_, _>>()?,
        next_cursor,
    })
}

pub(super) fn edit(store: &SqliteStore, session: &str, edit: &Edit) -> Result<(), String> {
    let mut conn = store.lock()?;
    let tx = conn
        .transaction_with_behavior(TransactionBehavior::Immediate)
        .map_err(storage_error)?;
    exists(&tx, session)?;
    match edit {
        Edit::Add { finding } => {
            finding.validate()?;
            if finding.revision != 0
                || !finding.submissions.is_empty()
                || finding.resolution.is_some()
            {
                return Err("New findings cannot contain delivery or resolution evidence.".into());
            }
            let raw = serde_json::to_string(finding).map_err(storage_error)?;
            let existing: Option<String> = tx
                .query_row(
                    "SELECT data FROM review_findings WHERE session=?1 AND id=?2",
                    params![session, finding.id],
                    |r| r.get(0),
                )
                .optional()
                .map_err(storage_error)?;
            if let Some(existing) = existing {
                if decode(&existing)? != **finding {
                    return Err(
                        "Finding identity was already used. Refresh before adding another.".into(),
                    );
                }
            } else {
                let count: usize = tx
                    .query_row(
                        "SELECT count(*) FROM review_findings WHERE session=?1",
                        [session],
                        |r| r.get(0),
                    )
                    .map_err(storage_error)?;
                if count >= MAX_FINDINGS {
                    return Err("128 findings are retained. Inspect and remove an unneeded local finding before adding another; files and submitted instructions remain.".into());
                }
                tx.execute(
                    "INSERT INTO review_findings(id,session,revision,data) VALUES(?1,?2,0,?3)",
                    params![finding.id, session, raw],
                )
                .map_err(storage_error)?;
            }
        }
        Edit::Note { id, revision, text } => {
            let old = find(&tx, session, id)?;
            if old.resolution.is_some() {
                return Err("Reopen the resolved finding before changing its note.".into());
            }
            let mut value = old.next(*revision)?;
            value.text = text.clone();
            update(&tx, session, &old, &value)?;
        }
        Edit::Remove { id, revision } => {
            let old = find(&tx, session, id)?;
            old.next(*revision)?;
            tx.execute(
                "DELETE FROM review_findings WHERE session=?1 AND id=?2 AND revision=?3",
                params![session, id, revision],
            )
            .map_err(storage_error)?;
        }
        Edit::Resolve {
            id,
            revision,
            evidence,
        } => {
            let old = find(&tx, session, id)?;
            let mut value = old.next(*revision)?;
            if old.resolution.is_some() {
                return Err("Finding is already resolved. Reopen explicitly before changing its assessment.".into());
            }
            value.resolution = Some(evidence.clone());
            update(&tx, session, &old, &value)?;
        }
        Edit::Reopen { id, revision } => {
            let old = find(&tx, session, id)?;
            let mut value = old.next(*revision)?;
            if old.resolution.is_none() {
                return Err(
                    "Finding is already open. Inspect its delivery receipt before resubmitting."
                        .into(),
                );
            }
            value.resolution = None;
            update(&tx, session, &old, &value)?;
        }
    }
    tx.commit().map_err(storage_error)
}

pub(super) fn submit(
    store: &SqliteStore,
    session: &str,
    submission: &Submission,
) -> Result<dolores_core::harness_task::TaskInputs, String> {
    identity(&submission.id)?;
    if submission.selected.is_empty() || submission.selected.len() > 8 {
        return Err("Select one to eight findings.".into());
    }
    let mut conn = store.lock()?;
    let tx = conn
        .transaction_with_behavior(TransactionBehavior::Immediate)
        .map_err(storage_error)?;
    exists(&tx, session)?;
    // An uncertain IPC reply can be retried with its exact request, including the
    // original revisions. It never appends another input or mutates newer notes.
    let request = serde_json::to_string(&submission.selected).map_err(storage_error)?;
    let mode = serde_json::to_string(&submission.mode).map_err(storage_error)?;
    let previous: Option<(String, String, String)> = tx
        .query_row(
            "SELECT session,selection,mode FROM review_submissions WHERE id=?1",
            [&submission.id],
            |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
        )
        .optional()
        .map_err(storage_error)?;
    if let Some(previous) = previous {
        if previous != (session.into(), request, mode) {
            return Err(
                "Review submission identity already used. Inspect the queue before retrying."
                    .into(),
            );
        }
        return super::task_inputs::read(&tx, session);
    }
    let used: bool = tx
        .query_row(
            "SELECT EXISTS(SELECT 1 FROM task_inputs WHERE id=?1)",
            [&submission.id],
            |r| r.get(0),
        )
        .map_err(storage_error)?;
    if used {
        return Err("Review submission identity belongs to another instruction. Inspect the queue and use a fresh submission.".into());
    }
    let mut selected = vec![];
    let mut ids = std::collections::BTreeSet::new();
    for selection in &submission.selected {
        identity(&selection.id)?;
        if !ids.insert(&selection.id) {
            return Err("Select each finding only once.".into());
        }
        let value = find(&tx, session, &selection.id)?;
        value.next(selection.revision)?;
        if value.resolution.is_some() {
            return Err("Reopen resolved findings explicitly before submitting them.".into());
        }
        if value.submissions.len() >= 16 {
            return Err("This finding retains sixteen submissions. Inspect its receipts and create a new finding if more work is needed.".into());
        }
        selected.push(value);
    }
    let text = instruction(&selected)?;
    super::task_inputs::edit_in_transaction(
        &tx,
        session,
        &InputEdit::Enqueue {
            id: submission.id.clone(),
            text,
            mode: submission.mode,
        },
        false,
    )?;
    for old in &selected {
        let mut value = old.next(old.revision)?;
        value.submissions.push(submission.id.clone());
        update(&tx, session, old, &value)?;
    }
    tx.execute(
        "INSERT INTO review_submissions(id,session,selection,mode) VALUES(?1,?2,?3,?4)",
        params![submission.id, session, request, mode],
    )
    .map_err(storage_error)?;
    tx.commit().map_err(storage_error)?;
    super::task_inputs::read(&conn, session)
}

#[cfg(test)]
mod tests {
    use super::*;
    use dolores_core::{
        harness_task::{InputMode, InputState},
        SessionStore,
    };

    fn finding(id: &str) -> Finding {
        Finding {
            id: id.into(),
            revision: 0,
            anchor: Anchor {
                comparison: Comparison {
                    root: "C:/public-fixture".into(),
                    repository: "public-repository".into(),
                    path: "src/main.rs".into(),
                    basis: "working".into(),
                    commit: None,
                    revision: "before".into(),
                    digest: "a".repeat(64),
                },
                side: Side::Right,
                line: 7,
                excerpt: "let count = input.len();".into(),
            },
            text: "Handle an empty input without losing the retained file.".into(),
            submissions: vec![],
            resolution: None,
        }
    }

    fn add(store: &SqliteStore, id: &str) {
        store
            .edit_review_finding(
                "chat",
                &Edit::Add {
                    finding: finding(id).into(),
                },
            )
            .unwrap();
    }

    fn submission() -> Submission {
        Submission {
            id: "repair-selection".into(),
            mode: InputMode::Queue,
            selected: vec![Selection {
                id: "finding-a".into(),
                revision: 0,
            }],
        }
    }

    #[test]
    fn findings_queue_atomically_reopen_with_uncertain_delivery_and_resolve_against_later_evidence()
    {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("state.db");
        let store = SqliteStore::open(&path).unwrap();
        store.create("chat").unwrap();
        store.create("other").unwrap();
        store.save_draft("chat", "Keep unsent 世界").unwrap();
        let untouched = dir.path().join("retained.txt");
        std::fs::write(&untouched, "original local work").unwrap();
        add(&store, "finding-a");
        add(&store, "finding-a");
        assert_eq!(store.review_findings("chat", None).unwrap().items.len(), 1);
        let mut forged = finding("finding-a");
        forged.text = "replace existing note".into();
        assert!(store
            .edit_review_finding(
                "chat",
                &Edit::Add {
                    finding: forged.into()
                }
            )
            .is_err());
        assert!(store
            .edit_review_finding(
                "other",
                &Edit::Remove {
                    id: "finding-a".into(),
                    revision: 0
                }
            )
            .is_err());
        assert!(store
            .edit_review_finding(
                "chat",
                &Edit::Note {
                    id: "finding-a".into(),
                    revision: 1,
                    text: "stale edit".into()
                }
            )
            .is_err());
        let request = submission();
        let queued = store.submit_review_findings("chat", &request).unwrap();
        assert_eq!(queued.entries.len(), 1);
        assert!(queued.entries[0].text.contains("frozen source data"));
        assert!(queued.entries[0].text.contains("src/main.rs"));
        assert_eq!(store.saved_draft("chat").unwrap(), "Keep unsent 世界");
        assert_eq!(
            store.submit_review_findings("chat", &request).unwrap(),
            queued
        );
        let sent = store.review_findings("chat", None).unwrap().items.remove(0);
        assert_eq!(sent.submissions, vec![request.id.clone()]);
        assert!(sent.resolution.is_none());
        store
            .claim_task_input(
                "chat",
                &request.id,
                queued.entries[0].revision,
                "public-run",
            )
            .unwrap();
        drop(store);
        let store = SqliteStore::open(&path).unwrap();
        store.recover_task_inputs().unwrap();
        let restored = store.submit_review_findings("chat", &request).unwrap();
        assert_eq!(restored.entries.len(), 1);
        assert_eq!(restored.entries[0].state, InputState::Uncertain);
        assert!(restored.recovery_required);
        assert!(store.review_findings("chat", None).unwrap().items[0]
            .resolution
            .is_none());
        let mut evidence = Resolution {
            comparison: sent.anchor.comparison.clone(),
            note: "Inspected the empty-input repair and its test.".into(),
        };
        assert!(store
            .edit_review_finding(
                "chat",
                &Edit::Resolve {
                    id: sent.id.clone(),
                    revision: sent.revision,
                    evidence: evidence.clone()
                }
            )
            .is_err());
        evidence.comparison.revision = "after".into();
        evidence.comparison.digest = "b".repeat(64);
        let mut wrong = evidence.clone();
        wrong.comparison.root = "C:/another-fixture".into();
        assert!(store
            .edit_review_finding(
                "chat",
                &Edit::Resolve {
                    id: sent.id.clone(),
                    revision: sent.revision,
                    evidence: wrong
                }
            )
            .is_err());
        store
            .edit_review_finding(
                "chat",
                &Edit::Resolve {
                    id: sent.id.clone(),
                    revision: sent.revision,
                    evidence,
                },
            )
            .unwrap();
        let resolved = store.review_findings("chat", None).unwrap().items.remove(0);
        assert!(resolved.resolution.is_some());
        store
            .edit_review_finding(
                "chat",
                &Edit::Reopen {
                    id: sent.id.clone(),
                    revision: resolved.revision,
                },
            )
            .unwrap();
        let reopened = store.review_findings("chat", None).unwrap().items.remove(0);
        assert_eq!(reopened.submissions, sent.submissions);
        assert!(reopened.resolution.is_none());
        store
            .edit_review_finding(
                "chat",
                &Edit::Remove {
                    id: sent.id,
                    revision: reopened.revision,
                },
            )
            .unwrap();
        assert!(store
            .review_findings("chat", None)
            .unwrap()
            .items
            .is_empty());
        assert_eq!(store.task_inputs("chat").unwrap().entries, restored.entries);
        assert_eq!(
            std::fs::read_to_string(untouched).unwrap(),
            "original local work"
        );
        assert_eq!(store.saved_draft("chat").unwrap(), "Keep unsent 世界");
    }

    #[test]
    fn failed_submission_and_full_queue_preserve_notes_draft_and_input_order() {
        let store = SqliteStore::open(std::path::Path::new(":memory:")).unwrap();
        store.create("chat").unwrap();
        add(&store, "finding-a");
        store
            .save_draft("chat", "Retained unsent question")
            .unwrap();
        store.lock().unwrap().execute_batch("CREATE TRIGGER refuse_finding_receipt BEFORE UPDATE ON review_findings BEGIN SELECT RAISE(ABORT,'fixture receipt failure'); END;").unwrap();
        assert!(store.submit_review_findings("chat", &submission()).is_err());
        assert!(store.task_inputs("chat").unwrap().entries.is_empty());
        assert!(store.review_findings("chat", None).unwrap().items[0]
            .submissions
            .is_empty());
        assert_eq!(
            store.saved_draft("chat").unwrap(),
            "Retained unsent question"
        );
        store
            .lock()
            .unwrap()
            .execute_batch("DROP TRIGGER refuse_finding_receipt;")
            .unwrap();
        for n in 0..8 {
            store
                .edit_task_inputs(
                    "chat",
                    &InputEdit::Enqueue {
                        id: format!("queued-{n}"),
                        text: format!("Existing task {n}"),
                        mode: InputMode::Queue,
                    },
                )
                .unwrap();
        }
        let before = store.task_inputs("chat").unwrap();
        assert!(store
            .submit_review_findings("chat", &submission())
            .unwrap_err()
            .contains("queue is full"));
        assert_eq!(store.task_inputs("chat").unwrap(), before);
        assert!(store.review_findings("chat", None).unwrap().items[0]
            .submissions
            .is_empty());
        store
            .edit_task_inputs(
                "chat",
                &InputEdit::Cancel {
                    id: before.entries[0].id.clone(),
                    revision: before.entries[0].revision,
                },
            )
            .unwrap();
        let after = store.submit_review_findings("chat", &submission()).unwrap();
        assert_eq!(after.entries.last().unwrap().id, "repair-selection");
        assert_eq!(after.entries[1].id, before.entries[1].id);
        assert_eq!(
            store.saved_draft("chat").unwrap(),
            "Retained unsent question"
        );
        let count: usize = store
            .lock()
            .unwrap()
            .query_row("SELECT count(*) FROM review_submissions", [], |r| r.get(0))
            .unwrap();
        assert_eq!(count, 1);
    }

    #[test]
    fn duplicate_oversized_and_colliding_submissions_cannot_claim_existing_instructions() {
        let store = SqliteStore::open(std::path::Path::new(":memory:")).unwrap();
        store.create("chat").unwrap();
        add(&store, "finding-a");
        let request = submission();
        let body = instruction(&[finding("finding-a")]).unwrap();
        store
            .edit_task_inputs(
                "chat",
                &InputEdit::Enqueue {
                    id: request.id.clone(),
                    text: body,
                    mode: request.mode,
                },
            )
            .unwrap();
        assert!(store
            .submit_review_findings("chat", &request)
            .unwrap_err()
            .contains("another instruction"));
        assert!(store.review_findings("chat", None).unwrap().items[0]
            .submissions
            .is_empty());
        assert_eq!(store.task_inputs("chat").unwrap().entries.len(), 1);
        let duplicate = Submission {
            id: "duplicate-selection".into(),
            mode: InputMode::Steer,
            selected: vec![
                Selection {
                    id: "finding-a".into(),
                    revision: 0,
                },
                Selection {
                    id: "finding-a".into(),
                    revision: 0,
                },
            ],
        };
        assert!(store
            .submit_review_findings("chat", &duplicate)
            .unwrap_err()
            .contains("only once"));
        let mut selected = vec![];
        for n in 0..3 {
            let mut large = finding(&format!("large-{n}"));
            large.text = "public finding ".repeat(270);
            large.anchor.excerpt = "public source ".repeat(140);
            selected.push(Selection {
                id: large.id.clone(),
                revision: 0,
            });
            store
                .edit_review_finding(
                    "chat",
                    &Edit::Add {
                        finding: large.into(),
                    },
                )
                .unwrap();
        }
        let oversized = Submission {
            id: "oversized-selection".into(),
            mode: InputMode::Queue,
            selected,
        };
        assert!(store
            .submit_review_findings("chat", &oversized)
            .unwrap_err()
            .contains("16 KiB"));
        assert_eq!(store.task_inputs("chat").unwrap().entries.len(), 1);
        assert!(store
            .review_findings("chat", None)
            .unwrap()
            .items
            .iter()
            .all(|f| f.submissions.is_empty()));
        assert_eq!(store.review_findings("chat", None).unwrap().items.len(), 4);
    }

    #[test]
    fn schema_45_migration_bounded_pages_capacity_and_delete_preserve_existing_history() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("state.db");
        let store = SqliteStore::open(&path).unwrap();
        store.create("chat").unwrap();
        store
            .commit_turn("chat", "Existing question 世界", "Retained reply")
            .unwrap();
        store.save_draft("chat", "Existing draft").unwrap();
        let history = store.messages("chat").unwrap();
        store.lock().unwrap().execute_batch("DROP TABLE review_findings; DROP TABLE review_submissions; PRAGMA user_version=44;").unwrap();
        drop(store);
        let store = SqliteStore::open(&path).unwrap();
        assert_eq!(store.messages("chat").unwrap(), history);
        assert_eq!(store.saved_draft("chat").unwrap(), "Existing draft");
        for n in 0..MAX_FINDINGS {
            add(&store, &format!("finding-{n}"));
        }
        assert!(store
            .edit_review_finding(
                "chat",
                &Edit::Add {
                    finding: finding("overflow").into()
                }
            )
            .unwrap_err()
            .contains("128 findings"));
        let mut cursor = None;
        let mut ids = std::collections::BTreeSet::new();
        loop {
            let page = store.review_findings("chat", cursor).unwrap();
            assert_eq!(page.items.len(), PAGE_SIZE);
            for finding in page.items {
                assert!(ids.insert(finding.id));
            }
            cursor = page.next_cursor;
            if cursor.is_none() {
                break;
            }
        }
        assert_eq!(ids.len(), MAX_FINDINGS);
        store
            .edit_review_finding(
                "chat",
                &Edit::Remove {
                    id: "finding-0".into(),
                    revision: 0,
                },
            )
            .unwrap();
        add(&store, "overflow");
        assert_eq!(store.messages("chat").unwrap(), history);
        store.delete("chat").unwrap();
        let remaining: i64 = store
            .lock()
            .unwrap()
            .query_row("SELECT count(*) FROM review_findings", [], |r| r.get(0))
            .unwrap();
        assert_eq!(remaining, 0);
    }
}
