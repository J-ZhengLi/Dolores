use super::{storage_error, SqliteStore};
use dolores_core::planning::PlanningState;
use rusqlite::{params, Connection, OptionalExtension};
pub(super) fn read(c: &Connection, s: &str) -> Result<PlanningState, String> {
    let data: Option<String> = c
        .query_row("SELECT data FROM planning WHERE session=?1", [s], |r| {
            r.get(0)
        })
        .optional()
        .map_err(storage_error)?;
    let state: PlanningState = data
        .map(|s| serde_json::from_str(&s))
        .transpose()
        .map_err(storage_error)?
        .unwrap_or_default();
    state.validate()?;
    Ok(state)
}
impl SqliteStore {
    pub(super) fn planning_save(
        &self,
        s: &str,
        expected: u32,
        state: &PlanningState,
    ) -> Result<PlanningState, String> {
        state.validate()?;
        let mut c = self.lock()?;
        let tx = c.transaction().map_err(storage_error)?;
        let previous = read(&tx, s)?;
        if previous.revision != expected {
            return Err(
                "Plan or context changed. Refresh and compare before saving; your edits remain."
                    .into(),
            );
        }
        let mut next = state.clone();
        next.revision = expected.checked_add(1).ok_or("Plan revision exhausted.")?;
        tx.execute("INSERT INTO planning(session,data) VALUES(?1,?2) ON CONFLICT(session) DO UPDATE SET data=excluded.data",params![s,serde_json::to_string(&next).map_err(storage_error)?]).map_err(storage_error)?;
        if let Some(plan) = &next.accepted {
            tx.execute(
                "INSERT OR IGNORE INTO plan_versions(session,revision,data) VALUES(?1,?2,?3)",
                params![
                    s,
                    plan.revision,
                    serde_json::to_string(plan).map_err(storage_error)?
                ],
            )
            .map_err(storage_error)?;
        }
        tx.commit().map_err(storage_error)?;
        Ok(next)
    }
}
