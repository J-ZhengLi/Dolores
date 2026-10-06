use crate::{forward, run_journal::RunLog, stopped};
use async_trait::async_trait;
use dolores_core::{
    AgentEvent, Message, ModelProvider, Role, SharedTaskBudget, TaskBudget, ToolApproval, ToolCall,
    ToolPlugin, ToolRequest, ToolSpec,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc;
use tokio_util::sync::CancellationToken;

pub fn spec() -> ToolSpec {
    ToolSpec {
        name: "delegate_tasks".into(),
        description: "Run one batch of 1–2 scoped subagents concurrently, using this model and SHARING this task's total model/tool/time limits. Reserve sufficient steps. Each gets a goal, relative file/folder scope and readOnly flag. Writable scopes must not overlap. Children have only scoped file tools, inherited permission checks, no commands/MCP/recursion. The parent waits and must verify reports. Stop cancels the whole batch; applied writes remain. One parent reporting call is reserved.".into(),
        parameters: json!({"type":"object","properties":{"tasks":{"type":"array","minItems":1,"maxItems":2,"items":{"type":"object","properties":{"goal":{"type":"string","maxLength":512},"scope":{"type":"string","description":"Direct relative file/folder; '.' for root","maxLength":128},"readOnly":{"type":"boolean"}},"required":["goal","scope","readOnly"],"additionalProperties":false}}},"required":["tasks"],"additionalProperties":false}),
    }
}
#[derive(Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct ChildTask {
    goal: String,
    scope: String,
    read_only: bool,
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
    pub provider: Arc<dyn ModelProvider>,
    pub context: Vec<Message>,
    pub tools: Vec<Arc<dyn ToolPlugin>>,
    pub approval: Arc<dyn ToolApproval>,
    pub shared: Arc<SharedTaskBudget>,
    pub budget: TaskBudget,
    pub log: Arc<RunLog>,
    pub output: mpsc::Sender<Value>,
    pub client_id: u64,
}
#[derive(Default)]
pub(super) struct DelegateTasks {
    runtime: Mutex<Option<RuntimeContext>>,
    children: Mutex<Vec<Value>>,
    claimed: std::sync::atomic::AtomicBool,
}
impl DelegateTasks {
    pub fn bind(&self, runtime: RuntimeContext) -> Result<(), String> {
        *self.runtime.lock().map_err(|_| "Subagents unavailable.")? = Some(runtime);
        Ok(())
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
        let mut receipt = json!({"parentRunId":rt.log.id,"childId":id,"goal":task.goal,"scope":task.scope,
            "readOnly":task.read_only,"status":"running","verified":false});
        self.state(rt, receipt.clone(), &cancel).await?;
        let plugins: Vec<Arc<dyn ToolPlugin>> = rt
            .tools
            .iter()
            .filter(|p| {
                matches!(
                    p.spec().name.as_str(),
                    "read_text_file" | "list_folder" | "search_text"
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
                    child: id.clone(),
                }) as Arc<dyn ToolPlugin>
            })
            .collect();
        let budget = TaskBudget {
            model_calls: rt.budget.model_calls.min(4),
            tool_calls: rt.budget.tool_calls.min(4),
            ..rt.budget
        };
        let mut system =
            dolores_core::reset_agent_budget_note(&rt.context[0].content, rt.budget, budget);
        system.push_str(&format!("\nShared parent totals: {} model calls and {} tool operations INCLUDING parent and sibling attempts. One model call stays reserved for the parent report; concurrency can consume the shared allowance before your local cap.",rt.budget.model_calls, rt.budget.tool_calls));
        system.push_str(&format!("\n\nYou are a scoped child of a Dolores task, not its coordinator. Ownership: {}. Access: {}. Use only paths within that scope. No delegation, commands, external tools or permission changes. Your report is not independent verification. Summarize exact work, actual evidence and remaining gaps in at most 150 words. Do not claim tests you did not run.", task.scope, if task.read_only {"read-only"} else {"reviewed file writes"}));
        system.push_str("\nA file scope does not allow listing or searching its parent directory, including '.'. Use the assigned direct file path. For a requested new file, propose create_text_file directly; its host preview checks occupancy and never replaces an existing file. If occupied, read that same file before editing. Do not spend steps locating a file whose exact path was supplied. The parent owns cross-file integration and command verification.");
        let original = rt.context.last().map(|m| m.content.as_str()).unwrap_or("");
        let context = vec![Message{role:Role::System, content:system,parts:vec![]},
            Message{role:Role::User, content:format!("Child goal:\n{}\n\nOriginal user request (background, scope still applies):\n{}", task.goal, short(original, 16*1024)),parts:vec![]}];
        let approval = ChildApproval {
            inner: rt.approval.clone(),
            serial,
            child: id.clone(),
        };
        let (events, mut receiver) = mpsc::channel(32);
        let run = dolores_core::run_agent_with_shared_budget(
            rt.provider.as_ref(),
            context,
            &plugins,
            &approval,
            events,
            cancel.clone(),
            budget,
            rt.shared.clone(),
            true,
        );
        tokio::pin!(run);
        let mut text = ChildText::default();
        let result = loop {
            tokio::select! { biased;
                _ = cancel.cancelled() => return Err(stopped()),
                result = &mut run => break result,
                Some(event) = receiver.recv() => text.event(rt, &id, event).await?,
            }
        };
        while let Some(event) = receiver.recv().await {
            text.event(rt, &id, event).await?;
        }
        text.flush(rt, &id).await?;
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
                receipt["status"] = json!("failed");
                receipt["answer"] = json!(short(&error, 512));
            }
        }
        // Bound the encoded report too: provider control characters can expand in JSON.
        while receipt.to_string().len() > 6 * 1024 {
            let answer = receipt["answer"].as_str().unwrap_or("");
            if answer.len() <= 1 {
                return Err("Subagent report metadata exceeded its limit.".into());
            }
            receipt["answer"] = json!(short(answer, answer.len() / 2));
            receipt["truncated"] = json!(true);
        }
        self.state(rt, receipt.clone(), &cancel).await?;
        Ok(receipt)
    }
}
#[derive(Default)]
struct ChildText {
    number: Option<usize>,
    text: String,
    truncated: bool,
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
        spec()
    }
    fn prepare(&self, call: &ToolCall) -> Result<ToolRequest, String> {
        if call.name != "delegate_tasks" {
            return Err("Invalid subagent tool.".into());
        }
        let plan = parse(&call.arguments)?;
        Ok(ToolRequest {
            call_id: call.id.clone(),
            name: call.name.clone(),
            target: format!("{} scoped subagent(s)", plan.tasks.len()),
            query: Some(serde_json::to_string(&plan).map_err(|_| "Invalid plan.")?),
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
        let plan = parse(request.query.as_deref().ok_or("Missing subagent plan.")?)?;
        if request.name != "delegate_tasks"
            || request.target != format!("{} scoped subagent(s)", plan.tasks.len())
            || request.diff.is_some()
            || request.command.is_some()
            || request.mcp.is_some()
        {
            return Err("Subagent proposal changed.".into());
        }
        if cancel.is_cancelled() {
            return Err(stopped());
        }
        if self.claimed.swap(true, std::sync::atomic::Ordering::SeqCst) {
            return Err("Only one subagent batch is allowed per run.".into());
        }
        let rt = self
            .runtime
            .lock()
            .map_err(|_| "Subagents unavailable.")?
            .take()
            .ok_or("Subagents not bound to a parent run.")?;
        let serial = Arc::new(tokio::sync::Mutex::new(()));
        let mut tasks = plan
            .tasks
            .into_iter()
            .map(|task| (task, uuid::Uuid::new_v4().simple().to_string()))
            .collect::<Vec<_>>();
        for (task, id) in &tasks {
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
        if call.id.len() > 88
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
            return Err("Prepared file target is outside child scope.".into());
        }
        request.call_id = raw;
        Ok(request)
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
