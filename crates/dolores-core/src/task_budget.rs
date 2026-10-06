use serde::{Deserialize, Serialize};

/// One explicit segment; continuations cannot silently reset the task allowance.
#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct TaskBudget {
    pub model_calls: Option<usize>,
    pub tool_calls: Option<usize>,
    pub segments: u32,
    pub elapsed_seconds: Option<u32>,
}
impl Default for TaskBudget {
    fn default() -> Self {
        Self {
            model_calls: None,
            tool_calls: None,
            segments: 4,
            elapsed_seconds: None,
        }
    }
}
impl TaskBudget {
    /// Automatic work remains bounded by a saved resource checkpoint, not a tiny
    /// user-facing execution allowance. Explicit saved limits remain authoritative.
    pub fn model_limit(self) -> usize {
        self.model_calls.unwrap_or(64)
    }
    pub fn tool_limit(self) -> usize {
        self.tool_calls.unwrap_or(128)
    }
    pub fn bounded(self) -> Self {
        Self {
            model_calls: Some(self.model_limit()),
            tool_calls: Some(self.tool_limit()),
            ..self
        }
    }
    pub fn validate(self) -> Result<(), String> {
        if self.model_calls.is_some_and(|n| !(2..=128).contains(&n))
            || self.tool_calls.is_some_and(|n| !(1..=256).contains(&n))
            || !(1..=8).contains(&self.segments)
            || self
                .elapsed_seconds
                .is_some_and(|s| !(30..=3600).contains(&s))
        {
            return Err("Task limits require 2–128 model calls and 1–256 tool operations (or blank for Automatic), 1–8 segments and 30–3600 seconds (or no task time limit). Previous settings remain unchanged.".into());
        }
        Ok(())
    }
    /// Only an unattended desktop-sharing handoff needs a sealed finite remainder.
    /// Normal execution uses elapsed_seconds directly; None has no whole-task clock.
    pub fn handoff_deadline(self, request_seconds: u32) -> u32 {
        self.elapsed_seconds.unwrap_or(request_seconds.min(300))
    }
    pub fn check_segment(self, used: u32) -> Result<(), String> {
        self.validate()?;
        if used >= self.segments {
            return Err(format!("This task has used its {} segments. Review saved progress and explicitly change Task limits before continuing. Nothing was replayed.", self.segments));
        }
        Ok(())
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn automatic_defaults_and_explicit_saved_limits_never_reset() {
        let b = TaskBudget::default();
        assert_eq!(
            (b.model_calls, b.tool_calls, b.elapsed_seconds),
            (None, None, None)
        );
        let old: TaskBudget = serde_json::from_value(
            serde_json::json!({"modelCalls":4,"toolCalls":4,"segments":4,"elapsedSeconds":120}),
        )
        .unwrap();
        assert_eq!(
            (old.model_calls, old.tool_calls, old.elapsed_seconds),
            (Some(4), Some(4), Some(120))
        );
        assert_eq!(
            (b.bounded().model_calls, b.bounded().tool_calls),
            (Some(64), Some(128))
        );
        assert!(b.check_segment(3).is_ok());
        assert!(b
            .check_segment(4)
            .unwrap_err()
            .contains("Nothing was replayed"));
        assert!(TaskBudget {
            tool_calls: Some(257),
            ..b
        }
        .validate()
        .is_err());
        assert_eq!(
            TaskBudget {
                elapsed_seconds: Some(600),
                ..b
            }
            .handoff_deadline(60),
            600
        );
    }
}
