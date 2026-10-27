use super::*;
use dolores_core::{ContextSummary, ExportFormat, SessionStore, TokenUsage, TurnMetadata};
use serde_json::json;

fn usage(input: u64) -> TokenUsage {
    TokenUsage {
        input_tokens: Some(input),
        output_tokens: Some(input / 10),
        total_tokens: Some(input + input / 10),
        cached_input_tokens: Some(input / 2),
        reasoning_tokens: Some(input / 20),
    }
}

#[test]
fn export_reconstructs_segmented_usage_and_full_evidence_without_original_database() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("export.db");
    let store = SqliteStore::open(&path).unwrap();
    store.create("chat").unwrap();
    let mut r = crate::run_segment_tests::run();
    r.id = "11111111-1111-4111-8111-111111111111".into();
    store.begin_run(&r).unwrap();
    let mut sequence = store
        .append_run_event(
            &r.id,
            0,
            Some(dolores_core::RunState::Running),
            "started",
            &json!({}),
        )
        .unwrap();
    for (n, values) in [vec![Some(usage(100)), None], vec![Some(usage(200))]]
        .into_iter()
        .enumerate()
    {
        let body=json!({"modelCalls":n+2,"usageByCall":values,"tools":[{"content":"Complete archived result 世界"}],"toolSteps":[],"steps":[],"thinking":[]}).to_string();
        let split = 37;
        let fragments = vec![
            json!({"encoding":"agentSummaryJsonFragment","offset":0,"totalBytes":body.len(),"text":&body[..split]}),
            json!({"encoding":"agentSummaryJsonFragment","offset":split,"totalBytes":body.len(),"text":&body[split..]}),
        ];
        sequence = store
            .seal_run_segment(&r.id, sequence, &fragments, &json!({"kind":"execution"}))
            .unwrap()
            .0;
    }
    let metadata=TurnMetadata{paused:None,model:"fixture".into(),usage:Some(usage(999)),request_settings:None,context:ContextSummary::from_messages(&dolores_core::preview_context(vec![],"Task").unwrap(),Some(0)),agent:Some(serde_json::from_value(json!({"archive":{"runId":r.id,"throughSegment":2,"toolCount":2,"modelCalls":3},"modelCalls":5,"usageByCall":[usage(300),null],"tools":[],"toolSteps":[],"steps":[],"thinking":[]})).unwrap())};
    store
        .commit_turn_metadata("chat", "Task", "Answer", &metadata)
        .unwrap();
    store.create("other-chat").unwrap();
    let mut other = r.clone();
    other.id = "22222222-2222-4222-8222-222222222222".into();
    other.thread = "other-chat".into();
    store.begin_run(&other).unwrap();
    store
        .append_run_event(
            &other.id,
            0,
            Some(dolores_core::RunState::Running),
            "privateOtherChat",
            &json!({"text":"OTHER_CHAT_MARKER"}),
        )
        .unwrap();
    let mut markdown = vec![];
    store
        .export_conversation("chat", ExportFormat::Markdown, &mut markdown)
        .unwrap();
    let markdown = String::from_utf8(markdown).unwrap();
    assert!(markdown.contains("Complete archived result 世界"));
    assert!(markdown.contains("knownUsage"));
    assert!(!markdown.contains("OTHER_CHAT_MARKER"));
    let mut bytes = vec![];
    store
        .export_conversation("chat", ExportFormat::Json, &mut bytes)
        .unwrap();
    drop(store);
    std::fs::remove_file(path).unwrap();
    let export: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(export["runs"].as_array().unwrap().len(), 1);
    assert_eq!(export["usageSummary"]["modelAttempts"], 5);
    assert_eq!(export["usageSummary"]["reportedCalls"], 3);
    assert_eq!(export["usageSummary"]["missingUsageCalls"], 2);
    assert_eq!(export["usageSummary"]["knownUsage"]["totalTokens"], 660);
    assert_eq!(
        export["usageSummary"]["knownUsage"]["cachedInputTokens"],
        300
    );
    let events = export["runs"][0]["events"].as_array().unwrap();
    let mut rebuilt = String::new();
    let mut total = 0;
    for event in events {
        let d = &event["data"];
        if d["encoding"] == "agentSummaryJsonFragment" {
            if d["offset"] == 0 {
                rebuilt.clear();
            }
            rebuilt.push_str(d["text"].as_str().unwrap());
            if rebuilt.len() == d["totalBytes"].as_u64().unwrap() as usize {
                let s: serde_json::Value = serde_json::from_str(&rebuilt).unwrap();
                assert!(s["tools"][0]["content"].as_str().unwrap().contains("世界"));
                total += s["usageByCall"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .filter_map(|u| u["totalTokens"].as_u64())
                    .sum::<u64>();
            }
        }
    }
    assert_eq!(total, 330);
}
