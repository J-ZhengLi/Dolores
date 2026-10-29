//! Bounded, on-demand MCP stdio tools. Servers are trusted external programs,
//! not sandboxed folder tools. No server is started by spec() or prepare().
use async_trait::async_trait;
use dolores_core::{
    CredentialStore, McpCallPreview, McpConnection, McpFingerprint, McpLaunch, McpTool, ToolCall,
    ToolPlugin, ToolRequest, ToolSpec,
};
use dolores_tools_command::process;
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::{
    collections::{BTreeSet, HashMap},
    fs::File,
    io::{Read, Write},
    path::{Path, PathBuf},
    sync::{mpsc, Arc, Mutex},
    thread,
    time::{Duration, Instant},
};
use tokio_util::sync::CancellationToken;

pub mod configured;
pub mod credentials;
pub mod remote;
use credentials::Credentials;
mod isolation;
pub use isolation::ExecutionHost;
use isolation::Reviewed;

const SECONDS: u64 = 30;
const MAX_FRAME: usize = 64 * 1024;
const MAX_WIRE: usize = 256 * 1024;
const VERSIONS: &[&str] = &["2025-11-25", "2025-06-18", "2025-03-26", "2024-11-05"];
#[cfg(test)]
mod tests;
#[derive(Clone, Debug)]
pub struct Inspection {
    pub launch: McpLaunch,
    pub fingerprints: Vec<McpFingerprint>,
    pub protocol_version: String,
    pub server_name: String,
    pub server_version: String,
    pub tools: Vec<McpTool>,
}
fn check(cancel: &CancellationToken, deadline: Instant) -> Result<(), String> {
    if cancel.is_cancelled() {
        return Err("MCP operation stopped. External effects may remain.".into());
    }
    if Instant::now() >= deadline {
        return Err("MCP operation exceeded its 30-second limit.".into());
    }
    Ok(())
}
fn fingerprints(
    root: &Path,
    launch: &McpLaunch,
    cancel: &CancellationToken,
    deadline: Instant,
) -> Result<Vec<McpFingerprint>, String> {
    launch.validate()?;
    let root = root
        .canonicalize()
        .map_err(|_| "Working folder is unavailable.")?;
    let mut files = vec![(PathBuf::from(&launch.executable), 128 * 1024 * 1024)];
    let mut seen = BTreeSet::from([PathBuf::from(&launch.executable)]);
    for arg in &launch.args {
        let path = if Path::new(arg).is_absolute() {
            PathBuf::from(arg)
        } else {
            root.join(arg)
        };
        if path.is_file() && seen.insert(path.clone()) {
            files.push((path, 4 * 1024 * 1024));
        }
    }
    let mut result = vec![];
    for (path, maximum) in files {
        check(cancel, deadline)?;
        let mut file = File::open(&path)
            .map_err(|_| "MCP launch file is unavailable. Review its executable and arguments.")?;
        let info = file
            .metadata()
            .map_err(|_| "MCP launch file is unavailable.")?;
        if !info.is_file() || info.len() > maximum {
            return Err(
                "MCP executable must be within 128 MiB; direct file arguments within 4 MiB.".into(),
            );
        }
        let mut hasher = Sha256::new();
        let mut bytes = [0; 64 * 1024];
        let mut total = 0u64;
        loop {
            check(cancel, deadline)?;
            let count = file
                .read(&mut bytes)
                .map_err(|_| "MCP launch file could not be read.")?;
            if count == 0 {
                break;
            }
            total += count as u64;
            if total > maximum {
                return Err("MCP launch file exceeds its size limit.".into());
            }
            hasher.update(&bytes[..count]);
        }
        result.push(McpFingerprint {
            path: path.to_str().ok_or("MCP paths need valid Unicode.")?.into(),
            sha256: format!("{:x}", hasher.finalize()),
        });
    }
    Ok(result)
}
pub fn verify_launch(
    root: &Path,
    inspection: &Inspection,
    cancel: &CancellationToken,
) -> Result<(), String> {
    if fingerprints(
        root,
        &inspection.launch,
        cancel,
        Instant::now() + Duration::from_secs(SECONDS),
    )? != inspection.fingerprints
    {
        return Err("MCP launch files changed. Inspect and review the server again.".into());
    }
    Ok(())
}
type WriteJob = (Vec<u8>, mpsc::SyncSender<Result<(), ()>>);
struct Session {
    process: process::Running,
    input: Option<mpsc::SyncSender<WriteJob>>,
    writer: Option<thread::JoinHandle<()>>,
    output: File,
    error: File,
    buffer: Vec<u8>,
    wire: usize,
    stderr: usize,
    notifications: usize,
    id: u64,
    credentials: Credentials,
    deadline: Instant,
    cancel: CancellationToken,
    instructions: String,
}
impl Session {
    fn start(
        root: &Path,
        launch: &McpLaunch,
        cancel: CancellationToken,
        deadline: Instant,
        credentials: Credentials,
        reviewed: &Reviewed,
    ) -> Result<(Self, String, String, String), String> {
        credentials.validate_launch(launch)?;
        let mut environment = process::environment();
        environment.extend(credentials.0.iter().map(|(n, v)| (n.into(), v.into())));
        Self::start_environment(
            root,
            launch,
            cancel,
            deadline,
            credentials,
            reviewed,
            &environment,
        )
    }
    fn start_environment(
        root: &Path,
        launch: &McpLaunch,
        cancel: CancellationToken,
        deadline: Instant,
        credentials: Credentials,
        reviewed: &Reviewed,
        environment: &[(String, String)],
    ) -> Result<(Self, String, String, String), String> {
        check(&cancel, deadline)?;
        launch.validate()?;
        let (process, mut input, output, error) =
            reviewed.spawn(root, launch, environment, &cancel)?;
        let (sender, receiver) = mpsc::sync_channel::<WriteJob>(1);
        // The owner can kill the process while a non-reading server blocks stdin.
        let writer = thread::spawn(move || {
            while let Ok((bytes, reply)) = receiver.recv() {
                let result = input
                    .write_all(&bytes)
                    .and_then(|()| input.flush())
                    .map_err(|_| ());
                let failed = result.is_err();
                let _ = reply.send(result);
                if failed {
                    break;
                }
            }
        });
        let mut session = Self {
            process,
            input: Some(sender),
            writer: Some(writer),
            output,
            error,
            buffer: vec![],
            wire: 0,
            stderr: 0,
            notifications: 0,
            id: 0,
            credentials,
            deadline,
            cancel,
            instructions: String::new(),
        };
        let result = session.request("initialize", json!({"protocolVersion":VERSIONS[0],"capabilities":{},"clientInfo":{"name":"Dolores","version":env!("CARGO_PKG_VERSION")}}))?;
        session.credentials.reject_metadata(&result)?;
        if let Some(text) = result.get("instructions").and_then(Value::as_str) {
            if text.len() > 64 * 1024 {
                return Err("MCP server instructions exceed 64 KiB.".into());
            }
            session.instructions = text.into();
        }
        let version = result["protocolVersion"]
            .as_str()
            .filter(|s| VERSIONS.contains(s))
            .ok_or("MCP server uses an unsupported protocol version.")?
            .to_owned();
        if !result["capabilities"]["tools"].is_object() {
            return Err("MCP server does not advertise tools.".into());
        }
        let name = bounded_info(&result["serverInfo"]["name"], false)?;
        let release = bounded_info(&result["serverInfo"]["version"], true)?;
        // Server instructions/icons are untrusted metadata, never host instructions or I/O.
        session.send(json!({"jsonrpc":"2.0","method":"notifications/initialized"}))?;
        Ok((session, version, name, release))
    }
    fn send(&mut self, value: Value) -> Result<(), String> {
        let mut bytes =
            serde_json::to_vec(&value).map_err(|_| "MCP request could not be encoded.")?;
        if bytes.len() > 8192 {
            return Err("MCP request exceeds 8 KiB.".into());
        }
        bytes.push(b'\n');
        let (reply, done) = mpsc::sync_channel(1);
        self.input
            .as_ref()
            .ok_or("MCP connection is closed.")?
            .try_send((bytes, reply))
            .map_err(|_| "MCP input is unavailable.")?;
        loop {
            check(&self.cancel, self.deadline)?;
            self.drain_error()?;
            match done.try_recv() {
                Ok(Ok(())) => return Ok(()),
                Ok(Err(())) | Err(mpsc::TryRecvError::Disconnected) => {
                    return Err("MCP server closed its input.".into())
                }
                Err(mpsc::TryRecvError::Empty) => thread::sleep(Duration::from_millis(5)),
            }
        }
    }
    fn drain_error(&mut self) -> Result<(), String> {
        let mut bytes = [0; 2048];
        match process::read_available(&mut self.error, &mut bytes) {
            Ok(count) => {
                self.stderr += count;
                if self.stderr > MAX_FRAME {
                    return Err("MCP server exceeded its diagnostic output limit.".into());
                }
            }
            Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {}
            Err(_) => return Err("MCP diagnostic stream failed.".into()),
        }
        Ok(())
    }
    fn next(&mut self) -> Result<Value, String> {
        loop {
            check(&self.cancel, self.deadline)?;
            self.drain_error()?;
            if let Some(end) = self.buffer.iter().position(|&b| b == b'\n') {
                let remainder = self.buffer.split_off(end + 1);
                let line = std::mem::replace(&mut self.buffer, remainder);
                let value: Value = serde_json::from_slice(&line[..end])
                    .map_err(|_| "MCP server sent invalid JSON-RPC text.")?;
                if value["jsonrpc"] != "2.0" || !value.is_object() {
                    return Err("MCP server sent an invalid JSON-RPC envelope.".into());
                }
                return Ok(value);
            }
            let mut bytes = [0; 4096];
            match process::read_available(&mut self.output, &mut bytes) {
                Ok(0) => return Err("MCP server exited or closed its output.".into()),
                Ok(count) => {
                    self.wire += count;
                    self.buffer.extend_from_slice(&bytes[..count]);
                    if self.wire > MAX_WIRE || self.buffer.len() > MAX_FRAME {
                        return Err("MCP server exceeded its protocol output limit.".into());
                    }
                }
                Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                    thread::sleep(Duration::from_millis(5))
                }
                Err(_) => return Err("MCP output stream failed.".into()),
            }
        }
    }
    fn request(&mut self, method: &str, params: Value) -> Result<Value, String> {
        self.id += 1;
        let id = self.id;
        self.send(json!({"jsonrpc":"2.0","id":id,"method":method,"params":params}))?;
        loop {
            let value = self.next()?;
            if let Some(method) = value["method"].as_str() {
                self.notifications += 1;
                if self.notifications > 64 {
                    return Err("MCP server exceeded its message limit.".into());
                }
                if method == "notifications/tools/list_changed" {
                    return Err(
                        "MCP tool list changed. Inspect and review the server again.".into(),
                    );
                }
                if let Some(request_id) = value.get("id") {
                    if !request_id.is_i64() && !request_id.is_u64() && !request_id.is_string() {
                        return Err("MCP server sent an invalid request ID.".into());
                    }
                    let response = if method == "ping" {
                        json!({"jsonrpc":"2.0","id":request_id,"result":{}})
                    } else {
                        // No sampling, roots, elicitation, resources or task execution is granted.
                        json!({"jsonrpc":"2.0","id":request_id,"error":{"code":-32601,"message":"Client capability is not supported"}})
                    };
                    self.send(response)?;
                }
                continue;
            }
            if value["id"] != id || value.get("result").is_some() == value.get("error").is_some() {
                return Err("MCP server sent an unmatched or invalid response.".into());
            }
            if value.get("error").is_some() {
                return Err(
                    "MCP server rejected the request; its private diagnostic text was not shared."
                        .into(),
                );
            }
            return Ok(value["result"].clone());
        }
    }
    fn tools(&mut self) -> Result<Vec<McpTool>, String> {
        self.catalog(false)
    }
    fn catalog(&mut self, configured: bool) -> Result<Vec<McpTool>, String> {
        let mut tools = vec![];
        let mut cursor = None;
        let mut cursors = BTreeSet::new();
        let mut pages = 0;
        loop {
            pages += 1;
            if configured {
                self.wire = 0;
                self.stderr = 0;
                self.notifications = 0;
            }
            if !configured && pages > 4 {
                return Err("MCP catalog exceeds four pages. Use a smaller server catalog.".into());
            }
            let page = self.request(
                "tools/list",
                cursor.as_ref().map_or(json!({}), |c| json!({"cursor":c})),
            )?;
            self.credentials.reject_metadata(&page)?;
            for item in page["tools"]
                .as_array()
                .ok_or("MCP tools/list did not contain a tool array.")?
            {
                if item["execution"]["taskSupport"] == "required" {
                    continue;
                }
                let tool = McpTool {
                    name: item["name"]
                        .as_str()
                        .ok_or("MCP tool name is missing.")?
                        .into(),
                    description: item["description"].as_str().unwrap_or("").into(),
                    input_schema: item["inputSchema"].clone(),
                };
                if configured {
                    // These schemas are discovered through a paged gateway, not
                    // registered in the model's fixed inline tool catalog.
                    if tool.name.is_empty()
                        || tool.name.len() > 128
                        || tool.name.chars().any(char::is_control)
                        || tool.description.contains('\0')
                        || !tool.input_schema.is_object()
                        || tool.input_schema["type"] != "object"
                    {
                        return Err(
                            "Configured MCP tool needs a valid name and object input schema."
                                .into(),
                        );
                    }
                } else {
                    tool.validate()?;
                }
                if tools.iter().any(|old: &McpTool| old.name == tool.name) {
                    return Err("MCP tool names are duplicated.".into());
                }
                tools.push(tool);
                if (!configured && tools.len() > 32)
                    || serde_json::to_vec(&tools).map_or(true, |v| {
                        v.len() > if configured { 1024 * 1024 } else { 32 * 1024 }
                    })
                {
                    return Err(if configured {
                        "Configured MCP catalog exceeds its 1 MiB transport allocation."
                    } else {
                        "MCP catalog exceeds 32 tools or 32 KiB. Use a smaller server catalog."
                    }
                    .into());
                }
            }
            match page.get("nextCursor") {
                None => {
                    tools.sort_by(|a, b| a.name.cmp(&b.name));
                    return Ok(tools);
                }
                Some(Value::String(next))
                    if !next.is_empty() && next.len() <= 1024 && cursors.insert(next.clone()) =>
                {
                    cursor = Some(next.clone())
                }
                _ => return Err("MCP pagination cursor is invalid or repeated.".into()),
            }
        }
    }
    fn close(&mut self) -> Result<(), String> {
        self.input.take();
        // Kill before joining: a server may stop reading after its last reply.
        let cleaned = self.process.stop_checked();
        if let Some(writer) = self.writer.take() {
            let _ = writer.join();
        }
        if cleaned {
            Ok(())
        } else {
            Err("MCP process cleanup is incomplete. Inspect Commands and Retry cleanup; the tool was not replayed.".into())
        }
    }
}
impl Drop for Session {
    fn drop(&mut self) {
        self.input.take();
        self.process.stop();
        if let Some(writer) = self.writer.take() {
            let _ = writer.join();
        }
    }
}
fn bounded_info(value: &Value, allow_empty: bool) -> Result<String, String> {
    let text = value.as_str().ok_or("MCP server information is missing.")?;
    if (!allow_empty && text.is_empty()) || text.len() > 128 || text.chars().any(char::is_control) {
        return Err("MCP server information exceeds its text limits.".into());
    }
    Ok(text.into())
}
pub fn inspect(
    root: &Path,
    launch: McpLaunch,
    cancel: CancellationToken,
) -> Result<Inspection, String> {
    inspect_with_credentials(root, launch, cancel, Credentials::empty())
}
pub fn inspect_with_credentials(
    root: &Path,
    launch: McpLaunch,
    cancel: CancellationToken,
    credentials: Credentials,
) -> Result<Inspection, String> {
    inspect_with_host(root, launch, cancel, credentials, ExecutionHost::default())
}
pub fn inspect_with_host(
    root: &Path,
    launch: McpLaunch,
    cancel: CancellationToken,
    credentials: Credentials,
    host: ExecutionHost,
) -> Result<Inspection, String> {
    credentials.validate_launch(&launch)?;
    let root = root
        .canonicalize()
        .map_err(|_| "Working folder is unavailable.")?;
    let deadline = Instant::now() + Duration::from_secs(SECONDS);
    let reviewed = host.review(&root, &launch, &cancel)?;
    let hashes = fingerprints(&root, &launch, &cancel, deadline)?;
    let (mut session, protocol_version, server_name, server_version) = Session::start(
        &root,
        &launch,
        cancel.clone(),
        deadline,
        credentials,
        &reviewed,
    )?;
    let tools = session.tools()?;
    session.close()?;
    if tools.is_empty() {
        return Err("MCP server has no supported ordinary tools.".into());
    }
    if fingerprints(&root, &launch, &cancel, deadline)? != hashes {
        return Err("MCP launch files changed during inspection. Inspect again.".into());
    }
    Ok(Inspection {
        launch,
        fingerprints: hashes,
        protocol_version,
        server_name,
        server_version,
        tools,
    })
}
struct CancelOnDrop(CancellationToken);
impl Drop for CancelOnDrop {
    fn drop(&mut self) {
        self.0.cancel();
    }
}
pub struct McpPlugin {
    root: PathBuf,
    connection: McpConnection,
    index: usize,
    plans: Mutex<HashMap<String, (ToolRequest, Reviewed)>>,
    vault: Option<Arc<dyn CredentialStore>>,
    host: ExecutionHost,
}
pub fn plugins(root: &Path, connection: McpConnection) -> Result<Vec<Arc<dyn ToolPlugin>>, String> {
    plugins_inner(root, connection, None, ExecutionHost::default())
}
pub fn plugins_with_credentials(
    root: &Path,
    connection: McpConnection,
    vault: Arc<dyn CredentialStore>,
) -> Result<Vec<Arc<dyn ToolPlugin>>, String> {
    plugins_inner(root, connection, Some(vault), ExecutionHost::default())
}
pub fn plugins_with_host(
    root: &Path,
    connection: McpConnection,
    vault: Arc<dyn CredentialStore>,
    host: ExecutionHost,
) -> Result<Vec<Arc<dyn ToolPlugin>>, String> {
    plugins_inner(root, connection, Some(vault), host)
}
fn plugins_inner(
    root: &Path,
    connection: McpConnection,
    vault: Option<Arc<dyn CredentialStore>>,
    host: ExecutionHost,
) -> Result<Vec<Arc<dyn ToolPlugin>>, String> {
    connection.validate()?;
    let root = root
        .canonicalize()
        .map_err(|_| "Working folder is unavailable.")?;
    if !connection.enabled {
        return Ok(vec![]);
    }
    if connection.remote.is_some() {
        return remote::approved::plugins(root, connection, vault);
    }
    Ok((0..connection.tools.len())
        .map(|index| {
            Arc::new(McpPlugin {
                root: root.clone(),
                connection: connection.clone(),
                index,
                plans: Mutex::new(HashMap::new()),
                vault: vault.clone(),
                host: host.clone(),
            }) as Arc<dyn ToolPlugin>
        })
        .collect())
}
#[async_trait]
impl ToolPlugin for McpPlugin {
    fn spec(&self) -> ToolSpec {
        self.connection.specs().remove(self.index)
    }
    fn prepare(&self, call: &ToolCall) -> Result<ToolRequest, String> {
        dolores_core::validate_call(call)?;
        if call.name != self.spec().name {
            return Err("MCP tool alias is unavailable.".into());
        }
        let arguments: Value =
            serde_json::from_str(&call.arguments).map_err(|_| "MCP arguments must be JSON.")?;
        if !arguments.is_object() || call.arguments.contains("\u{0}") {
            return Err("MCP arguments must be a JSON object.".into());
        }
        let tool = &self.connection.tools[self.index];
        let reviewed = self.host.review(
            &self.root,
            self.connection.local_launch()?,
            &CancellationToken::new(),
        )?;
        let request = ToolRequest {
            call_id: call.id.clone(),
            name: call.name.clone(),
            target: format!("{} / {}", self.connection.label(), tool.name),
            query: None,
            diff: None,
            command: None,
            mcp: Some(Box::new(McpCallPreview {
                connection_id: self.connection.id.clone(),
                server: self.connection.label().to_owned(),
                tool: tool.name.clone(),
                arguments: arguments.to_string(),
                revision: self.connection.revision,
                isolation: reviewed.preview(),
                remote_endpoint: None,
                credential_names: self
                    .connection
                    .credentials
                    .iter()
                    .map(|b| b.name.clone())
                    .collect(),
            })),
        };
        let mut plans = self
            .plans
            .lock()
            .map_err(|_| "MCP approval is unavailable.")?;
        if plans.len() >= dolores_core::MAX_TOOL_CALLS || plans.contains_key(&call.id) {
            return Err("MCP approval limit reached.".into());
        }
        plans.insert(call.id.clone(), (request.clone(), reviewed));
        Ok(request)
    }
    fn discard(&self, request: &ToolRequest) {
        if let Ok(mut plans) = self.plans.lock() {
            plans.remove(&request.call_id);
        }
    }
    async fn invoke(
        &self,
        request: &ToolRequest,
        cancel: CancellationToken,
    ) -> Result<String, String> {
        let (plan, reviewed) = self
            .plans
            .lock()
            .map_err(|_| "MCP approval is unavailable.")?
            .remove(&request.call_id)
            .ok_or("MCP approval expired. Request a fresh tool call.")?;
        if plan != *request {
            return Err("MCP call changed after approval. Nothing was run.".into());
        }
        let (root, connection) = (self.root.clone(), self.connection.clone());
        let index = self.index;
        let vault = self.vault.clone();
        let arguments = plan.mcp.ok_or("MCP approval has no arguments.")?.arguments;
        let cancel = cancel.child_token();
        let _guard = CancelOnDrop(cancel.clone());
        tokio::task::spawn_blocking(move || {
            invoke_reviewed(root, connection, index, arguments, cancel, vault, reviewed)
        })
        .await
        .map_err(|_| "MCP worker failed.")?
    }
}

fn invoke_reviewed(
    root: PathBuf,
    connection: McpConnection,
    index: usize,
    arguments: String,
    cancel: CancellationToken,
    vault: Option<Arc<dyn CredentialStore>>,
    reviewed: Reviewed,
) -> Result<String, String> {
    let deadline = Instant::now() + Duration::from_secs(SECONDS);
    if root.canonicalize().ok().as_ref() != Some(&root) {
        return Err("Working folder changed. Review the MCP connection again.".into());
    }
    if fingerprints(&root, connection.local_launch()?, &cancel, deadline)?
        != connection.fingerprints
    {
        return Err("MCP launch files changed. Inspect and review the server again.".into());
    }
    let credentials = if connection.credentials.is_empty() {
        Credentials::empty()
    } else {
        credentials::resolve(
            &root,
            &connection,
            vault.as_ref().ok_or(credentials::VAULT_ERROR)?.as_ref(),
        )?
    };
    let (mut session, protocol, server, release) = Session::start(
        &root,
        connection.local_launch()?,
        cancel.clone(),
        deadline,
        credentials,
        &reviewed,
    )?;
    let catalog = session.tools()?;
    if protocol != connection.protocol_version
        || server != connection.server_name
        || release != connection.server_version
        || connection
            .tools
            .iter()
            .any(|selected| !catalog.contains(selected))
    {
        return Err(
            "Reviewed MCP tool metadata changed. Inspect and review the server again.".into(),
        );
    }
    let arguments: Value =
        serde_json::from_str(&arguments).map_err(|_| "MCP arguments are invalid.")?;
    let result = session.request(
        "tools/call",
        json!({"name":connection.tools[index].name,"arguments":arguments}),
    )?;
    let (mut text, is_error) = text_content(&result)?;
    text = session.credentials.redact(&text);
    if text.len() > 8192 {
        return Err("MCP redacted result exceeds 8 KiB.".into());
    }
    let mut result = json!({"text":text,"isError":is_error});
    if let Some(isolation) = reviewed.preview() {
        result["execution"] = json!("isolated; Windows LPAC; network denied");
        result["isolation"] = json!(isolation);
    }
    let encoded = result.to_string();
    if encoded.len() > dolores_core::MAX_TOOL_BYTES {
        return Err("MCP result exceeds its encoded limit.".into());
    }
    session.close()?;
    check(&cancel, deadline)?;
    Ok(encoded)
}
fn text_content(result: &Value) -> Result<(String, bool), String> {
    let mut text = String::new();
    for item in result["content"]
        .as_array()
        .ok_or("MCP result has no content array.")?
    {
        if item["type"] != "text" {
            return Err("This MCP connection accepts text results only; resource/image/audio content was not loaded.".into());
        }
        let part = item["text"]
            .as_str()
            .ok_or("MCP text content is invalid.")?;
        if !text.is_empty() {
            text.push('\n');
        }
        text.push_str(part);
        if text.len() > 8192 || text.contains('\0') {
            return Err("MCP result exceeds 8 KiB or contains NUL.".into());
        }
    }
    let is_error = match result.get("isError") {
        None => false,
        Some(Value::Bool(value)) => *value,
        _ => return Err("MCP result has an invalid error flag.".into()),
    };
    Ok((text, is_error))
}
