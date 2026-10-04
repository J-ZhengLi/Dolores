use super::*;

#[test]
fn timestamps_are_atomic_persistent_exported_and_preserved_in_forks() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("timestamps.db");
    let store = SqliteStore::open(&path).unwrap();
    store.create("chat").unwrap();
    let before = now();
    store
        .commit_turn("chat", "question 世界", "answer")
        .unwrap();
    let page = store.messages_page("chat", None, false, 80).unwrap();
    let saved = page.items[0].saved_at.unwrap();
    assert!(saved >= before && saved <= now());
    assert_eq!(page.items[1].saved_at, Some(saved));
    let through = page.items[1].id;
    store.fork_session("chat", through, "fork").unwrap();
    let fork = store.messages_page("fork", None, false, 80).unwrap();
    assert!(fork.items.iter().all(|m| m.saved_at == Some(saved)));
    let mut export = vec![];
    store
        .export_conversation("chat", dolores_core::ExportFormat::Json, &mut export)
        .unwrap();
    let export: serde_json::Value = serde_json::from_slice(&export).unwrap();
    assert_eq!(export["messages"][0]["savedAt"], saved);
    store.lock().unwrap().execute_batch("CREATE TRIGGER reject_timestamp BEFORE INSERT ON message_timestamps BEGIN SELECT RAISE(ABORT,'fixture'); END;").unwrap();
    assert!(store.commit_turn("chat", "refused", "not saved").is_err());
    assert_eq!(store.messages("chat").unwrap().len(), 2);
    assert_eq!(
        store.messages_page("chat", None, false, 80).unwrap().items[0].saved_at,
        Some(saved)
    );
    drop(store);
    let store = SqliteStore::open(&path).unwrap();
    assert_eq!(
        store.messages_page("chat", None, false, 80).unwrap().items[1].saved_at,
        Some(saved)
    );
    store.delete("chat").unwrap();
    let c = store.lock().unwrap();
    assert_eq!(
        c.query_row::<i64, _, _>("SELECT COUNT(*) FROM message_timestamps", [], |r| r.get(0))
            .unwrap(),
        2
    );
}

#[test]
fn migration_does_not_fabricate_legacy_dates_or_change_message_bytes() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("legacy.db");
    let store = SqliteStore::open(&path).unwrap();
    store.create("legacy").unwrap();
    store
        .commit_turn("legacy", "old\r\nquestion", "old answer")
        .unwrap();
    let before = store.messages("legacy").unwrap();
    store
        .lock()
        .unwrap()
        .execute_batch("DROP TABLE message_timestamps; PRAGMA user_version=24;")
        .unwrap();
    drop(store);
    let store = SqliteStore::open(&path).unwrap();
    assert_eq!(store.messages("legacy").unwrap(), before);
    let page = store.messages_page("legacy", None, false, 80).unwrap();
    assert!(page.items.iter().all(|m| m.saved_at.is_none()));
    assert!(serde_json::to_value(&page.items[0])
        .unwrap()
        .get("savedAt")
        .is_none());
    store
        .fork_session("legacy", page.items[1].id, "legacy-fork")
        .unwrap();
    assert!(store
        .messages_page("legacy-fork", None, false, 80)
        .unwrap()
        .items
        .iter()
        .all(|m| m.saved_at.is_none()));
    store.commit_turn("legacy", "new", "new answer").unwrap();
    let page = store.messages_page("legacy", None, false, 80).unwrap();
    assert!(page.items[..2].iter().all(|m| m.saved_at.is_none()));
    assert!(page.items[2..].iter().all(|m| m.saved_at.is_some()));
}
