use crate::storage_error;
use dolores_core::ExperimentalPreferences;
use rusqlite::{Connection, OptionalExtension};

pub(super) fn read(c: &Connection) -> Result<ExperimentalPreferences, String> {
    let raw: Option<String> = c
        .query_row(
            "SELECT data FROM experimental_preferences WHERE id=1",
            [],
            |r| r.get(0),
        )
        .optional()
        .map_err(storage_error)?;
    match raw {
        None => Ok(Default::default()),
        Some(s) if s.len() <= 256 => {
            serde_json::from_str(&s).map_err(|_| "Experimental preferences are unreadable.".into())
        }
        _ => Err("Experimental preferences are unreadable.".into()),
    }
}
pub(super) fn save(
    c: &mut Connection,
    value: &ExperimentalPreferences,
) -> Result<ExperimentalPreferences, String> {
    let t = c.transaction().map_err(storage_error)?;
    if read(&t)?.revision != value.revision {
        return Err("Experimental preferences changed. Refresh before retrying.".into());
    }
    let mut next = value.clone();
    next.revision = next
        .revision
        .checked_add(1)
        .ok_or("Preference revision exhausted.")?;
    t.execute("INSERT INTO experimental_preferences(id,data) VALUES(1,?1) ON CONFLICT(id) DO UPDATE SET data=excluded.data", [serde_json::to_string(&next).map_err(storage_error)?]).map_err(storage_error)?;
    t.commit().map_err(storage_error)?;
    Ok(next)
}
#[cfg(test)]
mod tests {
    use crate::SqliteStore;
    use dolores_core::SessionStore;
    #[test]
    fn preferences_restart_stale_and_failed_write_keep_last_value() {
        let d = tempfile::tempdir().unwrap();
        let p = d.path().join("test.db");
        let s = SqliteStore::open(&p).unwrap();
        let mut value = s.experimental_preferences().unwrap();
        assert!(value.multiple_window);
        assert!(!value.prevent_windows_from_locked);
        value.multiple_window = false;
        let next = s.save_experimental_preferences(&value).unwrap();
        assert!(s.save_experimental_preferences(&value).is_err());
        s.lock().unwrap().execute_batch("CREATE TRIGGER reject_experimental BEFORE UPDATE ON experimental_preferences BEGIN SELECT RAISE(ABORT,'fixture'); END;").unwrap();
        let mut failed = next.clone();
        failed.multiple_window = true;
        assert!(s.save_experimental_preferences(&failed).is_err());
        drop(s);
        let s = SqliteStore::open(&p).unwrap();
        assert_eq!(s.experimental_preferences().unwrap(), next);
        assert_eq!(
            s.lock()
                .unwrap()
                .query_row("PRAGMA user_version", [], |r| r.get::<_, i64>(0))
                .unwrap(),
            33
        );
    }
}
