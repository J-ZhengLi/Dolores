use super::*;
use dolores_core::ChangeJournal;
use std::sync::Mutex;

#[derive(Default)]
struct Journal {
    fail_begin: bool,
    fail_finish: bool,
    cancel: Option<CancellationToken>,
    replace: Option<std::path::PathBuf>,
    entries: Mutex<Vec<(String, String)>>,
    completed: Mutex<Vec<bool>>,
}
impl ChangeJournal for Journal {
    fn begin(&self, _: &str, before: &str, after: &str) -> Result<i64, String> {
        if self.fail_begin {
            return Err("Intent fixture failure".into());
        }
        self.entries
            .lock()
            .unwrap()
            .push((before.into(), after.into()));
        if let Some(cancel) = &self.cancel {
            cancel.cancel();
        }
        if let Some(file) = &self.replace {
            std::fs::write(file, "external").unwrap();
        }
        Ok(42)
    }
    fn finish(&self, _: i64, applied: bool) -> Result<(), String> {
        if self.fail_finish {
            return Err("Receipt fixture failure".into());
        }
        self.completed.lock().unwrap().push(applied);
        Ok(())
    }
}
fn call() -> ToolCall {
    ToolCall {
        id: "edit".into(),
        name: "edit_text_file".into(),
        arguments: json!({"path":"note","old_text":"before","new_text":"after"}).to_string(),
    }
}

#[tokio::test]
async fn intent_failure_prevents_write_but_receipt_failure_preserves_applied_result() {
    let root = tempfile::tempdir().unwrap();
    let file = root.path().join("note");
    for finish in [false, true] {
        std::fs::write(&file, "before 世界\r\n").unwrap();
        let journal = Arc::new(Journal {
            fail_begin: !finish,
            fail_finish: finish,
            ..Default::default()
        });
        let tools = journaled_folder_tools(root.path(), journal.clone()).unwrap();
        let request = tools[3].prepare(&call()).unwrap();
        assert!(journal.entries.lock().unwrap().is_empty());
        let result = tools[3].invoke(&request, CancellationToken::new()).await;
        if finish {
            let value: serde_json::Value = serde_json::from_str(&result.unwrap()).unwrap();
            assert_eq!(value["applied"], true);
            assert_eq!(value["journalStatus"], "pending");
            assert_eq!(std::fs::read_to_string(&file).unwrap(), "after 世界\r\n");
            assert_eq!(journal.entries.lock().unwrap().len(), 1);
        } else {
            assert!(result.is_err());
            assert_eq!(std::fs::read_to_string(&file).unwrap(), "before 世界\r\n");
        }
        assert_eq!(std::fs::read_dir(root.path()).unwrap().count(), 1);
    }
}
#[tokio::test]
async fn cancellation_and_external_edit_during_intent_are_recorded_not_applied() {
    let root = tempfile::tempdir().unwrap();
    let file = root.path().join("note");
    for external in [false, true] {
        std::fs::write(&file, "before").unwrap();
        let cancel = CancellationToken::new();
        let journal = Arc::new(Journal {
            cancel: if external { None } else { Some(cancel.clone()) },
            replace: if external { Some(file.clone()) } else { None },
            ..Default::default()
        });
        let tools = journaled_folder_tools(root.path(), journal.clone()).unwrap();
        let request = tools[3].prepare(&call()).unwrap();
        assert!(tools[3].invoke(&request, cancel).await.is_err());
        assert_eq!(journal.completed.lock().unwrap().as_slice(), [false]);
        assert_eq!(
            std::fs::read_to_string(&file).unwrap(),
            if external { "external" } else { "before" }
        );
    }
}
#[test]
fn revert_checks_direct_paths_current_bytes_and_publication_snapshot() {
    let root = tempfile::tempdir().unwrap();
    let file = root.path().join("note");
    std::fs::write(&file, "after 世界\r\n").unwrap();
    for target in ["../note", ".env", "missing"] {
        assert!(RevertPlan::preview(root.path(), target, "after 世界\r\n", "before").is_err());
    }
    assert!(RevertPlan::preview(root.path(), "note", "wrong", "before").is_err());
    let preview =
        RevertPlan::preview(root.path(), "note", "after 世界\r\n", "before 世界\r\n").unwrap();
    assert!(preview.diff().contains("+before 世界\r\n"));
    std::fs::write(&file, "external").unwrap();
    let journal = Arc::new(Journal::default());
    assert!(preview.apply(journal.clone()).is_err());
    assert!(journal.entries.lock().unwrap().is_empty());
    std::fs::write(&file, "after 世界\r\n").unwrap();
    RevertPlan::preview(root.path(), "note", "after 世界\r\n", "before 世界\r\n")
        .unwrap()
        .apply(journal.clone())
        .unwrap();
    assert_eq!(std::fs::read_to_string(file).unwrap(), "before 世界\r\n");
    assert_eq!(journal.completed.lock().unwrap().as_slice(), [true]);
}
