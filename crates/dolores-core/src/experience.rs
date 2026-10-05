use crate::{AgentSummary, RequestSettings, SkillDocument};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
pub const EXPERIENCE_SUITE: &str = "config-command-v1";
pub const EXPERIENCE_CASES: usize = 2;
pub fn experience_config(case: usize, enabled: bool) -> serde_json::Value {
    if case == 0 {
        serde_json::json!({"enabled":enabled,"label":"original"})
    } else {
        serde_json::json!({"enabled":enabled,"label":"regression-世界","options":{"enabled":false,"keep":[1,"世界"]}})
    }
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ExperienceResult {
    pub violations: u32,
    pub case: usize,
    pub candidate: bool,
    pub complete: bool,
    pub passed: bool,
    pub detail: String,
    pub answer: String,
    pub elapsed_ms: u64,
    pub files: BTreeMap<String, String>,
    pub evidence: Option<AgentSummary>,
}
impl ExperienceResult {
    pub fn graded(&self) -> bool {
        if !self.complete || self.violations > 0 {
            return false;
        }
        let Some(e) = &self.evidence else {
            return false;
        };
        let write = e.tools.iter().rposition(|t| {
            t.name == "trial_file"
                && t.target == "config.json"
                && t.content == "Fixture file written."
        });
        let check=e.tools.iter().rposition(|t|t.name=="trial_check"&&t.target=="node verify.cjs"&&serde_json::from_str::<serde_json::Value>(&t.content).is_ok_and(|v|v==serde_json::json!({"simulated":true,"command":"node verify.cjs","exitCode":0,"criteriaPassed":true})));
        write.zip(check).is_some_and(|(w, c)| w < c)
            && self
                .files
                .get("config.json")
                .and_then(|s| serde_json::from_str::<serde_json::Value>(s).ok())
                == Some(experience_config(self.case, true))
            && self.files.get("protected.txt").map(String::as_str)
                == Some("Preserve this independent fixture.\n")
            && self.files.get("package.json").map(String::as_str)
                == Some(r#"{"scripts":{"check":"node verify.cjs"}}"#)
    }
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ExperienceTrial {
    pub id: String,
    pub session: String,
    pub revision: u32,
    pub created_at: i64,
    pub suite: String,
    pub model: String,
    pub settings: RequestSettings,
    pub source_revision: u32,
    pub baseline: SkillDocument,
    pub candidate: SkillDocument,
    pub status: String,
    pub results: Vec<ExperienceResult>,
}
impl ExperienceTrial {
    pub fn improved(&self) -> bool {
        self.status == "completed"
            && self.results.len() == 4
            && self.results.iter().all(|r| r.complete)
            && self
                .results
                .iter()
                .filter(|r| r.candidate)
                .all(|r| r.passed)
            && self.results.iter().any(|r| !r.candidate && !r.passed)
    }
    pub fn validate(&self) -> Result<(), String> {
        self.baseline.validate()?;
        self.candidate.validate()?;
        self.settings.validate()?;
        if !crate::valid_memory_id(&self.id)
            || self.session.is_empty()
            || self.session.len() > 128
            || self.suite != EXPERIENCE_SUITE
            || self.model.is_empty()
            || self.model.len() > 200
            || self.baseline.name != self.candidate.name
            || self.source_revision == 0
            || self.settings.max_output_tokens != 1024
            || self.settings.timeout_seconds != 30
            || !["running", "completed", "failed", "stopped", "interrupted"]
                .contains(&self.status.as_str())
            || self.results.len() > 4
            || (self.status == "completed" && self.results.len() != 4)
            || self.results.iter().enumerate().any(|(i, r)| {
                r.case != i / 2
                    || r.candidate != (i % 2 == 1)
                    || r.passed != r.graded()
                    || r.detail.len() > 512
                    || r.answer.len() > 2048
                    || r.files.len() > 4
                    || r.files.iter().any(|(k, v)| k.len() > 40 || v.len() > 2048)
                    || r.evidence
                        .as_ref()
                        .is_some_and(|e| e.model_calls > 5 || e.tools.len() > 8)
            })
        {
            return Err("Invalid or incomplete frozen trial receipt.".into());
        }
        if serde_json::to_vec(self)
            .map_err(|_| "Trial serialization failed.")?
            .len()
            > 128 * 1024
        {
            return Err("Trial evidence exceeds 128 KiB.".into());
        }
        Ok(())
    }
    pub fn same_request(&self, other: &Self) -> bool {
        let mut a = self.clone();
        let mut b = other.clone();
        a.revision = 0;
        b.revision = 0;
        a.status.clear();
        b.status.clear();
        a.results.clear();
        b.results.clear();
        a == b
    }
    pub fn regressed(&self) -> bool {
        self.status == "completed"
            && self.results.len() == 4
            && self.results.iter().all(|r| r.complete)
            && self
                .results
                .iter()
                .filter(|r| !r.candidate)
                .all(|r| r.passed)
            && self.results.iter().any(|r| r.candidate && !r.passed)
    }
}
