use serde::{Deserialize, Serialize};

/// One explicit segment; continuations cannot silently reset the task allowance.
#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct TaskBudget {
    pub model_calls: usize,
    pub tool_calls: usize,
    pub segments: u32,
    pub elapsed_seconds: Option<u32>,
}
impl Default for TaskBudget {
    fn default() -> Self {
        Self {
            model_calls: 4,
            tool_calls: 4,
            segments: 4,
            elapsed_seconds: None,
        }
    }
}
impl TaskBudget {
    pub fn validate(self) -> Result<(), String> {
        if !(2..=16).contains(&self.model_calls)
            || !(1..=32).contains(&self.tool_calls)
            || !(1..=8).contains(&self.segments)
            || self
                .elapsed_seconds
                .is_some_and(|s| !(30..=3600).contains(&s))
        {
            return Err("Task limits require 2–16 model calls, 1–32 tool operations, 1–8 segments and 30–3600 seconds (or an inherited deadline). Previous settings remain unchanged.".into());
        }
        Ok(())
    }
    pub fn deadline(self, request_seconds: u32) -> u32 {
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
    fn defaults_preserve_old_limits_and_segments_never_reset() {
        let b = TaskBudget::default();
        assert_eq!((b.model_calls, b.tool_calls, b.deadline(60)), (4, 4, 60));
        assert!(b.check_segment(3).is_ok());
        assert!(b
            .check_segment(4)
            .unwrap_err()
            .contains("Nothing was replayed"));
        assert!(TaskBudget {
            tool_calls: 33,
            ..b
        }
        .validate()
        .is_err());
        assert_eq!(
            TaskBudget {
                elapsed_seconds: Some(600),
                ..b
            }
            .deadline(60),
            600
        );
    }
}
