//! A local-only request, not desktop authority. The selected-window segment
//! starts only after a separate user choice and spends the saved remainder.
use async_trait::async_trait;
use base64::Engine as _;
use dolores_core::{SessionStore, TaskBudget, ToolCall, ToolPlugin, ToolRequest, ToolSpec};
use serde::Deserialize;
use serde_json::json;
use sha2::{Digest, Sha256};
use tokio_util::sync::CancellationToken;

impl crate::Engine {
    pub(super) fn check_image_support(
        &self,
        active: &mut crate::run_journal::RunCoordinator,
        id: u64,
        model: String,
        session: Option<String>,
        run_id: Option<String>,
    ) -> Result<serde_json::Value, String> {
        let parent = match (&session, &run_id) {
            (Some(session), Some(run_id)) => {
                let run =
                    crate::checkpoints::resume_source(self.store.as_ref(), session, run_id, true)?;
                let settings = self.effective_settings(Some(session))?;
                let (_, remaining) = source(
                    self.store.as_ref(),
                    session,
                    run_id,
                    &run.input,
                    settings.task,
                    settings.request.timeout_seconds,
                )?;
                if remaining.model_limit() < 3 {
                    return Err("There are too few remaining model calls for an image check and desktop task. Choose an already verified model or start a deliberate new task. Progress remains.".into());
                }
                Some((run, remaining))
            }
            (None, None) => None,
            _ => return Err("Image check needs both the chat and saved task.".into()),
        };
        let provider = self
            .connection
            .lock()
            .map_err(|_| "Connection unavailable.")?
            .image_provider(
                &model,
                dolores_core::RequestSettings {
                    max_output_tokens: Some(64),
                    timeout_seconds: 15,
                    ..Default::default()
                },
                true,
            )?;
        let data = base64::engine::general_purpose::STANDARD.decode("iVBORw0KGgoAAAANSUhEUgAAABAAAAAQCAIAAACQkWg2AAAAF0lEQVR4nGP4z8BAEiJN9aiGUQ1DSgMAkPn/Afnh+ngAAAAASUVORK5CYII=").map_err(|_| "Image check unavailable.")?;
        let reference = dolores_core::AttachmentRef {
            digest: format!("{:x}", Sha256::digest(&data)),
            name: "image-check.png".into(),
            mime: "image/png".into(),
            bytes: data.len(),
        };
        let provider = provider.with_attachment_assets(
            vec![dolores_core::AttachmentData {
                reference: reference.clone(),
                data,
            }],
            true,
        )?;
        let probe_log = if let Some((run, remaining)) = &parent {
            let probe_id = uuid::Uuid::new_v4().to_string();
            self.store.begin_run(&dolores_core::RunSnapshot {
                id: probe_id.clone(),
                parent_run: Some(run.id.clone()),
                segments: run.segments,
                thread: run.thread.clone(),
                model: model.clone(),
                input: run.input.clone(),
                settings: dolores_core::RequestSettings {
                    max_output_tokens: Some(64),
                    timeout_seconds: 15,
                    ..Default::default()
                },
                state: dolores_core::RunState::Prepared,
                sequence: 0,
                created_at: 0,
                build: env!("DOLORES_BUILD_REVISION").into(),
                tools: vec!["check_image_support".into()],
                extensions: vec![],
                effective_settings: None,
            })?;
            let sequence = self.store.append_run_event(
                &probe_id,
                0,
                Some(dolores_core::RunState::Running),
                "started",
                &json!({"syntheticImage":true}),
            )?;
            self.store.append_run_event(
                &probe_id,
                sequence,
                None,
                "desktopHandoff",
                &json!({"modelCalls":1,"toolCalls":0,"elapsedSeconds":15,"budget":remaining}),
            )?;
            Some(probe_id)
        } else {
            None
        };
        let cancel = CancellationToken::new();
        let (output, events) = tokio::sync::mpsc::channel(2);
        active.reserve(crate::Run {
            thread: session,
            id,
            cancel: cancel.clone(),
            events,
            approvals: std::sync::Arc::new(std::sync::Mutex::new(None)),
        })?;
        let store = self.store.clone();
        let endpoint = store.preferences()?.base_url;
        self.runtime.spawn(async move {
            let messages = vec![dolores_core::Message { role:dolores_core::Role::User, content:"Name the dominant color of this image using one English word. If no image is visible, answer unavailable.".into(), parts:vec![reference] }];
            let result = tokio::time::timeout(std::time::Duration::from_secs(15), crate::memory_suggestions::collect_review(provider, messages, cancel.clone(), "Image support check")).await;
            let result = match result {
                Ok(Ok((answer, _))) if answer.trim().trim_matches(|c:char| c.is_ascii_punctuation()).eq_ignore_ascii_case("red") && !cancel.is_cancelled() => {
                    if store.preferences().is_ok_and(|p| p.base_url == endpoint) {
                        store.image_models(&endpoint).and_then(|mut models| { if !models.contains(&model) { models.push(model.clone()); } store.save_image_models(&endpoint, &models) })
                    } else { Err("Connection changed during the image check. Try again with the current connection.".into()) }
                },
                _ => Err("This model did not pass the image check. Choose another model here; no window pixels were shared and no input ran.".into()),
            };
            let result = if let (Some((parent, _)), Some(probe_id)) = (parent, &probe_log) {
                match store.runs(&parent.thread).and_then(|runs| runs.into_iter().find(|r| r.id == *probe_id).ok_or("Saved task is unavailable.".into())).and_then(|run| store.append_run_event(probe_id,run.sequence,Some(dolores_core::RunState::Paused),"finished",&json!({"imageSupport":result.is_ok()}))) {
                    Ok(_) => result, Err(error) => Err(error),
                }
            } else { result };
            let _ = output.try_send(match result { Ok(()) => json!({"type":"done","id":id,"runId":probe_log,"imageSupport":true}), Err(e) => json!({"type":"done","id":id,"runId":probe_log,"error":e}) });
        });
        Ok(json!({"id":id}))
    }
}

pub struct RequestAccess;
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Purpose {
    purpose: String,
}
#[async_trait]
impl ToolPlugin for RequestAccess {
    fn spec(&self) -> ToolSpec {
        ToolSpec {
            name: "request_desktop_access".into(),
            description: "Request the user's local window chooser for a desktop task. This pauses the current goal for a user decision; it does not list windows, share pixels or grant input. Describe the purpose briefly. Never use run_command to bypass desktop consent. After sharing, the host continues this goal with only selected-window tools and the remaining allowance.".into(),
            parameters: json!({"type":"object","properties":{"purpose":{"type":"string","maxLength":240}},"required":["purpose"],"additionalProperties":false}),
        }
    }
    fn prepare(&self, call: &ToolCall) -> Result<ToolRequest, String> {
        let value: Purpose = serde_json::from_str(&call.arguments)
            .map_err(|_| "Describe the window task using a short purpose.".to_owned())?;
        if value.purpose.trim().is_empty()
            || value.purpose.len() > 240
            || value.purpose.chars().any(char::is_control)
        {
            return Err("Use a nonempty purpose of at most 240 bytes.".into());
        }
        Ok(ToolRequest {
            call_id: call.id.clone(),
            name: self.spec().name,
            target: "User-selected window".into(),
            query: Some(value.purpose),
            diff: None,
            command: None,
            mcp: None,
        })
    }
    async fn invoke(
        &self,
        request: &ToolRequest,
        cancel: CancellationToken,
    ) -> Result<String, String> {
        if cancel.is_cancelled() {
            return Err(crate::stopped());
        }
        Ok(
            json!({"purpose": request.query, "status":"awaitingWindowChoice", "shared":false})
                .to_string(),
        )
    }
}

pub fn remainder(
    budget: TaskBudget,
    models: usize,
    tools: usize,
    elapsed: u64,
    timeout: u32,
) -> Result<TaskBudget, String> {
    let remaining = TaskBudget {
        model_calls: Some(budget.model_limit().saturating_sub(models)),
        tool_calls: Some(budget.tool_limit().saturating_sub(tools)),
        elapsed_seconds: Some(
            u64::from(budget.handoff_deadline(timeout))
                .saturating_sub(elapsed)
                .min(3600) as u32,
        ),
        ..budget
    };
    remaining.validate().map_err(|_| "This task has too little remaining allowance to share a window. Progress is saved. Review Task limits or start a deliberate new task; sharing never resets limits.".to_owned())?;
    Ok(remaining)
}

pub fn source(
    store: &dyn SessionStore,
    session: &str,
    id: &str,
    input: &str,
    budget: TaskBudget,
    timeout: u32,
) -> Result<(dolores_core::RunSnapshot, TaskBudget), String> {
    let run = crate::checkpoints::resume_source(store, session, id, true)?;
    let events = store.run_events(session, id)?;
    let usage = events
        .into_iter()
        .rev()
        .find(|e| e.kind == "desktopHandoff")
        .ok_or("This task has no saved window-sharing request. Nothing was restarted.")?;
    if run.input != input || run.state != dolores_core::RunState::Paused {
        return Err("The window-sharing request changed. Return to the original task; nothing was restarted.".into());
    }
    let models = usage.data["modelCalls"]
        .as_u64()
        .ok_or("Saved task usage is unavailable.")? as usize;
    let tools = usage.data["toolCalls"]
        .as_u64()
        .ok_or("Saved task usage is unavailable.")? as usize;
    let elapsed = usage.data["elapsedSeconds"]
        .as_u64()
        .ok_or("Saved task timing is unavailable.")?;
    // A subsequently enlarged setting cannot enlarge this handoff implicitly.
    let original: TaskBudget = serde_json::from_value(usage.data["budget"].clone())
        .map_err(|_| "Saved task allowance is unavailable.")?;
    let bounded = TaskBudget {
        model_calls: Some(budget.model_limit().min(original.model_limit())),
        tool_calls: Some(budget.tool_limit().min(original.tool_limit())),
        segments: budget.segments.min(original.segments),
        elapsed_seconds: Some(
            budget
                .handoff_deadline(timeout)
                .min(original.handoff_deadline(timeout)),
        ),
    };
    bounded.check_segment(run.segments)?;
    Ok((run, remainder(bounded, models, tools, elapsed, timeout)?))
}

#[cfg(test)]
mod tests {
    use super::*;
    fn limited() -> TaskBudget {
        TaskBudget {
            model_calls: Some(4),
            tool_calls: Some(4),
            ..Default::default()
        }
    }
    #[test]
    fn saved_handoff_refuses_changed_goal_and_enlarged_limits_even_after_restart() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("state.db");
        let store = dolores_store_sqlite::SqliteStore::open(&path).unwrap();
        store.create("chat").unwrap();
        let id = "11111111-1111-4111-8111-111111111111";
        let run = dolores_core::RunSnapshot {
            id: id.into(),
            thread: "chat".into(),
            model: "fixture".into(),
            input: "Inspect my local form".into(),
            settings: Default::default(),
            state: dolores_core::RunState::Prepared,
            sequence: 0,
            created_at: 0,
            build: "fixture".into(),
            tools: vec!["request_desktop_access".into()],
            extensions: vec![],
            effective_settings: None,
            parent_run: None,
            segments: 1,
        };
        store.begin_run(&run).unwrap();
        let seq = store
            .append_run_event(
                id,
                0,
                Some(dolores_core::RunState::Running),
                "started",
                &json!({}),
            )
            .unwrap();
        let seq = store
            .append_run_event(
                id,
                seq,
                None,
                "desktopHandoff",
                &json!({"modelCalls":1,"toolCalls":1,"elapsedSeconds":3,"budget":limited()}),
            )
            .unwrap();
        let seq = store
            .append_run_event(
                id,
                seq,
                Some(dolores_core::RunState::Paused),
                "finished",
                &json!({}),
            )
            .unwrap();
        let _ = seq;
        drop(store);
        let store = dolores_store_sqlite::SqliteStore::open(&path).unwrap();
        let (_, remaining) = source(&store, "chat", id, &run.input, limited(), 180).unwrap();
        assert_eq!(
            (
                remaining.model_calls,
                remaining.tool_calls,
                remaining.elapsed_seconds
            ),
            (Some(3), Some(3), Some(177))
        );
        assert!(source(&store, "chat", id, "Different goal", limited(), 180).is_err());
        assert!(source(&store, "other", id, &run.input, limited(), 180).is_err());
        let enlarged = TaskBudget {
            model_calls: Some(16),
            tool_calls: Some(32),
            ..limited()
        };
        assert_eq!(
            source(&store, "chat", id, &run.input, enlarged, 180)
                .unwrap()
                .1
                .model_calls,
            Some(3)
        );
    }
    #[test]
    fn handoff_spends_attempts_and_refuses_exhausted_or_short_deadlines() {
        let b = limited();
        let next = remainder(b, 1, 1, 7, 180).unwrap();
        assert_eq!(
            (next.model_calls, next.tool_calls, next.elapsed_seconds),
            (Some(3), Some(3), Some(173))
        );
        assert!(remainder(b, 3, 1, 0, 180).is_err());
        assert!(remainder(b, 1, 4, 0, 180).is_err());
        assert!(remainder(b, 1, 1, 151, 180).is_err());
        // A model switch with a longer timeout cannot replenish the old deadline.
        assert!(remainder(
            TaskBudget {
                elapsed_seconds: Some(30),
                ..b
            },
            1,
            1,
            1,
            180
        )
        .is_err());
    }
    #[tokio::test]
    async fn request_has_no_window_identity_or_pixels_and_rejects_extra_authority() {
        let call = ToolCall {
            id: "one".into(),
            name: "request_desktop_access".into(),
            arguments: r#"{"purpose":"Inspect the local form"}"#.into(),
        };
        let request = RequestAccess.prepare(&call).unwrap();
        assert_eq!(
            RequestAccess
                .invoke(&request, CancellationToken::new())
                .await
                .unwrap(),
            r#"{"purpose":"Inspect the local form","shared":false,"status":"awaitingWindowChoice"}"#
        );
        assert!(RequestAccess
            .prepare(&ToolCall {
                arguments: r#"{"purpose":"Inspect","target":123}"#.into(),
                ..call
            })
            .is_err());
    }
}
