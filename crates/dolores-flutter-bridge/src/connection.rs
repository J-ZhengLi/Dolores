use dolores_core::{
    ConnectionPreferences, CredentialStore, ModelContexts, ModelProvider, RememberedConnection,
    RequestSettings, SessionStore,
};
use dolores_provider_openai::{validate_base_url, validate_model, OpenAiProvider};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Serialize, Deserialize)]
struct StoredKey {
    base_url: String,
    api_key: String,
}

pub struct ConnectionManager {
    store: Arc<dyn SessionStore>,
    credentials: Arc<dyn CredentialStore>,
    pub provider: Option<Arc<dyn ModelProvider>>,
    pub remembered: bool,
    pub has_key: bool,
    pub warning: Option<String>,
    active_key: Option<String>,
    active_base_url: Option<String>,
}
impl ConnectionManager {
    pub(super) fn review_provider(&self) -> Result<Arc<dyn ModelProvider>, String> {
        self.bounded_review_provider(1024, 30)
    }
    pub(super) fn automatic_memory_provider(&self) -> Result<Arc<dyn ModelProvider>, String> {
        self.bounded_review_provider(512, 10)
    }
    pub(super) fn skill_draft_provider(
        &self,
        settings: Option<RequestSettings>,
    ) -> Result<Arc<dyn ModelProvider>, String> {
        self.review_provider_with_settings(32768, 900, settings)
    }
    pub(super) fn bounded_review_provider(
        &self,
        max_output: u32,
        timeout: u32,
    ) -> Result<Arc<dyn ModelProvider>, String> {
        self.review_provider_with_settings(max_output, timeout, None)
    }
    fn review_provider_with_settings(
        &self,
        max_output: u32,
        timeout: u32,
        settings: Option<RequestSettings>,
    ) -> Result<Arc<dyn ModelProvider>, String> {
        let preferences = self.store.preferences()?;
        if self.provider.is_none()
            || self.active_base_url.as_deref() != Some(preferences.base_url.as_str())
        {
            return Err("Set up a model connection first.".into());
        }
        let settings = settings.unwrap_or(self.store.effective_request_settings(&preferences)?);
        settings.validate()?;
        Ok(Arc::new(
            OpenAiProvider::with_settings(
                &preferences,
                self.active_key
                    .clone()
                    .ok_or("Reconnect your model first.")?,
                RequestSettings {
                    max_output_tokens: settings.max_output_tokens.min(max_output),
                    timeout_seconds: settings.timeout_seconds.min(timeout),
                    ..settings
                },
            )?
            .with_context_window(self.context_window(&preferences)?),
        ))
    }
    pub fn new(store: Arc<dyn SessionStore>, credentials: Arc<dyn CredentialStore>) -> Self {
        Self {
            store,
            credentials,
            provider: None,
            remembered: false,
            has_key: false,
            warning: None,
            active_key: None,
            active_base_url: None,
        }
    }
    pub fn recover(&mut self) -> Result<(), String> {
        self.provider = None;
        self.active_key = None;
        self.active_base_url = None;
        self.remembered = false;
        self.has_key = false;
        self.warning = None;
        let result = self.restore();
        if let Err(error) = &result {
            self.warning = Some(error.clone());
        }
        result
    }
    fn restore(&mut self) -> Result<(), String> {
        let preferences = self.store.preferences()?;
        let saved = self.store.remembered_connection()?;
        self.remembered = saved.is_some();
        let Some(saved) = saved else {
            if !preferences.model.is_empty() {
                self.warning = Some(
                    "Connection is not remembered. Open Model connection to reconnect.".into(),
                );
            }
            return Ok(());
        };
        if saved.preferences != preferences {
            return Err(
                "Connection settings changed in another shell. Re-enter your key to reconnect."
                    .into(),
            );
        }
        let key = match saved.credential_id.as_deref() {
            Some(id) => self.read_key(id, &preferences.base_url)?,
            None => String::new(),
        };
        let provider = Arc::new(
            OpenAiProvider::with_settings(
                &preferences,
                key.clone(),
                self.store.effective_request_settings(&preferences)?,
            )?
            .with_context_window(self.context_window(&preferences)?),
        );
        self.active_key = Some(key);
        self.active_base_url = Some(preferences.base_url.clone());
        self.has_key = saved.credential_id.is_some();
        self.provider = Some(provider);
        Ok(())
    }
    fn read_key(&self, id: &str, base_url: &str) -> Result<String, String> {
        let secret = self
            .credentials
            .read(id)?
            .ok_or("Saved key is unavailable. Re-enter it to reconnect.")?;
        let saved: StoredKey = serde_json::from_str(&secret)
            .map_err(|_| "Saved key could not be read. Re-enter it to reconnect.")?;
        if saved.base_url != base_url {
            return Err(
                "The saved key belongs to a different endpoint. Re-enter it for this connection."
                    .into(),
            );
        }
        Ok(saved.api_key)
    }
    pub fn configure(
        &mut self,
        preferences: ConnectionPreferences,
        api_key: Option<String>,
        remember: bool,
    ) -> Result<(), String> {
        self.configure_models(preferences, api_key, remember, None)
    }
    pub fn configure_models(
        &mut self,
        preferences: ConnectionPreferences,
        api_key: Option<String>,
        remember: bool,
        models: Option<Vec<String>>,
    ) -> Result<(), String> {
        self.configure_model_contexts(preferences, api_key, remember, models, None)
    }
    pub fn configure_model_contexts(
        &mut self,
        preferences: ConnectionPreferences,
        api_key: Option<String>,
        remember: bool,
        models: Option<Vec<String>>,
        contexts: Option<ModelContexts>,
    ) -> Result<(), String> {
        let mut preferences = preferences;
        preferences.model = preferences.model.trim().to_string();
        let models = validate_choices(
            models.unwrap_or_else(|| vec![preferences.model.clone()]),
            &preferences.model,
        )?;
        if let Some(contexts) = &contexts {
            dolores_core::validate_model_contexts(contexts, &models)?;
        }
        let window = match &contexts {
            Some(contexts) => contexts.get(&preferences.model).copied().flatten(),
            None => self.context_window(&preferences)?,
        };
        let old = self.store.remembered_connection()?;
        let key = match api_key {
            Some(key) => key,
            None if self.active_key.is_some() && self.active_base_url.as_deref() == Some(preferences.base_url.as_str()) => self.active_key.clone().unwrap(),
            None => match old.as_ref().and_then(|record| record.credential_id.as_deref()) {
                Some(id) if old.as_ref().unwrap().preferences.base_url == preferences.base_url => self.read_key(id, &preferences.base_url)?,
                Some(_) => return Err("Enter a key for the new endpoint. Saved keys are never forwarded to another endpoint.".into()),
                None => String::new(),
            },
        };
        // Validate before any credential or preference write.
        let provider = Arc::new(
            OpenAiProvider::with_settings(
                &preferences,
                key.clone(),
                self.store.effective_request_settings(&preferences)?,
            )?
            .with_context_window(window),
        );
        let id = if remember && !key.is_empty() {
            Some(uuid::Uuid::new_v4().to_string())
        } else {
            None
        };
        if let Some(id) = &id {
            let secret = serde_json::to_string(&StoredKey {
                base_url: preferences.base_url.clone(),
                api_key: key.clone(),
            })
            .map_err(|_| "Could not prepare secure storage.")?;
            self.credentials.write(id, &secret)?;
        }
        let saved = remember.then(|| RememberedConnection {
            preferences: preferences.clone(),
            credential_id: id.clone(),
        });
        let result = match &contexts {
            Some(contexts) => self.store.save_connection_model_contexts(
                &preferences,
                saved.as_ref(),
                &models,
                contexts,
            ),
            None => self
                .store
                .save_connection_models(&preferences, saved.as_ref(), &models),
        };
        if let Err(error) = result {
            if let Some(id) = &id {
                if self.credentials.delete(id).is_err() {
                    return Err("Connection was not saved. A new secure entry could not be removed; check your OS credential store.".into());
                }
            }
            return Err(error);
        }
        self.provider = Some(provider);
        self.active_key = Some(key);
        self.active_base_url = Some(preferences.base_url.clone());
        self.remembered = remember;
        self.has_key = id.is_some();
        self.warning = None;
        if let Some(old_id) = old.and_then(|old| old.credential_id) {
            if self.credentials.delete(&old_id).is_err() {
                self.warning = Some("Connection updated, but an old secure entry could not be removed. Check your OS credential store.".into());
            }
        }
        Ok(())
    }
    pub fn model_choices(&self) -> Result<Vec<String>, String> {
        let preferences = self.store.preferences()?;
        let mut models = self.store.model_choices(&preferences.base_url)?;
        if !preferences.model.is_empty() && !models.contains(&preferences.model) {
            models.push(preferences.model);
        }
        Ok(models)
    }
    pub fn model_contexts(&self) -> Result<ModelContexts, String> {
        self.store
            .model_contexts(&self.store.preferences()?.base_url)
    }
    fn context_window(&self, preferences: &ConnectionPreferences) -> Result<Option<u32>, String> {
        Ok(self
            .store
            .model_contexts(&preferences.base_url)?
            .get(&preferences.model)
            .copied()
            .flatten())
    }
    pub fn update_request_settings(&mut self, settings: RequestSettings) -> Result<(), String> {
        settings.validate()?;
        let provider = match self.active_key.as_ref() {
            Some(key) => {
                let preferences = self.store.preferences()?;
                if self.active_base_url.as_deref() != Some(preferences.base_url.as_str()) {
                    return Err(
                        "Connection settings changed. Reconnect before changing request settings."
                            .into(),
                    );
                }
                Some(Arc::new(
                    OpenAiProvider::with_settings(
                        &preferences,
                        key.clone(),
                        self.store
                            .model_request_settings(&preferences.base_url)?
                            .get(&preferences.model)
                            .copied()
                            .unwrap_or(settings),
                    )?
                    .with_context_window(self.context_window(&preferences)?),
                ) as Arc<dyn ModelProvider>)
            }
            None => None,
        };
        self.store.save_request_settings(&settings)?;
        self.provider = provider;
        Ok(())
    }
    pub fn update_model_request_settings(
        &mut self,
        preferences: ConnectionPreferences,
        settings: Option<RequestSettings>,
    ) -> Result<(), String> {
        dolores_provider_openai::validate_preferences(&preferences)?;
        if let Some(settings) = settings {
            settings.validate()?;
        }
        let current = self.store.preferences()?;
        if preferences.base_url != current.base_url
            || !self.model_choices()?.contains(&preferences.model)
        {
            return Err(
                "Model connection changed. Reopen generation settings for an enabled model.".into(),
            );
        }
        let provider = if current == preferences {
            match self.active_key.as_ref() {
                Some(key) if self.active_base_url.as_deref() == Some(current.base_url.as_str()) => {
                    Some(Arc::new(
                        OpenAiProvider::with_settings(
                            &current,
                            key.clone(),
                            settings.unwrap_or(self.store.request_settings()?),
                        )?
                        .with_context_window(self.context_window(&current)?),
                    ) as Arc<dyn ModelProvider>)
                }
                Some(_) => {
                    return Err(
                        "Connection changed. Reconnect before saving generation settings.".into(),
                    )
                }
                None => None,
            }
        } else {
            self.provider.clone()
        };
        self.store
            .save_model_request_settings(&preferences, settings.as_ref())?;
        self.provider = provider;
        Ok(())
    }
    pub async fn list_models(
        &self,
        base_url: String,
        api_key: Option<String>,
    ) -> Result<Vec<String>, String> {
        validate_base_url(&base_url)?;
        // Reuse an active launch-only key without sending it back to Dart.
        if api_key.is_none() && self.active_base_url.as_deref() == Some(base_url.as_str()) {
            if let Some(provider) = &self.provider {
                return provider.list_models().await;
            }
        }
        let key = match api_key {
            Some(key) => key,
            None => match self.store.remembered_connection()? {
                Some(saved) if saved.credential_id.is_some() => {
                    if saved.preferences.base_url != base_url {
                        return Err(
                            "Enter a key for the new endpoint before fetching models.".into()
                        );
                    }
                    self.read_key(saved.credential_id.as_deref().unwrap(), &base_url)?
                }
                _ => String::new(),
            },
        };
        let provider = OpenAiProvider::new(
            &ConnectionPreferences {
                base_url,
                model: "discovery".into(),
            },
            key,
        )?;
        provider.list_models().await
    }
    pub fn select_model(&mut self, model: String) -> Result<(), String> {
        validate_model(&model)?;
        let models = self.model_choices()?;
        if !models.contains(&model) {
            return Err("Enable this model in Model connection first.".into());
        }
        if self.provider.is_none() {
            return Err("Reconnect your model first.".into());
        }
        let mut preferences = self.store.preferences()?;
        if self.active_base_url.as_deref() != Some(preferences.base_url.as_str()) {
            return Err("Connection settings changed. Reconnect before switching models.".into());
        }
        let mut saved = self.store.remembered_connection()?;
        if saved
            .as_ref()
            .is_some_and(|record| record.preferences != preferences)
        {
            return Err("Connection settings changed. Reconnect before switching models.".into());
        }
        preferences.model = model;
        let provider = Arc::new(
            OpenAiProvider::with_settings(
                &preferences,
                self.active_key
                    .clone()
                    .ok_or("Reconnect your model first.")?,
                self.store.effective_request_settings(&preferences)?,
            )?
            .with_context_window(self.context_window(&preferences)?),
        );
        if let Some(record) = &mut saved {
            record.preferences = preferences.clone();
        }
        self.store
            .save_connection_models(&preferences, saved.as_ref(), &models)?;
        self.provider = Some(provider);
        Ok(())
    }
    pub fn forget(&mut self) -> Result<(), String> {
        let saved = self.store.remembered_connection()?;
        self.remembered = saved.is_some();
        if let Some(id) = saved.and_then(|record| record.credential_id) {
            self.credentials.delete(&id)?;
        }
        self.provider = None;
        self.active_key = None;
        self.active_base_url = None;
        self.has_key = false;
        let preferences = self.store.preferences()?;
        let models = self.model_choices()?;
        if self
            .store
            .save_connection_models(&preferences, None, &models)
            .is_err()
        {
            self.warning = Some(
                "Secure key removed, but local connection metadata could not be cleared.".into(),
            );
            return Err(self.warning.clone().unwrap());
        }
        self.remembered = false;
        self.warning = None;
        Ok(())
    }
    pub fn descriptor(&self) -> dolores_core::PluginDescriptor {
        self.credentials.descriptor()
    }
}

fn validate_choices(models: Vec<String>, active: &str) -> Result<Vec<String>, String> {
    if models.is_empty() || models.len() > 32 {
        return Err("Choose between 1 and 32 models.".into());
    }
    let mut choices = Vec::new();
    for model in models {
        validate_model(&model)?;
        let model = model.trim().to_string();
        if !choices.contains(&model) {
            choices.push(model);
        }
    }
    if !choices.iter().any(|model| model == active.trim()) {
        return Err("Choose an enabled model as the active model.".into());
    }
    Ok(choices)
}

#[cfg(test)]
pub(crate) mod testing {
    use super::*;
    use std::{
        collections::HashMap,
        sync::{
            atomic::{AtomicBool, Ordering},
            Mutex,
        },
    };
    #[derive(Default)]
    pub struct MemoryCredentials {
        pub values: Mutex<HashMap<String, String>>,
        pub locked: AtomicBool,
    }
    impl CredentialStore for MemoryCredentials {
        fn descriptor(&self) -> dolores_core::PluginDescriptor {
            dolores_core::PluginDescriptor {
                id: "test.credentials",
                kind: "credentials",
                api_version: 1,
            }
        }
        fn read(&self, id: &str) -> Result<Option<String>, String> {
            if self.locked.load(Ordering::Relaxed) {
                return Err("Secure storage unavailable".into());
            }
            Ok(self.values.lock().unwrap().get(id).cloned())
        }
        fn write(&self, id: &str, value: &str) -> Result<(), String> {
            if self.locked.load(Ordering::Relaxed) {
                return Err("Secure storage unavailable".into());
            }
            self.values.lock().unwrap().insert(id.into(), value.into());
            Ok(())
        }
        fn delete(&self, id: &str) -> Result<(), String> {
            if self.locked.load(Ordering::Relaxed) {
                return Err("Secure storage unavailable".into());
            }
            self.values.lock().unwrap().remove(id);
            Ok(())
        }
    }
}
#[cfg(test)]
mod tests {
    use super::testing::MemoryCredentials;
    use super::*;
    use dolores_store_sqlite::SqliteStore;
    use std::sync::atomic::Ordering;
    fn preferences(endpoint: &str) -> ConnectionPreferences {
        ConnectionPreferences {
            base_url: endpoint.into(),
            model: "fixture".into(),
        }
    }
    #[tokio::test]
    async fn profiles_switch_restore_reset_and_failed_save_without_rotating_credentials() {
        let store = Arc::new(SqliteStore::open(std::path::Path::new(":memory:")).unwrap());
        let vault = Arc::new(MemoryCredentials::default());
        let mut manager = ConnectionManager::new(store.clone(), vault.clone());
        let preferences = preferences("https://example.com/v1");
        manager
            .configure_models(
                preferences.clone(),
                Some("fixture-key".into()),
                true,
                Some(vec!["fixture".into(), "other".into()]),
            )
            .unwrap();
        let remembered = store.remembered_connection().unwrap();
        let profile = RequestSettings {
            max_output_tokens: 8192,
            timeout_seconds: 300,
            reasoning: dolores_core::ReasoningControl::DeepseekThinkingOff,
        };
        manager
            .update_model_request_settings(preferences.clone(), Some(profile))
            .unwrap();
        assert_eq!(
            manager.provider.as_ref().unwrap().request_settings(),
            Some(profile)
        );
        let review = manager
            .review_provider()
            .unwrap()
            .request_settings()
            .unwrap();
        assert_eq!(
            (
                review.max_output_tokens,
                review.timeout_seconds,
                review.reasoning
            ),
            (1024, 30, profile.reasoning)
        );
        assert_eq!(
            manager
                .skill_draft_provider(None)
                .unwrap()
                .request_settings(),
            Some(profile)
        );
        manager.select_model("other".into()).unwrap();
        assert_eq!(
            manager.provider.as_ref().unwrap().request_settings(),
            Some(RequestSettings::default())
        );
        manager.select_model("fixture".into()).unwrap();
        let mut restarted = ConnectionManager::new(store.clone(), vault.clone());
        restarted.recover().unwrap();
        assert_eq!(
            restarted.provider.as_ref().unwrap().request_settings(),
            Some(profile)
        );
        assert_eq!(
            serde_json::to_value(store.remembered_connection().unwrap()).unwrap(),
            serde_json::to_value(remembered).unwrap()
        );
        assert_eq!(vault.values.lock().unwrap().len(), 1);
        let wrong = ConnectionPreferences {
            base_url: "https://other.example/v1".into(),
            ..preferences.clone()
        };
        assert!(restarted
            .update_model_request_settings(wrong, Some(profile))
            .is_err());
        assert_eq!(
            restarted.provider.as_ref().unwrap().request_settings(),
            Some(profile)
        );
        restarted
            .update_request_settings(RequestSettings {
                max_output_tokens: 4096,
                ..Default::default()
            })
            .unwrap();
        assert_eq!(
            restarted.provider.as_ref().unwrap().request_settings(),
            Some(profile)
        );
        restarted
            .update_model_request_settings(preferences, None)
            .unwrap();
        assert_eq!(
            restarted
                .provider
                .as_ref()
                .unwrap()
                .request_settings()
                .unwrap()
                .max_output_tokens,
            4096
        );
    }
    #[tokio::test]
    async fn request_settings_rebuild_active_provider_preserve_credentials_and_restore_on_restart()
    {
        let store = Arc::new(SqliteStore::open(std::path::Path::new(":memory:")).unwrap());
        let vault = Arc::new(MemoryCredentials::default());
        let mut manager = ConnectionManager::new(store.clone(), vault.clone());
        let settings = RequestSettings {
            max_output_tokens: 8192,
            timeout_seconds: 300,
            reasoning: Default::default(),
        };
        manager.update_request_settings(settings).unwrap();
        assert!(manager.provider.is_none());
        manager
            .configure_models(
                preferences("https://example.com/v1"),
                Some("fixture-key".into()),
                true,
                Some(vec!["fixture".into(), "other".into()]),
            )
            .unwrap();
        let saved = store.remembered_connection().unwrap().unwrap();
        assert_eq!(
            manager.provider.as_ref().unwrap().request_settings(),
            Some(settings)
        );
        let updated = RequestSettings {
            max_output_tokens: 4096,
            timeout_seconds: 120,
            reasoning: Default::default(),
        };
        manager.update_request_settings(updated).unwrap();
        assert_eq!(
            store
                .remembered_connection()
                .unwrap()
                .unwrap()
                .credential_id,
            saved.credential_id
        );
        assert_eq!(vault.values.lock().unwrap().len(), 1);
        manager.select_model("other".into()).unwrap();
        assert_eq!(
            manager.provider.as_ref().unwrap().request_settings(),
            Some(updated)
        );
        let mut restarted = ConnectionManager::new(store.clone(), vault.clone());
        restarted.recover().unwrap();
        assert_eq!(
            restarted.provider.as_ref().unwrap().request_settings(),
            Some(updated)
        );
        restarted.forget().unwrap();
        assert_eq!(store.request_settings().unwrap(), updated);
        assert!(vault.values.lock().unwrap().is_empty());
    }
    #[tokio::test]
    async fn model_switch_persists_choices_preserves_vault_and_handles_legacy_metadata() {
        let store = Arc::new(SqliteStore::open(std::path::Path::new(":memory:")).unwrap());
        let vault = Arc::new(MemoryCredentials::default());
        let mut manager = ConnectionManager::new(store.clone(), vault.clone());
        let mut prefs = preferences("https://example.com/v1");
        manager
            .configure_models(
                prefs.clone(),
                Some("fixture-key".into()),
                true,
                Some(vec!["fixture".into(), "other".into()]),
            )
            .unwrap();
        let id = store
            .remembered_connection()
            .unwrap()
            .unwrap()
            .credential_id;
        manager.select_model("other".into()).unwrap();
        assert!(manager.select_model("disabled".into()).is_err());
        assert_eq!(store.preferences().unwrap().model, "other");
        assert_eq!(
            store
                .remembered_connection()
                .unwrap()
                .unwrap()
                .credential_id,
            id
        );
        assert_eq!(vault.values.lock().unwrap().len(), 1);
        let mut restarted = ConnectionManager::new(store.clone(), vault.clone());
        restarted.recover().unwrap();
        assert!(restarted.provider.is_some());
        assert_eq!(restarted.model_choices().unwrap(), ["fixture", "other"]);
        assert!(manager
            .configure_models(prefs.clone(), None, true, Some(vec!["not-active".into()]))
            .is_err());
        // Launch-only credentials also survive model changes/settings edits in this process.
        manager
            .configure_models(
                prefs.clone(),
                Some("launch-key".into()),
                false,
                Some(vec!["fixture".into(), "other".into()]),
            )
            .unwrap();
        manager.select_model("other".into()).unwrap();
        prefs.model = "other".into();
        manager
            .configure_models(
                prefs,
                None,
                false,
                Some(vec!["fixture".into(), "other".into()]),
            )
            .unwrap();
        assert_eq!(manager.active_key.as_deref(), Some("launch-key"));
        assert!(vault.values.lock().unwrap().is_empty());
        let changed = preferences("https://different.example/v1");
        store.save_preferences(&changed).unwrap();
        assert!(manager.select_model("fixture".into()).is_err());
        manager
            .configure_models(changed, None, false, None)
            .unwrap();
        assert_eq!(manager.active_key.as_deref(), Some(""));
    }
    #[tokio::test]
    async fn database_failures_clean_new_key_and_keep_failed_forget_recoverable() {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("test.db");
        let store = Arc::new(SqliteStore::open(&path).unwrap());
        let vault = Arc::new(MemoryCredentials::default());
        let original = preferences("https://example.com/v1");
        let mut manager = ConnectionManager::new(store.clone(), vault.clone());
        manager
            .configure(original.clone(), Some("old-key".into()), true)
            .unwrap();
        let old = store.remembered_connection().unwrap().unwrap();
        let connection = rusqlite::Connection::open(&path).unwrap();
        connection.execute_batch("CREATE TRIGGER refuse_save BEFORE INSERT ON remembered_connection BEGIN SELECT RAISE(ABORT, 'fixture failure'); END;").unwrap();
        assert!(manager
            .configure(
                preferences("https://other.example/v1"),
                Some("new-key".into()),
                true
            )
            .is_err());
        assert_eq!(store.preferences().unwrap(), original);
        assert_eq!(
            store
                .remembered_connection()
                .unwrap()
                .unwrap()
                .credential_id,
            old.credential_id
        );
        assert_eq!(vault.values.lock().unwrap().len(), 1);
        assert!(manager.provider.is_some());
        connection.execute_batch("DROP TRIGGER refuse_save; CREATE TRIGGER refuse_delete BEFORE DELETE ON remembered_connection BEGIN SELECT RAISE(ABORT, 'fixture failure'); END;").unwrap();
        assert!(manager.forget().is_err());
        assert!(manager.provider.is_none() && !manager.has_key && manager.remembered);
        assert!(manager.warning.is_some());
        assert!(vault.values.lock().unwrap().is_empty());
        assert!(store.remembered_connection().unwrap().is_some());
        connection
            .execute_batch("DROP TRIGGER refuse_delete;")
            .unwrap();
        manager.forget().unwrap();
        assert!(!manager.remembered && store.remembered_connection().unwrap().is_none());
    }
    #[tokio::test]
    async fn remembers_key_restores_connection_and_forgets_without_deleting_history() {
        let store = Arc::new(SqliteStore::open(std::path::Path::new(":memory:")).unwrap());
        let vault = Arc::new(MemoryCredentials::default());
        store.create("s").unwrap();
        store.commit_turn("s", "question", "answer").unwrap();
        let mut first = ConnectionManager::new(store.clone(), vault.clone());
        first
            .configure(
                preferences("https://example.com/v1"),
                Some("fixture-test-key".into()),
                true,
            )
            .unwrap();
        let record = store.remembered_connection().unwrap().unwrap();
        assert!(!serde_json::to_string(&record)
            .unwrap()
            .contains("fixture-test-key"));
        let mut restarted = ConnectionManager::new(store.clone(), vault.clone());
        restarted.recover().unwrap();
        assert!(restarted.provider.is_some() && restarted.has_key);
        restarted
            .configure(preferences("https://example.com/v1"), None, true)
            .unwrap();
        assert_eq!(vault.values.lock().unwrap().len(), 1);
        restarted.forget().unwrap();
        assert!(vault.values.lock().unwrap().is_empty());
        assert!(store.remembered_connection().unwrap().is_none());
        assert_eq!(store.messages("s").unwrap().len(), 2);
        let mut again = ConnectionManager::new(store, vault);
        again.recover().unwrap();
        assert!(again.provider.is_none());
    }
    #[tokio::test]
    async fn keyless_restores_without_vault_and_session_only_never_saves_key() {
        let store = Arc::new(SqliteStore::open(std::path::Path::new(":memory:")).unwrap());
        let vault = Arc::new(MemoryCredentials::default());
        vault.locked.store(true, Ordering::Relaxed);
        let mut manager = ConnectionManager::new(store.clone(), vault.clone());
        manager
            .configure(
                preferences("http://localhost:19421/v1"),
                Some(String::new()),
                true,
            )
            .unwrap();
        let mut restarted = ConnectionManager::new(store.clone(), vault.clone());
        restarted.recover().unwrap();
        assert!(restarted.provider.is_some() && !restarted.has_key);
        manager
            .configure(
                preferences("https://example.com/v1"),
                Some("launch-only-key".into()),
                false,
            )
            .unwrap();
        assert!(store.remembered_connection().unwrap().is_none());
        assert!(vault.values.lock().unwrap().is_empty());
        let mut again = ConnectionManager::new(store, vault);
        again.recover().unwrap();
        assert!(again.provider.is_none());
    }
    #[tokio::test]
    async fn endpoint_changes_never_reuse_keys_and_tampered_binding_does_not_restore() {
        let store = Arc::new(SqliteStore::open(std::path::Path::new(":memory:")).unwrap());
        let vault = Arc::new(MemoryCredentials::default());
        let mut manager = ConnectionManager::new(store.clone(), vault.clone());
        let original = preferences("https://example.com/v1");
        manager
            .configure(original.clone(), Some("secret".into()), true)
            .unwrap();
        let different = preferences("https://other.example/v1");
        assert!(manager.configure(different.clone(), None, true).is_err());
        assert_eq!(store.preferences().unwrap(), original);
        let mut record = store.remembered_connection().unwrap().unwrap();
        record.preferences = different.clone();
        store.save_connection(&different, Some(&record)).unwrap();
        let mut restarted = ConnectionManager::new(store, vault);
        assert!(restarted.recover().is_err());
        assert!(restarted.provider.is_none());
        assert!(!restarted.warning.unwrap().contains("secret"));
    }
    #[tokio::test]
    async fn locked_or_missing_vault_preserves_history_and_failed_save_preserves_connection() {
        let store = Arc::new(SqliteStore::open(std::path::Path::new(":memory:")).unwrap());
        let vault = Arc::new(MemoryCredentials::default());
        let mut manager = ConnectionManager::new(store.clone(), vault.clone());
        let original = preferences("https://example.com/v1");
        manager
            .configure(original.clone(), Some("old-key".into()), true)
            .unwrap();
        vault.locked.store(true, Ordering::Relaxed);
        assert!(manager
            .configure(
                preferences("https://other.example/v1"),
                Some("new-key".into()),
                true
            )
            .is_err());
        assert_eq!(store.preferences().unwrap(), original);
        assert!(manager.provider.is_some());
        assert!(manager.forget().is_err());
        let mut restarted = ConnectionManager::new(store.clone(), vault.clone());
        assert!(restarted.recover().is_err());
        assert!(restarted.provider.is_none());
        assert!(store.list().is_ok());
        vault.locked.store(false, Ordering::Relaxed);
        restarted.recover().unwrap();
        assert!(restarted.provider.is_some());
        vault.values.lock().unwrap().clear();
        assert!(restarted.recover().is_err());
        assert!(restarted.provider.is_none());
        manager
            .configure(original, Some("memory-key".into()), false)
            .unwrap();
        assert!(manager.provider.is_some());
        assert!(store.remembered_connection().unwrap().is_none());
    }

    #[tokio::test]
    async fn model_contexts_switch_restore_prune_and_failed_save_are_atomic() {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("test.db");
        let store = Arc::new(SqliteStore::open(&path).unwrap());
        let vault = Arc::new(MemoryCredentials::default());
        let prefs = preferences("https://example.com/v1");
        let models = vec!["fixture".into(), "other".into()];
        let contexts = ModelContexts::from([
            ("fixture".into(), Some(32768)),
            ("other".into(), Some(8192)),
        ]);
        let mut manager = ConnectionManager::new(store.clone(), vault.clone());
        manager
            .configure_model_contexts(
                prefs.clone(),
                Some("old-key".into()),
                true,
                Some(models.clone()),
                Some(contexts.clone()),
            )
            .unwrap();
        assert_eq!(
            manager.provider.as_ref().unwrap().context_window_tokens(),
            Some(32768)
        );
        let before = store
            .remembered_connection()
            .unwrap()
            .unwrap()
            .credential_id;
        manager.select_model("other".into()).unwrap();
        assert_eq!(
            manager.provider.as_ref().unwrap().context_window_tokens(),
            Some(8192)
        );
        manager
            .update_request_settings(RequestSettings {
                max_output_tokens: 128,
                ..RequestSettings::default()
            })
            .unwrap();
        assert_eq!(
            manager.provider.as_ref().unwrap().context_window_tokens(),
            Some(8192)
        );
        assert_eq!(
            store
                .remembered_connection()
                .unwrap()
                .unwrap()
                .credential_id,
            before
        );
        let mut restarted =
            ConnectionManager::new(Arc::new(SqliteStore::open(&path).unwrap()), vault.clone());
        restarted.recover().unwrap();
        assert_eq!(
            restarted.provider.as_ref().unwrap().context_window_tokens(),
            Some(8192)
        );
        let db = rusqlite::Connection::open(&path).unwrap();
        db.execute_batch("CREATE TRIGGER refuse_context BEFORE INSERT ON model_contexts BEGIN SELECT RAISE(ABORT, 'fixture failure'); END;").unwrap();
        assert!(manager
            .configure_model_contexts(
                prefs.clone(),
                Some("new-key".into()),
                true,
                Some(models.clone()),
                Some(ModelContexts::from([("fixture".into(), Some(65536))]))
            )
            .is_err());
        assert_eq!(store.preferences().unwrap().model, "other");
        assert_eq!(store.model_contexts(&prefs.base_url).unwrap(), contexts);
        assert_eq!(vault.values.lock().unwrap().len(), 1);
        assert_eq!(
            manager.provider.as_ref().unwrap().context_window_tokens(),
            Some(8192)
        );
        db.execute_batch("DROP TRIGGER refuse_context;").unwrap();
        assert!(manager
            .configure_model_contexts(
                prefs.clone(),
                None,
                true,
                Some(models.clone()),
                Some(ModelContexts::from([("fixture".into(), Some(0))]))
            )
            .is_err());
        assert_eq!(vault.values.lock().unwrap().len(), 1);
        manager
            .configure_model_contexts(
                prefs.clone(),
                None,
                true,
                Some(vec!["fixture".into()]),
                Some(ModelContexts::from([("fixture".into(), None)])),
            )
            .unwrap();
        assert_eq!(
            manager.provider.as_ref().unwrap().context_window_tokens(),
            Some(dolores_core::DEFAULT_CONTEXT_WINDOW_TOKENS)
        );
        assert!(!store
            .model_contexts(&prefs.base_url)
            .unwrap()
            .contains_key("other"));
        manager
            .configure(
                preferences("https://different.example/v1"),
                Some("".into()),
                false,
            )
            .unwrap();
        assert!(manager.model_contexts().unwrap().is_empty());
        assert!(store.model_contexts(&prefs.base_url).unwrap().is_empty());
    }
}
