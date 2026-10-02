use super::*;
use dolores_core::{ChangeDraft, ExportFormat, SessionWorkspace, WorkspaceKind};

fn draft(root: &str) -> ChangeDraft {
    ChangeDraft {
        root: root.into(),
        session: "chat".into(),
        target: "note".into(),
        before: "before 世界".into(),
        after: "after 世界".into(),
        before_exists: true,
        after_exists: true,
        reverts: None,
    }
}
fn bind(store: &SqliteStore, root: &str) {
    store
        .create_workspace_session(
            "chat",
            &SessionWorkspace {
                kind: WorkspaceKind::Project,
                root: Some(root.into()),
            },
        )
        .unwrap();
}
#[test]
fn migration_restart_paging_chat_delete_and_export_keep_local_snapshots_separate() {
    let dir = tempfile::tempdir().unwrap();
    let db = dir.path().join("journal.db");
    let root = dir.path().to_string_lossy().into_owned();
    let store = SqliteStore::open(&db).unwrap();
    bind(&store, &root);
    for _ in 0..23 {
        let id = store.begin_change(&draft(&root)).unwrap();
        store.finish_change(id, true).unwrap();
    }
    let page = store.changes_page(&root, None).unwrap();
    assert_eq!(page.items.len(), 20);
    assert!(page.has_older);
    assert!(page.items[0].id > page.items[19].id);
    assert_eq!(page.items[0].bytes_before, "before 世界".len());
    assert_eq!(
        store
            .changes_page(&root, Some(page.items[19].id))
            .unwrap()
            .items
            .len(),
        3
    );
    assert!(store.changes_page("other", None).unwrap().items.is_empty());
    store.commit_turn("chat", "hello", "reply").unwrap();
    for format in [ExportFormat::Json, ExportFormat::Markdown] {
        let mut data = vec![];
        store
            .export_conversation("chat", format, &mut data)
            .unwrap();
        let text = String::from_utf8(data).unwrap();
        assert!(!text.contains("before 世界") && !text.contains(&root));
    }
    store.delete("chat").unwrap();
    drop(store);
    let store = SqliteStore::open(&db).unwrap();
    assert_eq!(store.changes_page(&root, None).unwrap().items.len(), 20);
    assert_eq!(
        store.change_snapshot(page.items[0].id).unwrap().before,
        "before 世界"
    );
    assert!(store.begin_change(&draft(&root)).is_err());
}
#[test]
fn revert_receipts_are_atomic_bound_and_rollback_to_pending_on_failure() {
    let store = SqliteStore::open(Path::new(":memory:")).unwrap();
    bind(&store, "project");
    let original = store.begin_change(&draft("project")).unwrap();
    // A pending intent may be reverted after the host verifies its applied bytes.
    let mut revert = draft("project");
    revert.reverts = Some(original);
    std::mem::swap(&mut revert.before, &mut revert.after);
    let id = store.begin_change(&revert).unwrap();
    store.lock().unwrap().execute_batch("CREATE TRIGGER fail_revert BEFORE UPDATE ON file_changes WHEN NEW.status='reverted' BEGIN SELECT RAISE(ABORT,'fixture'); END").unwrap();
    assert!(store.finish_change(id, true).is_err());
    assert_eq!(store.change_snapshot(id).unwrap().change.status, "pending");
    assert_eq!(
        store.change_snapshot(original).unwrap().change.status,
        "pending"
    );
    store
        .lock()
        .unwrap()
        .execute_batch("DROP TRIGGER fail_revert")
        .unwrap();
    store.finish_change(id, true).unwrap();
    assert_eq!(
        store.change_snapshot(original).unwrap().change.status,
        "reverted"
    );
    assert!(store.finish_change(id, true).is_err());
    assert!(store.begin_change(&revert).is_err());
    let mut invalid = draft("elsewhere");
    assert!(store.begin_change(&invalid).is_err());
    invalid.root = "project".into();
    invalid.after = "x".repeat(dolores_core::MAX_TOOL_BYTES + 1);
    assert!(store.begin_change(&invalid).is_err());
}

#[test]
fn schema_seven_migrates_existing_edits_and_empty_creation_removal_survives_restart() {
    let dir = tempfile::tempdir().unwrap();
    let db = dir.path().join("journal.db");
    let legacy = Connection::open(&db).unwrap();
    legacy.execute_batch("CREATE TABLE file_changes (id INTEGER PRIMARY KEY, root TEXT NOT NULL, session_id TEXT NOT NULL, target TEXT NOT NULL, created_at INTEGER NOT NULL, status TEXT NOT NULL, reverts INTEGER REFERENCES file_changes(id), before_text TEXT NOT NULL, after_text TEXT NOT NULL);
        INSERT INTO file_changes VALUES(1,'project','chat','old',1,'applied',NULL,'before','after'); PRAGMA user_version=7;").unwrap();
    drop(legacy);
    let store = SqliteStore::open(&db).unwrap();
    let legacy = store.change_snapshot(1).unwrap();
    assert!(legacy.change.before_exists && legacy.change.after_exists);
    assert_eq!(
        (legacy.before.as_str(), legacy.after.as_str()),
        ("before", "after")
    );
    bind(&store, "project");
    let mut creation = draft("project");
    creation.target = "empty".into();
    creation.before.clear();
    creation.after.clear();
    creation.before_exists = false;
    let original = store.begin_change(&creation).unwrap();
    store.finish_change(original, true).unwrap();
    let mut removal = creation;
    removal.before_exists = true;
    removal.after_exists = false;
    assert!(store.begin_change(&removal).is_err()); // Arbitrary deletion is unsupported.
    removal.reverts = Some(original);
    let reverse = store.begin_change(&removal).unwrap();
    store.finish_change(reverse, true).unwrap();
    drop(store);
    let store = SqliteStore::open(&db).unwrap();
    let created = store.change_snapshot(original).unwrap().change;
    assert_eq!(created.status, "reverted");
    assert!(!created.before_exists && created.after_exists);
    let removed = store.change_snapshot(reverse).unwrap().change;
    assert!(removed.before_exists && !removed.after_exists);
    assert_eq!((removed.bytes_before, removed.bytes_after), (0, 0));
    assert!(store.begin_change(&removal).is_err());
    store.delete("chat").unwrap();
    assert_eq!(store.changes_page("project", None).unwrap().items.len(), 3);
}
