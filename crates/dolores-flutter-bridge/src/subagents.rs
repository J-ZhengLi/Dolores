use crate::{forward, run_journal::RunLog, stopped};
use async_trait::async_trait;
use dolores_core::{
    AgentEvent, Message, ModelProvider, Role, SharedTaskBudget, TaskBudget, ToolApproval, ToolCall,
    ToolPlugin, ToolRequest, ToolSpec,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::{
    collections::BTreeMap,
    sync::{Arc, Mutex},
};
use tokio::sync::mpsc;
use tokio_util::sync::CancellationToken;

pub fn spec() -> ToolSpec {
    ToolSpec {
        name: "delegate_tasks".into(),
        description: "Run one batch of 1–2 scoped children sharing parent model/tool/time limits. Each has goal, relative scope, readOnly and optional enabled model. Writable scopes cannot overlap. Or use childId and action=inspect/followUp/interrupt; only followUp takes message. Identity, context, model, scope and permissions remain frozen. Each child has up to 32 model/64 file operations cumulatively, also bounded by parent allowance; follow-ups never reset usage. No commands, MCP or recursion. Parent verifies actual effects and keeps one reporting call. Stop cancels children; completed writes remain.".into(),
        parameters: json!({"type":"object","properties":{"tasks":{"type":"array","minItems":1,"maxItems":2,"items":{"type":"object","properties":{"goal":{"type":"string","maxLength":512},"scope":{"type":"string","description":"Direct relative file/folder; '.' for root","maxLength":128},"readOnly":{"type":"boolean"},"model":{"type":"string","maxLength":128}},"required":["goal","scope","readOnly"],"additionalProperties":false}},"childId":{"type":"string","maxLength":64},"action":{"type":"string","enum":["inspect","followUp","interrupt"]},"message":{"type":"string","maxLength":512},"offset":{"type":"integer","minimum":0}},"oneOf":[{"required":["tasks"]},{"required":["childId","action"]}],"additionalProperties":false}),
    }
}
#[derive(Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct ChildTask {
    goal: String,
    scope: String,
    read_only: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    model: Option<String>,
}
#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct Plan {
    tasks: Vec<ChildTask>,
}
fn valid_scope(path: &str) -> bool {
    path == "."
        || (!path.is_empty()
            && path.len() <= 128
            && !path.contains(['\\', ':'])
            && !path.chars().any(char::is_control)
            && path.split('/').all(|p| !matches!(p, "" | "." | "..")))
}
fn within(scope: &str, path: &str) -> bool {
    if !valid_scope(scope)
        || path.is_empty()
        || path.contains(['\\', ':'])
        || path.chars().any(char::is_control)
        || (path != "." && path.split('/').any(|p| matches!(p, "" | "." | "..")))
    {
        return false;
    }
    scope == "." || path == scope || path.strip_prefix(scope).is_some_and(|s| s.starts_with('/'))
}
fn parse(text: &str) -> Result<Plan, String> {
    if text.len() > 4096 {
        return Err("Subagent plan exceeds 4 KiB.".into());
    }
    let p: Plan = serde_json::from_str(text).map_err(|_| "Invalid subagent plan.")?;
    if !(1..=2).contains(&p.tasks.len())
        || p.tasks.iter().any(|t| {
            t.goal.trim().is_empty()
                || t.goal.len() > 512
                || t.goal
                    .chars()
                    .any(|c| c.is_control() && !matches!(c, '\n' | '\t'))
                || !valid_scope(&t.scope)
                || t.model.as_ref().is_some_and(|m| {
                    m.is_empty() || m.len() > 128 || m.chars().any(char::is_control)
                })
        })
    {
        return Err("Invalid subagent goal or scope.".into());
    }
    if p.tasks.len() == 2 {
        let a = &p.tasks[0];
        let b = &p.tasks[1];
        let x = a.scope.to_lowercase();
        let y = b.scope.to_lowercase();
        if (!a.read_only || !b.read_only) && (within(&x, &y) || within(&y, &x)) {
            return Err("Writable subagent scopes overlap. Split file ownership.".into());
        }
    }
    Ok(p)
}
fn short(text: &str, max: usize) -> String {
    let mut end = text.len().min(max);
    while !text.is_char_boundary(end) {
        end -= 1;
    }
    text[..end].into()
}
pub(super) struct RuntimeContext {
    pub hooks: Option<Arc<crate::lifecycle_hooks::Runtime>>,
    pub resources: Option<Arc<crate::skill_packages::resources::Reader>>,
    pub provider: Arc<dyn ModelProvider>,
    pub context: Vec<Message>,
    pub tools: Vec<Arc<dyn ToolPlugin>>,
    pub approval: Arc<dyn ToolApproval>,
    pub shared: Arc<SharedTaskBudget>,
    pub budget: TaskBudget,
    pub permissions: dolores_core::PermissionPolicy,
    pub log: Arc<RunLog>,
    pub output: mpsc::Sender<Value>,
    pub client_id: u64,
}
struct ChildLedger(Arc<SharedTaskBudget>);
impl dolores_core::BudgetLedger for ChildLedger {
    fn reserve(&self, models: usize, tools: usize) -> Result<bool, String> {
        let allowed = (models == 0 || self.0.reserve_model(true))
            && (tools == 0 || self.0.reserve_tools(tools));
        if let Some(error) = self.0.error() {
            return Err(error);
        }
        Ok(allowed)
    }
}
struct Child {
    resources: Option<Arc<crate::skill_packages::resources::Reader>>,
    task: ChildTask,
    model: String,
    provider: Arc<dyn ModelProvider>,
    context: Mutex<Vec<Message>>,
    budget: Arc<SharedTaskBudget>,
    running: Mutex<Option<CancellationToken>>,
    followups: std::sync::atomic::AtomicUsize,
}
struct Running(Arc<Child>);
impl Drop for Running {
    fn drop(&mut self) {
        self.0
            .running
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .take();
    }
}
#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(super) struct Supervision {
    pub child_id: String,
    pub action: String,
    pub message: Option<String>,
    pub offset: Option<usize>,
}
impl Supervision {
    fn parse(text: &str) -> Result<Self, String> {
        if text.len() > 4096 {
            return Err("Child request exceeds 4 KiB.".into());
        }
        let r: Self = serde_json::from_str(text).map_err(|_| "Invalid child request.")?;
        if r.child_id.is_empty()
            || r.child_id.len() > 64
            || !r.child_id.chars().all(|c| c.is_ascii_alphanumeric())
            || !matches!(r.action.as_str(), "inspect" | "followUp" | "interrupt")
            || (r.action != "inspect" && r.offset.is_some())
            || if r.action == "followUp" {
                r.message.as_ref().is_none_or(|m| {
                    m.trim().is_empty()
                        || m.len() > 512
                        || m.chars()
                            .any(|c| c.is_control() && !matches!(c, '\n' | '\t'))
                })
            } else {
                r.message.is_some()
            }
        {
            return Err("Choose inspect, interrupt, or followUp with a 1–512 byte message. Child scope and model cannot change.".into());
        }
        Ok(r)
    }
}
#[derive(Default)]
pub(super) struct DelegateTasks {
    runtime: Mutex<Option<Arc<RuntimeContext>>>,
    children: Mutex<Vec<Value>>,
    controls: Mutex<BTreeMap<String, Arc<Child>>>,
    pub parent_model: String,
    pub models: BTreeMap<String, Arc<dyn ModelProvider>>,
    claimed: std::sync::atomic::AtomicBool,
}
impl DelegateTasks {
    pub fn bind(&self, runtime: RuntimeContext) -> Result<(), String> {
        *self.runtime.lock().map_err(|_| "Subagents unavailable.")? = Some(Arc::new(runtime));
        Ok(())
    }
    pub fn new(parent_model: String, models: BTreeMap<String, Arc<dyn ModelProvider>>) -> Self {
        Self {
            parent_model,
            models,
            ..Default::default()
        }
    }
    pub fn inspect(&self, id: &str, offset: usize) -> Result<Value, String> {
        let children = self
            .controls
            .lock()
            .map_err(|_| "Child context unavailable.")?;
        let child = children
            .get(id)
            .ok_or("Child is not owned by this parent run.")?;
        let context = child
            .context
            .lock()
            .map_err(|_| "Child context unavailable.")?;
        let context_text = context
            .iter()
            .map(|message| format!("{:?}:\n{}\n\n", message.role, message.content))
            .collect::<String>();
        let report = self
            .children
            .lock()
            .map_err(|_| "Child evidence unavailable.")?
            .iter()
            .find(|v| v["childId"] == id)
            .cloned();
        if offset > context_text.len() || !context_text.is_char_boundary(offset) {
            return Err(
                "Child context page changed or offset is invalid. Inspect again from offset 0."
                    .into(),
            );
        }
        let mut page = short(&context_text[offset..], 8192);
        let mut value = json!({"childId":id,"model":child.model,"requestSettings":child.provider.request_settings(),"scope":child.task.scope,"readOnly":child.task.read_only,"usage":child.budget.usage(),"report":report,"context":page,"offset":offset,"contextBytes":context_text.len(),"contextVersion":child.followups.load(std::sync::atomic::Ordering::SeqCst),"contextKind":"Prepared child instructions and bounded report/tool checkpoints; no parent conversation or attachments. Provider tool-loop detail is in child evidence in Run history.","permissions":"Frozen parent admission; file tools intersect scope/readOnly and recheck the live parent grant. No commands/MCP/recursive delegation."});
        while value.to_string().len() > 14 * 1024 {
            page = short(&page, page.len() / 2);
            value["context"] = json!(page);
        }
        let end = offset + page.len();
        value["contextTruncated"] = json!(end < context_text.len());
        value["nextOffset"] = if end < context_text.len() {
            json!(end)
        } else {
            Value::Null
        };
        Ok(value)
    }
    pub fn interrupt(&self, id: &str) -> Result<Value, String> {
        let children = self.controls.lock().map_err(|_| "Child unavailable.")?;
        let child = children
            .get(id)
            .ok_or("Child is not owned by this parent run.")?;
        let running = child.running.lock().map_err(|_| "Child unavailable.")?;
        let token = running
            .as_ref()
            .ok_or("Child has already settled. Inspect its report; nothing was restarted.")?;
        token.cancel();
        Ok(
            json!({"childId":id,"interruptionRequested":true,"note":"Completed writes remain. Inspect child evidence and Changes before any explicit follow-up."}),
        )
    }
    async fn state(
        &self,
        rt: &RuntimeContext,
        value: Value,
        cancel: &CancellationToken,
    ) -> Result<(), String> {
        rt.log.record(None, "subagent", value.clone()).await?;
        {
            let mut children = self
                .children
                .lock()
                .map_err(|_| "Subagent evidence unavailable.")?;
            if let Some(old) = children
                .iter_mut()
                .find(|v| v["childId"] == value["childId"])
            {
                *old = value.clone();
            } else {
                children.push(value.clone());
            }
        }
        forward(
            &rt.output,
            json!({"type":"subagent", "id":rt.client_id,"child":value}),
            cancel,
        )
        .await
    }
    /// Called by the owning run before its terminal event, including dropped futures.
    pub async fn finish(&self, log: &Arc<RunLog>, reason: &str) -> Result<(), String> {
        self.runtime
            .lock()
            .map_err(|_| "Subagents unavailable.")?
            .take();
        for child in self
            .controls
            .lock()
            .map_err(|_| "Subagents unavailable.")?
            .values()
        {
            if let Some(token) = child
                .running
                .lock()
                .map_err(|_| "Child unavailable.")?
                .as_ref()
            {
                token.cancel();
            }
        }
        let unfinished: Vec<_> = {
            let mut children = self
                .children
                .lock()
                .map_err(|_| "Subagent evidence unavailable.")?;
            children
                .iter_mut()
                .filter(|v| matches!(v["status"].as_str(), Some("queued" | "running")))
                .map(|v| {
                    v["status"] = json!("interrupted");
                    v["note"] = json!(reason);
                    v.clone()
                })
                .collect()
        };
        for child in unfinished {
            log.record(None, "subagent", child).await?;
        }
        Ok(())
    }
    async fn child(
        &self,
        rt: &RuntimeContext,
        task: ChildTask,
        id: String,
        serial: Arc<tokio::sync::Mutex<()>>,
        cancel: CancellationToken,
    ) -> Result<Value, String> {
        let control = self
            .controls
            .lock()
            .map_err(|_| "Child unavailable.")?
            .get(&id)
            .cloned()
            .ok_or("Child snapshot unavailable.")?;
        let parent_cancel = cancel;
        let cancel = parent_cancel.child_token();
        {
            let mut running = control.running.lock().map_err(|_| "Child unavailable.")?;
            if running.is_some() {
                return Err(
                    "Child is still running. Wait for its report before following up.".into(),
                );
            }
            *running = Some(cancel.clone());
        }
        let _running = Running(control.clone());
        let mut receipt = json!({"parentRunId":rt.log.id,"childId":id,"goal":task.goal,"scope":task.scope,
            "readOnly":task.read_only,"model":control.model,"followUps":control.followups.load(std::sync::atomic::Ordering::SeqCst),"status":"running","verified":false});
        self.state(rt, receipt.clone(), &cancel).await?;
        let mut plugins: Vec<Arc<dyn ToolPlugin>> = rt
            .tools
            .iter()
            .filter(|p| {
                matches!(
                    p.spec().name.as_str(),
                    "read_text_file" | "list_folder" | "search_text" | "read_scoped_instructions"
                ) || (!task.read_only
                    && matches!(
                        p.spec().name.as_str(),
                        "edit_text_file" | "create_text_file"
                    ))
            })
            .map(|p| {
                Arc::new(ScopedTool {
                    inner: p.clone(),
                    scope: task.scope.clone(),
                    child: format!(
                        "{id}.{}",
                        control.followups.load(std::sync::atomic::Ordering::SeqCst)
                    ),
                }) as Arc<dyn ToolPlugin>
            })
            .collect();
        if let Some(resources) = &control.resources {
            let inner: Arc<dyn ToolPlugin> = Arc::new(crate::run_journal::LoggedTool {
                inner: resources.clone(),
                log: rt.log.clone(),
                policy: None,
            });
            let inner = if let Some(hooks) = &rt.hooks {
                Arc::new(crate::lifecycle_hooks::HookedTool {
                    inner,
                    hooks: hooks.clone(),
                }) as Arc<dyn ToolPlugin>
            } else {
                inner
            };
            plugins.push(Arc::new(ChildResource {
                inner,
                child: format!(
                    "{id}.{}",
                    control.followups.load(std::sync::atomic::Ordering::SeqCst)
                ),
            }));
        }
        let budget = TaskBudget {
            model_calls: Some(rt.budget.model_limit().min(32)),
            tool_calls: Some(rt.budget.tool_limit().min(64)),
            ..rt.budget
        };
        let mut system =
            dolores_core::reset_agent_budget_note(&rt.context[0].content, rt.budget, budget);
        system.push_str(&format!("\nShared parent totals: {} model calls and {} tool operations INCLUDING parent and sibling attempts. One model call stays reserved for the parent report; concurrency can consume the shared allowance before your local cap.",rt.budget.model_limit(), rt.budget.tool_limit()));
        system.push_str(&format!("\n\nYou are a scoped child of a Dolores task, not its coordinator. Ownership: {}. Access: {}. Use only paths within that scope. No delegation, commands, external tools or permission changes. Your report is not independent verification. Summarize exact work, actual evidence and remaining gaps in at most 150 words. Do not claim tests you did not run.", task.scope, if task.read_only {"read-only"} else {"reviewed file writes"}));
        system.push_str("\nA file scope does not allow listing or searching its parent directory, including '.'. Use the assigned direct file path. Do not spend steps locating a file whose exact path was supplied. The parent owns cross-file integration and command verification.");
        if !task.read_only {
            system.push_str("\nFor a requested new file, propose create_text_file directly; its host preview checks occupancy and never replaces an existing file. If occupied, read that same file before editing.");
        }
        if let Some(resources) = &control.resources {
            system.push_str(&format!("\nFrozen selected package resources: {}. Use read_skill_resource for these exact skill/scope/packageId snapshots; it counts toward your cumulative file-operation limit. Package reads do not widen project paths or authorize scripts, commands or MCP.",serde_json::to_string(&resources.snapshot()?).map_err(|_|"Child package snapshot unavailable.")?));
        }
        let original = rt.context.last().map(|m| m.content.as_str()).unwrap_or("");
        let initial = vec![Message{role:Role::System, content:system,parts:vec![]},
            Message{role:Role::User, content:format!("Child goal:\n{}\n\nOriginal user request (background, scope still applies):\n{}", task.goal, short(original, 16*1024)),parts:vec![]}];
        let context = {
            let mut saved = control
                .context
                .lock()
                .map_err(|_| "Child context unavailable.")?;
            if saved.is_empty() {
                let prepared = dolores_core::prepare_permission_context(
                    dolores_core::prepare_agent_context_with_budget(initial, budget)?,
                    &rt.permissions,
                )?;
                let prepared = dolores_core::prepare_external_tool_context(
                    prepared,
                    &plugins.iter().map(|p| p.spec()).collect::<Vec<_>>(),
                )?;
                *saved = dolores_core::prepare_response_context(
                    prepared,
                    control.provider.request_settings().unwrap_or_default(),
                )?;
            }
            saved.clone()
        };
        let mut offset = 0;
        loop {
            let mut snapshot = self.inspect(&id, offset)?;
            snapshot["parentRunId"] = json!(rt.log.id);
            snapshot["permissionSnapshot"] = json!(rt.permissions);
            let next = snapshot["nextOffset"].as_u64();
            rt.log.record(None, "subagentContext", snapshot).await?;
            if let Some(next) = next {
                offset = next as usize;
            } else {
                break;
            }
        }
        let approval = ChildApproval {
            inner: rt.approval.clone(),
            serial,
            child: format!(
                "{id}.{}",
                control.followups.load(std::sync::atomic::Ordering::SeqCst)
            ),
        };
        let (events, mut receiver) = mpsc::channel(32);
        let run = async {
            if let Some(hooks) = &rt.hooks {
                hooks
                    .fire_owned(
                        dolores_core::lifecycle_hooks::Event::TaskStart,
                        None,
                        None,
                        Some(&id),
                        cancel.clone(),
                    )
                    .await?;
            }
            dolores_core::run_agent_with_shared_budget(
                control.provider.as_ref(),
                context,
                &plugins,
                &approval,
                events,
                cancel.clone(),
                budget,
                control.budget.clone(),
                true,
            )
            .await
        };
        let mut run = Box::pin(run);
        let mut text = ChildText::default();
        let result = loop {
            tokio::select! { biased;
                _ = parent_cancel.cancelled() => return Err(stopped()),
                _ = cancel.cancelled() => break Err(stopped()),
                result = &mut run => break result,
                Some(event) = receiver.recv() => text.event(rt, &id, event).await?,
            }
        };
        drop(run);
        while let Ok(event) = receiver.try_recv() {
            text.event(rt, &id, event).await?;
        }
        text.flush(rt, &id).await?;
        receipt["observedTools"] = json!(text.tools);
        match result {
            Ok(reply) => {
                receipt["status"] = json!(if reply.pause.is_some() {
                    "paused"
                } else if reply
                    .summary
                    .tools
                    .iter()
                    .any(|t| matches!(t.status.as_str(), "blocked" | "denied" | "error"))
                {
                    "needsReview"
                } else {
                    "reported"
                });
                receipt["pause"] = json!(reply.pause);
                receipt["answer"] = json!(short(&reply.answer, 2048));
                receipt["truncated"] = json!(reply.answer.len() > 2048);
                receipt["modelCalls"] = json!(reply.summary.model_calls);
                receipt["tools"] = json!(reply
                    .summary
                    .tools
                    .iter()
                    .map(|t| json!({"name":t.name,"target":short(&t.target,128),"status":t.status}))
                    .collect::<Vec<_>>());
                // Usage is retained separately; reports stay small enough for the parent context.
                rt.log.record(None,"subagentUsage",json!({"childId":id,"parentRunId":rt.log.id,"usageByCall":reply.summary.usage_by_call})).await?;
            }
            Err(error) => {
                receipt["status"] = json!(if cancel.is_cancelled() {
                    "interrupted"
                } else {
                    "failed"
                });
                receipt["answer"] = json!(short(&error, 512));
            }
        }
        receipt["usage"] = json!(control.budget.usage());
        // Bound the encoded report too: provider control characters can expand in JSON.
        while receipt.to_string().len() > 6 * 1024 {
            let answer = receipt["answer"].as_str().unwrap_or("");
            if answer.len() <= 1 {
                return Err("Subagent report metadata exceeded its limit.".into());
            }
            receipt["answer"] = json!(short(answer, answer.len() / 2));
            receipt["truncated"] = json!(true);
        }
        self.state(rt, receipt.clone(), &parent_cancel).await?;
        if let Some(hooks) = &rt.hooks {
            hooks
                .observe_owned(
                    dolores_core::lifecycle_hooks::Event::TaskComplete,
                    None,
                    receipt["status"].as_str(),
                    Some(&id),
                    parent_cancel.clone(),
                )
                .await;
        }
        Ok(receipt)
    }
}
#[derive(Default)]
struct ChildText {
    number: Option<usize>,
    text: String,
    truncated: bool,
    tools: Vec<Value>,
}
impl ChildText {
    async fn event(
        &mut self,
        rt: &RuntimeContext,
        id: &str,
        event: AgentEvent,
    ) -> Result<(), String> {
        match event {
            // Thinking previews belong to the child's UI, never diagnostic journals.
            AgentEvent::ModelThinking { .. } | AgentEvent::ModelFinished { .. } => Ok(()),
            AgentEvent::ModelText { number, text } => {
                if self.number != Some(number) {
                    self.flush(rt, id).await?;
                    self.number = Some(number);
                }
                if !self.truncated {
                    let remaining = 8192 - self.text.len();
                    self.text.push_str(&short(&text, remaining));
                    self.truncated = text.len() > remaining;
                }
                Ok(())
            }
            other => {
                if let AgentEvent::ToolResult { record } = &other {
                    self.tools.push(json!({"name":record.name,"target":short(&record.target,128),"status":record.status}));
                }
                self.flush(rt, id).await?;
                child_event(rt, id, other).await
            }
        }
    }
    async fn flush(&mut self, rt: &RuntimeContext, id: &str) -> Result<(), String> {
        if self.text.is_empty() && !self.truncated {
            return Ok(());
        }
        let text = std::mem::take(&mut self.text);
        let truncated = std::mem::take(&mut self.truncated);
        rt.log.record(None,"subagentEvidence",json!({"parentRunId":rt.log.id,"childId":id,
            "event":{"type":"modelText","number":self.number,"text":text,"previewTruncated":truncated}})).await
    }
}
async fn child_event(rt: &RuntimeContext, id: &str, event: AgentEvent) -> Result<(), String> {
    // Encoded JSON may expand 6x. Full tool results remain in LoggedTool / Changes.
    // Do not merge child text into the parent's public stream.
    let mut event = serde_json::to_value(event).map_err(|_| "Child evidence unavailable.")?;
    match event["type"].as_str() {
        Some("modelText") => {
            if let Some(text) = event["text"].as_str().map(str::to_owned) {
                event["text"] = json!(short(&text, 8192));
                event["previewTruncated"] = json!(text.len() > 8192);
            }
        }
        Some("toolResult") => {
            for key in ["content", "diff"] {
                if let Some(text) = event["record"][key].as_str().map(str::to_owned) {
                    event["record"][key] = json!(short(&text, 8192));
                    if text.len() > 8192 {
                        event["previewTruncated"] = json!(true);
                    }
                }
            }
        }
        _ => {}
    }
    rt.log
        .record(
            None,
            "subagentEvidence",
            json!({"parentRunId":rt.log.id,"childId":id,"event":event}),
        )
        .await
}
#[async_trait]
impl ToolPlugin for DelegateTasks {
    fn spec(&self) -> ToolSpec {
        let mut spec = spec();
        spec.description.push_str(&format!(
            " Frozen enabled models: {}. Default parent model: {}.",
            self.models.keys().cloned().collect::<Vec<_>>().join(", "),
            self.parent_model
        ));
        spec
    }
    fn prepare(&self, call: &ToolCall) -> Result<ToolRequest, String> {
        if call.name != "delegate_tasks" {
            return Err("Invalid subagent tool.".into());
        }
        let value: Value =
            serde_json::from_str(&call.arguments).map_err(|_| "Invalid subagent request.")?;
        let (target, query) = if value.get("childId").is_some() {
            let r = Supervision::parse(&call.arguments)?;
            (
                format!("Child {} · {}", r.child_id, r.action),
                serde_json::to_string(&r).map_err(|_| "Invalid child request.")?,
            )
        } else {
            let plan = parse(&call.arguments)?;
            (
                format!("{} scoped subagent(s)", plan.tasks.len()),
                serde_json::to_string(&plan).map_err(|_| "Invalid plan.")?,
            )
        };
        Ok(ToolRequest {
            call_id: call.id.clone(),
            name: call.name.clone(),
            target,
            query: Some(query),
            diff: None,
            command: None,
            mcp: None,
        })
    }
    async fn invoke(
        &self,
        request: &ToolRequest,
        cancel: CancellationToken,
    ) -> Result<String, String> {
        let query = request.query.as_deref().ok_or("Missing subagent plan.")?;
        let prepared = self.prepare(&ToolCall {
            id: request.call_id.clone(),
            name: request.name.clone(),
            arguments: query.into(),
        })?;
        if &prepared != request {
            return Err("Subagent proposal changed.".into());
        }
        if cancel.is_cancelled() {
            return Err(stopped());
        }
        let rt = self
            .runtime
            .lock()
            .map_err(|_| "Subagents unavailable.")?
            .clone()
            .ok_or("Subagents not bound to a parent run.")?;
        let serial = Arc::new(tokio::sync::Mutex::new(()));
        let value: Value = serde_json::from_str(query).map_err(|_| "Invalid child request.")?;
        if value.get("childId").is_some() {
            let r = Supervision::parse(query)?;
            match r.action.as_str() {
                "inspect" => {
                    return Ok(self
                        .inspect(&r.child_id, r.offset.unwrap_or(0))?
                        .to_string())
                }
                "interrupt" => return Ok(self.interrupt(&r.child_id)?.to_string()),
                _ => {}
            }
            let child = self
                .controls
                .lock()
                .map_err(|_| "Child unavailable.")?
                .get(&r.child_id)
                .cloned()
                .ok_or("Child is not owned by this parent run.")?;
            if child
                .running
                .lock()
                .map_err(|_| "Child unavailable.")?
                .is_some()
            {
                return Err("Wait for the child to settle before following up.".into());
            }
            let used = child.budget.usage();
            if used.model_calls >= rt.budget.model_limit().min(32)
                || used.tool_calls >= rt.budget.tool_limit().min(64)
            {
                return Err("Child cumulative allowance is exhausted. Inspect saved work; a follow-up cannot reset shared parent or child usage.".into());
            }
            // A checkpoint is data, never replayable tool calls. Original scope,
            // provider and live parent grant checks remain unchanged.
            let report = self
                .children
                .lock()
                .map_err(|_| "Child evidence unavailable.")?
                .iter()
                .find(|v| v["childId"] == r.child_id)
                .cloned()
                .ok_or("Child report unavailable.")?;
            let checkpoint=json!({"untrustedPriorReport":report,"instruction":"Retain completed effects. Re-read affected files before proposing fresh writes; never replay old tool calls."}).to_string();
            {
                let mut context = child
                    .context
                    .lock()
                    .map_err(|_| "Child context unavailable.")?;
                if context.is_empty() {
                    return Err("Child context unavailable; inspect saved evidence before starting fresh work.".into());
                }
                context.push(Message {
                    role: Role::Assistant,
                    content: checkpoint,
                    parts: vec![],
                });
                context.push(Message {
                    role: Role::User,
                    content: r.message.unwrap(),
                    parts: vec![],
                });
            }
            child
                .followups
                .fetch_add(1, std::sync::atomic::Ordering::SeqCst);
            rt.log.record(None,"subagentFollowUp",json!({"childId":r.child_id,"number":child.followups.load(std::sync::atomic::Ordering::SeqCst)})).await?;
            let result = self
                .child(&rt, child.task.clone(), r.child_id, serial, cancel)
                .await?;
            return Ok(json!({"children":[result],"verified":false,"note":"Independently inspect actual file evidence and Changes before accepting this child report."}).to_string());
        }
        let plan = parse(query)?;
        for task in &plan.tasks {
            if task
                .model
                .as_ref()
                .is_some_and(|m| m != &self.parent_model && !self.models.contains_key(m))
            {
                return Err("Child model was not enabled at parent admission. Choose a frozen enabled model; no fallback was used.".into());
            }
        }
        if self.claimed.swap(true, std::sync::atomic::Ordering::SeqCst) {
            return Err("Only one subagent batch is allowed per run.".into());
        }
        let mut tasks = plan
            .tasks
            .into_iter()
            .map(|task| (task, uuid::Uuid::new_v4().simple().to_string()))
            .collect::<Vec<_>>();
        for (task, id) in &tasks {
            let model = task
                .model
                .clone()
                .unwrap_or_else(|| self.parent_model.clone());
            let provider = if model == self.parent_model {
                rt.provider.clone()
            } else {
                self.models
                    .get(&model)
                    .ok_or("Child model unavailable.")?
                    .clone()
            };
            // The ledger reserves the parent report. Local scope is useful for
            // real implementation and remains cumulative across follow-ups.
            let local = TaskBudget {
                model_calls: Some(rt.budget.model_limit().min(32) + 1),
                tool_calls: Some(rt.budget.tool_limit().min(64)),
                ..rt.budget
            };
            self.controls
                .lock()
                .map_err(|_| "Child unavailable.")?
                .insert(
                    id.clone(),
                    Arc::new(Child {
                        resources: rt
                            .resources
                            .as_ref()
                            .map(|r| r.fork().map(Arc::new))
                            .transpose()?,
                        task: task.clone(),
                        model,
                        provider,
                        context: Mutex::new(vec![]),
                        budget: Arc::new(SharedTaskBudget::with_ledger(
                            local,
                            Arc::new(ChildLedger(rt.shared.clone())),
                        )),
                        running: Mutex::new(None),
                        followups: 0.into(),
                    }),
                );
            self.state(&rt,json!({"parentRunId":rt.log.id,"childId":id,"goal":task.goal,"scope":task.scope,"readOnly":task.read_only,"status":"queued","verified":false}),&cancel).await?;
        }
        let (first, id) = tasks.remove(0);
        let result = if let Some((second, other)) = tasks.pop() {
            let (a, b) = tokio::join!(
                self.child(&rt, first, id, serial.clone(), cancel.child_token()),
                self.child(&rt, second, other, serial, cancel.child_token())
            );
            vec![a?, b?]
        } else {
            vec![
                self.child(&rt, first, id, serial, cancel.child_token())
                    .await?,
            ]
        };
        let text = json!({"children":result,"sharedUsage":rt.shared.usage(),"verified":false,
            "note":"Child reports are not proof of goal completion. Inspect child tool evidence in Run history and file changes in Changes; parent verification remains required. Paused children need explicit continuation, never automatic replay."}).to_string();
        if text.len() > dolores_core::MAX_TOOL_BYTES {
            return Err(
                "Subagent reports exceed the bounded result limit. Inspect Run history.".into(),
            );
        }
        Ok(text)
    }
}
struct ScopedTool {
    inner: Arc<dyn ToolPlugin>,
    scope: String,
    child: String,
}
struct ChildResource {
    inner: Arc<dyn ToolPlugin>,
    child: String,
}
#[async_trait]
impl ToolPlugin for ChildResource {
    fn spec(&self) -> ToolSpec {
        self.inner.spec()
    }
    fn prepare(&self, call: &ToolCall) -> Result<ToolRequest, String> {
        let mut call = call.clone();
        let raw = call.id.clone();
        if raw.len() > 80 {
            return Err("Child call identity exceeds its limit.".into());
        }
        call.id = format!("child.{}.{}", self.child, raw);
        let mut request = self.inner.prepare(&call)?;
        request.call_id = raw;
        Ok(request)
    }
    fn discard(&self, request: &ToolRequest) {
        let mut r = request.clone();
        r.call_id = format!("child.{}.{}", self.child, r.call_id);
        self.inner.discard(&r);
    }
    async fn invoke(
        &self,
        request: &ToolRequest,
        cancel: CancellationToken,
    ) -> Result<String, String> {
        let mut r = request.clone();
        r.call_id = format!("child.{}.{}", self.child, r.call_id);
        self.inner.invoke(&r, cancel).await
    }
}
impl ScopedTool {
    fn namespaced(&self, request: &ToolRequest) -> ToolRequest {
        let mut r = request.clone();
        r.call_id = format!("child.{}.{}", self.child, r.call_id);
        r
    }
}
#[async_trait]
impl ToolPlugin for ScopedTool {
    fn spec(&self) -> ToolSpec {
        self.inner.spec()
    }
    fn prepare(&self, call: &ToolCall) -> Result<ToolRequest, String> {
        let args: Value =
            serde_json::from_str(&call.arguments).map_err(|_| "Invalid child file arguments.")?;
        if call.id.len() > 80
            || !args["path"]
                .as_str()
                .is_some_and(|p| within(&self.scope, p))
        {
            return Err("Child file request is outside its assigned scope.".into());
        }
        let mut call = call.clone();
        let raw = call.id.clone();
        call.id = format!("child.{}.{}", self.child, call.id);
        let mut request = self.inner.prepare(&call)?;
        if !within(&self.scope, &request.target) {
            self.inner.discard(&request);
            return Err("Prepared file target is outside child scope.".into());
        }
        request.call_id = raw;
        Ok(request)
    }
    fn discard(&self, request: &ToolRequest) {
        self.inner.discard(&self.namespaced(request));
    }
    async fn invoke(
        &self,
        request: &ToolRequest,
        cancel: CancellationToken,
    ) -> Result<String, String> {
        if !within(&self.scope, &request.target) || cancel.is_cancelled() {
            return Err("Child scope or task access changed.".into());
        }
        self.inner.invoke(&self.namespaced(request), cancel).await
    }
}
struct ChildApproval {
    inner: Arc<dyn ToolApproval>,
    serial: Arc<tokio::sync::Mutex<()>>,
    child: String,
}
impl ChildApproval {
    fn request(&self, r: &ToolRequest) -> ToolRequest {
        let mut r = r.clone();
        r.call_id = format!("child.{}.{}", self.child, r.call_id);
        r
    }
}
#[async_trait]
impl ToolApproval for ChildApproval {
    async fn authorize(
        &self,
        request: &ToolRequest,
        cancel: CancellationToken,
    ) -> Result<bool, String> {
        let _guard = tokio::select! { biased; _ = cancel.cancelled() => return Err(stopped()), g = self.serial.lock() => g };
        self.inner.authorize(&self.request(request), cancel).await
    }
    async fn recheck(
        &self,
        request: &ToolRequest,
        cancel: CancellationToken,
    ) -> Result<(), String> {
        self.inner.recheck(&self.request(request), cancel).await
    }
}

#[cfg(test)]
#[path = "subagent_tests.rs"]
mod tests;
