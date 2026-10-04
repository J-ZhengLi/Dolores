use super::checkpoints::*;
use dolores_core::{RunSnapshot, RunState, SessionStore};
use dolores_store_sqlite::SqliteStore;
use serde_json::json;
#[test]
fn restart_lineage_and_reused_call_ids_keep_uncertainty_and_goal() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("state.db");
    let store = SqliteStore::open(&path).unwrap();
    store.create("chat").unwrap();
    store.create("other").unwrap();
    let original = RunSnapshot {
        id: "11111111-1111-4111-8111-111111111111".into(),
        thread: "chat".into(),
        model: "fixture".into(),
        input: "Repair the existing assertion".into(),
        settings: Default::default(),
        state: RunState::Prepared,
        sequence: 0,
        created_at: 0,
        build: "fixture".into(),
        tools: vec![],
        extensions: vec![],
        effective_settings: None,
        parent_run: None,
        segments: 1,
    };
    store.begin_run(&original).unwrap();
    store
        .append_run_event(
            &original.id,
            0,
            Some(RunState::Running),
            "started",
            &json!({}),
        )
        .unwrap();
    store
        .append_run_event(
            &original.id,
            1,
            None,
            "toolIntent",
            &json!({"callId":"same","name":"edit_text_file","target":"large.py"}),
        )
        .unwrap();
    store.save_draft("chat", "unfinished draft").unwrap();
    drop(store);
    let store = SqliteStore::open(&path).unwrap();
    store.interrupt_runs().unwrap();
    let prompt = resume_prompt(&store, "chat", &original.id, true).unwrap();
    assert!(prompt.contains("large.py") && prompt.contains("Repair the existing assertion"));
    assert!(resume_prompt(&store, "other", &original.id, true).is_err());
    let mut child = original.clone();
    child.id = "22222222-2222-4222-8222-222222222222".into();
    child.parent_run = Some(original.id.clone());
    child.input = "resume request".into();
    child.segments = 2;
    store.begin_run(&child).unwrap();
    store
        .append_run_event(&child.id, 0, Some(RunState::Running), "started", &json!({}))
        .unwrap();
    store
        .append_run_event(
            &child.id,
            1,
            None,
            "toolResult",
            &json!({"callId":"same","name":"read_text_file","returned":true,"content":"read"}),
        )
        .unwrap();
    store
        .append_run_event(
            &child.id,
            2,
            Some(RunState::Failed),
            "finished",
            &json!({"message":"fixture"}),
        )
        .unwrap();
    let report = view(&store, "chat", &child.id).unwrap();
    assert_eq!(report["goal"], original.input);
    assert_eq!(report["uncertainEffects"].as_array().unwrap().len(), 1);
    assert_eq!(report["savedDraft"], "unfinished draft");
    assert!(resume_prompt(&store, "chat", &original.id, true).is_err());
}
