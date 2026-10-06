use crate::{Message, RequestSettings, SkillTrial, TokenUsage};
use serde::{Deserialize, Serialize};
pub type ComparisonPrompts = (Vec<Vec<Message>>, Vec<Vec<Message>>);

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ContextVariant {
    pub label: String,
    pub text: String,
    pub source: Option<String>,
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ComparisonDraft {
    pub title: String,
    pub baseline: ContextVariant,
    pub candidate: ContextVariant,
    pub trials: Vec<SkillTrial>,
}
impl ComparisonDraft {
    pub fn validate(&self) -> Result<(), String> {
        crate::validate_skill_trials(&self.trials)?;
        if self.title.trim().is_empty()
            || self.title.len() > 200
            || self.title.chars().any(char::is_control)
        {
            return Err("Give the comparison a title within 200 bytes.".into());
        }
        for variant in [&self.baseline, &self.candidate] {
            if variant.label.trim().is_empty()
                || variant.label.len() > 200
                || variant.label.chars().any(char::is_control)
                || variant.text.len() > 8192
                || variant.text.contains('\0')
                || crate::credential_like(&variant.text)
                || variant.source.as_ref().is_some_and(|s| {
                    s.is_empty() || s.len() > 200 || s.chars().any(char::is_control)
                })
            {
                return Err("Each instruction snapshot needs a label and at most 8 KiB of text without credentials.".into());
            }
        }
        Ok(())
    }
    pub fn prompts(&self) -> Result<ComparisonPrompts, String> {
        self.validate()?;
        let build = |variant: &ContextVariant| -> Result<Vec<Vec<Message>>, String> {
            self.trials
                .iter()
                .map(|trial| {
                    let mut messages = crate::prepare_context(vec![], &trial.prompt)?;
                    if !variant.text.is_empty() {
                        messages[0].content.push_str(
                            "\n\nInstructions under evaluation (text only; no tool access):\n",
                        );
                        messages[0].content.push_str(&variant.text);
                    }
                    Ok(messages)
                })
                .collect()
        };
        Ok((build(&self.baseline)?, build(&self.candidate)?))
    }
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum ComparisonStatus {
    Running,
    Completed,
    Failed,
    Stopped,
}
#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum ComparisonOutcome {
    Completed,
    OutputLimit,
    Failed,
    Stopped,
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ComparisonResult {
    pub output: String,
    pub usage: Option<TokenUsage>,
    pub elapsed_ms: u64,
    pub outcome: ComparisonOutcome,
    pub detail: Option<String>,
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ContextComparison {
    pub id: i64,
    pub session: String,
    pub revision: u32,
    pub created_at: i64,
    pub model: String,
    pub settings: RequestSettings,
    pub context_window_tokens: Option<u32>,
    pub draft: ComparisonDraft,
    pub baseline_messages: Vec<Vec<Message>>,
    pub candidate_messages: Vec<Vec<Message>>,
    /// Baseline, candidate, baseline, candidate; only completed calls can pass.
    pub results: Vec<ComparisonResult>,
    pub status: ComparisonStatus,
}
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ComparisonSummary {
    pub id: i64,
    pub revision: u32,
    pub title: String,
    pub model: String,
    pub created_at: i64,
    pub status: ComparisonStatus,
    pub baseline_passed: usize,
    pub candidate_passed: usize,
    pub tests: usize,
    pub improved: bool,
}
impl ContextComparison {
    pub fn summary(&self) -> ComparisonSummary {
        let pass = |index: usize, result: &ComparisonResult| {
            result.outcome == ComparisonOutcome::Completed
                && crate::skill_trial_pass(&self.draft.trials[index / 2], &result.output)
        };
        let baseline = self
            .results
            .iter()
            .enumerate()
            .filter(|(i, r)| i % 2 == 0 && pass(*i, r))
            .count();
        let candidate = self
            .results
            .iter()
            .enumerate()
            .filter(|(i, r)| i % 2 == 1 && pass(*i, r))
            .count();
        ComparisonSummary {
            id: self.id,
            revision: self.revision,
            title: self.draft.title.clone(),
            model: self.model.clone(),
            created_at: self.created_at,
            status: self.status,
            baseline_passed: baseline,
            candidate_passed: candidate,
            tests: self.draft.trials.len(),
            improved: self.status == ComparisonStatus::Completed
                && self.results.len() == self.draft.trials.len() * 2
                && candidate == self.draft.trials.len()
                && candidate > baseline,
        }
    }
    pub fn validate(&self) -> Result<(), String> {
        self.draft.validate()?;
        self.settings.validate()?;
        let prompts = self.draft.prompts()?;
        if self.settings.max_output_tokens.is_none_or(|n| n > 2048)
            || self.settings.timeout_seconds > 60
            || self.model.is_empty()
            || self.model.len() > 200
            || self.model.chars().any(char::is_control)
            || self.baseline_messages != prompts.0
            || self.candidate_messages != prompts.1
            || self.results.len() > self.draft.trials.len() * 2
            || self.results.iter().enumerate().any(|(index, r)| {
                r.output.len() > 8192
                    || r.output.contains('\0')
                    || r.detail.as_ref().is_some_and(|d| d.len() > 512)
                    || (r.outcome != ComparisonOutcome::Completed
                        && index + 1 != self.results.len())
            })
            || (self.status == ComparisonStatus::Completed
                && (self.results.len() != self.draft.trials.len() * 2
                    || self
                        .results
                        .iter()
                        .any(|r| r.outcome != ComparisonOutcome::Completed)))
            || (self.status == ComparisonStatus::Running
                && self
                    .results
                    .iter()
                    .any(|r| r.outcome != ComparisonOutcome::Completed))
            || (self.status == ComparisonStatus::Failed
                && self.results.last().is_none_or(|r| {
                    matches!(
                        r.outcome,
                        ComparisonOutcome::Completed | ComparisonOutcome::Stopped
                    )
                }))
        {
            return Err("Comparison evidence is invalid. Create a fresh comparison.".into());
        }
        Ok(())
    }
    pub fn same_request(&self, other: &Self) -> bool {
        self.id == other.id
            && self.session == other.session
            && self.created_at == other.created_at
            && self.model == other.model
            && self.settings == other.settings
            && self.context_window_tokens == other.context_window_tokens
            && self.draft == other.draft
            && self.baseline_messages == other.baseline_messages
            && self.candidate_messages == other.candidate_messages
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    pub fn sample() -> ContextComparison {
        let draft = ComparisonDraft {
            title: "Memory revision comparison".into(),
            baseline: ContextVariant {
                label: "Before".into(),
                text: String::new(),
                source: None,
            },
            candidate: ContextVariant {
                label: "After".into(),
                text: "Include PASS".into(),
                source: None,
            },
            trials: vec![SkillTrial {
                prompt: "Respond briefly".into(),
                required: vec!["PASS".into()],
                forbidden: vec!["FAIL".into()],
            }],
        };
        let (baseline_messages, candidate_messages) = draft.prompts().unwrap();
        ContextComparison {
            id: 0,
            session: "task".into(),
            revision: 0,
            created_at: 42,
            model: "fixture".into(),
            settings: RequestSettings {
                max_output_tokens: Some(512),
                timeout_seconds: 10,
                reasoning: Default::default(),
            },
            context_window_tokens: None,
            draft,
            baseline_messages,
            candidate_messages,
            results: vec![],
            status: ComparisonStatus::Running,
        }
    }
    fn result(text: &str, outcome: ComparisonOutcome) -> ComparisonResult {
        ComparisonResult {
            output: text.into(),
            usage: None,
            elapsed_ms: 10,
            outcome,
            detail: None,
        }
    }
    #[test]
    fn improvement_requires_complete_all_pass_strict_gain_and_no_truncation() {
        let mut run = sample();
        run.results = vec![
            result("ordinary", ComparisonOutcome::Completed),
            result("PASS", ComparisonOutcome::Completed),
        ];
        assert!(!run.summary().improved);
        run.status = ComparisonStatus::Completed;
        run.validate().unwrap();
        assert!(run.summary().improved);
        run.results[0].output = "PASS".into();
        assert!(!run.summary().improved);
        run.results[0].output = "ordinary".into();
        run.results[1].outcome = ComparisonOutcome::OutputLimit;
        assert!(run.validate().is_err());
        run.status = ComparisonStatus::Failed;
        run.validate().unwrap();
        assert_eq!(run.summary().candidate_passed, 0);
        assert!(!run.summary().improved);
    }
    #[test]
    fn frozen_prompts_budget_invalid_credentials_and_incomplete_receipts_are_refused() {
        let mut run = sample();
        run.candidate_messages[0][0].content.push_str("tampered");
        assert!(run.validate().is_err());
        run = sample();
        run.settings.max_output_tokens = Some(2049);
        assert!(run.validate().is_err());
        run = sample();
        run.draft.candidate.text = "sk-12345678901234567890".into();
        assert!(run.validate().is_err());
        run = sample();
        run.status = ComparisonStatus::Completed;
        assert!(run.validate().is_err());
        run = sample();
        run.results = vec![result(&"x".repeat(8193), ComparisonOutcome::Completed)];
        assert!(run.validate().is_err());
    }
}
