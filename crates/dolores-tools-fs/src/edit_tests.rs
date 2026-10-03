use super::*;
use dolores_core::{
    AgentMessage, AgentTurn, Message, ModelProvider, PluginDescriptor, Role, ToolApproval,
};
use tokio::sync::mpsc;

fn call(path: &str, old: &str, new: &str) -> ToolCall {
    ToolCall {
        id: "edit-one".into(),
        name: "edit_text_file".into(),
        arguments: json!({"path":path,"old_text":old,"new_text":new}).to_string(),
    }
}
fn tool(root: &Path) -> EditTextFile {
    EditTextFile::new(ReadTextFile::new(root).unwrap())
}
use std::path::Path;
#[path = "edit_recovery_tests.rs"]
mod recovery;

#[tokio::test]
async fn multiline_lf_proposal_edits_crlf_source_without_rewriting_other_bytes() {
    let root = tempfile::tempdir().unwrap();
    let file = root.path().join("module.cjs");
    let before = "// 世界\r\nfunction sum(a,b) {\r\n  return a-b;\r\n}\r\n// last line";
    std::fs::write(&file, before).unwrap();
    let edit = tool(root.path());
    let request = edit
        .prepare(&call(
            "module.cjs",
            "function sum(a,b) {\n  return a-b;\n}",
            "function sum(a,b) {\n  return a+b;\n}",
        ))
        .expect("An otherwise exact LF proposal must reach CRLF review");
    assert_eq!(std::fs::read(&file).unwrap(), before.as_bytes());
    edit.invoke(&request, CancellationToken::new())
        .await
        .unwrap();
    assert_eq!(
        std::fs::read(&file).unwrap(),
        before.replace("a-b", "a+b").as_bytes()
    );
}

#[tokio::test]
async fn newline_adaptation_is_symmetric_preserves_bom_and_handles_insert_delete() {
    for ending in ["\n", "\r\n"] {
        for proposal_ending in ["\n", "\r\n"] {
            for (old, new) in [
                ("alpha\nbeta", "alpha\ngamma"),
                ("alpha", "alpha\nextra"),
                ("alpha\nbeta", "alpha"),
                ("alpha\nbeta", ""),
            ] {
                let root = tempfile::tempdir().unwrap();
                let file = root.path().join("note");
                let before = format!("\u{feff}// 世界{ending}alpha{ending}beta{ending}untouched");
                std::fs::write(&file, &before).unwrap();
                let request_old = old.replace('\n', proposal_ending);
                let request_new = new.replace('\n', proposal_ending);
                let expected =
                    before.replacen(&old.replace('\n', ending), &new.replace('\n', ending), 1);
                let edit = tool(root.path());
                let request = edit
                    .prepare(&call("note", &request_old, &request_new))
                    .unwrap();
                assert_eq!(
                    request.diff.as_deref(),
                    Some(diff(&before, &expected).as_str())
                );
                assert_eq!(std::fs::read(&file).unwrap(), before.as_bytes());
                edit.invoke(&request, CancellationToken::new())
                    .await
                    .unwrap();
                assert_eq!(std::fs::read(&file).unwrap(), expected.as_bytes());
            }
        }
    }
}

#[test]
fn adapted_matches_stay_exact_unique_bounded_and_refuse_mixed_ending_inference() {
    let root = tempfile::tempdir().unwrap();
    let file = root.path().join("note");
    for (before, old, new) in [
        ("one\r\ntwo\r\none\r\ntwo", "one\ntwo", "changed"),
        ("\r\n\r\n\r\n", "\n\n", "changed"),
        ("one\r\n  two", "one\n two", "changed"),
        ("one\r\ntwo", "one\ntwo", "one\r\ntwo"),
        ("one\r\ntwo", "one\ntwo", "changed\0"),
    ] {
        std::fs::write(&file, before).unwrap();
        assert!(tool(root.path()).prepare(&call("note", old, new)).is_err());
        assert_eq!(std::fs::read(&file).unwrap(), before.as_bytes());
    }
    let mixed = "prefix\none\r\ntwo\r\nlast";
    std::fs::write(&file, mixed).unwrap();
    let error = tool(root.path())
        .prepare(&call("note", "one\ntwo", "changed"))
        .unwrap_err();
    assert!(error.contains("mixed or lone-CR") && error.contains("single-line match"));
    assert_eq!(std::fs::read(&file).unwrap(), mixed.as_bytes());
    std::fs::write(&file, "one\rtwo").unwrap();
    assert!(tool(root.path())
        .prepare(&call("note", "one\ntwo", "changed"))
        .unwrap_err()
        .contains("lone-CR"));
    let before = format!("x\r\n{}", "z".repeat(MAX_TOOL_BYTES - 3));
    std::fs::write(&file, &before).unwrap();
    assert!(tool(root.path())
        .prepare(&call("note", "x", "x\n"))
        .is_err());
    assert_eq!(std::fs::read(&file).unwrap(), before.as_bytes());
}

#[tokio::test]
async fn adapted_preview_keeps_raw_snapshot_checks_stop_and_single_use() {
    let root = tempfile::tempdir().unwrap();
    let file = root.path().join("note");
    let before = "one\r\ntwo\r\n";
    let edit = tool(root.path());
    let proposal = call("note", "one\ntwo", "one\nthree");
    for stop in [false, true] {
        std::fs::write(&file, before).unwrap();
        let request = edit.prepare(&proposal).unwrap();
        let cancel = CancellationToken::new();
        if stop {
            cancel.cancel();
        } else {
            std::fs::write(&file, before.replace("\r\n", "\n")).unwrap();
        }
        assert!(edit.invoke(&request, cancel).await.is_err());
        let expected = if stop {
            before.into()
        } else {
            before.replace("\r\n", "\n")
        };
        assert_eq!(std::fs::read_to_string(&file).unwrap(), expected);
        assert!(edit
            .invoke(&request, CancellationToken::new())
            .await
            .is_err());
    }
}

#[tokio::test]
async fn larger_edit_retains_snapshot_binding_and_exact_bytes() {
    let root = tempfile::tempdir().unwrap();
    let file = root.path().join("main.js");
    let before = "// original 世界 \\\"text\\\"\r\n".repeat(150);
    let after = "// revised 世界 \\\"text\\\"\r\n".repeat(150);
    std::fs::write(&file, &before).unwrap();
    let edit = tool(root.path());
    let proposal = call("main.js", &before, &after);
    assert!(proposal.arguments.len() > 4096);
    dolores_core::validate_call(&proposal).unwrap();
    let request = edit.prepare(&proposal).unwrap();
    assert_eq!(std::fs::read(&file).unwrap(), before.as_bytes());
    std::fs::write(&file, "external update").unwrap();
    assert!(edit
        .invoke(&request, CancellationToken::new())
        .await
        .is_err());
    assert_eq!(std::fs::read_to_string(&file).unwrap(), "external update");
    std::fs::write(&file, &before).unwrap();
    let request = edit.prepare(&proposal).unwrap();
    edit.invoke(&request, CancellationToken::new())
        .await
        .unwrap();
    assert_eq!(std::fs::read(&file).unwrap(), after.as_bytes());
}

#[tokio::test]
async fn applies_reviewed_unique_unicode_change_preserves_line_endings_and_is_single_use() {
    let root = tempfile::tempdir().unwrap();
    std::fs::create_dir(root.path().join("notes")).unwrap();
    let file = root.path().join("notes/世界.txt");
    let before = "Title\r\nHello 世界\r\nlast line";
    std::fs::write(&file, before).unwrap();
    let edit = tool(root.path());
    let request = edit
        .prepare(&call("notes/世界.txt", "世界", "朋友"))
        .unwrap();
    assert_eq!(std::fs::read_to_string(&file).unwrap(), before);
    assert!(request
        .diff
        .as_ref()
        .unwrap()
        .contains("-Hello 世界\r\n+Hello 朋友\r\n"));
    assert!(request
        .diff
        .as_ref()
        .unwrap()
        .contains("\\ No newline at end of file"));
    let result = edit
        .invoke(&request, CancellationToken::new())
        .await
        .unwrap();
    assert_eq!(
        std::fs::read_to_string(&file).unwrap(),
        "Title\r\nHello 朋友\r\nlast line"
    );
    assert_eq!(
        serde_json::from_str::<serde_json::Value>(&result).unwrap()["applied"],
        true
    );
    assert!(edit
        .invoke(&request, CancellationToken::new())
        .await
        .is_err());
    assert_eq!(
        std::fs::read_dir(root.path().join("notes"))
            .unwrap()
            .count(),
        1
    );
}

#[test]
fn rejects_ambiguous_overlapping_invalid_binary_secret_and_oversized_edits() {
    let root = tempfile::tempdir().unwrap();
    std::fs::write(root.path().join("note"), "aaa 世界世界").unwrap();
    std::fs::write(root.path().join("binary"), b"a\0b").unwrap();
    std::fs::write(root.path().join(".env"), "a").unwrap();
    std::fs::write(root.path().join("large"), vec![b'a'; MAX_TOOL_BYTES + 1]).unwrap();
    let edit = tool(root.path());
    for input in [
        call("note", "aa", "b"),
        call("note", "世界", "朋友"),
        call("note", "missing", "b"),
        call("note", "", "b"),
        call("note", "aaa", "aaa"),
        call("binary", "a", "b"),
        call(".env", "a", "b"),
        call("../outside", "a", "b"),
        call("large", "a", "b"),
        call("note", "aaa", "b\0"),
    ] {
        assert!(edit.prepare(&input).is_err(), "{}", input.arguments);
    }
    let mut extra = call("note", "aaa", "b");
    extra.arguments = r#"{"path":"note","old_text":"aaa","new_text":"b","approved":true}"#.into();
    assert!(edit.prepare(&extra).is_err());
    std::fs::write(
        root.path().join("growing"),
        format!("a{}", "x".repeat(MAX_TOOL_BYTES - 1)),
    )
    .unwrap();
    assert!(edit.prepare(&call("growing", "a", "longer")).is_err());
    std::fs::write(root.path().join("readonly"), "a").unwrap();
    let original_permissions = std::fs::metadata(root.path().join("readonly"))
        .unwrap()
        .permissions();
    let mut permissions = original_permissions.clone();
    permissions.set_readonly(true);
    std::fs::set_permissions(root.path().join("readonly"), permissions.clone()).unwrap();
    assert!(edit.prepare(&call("readonly", "a", "b")).is_err());
    std::fs::set_permissions(root.path().join("readonly"), original_permissions).unwrap();
}

#[tokio::test]
async fn changed_file_cancel_and_changed_approval_leave_original_or_external_contents_untouched() {
    for mode in 0..4 {
        let root = tempfile::tempdir().unwrap();
        let file = root.path().join("note");
        std::fs::write(&file, "before").unwrap();
        let edit = tool(root.path());
        let mut request = edit.prepare(&call("note", "before", "after")).unwrap();
        let cancel = CancellationToken::new();
        match mode {
            0 => std::fs::write(&file, "external change").unwrap(),
            1 => cancel.cancel(),
            2 => request.diff = Some("fake diff".into()),
            _ => request.target = "another".into(),
        }
        let error = edit.invoke(&request, cancel).await.unwrap_err();
        if mode == 0 {
            assert_eq!(error, CHANGED);
        }
        assert_eq!(
            std::fs::read_to_string(&file).unwrap(),
            if mode == 0 {
                "external change"
            } else {
                "before"
            }
        );
        assert_eq!(std::fs::read_dir(root.path()).unwrap().count(), 1);
    }
}

#[tokio::test]
async fn capability_and_alias_checks_block_parent_redirection_after_preview() {
    let root = tempfile::tempdir().unwrap();
    let outside = tempfile::tempdir().unwrap();
    std::fs::create_dir(root.path().join("notes")).unwrap();
    std::fs::write(root.path().join("notes/note"), "before").unwrap();
    std::fs::write(outside.path().join("note"), "outside").unwrap();
    let edit = tool(root.path());
    let request = edit
        .prepare(&call("notes/note", "before", "after"))
        .unwrap();
    #[cfg(windows)]
    {
        // The held directory capability prevents Windows from renaming this
        // parent during approval. Independently prove outside aliases fail.
        assert!(std::fs::rename(root.path().join("notes"), root.path().join("original")).is_err());
        let output = std::process::Command::new("cmd")
            .args(["/c", "mklink", "/J"])
            .arg(root.path().join("alias"))
            .arg(outside.path())
            .output()
            .unwrap();
        assert!(output.status.success());
        assert!(edit
            .prepare(&call("alias/note", "outside", "after"))
            .is_err());
        std::fs::remove_dir(root.path().join("alias")).unwrap();
        let cancel = CancellationToken::new();
        cancel.cancel();
        assert!(edit.invoke(&request, cancel).await.is_err());
        assert_eq!(
            std::fs::read_to_string(root.path().join("notes/note")).unwrap(),
            "before"
        );
    }
    #[cfg(unix)]
    {
        std::fs::rename(root.path().join("notes"), root.path().join("original")).unwrap();
        std::os::unix::fs::symlink(outside.path(), root.path().join("notes")).unwrap();
        assert!(edit
            .invoke(&request, CancellationToken::new())
            .await
            .is_err());
        assert_eq!(
            std::fs::read_to_string(root.path().join("original/note")).unwrap(),
            "before"
        );
        assert!(edit
            .prepare(&call("notes/note", "outside", "after"))
            .is_err());
    }
    assert_eq!(
        std::fs::read_to_string(outside.path().join("note")).unwrap(),
        "outside"
    );
}

#[test]
fn diff_keeps_real_line_numbers_context_and_deletion_without_rendering_markdown() {
    let source = "1\n2\n3\n4\n5\n6\n7\n8\n9\n10\n";
    let preview = diff(
        source,
        &source.replace("8\n", "# heading\n<script>literal</script>\n"),
    );
    assert!(preview.contains("@@ -5,6 +5,7 @@"));
    assert!(preview.contains("-8\n+# heading\n+<script>literal</script>\n"));
    assert_eq!(
        diff("a", ""),
        "--- before\n+++ after\n@@ -1,1 +0,0 @@\n-a\n\\ No newline at end of file\n"
    );
}

#[cfg(windows)]
#[tokio::test]
async fn failed_publication_cleans_staged_file_and_preserves_original_bytes() {
    use std::os::windows::fs::OpenOptionsExt;
    let root = tempfile::tempdir().unwrap();
    let path = root.path().join("note");
    std::fs::write(&path, "before").unwrap();
    let edit = tool(root.path());
    let request = edit.prepare(&call("note", "before", "after")).unwrap();
    // Permit the validation reads, but prevent atomic replacement on Windows.
    let held = std::fs::OpenOptions::new()
        .read(true)
        .share_mode(3)
        .open(&path)
        .unwrap();
    assert!(edit
        .invoke(&request, CancellationToken::new())
        .await
        .is_err());
    assert_eq!(std::fs::read_to_string(&path).unwrap(), "before");
    assert_eq!(std::fs::read_dir(root.path()).unwrap().count(), 1);
    drop(held);
}

struct EditModel;
#[async_trait]
impl ModelProvider for EditModel {
    fn descriptor(&self) -> PluginDescriptor {
        PluginDescriptor {
            id: "test",
            kind: "provider",
            api_version: 1,
        }
    }
    async fn stream(
        &self,
        _: Vec<Message>,
        _: mpsc::Sender<String>,
        _: CancellationToken,
    ) -> Result<(), String> {
        unreachable!()
    }
    async fn tool_turn(
        &self,
        messages: &[AgentMessage],
        _: &[ToolSpec],
        _: CancellationToken,
    ) -> Result<AgentTurn, String> {
        let count = messages.iter().filter(|m| m.role == "tool").count();
        let calls = if count < 2 {
            let mut next = call(
                "note",
                "before",
                if count == 0 { "after" } else { "different" },
            );
            next.id = format!("edit-{count}");
            vec![next]
        } else {
            vec![]
        };
        Ok(AgentTurn {
            output_limit: false,
            content: if calls.is_empty() {
                "Finished".into()
            } else {
                String::new()
            },
            calls,
            usage: None,
        })
    }
}
struct DenyEdits {
    count: std::sync::atomic::AtomicUsize,
}
#[async_trait]
impl ToolApproval for DenyEdits {
    async fn authorize(&self, request: &ToolRequest, _: CancellationToken) -> Result<bool, String> {
        assert!(request.diff.as_ref().unwrap().contains("-before"));
        self.count.fetch_add(1, Ordering::SeqCst);
        Ok(false)
    }
}
#[tokio::test]
async fn model_requests_and_denied_diffs_never_write_and_different_proposals_need_separate_decisions(
) {
    let root = tempfile::tempdir().unwrap();
    std::fs::write(root.path().join("note"), "before").unwrap();
    let approval = DenyEdits {
        count: std::sync::atomic::AtomicUsize::new(0),
    };
    let context = vec![
        Message {
            role: Role::System,
            content: "Local rules".into(),
        },
        Message {
            role: Role::User,
            content: "Change the note".into(),
        },
    ];
    let (events, _receiver) = mpsc::channel(32);
    let result = dolores_core::run_agent(
        &EditModel,
        context,
        &super::super::folder_tools(root.path()).unwrap(),
        &approval,
        events,
        CancellationToken::new(),
    )
    .await
    .unwrap();
    assert_eq!(approval.count.load(Ordering::SeqCst), 2);
    assert_eq!(result.summary.tools.len(), 2);
    assert!(result
        .summary
        .tools
        .iter()
        .all(|r| r.status == "denied" && r.diff.is_some()));
    assert_eq!(
        std::fs::read_to_string(root.path().join("note")).unwrap(),
        "before"
    );
    assert_eq!(std::fs::read_dir(root.path()).unwrap().count(), 1);
}
