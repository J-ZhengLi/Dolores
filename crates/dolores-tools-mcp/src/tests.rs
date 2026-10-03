use super::*;

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
