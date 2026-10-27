use super::*;
use dolores_core::{RunSnapshot, RunState, SessionStore};
use serde_json::json;

fn run() -> RunSnapshot {
    RunSnapshot {
        parent_run: None,
        segments: 1,
        id: "11111111-1111-4111-8111-111111111111".into(),
        thread: "chat".into(),
        model: "fixture".into(),
        settings: Default::default(),
        input: "Keep working".into(),
        state: RunState::Prepared,
        sequence: 0,
        created_at: 0,
        build: "fixture".into(),
        tools: vec![],
        extensions: vec![],
        effective_settings: None,
    }
}

#[test]
fn segments_are_atomic_paged_exact_and_interrupted_after_restart() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("fixture.db");
    let store = SqliteStore::open(&path).unwrap();
    store.create("chat").unwrap();
    let r = run();
    store.begin_run(&r).unwrap();
    let seq = store
        .append_run_event(&r.id, 0, Some(RunState::Running), "started", &json!({}))
        .unwrap();
    let records: Vec<_> = (0..300)
        .map(|n| json!({"index":n,"text":"Complete receipt 世界. ".repeat(200)}))
        .collect();
    let (last, segment) = store
        .seal_run_segment(
            &r.id,
            seq,
            &records,
            &json!({"goal":"Original objective", "next":"Inspect remaining files"}),
        )
        .unwrap();
    assert_eq!(segment.through_sequence, last);
    assert_eq!(store.runs("chat").unwrap()[0].state, RunState::Running);
    assert!(store.seal_run_segment(&r.id, seq, &[], &json!({})).is_err());
    // Force a mid-transaction failure; neither inserted evidence nor ownership
    // can advance when the checkpoint save fails.
    store.lock().unwrap().execute_batch("CREATE TRIGGER reject_segment BEFORE INSERT ON run_segments BEGIN SELECT RAISE(ABORT,'fixture'); END;").unwrap();
    assert!(store
        .seal_run_segment(&r.id, last, &[json!({"mustRollback":true})], &json!({}))
        .is_err());
    assert_eq!(store.runs("chat").unwrap()[0].sequence, last);
    let mut cursor = 0;
    let mut items = vec![];
    loop {
        let page = store.run_events_page("chat", &r.id, cursor).unwrap();
        assert!(page.items.len() <= 128);
        items.extend(page.items);
        if let Some(next) = page.next {
            cursor = next;
        } else {
            break;
        }
    }
    assert_eq!(items.len(), 302);
    assert_eq!(items.last().unwrap().kind, "taskCheckpoint");
    for (n, event) in items.iter().skip(1).take(300).enumerate() {
        assert_eq!(event.data, records[n]);
    }
    assert!(store.run_events_page("another-chat", &r.id, 0).is_err());
    drop(store);
    let store = SqliteStore::open(&path).unwrap();
    store.interrupt_runs().unwrap();
    assert_eq!(store.runs("chat").unwrap()[0].state, RunState::Interrupted);
    assert!(store
        .seal_run_segment(&r.id, last + 1, &[], &json!({}))
        .is_err());
    assert_eq!(
        store.run_events_page("chat", &r.id, 0).unwrap().items[1].data,
        records[0]
    );
}
