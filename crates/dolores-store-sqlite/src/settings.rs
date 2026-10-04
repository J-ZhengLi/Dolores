use crate::{storage_error, SqliteStore};
use dolores_core::{ScopedSettings, SettingsPatch, SettingsScope};
use rusqlite::{params, Connection, OptionalExtension};
fn scope_exists(conn: &Connection, scope: SettingsScope, key: &str) -> Result<(), String> {
    let exists = match scope {
        SettingsScope::User => key == "user",
        SettingsScope::Project => conn
            .query_row(
                "SELECT EXISTS(SELECT 1 FROM projects WHERE root=?1)",
                [key],
                |r| r.get(0),
            )
            .map_err(storage_error)?,
        SettingsScope::Thread => conn
            .query_row(
                "SELECT EXISTS(SELECT 1 FROM sessions WHERE id=?1)",
                [key],
                |r| r.get(0),
            )
            .map_err(storage_error)?,
    };
    if !exists {
        return Err("Settings scope is unavailable. Reopen the saved chat or project.".into());
    }
    Ok(())
}
fn read(conn: &Connection, scope: SettingsScope, key: &str) -> Result<ScopedSettings, String> {
    scope_exists(conn, scope, key)?;
    let raw: Option<(u32, String)> = conn
        .query_row(
            "SELECT revision,data FROM scoped_settings WHERE scope=?1 AND scope_key=?2",
            params![scope.key(), key],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )
        .optional()
        .map_err(storage_error)?;
    let result = match raw {
        Some((revision, data)) => ScopedSettings {
            revision,
            patch: serde_json::from_str(&data).map_err(|_| {
                "Saved scoped settings could not be read. Restore a valid data backup."
            })?,
        },
        None => Default::default(),
    };
    result.patch.validate(scope)?;
    Ok(result)
}
impl SqliteStore {
    pub(crate) fn read_scoped_settings(
        &self,
        scope: SettingsScope,
        key: &str,
    ) -> Result<ScopedSettings, String> {
        let conn = self.lock()?;
        read(&conn, scope, key)
    }
    pub(crate) fn write_scoped_settings(
        &self,
        scope: SettingsScope,
        key: &str,
        revision: u32,
        patch: &SettingsPatch,
    ) -> Result<ScopedSettings, String> {
        patch.validate(scope)?;
        let mut conn = self.lock()?;
        let tx = conn
            .transaction_with_behavior(rusqlite::TransactionBehavior::Immediate)
            .map_err(storage_error)?;
        let prior = read(&tx, scope, key)?;
        if prior.revision != revision {
            return Err("Settings changed. Refresh, keep your draft and review it again.".into());
        }
        let revision = revision
            .checked_add(1)
            .ok_or("Settings revision exhausted.")?;
        let data = serde_json::to_string(patch).map_err(storage_error)?;
        tx.execute("INSERT INTO scoped_settings(scope,scope_key,revision,data) VALUES(?1,?2,?3,?4) ON CONFLICT(scope,scope_key) DO UPDATE SET revision=excluded.revision,data=excluded.data",params![scope.key(),key,revision,data]).map_err(storage_error)?;
        tx.commit().map_err(storage_error)?;
        Ok(ScopedSettings {
            revision,
            patch: patch.clone(),
        })
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use dolores_core::{GenerationOverride, SessionStore};
    #[test]
    fn revisions_invalid_and_failed_saves_keep_scoped_values_across_restart_and_delete() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("settings.db");
        let store = SqliteStore::open(&path).unwrap();
        store.create("chat").unwrap();
        store.create("other").unwrap();
        let patch = SettingsPatch {
            task: None,
            generation: Some(GenerationOverride {
                max_output_tokens: 512,
                timeout_seconds: 60,
            }),
            interaction: None,
        };
        store
            .save_scoped_settings(SettingsScope::Thread, "chat", 0, &patch)
            .unwrap();
        assert!(store
            .save_scoped_settings(SettingsScope::Thread, "chat", 0, &Default::default())
            .is_err());
        assert!(store
            .scoped_settings(SettingsScope::Thread, "other")
            .unwrap()
            .patch
            .generation
            .is_none());
        let invalid = SettingsPatch {
            task: None,
            generation: Some(GenerationOverride {
                max_output_tokens: 0,
                timeout_seconds: 0,
            }),
            interaction: None,
        };
        assert!(store
            .save_scoped_settings(SettingsScope::Thread, "chat", 1, &invalid)
            .is_err());
        assert!(store
            .save_scoped_settings(SettingsScope::Project, "missing", 0, &patch)
            .is_err());
        store.lock().unwrap().execute_batch("CREATE TRIGGER reject_settings BEFORE UPDATE ON scoped_settings BEGIN SELECT RAISE(ABORT,'fixture'); END;").unwrap();
        assert!(store
            .save_scoped_settings(SettingsScope::Thread, "chat", 1, &Default::default())
            .is_err());
        drop(store);
        let store = SqliteStore::open(&path).unwrap();
        assert_eq!(
            store
                .scoped_settings(SettingsScope::Thread, "chat")
                .unwrap()
                .patch,
            patch
        );
        store.delete("chat").unwrap();
        assert!(store
            .scoped_settings(SettingsScope::Thread, "chat")
            .is_err());
        let count: i64 = store
            .lock()
            .unwrap()
            .query_row("SELECT count(*) FROM scoped_settings", [], |r| r.get(0))
            .unwrap();
        assert_eq!(count, 0);
    }
}
