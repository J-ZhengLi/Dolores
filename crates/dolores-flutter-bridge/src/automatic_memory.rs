use dolores_core::*;
use serde_json::{json, Value};
use std::sync::Arc;
use tokio_util::sync::CancellationToken;

pub(super) async fn learn(
    store: Arc<dyn SessionStore>,
    provider: Arc<dyn ModelProvider>,
    session: &str,
    model: &str,
    cancel: CancellationToken,
    output: &tokio::sync::mpsc::Sender<Value>,
    id: u64,
) -> Option<Value> {
    let reader = store.clone();
    let sid = session.to_owned();
    let snapshot = super::blocking(move || {
        let policy = reader.automatic_memory_policy()?;
        if !policy.enabled {
            return Ok(None);
        }
        let source = reader
            .memory_source_messages(&sid)?
            .items
            .into_iter()
            .next();
        let Some(source) = source else {
            return Ok(None);
        };
        let existing = super::memory::preferences_for_session(reader.as_ref(), Some(&sid))?;
        if !reader.claim_automatic_memory(&sid, &source, policy.revision)? {
            return Ok(None);
        }
        Ok(Some((policy, source, existing)))
    })
    .await
    .ok()
    .flatten()?;
    let (policy, source, existing) = snapshot;
    let mut update = AutomaticMemoryUpdate {
        session: session.into(),
        source: source.clone(),
        policy_revision: policy.revision,
        existing,
        candidates: vec![],
        model: model.into(),
        status: "skipped".into(),
        note: "No eligible explicit preference; no extra model request.".into(),
        usage: None,
    };
    let literal = if cancel.is_cancelled() {
        None
    } else {
        literal_response_preference(&source)
    };
    if let Some(candidate) = literal {
        update.candidates = vec![candidate];
        update.model = "local-excerpt".into();
        update.status = "completed".into();
        update.note = "Exact response preference extracted locally; no extra model request.".into();
    } else if automatic_source_allowed(&source.text) && !cancel.is_cancelled() {
        let learning_cancel = cancel.child_token();
        let result = async {
            let prompt = automatic_memory_prompt(&source, &update.existing)?;
            let (prompt, _) = prepare_token_context(
                prompt,
                &[],
                provider.context_window_tokens(),
                provider.request_settings().unwrap_or_default(),
            )?;
            let _ = output.send(json!({"type":"memoryUpdating","id":id})).await;
            let (answer, usage) = super::memory_suggestions::collect_review(
                provider,
                prompt,
                learning_cancel.clone(),
                "Automatic memory",
            )
            .await?;
            update.usage = usage;
            parse_automatic_memories(&answer, &source)
        };
        match tokio::time::timeout(std::time::Duration::from_secs(10), result).await {
            Ok(Ok(candidates)) => {
                update.candidates = candidates;
                update.status = "completed".into();
                update.note.clear();
            }
            _ => {
                learning_cancel.cancel();
                update.status = "failed".into();
                update.note="Learning did not produce a verified preference. The reply is saved; no automatic retry.".into();
            }
        }
    }
    let token = cancel.clone();
    let fallback = AutomaticMemoryAttempt {
        message_id: update.source.message_id,
        status: "failed".into(),
        note: "Memory could not be saved. The reply is saved; no automatic retry.".into(),
        updated_at: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as i64,
        saved: 0,
        skipped: 0,
        usage: update.usage.clone(),
    };
    match super::blocking(move || store.finish_automatic_memory(&update, &token)).await {
        Ok(report) => Some(json!(report)),
        Err(_) => Some(json!(fallback)),
    }
}
