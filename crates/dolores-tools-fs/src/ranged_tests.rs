use super::*;
fn call(name: &str, value: serde_json::Value, id: &str) -> ToolCall {
    ToolCall {
        id: id.into(),
        name: name.into(),
        arguments: value.to_string(),
    }
}
#[tokio::test]
async fn large_unicode_crlf_snapshot_edit_and_conflict_refresh_preserve_other_lines() {
    let dir = tempfile::tempdir().unwrap();
    let prefix = "# 世界 preserved\r\n".repeat(1600);
    let text = format!("{prefix}answer = 41\r\n");
    std::fs::write(dir.path().join("large.py"), &text).unwrap();
    let tools = folder_tools(dir.path()).unwrap();
    let read = &tools[0];
    let edit = &tools[3];
    let r = read
        .prepare(&call(
            "read_text_file",
            json!({"path":"large.py","start_line":1601,"line_count":1}),
            "read",
        ))
        .unwrap();
    let value: serde_json::Value =
        serde_json::from_str(&read.invoke(&r, CancellationToken::new()).await.unwrap()).unwrap();
    assert_eq!(value["text"], "answer = 41\r\n");
    let arguments = json!({"path":"large.py","old_text":"answer = 41\n","new_text":"answer = 42\n","expected_snapshot":value["snapshot"]});
    let p = edit
        .prepare(&call("edit_text_file", arguments.clone(), "edit"))
        .unwrap();
    std::fs::write(
        dir.path().join("large.py"),
        format!("{text}# concurrent\r\n"),
    )
    .unwrap();
    assert!(edit
        .invoke(&p, CancellationToken::new())
        .await
        .unwrap_err()
        .contains("changed"));
    assert!(edit
        .prepare(&call("edit_text_file", arguments, "stale"))
        .unwrap_err()
        .contains("Snapshot changed"));
    let fresh: serde_json::Value =
        serde_json::from_str(&read.invoke(&r, CancellationToken::new()).await.unwrap()).unwrap();
    let p=edit.prepare(&call("edit_text_file",json!({"path":"large.py","old_text":"answer = 41\n","new_text":"answer = 42\n","expected_snapshot":fresh["snapshot"]}),"fresh")).unwrap();
    edit.invoke(&p, CancellationToken::new()).await.unwrap();
    assert_eq!(
        std::fs::read_to_string(dir.path().join("large.py")).unwrap(),
        format!("{prefix}answer = 42\r\n# concurrent\r\n")
    );
}
#[test]
fn invalid_ranges_and_long_lines_explain_recovery() {
    assert!(ranged::range(Some(0), Some(1)).is_err());
    assert!(ranged::range(Some(1), Some(121)).is_err());
    assert!(ranged::render(&"x".repeat(20000), None)
        .unwrap_err()
        .contains("start_line"));
    assert!(ranged::render(&"x".repeat(20000), Some("1:1"))
        .unwrap_err()
        .contains("split"));
}
