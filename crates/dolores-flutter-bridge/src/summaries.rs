use super::{Engine, Run};
use dolores_core::SummaryBatch;
use serde_json::{json, Value};
use std::{
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};
use tokio::sync::mpsc;
use tokio_util::sync::CancellationToken;

const STALE: &str = "Summary review expired or changed. Refresh and review again.";
pub(super) struct SummaryReview {
    token: String,
    session: String,
    batch: SummaryBatch,
    model: Option<String>,
    created: Instant,
}
fn fresh(
    store: &dyn dolores_core::SessionStore,
    session: &str,
    batch: &SummaryBatch,
) -> Result<(), String> {
    if store.review_summary_batch(session)? != *batch {
        return Err(STALE.into());
    }
    Ok(())
}
impl Engine {
    pub(super) fn clear_summary_review(&self) -> Result<(), String> {
        self.summary_review.lock().map_err(|_| STALE)?.take();
        Ok(())
    }
    pub(super) fn discard_summary_review(&self, token: &str) -> Result<Value, String> {
        let mut slot = self.summary_review.lock().map_err(|_| STALE)?;
        if slot.as_ref().is_some_and(|r| r.token == token) {
            slot.take();
        }
        Ok(Value::Null)
    }
    pub(super) fn review_summary(&self, session: &str) -> Result<Value, String> {
        self.clear_summary_review()?;
        let batch = self.store.review_summary_batch(session)?;
        let token = uuid::Uuid::new_v4().to_string();
        let result = json!({"token":token,"summary":batch.previous,"messages":batch.messages,"hasMore":batch.has_more,"model":self.store.preferences()?.model,"autoCompact":self.store.auto_compact(session)?});
        *self.summary_review.lock().map_err(|_| STALE)? = Some(SummaryReview {
            token,
            session: session.into(),
            batch,
            model: None,
            created: Instant::now(),
        });
        Ok(result)
    }
    pub(super) fn generate_summary(
        &self,
        active: &mut crate::run_journal::RunCoordinator,
        id: u64,
        session: String,
        token: String,
    ) -> Result<Value, String> {
        let review = self
            .summary_review
            .lock()
            .map_err(|_| STALE)?
            .take()
            .ok_or(STALE)?;
        if review.token != token
            || review.session != session
            || review.model.is_some()
            || review.created.elapsed() > Duration::from_secs(300)
        {
            return Err(STALE.into());
        }
        fresh(self.store.as_ref(), &session, &review.batch)?;
        let prompt = dolores_core::summary_prompt(&review.batch)?;
        let _entered = self.runtime.enter();
        let provider = self
            .connection
            .lock()
            .map_err(|_| "Connection unavailable.")?
            .review_provider()?;
        let model = self.store.preferences()?.model;
        let (prompt, tokens) = dolores_core::prepare_token_context(
            prompt,
            &[],
            provider.context_window_tokens(),
            provider.request_settings().unwrap_or_default(),
        )?;
        let cancel = CancellationToken::new();
        let (output, events) = mpsc::channel(4);
        active.reserve(Run {
            thread: None,
            id,
            cancel: cancel.clone(),
            events,
            approvals: Arc::new(Mutex::new(None)),
        })?;
        let store = self.store.clone();
        let slot = self.summary_review.clone();
        self.runtime.spawn(async move {
            let result = async {
                let (text, usage) = super::memory_suggestions::collect_review(
                    provider,
                    prompt,
                    cancel.clone(),
                    "Summary",
                )
                .await?;
                dolores_core::validate_summary(&text)?;
                let reader = store.clone();
                let check_session = session.clone();
                let batch = review.batch.clone();
                super::blocking(move || fresh(reader.as_ref(), &check_session, &batch)).await?;
                let token = uuid::Uuid::new_v4().to_string();
                let result =
                    json!({"token":token,"text":text,"model":model,"usage":usage,"tokens":tokens});
                let mut pending = slot.lock().map_err(|_| STALE)?;
                if cancel.is_cancelled() {
                    return Err("Summary stopped. Nothing was saved.".into());
                }
                *pending = Some(SummaryReview {
                    token,
                    session,
                    batch: review.batch,
                    model: Some(model),
                    created: Instant::now(),
                });
                Ok::<_, String>(result)
            }
            .await;
            let event = match result {
                Ok(summary) => json!({"type":"done","id":id,"summaryDraft":summary}),
                Err(error) => json!({"type":"done","id":id,"error":error}),
            };
            let _ = output.send(event).await;
        });
        Ok(Value::Null)
    }
    pub(super) fn save_summary(
        &self,
        session: &str,
        token: &str,
        text: &str,
    ) -> Result<Value, String> {
        let mut pending = self.summary_review.lock().map_err(|_| STALE)?;
        let review = pending.as_ref().ok_or(STALE)?;
        if review.token != token
            || review.session != session
            || review.created.elapsed() > Duration::from_secs(300)
        {
            return Err(STALE.into());
        }
        let model = review.model.as_deref().ok_or(STALE)?;
        let saved = self
            .store
            .save_session_summary(session, &review.batch, text, model)?;
        pending.take();
        Ok(json!(saved))
    }
    pub(super) fn correct_summary(
        &self,
        session: &str,
        revision: u32,
        text: &str,
    ) -> Result<Value, String> {
        self.clear_summary_review()?;
        Ok(json!(self
            .store
            .correct_session_summary(session, revision, text)?))
    }
    pub(super) fn delete_summary(&self, session: &str, revision: u32) -> Result<Value, String> {
        self.clear_summary_review()?;
        self.store.delete_session_summary(session, revision)?;
        Ok(Value::Null)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use dolores_core::SessionStore;
    use dolores_store_sqlite::SqliteStore;
    #[test]
    fn summary_draft_needs_correct_session_live_source_and_one_success() {
        let store = Arc::new(SqliteStore::open(std::path::Path::new(":memory:")).unwrap());
        store.create("chat").unwrap();
        store.create("other").unwrap();
        store
            .commit_turn("chat", "Goal: Unicode parser", "Decision: test errors.")
            .unwrap();
        let engine = Engine::new(
            store.clone(),
            Arc::new(crate::connection::testing::MemoryCredentials::default()),
        )
        .unwrap();
        let review = engine.review_summary("chat").unwrap();
        assert!(engine
            .save_summary("chat", review["token"].as_str().unwrap(), "Manual forgery")
            .is_err());
        let batch = store.review_summary_batch("chat").unwrap();
        *engine.summary_review.lock().unwrap() = Some(SummaryReview {
            token: "candidate".into(),
            session: "chat".into(),
            batch,
            model: Some("fixture".into()),
            created: Instant::now(),
        });
        engine
            .summary_review
            .lock()
            .unwrap()
            .as_mut()
            .unwrap()
            .created = Instant::now() - Duration::from_secs(301);
        assert!(engine.save_summary("chat", "candidate", "Expired").is_err());
        engine
            .summary_review
            .lock()
            .unwrap()
            .as_mut()
            .unwrap()
            .created = Instant::now();
        assert!(engine
            .save_summary("other", "candidate", "Wrong chat")
            .is_err());
        assert!(engine
            .save_summary("chat", "candidate", "api_key=secret")
            .is_err());
        engine
            .save_summary("chat", "candidate", "Goal: parser. Next: Unicode tests.")
            .unwrap();
        assert!(engine
            .save_summary("chat", "candidate", "Duplicate")
            .is_err());
        let preview = engine
            .call(crate::Command::Context {
                session: Some("chat".into()),
                input: "continue".into(),
                tools: false,
            })
            .unwrap();
        assert_eq!(preview["summary"]["coveredTurns"], 1);
        assert_eq!(preview["includedTurns"], 0);
        assert_eq!(preview["omittedTurns"], 0);
        assert!(preview["messages"][0]["content"]
            .as_str()
            .unwrap()
            .contains("Next: Unicode tests."));
        assert_eq!(preview["messages"].as_array().unwrap().len(), 2);
        assert!(engine
            .call(crate::Command::Context {
                session: Some("other".into()),
                input: "hello".into(),
                tools: false
            })
            .unwrap()
            .get("summary")
            .is_none());
        engine.delete_summary("chat", 1).unwrap();
        assert!(store.session_summary("chat").unwrap().is_none());
        assert_eq!(
            store
                .messages_page("chat", None, false, 80)
                .unwrap()
                .items
                .len(),
            2
        );
        let _ = engine.review_summary("chat").unwrap();
        engine.call(crate::Command::Shutdown).unwrap();
        assert!(engine.summary_review.lock().unwrap().is_none());
    }
}
