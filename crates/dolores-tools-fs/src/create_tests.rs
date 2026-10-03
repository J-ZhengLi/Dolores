use super::*;
use dolores_core::ChangeJournal;
use std::sync::Mutex;

#[derive(Default)]
struct Journal {
    fail_begin: bool,
    fail_finish: bool,
    cancel: Option<CancellationToken>,
    occupy: Option<std::path::PathBuf>,
    entries: Mutex<Vec<(Option<String>, Option<String>)>>,
    completed: Mutex<Vec<bool>>,
}
impl ChangeJournal for Journal {
    fn begin(&self, _: &str, _: &str, _: &str) -> Result<i64, String> {
        unreachable!()
    }
    fn begin_file_change(
        &self,
        _: &str,
        before: Option<&str>,
        after: Option<&str>,
    ) -> Result<i64, String> {
        if self.fail_begin {
            return Err("Fixture intent failure".into());
        }
        self.entries
            .lock()
            .unwrap()
            .push((before.map(str::to_owned), after.map(str::to_owned)));
        if let Some(cancel) = &self.cancel {
            cancel.cancel();
        }
        if let Some(file) = &self.occupy {
            std::fs::write(file, "external").unwrap();
        }
        Ok(42)
    }
    fn finish(&self, _: i64, applied: bool) -> Result<(), String> {
        if self.fail_finish {
            return Err("Fixture receipt failure".into());
        }
        self.completed.lock().unwrap().push(applied);
        Ok(())
    }
}
fn call(path: &str, content: &str) -> ToolCall {
    ToolCall {
        id: "create".into(),
        name: "create_text_file".into(),
        arguments: json!({"path":path,"content":content}).to_string(),
    }
}

#[tokio::test]
async fn larger_creation_keeps_complete_review_cancel_journal_and_file_diff_limits() {
    let dir = tempfile::tempdir().unwrap();
    let journal = Arc::new(Journal::default());
    let tools = journaled_folder_tools(dir.path(), journal.clone()).unwrap();
    let content = "// 世界 \\\"quoted\\\"\r\n".repeat(250);
    let proposal = call("main.js", &content);
    assert!(proposal.arguments.len() > 4096);
    dolores_core::validate_call(&proposal).unwrap();
    let request = tools[4].prepare(&proposal).unwrap();
    assert!(request
        .diff
        .as_ref()
        .unwrap()
        .ends_with("+// 世界 \\\"quoted\\\"\r\n"));
    assert!(!dir.path().join("main.js").exists());
    let cancel = CancellationToken::new();
    cancel.cancel();
    assert!(tools[4].invoke(&request, cancel).await.is_err());
    assert!(!dir.path().join("main.js").exists());
    assert!(journal.entries.lock().unwrap().is_empty());
    let request = tools[4].prepare(&proposal).unwrap();
    tools[4]
        .invoke(&request, CancellationToken::new())
        .await
        .unwrap();
    assert_eq!(
        std::fs::read(dir.path().join("main.js")).unwrap(),
        content.as_bytes()
    );
    assert_eq!(journal.entries.lock().unwrap()[0], (None, Some(content)));
    for text in [
        "x".repeat(MAX_TOOL_BYTES + 1),
        "\n".repeat(MAX_TOOL_BYTES / 2),
    ] {
        assert!(tools[4].prepare(&call("too-large.js", &text)).is_err());
        assert!(!dir.path().join("too-large.js").exists());
    }
}
#[tokio::test]
async fn creation_is_previewed_complete_empty_unicode_and_single_use() {
    for content in ["", "# 世界\r\n<script>literal</script>"] {
        let dir = tempfile::tempdir().unwrap();
        std::fs::create_dir(dir.path().join("notes")).unwrap();
        let journal = Arc::new(Journal::default());
        let tools = journaled_folder_tools(dir.path(), journal.clone()).unwrap();
        let request = tools[4].prepare(&call("notes/new.txt", content)).unwrap();
        assert!(request
            .diff
            .as_ref()
            .unwrap()
            .starts_with("--- /dev/null\n+++ after\n"));
        if !content.is_empty() {
            assert!(request.diff.as_ref().unwrap().contains("+# 世界\r\n"));
        }
        let file = dir.path().join("notes/new.txt");
        assert!(!file.exists() && journal.entries.lock().unwrap().is_empty());
        let result: serde_json::Value = serde_json::from_str(
            &tools[4]
                .invoke(&request, CancellationToken::new())
                .await
                .unwrap(),
        )
        .unwrap();
        assert_eq!(result["created"], true);
        assert_eq!(result["journalStatus"], "applied");
        assert_eq!(std::fs::read_to_string(&file).unwrap(), content);
        assert!(tools[4]
            .invoke(&request, CancellationToken::new())
            .await
            .is_err());
        assert_eq!(
            *journal.entries.lock().unwrap(),
            vec![(None, Some(content.into()))]
        );
        assert_eq!(
            std::fs::read_dir(dir.path().join("notes")).unwrap().count(),
            1
        );
    }
}
#[tokio::test]
async fn existing_paths_invalid_arguments_and_changed_approval_never_create() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("note"), "original").unwrap();
    std::fs::create_dir(dir.path().join("folder")).unwrap();
    let tools = folder_tools(dir.path()).unwrap();
    for path in [
        "note",
        "folder",
        "missing/new",
        "../escape",
        ".env",
        ".git/config",
        "CON",
        "note:stream",
        "a\\b",
    ] {
        assert!(tools[4].prepare(&call(path, "new")).is_err(), "{path}");
    }
    for arguments in [
        r#"{"path":"new","content":"x","approved":true}"#.to_owned(),
        call("new", &"x".repeat(dolores_core::MAX_FILE_ARGUMENT_BYTES)).arguments,
        call("new", "\0").arguments,
    ] {
        assert!(tools[4]
            .prepare(&ToolCall {
                arguments,
                ..call("new", "x")
            })
            .is_err());
    }
    let mut request = tools[4].prepare(&call("new", "new")).unwrap();
    request.diff = Some("changed approval".into());
    assert!(tools[4]
        .invoke(&request, CancellationToken::new())
        .await
        .is_err());
    assert!(!dir.path().join("new").exists());
    let request = tools[4]
        .prepare(&ToolCall {
            id: "second".into(),
            ..call("new", "new")
        })
        .unwrap();
    std::fs::write(dir.path().join("new"), "external").unwrap();
    assert_eq!(
        tools[4]
            .invoke(&request, CancellationToken::new())
            .await
            .unwrap_err(),
        create::EXISTS
    );
    assert_eq!(
        std::fs::read_to_string(dir.path().join("new")).unwrap(),
        "external"
    );
}
#[tokio::test]
async fn intent_failure_cancel_conflict_and_receipt_failure_keep_publication_honest() {
    for mode in 0..4 {
        let dir = tempfile::tempdir().unwrap();
        let file = dir.path().join("new");
        let cancel = CancellationToken::new();
        let journal = Arc::new(Journal {
            fail_begin: mode == 0,
            fail_finish: mode == 3,
            cancel: (mode == 1).then(|| cancel.clone()),
            occupy: (mode == 2).then(|| file.clone()),
            ..Default::default()
        });
        let tools = journaled_folder_tools(dir.path(), journal.clone()).unwrap();
        let request = tools[4].prepare(&call("new", "approved")).unwrap();
        let result = tools[4].invoke(&request, cancel).await;
        if mode == 3 {
            let value: serde_json::Value = serde_json::from_str(&result.unwrap()).unwrap();
            assert_eq!(value["journalStatus"], "pending");
            assert_eq!(std::fs::read_to_string(&file).unwrap(), "approved");
        } else {
            assert!(result.is_err());
            if mode == 2 {
                assert_eq!(std::fs::read_to_string(&file).unwrap(), "external");
            } else {
                assert!(!file.exists());
            }
            if mode > 0 {
                assert_eq!(journal.completed.lock().unwrap().as_slice(), [false]);
            }
        }
        assert_eq!(
            std::fs::read_dir(dir.path()).unwrap().count(),
            usize::from(file.exists())
        );
    }
}
#[test]
fn removal_checks_saved_bytes_before_and_after_intent_and_empty_files() {
    for mode in 0..5 {
        let dir = tempfile::tempdir().unwrap();
        let file = dir.path().join("new");
        let content = if mode == 4 { "" } else { "approved 世界\r\n" };
        std::fs::write(&file, content).unwrap();
        let plan = RemoveCreatedPlan::preview(dir.path(), "new", content).unwrap();
        assert!(plan.diff().contains("+++ /dev/null"));
        let journal = Arc::new(Journal {
            fail_begin: mode == 0,
            occupy: (mode == 1).then(|| file.clone()),
            fail_finish: mode == 3,
            ..Default::default()
        });
        if mode == 2 {
            std::fs::write(&file, "external").unwrap();
        }
        let result = plan.apply(journal.clone());
        if mode >= 3 {
            let value: serde_json::Value = serde_json::from_str(&result.unwrap()).unwrap();
            assert_eq!(value["removed"], true);
            assert_eq!(
                value["journalStatus"],
                if mode == 3 { "pending" } else { "applied" }
            );
            assert!(!file.exists());
            assert_eq!(
                *journal.entries.lock().unwrap(),
                vec![(Some(content.into()), None)]
            );
        } else {
            assert!(result.is_err());
            assert_eq!(
                std::fs::read_to_string(&file).unwrap(),
                if mode == 0 { content } else { "external" }
            );
            if mode == 1 {
                assert_eq!(journal.completed.lock().unwrap().as_slice(), [false]);
            }
        }
    }
}
#[tokio::test]
async fn creation_refuses_directory_aliases_even_with_an_inside_destination() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::create_dir(dir.path().join("actual")).unwrap();
    #[cfg(windows)]
    {
        let status = std::process::Command::new("cmd")
            .args(["/c", "mklink", "/J"])
            .arg(dir.path().join("alias"))
            .arg(dir.path().join("actual"))
            .output()
            .unwrap();
        assert!(status.status.success());
    }
    #[cfg(unix)]
    std::os::unix::fs::symlink(dir.path().join("actual"), dir.path().join("alias")).unwrap();
    let tools = folder_tools(dir.path()).unwrap();
    assert!(tools[4].prepare(&call("alias/new", "new")).is_err());
    let request = tools[4].prepare(&call("actual/new", "new")).unwrap();
    tools[4]
        .invoke(&request, CancellationToken::new())
        .await
        .unwrap();
    assert_eq!(
        std::fs::read_to_string(dir.path().join("actual/new")).unwrap(),
        "new"
    );
}
