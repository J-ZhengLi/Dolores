//! Bounded native trial evidence. Passing these tests grants no installation authority.
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct NativeTestRun {
    pub reason: String,
    pub exit_code: Option<i32>,
    pub passed: Vec<String>,
    pub failed: Vec<String>,
    pub complete: bool,
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct RepairEvaluation {
    pub id: String,
    pub session: String,
    pub repair_id: String,
    pub revision: u32,
    pub bundle_id: String,
    pub candidate_ids: Vec<String>,
    pub criteria_id: String,
    pub cargo_id: String,
    pub package: String,
    pub reproduction: String,
    pub status: String,
    pub artifact: String,
    pub baseline: Option<NativeTestRun>,
    pub candidate: Option<NativeTestRun>,
    pub regression: Option<NativeTestRun>,
    pub note: String,
}
impl RepairEvaluation {
    pub fn validate(&self) -> Result<(), String> {
        use crate::harness_repair::{valid_repair_id, valid_source_id};
        if !valid_repair_id(&self.id)
            || !valid_repair_id(&self.repair_id)
            || self.session.is_empty()
            || self.session.len() > 128
            || self.revision == 0
            || !valid_source_id(&self.bundle_id)
            || !valid_source_id(&self.criteria_id)
            || !valid_source_id(&self.cargo_id)
            || self.candidate_ids.is_empty()
            || self.candidate_ids.len() > 8
            || self.candidate_ids.iter().any(|id| !valid_source_id(id))
            || !["started", "withheld", "qualified", "failed", "stopped"]
                .contains(&self.status.as_str())
            || self.artifact != format!("evaluation-{}", self.id)
            || self.note.len() > 1024
            || !["dolores-core", "dolores-provider-openai"].contains(&self.package.as_str())
            || self.reproduction.is_empty()
            || self.reproduction.len() > 4096
            || (self.status == "qualified" && !self.improved())
            || [&self.baseline, &self.candidate, &self.regression]
                .into_iter()
                .flatten()
                .any(|r| {
                    r.reason.len() > 128
                        || r.passed.len() + r.failed.len() > 4096
                        || r.passed
                            .iter()
                            .chain(&r.failed)
                            .any(|s| s.len() > 512 || s.chars().any(char::is_control))
                })
        {
            return Err("Invalid native evaluation receipt; proposal remains.".into());
        }
        Ok(())
    }
    pub fn improved(&self) -> bool {
        fn covers(before: &NativeTestRun, after: &NativeTestRun) -> bool {
            let mut counts = std::collections::HashMap::<&str, usize>::new();
            for name in &after.passed {
                *counts.entry(name).or_default() += 1;
            }
            before.passed.iter().chain(&before.failed).all(|name| {
                match counts.get_mut(name.as_str()) {
                    Some(count) if *count > 0 => {
                        *count -= 1;
                        true
                    }
                    _ => false,
                }
            })
        }
        match (&self.baseline, &self.candidate, &self.regression) {
            (Some(b), Some(c), Some(r)) => {
                b.complete
                    && c.complete
                    && b.reason == "completed"
                    && c.reason == "completed"
                    && b.exit_code.is_some_and(|n| n != 0)
                    && !b.failed.is_empty()
                    && c.exit_code == Some(0)
                    && c.failed.is_empty()
                    && r.complete
                    && r.reason == "completed"
                    && r.exit_code == Some(0)
                    && !r.passed.is_empty()
                    && r.failed.is_empty()
                    && covers(b, c)
            }
            _ => false,
        }
    }
}
