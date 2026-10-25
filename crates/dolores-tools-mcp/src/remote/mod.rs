//! Streamable HTTP transport; the host owns persistence and foreground authority.
//! A caller must obtain host authorization before creating a remote operation.
use dolores_core::McpTool;
use reqwest::{
    header::{HeaderMap, HeaderValue, ACCEPT, CONTENT_TYPE, WWW_AUTHENTICATE},
    Client, Response, Url,
};
use serde_json::{json, Value};
use std::{fmt, time::Duration};
use tokio::time::Instant;
use tokio_util::sync::CancellationToken;
pub(crate) mod approved;
pub mod credentials;
mod headers;
mod lifecycle;
pub mod oauth;
#[cfg(test)]
mod tests;
mod wire;
pub use lifecycle::CleanupOwner;
pub fn plugins_owned(
    root: std::path::PathBuf,
    connection: dolores_core::McpConnection,
    vault: Option<std::sync::Arc<dyn dolores_core::CredentialStore>>,
    owner: std::sync::Arc<CleanupOwner>,
) -> Result<Vec<std::sync::Arc<dyn dolores_core::ToolPlugin>>, String> {
    connection.validate()?;
    if !connection.enabled {
        return Ok(vec![]);
    }
    let root = root
        .canonicalize()
        .map_err(|_| "Working folder unavailable.")?;
    approved::plugins_owned(root, connection, vault, Some(owner))
}

const FRAME: usize = 64 * 1024;
const WIRE: usize = 256 * 1024;
const MODERN: &str = "2026-07-28";
const LEGACY: &[&str] = &["2025-11-25", "2025-06-18", "2025-03-26"];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    Endpoint,
    Credential,
    Arguments,
    HeaderSchema,
    Network,
    Redirect,
    Http(u16),
    Authorization(u16),
    Protocol,
    Version,
    LegacyProbe,
    SessionExpired,
    Cancelled,
    Deadline,
    FrameLimit,
    WireLimit,
    Notifications,
    Catalog,
    MetadataSecret,
    Rpc(i64),
    UnsupportedInteraction,
}
/// Raw server errors and challenge text are intentionally absent from Debug/Display.
pub struct Failure {
    pub kind: Kind,
    challenge: Option<String>,
}
fn fail(kind: Kind) -> Failure {
    Failure {
        kind,
        challenge: None,
    }
}
impl Failure {
    /// Untrusted, bounded authentication data for the OAuth discovery layer.
    /// Never put this directly in a model prompt or a user-facing diagnostic.
    pub fn authentication_challenge(&self) -> Option<&str> {
        self.challenge.as_deref()
    }
}
impl fmt::Debug for Failure {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(self, f)
    }
}
impl fmt::Display for Failure {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let text=match self.kind {
            Kind::Endpoint=>"Choose an HTTPS MCP endpoint without URL credentials or fragments. Plain HTTP is available only for loopback endpoints.",
            Kind::Credential=>"The MCP bearer credential is invalid or appears in the endpoint. Use the separate authentication field.",
            Kind::Arguments=>"MCP arguments must be an object within 4 KiB and match any header-mapped primitive values.",
            Kind::HeaderSchema=>"This tool has invalid HTTP header annotations. Inspect another supported tool or ask the server owner to fix its schema.",
            Kind::Authorization(401)=>"Remote MCP authentication expired or is required. Reconnect and inspect again; no tool call was retried.",
            Kind::Authorization(_)=>"Remote MCP access was refused. Review the account and scopes, then reconnect explicitly.",
            Kind::SessionExpired=>"The remote MCP session expired. Inspect a fresh connection before another call; earlier effects may remain.",
            Kind::Redirect=>"The MCP endpoint redirected. Review the final endpoint explicitly; credentials were not forwarded.",
            Kind::Version=>"The server's MCP revision is unsupported or changed. Inspect the connection and choose a supported Streamable HTTP revision.",
            Kind::Cancelled=>"MCP operation stopped. External effects may remain; nothing was retried.",
            Kind::Deadline=>"MCP operation exceeded its 30-second limit. Inspect before another call; external effects may remain.",
            Kind::FrameLimit=>"MCP response exceeded the 64 KiB message limit.",
            Kind::WireLimit=>"MCP operation exceeded the 256 KiB response limit.",
            Kind::Notifications=>"MCP operation exceeded its notification or request count limit.",
            Kind::Catalog=>"MCP discovery exceeded 32 tools, four pages or 32 KiB. Use a smaller server catalog.",
            Kind::MetadataSecret=>"The MCP server exposed a configured credential in metadata. Nothing was shared.",
            Kind::UnsupportedInteraction=>"This MCP tool requires an unsupported client interaction. No automatic follow-up or replay was sent.",
            Kind::Network=>"The remote MCP connection failed. Check connection health before another call; earlier effects may remain.",
            Kind::Http(_)=>"The remote MCP endpoint refused the request. Inspect connection health before retrying explicitly.",
            Kind::Protocol|Kind::LegacyProbe|Kind::Rpc(_)=>"The remote MCP response was invalid or rejected. Inspect the server and its reviewed tools; no call was retried.",
        };
        f.write_str(text)
    }
}
impl std::error::Error for Failure {}

pub fn endpoint(text: &str) -> Result<Url, Failure> {
    dolores_core::validate_mcp_endpoint(text).map_err(|_| fail(Kind::Endpoint))
}
fn contains(value: &Value, secret: &str) -> bool {
    match value {
        Value::String(s) => s.contains(secret),
        Value::Object(v) => v
            .iter()
            .any(|(k, v)| k.contains(secret) || contains(v, secret)),
        Value::Array(v) => v.iter().any(|v| contains(v, secret)),
        _ => false,
    }
}
fn redact(value: &mut Value, secret: &str) -> Result<(), Failure> {
    match value {
        Value::String(s) => *s = s.replace(secret, "[redacted]"),
        Value::Array(v) => {
            for child in v {
                redact(child, secret)?;
            }
        }
        Value::Object(v) => {
            if v.keys().any(|k| k.contains(secret)) {
                return Err(fail(Kind::MetadataSecret));
            }
            for child in v.values_mut() {
                redact(child, secret)?;
            }
        }
        _ => {}
    };
    Ok(())
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ServerInfo {
    pub protocol_version: String,
    pub name: Option<String>,
    pub version: Option<String>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ExcludedTool {
    pub name: String,
    pub reason: &'static str,
}

/// One inspection/invocation lifetime. Does not retry or persist a remote session.
pub struct RemoteClient {
    cleanup_owner: Option<(std::sync::Arc<CleanupOwner>, String)>,
    client: Client,
    url: Url,
    bearer: Option<String>,
    session: Option<String>,
    protocol: String,
    cancel: CancellationToken,
    deadline: Instant,
    prefix: String,
    next: u32,
    wire: usize,
    notifications: usize,
    server: Option<ServerInfo>,
    catalog: Vec<McpTool>,
    pub excluded: Vec<ExcludedTool>,
}
impl RemoteClient {
    /// Sends an ephemeral credential to its explicitly chosen target without
    /// exposing its plaintext through the host's review/response structures.
    pub async fn connect_target(
        target: &dolores_core::McpRemoteTarget,
        credential: Option<&credentials::BearerCredential>,
        pinned: Option<&str>,
        cancel: CancellationToken,
    ) -> Result<Self, Failure> {
        Self::connect_target_until(
            target,
            credential,
            pinned,
            cancel,
            Instant::now() + Duration::from_secs(30),
        )
        .await
    }
    /// Shares an earlier operation deadline with credential resolution. A caller
    /// cannot extend the client's normal 30-second ceiling through this method.
    pub async fn connect_target_until(
        target: &dolores_core::McpRemoteTarget,
        credential: Option<&credentials::BearerCredential>,
        pinned: Option<&str>,
        cancel: CancellationToken,
        deadline: Instant,
    ) -> Result<Self, Failure> {
        target.validate().map_err(|_| fail(Kind::Endpoint))?;
        match (target.authentication, credential) {
            (dolores_core::McpRemoteAuthentication::None, None) => {}
            (
                dolores_core::McpRemoteAuthentication::Bearer
                | dolores_core::McpRemoteAuthentication::Oauth,
                Some(key),
            ) => key
                .validate_target(target)
                .map_err(|_| fail(Kind::Credential))?,
            _ => return Err(fail(Kind::Credential)),
        }
        Self::open(
            &target.endpoint,
            credential.map(|c| c.expose()),
            pinned,
            cancel,
            deadline.min(Instant::now() + Duration::from_secs(30)),
        )
        .await
    }
    pub async fn connect(
        url: &str,
        bearer: Option<&str>,
        pinned: Option<&str>,
        cancel: CancellationToken,
    ) -> Result<Self, Failure> {
        Self::open(
            url,
            bearer,
            pinned,
            cancel,
            Instant::now() + Duration::from_secs(30),
        )
        .await
    }
    async fn open(
        url: &str,
        bearer: Option<&str>,
        pinned: Option<&str>,
        cancel: CancellationToken,
        deadline: Instant,
    ) -> Result<Self, Failure> {
        Self::open_owned(url, bearer, pinned, cancel, deadline, None).await
    }
    async fn open_owned(
        url: &str,
        bearer: Option<&str>,
        pinned: Option<&str>,
        cancel: CancellationToken,
        deadline: Instant,
        cleanup_owner: Option<(std::sync::Arc<CleanupOwner>, String)>,
    ) -> Result<Self, Failure> {
        let url = endpoint(url)?;
        if bearer.is_some_and(|s| {
            s.is_empty()
                || s.len() > 4096
                || !s.bytes().all(|b| (0x21..=0x7e).contains(&b))
                || url.as_str().contains(s)
                || url.query_pairs().any(|(_, v)| v.contains(s))
        }) {
            return Err(fail(Kind::Credential));
        }
        if pinned.is_some_and(|v| v != MODERN && !LEGACY.contains(&v)) {
            return Err(fail(Kind::Version));
        }
        let client = http_client(&url, Client::builder())?;
        let mut this = Self {
            cleanup_owner,
            client,
            url,
            bearer: bearer.map(str::to_owned),
            session: None,
            protocol: pinned.unwrap_or(MODERN).into(),
            cancel,
            deadline,
            prefix: uuid::Uuid::new_v4().to_string(),
            next: 0,
            wire: 0,
            notifications: 0,
            server: None,
            catalog: vec![],
            excluded: vec![],
        };
        let metadata = if this.modern() {
            match this
                .rpc(
                    "server/discover",
                    json!({}),
                    HeaderMap::new(),
                    pinned.is_none(),
                )
                .await
            {
                Ok(value) => value,
                Err(error) if pinned.is_none() && error.kind == Kind::LegacyProbe => {
                    this.protocol = LEGACY[0].into();
                    this.initialize(pinned).await?
                }
                Err(e) => return Err(e),
            }
        } else {
            this.initialize(pinned).await?
        };
        this.check_metadata(&metadata)?;
        if !metadata["capabilities"]["tools"].is_object() {
            return Err(fail(Kind::Protocol));
        }
        let info = if this.modern() {
            if !metadata["supportedVersions"]
                .as_array()
                .is_some_and(|v| v.iter().any(|v| v == MODERN))
            {
                return Err(fail(Kind::Version));
            }
            metadata
                .get("_meta")
                .and_then(|v| v.get("io.modelcontextprotocol/serverInfo"))
        } else {
            metadata.get("serverInfo")
        };
        let bounded = |value: Option<&Value>| -> Result<Option<String>, Failure> {
            value
                .map(|v| {
                    v.as_str()
                        .filter(|s| s.len() <= 128 && !s.chars().any(char::is_control))
                        .map(str::to_owned)
                        .ok_or_else(|| fail(Kind::Protocol))
                })
                .transpose()
        };
        this.server = Some(ServerInfo {
            protocol_version: this.protocol.clone(),
            name: bounded(info.and_then(|v| v.get("name")))?,
            version: bounded(info.and_then(|v| v.get("version")))?,
        });
        Ok(this)
    }
    fn modern(&self) -> bool {
        self.protocol == MODERN
    }
    fn check_active(&self) -> Result<(), Failure> {
        if self.cancel.is_cancelled() {
            return Err(fail(Kind::Cancelled));
        }
        if Instant::now() >= self.deadline {
            return Err(fail(Kind::Deadline));
        }
        Ok(())
    }
    pub fn server(&self) -> &ServerInfo {
        self.server.as_ref().expect("connected MCP metadata")
    }
    fn check_metadata(&self, value: &Value) -> Result<(), Failure> {
        if self.bearer.as_deref().is_some_and(|s| contains(value, s)) {
            return Err(fail(Kind::MetadataSecret));
        }
        Ok(())
    }
    async fn initialize(&mut self, pinned: Option<&str>) -> Result<Value, Failure> {
        let value=self.rpc("initialize",json!({"protocolVersion":self.protocol,"capabilities":{},"clientInfo":{"name":"Dolores","version":env!("CARGO_PKG_VERSION")}}),HeaderMap::new(),false).await?;
        let version = value["protocolVersion"]
            .as_str()
            .filter(|v| LEGACY.contains(v))
            .ok_or_else(|| fail(Kind::Version))?;
        if pinned.is_some_and(|p| p != version) {
            return Err(fail(Kind::Version));
        }
        self.protocol = version.into();
        self.check_metadata(&value)?;
        self.post_one_way(json!({"jsonrpc":"2.0","method":"notifications/initialized"}))
            .await?;
        Ok(value)
    }
    fn request(&self, body: &Value, extra: HeaderMap) -> Result<reqwest::RequestBuilder, Failure> {
        let mut request = self
            .client
            .post(self.url.clone())
            .header(ACCEPT, "application/json, text/event-stream")
            .header(CONTENT_TYPE, "application/json")
            .header("mcp-protocol-version", &self.protocol)
            .headers(extra)
            .json(body);
        if self.modern() {
            request = request.header(
                "mcp-method",
                body["method"]
                    .as_str()
                    .ok_or_else(|| fail(Kind::Protocol))?,
            );
        }
        if let Some(token) = &self.bearer {
            request = request.bearer_auth(token);
        }
        if let Some(session) = &self.session {
            let mut header = HeaderValue::from_str(session).map_err(|_| fail(Kind::Protocol))?;
            header.set_sensitive(true);
            request = request.header("mcp-session-id", header);
        }
        Ok(request)
    }
    async fn post_one_way(&mut self, body: Value) -> Result<(), Failure> {
        self.check_active()?;
        if body.to_string().len() > FRAME {
            return Err(fail(Kind::FrameLimit));
        }
        let request = self.request(&body, HeaderMap::new())?;
        let cancel = self.cancel.clone();
        let deadline = self.deadline;
        tokio::select! {
            biased;
            _=cancel.cancelled()=>Err(fail(Kind::Cancelled)),
            _=tokio::time::sleep_until(deadline)=>Err(fail(Kind::Deadline)),
            result=async {
                let mut response=request.send().await.map_err(|_|fail(Kind::Network))?;
                self.status(&response)?;
                if response.status()!=202 || response.content_length().is_some_and(|n|n>0){return Err(fail(Kind::Protocol));}
                if wire::chunk(&mut response,&mut self.wire).await?.is_some_and(|b|!b.is_empty()){return Err(fail(Kind::Protocol));}
                Ok(())
            }=>result,
        }
    }
    fn status(&self, response: &Response) -> Result<(), Failure> {
        let code = response.status().as_u16();
        if matches!(code, 401 | 403) {
            let mut combined = String::new();
            for (index, value) in response
                .headers()
                .get_all(WWW_AUTHENTICATE)
                .iter()
                .enumerate()
            {
                let text = value.to_str().ok().filter(|s| {
                    s.is_ascii()
                        && !s.chars().any(char::is_control)
                        && self.bearer.as_ref().is_none_or(|token| !s.contains(token))
                });
                let Some(text) = text.filter(|s| index < 8 && combined.len() + s.len() + 2 <= 8192)
                else {
                    // Fail OAuth parsing instead of silently selecting a different
                    // header or falling back after malformed/secret-bearing data.
                    combined = "Bearer resource_metadata=".into();
                    break;
                };
                if !combined.is_empty() {
                    combined.push_str(", ");
                }
                combined.push_str(text);
            }
            let challenge = (!combined.is_empty()).then_some(combined);
            return Err(Failure {
                kind: Kind::Authorization(code),
                challenge,
            });
        }
        if response.status().is_redirection() {
            return Err(fail(Kind::Redirect));
        }
        if code == 404 && self.session.is_some() {
            return Err(fail(Kind::SessionExpired));
        }
        if !response.status().is_success() && code != 400 {
            return Err(fail(Kind::Http(code)));
        }
        Ok(())
    }
    async fn rpc(
        &mut self,
        method: &str,
        mut params: Value,
        extra: HeaderMap,
        probe: bool,
    ) -> Result<Value, Failure> {
        self.check_active()?;
        self.next = self
            .next
            .checked_add(1)
            .filter(|n| *n <= 128)
            .ok_or_else(|| fail(Kind::Notifications))?;
        let id = json!(format!("{}-{}", self.prefix, self.next));
        if self.modern() {
            params["_meta"] = json!({"io.modelcontextprotocol/protocolVersion":MODERN,"io.modelcontextprotocol/clientInfo":{"name":"Dolores","version":env!("CARGO_PKG_VERSION")},"io.modelcontextprotocol/clientCapabilities":{}});
        }
        let body = json!({"jsonrpc":"2.0","id":id,"method":method,"params":params});
        if body.to_string().len() > FRAME {
            return Err(fail(Kind::FrameLimit));
        }
        let cancel = self.cancel.clone();
        let deadline = self.deadline;
        let result = tokio::select! {
            biased;
            _=cancel.cancelled()=>Err(fail(Kind::Cancelled)),
            _=tokio::time::sleep_until(deadline)=>Err(fail(Kind::Deadline)),
            result=self.exchange(&body,extra,probe)=>result,
        };
        if result
            .as_ref()
            .is_err_and(|e| matches!(e.kind, Kind::Cancelled | Kind::Deadline))
            && !self.modern()
        {
            let notification = json!({"jsonrpc":"2.0","method":"notifications/cancelled","params":{"requestId":id}});
            let remaining = self
                .deadline
                .saturating_duration_since(Instant::now())
                .min(Duration::from_millis(250));
            // The owner's cancellation is already set. This bounded best-effort
            // notification must not pass through that same cancellation select.
            if !remaining.is_zero() {
                if let Ok(request) = self.request(&notification, HeaderMap::new()) {
                    let _ = tokio::time::timeout(remaining, request.send()).await;
                }
            }
        }
        let mut value = result?;
        if method == "tools/call" {
            if let Some(secret) = &self.bearer {
                redact(&mut value, secret)?;
            }
        } else {
            self.check_metadata(&value)?;
        }
        if value.get("inputRequests").is_some()
            || value.get("resultType").is_some_and(|v| v != "complete")
        {
            return Err(fail(Kind::UnsupportedInteraction));
        }
        if self.modern() && value["resultType"] != "complete" {
            return Err(fail(Kind::Protocol));
        }
        Ok(value)
    }
    async fn exchange(
        &mut self,
        body: &Value,
        extra: HeaderMap,
        probe: bool,
    ) -> Result<Value, Failure> {
        self.check_active()?;
        let mut response = self
            .request(body, extra)?
            .send()
            .await
            .map_err(|_| fail(Kind::Network))?;
        self.status(&response)?;
        let status = response.status().as_u16();
        if !self.modern() && body["method"] == "initialize" {
            let mut sessions = response.headers().get_all("mcp-session-id").iter();
            if let Some(value) = sessions.next() {
                let id = value
                    .to_str()
                    .ok()
                    .filter(|s| {
                        !s.is_empty()
                            && s.len() <= 256
                            && s.bytes().all(|b| (0x21..=0x7e).contains(&b))
                    })
                    .ok_or_else(|| fail(Kind::Protocol))?;
                if sessions.next().is_some() {
                    return Err(fail(Kind::Protocol));
                }
                self.session = Some(id.into());
            }
        }
        if response.content_length().is_some_and(|n| n > WIRE as u64) {
            return Err(fail(Kind::WireLimit));
        }
        let content_type = response
            .headers()
            .get(CONTENT_TYPE)
            .and_then(|s| s.to_str().ok())
            .and_then(|s| s.split(';').next())
            .unwrap_or("")
            .trim()
            .to_ascii_lowercase();
        if status == 400 || content_type == "application/json" {
            let mut bytes = vec![];
            while let Some(chunk) = wire::chunk(&mut response, &mut self.wire).await? {
                bytes.extend_from_slice(&chunk);
                if bytes.len() > FRAME {
                    return Err(fail(Kind::FrameLimit));
                }
            }
            let parsed = wire::parse(&bytes);
            if probe
                && status == 400
                && parsed
                    .as_ref()
                    .ok()
                    .and_then(|v| v["error"]["code"].as_i64())
                    .is_none_or(|code| !matches!(code, -32022..=-32020))
            {
                return Err(fail(Kind::LegacyProbe));
            }
            let value = parsed?;
            if probe && matches!(value["error"]["code"].as_i64(), Some(-32601 | -32600)) {
                return Err(fail(Kind::LegacyProbe));
            }
            if status == 400 && value.get("error").is_none() {
                return Err(fail(Kind::Http(status)));
            }
            return self.result(&value, &body["id"]);
        }
        if content_type != "text/event-stream" {
            return Err(fail(Kind::Protocol));
        }
        let mut sse = wire::Sse::default();
        while let Some(chunk) = wire::chunk(&mut response, &mut self.wire).await? {
            for value in sse.push(&chunk)? {
                if value.get("method").is_some() {
                    self.notifications += 1;
                    if self.notifications > 64 {
                        return Err(fail(Kind::Notifications));
                    }
                    if let Some(id) = value.get("id") {
                        if self.modern() {
                            return Err(fail(Kind::UnsupportedInteraction));
                        }
                        let reply = if value["method"] == "ping" {
                            json!({"jsonrpc":"2.0","id":id,"result":{}})
                        } else {
                            json!({"jsonrpc":"2.0","id":id,"error":{"code":-32601,"message":"Method not supported"}})
                        };
                        self.post_one_way(reply).await?;
                    }
                } else {
                    return self.result(&value, &body["id"]);
                }
            }
        }
        Err(fail(Kind::Protocol))
    }
    fn result(&self, value: &Value, id: &Value) -> Result<Value, Failure> {
        if value.get("id") != Some(id)
            || value.get("result").is_some() == value.get("error").is_some()
        {
            return Err(fail(Kind::Protocol));
        }
        if let Some(error) = value.get("error") {
            let code = error["code"].as_i64().ok_or_else(|| fail(Kind::Protocol))?;
            return Err(fail(if code == -32022 {
                Kind::Version
            } else {
                Kind::Rpc(code)
            }));
        }
        value
            .get("result")
            .filter(|v| v.is_object())
            .cloned()
            .ok_or_else(|| fail(Kind::Protocol))
    }
    pub async fn list_tools(&mut self) -> Result<Vec<McpTool>, Failure> {
        self.catalog.clear();
        let mut tools = vec![];
        let mut cursor = None;
        let mut cursors = std::collections::BTreeSet::new();
        let mut seen = std::collections::BTreeSet::new();
        let mut count = 0;
        self.excluded.clear();
        for _ in 0..4 {
            let params = cursor
                .as_ref()
                .map_or_else(|| json!({}), |c| json!({"cursor":c}));
            let page = self
                .rpc("tools/list", params, HeaderMap::new(), false)
                .await?;
            let entries = page["tools"]
                .as_array()
                .ok_or_else(|| fail(Kind::Protocol))?;
            count += entries.len();
            if count > 32 {
                return Err(fail(Kind::Catalog));
            }
            for item in entries {
                if item["execution"]["taskSupport"] == "required" {
                    continue;
                }
                let name = item["name"]
                    .as_str()
                    .ok_or_else(|| fail(Kind::Protocol))?
                    .to_owned();
                let tool = McpTool {
                    name,
                    description: item
                        .get("description")
                        .and_then(Value::as_str)
                        .unwrap_or("")
                        .into(),
                    input_schema: item["inputSchema"].clone(),
                };
                tool.validate().map_err(|_| fail(Kind::Protocol))?;
                if !seen.insert(tool.name.clone()) {
                    return Err(fail(Kind::Protocol));
                }
                if self.modern() && headers::validate(&tool.input_schema).is_err() {
                    self.excluded.push(ExcludedTool {
                        name: tool.name,
                        reason: "Invalid HTTP header annotations",
                    });
                    continue;
                }
                tools.push(tool);
                if serde_json::to_vec(&tools).map_or(true, |b| b.len() > 32 * 1024) {
                    return Err(fail(Kind::Catalog));
                }
            }
            match page.get("nextCursor").filter(|v| !v.is_null()) {
                None => {
                    tools.sort_by(|a, b| a.name.cmp(&b.name));
                    self.catalog = tools.clone();
                    return Ok(tools);
                }
                Some(Value::String(s))
                    if !s.is_empty() && s.len() <= 1024 && cursors.insert(s.clone()) =>
                {
                    cursor = Some(s.clone())
                }
                _ => return Err(fail(Kind::Catalog)),
            }
        }
        Err(fail(Kind::Catalog))
    }
    pub async fn call_tool(&mut self, reviewed: &McpTool, args: Value) -> Result<Value, Failure> {
        if !args.is_object() || serde_json::to_vec(&args).map_or(true, |b| b.len() > 4096) {
            return Err(fail(Kind::Arguments));
        }
        if !self.catalog.contains(reviewed) {
            return Err(fail(Kind::Protocol));
        }
        let headers = if self.modern() {
            headers::tool_headers(&reviewed.name, &reviewed.input_schema, &args)?
        } else {
            HeaderMap::new()
        };
        let result = self
            .rpc(
                "tools/call",
                json!({"name":reviewed.name,"arguments":args}),
                headers,
                false,
            )
            .await;
        if result.is_err() {
            self.catalog.clear();
        }
        result
    }
    pub async fn close(&mut self) -> Result<(), Failure> {
        let Some(session) = self.session.take() else {
            return Ok(());
        };
        if !matches!(
            lifecycle::start(self, session).wait().await,
            lifecycle::Outcome::Unconfirmed
        ) {
            Ok(())
        } else {
            Err(fail(Kind::Network))
        }
    }
}
impl Drop for RemoteClient {
    fn drop(&mut self) {
        if let Some(session) = self.session.take() {
            lifecycle::start(self, session);
        }
    }
}
fn http_client(url: &Url, builder: reqwest::ClientBuilder) -> Result<Client, Failure> {
    let host = url.host_str().unwrap_or("");
    let loopback = matches!(host, "localhost" | "localhost.")
        || host
            .trim_start_matches('[')
            .trim_end_matches(']')
            .parse::<std::net::IpAddr>()
            .is_ok_and(|ip| ip.is_loopback());
    // A proxy interprets loopback relative to its own host. In particular, plain
    // HTTP must not send a local bearer credential to that different machine.
    let builder = if loopback {
        builder.no_proxy()
    } else {
        builder
    };
    builder
        .redirect(reqwest::redirect::Policy::none())
        .retry(reqwest::retry::never())
        .timeout(Duration::from_secs(30))
        .pool_max_idle_per_host(0)
        .build()
        .map_err(|_| fail(Kind::Network))
}
