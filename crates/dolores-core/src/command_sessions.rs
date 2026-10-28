//! Durable evidence for supervised commands. These records never grant execution.
use serde::{Deserialize, Serialize};

pub const MAX_OUTPUT: usize = 262144;
pub const MAX_SESSION_SECONDS: u64 = 6 * 60 * 60;
pub const OUTPUT_PAGE: usize = 8192;
pub const RECEIPT_BYTES: usize = 8192;
pub const LIST_PAGE: usize = 16;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct InputReceipt {
    pub call_id: String,
    pub revision: u64,
    pub state: String,
    pub bytes: usize,
    pub written_bytes: usize,
    pub eof: bool,
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct SessionReceipt {
    pub id: String,
    pub task: String,
    pub run: String,
    pub start_call: String,
    pub program: String,
    pub execution: String,
    pub state: String,
    pub exit_code: Option<i32>,
    pub timeout_seconds: u64,
    pub capture_bytes: usize,
    pub output_bytes: usize,
    pub truncated: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub local_log: Option<String>,
    #[serde(default)]
    pub output_retained: bool,
    #[serde(default)]
    pub log_error: bool,
    pub output_error: bool,
    pub lossy_utf8: bool,
    pub cleanup_complete: bool,
    pub input_revision: u64,
    pub input_bytes: usize,
    pub stdin_closed: bool,
    pub stdin_closing: bool,
    pub last_input: Option<InputReceipt>,
    pub message: Option<String>,
}
impl SessionReceipt {
    pub fn live(&self) -> bool {
        matches!(self.state.as_str(), "starting" | "running" | "stopping")
    }
    pub fn validate(&self) -> Result<(), String> {
        let identifier =
            |v: &str, max| !v.is_empty() && v.len() <= max && !v.chars().any(char::is_control);
        let valid_input = self.last_input.as_ref().is_none_or(|i| {
            identifier(&i.call_id, 128)
                && i.revision > 0
                && i.revision <= self.input_revision
                && i.bytes <= 4096
                && i.written_bytes <= i.bytes
                && ["pending", "written", "uncertain", "failed"].contains(&i.state.as_str())
        });
        if !identifier(&self.id, 80)
            || !identifier(&self.task, 128)
            || !identifier(&self.run, 128)
            || !identifier(&self.start_call, 128)
            || !identifier(&self.program, 4096)
            || !identifier(&self.execution, 128)
            || ![
                "starting",
                "running",
                "stopping",
                "completed",
                "interrupted",
                "closed",
                "timedOut",
                "outputLimit",
                "processError",
                "lost",
                "failed",
            ]
            .contains(&self.state.as_str())
            || !(1..=MAX_SESSION_SECONDS).contains(&self.timeout_seconds)
            || !(1024..=MAX_OUTPUT).contains(&self.capture_bytes)
            || self.output_bytes > self.capture_bytes
            || self.input_bytes > 65536
            || !valid_input
            || self.live() && self.cleanup_complete
            || self.message.as_ref().is_some_and(|m| m.len() > 1024)
            || self.local_log.as_ref().is_some_and(|p| {
                p.len() > 4096
                    || p.is_empty()
                    || p.chars().any(char::is_control)
                    || (!std::path::Path::new(p).is_absolute() && p.contains(['/', '\\']))
            })
        {
            return Err("Invalid command-session evidence. Prior saved work remains.".into());
        }
        if serde_json::to_vec(self)
            .map_err(|_| "Invalid command receipt.")?
            .len()
            > RECEIPT_BYTES
        {
            return Err("Command-session receipt exceeds its 8 KiB limit.".into());
        }
        Ok(())
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SessionRecord {
    pub revision: u64,
    pub receipt: SessionReceipt,
    pub output: Vec<u8>,
}
impl SessionRecord {
    pub fn validate(&self) -> Result<(), String> {
        self.receipt.validate()?;
        if self.revision >= i64::MAX as u64 || self.output.len() != self.receipt.output_bytes {
            return Err(
                "Invalid command-session revision or output. Prior saved work remains.".into(),
            );
        }
        Ok(())
    }
}
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OutputPage {
    pub receipt: SessionReceipt,
    pub offset: usize,
    pub next_offset: usize,
    pub has_more: bool,
    pub output: String,
    pub lossy_utf8: bool,
}
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ReceiptPage {
    pub items: Vec<SessionReceipt>,
    pub next_cursor: Option<i64>,
}
