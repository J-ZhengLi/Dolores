use super::*;
use async_trait::async_trait;
use dolores_core::{AgentMessage, AgentTurn, Message, PluginDescriptor, ToolCall, ToolSpec};

struct Model;
struct StalledModel;
#[async_trait]
impl ModelProvider for StalledModel {
    fn descriptor(&self) -> PluginDescriptor {
        Model.descriptor()
    }
    fn request_settings(&self) -> Option<RequestSettings> {
        Model.request_settings()
    }
    async fn stream(
        &self,
        _: Vec<Message>,
        _: mpsc::Sender<String>,
        _: CancellationToken,
    ) -> Result<(), String> {
        unreachable!()
    }
    async fn stream_tool_turn(
        &self,
        _: &[AgentMessage],
        _: &[ToolSpec],
        output: mpsc::Sender<String>,
        _: CancellationToken,
    ) -> Result<AgentTurn, String> {
        output.send("Useful partial plan.".into()).await.unwrap();
        std::future::pending().await
    }
}
#[async_trait]
impl ModelProvider for Model {
    fn descriptor(&self) -> PluginDescriptor {
        PluginDescriptor {
            id: "deadline-fixture",
            kind: "provider",
            api_version: 1,
        }
    }
    fn request_settings(&self) -> Option<RequestSettings> {
        Some(RequestSettings {
            timeout_seconds: 1,
            ..Default::default()
        })
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
        let first = !messages.iter().any(|m| m.role == "tool");
        Ok(AgentTurn {
            content: if first {
                "Prepared the file."
            } else {
                "File created."
            }
            .into(),
            calls: if first {
                vec![ToolCall {
                    id: "create-one".into(),
                    name: "create_text_file".into(),
                    arguments: r#"{"path":"index.html","content":"<!doctype html>"}"#.into(),
                }]
            } else {
                vec![]
            },
            usage: None,
            output_limit: false,
        })
    }
}

#[tokio::test]
async fn slow_review_outlives_response_timeout_without_losing_the_file_or_turn() {
    let directory = tempfile::tempdir().unwrap();
    let store = Arc::new(SqliteStore::open(std::path::Path::new(":memory:")).unwrap());
    store.create("review").unwrap();
    let pending = Arc::new(Mutex::new(None));
    let (output, mut events) = mpsc::channel(64);
    let approval = Arc::new(approval::RunApproval {
        policy: None,
        id: 1,
        pending: pending.clone(),
        output: output.clone(),
        log: None,
    });
    let tools = dolores_tools_fs::folder_tools(directory.path()).unwrap();
    let saved = store.clone();
    let run = tokio::spawn(async move {
        execute(
            store,
            Arc::new(Model),
            TurnRequest {
                delegation: None,
                log: None,
                compaction_provider: None,
                compacted: false,
                resume_run: None,
                permissions: Default::default(),
                task: Default::default(),
                interaction: Default::default(),
                continuation: None,
                id: 1,
                session: Some("review".into()),
                input: "Build HTML".into(),
                model: "fixture".into(),
                settings: Some(RequestSettings {
                    timeout_seconds: 1,
                    ..Default::default()
                }),
                tools,
                approval: Some(approval),
            },
            CancellationToken::new(),
            &output,
        )
        .await
    });
    while events.recv().await.unwrap()["type"] != "toolApproval" {}
    tokio::time::sleep(std::time::Duration::from_millis(1300)).await;
    assert!(
        !run.is_finished(),
        "Human review must not spend the response timeout"
    );
    pending
        .lock()
        .unwrap()
        .take()
        .unwrap()
        .reply
        .send(true)
        .unwrap();
    assert_eq!(run.await.unwrap().unwrap(), "File created.");
    assert_eq!(
        std::fs::read_to_string(directory.path().join("index.html")).unwrap(),
        "<!doctype html>"
    );
    assert_eq!(saved.messages("review").unwrap().len(), 2);
}

#[tokio::test]
async fn a_stalled_model_saves_a_paused_turn_without_executing_incomplete_tools() {
    let directory = tempfile::tempdir().unwrap();
    let store = Arc::new(SqliteStore::open(std::path::Path::new(":memory:")).unwrap());
    store.create("stall").unwrap();
    let pending = Arc::new(Mutex::new(None));
    let (output, _events) = mpsc::channel(64);
    let approval = Arc::new(approval::RunApproval {
        policy: None,
        id: 2,
        pending: pending.clone(),
        output: output.clone(),
        log: None,
    });
    let cancel = CancellationToken::new();
    let answer = execute(
        store.clone(),
        Arc::new(StalledModel),
        TurnRequest {
            delegation: None,
            log: None,
            compaction_provider: None,
            compacted: false,
            resume_run: None,
            permissions: Default::default(),
            task: Default::default(),
            interaction: Default::default(),
            continuation: None,
            id: 2,
            session: Some("stall".into()),
            input: "Build HTML".into(),
            model: "fixture".into(),
            settings: Model.request_settings(),
            tools: dolores_tools_fs::folder_tools(directory.path()).unwrap(),
            approval: Some(approval),
        },
        cancel.clone(),
        &output,
    )
    .await
    .unwrap();
    assert!(answer.contains("Useful partial plan."));
    let saved = store.messages_page("stall", None, false, 80).unwrap();
    assert_eq!(saved.items.len(), 2);
    let metadata = saved.items.last().unwrap().metadata.as_ref().unwrap();
    assert_eq!(
        metadata.paused.as_ref().unwrap().reason,
        dolores_core::PauseReason::ResponseTimeout
    );
    assert_eq!(metadata.paused.as_ref().unwrap().task, "Build HTML");
    assert!(metadata.agent.as_ref().unwrap().tools.is_empty());
    assert!(pending.lock().unwrap().is_none());
    assert!(
        !cancel.is_cancelled(),
        "Saving a pause must not become a user Stop"
    );
    assert!(!directory.path().join("index.html").exists());
}

#[tokio::test]
async fn an_explicit_task_clock_excludes_review_and_stop_clears_the_pending_decision() {
    let pending = Arc::new(Mutex::new(None));
    let (output, mut events) = mpsc::channel(4);
    let approval = Arc::new(approval::RunApproval {
        policy: None,
        id: 3,
        pending: pending.clone(),
        output,
        log: None,
    });
    let token = CancellationToken::new();
    let work = token.clone();
    let choice = approval.clone();
    let run = tokio::spawn(async move {
        let request = dolores_core::ToolRequest {
            call_id: "file".into(),
            name: "read_text_file".into(),
            target: "index.html".into(),
            query: None,
            diff: None,
            command: None,
            mcp: None,
        };
        dolores_core::ToolApproval::authorize(choice.as_ref(), &request, work).await
    });
    events.recv().await.unwrap();
    assert!(tokio::time::timeout(
        std::time::Duration::from_millis(1300),
        task_execution::active_deadline(Some(1), approval.as_ref())
    )
    .await
    .is_err());
    token.cancel();
    assert!(run.await.unwrap().is_err());
    assert!(pending.lock().unwrap().is_none());
    tokio::time::timeout(
        std::time::Duration::from_millis(1300),
        task_execution::active_deadline(Some(1), approval.as_ref()),
    )
    .await
    .unwrap();
    let mut progress = task_execution::Progress::default();
    progress.record(&dolores_core::AgentEvent::ModelText {
        number: 1,
        text: "Public progress".into(),
    });
    assert_eq!(
        progress.paused().pause,
        Some(dolores_core::PauseReason::TaskTimeout)
    );
    assert!(progress.paused().answer.contains("Public progress"));
}
