use super::*;
use dolores_core::ModelDetails;

fn contexts(c: &Connection, base: &str) -> Result<ModelContexts, String> {
    let data: Option<String> = c
        .query_row(
            "SELECT data FROM model_contexts WHERE id=1 AND base_url=?1",
            [base],
            |r| r.get(0),
        )
        .optional()
        .map_err(storage_error)?;
    data.map(|s| serde_json::from_str(&s).map_err(storage_error))
        .transpose()
        .map(|c| c.unwrap_or_default())
}
pub(super) fn read(c: &Connection, p: &ConnectionPreferences) -> Result<ModelDetails, String> {
    let data: Option<String> = c
        .query_row(
            "SELECT data FROM model_request_settings WHERE base_url=?1 AND model=?2",
            params![p.base_url, p.model],
            |r| r.get(0),
        )
        .optional()
        .map_err(storage_error)?;
    Ok(ModelDetails {
        context_window_tokens: contexts(c, &p.base_url)?.get(&p.model).copied().flatten(),
        image_input: attachments::image_models(c, &p.base_url)?.contains(&p.model),
        request_settings: data
            .map(|s| serde_json::from_str(&s).map_err(storage_error))
            .transpose()?,
    })
}
impl SqliteStore {
    pub(super) fn write_model_details(
        &self,
        p: &ConnectionPreferences,
        expected: &ModelDetails,
        details: &ModelDetails,
    ) -> Result<(), String> {
        if let Some(settings) = details.request_settings {
            settings.validate()?;
        }
        let mut c = self.lock()?;
        let tx = c.transaction().map_err(storage_error)?;
        let enabled: Option<String> = tx
            .query_row(
                "SELECT models FROM model_choices WHERE id=1 AND base_url=?1",
                [&p.base_url],
                |r| r.get(0),
            )
            .optional()
            .map_err(storage_error)?;
        let models: Vec<String> = enabled
            .map(|s| serde_json::from_str(&s).map_err(storage_error))
            .transpose()?
            .unwrap_or_default();
        if !models.contains(&p.model) {
            return Err(
                "The connection changed. Reopen details for an enabled model; your edits remain."
                    .into(),
            );
        }
        if read(&tx, p)? != *expected {
            return Err(
                "Model settings changed. Refresh, keep your edits and review before saving again."
                    .into(),
            );
        }
        let mut contexts = contexts(&tx, &p.base_url)?;
        contexts.insert(p.model.clone(), details.context_window_tokens);
        dolores_core::validate_model_contexts(&contexts, &models)?;
        let mut images = attachments::image_models(&tx, &p.base_url)?;
        images.retain(|m| m != &p.model);
        if details.image_input {
            images.push(p.model.clone());
        }
        tx.execute("INSERT INTO model_contexts(id,base_url,data) VALUES(1,?1,?2) ON CONFLICT(id) DO UPDATE SET base_url=excluded.base_url,data=excluded.data", params![p.base_url,serde_json::to_string(&contexts).map_err(storage_error)?]).map_err(storage_error)?;
        attachments::save_image_models(&tx, &p.base_url, &images)?;
        if let Some(settings) = details.request_settings {
            tx.execute("INSERT INTO model_request_settings(base_url,model,data) VALUES(?1,?2,?3) ON CONFLICT(base_url,model) DO UPDATE SET data=excluded.data", params![p.base_url,p.model,serde_json::to_string(&settings).map_err(storage_error)?]).map_err(storage_error)?;
        } else {
            tx.execute(
                "DELETE FROM model_request_settings WHERE base_url=?1 AND model=?2",
                params![p.base_url, p.model],
            )
            .map_err(storage_error)?;
        }
        tx.commit().map_err(storage_error)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn details_are_atomic_stale_safe_and_survive_restart() {
        let dir = tempfile::tempdir().unwrap();
        let file = dir.path().join("models.db");
        let store = SqliteStore::open(&file).unwrap();
        let p = ConnectionPreferences {
            base_url: "http://localhost:1234/v1".into(),
            model: "fixture".into(),
        };
        store
            .save_connection_models(&p, None, &["fixture".into(), "other".into()])
            .unwrap();
        let before = store.model_details(&p).unwrap();
        let updated = ModelDetails {
            context_window_tokens: Some(32768),
            image_input: true,
            request_settings: Some(RequestSettings::default()),
        };
        store.lock().unwrap().execute_batch("CREATE TRIGGER reject_details BEFORE INSERT ON model_request_settings BEGIN SELECT RAISE(ABORT,'fixture'); END;").unwrap();
        assert!(store.save_model_details(&p, &before, &updated).is_err());
        assert_eq!(store.model_details(&p).unwrap(), before);
        store
            .lock()
            .unwrap()
            .execute_batch("DROP TRIGGER reject_details")
            .unwrap();
        store.save_model_details(&p, &before, &updated).unwrap();
        assert!(store
            .save_model_details(&p, &before, &before)
            .unwrap_err()
            .contains("changed"));
        assert_eq!(store.preferences().unwrap(), p);
        drop(store);
        let store = SqliteStore::open(&file).unwrap();
        assert_eq!(store.model_details(&p).unwrap(), updated);
        store.save_model_details(&p, &updated, &before).unwrap();
        assert_eq!(store.model_details(&p).unwrap().context_window_tokens, None);
        let other = ConnectionPreferences {
            model: "other".into(),
            ..p.clone()
        };
        assert_eq!(store.model_details(&other).unwrap(), before);
        store
            .save_connection_models(&other, None, &["other".into()])
            .unwrap();
        assert!(store.save_model_details(&p, &before, &updated).is_err());
    }
}
