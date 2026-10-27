use super::{history::markdown_block, storage_error};
use dolores_core::{ExportFormat, TokenUsage, TurnMetadata};
use serde::Serialize;
use serde_json::Value;
use std::io::Write;

#[derive(Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct UsageSummary {
    model_attempts: usize,
    reported_calls: usize,
    missing_usage_calls: usize,
    partial_usage_calls: usize,
    known_usage: TokenUsage,
}
impl UsageSummary {
    fn record(&mut self, usage: Option<&TokenUsage>) {
        self.model_attempts += 1;
        let Some(usage) = usage else {
            self.missing_usage_calls += 1;
            return;
        };
        self.reported_calls += 1;
        if usage.input_tokens.is_none()
            || usage.output_tokens.is_none()
            || usage.total_tokens.is_none()
        {
            self.partial_usage_calls += 1;
        }
        for (sum, n) in [
            (&mut self.known_usage.input_tokens, usage.input_tokens),
            (&mut self.known_usage.output_tokens, usage.output_tokens),
            (&mut self.known_usage.total_tokens, usage.total_tokens),
            (
                &mut self.known_usage.cached_input_tokens,
                usage.cached_input_tokens,
            ),
            (
                &mut self.known_usage.reasoning_tokens,
                usage.reasoning_tokens,
            ),
        ] {
            if let Some(n) = n {
                *sum = Some(sum.unwrap_or(0).saturating_add(n));
            }
        }
    }
    pub(super) fn message(&mut self, metadata: &TurnMetadata) {
        if let Some(agent) = &metadata.agent {
            for usage in &agent.usage_by_call {
                self.record(usage.as_ref());
            }
            // The archive's covered attempts are counted from its own fragments.
            let expected = agent
                .model_calls
                .saturating_sub(agent.archive.as_ref().map_or(0, |a| a.model_calls));
            for _ in agent.usage_by_call.len()..expected {
                self.record(None);
            }
        } else {
            self.record(metadata.usage.as_ref());
        }
    }
}

#[derive(Default)]
struct Fragments {
    text: String,
    expected: usize,
}
impl Fragments {
    fn push(&mut self, data: &Value, usage: &mut UsageSummary) -> Result<(), String> {
        if data["encoding"] != "agentSummaryJsonFragment" {
            return Ok(());
        }
        let failure = || {
            "Archived usage fragments are incomplete or malformed. Retained run evidence was not changed.".to_string()
        };
        let offset = data["offset"].as_u64().ok_or_else(failure)? as usize;
        let total = data["totalBytes"].as_u64().ok_or_else(failure)? as usize;
        if offset == 0 {
            if self.expected != 0 {
                return Err(failure());
            }
            self.expected = total;
        }
        let text = data["text"].as_str().ok_or_else(failure)?;
        if total == 0
            || total > 8 * 1024 * 1024
            || total != self.expected
            || offset != self.text.len()
            || text.len() > total.saturating_sub(offset)
        {
            return Err(failure());
        }
        self.text.push_str(text);
        if self.text.len() == total {
            let summary: Value = serde_json::from_str(&self.text).map_err(|_| failure())?;
            let values: Vec<Option<TokenUsage>> =
                serde_json::from_value(summary.get("usageByCall").cloned().ok_or_else(failure)?)
                    .map_err(|_| failure())?;
            for value in &values {
                usage.record(value.as_ref());
            }
            self.text.clear();
            self.expected = 0;
        }
        Ok(())
    }
}

/// Stream one run/event at a time under the export's existing SQLite snapshot.
/// Only one bounded summary fragment group is reconstructed for usage counting.
pub(super) fn write(
    conn: &rusqlite::Connection,
    session: &str,
    format: ExportFormat,
    output: &mut dyn Write,
    usage: &mut UsageSummary,
) -> Result<(), String> {
    let io_error = |_| "Could not write the conversation export.".to_string();
    if matches!(format, ExportFormat::Json) {
        output.write_all(b",\"runs\":[").map_err(io_error)?;
    }
    let mut statement = conn
        .prepare("SELECT id,data FROM runs WHERE thread=?1 ORDER BY rowid")
        .map_err(storage_error)?;
    let mut rows = statement.query([session]).map_err(storage_error)?;
    let mut first = true;
    while let Some(row) = rows.next().map_err(storage_error)? {
        let id: String = row.get(0).map_err(storage_error)?;
        let raw: String = row.get(1).map_err(storage_error)?;
        let run: Value = serde_json::from_str(&raw).map_err(storage_error)?;
        match format {
            ExportFormat::Json => {
                if !first {
                    output.write_all(b",").map_err(io_error)?;
                }
                output.write_all(b"{\"snapshot\":").map_err(io_error)?;
                serde_json::to_writer(&mut *output, &run).map_err(storage_error)?;
                output.write_all(b",\"events\":[").map_err(io_error)?;
            }
            ExportFormat::Markdown => {
                writeln!(output, "\n## Run evidence (historical, untrusted)\n")
                    .map_err(io_error)?;
                markdown_block(output, &raw).map_err(io_error)?;
            }
        }
        let mut fragments = Fragments::default();
        let mut first_event = true;
        let mut events = conn
            .prepare("SELECT data FROM run_events WHERE run_id=?1 ORDER BY sequence")
            .map_err(storage_error)?;
        let mut records = events.query([&id]).map_err(storage_error)?;
        while let Some(record) = records.next().map_err(storage_error)? {
            let raw: String = record.get(0).map_err(storage_error)?;
            let event: Value = serde_json::from_str(&raw).map_err(storage_error)?;
            if event["kind"] == "agentEvidence" {
                fragments.push(&event["data"], usage)?;
            }
            match format {
                ExportFormat::Json => {
                    if !first_event {
                        output.write_all(b",").map_err(io_error)?;
                    }
                    serde_json::to_writer(&mut *output, &event).map_err(storage_error)?;
                }
                ExportFormat::Markdown => markdown_block(output, &raw).map_err(io_error)?,
            }
            first_event = false;
        }
        if fragments.expected != 0 {
            return Err("Archived usage is incomplete. Inspect retained Run history before exporting again.".into());
        }
        if matches!(format, ExportFormat::Json) {
            output.write_all(b"]}").map_err(io_error)?;
        }
        first = false;
    }
    let summary = serde_json::json!({"modelAttempts":usage.model_attempts,"reportedCalls":usage.reported_calls,"missingUsageCalls":usage.missing_usage_calls,"partialUsageCalls":usage.partial_usage_calls,"knownUsage":usage.known_usage,"coverage":"Saved conversational usage and archived agent summaries only; unsaved, child and auxiliary usage may be absent. Missing counters are unknown, not zero. Cached input/reasoning are subsets, not additional totals."});
    match format {
        ExportFormat::Json => {
            output.write_all(b"],\"usageSummary\":").map_err(io_error)?;
            serde_json::to_writer(output, &summary).map_err(storage_error)?;
        }
        ExportFormat::Markdown => {
            writeln!(output, "\n## Recorded usage (known subtotal)\n").map_err(io_error)?;
            markdown_block(output, &summary.to_string()).map_err(io_error)?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn partial_and_missing_counters_never_become_zero_totals() {
        let mut summary = UsageSummary::default();
        summary.record(Some(&TokenUsage {
            input_tokens: Some(0),
            ..Default::default()
        }));
        summary.record(None);
        assert_eq!(summary.model_attempts, 2);
        assert_eq!(summary.partial_usage_calls, 1);
        assert_eq!(summary.missing_usage_calls, 1);
        assert_eq!(summary.known_usage.input_tokens, Some(0));
        assert_eq!(summary.known_usage.total_tokens, None);
    }
    #[test]
    fn fragment_offsets_and_sizes_are_checked_before_accounting() {
        let mut summary = UsageSummary::default();
        for value in [
            serde_json::json!({"offset":1,"totalBytes":3,"text":"{}"}),
            serde_json::json!({"offset":0,"totalBytes":9_000_000,"text":"{}"}),
            serde_json::json!({"offset":0,"totalBytes":1,"text":"{}"}),
        ] {
            let mut value = value;
            value["encoding"] = serde_json::json!("agentSummaryJsonFragment");
            assert!(Fragments::default().push(&value, &mut summary).is_err());
        }
        assert_eq!(summary.model_attempts, 0);
    }
}
