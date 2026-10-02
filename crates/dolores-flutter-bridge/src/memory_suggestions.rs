use super::{Engine, Run};
use dolores_core::{
    MemoryDraft, MemoryMessage, MemoryScope, MemorySuggestion, ModelProvider, SessionStore,
};
use serde_json::{json, Value};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use tokio::sync::mpsc;
use tokio_util::sync::CancellationToken;

const EXPIRED: &str = "Memory review expired or changed. Discard and review this chat again.";
pub(super) enum MemoryReview {
    Sources {
        token: String,
        session: String,
        root: Option<String>,
        messages: Vec<MemoryMessage>,
        created: Instant,
    },
    Suggestions {
        token: String,
        session: String,
        root: Option<String>,
        messages: Vec<MemoryMessage>,
        candidates: Vec<Option<MemorySuggestion>>,
        model: String,
        created: Instant,
    },
}
impl MemoryReview {
    fn token(&self) -> &str {
        match self {
            Self::Sources { token, .. } | Self::Suggestions { token, .. } => token,
        }
    }
}
fn validate_sources(
    store: &dyn SessionStore,
    session: &str,
    root: &Option<String>,
    messages: &[MemoryMessage],
) -> Result<(), String> {
    if store.workspace(session)?.root != *root {
        return Err(EXPIRED.into());
    }
    for source in messages {
        if store
            .memory_source_message(session, source.message_id)?
            .as_ref()
            != Some(source)
        {
            return Err(EXPIRED.into());
        }
    }
    Ok(())
}
fn publish_review(
    mut slot: std::sync::MutexGuard<'_, Option<MemoryReview>>,
    cancel: &CancellationToken,
    review: MemoryReview,
) -> Result<(), String> {
    if cancel.is_cancelled() {
        return Err("Memory suggestions stopped. Nothing was saved.".into());
    }
    *slot = Some(review);
    Ok(())
}
impl Engine {
    pub(super) fn clear_memory_review(&self) -> Result<(), String> {
        self.memory_review
            .lock()
            .map_err(|_| "Memory review is unavailable.")?
            .take();
        Ok(())
    }
    pub(super) fn discard_memory_review(&self, token: &str) -> Result<Value, String> {
        let mut review = self
            .memory_review
            .lock()
            .map_err(|_| "Memory review is unavailable.")?;
        if review.as_ref().is_some_and(|r| r.token() == token) {
            review.take();
        }
        Ok(Value::Null)
    }
    pub(super) fn review_memory_sources(&self, session: &str) -> Result<Value, String> {
        self.clear_memory_review()?;
        let page = self.store.memory_source_messages(session)?;
        let root = self.store.workspace(session)?.root;
        let token = uuid::Uuid::new_v4().to_string();
        let result = json!({"token":token,"items":page.items,"hasOlder":page.has_older,
            "folderAvailable":root.is_some(),"model":self.store.preferences()?.model});
        *self
            .memory_review
            .lock()
            .map_err(|_| "Memory review is unavailable.")? = Some(MemoryReview::Sources {
            token,
            session: session.into(),
            root,
            messages: page.items,
            created: Instant::now(),
        });
        Ok(result)
    }
    pub(super) fn suggest_memories(
        &self,
        active: &mut Option<Run>,
        id: u64,
        session: String,
        token: String,
        message_ids: Vec<i64>,
    ) -> Result<Value, String> {
        let review = self
            .memory_review
            .lock()
            .map_err(|_| "Memory review is unavailable.")?
            .take()
            .ok_or(EXPIRED)?;
        let MemoryReview::Sources {
            token: expected,
            session: bound,
            root,
            messages,
            created,
        } = review
        else {
            return Err(EXPIRED.into());
        };
        if expected != token || bound != session || created.elapsed() > Duration::from_secs(300) {
            return Err(EXPIRED.into());
        }
        let selected: Vec<_> = message_ids
            .iter()
            .map(|id| {
                messages
                    .iter()
                    .find(|m| m.message_id == *id)
                    .cloned()
                    .ok_or(EXPIRED)
            })
            .collect::<Result<_, _>>()?;
        let prompt = dolores_core::memory_suggestion_prompt(&selected)?;
        validate_sources(self.store.as_ref(), &session, &root, &selected)?;
        let _entered = self.runtime.enter();
        let provider = self
            .connection
            .lock()
            .map_err(|_| "Connection unavailable.")?
            .memory_suggestion_provider()?;
        let model = self.store.preferences()?.model;
        let settings = provider.request_settings().unwrap_or_default();
        let (prompt, tokens) = dolores_core::prepare_token_context(
            prompt,
            &[],
            provider.context_window_tokens(),
            settings,
        )?;
        let cancel = CancellationToken::new();
        let (output, events) = mpsc::channel(4);
        *active = Some(Run {
            id,
            cancel: cancel.clone(),
            events,
            approvals: Arc::new(Mutex::new(None)),
        });
        let slot = self.memory_review.clone();
        let store = self.store.clone();
        self.runtime.spawn(async move {
            let result = collect(provider,prompt,cancel.clone()).await;
            let result = match result {
                Ok((answer,usage)) => {
                    let candidates = dolores_core::parse_memory_suggestions(&answer,&selected);
                    match candidates {
                        Ok(candidates) => {
                            let reader=store.clone(); let check_session=session.clone(); let check_root=root.clone(); let check_sources=selected.clone();
                            match super::blocking(move || validate_sources(reader.as_ref(),&check_session,&check_root,&check_sources)).await {
                                Ok(()) if !cancel.is_cancelled() => {
                                    let token = uuid::Uuid::new_v4().to_string();
                                    let response = json!({"token":token,"items":candidates,"model":model,"usage":usage,"tokens":tokens});
                                    match slot.lock() {
                                        Ok(review) => publish_review(review, &cancel, MemoryReview::Suggestions {token,session,root,messages:selected,candidates:candidates.into_iter().map(Some).collect(),model,created:Instant::now()}).map(|()| response),
                                        Err(_) => Err("Memory review is unavailable.".into()),
                                    }
                                }
                                Ok(()) => Err("Memory suggestions stopped. Nothing was saved.".into()),
                                Err(error) => Err(error),
                            }
                        }
                        Err(error) => Err(error),
                    }
                }
                Err(error) => Err(error),
            };
            let event = match result {
                Ok(review) => json!({"type":"done","id":id,"memorySuggestions":review}),
                Err(error) => json!({"type":"done","id":id,"error":error}),
            };
            let _ = output.send(event).await;
        });
        Ok(Value::Null)
    }
    pub(super) fn save_memory_suggestion(
        &self,
        session: &str,
        token: &str,
        index: usize,
        scope: MemoryScope,
        input: super::memory::MemoryInput,
    ) -> Result<Value, String> {
        if input.id.is_some() || input.revision.is_some() {
            return Err(
                "Memory suggestions create new preferences; edit existing entries in Memory."
                    .into(),
            );
        }
        dolores_core::validate_preference(&input.title, &input.text)?;
        if dolores_core::credential_like(&input.title) || dolores_core::credential_like(&input.text)
        {
            return Err(
                "Memory preference appears to contain credentials. Remove them before saving."
                    .into(),
            );
        }
        let mut slot = self
            .memory_review
            .lock()
            .map_err(|_| "Memory review is unavailable.")?;
        let Some(MemoryReview::Suggestions {
            token: expected,
            session: bound,
            root,
            messages,
            candidates,
            model,
            created,
        }) = slot.as_mut()
        else {
            return Err(EXPIRED.into());
        };
        if expected != token || bound != session || created.elapsed() > Duration::from_secs(300) {
            return Err(EXPIRED.into());
        }
        validate_sources(self.store.as_ref(), session, root, messages)?;
        let candidate = candidates
            .get(index)
            .and_then(Option::as_ref)
            .ok_or(EXPIRED)?;
        let root = match scope {
            MemoryScope::All => None,
            MemoryScope::Folder => Some(
                root.as_deref()
                    .ok_or("Memory folder preferences need an existing working folder.")?,
            ),
        };
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as i64;
        let result = self.store.save_suggested_memory_preference(
            root,
            &MemoryDraft {
                id: uuid::Uuid::new_v4().to_string(),
                revision: None,
                title: input.title,
                text: input.text,
                enabled: input.enabled,
                origin: Some(candidate.origin(session, model, timestamp)),
            },
            messages,
        )?;
        candidates[index] = None;
        Ok(json!(result))
    }
}

async fn collect(
    provider: Arc<dyn ModelProvider>,
    prompt: Vec<dolores_core::Message>,
    cancel: CancellationToken,
) -> Result<(String, Option<dolores_core::TokenUsage>), String> {
    let (sender, mut receiver) = mpsc::channel(8);
    let request = provider.stream_with_usage(prompt, sender, cancel.clone());
    tokio::pin!(request);
    let mut answer = String::new();
    let mut finished = false;
    let mut closed = false;
    let mut usage = None;
    let timeout = tokio::time::sleep(Duration::from_secs(30));
    tokio::pin!(timeout);
    loop {
        if finished && closed {
            break;
        }
        tokio::select! { biased;
            _=cancel.cancelled()=>return Err("Memory suggestions stopped. Nothing was saved.".into()),
            _=&mut timeout=>{cancel.cancel();return Err("Memory suggestions timed out. Nothing was saved. Try again with fewer messages.".into());},
            result=&mut request,if !finished=>{usage=result?;finished=true;},
            delta=receiver.recv(), if !closed=>match delta {
                Some(delta)=>{if answer.len()+delta.len()>dolores_core::MAX_MEMORY_SUGGESTION_BYTES {cancel.cancel();return Err("Memory suggestions exceed the response limit. Nothing was saved.".into());} answer.push_str(&delta);},
                None=>closed=true,
            }
        }
    }
    Ok((answer, usage))
}

#[cfg(test)]
mod tests {
    use super::*;
    use dolores_core::{Message, PluginDescriptor, Role};
    use dolores_store_sqlite::SqliteStore;
    #[test]
    fn canceled_completion_cannot_publish_over_a_newer_review() {
        let slot = Mutex::new(Some(MemoryReview::Sources {
            token: "new-review".into(),
            session: "new-chat".into(),
            root: None,
            messages: vec![],
            created: Instant::now(),
        }));
        let cancel = CancellationToken::new();
        // Cancellation can happen after the worker's earlier source check but
        // before it acquires this commit-boundary lock.
        cancel.cancel();
        let old = MemoryReview::Suggestions {
            token: "old-review".into(),
            session: "old-chat".into(),
            root: None,
            messages: vec![],
            candidates: vec![],
            model: "fixture".into(),
            created: Instant::now(),
        };
        assert!(publish_review(slot.lock().unwrap(), &cancel, old).is_err());
        assert_eq!(slot.lock().unwrap().as_ref().unwrap().token(), "new-review");
    }
    fn pending(engine: &Engine, session: &str, source: MemoryMessage) -> String {
        let token = uuid::Uuid::new_v4().to_string();
        *engine.memory_review.lock().unwrap() = Some(MemoryReview::Suggestions {
            token: token.clone(),
            session: session.into(),
            root: None,
            messages: vec![source.clone()],
            candidates: vec![Some(MemorySuggestion {
                title: "Style".into(),
                text: "Prefer concise examples.".into(),
                message_id: source.message_id,
                quote: source.text,
            })],
            model: "fixture".into(),
            created: Instant::now(),
        });
        token
    }
    fn input() -> super::super::memory::MemoryInput {
        super::super::memory::MemoryInput {
            id: None,
            revision: None,
            title: "Corrected style".into(),
            text: "Prefer short examples.".into(),
            enabled: true,
        }
    }
    #[test]
    fn review_save_requires_exact_source_scope_and_single_success_while_failures_retain_it() {
        let temp = tempfile::tempdir().unwrap();
        let db = temp.path().join("state.db");
        let store = Arc::new(SqliteStore::open(&db).unwrap());
        store.create("chat").unwrap();
        store.create("other").unwrap();
        store
            .commit_turn(
                "chat",
                "I prefer concise examples.",
                "TOOLS_AND_ASSISTANT_MUST_STAY_OUT",
            )
            .unwrap();
        let engine = Engine::new(
            store.clone(),
            Arc::new(crate::connection::testing::MemoryCredentials::default()),
        )
        .unwrap();
        let review = engine.review_memory_sources("chat").unwrap();
        assert!(!review.to_string().contains("TOOLS_AND_ASSISTANT"));
        assert_eq!(review["items"].as_array().unwrap().len(), 1);
        let source = store
            .memory_source_messages("chat")
            .unwrap()
            .items
            .remove(0);
        let token = pending(&engine, "chat", source.clone());
        assert!(engine
            .save_memory_suggestion("other", &token, 0, MemoryScope::All, input())
            .is_err());
        assert!(engine
            .save_memory_suggestion("chat", &token, 0, MemoryScope::Folder, input())
            .is_err());
        let connection = rusqlite::Connection::open(&db).unwrap();
        connection.execute_batch("CREATE TRIGGER fail_memory BEFORE INSERT ON memory_preferences BEGIN SELECT RAISE(ABORT,'fixture'); END;").unwrap();
        assert!(engine
            .save_memory_suggestion("chat", &token, 0, MemoryScope::All, input())
            .is_err());
        assert!(store.memory_preferences(None).unwrap().is_empty());
        connection
            .execute_batch("DROP TRIGGER fail_memory;")
            .unwrap();
        let saved = engine
            .save_memory_suggestion("chat", &token, 0, MemoryScope::All, input())
            .unwrap();
        assert_eq!(saved["source"], "conversation");
        assert_eq!(saved["origin"]["messageId"], source.message_id);
        assert_eq!(saved["text"], "Prefer short examples.");
        assert!(engine
            .save_memory_suggestion("chat", &token, 0, MemoryScope::All, input())
            .is_err());
        let context = engine
            .call(crate::Command::Context {
                session: Some("chat".into()),
                input: "hello".into(),
                tools: false,
            })
            .unwrap();
        assert_eq!(context["memory"]["used"][0]["origin"], saved["origin"]);
        assert!(!context["messages"][0]["content"]
            .as_str()
            .unwrap()
            .contains("I prefer concise examples."));
        let token = pending(&engine, "chat", source.clone());
        connection
            .execute(
                "UPDATE messages SET content='Changed' WHERE id=?1",
                [source.message_id],
            )
            .unwrap();
        assert!(engine
            .save_memory_suggestion("chat", &token, 0, MemoryScope::All, input())
            .is_err());
        store.delete("chat").unwrap();
        assert!(
            !engine.memories(None).unwrap()["items"][0]["originAvailable"]
                .as_bool()
                .unwrap()
        );
        engine.discard_memory_review(&token).unwrap();
        assert!(engine.memory_review.lock().unwrap().is_none());
    }

    struct Fixture {
        overflow: bool,
    }
    #[async_trait::async_trait]
    impl ModelProvider for Fixture {
        fn descriptor(&self) -> PluginDescriptor {
            PluginDescriptor {
                id: "fixture",
                kind: "provider",
                api_version: 1,
            }
        }
        async fn stream(
            &self,
            _: Vec<Message>,
            output: mpsc::Sender<String>,
            _: CancellationToken,
        ) -> Result<(), String> {
            if self.overflow {
                output.send("x".repeat(8193)).await.unwrap();
                Ok(())
            } else {
                drop(output);
                std::future::pending().await
            }
        }
    }
    #[tokio::test]
    async fn overflow_and_closed_output_hung_provider_remain_cancelable_without_results() {
        let prompt = vec![Message {
            role: Role::User,
            content: "fixture".into(),
        }];
        assert!(collect(
            Arc::new(Fixture { overflow: true }),
            prompt.clone(),
            CancellationToken::new()
        )
        .await
        .is_err());
        let cancel = CancellationToken::new();
        let task_cancel = cancel.clone();
        let task = tokio::spawn(collect(
            Arc::new(Fixture { overflow: false }),
            prompt,
            task_cancel,
        ));
        tokio::task::yield_now().await;
        cancel.cancel();
        assert!(tokio::time::timeout(Duration::from_millis(200), task)
            .await
            .unwrap()
            .unwrap()
            .is_err());
    }
}
