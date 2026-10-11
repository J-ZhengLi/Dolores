/// Normalize only the stray closing envelope emitted with explicit thinking off.
/// Keep raw protocol bounds and leave tool arguments untouched.
#[derive(Default)]
pub(crate) struct ContentPrefix {
    enabled: bool,
    resolved: bool,
    pending: String,
}

impl ContentPrefix {
    pub(crate) fn new(reasoning: dolores_core::ReasoningControl) -> Self {
        Self {
            enabled: reasoning == dolores_core::ReasoningControl::DeepseekThinkingOff,
            ..Default::default()
        }
    }

    pub(crate) fn push(&mut self, text: &str) -> String {
        if !self.enabled || self.resolved {
            return text.to_owned();
        }
        self.pending.push_str(text);
        if "</think>".starts_with(&self.pending) {
            return String::new();
        }
        self.resolved = true;
        let value = std::mem::take(&mut self.pending);
        value.strip_prefix("</think>").unwrap_or(&value).to_owned()
    }

    pub(crate) fn finish(&mut self) -> String {
        self.resolved = true;
        std::mem::take(&mut self.pending)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use dolores_core::ReasoningControl::*;

    #[test]
    fn every_fragment_boundary_preserves_unicode_and_embedded_markers() {
        let raw = "</think>中文 `</think>`";
        for split in raw
            .char_indices()
            .map(|(index, _)| index)
            .chain([raw.len()])
        {
            let mut prefix = ContentPrefix::new(DeepseekThinkingOff);
            let actual =
                prefix.push(&raw[..split]) + &prefix.push(&raw[split..]) + &prefix.finish();
            assert_eq!(actual, "中文 `</think>`");
        }
    }

    #[test]
    fn standalone_partial_wrong_or_later_markers_are_literal() {
        for raw in [
            "",
            "<",
            "</thi",
            "</think>",
            "<think>text",
            " </think>text",
            "text </think>",
        ] {
            for reasoning in [DeepseekThinkingOff, ProviderDefault] {
                let mut prefix = ContentPrefix::new(reasoning);
                let actual = prefix.push(raw) + &prefix.finish();
                assert_eq!(actual, raw);
            }
        }
        let mut prefix = ContentPrefix::new(ProviderDefault);
        assert_eq!(
            prefix.push("</think>text") + &prefix.finish(),
            "</think>text"
        );
    }
}
