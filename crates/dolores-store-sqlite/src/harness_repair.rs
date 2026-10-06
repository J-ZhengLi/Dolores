use super::*;
use dolores_core::RepairWorkspace;

impl SqliteStore {
    pub(super) fn repair_list(&self, session: &str) -> Result<Vec<String>, String> {
        let c = self.lock()?;
        let mut statement = c
            .prepare("SELECT id FROM harness_repairs WHERE session=?1 ORDER BY rowid DESC LIMIT 4")
            .map_err(storage_error)?;
        let rows = statement
            .query_map([session], |r| r.get(0))
            .map_err(storage_error)?;
        rows.collect::<Result<Vec<_>, _>>().map_err(storage_error)
    }
    pub(super) fn read_repair(&self, session: &str, id: &str) -> Result<RepairWorkspace, String> {
        let raw: Option<String> = self
            .lock()?
            .query_row(
                "SELECT data FROM harness_repairs WHERE session=?1 AND id=?2",
                params![session, id],
                |r| r.get(0),
            )
            .optional()
            .map_err(storage_error)?;
        let state: RepairWorkspace = serde_json::from_str(
            &raw.ok_or("Repair belongs to another chat or is unavailable. Original task remains.")?,
        )
        .map_err(storage_error)?;
        state.validate()?;
        if state.session != session || state.id != id {
            return Err("Repair identity changed; retained work remains.".into());
        }
        Ok(state)
    }
    pub(super) fn write_repair(
        &self,
        state: &RepairWorkspace,
        expected: Option<u32>,
    ) -> Result<RepairWorkspace, String> {
        state.validate()?;
        let mut c = self.lock()?;
        let tx = c.transaction().map_err(storage_error)?;
        let old: Option<String> = tx
            .query_row(
                "SELECT data FROM harness_repairs WHERE session=?1 AND id=?2",
                params![state.session, state.id],
                |r| r.get(0),
            )
            .optional()
            .map_err(storage_error)?;
        let previous = old
            .map(|raw| serde_json::from_str::<RepairWorkspace>(&raw).map_err(storage_error))
            .transpose()?;
        if previous.as_ref().map(|s| s.revision) != expected
            || state.revision != expected.unwrap_or(0)
        {
            return Err("Repair changed since preview. Refresh its retained source and prepare a new proposal.".into());
        }
        if let Some(old) = &previous {
            if old.bundle_id != state.bundle_id
                || old.build != state.build
                || old.files.len() > state.files.len()
                || old.files.iter().any(|f| {
                    !state.files.iter().any(|n| {
                        n.path == f.path && n.source_id == f.source_id && n.before == f.before
                    })
                })
            {
                return Err("Repair baseline cannot be changed. Keep it and prepare a separate matching repair.".into());
            }
        } else {
            let count: usize = tx
                .query_row(
                    "SELECT COUNT(*) FROM harness_repairs WHERE session=?1",
                    [&state.session],
                    |r| r.get(0),
                )
                .map_err(storage_error)?;
            if count >= 4 {
                return Err("This chat has four retained repair workspaces. Inspect them or start another chat; nothing was replaced.".into());
            }
        }
        let mut next = state.clone();
        next.revision = expected
            .unwrap_or(0)
            .checked_add(1)
            .ok_or("Repair revision exhausted.")?;
        let written=tx.execute("INSERT INTO harness_repairs(id,session,data) VALUES(?1,?2,?3) ON CONFLICT(id) DO UPDATE SET data=excluded.data WHERE harness_repairs.session=excluded.session",params![next.id,next.session,serde_json::to_string(&next).map_err(storage_error)?]).map_err(storage_error)?;
        if written != 1 {
            return Err("Repair identity belongs to another chat. Retained work remains.".into());
        }
        tx.commit().map_err(storage_error)?;
        Ok(next)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn changed_baseline_cross_chat_stale_save_and_restart_preserve_snapshot() {
        let temp = tempfile::tempdir().unwrap();
        let path = temp.path().join("state.db");
        let store = SqliteStore::open(&path).unwrap();
        store.create("a").unwrap();
        store.create("b").unwrap();
        let hash = format!("sha256:{}", "a".repeat(64));
        let state = RepairWorkspace {
            id: "00000000-0000-0000-0000-000000000001".into(),
            session: "a".into(),
            revision: 0,
            bundle_id: hash.clone(),
            build: "fixture".into(),
            artifact: "repair-fixture".into(),
            status: "prepared".into(),
            files: vec![dolores_core::RepairFile {
                path: "crates/example.rs".into(),
                source_id: hash.clone(),
                candidate_id: hash,
                before: "baseline".into(),
                after: "baseline".into(),
            }],
        };
        let saved = store.save_repair_workspace(&state, None).unwrap();
        assert!(store.repair_workspace("b", &saved.id).is_err());
        assert!(store.save_repair_workspace(&state, None).is_err());
        let mut changed = saved.clone();
        changed.files[0].before = "weakened baseline".into();
        assert!(store.save_repair_workspace(&changed, Some(1)).is_err());
        assert_eq!(store.repair_workspace("a", &saved.id).unwrap(), saved);
        drop(store);
        assert_eq!(
            SqliteStore::open(&path)
                .unwrap()
                .repair_workspace("a", &saved.id)
                .unwrap(),
            saved
        );
    }
}
