use crate::{forward, stopped};
use dolores_core::{AgentEvent, AgentReply, AgentSummary, ModelText, PauseReason, ToolApproval};
use serde_json::Value;
use tokio::sync::mpsc;
use tokio_util::sync::CancellationToken;

pub struct CancelOnDrop(pub CancellationToken);
impl Drop for CancelOnDrop {
    fn drop(&mut self) {
        self.0.cancel();
    }
}

/// An optional active-work cap, independent of the model's inactivity allowance.
/// Checking host review state also covers a subagent waiting on the shared chooser.
pub async fn active_deadline(seconds: Option<u32>, approval: &dyn ToolApproval) {
    let Some(seconds) = seconds else {
        std::future::pending::<()>().await;
        return;
    };
    let mut remaining = std::time::Duration::from_secs(seconds.into());
    let mut last = tokio::time::Instant::now();
    let mut waiting = approval.is_waiting();
    loop {
        tokio::time::sleep(remaining.min(std::time::Duration::from_millis(50))).await;
        let now = tokio::time::Instant::now();
        let next_waiting = approval.is_waiting();
        if !waiting && !next_waiting {
            remaining = remaining.saturating_sub(now - last);
        }
        if remaining.is_zero() {
            return;
        }
        waiting = next_waiting;
        last = now;
    }
}

pub async fn deliver(
    output: &mpsc::Sender<Value>,
    event: Value,
    cancel: &CancellationToken,
    seconds: u32,
) -> Result<(), String> {
    tokio::select! { biased;
        _ = cancel.cancelled() => Err(stopped()),
        result = tokio::time::timeout(std::time::Duration::from_secs(seconds.into()), forward(output, event, cancel)) => result.map_err(|_| "Conversation delivery stalled. Reopen the chat; completed changes remain. Nothing was retried.")?,
    }
}

pub struct Progress(AgentSummary);
impl Default for Progress {
    fn default() -> Self {
        Self(AgentSummary {
            archive: None,
            model_calls: 0,
            usage_by_call: vec![],
            tools: vec![],
            tool_steps: vec![],
            steps: vec![],
            thinking: vec![],
        })
    }
}
impl Progress {
    pub fn record(&mut self, event: &AgentEvent) {
        match event {
            AgentEvent::ModelStep { number } => {
                self.0.model_calls = self.0.model_calls.max(*number);
                self.0.usage_by_call.push(None);
            }
            AgentEvent::ModelText { number, text } => {
                if let Some(step) = self.0.steps.iter_mut().find(|s| s.number == *number) {
                    step.text.push_str(text);
                } else {
                    self.0.steps.push(ModelText {
                        number: *number,
                        text: text.clone(),
                    });
                }
            }
            AgentEvent::ModelThinking { number, text } => {
                if let Some(step) = self.0.thinking.iter_mut().find(|s| s.number == *number) {
                    step.text = text.clone();
                } else {
                    self.0.thinking.push(ModelText {
                        number: *number,
                        text: text.clone(),
                    });
                }
            }
            AgentEvent::ModelFinished { usage, .. } => {
                if let Some(last) = self.0.usage_by_call.last_mut() {
                    *last = usage.clone();
                }
            }
            AgentEvent::ModelRetry { number, .. } => {
                if let Some(step) = self.0.steps.iter_mut().find(|s| s.number == *number) {
                    step.text.push_str("\n\n[This model attempt was interrupted before tool dispatch. Its proposed actions were not executed.]");
                }
            }
            AgentEvent::TaskSegment {
                archive,
                retained,
                tool_steps,
            } => {
                self.0.model_calls = archive.model_calls;
                self.0.archive = Some(archive.clone());
                self.0.steps.clear();
                self.0.thinking.clear();
                self.0.tools = retained.clone();
                self.0.tool_steps = tool_steps.clone();
                self.0.usage_by_call.clear();
            }
            AgentEvent::ToolResult { record } => {
                self.0.tools.push(*record.clone());
                self.0.tool_steps.push(self.0.model_calls);
            }
            _ => {}
        }
    }
    pub fn paused(&self) -> AgentReply {
        let partial = self.0.steps.last().map(|s| s.text.as_str()).unwrap_or("");
        AgentReply {
            pause: Some(PauseReason::TaskTimeout),
            answer: format!("{partial}\n\nPaused at your task time limit. Progress and completed changes are saved. Inspect Changes before continuing; an interrupted operation may have effects. Pending approvals and incomplete tool calls will not be replayed."),
            summary: self.0.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn maintenance_retains_uncertain_receipts_for_timeout_recovery() {
        let receipt = serde_json::from_value(serde_json::json!({
            "callId":"uncertain", "name":"command_session", "target":"fixture",
            "status":"running", "content":"{\"session\":{\"id\":\"owned\",\"state\":\"running\"}}"
        }))
        .unwrap();
        let mut progress = Progress::default();
        progress.record(&AgentEvent::TaskSegment {
            archive: dolores_core::TaskArchive {
                run_id: "fixture".into(),
                through_segment: 2,
                tool_count: 32,
                model_calls: 40,
            },
            retained: vec![receipt],
            tool_steps: vec![39],
        });
        let paused = progress.paused();
        assert_eq!(paused.summary.tools[0].call_id, "uncertain");
        assert_eq!(paused.summary.tool_steps, [39]);
        assert_eq!(paused.summary.model_calls, 40);
        assert_eq!(paused.summary.archive.unwrap().tool_count, 32);
    }
}
