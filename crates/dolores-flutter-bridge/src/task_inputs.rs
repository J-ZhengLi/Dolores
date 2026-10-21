//! Live input uses the existing run owner and admission path.
use super::*;
use dolores_core::harness_task::{InputMode, InputState, TaskInput};

pub(super) struct SteeringApproval {
    pub inner: Arc<dyn dolores_core::ToolApproval>,
    pub store: Arc<dyn SessionStore>,
    pub session: String,
    pub run: String,
    pub cancel: CancellationToken,
    pub claimed: Mutex<Vec<String>>,
}

impl SteeringApproval {
    fn pending(&self) -> Result<Vec<TaskInput>, String> {
        let state = self.store.task_inputs(&self.session)?;
        if self.cancel.is_cancelled() || state.suspended || state.recovery_required {
            return Ok(vec![]);
        }
        Ok(state
            .entries
            .into_iter()
            .filter(|v| v.mode == InputMode::Steer && v.state == InputState::Pending)
            .collect())
    }
}
#[async_trait::async_trait]
impl dolores_core::ToolApproval for SteeringApproval {
    fn is_waiting(&self) -> bool {
        self.inner.is_waiting()
    }
    fn steering_pending(&self) -> Result<bool, String> {
        Ok(!self.pending()?.is_empty())
    }
    fn take_steering(&self) -> Result<Vec<String>, String> {
        let mut text = vec![];
        let mut claimed_ids = self
            .claimed
            .lock()
            .map_err(|_| "Steering receipt is unavailable. Inspect pending inputs.".to_string())?;
        if !claimed_ids.is_empty() {
            return Err(
                "Steering delivery is uncertain. Inspect the saved run before resuming.".into(),
            );
        }
        for input in self.pending()? {
            // Editing may win between list and claim. Refuse that stale boundary
            // instead of applying different text under an acknowledged revision.
            let claimed =
                self.store
                    .claim_task_input(&self.session, &input.id, input.revision, &self.run)?;
            text.push(claimed.text);
            claimed_ids.push(input.id);
        }
        Ok(text)
    }
    fn acknowledge_steering(&self) -> Result<(), String> {
        let mut ids = self
            .claimed
            .lock()
            .map_err(|_| "Steering receipt is unavailable.".to_string())?;
        while let Some(id) = ids.first() {
            self.store
                .finish_task_input(&self.session, id, InputState::Applied)?;
            ids.remove(0);
        }
        Ok(())
    }
    async fn recheck(
        &self,
        r: &dolores_core::ToolRequest,
        c: CancellationToken,
    ) -> Result<(), String> {
        self.inner.recheck(r, c).await
    }
    async fn authorize(
        &self,
        r: &dolores_core::ToolRequest,
        c: CancellationToken,
    ) -> Result<bool, String> {
        self.inner.authorize(r, c).await
    }
}

pub(super) fn next(store: &dyn SessionStore, session: &str) -> Result<TaskInput, String> {
    let state = store.task_inputs(session)?;
    if state.suspended || state.recovery_required {
        return Err("Task is paused. Inspect pending inputs and Resume explicitly.".into());
    }
    state
        .entries
        .into_iter()
        .filter(|v| v.state == InputState::Pending)
        .min_by_key(|v| (v.mode != InputMode::Steer, v.position))
        .ok_or_else(|| "No pending instruction. Refresh the queue.".into())
}

#[cfg(test)]
mod tests {
    use super::*;
    use dolores_core::{
        AgentMessage, AgentTurn, Message, PluginDescriptor, ToolApproval, ToolCall, ToolPlugin,
        ToolRequest, ToolSpec,
    };
    use std::sync::atomic::{AtomicUsize, Ordering};
    #[test]
    fn stop_cancels_execution_even_when_suspension_cannot_be_saved() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("stop.db");
        let store = Arc::new(SqliteStore::open(&path).unwrap());
        store.create("chat").unwrap();
        store.suspend_task_inputs("chat").unwrap();
        let engine = Engine::new(
            store,
            Arc::new(crate::connection::testing::MemoryCredentials::default()),
        )
        .unwrap();
        rusqlite::Connection::open(path).unwrap().execute_batch("CREATE TRIGGER reject_suspend BEFORE UPDATE ON task_input_state BEGIN SELECT RAISE(ABORT,'fixture'); END;").unwrap();
        let cancel = CancellationToken::new();
        engine
            .active
            .lock()
            .unwrap()
            .reserve(Run {
                thread: Some("chat".into()),
                id: 777,
                cancel: cancel.clone(),
                events: mpsc::channel(1).1,
                approvals: Arc::new(Mutex::new(None)),
            })
            .unwrap();
        assert!(engine.call(Command::Cancel { id: 777 }).is_err());
        assert!(cancel.is_cancelled());
    }
    struct Fixture {
        store: Arc<SqliteStore>,
        calls: AtomicUsize,
        arrival: &'static str,
    }
    fn steer(store: &dyn SessionStore) {
        store
            .edit_task_inputs(
                "chat",
                &dolores_core::harness_task::InputEdit::Enqueue {
                    id: "steer".into(),
                    text: "Keep the original goal; use UTF-8".into(),
                    mode: InputMode::Steer,
                },
            )
            .unwrap();
    }
    #[async_trait::async_trait]
    impl ModelProvider for Fixture {
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
            _: &[ToolSpec],
            _: CancellationToken,
        ) -> Result<AgentTurn, String> {
            let n = self.calls.fetch_add(1, Ordering::SeqCst);
            if n == 0 {
                if self.arrival == "generation" {
                    steer(self.store.as_ref());
                }
                Ok(AgentTurn {
                    content: "Proposed old operation".into(),
                    calls: vec![ToolCall {
                        id: "call".into(),
                        name: "fixture_tool".into(),
                        arguments: "{}".into(),
                    }],
                    usage: None,
                    output_limit: false,
                })
            } else {
                assert!(messages
                    .iter()
                    .any(|m| m.role == "user" && m.content == "Original goal: write export"));
                assert!(messages
                    .iter()
                    .any(|m| m.role == "user" && m.content.contains("use UTF-8")));
                Ok(AgentTurn {
                    content: "Corrected task finished".into(),
                    calls: vec![],
                    usage: None,
                    output_limit: false,
                })
            }
        }
    }
    struct Tool {
        store: Arc<SqliteStore>,
        effects: AtomicUsize,
        arrival: &'static str,
    }
    #[async_trait::async_trait]
    impl ToolPlugin for Tool {
        fn spec(&self) -> ToolSpec {
            ToolSpec {
                name: "fixture_tool".into(),
                description: "Public fixture".into(),
                parameters: json!({"type":"object"}),
            }
        }
        fn prepare(&self, c: &ToolCall) -> Result<ToolRequest, String> {
            Ok(ToolRequest {
                call_id: c.id.clone(),
                name: c.name.clone(),
                target: "public fixture".into(),
                query: None,
                diff: None,
                command: None,
                mcp: None,
            })
        }
        async fn invoke(&self, _: &ToolRequest, _: CancellationToken) -> Result<String, String> {
            self.effects.fetch_add(1, Ordering::SeqCst);
            if self.arrival == "tool" {
                steer(self.store.as_ref());
            }
            Ok("Completed effect remains".into())
        }
    }
    struct Approve {
        store: Arc<SqliteStore>,
        arrival: &'static str,
    }
    #[async_trait::async_trait]
    impl ToolApproval for Approve {
        async fn recheck(&self, _: &ToolRequest, _: CancellationToken) -> Result<(), String> {
            if self.arrival == "recheck" {
                steer(self.store.as_ref());
            }
            Ok(())
        }
        async fn authorize(&self, _: &ToolRequest, _: CancellationToken) -> Result<bool, String> {
            if self.arrival == "review" {
                steer(self.store.as_ref());
            }
            Ok(true)
        }
    }
    #[test]
    fn steering_at_generation_review_and_tool_boundaries_preserves_goal_and_effects() {
        for arrival in ["generation", "review", "recheck", "tool"] {
            let store = Arc::new(SqliteStore::open(std::path::Path::new(":memory:")).unwrap());
            store.create("chat").unwrap();
            store
                .edit_task_inputs(
                    "chat",
                    &dolores_core::harness_task::InputEdit::Enqueue {
                        id: "queued".into(),
                        text: "Then add an example".into(),
                        mode: InputMode::Queue,
                    },
                )
                .unwrap();
            let fixture = Fixture {
                store: store.clone(),
                calls: AtomicUsize::new(0),
                arrival,
            };
            let tool = Arc::new(Tool {
                store: store.clone(),
                effects: AtomicUsize::new(0),
                arrival,
            });
            let approval = SteeringApproval {
                claimed: Mutex::new(vec![]),
                inner: Arc::new(Approve {
                    store: store.clone(),
                    arrival,
                }),
                store: store.clone(),
                session: "chat".into(),
                run: "run".into(),
                cancel: CancellationToken::new(),
            };
            let runtime = Runtime::new().unwrap();
            let (events, _receiver) = mpsc::channel(64);
            let result = runtime
                .block_on(dolores_core::run_agent(
                    &fixture,
                    dolores_core::prepare_agent_context(
                        prepare_context(vec![], "Original goal: write export").unwrap(),
                    )
                    .unwrap(),
                    &[tool.clone() as Arc<dyn ToolPlugin>],
                    &approval,
                    events,
                    CancellationToken::new(),
                ))
                .unwrap();
            assert_eq!(result.answer, "Corrected task finished");
            assert_eq!(fixture.calls.load(Ordering::SeqCst), 2);
            assert_eq!(
                tool.effects.load(Ordering::SeqCst),
                usize::from(arrival == "tool")
            );
            let state = store.task_inputs("chat").unwrap();
            assert_eq!(state.entries[0].state, InputState::Pending);
            assert_eq!(state.entries[1].state, InputState::Applied);
        }
    }
}
