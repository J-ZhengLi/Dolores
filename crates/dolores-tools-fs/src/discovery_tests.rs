use super::*;
use crate::folder_tools;
use serde_json::Value;
fn call(name: &str, args: Value) -> ToolCall {
    ToolCall {
        id: "discovery".into(),
        name: name.into(),
        arguments: args.to_string(),
    }
}
async fn execute(tool: &dyn ToolPlugin, args: Value) -> Value {
    let request = tool.prepare(&call(&tool.spec().name, args)).unwrap();
    serde_json::from_str(
        &tool
            .invoke(&request, CancellationToken::new())
            .await
            .unwrap(),
    )
    .unwrap()
}
#[tokio::test]
async fn discovery_keeps_root_relative_paths_unicode_and_literal_query_binding() {
    let root = tempfile::tempdir().unwrap();
    std::fs::create_dir(root.path().join("docs")).unwrap();
    std::fs::write(
        root.path().join("docs/notes.txt"),
        "intro\nFind 世界.* here\nworld lowercase",
    )
    .unwrap();
    std::fs::write(root.path().join("top.txt"), "Other content").unwrap();
    let tools = folder_tools(root.path()).unwrap();
    assert_eq!(tools.len(), 4);
    let listing = execute(tools[1].as_ref(), json!({"path":"docs"})).await;
    assert_eq!(listing["entries"][0]["path"], "docs/notes.txt");
    let result = execute(tools[2].as_ref(), json!({"path":"docs","query":"世界.*"})).await;
    assert_eq!(result["matches"].as_array().unwrap().len(), 1);
    assert_eq!(result["matches"][0]["path"], "docs/notes.txt");
    assert_eq!(result["matches"][0]["line"], 2);
    assert_eq!(result["matches"][0]["text"], "Find 世界.* here");
    let request = tools[2]
        .prepare(&call("search_text", json!({"path":"docs","query":"WORLD"})))
        .unwrap();
    assert_eq!(request.query.as_deref(), Some("WORLD"));
    let result: Value = serde_json::from_str(
        &tools[2]
            .invoke(&request, CancellationToken::new())
            .await
            .unwrap(),
    )
    .unwrap();
    assert!(result["matches"].as_array().unwrap().is_empty());
    let request = tools[0]
        .prepare(&call("read_text_file", json!({"path":"docs/notes.txt"})))
        .unwrap();
    assert!(tools[0]
        .invoke(&request, CancellationToken::new())
        .await
        .unwrap()
        .contains("世界.*"));
}
#[tokio::test]
async fn discovery_excludes_secret_generated_binary_and_unsafe_scopes() {
    let root = tempfile::tempdir().unwrap();
    std::fs::write(root.path().join(".env"), "NEEDLE private").unwrap();
    std::fs::write(root.path().join("binary"), b"NEEDLE\0binary").unwrap();
    std::fs::write(root.path().join("invalid"), [0xff, 0xfe]).unwrap();
    std::fs::write(root.path().join("large"), vec![b'x'; MAX_TOOL_BYTES + 1]).unwrap();
    for folder in [".git", "node_modules", "target"] {
        std::fs::create_dir(root.path().join(folder)).unwrap();
        std::fs::write(root.path().join(folder).join("private.txt"), "NEEDLE").unwrap();
    }
    let tools = folder_tools(root.path()).unwrap();
    let listing = execute(tools[1].as_ref(), json!({"path":"."})).await;
    assert_eq!(listing["entries"].as_array().unwrap().len(), 3);
    let result = execute(tools[2].as_ref(), json!({"path":".","query":"NEEDLE"})).await;
    assert!(result["matches"].as_array().unwrap().is_empty());
    assert!(result["skippedFiles"].as_u64().unwrap() >= 3);
    for path in [
        "../outside",
        "/absolute",
        ".env",
        ".git",
        "node_modules",
        "target",
        "a/../b",
        "C:/outside",
    ] {
        for tool in [&tools[1], &tools[2]] {
            let args = if tool.spec().name == "search_text" {
                json!({"path":path,"query":"NEEDLE"})
            } else {
                json!({"path":path})
            };
            assert!(
                tool.prepare(&call(&tool.spec().name, args)).is_err(),
                "{path}"
            );
        }
    }
    for query in ["", " ", "line\nbreak", "nul\0", "x".repeat(257).as_str()] {
        assert!(tools[2]
            .prepare(&call("search_text", json!({"path":".","query":query})))
            .is_err());
    }
    assert!(tools[1]
        .prepare(&call(
            "list_folder",
            json!({"path":".","query":"unexpected"})
        ))
        .is_err());
    assert!(tools[2]
        .prepare(&call(
            "search_text",
            json!({"path":".","query":"NEEDLE","approved":true})
        ))
        .is_err());
}
#[tokio::test]
async fn listing_search_and_long_results_respect_scan_and_output_budgets() {
    let root = tempfile::tempdir().unwrap();
    for i in 0..600 {
        std::fs::write(root.path().join(format!("{i:04}.txt")), "match\n".repeat(4)).unwrap();
    }
    let tools = folder_tools(root.path()).unwrap();
    let listing = execute(tools[1].as_ref(), json!({"path":"."})).await;
    assert_eq!(listing["inspectedEntries"], 512);
    assert_eq!(listing["entries"].as_array().unwrap().len(), 100);
    assert_eq!(listing["truncated"], true);
    let result = execute(tools[2].as_ref(), json!({"path":".","query":"match"})).await;
    assert_eq!(result["inspectedEntries"], 256);
    assert_eq!(result["matches"].as_array().unwrap().len(), 30);
    assert_eq!(result["truncated"], true);
    let no_match = execute(tools[2].as_ref(), json!({"path":".","query":"absent"})).await;
    assert_eq!(no_match["attemptedFiles"], 64);
    assert_eq!(no_match["truncated"], true);
    let root = tempfile::tempdir().unwrap();
    for i in 0..25 {
        std::fs::write(
            root.path().join(format!("{i:03}")),
            "x".repeat(MAX_TOOL_BYTES),
        )
        .unwrap();
    }
    let tools = folder_tools(root.path()).unwrap();
    let result = execute(tools[2].as_ref(), json!({"path":".","query":"absent"})).await;
    assert_eq!(result["bytesRead"], MAX_SEARCH_BYTES);
    assert_eq!(result["truncated"], true);
    let root = tempfile::tempdir().unwrap();
    let name = "界".repeat(65);
    let mut folder = root.path().to_owned();
    for _ in 0..4 {
        folder.push(&name);
        std::fs::create_dir(&folder).unwrap();
    }
    std::fs::write(
        folder.join("notes"),
        format!("{}needle{}\n", "界".repeat(120), "界".repeat(120)).repeat(15),
    )
    .unwrap();
    let tools = folder_tools(root.path()).unwrap();
    let result = execute(tools[2].as_ref(), json!({"path":".","query":"needle"})).await;
    assert!(result.to_string().len() <= MAX_TOOL_BYTES);
    assert!(result["matches"].as_array().unwrap().len() < 15);
    assert_eq!(result["truncated"], true);
    for m in result["matches"].as_array().unwrap() {
        assert!(m["text"].as_str().unwrap().chars().count() <= 240);
    }
}
#[tokio::test]
async fn scan_stops_at_depth_and_honors_pre_cancelled_operations() {
    let root = tempfile::tempdir().unwrap();
    let mut path = root.path().to_owned();
    for _ in 0..5 {
        path.push("deep");
        std::fs::create_dir(&path).unwrap();
    }
    std::fs::write(path.join("hidden"), "needle").unwrap();
    let tools = folder_tools(root.path()).unwrap();
    let result = execute(tools[2].as_ref(), json!({"path":".","query":"needle"})).await;
    assert!(result["matches"].as_array().unwrap().is_empty());
    assert_eq!(result["truncated"], true);
    for tool in [&tools[1], &tools[2]] {
        let args = if tool.spec().name == "search_text" {
            json!({"path":".","query":"needle"})
        } else {
            json!({"path":"."})
        };
        let request = tool.prepare(&call(&tool.spec().name, args)).unwrap();
        let cancel = CancellationToken::new();
        cancel.cancel();
        assert!(tool.invoke(&request, cancel).await.is_err());
    }
}
