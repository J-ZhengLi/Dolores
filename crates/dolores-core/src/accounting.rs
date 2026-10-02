use crate::{Message, HISTORY_LIMIT, MAX_CONTEXT_BYTES};
use serde::{Deserialize, Serialize};

/// Only counters actually reported by the provider; None never means zero.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct TokenUsage {
    pub input_tokens: Option<u64>,
    pub output_tokens: Option<u64>,
    pub total_tokens: Option<u64>,
    pub cached_input_tokens: Option<u64>,
    pub reasoning_tokens: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ContextSummary {
    pub included_turns: usize,
    pub saved_turns: Option<u64>,
    pub omitted_turns: Option<u64>,
    pub text_bytes: usize,
    pub max_text_bytes: usize,
    pub max_turns: usize,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tokens: Option<crate::TokenContext>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub instructions: Option<crate::InstructionSource>,
}

impl ContextSummary {
    pub fn from_messages(messages: &[Message], saved_turns: Option<u64>) -> Self {
        let included_turns = messages.len().saturating_sub(2) / 2;
        Self {
            included_turns,
            saved_turns,
            omitted_turns: saved_turns.map(|n| n.saturating_sub(included_turns as u64)),
            text_bytes: messages.iter().map(|m| m.content.len()).sum(),
            max_text_bytes: MAX_CONTEXT_BYTES,
            max_turns: HISTORY_LIMIT / 2,
            tokens: None,
            instructions: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct TurnMetadata {
    pub model: String,
    pub usage: Option<TokenUsage>,
    pub context: ContextSummary,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub request_settings: Option<crate::RequestSettings>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub agent: Option<crate::AgentSummary>,
}

pub struct Reply {
    pub answer: String,
    pub usage: Option<TokenUsage>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{preview_context, Role};
    #[test]
    fn summary_counts_latest_whole_turns_and_utf8_bytes_with_unknown_totals() {
        let history: Vec<_> = (0..50)
            .flat_map(|n| {
                [
                    Message {
                        role: Role::User,
                        content: format!("你好 {n}"),
                    },
                    Message {
                        role: Role::Assistant,
                        content: "answer".into(),
                    },
                ]
            })
            .collect();
        let messages = preview_context(history, "世界").unwrap();
        let summary = ContextSummary::from_messages(&messages, Some(50));
        assert_eq!(summary.included_turns, 40);
        assert_eq!(summary.omitted_turns, Some(10));
        assert_eq!(messages[1].content, "你好 10");
        assert_eq!(
            summary.text_bytes,
            messages.iter().map(|m| m.content.len()).sum::<usize>()
        );
        assert!(ContextSummary::from_messages(&messages, None)
            .omitted_turns
            .is_none());
    }
    #[test]
    fn preview_of_empty_draft_is_readable_and_byte_budget_keeps_newest_turns() {
        let history = (0..40)
            .flat_map(|n| {
                [
                    Message {
                        role: Role::User,
                        content: "界".repeat(2000),
                    },
                    Message {
                        role: Role::Assistant,
                        content: format!("{n}{}", "a".repeat(3000)),
                    },
                ]
            })
            .collect();
        let messages = preview_context(history, "").unwrap();
        let summary = ContextSummary::from_messages(&messages, Some(40));
        assert!(summary.text_bytes <= MAX_CONTEXT_BYTES);
        assert!(summary.included_turns < 40);
        assert!(messages[messages.len() - 2].content.starts_with("39"));
        assert_eq!(
            summary.omitted_turns,
            Some(40 - summary.included_turns as u64)
        );
    }
}
