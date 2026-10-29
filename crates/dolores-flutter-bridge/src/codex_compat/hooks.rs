use super::{
    config::{Hook, Snapshot},
    process::{self, Shell},
};
use dolores_core::{ToolCall, ToolPlugin, ToolRequest, ToolSpec};
use serde_json::{json, Value};
use std::{
    collections::BTreeMap,
    path::PathBuf,
    sync::{Arc, Mutex},
};
use tokio_util::sync::CancellationToken;

pub(crate) struct Runtime {
    root: PathBuf,
    home: PathBuf,
    hooks: Vec<Hook>,
    shell: Shell,
    session: String,
    turn: String,
    model: String,
    permission: String,
    log: Option<Arc<crate::run_journal::RunLog>>,
    output: tokio::sync::mpsc::Sender<Value>,
    id: u64,
    serial: tokio::sync::Mutex<()>,
}
#[derive(Default)]
pub(crate) struct Outcome {
    pub input: Option<Value>,
    pub context: String,
    pub blocked: Option<String>,
}
pub(crate) struct Identity {
    pub session: String,
    pub turn: String,
    pub model: String,
    pub permission: String,
}
impl Runtime {
    pub fn new(
        snapshot: &Snapshot,
        identity: Identity,
        log: Option<Arc<crate::run_journal::RunLog>>,
        output: tokio::sync::mpsc::Sender<Value>,
        id: u64,
    ) -> Result<Option<Arc<Self>>, String> {
        let (Some(home), Some(root)) = (&snapshot.home, &snapshot.root) else {
            return Ok(None);
        };
        if snapshot.hooks.is_empty() {
            return Ok(None);
        }
        let shell =
            Shell::detect(root).map_err(|e| format!("Codex hook shell unavailable: {e}"))?;
        Ok(Some(Arc::new(Self {
            root: root.clone(),
            home: home.clone(),
            hooks: snapshot.hooks.clone(),
            shell,
            session: identity.session,
            turn: identity.turn,
            model: identity.model,
            permission: identity.permission,
            log,
            output,
            id,
            serial: tokio::sync::Mutex::new(()),
        })))
    }
    async fn record(&self, kind: &str, event: &str, data: Value) -> Result<(), String> {
        if let Some(log) = &self.log {
            log.record(None, kind, data.clone()).await?;
        }
        if kind == "codexHookResult" {
            self.output.try_send(json!({"type":"hookReceipt","id":self.id,"receipt":{"hook":"Codex setup","event":event,"status":data["status"],"message":data["message"]}})).ok();
        }
        Ok(())
    }
    pub async fn fire(
        &self,
        event: &str,
        matched: &str,
        fields: Value,
        cancel: CancellationToken,
    ) -> Result<Outcome, String> {
        let _serial = tokio::select! {biased;_=cancel.cancelled()=>return Err(crate::stopped()),guard=self.serial.lock()=>guard};
        let mut outcome = Outcome::default();
        let mut input = json!({"session_id":self.session,"turn_id":self.turn,"transcript_path":null,"cwd":self.root,"hook_event_name":event,"model":self.model,"permission_mode":self.permission});
        input
            .as_object_mut()
            .unwrap()
            .extend(fields.as_object().cloned().unwrap_or_default());
        for hook in self.hooks.iter().filter(|h| {
            h.event == event
                && (matches!(event, "UserPromptSubmit" | "Stop") || h.matcher.is_match(matched))
        }) {
            if cancel.is_cancelled() {
                return Err(crate::stopped());
            }
            self.record("codexHookIntent",event,json!({"event":event,"source":hook.source,"originalInput":input,"certainty":"Hook effects unknown until result; never replay."})).await?;
            let result = process::run(
                self.root.clone(),
                self.home.clone(),
                self.shell.clone(),
                hook.command.clone(),
                input.clone(),
                hook.timeout,
                cancel.clone(),
            )
            .await;
            if cancel.is_cancelled() {
                return Err(crate::stopped());
            }
            let parsed = match result {
                Ok(result) if result.code == 2 && event == "Stop" => Err(
                    "Stop-driven continuation is not supported; the completed answer is retained."
                        .into(),
                ),
                Ok(result) if result.code == 2 => Ok(Outcome {
                    blocked: Some(result.stderr.trim().chars().take(2048).collect()),
                    ..Default::default()
                }),
                Ok(result) if result.code == 0 => parse(event, &result.stdout),
                Ok(result) => Err(format!(
                    "Hook exited with code {}. Inspect its source.",
                    result.code
                )),
                Err(e) => Err(e),
            };
            match parsed {
                Ok(next) => {
                    self.record("codexHookResult",event,json!({"event":event,"source":hook.source,"status":if next.blocked.is_some(){"blocked"}else{"completed"},"effectiveInput":next.input,"message":next.blocked.as_deref().unwrap_or("Hook completed.")})).await?;
                    if let Some(next_input) = next.input {
                        input["tool_input"] = next_input.clone();
                        outcome.input = Some(next_input);
                    }
                    if !next.context.is_empty() {
                        outcome.context.push_str(&next.context);
                        outcome.context.push('\n');
                    }
                    if next.blocked.is_some() {
                        outcome.blocked = next.blocked;
                        break;
                    }
                }
                Err(e) => {
                    self.record(
                        "codexHookResult",
                        event,
                        json!({"event":event,"source":hook.source,"status":"error","message":e}),
                    )
                    .await?;
                    outcome
                        .context
                        .push_str(&format!("\nCodex hook diagnostic: {e}"));
                }
            }
        }
        // Context stays bounded in model inputs; the complete response is not an
        // executable proposal. Diagnostics remain available through receipts.
        let mut end = outcome.context.len().min(4000);
        while !outcome.context.is_char_boundary(end) {
            end -= 1;
        }
        outcome.context.truncate(end);
        Ok(outcome)
    }
    pub fn wrap(self: &Arc<Self>, tools: Vec<Arc<dyn ToolPlugin>>) -> Vec<Arc<dyn ToolPlugin>> {
        tools
            .into_iter()
            .map(|inner| {
                Arc::new(Hooked {
                    inner,
                    runtime: self.clone(),
                    plans: Mutex::new(BTreeMap::new()),
                }) as Arc<dyn ToolPlugin>
            })
            .collect()
    }
    pub fn guidance(&self) -> String {
        format!("\nCodex command hooks are active. run_command additionally accepts {{command:<shell command>}} using shell {}. Use command form for shell syntax and Bash-compatible hooks. Native program/args remain literal and match run_command hooks; they are never joined into a shell string. Rewritten commands still require normal task access.\n",self.shell.program)
    }
}
fn parse(event: &str, text: &str) -> Result<Outcome, String> {
    let text = text.trim();
    if text.is_empty() {
        return Ok(Outcome::default());
    }
    let value: Value = match serde_json::from_str(text) {
        Ok(v) => v,
        Err(_) if matches!(event, "SessionStart" | "UserPromptSubmit") => {
            return Ok(Outcome {
                context: text.into(),
                ..Default::default()
            })
        }
        Err(_) => return Err("Hook output was not a JSON object.".into()),
    };
    if !value.is_object() {
        return Err("Hook output must be an object.".into());
    }
    let specific = &value["hookSpecificOutput"];
    if specific.get("hookEventName").is_some_and(|v| v != event) {
        return Err("Hook output event does not match its input.".into());
    }
    let mut context = value["systemMessage"].as_str().unwrap_or("").to_owned();
    if let Some(text) = specific["additionalContext"].as_str() {
        context.push('\n');
        context.push_str(text);
    }
    if specific.get("updatedMCPToolOutput").is_some() || value.get("suppressOutput").is_some() {
        return Err("Hook output uses unsupported suppressOutput or updatedMCPToolOutput.".into());
    }
    let mut outcome = Outcome {
        context,
        ..Default::default()
    };
    if value["decision"] == "block"
        || (event == "PreToolUse" && specific["permissionDecision"] == "deny")
    {
        outcome.blocked = Some(
            value["reason"]
                .as_str()
                .or_else(|| specific["permissionDecisionReason"].as_str())
                .unwrap_or("Hook blocked this operation.")
                .into(),
        );
    }
    if value["continue"] == false {
        if event == "PreToolUse" {
            return Err(
                "continue:false is unsupported for PreToolUse; use permissionDecision:deny.".into(),
            );
        }
        outcome.blocked = Some(
            value["stopReason"]
                .as_str()
                .unwrap_or("Hook stopped this event.")
                .into(),
        );
    }
    if let Some(updated) = specific.get("updatedInput") {
        if event != "PreToolUse"
            || specific["permissionDecision"] != "allow"
            || !updated.is_object()
        {
            return Err(
                "PreToolUse updatedInput requires permissionDecision:allow and object input."
                    .into(),
            );
        }
        outcome.input = Some(updated.clone());
    }
    if specific["permissionDecision"] == "ask" {
        return Err(
            "Codex permissionDecision:ask is unsupported; normal task access remains in force."
                .into(),
        );
    }
    if event == "Stop" && outcome.blocked.is_some() {
        return Err("Stop-driven continuation is not supported by this adapter. The completed answer and effects remain saved; no new task was started.".into());
    }
    Ok(outcome)
}
struct Prepared {
    outer: ToolRequest,
    inner: ToolRequest,
    input: Value,
    matched: String,
    context: String,
}
struct Hooked {
    inner: Arc<dyn ToolPlugin>,
    runtime: Arc<Runtime>,
    plans: Mutex<BTreeMap<String, Prepared>>,
}
#[async_trait::async_trait]
impl ToolPlugin for Hooked {
    fn spec(&self) -> ToolSpec {
        let mut spec = self.inner.spec();
        if spec.name == "run_command" {
            spec.description.push_str(&self.runtime.guidance());
            spec.parameters["properties"]["command"] = json!({"type":"string","description":"Shell command in the shell described above. Omit program/args when using command."});
            spec.parameters.as_object_mut().unwrap().remove("required");
            spec.parameters["oneOf"] =
                json!([{"required":["command"]},{"required":["program","args"]}]);
        }
        spec
    }
    fn image_results(&self) -> Vec<dolores_core::AttachmentRef> {
        self.inner.image_results()
    }
    fn prepare(&self, _: &ToolCall) -> Result<ToolRequest, String> {
        Err("Codex hooks need cancelable preparation.".into())
    }
    async fn prepare_for_run(
        &self,
        call: &ToolCall,
        cancel: CancellationToken,
    ) -> Result<ToolRequest, String> {
        dolores_core::validate_call(call)?;
        let original: Value =
            serde_json::from_str(&call.arguments).map_err(|_| "Codex tool input must be JSON.")?;
        let shell = call.name == "run_command" && original.get("command").is_some();
        let matched = if shell {
            "Bash".to_owned()
        } else if call.name == "codex_mcp" && original["action"] == "call" {
            original["tool"].as_str().unwrap_or("codex_mcp").into()
        } else {
            call.name.clone()
        };
        let hook_input = if call.name == "codex_mcp" && original["action"] == "call" {
            original["arguments"].clone()
        } else {
            original.clone()
        };
        let outcome = self
            .runtime
            .fire(
                "PreToolUse",
                &matched,
                json!({"tool_name":matched,"tool_use_id":call.id,"tool_input":hook_input}),
                cancel.clone(),
            )
            .await?;
        if let Some(reason) = outcome.blocked {
            return Err(format!("Codex hook blocked this action: {reason}"));
        }
        let mut effective = if let Some(input) = outcome.input {
            if call.name == "codex_mcp" && original["action"] == "call" {
                let mut full = original.clone();
                full["arguments"] = input;
                full
            } else {
                input
            }
        } else {
            original.clone()
        };
        let post_input = if call.name == "codex_mcp" && effective["action"] == "call" {
            effective["arguments"].clone()
        } else {
            effective.clone()
        };
        if shell {
            let command = effective["command"].as_str().ok_or(
                "Codex hook blocked invalid rewritten shell input: command must be a string.",
            )?;
            if effective.get("program").is_some() || effective.get("args").is_some() {
                return Err("Codex hook blocked mixed shell and argv input.".into());
            }
            let launch = self.runtime.shell.command(command);
            effective
                .as_object_mut()
                .ok_or("Shell input must be an object.")?
                .remove("command");
            effective["program"] = json!(launch.program);
            effective["args"] = json!(launch.args);
        }
        let effective_call = ToolCall {
            id: call.id.clone(),
            name: call.name.clone(),
            arguments: effective.to_string(),
        };
        let inner = self.inner.prepare(&effective_call)?;
        if cancel.is_cancelled() {
            self.inner.discard(&inner);
            return Err(crate::stopped());
        }
        self.runtime.record("codexHookInput","PreToolUse",json!({"callId":call.id,"tool":matched,"originalInput":original,"effectiveInput":post_input})).await.inspect_err(|_|{self.inner.discard(&inner);})?;
        let mut plans = self
            .plans
            .lock()
            .map_err(|_| "Codex prepared call unavailable.")?;
        if plans.contains_key(&call.id) {
            self.inner.discard(&inner);
            return Err("Duplicate prepared Codex call.".into());
        }
        plans.insert(
            call.id.clone(),
            Prepared {
                outer: inner.clone(),
                inner: inner.clone(),
                input: post_input,
                matched,
                context: outcome.context,
            },
        );
        Ok(inner)
    }
    fn discard(&self, r: &ToolRequest) {
        if let Ok(mut plans) = self.plans.lock() {
            if let Some(p) = plans.remove(&r.call_id) {
                self.inner.discard(&p.inner);
            }
        }
    }
    async fn invoke(&self, r: &ToolRequest, cancel: CancellationToken) -> Result<String, String> {
        let prepared = self
            .plans
            .lock()
            .map_err(|_| "Codex prepared call unavailable.")?
            .remove(&r.call_id)
            .ok_or("Codex prepared call expired.")?;
        if prepared.outer != *r {
            self.inner.discard(&prepared.inner);
            return Err("Codex prepared input changed after approval.".into());
        }
        if cancel.is_cancelled() {
            self.inner.discard(&prepared.inner);
            return Err(crate::stopped());
        }
        let result = self.inner.invoke(&prepared.inner, cancel.clone()).await;
        if cancel.is_cancelled() {
            return result;
        }
        let post=self.runtime.fire("PostToolUse",&prepared.matched,json!({"tool_name":prepared.matched,"tool_use_id":r.call_id,"tool_input":prepared.input,"tool_response":result.as_ref().map(|v|json!(v)).unwrap_or_else(|e|json!({"error":e}))}),cancel).await;
        let mut feedback = prepared.context;
        match post {
            Ok(outcome) => {
                feedback.push_str(&outcome.context);
                if let Some(reason) = outcome.blocked {
                    feedback.push_str(&format!(
                        "\nPostToolUse feedback: {reason}. The tool already ran; do not replay it."
                    ));
                }
            }
            Err(e) => feedback.push_str(&format!(
                "\nPostToolUse evidence error: {e}. The tool already ran; do not replay it."
            )),
        }
        if feedback.is_empty() {
            return result;
        }
        match result {
            Ok(mut text) => {
                if let Ok(mut value) = serde_json::from_str::<Value>(&text) {
                    if let Some(object) = value.as_object_mut() {
                        // Command/MCP receipts remain parseable by status and
                        // recovery code after adding model-visible hook context.
                        object.insert("codexHookContext".into(), json!(feedback));
                        let encoded = value.to_string();
                        if encoded.len() <= dolores_core::MAX_TOOL_BYTES {
                            return Ok(encoded);
                        }
                        return Ok(text);
                    }
                }
                let extra = format!("\n\nCodex hook context:\n{feedback}");
                let mut end = extra
                    .len()
                    .min(dolores_core::MAX_TOOL_BYTES.saturating_sub(text.len()));
                while !extra.is_char_boundary(end) {
                    end -= 1;
                }
                text.push_str(&extra[..end]);
                Ok(text)
            }
            Err(error) => Err(format!("{error}\nCodex hook context:\n{feedback}")),
        }
    }
}
