use super::{skills, storage_error, SqliteStore};
use dolores_core::*;
use rusqlite::{params, Connection, OptionalExtension};

pub(super) fn read(c: &Connection, root: &str) -> Result<AdaptationState, String> {
    let v: Option<String> = c
        .query_row(
            "SELECT data FROM project_adaptation WHERE root=?1",
            [root],
            |r| r.get(0),
        )
        .optional()
        .map_err(storage_error)?;
    let s: AdaptationState = v
        .map(|v| serde_json::from_str(&v).map_err(storage_error))
        .transpose()?
        .unwrap_or_default();
    s.validate()?;
    Ok(s)
}
fn write(
    c: &Connection,
    root: &str,
    revision: u32,
    s: &AdaptationState,
) -> Result<AdaptationState, String> {
    if !std::path::Path::new(root).is_absolute()
        || read(c, root)?.revision != revision
        || s.revision != revision
    {
        return Err("Learning revision changed. Refresh; no change was replayed.".into());
    }
    s.validate()?;
    let mut saved = s.clone();
    saved.revision = saved
        .revision
        .checked_add(1)
        .ok_or("Learning revision exhausted.")?;
    c.execute("INSERT INTO project_adaptation(root,data) VALUES(?1,?2) ON CONFLICT(root) DO UPDATE SET data=excluded.data",params![root,serde_json::to_string(&saved).map_err(storage_error)?]).map_err(storage_error)?;
    Ok(saved)
}
impl SqliteStore {
    pub(super) fn interrupt_learning(&self) -> Result<(), String> {
        let mut guard = self.lock()?;
        let tx = guard.transaction().map_err(storage_error)?;
        let roots = {
            let mut statement = tx
                .prepare("SELECT root FROM project_adaptation")
                .map_err(storage_error)?;
            let rows = statement
                .query_map([], |r| r.get::<_, String>(0))
                .map_err(storage_error)?;
            rows.collect::<Result<Vec<_>, _>>().map_err(storage_error)?
        };
        for root in roots {
            let mut state = read(&tx, &root)?;
            let mut changed = false;
            for e in &mut state.events {
                if e.status == "pending" {
                    e.status = "interrupted".into();
                    e.reason="Reflection ended before a qualifying decision. Baseline retained; inspect receipts or use Library review. This event is not replayed.".into();
                    changed = true;
                }
                if e.monitor_status == "running" {
                    e.monitor_status = "interrupted".into();
                    changed = true;
                }
            }
            if changed {
                state.notice="Unfinished learning was interrupted. No activation or automatic replay; inspect Learning history.".into();
                write(&tx, &root, state.revision, &state)?;
            }
        }
        tx.commit().map_err(storage_error)
    }
    pub(super) fn restore_learning(
        &self,
        root: &str,
        revision: u32,
        id: &str,
        confirmed: bool,
    ) -> Result<AdaptationState, String> {
        let mut guard = self.lock()?;
        let tx = guard.transaction().map_err(storage_error)?;
        let mut state = read(&tx, root)?;
        if state.revision != revision {
            return Err("Learning history changed. Refresh before restoring.".into());
        }
        if confirmed && (!state.enabled || !state.automatic || state.paused) {
            return Err("Automatic recovery policy changed. Review a manual restore.".into());
        }
        let index = state
            .events
            .iter()
            .position(|e| e.id == id)
            .ok_or("Learning event missing.")?;
        let event = state.events[index].clone();
        if event.status != "active" {
            return Err("Only an active learning update can be restored.".into());
        }
        let baseline = event
            .baseline
            .as_ref()
            .ok_or("Baseline snapshot missing; current skill retained.")?;
        let candidate = event
            .candidate
            .as_ref()
            .ok_or("Candidate snapshot missing.")?;
        let mut skill = skills::read(&tx, root)?
            .into_iter()
            .find(|s| s.name == baseline.name)
            .ok_or("Current skill missing; inspect Library.")?;
        if Some(skill.revision) != event.activated_revision
            || &skill.current().document != candidate
            || skill.enabled != baseline.enabled
        {
            state.events[index].monitor_status = "conflict".into();
            state.notice="Restore conflict: a manual skill change was preserved. Inspect Library before choosing a version.".into();
            let saved = write(&tx, root, revision, &state)?;
            tx.commit().map_err(storage_error)?;
            return Ok(saved);
        }
        if confirmed {
            let row: String = tx
                .query_row(
                    "SELECT data FROM experience_trials WHERE id=?1",
                    [event
                        .monitor_trial_id
                        .as_ref()
                        .ok_or("Regression evidence missing.")?],
                    |r| r.get(0),
                )
                .map_err(storage_error)?;
            let trial: ExperienceTrial = serde_json::from_str(&row).map_err(storage_error)?;
            trial.validate()?;
            let folder: Option<String> = tx
                .query_row(
                    "SELECT root FROM session_workspaces WHERE session_id=?1",
                    [&trial.session],
                    |r| r.get(0),
                )
                .map_err(storage_error)?;
            if !trial.regressed()
                || trial.baseline != baseline.current().document
                || trial.candidate != *candidate
                || Some(trial.source_revision) != event.activated_revision
                || folder.as_deref() != Some(root)
                || event.monitor_session.as_ref() != Some(&trial.session)
                || event.monitor_status != "regressed"
            {
                return Err(
                    "No complete matching independent regression. Current skill retained.".into(),
                );
            }
        }
        let previous = skill.current().version;
        skill.revision = skill
            .revision
            .checked_add(1)
            .ok_or("Skill revision exhausted.")?;
        skill.versions.push(SkillVersion {
            version: previous.checked_add(1).ok_or("Skill version exhausted.")?,
            reviewed_at: super::now(),
            document: baseline.current().document.clone(),
            rollback_from: Some(previous),
            evaluation: None,
        });
        if skill.versions.len() > MAX_SKILL_VERSIONS {
            skill.versions.remove(0);
        }
        skill.validate()?;
        tx.execute(
            "UPDATE project_skills SET data=?3 WHERE root=?1 AND name=?2",
            params![
                root,
                skill.name,
                serde_json::to_string(&skill).map_err(storage_error)?
            ],
        )
        .map_err(storage_error)?;
        state.events[index].status = "quarantined".into();
        state.events[index].reason=if confirmed {"Matching independent cases confirmed regression. Baseline restored atomically and candidate quarantined."} else {"User restored the retained baseline. Candidate quarantined; previous evidence remains inspectable."}.into();
        state.notice="Project skill baseline restored. Candidate quarantined; manual Library choices remain available.".into();
        let saved = write(&tx, root, revision, &state)?;
        tx.commit().map_err(storage_error)?;
        Ok(saved)
    }
    pub(super) fn read_adaptation(&self, root: &str) -> Result<AdaptationState, String> {
        let guard = self.lock()?;
        read(&guard, root)
    }
    pub(super) fn write_adaptation(
        &self,
        root: &str,
        revision: u32,
        s: &AdaptationState,
    ) -> Result<AdaptationState, String> {
        let mut guard = self.lock()?;
        let tx = guard.transaction().map_err(storage_error)?;
        let saved = write(&tx, root, revision, s)?;
        tx.commit().map_err(storage_error)?;
        Ok(saved)
    }
    pub(super) fn activate_learning(
        &self,
        root: &str,
        revision: u32,
        id: &str,
    ) -> Result<AdaptationState, String> {
        let mut guard = self.lock()?;
        let tx = guard.transaction().map_err(storage_error)?;
        let mut state = read(&tx, root)?;
        if state.revision != revision || !state.enabled || state.paused {
            return Err("Learning policy changed or paused. Baseline preserved.".into());
        }
        let quarantined = state
            .events
            .iter()
            .filter(|e| e.status == "quarantined")
            .filter_map(|e| e.candidate.clone())
            .collect::<Vec<_>>();
        let event = state
            .events
            .iter_mut()
            .find(|e| e.id == id)
            .ok_or("Learning event missing.")?;
        if event.status != "review" || event.policy_revision != state.policy_revision {
            return Err("Proposal or policy changed. Inspect a fresh event.".into());
        }
        let baseline = event
            .baseline
            .as_ref()
            .ok_or("No targeted skill baseline.")?;
        let folder: Option<String> = tx
            .query_row(
                "SELECT root FROM session_workspaces WHERE session_id=?1",
                [&event.session],
                |r| r.get(0),
            )
            .map_err(storage_error)?;
        if folder.as_deref() != Some(root) {
            return Err("Learning source belongs to another working folder.".into());
        }
        let candidate = event.candidate.as_ref().ok_or("No candidate.")?;
        if quarantined.contains(candidate) {
            return Err("Candidate quarantined. Use an explicit Library review.".into());
        }
        if !limited_repair(baseline, candidate) {
            return Err(
                "Unknown instruction impact. Review through Library; automatic activation refused."
                    .into(),
            );
        }
        let current = skills::read(&tx, root)?
            .into_iter()
            .find(|s| s.name == baseline.name)
            .ok_or("Baseline missing.")?;
        if &current != baseline {
            return Err("Skill changed after trials. Manual choices were preserved.".into());
        }
        let row: String = tx
            .query_row(
                "SELECT data FROM experience_trials WHERE id=?1 AND session=?2",
                params![
                    event
                        .trial_id
                        .as_ref()
                        .ok_or("Independent evidence missing.")?,
                    event.session
                ],
                |r| r.get(0),
            )
            .map_err(storage_error)?;
        let trial: ExperienceTrial = serde_json::from_str(&row).map_err(storage_error)?;
        trial.validate()?;
        if !trial.improved()
            || trial.baseline != baseline.current().document
            || trial.candidate != *candidate
            || trial.source_revision != baseline.revision
        {
            return Err("No complete independent improvement evidence. Baseline preserved.".into());
        }
        let knowledge: KnowledgeState = crate::knowledge::read(&tx, root)?;
        if knowledge.revision != event.knowledge_revision
            || !knowledge
                .facts
                .iter()
                .any(|f| Some(&f.id) == event.fact_id.as_ref() && f.enabled)
        {
            return Err("Knowledge source changed. Reinspect before activation.".into());
        }
        let mut skill = current;
        let version = skill
            .current()
            .version
            .checked_add(1)
            .ok_or("Skill version exhausted.")?;
        skill.revision = skill
            .revision
            .checked_add(1)
            .ok_or("Skill revision exhausted.")?;
        skill.versions.push(SkillVersion {
            evaluation: None,
            version,
            reviewed_at: super::now(),
            document: candidate.clone(),
            rollback_from: None,
        });
        if skill.versions.len() > MAX_SKILL_VERSIONS {
            skill.versions.remove(0);
        }
        skill.validate()?;
        tx.execute(
            "UPDATE project_skills SET data=?3 WHERE root=?1 AND name=?2",
            params![
                root,
                skill.name,
                serde_json::to_string(&skill).map_err(storage_error)?
            ],
        )
        .map_err(storage_error)?;
        event.status = "active".into();
        event.activated_revision = Some(skill.revision);
        event.reason.push_str(
            " Independent fixtures improved; exact command slot activated with baseline retained.",
        );
        state.notice =
            "Project check workflow updated. Inspect evidence or restore it in Skills → Learning."
                .into();
        let saved = write(&tx, root, revision, &state)?;
        tx.commit().map_err(storage_error)?;
        Ok(saved)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn active(store: &SqliteStore, root: &str) -> AdaptationState {
        store
            .create_workspace_session(
                "task",
                &SessionWorkspace {
                    kind: WorkspaceKind::Project,
                    root: Some(root.into()),
                },
            )
            .unwrap();
        let baseline = store
            .activate_project_skill(
                root,
                &check_workflow("node obsolete.cjs").unwrap(),
                None,
                None,
            )
            .unwrap();
        let candidate = check_workflow("node verify.cjs").unwrap();
        let skill = store
            .activate_project_skill(root, &candidate, Some(1), None)
            .unwrap();
        let event = AdaptationEvent {
            id: "event".into(),
            session: "task".into(),
            message_id: 1,
            created_at: 1,
            cause: "skill".into(),
            confidence: "high".into(),
            reason: "fixture activation".into(),
            status: "active".into(),
            policy_revision: 1,
            knowledge_revision: 0,
            fact_id: None,
            baseline: Some(baseline),
            candidate: Some(candidate),
            trial_id: None,
            activated_revision: Some(skill.revision),
            monitor_message: 1,
            monitor_session: None,
            monitor_trial_id: None,
            monitor_status: String::new(),
        };
        store
            .save_adaptation(
                root,
                0,
                &AdaptationState {
                    revision: 0,
                    policy_revision: 1,
                    enabled: true,
                    automatic: true,
                    paused: false,
                    events: vec![event],
                    notice: String::new(),
                },
            )
            .unwrap()
    }
    #[test]
    fn restore_retention_failure_is_atomic_restart_and_manual_choices_survive() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("data.db");
        let root = dir
            .path()
            .canonicalize()
            .unwrap()
            .to_string_lossy()
            .to_string();
        let store = SqliteStore::open(&path).unwrap();
        let state = active(&store, &root);
        let original = store.project_skills(&root).unwrap();
        store.lock().unwrap().execute_batch("CREATE TRIGGER fail_retention BEFORE UPDATE ON project_adaptation BEGIN SELECT RAISE(ABORT,'retention failure'); END;").unwrap();
        assert!(store
            .restore_adaptation(&root, state.revision, "event", false)
            .is_err());
        assert_eq!(store.project_skills(&root).unwrap(), original);
        assert_eq!(store.adaptation(&root).unwrap(), state);
        store
            .lock()
            .unwrap()
            .execute_batch("DROP TRIGGER fail_retention;")
            .unwrap();
        drop(store);
        let store = SqliteStore::open(&path).unwrap();
        assert_eq!(store.adaptation(&root).unwrap(), state);
        let saved = store
            .restore_adaptation(&root, state.revision, "event", false)
            .unwrap();
        assert_eq!(saved.events[0].status, "quarantined");
        assert_eq!(
            workflow_command(&store.project_skills(&root).unwrap()[0].current().document)
                .as_deref(),
            Some("node obsolete.cjs")
        );
        assert!(store
            .restore_adaptation(&root, state.revision, "event", false)
            .is_err());
        drop(store);
        let store = SqliteStore::open(&path).unwrap();
        assert_eq!(store.adaptation(&root).unwrap(), saved);
        let mut another = saved;
        another.events[0].status = "active".into();
        let updated = store
            .activate_project_skill(
                &root,
                &check_workflow("node manual.cjs").unwrap(),
                Some(3),
                None,
            )
            .unwrap();
        another = store
            .save_adaptation(&root, another.revision, &another)
            .unwrap();
        let conflict = store
            .restore_adaptation(&root, another.revision, "event", false)
            .unwrap();
        assert_eq!(conflict.events[0].monitor_status, "conflict");
        assert_eq!(store.project_skills(&root).unwrap()[0], updated);
    }
    #[test]
    fn restart_interrupts_pending_and_monitoring_without_activation_or_replay() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("data.db");
        let root = dir
            .path()
            .canonicalize()
            .unwrap()
            .to_string_lossy()
            .to_string();
        let store = SqliteStore::open(&path).unwrap();
        let mut state = active(&store, &root);
        state.events[0].status = "pending".into();
        state.events[0].monitor_status = "running".into();
        store
            .save_adaptation(&root, state.revision, &state)
            .unwrap();
        let skill = store.project_skills(&root).unwrap();
        drop(store);
        let store = SqliteStore::open(&path).unwrap();
        store.interrupt_adaptations().unwrap();
        let state = store.adaptation(&root).unwrap();
        assert_eq!(state.events[0].status, "interrupted");
        assert_eq!(state.events[0].monitor_status, "interrupted");
        assert!(store
            .activate_adaptation(&root, state.revision, "event")
            .is_err());
        assert_eq!(store.project_skills(&root).unwrap(), skill);
        store.interrupt_adaptations().unwrap();
        assert_eq!(store.adaptation(&root).unwrap(), state);
    }
}
