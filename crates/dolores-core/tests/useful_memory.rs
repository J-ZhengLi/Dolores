use dolores_core::{automatic_source_allowed, parse_automatic_memories, MemoryMessage};

#[test]
fn frozen_useful_corpus_requires_exact_unambiguous_evidence() {
    let corpus: serde_json::Value =
        serde_json::from_str(include_str!("../../../docs/fixtures/memory-corpus.json")).unwrap();
    for item in corpus["eligible"].as_array().unwrap() {
        let text = item["text"].as_str().unwrap();
        let source = MemoryMessage {
            message_id: 7,
            text: text.into(),
        };
        let kind = match item["kind"].as_str().unwrap() {
            "fact" => "Fact",
            "decision" => "Decision",
            "outcome" => "Outcome",
            "open-work" => "Open work",
            _ => "Preference",
        };
        let answer = serde_json::json!({"suggestions":[{"title":format!("{kind}: subject"),"text":text,"quote":text,"messageId":7}]}).to_string();
        assert!(automatic_source_allowed(text), "{text}");
        assert_eq!(
            parse_automatic_memories(&answer, &source).unwrap().len(),
            1,
            "{text}"
        );
        let invented = answer.replace(text, "Our project codename is Invented.");
        assert!(parse_automatic_memories(&invented, &source).is_err());
    }
    for item in corpus["negative"].as_array().unwrap() {
        let text = item.as_str().unwrap();
        assert!(!automatic_source_allowed(text), "{text}");
        let source = MemoryMessage {
            message_id: 7,
            text: text.into(),
        };
        let answer=serde_json::json!({"suggestions":[{"title":"Fact: subject","text":text,"quote":text,"messageId":7}]}).to_string();
        assert!(parse_automatic_memories(&answer, &source).is_err());
    }
}

#[test]
fn one_invalid_candidate_refuses_whole_batch_and_partial_qualifiers() {
    let source = MemoryMessage {
        message_id: 1,
        text: "Our project codename is Cedar, only for the demo.".into(),
    };
    let partial = "Our project codename is Cedar";
    let candidate = |quote: &str| serde_json::json!({"title":"Fact: codename","text":quote,"quote":quote,"messageId":1});
    assert!(parse_automatic_memories(
        &serde_json::json!({"suggestions":[candidate(partial)]}).to_string(),
        &source
    )
    .is_err());
    let mut bad = candidate(&source.text);
    bad["messageId"] = serde_json::json!(2);
    assert!(parse_automatic_memories(
        &serde_json::json!({"suggestions":[candidate(&source.text),bad]}).to_string(),
        &source
    )
    .is_err());
}
