use crate::{CommandSpec, ToolRecord};
use serde::Deserialize;

/// Process completion is not evidence that its checks passed.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CommandOutcome {
    Succeeded,
    Failed,
    Incomplete,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Receipt {
    reason: String,
    exit_code: Option<i64>,
    truncated: bool,
    #[serde(default)]
    lossy_utf8: bool,
    #[serde(default)]
    output_error: bool,
}

impl CommandOutcome {
    pub fn from_content(content: &str) -> Self {
        let Ok(receipt) = serde_json::from_str::<Receipt>(content) else {
            return Self::Incomplete;
        };
        if receipt.reason != "completed" {
            return Self::Incomplete;
        }
        match receipt.exit_code {
            Some(code) if code != 0 => Self::Failed,
            Some(0) if !receipt.truncated && !receipt.lossy_utf8 && !receipt.output_error => {
                Self::Succeeded
            }
            _ => Self::Incomplete,
        }
    }

    pub fn status(self) -> &'static str {
        match self {
            Self::Succeeded => "completed",
            Self::Failed => "failed",
            Self::Incomplete => "incomplete",
        }
    }
}

/// Only a later complete, zero-exit receipt for the same literal invocation
/// resolves a failed check. Unrelated successful commands cannot clear it.
pub fn unresolved_commands(records: &[ToolRecord]) -> Vec<CommandSpec> {
    let mut pending = Vec::new();
    for record in records {
        if record.name != "run_command" {
            continue;
        }
        let Some(command) = &record.command else {
            continue;
        };
        if matches!(record.status.as_str(), "denied" | "blocked") {
            continue;
        }
        let success = record.status != "error"
            && CommandOutcome::from_content(&record.content) == CommandOutcome::Succeeded;
        if success {
            pending.retain(|c| c != command);
        } else if !pending.contains(command) {
            pending.push(command.clone());
        }
    }
    pending
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn exit_and_capture_evidence_must_both_be_complete() {
        for (content, expected) in [
            (
                r#"{"reason":"completed","exitCode":0,"truncated":false}"#,
                CommandOutcome::Succeeded,
            ),
            (
                r#"{"reason":"completed","exitCode":7,"truncated":true}"#,
                CommandOutcome::Failed,
            ),
            (
                r#"{"reason":"completed","exitCode":0,"truncated":true}"#,
                CommandOutcome::Incomplete,
            ),
            (
                r#"{"reason":"timedOut","exitCode":0,"truncated":false}"#,
                CommandOutcome::Incomplete,
            ),
            (
                r#"{"reason":"completed","exitCode":null,"truncated":false}"#,
                CommandOutcome::Incomplete,
            ),
            (
                r#"{"reason":"completed","exitCode":0,"truncated":false,"outputError":true}"#,
                CommandOutcome::Incomplete,
            ),
            (
                r#"{"reason":"completed","exitCode":0,"truncated":false,"lossyUtf8":true}"#,
                CommandOutcome::Incomplete,
            ),
            (r#"{"exitCode":0}"#, CommandOutcome::Incomplete),
            ("Checks passed", CommandOutcome::Incomplete),
        ] {
            assert_eq!(CommandOutcome::from_content(content), expected);
        }
    }
    #[test]
    fn only_exact_successful_rerun_clears_failure_including_legacy_receipts() {
        let record = |arg: &str, code: i64, status: &str| ToolRecord {
            call_id: "check".into(),
            name: "run_command".into(),
            target: "node".into(),
            status: status.into(),
            content: format!(r#"{{"reason":"completed","exitCode":{code},"truncated":false}}"#),
            query: None,
            diff: None,
            mcp: None,
            command: Some(CommandSpec {
                program: "node".into(),
                args: vec![arg.into()],
            }),
        };
        let mut records = vec![
            record("check.cjs", 1, "completed"),
            record("other.cjs", 0, "completed"),
        ];
        assert_eq!(unresolved_commands(&records).len(), 1);
        records.push(record("check.cjs", 0, "denied"));
        assert_eq!(unresolved_commands(&records).len(), 1);
        records.push(record("check.cjs", 0, "completed"));
        assert!(unresolved_commands(&records).is_empty());
    }
}
