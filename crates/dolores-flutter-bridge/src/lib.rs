//! C ABI for the selected Flutter shell. No server or subprocess.
mod connection;
mod export;
mod recovery;
use connection::ConnectionManager;
use dolores_core::{
    prepare_context, preview_context, stream_reply_with_usage, ConnectionPreferences,
    ContextSummary, CredentialStore, ModelProvider, RequestSettings, SessionStore, TurnMetadata,
};
use dolores_store_sqlite::SqliteStore;
use serde::Deserialize;
use serde_json::{json, Value};
use std::{
    ffi::{c_char, CString},
    path::PathBuf,
    sync::{Arc, Mutex, OnceLock},
};
use tokio::{runtime::Runtime, sync::mpsc};
use tokio_util::sync::CancellationToken;

struct Run {
    id: u64,
    cancel: CancellationToken,
    events: mpsc::Receiver<Value>,
}
struct TurnRequest {
    id: u64,
    session: Option<String>,
    input: String,
    model: String,
    settings: Option<RequestSettings>,
}
struct Engine {
    runtime: Runtime,
    store: Arc<dyn SessionStore>,
    connection: Mutex<ConnectionManager>,
    active: Mutex<Option<Run>>,
}
static ENGINE: OnceLock<Result<Engine, String>> = OnceLock::new();

#[derive(Deserialize)]
#[serde(tag = "command", rename_all = "camelCase")]
enum Command {
    Bootstrap,
    SessionsPage {
        cursor: Option<dolores_core::SessionCursor>,
        #[serde(default)]
        newer: bool,
    },
    MessagesPage {
        session: String,
        cursor: Option<i64>,
        #[serde(default)]
        newer: bool,
    },
    Export {
        session: String,
        path: PathBuf,
        format: dolores_core::ExportFormat,
    },
    Messages {
        session: String,
    },
    Context {
        session: Option<String>,
        input: String,
    },
    SetRequestSettings {
        settings: RequestSettings,
    },
    Delete {
        session: String,
    },
    Configure {
        preferences: ConnectionPreferences,
        #[serde(rename = "apiKey")]
        api_key: Option<String>,
        #[serde(default, rename = "rememberConnection")]
        remember: bool,
        #[serde(rename = "enabledModels")]
        enabled_models: Option<Vec<String>>,
    },
    ListModels {
        #[serde(rename = "baseUrl")]
        base_url: String,
        #[serde(rename = "apiKey")]
        api_key: Option<String>,
    },
    SelectModel {
        model: String,
    },
    RecoverConnection,
    ForgetConnection,
    Start {
        id: u64,
        session: Option<String>,
        input: String,
    },
    Poll {
        id: u64,
    },
    Cancel {
        id: u64,
    },
    Shutdown,
}
impl Engine {
    fn new(
        store: Arc<dyn SessionStore>,
        credentials: Arc<dyn CredentialStore>,
    ) -> Result<Self, String> {
        let runtime = tokio::runtime::Builder::new_multi_thread()
            .worker_threads(2)
            .max_blocking_threads(2)
            .enable_all()
            .build()
            .map_err(|_| "Could not start the chat runtime.")?;
        let mut connection = ConnectionManager::new(store.clone(), credentials);
        {
            let _entered = runtime.enter();
            let _ = connection.recover(); // Recovery warnings keep history available.
        }
        Ok(Self {
            runtime,
            store,
            connection: Mutex::new(connection),
            active: Mutex::new(None),
        })
    }
    fn call(&self, command: Command) -> Result<Value, String> {
        // Reserve/prepare/cancel are synchronized. Persistence runs on Dart's worker
        // isolate; network and generation run on the bounded Rust runtime.
        let mut active = self.active.lock().map_err(|_| "Chat state unavailable.")?;
        match command {
            Command::Poll { id } => {
                let Some(run) = active.as_mut().filter(|run| run.id == id) else {
                    return Ok(json!([]));
                };
                let mut events = Vec::new();
                let mut done = false;
                for _ in 0..32 {
                    match run.events.try_recv() {
                        Ok(event) => {
                            done |= event["type"] == "done";
                            events.push(event);
                        }
                        Err(_) => break,
                    }
                }
                if done {
                    *active = None;
                }
                return Ok(json!(events));
            }
            Command::Cancel { id } => {
                if let Some(run) = active.as_ref().filter(|run| run.id == id) {
                    run.cancel.cancel();
                }
                return Ok(Value::Null);
            }
            Command::Shutdown => {
                if let Some(run) = active.take() {
                    run.cancel.cancel();
                }
                return Ok(Value::Null);
            }
            _ => {}
        }
        if active.is_some() {
            return Err("Stop the current response first.".into());
        }
        match command {
            Command::Bootstrap => {
                let connection = self
                    .connection
                    .lock()
                    .map_err(|_| "Connection unavailable.")?;
                let page = self.store.sessions_page(None, false, 50)?;
                Ok(
                    json!({"sessions":page.items,"sessionPage":page,"preferences":self.store.preferences()?,"requestSettings":self.store.request_settings()?,"enabledModels":connection.model_choices()?,"configured":connection.provider.is_some(),"rememberConnection":connection.remembered,"hasSavedKey":connection.has_key,"connectionWarning":connection.warning,"plugins":[self.store.descriptor(), connection.descriptor()]}),
                )
            }
            Command::SessionsPage { cursor, newer } => {
                Ok(json!(self.store.sessions_page(cursor, newer, 50)?))
            }
            Command::MessagesPage {
                session,
                cursor,
                newer,
            } => Ok(json!(self
                .store
                .messages_page(&session, cursor, newer, 80)?)),
            Command::Export {
                session,
                path,
                format,
            } => Ok(
                json!({"messageCount": export::save(self.store.as_ref(), &session, &path, format)?}),
            ),
            Command::Messages { session } => Ok(json!(self.store.messages(&session)?)),
            Command::Context { session, input } => {
                let (history, count) = match session {
                    Some(session) => self.store.context_history(&session)?,
                    None => (vec![], Some(0)),
                };
                let messages = preview_context(history, &input)?;
                let mut report = json!(ContextSummary::from_messages(&messages, count));
                report["messages"] = json!(messages);
                Ok(report)
            }
            Command::Delete { session } => {
                self.store.delete(&session)?;
                Ok(Value::Null)
            }
            Command::SetRequestSettings { settings } => {
                let _runtime = self.runtime.enter();
                self.connection
                    .lock()
                    .map_err(|_| "Connection unavailable.")?
                    .update_request_settings(settings)?;
                Ok(Value::Null)
            }
            Command::Configure {
                preferences,
                api_key,
                remember,
                enabled_models,
            } => {
                let _runtime = self.runtime.enter();
                let mut connection = self
                    .connection
                    .lock()
                    .map_err(|_| "Connection unavailable.")?;
                if let Some(models) = enabled_models {
                    connection.configure_models(preferences, api_key, remember, Some(models))?;
                } else {
                    connection.configure(preferences, api_key, remember)?;
                }
                Ok(Value::Null)
            }
            Command::ListModels { base_url, api_key } => {
                let connection = self
                    .connection
                    .lock()
                    .map_err(|_| "Connection unavailable.")?;
                Ok(json!(self
                    .runtime
                    .block_on(connection.list_models(base_url, api_key))?))
            }
            Command::SelectModel { model } => {
                let _entered = self.runtime.enter();
                self.connection
                    .lock()
                    .map_err(|_| "Connection unavailable.")?
                    .select_model(model)?;
                Ok(Value::Null)
            }
            Command::RecoverConnection => {
                let _runtime = self.runtime.enter();
                self.connection
                    .lock()
                    .map_err(|_| "Connection unavailable.")?
                    .recover()?;
                Ok(Value::Null)
            }
            Command::ForgetConnection => {
                self.connection
                    .lock()
                    .map_err(|_| "Connection unavailable.")?
                    .forget()?;
                Ok(Value::Null)
            }
            Command::Start { id, session, input } => {
                prepare_context(vec![], &input)?;
                let provider = self
                    .connection
                    .lock()
                    .map_err(|_| "Connection unavailable.")?
                    .provider
                    .clone()
                    .ok_or("Set up a model connection first.")?;
                let model = self.store.preferences()?.model;
                let settings = provider.request_settings();
                let cancel = CancellationToken::new();
                let (output, events) = mpsc::channel(32);
                *active = Some(Run {
                    id,
                    cancel: cancel.clone(),
                    events,
                });
                let store = self.store.clone();
                self.runtime.spawn(async move {
                    let result = execute(
                        store,
                        provider,
                        TurnRequest {
                            id,
                            session,
                            input,
                            model,
                            settings,
                        },
                        cancel,
                        &output,
                    )
                    .await;
                    let event = match result {
                        Ok(answer) => json!({"type":"done", "id":id, "answer":answer}),
                        Err(error) => json!({"type":"done", "id":id, "recovery":recovery::advice(&error), "error":error}),
                    };
                    let _ = output.send(event).await;
                });
                Ok(Value::Null)
            }
            _ => unreachable!(),
        }
    }
}
async fn blocking<T: Send + 'static>(
    f: impl FnOnce() -> Result<T, String> + Send + 'static,
) -> Result<T, String> {
    tokio::task::spawn_blocking(f)
        .await
        .map_err(|_| "Local storage task failed.".to_string())?
}
async fn execute(
    store: Arc<dyn SessionStore>,
    provider: Arc<dyn ModelProvider>,
    request: TurnRequest,
    cancel: CancellationToken,
    output: &mpsc::Sender<Value>,
) -> Result<String, String> {
    let TurnRequest {
        id,
        session,
        input,
        model,
        settings,
    } = request;
    if cancel.is_cancelled() {
        return Err(stopped());
    }
    let reader = store.clone();
    let (session, history, count) = blocking(move || {
        let session = match session {
            Some(id) => id,
            None => reader.create(&uuid::Uuid::new_v4().to_string())?.id,
        };
        let (history, count) = reader.context_history(&session)?;
        Ok((session, history, count))
    })
    .await?;
    let context = prepare_context(history, &input)?;
    let summary = ContextSummary::from_messages(&context, count);
    forward(
        output,
        json!({"type":"started", "id":id, "session":session, "context":summary, "requestSettings":settings}),
        &cancel,
    )
    .await?;
    // The adapter deadline alone cannot run while this host awaits a full UI
    // queue. Bound streaming AND delivery, excluding history reads and commit.
    let streaming = async {
        let (sender, mut receiver) = mpsc::channel(32);
        let request = stream_reply_with_usage(provider.as_ref(), context, sender, cancel.clone());
        tokio::pin!(request);
        let answer = loop {
            tokio::select! {
                biased;
                _ = cancel.cancelled() => return Err(stopped()),
                result = &mut request => break result?,
                Some(text) = receiver.recv() => forward(output, json!({"type":"delta", "id":id, "text":text}), &cancel).await?,
            }
        };
        while let Some(text) = receiver.recv().await {
            forward(
                output,
                json!({"type":"delta", "id":id, "text":text}),
                &cancel,
            )
            .await?;
        }
        Ok::<_, String>(answer)
    };
    let answer = match settings {
        Some(settings) => tokio::select! {
            biased;
            _ = cancel.cancelled() => return Err(stopped()),
            result = tokio::time::timeout(std::time::Duration::from_secs(settings.timeout_seconds.into()), streaming) =>
                result.map_err(|_| "Model request timed out. Adjust the request timeout or try again.")??,
        },
        None => streaming.await?,
    };
    if cancel.is_cancelled() {
        return Err(stopped());
    }
    let saved = answer.answer.clone();
    let metadata = TurnMetadata {
        model,
        usage: answer.usage,
        context: summary,
        request_settings: settings,
    };
    // Once the complete-pair transaction starts, completion wins over late Stop.
    blocking(move || store.commit_turn_metadata(&session, &input, &saved, &metadata)).await?;
    Ok(answer.answer)
}
fn stopped() -> String {
    "Response stopped. Your message was not saved.".into()
}
async fn forward(
    output: &mpsc::Sender<Value>,
    event: Value,
    cancel: &CancellationToken,
) -> Result<(), String> {
    tokio::select! {
        biased;
        _ = cancel.cancelled() => Err(stopped()),
        result = output.send(event) => result.map_err(|_| "Conversation window closed.".into()),
    }
}
fn initialize() -> Result<Engine, String> {
    let directory = match std::env::var_os("DOLORES_DATA_DIR") {
        Some(value) => {
            let path = PathBuf::from(value);
            if !path.is_absolute() {
                return Err("DOLORES_DATA_DIR must be absolute.".into());
            }
            path
        }
        None => directories::BaseDirs::new()
            .ok_or("No application data directory.")?
            .data_dir()
            .join("dev.dolores.desktop"),
    };
    std::fs::create_dir_all(&directory).map_err(|_| "Could not create the data directory.")?;
    let credentials = Arc::new(dolores_credentials::OsCredentialStore::new(&directory)?);
    Engine::new(
        Arc::new(SqliteStore::open(&directory.join("dolores.db"))?),
        credentials,
    )
}
fn reply(input: &[u8]) -> Value {
    let result = serde_json::from_slice::<Command>(input)
        .map_err(|_| "Invalid bridge request.".to_string())
        .and_then(|command| match ENGINE.get_or_init(initialize) {
            Ok(engine) => engine.call(command),
            Err(error) => Err(error.clone()),
        });
    match result {
        Ok(result) => json!({"ok":true,"result":result}),
        Err(error) => json!({"ok":false,"error":error}),
    }
}

/// Returns an owned, NUL-terminated UTF-8 JSON envelope; free exactly once.
/// # Safety
/// `input` must point to `length` readable bytes for the duration of this call.
#[no_mangle]
pub unsafe extern "C" fn dolores_call(input: *const u8, length: usize) -> *mut c_char {
    let value = std::panic::catch_unwind(|| {
        if input.is_null() || length > 128 * 1024 {
            return json!({"ok":false,"error":"Invalid bridge request size."});
        }
        // SAFETY: caller provides a live input allocation, validated size above.
        reply(unsafe { std::slice::from_raw_parts(input, length) })
    })
    .unwrap_or_else(|_| json!({"ok":false,"error":"Native bridge failed."}));
    CString::new(value.to_string())
        .expect("JSON has no literal NUL")
        .into_raw()
}
/// # Safety
/// `value` must be a live pointer returned by `dolores_call`, freed only once.
#[no_mangle]
pub unsafe extern "C" fn dolores_free(value: *mut c_char) {
    if !value.is_null() {
        drop(unsafe { CString::from_raw(value) });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;
    use dolores_core::{Message, PluginDescriptor};
    struct Fixture {
        hang: bool,
        fail: bool,
    }
    #[async_trait]
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
            for _ in 0..96 {
                output.send("你好!".into()).await.map_err(|_| "closed")?;
            }
            if self.fail {
                return Err("Fixture failed".into());
            }
            if self.hang {
                std::future::pending::<()>().await;
            }
            Ok(())
        }
        async fn stream_with_usage(
            &self,
            messages: Vec<Message>,
            output: mpsc::Sender<String>,
            cancel: CancellationToken,
        ) -> Result<Option<dolores_core::TokenUsage>, String> {
            self.stream(messages, output, cancel).await?;
            Ok(Some(dolores_core::TokenUsage {
                input_tokens: Some(0),
                output_tokens: Some(96),
                ..Default::default()
            }))
        }
    }
    #[test]
    fn context_preview_uses_latest_saved_history_and_does_not_create_or_mutate_sessions() {
        let store = Arc::new(SqliteStore::open(std::path::Path::new(":memory:")).unwrap());
        let engine = Engine::new(
            store.clone(),
            Arc::new(connection::testing::MemoryCredentials::default()),
        )
        .unwrap();
        let empty = engine
            .call(Command::Context {
                session: None,
                input: "".into(),
            })
            .unwrap();
        assert_eq!(empty["savedTurns"], 0);
        assert!(store.list().unwrap().is_empty());
        store.create("long").unwrap();
        for n in 0..61 {
            store
                .commit_turn("long", &format!("u{n}"), "reply")
                .unwrap();
        }
        let context = engine
            .call(Command::Context {
                session: Some("long".into()),
                input: "你好".into(),
            })
            .unwrap();
        assert_eq!(context["includedTurns"], 40);
        assert_eq!(context["savedTurns"], 61);
        assert_eq!(context["omittedTurns"], 21);
        let prepared = context["messages"].as_array().unwrap();
        assert_eq!(prepared.len(), 82);
        assert_eq!(prepared[1]["content"], "u21");
        assert_eq!(prepared.last().unwrap()["content"], "你好");
        assert_eq!(prepared[0]["role"], "system");
        assert_eq!(
            prepared
                .iter()
                .map(|m| m["content"].as_str().unwrap().len())
                .sum::<usize>(),
            context["textBytes"].as_u64().unwrap() as usize
        );
        assert!(engine
            .call(Command::Context {
                session: Some("missing".into()),
                input: "".into()
            })
            .is_err());
        assert!(engine
            .call(Command::Context {
                session: None,
                input: "x".repeat(dolores_core::MAX_INPUT_BYTES + 1)
            })
            .is_err());
    }
    #[test]
    fn request_settings_command_is_validated_and_visible_before_connecting() {
        let store = Arc::new(SqliteStore::open(std::path::Path::new(":memory:")).unwrap());
        let engine = Engine::new(
            store,
            Arc::new(connection::testing::MemoryCredentials::default()),
        )
        .unwrap();
        assert_eq!(
            engine.call(Command::Bootstrap).unwrap()["requestSettings"]["maxOutputTokens"],
            2048
        );
        let settings = RequestSettings {
            max_output_tokens: 4096,
            timeout_seconds: 300,
        };
        engine
            .call(Command::SetRequestSettings { settings })
            .unwrap();
        let state = engine.call(Command::Bootstrap).unwrap();
        assert_eq!(state["requestSettings"]["timeoutSeconds"], 300);
        assert_eq!(state["configured"], false);
        assert!(engine
            .call(Command::SetRequestSettings {
                settings: RequestSettings {
                    max_output_tokens: 0,
                    ..settings
                }
            })
            .is_err());
        assert_eq!(
            engine.call(Command::Bootstrap).unwrap()["requestSettings"]["maxOutputTokens"],
            4096
        );
    }
    #[test]
    fn bridge_null_and_invalid_json_return_owned_error_envelopes() {
        for (pointer, length) in [(std::ptr::null(), 0), (b"bad".as_ptr(), 3)] {
            let result = unsafe { dolores_call(pointer, length) };
            let text = unsafe { std::ffi::CStr::from_ptr(result) }
                .to_str()
                .unwrap();
            assert_eq!(serde_json::from_str::<Value>(text).unwrap()["ok"], false);
            unsafe {
                dolores_free(result);
            }
        }
    }
    #[tokio::test]
    async fn stream_deadline_still_expires_when_flutter_stops_draining_events() {
        let store = Arc::new(SqliteStore::open(std::path::Path::new(":memory:")).unwrap());
        store.create("paused-window").unwrap();
        let saved = store.clone();
        let (output, mut events) = mpsc::channel(1);
        let task = tokio::spawn(async move {
            execute(
                store,
                Arc::new(Fixture {
                    hang: false,
                    fail: false,
                }),
                TurnRequest {
                    id: 1,
                    session: Some("paused-window".into()),
                    input: "unsent".into(),
                    model: "fixture".into(),
                    settings: Some(RequestSettings {
                        max_output_tokens: 2048,
                        timeout_seconds: 1,
                    }),
                },
                CancellationToken::new(),
                &output,
            )
            .await
        });
        assert_eq!(events.recv().await.unwrap()["type"], "started");
        tokio::time::sleep(std::time::Duration::from_millis(1300)).await;
        let finished = task.is_finished();
        if !finished {
            task.abort();
        }
        assert!(
            finished,
            "A full Flutter event queue must not suspend the request deadline"
        );
        assert_eq!(
            task.await.unwrap().unwrap_err(),
            "Model request timed out. Adjust the request timeout or try again."
        );
        assert!(saved.messages("paused-window").unwrap().is_empty());
    }
    #[test]
    fn backpressure_cancel_failure_and_success_preserve_atomic_turns() {
        for (hang, fail) in [(false, false), (true, false), (false, true)] {
            let directory = tempfile::tempdir().unwrap();
            let store = Arc::new(SqliteStore::open(&directory.path().join("fixture.db")).unwrap());
            let engine = Engine::new(
                store.clone(),
                Arc::new(connection::testing::MemoryCredentials::default()),
            )
            .unwrap();
            engine.connection.lock().unwrap().provider = Some(Arc::new(Fixture { hang, fail }));
            store
                .save_preferences(&ConnectionPreferences {
                    base_url: "http://localhost/v1".into(),
                    model: "fixture".into(),
                })
                .unwrap();
            engine
                .call(Command::Start {
                    id: 7,
                    session: None,
                    input: "user".into(),
                })
                .unwrap();
            assert!(engine
                .call(Command::Delete {
                    session: "anything".into()
                })
                .is_err());
            assert!(engine
                .call(Command::Context {
                    session: None,
                    input: "".into()
                })
                .is_err());
            assert_eq!(engine.call(Command::Poll { id: 6 }).unwrap(), json!([]));
            assert!(engine
                .call(Command::SetRequestSettings {
                    settings: RequestSettings::default()
                })
                .is_err());
            assert!(engine
                .call(Command::SelectModel {
                    model: "other".into()
                })
                .is_err());
            assert!(engine
                .call(Command::ListModels {
                    base_url: "http://localhost:19421/v1".into(),
                    api_key: None
                })
                .is_err());
            assert!(engine
                .call(Command::MessagesPage {
                    session: "anything".into(),
                    cursor: None,
                    newer: false
                })
                .is_err());
            assert!(engine
                .call(Command::Export {
                    session: "anything".into(),
                    path: directory.path().join("busy.json"),
                    format: dolores_core::ExportFormat::Json
                })
                .is_err());
            // Give the producer time to fill its bounded queue, then cancel it.
            std::thread::sleep(std::time::Duration::from_millis(60));
            if hang {
                engine.call(Command::Cancel { id: 7 }).unwrap();
            }
            let mut answer = String::new();
            let deadline = std::time::Instant::now() + std::time::Duration::from_secs(3);
            let done = loop {
                assert!(
                    std::time::Instant::now() < deadline,
                    "missing terminal event"
                );
                let events = engine.call(Command::Poll { id: 7 }).unwrap();
                let mut done = None;
                for event in events.as_array().unwrap() {
                    if event["type"] == "delta" {
                        answer.push_str(event["text"].as_str().unwrap());
                    }
                    if event["type"] == "done" {
                        done = Some(event.clone());
                    }
                }
                if let Some(done) = done {
                    break done;
                }
                std::thread::sleep(std::time::Duration::from_millis(5));
            };
            let sessions = store.list().unwrap();
            let messages = store.messages(&sessions[0].id).unwrap();
            if hang || fail {
                assert!(done["error"].is_string());
                assert!(messages.is_empty());
            } else {
                assert_eq!(answer, "你好!".repeat(96));
                assert_eq!(done["answer"], answer);
                assert_eq!(messages.len(), 2);
                assert_eq!(messages[1].content, answer);
                let page = store
                    .messages_page(&sessions[0].id, None, false, 80)
                    .unwrap();
                let metadata = page.items[1].metadata.as_ref().unwrap();
                assert_eq!(metadata.model, "fixture");
                assert_eq!(metadata.usage.as_ref().unwrap().input_tokens, Some(0));
                assert_eq!(metadata.context.included_turns, 0);
            }
            assert!(engine.call(Command::Bootstrap).is_ok());
        }
    }
}
