use super::*;
use dolores_core::{NativeTestRun, RepairEvaluation, RepairFile, SessionStore};

fn receipt() -> RepairEvaluation {
    RepairEvaluation {
        id: "00000000-0000-0000-0000-000000000002".into(),
        session: "a".into(),
        repair_id: "00000000-0000-0000-0000-000000000001".into(),
        revision: 1,
        bundle_id: identity("bundle"),
        candidate_ids: vec![identity("candidate")],
        criteria_id: identity("criteria"),
        cargo_id: identity("cargo"),
        package: "dolores-core".into(),
        reproduction: "#[test] fn reproduction(){assert_eq!(1,1);}".into(),
        status: "started".into(),
        artifact: "evaluation-00000000-0000-0000-0000-000000000002".into(),
        baseline: None,
        candidate: None,
        regression: None,
        note: "Native tests only; no install.".into(),
    }
}
#[test]
fn grading_requires_complete_reproduction_and_preserves_duplicate_test_counts() {
    let mut r = receipt();
    r.baseline = Some(NativeTestRun {
        reason: "completed".into(),
        exit_code: Some(101),
        passed: vec!["regression".into()],
        failed: vec!["reproduce".into()],
        complete: true,
    });
    r.candidate = Some(NativeTestRun {
        reason: "completed".into(),
        exit_code: Some(0),
        passed: vec!["regression".into(), "reproduce".into()],
        failed: vec![],
        complete: true,
    });
    r.regression = r.candidate.clone();
    assert!(r.improved());
    r.candidate.as_mut().unwrap().complete = false;
    assert!(!r.improved());
    r.status = "qualified".into();
    assert!(r.validate().is_err());
    r.candidate.as_mut().unwrap().complete = true;
    r.baseline
        .as_mut()
        .unwrap()
        .passed
        .push("regression".into());
    assert!(!r.improved());
    r.candidate
        .as_mut()
        .unwrap()
        .passed
        .push("regression".into());
    assert!(r.improved());
    r.baseline.as_mut().unwrap().exit_code = Some(0);
    assert!(!r.improved());
}
#[test]
fn fixed_inline_tests_cannot_be_weakened_or_disabled() {
    let before =
        "fn f()->u8 {1} #[cfg(test)] mod tests { #[test] fn equal(){assert_eq!(super::f(),1);} }";
    assert_eq!(
        tests(before).unwrap(),
        tests(&before.replace("{1}", "{2}")).unwrap()
    );
    assert_ne!(
        tests(before).unwrap(),
        tests(&before.replace("assert_eq!(super::f(),1);", "")).unwrap()
    );
    assert_ne!(
        tests(before).unwrap(),
        tests(&before.replace("#[test]", "#[test] #[ignore]")).unwrap()
    );
    assert_ne!(
        tests(before).unwrap(),
        tests(&before.replace("#[cfg(test)]", "#[cfg(any())]")).unwrap()
    );
    assert!(tests("fn f(").is_err());
}
#[test]
fn storage_restart_cross_chat_and_finished_receipt_are_not_replayed() {
    let directory = tempfile::tempdir().unwrap();
    let db = directory.path().join("state.db");
    let store = crate::SqliteStore::open(&db).unwrap();
    store.create("a").unwrap();
    store.create("b").unwrap();
    let r = receipt();
    let mut state = RepairWorkspace {
        id: r.repair_id.clone(),
        session: r.session.clone(),
        revision: 0,
        bundle_id: r.bundle_id.clone(),
        build: "test".into(),
        artifact: "repair-test".into(),
        status: "proposed".into(),
        files: vec![RepairFile {
            path: "crates/fixture/src/lib.rs".into(),
            source_id: identity("before"),
            candidate_id: r.candidate_ids[0].clone(),
            before: "before".into(),
            after: "candidate".into(),
        }],
    };
    store.save_repair_workspace(&state, None).unwrap();
    store.save_repair_evaluation(&r).unwrap();
    assert!(store
        .repair_evaluations("b", &r.repair_id)
        .unwrap()
        .is_empty());
    let mut stolen = r.clone();
    stolen.session = "b".into();
    assert!(store.save_repair_evaluation(&stolen).is_err());
    let mut final_receipt = r.clone();
    final_receipt.status = "stopped".into();
    store.save_repair_evaluation(&final_receipt).unwrap();
    assert!(store.save_repair_evaluation(&r).is_err());
    state.revision = 1;
    assert_eq!(
        store
            .save_repair_workspace(&state, Some(1))
            .unwrap()
            .revision,
        2
    );
    drop(store);
    let reopened = crate::SqliteStore::open(&db).unwrap();
    assert_eq!(
        reopened.repair_evaluations("a", &r.repair_id).unwrap(),
        vec![final_receipt]
    );
}

fn fixture(root: &Path, value: u8) {
    fs::create_dir_all(root.join("src")).unwrap();
    fs::write(
        root.join("Cargo.toml"),
        "[package]\nname=\"dolores-core\"\nversion=\"0.1.0\"\nedition=\"2021\"\n[workspace]\n",
    )
    .unwrap();
    fs::write(
        root.join("Cargo.lock"),
        "version = 4\n[[package]]\nname = \"dolores-core\"\nversion = \"0.1.0\"\n",
    )
    .unwrap();
    fs::write(root.join("src/lib.rs"),format!("pub fn value()->u8 {{{value}}}\n#[cfg(test)] mod tests {{#[test] fn regression(){{assert_eq!(2+2,4);}}}}\n")).unwrap();
    fs::create_dir_all(root.join("tests")).unwrap();
    fs::write(
        root.join("tests/dolores_repair_reproduction.rs"),
        "#[test] fn reproduction(){assert_eq!(dolores_core::value(),2);}",
    )
    .unwrap();
}
#[tokio::test]
async fn real_installed_cargo_executes_fixed_baseline_and_candidate_fixture() {
    let temp = tempfile::tempdir().unwrap();
    let baseline = temp.path().join("baseline");
    let candidate = temp.path().join("candidate");
    fixture(&baseline, 1);
    fixture(&candidate, 2);
    let probe = dolores_tools_command::RunCommand::new(temp.path())
        .unwrap()
        .prepare(&command_call("probe"))
        .unwrap()
        .command
        .unwrap();
    let reproducer=dolores_tools_command::RunCommand::new(temp.path()).unwrap().prepare(&ToolCall{id:"repro".into(),name:"run_command".into(),arguments:json!({"program":"cargo","args":["test","--offline","--locked","-p","dolores-core","--test","dolores_repair_reproduction","--","--test-threads=1"],"timeout_seconds":SECONDS,"capture_bytes":CAPTURE}).to_string()}).unwrap().command.unwrap();
    let b = variant(&baseline, &reproducer, CancellationToken::new())
        .await
        .unwrap();
    let c = variant(&candidate, &reproducer, CancellationToken::new())
        .await
        .unwrap();
    let mut r = receipt();
    r.baseline = Some(b);
    r.regression = Some(
        variant(&candidate, &probe, CancellationToken::new())
            .await
            .unwrap(),
    );
    r.candidate = Some(c);
    assert!(
        r.improved(),
        "{r:?} baseline log: {}",
        fs::read_to_string(baseline.join("reproduction-command.json")).unwrap()
    );
    let token = CancellationToken::new();
    token.cancel();
    assert!(variant(&candidate, &probe, token).await.is_err());
}
#[test]
fn bounded_output_and_changed_source_cannot_qualify() {
    let temp = tempfile::tempdir().unwrap();
    let result = json!({"reason":"outputLimit","exitCode":null,"stdout":"test reproduction ... ok\ntest result: ok","stderr":"","truncated":true,"outputError":false,"lossyUtf8":false});
    assert!(!measured(temp.path(), &result).unwrap().complete);
    let state = RepairWorkspace {
        id: receipt().repair_id,
        session: "a".into(),
        revision: 1,
        bundle_id: bundle::ID.into(),
        build: "test".into(),
        artifact: "repair-test".into(),
        status: "proposed".into(),
        files: vec![RepairFile {
            path: "crates/dolores-core/src/task_budget.rs".into(),
            source_id: "sha256:stale".into(),
            candidate_id: identity("candidate"),
            before: "before".into(),
            after: "candidate".into(),
        }],
    };
    assert!(eligible(&state).is_err());
}

#[tokio::test]
async fn stop_during_native_child_preserves_staged_source_and_completed_marker() {
    let temp = tempfile::tempdir().unwrap();
    fixture(temp.path(), 2);
    let before = fs::read(temp.path().join("src/lib.rs")).unwrap();
    fs::write(temp.path().join("src/lib.rs"),"pub fn value()->u8 {2} #[cfg(test)] mod tests { #[test] fn waiting(){std::fs::write(\"started.marker\",\"preserved\").unwrap();std::thread::sleep(std::time::Duration::from_secs(5));} }").unwrap();
    let source = fs::read(temp.path().join("src/lib.rs")).unwrap();
    let command = dolores_tools_command::RunCommand::new(temp.path())
        .unwrap()
        .prepare(&command_call("stop-probe"))
        .unwrap()
        .command
        .unwrap();
    let token = CancellationToken::new();
    let worker_token = token.clone();
    let marker = temp.path().join("started.marker");
    let observed = marker.clone();
    let cancel_thread = std::thread::spawn(move || {
        let start = std::time::Instant::now();
        while !observed.exists() && start.elapsed() < std::time::Duration::from_secs(10) {
            std::thread::sleep(std::time::Duration::from_millis(10));
        }
        let seen = observed.exists();
        worker_token.cancel();
        seen
    });
    assert!(variant(temp.path(), &command, token).await.is_err());
    assert!(cancel_thread.join().unwrap());
    assert_eq!(fs::read(temp.path().join("src/lib.rs")).unwrap(), source);
    assert_eq!(fs::read_to_string(marker).unwrap(), "preserved");
    assert_ne!(source, before);
}
