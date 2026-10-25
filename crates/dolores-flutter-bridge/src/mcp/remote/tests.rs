use super::*;
use dolores_core::{SessionStore, SessionWorkspace, WorkspaceKind};
use dolores_store_sqlite::SqliteStore;
use std::{
    collections::BTreeMap,
    io::{BufRead, Read, Write},
    net::{TcpListener, TcpStream},
    sync::{
        atomic::{AtomicBool, Ordering::SeqCst},
        mpsc as signal,
    },
    thread,
};

struct Server {
    url: String,
    stop: Arc<AtomicBool>,
    hold: Arc<AtomicBool>,
    key: Arc<Mutex<Option<String>>>,
    arrivals: signal::Receiver<Value>,
    worker: Option<thread::JoinHandle<()>>,
}
impl Server {
    fn new(key: Option<&str>) -> Self {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let url = format!("http://{}/mcp", listener.local_addr().unwrap());
        let base = url.trim_end_matches("/mcp").to_owned();
        let stop = Arc::new(AtomicBool::new(false));
        let hold = Arc::new(AtomicBool::new(false));
        let key = Arc::new(Mutex::new(key.map(str::to_owned)));
        let (stopped, held, expected) = (stop.clone(), hold.clone(), key.clone());
        let (sender, arrivals) = signal::channel();
        let worker = thread::spawn(move || {
            let mut pending = vec![];
            while let Ok((mut stream, _)) = listener.accept() {
                if stopped.load(SeqCst) {
                    break;
                }
                stream
                    .set_read_timeout(Some(Duration::from_secs(5)))
                    .unwrap();
                let mut reader = std::io::BufReader::new(&mut stream);
                let mut line = String::new();
                reader.read_line(&mut line).unwrap();
                let path = line.split_whitespace().nth(1).unwrap_or("").to_owned();
                let mut headers = BTreeMap::new();
                loop {
                    line.clear();
                    reader.read_line(&mut line).unwrap();
                    if line == "\r\n" {
                        break;
                    }
                    assert!(line.len() < 8192);
                    let (name, value) = line.split_once(':').unwrap();
                    headers.insert(name.to_lowercase(), value.trim().to_owned());
                }
                let count = headers
                    .get("content-length")
                    .map_or(0, |v| v.parse::<usize>().unwrap());
                assert!(count < 64 * 1024);
                let mut body = vec![0; count];
                reader.read_exact(&mut body).unwrap();
                if path == "/mcp" && body.is_empty() {
                    let _=stream.write_all(format!("HTTP/1.1 401 Fixture\r\nWWW-Authenticate: Bearer resource_metadata=\"{base}/resource-metadata\"\r\nContent-Length: 0\r\nConnection: close\r\n\r\n").as_bytes());
                    continue;
                }
                if path != "/mcp" {
                    let response=match path.as_str() {
                        "/resource-metadata"=>json!({"resource":format!("{base}/mcp"),"authorization_servers":[format!("{base}/issuer")]}),
                        "/.well-known/oauth-authorization-server/issuer"=>json!({"issuer":format!("{base}/issuer"),"authorization_endpoint":format!("{base}/authorize"),"token_endpoint":format!("{base}/token"),"registration_endpoint":format!("{base}/register"),"response_types_supported":["code"],"token_endpoint_auth_methods_supported":["none"],"code_challenge_methods_supported":["S256"],"authorization_response_iss_parameter_supported":true}),
                        "/register"=>json!({"client_id":"public-native-client","token_endpoint_auth_method":"none","redirect_uris":serde_json::from_slice::<Value>(&body).unwrap()["redirect_uris"]}),
                        "/token"=>json!({"token_type":"Bearer","access_token":"public-oauth-access","refresh_token":"public-oauth-refresh","expires_in":3600}),
                        _=>panic!("unexpected public OAuth fixture path"),
                    }.to_string();
                    let head=format!("HTTP/1.1 200 Fixture\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{response}",response.len());
                    let _ = stream.write_all(head.as_bytes());
                    continue;
                }
                let request: Value = serde_json::from_slice(&body).unwrap();
                sender.send(request.clone()).ok();
                if held.load(SeqCst) {
                    pending.push(stream);
                    continue;
                }
                let authorized = expected.lock().unwrap().as_ref().is_none_or(|key| {
                    headers.get("authorization") == Some(&format!("Bearer {key}"))
                });
                let (status, response) = if !authorized {
                    (401, json!({"error":"public auth refusal"}))
                } else {
                    let result = match request["method"].as_str().unwrap() {
                        "server/discover" => {
                            json!({"resultType":"complete","supportedVersions":["2026-07-28"],"capabilities":{"tools":{}},"_meta":{"io.modelcontextprotocol/serverInfo":{"name":"Public remote fixture","version":"1"}}})
                        }
                        "tools/list" => {
                            json!({"resultType":"complete","tools":[{"name":"echo","description":"Public echo","inputSchema":{"type":"object"}}]})
                        }
                        "tools/call" => {
                            json!({"resultType":"complete","content":[{"type":"text","text":"PUBLIC_OAUTH_RESULT"}]})
                        }
                        _ => panic!("inspection must not invoke a tool"),
                    };
                    (
                        200,
                        json!({"jsonrpc":"2.0","id":request["id"],"result":result}),
                    )
                };
                let body = response.to_string();
                let head=format!("HTTP/1.1 {status} Fixture\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",body.len());
                if stream.write_all(head.as_bytes()).is_ok() {
                    let _ = stream.write_all(body.as_bytes());
                }
            }
        });
        Self {
            url,
            stop,
            hold,
            key,
            arrivals,
            worker: Some(worker),
        }
    }
    fn target(&self) -> McpRemoteTarget {
        McpRemoteTarget {
            label: "Public remote".into(),
            endpoint: self.url.clone(),
            authentication: if self.key.lock().unwrap().is_some() {
                McpRemoteAuthentication::Bearer
            } else {
                McpRemoteAuthentication::None
            },
        }
    }
}
impl Drop for Server {
    fn drop(&mut self) {
        self.stop.store(true, SeqCst);
        let _ = TcpStream::connect(
            self.url
                .trim_start_matches("http://")
                .trim_end_matches("/mcp"),
        );
        if let Some(worker) = self.worker.take() {
            worker.join().unwrap();
        }
    }
}
fn call(engine: &Engine, value: Value) -> Result<Value, String> {
    engine.call(serde_json::from_value(value).unwrap())
}
fn done(engine: &Engine, id: u64) -> Value {
    let deadline = Instant::now() + Duration::from_secs(10);
    loop {
        let events = call(engine, json!({"command":"poll","id":id})).unwrap();
        if let Some(event) = events
            .as_array()
            .unwrap()
            .iter()
            .find(|e| e["type"] == "done")
        {
            return event.clone();
        }
        assert!(Instant::now() < deadline, "inspection did not finish");
        thread::sleep(Duration::from_millis(1));
    }
}
fn inspect(
    engine: &Engine,
    server: &Server,
    id: u64,
    connection: &str,
    key: Option<&str>,
) -> Value {
    call(engine,json!({"command":"inspectRemoteMcp","id":id,"session":"work","connectionId":connection,"remote":server.target(),"bearer":key})).unwrap();
    done(engine, id)
}
fn setup() -> (
    tempfile::TempDir,
    Arc<SqliteStore>,
    Arc<crate::connection::testing::MemoryCredentials>,
    Engine,
    String,
) {
    let directory = tempfile::tempdir().unwrap();
    let root = directory.path().join("project");
    std::fs::create_dir(&root).unwrap();
    let root = root.canonicalize().unwrap().to_str().unwrap().to_owned();
    let store = Arc::new(SqliteStore::open(&directory.path().join("fixture.db")).unwrap());
    for session in ["work", "other"] {
        store
            .create_workspace_session(
                session,
                &SessionWorkspace {
                    kind: WorkspaceKind::Project,
                    root: Some(root.clone()),
                },
            )
            .unwrap();
    }
    let vault = Arc::new(crate::connection::testing::MemoryCredentials::default());
    let engine = Engine::new(store.clone(), vault.clone()).unwrap();
    (directory, store, vault, engine, root)
}
fn enable(engine: &Engine, review: &Value) -> Result<Value, String> {
    call(
        engine,
        json!({"command":"enableMcp","session":"work","token":review["token"],"names":["echo"]}),
    )
}

fn fixture_browser(url: &str) -> Result<(), String> {
    let authorization = url::Url::parse(url).unwrap();
    let fields: std::collections::BTreeMap<_, _> = authorization
        .query_pairs()
        .map(|(k, v)| (k.into_owned(), v.into_owned()))
        .collect();
    let mut callback = url::Url::parse(&fields["redirect_uri"]).unwrap();
    let mut issuer = authorization.clone();
    issuer.set_path("/issuer");
    issuer.set_query(None);
    callback
        .query_pairs_mut()
        .append_pair("state", &fields["state"])
        .append_pair("iss", issuer.as_str())
        .append_pair("code", "public-code");
    thread::spawn(move || {
        let mut socket = TcpStream::connect(("127.0.0.1", callback.port().unwrap())).unwrap();
        socket
            .set_read_timeout(Some(Duration::from_secs(5)))
            .unwrap();
        let request = format!(
            "GET {}?{} HTTP/1.1\r\nHost: 127.0.0.1:{}\r\nConnection: close\r\n\r\n",
            callback.path(),
            callback.query().unwrap(),
            callback.port().unwrap()
        );
        socket.write_all(request.as_bytes()).unwrap();
        let mut response = String::new();
        socket.read_to_string(&mut response).ok();
    });
    Ok(())
}

#[test]
fn oauth_sign_in_review_vault_save_retry_reopen_and_approved_call() {
    let server = Server::new(Some("public-oauth-access"));
    let (directory, store, vault, mut engine, root) = setup();
    engine.mcp_browser = Arc::new(fixture_browser);
    let mut target = server.target();
    target.authentication = McpRemoteAuthentication::Oauth;
    call(
        &engine,
        json!({"command":"inspectRemoteMcp","id":41,"session":"work","remote":target,"signIn":{}}),
    )
    .unwrap();
    let response = done(&engine, 41);
    assert!(response.get("error").is_none(), "{response}");
    let review = &response["mcpInspection"];
    assert!(!response.to_string().contains("public-oauth-access"));
    assert!(!response.to_string().contains("public-oauth-refresh"));
    assert!(store.mcp_connections(&root).unwrap().is_empty());
    assert!(vault.values.lock().unwrap().is_empty());
    let db = rusqlite::Connection::open(directory.path().join("fixture.db")).unwrap();
    db.execute_batch("CREATE TRIGGER refuse_oauth BEFORE INSERT ON mcp_connections BEGIN SELECT RAISE(ABORT,'public fixture'); END;").unwrap();
    assert!(enable(&engine, review).is_err());
    assert!(engine.mcp_remote_review.lock().unwrap().is_some());
    assert!(vault.values.lock().unwrap().is_empty());
    db.execute_batch("DROP TRIGGER refuse_oauth").unwrap();
    vault.locked.store(true, SeqCst);
    assert!(enable(&engine, review).is_err());
    vault.locked.store(false, SeqCst);
    let saved = enable(&engine, review).unwrap();
    assert_eq!(saved["credentials"][0]["name"], "MCP_OAUTH_GRANT");
    let raw: String = db
        .query_row("SELECT data FROM mcp_connections", [], |row| row.get(0))
        .unwrap();
    assert!(!raw.contains("public-oauth-access"));
    assert!(!raw.contains("public-oauth-refresh"));
    let id = saved["id"].as_str().unwrap();
    call(&engine,json!({"command":"inspectRemoteMcp","id":42,"session":"work","connectionId":id,"remote":target})).unwrap();
    let recheck = done(&engine, 42);
    assert!(recheck.get("error").is_none(), "{recheck}");
    // A restarted owner reuses the secure login without opening a browser.
    let reopened = Engine::new(store.clone(), vault.clone()).unwrap();
    let saved = store.mcp_connection_by_id(&root, id).unwrap().unwrap();
    let plugin =
        dolores_tools_mcp::plugins_with_credentials(Path::new(&root), saved.clone(), vault.clone())
            .unwrap()
            .remove(0);
    let request = plugin
        .prepare(&dolores_core::ToolCall {
            id: "oauth-call".into(),
            name: plugin.spec().name,
            arguments: "{}".into(),
        })
        .unwrap();
    let result = reopened
        .runtime
        .block_on(plugin.invoke(&request, CancellationToken::new()))
        .unwrap();
    assert!(result.contains("PUBLIC_OAUTH_RESULT"));
    vault.locked.store(true, SeqCst);
    call(&reopened,json!({"command":"inspectRemoteMcp","id":43,"session":"work","connectionId":id,"remote":target})).unwrap();
    assert!(done(&reopened, 43)["error"]
        .as_str()
        .unwrap()
        .contains("Unlock"));
    vault.locked.store(false, SeqCst);
    assert_eq!(store.mcp_connection_by_id(&root, id).unwrap(), Some(saved));
}

#[test]
fn oauth_browser_failure_stop_and_stale_issuer_preserve_the_previous_connection() {
    let server = Server::new(Some("public-oauth-access"));
    let (_directory, store, _vault, mut engine, root) = setup();
    let mut target = server.target();
    target.authentication = McpRemoteAuthentication::Oauth;
    engine.mcp_browser = Arc::new(|_| Err("Public browser failure. Try Sign in again.".into()));
    call(
        &engine,
        json!({"command":"inspectRemoteMcp","id":51,"session":"work","remote":target,"signIn":{}}),
    )
    .unwrap();
    assert!(done(&engine, 51)["error"]
        .as_str()
        .unwrap()
        .contains("browser failure"));
    call(&engine,json!({"command":"inspectRemoteMcp","id":52,"session":"work","remote":target,"signIn":{"issuer":"https://wrong.example.test"}})).unwrap();
    assert!(done(&engine, 52)["error"]
        .as_str()
        .unwrap()
        .contains("issuer"));
    let (sender, receiver) = signal::channel();
    engine.mcp_browser = Arc::new(move |_| {
        sender.send(()).unwrap();
        Ok(())
    });
    call(
        &engine,
        json!({"command":"inspectRemoteMcp","id":53,"session":"work","remote":target,"signIn":{}}),
    )
    .unwrap();
    receiver.recv_timeout(Duration::from_secs(5)).unwrap();
    call(&engine, json!({"command":"cancel","id":53})).unwrap();
    assert!(done(&engine, 53).get("error").is_some());
    assert!(engine.mcp_remote_review.lock().unwrap().is_none());
    assert!(store.mcp_connections(&root).unwrap().is_empty());
}

#[test]
fn remote_inspection_save_reuse_rotation_and_locked_forget_preserve_recovery() {
    let server = Server::new(Some("public-fixture-key"));
    let (directory, store, vault, engine, root) = setup();
    let event = inspect(&engine, &server, 1, "", Some("public-fixture-key"));
    let review = &event["mcpInspection"];
    assert!(event.get("error").is_none(), "{event}");
    assert!(!event.to_string().contains("public-fixture-key"));
    assert!(store.mcp_connections(&root).unwrap().is_empty());
    assert!(vault.values.lock().unwrap().is_empty());
    assert!(call(
        &engine,
        json!({"command":"enableMcp","session":"other","token":review["token"],"names":["echo"]})
    )
    .is_err());
    vault.locked.store(true, SeqCst);
    assert!(enable(&engine, review).is_err());
    vault.locked.store(false, SeqCst);
    let db = rusqlite::Connection::open(directory.path().join("fixture.db")).unwrap();
    db.execute_batch("CREATE TRIGGER refuse_remote BEFORE INSERT ON mcp_connections BEGIN SELECT RAISE(ABORT,'public fixture save refusal'); END;").unwrap();
    assert!(enable(&engine, review).is_err());
    assert!(vault.values.lock().unwrap().is_empty());
    assert!(engine.mcp_remote_review.lock().unwrap().is_some());
    assert!(store.mcp_connections(&root).unwrap().is_empty());
    db.execute_batch("DROP TRIGGER refuse_remote").unwrap();
    let saved = enable(&engine, review).unwrap();
    let id = saved["id"].as_str().unwrap();
    let first = store.mcp_connection_by_id(&root, id).unwrap().unwrap();
    assert_eq!(first.credentials.len(), 1);
    let raw: String = db
        .query_row("SELECT data FROM mcp_connections WHERE id=?1", [id], |r| {
            r.get(0)
        })
        .unwrap();
    assert!(!raw.contains("public-fixture-key"));
    assert!(saved.get("launch").is_none());
    assert!(enable(&engine, review).is_err());
    let reused = inspect(&engine, &server, 2, id, None);
    assert!(reused.get("error").is_none(), "{reused}");
    assert_eq!(vault.values.lock().unwrap().len(), 1);
    *server.key.lock().unwrap() = Some("public-rotated-key".into());
    let expired = inspect(&engine, &server, 3, id, None);
    assert!(expired["error"]
        .as_str()
        .unwrap()
        .contains("authentication"));
    assert_eq!(
        store.mcp_connection_by_id(&root, id).unwrap(),
        Some(first.clone())
    );
    let rotated = inspect(&engine, &server, 4, id, Some("public-rotated-key"));
    assert!(rotated.get("error").is_none(), "{rotated}");
    db.execute_batch("CREATE TRIGGER refuse_remote BEFORE UPDATE ON mcp_connections BEGIN SELECT RAISE(ABORT,'public fixture save refusal'); END;").unwrap();
    assert!(enable(&engine, &rotated["mcpInspection"]).is_err());
    assert_eq!(vault.values.lock().unwrap().len(), 1);
    assert_eq!(
        store.mcp_connection_by_id(&root, id).unwrap(),
        Some(first.clone())
    );
    db.execute_batch("DROP TRIGGER refuse_remote").unwrap();
    let saved = enable(&engine, &rotated["mcpInspection"]).unwrap();
    assert_eq!(saved["revision"], 2);
    assert_eq!(vault.values.lock().unwrap().len(), 1);
    assert!(!vault
        .values
        .lock()
        .unwrap()
        .contains_key(&first.credentials[0].credential_id));
    vault.locked.store(true, SeqCst);
    assert!(call(
        &engine,
        json!({"command":"forgetMcp","session":"work","connectionId":id,"revision":2})
    )
    .is_err());
    let disabled = store.mcp_connection_by_id(&root, id).unwrap().unwrap();
    assert!(!disabled.enabled);
    assert_eq!(disabled.revision, 3);
    vault.locked.store(false, SeqCst);
    call(
        &engine,
        json!({"command":"forgetMcp","session":"work","connectionId":id,"revision":3}),
    )
    .unwrap();
    assert!(store.mcp_connections(&root).unwrap().is_empty());
    assert!(vault.values.lock().unwrap().is_empty());
}

#[test]
fn remote_review_expiry_previous_snapshot_and_stop_refuse_without_saving() {
    let server = Server::new(None);
    let (_directory, store, vault, engine, root) = setup();
    let event = inspect(&engine, &server, 11, "", None);
    assert!(event.get("error").is_none(), "{event}");
    engine
        .mcp_remote_review
        .lock()
        .unwrap()
        .as_mut()
        .unwrap()
        .created = Instant::now() - Duration::from_secs(301);
    assert!(enable(&engine, &event["mcpInspection"]).is_err());
    assert!(store.mcp_connections(&root).unwrap().is_empty());
    let event = inspect(&engine, &server, 12, "", None);
    let changed = {
        let slot = engine.mcp_remote_review.lock().unwrap();
        let r = slot.as_ref().unwrap();
        McpConnection {
            id: r.connection_id.clone(),
            revision: 1,
            enabled: true,
            launch: None,
            remote: Some(r.target.clone()),
            fingerprints: vec![],
            protocol_version: r.protocol.clone(),
            server_name: r.server_name.clone(),
            server_version: r.server_version.clone(),
            tools: r.tools.clone(),
            credentials: vec![],
            retired_credentials: vec![],
        }
    };
    store.save_mcp_connection(&root, &changed, None).unwrap();
    assert!(enable(&engine, &event["mcpInspection"]).is_err());
    assert!(vault.values.lock().unwrap().is_empty());
    store
        .mutate_mcp_connection_by_id(&root, &changed.id, 1, true)
        .unwrap();
    while server.arrivals.try_recv().is_ok() {}
    server.hold.store(true, SeqCst);
    call(&engine,json!({"command":"inspectRemoteMcp","id":13,"session":"work","connectionId":"","remote":server.target()})).unwrap();
    assert_eq!(
        server
            .arrivals
            .recv_timeout(Duration::from_secs(5))
            .unwrap()["method"],
        "server/discover"
    );
    call(&engine, json!({"command":"cancel","id":13})).unwrap();
    let stopped = done(&engine, 13);
    assert!(stopped["error"].as_str().unwrap().contains("stopped"));
    assert!(engine.mcp_remote_review.lock().unwrap().is_none());
    assert!(store.mcp_connections(&root).unwrap().is_empty());
}
