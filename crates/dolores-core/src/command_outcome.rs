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
    #[serde(default)]
    output_retained: bool,
    #[serde(default)]
    log_error: bool,
}

impl CommandOutcome {
    pub fn from_content(content: &str) -> Self {
        let Ok(mut value) = serde_json::from_str::<serde_json::Value>(content) else {
            return Self::Incomplete;
        };
        if value.get("cleanupComplete").is_some_and(|v| v != true) {
            return Self::Incomplete;
        }
        if let Some(session) = value.get("session") {
            if session["cleanupComplete"] != true {
                return Self::Incomplete;
            }
            let mut receipt = session.clone();
            receipt["reason"] = session["state"].clone();
            value = receipt;
        }
        let Ok(receipt) = serde_json::from_value::<Receipt>(value) else {
            return Self::Incomplete;
        };
        if receipt.reason != "completed" {
            return Self::Incomplete;
        }
        match receipt.exit_code {
            Some(code) if code != 0 => Self::Failed,
            Some(0)
                if ((!receipt.truncated && !receipt.lossy_utf8) || receipt.output_retained)
                    && !receipt.output_error
                    && !receipt.log_error =>
            {
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
    let mut sessions = std::collections::HashMap::<String, (CommandSpec, bool)>::new();
    for record in records {
        if record.name == "command_session" {
            if matches!(record.status.as_str(), "denied" | "blocked") {
                continue;
            }
            let value =
                serde_json::from_str::<serde_json::Value>(&record.content).unwrap_or_default();
            if let Some(id) = value["session"]["id"].as_str() {
                if let Some(command) = &record.command {
                    sessions.entry(id.into()).or_insert((command.clone(), true));
                }
                if let Some((command, live)) = sessions.get_mut(id) {
                    *live = matches!(
                        value["session"]["state"].as_str(),
                        Some("starting" | "running" | "stopping")
                    );
                    if !*live {
                        if record.status != "error"
                            && CommandOutcome::from_content(&record.content)
                                == CommandOutcome::Succeeded
                        {
                            pending.retain(|c| c != command);
                        } else if !pending.contains(command) {
                            pending.push(command.clone());
                        }
                    }
                }
            } else if let Some(command) = &record.command {
                if !pending.contains(command) {
                    pending.push(command.clone());
                }
            }
            continue;
        }
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
    // A second successful invocation cannot resolve an identical command that
    // is still running in another owned session.
    for (command, live) in sessions.values() {
        if *live && !pending.contains(command) {
            pending.push(command.clone());
        }
    }
    pending
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn live_duplicate_sessions_require_individual_final_receipts_before_success() {
        let command = CommandSpec {
            program: "node".into(),
            args: vec!["check.cjs".into()],
        };
        let record = |id: &str, state: &str, start: bool| {
            ToolRecord {
            call_id:format!("{id}-{state}"),name:"command_session".into(),target:id.into(),status:"completed".into(),parts:vec![],query:None,diff:None,mcp:None,
            command:if start{Some(command.clone())}else{None},
            content:serde_json::json!({"session":{"id":id,"state":state,"exitCode":0,"cleanupComplete":state=="completed"||state=="closed","truncated":false,"outputError":false,"lossyUtf8":false}}).to_string(),
        }
        };
        let mut records = vec![
            record("a", "running", true),
            record("b", "running", true),
            record("b", "completed", false),
        ];
        assert_eq!(unresolved_commands(&records), vec![command.clone()]);
        records.push(record("a", "closed", false));
        assert_eq!(unresolved_commands(&records), vec![command.clone()]);
        records.push(record("c", "running", true));
        records.push(record("c", "completed", false));
        assert!(unresolved_commands(&records).is_empty());
        let mut uncertain = record("c", "completed", false);
        uncertain.content = uncertain.content.replace("true", "false");
        assert_eq!(
            CommandOutcome::from_content(&uncertain.content),
            CommandOutcome::Incomplete
        );
    }
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
            parts: vec![],
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
