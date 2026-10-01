use dolores_core::{prepare_context, stream_reply, ModelProvider, SessionStore};
use iced::futures::{channel::mpsc::Sender, SinkExt, Stream};
use std::sync::Arc;
use tokio::sync::mpsc;
use tokio_util::sync::CancellationToken;

#[derive(Debug, Clone)]
pub enum TurnEvent {
    Started {
        id: u64,
        session: String,
    },
    Delta {
        id: u64,
        text: String,
    },
    Done {
        id: u64,
        result: Result<String, String>,
    },
}
impl TurnEvent {
    pub fn id(&self) -> u64 {
        match self {
            Self::Started { id, .. } | Self::Delta { id, .. } | Self::Done { id, .. } => *id,
        }
    }
}
pub async fn blocking<T: Send + 'static>(
    operation: impl FnOnce() -> Result<T, String> + Send + 'static,
) -> Result<T, String> {
    tokio::task::spawn_blocking(operation)
        .await
        .map_err(|_| "Local storage task failed.".to_string())?
}
pub fn stream(
    store: Arc<dyn SessionStore>,
    provider: Arc<dyn ModelProvider>,
    session: Option<String>,
    user: String,
    cancel: CancellationToken,
    id: u64,
) -> impl Stream<Item = TurnEvent> {
    iced::stream::channel(32, move |mut output| async move {
        let result = execute(store, provider, session, user, cancel, id, &mut output).await;
        let _ = output.send(TurnEvent::Done { id, result }).await;
    })
}
async fn execute(
    store: Arc<dyn SessionStore>,
    provider: Arc<dyn ModelProvider>,
    session: Option<String>,
    user: String,
    cancel: CancellationToken,
    id: u64,
    output: &mut Sender<TurnEvent>,
) -> Result<String, String> {
    if cancel.is_cancelled() {
        return Err(stopped());
    }
    let reader = store.clone();
    let (session, history) = blocking(move || {
        let session = match session {
            Some(id) => id,
            None => reader.create(&uuid::Uuid::new_v4().to_string())?.id,
        };
        let history = reader.messages(&session)?;
        Ok((session, history))
    })
    .await?;
    output
        .send(TurnEvent::Started {
            id,
            session: session.clone(),
        })
        .await
        .map_err(|_| "Conversation window closed.".to_string())?;
    let context = prepare_context(history, &user)?;
    let (sender, mut receiver) = mpsc::channel(32);
    let request = stream_reply(provider.as_ref(), context, sender, cancel.clone());
    tokio::pin!(request);
    let answer = loop {
        tokio::select! {
            biased;
            _ = cancel.cancelled() => return Err(stopped()),
            result = &mut request => break result?,
            Some(text) = receiver.recv() => forward(output, TurnEvent::Delta { id, text }, &cancel).await?,
        }
    };
    while let Some(text) = receiver.recv().await {
        forward(output, TurnEvent::Delta { id, text }, &cancel).await?;
    }
    if cancel.is_cancelled() {
        return Err(stopped());
    }
    // Once this atomic transaction starts, completion wins over a late Stop.
    let saved = answer.clone();
    blocking(move || store.commit_turn(&session, &user, &saved)).await?;
    Ok(answer)
}
fn stopped() -> String {
    "Response stopped. Your message was not saved.".into()
}
async fn forward(
    output: &mut Sender<TurnEvent>,
    event: TurnEvent,
    cancel: &CancellationToken,
) -> Result<(), String> {
    tokio::select! {
        biased;
        _ = cancel.cancelled() => Err(stopped()),
        result = output.send(event) => result.map_err(|_| "Conversation window closed.".to_string()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;
    use dolores_core::{Message, PluginDescriptor};
    use dolores_store_sqlite::SqliteStore;
    use iced::futures::{pin_mut, StreamExt};

    struct Fixture {
        hang: bool,
        fail: bool,
    }
    #[async_trait]
    impl ModelProvider for Fixture {
        fn descriptor(&self) -> PluginDescriptor {
            PluginDescriptor {
                id: "test",
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
            for _ in 0..64 {
                output
                    .send("你好!".into())
                    .await
                    .map_err(|_| "closed".to_string())?;
            }
            if self.fail {
                return Err("Fixture failed".into());
            }
            if self.hang {
                std::future::pending::<()>().await;
            }
            Ok(())
        }
    }
    async fn run_case(hang: bool, fail: bool, pre_cancel: bool) {
        let directory = tempfile::tempdir().unwrap();
        let store = Arc::new(SqliteStore::open(&directory.path().join("test.db")).unwrap());
        let cancel = CancellationToken::new();
        if pre_cancel {
            cancel.cancel();
        }
        let events = stream(
            store.clone(),
            Arc::new(Fixture { hang, fail }),
            None,
            "user".into(),
            cancel.clone(),
            9,
        );
        pin_mut!(events);
        let mut session = None;
        let mut deltas = String::new();
        let mut result = None;
        tokio::time::timeout(std::time::Duration::from_secs(3), async {
            while let Some(event) = events.next().await {
                assert_eq!(event.id(), 9);
                match event {
                    TurnEvent::Started { session: id, .. } => session = Some(id),
                    TurnEvent::Delta { text, .. } => {
                        deltas.push_str(&text);
                        if hang {
                            cancel.cancel();
                        }
                    }
                    TurnEvent::Done { result: value, .. } => result = Some(value),
                }
            }
        })
        .await
        .expect("generation must terminate");
        let result = result.expect("terminal event");
        if hang || fail || pre_cancel {
            assert!(result.is_err());
            if let Some(session) = session {
                assert!(store.messages(&session).unwrap().is_empty());
            }
            if pre_cancel {
                assert!(store.list().unwrap().is_empty());
            }
        } else {
            let answer = result.unwrap();
            assert_eq!(answer, "你好!".repeat(64));
            assert_eq!(answer, deltas);
            let messages = store.messages(&session.unwrap()).unwrap();
            assert_eq!(messages.len(), 2);
            assert_eq!(messages[0].content, "user");
            assert_eq!(messages[1].content, answer);
        }
    }
    #[tokio::test]
    async fn drains_bounded_stream_and_commits_complete_pair() {
        run_case(false, false, false).await;
    }
    #[tokio::test]
    async fn cancel_hung_stream_never_saves_partial_turn() {
        run_case(true, false, false).await;
    }
    #[tokio::test]
    async fn provider_failure_never_saves_partial_turn() {
        run_case(false, true, false).await;
    }
    #[tokio::test]
    async fn stop_before_start_creates_no_session() {
        run_case(false, false, true).await;
    }
}
