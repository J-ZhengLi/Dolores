use super::*;
use dolores_core::RepairEvaluation;
impl SqliteStore {
    pub(super) fn evaluation_list(
        &self,
        session: &str,
        repair: &str,
    ) -> Result<Vec<RepairEvaluation>, String> {
        let c = self.lock()?;
        let mut stmt=c.prepare("SELECT data FROM repair_evaluations WHERE session=?1 AND repair=?2 ORDER BY rowid DESC LIMIT 4").map_err(storage_error)?;
        let rows = stmt
            .query_map(params![session, repair], |r| r.get::<_, String>(0))
            .map_err(storage_error)?;
        rows.map(|row| {
            let value: RepairEvaluation =
                serde_json::from_str(&row.map_err(storage_error)?).map_err(storage_error)?;
            value.validate()?;
            Ok(value)
        })
        .collect()
    }
    pub(super) fn evaluation_save(&self, value: &RepairEvaluation) -> Result<(), String> {
        value.validate()?;
        let mut c = self.lock()?;
        let tx = c.transaction().map_err(storage_error)?;
        let previous: Option<String> = tx
            .query_row(
                "SELECT data FROM repair_evaluations WHERE id=?1",
                [&value.id],
                |r| r.get(0),
            )
            .optional()
            .map_err(storage_error)?;
        if let Some(raw) = previous {
            let old: RepairEvaluation = serde_json::from_str(&raw).map_err(storage_error)?;
            if old.status != "started"
                || old.session != value.session
                || old.repair_id != value.repair_id
                || old.revision != value.revision
                || old.bundle_id != value.bundle_id
                || old.criteria_id != value.criteria_id
                || old.cargo_id != value.cargo_id
                || old.candidate_ids != value.candidate_ids
                || old.artifact != value.artifact
                || old.package != value.package
                || old.reproduction != value.reproduction
            {
                return Err(
                    "Native trial changed or finished; retain it and prepare a fresh review."
                        .into(),
                );
            }
            tx.execute(
                "UPDATE repair_evaluations SET data=?1 WHERE id=?2",
                params![
                    serde_json::to_string(value).map_err(storage_error)?,
                    value.id
                ],
            )
            .map_err(storage_error)?;
        } else {
            let count: usize = tx
                .query_row(
                    "SELECT COUNT(*) FROM repair_evaluations WHERE session=?1 AND repair=?2",
                    params![value.session, value.repair_id],
                    |r| r.get(0),
                )
                .map_err(storage_error)?;
            let repair: Option<String> = tx
                .query_row(
                    "SELECT data FROM harness_repairs WHERE session=?1 AND id=?2",
                    params![value.session, value.repair_id],
                    |r| r.get(0),
                )
                .optional()
                .map_err(storage_error)?;
            let repair: dolores_core::RepairWorkspace =
                serde_json::from_str(&repair.ok_or("Repair unavailable in this chat.")?)
                    .map_err(storage_error)?;
            if count >= 4 || repair.revision != value.revision || value.status != "started" {
                return Err("Trial allowance or repair revision changed. Existing trials and proposal remain; inspect them before a fresh review.".into());
            }
            tx.execute(
                "INSERT INTO repair_evaluations(id,session,repair,data) VALUES(?1,?2,?3,?4)",
                params![
                    value.id,
                    value.session,
                    value.repair_id,
                    serde_json::to_string(value).map_err(storage_error)?
                ],
            )
            .map_err(storage_error)?;
        }
        tx.commit().map_err(storage_error)
    }
}
