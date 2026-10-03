use super::*;
use std::collections::BTreeMap;

impl SqliteStore {
    pub(super) fn read_generation_profiles(
        &self,
        base_url: &str,
    ) -> Result<BTreeMap<String, RequestSettings>, String> {
        let connection = self.lock()?;
        let mut statement = connection
            .prepare(
                "SELECT model,data FROM model_request_settings WHERE base_url=?1 ORDER BY model",
            )
            .map_err(storage_error)?;
        let rows = statement
            .query_map([base_url], |row| {
                Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
            })
            .map_err(storage_error)?;
        let mut profiles = BTreeMap::new();
        for row in rows {
            let (model, data) = row.map_err(storage_error)?;
            let settings: RequestSettings = serde_json::from_str(&data).map_err(|_| {
                "Saved model generation settings could not be read. Reset this model's profile."
                    .to_string()
            })?;
            settings.validate()?;
            profiles.insert(model, settings);
        }
        Ok(profiles)
    }
    pub(super) fn write_generation_profile(
        &self,
        preferences: &ConnectionPreferences,
        settings: Option<&RequestSettings>,
    ) -> Result<(), String> {
        if let Some(settings) = settings {
            settings.validate()?;
        }
        let mut connection = self.lock()?;
        let transaction = connection.transaction().map_err(storage_error)?;
        // Bind a stale dialog to the exact endpoint and enabled model set.
        let enabled: Option<String> = transaction
            .query_row(
                "SELECT models FROM model_choices WHERE id=1 AND base_url=?1",
                [&preferences.base_url],
                |r| r.get(0),
            )
            .optional()
            .map_err(storage_error)?;
        let models: Vec<String> = enabled
            .map(|data| serde_json::from_str(&data).map_err(storage_error))
            .transpose()?
            .unwrap_or_default();
        if !models.contains(&preferences.model) {
            return Err(
                "Model connection changed. Reopen generation settings for an enabled model.".into(),
            );
        }
        match settings {
            Some(settings) => {
                let data = serde_json::to_string(settings).map_err(storage_error)?;
                transaction.execute("INSERT INTO model_request_settings(base_url,model,data) VALUES(?1,?2,?3) ON CONFLICT(base_url,model) DO UPDATE SET data=excluded.data", params![preferences.base_url, preferences.model, data]).map_err(storage_error)?;
            }
            None => {
                transaction
                    .execute(
                        "DELETE FROM model_request_settings WHERE base_url=?1 AND model=?2",
                        params![preferences.base_url, preferences.model],
                    )
                    .map_err(storage_error)?;
            }
        }
        transaction.commit().map_err(storage_error)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn endpoint_model_profiles_survive_restart_reset_and_failed_or_stale_writes() {
        let dir = tempfile::tempdir().unwrap();
        let file = dir.path().join("profiles.db");
        let store = SqliteStore::open(&file).unwrap();
        let preferences = ConnectionPreferences {
            base_url: "http://localhost:1234/v1".into(),
            model: "one".into(),
        };
        let models = vec!["one".into(), "two".into()];
        store
            .save_connection_models(&preferences, None, &models)
            .unwrap();
        let settings = RequestSettings {
            max_output_tokens: 8192,
            timeout_seconds: 300,
            reasoning: dolores_core::ReasoningControl::DeepseekThinkingOff,
        };
        store
            .save_model_request_settings(&preferences, Some(&settings))
            .unwrap();
        assert_eq!(
            store.effective_request_settings(&preferences).unwrap(),
            settings
        );
        let second = ConnectionPreferences {
            model: "two".into(),
            ..preferences.clone()
        };
        assert_eq!(
            store.effective_request_settings(&second).unwrap(),
            RequestSettings::default()
        );
        let other = ConnectionPreferences {
            base_url: "http://localhost:5678/v1".into(),
            ..preferences.clone()
        };
        assert!(store
            .model_request_settings(&other.base_url)
            .unwrap()
            .is_empty());
        assert!(store
            .save_model_request_settings(&other, Some(&settings))
            .is_err());
        store.lock().unwrap().execute_batch("CREATE TRIGGER reject_profile BEFORE UPDATE ON model_request_settings BEGIN SELECT RAISE(ABORT,'fixture'); END;").unwrap();
        assert!(store
            .save_model_request_settings(&preferences, Some(&RequestSettings::default()))
            .is_err());
        assert_eq!(
            store.effective_request_settings(&preferences).unwrap(),
            settings
        );
        drop(store);
        let store = SqliteStore::open(&file).unwrap();
        assert_eq!(
            store.effective_request_settings(&preferences).unwrap(),
            settings
        );
        store
            .save_model_request_settings(&preferences, None)
            .unwrap();
        assert_eq!(
            store.effective_request_settings(&preferences).unwrap(),
            RequestSettings::default()
        );
        // Disabled models cannot be edited by a dialog from the old selection.
        store
            .save_model_request_settings(&second, Some(&settings))
            .unwrap();
        store
            .save_model_request_settings(&preferences, Some(&settings))
            .unwrap();
        store
            .save_connection_models(&preferences, None, &["one".into()])
            .unwrap();
        let profiles = store.model_request_settings(&preferences.base_url).unwrap();
        assert_eq!(profiles.len(), 1);
        assert_eq!(profiles.get("one"), Some(&settings));
        assert!(store
            .save_model_request_settings(&second, Some(&settings))
            .is_err());
    }
    #[test]
    fn v16_migration_preserves_exact_legacy_settings_and_history() {
        let dir = tempfile::tempdir().unwrap();
        let file = dir.path().join("migration.db");
        let store = SqliteStore::open(&file).unwrap();
        store.create("original").unwrap();
        store
            .commit_turn("original", "question 世界", "answer")
            .unwrap();
        store
            .save_request_settings(&RequestSettings {
                max_output_tokens: 4096,
                ..Default::default()
            })
            .unwrap();
        let before = store.messages("original").unwrap();
        let raw: String = store
            .lock()
            .unwrap()
            .query_row("SELECT data FROM request_settings", [], |r| r.get(0))
            .unwrap();
        store
            .lock()
            .unwrap()
            .execute_batch("DROP TABLE model_request_settings;PRAGMA user_version=16;")
            .unwrap();
        drop(store);
        let store = SqliteStore::open(&file).unwrap();
        assert_eq!(store.messages("original").unwrap(), before);
        assert_eq!(
            store
                .lock()
                .unwrap()
                .query_row::<String, _, _>("SELECT data FROM request_settings", [], |r| r.get(0))
                .unwrap(),
            raw
        );
        assert_eq!(
            store
                .lock()
                .unwrap()
                .query_row::<i64, _, _>("PRAGMA user_version", [], |r| r.get(0))
                .unwrap(),
            17
        );
    }
}
