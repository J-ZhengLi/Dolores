use super::*;
use dolores_core::*;
use std::sync::Arc;

fn capture(store: &dyn SessionStore, session: &str, text: &str) -> MemoryPreference {
    store
        .commit_turn(session, text, "Retained completed answer")
        .unwrap();
    let source = store.memory_source_messages(session).unwrap().items[0].clone();
    let policy = store.automatic_memory_policy().unwrap();
    store
        .claim_automatic_memory(session, &source, policy.revision)
        .unwrap();
    let existing = memory::preferences_for_session(store, Some(session)).unwrap();
    let update = AutomaticMemoryUpdate {
        session: session.into(),
        source: source.clone(),
        policy_revision: policy.revision,
        existing,
        candidates: vec![MemorySuggestion {
            title: "Fact: project codename".into(),
            text: text.into(),
            message_id: source.message_id,
            quote: text.into(),
        }],
        model: "fixture".into(),
        status: "completed".into(),
        note: String::new(),
        usage: None,
    };
    assert_eq!(
        store
            .finish_automatic_memory(&update, &CancellationToken::new())
            .unwrap()
            .saved,
        1
    );
    memory::preferences_for_session(store, Some(session))
        .unwrap()
        .into_iter()
        .find(|p| p.text == text)
        .unwrap()
}

#[test]
fn cross_chat_recall_source_open_and_missing_evidence_never_mix_projects() {
    let a = tempfile::tempdir().unwrap();
    let b = tempfile::tempdir().unwrap();
    let store =
        Arc::new(dolores_store_sqlite::SqliteStore::open(&a.path().join("memory.db")).unwrap());
    store.set_automatic_memory_policy(true, 1).unwrap();
    for (id, path) in [("a", a.path()), ("a-next", a.path()), ("b", b.path())] {
        store
            .create_workspace_session(
                id,
                &SessionWorkspace {
                    kind: WorkspaceKind::Project,
                    root: Some(path.to_str().unwrap().into()),
                },
            )
            .unwrap();
    }
    let cedar = capture(store.as_ref(), "a", "Our project codename is Cedar.");
    capture(store.as_ref(), "b", "Our project codename is Maple.");
    let engine = Engine::new(
        store.clone(),
        Arc::new(connection::testing::MemoryCredentials::default()),
    )
    .unwrap();
    let context = |session: &str| {
        engine
            .call(Command::Context {
                session: Some(session.into()),
                input: "What is our project codename?".into(),
                tools: false,
            })
            .unwrap()
    };
    let next = context("a-next");
    assert!(next["messages"][0]["content"]
        .as_str()
        .unwrap()
        .contains("Cedar"));
    assert!(!next.to_string().contains("Maple"));
    let other = context("b");
    assert!(other.to_string().contains("Maple"));
    assert!(!other.to_string().contains("Cedar"));
    let source = engine
        .memory_evidence(Some("a-next"), &cedar.id, cedar.revision)
        .unwrap();
    assert_eq!(source["text"], "Our project codename is Cedar.");
    assert!(engine
        .memory_evidence(Some("b"), &cedar.id, cedar.revision)
        .is_err());
    store.delete("a").unwrap();
    assert!(engine
        .memory_evidence(Some("a-next"), &cedar.id, cedar.revision)
        .unwrap_err()
        .contains("unavailable"));
    assert!(!context("a-next").to_string().contains("Cedar"));
    assert_eq!(
        engine.memories(Some("a-next")).unwrap()["items"][0]["originAvailable"],
        false
    );
}
