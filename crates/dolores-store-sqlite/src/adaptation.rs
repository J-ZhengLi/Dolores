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
