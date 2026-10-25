use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum Event {
    TaskStart,
    BeforeTool,
    AfterTool,
    BeforeCompaction,
    TaskComplete,
}
#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum Mode {
    Blocking,
    Observe,
}
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Hook {
    pub name: String,
    pub event: Event,
    pub mode: Mode,
    pub program: String,
    pub script: String,
    pub args: Vec<String>,
    pub timeout_seconds: u64,
}
impl Hook {
    pub fn validate(&self) -> Result<(), String> {
        if self.name.is_empty()
            || self.name.len() > 64
            || !self
                .name
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || b"_-".contains(&b))
            || !matches!(self.program.as_str(), "node" | "python")
            || !(1..=10).contains(&self.timeout_seconds)
            || self.args.len() > 16
            || self
                .args
                .iter()
                .any(|s| s.len() > 1024 || s.chars().any(char::is_control))
            || self.args.iter().map(String::len).sum::<usize>() > 4096
            || self.script.is_empty()
            || self.script.len() > 240
            || self.script.contains(['\\', ':'])
            || self
                .script
                .split('/')
                .any(|s| s.is_empty() || matches!(s, "." | "..") || s.chars().any(char::is_control))
            || self.mode == Mode::Blocking
                && matches!(self.event, Event::AfterTool | Event::TaskComplete)
        {
            return Err("Use a unique short hook name, node/python, a project-relative script, bounded literal arguments and 1–10 seconds. Post-event hooks must observe.".into());
        }
        Ok(())
    }
}
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ReviewedHook {
    pub hook: Hook,
    pub executable: String,
    pub script_sha256: String,
}
#[derive(Clone, Debug, Default, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Configuration {
    pub revision: u32,
    pub hooks: Vec<ReviewedHook>,
}
impl Configuration {
    pub fn validate(&self) -> Result<(), String> {
        if self.hooks.len() > 4 {
            return Err("Choose at most four project hooks.".into());
        }
        let mut names = std::collections::BTreeSet::new();
        for saved in &self.hooks {
            saved.hook.validate()?;
            if !names.insert(&saved.hook.name)
                || !std::path::Path::new(&saved.executable).is_absolute()
                || saved.executable.len() > 4096
                || saved.script_sha256.len() != 64
                || !saved.script_sha256.bytes().all(|b| b.is_ascii_hexdigit())
            {
                return Err("Reviewed hook identity is invalid. Review the hooks again.".into());
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn hook_authority_and_shape_are_bounded_before_review() {
        let hook = Hook {
            name: "check".into(),
            event: Event::BeforeTool,
            mode: Mode::Blocking,
            program: "node".into(),
            script: "scripts/check.cjs".into(),
            args: vec![],
            timeout_seconds: 5,
        };
        hook.validate().unwrap();
        for script in [
            "../escape.cjs",
            "/absolute.cjs",
            "C:/file.cjs",
            "x\\file.cjs",
            "a//b",
        ] {
            assert!(Hook {
                script: script.into(),
                ..hook.clone()
            }
            .validate()
            .is_err());
        }
        assert!(Hook {
            event: Event::AfterTool,
            ..hook.clone()
        }
        .validate()
        .is_err());
        assert!(Hook {
            timeout_seconds: 11,
            ..hook.clone()
        }
        .validate()
        .is_err());
        let saved = ReviewedHook {
            hook,
            executable: if cfg!(windows) {
                "C:/Runtime/node.exe"
            } else {
                "/usr/bin/node"
            }
            .into(),
            script_sha256: "a".repeat(64),
        };
        assert!(Configuration {
            revision: 0,
            hooks: vec![saved.clone(), saved]
        }
        .validate()
        .is_err());
        assert!(serde_json::from_value::<Hook>(serde_json::json!({"name":"check","event":"beforeTool","mode":"blocking","program":"node","script":"check.cjs","args":[],"timeoutSeconds":5,"permissions":"all"})).is_err());
    }
}
