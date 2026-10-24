use super::*;
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
use serde_json::json;
use sha2::{Digest, Sha256};
use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc, Mutex,
};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpListener,
    sync::Notify,
};

#[derive(Clone)]
struct Request {
    path: String,
    method: String,
    headers: String,
    body: String,
}
impl Request {
    fn form(&self) -> std::collections::BTreeMap<String, String> {
        Url::parse(&format!("http://fixture.invalid/?{}", self.body))
            .unwrap()
            .query_pairs()
            .map(|(k, v)| (k.into_owned(), v.into_owned()))
            .collect()
    }
}
struct Reply {
    status: u16,
    body: String,
    headers: String,
    hold: Option<Arc<Notify>>,
}
impl Reply {
    fn json(value: Value) -> Self {
        Self {
            status: 200,
            body: value.to_string(),
            headers: "Content-Type: application/json\r\n".into(),
            hold: None,
        }
    }
    fn status(status: u16) -> Self {
        Self {
            status,
            ..Self::json(json!({}))
        }
    }
}
struct Fixture {
    base: String,
    seen: Arc<Mutex<Vec<Request>>>,
    task: tokio::task::JoinHandle<()>,
}
impl Drop for Fixture {
    fn drop(&mut self) {
        self.task.abort();
    }
}
impl Fixture {
    async fn start(handler: impl Fn(&str, &Request) -> Reply + Send + 'static) -> Self {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let base = format!("http://{}", listener.local_addr().unwrap());
        let fixture_base = base.clone();
        let seen = Arc::new(Mutex::new(vec![]));
        let recorded = seen.clone();
        let task = tokio::spawn(async move {
            loop {
                let (mut socket, _) = listener.accept().await.unwrap();
                let mut data = Vec::new();
                let header_end = loop {
                    let mut bytes = [0; 2048];
                    let n = socket.read(&mut bytes).await.unwrap();
                    if n == 0 {
                        break None;
                    }
                    data.extend_from_slice(&bytes[..n]);
                    assert!(data.len() <= 16 * 1024);
                    if let Some(end) = data.windows(4).position(|b| b == b"\r\n\r\n") {
                        break Some(end + 4);
                    }
                };
                let Some(header_end) = header_end else {
                    continue;
                };
                let headers = String::from_utf8(data[..header_end].to_vec()).unwrap();
                let length = headers
                    .lines()
                    .find_map(|s| {
                        s.split_once(':')
                            .filter(|(k, _)| k.eq_ignore_ascii_case("content-length"))
                            .map(|(_, v)| v.trim().parse::<usize>().unwrap())
                    })
                    .unwrap_or(0);
                while data.len() < header_end + length {
                    let mut bytes = [0; 2048];
                    let n = socket.read(&mut bytes).await.unwrap();
                    assert!(n > 0);
                    data.extend_from_slice(&bytes[..n]);
                }
                let mut first = headers.lines().next().unwrap().split_whitespace();
                let request = Request {
                    method: first.next().unwrap().into(),
                    path: first.next().unwrap().into(),
                    headers,
                    body: String::from_utf8(data[header_end..header_end + length].to_vec())
                        .unwrap(),
                };
                recorded.lock().unwrap().push(request.clone());
                let reply = handler(&fixture_base, &request);
                if let Some(hold) = reply.hold {
                    hold.notified().await;
                }
                let response = format!(
                    "HTTP/1.1 {} Fixture\r\n{}Content-Length: {}\r\nConnection: close\r\n\r\n{}",
                    reply.status,
                    reply.headers,
                    reply.body.len(),
                    reply.body
                );
                let _ = socket.write_all(response.as_bytes()).await;
            }
        });
        Self { base, seen, task }
    }
    fn count(&self) -> usize {
        self.seen.lock().unwrap().len()
    }
}
fn metadata(base: &str) -> Value {
    json!({"issuer":format!("{base}/tenant"), "authorization_endpoint":format!("{base}/authorize"), "token_endpoint":format!("{base}/token"), "registration_endpoint":format!("{base}/register"), "code_challenge_methods_supported":["S256"], "response_types_supported":["code"], "grant_types_supported":["authorization_code","refresh_token"], "token_endpoint_auth_methods_supported":["none"], "authorization_response_iss_parameter_supported":true})
}
fn resource(base: &str) -> Value {
    json!({"resource":format!("{base}/mcp"), "authorization_servers":[format!("{base}/tenant")], "scopes_supported":["tools:read"], "bearer_methods_supported":["header"]})
}
fn route(base: &str, request: &Request) -> Reply {
    match request.path.as_str() {
        "/.well-known/oauth-protected-resource/mcp" => Reply::json(resource(base)),
        "/.well-known/oauth-authorization-server/tenant" => Reply::json(metadata(base)),
        _ => Reply::status(404),
    }
}
async fn discover(fixture: &Fixture) -> (Operation, AuthorizationServer) {
    let mut operation = Operation::new(CancellationToken::new());
    let resource = operation
        .discover_resource(&format!("{}/mcp", fixture.base), None)
        .await
        .unwrap();
    let server = operation
        .discover_server(&resource, &resource.issuers()[0])
        .await
        .unwrap();
    (operation, server)
}
async fn registered(operation: &mut Operation, server: &AuthorizationServer) -> RegisteredClient {
    operation
        .register_client(
            server,
            "http://127.0.0.1:49152/oauth/callback",
            &ClientConfiguration {
                pre_registered: Some(PreRegisteredClient {
                    issuer: server.issuer().into(),
                    client_id: "public-fixture-client".into(),
                }),
                metadata_document: None,
            },
        )
        .await
        .unwrap()
}
fn callback(pending: &PendingAuthorization, issuer: Option<&str>) -> String {
    let authorization = Url::parse(pending.authorization_url()).unwrap();
    let state = authorization
        .query_pairs()
        .find(|(k, _)| k == "state")
        .unwrap()
        .1
        .into_owned();
    let mut callback = Url::parse(pending.redirect_uri()).unwrap();
    callback
        .query_pairs_mut()
        .append_pair("state", &state)
        .append_pair("code", "public-fixture-code");
    if let Some(issuer) = issuer {
        callback.query_pairs_mut().append_pair("iss", issuer);
    }
    callback.to_string()
}
fn kind<T>(result: Result<T>) -> Kind {
    match result {
        Err(Failure(kind)) => kind,
        Ok(_) => panic!("unexpected success"),
    }
}

#[tokio::test]
async fn native_registration_pkce_exchange_rotation_and_revoked_refresh() {
    let exchanges = Arc::new(AtomicUsize::new(0));
    let expected_challenge = Arc::new(Mutex::new(String::new()));
    let observed_challenge = expected_challenge.clone();
    let count = exchanges.clone();
    let fixture = Fixture::start(move |base, request| {
        assert!(!request.headers.to_ascii_lowercase().contains("authorization:"));
        match request.path.as_str() {
            "/register" => {
                assert_eq!(request.method, "POST");
                let value:Value = serde_json::from_str(&request.body).unwrap();
                assert_eq!(value["application_type"], "native");
                assert_eq!(value["grant_types"], json!(["authorization_code","refresh_token"]));
                Reply::json(json!({"client_id":"registered-fixture", "redirect_uris":value["redirect_uris"], "token_endpoint_auth_method":"none"}))
            }
            "/token" => {
                let fields = request.form();
                assert_eq!(fields["resource"], format!("{base}/mcp"));
                assert_eq!(fields["client_id"], "registered-fixture");
                let n = count.fetch_add(1, Ordering::SeqCst);
                if n == 0 {
                    assert_eq!(fields["grant_type"], "authorization_code");
                    assert_eq!(fields["code"], "public-fixture-code");
                    assert_eq!(fields["redirect_uri"], "http://127.0.0.1:49152/oauth/callback");
                    assert_eq!(fields["code_verifier"].len(), 64);
                    assert_eq!(URL_SAFE_NO_PAD.encode(Sha256::digest(fields["code_verifier"].as_bytes())), *observed_challenge.lock().unwrap());
                    Reply::json(json!({"access_token":"fixture-access-v1", "refresh_token":"fixture-refresh-v1", "token_type":"Bearer", "expires_in":3600, "scope":"tools:read"}))
                } else {
                    assert_eq!(fields["grant_type"], "refresh_token");
                    assert_eq!(fields["scope"], "tools:read");
                    assert_eq!(fields["refresh_token"], if n == 1 { "fixture-refresh-v1" } else { "fixture-refresh-v2" });
                    match n {
                        1 => Reply::json(json!({"access_token":"fixture-access-v2", "refresh_token":"fixture-refresh-v2", "token_type":"bearer", "expires_in":3600})),
                        2 => Reply::json(json!({"access_token":"fixture-access-v3", "token_type":"Bearer"})),
                        _ => Reply { status:400, ..Reply::json(json!({"error":"invalid_grant","error_description":"private-server-detail"})) },
                    }
                }
            }
            _ => route(base, request),
        }
    }).await;
    let (mut operation, server) = discover(&fixture).await;
    let client = operation
        .register_client(
            &server,
            "http://127.0.0.1:49152/oauth/callback",
            &ClientConfiguration::default(),
        )
        .await
        .unwrap();
    let pending = PendingAuthorization::new(server.clone(), client.clone()).unwrap();
    let authorization = Url::parse(pending.authorization_url()).unwrap();
    let fields: std::collections::BTreeMap<_, _> = authorization.query_pairs().collect();
    assert_eq!(fields["code_challenge_method"], "S256");
    assert_eq!(fields["resource"], format!("{}/mcp", fixture.base));
    assert_eq!(fields["scope"], "tools:read");
    assert_eq!(fields["state"].len(), 64);
    *expected_challenge.lock().unwrap() = fields["code_challenge"].to_string();
    let response = callback(&pending, Some(server.issuer()));
    let tokens = pending.complete(&response, &mut operation).await.unwrap();
    assert!(tokens.can_refresh());
    assert!(tokens.expires_at().is_some());
    assert!(tokens.contains_secret(&json!({"echo":"fixture-access-v1"})));
    assert!(tokens.contains_secret(&json!({"fixture-refresh-v1":"echo"})));
    let next = operation.refresh(&server, &client, &tokens).await.unwrap();
    assert!(next.contains_secret(&json!("fixture-refresh-v2")));
    assert!(!next.contains_secret(&json!("fixture-refresh-v1")));
    let retained_refresh = operation.refresh(&server, &client, &next).await.unwrap();
    assert!(retained_refresh.contains_secret(&json!("fixture-refresh-v2")));
    assert!(retained_refresh.expires_at().is_none());
    let error = operation
        .refresh(&server, &client, &retained_refresh)
        .await
        .err()
        .unwrap();
    assert_eq!(error.0, Kind::SignInRequired);
    assert!(!format!("{error:?}").contains("private-server-detail"));
    assert!(retained_refresh.contains_secret(&json!("fixture-access-v3")));
    assert_eq!(exchanges.load(Ordering::SeqCst), 4);
    assert_eq!(fixture.count(), 7); // metadata x2, registration, exchange, refresh x3
}

#[tokio::test]
async fn challenge_discovery_scope_priority_and_oidc_fallback_order() {
    let fixture = Fixture::start(|base, request| match request.path.as_str() {
        "/custom-metadata" => Reply::json(resource(base)),
        "/tenant/.well-known/openid-configuration" => Reply::json(metadata(base)),
        _ => Reply::status(404),
    })
    .await;
    let mut operation = Operation::new(CancellationToken::new());
    let challenge = format!("Basic realm=\"unrelated, realm\", Bearer resource_metadata=\"{}/custom-metadata\", scope=\"tools:write tools:extra\"", fixture.base);
    let resource = operation
        .discover_resource(&format!("{}/mcp", fixture.base), Some(&challenge))
        .await
        .unwrap();
    assert_eq!(resource.scopes(), &["tools:extra", "tools:write"]);
    assert_eq!(resource.resource(), format!("{}/mcp", fixture.base));
    let server = operation
        .discover_server(&resource, &resource.issuers()[0])
        .await
        .unwrap();
    assert_eq!(server.scopes(), resource.scopes());
    let paths: Vec<_> = fixture
        .seen
        .lock()
        .unwrap()
        .iter()
        .map(|r| r.path.clone())
        .collect();
    assert_eq!(
        paths,
        [
            "/custom-metadata",
            "/.well-known/oauth-authorization-server/tenant",
            "/.well-known/openid-configuration/tenant",
            "/tenant/.well-known/openid-configuration"
        ]
    );
    let before = fixture.count();
    assert_eq!(
        kind(
            operation
                .discover_server(&resource, "https://different.invalid")
                .await
        ),
        Kind::Issuer
    );
    let mismatch = ClientConfiguration {
        pre_registered: Some(PreRegisteredClient {
            issuer: "https://different.invalid".into(),
            client_id: "do-not-forward".into(),
        }),
        metadata_document: None,
    };
    assert_eq!(
        kind(
            operation
                .register_client(&server, "http://127.0.0.1:49152/callback", &mismatch)
                .await
        ),
        Kind::Issuer
    );
    assert_eq!(fixture.count(), before);
}

#[tokio::test]
async fn root_resource_fallback_and_strict_metadata_refusals() {
    let mode = Arc::new(AtomicUsize::new(0));
    let current = mode.clone();
    let fixture = Fixture::start(move |base, request| match request.path.as_str() {
        "/.well-known/oauth-protected-resource" => {
            let mut value = resource(base);
            if current.load(Ordering::SeqCst) == 4 {
                value["resource"] = json!(format!("{base}/other"));
            }
            Reply::json(value)
        }
        "/.well-known/oauth-authorization-server/tenant" => {
            let mut value = metadata(base);
            match current.load(Ordering::SeqCst) {
                1 => {
                    value["issuer"] = json!(format!("{base}/tenant/"));
                }
                2 => {
                    value
                        .as_object_mut()
                        .unwrap()
                        .remove("code_challenge_methods_supported");
                }
                3 => {
                    value["token_endpoint_auth_methods_supported"] = json!(["client_secret_basic"]);
                }
                _ => {}
            }
            Reply::json(value)
        }
        _ => Reply::status(404),
    })
    .await;
    for (m, expected) in [
        (0, None),
        (1, Some(Kind::Issuer)),
        (2, Some(Kind::Pkce)),
        (3, Some(Kind::Registration)),
        (4, Some(Kind::Resource)),
    ] {
        mode.store(m, Ordering::SeqCst);
        let mut operation = Operation::new(CancellationToken::new());
        let resource = operation
            .discover_resource(&format!("{}/mcp", fixture.base), None)
            .await;
        if m == 4 {
            assert_eq!(kind(resource), Kind::Resource);
            continue;
        }
        let resource = resource.unwrap();
        let result = operation
            .discover_server(&resource, &resource.issuers()[0])
            .await;
        if let Some(expected) = expected {
            assert_eq!(kind(result), expected);
        } else {
            assert!(result.is_ok());
        }
    }
    assert_eq!(
        fixture.seen.lock().unwrap()[0].path,
        "/.well-known/oauth-protected-resource/mcp"
    );
    assert_eq!(
        fixture.seen.lock().unwrap()[1].path,
        "/.well-known/oauth-protected-resource"
    );
}

#[tokio::test]
async fn callback_state_issuer_and_errors_are_validated_before_exchange() {
    let fixture = Fixture::start(route).await;
    let (mut operation, mut server) = discover(&fixture).await;
    // An exact string with case/default port must survive form decoding without
    // normalization, including when the callback carries an error.
    server.issuer = "https://Example.invalid:443/tenant".into();
    let client = registered(&mut operation, &server).await;
    let pending = PendingAuthorization::new(server.clone(), client.clone()).unwrap();
    let correct = callback(&pending, Some(server.issuer()));
    assert_eq!(pending.code(&correct).unwrap(), "public-fixture-code");
    for issuer in [
        None,
        Some("https://example.invalid/tenant"),
        Some("https://Example.invalid:443/tenant/"),
    ] {
        assert_eq!(
            kind(pending.code(&callback(&pending, issuer))),
            Kind::Issuer
        );
    }
    let bad_state = correct.replace("state=", "state=wrong");
    assert_eq!(kind(pending.code(&bad_state)), Kind::Callback);
    assert_eq!(
        kind(pending.code(&format!("{correct}&state=duplicate"))),
        Kind::Callback
    );
    assert_eq!(
        kind(pending.code(&format!("{correct}&code=duplicate"))),
        Kind::Callback
    );
    assert_eq!(
        kind(pending.code(&format!("{correct}&bad=%zz"))),
        Kind::Callback
    );
    assert_eq!(
        kind(pending.code(&format!("{correct}&bad=%ff"))),
        Kind::Callback
    );
    let error = format!(
        "{}&error=access_denied&error_description=private-message",
        callback(&pending, Some("https://wrong.invalid"))
    );
    assert_eq!(kind(pending.code(&error)), Kind::Issuer);
    let denial = format!("{correct}&error=access_denied&error_description=private-message");
    let before = fixture.count();
    let error = pending
        .complete(&denial, &mut operation)
        .await
        .err()
        .unwrap();
    assert_eq!(error.0, Kind::Denied);
    assert!(!error.to_string().contains("private-message"));
    assert_eq!(fixture.count(), before);
    server.issuer_in_callback = false;
    let pending = PendingAuthorization::new(server.clone(), client.clone()).unwrap();
    assert!(pending.code(&callback(&pending, None)).is_ok());
    assert_eq!(
        kind(pending.code(&callback(&pending, Some("https://wrong.invalid")))),
        Kind::Issuer
    );
    tokio::time::pause();
    tokio::time::advance(Duration::from_secs(301)).await;
    let response = callback(&pending, Some(server.issuer()));
    assert_eq!(
        kind(
            pending
                .complete(&response, &mut Operation::new(CancellationToken::new()))
                .await
        ),
        Kind::AttemptExpired
    );
    tokio::time::resume();
}

#[test]
fn untrusted_challenges_and_insecure_endpoints_are_refused() {
    for value in [
        "Bearer scope=\"a\", scope=\"b\"",
        "Bearer scope=\"a\", Bearer scope=\"b\"",
        "Bearer scope=\"unterminated",
        "Bearer scope=\"x\"suffix",
        "Bearer resource_metadata=\"https://example.invalid/\r\nx\"",
    ] {
        assert_eq!(kind(challenge::parse(Some(value))), Kind::Challenge);
    }
    assert_eq!(
        challenge::parse(Some("Digest realm=\"not Bearer\", Bearer scope=\"x\\! y\""))
            .unwrap()
            .scopes
            .unwrap(),
        ["x!", "y"]
    );
    assert!(oauth_url("http://127.0.0.1:9000/as", false).is_err());
    assert!(oauth_url("http://public.invalid/as", true).is_err());
    assert!(oauth_url("https://user:password@public.invalid/as", false).is_err());
    for redirect in [
        "http://localhost:9000/callback",
        "https://127.0.0.1:9000/callback",
        "http://127.0.0.1:9000/",
        "http://127.0.0.1:9000/callback?x=y",
    ] {
        assert_eq!(kind(discovery::loopback_redirect(redirect)), Kind::Callback);
    }
}

#[tokio::test]
async fn redirects_malformed_and_oversized_metadata_do_not_fall_through() {
    for mode in 0..5 {
        let fixture = Fixture::start(move |base, request| match mode {
            0 => Reply {
                headers: format!("Location: {base}/do-not-follow\r\n"),
                ..Reply::status(302)
            },
            1 => Reply {
                body: "{malformed".into(),
                ..Reply::json(json!({}))
            },
            2 => Reply::json(json!(["wrong-shape"])),
            3 => Reply::json(json!({"padding":"x".repeat(DOCUMENT)})),
            _ => Reply {
                headers: "Content-Type: Application/Mcp+Json; charset=UTF-8\r\n".into(),
                ..route(base, request)
            },
        })
        .await;
        let result = Operation::new(CancellationToken::new())
            .discover_resource(&format!("{}/mcp", fixture.base), None)
            .await;
        if mode == 4 {
            assert!(result.is_ok());
            assert_eq!(fixture.count(), 1);
            continue;
        }
        assert_eq!(
            kind(result),
            match mode {
                0 => Kind::Redirect,
                3 => Kind::Limit,
                _ => Kind::Metadata,
            }
        );
        assert_eq!(fixture.count(), 1);
    }
}

#[tokio::test]
async fn stopped_and_expired_phases_dispatch_nothing_and_inflight_stop_recovers() {
    let arrived = Arc::new(Notify::new());
    let signal = arrived.clone();
    let fixture = Fixture::start(move |_, _| {
        signal.notify_one();
        Reply {
            hold: Some(Arc::new(Notify::new())),
            ..Reply::json(json!({}))
        }
    })
    .await;
    let cancel = CancellationToken::new();
    cancel.cancel();
    let endpoint = format!("{}/mcp", fixture.base);
    assert_eq!(
        kind(
            Operation::new(cancel)
                .discover_resource(&endpoint, None)
                .await
        ),
        Kind::Cancelled
    );
    assert_eq!(
        kind(
            Operation::until(CancellationToken::new(), Instant::now())
                .discover_resource(&endpoint, None)
                .await
        ),
        Kind::Deadline
    );
    assert_eq!(fixture.count(), 0);
    let cancel = CancellationToken::new();
    let active = cancel.clone();
    let endpoint_copy = endpoint.clone();
    let running = tokio::spawn(async move {
        Operation::new(active)
            .discover_resource(&endpoint_copy, None)
            .await
    });
    arrived.notified().await;
    cancel.cancel();
    assert_eq!(kind(running.await.unwrap()), Kind::Cancelled);
    assert_eq!(fixture.count(), 1);
    // A fresh independent phase can proceed; Stop does not poison a shared client.
    let next = Fixture::start(route).await;
    assert!(Operation::new(CancellationToken::new())
        .discover_resource(&format!("{}/mcp", next.base), None)
        .await
        .is_ok());
}

#[test]
fn hosted_client_metadata_binds_identity_and_loopback_path_but_allows_port_change() {
    let document = "https://client.invalid/oauth/client.json";
    let value = json!({"client_id":document,"client_name":"Dolores", "redirect_uris":["http://127.0.0.1/oauth/callback"], "token_endpoint_auth_method":"none", "grant_types":["authorization_code","refresh_token"], "response_types":["code"]});
    assert!(discovery::validate_client_document(
        &value,
        document,
        "http://127.0.0.1:49152/oauth/callback"
    )
    .is_ok());
    for redirect in [
        "http://127.0.0.1:49152/wrong",
        "http://127.0.0.2:49152/oauth/callback",
        "http://[::1]:49152/oauth/callback",
    ] {
        assert_eq!(
            kind(discovery::validate_client_document(
                &value, document, redirect
            )),
            Kind::Registration
        );
    }
    assert_eq!(
        kind(discovery::validate_client_document(
            &value,
            "https://other.invalid/client.json",
            "http://127.0.0.1:49152/oauth/callback"
        )),
        Kind::Registration
    );
    let mut wrong = value.clone();
    wrong["response_types"] = json!(["token"]);
    assert_eq!(
        kind(discovery::validate_client_document(
            &wrong,
            document,
            "http://127.0.0.1:49152/oauth/callback"
        )),
        Kind::Registration
    );
}

#[tokio::test]
async fn bad_token_replies_and_registration_failures_preserve_prior_tokens() {
    let mode = Arc::new(AtomicUsize::new(0));
    let current = mode.clone();
    let fixture = Fixture::start(move |base,request| {
        match request.path.as_str() {
            "/token" => {
                let mut value = json!({"access_token":"public-fixture-access", "refresh_token":"public-fixture-refresh", "token_type":"Bearer", "expires_in":3600});
                match current.load(Ordering::SeqCst) {
                    1 => { value["access_token"] = json!("bad\nheader"); }
                    2 => { value["token_type"] = json!("MAC"); }
                    3 => { value["expires_in"] = json!(-1); }
                    4 => { value["scope"] = json!("unrequested:write"); }
                    5 => { value["refresh_token"] = json!("x".repeat(4097)); }
                    6 => { return Reply {status:302,headers:format!("Location: {base}/do-not-forward\r\n"),..Reply::json(json!({}))}; }
                    7 => { value["scope"] = json!("default:read"); }
                    _ => {}
                }
                Reply::json(value)
            }
            "/register" => Reply::json(json!({"client_id":"public-client", "client_secret":"unsupported-public-client-secret", "redirect_uris":["http://127.0.0.1:49152/oauth/callback"], "token_endpoint_auth_method":"none"})),
            "/.well-known/oauth-protected-resource/mcp" if current.load(Ordering::SeqCst) == 7 => {
                let mut value = resource(base);
                value.as_object_mut().unwrap().remove("scopes_supported");
                Reply::json(value)
            }
            _ => route(base,request),
        }
    }).await;
    let (mut operation, server) = discover(&fixture).await;
    let client = registered(&mut operation, &server).await;
    let pending = PendingAuthorization::new(server.clone(), client.clone()).unwrap();
    let response = callback(&pending, Some(server.issuer()));
    let tokens = pending.complete(&response, &mut operation).await.unwrap();
    for m in 1..=6 {
        mode.store(m, Ordering::SeqCst);
        let mut phase = Operation::new(CancellationToken::new());
        let before = fixture.count();
        assert_eq!(
            kind(phase.refresh(&server, &client, &tokens).await),
            if m == 6 { Kind::Redirect } else { Kind::Tokens }
        );
        assert_eq!(fixture.count(), before + 1);
        assert!(tokens.can_refresh());
        assert!(tokens.contains_secret(&json!("public-fixture-refresh")));
    }
    assert_eq!(
        kind(
            operation
                .register_client(
                    &server,
                    "http://127.0.0.1:49152/oauth/callback",
                    &ClientConfiguration::default()
                )
                .await
        ),
        Kind::Registration
    );
    let mut changed_client = client.clone();
    changed_client.client_id = "changed-client".into();
    let before = fixture.count();
    assert_eq!(
        kind(operation.refresh(&server, &changed_client, &tokens).await),
        Kind::Issuer
    );
    let cancel = CancellationToken::new();
    cancel.cancel();
    assert_eq!(
        kind(
            Operation::new(cancel)
                .refresh(&server, &client, &tokens)
                .await
        ),
        Kind::Cancelled
    );
    assert_eq!(fixture.count(), before);

    mode.store(7, Ordering::SeqCst);
    let (mut phase, server) = discover(&fixture).await;
    assert!(server.scopes().is_empty());
    let client = registered(&mut phase, &server).await;
    let pending = PendingAuthorization::new(server.clone(), client).unwrap();
    let response = callback(&pending, Some(server.issuer()));
    let defaults = pending.complete(&response, &mut phase).await.unwrap();
    assert_eq!(defaults.scopes(), ["default:read"]);
}

#[tokio::test]
async fn one_deadline_and_cumulative_limits_bound_all_network_requests() {
    let arrived = Arc::new(Notify::new());
    let signal = arrived.clone();
    let fixture = Fixture::start(move |_, _| {
        signal.notify_one();
        Reply {
            hold: Some(Arc::new(Notify::new())),
            ..Reply::json(json!({}))
        }
    })
    .await;
    let endpoint = format!("{}/mcp", fixture.base);
    let running = tokio::spawn(async move {
        Operation::new(CancellationToken::new())
            .discover_resource(&endpoint, None)
            .await
    });
    arrived.notified().await;
    tokio::time::pause();
    tokio::time::advance(Duration::from_secs(31)).await;
    assert_eq!(kind(running.await.unwrap()), Kind::Deadline);
    tokio::time::resume();
    assert_eq!(fixture.count(), 1);
    let fixture = Fixture::start(|base, request| {
        let mut response = route(base, request);
        let mut value: Value = serde_json::from_str(&response.body).unwrap();
        value["padding"] = json!("x".repeat(30 * 1024));
        response.body = value.to_string();
        response
    })
    .await;
    let mut operation = Operation::new(CancellationToken::new());
    let endpoint = format!("{}/mcp", fixture.base);
    for _ in 0..4 {
        operation.discover_resource(&endpoint, None).await.unwrap();
    }
    assert_eq!(
        kind(operation.discover_resource(&endpoint, None).await),
        Kind::Limit
    );
    assert_eq!(fixture.count(), 5);
    let fixture = Fixture::start(route).await;
    let mut operation = Operation::new(CancellationToken::new());
    for _ in 0..12 {
        operation
            .discover_resource(&format!("{}/mcp", fixture.base), None)
            .await
            .unwrap();
    }
    assert_eq!(
        kind(
            operation
                .discover_resource(&format!("{}/mcp", fixture.base), None)
                .await
        ),
        Kind::Limit
    );
    assert_eq!(fixture.count(), 12);
}

#[tokio::test]
async fn stop_after_refresh_dispatch_retains_old_tokens_without_replay() {
    let arrived = Arc::new(Notify::new());
    let signal = arrived.clone();
    let fixture = Fixture::start(move |base, request| {
        if request.path != "/token" { return route(base,request); }
        if request.form()["grant_type"] == "refresh_token" {
            signal.notify_one();
            // A real server may have rotated already; stopping cannot undo that
            // effect. Retain local evidence and require a fresh explicit sign-in.
            Reply { hold:Some(Arc::new(Notify::new())), ..Reply::json(json!({})) }
        } else {
            Reply::json(json!({"access_token":"public-prior-access", "refresh_token":"public-prior-refresh", "token_type":"Bearer", "expires_in":3600}))
        }
    }).await;
    let (mut operation, server) = discover(&fixture).await;
    let client = registered(&mut operation, &server).await;
    let pending = PendingAuthorization::new(server.clone(), client.clone()).unwrap();
    let response = callback(&pending, Some(server.issuer()));
    let tokens = pending.complete(&response, &mut operation).await.unwrap();
    let cancel = CancellationToken::new();
    let mut refresh_operation = Operation::new(cancel.clone());
    let refresh = refresh_operation.refresh(&server, &client, &tokens);
    tokio::pin!(refresh);
    tokio::select! {
        _ = &mut refresh => panic!("refresh completed before its held response"),
        _ = arrived.notified() => {}
    }
    cancel.cancel();
    assert_eq!(kind(refresh.await), Kind::Cancelled);
    assert!(tokens.contains_secret(&json!("public-prior-access")));
    assert!(tokens.contains_secret(&json!("public-prior-refresh")));
    assert_eq!(fixture.count(), 4);
}
