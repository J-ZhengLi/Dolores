use super::*;
use dolores_core::{AttachmentData, AttachmentRef};

#[tokio::test]
async fn text_only_tool_requests_keep_the_original_byte_limit_before_http() {
    let provider = OpenAiProvider::new(
        &ConnectionPreferences {
            base_url: "http://127.0.0.1:1/v1".into(),
            model: "fixture".into(),
        },
        String::new(),
    )
    .unwrap();
    let messages = vec![dolores_core::AgentMessage {
        parts: vec![],
        role: "user".into(),
        content: "x".repeat(dolores_core::MAX_CONTEXT_BYTES),
        calls: vec![],
        call_id: None,
    }];
    let error = provider
        .tool_turn(&messages, &[], CancellationToken::new())
        .await
        .err()
        .unwrap();
    assert!(error.contains("128 KiB"));
    let (tx, _rx) = mpsc::channel(1);
    let error = provider
        .stream_tool_turn(&messages, &[], tx, CancellationToken::new())
        .await
        .err()
        .unwrap();
    assert!(error.contains("128 KiB"));
}

#[test]
fn image_wire_is_explicit_and_text_wire_remains_compatible() {
    let provider = OpenAiProvider::new(
        &ConnectionPreferences {
            base_url: "http://localhost:1234/v1".into(),
            model: "fixture".into(),
        },
        String::new(),
    )
    .unwrap();
    assert_eq!(
        provider.wire_content("legacy text", &[]).unwrap(),
        json!("legacy text")
    );
    let part = AttachmentRef {
        digest: "a".repeat(64),
        name: "picture.png".into(),
        mime: "image/png".into(),
        bytes: 3,
    };
    assert!(provider
        .wire_content("look", std::slice::from_ref(&part))
        .unwrap_err()
        .contains("disabled"));
    assert!(provider
        .with_attachment_assets(
            vec![AttachmentData {
                reference: part.clone(),
                data: vec![1, 2, 3],
            }],
            true,
        )
        .is_ok());
    let mut enabled = provider.clone();
    enabled.images = true;
    enabled.assets = std::sync::Arc::new(std::collections::BTreeMap::from([(
        part.digest.clone(),
        AttachmentData {
            reference: part.clone(),
            data: vec![1, 2, 3],
        },
    )]));
    let wire = enabled
        .wire_content("look", std::slice::from_ref(&part))
        .unwrap();
    assert_eq!(wire[0], json!({"type":"text","text":"look"}));
    assert_eq!(
        wire[1],
        json!({"type":"image_url","image_url":{"url":"data:image/png;base64,AQID","detail":"low"}})
    );
    assert!(!wire.to_string().contains("picture.png"));
    let mut missing = part;
    missing.digest = "b".repeat(64);
    assert!(enabled
        .wire_content("look", &[missing])
        .unwrap_err()
        .contains("Reattach"));
}

#[test]
fn image_rejection_names_recovery_without_exposing_provider_body() {
    let error = check_generation_rejection(&json!({"error":{"param":"messages.0.content.image_url","message":"This model does not support image input. PRIVATE"}})).unwrap_err();
    assert!(error.contains("Choose a capable model"));
    assert!(error.contains("your draft remains"));
    assert!(!error.contains("PRIVATE"));
    assert!(check_generation_rejection(&json!({"error":{"message":"Unknown model"}})).is_ok());
}
