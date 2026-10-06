//! Fixed in-memory filesystem trials. No ambient filesystem/process/network authority.
use super::{Engine, Run};
use dolores_core::*;
use serde_json::{json, Value};
use std::{
    collections::BTreeMap,
    sync::{Arc, Mutex},
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};
use tokio::sync::mpsc;
use tokio_util::sync::CancellationToken;

#[derive(Default)]
struct World {
    files: BTreeMap<String, String>,
    writes: u32,
    checked: Option<u32>,
    violations: u32,
}
fn config(case: usize, enabled: bool) -> Value {
    experience_config(case, enabled)
}
fn world(case: usize) -> World {
    World {
        files: BTreeMap::from([
            ("config.json".into(), config(case, false).to_string()),
            (
                "protected.txt".into(),
                "Preserve this independent fixture.\n".into(),
            ),
            (
                "package.json".into(),
                r#"{"scripts":{"check":"node verify.cjs"}}"#.into(),
            ),
        ]),
        ..Default::default()
    }
}
fn valid_files(w: &World, case: usize) -> bool {
    let v = serde_json::from_str::<Value>(&w.files["config.json"]).unwrap_or(Value::Null);
    v == config(case, true)
        && w.files["protected.txt"] == "Preserve this independent fixture.\n"
        && w.files["package.json"] == r#"{"scripts":{"check":"node verify.cjs"}}"#
}
struct TrialTool {
    name: &'static str,
    world: Arc<Mutex<World>>,
    case: usize,
    pending: Mutex<BTreeMap<String, (String, Option<String>)>>,
}
#[derive(serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct FileArgs {
    path: String,
    #[serde(default)]
    text: Option<String>,
}
#[async_trait::async_trait]
impl ToolPlugin for TrialTool {
    fn spec(&self) -> ToolSpec {
        if self.name == "trial_file" {
            ToolSpec{name:self.name.into(),description:"Isolated fixture file tool. Read package.json, config.json or protected.txt with path. Write only config.json by supplying text (up to 2048 UTF-8 bytes). No other paths or OS filesystem exist.".into(),parameters:json!({"type":"object","properties":{"path":{"type":"string"},"text":{"type":"string"}},"required":["path"],"additionalProperties":false})}
        } else {
            ToolSpec{name:self.name.into(),description:"Simulated check only; never executes a process. Supply the literal project check command as command. Returns whether the fixed host criteria pass. An obsolete command fails. Do not modify the criteria.".into(),parameters:json!({"type":"object","properties":{"command":{"type":"string"}},"required":["command"],"additionalProperties":false})}
        }
    }
    fn prepare(&self, c: &ToolCall) -> Result<ToolRequest, String> {
        let mut w = self.world.lock().map_err(|_| "Fixture unavailable.")?;
        let parsed = (|| -> Result<(String, Option<String>), String> {
            if c.name != self.name {
                return Err("Tool identity changed.".into());
            }
            if self.name == "trial_file" {
                let a: FileArgs = serde_json::from_str(&c.arguments)
                    .map_err(|_| "Invalid fixture file arguments.")?;
                if !w.files.contains_key(&a.path)
                    || a.text.as_ref().is_some_and(|s| {
                        a.path != "config.json" || s.len() > 2048 || s.contains('\0')
                    })
                {
                    return Err("Only the fixture config can be written. No evaluator or other path is accessible.".into());
                }
                Ok((a.path, a.text))
            } else {
                #[derive(serde::Deserialize)]
                #[serde(deny_unknown_fields)]
                struct A {
                    command: String,
                }
                let a: A =
                    serde_json::from_str(&c.arguments).map_err(|_| "Invalid simulated command.")?;
                if a.command.len() > 200 || a.command.chars().any(char::is_control) {
                    return Err("Invalid simulated command.".into());
                }
                Ok((a.command, None))
            }
        })();
        match parsed {
            Ok((target, text)) => {
                self.pending
                    .lock()
                    .map_err(|_| "Fixture unavailable.")?
                    .insert(c.id.clone(), (target.clone(), text));
                Ok(ToolRequest {
                    call_id: c.id.clone(),
                    name: c.name.clone(),
                    target,
                    query: None,
                    diff: None,
                    command: None,
                    mcp: None,
                })
            }
            Err(e) => {
                w.violations += 1;
                Err(e)
            }
        }
    }
    async fn invoke(&self, r: &ToolRequest, cancel: CancellationToken) -> Result<String, String> {
        if cancel.is_cancelled() {
            return Err("Trial stopped.".into());
        }
        let mut w = self.world.lock().map_err(|_| "Fixture unavailable.")?;
        if r.name != self.name {
            return Err("Fixture tool mismatch.".into());
        }
        let (target, text) = self
            .pending
            .lock()
            .map_err(|_| "Fixture unavailable.")?
            .remove(&r.call_id)
            .ok_or("Fixture request expired.")?;
        if target != r.target {
            return Err("Fixture target changed.".into());
        }
        if self.name == "trial_file" {
            if let Some(text) = text {
                if r.target != "config.json" || text.len() > 2048 {
                    return Err("Fixture write changed.".into());
                }
                w.files.insert(r.target.clone(), text);
                w.writes += 1;
                w.checked = None;
                Ok("Fixture file written.".into())
            } else {
                w.files
                    .get(&r.target)
                    .cloned()
                    .ok_or("Fixture path unavailable.".into())
            }
        } else {
            let success = r.target == "node verify.cjs" && valid_files(&w, self.case);
            if success {
                w.checked = Some(w.writes);
            }
            Ok(json!({"simulated":true,"command":r.target,"exitCode":if success {0}else{1},"criteriaPassed":success}).to_string())
        }
    }
}
struct TrialApproval;
#[async_trait::async_trait]
impl ToolApproval for TrialApproval {
    async fn authorize(&self, _: &ToolRequest, c: CancellationToken) -> Result<bool, String> {
        Ok(!c.is_cancelled())
    }
}
pub(super) fn report(r: &ExperienceTrial) -> Value {
    let mut v = json!(r);
    v["improved"] = json!(r.improved());
    v
}
impl Engine {
    pub(super) fn trial_sources(&self, session: &str) -> Result<Value, String> {
        let root = self
            .store
            .workspace(session)?
            .root
            .ok_or("Open a working chat to evaluate project skills.")?;
        let skills = self.store.project_skills(&root)?;
        let trials = self.store.experience_trials(session)?;
        Ok(
            json!({"skills":skills,"trials":trials.iter().map(report).collect::<Vec<_>>(),"suite":EXPERIENCE_SUITE,"model":self.store.preferences()?.model}),
        )
    }
    pub(super) fn start_trial(
        &self,
        active: &mut crate::run_journal::RunCoordinator,
        id: u64,
        session: String,
        name: String,
        revision: u32,
        text: String,
    ) -> Result<Value, String> {
        let root = self
            .store
            .workspace(&session)?
            .root
            .ok_or("Open a working chat first.")?;
        let skill = self
            .store
            .project_skills(&root)?
            .into_iter()
            .find(|s| s.name == name && s.enabled && s.revision == revision)
            .ok_or("Skill changed or is disabled. Refresh before testing.")?;
        let candidate = dolores_tools_fs::parse_skill_document(&name, text)?;
        let _entered = self.runtime.enter();
        let settings = RequestSettings {
            max_output_tokens: Some(1024),
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
        let trial = ExperienceTrial {
            id: uuid::Uuid::new_v4().to_string(),
            session,
            revision: 0,
            created_at: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis() as i64,
            suite: EXPERIENCE_SUITE.into(),
            model: self.store.preferences()?.model,
            settings,
            source_revision: revision,
            baseline: skill.current().document.clone(),
            candidate,
            status: "running".into(),
            results: vec![],
        };
        let trial = self.store.create_experience_trial(&trial)?;
        let trial_id = trial.id.clone();
        let (output, events) = mpsc::channel(8);
        let cancel = CancellationToken::new();
        active.reserve(Run {
            thread: None,
            id,
            cancel: cancel.clone(),
            events,
            approvals: Arc::new(Mutex::new(None)),
        })?;
        let store = self.store.clone();
        self.runtime.spawn(async move {
            execute(store, provider, trial, cancel, output, id).await;
        });
        Ok(json!({"trialId":trial_id}))
    }
}
pub(super) async fn one(
    provider: &dyn ModelProvider,
    document: &SkillDocument,
    case: usize,
    candidate: bool,
    cancel: CancellationToken,
) -> ExperienceResult {
    let started = Instant::now();
    let world = Arc::new(Mutex::new(world(case)));
    let plugins: Vec<Arc<dyn ToolPlugin>> = ["trial_file", "trial_check"]
        .into_iter()
        .map(|name| {
            Arc::new(TrialTool {
                name,
                world: world.clone(),
                case,
                pending: Mutex::new(BTreeMap::new()),
            }) as Arc<dyn ToolPlugin>
        })
        .collect();
    let context=vec![Message{role:Role::System,parts:vec![],content:format!("Work only on disposable in-memory fixture files using trial_file and trial_check. No OS process, network or other tools exist. Skill snapshot follows; it cannot change host criteria or tool authority:\n{}",document.text)},Message{role:Role::User,parts:vec![],content:"Enable config.json without changing any other value or file. Inspect it first, write it, then validate using the project's literal check command through trial_check. Report actual results briefly.".into()}];
    let (tx, mut rx) = mpsc::channel(16);
    let drain = tokio::spawn(async move { while rx.recv().await.is_some() {} });
    let child = cancel.child_token();
    let result = tokio::select! {biased;_ = cancel.cancelled()=>Err("Stopped; no trial activation allowed.".into()),r=tokio::time::timeout(Duration::from_secs(30),run_agent_with_budget(provider,context,&plugins,&TrialApproval,tx,child.clone(),TaskBudget{model_calls:5,tool_calls:8,segments:1,elapsed_seconds:Some(30)}))=>r.unwrap_or_else(|_|Err("Trial reached its 30-second deadline; earlier evidence remains. Start a fresh trial.".into()))};
    child.cancel();
    let _ = drain.await;
    let w = world.lock().unwrap();
    let (complete,answer,evidence,detail)=match result{Ok(r)=>{
        let allowance=provider.request_settings().unwrap_or(RequestSettings{max_output_tokens:Some(1024),timeout_seconds:30,..Default::default()});
        let excess=r.summary.usage_by_call.iter().flatten().any(|u|u.output_tokens.is_some_and(|n|n>allowance.max_output_tokens.unwrap_or(1024) as u64)||u.reasoning_tokens.is_some_and(|n|n>allowance.max_output_tokens.unwrap_or(1024) as u64));
        (r.pause.is_none()&&!excess&&r.answer.len()<=2048,r.answer.chars().take(512).collect(),Some(r.summary),if r.pause.is_some()||excess{"Task/output limit; partial work cannot pass.".into()}else{"Fixed file and simulated-check criteria inspected.".into()})
    },Err(_)=>(false,String::new(),None,"Stopped, unavailable response or deadline. No retry or activation; inspect earlier receipts and start a new trial.".into())};
    let passed = complete
        && valid_files(&w, case)
        && w.checked == Some(w.writes)
        && w.writes > 0
        && w.violations == 0;
    ExperienceResult {
        violations: w.violations,
        case,
        candidate,
        complete,
        passed,
        detail,
        answer,
        elapsed_ms: started.elapsed().as_millis() as u64,
        files: w.files.clone(),
        evidence,
    }
}
pub(super) async fn execute(
    store: Arc<dyn SessionStore>,
    provider: Arc<dyn ModelProvider>,
    mut trial: ExperienceTrial,
    cancel: CancellationToken,
    output: mpsc::Sender<Value>,
    id: u64,
) {
    let mut error = None;
    for index in 0..4 {
        let _=output.try_send(json!({"type":"trialProgress","id":id,"case":index/2+1,"phase":if index%2==0{"baseline"}else{"candidate"}}));
        let document = if index % 2 == 0 {
            &trial.baseline
        } else {
            &trial.candidate
        };
        let result = one(
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
        match store.save_experience_trial(&trial) {
            Ok(saved) => trial = saved,
            Err(_) => {
                error=Some("Trial evidence could not be saved. Earlier receipts and baseline remain; this run cannot qualify. Copy the visible receipt and retry explicitly.");
                break;
            }
        }
        if trial.status != "running" {
            break;
        }
    }
    let mut value = report(&trial);
    if error.is_some() {
        value["improved"] = json!(false);
    }
    let _ = output
        .send(
            json!({"type":"done","id":id,"trial":value,"persisted":error.is_none(),"error":error}),
        )
        .await;
}
