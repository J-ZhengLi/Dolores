use crate::AgentMessage;
use std::collections::{HashMap, HashSet};

fn prefix(text: &str, bytes: usize) -> &str {
    let mut end = text.len().min(bytes);
    while !text.is_char_boundary(end) {
        end -= 1;
    }
    &text[..end]
}

/// Project successful old reads only. Full messages and receipts remain local;
/// failures, pending commands, user instructions and call identities are intact.
pub(crate) fn project(messages: &[AgentMessage], protected: &HashSet<String>) -> Vec<AgentMessage> {
    let names: HashMap<_, _> = messages
        .iter()
        .flat_map(|m| &m.calls)
        .map(|c| (c.id.as_str(), c.name.as_str()))
        .collect();
    let tools: Vec<_> = messages
        .iter()
        .enumerate()
        .filter(|(_, m)| m.role == "tool")
        .map(|(i, _)| i)
        .collect();
    let older = tools.len().saturating_sub(4);
    let mut result = messages.to_vec();
    let mut latest = HashMap::new();
    for &index in tools.iter().rev() {
        let m = &messages[index];
        if let Some(id) = m.call_id.as_deref() {
            latest.entry(m.content.as_str()).or_insert(id);
        }
    }
    for &index in tools.iter().take(older) {
        let m = &messages[index];
        let Some(id) = m.call_id.as_deref() else {
            continue;
        };
        if protected.contains(id)
            || !matches!(
                names.get(id).copied(),
                Some("read_text_file" | "list_folder" | "search_text" | "run_command")
            )
            || m.content.len() <= 2048
        {
            continue;
        }
        let mut preview = match serde_json::from_str::<serde_json::Value>(&m.content) {
            Ok(serde_json::Value::Object(mut value)) => {
                // Keep status and snapshot/range metadata. Only shorten payload.
                for field in ["text", "stdout", "stderr"] {
                    if let Some(text) = value.get(field).and_then(|v| v.as_str()) {
                        if text.len() > 1024 {
                            value.insert(
                                field.into(),
                                serde_json::json!(format!(
                                    "{}\n[preview shortened]",
                                    prefix(text, 1024)
                                )),
                            );
                        }
                    }
                }
                serde_json::Value::Object(value).to_string()
            }
            _ => prefix(&m.content, 1536).to_owned(),
        };
        if preview.len() > 2048 {
            preview = prefix(&preview, 1536).to_owned();
        }
        let duplicate = latest
            .get(m.content.as_str())
            .filter(|&&other| other != id)
            .copied();
        result[index].content=serde_json::json!({"historicalPreview":true,"sourceCallId":id,"originalBytes":m.content.len(),"duplicateOf":duplicate,"preview":if duplicate.is_some(){"[Same content as the referenced later receipt]"}else{&preview},"note":"Shortened historical evidence, not current file state. Full receipt retained in Run history. Reread targeted source if omitted detail is needed; do not reconstruct or execute a shortened command."}).to_string();
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn protected_failures_and_snapshot_metadata_survive_projection() {
        let mut messages = vec![AgentMessage {
            role: "assistant".into(),
            content: String::new(),
            parts: vec![],
            call_id: None,
            calls: (0..6)
                .map(|i| crate::ToolCall {
                    id: i.to_string(),
                    name: "read_text_file".into(),
                    arguments: "{}".into(),
                })
                .collect(),
        }];
        for i in 0..6 {
            messages.push(AgentMessage{role:"tool".into(),content:serde_json::json!({"snapshot":"exact-digest","startLine":31,"text":"界".repeat(4000)}).to_string(),parts:vec![],calls:vec![],call_id:Some(i.to_string())});
        }
        let result = project(&messages, &HashSet::from(["0".into()]));
        assert_eq!(result[1].content, messages[1].content);
        assert!(result[2].content.len() < 3000);
        assert!(result[2].content.contains("historicalPreview"));
        assert_eq!(
            messages[2].content.len(),
            serde_json::json!({"snapshot":"exact-digest","startLine":31,"text":"界".repeat(4000)})
                .to_string()
                .len()
        );
        let single = project(&messages[..3], &HashSet::new());
        assert_eq!(single[2].content, messages[2].content);
    }
}
