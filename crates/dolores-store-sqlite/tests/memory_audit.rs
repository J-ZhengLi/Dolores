use dolores_core::{automatic_source_allowed, parse_automatic_memories, SessionStore};
use dolores_store_sqlite::SqliteStore;

#[test]
fn baseline_missed_decision_is_retained_and_disabled_differs_from_malformed() {
    let temp = tempfile::tempdir().unwrap();
    let store = SqliteStore::open(&temp.path().join("audit.db")).unwrap();
    store.create("audit").unwrap();
    store.commit_turn("audit", "We decided to use SQLite for the local task index.", "Acknowledged.").unwrap();
    let source = store.memory_source_messages("audit").unwrap().items[0].clone();
    assert!(!automatic_source_allowed(&source.text));
    assert_eq!(store.messages("audit").unwrap().len(), 2);
    let policy = store.automatic_memory_policy().unwrap();
    let off = store.set_automatic_memory_policy(false, policy.revision).unwrap();
    assert!(!store.claim_automatic_memory("audit", &source, off.revision).unwrap());
    assert!(store.automatic_memory_attempt("audit").unwrap().is_none());
    assert!(parse_automatic_memories("malformed", &source).is_err());
}

#[test]
fn master_choice_and_history_survive_restart_and_atomic_source_index_migration() {
    let temp = tempfile::tempdir().unwrap();
    let path = temp.path().join("migration.db");
    let store = SqliteStore::open(&path).unwrap();
    assert!(!store.automatic_memory_policy().unwrap().enabled);
    store.create("migration").unwrap();
    store.commit_turn("migration", "Useful original history", "Retained reply").unwrap();
    let on = store.set_automatic_memory_policy(true, 1).unwrap();
    drop(store);
    let store = SqliteStore::open(&path).unwrap();
    assert_eq!(store.automatic_memory_policy().unwrap(), on);
    let off = store.set_automatic_memory_policy(false, on.revision).unwrap();
    drop(store);
    let store = SqliteStore::open(&path).unwrap();
    assert_eq!(store.automatic_memory_policy().unwrap(), off);
    assert_eq!(store.messages("migration").unwrap().len(), 2);
    let conn = rusqlite::Connection::open(&path).unwrap();
    let count: i64 = conn.query_row("SELECT count(*) FROM memory_source_index", [], |r| r.get(0)).unwrap();
    assert_eq!(count, 0);
}

#[test]
fn migration_failure_keeps_previous_schema_and_conversation() {
    let temp = tempfile::tempdir().unwrap();
    let path = temp.path().join("failure.db");
    let store = SqliteStore::open(&path).unwrap();
    store.create("retained").unwrap();
    store.commit_turn("retained", "Original", "Reply").unwrap();
    drop(store);
    let conn = rusqlite::Connection::open(&path).unwrap();
    // An occupied destination forces migration to fail inside its transaction.
    conn.execute_batch("PRAGMA user_version=33;").unwrap();
    drop(conn);
    assert!(SqliteStore::open(&path).is_err());
    let conn = rusqlite::Connection::open(&path).unwrap();
    assert_eq!(conn.query_row("PRAGMA user_version", [], |r| r.get::<_,i64>(0)).unwrap(), 33);
    assert_eq!(conn.query_row("SELECT count(*) FROM messages", [], |r| r.get::<_,i64>(0)).unwrap(), 2);
}
