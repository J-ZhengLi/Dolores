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
            model_calls: 0,
            usage_by_call: vec![],
            tools: vec![],
            steps: vec![],
            thinking: vec![],
        })
    }
}
impl Progress {
    pub fn record(&mut self, event: &AgentEvent) {
        match event {
            AgentEvent::ModelStep { number } => {
                self.0.model_calls = *number;
                self.0.usage_by_call.resize(*number, None);
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
            AgentEvent::ModelFinished { number, usage } => {
                self.0.usage_by_call.resize(*number, None);
                self.0.usage_by_call[*number - 1] = usage.clone();
            }
            AgentEvent::ToolResult { record } => self.0.tools.push(*record.clone()),
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
