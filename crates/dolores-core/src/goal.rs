use crate::TaskBudget;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum GoalState {
    Active,
    Paused,
    Blocked,
    BudgetLimited,
    Completed,
    Cleared,
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct GoalEvidence {
    pub run: String,
    pub sequence: u32,
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct GoalStep {
    pub criterion: String,
    pub done: bool,
    pub evidence: Vec<GoalEvidence>,
}
#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct GoalUsage {
    pub model_calls: usize,
    pub tool_calls: usize,
    pub segments: u32,
    pub active_seconds: u32,
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Goal {
    pub id: String,
    pub revision: u32,
    pub progress_revision: u32,
    pub objective: String,
    pub constraints: String,
    pub state: GoalState,
    pub steps: Vec<GoalStep>,
    pub limits: TaskBudget,
    pub used: GoalUsage,
    pub runs: Vec<String>,
    pub current_run: Option<String>,
    pub no_progress: u32,
    pub completion_requested: bool,
    pub note: String,
}
impl Goal {
    pub fn validate(&self) -> Result<(), String> {
        self.limits.validate()?;
        if self.objective.trim().is_empty()
            || self.objective.len() > 4096
            || self.constraints.len() > 2048
            || !(1..=12).contains(&self.steps.len())
            || self.steps.iter().any(|s| {
                s.criterion.trim().is_empty() || s.criterion.len() > 512 || s.evidence.len() > 4
            })
        {
            return Err("Use an objective up to 4 KiB, constraints up to 2 KiB and 1–12 criteria up to 512 bytes each. Your edits remain.".into());
        }
        Ok(())
    }
    pub fn prompt(&self) -> Result<String, String> {
        self.validate()?;
        let data = serde_json::to_string(self).map_err(|_| "Could not prepare goal.")?;
        Ok(format!("Continue the explicitly saved goal. Durable task state: {data}\nPreserve constraints and inspect prior run receipts/current files before acting. Never replay completed tools or infer approval from history. Use goal_progress to record each criterion with successful tool receipt references (run, sequence); use inspect_goal to locate receipts. Request completion only with all criteria evidenced. A final answer alone does not complete this goal. Context compaction does not reset cumulative task limits."))
    }
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "action", rename_all = "camelCase", deny_unknown_fields)]
pub enum GoalEdit {
    Set {
        revision: Option<u32>,
        objective: String,
        criteria: Vec<String>,
        constraints: String,
    },
    Pause {
        revision: u32,
    },
    Resume {
        revision: u32,
    },
    Clear {
        revision: u32,
    },
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct GoalProgress {
    pub revision: u32,
    pub steps: Vec<GoalStep>,
    pub complete: bool,
    pub blocked: bool,
    pub note: String,
}
#[derive(Clone, Copy)]
pub enum GoalCharge {
    Model,
    Tools(usize),
    ActiveSecond,
}
