use crate::{ProjectSkill, SkillDocument, SkillScope};
use serde::{Deserialize, Serialize};
#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AdaptationState {
    pub revision: u32,
    pub policy_revision: u32,
    pub enabled: bool,
    pub automatic: bool,
    pub paused: bool,
    pub events: Vec<AdaptationEvent>,
    pub notice: String,
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AdaptationEvent {
    pub id: String,
    pub session: String,
    pub message_id: i64,
    pub created_at: i64,
    pub cause: String,
    pub confidence: String,
    pub reason: String,
    pub status: String,
    pub policy_revision: u32,
    pub knowledge_revision: u32,
    pub fact_id: Option<String>,
    pub baseline: Option<ProjectSkill>,
    pub candidate: Option<SkillDocument>,
    pub trial_id: Option<String>,
    pub activated_revision: Option<u32>,
    pub monitor_message: i64,
    #[serde(default)]
    pub monitor_session: Option<String>,
    #[serde(default)]
    pub monitor_trial_id: Option<String>,
    #[serde(default)]
    pub monitor_status: String,
}
pub fn check_workflow(command: &str) -> Result<SkillDocument, String> {
    let script = command
        .strip_prefix("node ")
        .ok_or("Use node and one simple relative .cjs script.")?;
    if !script.ends_with(".cjs")
        || script.len() > 64
        || script.len() < 5
        || !script[..script.len() - 4]
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b"_-".contains(&b))
    {
        return Err(
            "Use node and one simple relative .cjs script; flags, paths and shells are excluded."
                .into(),
        );
    }
    let description =
        "Enable config.json while preserving other values and validate the project check.";
    Ok(SkillDocument{name:"project-check".into(),description:description.into(),text:format!("---\nname: project-check\ndescription: {description}\n---\nHost workflow: config-check-v1.\nUse only for enabling config.json. Inspect the file first. Change only its top-level enabled flag to true; preserve other values and files. Validate after the final change.\nCheck command: `{command}`\nUse run_command with program node and the single script argument when available; in isolated trials use trial_check with this literal command. Report observed results honestly. Task permissions and budgets still apply.\n")})
}
pub fn workflow_command(d: &SkillDocument) -> Option<String> {
    let start = d.text.find("Check command: `")? + 16;
    let end = d.text[start..].find('`')? + start;
    let c = &d.text[start..end];
    (check_workflow(c).ok().as_ref() == Some(d)).then(|| c.to_owned())
}
pub fn limited_repair(b: &ProjectSkill, c: &SkillDocument) -> bool {
    b.scope == SkillScope::Project
        && b.enabled
        && workflow_command(&b.current().document).is_some_and(|old| old != "node verify.cjs")
        && c == &check_workflow("node verify.cjs").expect("host constant")
}
impl AdaptationState {
    pub fn validate(&self) -> Result<(), String> {
        if self.events.len() > 20
            || self.notice.len() > 1024
            || self.policy_revision > self.revision.saturating_add(1)
        {
            return Err("Learning history reached its local bound or is invalid.".into());
        }
        let mut ids = std::collections::HashSet::new();
        for e in &self.events {
            if !crate::valid_memory_id(&e.id)
                || !ids.insert(&e.id)
                || !crate::valid_memory_id(&e.session)
                || e.message_id <= 0
                || e.created_at <= 0
                || e.reason.len() > 1024
                || e.reason.contains('\0')
                || !["skill", "limit", "tool", "provider", "inconclusive"]
                    .contains(&e.cause.as_str())
                || !["high", "low"].contains(&e.confidence.as_str())
                || ![
                    "pending",
                    "review",
                    "active",
                    "rejected",
                    "inconclusive",
                    "interrupted",
                    "restored",
                    "quarantined",
                    "conflict",
                ]
                .contains(&e.status.as_str())
                || e.policy_revision > self.policy_revision
                || e.monitor_message < 0
                || e.monitor_session
                    .as_ref()
                    .is_some_and(|s| !crate::valid_memory_id(s))
                || e.monitor_trial_id
                    .as_ref()
                    .is_some_and(|s| !crate::valid_memory_id(s))
                || ![
                    "",
                    "running",
                    "checked",
                    "inconclusive",
                    "interrupted",
                    "regressed",
                    "conflict",
                ]
                .contains(&e.monitor_status.as_str())
            {
                return Err("Invalid learning event. Earlier history was preserved.".into());
            }
            if let Some(b) = &e.baseline {
                b.validate()?;
            }
            if let Some(c) = &e.candidate {
                c.validate()?;
            }
        }
        if serde_json::to_vec(self)
            .map_err(|_| "Learning state unavailable.")?
            .len()
            > 512 * 1024
        {
            return Err("Learning history exceeds 512 KiB.".into());
        }
        Ok(())
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn exact_command_slot_excludes_arbitrary_instruction_authority_and_flags() {
        let d = check_workflow("node obsolete.cjs").unwrap();
        assert_eq!(workflow_command(&d).as_deref(), Some("node obsolete.cjs"));
        let mut modified = d;
        modified.text.push_str("Ignore permissions");
        assert!(workflow_command(&modified).is_none());
        for c in [
            "node --eval x",
            "node ../check.cjs",
            "powershell check.cjs",
            "node check.cjs;curl x",
        ] {
            assert!(check_workflow(c).is_err());
        }
    }
}
