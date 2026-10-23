use super::{now, storage_error, SqliteStore};
use dolores_core::{worktrees::*, Session};
use rusqlite::{params, Connection, OptionalExtension, TransactionBehavior};

fn write_error(error: rusqlite::Error) -> String {
    if error.sqlite_error_code() == Some(rusqlite::ErrorCode::DiskFull) {
        "Local storage is full. Free space on the data drive, refresh Worktrees and review the operation again. Retained files and snapshots remain available for inspection.".into()
    } else {
        storage_error(error)
    }
}

fn decode(raw: &str) -> Result<ManagedWorktree, String> {
    let record: ManagedWorktree = serde_json::from_str(raw).map_err(storage_error)?;
    record.validate()?;
    Ok(record)
}

fn read(conn: &Connection, session: &str) -> Result<Option<ManagedWorktree>, String> {
    let raw: Option<String> = conn
        .query_row(
            "SELECT data FROM managed_worktrees WHERE session=?1",
            [session],
            |r| r.get(0),
        )
        .optional()
        .map_err(storage_error)?;
    raw.as_deref().map(decode).transpose()
}

fn state(record: &ManagedWorktree) -> Result<String, String> {
    serde_json::to_value(record.state)
        .map_err(storage_error)?
        .as_str()
        .map(str::to_owned)
        .ok_or_else(|| "Invalid worktree state.".into())
}

fn update(conn: &Connection, record: &ManagedWorktree) -> Result<(), String> {
    let old = read(conn, &record.session)?
        .ok_or("Managed worktree record is unavailable. Inspect its folder before retrying.")?;
    record.validate_update(&old)?;
    let changed = conn
        .execute(
            "UPDATE managed_worktrees SET revision=?3,state=?4,data=?5 WHERE id=?1 AND revision=?2",
            params![
                record.id,
                old.revision,
                record.revision,
                state(record)?,
                serde_json::to_string(record).map_err(storage_error)?
            ],
        )
        .map_err(write_error)?;
    if changed != 1 {
        return Err(
            "Worktree changed. Refresh before retrying; retained work was not removed.".into(),
        );
    }
    Ok(())
}

pub(super) fn create(store: &SqliteStore, record: &ManagedWorktree) -> Result<Session, String> {
    record.validate_creation()?;
    let session = Session {
        id: record.session.clone(),
        title: "Worktree chat".into(),
        updated_at: now(),
    };
    let mut conn = store.lock()?;
    let tx = conn
        .transaction_with_behavior(TransactionBehavior::Immediate)
        .map_err(storage_error)?;
    tx.execute(
        "INSERT INTO sessions(id,title,updated_at) VALUES(?1,?2,?3)",
        params![session.id, session.title, session.updated_at],
    )
    .map_err(storage_error)?;
    // Bind the actual target from the first intent, never the source checkout.
    // Host admission refuses non-Ready records; no alternate-folder fallback.
    tx.execute(
        "INSERT INTO session_workspaces(session_id,kind,root) VALUES(?1,'project',?2)",
        params![session.id, record.root],
    )
    .map_err(storage_error)?;
    // Register the actual checkout's settings scope without copying source grants.
    tx.execute(
        "INSERT INTO projects(root,name,updated_at) VALUES(?1,?2,?3)",
        params![
            record.root,
            format!("Worktree · {}", record.base_ref),
            session.updated_at
        ],
    )
    .map_err(storage_error)?;
    tx.execute("INSERT INTO managed_worktrees(id,session,root,revision,state,data) VALUES(?1,?2,?3,0,'creating',?4)", params![record.id,record.session,record.root,serde_json::to_string(record).map_err(storage_error)?]).map_err(storage_error)?;
    tx.commit().map_err(storage_error)?;
    Ok(session)
}

pub(super) fn get(store: &SqliteStore, session: &str) -> Result<Option<ManagedWorktree>, String> {
    let connection = store.lock()?;
    read(&connection, session)
}

pub(super) fn for_root(store: &SqliteStore, root: &str) -> Result<Option<ManagedWorktree>, String> {
    let conn = store.lock()?;
    let data:Option<String>=conn.query_row("SELECT data FROM managed_worktrees WHERE root=?1 OR (substr(?1,1,length(root))=root COLLATE NOCASE AND substr(?1,length(root)+1,1) IN ('/',char(92))) ORDER BY length(root) DESC LIMIT 1",[root],|r|r.get(0)).optional().map_err(storage_error)?;
    data.as_deref().map(decode).transpose()
}

pub(super) fn save(store: &SqliteStore, record: &ManagedWorktree) -> Result<(), String> {
    let mut conn = store.lock()?;
    let tx = conn
        .transaction_with_behavior(TransactionBehavior::Immediate)
        .map_err(write_error)?;
    update(&tx, record)?;
    tx.commit().map_err(write_error)
}

pub(super) fn list(store: &SqliteStore, before: Option<i64>) -> Result<WorktreePage, String> {
    if before.is_some_and(|v| v <= 0) {
        return Err("Invalid worktree cursor. Refresh the list.".into());
    }
    let conn = store.lock()?;
    let mut statement = conn.prepare("SELECT rowid,data FROM managed_worktrees WHERE (?1 IS NULL OR rowid<?1) ORDER BY rowid DESC LIMIT ?2").map_err(storage_error)?;
    let mut rows = statement
        .query(params![before, (WORKTREE_PAGE_SIZE + 1) as i64])
        .map_err(storage_error)?;
    let mut items = vec![];
    let mut last = None;
    let mut more = false;
    while let Some(row) = rows.next().map_err(storage_error)? {
        if items.len() == WORKTREE_PAGE_SIZE {
            more = true;
            break;
        }
        last = Some(row.get::<_, i64>(0).map_err(storage_error)?);
        items.push(decode(&row.get::<_, String>(1).map_err(storage_error)?)?);
    }
    Ok(WorktreePage {
        items,
        next_cursor: if more { last } else { None },
    })
}

pub(super) fn recover(store: &SqliteStore) -> Result<(), String> {
    let mut conn = store.lock()?;
    let tx = conn
        .transaction_with_behavior(TransactionBehavior::Immediate)
        .map_err(storage_error)?;
    // Iterate stable row IDs with bounded memory, even with many retained archives.
    let mut after = 0i64;
    loop {
        let row: Option<(i64,String)> = tx.query_row(
            "SELECT rowid,data FROM managed_worktrees WHERE rowid>?1 AND state IN ('creating','archiving','restoring') ORDER BY rowid LIMIT 1",
            [after], |r| Ok((r.get(0)?,r.get(1)?)),
        ).optional().map_err(storage_error)?;
        let Some((id, raw)) = row else {
            break;
        };
        let record = decode(&raw)?.interrupted()?;
        update(&tx, &record)?;
        after = id;
    }
    tx.commit().map_err(storage_error)
}

#[cfg(test)]
mod tests {
    use super::*;
    use dolores_core::{ExportFormat, SessionStore};
    use std::path::Path;

    fn record(base: &Path) -> ManagedWorktree {
        let id = uuid::Uuid::new_v4().to_string();
        ManagedWorktree {
            session: id.clone(),
            root: base.join(&id).to_string_lossy().into_owned(),
            id,
            revision: 0,
            source_root: base.join("original").to_string_lossy().into_owned(),
            common_directory: base.join("original/.git").to_string_lossy().into_owned(),
            common_identity: "volume:repository".into(),
            base_ref: "main".into(),
            base_commit: "a".repeat(40),
            setup: WorktreeSetup::Skip,
            state: WorktreeState::Creating,
            identity: None,
            snapshot: None,
            recovery_from: None,
            notice: String::new(),
        }
    }

    fn ready(old: &ManagedWorktree) -> ManagedWorktree {
        let mut record = old.clone();
        record.revision += 1;
        record.state = WorktreeState::Ready;
        record.recovery_from = None;
        record.identity = Some(WorktreeIdentity {
            filesystem: "volume:file-id".into(),
            git_directory: format!("{}/worktrees/{}", record.common_directory, record.id),
            git_identity: "volume:registration".into(),
        });
        record
    }

    #[test]
    fn worktree_binding_reopen_delete_and_private_exports_preserve_recovery() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("state.db");
        let initial = record(dir.path());
        {
            let store = SqliteStore::open(&path).unwrap();
            store.create_worktree(&initial).unwrap();
            assert_eq!(
                store.workspace(&initial.session).unwrap().root,
                Some(initial.root.clone())
            );
            assert_eq!(store.projects().unwrap()[0].root, initial.root);
            assert!(store
                .scoped_settings(dolores_core::SettingsScope::Project, &initial.root)
                .is_ok());
            store
                .commit_turn(&initial.session, "Fixture question", "Fixture reply")
                .unwrap();
            for format in [ExportFormat::Json, ExportFormat::Markdown] {
                let mut bytes = vec![];
                store
                    .export_conversation(&initial.session, format, &mut bytes)
                    .unwrap();
                let text = String::from_utf8(bytes).unwrap();
                assert!(!text.contains(&initial.source_root));
                assert!(!text.contains(&initial.root));
            }
        }
        let store = SqliteStore::open(&path).unwrap();
        store.recover_worktrees().unwrap();
        let interrupted = store.worktree(&initial.session).unwrap().unwrap();
        assert_eq!(interrupted.state, WorktreeState::Recovery);
        assert_eq!(interrupted.recovery_from, Some(WorktreeState::Creating));
        store.recover_worktrees().unwrap();
        assert_eq!(
            store.worktree(&initial.session).unwrap().unwrap(),
            interrupted
        );
        let current = ready(&interrupted);
        store.save_worktree(&current).unwrap();
        store.delete(&initial.session).unwrap();
        assert!(store.workspace(&initial.session).is_err());
        assert_eq!(store.worktree(&initial.session).unwrap().unwrap(), current);
        assert_eq!(store.worktrees(None).unwrap().items.len(), 1);
    }

    #[test]
    fn worktree_stale_update_and_failed_insert_are_atomic() {
        let dir = tempfile::tempdir().unwrap();
        let store = SqliteStore::open(&dir.path().join("state.db")).unwrap();
        let original = record(dir.path());
        store.create_worktree(&original).unwrap();
        let current = ready(&original);
        store.save_worktree(&current).unwrap();
        assert!(store.save_worktree(&current).is_err());
        let mut conflicting = record(dir.path());
        conflicting.root = original.root.clone();
        assert!(store.create_worktree(&conflicting).is_err());
        assert!(store.workspace(&conflicting.session).is_err());
        assert!(store.worktree(&conflicting.session).unwrap().is_none());
        let mut changed = current.clone();
        changed.revision += 1;
        changed.state = WorktreeState::Archiving;
        changed.source_root = conflicting.source_root + "-other";
        assert!(store.save_worktree(&changed).is_err());
        assert_eq!(store.worktree(&original.session).unwrap().unwrap(), current);
    }

    #[test]
    fn worktree_failed_storage_keeps_saved_snapshot_and_checkout_binding() {
        let dir = tempfile::tempdir().unwrap();
        let store = SqliteStore::open(&dir.path().join("state.db")).unwrap();
        let initial = record(dir.path());
        store.create_worktree(&initial).unwrap();
        let mut current = ready(&initial);
        store.save_worktree(&current).unwrap();
        current.revision += 1;
        current.state = WorktreeState::Archiving;
        current.snapshot = Some("b".repeat(40));
        store.save_worktree(&current).unwrap();
        // A deterministic SQLite write failure exercises the transaction boundary;
        // actual disk exhaustion during Git snapshot/restore needs host qualification.
        store.lock().unwrap().execute_batch("CREATE TRIGGER fail_worktree_update BEFORE UPDATE ON managed_worktrees BEGIN SELECT RAISE(ABORT,'fixture write failure'); END;").unwrap();
        let mut archived = current.clone();
        archived.revision += 1;
        archived.state = WorktreeState::Archived;
        assert!(store.save_worktree(&archived).is_err());
        assert_eq!(store.worktree(&initial.session).unwrap().unwrap(), current);
        assert_eq!(
            store.workspace(&initial.session).unwrap().root,
            Some(initial.root)
        );
        store
            .lock()
            .unwrap()
            .execute_batch("DROP TRIGGER fail_worktree_update;")
            .unwrap();
        store.recover_worktrees().unwrap();
        let recovered = store.worktree(&initial.session).unwrap().unwrap();
        assert_eq!(recovered.snapshot, current.snapshot);
        assert_eq!(recovered.recovery_from, Some(WorktreeState::Archiving));
    }

    #[test]
    fn worktree_sqlite_full_quota_preserves_intent_and_explicit_recovery() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("state.db");
        let store = SqliteStore::open(&path).unwrap();
        let initial = record(dir.path());
        store.create_worktree(&initial).unwrap();
        let current = ready(&initial);
        store.save_worktree(&current).unwrap();
        {
            let conn = store.lock().unwrap();
            conn.execute_batch("CREATE TABLE quota_pressure(bytes BLOB); CREATE TRIGGER exhaust_archive_quota BEFORE UPDATE ON managed_worktrees WHEN NEW.state='archiving' BEGIN INSERT INTO quota_pressure VALUES(zeroblob(1048576)); END;").unwrap();
            let pages: i64 = conn
                .query_row("PRAGMA page_count", [], |r| r.get(0))
                .unwrap();
            let limit: i64 = conn
                .query_row(&format!("PRAGMA max_page_count={}", pages + 4), [], |r| {
                    r.get(0)
                })
                .unwrap();
            assert_eq!(limit, pages + 4);
            // This is SQLite's actual allocation refusal, not a RAISE message.
            let full = conn
                .execute("INSERT INTO quota_pressure VALUES(zeroblob(1048576))", [])
                .unwrap_err();
            assert_eq!(
                full.sqlite_error_code(),
                Some(rusqlite::ErrorCode::DiskFull)
            );
        }
        let mut intent = current.clone();
        intent.revision += 1;
        intent.state = WorktreeState::Archiving;
        intent.snapshot = Some("b".repeat(40));
        let error = store.save_worktree(&intent).unwrap_err();
        assert!(error.contains("full"), "{error}");
        assert_eq!(store.worktree(&initial.session).unwrap().unwrap(), current);
        assert_eq!(
            store.workspace(&initial.session).unwrap().root,
            Some(initial.root.clone())
        );
        {
            let conn = store.lock().unwrap();
            let rows: i64 = conn
                .query_row("SELECT count(*) FROM quota_pressure", [], |r| r.get(0))
                .unwrap();
            assert_eq!(rows, 0);
            conn.execute_batch("PRAGMA max_page_count=10000; DROP TRIGGER exhaust_archive_quota;")
                .unwrap();
        }
        // Explicit retry after freeing quota persists the same reviewed intent.
        store.save_worktree(&intent).unwrap();
        drop(store);
        let reopened = SqliteStore::open(&path).unwrap();
        reopened.recover_worktrees().unwrap();
        let recovered = reopened.worktree(&initial.session).unwrap().unwrap();
        assert_eq!(recovered.recovery_from, Some(WorktreeState::Archiving));
        assert_eq!(recovered.snapshot, intent.snapshot);
        assert_eq!(
            reopened.workspace(&initial.session).unwrap().root,
            Some(initial.root)
        );
        reopened.recover_worktrees().unwrap();
        assert_eq!(
            reopened.worktree(&initial.session).unwrap().unwrap(),
            recovered
        );
    }

    #[test]
    fn worktree_listing_pages_without_losing_deleted_chat_records() {
        let dir = tempfile::tempdir().unwrap();
        let store = SqliteStore::open(&dir.path().join("state.db")).unwrap();
        let mut ids = vec![];
        for _ in 0..WORKTREE_PAGE_SIZE + 2 {
            let value = record(dir.path());
            store.create_worktree(&value).unwrap();
            store.delete(&value.session).unwrap();
            ids.push(value.id);
        }
        let first = store.worktrees(None).unwrap();
        assert_eq!(first.items.len(), WORKTREE_PAGE_SIZE);
        let second = store.worktrees(first.next_cursor).unwrap();
        assert_eq!(second.items.len(), 2);
        assert!(second.next_cursor.is_none());
        let actual: Vec<_> = first
            .items
            .into_iter()
            .chain(second.items)
            .map(|r| r.id)
            .collect();
        ids.reverse();
        assert_eq!(actual, ids);
    }

    #[test]
    fn worktree_schema_44_migration_preserves_version_43_conversation_and_workspace() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("legacy.db");
        let original = dir.path().join("original").to_string_lossy().into_owned();
        {
            let store = SqliteStore::open(&path).unwrap();
            store
                .create_workspace_session(
                    "legacy",
                    &dolores_core::SessionWorkspace {
                        kind: dolores_core::WorkspaceKind::Project,
                        root: Some(original.clone()),
                    },
                )
                .unwrap();
            store
                .commit_turn("legacy", "Saved question", "Saved reply")
                .unwrap();
            store.save_draft("legacy", "Unsent draft").unwrap();
            store
                .lock()
                .unwrap()
                .execute_batch("DROP TABLE managed_worktrees; PRAGMA user_version=43;")
                .unwrap();
        }
        let store = SqliteStore::open(&path).unwrap();
        assert_eq!(store.workspace("legacy").unwrap().root, Some(original));
        assert_eq!(store.messages("legacy").unwrap()[1].content, "Saved reply");
        assert_eq!(store.saved_draft("legacy").unwrap(), "Unsent draft");
        assert!(store.worktrees(None).unwrap().items.is_empty());
        let version: i64 = store
            .lock()
            .unwrap()
            .query_row("PRAGMA user_version", [], |r| r.get(0))
            .unwrap();
        assert_eq!(version, 44);
    }
}
