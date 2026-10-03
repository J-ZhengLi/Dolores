use super::*;

#[tokio::test]
async fn identical_tool_names_route_to_separate_servers_and_one_missing_key_does_not_block_the_other(
) {
    let folder = tempfile::tempdir().unwrap();
    let root = folder.path().canonicalize().unwrap();
    let mut a = connection(
        inspect_with_credentials(
            &root,
            launch("credential-echo"),
            CancellationToken::new(),
            credentials(),
        )
        .unwrap(),
    );
    let mut b = connection(inspect(&root, launch("normal"), CancellationToken::new()).unwrap());
    a.id = "00000000-0000-4000-8000-000000000001".into();
    b.id = "00000000-0000-4000-8000-000000000002".into();
    a.tools.truncate(1);
    b.tools.truncate(1);
    assert_eq!(a.tools[0].name, b.tools[0].name);
    assert_ne!(a.specs()[0].name, b.specs()[0].name);
    let vault = Arc::new(Vault::default());
    let id = "00000000-0000-4000-8000-000000000003";
    a.credentials = vec![dolores_core::McpCredentialBinding {
        name: "DOLORES_MCP_TEST_TOKEN".into(),
        credential_id: id.into(),
    }];
    vault
        .write(
            id,
            &credentials()
                .encoded_for(
                    &a.id,
                    "DOLORES_MCP_TEST_TOKEN",
                    &root,
                    &a.launch,
                    &a.fingerprints,
                )
                .unwrap(),
        )
        .unwrap();
    let mut wrong = a.clone();
    wrong.id = b.id.clone();
    assert!(
        credentials::resolve(&root, &wrong, vault.as_ref()).is_err(),
        "another connection reused a key"
    );
    let left = plugins_with_credentials(&root, a, vault.clone())
        .unwrap()
        .remove(0);
    let right = plugins_with_credentials(&root, b, vault.clone())
        .unwrap()
        .remove(0);
    let proposal = |plugin: &Arc<dyn ToolPlugin>, id: &str| ToolCall {
        id: id.into(),
        name: plugin.spec().name,
        arguments: r#"{"text":"healthy server"}"#.into(),
    };
    let left_request = left.prepare(&proposal(&left, "left")).unwrap();
    let right_request = right.prepare(&proposal(&right, "right")).unwrap();
    assert_ne!(
        left_request.mcp.as_ref().unwrap().connection_id,
        right_request.mcp.as_ref().unwrap().connection_id
    );
    vault.delete(id).unwrap();
    let before = events(&root).len();
    assert!(left
        .invoke(&left_request, CancellationToken::new())
        .await
        .unwrap_err()
        .contains("key is missing"));
    assert_eq!(events(&root).len(), before);
    let result = right
        .invoke(&right_request, CancellationToken::new())
        .await
        .unwrap();
    assert_eq!(
        serde_json::from_str::<Value>(&result).unwrap()["text"],
        "healthy server"
    );
    assert!(events(&root)
        .iter()
        .filter(|e| e["method"] == "tools/call")
        .all(|e| e["params"]["name"] == "echo"));
}

#[derive(Default)]
struct Vault {
    values: Mutex<HashMap<String, String>>,
    reads: std::sync::atomic::AtomicUsize,
    locked: std::sync::atomic::AtomicBool,
}
impl CredentialStore for Vault {
    fn descriptor(&self) -> dolores_core::PluginDescriptor {
        dolores_core::PluginDescriptor {
            id: "test.vault",
            kind: "credentials",
            api_version: 1,
        }
    }
    fn read(&self, id: &str) -> Result<Option<String>, String> {
        self.reads.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        if self.locked.load(std::sync::atomic::Ordering::SeqCst) {
            return Err("private diagnostic".into());
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
fn credentials() -> Credentials {
    Credentials::new(vec![(
        "DOLORES_MCP_TEST_TOKEN".into(),
        "synthetic-key-credential-test".into(),
    )])
    .unwrap()
}
#[tokio::test]
async fn credentials_are_read_only_after_approval_and_echoes_are_redacted() {
    use std::sync::atomic::Ordering::SeqCst;
    let folder = tempfile::tempdir().unwrap();
    let root = folder.path().canonicalize().unwrap();
    let secret = credentials();
    let inspected = inspect_with_credentials(
        &root,
        launch("credential-echo"),
        CancellationToken::new(),
        secret.clone(),
    )
    .unwrap();
    let mut conn = connection(inspected);
    let vault = Arc::new(Vault::default());
    let id = "00000000-0000-4000-8000-000000000001";
    vault
        .write(
            id,
            &secret
                .encoded(
                    "DOLORES_MCP_TEST_TOKEN",
                    &root,
                    &conn.launch,
                    &conn.fingerprints,
                )
                .unwrap(),
        )
        .unwrap();
    conn.credentials = vec![dolores_core::McpCredentialBinding {
        name: "DOLORES_MCP_TEST_TOKEN".into(),
        credential_id: id.into(),
    }];
    let plugin = plugins_with_credentials(&root, conn.clone(), vault.clone())
        .unwrap()
        .remove(0);
    let request = plugin
        .prepare(&call("safe", r#"{"text":"hello"}"#))
        .unwrap();
    assert_eq!(
        request.mcp.as_ref().unwrap().credential_names,
        ["DOLORES_MCP_TEST_TOKEN"]
    );
    assert_eq!(vault.reads.load(SeqCst), 0);
    assert_eq!(
        events(&root)
            .iter()
            .filter(|e| e["type"] == "start")
            .count(),
        1
    );
    let result = plugin
        .invoke(&request, CancellationToken::new())
        .await
        .unwrap();
    assert!(result.contains("[redacted]"));
    assert!(!result.contains("synthetic-key"));
    assert_eq!(vault.reads.load(SeqCst), 1);
    assert!(events(&root)
        .iter()
        .filter(|e| e["type"] == "start")
        .all(|e| e["credentialReceived"] == true && e["secretInherited"] == false));
    for state in ["locked", "missing", "wrong-root", "wrong-launch"] {
        vault.locked.store(state == "locked", SeqCst);
        if state == "missing" {
            vault.delete(id).unwrap();
        }
        if state == "wrong-root" {
            std::fs::create_dir(root.join("another")).unwrap();
            vault
                .write(
                    id,
                    &secret
                        .encoded(
                            "DOLORES_MCP_TEST_TOKEN",
                            &root.join("another"),
                            &conn.launch,
                            &conn.fingerprints,
                        )
                        .unwrap(),
                )
                .unwrap();
        }
        if state == "wrong-launch" {
            let mut other = conn.launch.clone();
            other.label = "Other".into();
            vault
                .write(
                    id,
                    &secret
                        .encoded("DOLORES_MCP_TEST_TOKEN", &root, &other, &conn.fingerprints)
                        .unwrap(),
                )
                .unwrap();
        }
        let before = events(&root).len();
        let request = plugin.prepare(&call(state, r#"{"text":"hello"}"#)).unwrap();
        let error = plugin
            .invoke(&request, CancellationToken::new())
            .await
            .unwrap_err();
        assert!(!error.contains("private diagnostic") && !error.contains("synthetic-key"));
        assert_eq!(events(&root).len(), before, "server started for {state}");
    }
}
#[test]
fn credential_validation_and_metadata_leaks_are_refused() {
    for name in [
        "PATH",
        "LD_PRELOAD",
        "NODE_OPTIONS",
        "PYTHONPATH",
        "GIT_CONFIG_COUNT",
        "lowercase",
        "0KEY",
        "A=B",
        "HOME",
        "COMSPEC",
    ] {
        assert!(
            Credentials::new(vec![(name.into(), "synthetic-value".into())]).is_err(),
            "{name}"
        );
    }
    assert!(Credentials::new(vec![("KEY".into(), "".into())]).is_err());
    assert!(Credentials::new(vec![
        ("KEY".into(), "one".into()),
        ("KEY".into(), "two".into())
    ])
    .is_err());
    let folder = tempfile::tempdir().unwrap();
    let error = inspect_with_credentials(
        folder.path(),
        launch("credential-metadata"),
        CancellationToken::new(),
        credentials(),
    )
    .unwrap_err();
    assert!(error.contains("exposed a credential") && !error.contains("synthetic-key"));
    assert!(inspect(
        folder.path(),
        launch("credential-echo"),
        CancellationToken::new()
    )
    .is_err());
    let mut unsafe_launch = launch("normal");
    unsafe_launch
        .args
        .push("--secret=synthetic-key-credential-test".into());
    let before = events(folder.path()).len();
    assert!(inspect_with_credentials(
        folder.path(),
        unsafe_launch,
        CancellationToken::new(),
        credentials()
    )
    .is_err());
    assert_eq!(events(folder.path()).len(), before);
    let overlap = Credentials::new(vec![
        ("ONE".into(), "abc".into()),
        ("TWO".into(), "bcd".into()),
    ])
    .unwrap();
    assert_eq!(
        overlap.redact("before abcd after"),
        "before [redacted] after"
    );
    assert!(credentials()
        .reject_metadata(&json!({"nested": {"synthetic-key-credential-test": "ignored"}}))
        .is_err());
}

fn launch(mode: &str) -> McpLaunch {
    let filename = if cfg!(windows) { "node.exe" } else { "node" };
    let executable = std::env::split_paths(&std::env::var_os("PATH").unwrap())
        .map(|p| p.join(filename))
        .find(|p| p.is_absolute() && p.is_file())
        .expect("Install Node to run MCP transport diagnostics");
    McpLaunch {
        label: "Synthetic server".into(),
        executable: executable.to_str().unwrap().into(),
        args: vec![
            Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("../../scripts/mock-mcp.mjs")
                .to_str()
                .unwrap()
                .into(),
            format!("--mode={mode}"),
        ],
    }
}
fn connection(i: Inspection) -> McpConnection {
    McpConnection {
        id: "legacy".into(),
        credentials: vec![],
        retired_credentials: vec![],
        revision: 1,
        enabled: true,
        launch: i.launch,
        fingerprints: i.fingerprints,
        protocol_version: i.protocol_version,
        server_name: i.server_name,
        server_version: i.server_version,
        tools: i.tools,
    }
}
fn events(root: &Path) -> Vec<Value> {
    std::fs::read_to_string(root.join("mcp-events.ndjson"))
        .unwrap_or_default()
        .lines()
        .map(|s| serde_json::from_str(s).unwrap())
        .collect()
}
fn call(id: &str, arguments: &str) -> ToolCall {
    ToolCall {
        id: id.into(),
        name: "mcp_tool_1".into(),
        arguments: arguments.into(),
    }
}

#[tokio::test]
async fn reviewed_calls_are_exact_one_use_and_do_not_start_during_prepare() {
    let root = tempfile::tempdir().unwrap();
    let i = inspect(root.path(), launch("normal"), CancellationToken::new()).unwrap();
    assert_eq!(
        events(root.path())
            .iter()
            .filter(|e| e["type"] == "start")
            .count(),
        1
    );
    let plugin = plugins(root.path(), connection(i)).unwrap().remove(0);
    assert_eq!(plugin.spec().name, "mcp_tool_1");
    let request = plugin
        .prepare(&call("one", r#"{"text":"hello 世界"}"#))
        .unwrap();
    assert_eq!(
        events(root.path())
            .iter()
            .filter(|e| e["type"] == "start")
            .count(),
        1
    );
    let result: Value = serde_json::from_str(
        &plugin
            .invoke(&request, CancellationToken::new())
            .await
            .unwrap(),
    )
    .unwrap();
    assert_eq!(result, json!({"text":"hello 世界","isError":false}));
    assert!(plugin
        .invoke(&request, CancellationToken::new())
        .await
        .is_err());
    let mut forged = plugin.prepare(&call("two", r#"{"text":"safe"}"#)).unwrap();
    forged.mcp.as_mut().unwrap().arguments = r#"{"text":"different"}"#.into();
    assert!(plugin
        .invoke(&forged, CancellationToken::new())
        .await
        .unwrap_err()
        .contains("changed"));
    let all = events(root.path());
    assert_eq!(all.iter().filter(|e| e["type"] == "start").count(), 2);
    assert!(all
        .iter()
        .filter(|e| e["type"] == "start")
        .all(|e| e["secretInherited"] == false));
    assert!(all
        .iter()
        .filter(|e| e["method"] == "initialize")
        .all(|e| e["params"]["capabilities"] == json!({})));
    assert!(plugin.prepare(&call("bad", "[]")).is_err());
}
#[test]
fn pages_and_unsupported_client_requests_follow_protocol() {
    for mode in ["pages", "requests"] {
        let root = tempfile::tempdir().unwrap();
        let i = inspect(root.path(), launch(mode), CancellationToken::new()).unwrap();
        assert_eq!(i.tools.len(), 2);
        if mode == "requests" {
            let all = events(root.path());
            assert!(all
                .iter()
                .any(|e| e["type"] == "clientResponse" && e["error"] == -32601));
        }
    }
}
#[test]
fn malformed_flood_unmatched_version_missing_tools_and_changed_list_are_refused() {
    for mode in [
        "malformed",
        "flood",
        "wrong-id",
        "version",
        "no-tools",
        "changed",
        "cycle",
        "crash",
    ] {
        let root = tempfile::tempdir().unwrap();
        assert!(
            inspect(root.path(), launch(mode), CancellationToken::new()).is_err(),
            "mode {mode}"
        );
    }
}
#[tokio::test]
async fn changed_metadata_never_sends_tools_call_and_errors_are_explicit() {
    let root = tempfile::tempdir().unwrap();
    let i = inspect(root.path(), launch("normal"), CancellationToken::new()).unwrap();
    let plugin = plugins(root.path(), connection(i)).unwrap().remove(0);
    let request = plugin
        .prepare(&call("drift", r#"{"text":"hello"}"#))
        .unwrap();
    std::fs::write(root.path().join("mcp-drift"), "").unwrap();
    assert!(plugin
        .invoke(&request, CancellationToken::new())
        .await
        .unwrap_err()
        .contains("metadata changed"));
    assert!(!events(root.path())
        .iter()
        .any(|e| e["method"] == "tools/call"));
    std::fs::remove_file(root.path().join("mcp-drift")).unwrap();
    let request = plugin.prepare(&call("error", r#"{"text":1}"#)).unwrap();
    let result: Value = serde_json::from_str(
        &plugin
            .invoke(&request, CancellationToken::new())
            .await
            .unwrap(),
    )
    .unwrap();
    assert_eq!(result["isError"], true);
    assert_eq!(result["text"], "Expected a text string.");
}
#[tokio::test]
async fn text_only_results_and_output_budget_are_enforced() {
    for (mode, error) in [
        ("image", "text results only"),
        ("large-text", "exceeds 8 KiB"),
    ] {
        let root = tempfile::tempdir().unwrap();
        let i = inspect(root.path(), launch(mode), CancellationToken::new()).unwrap();
        let plugin = plugins(root.path(), connection(i)).unwrap().remove(0);
        let request = plugin
            .prepare(&call("result", r#"{"text":"hello"}"#))
            .unwrap();
        assert!(plugin
            .invoke(&request, CancellationToken::new())
            .await
            .unwrap_err()
            .contains(error));
    }
}
#[tokio::test]
async fn launch_file_changes_are_refused_before_starting_a_process() {
    let root = tempfile::tempdir().unwrap();
    let source = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../scripts/mock-mcp.mjs");
    let copy = root.path().join("fixture.mjs");
    std::fs::copy(source, &copy).unwrap();
    let mut l = launch("normal");
    l.args[0] = copy.to_str().unwrap().into();
    let i = inspect(root.path(), l, CancellationToken::new()).unwrap();
    let plugin = plugins(root.path(), connection(i)).unwrap().remove(0);
    let request = plugin
        .prepare(&call("changed", r#"{"text":"hello"}"#))
        .unwrap();
    std::fs::write(copy, "// changed").unwrap();
    assert!(plugin
        .invoke(&request, CancellationToken::new())
        .await
        .unwrap_err()
        .contains("launch files changed"));
    assert_eq!(
        events(root.path())
            .iter()
            .filter(|e| e["type"] == "start")
            .count(),
        1
    );
}
#[test]
fn cancellation_and_deadline_interrupt_nonresponsive_servers() {
    for mode in ["hang", "no-input"] {
        let root = tempfile::tempdir().unwrap();
        let cancel = CancellationToken::new();
        let c = cancel.clone();
        let killer = thread::spawn(move || {
            thread::sleep(Duration::from_millis(100));
            c.cancel();
        });
        assert!(inspect(root.path(), launch(mode), cancel)
            .unwrap_err()
            .contains("stopped"));
        killer.join().unwrap();
    }
    let root = tempfile::tempdir().unwrap();
    let result = Session::start(
        root.path(),
        &launch("hang"),
        CancellationToken::new(),
        Instant::now() + Duration::from_millis(250),
        Credentials::empty(),
    );
    assert!(matches!(result,Err(e) if e.contains("limit")));
}
#[tokio::test]
async fn dropped_invoke_stops_the_server_and_its_descendants() {
    let root = tempfile::tempdir().unwrap();
    let i = inspect(root.path(), launch("child"), CancellationToken::new()).unwrap();
    let plugin = plugins(root.path(), connection(i)).unwrap().remove(0);
    std::fs::write(root.path().join("mcp-call-hang"), "").unwrap();
    let request = plugin
        .prepare(&call("drop", r#"{"text":"hello"}"#))
        .unwrap();
    let worker =
        tokio::spawn(async move { plugin.invoke(&request, CancellationToken::new()).await });
    for _ in 0..400 {
        if events(root.path())
            .iter()
            .any(|e| e["method"] == "tools/call")
        {
            break;
        }
        tokio::time::sleep(Duration::from_millis(5)).await;
    }
    assert!(events(root.path())
        .iter()
        .any(|e| e["method"] == "tools/call"));
    worker.abort();
    let _ = worker.await;
    tokio::time::sleep(Duration::from_millis(200)).await;
    // Probe PID liveness with a direct installed runtime, without a shell.
    let child = std::fs::read_to_string(root.path().join("mcp-child.pid")).unwrap();
    let result = std::process::Command::new(&launch("normal").executable)
        .args([
            "-e",
            "try {process.kill(Number(process.argv[1]),0);process.exit(1)} catch {process.exit(0)}",
            child.trim(),
        ])
        .output()
        .unwrap();
    assert!(
        result.status.success(),
        "descendant survived dropped invoke"
    );
}
