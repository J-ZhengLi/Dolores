use super::*;
#[test]
fn arguments_refs_and_exact_one_use_reviews_are_bounded_without_starting_runtime() {
    for q in [
        r#"{"operation":"open","url":"file:///secret"}"#,
        r#"{"operation":"open","url":"http://example.com"}"#,
        r#"{"operation":"click","ref":"e1"}"#,
        r#"{"operation":"press","ref":"e1","state":"s","key":"Control+A"}"#,
        r#"{"operation":"state","text":"ignored"}"#,
        r#"{"operation":"fill","ref":"e1","state":"s","text":"x","selector":"body"}"#,
    ] {
        assert!(parse(q).is_err(), "{q}");
    }
    assert!(parse(r#"{"operation":"open","url":"http://127.0.0.1:1234/"}"#).is_ok());
    assert!(parse(r#"{"operation":"fill","ref":"e1","state":"s","text":""}"#).is_ok());
    let b = Browser::new(BrowserRuntime {
        adapter: PathBuf::from("missing"),
        node: PathBuf::from("missing"),
        captures: PathBuf::from("missing"),
    });
    let call = ToolCall {
        id: "one".into(),
        name: "browser".into(),
        arguments: r#"{"operation":"state"}"#.into(),
    };
    let request = b.prepare(&call).unwrap();
    assert!(b.worker.lock().unwrap().is_none());
    assert!(b.prepare(&call).is_err());
    let rt = tokio::runtime::Runtime::new().unwrap();
    let mut changed = request.clone();
    changed.query = Some(r#"{"operation":"close"}"#.into());
    assert!(rt
        .block_on(b.invoke(&changed, CancellationToken::new()))
        .unwrap_err()
        .contains("changed"));
    assert!(rt
        .block_on(b.invoke(&request, CancellationToken::new()))
        .unwrap_err()
        .contains("expired"));
}
#[test]
fn malformed_and_oversized_receipts_never_claim_success() {
    assert!(decode(b"garbage").is_err());
    assert!(decode(br#"{"ok":true,"result":null}"#).is_err());
    assert!(decode(br#"{"ok":false,"error":"fixture unavailable"}"#)
        .unwrap_err()
        .contains("unavailable"));
    let large = json!({"ok":true,"result":{"text":"x".repeat(17000)}});
    assert!(decode(large.to_string().as_bytes()).is_err());
}

#[test]
fn stop_and_dropped_receiver_kill_supervised_descendants_without_late_effects() {
    let filename = if cfg!(windows) { "node.exe" } else { "node" };
    let node = std::env::var_os("PATH")
        .into_iter()
        .flat_map(|v| std::env::split_paths(&v).collect::<Vec<_>>())
        .map(|p| p.join(filename))
        .find(|p| p.is_file());
    let Some(node) = node else {
        eprintln!("Node not installed: process fixture not exercised.");
        return;
    };
    for dropped in [false, true] {
        let dir = tempfile::tempdir().unwrap();
        let runtime = BrowserRuntime {
            adapter: dir.path().to_path_buf(),
            node: node.clone(),
            captures: dir.path().to_path_buf(),
        };
        let (sender, incoming) = mpsc::sync_channel(1);
        let thread = std::thread::spawn(move || {
            supervise_source(
                runtime,
                incoming,
                r#"
const fs=require('fs'),path=require('path'),cp=require('child_process');
require('readline').createInterface({input:process.stdin}).once('line',()=>{
 const folder=process.argv[3];
 cp.spawn(process.execPath,['-e',`require('fs').writeFileSync(process.argv[1], 'started');setTimeout(()=>require('fs').writeFileSync(process.argv[2], 'late'),1500);`,path.join(folder,'ready'),path.join(folder,'late')],{stdio:'ignore'});
});
setInterval(()=>{},1000);
"#,
            )
        });
        let (reply, receiver) = oneshot::channel();
        let cancel = CancellationToken::new();
        sender
            .send(Job {
                query: "{}".into(),
                cancel: cancel.clone(),
                result: reply,
            })
            .unwrap();
        let until = Instant::now() + Duration::from_secs(5);
        while !dir.path().join("ready").exists() && Instant::now() < until {
            std::thread::sleep(Duration::from_millis(10));
        }
        assert!(
            dir.path().join("ready").exists(),
            "fixture child did not start"
        );
        if dropped {
            drop(receiver);
        } else {
            cancel.cancel();
            drop(receiver);
        }
        drop(sender);
        thread.join().unwrap();
        std::thread::sleep(Duration::from_millis(1700));
        assert!(
            !dir.path().join("late").exists(),
            "owned descendant survived Stop/drop"
        );
    }
}
