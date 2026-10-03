use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum ReasoningControl {
    #[default]
    ProviderDefault,
    DeepseekThinkingOff,
    OpenaiLow,
    OpenaiMedium,
    OpenaiHigh,
}
impl ReasoningControl {
    pub fn is_default(&self) -> bool {
        *self == Self::ProviderDefault
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct RequestSettings {
    pub max_output_tokens: u32,
    pub timeout_seconds: u32,
    #[serde(default, skip_serializing_if = "ReasoningControl::is_default")]
    pub reasoning: ReasoningControl,
}

impl Default for RequestSettings {
    fn default() -> Self {
        Self {
            max_output_tokens: 2048,
            timeout_seconds: 180,
            reasoning: ReasoningControl::ProviderDefault,
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
                    timeout_seconds: seconds,
                    ..Default::default()
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
    #[test]
    fn old_settings_keep_wire_shape_and_unknown_reasoning_is_rejected() {
        let old = r#"{"maxOutputTokens":2048,"timeoutSeconds":180}"#;
        let settings: RequestSettings = serde_json::from_str(old).unwrap();
        assert_eq!(settings.reasoning, ReasoningControl::ProviderDefault);
        assert_eq!(serde_json::to_string(&settings).unwrap(), old);
        for reasoning in ["auto", "deepseekThinkingOn", "arbitrary"] {
            assert!(serde_json::from_value::<RequestSettings>(serde_json::json!({"maxOutputTokens":2048,"timeoutSeconds":180,"reasoning":reasoning})).is_err());
        }
    }
}
