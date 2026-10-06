use super::*;
fn call(script: &str) -> ToolCall {
    ToolCall {
        id: "one".into(),
        name: "run_command".into(),
        arguments: json!({"program":"node","args":["-e",script]}).to_string(),
    }
}
async fn invoke(tool: &RunCommand, script: &str) -> serde_json::Value {
    let request = tool.prepare(&call(script)).unwrap();
    serde_json::from_str(
        &tool
            .invoke(&request, CancellationToken::new())
            .await
            .unwrap(),
    )
    .unwrap()
}
#[tokio::test]
async fn node_runs_relative_script_from_a_unicode_folder_with_spaces() {
    let dir = tempfile::tempdir().unwrap();
    let folder = dir.path().join("project 世界 with spaces");
    std::fs::create_dir(&folder).unwrap();
    std::fs::write(folder.join("check.cjs"),"const assert=require('node:assert/strict');assert.equal(require('./module.cjs'),42);console.log('relative module passed');").unwrap();
    std::fs::write(folder.join("module.cjs"), "module.exports=42;").unwrap();
    let tool = RunCommand::new(&folder).unwrap();
    let request = tool
        .prepare(&ToolCall {
            arguments: json!({"program":"node","args":["check.cjs"]}).to_string(),
            ..call("")
        })
        .unwrap();
    let result: serde_json::Value = serde_json::from_str(
        &tool
            .invoke(&request, CancellationToken::new())
            .await
            .unwrap(),
    )
    .unwrap();
    assert_eq!(result["exitCode"], 0, "{}", result["stderr"]);
    assert!(result["stdout"]
        .as_str()
        .unwrap()
        .contains("relative module passed"));
}
#[tokio::test]
async fn literal_arguments_unicode_cwd_filtered_environment_and_nonzero_exit_are_retained() {
    let dir = tempfile::tempdir().unwrap();
    let tool = RunCommand::new(dir.path()).unwrap();
    let args = vec!["", "a b", "a\"b", "trailing\\", "$(ignored);&|>", "世界"];
    let script="const fs=require('fs');fs.writeFileSync('proof.txt','世界');process.stdout.write(JSON.stringify(process.argv.slice(1)));process.stderr.write('stderr 世界');process.exitCode=7";
    let request=tool.prepare(&ToolCall{arguments:json!({"program":"node","args":std::iter::once("-e").chain(std::iter::once(script)).chain(args.iter().copied()).collect::<Vec<_>>()} ).to_string(),..call("")}).unwrap();
    assert!(!dir.path().join("proof.txt").exists());
    assert_eq!(request.command.as_ref().unwrap().invocation.args.len(), 8);
    let output: serde_json::Value = serde_json::from_str(
        &tool
            .invoke(&request, CancellationToken::new())
            .await
            .unwrap(),
    )
    .unwrap();
    assert_eq!(
        serde_json::from_str::<Vec<String>>(output["stdout"].as_str().unwrap()).unwrap(),
        args
    );
    assert_eq!(output["exitCode"], 7);
    assert_eq!(output["reason"], "completed");
    assert_eq!(output["stderr"], "stderr 世界");
    assert_eq!(
        std::fs::read_to_string(dir.path().join("proof.txt")).unwrap(),
        "世界"
    );
    assert!(tool
        .invoke(&request, CancellationToken::new())
        .await
        .is_err());
    let env = environment();
    assert!(env
        .iter()
        .all(|(k, _)| !k.ends_with("API_KEY") && !k.ends_with("TOKEN")));
}
#[tokio::test]
async fn invalid_unknown_tampered_and_cancelled_requests_never_execute() {
    let dir = tempfile::tempdir().unwrap();
    let tool = RunCommand::new(dir.path()).unwrap();
    for arguments in [
        json!({"program":"cmd","args":["/c","echo unsafe"]}),
        json!({"program":"./node","args":[]}),
        json!({"program":"node","args":[],"approved":true}),
        json!({"program":"node","args":["\0"]}),
        json!({"program":"node","args":vec!["a";33]}),
    ] {
        assert!(tool
            .prepare(&ToolCall {
                arguments: arguments.to_string(),
                ..call("")
            })
            .is_err());
    }
    let mut request = tool
        .prepare(&call("require('fs').writeFileSync('bad','bad')"))
        .unwrap();
    request
        .command
        .as_mut()
        .unwrap()
        .invocation
        .args
        .push("changed".into());
    assert!(tool
        .invoke(&request, CancellationToken::new())
        .await
        .is_err());
    let request = tool
        .prepare(&ToolCall {
            id: "two".into(),
            ..call("require('fs').writeFileSync('bad','bad')")
        })
        .unwrap();
    let cancel = CancellationToken::new();
    cancel.cancel();
    assert!(tool.invoke(&request, cancel).await.is_err());
    assert!(!dir.path().join("bad").exists());
}
#[tokio::test]
async fn timeout_and_output_budget_stop_processes_and_keep_json_bounded() {
    let dir = tempfile::tempdir().unwrap();
    let mut tool = RunCommand::new(dir.path()).unwrap();
    tool.deadline = Duration::from_millis(150);
    let value = invoke(
        &tool,
        "setTimeout(()=>require('fs').writeFileSync('late','bad'),800)",
    )
    .await;
    assert_eq!(value["reason"], "timedOut");
    assert!(value["exitCode"].is_null());
    std::thread::sleep(Duration::from_millis(850));
    assert!(!dir.path().join("late").exists());
    let tool = RunCommand::new(dir.path()).unwrap();
    let value = invoke(
        &tool,
        "process.stdout.write(Buffer.alloc(65536,0));setTimeout(()=>{},10000)",
    )
    .await;
    assert_eq!(value["reason"], "outputLimit");
    assert_eq!(value["truncated"], true);
    assert!(value.to_string().len() <= MAX_TOOL_BYTES);
    let tool = RunCommand::new(dir.path()).unwrap();
    let value = invoke(&tool, "process.stdout.write(Buffer.from([255,254]))").await;
    assert_eq!(value["lossyUtf8"], true);
}
#[tokio::test]
async fn stop_and_dropped_invocation_kill_child_and_grandchild_without_late_writes() {
    for abort in [false, true] {
        let dir = tempfile::tempdir().unwrap();
        let tool = Arc::new(RunCommand::new(dir.path()).unwrap());
        let child = "setTimeout(()=>require('fs').writeFileSync('late','bad'),1200)";
        let script=format!("require('child_process').spawn(process.execPath,['-e',{}],{{stdio:'inherit'}});require('fs').writeFileSync('started','ok');setTimeout(()=>{{}},10000)",serde_json::to_string(child).unwrap());
        let request = tool.prepare(&call(&script)).unwrap();
        let cancel = CancellationToken::new();
        let copy = tool.clone();
        let token = cancel.clone();
        let task = tokio::spawn(async move { copy.invoke(&request, token).await });
        let deadline = Instant::now() + Duration::from_secs(3);
        while !dir.path().join("started").exists() && Instant::now() < deadline {
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
        assert!(dir.path().join("started").exists());
        if abort {
            task.abort();
            let _ = task.await;
        } else {
            cancel.cancel();
            assert!(task.await.unwrap().is_err());
        }
        tokio::time::sleep(Duration::from_millis(1400)).await;
        assert!(!dir.path().join("late").exists());
    }
}
#[test]
fn path_resolution_refuses_project_shadow_and_relative_entries() {
    let dir = tempfile::tempdir().unwrap();
    let file = resolve(&dir.path().canonicalize().unwrap(), "node").unwrap();
    assert!(!file.starts_with(dir.path()));
    assert!(
        resolve(file.parent().unwrap(), "node").is_err()
            || resolve(file.parent().unwrap(), "node").unwrap() != file
    );
}

#[tokio::test]
async fn reviewed_validation_script_over_one_kib_runs_without_quote_rewriting() {
    let dir = tempfile::tempdir().unwrap();
    let tool = RunCommand::new(dir.path()).unwrap();
    let script = format!(
        "/*{}*/console.log(\"validation passed 世界\")",
        "padding ".repeat(250)
    );
    assert!(script.len() > 1024);
    let request = tool.prepare(&call(&script)).unwrap();
    assert_eq!(request.command.as_ref().unwrap().invocation.args[1], script);
    let output: serde_json::Value = serde_json::from_str(
        &tool
            .invoke(&request, CancellationToken::new())
            .await
            .unwrap(),
    )
    .unwrap();
    assert_eq!(output["exitCode"], 0);
    assert_eq!(
        output["stdout"].as_str().unwrap().trim(),
        "validation passed 世界"
    );
}

#[test]
fn oversized_single_and_aggregate_arguments_offer_script_file_recovery() {
    let dir = tempfile::tempdir().unwrap();
    let tool = RunCommand::new(dir.path()).unwrap();
    for args in [
        vec!["a".repeat(MAX_ARGUMENT_BYTES + 1)],
        vec!["a".repeat(6000); 3],
    ] {
        let error = tool
            .prepare(&ToolCall {
                arguments: json!({"program":"node","args":args}).to_string(),
                ..call("")
            })
            .unwrap_err();
        assert!(error.contains("8 KiB per argument and 16 KiB total"));
        assert!(error.contains("create a script file"));
    }
    assert!(dir.path().read_dir().unwrap().next().is_none());
}

#[tokio::test]
async fn parent_completion_closes_descendants_and_does_not_hang_on_inherited_pipes() {
    let dir = tempfile::tempdir().unwrap();
    let tool = RunCommand::new(dir.path()).unwrap();
    let child = "setTimeout(()=>require('fs').writeFileSync('late','bad'),1200)";
    let script = format!(
        "require('child_process').spawn(process.execPath,['-e',{}],{{stdio:'inherit'}});process.stdout.write('parent done');process.exit(0)",
        serde_json::to_string(child).unwrap()
    );
    let started = Instant::now();
    let result = invoke(&tool, &script).await;
    assert!(started.elapsed() < Duration::from_secs(2));
    assert_eq!(result["exitCode"], 0);
    assert_eq!(result["stdout"], "parent done");
    assert_eq!(result["outputError"], false);
    assert_eq!(result["truncated"], false);
    tokio::time::sleep(Duration::from_millis(1400)).await;
    assert!(!dir.path().join("late").exists());
}

#[tokio::test]
async fn explicit_capture_saves_large_log_and_fresh_deadline_recovers() {
    let dir = tempfile::tempdir().unwrap();
    let tool = RunCommand::new(dir.path()).unwrap();
    let request = tool.prepare(&ToolCall { arguments: json!({"program":"node","args":["-e","process.stdout.write('世界\\n'.repeat(5000))"],"capture_bytes":65536,"timeout_seconds":5}).to_string(), ..call("") }).unwrap();
    let value: serde_json::Value = serde_json::from_str(
        &tool
            .invoke(&request, CancellationToken::new())
            .await
            .unwrap(),
    )
    .unwrap();
    assert_eq!(value["exitCode"], 0);
    assert_eq!(value["truncated"], false);
    assert_eq!(value["previewTruncated"], true);
    let saved =
        std::fs::read_to_string(dir.path().join(value["localLog"].as_str().unwrap())).unwrap();
    assert_eq!(saved.lines().filter(|s| *s == "世界").count(), 5000);
    let request=tool.prepare(&ToolCall{id:"slow".into(),arguments:json!({"program":"node","args":["-e","setTimeout(()=>process.stdout.write('done'),1300)"],"timeout_seconds":1}).to_string(),..call("")}).unwrap();
    let value: serde_json::Value = serde_json::from_str(
        &tool
            .invoke(&request, CancellationToken::new())
            .await
            .unwrap(),
    )
    .unwrap();
    assert_eq!(value["reason"], "timedOut");
    let request=tool.prepare(&ToolCall{id:"retry".into(),arguments:json!({"program":"node","args":["-e","setTimeout(()=>process.stdout.write('done'),1300)"],"timeout_seconds":3}).to_string(),..call("")}).unwrap();
    let value: serde_json::Value = serde_json::from_str(
        &tool
            .invoke(&request, CancellationToken::new())
            .await
            .unwrap(),
    )
    .unwrap();
    assert_eq!(value["stdout"], "done");
    assert_eq!(value["exitCode"], 0);
}
