use super::Engine;
use dolores_core::SearchProvider;
use serde_json::{json, Value};
impl Engine {
    pub(super) fn web_settings(&self) -> Result<Value, String> {
        let c = self.store.web_configuration()?;
        Ok(
            json!({"revision":c.revision,"enabled":c.enabled,"provider":c.provider,"endpoint":c.endpoint,"hasSavedKey":c.credential_id.is_some(),"cleanupPending":!c.retired_credentials.is_empty(),"searchEndpoint":dolores_tools_web::search_endpoint(&c)}),
        )
    }
    pub(super) fn save_web_settings(
        &self,
        revision: u32,
        enabled: bool,
        provider: SearchProvider,
        endpoint: Option<String>,
        api_key: Option<String>,
        clear_key: bool,
    ) -> Result<Value, String> {
        let previous = self.store.web_configuration()?;
        if revision != previous.revision {
            return Err(
                "Web settings changed. Refresh and review your retained draft before saving."
                    .into(),
            );
        }
        if clear_key && api_key.is_some() {
            return Err("Choose key replacement or removal, not both.".into());
        }
        if api_key.is_some() && provider != SearchProvider::Brave {
            return Err("API keys are supported only for the Brave connection.".into());
        }
        let mut config = previous.clone();
        config.enabled = enabled;
        config.provider = provider;
        config.endpoint = endpoint;
        dolores_tools_web::validate_configuration(&config)?;
        config
            .retired_credentials
            .retain(|id| self.mcp_credentials.delete(id).is_err());
        if config.retired_credentials.len() >= 8 {
            return Err("Credential cleanup is pending. Unlock secure storage and Save again before replacing the key.".into());
        }
        let mut new_id = None;
        if let Some(key) = api_key.as_deref() {
            if key.trim().is_empty() || key.len() > 4096 || key.chars().any(char::is_control) {
                return Err("Enter a nonempty API key without control characters.".into());
            }
            let id = uuid::Uuid::new_v4().to_string();
            if self.mcp_credentials.write(&id, key).is_err() {
                let _ = self.mcp_credentials.delete(&id);
                return Err("Web key could not be saved securely. Unlock secure storage and retry; existing settings remain.".into());
            }
            new_id = Some(id);
        }
        if clear_key || new_id.is_some() {
            if let Some(old) = config.credential_id.take() {
                config.retired_credentials.push(old);
            }
            config.credential_id = new_id.clone();
        }
        if enabled && provider == SearchProvider::Brave && config.credential_id.is_none() {
            return Err("Brave requires an API key. Enter it here or choose Default (Mwmbl); your draft is retained.".into());
        }
        if let Err(error) = self.store.save_web_configuration(revision, &config) {
            if let Some(id) = new_id {
                let _ = self.mcp_credentials.delete(&id);
            }
            return Err(error);
        }
        let saved = self.store.web_configuration()?;
        let mut cleaned = saved.clone();
        cleaned
            .retired_credentials
            .retain(|id| self.mcp_credentials.delete(id).is_err());
        if cleaned.retired_credentials != saved.retired_credentials {
            // Retain references on bookkeeping failure so cleanup remains retryable.
            let _ = self.store.save_web_configuration(saved.revision, &cleaned);
        }
        let mut result = self.web_settings()?;
        result["notice"] = json!(if result["cleanupPending"] == true {
            "Settings saved. Old key cleanup is pending; unlock secure storage and Save again."
        } else {
            "Web settings saved for future working runs. No request was sent."
        });
        Ok(result)
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::{connection::testing::MemoryCredentials, Command};
    use dolores_core::{SessionStore, WebConfiguration};
    use dolores_store_sqlite::SqliteStore;
    use std::sync::{atomic::Ordering, Arc};
    fn save(revision: u32, provider: SearchProvider, key: Option<&str>) -> Command {
        Command::SaveWebSettings {
            revision,
            enabled: true,
            provider,
            endpoint: None,
            api_key: key.map(str::to_string),
            clear_key: false,
        }
    }
    #[test]
    fn no_setup_defaults_stale_locked_and_atomic_failure_preserve_saved_key() {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("test.sqlite");
        let store = Arc::new(SqliteStore::open(&path).unwrap());
        let vault = Arc::new(MemoryCredentials::default());
        let engine = Engine::new(store.clone(), vault.clone()).unwrap();
        let initial = engine.call(Command::WebSettings).unwrap();
        assert_eq!(initial["provider"], "mwmbl");
        assert_eq!(initial["enabled"], true);
        assert_eq!(initial["hasSavedKey"], false);
        assert!(engine
            .call(save(0, SearchProvider::Brave, None))
            .unwrap_err()
            .contains("requires an API key"));
        assert!(engine
            .call(save(0, SearchProvider::Mwmbl, Some("fixture-secret")))
            .is_err());
        let saved = engine
            .call(save(0, SearchProvider::Brave, Some("fixture-secret")))
            .unwrap();
        assert!(!saved.to_string().contains("fixture-secret"));
        assert_eq!(saved["revision"], 1);
        let before = store.web_configuration().unwrap();
        assert!(engine
            .call(save(0, SearchProvider::Brave, Some("stale-key")))
            .unwrap_err()
            .contains("Refresh"));
        vault.locked.store(true, Ordering::Relaxed);
        assert!(engine
            .call(save(1, SearchProvider::Brave, Some("locked-key")))
            .is_err());
        vault.locked.store(false, Ordering::Relaxed);
        let connection = rusqlite::Connection::open(&path).unwrap();
        connection.execute_batch("CREATE TRIGGER fail_web BEFORE UPDATE ON web_configuration BEGIN SELECT RAISE(ABORT, 'fixture storage failure'); END;").unwrap();
        assert!(engine
            .call(save(1, SearchProvider::Brave, Some("replacement-key")))
            .is_err());
        assert_eq!(store.web_configuration().unwrap(), before);
        assert_eq!(vault.values.lock().unwrap().len(), 1);
        assert_eq!(
            vault
                .values
                .lock()
                .unwrap()
                .get(before.credential_id.as_ref().unwrap())
                .unwrap(),
            "fixture-secret"
        );
        connection.execute_batch("DROP TRIGGER fail_web;").unwrap();
        let replaced = engine
            .call(save(1, SearchProvider::Brave, Some("replacement-key")))
            .unwrap();
        assert_eq!(replaced["cleanupPending"], false);
        assert_eq!(vault.values.lock().unwrap().len(), 1);
        drop(engine);
        drop(store);
        let restarted = SqliteStore::open(&path).unwrap();
        assert_eq!(
            restarted.web_configuration().unwrap().provider,
            SearchProvider::Brave
        );
    }
    #[test]
    fn disabled_tools_and_side_chat_never_receive_web_catalogue() {
        let store = Arc::new(SqliteStore::open(std::path::Path::new(":memory:")).unwrap());
        store.create("side").unwrap();
        let engine = Engine::new(store.clone(), Arc::new(MemoryCredentials::default())).unwrap();
        let inventory = engine
            .call(Command::HarnessInventory {
                session: Some("side".into()),
            })
            .unwrap();
        assert!(!inventory.to_string().contains("\"id\":\"web_search\""));
        let config = WebConfiguration {
            enabled: false,
            ..Default::default()
        };
        store.save_web_configuration(0, &config).unwrap();
        assert!(dolores_tools_web::specs(&store.web_configuration().unwrap()).is_empty());
        assert_eq!(engine.call(Command::WebSettings).unwrap()["enabled"], false);
    }
}
