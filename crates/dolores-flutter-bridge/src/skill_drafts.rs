use super::{Engine, Run};
use dolores_core::{
    ProjectSkill, Role, SessionStore, SkillDraft, SkillEvaluation, SkillExample, SkillPromotion,
    SkillScope, SkillTrial, SkillTrialResult,
};
use serde_json::{json, Value};
use std::{
    sync::{Arc, Mutex},
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};
use tokio::sync::mpsc;
use tokio_util::sync::CancellationToken;

const STALE: &str = "Skill draft review expired or changed. Review the exchanges again.";
#[derive(Clone)]
pub(super) struct DraftReview {
    token: String,
    session: String,
    scope: SkillScope,
    folder: Option<String>,
    examples: Vec<SkillExample>,
    draft: Option<SkillDraft>,
    model: Option<String>,
    promotion: Option<SkillPromotion>,
    created: Instant,
}

fn fresh(store: &dyn SessionStore, review: &DraftReview) -> Result<(), String> {
    if store.workspace(&review.session)?.root != review.folder {
        return Err(STALE.into());
    }
    for source in &review.examples {
        let before = source.message_id.checked_add(1).ok_or(STALE)?;
        let page = store.messages_page(&review.session, Some(before), false, 2)?;
        if page.items.len() != 2
            || page.items[0].id != source.user_id
            || page.items[0].role != Role::User
            || page.items[0].content != source.request
            || page.items[1].id != source.message_id
            || page.items[1].role != Role::Assistant
            || page.items[1].content != source.response
        {
            return Err(STALE.into());
        }
    }
    Ok(())
}
fn check(review: &DraftReview, session: &str, token: &str) -> Result<(), String> {
    if review.token != token
        || review.session != session
        || review.created.elapsed() > Duration::from_secs(300)
    {
        return Err(STALE.into());
    }
    Ok(())
}
fn publish(
    slot: &Mutex<Option<DraftReview>>,
    cancel: &CancellationToken,
    expected: &str,
    review: DraftReview,
) -> Result<(), String> {
    let mut guard = slot.lock().map_err(|_| STALE)?;
    if cancel.is_cancelled() || guard.as_ref().is_none_or(|r| r.token != expected) {
        return Err("Skill draft stopped. Nothing was saved.".into());
    }
    *guard = Some(review);
    Ok(())
}
fn candidate_skills(
    saved: &[ProjectSkill],
    draft: &dolores_core::SkillDocument,
    scope: SkillScope,
) -> Vec<ProjectSkill> {
    let mut skills: Vec<_> = saved
        .iter()
        .filter(|s| s.scope != scope || s.name != draft.name)
        .cloned()
        .collect();
    skills.push(ProjectSkill {
        scope,
        name: draft.name.clone(),
        revision: 1,
        enabled: true,
        versions: vec![dolores_core::SkillVersion {
            version: 1,
            reviewed_at: 0,
            document: draft.clone(),
            rollback_from: None,
            evaluation: None,
        }],
    });
    skills
}
impl Engine {
    pub(super) fn discard_skill_draft(&self, token: &str) -> Result<Value, String> {
        let mut slot = self.skill_draft_review.lock().map_err(|_| STALE)?;
        if slot.as_ref().is_some_and(|r| r.token == token) {
            slot.take();
        }
        Ok(Value::Null)
    }
    pub(super) fn review_skill_examples(
        &self,
        session: &str,
        scope: SkillScope,
    ) -> Result<Value, String> {
        let folder = self.store.workspace(session)?.root;
        if scope == SkillScope::Project && folder.is_none() {
            return Err("Choose Global skills in a side chat.".into());
        }
        let page = self.store.messages_page(session, None, false, 40)?;
        let examples: Vec<_> = page
            .items
            .chunks_exact(2)
            .filter(|p| p[0].role == Role::User && p[1].role == Role::Assistant)
            .map(|p| SkillExample {
                user_id: p[0].id,
                message_id: p[1].id,
                request: p[0].content.clone(),
                response: p[1].content.clone(),
            })
            .collect();
        let token = uuid::Uuid::new_v4().to_string();
        let result = json!({"token":token,"examples":examples,"hasOlder":page.has_older,"scope":scope,"model":self.store.preferences()?.model,"settings":self.store.request_settings()?});
        self.skill_review.lock().map_err(|_| STALE)?.take();
        *self.skill_draft_review.lock().map_err(|_| STALE)? = Some(DraftReview {
            token,
            session: session.into(),
            scope,
            folder,
            examples,
            draft: None,
            model: None,
            promotion: None,
            created: Instant::now(),
        });
        Ok(result)
    }
    pub(super) fn generate_skill_draft(
        &self,
        active: &mut Option<Run>,
        id: u64,
        session: String,
        token: String,
        ids: Vec<i64>,
        settings: Option<dolores_core::RequestSettings>,
    ) -> Result<Value, String> {
        let mut review = self
            .skill_draft_review
            .lock()
            .map_err(|_| STALE)?
            .clone()
            .ok_or(STALE)?;
        check(&review, &session, &token)?;
        if review.draft.is_some() {
            return Err(STALE.into());
        }
        review.examples = ids
            .iter()
            .map(|id| {
                review
                    .examples
                    .iter()
                    .find(|e| e.message_id == *id)
                    .cloned()
                    .ok_or(STALE)
            })
            .collect::<Result<_, _>>()?;
        fresh(self.store.as_ref(), &review)?;
        let prompt = dolores_core::skill_draft_prompt(&review.examples)?;
        let _entered = self.runtime.enter();
        let provider = self
            .connection
            .lock()
            .map_err(|_| "Connection unavailable.")?
            .skill_draft_provider(settings)?;
        let settings = provider.request_settings().unwrap_or_default();
        let model = self.store.preferences()?.model;
        let (prompt, tokens) = dolores_core::prepare_token_context(
            prompt,
            &[],
            provider.context_window_tokens(),
            settings,
        )?;
        let (output, events) = mpsc::channel(8);
        let cancel = CancellationToken::new();
        *active = Some(Run {
            id,
            cancel: cancel.clone(),
            events,
            approvals: Arc::new(Mutex::new(None)),
        });
        let slot = self.skill_draft_review.clone();
        let store = self.store.clone();
        self.runtime.spawn(async move {
            let result = async {
                let (text, usage) = super::memory_suggestions::collect_review(
                    provider, prompt, cancel.clone(), "Skill draft",
                ).await.map_err(|error| {
                    if error == dolores_provider_openai::OUTPUT_LIMIT_ERROR {
                        format!("Skill draft reached its {}-token output limit. Increase Draft output tokens and generate again, or select less source text. Nothing was saved.", settings.max_output_tokens)
                    } else { error }
                })?;
                let draft = dolores_core::parse_skill_draft(&text, &review.examples)?;
                let reader = store.clone();
                let frozen = review.clone();
                super::blocking(move || fresh(reader.as_ref(), &frozen)).await?;
                let old = review.token.clone();
                review.token = uuid::Uuid::new_v4().to_string();
                review.created = Instant::now();
                review.draft = Some(draft.clone());
                review.model = Some(model.clone());
                let warning = dolores_core::skill_document(&draft.name, &draft.description, &draft.instructions).err();
                let result = json!({"token":review.token,"draft":draft,"model":model,"usage":usage,"tokens":tokens,"warning":warning,"settings":settings});
                publish(&slot, &cancel, &old, review)?;
                Ok::<_, String>(result)
            }.await;
            let event = match result {
                Ok(draft) => json!({"type":"done","id":id,"skillDraft":draft}),
                Err(error) => json!({"type":"done","id":id,"error":error}),
            };
            let _ = output.send(event).await;
        });
        Ok(Value::Null)
    }
    pub(super) fn evaluate_skill_draft(
        &self,
        active: &mut Option<Run>,
        id: u64,
        session: String,
        token: String,
        draft: SkillDraft,
        trials: Vec<SkillTrial>,
    ) -> Result<Value, String> {
        let mut review = self
            .skill_draft_review
            .lock()
            .map_err(|_| STALE)?
            .clone()
            .ok_or(STALE)?;
        check(&review, &session, &token)?;
        let generated = review.draft.as_ref().ok_or(STALE)?;
        if draft.evidence != generated.evidence {
            return Err(
                "Skill evidence is bound to the generated draft. Edit its instructions instead."
                    .into(),
            );
        }
        fresh(self.store.as_ref(), &review)?;
        let document =
            dolores_core::skill_document(&draft.name, &draft.description, &draft.instructions)?;
        dolores_core::validate_skill_trials(&trials)?;
        let saved = super::skills::for_session(self.store.as_ref(), Some(&session))?;
        if saved.iter().filter(|s| s.scope == review.scope).count()
            >= dolores_core::MAX_SAVED_SKILLS
            && !saved
                .iter()
                .any(|s| s.scope == review.scope && s.name == document.name)
        {
            return Err("Skills allow 12 saved entries per scope. Forget an unused entry before testing a new skill.".into());
        }
        let candidate = candidate_skills(&saved, &document, review.scope);
        if review.scope == SkillScope::Global
            && saved
                .iter()
                .any(|s| s.scope == SkillScope::Project && s.enabled && s.name == document.name)
        {
            return Err("This global draft is overridden in this chat. Test it in a side chat or a folder without that project override.".into());
        }
        let _entered = self.runtime.enter();
        let provider = self
            .connection
            .lock()
            .map_err(|_| "Connection unavailable.")?
            .bounded_review_provider(512, 10)?;
        let model = self.store.preferences()?.model;
        // Freeze both prompt sets and check every budget before the first request.
        let mut prompts = vec![];
        let mut baseline_sources = vec![];
        for trial in &trials {
            let base = dolores_core::preview_context(vec![], &trial.prompt)?;
            let (baseline, sources) = dolores_core::prepare_skill_context(base.clone(), &saved)?;
            baseline_sources = sources;
            let (candidate, _) = dolores_core::prepare_skill_context(base, &candidate)?;
            let baseline = dolores_core::prepare_token_context(
                baseline,
                &[],
                provider.context_window_tokens(),
                provider.request_settings().unwrap_or_default(),
            )?;
            let candidate = dolores_core::prepare_token_context(
                candidate,
                &[],
                provider.context_window_tokens(),
                provider.request_settings().unwrap_or_default(),
            )?;
            prompts.push((baseline, candidate));
        }
        // Any previous passing result ceases to be promotable as evaluation starts.
        review.promotion = None;
        review.draft = Some(draft);
        *self.skill_draft_review.lock().map_err(|_| STALE)? = Some(review.clone());
        let (output, events) = mpsc::channel(8);
        let cancel = CancellationToken::new();
        *active = Some(Run {
            id,
            cancel: cancel.clone(),
            events,
            approvals: Arc::new(Mutex::new(None)),
        });
        let slot = self.skill_draft_review.clone();
        let store = self.store.clone();
        self.runtime.spawn(async move {
            let result = async {
                let mut results = vec![];
                let mut token_reports = vec![];
                for (index, (trial, (baseline, candidate))) in trials.into_iter().zip(prompts).enumerate() {
                    let baseline_messages = baseline.0.clone();
                    let candidate_messages = candidate.0.clone();
                    let _ = output.try_send(json!({"type":"skillEvaluationProgress","id":id,"test":index+1,"phase":"baseline"}));
                    let (base, base_usage) = super::memory_suggestions::collect_review(
                        provider.clone(), baseline.0, cancel.clone(), "Skill evaluation",
                    ).await?;
                    let _ = output.try_send(json!({"type":"skillEvaluationProgress","id":id,"test":index+1,"phase":"candidate"}));
                    let (answer, usage) = super::memory_suggestions::collect_review(
                        provider.clone(), candidate.0, cancel.clone(), "Skill evaluation",
                    ).await?;
                    token_reports.push(json!({"baseline":baseline.1,"candidate":candidate.1}));
                    results.push(SkillTrialResult {
                        trial, baseline_messages, candidate_messages,
                        baseline: base, candidate: answer,
                        baseline_usage: base_usage, candidate_usage: usage,
                    });
                }
                let evaluation = SkillEvaluation {
                    session: session.clone(), model,
                    generated_model: review.model.clone().ok_or(STALE)?,
                    settings: provider.request_settings().unwrap_or_default(),
                    context_window_tokens: provider.context_window_tokens(),
                    reviewed_at: SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_millis() as i64,
                    evidence: review.draft.as_ref().ok_or(STALE)?.evidence.clone(),
                    baseline_skills: baseline_sources, results,
                };
                evaluation.validate()?;
                let reader = store.clone();
                let frozen = review.clone();
                let expected = saved.clone();
                super::blocking(move || {
                    fresh(reader.as_ref(), &frozen)?;
                    if super::skills::for_session(reader.as_ref(), Some(&frozen.session))? != expected {
                        return Err(STALE.into());
                    }
                    Ok(())
                }).await?;
                let (baseline, candidate) = evaluation.scores();
                let old = review.token.clone();
                review.token = uuid::Uuid::new_v4().to_string();
                review.created = Instant::now();
                review.promotion = Some(SkillPromotion {
                    scope: review.scope, folder: review.folder.clone(),
                    document: document.clone(), examples: review.examples.clone(),
                    saved, evaluation: evaluation.clone(),
                });
                let result = json!({"token":review.token,"document":document,"evaluation":evaluation,"baselinePassed":baseline,"candidatePassed":candidate,"promotable":evaluation.promotable(),"tokens":token_reports});
                publish(&slot, &cancel, &old, review)?;
                Ok::<_, String>(result)
            }.await;
            let event = match result {
                Ok(result) => json!({"type":"done","id":id,"skillEvaluation":result}),
                Err(error) => json!({"type":"done","id":id,"error":error}),
            };
            let _ = output.send(event).await;
        });
        Ok(Value::Null)
    }
    pub(super) fn promote_skill_draft(&self, session: &str, token: &str) -> Result<Value, String> {
        let mut slot = self.skill_draft_review.lock().map_err(|_| STALE)?;
        let review = slot.as_ref().ok_or(STALE)?;
        check(review, session, token)?;
        fresh(self.store.as_ref(), review)?;
        let promotion = review
            .promotion
            .as_ref()
            .ok_or("Evaluate the draft before activation.")?;
        let saved = self.store.promote_skill(promotion)?;
        slot.take();
        Ok(json!(saved))
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use dolores_store_sqlite::SqliteStore;
    #[test]
    fn generation_can_finish_with_the_configured_output_budget() {
        use std::io::{Read, Write};
        for (configured, requested, expected, success) in [
            (4096, None, 4096, true),
            (1024, None, 1024, false),
            (2048, Some(8192), 8192, true),
        ] {
            let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
            let endpoint = format!("http://{}/v1", listener.local_addr().unwrap());
            let fixture = std::thread::spawn(move || {
                let (mut socket, _) = listener.accept().unwrap();
                socket
                    .set_read_timeout(Some(Duration::from_secs(5)))
                    .unwrap();
                let mut bytes = Vec::new();
                let request = loop {
                    let mut chunk = [0; 4096];
                    let n = socket.read(&mut chunk).unwrap();
                    assert_ne!(n, 0);
                    bytes.extend_from_slice(&chunk[..n]);
                    if let Some(offset) = bytes.windows(4).position(|w| w == b"\r\n\r\n") {
                        let headers = String::from_utf8_lossy(&bytes[..offset]);
                        let length: usize = headers
                            .lines()
                            .find_map(|line| {
                                line.to_ascii_lowercase()
                                    .strip_prefix("content-length:")
                                    .map(|n| n.trim().parse().unwrap())
                            })
                            .unwrap();
                        if bytes.len() >= offset + 4 + length {
                            break serde_json::from_slice::<Value>(
                                &bytes[offset + 4..offset + 4 + length],
                            )
                            .unwrap();
                        }
                    }
                };
                assert!(request.get("tools").is_none());
                let complete = request["max_tokens"].as_u64().unwrap() >= 4096;
                let text = if complete {
                    json!({"name":"review","description":"When reviewing work.","instructions":"Use focused tests.","evidence":[{"messageId":2,"quote":"Use focused tests."}]}).to_string()
                } else {
                    "{\"name\":\"review\",\"instructions\":\"".into()
                };
                let frames = format!(
                    "data: {}\n\ndata: {}\n\ndata: [DONE]\n\n",
                    json!({"choices":[{"delta":{"content":text},"finish_reason":null}]}),
                    json!({"choices":[{"delta":{},"finish_reason":if complete {"stop"} else {"length"}}]})
                );
                write!(socket,"HTTP/1.1 200 OK\r\nContent-Type: text/event-stream\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}", frames.len(), frames).unwrap();
                request
            });
            let store = Arc::new(SqliteStore::open(std::path::Path::new(":memory:")).unwrap());
            store.create("side").unwrap();
            store
                .commit_turn("side", "Review", "Use focused tests.")
                .unwrap();
            let saved_settings = dolores_core::RequestSettings {
                max_output_tokens: configured,
                timeout_seconds: 90,
            };
            store.save_request_settings(&saved_settings).unwrap();
            let engine = Engine::new(
                store.clone(),
                Arc::new(crate::connection::testing::MemoryCredentials::default()),
            )
            .unwrap();
            {
                let _entered = engine.runtime.enter();
                engine
                    .connection
                    .lock()
                    .unwrap()
                    .configure(
                        dolores_core::ConnectionPreferences {
                            base_url: endpoint,
                            model: "fixture".into(),
                        },
                        Some(String::new()),
                        false,
                    )
                    .unwrap();
            }
            let review = engine
                .review_skill_examples("side", SkillScope::Global)
                .unwrap();
            assert_eq!(review["settings"], json!(saved_settings));
            let mut active = None;
            let token = review["token"].as_str().unwrap().to_string();
            let invalid = engine
                .generate_skill_draft(
                    &mut active,
                    0,
                    "side".into(),
                    token.clone(),
                    vec![2],
                    Some(dolores_core::RequestSettings {
                        max_output_tokens: 0,
                        timeout_seconds: 90,
                    }),
                )
                .unwrap_err();
            assert!(invalid.contains("Output token limit"));
            assert!(active.is_none());
            let override_settings = requested.map(|n| dolores_core::RequestSettings {
                max_output_tokens: n,
                timeout_seconds: 120,
            });
            engine
                .generate_skill_draft(
                    &mut active,
                    1,
                    "side".into(),
                    token.clone(),
                    vec![2],
                    override_settings,
                )
                .unwrap();
            let event = engine.runtime.block_on(async {
                tokio::time::timeout(
                    Duration::from_secs(5),
                    active.as_mut().unwrap().events.recv(),
                )
                .await
                .unwrap()
                .unwrap()
            });
            let request = fixture.join().unwrap();
            assert_eq!(request["max_tokens"], expected);
            if success {
                assert!(event.get("error").is_none(), "{event}");
                assert_eq!(event["skillDraft"]["draft"]["name"], "review");
                assert_eq!(
                    event["skillDraft"]["settings"]["timeoutSeconds"],
                    if requested.is_some() { 120 } else { 90 }
                );
            } else {
                let error = event["error"].as_str().unwrap();
                assert!(
                    error.contains("1024-token output limit")
                        && error.contains("Increase Draft output tokens"),
                    "{error}"
                );
                let slot = engine.skill_draft_review.lock().unwrap();
                assert_eq!(slot.as_ref().unwrap().token, token);
                assert!(slot.as_ref().unwrap().draft.is_none());
            }
            assert!(store.global_skills().unwrap().is_empty());
            assert_eq!(store.request_settings().unwrap(), saved_settings);
        }
    }
    #[test]
    fn saved_entry_limit_is_checked_before_provider_setup_and_allows_replacing_an_entry() {
        let temp = tempfile::tempdir().unwrap();
        let store = Arc::new(SqliteStore::open(&temp.path().join("state.db")).unwrap());
        store.create("side").unwrap();
        store
            .commit_turn("side", "Review", "Use focused tests.")
            .unwrap();
        let engine = Engine::new(
            store.clone(),
            Arc::new(crate::connection::testing::MemoryCredentials::default()),
        )
        .unwrap();
        engine
            .review_skill_examples("side", SkillScope::Global)
            .unwrap();
        let review = engine.skill_draft_review.lock().unwrap().clone().unwrap();
        let mut draft = SkillDraft {
            name: "new-entry".into(),
            description: "Review changes".into(),
            instructions: "Use focused tests.".into(),
            evidence: vec![dolores_core::SkillEvidence {
                message_id: review.examples[0].message_id,
                quote: "Use focused tests.".into(),
            }],
        };
        engine
            .skill_draft_review
            .lock()
            .unwrap()
            .as_mut()
            .unwrap()
            .draft = Some(draft.clone());
        for number in 0..12 {
            let name = format!("entry-{number}");
            let doc = dolores_core::skill_document(&name, "Review", "Check").unwrap();
            store.activate_global_skill(&doc, None, None).unwrap();
            store.disable_global_skill(&name, 1).unwrap();
        }
        let trials = vec![SkillTrial {
            prompt: "Review".into(),
            required: vec!["focused".into()],
            forbidden: vec![],
        }];
        let mut active = None;
        let error = engine
            .evaluate_skill_draft(
                &mut active,
                1,
                "side".into(),
                review.token.clone(),
                draft.clone(),
                trials.clone(),
            )
            .unwrap_err();
        assert!(error.contains("12 saved entries"));
        assert!(active.is_none());
        draft.name = "entry-0".into();
        let error = engine
            .evaluate_skill_draft(&mut active, 2, "side".into(), review.token, draft, trials)
            .unwrap_err();
        assert!(error.contains("model connection"));
        assert!(active.is_none());
    }
    #[test]
    fn source_review_is_local_scoped_bounded_and_requires_fresh_complete_exchanges() {
        let temp = tempfile::tempdir().unwrap();
        let store = Arc::new(SqliteStore::open(&temp.path().join("state.db")).unwrap());
        store.create("side").unwrap();
        for index in 0..25 {
            store
                .commit_turn("side", &format!("Request {index}"), "Use focused tests.")
                .unwrap();
        }
        let engine = Engine::new(
            store.clone(),
            Arc::new(crate::connection::testing::MemoryCredentials::default()),
        )
        .unwrap();
        assert!(engine
            .review_skill_examples("side", SkillScope::Project)
            .is_err());
        let result = engine
            .review_skill_examples("side", SkillScope::Global)
            .unwrap();
        assert_eq!(result["examples"].as_array().unwrap().len(), 20);
        assert_eq!(result["hasOlder"], true);
        let mut review = engine.skill_draft_review.lock().unwrap().clone().unwrap();
        check(&review, "side", result["token"].as_str().unwrap()).unwrap();
        assert!(check(&review, "other", result["token"].as_str().unwrap()).is_err());
        assert!(engine.promote_skill_draft("side", &review.token).is_err());
        fresh(store.as_ref(), &review).unwrap();
        review.created = Instant::now() - Duration::from_secs(301);
        assert!(check(&review, "side", &review.token).is_err());
        store.delete("side").unwrap();
        assert!(fresh(store.as_ref(), &review).is_err());
    }
    #[test]
    fn cancelled_or_replaced_review_cannot_publish_a_late_result() {
        let temp = tempfile::tempdir().unwrap();
        let store = Arc::new(SqliteStore::open(&temp.path().join("state.db")).unwrap());
        store.create("side").unwrap();
        let engine = Engine::new(
            store,
            Arc::new(crate::connection::testing::MemoryCredentials::default()),
        )
        .unwrap();
        engine
            .review_skill_examples("side", SkillScope::Global)
            .unwrap();
        let review = engine.skill_draft_review.lock().unwrap().clone().unwrap();
        let cancel = CancellationToken::new();
        cancel.cancel();
        assert!(publish(
            &engine.skill_draft_review,
            &cancel,
            &review.token,
            review.clone()
        )
        .is_err());
        engine
            .review_skill_examples("side", SkillScope::Global)
            .unwrap();
        assert!(publish(
            &engine.skill_draft_review,
            &CancellationToken::new(),
            &review.token,
            review.clone()
        )
        .is_err());
        engine.discard_skill_draft(&review.token).unwrap();
        assert!(engine.skill_draft_review.lock().unwrap().is_some());
        let current = engine
            .skill_draft_review
            .lock()
            .unwrap()
            .as_ref()
            .unwrap()
            .token
            .clone();
        engine.discard_skill_draft(&current).unwrap();
        assert!(engine.skill_draft_review.lock().unwrap().is_none());
    }
}
