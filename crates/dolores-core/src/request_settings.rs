use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct RequestSettings {
    pub max_output_tokens: u32,
    pub timeout_seconds: u32,
}

impl Default for RequestSettings {
    fn default() -> Self {
        Self {
            max_output_tokens: 2048,
            timeout_seconds: 180,
        }
    }
}
impl RequestSettings {
    pub fn validate(&self) -> Result<(), String> {
        if !(1..=32768).contains(&self.max_output_tokens) {
            return Err("Output token limit must be between 1 and 32768.".into());
        }
        if !(1..=900).contains(&self.timeout_seconds) {
            return Err("Request timeout must be between 1 and 900 seconds.".into());
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn settings_validate_bounds_and_reject_malformed_contracts() {
        assert!(RequestSettings::default().validate().is_ok());
        for (tokens, seconds, valid) in [
            (1, 1, true),
            (32768, 900, true),
            (0, 180, false),
            (32769, 180, false),
            (2048, 0, false),
            (2048, 901, false),
        ] {
            assert_eq!(
                RequestSettings {
                    max_output_tokens: tokens,
                    timeout_seconds: seconds
                }
                .validate()
                .is_ok(),
                valid
            );
        }
        for json in [
            r#"{"maxOutputTokens":1.5,"timeoutSeconds":180}"#,
            r#"{"maxOutputTokens":-1,"timeoutSeconds":180}"#,
            r#"{"maxOutputTokens":2048}"#,
            r#"{"maxOutputTokens":2048,"timeoutSeconds":180,"automaticRetry":true}"#,
        ] {
            assert!(serde_json::from_str::<RequestSettings>(json).is_err());
        }
    }
}
