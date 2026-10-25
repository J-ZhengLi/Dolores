use super::*;
use dolores_core::{
    CredentialStore, McpConnection, McpCredentialBinding, McpRemoteAuthentication, McpRemoteTarget,
    ToolCall, ToolPlugin, MCP_REMOTE_BEARER,
};
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering::SeqCst};
use std::{
    collections::BTreeMap,
    sync::{Arc, Mutex},
};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpListener,
    sync::mpsc,
    task::JoinHandle,
};

#[derive(Clone, Debug)]
struct Request {
    verb: String,
    headers: BTreeMap<String, String>,
    body: Value,
}
enum Reply {
    Data {
        status: u16,
        mime: &'static str,
        headers: Vec<(&'static str, String)>,
        bytes: Vec<u8>,
    },
    Hold,
}
fn reply(status: u16, mime: &'static str, bytes: impl Into<Vec<u8>>) -> Reply {
    Reply::Data {
        status,
        mime,
        headers: vec![],
        bytes: bytes.into(),
    }
}
fn rpc(request: &Request, mut result: Value) -> Reply {
    if request.body["params"]["_meta"]["io.modelcontextprotocol/protocolVersion"] == MODERN
        && result.get("resultType").is_none()
    {
        result["resultType"] = json!("complete");
    }
    reply(
        200,
        "application/json",
        json!({"jsonrpc":"2.0","id":request.body["id"],"result":result}).to_string(),
    )
}
fn discovery() -> Value {
    json!({"resultType":"complete","supportedVersions":[MODERN],"capabilities":{"tools":{}},"_meta":{"io.modelcontextprotocol/serverInfo":{"name":"Public fixture","version":"1"}}})
}
fn tool() -> Value {
    json!({"name":"echo","description":"Public echo","inputSchema":{"type":"object"}})
}
fn common(request: &Request) -> Reply {
    match request.body["method"].as_str() {
        Some("server/discover") => rpc(request, discovery()),
        Some("tools/list") => rpc(request, json!({"tools":[tool()]})),
        Some("tools/call") => rpc(request, json!({"content":[{"type":"text","text":"done"}]})),
        _ => panic!("unexpected fixture request"),
    }
}
#[tokio::test]
async fn owned_legacy_cleanup_survives_cancel_drop_and_open_failure_without_replay() {
    for mode in 0..3 {
        let mut fixture=Fixture::new(move |r,index| {
            if r.verb=="DELETE" {return reply(204,"application/json","");}
            if index==0 {
                let mut response=rpc(r,json!({"protocolVersion":"2025-11-25","capabilities":{"tools":{}},"serverInfo":{"name":"Public fixture","version":"1"}}));
                if let Reply::Data{headers,..}=&mut response {headers.push(("Mcp-Session-Id","public-session".into()));}
                return response;
            }
            if index==1 {return if mode==2 {Reply::Hold}else{reply(202,"application/json","")};}
            if r.body["method"]=="notifications/cancelled" {return reply(202,"application/json","");}
            if r.body["method"]=="tools/call" {return Reply::Hold;}
            common(r)
        }).await;
        let owner = Arc::new(CleanupOwner::default());
        let mut connection = configured(&fixture);
        connection.protocol_version = "2025-11-25".into();
        let directory = tempfile::tempdir().unwrap();
        let plugin = plugins_owned(directory.path().into(), connection, None, owner.clone())
            .unwrap()
            .remove(0);
        let request = prepared(&plugin, "owned");
        let cancel = CancellationToken::new();
        let token = cancel.clone();
        let task = tokio::spawn(async move { plugin.invoke(&request, token).await });
        for _ in 0..if mode == 2 { 2 } else { 4 } {
            fixture.arrived().await;
        }
        if mode == 0 {
            cancel.cancel();
            assert!(task.await.unwrap().is_err());
        } else {
            task.abort();
            let _ = task.await;
        }
        let receipts = tokio::time::timeout(Duration::from_secs(3), owner.finish())
            .await
            .unwrap();
        assert_eq!(receipts.len(), 1);
        assert_eq!(receipts[0]["confirmed"], true);
        let seen = fixture.requests();
        assert_eq!(seen.iter().filter(|r| r.verb == "DELETE").count(), 1);
        assert_eq!(
            seen.iter()
                .filter(|r| r.body["method"] == "tools/call")
                .count(),
            usize::from(mode != 2)
        );
        assert!(!json!(receipts).to_string().contains("public-session"));
    }
}
#[tokio::test]
async fn owned_cleanup_timeout_is_bounded_and_does_not_erase_a_returned_result() {
    let fixture=Fixture::new(|r,index| {
        if index==0 {let mut response=rpc(r,json!({"protocolVersion":"2025-11-25","capabilities":{"tools":{}},"serverInfo":{"name":"Public fixture","version":"1"}}));if let Reply::Data{headers,..}=&mut response {headers.push(("Mcp-Session-Id","public-session".into()));}return response;}
        if index==1 {return reply(202,"application/json","");}
        if r.verb=="DELETE" {Reply::Hold}else{common(r)}
    }).await;
    let owner = Arc::new(CleanupOwner::default());
    let mut connection = configured(&fixture);
    connection.protocol_version = "2025-11-25".into();
    let directory = tempfile::tempdir().unwrap();
    let plugin = plugins_owned(directory.path().into(), connection, None, owner.clone())
        .unwrap()
        .remove(0);
    let start = Instant::now();
    let result = plugin
        .invoke(&prepared(&plugin, "done"), CancellationToken::new())
        .await
        .unwrap();
    assert!(start.elapsed() < Duration::from_secs(4));
    let result: Value = serde_json::from_str(&result).unwrap();
    assert_eq!(result["text"], "done");
    assert!(result["cleanupWarning"].is_string());
    assert_eq!(owner.finish().await[0]["confirmed"], false);
}
struct Fixture {
    url: String,
    task: JoinHandle<()>,
    seen: Arc<Mutex<Vec<Request>>>,
    arrivals: mpsc::UnboundedReceiver<Request>,
}
impl Drop for Fixture {
    fn drop(&mut self) {
        self.task.abort();
    }
}
impl Fixture {
    async fn new(handler: impl Fn(&Request, usize) -> Reply + Send + 'static) -> Self {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let url = format!("http://{}/mcp", listener.local_addr().unwrap());
        let seen = Arc::new(Mutex::new(vec![]));
        let saved = seen.clone();
        let (sender, arrivals) = mpsc::unbounded_channel();
        let task = tokio::spawn(async move {
            let mut held_connections = vec![];
            loop {
                let (mut stream, _) = listener.accept().await.unwrap();
                let mut data = vec![];
                let mut buf = [0; 2048];
                let boundary = loop {
                    let n = stream.read(&mut buf).await.unwrap();
                    assert!(n > 0);
                    data.extend_from_slice(&buf[..n]);
                    assert!(data.len() < FRAME + 16 * 1024);
                    if let Some(p) = data.windows(4).position(|w| w == b"\r\n\r\n") {
                        break p + 4;
                    }
                };
                let header = std::str::from_utf8(&data[..boundary]).unwrap();
                let verb = header.split_whitespace().next().unwrap().to_owned();
                let headers: BTreeMap<String, String> = header
                    .lines()
                    .skip(1)
                    .filter_map(|s| s.split_once(':'))
                    .map(|(k, v)| (k.to_ascii_lowercase(), v.trim().into()))
                    .collect();
                let length = headers
                    .get("content-length")
                    .map_or(0, |n| n.parse::<usize>().unwrap());
                while data.len() < boundary + length {
                    let n = stream.read(&mut buf).await.unwrap();
                    assert!(n > 0);
                    data.extend_from_slice(&buf[..n]);
                }
                let body = if length == 0 {
                    Value::Null
                } else {
                    serde_json::from_slice(&data[boundary..boundary + length]).unwrap()
                };
                let request = Request {
                    verb,
                    headers,
                    body,
                };
                let index = {
                    let mut records = saved.lock().unwrap();
                    let index = records.len();
                    records.push(request.clone());
                    index
                };
                let response = handler(&request, index);
                let _ = sender.send(request);
                match response {
                    Reply::Hold => held_connections.push(stream),
                    Reply::Data {
                        status,
                        mime,
                        headers,
                        bytes,
                    } => {
                        let mut head=format!("HTTP/1.1 {status} Fixture\r\nContent-Type: {mime}\r\nContent-Length: {}\r\nConnection: close\r\n",bytes.len());
                        for (key, value) in headers {
                            head.push_str(&format!("{key}: {value}\r\n"));
                        }
                        head.push_str("\r\n");
                        if stream.write_all(head.as_bytes()).await.is_ok() {
                            let _ = stream.write_all(&bytes).await;
                        }
                    }
                }
            }
        });
        Self {
            url,
            task,
            seen,
            arrivals,
        }
    }
    fn requests(&self) -> Vec<Request> {
        self.seen.lock().unwrap().clone()
    }
    async fn arrived(&mut self) {
        tokio::time::timeout(Duration::from_secs(10), self.arrivals.recv())
            .await
            .expect("fixture request did not arrive")
            .expect("fixture stopped");
    }
}
async fn connect(f: &Fixture) -> RemoteClient {
    RemoteClient::connect(&f.url, None, Some(MODERN), CancellationToken::new())
        .await
        .unwrap()
}

#[derive(Default)]
struct Vault {
    values: Mutex<BTreeMap<String, String>>,
    reads: AtomicUsize,
    locked: AtomicBool,
}
impl CredentialStore for Vault {
    fn descriptor(&self) -> dolores_core::PluginDescriptor {
        dolores_core::PluginDescriptor {
            id: "public.fixture.vault",
            kind: "credentials",
            api_version: 1,
        }
    }
    fn read(&self, id: &str) -> Result<Option<String>, String> {
        self.reads.fetch_add(1, SeqCst);
        if self.locked.load(SeqCst) {
            return Err("private vault diagnostic".into());
        }
        Ok(self.values.lock().unwrap().get(id).cloned())
    }
    fn write(&self, id: &str, value: &str) -> Result<(), String> {
        self.values.lock().unwrap().insert(id.into(), value.into());
        Ok(())
    }
    fn delete(&self, id: &str) -> Result<(), String> {
        self.values.lock().unwrap().remove(id);
        Ok(())
    }
}
fn configured(f: &Fixture) -> McpConnection {
    McpConnection {
        id: "00000000-0000-4000-8000-000000000001".into(),
        revision: 1,
        enabled: true,
        launch: None,
        remote: Some(McpRemoteTarget {
            label: "Public remote".into(),
            endpoint: f.url.clone(),
            authentication: McpRemoteAuthentication::None,
        }),
        fingerprints: vec![],
        protocol_version: MODERN.into(),
        server_name: "Public fixture".into(),
        server_version: "1".into(),
        tools: vec![serde_json::from_value(tool()).unwrap()],
        credentials: vec![],
        retired_credentials: vec![],
    }
}
fn with_key(root: &std::path::Path, connection: &mut McpConnection, vault: &Vault) {
    let target = connection.remote.as_mut().unwrap();
    target.authentication = McpRemoteAuthentication::Bearer;
    let id = "00000000-0000-4000-8000-000000000002";
    let key = credentials::BearerCredential::new("fixture-secret-token".into()).unwrap();
    vault
        .write(id, &key.encoded_for(root, &connection.id, target).unwrap())
        .unwrap();
    connection.credentials = vec![McpCredentialBinding {
        name: MCP_REMOTE_BEARER.into(),
        credential_id: id.into(),
    }];
}
fn prepared(plugin: &Arc<dyn ToolPlugin>, id: &str) -> dolores_core::ToolRequest {
    plugin
        .prepare(&ToolCall {
            id: id.into(),
            name: plugin.spec().name,
            arguments: "{}".into(),
        })
        .unwrap()
}

#[tokio::test]
async fn approved_remote_call_reads_bound_key_only_after_approval_and_is_one_use() {
    let fixture = Fixture::new(|r, index| {
        if index == 2 {
            rpc(
                r,
                json!({"content":[{"type":"text","text":"fixture-secret-token done"}]}),
            )
        } else {
            common(r)
        }
    })
    .await;
    let folder = tempfile::tempdir().unwrap();
    let vault = Arc::new(Vault::default());
    let mut connection = configured(&fixture);
    with_key(folder.path(), &mut connection, &vault);
    let plugin = crate::plugins_with_credentials(folder.path(), connection, vault.clone())
        .unwrap()
        .remove(0);
    let plan = prepared(&plugin, "approved");
    assert_eq!(vault.reads.load(SeqCst), 0);
    assert!(fixture.requests().is_empty());
    assert_eq!(
        plan.mcp.as_ref().unwrap().remote_endpoint.as_deref(),
        Some(fixture.url.as_str())
    );
    assert!(plan.mcp.as_ref().unwrap().isolation.is_none());
    assert!(!serde_json::to_string(&plan)
        .unwrap()
        .contains("fixture-secret-token"));
    let result: Value = serde_json::from_str(
        &plugin
            .invoke(&plan, CancellationToken::new())
            .await
            .unwrap(),
    )
    .unwrap();
    assert_eq!(result["text"], "[redacted] done");
    assert_eq!(vault.reads.load(SeqCst), 1);
    assert_eq!(fixture.requests().len(), 3);
    assert!(plugin
        .invoke(&plan, CancellationToken::new())
        .await
        .unwrap_err()
        .contains("expired"));
    assert_eq!(fixture.requests().len(), 3);
}

#[tokio::test]
async fn tampered_or_revoked_remote_approval_never_reads_a_key_or_contacts_the_server() {
    let fixture = Fixture::new(|r, _| common(r)).await;
    let folder = tempfile::tempdir().unwrap();
    let vault = Arc::new(Vault::default());
    let mut connection = configured(&fixture);
    with_key(folder.path(), &mut connection, &vault);
    let plugin = crate::plugins_with_credentials(folder.path(), connection, vault.clone())
        .unwrap()
        .remove(0);
    let plan = prepared(&plugin, "tampered");
    let mut changed = plan.clone();
    changed.mcp.as_mut().unwrap().remote_endpoint = Some("https://other.test/mcp".into());
    assert!(plugin
        .invoke(&changed, CancellationToken::new())
        .await
        .unwrap_err()
        .contains("changed after approval"));
    assert!(plugin
        .invoke(&plan, CancellationToken::new())
        .await
        .is_err());
    let discarded = prepared(&plugin, "discarded");
    plugin.discard(&discarded);
    assert!(plugin
        .invoke(&discarded, CancellationToken::new())
        .await
        .is_err());
    let stopped = prepared(&plugin, "stopped");
    let cancel = CancellationToken::new();
    cancel.cancel();
    assert!(plugin.invoke(&stopped, cancel).await.is_err());
    assert_eq!(vault.reads.load(SeqCst), 0);
    assert!(fixture.requests().is_empty());
}

#[tokio::test]
async fn changed_remote_metadata_and_missing_key_refuse_without_business_calls() {
    for mode in 0..4 {
        let fixture=Fixture::new(move |r,index|if index==1 && mode==0{rpc(r,json!({"tools":[{"name":"echo","description":"Changed","inputSchema":{"type":"object"}}]}))}else{common(r)}).await;
        let folder = tempfile::tempdir().unwrap();
        let vault = Arc::new(Vault::default());
        let mut connection = configured(&fixture);
        if mode == 1 || mode == 2 {
            with_key(folder.path(), &mut connection, &vault);
            if mode == 1 {
                vault.values.lock().unwrap().clear();
            } else {
                vault.locked.store(true, SeqCst);
            }
        }
        if mode == 3 {
            connection.server_version = "changed".into();
        }
        let plugin = crate::plugins_with_credentials(folder.path(), connection, vault)
            .unwrap()
            .remove(0);
        let plan = prepared(&plugin, "refuse");
        let error = plugin
            .invoke(&plan, CancellationToken::new())
            .await
            .unwrap_err();
        assert!(!error.contains("private vault diagnostic"));
        assert!(fixture
            .requests()
            .iter()
            .all(|r| r.body["method"] != "tools/call"));
        assert_eq!(
            fixture.requests().len(),
            if mode == 1 || mode == 2 { 0 } else { 2 }
        );
    }
}

#[test]
fn remote_credential_binding_refuses_changed_root_id_target_purpose_and_local_payload() {
    let folder = tempfile::tempdir().unwrap();
    let other = tempfile::tempdir().unwrap();
    let target = McpRemoteTarget {
        label: "Public remote".into(),
        endpoint: "https://example.test/mcp?tenant=one".into(),
        authentication: McpRemoteAuthentication::Bearer,
    };
    let mut connection = McpConnection {
        id: "00000000-0000-4000-8000-000000000001".into(),
        revision: 1,
        enabled: true,
        launch: None,
        remote: Some(target.clone()),
        fingerprints: vec![],
        protocol_version: MODERN.into(),
        server_name: "Public".into(),
        server_version: "1".into(),
        tools: vec![serde_json::from_value(tool()).unwrap()],
        credentials: vec![],
        retired_credentials: vec![],
    };
    let vault = Vault::default();
    with_key(folder.path(), &mut connection, &vault);
    credentials::resolve_bearer(folder.path(), &connection, &vault).unwrap();
    assert!(credentials::resolve_bearer(other.path(), &connection, &vault).is_err());
    let original = connection.clone();
    connection.id = "00000000-0000-4000-8000-000000000003".into();
    assert!(credentials::resolve_bearer(folder.path(), &connection, &vault).is_err());
    connection = original.clone();
    connection.remote.as_mut().unwrap().endpoint = "https://other.test/mcp".into();
    assert!(credentials::resolve_bearer(folder.path(), &connection, &vault).is_err());
    connection = original.clone();
    connection.credentials[0].name = "SERVICE_API_KEY".into();
    assert!(credentials::resolve_bearer(folder.path(), &connection, &vault).is_err());
    connection = original;
    vault
        .write(
            &connection.credentials[0].credential_id,
            "{\"launch\":{},\"value\":\"fixture-secret-token\"}",
        )
        .unwrap();
    assert!(credentials::resolve_bearer(folder.path(), &connection, &vault).is_err());
    let key = credentials::BearerCredential::new("fixture-secret-token".into()).unwrap();
    let mut leaked = target;
    leaked.endpoint.push_str("&token=fixture-secret-token");
    assert!(key
        .encoded_for(folder.path(), &connection.id, &leaked)
        .is_err());
    leaked.endpoint = "https://example.test/mcp".into();
    leaked.label = "fixture-secret-token".into();
    assert!(key.validate_target(&leaked).is_err());
}

#[tokio::test]
async fn remote_text_result_survives_legacy_session_cleanup_failure() {
    let fixture=Fixture::new(|r,index|match index {
        0=>{let mut response=rpc(r,json!({"protocolVersion":"2025-11-25","capabilities":{"tools":{}},"serverInfo":{"name":"Public fixture","version":"1"}}));if let Reply::Data{headers,..}=&mut response{headers.push(("Mcp-Session-Id","public-session".into()));}response},
        1=>reply(202,"application/json",""),2|3=>common(r),4=>reply(503,"text/plain","cleanup unavailable"),_=>panic!("tool repeated"),
    }).await;
    let folder = tempfile::tempdir().unwrap();
    let mut connection = configured(&fixture);
    connection.protocol_version = "2025-11-25".into();
    let plugin = crate::plugins(folder.path(), connection).unwrap().remove(0);
    let plan = prepared(&plugin, "cleanup");
    let result: Value = serde_json::from_str(
        &plugin
            .invoke(&plan, CancellationToken::new())
            .await
            .unwrap(),
    )
    .unwrap();
    assert_eq!(result["text"], "done");
    assert!(result["cleanupWarning"]
        .as_str()
        .unwrap()
        .contains("result was retained"));
    assert_eq!(
        fixture
            .requests()
            .iter()
            .filter(|r| r.body["method"] == "tools/call")
            .count(),
        1
    );
    assert_eq!(fixture.requests().last().unwrap().verb, "DELETE");
}

#[tokio::test]
async fn approved_remote_result_rejects_nontext_and_stops_uncertain_dispatch_without_replay() {
    for mode in 0..4 {
        let mut fixture = Fixture::new(move |r, index| {
            if index < 2 {
                common(r)
            } else if mode == 0 {
                rpc(
                    r,
                    json!({"content":[{"type":"image","data":"public fixture"}]}),
                )
            } else if mode == 2 {
                rpc(
                    r,
                    json!({"content":[{"type":"text","text":"x".repeat(8193)}]}),
                )
            } else if mode == 3 {
                rpc(
                    r,
                    json!({"content":[{"type":"text","text":"\u{1}".repeat(5000)}]}),
                )
            } else {
                Reply::Hold
            }
        })
        .await;
        let folder = tempfile::tempdir().unwrap();
        let plugin = crate::plugins(folder.path(), configured(&fixture))
            .unwrap()
            .remove(0);
        let plan = prepared(&plugin, "result");
        let cancel = CancellationToken::new();
        let owned_cancel = cancel.clone();
        let task = tokio::spawn(async move { plugin.invoke(&plan, owned_cancel).await });
        for _ in 0..3 {
            fixture.arrived().await;
        }
        if mode == 1 {
            cancel.cancel();
        }
        let error = task.await.unwrap().unwrap_err();
        assert!(if mode == 0 {
            error.contains("text results only")
        } else if mode == 2 {
            error.contains("8 KiB")
        } else if mode == 3 {
            error.contains("encoded limit")
        } else {
            error.contains("effects may remain")
        });
        assert!(error.contains("do not repeat it just to recover its output"));
        assert_eq!(fixture.requests().len(), 3);
    }
}
async fn rejected(f: &Fixture, pinned: Option<&str>) -> Failure {
    match RemoteClient::connect(
        &f.url,
        Some("fixture-secret-token"),
        pinned,
        CancellationToken::new(),
    )
    .await
    {
        Ok(_) => panic!("connection unexpectedly succeeded"),
        Err(e) => e,
    }
}

#[test]
fn endpoint_validation_is_local_and_credentials_never_use_the_url() {
    for url in [
        "https://example.test/mcp?tenant=one",
        "http://127.0.0.2/mcp",
        "http://[::1]/mcp",
        "http://localhost/mcp",
    ] {
        assert!(endpoint(url).is_ok());
    }
    for url in [
        "file:///mcp",
        "http://example.test/mcp",
        "https://user:pass@example.test/mcp",
        "https://example.test/mcp#fragment",
        "https://example.test/\n",
    ] {
        assert_eq!(endpoint(url).unwrap_err().kind, Kind::Endpoint);
    }
}

#[tokio::test]
async fn http_loopback_credentials_bypass_even_an_explicit_proxy() {
    let target = Fixture::new(|r, _| common(r)).await;
    let proxy = Fixture::new(|r, _| common(r)).await;
    let url = endpoint(&target.url).unwrap();
    let client = http_client(
        &url,
        Client::builder().proxy(reqwest::Proxy::all(&proxy.url).unwrap()),
    )
    .unwrap();
    let response=client.post(url).bearer_auth("fixture-secret-token").json(&json!({"jsonrpc":"2.0","id":"public-proxy-probe","method":"server/discover","params":{}})).send().await.unwrap();
    assert!(response.status().is_success());
    assert_eq!(
        proxy.requests().len(),
        0,
        "a loopback credential was sent to the proxy"
    );
    assert_eq!(target.requests().len(), 1);
    assert_eq!(
        target.requests()[0].headers["authorization"],
        "Bearer fixture-secret-token"
    );
}

#[tokio::test]
async fn modern_discovery_headers_call_and_redaction_use_one_stateless_endpoint() {
    let schema = json!({"type":"object","properties":{"nested":{"type":"object","properties":{"q":{"type":"string","x-mcp-header":"Query"},"n":{"type":"integer","x-mcp-header":"Count"},"b":{"type":"boolean","x-mcp-header":"Enabled"}}}}});
    let saved = schema.clone();
    let fixture=Fixture::new(move |r,_|match r.body["method"].as_str() {
        Some("tools/list")=>rpc(r,json!({"tools":[{"name":"查看","inputSchema":saved}]})),
        Some("tools/call")=>rpc(r,json!({"resultType":"complete","content":[{"type":"text","text":"fixture-secret-token done"}]})),
        _=>common(r),
    }).await;
    let mut client = RemoteClient::connect(
        &fixture.url,
        Some("fixture-secret-token"),
        None,
        CancellationToken::new(),
    )
    .await
    .unwrap();
    assert_eq!(client.server().name.as_deref(), Some("Public fixture"));
    let tools = client.list_tools().await.unwrap();
    let value = client
        .call_tool(&tools[0], json!({"nested":{"q":"中文","n":42.0,"b":false}}))
        .await
        .unwrap();
    assert_eq!(value["content"][0]["text"], "[redacted] done");
    client.close().await.unwrap();
    let requests = fixture.requests();
    assert_eq!(requests.len(), 3);
    for request in &requests {
        assert_eq!(request.verb, "POST");
        assert!(!request.headers.contains_key("mcp-session-id"));
        assert_eq!(
            request.headers["authorization"],
            "Bearer fixture-secret-token"
        );
        assert_eq!(request.headers["mcp-method"], request.body["method"]);
        assert_eq!(request.headers["mcp-protocol-version"], MODERN);
        assert_eq!(
            request.body["params"]["_meta"]["io.modelcontextprotocol/protocolVersion"],
            MODERN
        );
    }
    assert_eq!(requests[2].headers["mcp-name"], headers::encoded("查看"));
    assert_eq!(
        requests[2].headers["mcp-param-query"],
        headers::encoded("中文")
    );
    assert_eq!(requests[2].headers["mcp-param-count"], "42");
    assert_eq!(requests[2].headers["mcp-param-enabled"], "false");
}

#[tokio::test]
async fn legacy_probe_handshake_sse_session_and_explicit_close() {
    let fixture=Fixture::new(|r,index|match index {
        0=>reply(400,"text/plain","Unknown request"),
        1=>{assert_eq!(r.body["method"],"initialize");let result=json!({"protocolVersion":"2025-11-25","capabilities":{"tools":{}},"serverInfo":{"name":"Legacy","version":"1"}});
            let mut response=rpc(r,result);if let Reply::Data{headers,..}=&mut response{headers.push(("Mcp-Session-Id","public-session".into()));}response},
        2=>{assert_eq!(r.body["method"],"notifications/initialized");reply(202,"application/json","")},
        3=>rpc(r,json!({"tools":[tool()]})),
        4=>reply(200,"text/event-stream",format!("\u{feff}: comment\r\ndata: {{\"jsonrpc\":\"2.0\",\"method\":\"notifications/progress\",\"params\":{{}}}}\r\n\r\ndata: {}\r\n\r\n",json!({"jsonrpc":"2.0","id":r.body["id"],"result":{"content":[{"type":"text","text":"legacy done"}]}}))),
        5=>{assert_eq!(r.verb,"DELETE");reply(405,"application/json","")},
        _=>panic!("unexpected legacy request"),
    }).await;
    let mut client = RemoteClient::connect(&fixture.url, None, None, CancellationToken::new())
        .await
        .unwrap();
    assert_eq!(client.server().protocol_version, "2025-11-25");
    let tools = client.list_tools().await.unwrap();
    assert_eq!(
        client.call_tool(&tools[0], json!({})).await.unwrap()["content"][0]["text"],
        "legacy done"
    );
    client.close().await.unwrap();
    for request in fixture.requests().iter().skip(2) {
        assert_eq!(request.headers["mcp-session-id"], "public-session");
        assert!(!request.headers.contains_key("mcp-method"));
    }
}

#[tokio::test]
async fn auth_redirect_and_modern_version_errors_do_not_downgrade_or_leak() {
    for (status, expected) in [
        (401, Kind::Authorization(401)),
        (403, Kind::Authorization(403)),
        (302, Kind::Redirect),
        (503, Kind::Http(503)),
    ] {
        let fixture = Fixture::new(move |_, _| Reply::Data {
            status,
            mime: "application/json",
            headers: vec![
                (
                    "WWW-Authenticate",
                    "Bearer resource_metadata=\"https://example.test/protected\"".into(),
                ),
                ("Location", "http://127.0.0.1:9/other".into()),
            ],
            bytes: b"private server error fixture-secret-token".to_vec(),
        })
        .await;
        let error = rejected(&fixture, None).await;
        assert_eq!(error.kind, expected);
        assert_eq!(fixture.requests().len(), 1);
        assert!(!format!("{error:?} {error}").contains("fixture-secret-token"));
        assert!(!error.to_string().contains("private server"));
        assert_eq!(
            error.authentication_challenge().is_some(),
            matches!(status, 401 | 403)
        );
    }
    let fixture=Fixture::new(|r,_|reply(400,"application/json",json!({"jsonrpc":"2.0","id":r.body["id"],"error":{"code":-32022,"message":"unsupported"}}).to_string())).await;
    assert_eq!(rejected(&fixture, None).await.kind, Kind::Version);
    assert_eq!(fixture.requests().len(), 1);
}

#[tokio::test]
async fn authentication_keeps_all_challenges_without_skipping_invalid_or_secret_headers() {
    for mode in 0..3 {
        let fixture = Fixture::new(move |_,_| Reply::Data {
            status:401, mime:"application/json", bytes:vec![],
            headers:vec![
                ("WWW-Authenticate","Basic realm=\"public fixture\"".into()),
                ("WWW-Authenticate", match mode {
                    0 => "Bearer resource_metadata=\"https://auth.example.test/metadata\", scope=\"tools:read\"".into(),
                    1 => "Bearer resource_metadata=\"https://auth.example.test/fixture-secret-token\"".into(),
                    _ => "x".repeat(8193),
                }),
            ],
        }).await;
        let error = rejected(&fixture, None).await;
        let challenge = error.authentication_challenge().unwrap();
        assert!(!challenge.contains("fixture-secret-token"));
        if mode == 0 {
            assert!(challenge.starts_with("Basic"));
            assert!(challenge.contains("Bearer resource_metadata="));
        } else {
            assert_eq!(challenge, "Bearer resource_metadata=");
        }
        assert_eq!(fixture.requests().len(), 1);
    }
}

#[tokio::test]
async fn expired_session_and_changed_pin_require_reinspection_without_call_replay() {
    let fixture=Fixture::new(|r,index|match index {
        0=>{let mut response=rpc(r,json!({"protocolVersion":"2025-11-25","capabilities":{"tools":{}},"serverInfo":{"name":"Legacy","version":"1"}}));if let Reply::Data{headers,..}=&mut response{headers.push(("Mcp-Session-Id","public-session".into()));}response},
        1=>reply(202,"application/json",""),2=>common(r),_=>reply(404,"application/json","expired"),
    }).await;
    let mut client = RemoteClient::connect(
        &fixture.url,
        None,
        Some("2025-11-25"),
        CancellationToken::new(),
    )
    .await
    .unwrap();
    let tools = client.list_tools().await.unwrap();
    assert_eq!(
        client
            .call_tool(&tools[0], json!({}))
            .await
            .unwrap_err()
            .kind,
        Kind::SessionExpired
    );
    assert_eq!(fixture.requests().len(), 4);
    let changed = Fixture::new(|r, _| {
        rpc(
            r,
            json!({"protocolVersion":"2025-06-18","capabilities":{"tools":{}}}),
        )
    })
    .await;
    assert_eq!(
        rejected(&changed, Some("2025-11-25")).await.kind,
        Kind::Version
    );
    assert_eq!(changed.requests().len(), 1);
}

#[tokio::test]
async fn invalid_annotation_excludes_only_affected_tool_and_failed_refresh_revokes_catalog() {
    let fixture=Fixture::new(|r,index|match index {
        1=>rpc(r,json!({"tools":[tool(),{"name":"bad","inputSchema":{"type":"object","properties":{"x":{"type":"string","x-mcp-header":"bad header"}}}}]})),
        2=>rpc(r,json!({"tools":[tool()],"nextCursor":"same"})),
        3=>rpc(r,json!({"tools":[],"nextCursor":"same"})),_=>common(r),
    }).await;
    let mut client = connect(&fixture).await;
    let tools = client.list_tools().await.unwrap();
    assert_eq!(tools.len(), 1);
    assert_eq!(client.excluded[0].name, "bad");
    assert_eq!(client.list_tools().await.unwrap_err().kind, Kind::Catalog);
    assert_eq!(
        client
            .call_tool(&tools[0], json!({}))
            .await
            .unwrap_err()
            .kind,
        Kind::Protocol
    );
    assert_eq!(fixture.requests().len(), 4);
}

#[tokio::test]
async fn stop_after_dispatch_retains_uncertainty_and_sends_no_replay() {
    let mut fixture =
        Fixture::new(|r, index| if index == 2 { Reply::Hold } else { common(r) }).await;
    let mut client = connect(&fixture).await;
    let tools = client.list_tools().await.unwrap();
    fixture.arrived().await;
    fixture.arrived().await;
    let cancel = client.cancel.clone();
    let task = tokio::spawn(async move { client.call_tool(&tools[0], json!({})).await });
    fixture.arrived().await;
    cancel.cancel();
    let error = task.await.unwrap().unwrap_err();
    assert_eq!(error.kind, Kind::Cancelled);
    assert!(error.to_string().contains("effects may remain"));
    assert_eq!(fixture.requests().len(), 3);
}

#[tokio::test]
async fn one_deadline_covers_discovery_and_later_calls() {
    let fixture = Fixture::new(|r, _| common(r)).await;
    let mut client = connect(&fixture).await;
    let tools = client.list_tools().await.unwrap();
    client.deadline = Instant::now();
    assert_eq!(
        client
            .call_tool(&tools[0], json!({}))
            .await
            .unwrap_err()
            .kind,
        Kind::Deadline
    );
    assert_eq!(fixture.requests().len(), 2);
}

#[tokio::test]
async fn deadline_interrupts_a_confirmed_dispatched_call_without_reset_or_replay() {
    let mut fixture =
        Fixture::new(|r, index| if index == 2 { Reply::Hold } else { common(r) }).await;
    let mut client = connect(&fixture).await;
    let tools = client.list_tools().await.unwrap();
    fixture.arrived().await;
    fixture.arrived().await;
    let task = tokio::spawn(async move { client.call_tool(&tools[0], json!({})).await });
    fixture.arrived().await; // Establish blocked network IO before pausing time.
    tokio::time::pause();
    tokio::time::advance(Duration::from_secs(31)).await;
    assert_eq!(task.await.unwrap().unwrap_err().kind, Kind::Deadline);
    assert_eq!(fixture.requests().len(), 3);
}

#[tokio::test]
async fn malformed_or_oversized_output_is_not_retried_and_credentials_are_scrubbed() {
    for mode in 0..7 {
        let fixture = Fixture::new(move |r, index| {
            if index < 2 {
                common(r)
            } else {
                match mode {
                    0 => reply(
                        200,
                        "application/json",
                        json!({"jsonrpc":"2.0","id":"wrong","result":{}}).to_string(),
                    ),
                    1 => reply(200, "text/event-stream", "data: invalid\n\n"),
                    2 => reply(200, "application/json", vec![b' '; FRAME + 1]),
                    3 => reply(200, "application/json", vec![b' '; WIRE + 1]),
                    4 => rpc(r, json!({"inputRequests":{},"resultType":"input_required"})),
                    5 => rpc(r, json!({"fixture-secret-token":"secret key"})),
                    _ => reply(
                        200,
                        "application/json",
                        json!({"jsonrpc":"2.0","id":r.body["id"],"result":{"content":[]}})
                            .to_string(),
                    ),
                }
            }
        })
        .await;
        let mut client = RemoteClient::connect(
            &fixture.url,
            Some("fixture-secret-token"),
            Some(MODERN),
            CancellationToken::new(),
        )
        .await
        .unwrap();
        let tools = client.list_tools().await.unwrap();
        let error = client.call_tool(&tools[0], json!({})).await.unwrap_err();
        assert_eq!(
            error.kind,
            [
                Kind::Protocol,
                Kind::Protocol,
                Kind::FrameLimit,
                Kind::WireLimit,
                Kind::UnsupportedInteraction,
                Kind::MetadataSecret,
                Kind::Protocol
            ][mode]
        );
        assert!(!error.to_string().contains("fixture-secret-token"));
        assert_eq!(
            client
                .call_tool(&tools[0], json!({}))
                .await
                .unwrap_err()
                .kind,
            Kind::Protocol
        );
        assert_eq!(fixture.requests().len(), 3);
    }
}

#[test]
fn sse_frames_handle_arbitrary_utf8_crlf_boundaries_and_bound_each_message() {
    let text="\u{feff}: comment\r\nevent: message\r\ndata: {\"jsonrpc\":\"2.0\",\r\ndata: \"id\":1,\"result\":{\"text\":\"中文\"}}\r\n\r\n";
    let mut parser = wire::Sse::default();
    let mut out = vec![];
    for byte in text.as_bytes() {
        out.extend(parser.push(&[*byte]).unwrap());
    }
    assert_eq!(out.len(), 1);
    assert_eq!(out[0]["result"]["text"], "中文");
    assert_eq!(
        wire::Sse::default()
            .push(&vec![b'x'; FRAME + 1])
            .unwrap_err()
            .kind,
        Kind::FrameLimit
    );
    for bad in ["[]", "{\"jsonrpc\":\"1.0\"}", "invalid"] {
        assert_eq!(
            wire::parse(bad.as_bytes()).unwrap_err().kind,
            Kind::Protocol
        );
    }
}

#[tokio::test]
async fn legacy_stop_notifies_the_exact_request_without_replaying_it() {
    let mut fixture=Fixture::new(|r,index|match index {
        0=>rpc(r,json!({"protocolVersion":"2025-03-26","capabilities":{"tools":{}},"serverInfo":{"name":"Legacy","version":"1"}})),
        1|4=>reply(202,"application/json",""),
        2=>common(r),3=>Reply::Hold,_=>panic!("request replayed"),
    }).await;
    let mut client = RemoteClient::connect(
        &fixture.url,
        None,
        Some("2025-03-26"),
        CancellationToken::new(),
    )
    .await
    .unwrap();
    let tools = client.list_tools().await.unwrap();
    for _ in 0..3 {
        fixture.arrived().await;
    }
    let cancel = client.cancel.clone();
    let task = tokio::spawn(async move { client.call_tool(&tools[0], json!({})).await });
    fixture.arrived().await;
    cancel.cancel();
    assert_eq!(task.await.unwrap().unwrap_err().kind, Kind::Cancelled);
    let requests = fixture.requests();
    assert_eq!(requests.len(), 5);
    assert_eq!(requests[4].body["method"], "notifications/cancelled");
    assert_eq!(
        requests[4].body["params"]["requestId"],
        requests[3].body["id"]
    );
}

#[tokio::test]
async fn modern_server_requests_floods_and_http_errors_stop_with_a_classified_limit() {
    for mode in 0..4 {
        let fixture=Fixture::new(move |r,index|if index<2{common(r)}else{match mode {
            0=>reply(200,"text/event-stream","data: {\"jsonrpc\":\"2.0\",\"id\":\"server-request\",\"method\":\"sampling/createMessage\",\"params\":{}}\n\n"),
            1=>reply(200,"text/event-stream","data: {\"jsonrpc\":\"2.0\",\"method\":\"notifications/progress\",\"params\":{}}\n\n".repeat(65)),
            2=>reply(400,"application/json",json!({"jsonrpc":"2.0","id":r.body["id"],"result":{"content":[]}}).to_string()),
            _=>reply(200,"text/event-stream",format!("data: {}\n\n",json!({"jsonrpc":"2.0","id":r.body["id"],"result":{"content":[]}})).trim_end().to_owned()),
        }}).await;
        let mut client = connect(&fixture).await;
        let tools = client.list_tools().await.unwrap();
        assert_eq!(
            client
                .call_tool(&tools[0], json!({}))
                .await
                .unwrap_err()
                .kind,
            [
                Kind::UnsupportedInteraction,
                Kind::Notifications,
                Kind::Http(400),
                Kind::Protocol
            ][mode]
        );
        assert_eq!(fixture.requests().len(), 3);
    }
}

#[tokio::test]
async fn local_rejections_and_secret_metadata_do_not_grant_a_tool() {
    let fixture=Fixture::new(|r,index|if index==1{rpc(r,json!({"tools":[{"name":"echo","description":"fixture-secret-token","inputSchema":{"type":"object"}}]}))}else{common(r)}).await;
    let mut client = RemoteClient::connect(
        &fixture.url,
        Some("fixture-secret-token"),
        Some(MODERN),
        CancellationToken::new(),
    )
    .await
    .unwrap();
    assert_eq!(
        client.list_tools().await.unwrap_err().kind,
        Kind::MetadataSecret
    );
    assert!(client.catalog.is_empty());
    let fake = McpTool {
        name: "fake".into(),
        description: "".into(),
        input_schema: json!({"type":"object"}),
    };
    assert_eq!(
        client.call_tool(&fake, json!({})).await.unwrap_err().kind,
        Kind::Protocol
    );
    assert_eq!(
        client
            .call_tool(&fake, json!({"too_large":"x".repeat(4096)}))
            .await
            .unwrap_err()
            .kind,
        Kind::Arguments
    );
    assert_eq!(fixture.requests().len(), 2);
    let url = format!("{}?access_token=fixture-secret-token", fixture.url);
    assert!(matches!(
        RemoteClient::connect(
            &url,
            Some("fixture-secret-token"),
            None,
            CancellationToken::new()
        )
        .await,
        Err(Failure {
            kind: Kind::Credential,
            ..
        })
    ));
    assert_eq!(fixture.requests().len(), 2);
}

#[tokio::test]
async fn initialization_notification_remains_cancellable_without_opening_discovery() {
    let mut fixture=Fixture::new(|r,index|if index==0{rpc(r,json!({"protocolVersion":"2025-11-25","capabilities":{"tools":{}},"serverInfo":{"name":"Legacy","version":"1"}}))}else{Reply::Hold}).await;
    let cancel = CancellationToken::new();
    let owned_cancel = cancel.clone();
    let url = fixture.url.clone();
    let task = tokio::spawn(async move {
        RemoteClient::connect(&url, None, Some("2025-11-25"), owned_cancel).await
    });
    fixture.arrived().await;
    fixture.arrived().await;
    cancel.cancel();
    match task.await.unwrap() {
        Err(error) => assert_eq!(error.kind, Kind::Cancelled),
        Ok(_) => panic!("cancelled connection succeeded"),
    };
    assert_eq!(fixture.requests().len(), 2);
}

#[test]
fn annotated_headers_reject_ambiguous_paths_and_encode_values_exactly() {
    for schema in [
        json!({"type":"object","x-mcp-header":"Root"}),
        json!({"type":"object","properties":{"a":{"type":"string","x-mcp-header":"Same"},"b":{"type":"string","x-mcp-header":"same"}}}),
        json!({"type":"object","properties":{"a":{"type":"array","items":{"type":"string","x-mcp-header":"Item"}}}}),
        json!({"type":"object","allOf":[{"properties":{"a":{"type":"string","x-mcp-header":"A"}}}]}),
        json!({"type":"object","$defs":{"a":{"type":"string","x-mcp-header":"A"}},"properties":{"x":{"$ref":"#/$defs/a"}}}),
        json!({"type":"object","properties":{"a":{"type":"number","x-mcp-header":"Number"}}}),
    ] {
        assert_eq!(
            headers::validate(&schema).unwrap_err().kind,
            Kind::HeaderSchema
        );
    }
    let data = json!({"type":"object","examples":[{"x-mcp-header":"sample data"}],"properties":{"a":{"type":"integer","x-mcp-header":"A"},"b":{"type":"string","x-mcp-header":"B"}}});
    headers::validate(&data).unwrap();
    for invalid in [json!(9_007_199_254_740_992_u64), json!(1.5), json!("1")] {
        assert_eq!(
            headers::tool_headers("echo", &data, &json!({"a":invalid}))
                .unwrap_err()
                .kind,
            Kind::Arguments
        );
    }
    assert!(headers::tool_headers("echo", &data, &json!({"a":null}))
        .unwrap()
        .get("mcp-param-a")
        .is_none());
    for value in [
        "中文",
        " leading",
        "trailing ",
        "=?base64?already?=",
        "line\nbreak",
    ] {
        assert!(headers::encoded(value).starts_with("=?base64?"));
    }
    assert_eq!(headers::encoded("normal\ttext"), "normal\ttext");
}

#[tokio::test]
async fn bounded_catalog_refuses_duplicates_count_pages_and_bytes_then_recovers() {
    for mode in 0..4 {
        let fixture=Fixture::new(move |r,index|if index==0{common(r)}else{match mode {
            0=>rpc(r,json!({"tools":[tool(),tool()]})),
            1=>rpc(r,json!({"tools":(0..33).map(|n|json!({"name":format!("tool{n}"),"inputSchema":{"type":"object"}})).collect::<Vec<_>>()})),
            2=>rpc(r,json!({"tools":[],"nextCursor":format!("page{index}")})),
            _=>rpc(r,json!({"tools":(0..5).map(|n|json!({"name":format!("tool{n}"),"description":"d".repeat(2048),"inputSchema":{"type":"object","description":"s".repeat(5000)}})).collect::<Vec<_>>()})),
        }}).await;
        let mut client = connect(&fixture).await;
        assert_eq!(
            client.list_tools().await.unwrap_err().kind,
            if mode == 0 {
                Kind::Protocol
            } else {
                Kind::Catalog
            }
        );
        assert!(client.catalog.is_empty());
    }
    // A rejected metadata response keeps no callable tools. A fresh explicit
    // inspection can succeed; it never replays a business invocation.
    let fixture = Fixture::new(|r, index| {
        if index == 1 {
            reply(401, "application/json", "expired")
        } else {
            common(r)
        }
    })
    .await;
    let mut client = connect(&fixture).await;
    assert_eq!(
        client.list_tools().await.unwrap_err().kind,
        Kind::Authorization(401)
    );
    assert!(client.catalog.is_empty());
    let tools = client.list_tools().await.unwrap();
    assert_eq!(
        client.call_tool(&tools[0], json!({})).await.unwrap()["content"][0]["text"],
        "done"
    );
    assert_eq!(
        fixture
            .requests()
            .iter()
            .filter(|r| r.body["method"] == "tools/call")
            .count(),
        1
    );
}
