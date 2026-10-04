use crate::RequestSettings;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum RunState {
    Prepared,
    Running,
    WaitingForApproval,
    Paused,
    Completed,
    Failed,
    Cancelled,
    Interrupted,
}
impl RunState {
    pub fn can_transition(&self, next: &Self) -> bool {
        !self.terminal()
            && (next.terminal()
                || matches!(
                    (self, next),
                    (Self::Prepared, Self::Running)
                        | (Self::Running, Self::WaitingForApproval)
                        | (Self::WaitingForApproval, Self::Running)
                ))
    }
    pub fn terminal(&self) -> bool {
        matches!(
            self,
            Self::Paused | Self::Completed | Self::Failed | Self::Cancelled | Self::Interrupted
        )
    }
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RunSnapshot {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parent_run: Option<String>,
    #[serde(default = "first_segment")]
    pub segments: u32,
    pub id: String,
    pub thread: String,
    pub model: String,
    pub settings: RequestSettings,
    pub input: String,
    pub state: RunState,
    pub sequence: u32,
    pub created_at: i64,
    pub build: String,
    pub tools: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub extensions: Vec<crate::ResolvedExtension>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub effective_settings: Option<crate::EffectiveSettings>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RunEvent {
    pub sequence: u32,
    pub kind: String,
    pub state: RunState,
    pub data: serde_json::Value,
}
pub const MAX_RUN_EVENTS: u32 = 256;
pub const MAX_RUN_EVENT_BYTES: usize = 128 * 1024;

fn first_segment() -> u32 {
    1
}
