use super::*;
use dolores_core::ModState;
fn read(conn: &Connection, root: &str) -> Result<ModState, String> {
    let raw: Option<String> = conn
        .query_row("SELECT data FROM project_mods WHERE root=?1", [root], |r| {
            r.get(0)
        })
        .optional()
        .map_err(storage_error)?;
    let s = raw
        .map(|raw| serde_json::from_str::<ModState>(&raw).map_err(storage_error))
        .transpose()?
        .unwrap_or_default();
    s.validate()?;
    Ok(s)
}
impl SqliteStore {
    pub(super) fn read_mods(&self, root: &str) -> Result<ModState, String> {
        read(&*self.lock()?, root)
    }
    pub(super) fn write_mods(
        &self,
        root: &str,
        revision: u32,
        state: &ModState,
    ) -> Result<ModState, String> {
        if !Path::new(root).is_absolute() || state.revision != revision {
            return Err("Invalid mod scope/revision.".into());
        }
        state.validate()?;
        let mut conn = self.lock()?;
        let tx = conn.transaction().map_err(storage_error)?;
        if read(&tx, root)?.revision != revision {
            return Err("Mods changed. Refresh and test again; baseline retained.".into());
        }
        let mut next = state.clone();
        next.revision = revision.checked_add(1).ok_or("Mod revision exhausted.")?;
        tx.execute("INSERT INTO project_mods(root,data) VALUES(?1,?2) ON CONFLICT(root) DO UPDATE SET data=excluded.data",params![root,serde_json::to_string(&next).map_err(storage_error)?]).map_err(storage_error)?;
        tx.commit().map_err(storage_error)?;
        Ok(next)
    }
    pub(super) fn recover_mods(&self) -> Result<(), String> {
        let mut conn = self.lock()?;
        let tx = conn.transaction().map_err(storage_error)?;
        let roots = {
            let mut stmt = tx
                .prepare("SELECT root FROM project_mods")
                .map_err(storage_error)?;
            let rows = stmt
                .query_map([], |r| r.get::<_, String>(0))
                .map_err(storage_error)?;
            rows.collect::<Result<Vec<_>, _>>().map_err(storage_error)?
        };
        for root in roots {
            // An unreadable project's mod state must not block recovery elsewhere.
            // Its inspector reports the local error; ordinary chats stay usable.
            let Ok(mut state) = read(&tx, &root) else {
                continue;
            };
            if state.pending.take().is_some() {
                state.recovery_receipt = "Interrupted activation discarded on restart. Last working version retained; test again explicitly.".into();
                if state.events.len() < 32 {
                    state.events.push(state.recovery_receipt.clone());
                }
                state.revision = state
                    .revision
                    .checked_add(1)
                    .ok_or("Mod revision exhausted.")?;
                state.validate()?;
                tx.execute(
                    "UPDATE project_mods SET data=?2 WHERE root=?1",
                    params![root, serde_json::to_string(&state).map_err(storage_error)?],
                )
                .map_err(storage_error)?;
            }
        }
        tx.commit().map_err(storage_error)?;
        Ok(())
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn malformed_project_cannot_block_another_projects_pending_recovery() {
        let folder = tempfile::tempdir().unwrap();
        let path = folder.path().join("mods.db");
        let store = SqliteStore::open(&path).unwrap();
        let root = folder.path().to_str().unwrap();
        let id = "a".repeat(64);
        let state = ModState {
            pending: Some(id.clone()),
            versions: vec![dolores_core::ModVersion {
                identity: id,
                manifest: Default::default(),
                source: "(module)".into(),
                baseline: None,
                results: vec![],
                baseline_results: vec![],
                status: "review".into(),
                reason: "fixture".into(),
                model: "fixture".into(),
            }],
            ..Default::default()
        };
        store.save_mod_state(root, 0, &state).unwrap();
        let broken = folder.path().join("broken");
        store
            .lock()
            .unwrap()
            .execute(
                "INSERT INTO project_mods(root,data) VALUES(?1,'malformed fixture')",
                [broken.to_str().unwrap()],
            )
            .unwrap();
        store.recover_mod_activations().unwrap();
        let recovered = store.mod_state(root).unwrap();
        assert!(recovered.pending.is_none() && recovered.active.is_none());
        assert!(recovered
            .recovery_receipt
            .contains("Interrupted activation"));
        assert!(store.mod_state(broken.to_str().unwrap()).is_err());
    }
    #[test]
    fn revision_restart_and_failed_receipt_preserve_pointer() {
        let folder = tempfile::tempdir().unwrap();
        let path = folder.path().join("mods.db");
        let root = folder.path().to_str().unwrap();
        let store = SqliteStore::open(&path).unwrap();
        let state = store.mod_state(root).unwrap();
        let saved = store.save_mod_state(root, 0, &state).unwrap();
        assert!(store.save_mod_state(root, 0, &state).is_err());
        store.lock().unwrap().execute_batch("CREATE TRIGGER reject_mod BEFORE UPDATE ON project_mods BEGIN SELECT RAISE(ABORT,'fixture'); END;").unwrap();
        let mut change = saved.clone();
        change.automatic = true;
        assert!(store.save_mod_state(root, 1, &change).is_err());
        assert_eq!(store.mod_state(root).unwrap(), saved);
        drop(store);
        assert_eq!(
            SqliteStore::open(&path).unwrap().mod_state(root).unwrap(),
            saved
        );
    }
}
