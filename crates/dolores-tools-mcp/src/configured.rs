//! Run-owned MCP stdio sessions loaded from an explicitly selected setup profile.
//! Does not persist a manual review, environment value or server credential.
use super::*;

pub struct Options {
    pub name: String,
    pub root: PathBuf,
    pub launch: dolores_core::CommandSpec,
    pub environment: Vec<(String, String)>,
    pub redactions: Vec<String>,
    pub startup_timeout: Duration,
    pub tool_timeout: Duration,
    pub enabled_tools: Option<Vec<String>>,
    pub disabled_tools: Vec<String>,
}
pub struct Connected {
    pub tools: Vec<Arc<dyn ToolPlugin>>,
    pub instructions: String,
}
struct Owner {
    name: String,
    input: Mutex<Option<mpsc::Sender<Job>>>,
    worker: Mutex<Option<thread::JoinHandle<()>>>,
    lifetime: CancellationToken,
}
struct Job {
    tool: String,
    arguments: Value,
    cancel: CancellationToken,
    reply: tokio::sync::oneshot::Sender<Result<String, String>>,
}
struct StartupGuard(Option<CancellationToken>);
impl Drop for StartupGuard {
    fn drop(&mut self) {
        if let Some(c) = self.0.take() {
            c.cancel();
        }
    }
}
impl Drop for Owner {
    fn drop(&mut self) {
        self.lifetime.cancel();
        if let Ok(input) = self.input.get_mut() {
            input.take();
        }
        if let Ok(worker) = self.worker.get_mut() {
            if let Some(worker) = worker.take() {
                let _ = worker.join();
            }
        }
    }
}
struct Tool {
    owner: Arc<Owner>,
    metadata: McpTool,
    alias: String,
    plans: Mutex<HashMap<String, ToolRequest>>,
}
pub async fn connect(options: Options, cancel: CancellationToken) -> Result<Connected, String> {
    let lifetime = cancel.child_token();
    let mut guard = StartupGuard(Some(lifetime.clone()));
    let (input, receiver) = mpsc::channel::<Job>();
    let (ready_tx, ready_rx) = tokio::sync::oneshot::channel();
    let name = options.name.clone();
    let allowed = options.enabled_tools.clone();
    let disabled = options.disabled_tools.clone();
    let worker_cancel = lifetime.clone();
    let worker = thread::spawn(move || {
        // Native process handles and the session stay on one owner thread.
        // Only protocol requests/results cross the channel.
        let startup = (|| {
            let search = options
                .environment
                .iter()
                .find(|(k, _)| k.eq_ignore_ascii_case("PATH"))
                .map(|(_, v)| std::ffi::OsString::from(v))
                .unwrap_or_default();
            let resolved =
                dolores_tools_command::resolve_launcher(&options.root, options.launch, &search)?;
            let launch = McpLaunch {
                label: options.name,
                executable: resolved.program,
                args: resolved.args,
                isolation: None,
            };
            let credentials = Credentials(
                options
                    .redactions
                    .into_iter()
                    .enumerate()
                    .filter(|(_, v)| !v.is_empty())
                    .map(|(i, v)| (format!("CONFIG_{i}"), v))
                    .collect(),
            );
            let deadline = Instant::now() + options.startup_timeout;
            let (mut session, _, _, _) = Session::start_environment(
                &options.root,
                &launch,
                worker_cancel.clone(),
                deadline,
                credentials,
                &Reviewed::Native,
                &options.environment,
            )?;
            let catalog = session.catalog(true)?;
            Ok::<_, String>((session, catalog))
        })();
        let (mut session, catalog) = match startup {
            Ok(v) => v,
            Err(e) => {
                let _ = ready_tx.send(Err(e));
                return;
            }
        };
        if ready_tx
            .send(Ok((catalog, session.instructions.clone())))
            .is_err()
        {
            return;
        }
        while !worker_cancel.is_cancelled() {
            let job = match receiver.recv_timeout(Duration::from_millis(25)) {
                Ok(job) => job,
                Err(mpsc::RecvTimeoutError::Timeout) => continue,
                Err(_) => break,
            };
            session.deadline = Instant::now() + options.tool_timeout;
            session.cancel = job.cancel;
            session.wire = 0;
            session.stderr = 0;
            session.notifications = 0;
            let result = session
                .request(
                    "tools/call",
                    json!({"name":job.tool,"arguments":job.arguments}),
                )
                .and_then(|v| {
                    let mut text = String::new();
                    for item in v["content"]
                        .as_array()
                        .ok_or("MCP result has no content array.")?
                    {
                        if item["type"] != "text" {
                            return Err(
                                "Configured MCP currently supports text results only.".into()
                            );
                        }
                        if !text.is_empty() {
                            text.push('\n');
                        }
                        text.push_str(item["text"].as_str().ok_or("MCP text is invalid.")?);
                    }
                    let is_error = v
                        .get("isError")
                        .map_or(Some(false), Value::as_bool)
                        .ok_or("MCP error flag is invalid.")?;
                    Ok(
                        json!({"text":session.credentials.redact(&text),"isError":is_error})
                            .to_string(),
                    )
                });
            let failed = result.is_err();
            let _ = job.reply.send(result);
            if failed {
                break;
            }
        }
        let _ = session.close();
    });
    let (catalog, instructions) = tokio::select! {biased;
        _=cancel.cancelled()=>return Err("MCP startup stopped.".into()),
        result=ready_rx=>result.map_err(|_|"Configured MCP startup worker failed.")??,
    };
    let owner = Arc::new(Owner {
        name,
        input: Mutex::new(Some(input)),
        worker: Mutex::new(Some(worker)),
        lifetime,
    });
    guard.0.take();
    let mut aliases = BTreeSet::new();
    let tools = catalog
        .into_iter()
        .filter(|t| {
            allowed.as_ref().is_none_or(|names| names.contains(&t.name))
                && !disabled.contains(&t.name)
        })
        .map(|metadata| {
            let alias = alias(&owner.name, &metadata.name);
            if !aliases.insert(alias.clone()) {
                return Err("Configured MCP tool aliases collided.".into());
            }
            Ok(Arc::new(Tool {
                owner: owner.clone(),
                metadata,
                alias,
                plans: Mutex::new(HashMap::new()),
            }) as Arc<dyn ToolPlugin>)
        })
        .collect::<Result<Vec<_>, String>>()?;
    Ok(Connected {
        tools,
        instructions,
    })
}
/// Stable, collision-resistant names preserve readable server/tool identities.
pub fn alias(server: &str, tool: &str) -> String {
    let clean = |s: &str| {
        s.chars()
            .map(|c| {
                if c.is_ascii_alphanumeric() || c == '_' {
                    c
                } else {
                    '_'
                }
            })
            .collect::<String>()
    };
    let raw = format!("mcp__{server}__{tool}");
    let normal = format!("mcp__{}__{}", clean(server), clean(tool));
    if raw == normal && normal.len() <= 64 {
        normal
    } else {
        let digest = format!("{:x}", Sha256::digest(raw.as_bytes()));
        format!("{}__{}", &normal[..normal.len().min(45)], &digest[..16])
    }
}
#[async_trait]
impl ToolPlugin for Tool {
    fn spec(&self) -> ToolSpec {
        ToolSpec {
            name: self.alias.clone(),
            description: self.metadata.description.clone(),
            parameters: self.metadata.input_schema.clone(),
        }
    }
    fn prepare(&self, call: &ToolCall) -> Result<ToolRequest, String> {
        dolores_core::validate_call(call)?;
        if call.name != self.alias {
            return Err("Configured MCP tool is unavailable.".into());
        }
        let arguments: Value =
            serde_json::from_str(&call.arguments).map_err(|_| "MCP arguments need JSON.")?;
        if !arguments.is_object() {
            return Err("MCP arguments must be an object.".into());
        }
        let request = ToolRequest {
            call_id: call.id.clone(),
            name: call.name.clone(),
            target: format!("{} / {}", self.owner.name, self.metadata.name),
            query: None,
            diff: None,
            command: None,
            mcp: Some(Box::new(McpCallPreview {
                connection_id: {
                    let hash = Sha256::digest(self.owner.name.as_bytes());
                    let mut bytes = [0; 16];
                    bytes.copy_from_slice(&hash[..16]);
                    uuid::Uuid::from_bytes(bytes).to_string()
                },
                server: self.owner.name.clone(),
                tool: self.metadata.name.clone(),
                arguments: arguments.to_string(),
                revision: 1,
                credential_names: vec![],
                isolation: None,
                remote_endpoint: None,
            })),
        };
        let mut plans = self
            .plans
            .lock()
            .map_err(|_| "MCP preparation unavailable.")?;
        if plans.contains_key(&call.id) {
            return Err("Duplicate MCP call ID.".into());
        }
        plans.insert(call.id.clone(), request.clone());
        Ok(request)
    }
    fn discard(&self, request: &ToolRequest) {
        if let Ok(mut p) = self.plans.lock() {
            p.remove(&request.call_id);
        }
    }
    async fn invoke(
        &self,
        request: &ToolRequest,
        cancel: CancellationToken,
    ) -> Result<String, String> {
        let planned = self
            .plans
            .lock()
            .map_err(|_| "MCP preparation unavailable.")?
            .remove(&request.call_id)
            .ok_or("MCP call expired.")?;
        if planned != *request {
            return Err("MCP input changed after approval. Nothing ran.".into());
        }
        let tool = self.metadata.name.clone();
        let arguments: Value = serde_json::from_str(&planned.mcp.unwrap().arguments)
            .map_err(|_| "MCP arguments invalid.")?;
        let token = self.owner.lifetime.child_token();
        let _guard = CancelOnDrop(token.clone());
        if cancel.is_cancelled() || token.is_cancelled() {
            return Err("MCP call stopped.".into());
        }
        let (reply, result) = tokio::sync::oneshot::channel();
        self.owner
            .input
            .lock()
            .map_err(|_| "MCP session unavailable.")?
            .as_ref()
            .ok_or("MCP session ended.")?
            .send(Job {
                tool,
                arguments,
                cancel: token,
                reply,
            })
            .map_err(|_| {
                "MCP session ended. Start a new task to reconnect; the action was not replayed."
            })?;
        tokio::select! {biased;
            _=cancel.cancelled()=>Err("MCP call stopped.".into()),
            result=result=>result.map_err(|_|"MCP worker ended; action effects may remain.")?,
        }
    }
}
