use crate::TurnMetadata;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum TaskOutcome {
    Worked,
    NeedsWork,
}

/// Explicit local user assessment; it never overrides command evidence.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct TaskFeedback {
    pub revision: u32,
    pub updated_at: i64,
    pub outcome: Option<TaskOutcome>,
    pub note: String,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct FeedbackDraft {
    pub message_id: i64,
    pub expected_content: String,
    pub expected_metadata: Option<TurnMetadata>,
    pub revision: u32,
    pub outcome: Option<TaskOutcome>,
    pub note: String,
}
impl FeedbackDraft {
    pub fn validate(&self) -> Result<(), String> {
        if self.message_id <= 0
            || self.note.len() > 2048
            || self
                .note
                .chars()
                .any(|c| c.is_control() && c != '\n' && c != '\t')
            || crate::credential_like(&self.note)
            || (self.outcome.is_none() && !self.note.is_empty())
        {
            return Err("Choose an outcome and a note within 2 KiB without credentials; clearing feedback also clears its note.".into());
        }
        Ok(())
    }
}
