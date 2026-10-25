//! Attention is a navigational view of saved evidence, never execution authority.
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Kind {
    Completed,
    Failed,
    Approval,
    Question,
    App,
}
impl Kind {
    pub fn from_event(kind: &str, state: &str) -> Option<Self> {
        match (kind, state) {
            ("approvalRequested", "waitingForApproval") => Some(Self::Approval),
            ("questionAsked", "waitingForApproval") => Some(Self::Question),
            ("finished", "completed") => Some(Self::Completed),
            ("finished", "failed" | "paused" | "interrupted") | ("restart", "interrupted") => {
                Some(Self::Failed)
            }
            _ => None,
        }
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Preferences {
    pub revision: u32,
    pub enabled: bool,
    pub completed: bool,
    pub failed: bool,
    pub approval: bool,
    pub question: bool,
}
impl Default for Preferences {
    fn default() -> Self {
        Self {
            revision: 0,
            enabled: false,
            completed: true,
            failed: true,
            approval: true,
            question: true,
        }
    }
}
impl Preferences {
    pub fn permits(&self, kind: Kind) -> bool {
        self.enabled
            && match kind {
                Kind::Completed => self.completed,
                Kind::Failed => self.failed,
                Kind::Approval => self.approval,
                Kind::Question => self.question,
                Kind::App => false,
            }
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Delivery {
    Pending,
    Claimed,
    Accepted,
    Failed,
    Suppressed,
    Uncertain,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Notice {
    pub id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub details: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub severity: Option<String>,
    pub session: String,
    pub run: String,
    pub sequence: u32,
    pub kind: Kind,
    pub created_at: i64,
    pub seen: bool,
    pub delivery: Delivery,
}
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Page {
    pub preferences: Preferences,
    pub items: Vec<Notice>,
    pub next_cursor: Option<i64>,
    pub more_sources: bool,
    pub omitted: u64,
    pub unread: u64,
    pub pending_delivery: Vec<Notice>,
}
#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "action", rename_all = "camelCase", deny_unknown_fields)]
pub enum Request {
    Post {
        source: String,
        title: String,
        details: String,
        severity: String,
    },
    ClearRead,
    SeenAll,
    Poll {
        cursor: Option<i64>,
    },
    Preferences,
    SavePreferences {
        preferences: Preferences,
    },
    Inspect {
        id: String,
    },
    Resolve {
        id: String,
    },
    Seen {
        id: String,
    },
    Claim {
        id: String,
    },
    Finish {
        id: String,
        delivery: Delivery,
    },
}
