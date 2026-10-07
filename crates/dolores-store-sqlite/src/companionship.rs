use crate::{storage_error, SqliteStore};
use dolores_core::companionship::*;
use rusqlite::{Connection, OptionalExtension};
pub(super) fn read(c: &Connection) -> Result<CompanionState, String> {
    let data: Option<String> = c
        .query_row("SELECT data FROM companion_state WHERE id=1", [], |r| {
            r.get(0)
        })
        .optional()
        .map_err(storage_error)?;
    let state: CompanionState = data
        .map(|v| serde_json::from_str(&v).map_err(storage_error))
        .transpose()?
        .unwrap_or_default();
    state.validate()?;
    Ok(state)
}
pub(super) fn write(c: &Connection, state: &CompanionState) -> Result<(), String> {
    state.validate()?;
    c.execute("INSERT INTO companion_state(id,data) VALUES(1,?1) ON CONFLICT(id) DO UPDATE SET data=excluded.data",[serde_json::to_string(state).map_err(storage_error)?]).map_err(storage_error)?;
    Ok(())
}
impl SqliteStore {
    pub(super) fn companion_read(&self) -> Result<CompanionState, String> {
        read(&*self.lock()?)
    }
    pub(super) fn companion_save(
        &self,
        state: &CompanionState,
        revision: u64,
    ) -> Result<CompanionState, String> {
        let mut c = self.lock()?;
        let tx = c.transaction().map_err(storage_error)?;
        let old = read(&tx)?;
        if old.revision != revision {
            return Err("Companionship changed. Refresh before saving.".into());
        }
        let mut next = state.clone();
        next.revision = revision
            .checked_add(1)
            .ok_or("Companion revision exhausted.")?;
        write(&tx, &next)?;
        tx.commit().map_err(storage_error)?;
        Ok(next)
    }
    pub(super) fn companion_claim(
        &self,
        now: i64,
        present: bool,
        busy: bool,
        jitter: u32,
        candidate: CompanionCandidate,
    ) -> Result<Option<CompanionCandidate>, String> {
        let mut c = self.lock()?;
        let tx = c.transaction().map_err(storage_error)?;
        let mut state = read(&tx)?;
        let old = state.clone();
        let claimed = state.reserve(now, present, busy, jitter, candidate.clone())?;
        if old != state {
            state.revision = state
                .revision
                .checked_add(1)
                .ok_or("Companion revision exhausted.")?;
            write(&tx, &state)?;
        }
        tx.commit().map_err(storage_error)?;
        Ok(claimed.then_some(candidate))
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use dolores_core::SessionStore;
    #[test]
    fn restart_retains_daily_cap_and_stale_or_failed_saves_preserve_policy() {
        let d = tempfile::tempdir().unwrap();
        let p = d.path().join("db");
        let s = SqliteStore::open(&p).unwrap();
        let mut state = s.companion_state().unwrap();
        assert!(!state.policy.enabled);
        state.policy.enabled = true;
        state.policy.model = "weak".into();
        state.attempts = 2;
        state.day = "2026-10-07".into();
        let saved = s.save_companion_state(&state, 0).unwrap();
        assert!(s.save_companion_state(&state, 0).is_err());
        s.lock().unwrap().execute_batch("CREATE TRIGGER deny_companion BEFORE UPDATE ON companion_state BEGIN SELECT RAISE(ABORT,'fixture'); END;").unwrap();
        state.policy.enabled = false;
        assert!(s.save_companion_state(&state, saved.revision).is_err());
        drop(s);
        assert_eq!(
            SqliteStore::open(&p).unwrap().companion_state().unwrap(),
            saved
        );
    }
}
