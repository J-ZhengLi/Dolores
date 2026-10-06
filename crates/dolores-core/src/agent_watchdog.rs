use super::{CommandOutcome, ToolCall, ToolRecord};
use std::collections::BTreeMap;

/// Detect failed attempts, not elapsed time or verbose thinking. Successful work
/// clears the streak; a repaired request can still run through normal approval.
#[derive(Default)]
pub(crate) struct FailureWatchdog {
    failed: BTreeMap<String, usize>,
    consecutive: usize,
}
impl FailureWatchdog {
    fn key(call: &ToolCall) -> String {
        let arguments = serde_json::from_str::<serde_json::Value>(&call.arguments)
            .map(|v| v.to_string())
            .unwrap_or_else(|_| call.arguments.clone());
        format!("{}:{arguments}", call.name)
    }
    pub fn repeats_failure(&self, call: &ToolCall) -> bool {
        self.failed.get(&Self::key(call)).copied().unwrap_or(0) >= 2
    }
    pub fn record(&mut self, call: &ToolCall, receipt: &ToolRecord) -> bool {
        let failed = !matches!(
            receipt.status.as_str(),
            "completed" | "read" | "created" | "edited"
        ) || (receipt.name.starts_with("mcp_tool_")
            && serde_json::from_str::<serde_json::Value>(&receipt.content)
                .is_ok_and(|v| v["isError"] == true))
            || (receipt.name == "run_command"
                && CommandOutcome::from_content(&receipt.content) != CommandOutcome::Succeeded);
        if failed {
            self.consecutive += 1;
            *self.failed.entry(Self::key(call)).or_default() += 1;
        } else {
            self.failed.clear();
            self.consecutive = 0;
        }
        self.consecutive >= 6
    }
}
