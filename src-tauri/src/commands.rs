use dolores_core::{
    prepare_context, stream_reply, ConnectionPreferences, Message, ModelProvider, PluginDescriptor,
    Session, SessionStore,
};
use dolores_provider_openai::OpenAiProvider;
use serde::Serialize;
use std::sync::{Arc, Mutex};
use tauri::{ipc::Channel, State};
use tokio::sync::mpsc;
use tokio_util::sync::CancellationToken;

pub struct AppState {
    pub store: Arc<dyn SessionStore>,
    pub provider: Mutex<Option<Arc<dyn ModelProvider>>>,
    pub active: Mutex<Option<ActiveRun>>,
}
pub struct ActiveRun {
    id: String,
    cancel: CancellationToken,
}
struct RunGuard<'a>(&'a Mutex<Option<ActiveRun>>);
impl Drop for RunGuard<'_> {
    fn drop(&mut self) {
        if let Ok(mut active) = self.0.lock() {
            *active = None;
        }
    }
}
fn reserve<'a>(
    state: &'a AppState,
    id: String,
    cancel: CancellationToken,
) -> Result<RunGuard<'a>, String> {
    let mut active = state
        .active
        .lock()
        .map_err(|_| "Run state unavailable.".to_string())?;
    if active.is_some() {
        return Err("Finish or stop the current response first.".into());
    }
    *active = Some(ActiveRun { id, cancel });
    Ok(RunGuard(&state.active))
}
async fn storage<T: Send + 'static>(
    store: Arc<dyn SessionStore>,
    work: impl FnOnce(&dyn SessionStore) -> Result<T, String> + Send + 'static,
) -> Result<T, String> {
    tokio::task::spawn_blocking(move || work(store.as_ref()))
        .await
        .map_err(|_| "Local storage operation failed.".to_string())?
}
fn valid_id(id: &str) -> Result<(), String> {
    uuid::Uuid::parse_str(id)
        .map(|_| ())
        .map_err(|_| "Invalid conversation or run ID.".into())
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Bootstrap {
    sessions: Vec<Session>,
    preferences: ConnectionPreferences,
    connected: bool,
    plugins: Vec<PluginDescriptor>,
}

#[tauri::command]
pub async fn bootstrap(state: State<'_, AppState>) -> Result<Bootstrap, String> {
    let (sessions, preferences, storage_plugin) = storage(state.store.clone(), |store| {
        Ok((store.list()?, store.preferences()?, store.descriptor()))
    })
    .await?;
    let provider = state
        .provider
        .lock()
        .map_err(|_| "Connection unavailable.".to_string())?;
    Ok(Bootstrap {
        sessions,
        preferences,
        connected: provider.is_some(),
        plugins: vec![
            PluginDescriptor {
                id: "dolores.provider.openai-compatible",
                kind: "provider",
                api_version: 1,
            },
            storage_plugin,
        ],
    })
}

#[tauri::command]
pub async fn create_session(state: State<'_, AppState>) -> Result<Session, String> {
    let id = uuid::Uuid::new_v4().to_string();
    storage(state.store.clone(), move |store| store.create(&id)).await
}

#[tauri::command]
pub async fn get_messages(
    state: State<'_, AppState>,
    session_id: String,
) -> Result<Vec<Message>, String> {
    valid_id(&session_id)?;
    storage(state.store.clone(), move |store| {
        store.messages(&session_id)
    })
    .await
}

#[tauri::command]
pub async fn delete_session(state: State<'_, AppState>, session_id: String) -> Result<(), String> {
    valid_id(&session_id)?;
    let _guard = reserve(&state, "delete".into(), CancellationToken::new())?;
    storage(state.store.clone(), move |store| store.delete(&session_id)).await
}

#[tauri::command]
pub async fn configure_connection(
    state: State<'_, AppState>,
    preferences: ConnectionPreferences,
    api_key: String,
) -> Result<(), String> {
    let _guard = reserve(&state, "configure".into(), CancellationToken::new())?;
    let provider = Arc::new(OpenAiProvider::new(&preferences, api_key)?);
    storage(state.store.clone(), move |store| {
        store.save_preferences(&preferences)
    })
    .await?;
    *state
        .provider
        .lock()
        .map_err(|_| "Connection unavailable.".to_string())? = Some(provider);
    Ok(())
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DeltaEvent {
    run_id: String,
    delta: String,
}

#[tauri::command]
pub async fn generate(
    state: State<'_, AppState>,
    session_id: String,
    run_id: String,
    input: String,
    on_event: Channel<DeltaEvent>,
) -> Result<(), String> {
    valid_id(&session_id)?;
    valid_id(&run_id)?;
    prepare_context(Vec::new(), &input)?;
    let cancel = CancellationToken::new();
    let _guard = reserve(&state, run_id.clone(), cancel.clone())?;
    let provider = state
        .provider
        .lock()
        .map_err(|_| "Connection unavailable.".to_string())?
        .clone()
        .ok_or("Set up a model connection first.")?;
    // Acknowledge reservation before any awaited work so an early Stop cannot
    // race a not-yet-active run. Empty deltas carry no model text.
    on_event
        .send(DeltaEvent {
            run_id: run_id.clone(),
            delta: String::new(),
        })
        .map_err(|_| "Conversation window closed.".to_string())?;
    let history_id = session_id.clone();
    let history = storage(state.store.clone(), move |store| {
        store.messages(&history_id)
    })
    .await?;
    let context = prepare_context(history, &input)?;
    let (sender, mut receiver) = mpsc::channel(32);
    let forwarding_cancel = cancel.clone();
    let forwarding = tokio::spawn(async move {
        while let Some(delta) = receiver.recv().await {
            if on_event
                .send(DeltaEvent {
                    run_id: run_id.clone(),
                    delta,
                })
                .is_err()
            {
                forwarding_cancel.cancel();
                return Err("Conversation window closed.".to_string());
            }
        }
        Ok(())
    });
    let answer = stream_reply(provider.as_ref(), context, sender, cancel.clone()).await;
    let forwarded = forwarding
        .await
        .map_err(|_| "Could not display the response.".to_string())?;
    let answer = answer?;
    forwarded?;
    if cancel.is_cancelled() {
        return Err("Response stopped. Your message was not saved.".into());
    }
    storage(state.store.clone(), move |store| {
        store.commit_turn(&session_id, &input, &answer)
    })
    .await
}

#[tauri::command]
pub fn cancel_run(state: State<'_, AppState>, run_id: String) -> Result<(), String> {
    valid_id(&run_id)?;
    let active = state
        .active
        .lock()
        .map_err(|_| "Run state unavailable.".to_string())?;
    if let Some(run) = active.as_ref().filter(|run| run.id == run_id) {
        run.cancel.cancel();
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use dolores_store_sqlite::SqliteStore;
    #[test]
    fn a_run_excludes_another_and_guard_releases_on_error() {
        let state = AppState {
            store: Arc::new(SqliteStore::open(std::path::Path::new(":memory:")).unwrap()),
            provider: Mutex::new(None),
            active: Mutex::new(None),
        };
        let guard = reserve(&state, "first".into(), CancellationToken::new()).unwrap();
        assert!(reserve(&state, "second".into(), CancellationToken::new()).is_err());
        drop(guard);
        assert!(reserve(&state, "third".into(), CancellationToken::new()).is_ok());
    }
}
