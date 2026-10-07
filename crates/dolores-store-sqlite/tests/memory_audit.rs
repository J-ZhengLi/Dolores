use dolores_core::{automatic_source_allowed, parse_automatic_memories, SessionStore};
use dolores_store_sqlite::SqliteStore;

#[test]
fn useful_decision_is_now_eligible_and_disabled_differs_from_malformed() {
    let temp = tempfile::tempdir().unwrap();
    let store = SqliteStore::open(&temp.path().join("audit.db")).unwrap();
    store.create("audit").unwrap();
    store
        .commit_turn(
            "audit",
            "We decided to use SQLite for the local task index.",
            "Acknowledged.",
        )
        .unwrap();
    let source = store.memory_source_messages("audit").unwrap().items[0].clone();
    assert!(automatic_source_allowed(&source.text));
    assert_eq!(store.messages("audit").unwrap().len(), 2);
    let policy = store.automatic_memory_policy().unwrap();
    let off = store
        .set_automatic_memory_policy(false, policy.revision)
        .unwrap();
    assert!(!store
        .claim_automatic_memory("audit", &source, off.revision)
        .unwrap());
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
    store
        .commit_turn("migration", "Useful original history", "Retained reply")
        .unwrap();
    let on = store.set_automatic_memory_policy(true, 1).unwrap();
    drop(store);
    let store = SqliteStore::open(&path).unwrap();
    assert_eq!(store.automatic_memory_policy().unwrap(), on);
    let off = store
        .set_automatic_memory_policy(false, on.revision)
        .unwrap();
    drop(store);
    let store = SqliteStore::open(&path).unwrap();
    assert_eq!(store.automatic_memory_policy().unwrap(), off);
    assert_eq!(store.messages("migration").unwrap().len(), 2);
    let conn = rusqlite::Connection::open(&path).unwrap();
    let count: i64 = conn
        .query_row("SELECT count(*) FROM memory_source_index", [], |r| r.get(0))
        .unwrap();
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
    conn.execute_batch("PRAGMA user_version=33; DROP TABLE memory_source_index; CREATE TABLE memory_source_index(incompatible INTEGER);").unwrap();
    drop(conn);
    assert!(SqliteStore::open(&path).is_err());
    let conn = rusqlite::Connection::open(&path).unwrap();
    assert_eq!(
        conn.query_row("PRAGMA user_version", [], |r| r.get::<_, i64>(0))
            .unwrap(),
        33
    );
    assert_eq!(
        conn.query_row("SELECT count(*) FROM messages", [], |r| r.get::<_, i64>(0))
            .unwrap(),
        2
    );
}

fn proposal(store:&SqliteStore, text:&str, title:&str)->dolores_core::AutomaticMemoryUpdate {
    store.commit_turn("source",text,"Completed reply retained").unwrap();
    let source=store.memory_source_messages("source").unwrap().items[0].clone();
    let policy=store.automatic_memory_policy().unwrap();
    assert!(store.claim_automatic_memory("source",&source,policy.revision).unwrap());
    dolores_core::AutomaticMemoryUpdate{session:"source".into(),source:source.clone(),policy_revision:policy.revision,existing:store.memory_preferences(None).unwrap(),candidates:vec![dolores_core::MemorySuggestion{title:title.into(),text:text.into(),quote:text.into(),message_id:source.message_id}],model:"fixture".into(),status:"completed".into(),note:String::new(),usage:None}
}

#[test]
fn memory_correction_history_forget_pending_and_replay_are_atomic_and_persistent() {
    let temp=tempfile::tempdir().unwrap();let path=temp.path().join("retention.db");
    let store=SqliteStore::open(&path).unwrap();store.create("source").unwrap();
    store.set_automatic_memory_policy(true,1).unwrap();
    let cancel=tokio_util::sync::CancellationToken::new();
    let original=proposal(&store,"Our project codename is Cedar.","Fact: project codename");
    assert_eq!(store.finish_automatic_memory(&original,&cancel).unwrap().saved,1);
    let before=store.memory_preferences(None).unwrap()[0].clone();
    let corrected=proposal(&store,"Actually, our project codename is Birch instead.","Fact: project codename");
    assert_eq!(store.finish_automatic_memory(&corrected,&cancel).unwrap().saved,1);
    let current=store.memory_preferences(None).unwrap()[0].clone();
    assert_eq!(current.id,before.id);assert_eq!(current.revision,2);
    assert_eq!(store.memory_versions(&current.id).unwrap()[0],before);
    let pending=proposal(&store,"Actually, our project codename is Fir instead.","Fact: project codename");
    store.delete_memory_preference(None,&current.id,current.revision).unwrap();
    assert_eq!(store.finish_automatic_memory(&pending,&cancel).unwrap().status,"changed");
    assert!(store.memory_preferences(None).unwrap().is_empty());
    assert!(store.memory_versions(&current.id).unwrap().is_empty());
    drop(store);
    let conn=rusqlite::Connection::open(&path).unwrap();
    assert_eq!(conn.query_row("SELECT count(*) FROM memory_source_index",[],|r|r.get::<_,i64>(0)).unwrap(),0);
    // Even an explicit future catch-up with cleared attempt state cannot replay
    // a forgotten retained source. This mutation belongs only to this fixture.
    conn.execute("DELETE FROM automatic_memory_attempts",[]).unwrap();drop(conn);
    let store=SqliteStore::open(&path).unwrap();
    assert!(!store.claim_automatic_memory("source",&pending.source,2).unwrap());
    assert_eq!(store.messages("source").unwrap().len(),6);
    let fresh=proposal(&store,"Our project codename is Maple.","Fact: project codename");
    assert_eq!(store.finish_automatic_memory(&fresh,&cancel).unwrap().saved,1);
}

#[test]
fn memory_open_work_resolution_and_manual_correction_take_priority() {
    let temp=tempfile::tempdir().unwrap();let store=SqliteStore::open(&temp.path().join("work.db")).unwrap();
    store.create("source").unwrap();store.set_automatic_memory_policy(true,1).unwrap();
    let cancel=tokio_util::sync::CancellationToken::new();
    let open=proposal(&store,"We still need to document the recovery flow.","Open work: recovery flow");
    assert_eq!(store.finish_automatic_memory(&open,&cancel).unwrap().saved,1);
    let done=proposal(&store,"The recovery flow documentation is completed.","Outcome: recovery flow");
    assert_eq!(store.finish_automatic_memory(&done,&cancel).unwrap().saved,1);
    let current=store.memory_preferences(None).unwrap()[0].clone();
    assert!(current.title.starts_with("Outcome:"));assert_eq!(current.revision,2);
    store.save_memory_preference(None,&dolores_core::MemoryDraft{id:current.id.clone(),revision:Some(2),title:current.title.clone(),text:"The recovery documentation still needs a final user review.".into(),enabled:true,origin:None}).unwrap();
    let attempted=proposal(&store,"Actually, the recovery flow documentation is completed instead.","Outcome: recovery flow");
    assert_eq!(store.finish_automatic_memory(&attempted,&cancel).unwrap().saved,0);
    assert!(!store.memory_preferences(None).unwrap()[0].auto_update);
    assert!(store.memory_preferences(None).unwrap()[0].text.contains("user review"));
}
