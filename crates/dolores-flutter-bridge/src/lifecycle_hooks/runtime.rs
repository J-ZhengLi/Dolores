use super::*;
use async_trait::async_trait;

pub(crate) struct Runtime {
    root: PathBuf,
    configuration: Configuration,
    log: Arc<RunLog>,
    approval: Arc<dyn ToolApproval>,
    output: tokio::sync::mpsc::Sender<Value>,
    id: u64,
    allowance: Mutex<(usize, u64)>,
}
#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct Reply {
    allow: Option<bool>,
    message: String,
}
impl Runtime {
    pub fn for_session(
        store: &dyn SessionStore,
        session: &str,
        log: Arc<RunLog>,
        approval: Arc<dyn ToolApproval>,
        output: tokio::sync::mpsc::Sender<Value>,
        id: u64,
    ) -> Result<Option<Arc<Self>>, String> {
        let root = scope(store, session)?;
        let configuration =
            store.lifecycle_hooks(root.to_str().ok_or("Unsupported hook folder.")?)?;
        Self::new(root, configuration, log, approval, output, id)
    }
    pub fn new(
        root: PathBuf,
        configuration: Configuration,
        log: Arc<RunLog>,
        approval: Arc<dyn ToolApproval>,
        output: tokio::sync::mpsc::Sender<Value>,
        id: u64,
    ) -> Result<Option<Arc<Self>>, String> {
        configuration.validate()?;
        if configuration.hooks.is_empty() {
            return Ok(None);
        }
        Ok(Some(Arc::new(Self {
            root,
            configuration,
            log,
            approval,
            output,
            id,
            allowance: Mutex::new((0, 0)),
        })))
    }
    async fn receipt(
        &self,
        hook: &ReviewedHook,
        event: Event,
        status: &str,
        message: &str,
        command: Option<Value>,
    ) -> Result<(), String> {
        let data = json!({"hook":hook.hook.name,"event":event,"mode":hook.hook.mode,"revision":self.configuration.revision,"scriptSha256":hook.script_sha256,"status":status,"message":message,"commandOutcome":command,"untrusted":true});
        self.log.record(None, "hookResult", data.clone()).await?;
        self.output
            .try_send(json!({"type":"hookReceipt","id":self.id,"receipt":data}))
            .ok();
        Ok(())
    }
    pub async fn fire(
        &self,
        event: Event,
        tool: Option<&str>,
        outcome: Option<&str>,
        cancel: CancellationToken,
    ) -> Result<(), String> {
        for hook in self
            .configuration
            .hooks
            .iter()
            .filter(|h| h.hook.event == event)
        {
            if cancel.is_cancelled() {
                return Err(crate::stopped());
            }
            let request = ToolRequest {
                call_id: uuid::Uuid::new_v4().to_string(),
                name: "lifecycle_hook".into(),
                target: hook.hook.name.clone(),
                query: None,
                diff: None,
                command: None,
                mcp: None,
            };
            self.approval.recheck(&request, cancel.clone()).await?;
            let permitted = {
                let mut allowance = self
                    .allowance
                    .lock()
                    .map_err(|_| "Hook accounting unavailable.")?;
                if allowance.0 >= 24 || allowance.1 + hook.hook.timeout_seconds > 30 {
                    false
                } else {
                    allowance.0 += 1;
                    allowance.1 += hook.hook.timeout_seconds;
                    true
                }
            };
            let mut command = None;
            let result = if permitted {
                self.invoke(hook, event, tool, outcome, cancel.clone(), &mut command)
                    .await
            } else {
                Err("Run hook allowance reached (24 calls / 30 seconds). Inspect hooks before explicit retry.".into())
            };
            let (status, message) = match &result {
                Ok(reply) if hook.hook.mode == Mode::Blocking && reply.allow == Some(false) => {
                    ("blocked", reply.message.clone())
                }
                Ok(reply) => ("completed", reply.message.clone()),
                Err(error) => ("error", error.clone()),
            };
            // Post-effect evidence failures must never erase the original result.
            // Retain a live warning if durable recording fails; caller preserves it.
            self.receipt(hook, event, status, &message, command).await?;
            if hook.hook.mode == Mode::Blocking && (result.is_err() || status == "blocked") {
                return Err(format!("Lifecycle hook '{}' refused this action. Inspect Run history and Settings → Advanced → Lifecycle hooks; review or disable it before an explicit retry. Hook effects may remain.",hook.hook.name));
            }
            self.approval.recheck(&request, cancel.clone()).await?;
        }
        Ok(())
    }
    async fn invoke(
        &self,
        hook: &ReviewedHook,
        event: Event,
        tool: Option<&str>,
        outcome: Option<&str>,
        cancel: CancellationToken,
        command: &mut Option<Value>,
    ) -> Result<Reply, String> {
        if script_hash(&self.root, &hook.hook)? != hook.script_sha256 {
            return Err("Hook script changed. Review exact bytes again or disable it.".into());
        }
        let (runner, request) = prepare(
            &self.root,
            &hook.hook,
            json!({"schema":1,"event":event,"tool":tool,"outcome":outcome}),
        )?;
        if request
            .command
            .as_ref()
            .is_none_or(|p| p.executable != hook.executable)
        {
            runner.discard(&request);
            return Err("Hook runtime changed. Review again or disable it.".into());
        }
        self.log.record(None,"hookIntent",json!({"hook":hook.hook.name,"event":event,"revision":self.configuration.revision,"reviewed":hook,"certainty":"Effects unknown until hookResult; never replay on reopen."})).await?;
        self.approval.recheck(&request, cancel.clone()).await?;
        let raw = runner.invoke(&request, cancel).await?;
        let value: Value =
            serde_json::from_str(&raw).map_err(|_| "Hook command receipt is invalid.")?;
        *command = Some(value.clone());
        if value["exitCode"] != 0
            || value["reason"] != "completed"
            || value["truncated"] != false
            || value["lossyUtf8"] != false
            || value["outputError"] != false
            || value["cleanupComplete"] != true
        {
            return Err("Hook failed, timed out, exceeded output limits or has unconfirmed cleanup. Inspect files and hook settings; nothing was retried.".into());
        }
        let stdout = value["stdout"]
            .as_str()
            .ok_or("Hook output is unavailable.")?;
        let reply:Reply=serde_json::from_str(stdout).map_err(|_|"Hook must return one JSON object with message and optional allow. Fix the script and review again.")?;
        if reply.message.len() > 512
            || reply
                .message
                .chars()
                .any(|c| c.is_control() && !matches!(c, '\n' | '\t'))
            || hook.hook.mode == Mode::Blocking && reply.allow.is_none()
        {
            return Err(
                "Hook result is malformed or exceeds 512 bytes. Fix and review the script.".into(),
            );
        }
        Ok(reply)
    }
    pub async fn observe(
        &self,
        event: Event,
        tool: Option<&str>,
        outcome: Option<&str>,
        cancel: CancellationToken,
    ) {
        if let Err(error) = self.fire(event, tool, outcome, cancel).await {
            let _=self.output.try_send(json!({"type":"hookReceipt","id":self.id,"receipt":{"hook":"Host","status":"error","message":error,"untrusted":true}}));
        }
    }
}
pub(crate) struct HookedTool {
    pub inner: Arc<dyn ToolPlugin>,
    pub hooks: Arc<Runtime>,
}
#[async_trait]
impl ToolPlugin for HookedTool {
    fn spec(&self) -> dolores_core::ToolSpec {
        self.inner.spec()
    }
    fn prepare(&self, call: &ToolCall) -> Result<ToolRequest, String> {
        self.inner.prepare(call)
    }
    fn discard(&self, request: &ToolRequest) {
        self.inner.discard(request)
    }
    fn image_results(&self) -> Vec<dolores_core::AttachmentRef> {
        self.inner.image_results()
    }
    async fn invoke(
        &self,
        request: &ToolRequest,
        cancel: CancellationToken,
    ) -> Result<String, String> {
        if let Err(error) = self
            .hooks
            .fire(Event::BeforeTool, Some(&request.name), None, cancel.clone())
            .await
        {
            self.inner.discard(request);
            return Err(error);
        }
        let result = self.inner.invoke(request, cancel.clone()).await;
        self.hooks
            .observe(
                Event::AfterTool,
                Some(&request.name),
                Some(if result.is_ok() { "returned" } else { "error" }),
                cancel,
            )
            .await;
        result
    }
}
pub(crate) struct HookedApproval {
    pub inner: Arc<dyn ToolApproval>,
    pub hooks: Arc<Runtime>,
}
#[async_trait]
impl ToolApproval for HookedApproval {
    fn is_waiting(&self) -> bool {
        self.inner.is_waiting()
    }
    fn steering_pending(&self) -> Result<bool, String> {
        self.inner.steering_pending()
    }
    fn take_steering(&self) -> Result<Vec<dolores_core::SteeringInput>, String> {
        self.inner.take_steering()
    }
    fn acknowledge_steering(&self) -> Result<(), String> {
        self.inner.acknowledge_steering()
    }
    fn context_recovery_available(&self) -> bool {
        self.inner.context_recovery_available()
    }
    async fn recheck(&self, r: &ToolRequest, c: CancellationToken) -> Result<(), String> {
        self.inner.recheck(r, c).await
    }
    async fn authorize(&self, r: &ToolRequest, c: CancellationToken) -> Result<bool, String> {
        self.inner.authorize(r, c).await
    }
    async fn compact_context(
        &self,
        m: &[dolores_core::AgentMessage],
        c: CancellationToken,
    ) -> Result<Option<dolores_core::context_recovery::ContextCheckpoint>, String> {
        self.hooks
            .fire(Event::BeforeCompaction, None, None, c.clone())
            .await?;
        self.inner.compact_context(m, c).await
    }
}
