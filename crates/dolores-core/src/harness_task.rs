//! EH00 delivery contract. This does not dispatch work or grant tool access.
//! EH01 owns persistence and integration with the existing run owner.
use serde::{Deserialize, Serialize};

/// Per-chat pending input capacity, independent of host run admission.
/// Eight existing-size inputs bound text storage to 128 KiB before references.
pub const MAX_PENDING_INPUTS: usize = 8;

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TaskInputs {
    pub revision: u32,
    pub suspended: bool,
    pub recovery_required: bool,
    pub entries: Vec<TaskInput>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TaskInput {
    pub id: String,
    pub revision: u32,
    pub position: u64,
    pub mode: InputMode,
    pub text: String,
    pub state: InputState,
    pub run: Option<String>,
    #[serde(default)]
    pub inspected: bool,
    #[serde(default)]
    pub selected_skills: Vec<crate::SkillSelection>,
    #[serde(default)]
    pub planning: crate::planning::PlanningState,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "action", rename_all = "camelCase", deny_unknown_fields)]
pub enum InputEdit {
    Acknowledge {
        id: String,
        revision: u32,
    },
    Enqueue {
        id: String,
        text: String,
        mode: InputMode,
    },
    Steer {
        id: String,
        revision: u32,
        // The bridge binds this to its live owner, never to a client-supplied run.
        #[serde(skip)]
        run: Option<String>,
    },
    Edit {
        id: String,
        revision: u32,
        text: String,
    },
    Cancel {
        id: String,
        revision: u32,
    },
    Reorder {
        revision: u32,
        ids: Vec<String>,
    },
    Resume {
        revision: u32,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum InputMode {
    Queue,
    Steer,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum InputState {
    Pending,
    Delivering,
    Applied,
    Cancelled,
    Failed,
    Uncertain,
}

impl InputState {
    /// A crash can occur after submission but before its receipt is saved.
    /// Neither a missing receipt nor a stored intent permits automatic replay.
    pub fn after_restart(self) -> Self {
        match self {
            Self::Delivering => Self::Uncertain,
            other => other,
        }
    }

    pub fn editable(self) -> bool {
        self == Self::Pending
    }

    /// Delivery is terminal unless it has an acknowledged safe outcome.
    /// Retry from failed/uncertain requires a new input ID after inspection.
    pub fn transition(self, next: Self) -> Result<Self, &'static str> {
        use InputState::*;
        match (self, next) {
            (Pending, Delivering | Cancelled) | (Delivering, Applied | Failed | Uncertain) => {
                Ok(next)
            }
            _ => {
                Err("Instruction changed or was already delivered. Refresh to inspect its receipt.")
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum DispatchAction {
    Stop,
    Wait,
    Steer,
    Queue,
    ContinueGoal,
}

/// Snapshot evaluated under the task-owner lock, never used as a new run queue.
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct DispatchState {
    pub stop_requested: bool,
    pub suspended: bool,
    pub recovery_required: bool,
    pub decision_pending: bool,
    pub primary_active: bool,
    pub safe_boundary: bool,
    pub steering_pending: bool,
    pub queued_pending: bool,
    pub goal_ready: bool,
}

impl DispatchState {
    pub fn next_action(self) -> DispatchAction {
        use DispatchAction::*;
        if self.stop_requested {
            Stop
        } else if self.suspended || self.recovery_required || self.decision_pending {
            Wait
        } else if self.primary_active {
            if self.safe_boundary && self.steering_pending {
                Steer
            } else {
                Wait
            }
        } else if self.steering_pending {
            // A late steer keeps its visible mode but becomes the next turn.
            Steer
        } else if self.queued_pending {
            Queue
        } else if self.goal_ready {
            ContinueGoal
        } else {
            Wait
        }
    }
}

/// Stale writes and revision overflow leave the original record unchanged.
pub fn next_revision(current: u32, expected: u32) -> Result<u32, &'static str> {
    if current != expected {
        return Err("Task changed. Refresh before trying again.");
    }
    current.checked_add(1).ok_or("Task revision exhausted.")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn public_dispatch_corpus_freezes_stop_decision_and_user_input_priority() {
        #[derive(Deserialize)]
        struct Case {
            name: String,
            state: DispatchState,
            expected: DispatchAction,
        }
        let cases: Vec<Case> =
            serde_json::from_str(include_str!("../../../docs/fixtures/harness-dispatch.json"))
                .unwrap();
        assert!(cases.len() >= 10);
        for case in cases {
            assert_eq!(case.state.next_action(), case.expected, "{}", case.name);
        }
    }

    #[test]
    fn crash_after_delivery_intent_requires_inspection_without_replay() {
        let accepted = InputState::Pending;
        assert_eq!(accepted.after_restart(), accepted);
        let sending = accepted.transition(InputState::Delivering).unwrap();
        let recovered = sending.after_restart();
        assert_eq!(recovered, InputState::Uncertain);
        assert!(!recovered.editable());
        assert!(recovered.transition(InputState::Pending).is_err());
        assert!(recovered.transition(InputState::Applied).is_err());
        assert_eq!(InputState::Applied.after_restart(), InputState::Applied);
    }

    #[test]
    fn stale_edit_and_terminal_receipt_cannot_be_overwritten() {
        assert_eq!(next_revision(2, 2), Ok(3));
        assert!(next_revision(2, 1).is_err());
        assert!(next_revision(u32::MAX, u32::MAX).is_err());
        assert!(InputState::Pending.editable());
        assert!(InputState::Pending.transition(InputState::Applied).is_err());
        for state in [
            InputState::Applied,
            InputState::Cancelled,
            InputState::Failed,
            InputState::Uncertain,
        ] {
            for next in [
                InputState::Pending,
                InputState::Delivering,
                InputState::Applied,
                InputState::Cancelled,
                InputState::Failed,
                InputState::Uncertain,
            ] {
                assert!(state.transition(next).is_err());
            }
        }
    }
}
