use crate::{storage_error, SqliteStore};
use dolores_core::WebConfiguration;
use rusqlite::{Connection, OptionalExtension};
fn read(conn: &Connection) -> Result<WebConfiguration, String> {
    let text: Option<String> = conn
        .query_row("SELECT data FROM web_configuration WHERE id=1", [], |r| {
            r.get(0)
        })
        .optional()
        .map_err(storage_error)?;
    let value: WebConfiguration = match text {
        Some(text) => serde_json::from_str(&text)
            .map_err(|_| "Web settings are unreadable. Restore a valid data backup.")?,
        None => Default::default(),
    };
    value.validate()?;
    Ok(value)
}
impl SqliteStore {
    pub(crate) fn read_web_configuration(&self) -> Result<WebConfiguration, String> {
        let conn = self.lock()?;
        read(&conn)
    }
    pub(crate) fn write_web_configuration(
        &self,
        revision: u32,
        value: &WebConfiguration,
    ) -> Result<WebConfiguration, String> {
        value.validate()?;
        let mut conn = self.lock()?;
        let tx = conn
            .transaction_with_behavior(rusqlite::TransactionBehavior::Immediate)
            .map_err(storage_error)?;
        if read(&tx)?.revision != revision {
            return Err(
                "Web settings changed. Refresh and review your retained draft before saving."
                    .into(),
            );
        }
        let mut saved = value.clone();
        saved.revision = revision
            .checked_add(1)
            .ok_or("Web settings revision exhausted.")?;
        tx.execute("INSERT INTO web_configuration(id,data) VALUES(1,?1) ON CONFLICT(id) DO UPDATE SET data=excluded.data", [serde_json::to_string(&saved).map_err(storage_error)?]).map_err(storage_error)?;
        tx.commit().map_err(storage_error)?;
        Ok(saved)
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use dolores_core::SessionStore;
    #[test]
    fn defaults_restart_stale_and_failed_saves_preserve_configuration() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("web.db");
        let store = SqliteStore::open(&path).unwrap();
        let mut value = store.web_configuration().unwrap();
        assert!(value.enabled);
        value.enabled = false;
        let saved = store.save_web_configuration(0, &value).unwrap();
        assert!(store
            .save_web_configuration(0, &Default::default())
            .is_err());
        store.lock().unwrap().execute_batch("CREATE TRIGGER reject_web BEFORE UPDATE ON web_configuration BEGIN SELECT RAISE(ABORT,'fixture'); END;").unwrap();
        assert!(store
            .save_web_configuration(1, &Default::default())
            .is_err());
        drop(store);
        assert_eq!(
            SqliteStore::open(&path)
                .unwrap()
                .web_configuration()
                .unwrap(),
            saved
        );
    }
}
