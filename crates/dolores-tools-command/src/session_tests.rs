use super::*;
use serde_json::Value;

#[cfg(windows)]
#[tokio::test]
async fn isolated_session_builds_local_modules_and_preserves_review_boundaries() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path().join("project 世界");
    std::fs::create_dir(&root).unwrap();
    std::fs::write(
        root.join("helper.cjs"),
        "exports.render = s => '<h1>'+s+'</h1>';",
    )
    .unwrap();
    std::fs::write(root.join("build.cjs"), "const fs=require('fs'); const helper=require('./helper.cjs'); fs.writeFileSync('index.html',helper.render('PUBLIC_BUILD')); const child=require('child_process').spawnSync('node',['-e',\"require('fs').writeFileSync('child.txt','PUBLIC_CHILD')\"],{stdio:'inherit',timeout:2000}); if(child.error||child.status!==0) throw child.error||Error(child.stderr); console.log('BUILT');").unwrap();
    let outside = dir.path().join("outside.txt");
    std::fs::write(&outside, "KEEP").unwrap();
    let host = isolation::Host::open(&dir.path().join("private-state")).unwrap();
    let commands = tool(&root).with_isolation(host.clone());
    let request = commands.prepare(&call(json!({"operation":"start","program":"node","args":["--preserve-symlinks","--preserve-symlinks-main","build.cjs"],"isolation":"isolated"}))).unwrap();
    assert_eq!(
        request
            .command
            .as_ref()
            .unwrap()
            .isolation
            .as_ref()
            .unwrap()
            .workspace,
        root.canonicalize().unwrap().to_string_lossy()
    );
    let value: Value = serde_json::from_str(
        &commands
            .invoke(&request, CancellationToken::new())
            .await
            .unwrap(),
    )
    .unwrap();
    let id = value["session"]["id"].as_str().unwrap();
    let receipt = until(&commands, id, |r| !r.live()).await;
    assert_eq!(
        receipt.exit_code,
        Some(0),
        "{:?}",
        commands.snapshots().unwrap()[0].output
    );
    assert!(receipt.execution.starts_with("isolated"));
    assert!(receipt.cleanup_complete);
    assert_eq!(
        std::fs::read_to_string(root.join("index.html")).unwrap(),
        "<h1>PUBLIC_BUILD</h1>"
    );
    assert_eq!(
        std::fs::read_to_string(root.join("child.txt")).unwrap(),
        "PUBLIC_CHILD"
    );
    let script = format!(
        "try{{require('fs').writeFileSync({},'BAD')}}catch(e){{console.log(e.code)}}",
        serde_json::to_string(&outside).unwrap()
    );
    let value = op(
        &commands,
        json!({"operation":"start","program":"node","args":["-e",script],"isolation":"isolated"}),
    )
    .await
    .unwrap();
    let id = value["session"]["id"].as_str().unwrap();
    until(&commands, id, |r| !r.live()).await;
    assert!(commands.inspect(id, 0).unwrap()["output"]
        .as_str()
        .unwrap()
        .contains("EPERM"));
    assert_eq!(std::fs::read_to_string(outside).unwrap(), "KEEP");
    assert!(commands
        .prepare(&call(
            json!({"operation":"start","program":"python","args":[],"isolation":"isolated"})
        ))
        .err()
        .unwrap()
        .contains("unsupported"));
    commands.finish().unwrap();
    assert_eq!(host.pending_cleanup().unwrap(), 0);
}

struct Evidence(AtomicBool);
impl SessionEvidence for Evidence {
    fn save(&self, record: &SessionRecord) -> Result<(), String> {
        record.validate()?;
        if self.0.load(Ordering::SeqCst) {
            Ok(())
        } else {
            Err("Fixture evidence storage unavailable.".into())
        }
    }
}
#[tokio::test]
async fn unavailable_evidence_prevents_launch_and_explicit_fresh_run_recovers() {
    let dir = tempfile::tempdir().unwrap();
    let evidence = Arc::new(Evidence(AtomicBool::new(false)));
    let failed = tool(dir.path()).with_evidence(evidence.clone());
    assert!(op(&failed,json!({"operation":"start","program":"node","args":["-e","require('fs').writeFileSync('bad','bad')"]})).await.is_err());
    let id = failed.receipts().unwrap()[0].id.clone();
    let receipt = until(&failed, &id, |r| !r.live()).await;
    assert_eq!(receipt.state, "failed");
    assert!(receipt.cleanup_complete);
    assert!(!dir.path().join("bad").exists());
    evidence.0.store(true, Ordering::SeqCst);
    let fresh = tool(dir.path()).with_evidence(evidence);
    let id = start(&fresh, "console.log('explicit recovery')").await;
    until(&fresh, &id, |r| !r.live()).await;
    fresh.finish().unwrap();
    fresh.persist().unwrap();
    assert_eq!(fresh.snapshots().unwrap()[0].receipt.exit_code, Some(0));
}

fn tool(root: &Path) -> CommandSessions {
    let tool = CommandSessions::with_host(
        root,
        "public-task".into(),
        "public-run".into(),
        CancellationToken::new(),
        Arc::new(Host::default()),
    )
    .unwrap();
    // Independent deterministic hosts; the admission test explicitly shares one.
    tool
}
fn call(value: Value) -> ToolCall {
    ToolCall {
        id: format!("test-{}", NEXT.fetch_add(1, Ordering::SeqCst)),
        name: "command_session".into(),
        arguments: value.to_string(),
    }
}
async fn op(tool: &CommandSessions, value: Value) -> Result<Value, String> {
    let request = tool.prepare(&call(value))?;
    let output = tool.invoke(&request, CancellationToken::new()).await?;
    assert!(output.len() <= MAX_TOOL_BYTES);
    Ok(serde_json::from_str(&output).unwrap())
}
async fn start(tool: &CommandSessions, script: &str) -> String {
    let value = op(
        tool,
        json!({"operation":"start","program":"node","args":["-e",script]}),
    )
    .await
    .unwrap();
    value["session"]["id"].as_str().unwrap().into()
}
async fn until(
    tool: &CommandSessions,
    id: &str,
    predicate: impl Fn(&SessionReceipt) -> bool,
) -> SessionReceipt {
    let end = Instant::now() + Duration::from_secs(6);
    loop {
        let receipt = tool
            .receipts()
            .unwrap()
            .into_iter()
            .find(|r| r.id == id)
            .unwrap();
        if predicate(&receipt) {
            return receipt;
        }
        assert!(Instant::now() < end, "receipt did not settle: {receipt:?}");
        tokio::time::sleep(Duration::from_millis(10)).await;
    }
}

#[tokio::test]
async fn unicode_input_is_reviewed_once_and_stale_input_never_replays() {
    let dir = tempfile::tempdir().unwrap();
    let tool = tool(dir.path());
    let id = start(&tool, "process.stdin.on('data',b=>{require('fs').appendFileSync('received',b);process.stdout.write(b)});console.log('ready')").await;
    until(&tool, &id, |r| r.output_bytes >= 5).await;
    let value = json!({"operation":"input","session":id,"revision":0,"text":"public 世界\n"});
    let mut pretty = call(value.clone());
    pretty.arguments = serde_json::to_string_pretty(&value).unwrap();
    let first = tool.prepare(&pretty).unwrap();
    assert!(!first.query.as_ref().unwrap().chars().any(char::is_control));
    let stale = tool.prepare(&call(value)).unwrap();
    assert!(!dir.path().join("received").exists());
    let sent: Value =
        serde_json::from_str(&tool.invoke(&first, CancellationToken::new()).await.unwrap())
            .unwrap();
    assert_eq!(sent["session"]["lastInput"]["state"], "pending");
    assert!(tool.invoke(&first, CancellationToken::new()).await.is_err());
    until(&tool, &id, |r| {
        r.last_input.as_ref().is_some_and(|i| i.state == "written") && r.output_bytes > 10
    })
    .await;
    assert!(tool
        .invoke(&stale, CancellationToken::new())
        .await
        .unwrap_err()
        .contains("Input changed"));
    assert_eq!(
        std::fs::read_to_string(dir.path().join("received")).unwrap(),
        "public 世界\n"
    );
    let receipt = op(&tool, json!({"operation":"read","session":id}))
        .await
        .unwrap();
    assert!(receipt["output"].as_str().unwrap().contains("public 世界"));
    assert_eq!(receipt["session"]["execution"], "native; not sandboxed");
    op(
        &tool,
        json!({"operation":"input","session":id,"revision":1,"text":"","eof":true}),
    )
    .await
    .unwrap();
    assert!(op(
        &tool,
        json!({"operation":"input","session":id,"revision":2,"text":"no"})
    )
    .await
    .is_err());
    op(&tool, json!({"operation":"interrupt","session":id}))
        .await
        .unwrap();
    let final_receipt = until(&tool, &id, |r| !r.live()).await;
    assert_eq!(final_receipt.state, "interrupted");
    assert!(final_receipt.cleanup_complete);
}

#[tokio::test]
async fn flood_and_deadline_preserve_output_and_allow_explicit_new_session() {
    let dir = tempfile::tempdir().unwrap();
    let tool = tool(dir.path());
    let output = op(&tool,json!({"operation":"start","program":"node","args":["-e","process.stdout.write(Buffer.alloc(65536,65));setInterval(()=>{},10000)"],"capture_bytes":1024})).await.unwrap();
    let id = output["session"]["id"].as_str().unwrap();
    let receipt = until(&tool, id, |r| !r.live()).await;
    assert_eq!(receipt.state, "outputLimit");
    assert!(receipt.truncated);
    assert!(receipt.cleanup_complete);
    let page = op(&tool, json!({"operation":"read","session":id}))
        .await
        .unwrap();
    assert_eq!(page["output"].as_str().unwrap().len(), 1024);
    let output = op(&tool,json!({"operation":"start","program":"node","args":["-e","console.log('retained');setTimeout(()=>require('fs').writeFileSync('late','bad'),3000)"],"timeout_seconds":1})).await.unwrap();
    let timed = output["session"]["id"].as_str().unwrap();
    assert_eq!(until(&tool, timed, |r| !r.live()).await.state, "timedOut");
    assert!(!dir.path().join("late").exists());
    let recovery = start(&tool, "console.log('explicit recovery')").await;
    let receipt = until(&tool, &recovery, |r| !r.live()).await;
    assert_eq!(receipt.state, "completed");
    assert_eq!(receipt.exit_code, Some(0));
}

#[tokio::test]
async fn unread_stdin_cannot_block_interrupt_or_leak_replacement_workers() {
    let dir = tempfile::tempdir().unwrap();
    let tool = tool(dir.path());
    let id = start(&tool, "console.log('ready');setInterval(()=>{},10000)").await;
    until(&tool, &id, |r| r.output_bytes > 0).await;
    let mut blocked = false;
    for revision in 0..16 {
        op(&tool,json!({"operation":"input","session":id,"revision":revision,"text":"x".repeat(MAX_INPUT)})).await.unwrap();
        let until_write = Instant::now() + Duration::from_millis(120);
        loop {
            let receipt = tool.receipts().unwrap().pop().unwrap();
            if receipt
                .last_input
                .as_ref()
                .is_some_and(|i| i.state != "pending")
            {
                break;
            }
            if Instant::now() >= until_write {
                blocked = true;
                break;
            }
            tokio::time::sleep(Duration::from_millis(5)).await;
        }
        if blocked {
            break;
        }
    }
    assert!(blocked, "fixture did not exhaust the unread pipe");
    let began = Instant::now();
    op(&tool, json!({"operation":"interrupt","session":id}))
        .await
        .unwrap();
    let receipt = until(&tool, &id, |r| !r.live()).await;
    assert!(began.elapsed() < Duration::from_secs(3));
    assert!(receipt.cleanup_complete);
    assert_eq!(receipt.state, "interrupted");
    assert_ne!(receipt.last_input.unwrap().state, "pending");
    let replacement = start(&tool, "console.log('fresh')").await;
    assert_eq!(
        until(&tool, &replacement, |r| !r.live()).await.exit_code,
        Some(0)
    );
}

#[tokio::test]
async fn owner_stop_and_finish_kill_descendants_and_handles_never_reattach() {
    for finish in [false, true] {
        let dir = tempfile::tempdir().unwrap();
        let tool = tool(dir.path());
        let child = "setTimeout(()=>require('fs').writeFileSync('late','bad'),2500)";
        let script=format!("require('child_process').spawn(process.execPath,['-e',{}],{{stdio:'inherit'}});console.log('ready');setInterval(()=>{{}},10000)",serde_json::to_string(child).unwrap());
        let id = start(&tool, &script).await;
        until(&tool, &id, |r| r.output_bytes > 0).await;
        let held = tool
            .prepare(&call(json!({"operation":"start","program":"node",
            "args":["-e","require('fs').writeFileSync('posthumous','bad')"]})))
            .unwrap();
        let other = CommandSessions::new(
            dir.path(),
            "other-task".into(),
            "other-run".into(),
            CancellationToken::new(),
        )
        .unwrap();
        assert!(other
            .prepare(&call(json!({"operation":"read","session":id})))
            .is_err());
        if finish {
            tool.finish().unwrap();
        } else {
            tool.owner.cancel();
        }
        let receipt = until(&tool, &id, |r| !r.live()).await;
        assert!(receipt.cleanup_complete);
        assert!(receipt.output_bytes > 0);
        assert!(tool.invoke(&held, CancellationToken::new()).await.is_err());
        assert!(!dir.path().join("posthumous").exists());
        assert!(tool.inspect(&id, 0).unwrap()["output"]
            .as_str()
            .unwrap()
            .contains("ready"));
        tokio::time::sleep(Duration::from_millis(2600)).await;
        assert!(!dir.path().join("late").exists());
        assert!(other.receipts().unwrap().is_empty());
    }
}

#[tokio::test]
async fn task_host_admission_and_close_release_only_finished_slots() {
    let dir = tempfile::tempdir().unwrap();
    let first = tool(dir.path());
    let second = CommandSessions::with_host(
        dir.path(),
        "second-task".into(),
        "second-run".into(),
        CancellationToken::new(),
        first.host.clone(),
    )
    .unwrap();
    let third = CommandSessions::with_host(
        dir.path(),
        "third-task".into(),
        "third-run".into(),
        CancellationToken::new(),
        first.host.clone(),
    )
    .unwrap();
    let a = start(&first, "setInterval(()=>{},10000)").await;
    start(&first, "setInterval(()=>{},10000)").await;
    let start_args =
        json!({"operation":"start","program":"node","args":["-e","setInterval(()=>{},10000)"]});
    let same_task = CommandSessions::with_host(
        dir.path(),
        first.task.clone(),
        "later-run".into(),
        CancellationToken::new(),
        first.host.clone(),
    )
    .unwrap();
    assert!(op(&same_task, start_args.clone())
        .await
        .unwrap_err()
        .contains("Two command"));
    assert!(op(&first, start_args.clone())
        .await
        .unwrap_err()
        .contains("Two command"));
    start(&second, "setInterval(()=>{},10000)").await;
    start(&second, "setInterval(()=>{},10000)").await;
    assert!(op(&third, start_args.clone())
        .await
        .unwrap_err()
        .contains("Four command"));
    let closing = op(&first, json!({"operation":"close","session":a}))
        .await
        .unwrap();
    assert_eq!(closing["handleReleased"], false);
    until(&first, &a, |r| !r.live()).await;
    let closed = op(&first, json!({"operation":"close","session":a}))
        .await
        .unwrap();
    assert_eq!(closed["handleReleased"], true);
    assert!(op(&first, json!({"operation":"read","session":a}))
        .await
        .is_err());
    let fresh = start(&third, "console.log('available')").await;
    assert_eq!(
        until(&third, &fresh, |r| !r.live()).await.state,
        "completed"
    );
}

#[tokio::test]
async fn unsupported_isolation_tampering_and_stale_folder_never_start_commands() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path().join("project");
    std::fs::create_dir(&root).unwrap();
    let tool = tool(&root);
    let args = json!({"operation":"start","program":"node","args":["-e","require('fs').writeFileSync('bad','bad')"]});
    let mut isolated = args.clone();
    isolated["isolation"] = json!("isolated");
    assert!(op(&tool, isolated)
        .await
        .unwrap_err()
        .contains("no native fallback"));
    let request = tool.prepare(&call(args.clone())).unwrap();
    let mut changed = request.clone();
    changed.query = Some("changed".into());
    assert!(tool
        .invoke(&changed, CancellationToken::new())
        .await
        .is_err());
    assert!(tool
        .invoke(&request, CancellationToken::new())
        .await
        .is_err());
    let request = tool.prepare(&call(args)).unwrap();
    std::fs::rename(&root, dir.path().join("moved")).unwrap();
    assert!(tool
        .invoke(&request, CancellationToken::new())
        .await
        .unwrap_err()
        .contains("Working folder changed"));
    assert!(!dir.path().join("moved/bad").exists());
    assert!(tool.receipts().unwrap().is_empty());
}

#[tokio::test]
async fn escaped_output_pages_obey_result_bound_and_cursor_keeps_every_byte() {
    let dir = tempfile::tempdir().unwrap();
    let tool = tool(dir.path());
    let output=op(&tool,json!({"operation":"start","program":"node","args":["-e","process.stdout.write(Buffer.alloc(32768,0))"],"capture_bytes":65536})).await.unwrap();
    let id = output["session"]["id"].as_str().unwrap();
    let receipt = until(&tool, id, |r| !r.live()).await;
    assert_eq!(receipt.state, "completed");
    assert_eq!(receipt.output_bytes, 32768);
    assert!(!receipt.truncated);
    let mut offset = 0;
    loop {
        let page = op(
            &tool,
            json!({"operation":"read","session":id,"offset":offset}),
        )
        .await
        .unwrap();
        let bytes = page["output"].as_str().unwrap().as_bytes();
        assert!(bytes.iter().all(|b| *b == 0));
        assert_eq!(
            page["nextOffset"].as_u64().unwrap() as usize,
            offset + bytes.len()
        );
        offset += bytes.len();
        if page["hasMore"] == false {
            break;
        }
        assert!(!bytes.is_empty());
    }
    assert_eq!(offset, 32768);
    assert!(op(
        &tool,
        json!({"operation":"read","session":id,"offset":32769})
    )
    .await
    .unwrap_err()
    .contains("cursor"));
}
