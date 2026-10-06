use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ExperimentalPreferences {
    pub revision: u32,
    pub multiple_window: bool,
    pub prevent_windows_from_locked: bool,
}
impl Default for ExperimentalPreferences {
    fn default() -> Self {
        Self {
            revision: 0,
            multiple_window: true,
            prevent_windows_from_locked: false,
        }
    }
}
