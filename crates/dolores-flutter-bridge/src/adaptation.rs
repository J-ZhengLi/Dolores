use super::{experience, knowledge, Engine, Run};
use dolores_core::*;
use serde_json::{json, Value};
use std::{
    sync::{Arc, Mutex},
    time::{SystemTime, UNIX_EPOCH},
};
use tokio::sync::mpsc;
use tokio_util::sync::CancellationToken;
fn now() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as i64
}
fn root(store: &dyn SessionStore, session: &str) -> Result<String, String> {
    store
        .workspace(session)?
        .root
        .ok_or("Open a project or temporary working chat for skill learning.".into())
}
fn source_current(
    store: &dyn SessionStore,
    root: &str,
    session: &str,
    e: &AdaptationEvent,
) -> Result<bool, String> {
    Ok(store.knowledge(root)?.revision == e.knowledge_revision
        && knowledge::facts(store, session)?
            .iter()
            .any(|f| Some(&f.id) == e.fact_id.as_ref()))
}
impl Engine {
    pub(super) fn learning_view(&self, session: &str) -> Result<Value, String> {
        let root = root(self.store.as_ref(), session)?;
        let state = self.store.adaptation(&root)?;
        let trials = state
            .events
            .iter()
            .filter_map(|e| e.trial_id.as_ref())
            .collect::<Vec<_>>();
        let evidence = self
            .store
            .experience_trials(session)?
            .into_iter()
            .filter(|t| trials.contains(&&t.id))
            .map(|t| experience::report(&t))
            .collect::<Vec<_>>();
        Ok(
            json!({"state":state,"trials":evidence,"workflowAvailable":self.store.project_skills(&root)?.iter().all(|s|s.name!="project-check"),"boundary":"config-check-v1 only; exact node command slot; no global, arbitrary prompt or executable changes"}),
        )
    }
    pub(super) fn learning_policy(
        &self,
        session: &str,
        revision: u32,
        enabled: bool,
        automatic: bool,
        paused: bool,
    ) -> Result<Value, String> {
        let root = root(self.store.as_ref(), session)?;
        let mut s = self.store.adaptation(&root)?;
        if s.revision != revision {
            return Err("Learning settings changed. Refresh before retrying.".into());
        }
        s.enabled = enabled;
        s.automatic = automatic;
        s.paused = paused;
        s.policy_revision = s
            .policy_revision
            .checked_add(1)
            .ok_or("Policy revision exhausted.")?;
        self.store.save_adaptation(&root, revision, &s)?;
        self.learning_view(session)
    }
    pub(super) fn create_check_workflow(
        &self,
        session: &str,
        command: &str,
    ) -> Result<Value, String> {
        let root = root(self.store.as_ref(), session)?;
        let doc = check_workflow(command)?;
        self.store.activate_project_skill(&root, &doc, None, None)?;
        self.learning_view(session)
    }
    pub(super) fn approve_learning(
        &self,
        session: &str,
        revision: u32,
        event: &str,
    ) -> Result<Value, String> {
        let root = root(self.store.as_ref(), session)?;
        let state = self.store.adaptation(&root)?;
        let e = state
            .events
            .iter()
            .find(|e| e.id == event)
            .ok_or("Proposal missing.")?;
        if !source_current(self.store.as_ref(), &root, session, e)? {
            return Err("Knowledge is stale or changed. Reinspect; baseline retained.".into());
        }
        self.store.activate_adaptation(&root, revision, event)?;
        self.learning_view(session)
    }
    pub(super) fn start_reflection(
        &self,
        active: &mut crate::run_journal::RunCoordinator,
        id: u64,
        session: String,
    ) -> Result<Value, String> {
        let _entered = self.runtime.enter();
        let settings = RequestSettings {
            max_output_tokens: 1024,
            timeout_seconds: 30,
            ..self
                .store
                .effective_request_settings(&self.store.preferences()?)?
        };
        let provider = self
            .connection
            .lock()
            .map_err(|_| "Model connection unavailable.")?
            .comparison_provider(settings)?;
        let (output, events) = mpsc::channel(8);
        let cancel = CancellationToken::new();
        active.reserve(Run {
            thread: Some(session.clone()),
            id,
            cancel: cancel.clone(),
            events,
            approvals: Arc::new(Mutex::new(None)),
        })?;
        let store = self.store.clone();
        let model = self.store.preferences()?.model;
        self.runtime.spawn(async move{
   let result=reflect(store,Some(provider),&session,&model,cancel,&output,id).await;
   let _=output.send(json!({"type":"done","id":id,"learning":result.as_ref().ok(),"error":result.err()})).await;
  });
        Ok(Value::Null)
    }
}
pub(super) async fn reflect(
    store: Arc<dyn SessionStore>,
    provider: Option<Arc<dyn ModelProvider>>,
    session: &str,
    model: &str,
    cancel: CancellationToken,
    output: &mpsc::Sender<Value>,
    id: u64,
) -> Result<Option<String>, String> {
    let Ok(root) = root(store.as_ref(), session) else {
        return Ok(None);
    };
    let mut state = store.adaptation(&root)?;
    if !state.enabled || state.paused || cancel.is_cancelled() {
        return Ok(None);
    }
    let page = store.messages_page(session, None, false, 2)?;
    let Some(message) = page.items.last().filter(|m| m.role == Role::Assistant) else {
        return Ok(None);
    };
    if state
        .events
        .iter()
        .any(|e| e.session == session && e.message_id == message.id)
    {
        return Ok(None);
    }
    if state.events.len() >= 20 {
        return Err("Learning history reached 20 events. Earlier history and baseline remain; disable learning or use another project. No retry.".into());
    }
    let metadata = message.metadata.as_ref();
    let records = metadata
        .and_then(|m| m.agent.as_ref())
        .map(|a| a.tools.as_slice())
        .unwrap_or_default();
    let commands = unresolved_commands(records);
    let note = store.knowledge(&root)?.share_feedback
        && message
            .feedback
            .as_ref()
            .is_some_and(|f| f.outcome == Some(TaskOutcome::NeedsWork));
    if commands.is_empty() && metadata.and_then(|m| m.paused.as_ref()).is_none() && !note {
        return Ok(None);
    }
    let knowledge = store.knowledge(&root)?;
    let facts = knowledge::facts(store.as_ref(), session)?;
    let baseline = store.project_skills(&root)?.into_iter().find(|s| {
        s.enabled
            && s.name == "project-check"
            && metadata.is_some_and(|m| {
                m.context.skills.iter().any(|src| {
                    src.scope == SkillScope::Project
                        && src.name == s.name
                        && src.version == s.current().version
                })
            })
    });
    let fact = facts.iter().find(|f| {
        f.kind == "convention"
            && f.title == "Project script: check"
            && f.basis == "observed"
            && f.source.as_ref().is_some_and(|s| {
                s.path.as_deref() == Some("package.json") && s.quote == "node verify.cjs"
            })
    });
    let mut event=AdaptationEvent{id:uuid::Uuid::new_v4().to_string(),session:session.into(),message_id:message.id,created_at:now(),cause:"inconclusive".into(),confidence:"low".into(),reason:"No evidence ties this incident to a repairable skill. Baseline retained; review the task and its limits.".into(),status:"inconclusive".into(),policy_revision:state.policy_revision,knowledge_revision:knowledge.revision,fact_id:None,baseline:None,candidate:None,trial_id:None,activated_revision:None,monitor_message:message.id};
    if let (Some(b), Some(f)) = (baseline, fact) {
        let old = workflow_command(&b.current().document);
        let failed = commands.iter().any(|c| {
            Some(format!("{} {}", c.program, c.args.join(" "))) == old
                && c.args.len() == 1
                && records.iter().any(|r| {
                    r.command.as_ref() == Some(c)
                        && CommandOutcome::from_content(&r.content) == CommandOutcome::Failed
                })
        });
        let request = page
            .items
            .first()
            .map(|m| m.content.to_lowercase())
            .unwrap_or_default();
        let candidate = check_workflow("node verify.cjs")?;
        if failed
            && request.contains("config.json")
            && request.contains("enable")
            && limited_repair(&b, &candidate)
        {
            event.cause = "skill".into();
            event.confidence = "high".into();
            event.reason="The used host check workflow's failed literal command disagrees with a current approved package declaration. This identifies an obsolete command binding, not proof that all task failures have the same cause.".into();
            event.status = "pending".into();
            event.baseline = Some(b);
            event.candidate = Some(candidate);
            event.fact_id = Some(f.id.clone());
        }
    }
    if event.status != "pending" {
        if metadata.and_then(|m| m.paused.as_ref()).is_some() {
            event.cause = "limit".into();
            event.reason="Task/output/command review stopped completion. Limits and unrelated failures do not justify instruction edits. Inspect saved tool evidence and continue explicitly.".into();
        }
        if records
            .iter()
            .any(|r| matches!(r.status.as_str(), "denied" | "blocked"))
        {
            event.cause = "tool".into();
            event.reason="Access or tool preparation was refused; do not broaden authority or rewrite a skill to bypass it.".into();
        }
    }
    let event_id = event.id.clone();
    let pending = event.status == "pending";
    state.events.push(event);
    state = store.save_adaptation(&root, state.revision, &state)?;
    if !pending {
        return Ok(Some(state.events.last().unwrap().reason.clone()));
    }
    let _=output.try_send(json!({"type":"learningProgress","id":id,"note":"Reply saved. Testing a targeted check-command repair under separate fixed allowances…"}));
    let event = state.events.last().unwrap().clone();
    let baseline = event.baseline.as_ref().unwrap();
    let mut trial = ExperienceTrial {
        id: uuid::Uuid::new_v4().to_string(),
        session: session.into(),
        revision: 0,
        created_at: now(),
        suite: EXPERIENCE_SUITE.into(),
        model: model.into(),
        settings: provider
            .as_ref()
            .and_then(|p| p.request_settings())
            .unwrap_or(RequestSettings {
                max_output_tokens: 1024,
                timeout_seconds: 30,
                ..Default::default()
            }),
        source_revision: baseline.revision,
        baseline: baseline.current().document.clone(),
        candidate: event.candidate.clone().unwrap(),
        status: "running".into(),
        results: vec![],
    };
    trial = store.create_experience_trial(&trial)?;
    state.events.last_mut().unwrap().trial_id = Some(trial.id.clone());
    state = store.save_adaptation(&root, state.revision, &state)?;
    if let Some(provider) = provider {
        for index in 0..4 {
            let document = if index % 2 == 0 {
                &trial.baseline
            } else {
                &trial.candidate
            };
            let result = experience::one(
                provider.as_ref(),
                document,
                index / 2,
                index % 2 == 1,
                cancel.clone(),
            )
            .await;
            let complete = result.complete;
            trial.results.push(result);
            trial.status = if cancel.is_cancelled() {
                "stopped"
            } else if !complete {
                "failed"
            } else if index == 3 {
                "completed"
            } else {
                "running"
            }
            .into();
            trial = store.save_experience_trial(&trial)?;
            if trial.status != "running" {
                break;
            }
        }
    } else {
        trial.status = "failed".into();
        trial = store.save_experience_trial(&trial)?;
    }
    let latest = store.adaptation(&root)?;
    if latest.revision != state.revision {
        return Err(
            "Learning policy changed during trials. Evidence retained; baseline unchanged.".into(),
        );
    }
    let e = state.events.last_mut().unwrap();
    e.status = if cancel.is_cancelled() {
        "interrupted"
    } else if trial.improved() {
        "review"
    } else {
        "inconclusive"
    }
    .into();
    e.reason.push_str(if trial.improved(){" All candidate fixture cases passed and improved on baseline."}else{" Independent trials were incomplete, tied or failed. No activation; inspect receipts and use reviewed updates for broader repairs."});
    state = store.save_adaptation(&root, state.revision, &state)?;
    if trial.improved()
        && state.automatic
        && !cancel.is_cancelled()
        && source_current(store.as_ref(), &root, session, state.events.last().unwrap())?
    {
        state = store.activate_adaptation(&root, state.revision, &event_id)?;
    }
    Ok(Some(state.events.last().unwrap().reason.clone()))
}
