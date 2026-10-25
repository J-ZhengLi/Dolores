use crate::{storage_error, SqliteStore};
use dolores_core::desktop_preferences::ClosePreferences;
use rusqlite::{Connection, OptionalExtension};
fn read(c: &Connection) -> Result<ClosePreferences, String> {
    let data: Option<String> = c
        .query_row(
            "SELECT data FROM desktop_close_preferences WHERE id=1",
            [],
            |r| r.get(0),
        )
        .optional()
        .map_err(storage_error)?;
    data.map(|s| serde_json::from_str(&s).map_err(storage_error))
        .transpose()
        .map(|v| v.unwrap_or_default())
}
impl SqliteStore {
    pub(super) fn read_close_preferences(&self) -> Result<ClosePreferences, String> {
        read(&*self.lock()?)
    }
    pub(super) fn write_close_preferences(
        &self,
        value: &ClosePreferences,
    ) -> Result<ClosePreferences, String> {
        let mut c = self.lock()?;
        let t = c.transaction().map_err(storage_error)?;
        if read(&t)?.revision != value.revision {
            return Err("Close preference changed. Reopen General settings and try again.".into());
        }
        let mut next = value.clone();
        next.revision = next
            .revision
            .checked_add(1)
            .ok_or("Close preference revision limit reached.")?;
        t.execute("INSERT INTO desktop_close_preferences(id,data) VALUES(1,?1) ON CONFLICT(id) DO UPDATE SET data=excluded.data",[serde_json::to_string(&next).map_err(storage_error)?]).map_err(storage_error)?;
        t.commit().map_err(storage_error)?;
        Ok(next)
    }
}
