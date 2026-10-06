use crate::{AgentMessage, Message, RequestSettings, Role, ToolSpec};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

pub const DEFAULT_CONTEXT_WINDOW_TOKENS: u32 = 128 * 1024;
/// Endpoint-scoped overrides. Missing/None uses the host's 128K default.
pub type ModelContexts = BTreeMap<String, Option<u32>>;
pub fn validate_model_contexts(contexts: &ModelContexts, models: &[String]) -> Result<(), String> {
    if contexts.len() > 32 || contexts.keys().any(|id| !models.contains(id)) {
        return Err("Context windows must belong to enabled models.".into());
    }
    if contexts
        .values()
        .flatten()
        .any(|n| !(1024..=16_777_216).contains(n))
    {
        return Err(
            "Context window must be a whole number between 1024 and 16777216 tokens, or blank."
                .into(),
        );
    }
    Ok(())
}

/// Deliberately approximate; never mixed into provider-reported usage.
pub fn estimate_text_tokens(text: &str) -> u64 {
    text.len().div_ceil(4) as u64
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct TokenContext {
    #[serde(default)]
    pub image_tokens: u64,
    pub estimator: String,
    pub input_tokens: u64,
    pub system_tokens: u64,
    pub history_tokens: u64,
    pub draft_tokens: u64,
    pub tool_tokens: u64,
    pub framing_tokens: u64,
    pub context_window_tokens: Option<u32>,
    pub max_input_tokens: Option<u64>,
    pub reserved_output_tokens: u32,
    pub headroom_tokens: u64,
}

pub fn input_token_allowance(
    window: Option<u32>,
    settings: RequestSettings,
) -> Result<Option<u64>, String> {
    settings.validate()?;
    window.map(|window| {
        let usable = u64::from(window) * 95 / 100;
        usable.checked_sub(u64::from(output_reserve(window, settings))).filter(|n| *n > 0)
            .ok_or_else(|| "Context window leaves no input room. Increase the model context window or reduce the output token limit.".into())
    }).transpose()
}

/// Planning headroom only, never a provider generation limit. With an unknown
/// provider allowance keep a quarter of the window, up to 32K, for output.
fn output_reserve(window: u32, settings: RequestSettings) -> u32 {
    settings
        .max_output_tokens
        .unwrap_or((window / 4).min(32768))
}

fn image_allowance(parts: &[crate::AttachmentRef]) -> u64 {
    parts.iter().filter(|p| p.is_image()).count() as u64 * 4096
}
fn tool_tokens(tools: &[ToolSpec]) -> Result<u64, String> {
    if tools.is_empty() {
        return Ok(0);
    }
    let specs: Vec<_> = tools.iter().map(|s| serde_json::json!({"type":"function","function":{"name":s.name,"description":s.description,"parameters":s.parameters}})).collect();
    Ok(estimate_text_tokens(
        &serde_json::to_string(&specs).map_err(|_| "Could not count tool definitions.")?,
    ))
}

pub fn estimate_agent_tokens(messages: &[AgentMessage], tools: &[ToolSpec]) -> Result<u64, String> {
    let mut tokens = tool_tokens(tools)? + 3;
    for message in messages {
        tokens += estimate_text_tokens(&message.content) + 4 + image_allowance(&message.parts);
        if !message.calls.is_empty() {
            let calls: Vec<_> = message.calls.iter().map(|c| serde_json::json!({"id":c.id,"type":"function","function":{"name":c.name,"arguments":c.arguments}})).collect();
            tokens += estimate_text_tokens(
                &serde_json::to_string(&calls).map_err(|_| "Could not count tool calls.")?,
            );
        }
        if let Some(id) = &message.call_id {
            tokens += estimate_text_tokens(id);
        }
    }
    Ok(tokens)
}

/// Shared by preview and send after byte preparation and optional tool guidance.
pub fn prepare_token_context(
    mut messages: Vec<Message>,
    tools: &[ToolSpec],
    window: Option<u32>,
    settings: RequestSettings,
) -> Result<(Vec<Message>, TokenContext), String> {
    if messages.len() < 2 || !messages.len().is_multiple_of(2) || messages[0].role != Role::System {
        return Err("Context needs system instructions and complete turns.".into());
    }
    let max_input = input_token_allowance(window, settings)?;
    let tool_tokens = tool_tokens(tools)?;
    loop {
        let system_tokens = estimate_text_tokens(&messages[0].content);
        let draft_tokens = estimate_text_tokens(&messages.last().unwrap().content);
        let history_tokens = messages[1..messages.len() - 1]
            .iter()
            .map(|m| estimate_text_tokens(&m.content))
            .sum();
        let framing_tokens = messages.len() as u64 * 4 + 3;
        let image_tokens = messages
            .iter()
            .map(|m| image_allowance(&m.parts))
            .sum::<u64>();
        let input_tokens = system_tokens
            + draft_tokens
            + history_tokens
            + tool_tokens
            + framing_tokens
            + image_tokens;
        if max_input.is_none_or(|limit| input_tokens <= limit) {
            return Ok((
                messages,
                TokenContext {
                    image_tokens,
                    estimator: "utf8-div4-v1".into(),
                    input_tokens,
                    system_tokens,
                    history_tokens,
                    draft_tokens,
                    tool_tokens,
                    framing_tokens,
                    context_window_tokens: window,
                    max_input_tokens: max_input,
                    reserved_output_tokens: window.map_or(0, |w| output_reserve(w, settings)),
                    headroom_tokens: window.map_or(0, |w| u64::from(w) - u64::from(w) * 95 / 100),
                },
            ));
        }
        if messages.len() == 2 {
            return Err("Message, instructions and tools exceed the model context budget. Shorten your message, increase the context window, or reduce the output token limit.".into());
        }
        messages.drain(1..3);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn estimates_unicode_rounding_schema_and_protocol_without_inventing_usage() {
        assert_eq!(estimate_text_tokens(""), 0);
        assert_eq!(estimate_text_tokens("hello"), 2);
        assert_eq!(estimate_text_tokens("你好🌍"), 3);
        let messages = crate::preview_context(vec![], "你好🌍").unwrap();
        let tools = [ToolSpec {
            name: "read".into(),
            description: "Read text".into(),
            parameters: serde_json::json!({"type":"object"}),
        }];
        let (_, count) =
            prepare_token_context(messages.clone(), &tools, None, RequestSettings::default())
                .unwrap();
        assert_eq!(count.context_window_tokens, None);
        assert_eq!(count.max_input_tokens, None);
        assert_eq!(
            count.input_tokens,
            count.system_tokens
                + count.draft_tokens
                + count.history_tokens
                + count.tool_tokens
                + count.framing_tokens
        );
        let mut agent: Vec<_> = messages
            .iter()
            .map(|m| AgentMessage {
                parts: vec![],
                role: format!("{:?}", m.role),
                content: m.content.clone(),
                calls: vec![],
                call_id: None,
            })
            .collect();
        assert_eq!(
            estimate_agent_tokens(&agent, &tools).unwrap(),
            count.input_tokens
        );
        agent[1].calls.push(crate::ToolCall {
            id: "call_1".into(),
            name: "read".into(),
            arguments: "{}".into(),
        });
        agent[1].call_id = Some("call_1".into());
        assert!(estimate_agent_tokens(&agent, &tools).unwrap() > count.input_tokens);
    }
    #[test]
    fn budget_keeps_newest_pairs_and_refuses_oversized_fixed_input() {
        let history = (0..8)
            .flat_map(|n| {
                [
                    Message {
                        parts: vec![],
                        role: Role::User,
                        content: format!("{n}{}", "x".repeat(400)),
                    },
                    Message {
                        parts: vec![],
                        role: Role::Assistant,
                        content: "a".repeat(400),
                    },
                ]
            })
            .collect();
        let settings = RequestSettings {
            max_output_tokens: Some(128),
            ..RequestSettings::default()
        };
        let messages = crate::preview_context(history, "draft").unwrap();
        let (prepared, count) = prepare_token_context(messages, &[], Some(1024), settings).unwrap();
        assert!(prepared.len() < 18);
        assert!(prepared[prepared.len() - 2].content.starts_with('a'));
        assert!(prepared[prepared.len() - 3].content.starts_with('7'));
        assert_eq!(count.max_input_tokens, Some(844));
        assert!(count.input_tokens <= 844);
        assert_eq!(prepared.last().unwrap().content, "draft");
        assert!(prepare_token_context(
            crate::preview_context(vec![], &"x".repeat(4000)).unwrap(),
            &[],
            Some(1024),
            settings
        )
        .is_err());
        assert!(input_token_allowance(
            Some(1024),
            RequestSettings {
                max_output_tokens: Some(2048),
                ..Default::default()
            }
        )
        .is_err());
        assert!(
            input_token_allowance(Some(1024), RequestSettings::default())
                .unwrap()
                .unwrap()
                > 0
        );
        assert!(
            validate_model_contexts(&BTreeMap::from([("a".into(), Some(0))]), &["a".into()])
                .is_err()
        );
        assert!(
            validate_model_contexts(&BTreeMap::from([("b".into(), None)]), &["a".into()]).is_err()
        );
    }
}
