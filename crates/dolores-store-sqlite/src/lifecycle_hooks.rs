use super::*;
use dolores_core::lifecycle_hooks::Configuration;
pub(super) fn read(conn: &Connection, root: &str) -> Result<Configuration, String> {
    let raw: Option<String> = conn
        .query_row(
            "SELECT data FROM lifecycle_hooks WHERE root=?1",
            [root],
            |r| r.get(0),
        )
        .optional()
        .map_err(storage_error)?;
    let value = raw
        .map(|s| serde_json::from_str::<Configuration>(&s).map_err(storage_error))
        .transpose()?
        .unwrap_or_default();
    value.validate()?;
    Ok(value)
}
pub(super) fn save(
    conn: &mut Connection,
    root: &str,
    value: &Configuration,
) -> Result<Configuration, String> {
    value.validate()?;
    if !Path::new(root).is_absolute() {
        return Err("Hook folder must be absolute.".into());
    }
    let tx = conn.transaction().map_err(storage_error)?;
    if read(&tx, root)?.revision != value.revision {
        return Err(
            "Hooks changed. Refresh and review again; prior configuration retained.".into(),
        );
    }
    let mut next = value.clone();
    next.revision = next
        .revision
        .checked_add(1)
        .ok_or("Hook revision exhausted.")?;
    tx.execute("INSERT INTO lifecycle_hooks(root,data) VALUES(?1,?2) ON CONFLICT(root) DO UPDATE SET data=excluded.data",params![root,serde_json::to_string(&next).map_err(storage_error)?]).map_err(storage_error)?;
    tx.commit().map_err(storage_error)?;
    Ok(next)
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn migration_revision_and_failed_save_preserve_old_profile() {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("state.db");
        let store = SqliteStore::open(&path).unwrap();
        let root = directory.path().to_str().unwrap();
        store.create("retained-chat").unwrap();
        store
            .lock()
            .unwrap()
            .execute_batch("DROP TABLE lifecycle_hooks; PRAGMA user_version=49;")
            .unwrap();
        drop(store);
        let store = SqliteStore::open(&path).unwrap();
        let value = store.lifecycle_hooks(root).unwrap();
        assert!(value.hooks.is_empty());
        let saved = store.save_lifecycle_hooks(root, &value).unwrap();
        assert!(store.save_lifecycle_hooks(root, &value).is_err());
        store.lock().unwrap().execute_batch("CREATE TRIGGER refuse_hook BEFORE UPDATE ON lifecycle_hooks BEGIN SELECT RAISE(ABORT,'public fixture'); END;").unwrap();
        assert!(store.save_lifecycle_hooks(root, &saved).is_err());
        assert_eq!(store.lifecycle_hooks(root).unwrap(), saved);
        drop(store);
        let reopened = SqliteStore::open(&path).unwrap();
        assert_eq!(reopened.lifecycle_hooks(root).unwrap(), saved);
        assert!(reopened.messages("retained-chat").is_ok());
    }
}
