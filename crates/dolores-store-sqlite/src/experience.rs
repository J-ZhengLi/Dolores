use super::{storage_error, SqliteStore};
use dolores_core::ExperienceTrial;
use rusqlite::{params, OptionalExtension};
impl SqliteStore {
    pub(super) fn trials(&self, session: &str) -> Result<Vec<ExperienceTrial>, String> {
        let guard = self.lock()?;
        let mut q=guard.prepare("SELECT data FROM experience_trials WHERE session=?1 ORDER BY created_at DESC LIMIT 20").map_err(storage_error)?;
        let values = q
            .query_map([session], |r| r.get::<_, String>(0))
            .map_err(storage_error)?;
        values
            .map(|v| {
                let data = v.map_err(storage_error)?;
                let t: ExperienceTrial =
                    serde_json::from_str(&data).map_err(|_| "Trial evidence could not be read.")?;
                t.validate()?;
                Ok(t)
            })
            .collect()
    }
    pub(super) fn write_trial(
        &self,
        run: &ExperienceTrial,
        create: bool,
    ) -> Result<ExperienceTrial, String> {
        run.validate()?;
        let mut guard = self.lock()?;
        let tx = guard.transaction().map_err(storage_error)?;
        let old: Option<String> = tx
            .query_row(
                "SELECT data FROM experience_trials WHERE id=?1",
                [&run.id],
                |r| r.get(0),
            )
            .optional()
            .map_err(storage_error)?;
        if create {
            let count: i64 = tx
                .query_row(
                    "SELECT count(*) FROM experience_trials WHERE session=?1",
                    [&run.session],
                    |r| r.get(0),
                )
                .map_err(storage_error)?;
            if old.is_some()
                || run.revision != 0
                || run.status != "running"
                || !run.results.is_empty()
                || count >= 20
            {
                return Err("Trial history full or identity changed. Keep earlier evidence; use a new chat for further trials.".into());
            }
        } else {
            let old: ExperienceTrial = serde_json::from_str(&old.ok_or("Trial no longer exists.")?)
                .map_err(storage_error)?;
            old.validate()?;
            if old.status != "running"
                || old.revision != run.revision
                || !old.same_request(run)
                || run.results.len() < old.results.len()
                || run.results.len() > old.results.len() + 1
                || !run.results.starts_with(&old.results)
            {
                return Err("Trial changed or is terminal. Saved evidence was preserved; start a fresh trial.".into());
            }
        }
        let mut saved = run.clone();
        saved.revision = saved
            .revision
            .checked_add(1)
            .ok_or("Trial revision exhausted.")?;
        tx.execute("INSERT INTO experience_trials(id,session,created_at,data) VALUES(?1,?2,?3,?4) ON CONFLICT(id) DO UPDATE SET data=excluded.data",params![saved.id,saved.session,saved.created_at,serde_json::to_string(&saved).map_err(storage_error)?]).map_err(storage_error)?;
        tx.commit().map_err(storage_error)?;
        Ok(saved)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use dolores_core::{
        ExperienceResult, RequestSettings, SessionStore, SkillDocument, EXPERIENCE_SUITE,
    };
    fn sample() -> ExperienceTrial {
        let d = SkillDocument {
            name: "check".into(),
            description: "Check".into(),
            text: "Check fixture".into(),
        };
        ExperienceTrial {
            id: "trial".into(),
            session: "task".into(),
            revision: 0,
            created_at: 1,
            suite: EXPERIENCE_SUITE.into(),
            model: "fixture".into(),
            settings: RequestSettings {
                max_output_tokens: 1024,
                timeout_seconds: 30,
                ..Default::default()
            },
            source_revision: 1,
            baseline: d.clone(),
            candidate: d,
            status: "running".into(),
            results: vec![],
        }
    }
    #[test]
    fn frozen_append_restart_storage_failure_and_stale_save_preserve_evidence() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("trial.db");
        let store = SqliteStore::open(&path).unwrap();
        store.create("task").unwrap();
        let mut r = store.create_experience_trial(&sample()).unwrap();
        let stale = r.clone();
        r.results.push(ExperienceResult {
            violations: 0,
            case: 0,
            candidate: false,
            complete: true,
            passed: false,
            detail: "Baseline obsolete".into(),
            answer: "Claim".into(),
            elapsed_ms: 1,
            files: Default::default(),
            evidence: None,
        });
        r = store.save_experience_trial(&r).unwrap();
        assert!(store.save_experience_trial(&stale).is_err());
        let mut tampered = r.clone();
        tampered.results[0].passed = true;
        assert!(store.save_experience_trial(&tampered).is_err());
        tampered = r.clone();
        tampered.suite = "candidate-grader".into();
        assert!(store.save_experience_trial(&tampered).is_err());
        store.lock().unwrap().execute_batch("CREATE TRIGGER refuse_trial BEFORE UPDATE ON experience_trials BEGIN SELECT RAISE(ABORT,'fixture'); END;").unwrap();
        tampered = r.clone();
        tampered.status = "stopped".into();
        assert!(store.save_experience_trial(&tampered).is_err());
        drop(store);
        let store = SqliteStore::open(&path).unwrap();
        assert_eq!(store.experience_trials("task").unwrap()[0], r);
        assert!(!r.improved());
    }
}
