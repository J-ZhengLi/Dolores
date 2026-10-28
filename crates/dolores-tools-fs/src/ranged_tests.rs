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
fn invalid_ranges_fail_and_long_unicode_lines_are_exactly_retrievable() {
    assert!(ranged::range(Some(0), Some(1)).is_err());
    assert!(ranged::range(Some(1), Some(121)).is_err());
    let original = "世界\\\"".repeat(16000);
    let mut reconstructed = String::new();
    let mut range = None;
    loop {
        let page: serde_json::Value =
            serde_json::from_str(&ranged::render(&original, range.as_deref()).unwrap()).unwrap();
        assert!(page.to_string().len() <= MAX_TOOL_BYTES);
        reconstructed.push_str(page["text"].as_str().unwrap());
        if page["hasMore"] == false {
            break;
        }
        range = Some(
            ranged::byte_range(
                Some(page["nextByte"].as_u64().unwrap() as usize),
                Some(65536),
            )
            .unwrap(),
        );
    }
    assert_eq!(reconstructed, original);
    assert!(ranged::render(&original, Some("bytes:1:4096")).is_err());
}
#[tokio::test]
async fn multi_megabyte_source_is_read_without_loading_an_entire_snapshot() {
    use sha2::{Digest, Sha256};
    let root = tempfile::tempdir().unwrap();
    let text = format!("{}\npublic final 世界", "界".repeat(600000));
    std::fs::write(root.path().join("large.txt"), &text).unwrap();
    let read = ReadTextFile::new(root.path()).unwrap();
    let request = read
        .prepare(&call(
            "read_text_file",
            json!({"path":"large.txt","start_line":2,"line_count":1}),
            "large",
        ))
        .unwrap();
    let value: serde_json::Value = serde_json::from_str(
        &read
            .invoke(&request, CancellationToken::new())
            .await
            .unwrap(),
    )
    .unwrap();
    assert_eq!(value["text"], "public final 世界");
    assert_eq!(
        value["snapshot"],
        format!("{:x}", Sha256::digest(text.as_bytes()))
    );
    assert_eq!(value["hasMore"], false);
    let request = read
        .prepare(&call(
            "read_text_file",
            json!({"path":"large.txt","start_byte":65535,"max_bytes":8192}),
            "offset",
        ))
        .unwrap();
    let value: serde_json::Value = serde_json::from_str(
        &read
            .invoke(&request, CancellationToken::new())
            .await
            .unwrap(),
    )
    .unwrap();
    assert_eq!(value["startByte"], 65535);
    assert_eq!(
        value["text"].as_str().unwrap(),
        &text[65535..value["nextByte"].as_u64().unwrap() as usize]
    );
    let request = read
        .prepare(&call(
            "read_text_file",
            json!({"path":"large.txt","start_byte":0,"max_bytes":1}),
            "one-codepoint",
        ))
        .unwrap();
    let first: serde_json::Value = serde_json::from_str(
        &read
            .invoke(&request, CancellationToken::new())
            .await
            .unwrap(),
    )
    .unwrap();
    assert_eq!(first["text"], "界");
    assert_eq!(first["nextByte"], 3);
    assert_eq!(first["hasMore"], true);
}
