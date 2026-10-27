use super::{storage_error, SqliteStore};
use dolores_core::{goal::*, TaskBudget};
use rusqlite::{params, Connection, OptionalExtension};
pub(super) fn read(c: &Connection, s: &str) -> Result<Option<Goal>, String> {
    let data: Option<String> = c
        .query_row("SELECT data FROM goals WHERE session=?1", [s], |r| r.get(0))
        .optional()
        .map_err(storage_error)?;
    data.map(|v| {
        serde_json::from_str::<Goal>(&v)
            .map_err(storage_error)
            .and_then(|g| {
                g.validate()?;
                Ok(g)
            })
    })
    .transpose()
}
fn save(c: &Connection, s: &str, g: &Goal) -> Result<(), String> {
    g.validate()?;
    c.execute("INSERT INTO goals(session,data) VALUES(?1,?2) ON CONFLICT(session) DO UPDATE SET data=excluded.data",params![s,serde_json::to_string(g).map_err(storage_error)?]).map_err(storage_error)?;
    Ok(())
}
fn bump(g: &mut Goal) -> Result<(), String> {
    g.revision = g
        .revision
        .checked_add(1)
        .ok_or("Goal revision exhausted. Clear and create a new goal.")?;
    Ok(())
}
fn stale() -> String {
    "Goal changed. Refresh progress before trying again; your edits remain.".into()
}
fn owned(c: &Connection, s: &str, id: &str, run: &str) -> Result<Goal, String> {
    let g = read(c, s)?.ok_or_else(stale)?;
    if g.id != id || g.current_run.as_deref() != Some(run) {
        return Err(stale());
    }
    Ok(g)
}
impl SqliteStore {
    pub(super) fn goal_edit(
        &self,
        s: &str,
        e: &GoalEdit,
        limits: TaskBudget,
    ) -> Result<Goal, String> {
        let mut c = self.lock()?;
        let tx = c.transaction().map_err(storage_error)?;
        let old = read(&tx, s)?;
        if old.as_ref().is_some_and(|g| g.current_run.is_some()) {
            return Err("Stop the goal's current response before changing it.".into());
        }
        let mut g = match e {
            GoalEdit::Set {
                revision,
                objective,
                criteria,
                constraints,
            } => {
                if old.as_ref().map(|g| g.revision) != *revision {
                    return Err(stale());
                }
                if old
                    .as_ref()
                    .is_some_and(|g| !matches!(g.state, GoalState::Cleared | GoalState::Completed))
                {
                    return Err("Clear the prior goal before creating a new one. Its progress remains inspectable.".into());
                }
                if let Some(previous) = &old {
                    tx.execute(
                        "INSERT INTO goal_history(session,id,data) VALUES(?1,?2,?3)",
                        params![
                            s,
                            previous.id,
                            serde_json::to_string(previous).map_err(storage_error)?
                        ],
                    )
                    .map_err(storage_error)?;
                }
                Goal {
                    id: uuid::Uuid::new_v4().to_string(),
                    revision: old.as_ref().map_or(0, |g| g.revision),
                    progress_revision: 0,
                    objective: objective.clone(),
                    constraints: constraints.clone(),
                    state: GoalState::Active,
                    steps: criteria
                        .iter()
                        .map(|c| GoalStep {
                            criterion: c.clone(),
                            done: false,
                            evidence: vec![],
                        })
                        .collect(),
                    limits,
                    used: GoalUsage::default(),
                    runs: vec![],
                    current_run: None,
                    no_progress: 0,
                    completion_requested: false,
                    note: "Saved. Start the goal when ready.".into(),
                }
            }
            GoalEdit::Pause { revision }
            | GoalEdit::Resume { revision }
            | GoalEdit::Clear { revision } => {
                let mut g = old.ok_or_else(stale)?;
                if g.revision != *revision {
                    return Err(stale());
                }
                match e {
                    GoalEdit::Resume { .. } => {
                        if matches!(g.state, GoalState::Completed | GoalState::Cleared) {
                            return Err("Create a new goal to start more work.".into());
                        }
                        if g.used.model_calls >= g.limits.model_calls.unwrap_or(usize::MAX)
                            || g.used.tool_calls >= g.limits.tool_calls.unwrap_or(usize::MAX)
                            || g.used.segments >= g.limits.segments
                            || g.limits
                                .elapsed_seconds
                                .is_some_and(|n| g.used.active_seconds >= n)
                        {
                            return Err("Goal budget exhausted. Inspect progress, clear this goal and explicitly save a new scoped goal; counters are not reset by Resume.".into());
                        }
                        g.state = GoalState::Active;
                        g.no_progress = 0;
                        g.note = "Resumed explicitly. Saved effects will not be replayed.".into();
                    }
                    GoalEdit::Clear { .. } => {
                        g.state = GoalState::Cleared;
                        g.note = "Cleared. Receipts and working files remain.".into();
                    }
                    _ => {
                        g.state = GoalState::Paused;
                        g.note = "Paused. Inspect progress before Resume.".into();
                    }
                }
                g
            }
        };
        bump(&mut g)?;
        save(&tx, s, &g)?;
        tx.commit().map_err(storage_error)?;
        Ok(g)
    }
    pub(super) fn goal_claim(
        &self,
        s: &str,
        id: &str,
        revision: u32,
        run: &str,
    ) -> Result<Goal, String> {
        let mut c = self.lock()?;
        let tx = c.transaction().map_err(storage_error)?;
        let mut g = read(&tx, s)?.ok_or_else(stale)?;
        if g.id != id
            || g.revision != revision
            || g.state != GoalState::Active
            || g.current_run.is_some()
        {
            return Err(stale());
        }
        let inputs = super::task_inputs::read(&tx, s)?;
        if inputs.suspended
            || inputs.recovery_required
            || inputs.entries.iter().any(|v| {
                matches!(
                    v.state,
                    dolores_core::harness_task::InputState::Pending
                        | dolores_core::harness_task::InputState::Delivering
                ) || (v.state == dolores_core::harness_task::InputState::Uncertain && !v.inspected)
            })
        {
            return Err("Inspect and resume/cancel saved instructions before starting the goal. User input takes priority.".into());
        }
        if g.used.segments >= g.limits.segments
            || g.used.model_calls >= g.limits.model_calls.unwrap_or(usize::MAX)
            || g.used.tool_calls >= g.limits.tool_calls.unwrap_or(usize::MAX)
        {
            return Err(
                "Goal budget exhausted. Inspect progress and save a new scoped goal explicitly."
                    .into(),
            );
        }
        g.used.segments += 1;
        g.runs.push(run.into());
        g.current_run = Some(run.into());
        g.completion_requested = false;
        g.note = "Working toward saved criteria.".into();
        bump(&mut g)?;
        save(&tx, s, &g)?;
        tx.commit().map_err(storage_error)?;
        Ok(g)
    }
    pub(super) fn goal_charge(
        &self,
        s: &str,
        id: &str,
        run: &str,
        charge: GoalCharge,
    ) -> Result<bool, String> {
        let mut c = self.lock()?;
        let tx = c.transaction().map_err(storage_error)?;
        let mut g = owned(&tx, s, id, run)?;
        if g.state != GoalState::Active {
            return Ok(false);
        }
        let allowed = match charge {
            GoalCharge::Model => {
                if g.used.model_calls >= g.limits.model_calls.unwrap_or(usize::MAX) {
                    false
                } else {
                    g.used.model_calls += 1;
                    true
                }
            }
            GoalCharge::Tools(n) => {
                if n > g
                    .limits
                    .tool_calls
                    .unwrap_or(usize::MAX)
                    .saturating_sub(g.used.tool_calls)
                {
                    false
                } else {
                    g.used.tool_calls += n;
                    true
                }
            }
            GoalCharge::ActiveSecond => {
                if g.limits
                    .elapsed_seconds
                    .is_some_and(|n| g.used.active_seconds >= n)
                {
                    false
                } else {
                    g.used.active_seconds = g.used.active_seconds.saturating_add(1);
                    true
                }
            }
        };
        if !allowed {
            g.state = GoalState::BudgetLimited;
            g.note="Cumulative goal budget reached. Inspect retained work; compaction and Resume do not reset it.".into();
        }
        save(&tx, s, &g)?;
        tx.commit().map_err(storage_error)?;
        Ok(allowed)
    }
    pub(super) fn goal_progress(
        &self,
        s: &str,
        id: &str,
        run: &str,
        p: &GoalProgress,
    ) -> Result<Goal, String> {
        if p.note.len() > 1024 {
            return Err("Keep the progress note under 1 KiB.".into());
        }
        let mut c = self.lock()?;
        let tx = c.transaction().map_err(storage_error)?;
        let mut g = owned(&tx, s, id, run)?;
        if g.state != GoalState::Active
            || p.revision != g.progress_revision
            || p.steps.len() != g.steps.len()
            || p.steps.iter().zip(&g.steps).any(|(a, b)| {
                a.criterion != b.criterion
                    || a.evidence.len() > 4
                    || a.done && a.evidence.is_empty()
            })
        {
            return Err("Progress changed or criteria lack evidence. Inspect the current goal and retain its criteria.".into());
        }
        for step in &p.steps {
            for evidence in &step.evidence {
                if !g.runs.contains(&evidence.run) {
                    return Err("Evidence must belong to this goal's runs.".into());
                }
                let data: Option<String> = tx
                    .query_row(
                        "SELECT data FROM run_events WHERE run_id=?1 AND sequence=?2",
                        params![evidence.run, evidence.sequence],
                        |r| r.get(0),
                    )
                    .optional()
                    .map_err(storage_error)?;
                let event: dolores_core::RunEvent = data
                    .map(|s| serde_json::from_str(&s).map_err(storage_error))
                    .transpose()?
                    .ok_or(
                        "Successful tool receipt is missing. Inspect the goal's retained receipts.",
                    )?;
                if event.kind != "toolResult" {
                    return Err("Evidence must reference a successful tool result.".into());
                }
                let value = event.data;
                if value["name"] == "run_command" {
                    let command: serde_json::Value = serde_json::from_str(
                        value["content"].as_str().unwrap_or(""),
                    )
                    .map_err(|_| "Command evidence is malformed. Verify with a fresh check.")?;
                    if command["exitCode"] != 0 || command["reason"] != "completed" {
                        return Err(
                            "A failed or incomplete command cannot verify a completed criterion."
                                .into(),
                        );
                    }
                }
                if value["returned"] != true
                    || matches!(
                        value["name"].as_str(),
                        Some("goal_progress" | "inspect_goal")
                    )
                {
                    return Err("Completion needs successful work/verification receipts, not progress narration.".into());
                }
            }
        }
        if p.complete && p.steps.iter().any(|s| !s.done) {
            return Err("Every criterion needs evidence before requesting completion.".into());
        }
        g.steps = p.steps.clone();
        g.progress_revision = g
            .progress_revision
            .checked_add(1)
            .ok_or("Progress revision exhausted.")?;
        g.completion_requested = p.complete;
        g.note = p.note.clone();
        if p.blocked {
            g.state = GoalState::Blocked;
        }
        bump(&mut g)?;
        save(&tx, s, &g)?;
        tx.commit().map_err(storage_error)?;
        Ok(g)
    }
    pub(super) fn goal_finish(
        &self,
        s: &str,
        id: &str,
        run: &str,
        normal: bool,
        progress: bool,
    ) -> Result<Goal, String> {
        let mut c = self.lock()?;
        let tx = c.transaction().map_err(storage_error)?;
        let mut g = owned(&tx, s, id, run)?;
        g.current_run = None;
        if g.state == GoalState::Active {
            if !normal {
                g.state = GoalState::Paused;
                g.completion_requested = false;
                g.note="Interrupted or paused. Inspect retained receipts, then Resume explicitly; nothing was replayed.".into();
            } else if g.completion_requested
                && g.steps.iter().all(|s| s.done && !s.evidence.is_empty())
            {
                g.state = GoalState::Completed;
                g.note="All criteria linked to retained receipts; final response saved. Inspect evidence for the result.".into();
            } else {
                g.no_progress = if progress {
                    0
                } else {
                    g.no_progress.saturating_add(1)
                };
                if g.no_progress >= 2 {
                    g.state = GoalState::Blocked;
                    g.note="Two segments made no work/verification progress. Inspect the cause before Resume.".into();
                } else if g.used.segments >= g.limits.segments
                    || g.used.model_calls >= g.limits.model_calls.unwrap_or(usize::MAX)
                    || g.used.tool_calls >= g.limits.tool_calls.unwrap_or(usize::MAX)
                {
                    g.state = GoalState::BudgetLimited;
                    g.note="Cumulative goal allowance reached. Progress is retained; save a new scoped goal explicitly.".into();
                }
            }
        }
        bump(&mut g)?;
        save(&tx, s, &g)?;
        tx.commit().map_err(storage_error)?;
        Ok(g)
    }
    pub(super) fn goals_recover(&self) -> Result<(), String> {
        let mut c = self.lock()?;
        let tx = c.transaction().map_err(storage_error)?;
        let sessions: Vec<String> = tx
            .prepare("SELECT session FROM goals")
            .map_err(storage_error)?
            .query_map([], |r| r.get(0))
            .map_err(storage_error)?
            .collect::<Result<_, _>>()
            .map_err(storage_error)?;
        for s in sessions {
            let mut g = read(&tx, &s)?.unwrap();
            if g.state == GoalState::Active || g.current_run.is_some() {
                g.current_run = None;
                g.completion_requested = false;
                g.state = GoalState::Paused;
                g.note =
                    "App reopened. Inspect the goal and uncertain effects before Resume.".into();
                bump(&mut g)?;
                save(&tx, &s, &g)?;
            }
        }
        tx.commit().map_err(storage_error)
    }
}
