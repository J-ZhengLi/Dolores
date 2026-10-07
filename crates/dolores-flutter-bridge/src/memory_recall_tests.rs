use super::*;
use dolores_core::*;
use std::sync::Arc;

#[test]
fn shared_image_recall_is_scoped_uncertain_bounded_and_missing_safe() {
    use sha2::{Digest, Sha256};
    let dir = tempfile::tempdir().unwrap();
    let store =
        Arc::new(dolores_store_sqlite::SqliteStore::open(&dir.path().join("image.db")).unwrap());
    for id in ["source", "next"] {
        store.create(id).unwrap();
    }
    store.set_automatic_memory_policy(true, 1).unwrap();
    let data = b"public image fixture".to_vec();
    let asset = AttachmentRef {
        digest: format!("{:x}", Sha256::digest(&data)),
        name: "blue-square.png".into(),
        mime: "image/png".into(),
        bytes: data.len(),
    };
    store
        .add_attachment(
            "source",
            &AttachmentData {
                reference: asset.clone(),
                data,
            },
        )
        .unwrap();
    store
        .commit_turn(
            "source",
            "What is in this image?",
            "A blue square appears visible.",
        )
        .unwrap();
    let source = store.memory_source_messages("source").unwrap().items[0].clone();
    let policy = store.automatic_memory_policy().unwrap();
    assert!(store
        .claim_automatic_memory("source", &source, policy.revision)
        .unwrap());
    let mut update = AutomaticMemoryUpdate {
        session: "source".into(),
        source,
        policy_revision: policy.revision,
        existing: vec![],
        candidates: vec![],
        model: "fixture".into(),
        status: "completed".into(),
        note: String::new(),
        usage: None,
        image: Some(MemoryImageCaption {
            asset: asset.clone(),
            title: "Image: blue square".into(),
            description: "A blue square on a white background.".into(),
            uncertainty: "Simple shape; no identity or context inferred.".into(),
        }),
    };
    update.image.as_mut().unwrap().uncertainty.clear();
    assert!(store
        .finish_automatic_memory(&update, &CancellationToken::new())
        .is_err());
    assert!(store.memory_preferences(None).unwrap().is_empty());
    update.image.as_mut().unwrap().uncertainty = "Simple shape; other details are unknown.".into();
    assert_eq!(
        store
            .finish_automatic_memory(&update, &CancellationToken::new())
            .unwrap()
            .saved,
        1
    );
    let engine = Engine::new(
        store.clone(),
        Arc::new(connection::testing::MemoryCredentials::default()),
    )
    .unwrap();
    let record = store.memory_preferences(None).unwrap()[0].clone();
    let evidence = engine.memory_evidence(Some("next"), &record.id, 1).unwrap();
    assert!(evidence["imageBase64"].is_string());
    let mut prefs = store.preferences().unwrap();
    prefs.model = "fixture".into();
    store.save_preferences(&prefs).unwrap();
    store
        .save_image_models(&prefs.base_url, &["fixture".into()])
        .unwrap();
    let (mut context, mut report) = memory::prepare_recall(
        preview_context(vec![], "Which shared image shows the blue square?").unwrap(),
        memory::recall_for_session(store.as_ref(), Some("next")).unwrap(),
        &[],
        Some(8192),
        RequestSettings::default(),
    )
    .unwrap();
    let images = memory::recall_image(
        store.as_ref(),
        "next",
        &mut context,
        &mut report,
        &[],
        Some(8192),
        RequestSettings::default(),
        "fixture",
    )
    .unwrap();
    assert_eq!(images.len(), 1);
    assert_eq!(context.last().unwrap().parts, vec![asset.clone()]);
    assert!(context[0]
        .content
        .contains("model description is uncertain"));
    let (mut small, mut report) = memory::prepare_recall(
        preview_context(vec![], "blue square image").unwrap(),
        vec![record.clone()],
        &[],
        Some(4096),
        RequestSettings::default(),
    )
    .unwrap();
    assert!(memory::recall_image(
        store.as_ref(),
        "next",
        &mut small,
        &mut report,
        &[],
        Some(4096),
        RequestSettings::default(),
        "fixture"
    )
    .unwrap()
    .is_empty());
    assert!(small.last().unwrap().parts.is_empty());
    // A recalled asset must not turn an otherwise valid full-image request into
    // a 17-image / over-8-MiB adapter failure. Keep the caption and current input.
    for count in [4, 16] {
        let (mut full, mut full_report) = memory::prepare_recall(
            preview_context(vec![], "blue square image").unwrap(),
            vec![record.clone()],
            &[],
            None,
            RequestSettings::default(),
        )
        .unwrap();
        full.last_mut().unwrap().parts = (0..count)
            .map(|n| AttachmentRef {
                digest: format!("{n:064x}"),
                bytes: 2 * 1024 * 1024,
                name: format!("current-{n}.png"),
                mime: "image/png".into(),
            })
            .collect();
        let before = full.clone();
        assert!(memory::recall_image(
            store.as_ref(),
            "next",
            &mut full,
            &mut full_report,
            &[],
            None,
            RequestSettings::default(),
            "fixture"
        )
        .unwrap()
        .is_empty());
        assert_eq!(full, before);
        assert!(full_report.unwrap().note.contains("request limit"));
    }
    store.delete("source").unwrap();
    assert!(memory::recall_for_session(store.as_ref(), Some("next"))
        .unwrap()
        .is_empty());
    assert!(engine
        .memory_evidence(Some("next"), &record.id, 1)
        .unwrap_err()
        .contains("unavailable"));
    assert_eq!(
        engine.memories(Some("next")).unwrap()["items"][0]["originAvailable"],
        false
    );
    engine
        .delete_memory(Some("next"), MemoryScope::All, &record.id, 1)
        .unwrap();
    assert!(store.memory_preferences(None).unwrap().is_empty());
}

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
        image: None,
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
