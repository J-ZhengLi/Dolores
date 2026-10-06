use super::*;
use dolores_core::{Message, PluginDescriptor};
struct Summarizer {
    calls: std::sync::atomic::AtomicU32,
    invalid: bool,
}
#[async_trait::async_trait]
impl ModelProvider for Summarizer {
    fn descriptor(&self) -> PluginDescriptor {
        PluginDescriptor {
            id: "fixture",
            kind: "provider",
            api_version: 1,
        }
    }
    fn request_settings(&self) -> Option<RequestSettings> {
        Some(RequestSettings {
            max_output_tokens: Some(128),
            timeout_seconds: 30,
            ..Default::default()
        })
    }
    fn context_window_tokens(&self) -> Option<u32> {
        Some(4096)
    }
    async fn stream(
        &self,
        messages: Vec<Message>,
        output: mpsc::Sender<String>,
        _: CancellationToken,
    ) -> Result<(), String> {
        self.calls.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        let summary = messages[0]
            .content
            .starts_with("You draft a session summary");
        let text = if summary && self.invalid {
            "x".repeat(8193)
        } else if summary {
            "Decisions: retain Unicode. Unresolved: finish tests.".into()
        } else {
            assert!(messages[0].content.contains("Original goal: Build parser"));
            "Task reply".into()
        };
        output.send(text).await.map_err(|_| "Closed".into())
    }
}
#[test]
fn compaction_is_opt_in_once_charged_and_failure_preserves_summary_history_and_draft() {
    for invalid in [false, true] {
        let store = Arc::new(SqliteStore::open(std::path::Path::new(":memory:")).unwrap());
        store.create("chat").unwrap();
        for n in 0..30 {
            store
                .commit_turn(
                    "chat",
                    &format!("Build parser goal {n}. {}", "u".repeat(380)),
                    &"a".repeat(400),
                )
                .unwrap();
        }
        store.save_draft("chat", "continue").unwrap();
        assert!(!store.auto_compact("chat").unwrap());
        store.set_auto_compact("chat", true).unwrap();
        let provider = Arc::new(Summarizer {
            calls: std::sync::atomic::AtomicU32::new(0),
            invalid,
        });
        let runtime = Runtime::new().unwrap();
        let (output, mut events) = mpsc::channel(32);
        let result = runtime.block_on(execute(
            store.clone(),
            provider.clone(),
            TurnRequest {
                delegation: None,
                log: None,
                compaction_provider: Some(provider.clone()),
                compacted: false,
                resume_run: None,
                permissions: Default::default(),
                task: Default::default(),
                interaction: Default::default(),
                continuation: None,
                id: 1,
                session: Some("chat".into()),
                input: "continue".into(),
                model: "fixture".into(),
                settings: provider.request_settings(),
                tools: vec![],
                approval: None,
            },
            CancellationToken::new(),
            &output,
        ));
        if invalid {
            assert!(result.unwrap_err().contains("Last valid summary"));
            assert!(store.session_summary("chat").unwrap().is_none());
            assert_eq!(store.context_history("chat").unwrap().1, Some(30));
            assert_eq!(store.saved_draft("chat").unwrap(), "continue");
            assert_eq!(provider.calls.load(std::sync::atomic::Ordering::SeqCst), 1);
        } else {
            assert!(result.is_ok(), "{result:?}");
            assert_eq!(provider.calls.load(std::sync::atomic::Ordering::SeqCst), 2);
            let saved = store.session_summary("chat").unwrap().unwrap();
            assert!(saved.text.starts_with("Original goal: Build parser"));
            assert!(saved.provenance.covered_turns > 0);
            assert_eq!(store.context_history("chat").unwrap().1, Some(31));
            let mut remaining = None;
            while let Ok(e) = events.try_recv() {
                if e["type"] == "started" {
                    remaining = e["taskBudget"]["modelCalls"].as_u64();
                }
            }
            assert_eq!(remaining, Some(63));
        }
    }
}
