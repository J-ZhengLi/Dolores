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
