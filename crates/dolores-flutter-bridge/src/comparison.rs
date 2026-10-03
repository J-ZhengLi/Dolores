use super::{blocking, Engine, Run};
use dolores_core::{
    ComparisonDraft, ComparisonOutcome, ComparisonResult, ComparisonStatus, ContextComparison,
    ContextVariant, Message, ModelProvider, RequestSettings, SessionStore,
};
use serde_json::{json, Value};
use std::{
    sync::{Arc, Mutex},
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};
use tokio::sync::mpsc;
use tokio_util::sync::CancellationToken;

fn catalog(store: &dyn SessionStore, session: &str) -> Result<Vec<ContextVariant>, String> {
    let root = store.workspace(session)?.root;
    let mut items = vec![];
    for memory in store.memory_preferences(root.as_deref())? {
        items.push(ContextVariant {
            source: Some(format!(
                "memory:{:?}:{}:{}",
                memory.scope, memory.id, memory.revision
            )),
            label: format!(
                "Memory · {} · r{}",
                memory.title.chars().take(40).collect::<String>(),
                memory.revision
            ),
            text: memory.text,
        });
    }
    for skill in super::skills::for_session(store, Some(session))? {
        for version in skill.versions {
            items.push(ContextVariant {
                source: Some(format!(
                    "skill:{:?}:{}:{}",
                    skill.scope, skill.name, version.version
                )),
                label: format!(
                    "{:?} skill · {} · v{}",
                    skill.scope, skill.name, version.version
                ),
                text: version.document.text,
            });
        }
    }
    Ok(items)
}

fn report(run: &ContextComparison) -> Value {
    let mut value = json!(run);
    value["summary"] = json!(run.summary());
    value
}

impl Engine {
    pub(super) fn comparison_sources(&self, session: &str) -> Result<Value, String> {
        self.store.messages_page(session, None, false, 1)?;
        let settings = self
            .store
            .effective_request_settings(&self.store.preferences()?)?;
        Ok(
            json!({"items":catalog(self.store.as_ref(),session)?, "model":self.store.preferences()?.model,
            "settings":RequestSettings{max_output_tokens: settings.max_output_tokens.min(2048), timeout_seconds:settings.timeout_seconds.min(60), ..settings}}),
        )
    }
    pub(super) fn comparison_report(
        &self,
        session: &str,
        comparison_id: i64,
    ) -> Result<Value, String> {
        Ok(report(&self.store.comparison(session, comparison_id)?))
    }
    pub(super) fn start_comparison(
        &self,
        active: &mut Option<Run>,
        id: u64,
        session: String,
        draft: ComparisonDraft,
        settings: RequestSettings,
    ) -> Result<Value, String> {
        draft.validate()?;
        settings.validate()?;
        if settings.max_output_tokens > 2048 || settings.timeout_seconds > 60 {
            return Err("Comparisons allow up to 2048 output tokens and 60 seconds per response. Change the comparison fields; chat settings remain unchanged.".into());
        }
        self.store.messages_page(&session, None, false, 1)?;
        let sources = catalog(self.store.as_ref(), &session)?;
        for variant in [&draft.baseline, &draft.candidate] {
            if variant.source.is_some() && !sources.contains(variant) {
                return Err("Instruction snapshot changed. Refresh sources or use a labeled manual copy before comparing.".into());
            }
        }
        let _entered = self.runtime.enter();
        let provider = self
            .connection
            .lock()
            .map_err(|_| "Model connection unavailable.")?
            .comparison_provider(settings)?;
        let (baseline_messages, candidate_messages) = draft.prompts()?;
        for messages in baseline_messages.iter().chain(&candidate_messages) {
            let prepared = dolores_core::prepare_token_context(
                messages.clone(),
                &[],
                provider.context_window_tokens(),
                settings,
            )?;
            if prepared.0 != *messages {
                return Err("Comparison input does not fit. Shorten the instructions/test or configure the model context window.".into());
            }
        }
        let run = ContextComparison {
            id: 0,
            session,
            revision: 0,
            created_at: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis() as i64,
            model: self.store.preferences()?.model,
            settings,
            context_window_tokens: provider.context_window_tokens(),
            draft,
            baseline_messages,
            candidate_messages,
            results: vec![],
            status: ComparisonStatus::Running,
        };
        let run = self.store.create_comparison(&run)?;
        let comparison_id = run.id;
        let (output, events) = mpsc::channel(8);
        let cancel = CancellationToken::new();
        *active = Some(Run {
            id,
            cancel: cancel.clone(),
            events,
            approvals: Arc::new(Mutex::new(None)),
        });
        let store = self.store.clone();
        self.runtime.spawn(async move {
            execute(store, provider, run, cancel, output, id).await;
        });
        Ok(json!({"comparisonId":comparison_id}))
    }
}

async fn response(
    provider: &dyn ModelProvider,
    messages: Vec<Message>,
    settings: RequestSettings,
    cancel: CancellationToken,
) -> ComparisonResult {
    let started = Instant::now();
    let request_cancel = cancel.child_token();
    let (send, mut receive) = mpsc::channel(8);
    let request = provider.stream_chat_outcome(messages, send, request_cancel.clone());
    tokio::pin!(request);
    let deadline = tokio::time::sleep(Duration::from_secs(settings.timeout_seconds as u64));
    tokio::pin!(deadline);
    let mut result = ComparisonResult {
        output: String::new(),
        usage: None,
        elapsed_ms: 0,
        outcome: ComparisonOutcome::Completed,
        detail: None,
    };
    let mut finished = false;
    let mut closed = false;
    loop {
        tokio::select! { biased;
            _ = cancel.cancelled() => { result.outcome=ComparisonOutcome::Stopped; result.detail=Some("Stopped. Earlier completed responses remain; start a new comparison to test again.".into()); break; }
            _ = &mut deadline => { result.outcome=ComparisonOutcome::Failed; result.detail=Some(format!("Response timed out at {} seconds. Earlier evidence remains; change comparison timeout and start a new run.",settings.timeout_seconds)); break; }
            value = &mut request, if !finished => {
                finished=true;
                match value {
                    Ok(value) => { result.usage=value.usage; if value.output_limit { result.outcome=ComparisonOutcome::OutputLimit; result.detail=Some(format!("Output limit: {} tokens. Partial response is not a passing test. Adjust comparison output tokens and start a new run.",settings.max_output_tokens)); } }
                    Err(_) => { result.outcome=ComparisonOutcome::Failed; result.detail=Some("Model response failed. Check Model connection / Request settings and start a new comparison. Earlier completed evidence remains.".into()); }
                }
            }
            delta = receive.recv(), if !closed => match delta {
                Some(delta) => {
                    if result.output.len()+delta.len()>8192 { result.outcome=ComparisonOutcome::Failed; result.detail=Some("Response exceeded the 8 KiB comparison limit. Use shorter tests; previous evidence remains.".into()); break; }
                    result.output.push_str(&delta);
                }
                None if finished => break,
                None => { closed = true; }
            }
        }
        if closed && finished {
            break;
        }
    }
    request_cancel.cancel();
    if result.outcome == ComparisonOutcome::Completed
        && result.usage.as_ref().is_some_and(|u| {
            u.output_tokens
                .is_some_and(|n| n > settings.max_output_tokens as u64)
                || u.reasoning_tokens
                    .is_some_and(|n| n > settings.max_output_tokens as u64)
        })
    {
        result.outcome = ComparisonOutcome::Failed;
        result.detail = Some("Provider-reported output exceeded the frozen allowance. This response cannot pass; review the provider/profile before another comparison.".into());
    }
    if result.outcome == ComparisonOutcome::Completed && result.output.trim().is_empty() {
        result.outcome = ComparisonOutcome::Failed;
        result.detail=Some("The model returned no text. Review the connection/settings and start a new comparison.".into());
    }
    result.elapsed_ms = started.elapsed().as_millis() as u64;
    result
}

async fn execute(
    store: Arc<dyn SessionStore>,
    provider: Arc<dyn ModelProvider>,
    mut run: ContextComparison,
    cancel: CancellationToken,
    output: mpsc::Sender<Value>,
    id: u64,
) {
    let total = run.draft.trials.len() * 2;
    let mut persistence_error = None;
    for index in 0..total {
        let _=output.try_send(json!({"type":"comparisonProgress","id":id,"test":index/2+1,"phase":if index%2==0 {"baseline"} else {"candidate"}}));
        let messages = if index % 2 == 0 {
            run.baseline_messages[index / 2].clone()
        } else {
            run.candidate_messages[index / 2].clone()
        };
        let result = response(provider.as_ref(), messages, run.settings, cancel.clone()).await;
        run.status = match result.outcome {
            ComparisonOutcome::Stopped => ComparisonStatus::Stopped,
            ComparisonOutcome::Failed | ComparisonOutcome::OutputLimit => ComparisonStatus::Failed,
            ComparisonOutcome::Completed if index + 1 == total => ComparisonStatus::Completed,
            _ => ComparisonStatus::Running,
        };
        run.results.push(result);
        if cancel.is_cancelled() {
            run.status = ComparisonStatus::Stopped;
        }
        let writer = store.clone();
        let candidate = run.clone();
        match blocking(move || writer.update_comparison(&candidate)).await {
            Ok(saved) => run = saved,
            Err(_) => {
                persistence_error=Some("Comparison result could not be saved. Earlier saved evidence remains. Copy the visible receipt before closing; do not treat this run as complete.".to_string());
                break;
            }
        }
        if run.status != ComparisonStatus::Running {
            break;
        }
    }
    let _=output.send(json!({"type":"done","id":id,"comparison":report(&run),"persisted":persistence_error.is_none(),"error":persistence_error})).await;
}

#[cfg(test)]
mod tests {
    use super::*;
    use dolores_core::{PluginDescriptor, StreamOutcome, TokenUsage};
    struct Fixture {
        candidate_limit: bool,
        hang: bool,
        close_first: bool,
        oversized: bool,
        excessive_usage: bool,
    }
    #[async_trait::async_trait]
    impl ModelProvider for Fixture {
        fn descriptor(&self) -> PluginDescriptor {
            PluginDescriptor {
                id: "comparison.fixture",
                kind: "model",
                api_version: 1,
            }
        }
        async fn stream(
            &self,
            _: Vec<Message>,
            _: mpsc::Sender<String>,
            _: CancellationToken,
        ) -> Result<(), String> {
            unreachable!()
        }
        async fn stream_chat_outcome(
            &self,
            messages: Vec<Message>,
            output: mpsc::Sender<String>,
            _: CancellationToken,
        ) -> Result<StreamOutcome, String> {
            if self.hang {
                if self.close_first {
                    drop(output);
                }
                std::future::pending::<()>().await;
                unreachable!();
            }
            let candidate = messages
                .first()
                .is_some_and(|m| m.content.contains("Include PASS"));
            output
                .send(if self.oversized {
                    "x".repeat(8193)
                } else if candidate {
                    "PASS".into()
                } else {
                    "ordinary".into()
                })
                .await
                .unwrap();
            Ok(StreamOutcome {
                usage: if self.excessive_usage {
                    Some(TokenUsage {
                        output_tokens: Some(4096),
                        ..Default::default()
                    })
                } else {
                    None
                },
                output_limit: candidate && self.candidate_limit,
            })
        }
    }
    fn fixture() -> Fixture {
        Fixture {
            candidate_limit: false,
            hang: false,
            close_first: false,
            oversized: false,
            excessive_usage: false,
        }
    }
    #[tokio::test]
    async fn truncation_preserves_completed_baseline_and_never_counts_partial_match_as_pass() {
        let store = Arc::new(
            dolores_store_sqlite::SqliteStore::open(std::path::Path::new(":memory:")).unwrap(),
        );
        store.create("task").unwrap();
        let draft: ComparisonDraft=serde_json::from_value(json!({"title":"test","baseline":{"label":"Before","text":"","source":null},"candidate":{"label":"After","text":"Include PASS","source":null},"trials":[{"prompt":"brief","required":["PASS"],"forbidden":[]}]})).unwrap();
        let (baseline_messages, candidate_messages) = draft.prompts().unwrap();
        let run = store
            .create_comparison(&ContextComparison {
                id: 0,
                session: "task".into(),
                revision: 0,
                created_at: 0,
                model: "fixture".into(),
                settings: RequestSettings {
                    max_output_tokens: 128,
                    timeout_seconds: 1,
                    ..Default::default()
                },
                context_window_tokens: None,
                draft,
                baseline_messages,
                candidate_messages,
                results: vec![],
                status: ComparisonStatus::Running,
            })
            .unwrap();
        let id = run.id;
        let (send, mut events) = mpsc::channel(8);
        execute(
            store.clone(),
            Arc::new(Fixture {
                candidate_limit: true,
                ..fixture()
            }),
            run,
            CancellationToken::new(),
            send,
            1,
        )
        .await;
        let saved = store.comparison("task", id).unwrap();
        assert_eq!(saved.results.len(), 2);
        assert_eq!(saved.results[0].output, "ordinary");
        assert_eq!(saved.results[1].outcome, ComparisonOutcome::OutputLimit);
        assert_eq!(saved.summary().candidate_passed, 0);
        assert!(!saved.summary().improved);
        let mut done = None;
        while let Some(event) = events.recv().await {
            if event["type"] == "done" {
                done = Some(event);
            }
        }
        assert_eq!(done.unwrap()["persisted"], true);
        assert!(store.messages("task").unwrap().is_empty());
    }
    #[tokio::test]
    async fn deadline_cancellation_closed_stream_and_provider_budget_mismatch_are_bounded() {
        let settings = RequestSettings {
            max_output_tokens: 128,
            timeout_seconds: 1,
            ..Default::default()
        };
        let timeout = response(
            &Fixture {
                hang: true,
                close_first: true,
                ..fixture()
            },
            vec![],
            settings,
            CancellationToken::new(),
        )
        .await;
        assert_eq!(timeout.outcome, ComparisonOutcome::Failed);
        assert!(timeout.detail.unwrap().contains("1 seconds"));
        let cancel = CancellationToken::new();
        cancel.cancel();
        assert_eq!(
            response(
                &Fixture {
                    hang: true,
                    ..fixture()
                },
                vec![],
                settings,
                cancel
            )
            .await
            .outcome,
            ComparisonOutcome::Stopped
        );
        assert_eq!(
            response(
                &Fixture {
                    oversized: true,
                    ..fixture()
                },
                vec![],
                settings,
                CancellationToken::new()
            )
            .await
            .outcome,
            ComparisonOutcome::Failed
        );
        assert_eq!(
            response(
                &Fixture {
                    excessive_usage: true,
                    ..fixture()
                },
                vec![],
                settings,
                CancellationToken::new()
            )
            .await
            .outcome,
            ComparisonOutcome::Failed
        );
    }
}
