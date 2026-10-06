use super::*;
use dolores_core::{
    AgentMessage, AgentTurn, PluginDescriptor, RunSnapshot, RunState, SessionStore,
};
use std::sync::atomic::{AtomicUsize, Ordering};

struct Fixture {
    active: AtomicUsize,
    peak: AtomicUsize,
    calls: AtomicUsize,
    hang: bool,
    fail: bool,
    escaped: bool,
    fragmented: bool,
}
struct Active<'a>(&'a AtomicUsize);
impl Drop for Active<'_> {
    fn drop(&mut self) {
        self.0.fetch_sub(1, Ordering::SeqCst);
    }
}
#[async_trait]
impl ModelProvider for Fixture {
    async fn stream_tool_turn(
        &self,
        messages: &[AgentMessage],
        tools: &[ToolSpec],
        text: mpsc::Sender<String>,
        cancel: CancellationToken,
    ) -> Result<AgentTurn, String> {
        let turn = self.tool_turn(messages, tools, cancel).await?;
        if self.fragmented {
            for c in turn.content.chars() {
                text.send(c.to_string())
                    .await
                    .map_err(|_| "Closed fixture stream.")?;
            }
        } else if !turn.content.is_empty() {
            text.send(turn.content.clone())
                .await
                .map_err(|_| "Closed fixture stream.")?;
        }
        Ok(turn)
    }
    fn descriptor(&self) -> PluginDescriptor {
        PluginDescriptor {
            id: "fixture",
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
        tools: &[ToolSpec],
        _: CancellationToken,
    ) -> Result<AgentTurn, String> {
        self.calls.fetch_add(1, Ordering::SeqCst);
        let active = self.active.fetch_add(1, Ordering::SeqCst) + 1;
        let _active = Active(&self.active);
        self.peak.fetch_max(active, Ordering::SeqCst);
        assert!(tools.iter().all(|p| matches!(
            p.name.as_str(),
            "read_text_file" | "list_folder" | "search_text"
        )));
        if self.hang {
            std::future::pending::<()>().await;
        }
        tokio::time::sleep(std::time::Duration::from_millis(20)).await;
        let right = messages[1].content.contains("Read right.txt");
        if self.fail && right {
            return Err("Fixture model unavailable. Retry explicitly.".into());
        }
        let path = if right { "right.txt" } else { "left.txt" };
        Ok(AgentTurn {
            output_limit: false,
            usage: None,
            content: if messages.len() > 2 {
                if self.escaped {
                    "\u{1}".repeat(4000)
                } else {
                    "Read evidence received; no tests were run.".into()
                }
            } else {
                String::new()
            },
            calls: if messages.len() > 2 {
                vec![]
            } else {
                vec![ToolCall {
                    id: "read".into(),
                    name: "read_text_file".into(),
                    arguments: json!({"path":path}).to_string(),
                }]
            },
        })
    }
}
struct Allow;
#[async_trait]
impl ToolApproval for Allow {
    async fn authorize(&self, r: &ToolRequest, _: CancellationToken) -> Result<bool, String> {
        assert!(r.call_id.starts_with("child."));
        Ok(true)
    }
}
fn fixture(hang: bool, fail: bool) -> Arc<Fixture> {
    Arc::new(Fixture {
        active: AtomicUsize::new(0),
        peak: AtomicUsize::new(0),
        calls: AtomicUsize::new(0),
        hang,
        fail,
        escaped: false,
        fragmented: false,
    })
}

#[tokio::test]
async fn encoded_reports_are_shortened_without_losing_status_or_evidence() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("left.txt"), "left").unwrap();
    std::fs::write(dir.path().join("right.txt"), "right").unwrap();
    let mut provider = fixture(false, false);
    Arc::get_mut(&mut provider).unwrap().escaped = true;
    Arc::get_mut(&mut provider).unwrap().fragmented = true;
    let (rt, store, _events) = runtime(
        provider,
        dir.path(),
        TaskBudget {
            model_calls: Some(8),
            tool_calls: Some(8),
            ..Default::default()
        },
    );
    let log = rt.log.clone();
    let tool = DelegateTasks::default();
    tool.bind(rt).unwrap();
    let text = tool
        .invoke(&tool.prepare(&plan()).unwrap(), CancellationToken::new())
        .await
        .unwrap();
    assert!(text.len() < dolores_core::MAX_TOOL_BYTES);
    let value: Value = serde_json::from_str(&text).unwrap();
    assert!(value["children"]
        .as_array()
        .unwrap()
        .iter()
        .all(|c| c["status"] == "reported" && c["truncated"] == true));
    assert!(store
        .run_events("chat", &log.id)
        .unwrap()
        .iter()
        .any(|e| e.kind == "subagentEvidence"));
    assert!(
        store.run_events("chat", &log.id).unwrap().len() < 24,
        "Thousands of SSE text deltas must not consume the durable event allowance"
    );
}
fn runtime(
    provider: Arc<Fixture>,
    directory: &std::path::Path,
    budget: TaskBudget,
) -> (
    RuntimeContext,
    Arc<dolores_store_sqlite::SqliteStore>,
    mpsc::Receiver<Value>,
) {
    let store = Arc::new(
        dolores_store_sqlite::SqliteStore::open(std::path::Path::new(":memory:")).unwrap(),
    );
    store.create("chat").unwrap();
    let id = uuid::Uuid::new_v4().to_string();
    store
        .begin_run(&RunSnapshot {
            id: id.clone(),
            thread: "chat".into(),
            parent_run: None,
            segments: 1,
            model: "fixture".into(),
            settings: Default::default(),
            input: "Read files".into(),
            state: RunState::Prepared,
            sequence: 0,
            created_at: 0,
            build: "fixture".into(),
            tools: vec!["delegate_tasks".into()],
            extensions: vec![],
            effective_settings: None,
        })
        .unwrap();
    let (output, receiver) = mpsc::channel(64);
    let context = vec![
        Message {
            role: Role::System,
            content: "Follow the child's scope.".into(),
            parts: vec![],
        },
        Message {
            role: Role::User,
            content: "Read files".into(),
            parts: vec![],
        },
    ];
    (
        RuntimeContext {
            provider,
            context,
            tools: dolores_tools_fs::folder_tools(directory).unwrap(),
            approval: Arc::new(Allow),
            shared: Arc::new(SharedTaskBudget::new(budget)),
            budget,
            log: RunLog::new(store.clone(), id),
            output,
            client_id: 1,
        },
        store,
        receiver,
    )
}
fn plan() -> ToolCall {
    ToolCall {
        id: "batch".into(),
        name: "delegate_tasks".into(),
        arguments: json!({"tasks":[
        {"goal":"Read left.txt","scope":"left.txt","readOnly":true},
        {"goal":"Read right.txt","scope":"right.txt","readOnly":true}]})
        .to_string(),
    }
}
#[test]
fn malformed_overlap_and_case_aliases_are_refused_before_child_start() {
    let tool = DelegateTasks::default();
    for scopes in [
        ("src", "src/file.rs"),
        ("Src", "src"),
        (".", "other"),
        ("src", "../outside"),
        ("src", "C:/outside"),
    ] {
        let mut call = plan();
        call.arguments=json!({"tasks":[{"goal":"a","scope":scopes.0,"readOnly":false},{"goal":"b","scope":scopes.1,"readOnly":true}]}).to_string();
        assert!(tool.prepare(&call).is_err());
    }
    let mut call = plan();
    call.arguments = "{\"tasks\":[],\"permissions\":\"fullAccess\"}".into();
    assert!(tool.prepare(&call).is_err());
    assert!(tool.prepare(&plan()).is_ok());
    assert!(!within("src", "src2/file"));
    assert!(!within(".", "../outside"));
}
#[tokio::test]
async fn scope_checks_precede_file_preview_and_recheck_changed_dispatch() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("left.txt"), "left").unwrap();
    std::fs::write(dir.path().join("right.txt"), "right").unwrap();
    let tool = ScopedTool {
        inner: dolores_tools_fs::folder_tools(dir.path())
            .unwrap()
            .remove(0),
        scope: "left.txt".into(),
        child: "id".into(),
    };
    let mut call = ToolCall {
        id: "read".into(),
        name: "read_text_file".into(),
        arguments: json!({"path":"right.txt"}).to_string(),
    };
    assert!(tool.prepare(&call).unwrap_err().contains("outside"));
    call.arguments = json!({"path":"left.txt"}).to_string();
    let mut request = tool.prepare(&call).unwrap();
    request.target = "right.txt".into();
    assert!(tool
        .invoke(&request, CancellationToken::new())
        .await
        .is_err());
}
#[tokio::test]
async fn two_children_share_allowance_and_retained_evidence_without_recursive_tools() {
    for (limit, fail) in [(8, false), (4, false), (8, true)] {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("left.txt"), "left").unwrap();
        std::fs::write(dir.path().join("right.txt"), "right").unwrap();
        let provider = fixture(false, fail);
        let budget = TaskBudget {
            model_calls: Some(limit),
            tool_calls: Some(8),
            ..Default::default()
        };
        let (rt, store, _events) = runtime(provider.clone(), dir.path(), budget);
        let shared = rt.shared.clone();
        let log = rt.log.clone();
        assert!(shared.reserve_model(false));
        assert!(shared.reserve_tools(1));
        let tool = DelegateTasks::default();
        tool.bind(rt).unwrap();
        let request = tool.prepare(&plan()).unwrap();
        let report: Value = serde_json::from_str(
            &tool
                .invoke(&request, CancellationToken::new())
                .await
                .unwrap(),
        )
        .unwrap();
        assert_eq!(report["children"].as_array().unwrap().len(), 2);
        assert_eq!(provider.peak.load(Ordering::SeqCst), 2);
        assert!(shared.usage().model_calls < limit);
        assert!(shared.reserve_model(false));
        let statuses: Vec<_> = report["children"]
            .as_array()
            .unwrap()
            .iter()
            .map(|c| c["status"].as_str().unwrap())
            .collect();
        if limit == 4 {
            assert!(statuses.contains(&"paused"));
        } else if fail {
            assert!(statuses.contains(&"failed") && statuses.contains(&"reported"));
        } else {
            assert_eq!(statuses, vec!["reported", "reported"]);
        }
        let evidence = store.run_events("chat", &log.id).unwrap();
        assert!(evidence.iter().any(|e| e.kind == "subagentEvidence"));
        assert!(evidence
            .iter()
            .filter(|e| e.kind == "subagent")
            .all(|e| e.data["parentRunId"] == log.id));
        assert!(tool
            .invoke(&request, CancellationToken::new())
            .await
            .unwrap_err()
            .contains("one subagent batch"));
    }
}
#[tokio::test]
async fn stop_drops_inflight_models_and_records_interruption_before_parent_terminal() {
    let dir = tempfile::tempdir().unwrap();
    let provider = fixture(true, false);
    let (rt, store, _events) = runtime(
        provider.clone(),
        dir.path(),
        TaskBudget {
            model_calls: Some(8),
            ..Default::default()
        },
    );
    let log = rt.log.clone();
    let tool = DelegateTasks::default();
    tool.bind(rt).unwrap();
    let request = tool.prepare(&plan()).unwrap();
    let cancel = CancellationToken::new();
    {
        let run = tool.invoke(&request, cancel.clone());
        tokio::pin!(run);
        tokio::select! { result=&mut run=>panic!("Unexpected result: {result:?}"), _=tokio::time::sleep(std::time::Duration::from_millis(80))=>{} }
        assert_eq!(provider.active.load(Ordering::SeqCst), 2);
        cancel.cancel();
        assert!(run.await.is_err());
    }
    assert_eq!(provider.active.load(Ordering::SeqCst), 0);
    tool.finish(&log, "Stopped; inspect evidence, no replay.")
        .await
        .unwrap();
    let events = store.run_events("chat", &log.id).unwrap();
    assert_eq!(
        events
            .iter()
            .filter(|e| e.kind == "subagent" && e.data["status"] == "interrupted")
            .count(),
        2
    );
}
