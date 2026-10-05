//! Closed desktop-input vocabulary shared with the isolated native broker.
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(tag = "operation", rename_all = "camelCase", deny_unknown_fields)]
pub enum Input {
    Click {
        x: u32,
        y: u32,
    },
    DoubleClick {
        x: u32,
        y: u32,
    },
    Type {
        text: String,
    },
    Scroll {
        x: u32,
        y: u32,
        delta: i32,
    },
    Key {
        key: String,
    },
    Drag {
        x: u32,
        y: u32,
        #[serde(rename = "endX")]
        end_x: u32,
        #[serde(rename = "endY")]
        end_y: u32,
    },
}
impl Input {
    pub fn validate(&self, width: u32, height: u32) -> Result<(), String> {
        let point = |x: u32, y: u32| x < width && y < height && width > 0 && height > 0;
        let valid = match self {
            Self::Click { x, y } | Self::DoubleClick { x, y } => point(*x, *y),
            Self::Drag { x, y, end_x, end_y } => point(*x, *y) && point(*end_x, *end_y),
            Self::Scroll { x, y, delta } => {
                point(*x, *y) && *delta != 0 && delta.unsigned_abs() <= 1200
            }
            Self::Type { text } => {
                !text.is_empty() && text.len() <= 512 && !text.chars().any(char::is_control)
            }
            Self::Key { key } => [
                "Tab",
                "Shift+Tab",
                "Left",
                "Right",
                "Up",
                "Down",
                "Home",
                "End",
                "Ctrl+A",
                "Backspace",
                "Delete",
                "Enter",
                "Escape",
            ]
            .contains(&key.as_str()),
        };
        if valid {
            Ok(())
        } else {
            Err("Use checked screenshot coordinates, 1–512 bytes of plain text, a supported key, or a scroll delta within ±1200. No input was dispatched.".into())
        }
    }
    /// Host classification; a model cannot classify its own external effects.
    pub fn ordinary(&self) -> bool {
        matches!(self, Self::Type { .. } | Self::Scroll { .. })
            || matches!(self, Self::Key { key } if ["Tab", "Shift+Tab", "Left", "Right", "Up", "Down", "Home", "End", "Ctrl+A"].contains(&key.as_str()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn closed_vocabulary_and_coordinate_bounds() {
        assert!(serde_json::from_str::<Input>(
            r#"{"operation":"key","key":"Enter","script":"run"}"#
        )
        .is_err());
        assert!(Input::Click { x: 100, y: 10 }.validate(100, 40).is_err());
        assert!(Input::Key {
            key: "Alt+Tab".into()
        }
        .validate(10, 10)
        .is_err());
        assert!(Input::Type {
            text: "line\nsubmit".into()
        }
        .validate(10, 10)
        .is_err());
        assert!(Input::Scroll {
            x: 0,
            y: 0,
            delta: i32::MIN
        }
        .validate(10, 10)
        .is_err());
    }
    #[test]
    fn submission_and_pointer_actions_always_require_review() {
        assert!(!Input::Click { x: 0, y: 0 }.ordinary());
        assert!(!Input::Key {
            key: "Enter".into()
        }
        .ordinary());
        assert!(Input::Type {
            text: "local draft".into()
        }
        .ordinary());
    }
}
