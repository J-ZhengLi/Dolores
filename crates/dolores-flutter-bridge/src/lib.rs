//! C ABI for the selected Flutter shell. No server or subprocess.
mod approval;
mod automatic_memory;
mod changes;
mod connection;
mod export;
mod instructions;
mod memory;
mod memory_suggestions;
mod recovery;
mod skill_drafts;
mod skills;
mod summaries;
mod workspace;
use approval::{ApprovalSlot, RunApproval};
use connection::ConnectionManager;
use dolores_core::{
    prepare_context, preview_context, stream_reply_with_usage, ConnectionPreferences,
    ContextSummary, CredentialStore, ModelProvider, RequestSettings, SessionStore, TurnMetadata,
};
use dolores_store_sqlite::SqliteStore;
use serde::Deserialize;
use serde_json::{json, Value};
use std::{
    ffi::{c_char, CString},
    path::PathBuf,
    sync::{Arc, Mutex, OnceLock},
};
use tokio::{runtime::Runtime, sync::mpsc};
use tokio_util::sync::CancellationToken;

struct Run {
    id: u64,
    cancel: CancellationToken,
    events: mpsc::Receiver<Value>,
    approvals: ApprovalSlot,
}
struct TurnRequest {
    id: u64,
    session: Option<String>,
    input: String,
    model: String,
    settings: Option<RequestSettings>,
    tools: Vec<Arc<dyn dolores_core::ToolPlugin>>,
    approval: Option<Arc<dyn dolores_core::ToolApproval>>,
}
struct Engine {
    runtime: Runtime,
    store: Arc<dyn SessionStore>,
    connection: Mutex<ConnectionManager>,
    active: Mutex<Option<Run>>,
    workspace_directory: Option<PathBuf>,
    revert: Mutex<Option<changes::PendingRevert>>,
    instruction_review: Mutex<Option<instructions::PendingInstructions>>,
    skill_review: Mutex<Option<skills::SkillReview>>,
    skill_draft_review: Arc<Mutex<Option<skill_drafts::DraftReview>>>,
    global_skills_directory: Option<PathBuf>,
    memory_review: Arc<Mutex<Option<memory_suggestions::MemoryReview>>>,
    summary_review: Arc<Mutex<Option<summaries::SummaryReview>>>,
}
static ENGINE: OnceLock<Result<Engine, String>> = OnceLock::new();

#[derive(Deserialize)]
#[serde(tag = "command", rename_all = "camelCase")]
enum Command {
    ReviewSkillExamples {
        session: String,
        scope: dolores_core::SkillScope,
    },
    GenerateSkillDraft {
        id: u64,
        session: String,
        token: String,
        #[serde(rename = "messageIds")]
        message_ids: Vec<i64>,
        settings: Option<RequestSettings>,
    },
    EvaluateSkillDraft {
        id: u64,
        session: String,
        token: String,
        draft: dolores_core::SkillDraft,
        trials: Vec<dolores_core::SkillTrial>,
    },
    PromoteSkillDraft {
        session: String,
        token: String,
    },
    DiscardSkillDraft {
        token: String,
    },
    ProjectSkills {
        session: String,
        #[serde(default)]
        scope: dolores_core::SkillScope,
    },
    ReviewSkill {
        session: String,
        name: String,
        version: Option<u32>,
        #[serde(default)]
        scope: dolores_core::SkillScope,
    },
    ActivateSkill {
        session: String,
        token: String,
    },
    ExportSkill {
        session: String,
        token: String,
        path: PathBuf,
    },
    DisableSkill {
        session: String,
        name: String,
        revision: u32,
        #[serde(default)]
        scope: dolores_core::SkillScope,
    },
    ForgetSkill {
        session: String,
        name: String,
        revision: u32,
        #[serde(default)]
        scope: dolores_core::SkillScope,
    },
    CancelSkillReview {
        token: String,
    },
    ReviewSummary {
        session: String,
    },
    GenerateSummary {
        id: u64,
        session: String,
        token: String,
    },
    SaveSummary {
        session: String,
        token: String,
        text: String,
    },
    CorrectSummary {
        session: String,
        revision: u32,
        text: String,
    },
    DeleteSummary {
        session: String,
        revision: u32,
    },
    DiscardSummaryReview {
        token: String,
    },
    ReviewMemorySources {
        session: String,
    },
    SuggestMemories {
        id: u64,
        session: String,
        token: String,
        #[serde(rename = "messageIds")]
        message_ids: Vec<i64>,
    },
    SaveMemorySuggestion {
        session: String,
        token: String,
        index: usize,
        scope: dolores_core::MemoryScope,
        #[serde(flatten)]
        input: memory::MemoryInput,
    },
    DiscardMemoryReview {
        token: String,
    },
    Memories {
        session: Option<String>,
    },
    SetAutomaticMemory {
        enabled: bool,
        revision: u32,
    },
    SaveMemory {
        session: Option<String>,
        scope: dolores_core::MemoryScope,
        #[serde(flatten)]
        input: memory::MemoryInput,
    },
    DeleteMemory {
        session: Option<String>,
        scope: dolores_core::MemoryScope,
        id: String,
        revision: u32,
    },
    ReviewInstructions {
        session: String,
    },
    EnableInstructions {
        session: String,
        token: String,
    },
    DisableInstructions {
        session: String,
    },
    CancelInstructionReview {
        token: String,
    },
    ChangesPage {
        session: String,
        cursor: Option<i64>,
    },
    ChangeDetails {
        session: String,
        #[serde(rename = "changeId")]
        change_id: i64,
    },
    PreviewRevert {
        session: String,
        #[serde(rename = "changeId")]
        change_id: i64,
    },
    ApplyRevert {
        session: String,
        token: String,
    },
    CancelRevert {
        token: String,
    },
    Bootstrap,
    CreateSession {
        kind: dolores_core::WorkspaceKind,
        path: Option<PathBuf>,
    },
    Workspace {
        session: String,
    },
    SessionsPage {
        cursor: Option<dolores_core::SessionCursor>,
        #[serde(default)]
        newer: bool,
    },
    MessagesPage {
        session: String,
        cursor: Option<i64>,
        #[serde(default)]
        newer: bool,
    },
    Export {
        session: String,
        path: PathBuf,
        format: dolores_core::ExportFormat,
    },
    Messages {
        session: String,
    },
    Context {
        session: Option<String>,
        input: String,
        #[serde(default)]
        tools: bool,
    },
    SetRequestSettings {
        settings: RequestSettings,
    },
    Delete {
        session: String,
    },
    Configure {
        preferences: ConnectionPreferences,
        #[serde(rename = "apiKey")]
        api_key: Option<String>,
        #[serde(default, rename = "rememberConnection")]
        remember: bool,
        #[serde(rename = "enabledModels")]
        enabled_models: Option<Vec<String>>,
        #[serde(rename = "modelContexts")]
        model_contexts: Option<dolores_core::ModelContexts>,
    },
    ListModels {
        #[serde(rename = "baseUrl")]
        base_url: String,
        #[serde(rename = "apiKey")]
        api_key: Option<String>,
    },
    SelectModel {
        model: String,
    },
    RecoverConnection,
    ForgetConnection,
    Start {
        id: u64,
        session: Option<String>,
        input: String,
        workspace: Option<PathBuf>,
    },
    ApproveTool {
        id: u64,
        #[serde(rename = "callId")]
        call_id: String,
        allow: bool,
    },
    Poll {
        id: u64,
    },
    Cancel {
        id: u64,
    },
    Shutdown,
}
impl Engine {
    fn session_page(
        &self,
        cursor: Option<dolores_core::SessionCursor>,
        newer: bool,
    ) -> Result<Value, String> {
        let page = self.store.sessions_page(cursor, newer, 50)?;
        let mut value = json!(page);
        for item in value["items"].as_array_mut().unwrap() {
            let id = item["id"].as_str().unwrap();
            item["workspace"] = json!(self.store.workspace(id)?);
        }
        Ok(value)
    }
    fn new(
        store: Arc<dyn SessionStore>,
        credentials: Arc<dyn CredentialStore>,
    ) -> Result<Self, String> {
        let runtime = tokio::runtime::Builder::new_multi_thread()
            .worker_threads(2)
            .max_blocking_threads(2)
            .enable_all()
            .build()
            .map_err(|_| "Could not start the chat runtime.")?;
        let mut connection = ConnectionManager::new(store.clone(), credentials);
        {
            let _entered = runtime.enter();
            let _ = connection.recover(); // Recovery warnings keep history available.
        }
        Ok(Self {
            runtime,
            store,
            connection: Mutex::new(connection),
            active: Mutex::new(None),
            workspace_directory: None,
            revert: Mutex::new(None),
            instruction_review: Mutex::new(None),
            skill_review: Mutex::new(None),
            skill_draft_review: Arc::new(Mutex::new(None)),
            global_skills_directory: None,
            memory_review: Arc::new(Mutex::new(None)),
            summary_review: Arc::new(Mutex::new(None)),
        })
    }
    fn call(&self, command: Command) -> Result<Value, String> {
        // Reserve/prepare/cancel are synchronized. Persistence runs on Dart's worker
        // isolate; network and generation run on the bounded Rust runtime.
        let mut active = self.active.lock().map_err(|_| "Chat state unavailable.")?;
        match command {
            Command::Poll { id } => {
                let Some(run) = active.as_mut().filter(|run| run.id == id) else {
                    return Ok(json!([]));
                };
                let mut events = Vec::new();
                let mut done = false;
                for _ in 0..32 {
                    match run.events.try_recv() {
                        Ok(event) => {
                            done |= event["type"] == "done";
                            events.push(event);
                        }
                        Err(_) => break,
                    }
                }
                if done {
                    *active = None;
                }
                return Ok(json!(events));
            }
            Command::ApproveTool { id, call_id, allow } => {
                let run = active
                    .as_ref()
                    .filter(|run| run.id == id)
                    .ok_or("Tool request is no longer waiting.")?;
                let mut slot = run
                    .approvals
                    .lock()
                    .map_err(|_| "Tool approval is unavailable.")?;
                if slot
                    .as_ref()
                    .is_none_or(|pending| pending.call_id != call_id)
                {
                    return Err("Tool request is no longer waiting.".into());
                }
                if run.cancel.is_cancelled() {
                    return Err(stopped());
                }
                slot.take()
                    .unwrap()
                    .reply
                    .send(allow)
                    .map_err(|_| "Tool request is no longer waiting.")?;
                return Ok(Value::Null);
            }
            Command::Cancel { id } => {
                if let Some(run) = active.as_ref().filter(|run| run.id == id) {
                    run.cancel.cancel();
                    self.clear_memory_review()?;
                    self.clear_summary_review()?;
                }
                return Ok(Value::Null);
            }
            Command::Shutdown => {
                self.clear_revert()?;
                if let Some(run) = active.take() {
                    run.cancel.cancel();
                }
                self.clear_memory_review()?;
                self.clear_summary_review()?;
                return Ok(Value::Null);
            }
            _ => {}
        }
        if active.is_some() {
            return Err("Stop the current response first.".into());
        }
        match command {
            Command::ReviewSkillExamples { session, scope } => {
                self.review_skill_examples(&session, scope)
            }
            Command::GenerateSkillDraft {
                id,
                session,
                token,
                message_ids,
                settings,
            } => self.generate_skill_draft(&mut active, id, session, token, message_ids, settings),
            Command::EvaluateSkillDraft {
                id,
                session,
                token,
                draft,
                trials,
            } => self.evaluate_skill_draft(&mut active, id, session, token, draft, trials),
            Command::PromoteSkillDraft { session, token } => {
                self.promote_skill_draft(&session, &token)
            }
            Command::DiscardSkillDraft { token } => self.discard_skill_draft(&token),
            Command::ReviewSummary { session } => self.review_summary(&session),
            Command::ProjectSkills { session, scope } => self.scoped_skills(&session, scope),
            Command::ReviewSkill {
                session,
                name,
                version,
                scope,
            } => self.review_scoped_skill(&session, &name, version, scope),
            Command::ActivateSkill { session, token } => self.activate_skill(&session, &token),
            Command::ExportSkill {
                session,
                token,
                path,
            } => self.export_skill(&session, &token, &path),
            Command::DisableSkill {
                session,
                name,
                revision,
                scope,
            } => self.mutate_scoped_skill(&session, &name, revision, false, scope),
            Command::ForgetSkill {
                session,
                name,
                revision,
                scope,
            } => self.mutate_scoped_skill(&session, &name, revision, true, scope),
            Command::CancelSkillReview { token } => self.cancel_skill_review(&token),
            Command::GenerateSummary { id, session, token } => {
                self.generate_summary(&mut active, id, session, token)
            }
            Command::SaveSummary {
                session,
                token,
                text,
            } => self.save_summary(&session, &token, &text),
            Command::CorrectSummary {
                session,
                revision,
                text,
            } => self.correct_summary(&session, revision, &text),
            Command::DeleteSummary { session, revision } => self.delete_summary(&session, revision),
            Command::DiscardSummaryReview { token } => self.discard_summary_review(&token),
            Command::ReviewMemorySources { session } => self.review_memory_sources(&session),
            Command::SuggestMemories {
                id,
                session,
                token,
                message_ids,
            } => self.suggest_memories(&mut active, id, session, token, message_ids),
            Command::SaveMemorySuggestion {
                session,
                token,
                index,
                scope,
                input,
            } => self.save_memory_suggestion(&session, &token, index, scope, input),
            Command::DiscardMemoryReview { token } => self.discard_memory_review(&token),
            Command::Memories { session } => self.memories(session.as_deref()),
            Command::SetAutomaticMemory { enabled, revision } => Ok(json!(self
                .store
                .set_automatic_memory_policy(enabled, revision)?)),
            Command::SaveMemory {
                session,
                scope,
                input,
            } => self.save_memory(session.as_deref(), scope, input),
            Command::DeleteMemory {
                session,
                scope,
                id,
                revision,
            } => self.delete_memory(session.as_deref(), scope, &id, revision),
            Command::ReviewInstructions { session } => self.review_instructions(&session),
            Command::EnableInstructions { session, token } => {
                self.enable_instructions(&session, &token)
            }
            Command::DisableInstructions { session } => self.disable_instructions(&session),
            Command::CancelInstructionReview { token } => self.cancel_instruction_review(&token),
            Command::ChangesPage { session, cursor } => self.changes_page(&session, cursor),
            Command::ChangeDetails { session, change_id } => {
                self.change_details(&session, change_id)
            }
            Command::PreviewRevert { session, change_id } => {
                self.preview_revert(&session, change_id)
            }
            Command::ApplyRevert { session, token } => self.apply_revert(&session, &token),
            Command::CancelRevert { token } => self.cancel_revert(&token),
            Command::Bootstrap => {
                let connection = self
                    .connection
                    .lock()
                    .map_err(|_| "Connection unavailable.")?;
                let page = self.session_page(None, false)?;
                Ok(
                    json!({"sessions":page["items"],"sessionPage":page,"projects":self.store.projects()?,"preferences":self.store.preferences()?,"requestSettings":self.store.request_settings()?,"enabledModels":connection.model_choices()?,"modelContexts":connection.model_contexts()?,"configured":connection.provider.is_some(),"rememberConnection":connection.remembered,"hasSavedKey":connection.has_key,"connectionWarning":connection.warning,"plugins":[self.store.descriptor(), connection.descriptor()]}),
                )
            }
            Command::CreateSession { kind, path } => self.create_working_session(kind, path),
            Command::Workspace { session } => Ok(json!(self.store.workspace(&session)?)),
            Command::SessionsPage { cursor, newer } => self.session_page(cursor, newer),
            Command::MessagesPage {
                session,
                cursor,
                newer,
            } => Ok(json!(self
                .store
                .messages_page(&session, cursor, newer, 80)?)),
            Command::Export {
                session,
                path,
                format,
            } => Ok(
                json!({"messageCount": export::save(self.store.as_ref(), &session, &path, format)?}),
            ),
            Command::Messages { session } => Ok(json!(self.store.messages(&session)?)),
            Command::Context {
                session,
                input,
                tools,
            } => {
                let guidance =
                    instructions::effective_instructions(self.store.as_ref(), session.as_deref())?;
                let memories =
                    memory::preferences_for_session(self.store.as_ref(), session.as_deref())?;
                let skills = skills::for_session(self.store.as_ref(), session.as_deref())?;
                let tools = match session.as_ref() {
                    Some(id) => self.store.workspace(id)?.root.is_some(),
                    None => tools,
                };
                let (history, count, session_summary) = match session {
                    Some(session) => self.store.summary_context_history(&session)?,
                    None => (vec![], Some(0), None),
                };
                let messages = preview_context(history, &input)?;
                let messages = if tools {
                    dolores_core::prepare_agent_context(messages)?
                } else {
                    messages
                };
                let messages =
                    dolores_core::prepare_instruction_context(messages, guidance.as_ref())?;
                let (messages, skill_sources) =
                    dolores_core::prepare_skill_context(messages, &skills)?;
                let (messages, memory_context) =
                    dolores_core::prepare_memory_context(messages, memories.clone())?;
                let messages =
                    dolores_core::prepare_summary_context(messages, session_summary.as_ref())?;
                let mut specs = if tools {
                    dolores_tools_fs::folder_tool_specs()
                } else {
                    vec![]
                };
                if tools {
                    specs.push(dolores_tools_command::command_spec());
                }
                let preferences = self.store.preferences()?;
                let window = self
                    .store
                    .model_contexts(&preferences.base_url)?
                    .get(&preferences.model)
                    .copied()
                    .flatten();
                let (messages, tokens) = dolores_core::prepare_token_context(
                    messages,
                    &specs,
                    Some(window.unwrap_or(dolores_core::DEFAULT_CONTEXT_WINDOW_TOKENS)),
                    self.store.request_settings()?,
                )?;
                let mut summary = ContextSummary::from_messages(&messages, count);
                summary.tokens = Some(tokens);
                summary.instructions = guidance.map(|g| g.provenance);
                summary.skills = skill_sources;
                summary.memory = memory_context;
                dolores_core::account_summary(&mut summary, session_summary.as_ref());
                let mut report = json!(summary);
                report["messages"] = json!(messages);
                report["tools"] = json!(specs);
                report["model"] = json!(preferences.model);
                let used: Vec<_> = summary
                    .memory
                    .iter()
                    .flat_map(|c| &c.used)
                    .filter_map(|s| {
                        memories
                            .iter()
                            .find(|m| m.id == s.id && m.revision == s.revision)
                    })
                    .collect();
                report["memoryEntries"] = json!(used);
                report["sessionSummary"] = json!(session_summary);
                report["skillEntries"] = json!(dolores_core::effective_skills(&skills)
                    .into_iter()
                    .map(|s| {
                        let mut value = json!(s.current());
                        if s.scope == dolores_core::SkillScope::Global {
                            value["scope"] = json!(s.scope);
                        }
                        value
                    })
                    .collect::<Vec<_>>());
                Ok(report)
            }
            Command::Delete { session } => {
                self.clear_revert()?;
                self.clear_memory_review()?;
                self.clear_summary_review()?;
                self.store.delete(&session)?;
                Ok(Value::Null)
            }
            Command::SetRequestSettings { settings } => {
                let _runtime = self.runtime.enter();
                self.connection
                    .lock()
                    .map_err(|_| "Connection unavailable.")?
                    .update_request_settings(settings)?;
                Ok(Value::Null)
            }
            Command::Configure {
                preferences,
                api_key,
                remember,
                enabled_models,
                model_contexts,
            } => {
                let _runtime = self.runtime.enter();
                let mut connection = self
                    .connection
                    .lock()
                    .map_err(|_| "Connection unavailable.")?;
                if model_contexts.is_some() {
                    connection.configure_model_contexts(
                        preferences,
                        api_key,
                        remember,
                        enabled_models,
                        model_contexts,
                    )?;
                } else if enabled_models.is_some() {
                    connection.configure_models(preferences, api_key, remember, enabled_models)?;
                } else {
                    connection.configure(preferences, api_key, remember)?;
                }
                Ok(Value::Null)
            }
            Command::ListModels { base_url, api_key } => {
                let connection = self
                    .connection
                    .lock()
                    .map_err(|_| "Connection unavailable.")?;
                Ok(json!(self
                    .runtime
                    .block_on(connection.list_models(base_url, api_key))?))
            }
            Command::SelectModel { model } => {
                let _entered = self.runtime.enter();
                self.connection
                    .lock()
                    .map_err(|_| "Connection unavailable.")?
                    .select_model(model)?;
                Ok(Value::Null)
            }
            Command::RecoverConnection => {
                let _runtime = self.runtime.enter();
                self.connection
                    .lock()
                    .map_err(|_| "Connection unavailable.")?
                    .recover()?;
                Ok(Value::Null)
            }
            Command::ForgetConnection => {
                self.connection
                    .lock()
                    .map_err(|_| "Connection unavailable.")?
                    .forget()?;
                Ok(Value::Null)
            }
            Command::Start {
                id,
                session,
                input,
                workspace,
            } => {
                self.clear_revert()?;
                self.clear_memory_review()?;
                self.clear_summary_review()?;
                prepare_context(vec![], &input)?;
                let provider = self
                    .connection
                    .lock()
                    .map_err(|_| "Connection unavailable.")?
                    .provider
                    .clone()
                    .ok_or("Set up a model connection first.")?;
                let session = match session {
                    Some(id) => Some(id),
                    None => {
                        let kind = if workspace.is_some() {
                            dolores_core::WorkspaceKind::Project
                        } else {
                            dolores_core::WorkspaceKind::Temporary
                        };
                        let created = self.create_working_session(kind, workspace.clone())?;
                        Some(created["session"]["id"].as_str().unwrap().to_owned())
                    }
                };
                // Legacy callers can choose a folder when starting a new chat.
                // A saved workspace is authoritative and cannot be redirected per request.
                let saved = session
                    .as_ref()
                    .map(|id| self.store.workspace(id))
                    .transpose()?;
                if let (Some(saved), Some(root)) = (&saved, &workspace) {
                    if saved.root.as_ref() != Some(&workspace::canonical_folder(root)?) {
                        return Err(
                            "This chat uses a different working folder. Start a new project chat."
                                .into(),
                        );
                    }
                }
                let workspace = saved.and_then(|s| s.root).map(PathBuf::from).or(workspace);
                let tools = workspace
                    .map(|root| {
                        let journal = Arc::new(dolores_core::WorkspaceJournal {
                            store: self.store.clone(),
                            root: root.to_string_lossy().into_owned(),
                            session: session.clone().unwrap(),
                            reverts: None,
                        });
                        let mut plugins = dolores_tools_fs::journaled_folder_tools(&root, journal)?;
                        plugins.push(Arc::new(dolores_tools_command::RunCommand::new(&root)?));
                        Ok::<_, String>(plugins)
                    })
                    .transpose()?
                    .unwrap_or_default();
                let model = self.store.preferences()?.model;
                let settings = provider.request_settings();
                let learner = if self.store.automatic_memory_policy()?.enabled {
                    self.connection
                        .lock()
                        .map_err(|_| "Connection unavailable.")?
                        .automatic_memory_provider()
                        .ok()
                } else {
                    None
                };
                let cancel = CancellationToken::new();
                let (output, events) = mpsc::channel(32);
                let approvals = Arc::new(Mutex::new(None));
                let approval = Arc::new(RunApproval {
                    id,
                    pending: approvals.clone(),
                    output: output.clone(),
                });
                *active = Some(Run {
                    id,
                    cancel: cancel.clone(),
                    events,
                    approvals,
                });
                let store = self.store.clone();
                self.runtime.spawn(async move {
                    let learning_session = session.clone();
                    let learning_model = model.clone();
                    let result = execute(
                        store.clone(),
                        provider,
                        TurnRequest {
                            id,
                            session,
                            input,
                            model,
                            settings,
                            tools,
                            approval: Some(approval),
                        },
                        cancel.clone(),
                        &output,
                    )
                    .await;
                    let memory_update = if result.is_ok() {
                        if let (Some(session), Some(learner)) = (learning_session, learner) {
                            automatic_memory::learn(store, learner, &session, &learning_model, cancel.clone(), &output, id).await
                        } else { None }
                    } else { None };
                    let event = match result {
                        Ok(answer) => json!({"type":"done", "id":id, "answer":answer, "memoryUpdate":memory_update}),
                        Err(error) => json!({"type":"done", "id":id, "recovery":recovery::advice(&error), "error":error}),
                    };
                    let _ = output.send(event).await;
                });
                Ok(Value::Null)
            }
            _ => unreachable!(),
        }
    }
}
async fn blocking<T: Send + 'static>(
    f: impl FnOnce() -> Result<T, String> + Send + 'static,
) -> Result<T, String> {
    tokio::task::spawn_blocking(f)
        .await
        .map_err(|_| "Local storage task failed.".to_string())?
}
async fn execute(
    store: Arc<dyn SessionStore>,
    provider: Arc<dyn ModelProvider>,
    request: TurnRequest,
    cancel: CancellationToken,
    output: &mpsc::Sender<Value>,
) -> Result<String, String> {
    let TurnRequest {
        id,
        session,
        input,
        model,
        settings,
        tools,
        approval,
    } = request;
    if cancel.is_cancelled() {
        return Err(stopped());
    }
    let reader = store.clone();
    let (session, history, count, guidance, memories, session_summary, skills) =
        blocking(move || {
            let session = match session {
                Some(id) => id,
                None => reader.create(&uuid::Uuid::new_v4().to_string())?.id,
            };
            let (history, count, session_summary) = reader.summary_context_history(&session)?;
            let guidance = instructions::effective_instructions(reader.as_ref(), Some(&session))?;
            let memories = memory::preferences_for_session(reader.as_ref(), Some(&session))?;
            let skills = skills::for_session(reader.as_ref(), Some(&session))?;
            Ok((
                session,
                history,
                count,
                guidance,
                memories,
                session_summary,
                skills,
            ))
        })
        .await?;
    let context = prepare_context(history, &input)?;
    let context = if !tools.is_empty() {
        dolores_core::prepare_agent_context(context)?
    } else {
        context
    };
    let context = dolores_core::prepare_instruction_context(context, guidance.as_ref())?;
    let (context, skill_sources) = dolores_core::prepare_skill_context(context, &skills)?;
    let (context, memory_context) = dolores_core::prepare_memory_context(context, memories)?;
    let context = dolores_core::prepare_summary_context(context, session_summary.as_ref())?;
    let specs: Vec<_> = tools.iter().map(|tool| tool.spec()).collect();
    let (context, tokens) = dolores_core::prepare_token_context(
        context,
        &specs,
        provider.context_window_tokens(),
        settings.unwrap_or_default(),
    )?;
    let mut summary = ContextSummary::from_messages(&context, count);
    summary.tokens = Some(tokens);
    summary.instructions = guidance.map(|g| g.provenance);
    summary.skills = skill_sources;
    summary.memory = memory_context;
    dolores_core::account_summary(&mut summary, session_summary.as_ref());
    forward(
        output,
        json!({"type":"started", "id":id, "session":session, "context":summary, "requestSettings":settings}),
        &cancel,
    )
    .await?;
    if !tools.is_empty() {
        let approval = approval.ok_or("Tool approval is unavailable.")?;
        let running = async {
            let (events, mut receiver) = mpsc::channel(32);
            let plugins = tools;
            let request = dolores_core::run_agent(
                provider.as_ref(),
                context,
                &plugins,
                approval.as_ref(),
                events,
                cancel.clone(),
            );
            tokio::pin!(request);
            let reply = loop {
                tokio::select! { biased;
                    _ = cancel.cancelled() => return Err(stopped()),
                    result = &mut request => break result?,
                    Some(event) = receiver.recv() => {
                        let mut event = serde_json::to_value(event).map_err(|_| "Tool event is unavailable.")?;
                        event["id"] = json!(id);
                        forward(output, event, &cancel).await?;
                    }
                }
            };
            while let Some(event) = receiver.recv().await {
                let mut event =
                    serde_json::to_value(event).map_err(|_| "Tool event is unavailable.")?;
                event["id"] = json!(id);
                forward(output, event, &cancel).await?;
            }
            Ok::<_, String>(reply)
        };
        let seconds = settings.unwrap_or_default().timeout_seconds.min(300);
        let reply = tokio::select! { biased;
            _ = cancel.cancelled() => return Err(stopped()),
            result = tokio::time::timeout(std::time::Duration::from_secs(seconds.into()), running) => match result {
                Ok(reply) => reply?,
                Err(_) => {
                    cancel.cancel();
                    return Err("Agent run timed out while working or waiting for approval. Your message was not saved.".into());
                }
            },
        };
        if cancel.is_cancelled() {
            return Err(stopped());
        }
        let saved = reply.answer.clone();
        let metadata = TurnMetadata {
            model,
            usage: None,
            context: summary,
            request_settings: settings,
            agent: Some(reply.summary),
        };
        blocking(move || store.commit_turn_metadata(&session, &input, &saved, &metadata)).await?;
        return Ok(reply.answer);
    }
    // The adapter deadline alone cannot run while this host awaits a full UI
    // queue. Bound streaming AND delivery, excluding history reads and commit.
    let streaming = async {
        let (sender, mut receiver) = mpsc::channel(32);
        let request = stream_reply_with_usage(provider.as_ref(), context, sender, cancel.clone());
        tokio::pin!(request);
        let answer = loop {
            tokio::select! {
                biased;
                _ = cancel.cancelled() => return Err(stopped()),
                result = &mut request => break result?,
                Some(text) = receiver.recv() => forward(output, json!({"type":"delta", "id":id, "text":text}), &cancel).await?,
            }
        };
        while let Some(text) = receiver.recv().await {
            forward(
                output,
                json!({"type":"delta", "id":id, "text":text}),
                &cancel,
            )
            .await?;
        }
        Ok::<_, String>(answer)
    };
    let answer = match settings {
        Some(settings) => tokio::select! {
            biased;
            _ = cancel.cancelled() => return Err(stopped()),
            result = tokio::time::timeout(std::time::Duration::from_secs(settings.timeout_seconds.into()), streaming) =>
                result.map_err(|_| "Model request timed out. Adjust the request timeout or try again.")??,
        },
        None => streaming.await?,
    };
    if cancel.is_cancelled() {
        return Err(stopped());
    }
    let saved = answer.answer.clone();
    let metadata = TurnMetadata {
        model,
        usage: answer.usage,
        context: summary,
        request_settings: settings,
        agent: None,
    };
    // Once the complete-pair transaction starts, completion wins over late Stop.
    blocking(move || store.commit_turn_metadata(&session, &input, &saved, &metadata)).await?;
    Ok(answer.answer)
}
fn stopped() -> String {
    "Response stopped. Your message was not saved.".into()
}
async fn forward(
    output: &mpsc::Sender<Value>,
    event: Value,
    cancel: &CancellationToken,
) -> Result<(), String> {
    tokio::select! {
        biased;
        _ = cancel.cancelled() => Err(stopped()),
        result = output.send(event) => result.map_err(|_| "Conversation window closed.".into()),
    }
}
fn initialize() -> Result<Engine, String> {
    let directory = match std::env::var_os("DOLORES_DATA_DIR") {
        Some(value) => {
            let path = PathBuf::from(value);
            if !path.is_absolute() {
                return Err("DOLORES_DATA_DIR must be absolute.".into());
            }
            path
        }
        None => directories::BaseDirs::new()
            .ok_or("No application data directory.")?
            .data_dir()
            .join("dev.dolores.desktop"),
    };
    std::fs::create_dir_all(&directory).map_err(|_| "Could not create the data directory.")?;
    let credentials = Arc::new(dolores_credentials::OsCredentialStore::new(&directory)?);
    let mut engine = Engine::new(
        Arc::new(SqliteStore::open(&directory.join("dolores.db"))?),
        credentials,
    )?;
    engine.workspace_directory = Some(directory.join("workspaces"));
    engine.global_skills_directory = Some(match std::env::var_os("DOLORES_GLOBAL_SKILLS_DIR") {
        Some(value) => {
            let path = PathBuf::from(value);
            if !path.is_absolute() {
                return Err("DOLORES_GLOBAL_SKILLS_DIR must be absolute.".into());
            }
            path
        }
        None => directories::BaseDirs::new()
            .ok_or("No user home directory.")?
            .home_dir()
            .join(".agents")
            .join("skills"),
    });
    Ok(engine)
}
fn reply(input: &[u8]) -> Value {
    let result = serde_json::from_slice::<Command>(input)
        .map_err(|_| "Invalid bridge request.".to_string())
        .and_then(|command| match ENGINE.get_or_init(initialize) {
            Ok(engine) => engine.call(command),
            Err(error) => Err(error.clone()),
        });
    match result {
        Ok(result) => json!({"ok":true,"result":result}),
        Err(error) => json!({"ok":false,"error":error}),
    }
}

/// Returns an owned, NUL-terminated UTF-8 JSON envelope; free exactly once.
/// # Safety
/// `input` must point to `length` readable bytes for the duration of this call.
#[no_mangle]
pub unsafe extern "C" fn dolores_call(input: *const u8, length: usize) -> *mut c_char {
    let value = std::panic::catch_unwind(|| {
        if input.is_null() || length > 128 * 1024 {
            return json!({"ok":false,"error":"Invalid bridge request size."});
        }
        // SAFETY: caller provides a live input allocation, validated size above.
        reply(unsafe { std::slice::from_raw_parts(input, length) })
    })
    .unwrap_or_else(|_| json!({"ok":false,"error":"Native bridge failed."}));
    CString::new(value.to_string())
        .expect("JSON has no literal NUL")
        .into_raw()
}
/// # Safety
/// `value` must be a live pointer returned by `dolores_call`, freed only once.
#[no_mangle]
pub unsafe extern "C" fn dolores_free(value: *mut c_char) {
    if !value.is_null() {
        drop(unsafe { CString::from_raw(value) });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;
    use dolores_core::{Message, PluginDescriptor};
    struct Fixture {
        hang: bool,
        fail: bool,
    }
    struct CapturingBudgetFixture(Mutex<Vec<Message>>);
    #[async_trait]
    impl ModelProvider for CapturingBudgetFixture {
        fn descriptor(&self) -> PluginDescriptor {
            PluginDescriptor {
                id: "budget",
                kind: "provider",
                api_version: 1,
            }
        }
        fn context_window_tokens(&self) -> Option<u32> {
            Some(1024)
        }
        fn request_settings(&self) -> Option<RequestSettings> {
            Some(RequestSettings {
                max_output_tokens: 128,
                ..Default::default()
            })
        }
        async fn stream(
            &self,
            messages: Vec<Message>,
            output: mpsc::Sender<String>,
            _: CancellationToken,
        ) -> Result<(), String> {
            *self.0.lock().unwrap() = messages;
            output
                .send("Complete reply".into())
                .await
                .map_err(|_| "closed".into())
        }
    }
    #[test]
    fn token_preview_matches_sent_history_and_saved_snapshot_without_mutating_old_turns() {
        let store = Arc::new(SqliteStore::open(std::path::Path::new(":memory:")).unwrap());
        let engine = Engine::new(
            store.clone(),
            Arc::new(connection::testing::MemoryCredentials::default()),
        )
        .unwrap();
        let preferences = ConnectionPreferences {
            base_url: "http://localhost/v1".into(),
            model: "budget".into(),
        };
        store
            .save_connection_model_contexts(
                &preferences,
                None,
                &["budget".into()],
                &dolores_core::ModelContexts::from([("budget".into(), Some(1024))]),
            )
            .unwrap();
        store
            .save_request_settings(&RequestSettings {
                max_output_tokens: 128,
                ..Default::default()
            })
            .unwrap();
        store.create("side").unwrap();
        for n in 0..10 {
            store
                .commit_turn("side", &format!("{n}{}", "x".repeat(400)), &"a".repeat(400))
                .unwrap();
        }
        let preview = engine
            .call(Command::Context {
                session: Some("side".into()),
                input: "draft".into(),
                tools: false,
            })
            .unwrap();
        assert!(preview["includedTurns"].as_u64().unwrap() < 10);
        let provider = Arc::new(CapturingBudgetFixture(Mutex::new(vec![])));
        let (output, _receiver) = mpsc::channel(32);
        engine
            .runtime
            .block_on(execute(
                store.clone(),
                provider.clone(),
                TurnRequest {
                    id: 1,
                    session: Some("side".into()),
                    input: "draft".into(),
                    model: "budget".into(),
                    settings: provider.request_settings(),
                    tools: vec![],
                    approval: None,
                },
                CancellationToken::new(),
                &output,
            ))
            .unwrap();
        assert_eq!(preview["messages"], json!(*provider.0.lock().unwrap()));
        let history = store.messages_page("side", None, false, 80).unwrap();
        let metadata = history.items.last().unwrap().metadata.as_ref().unwrap();
        assert_eq!(preview["tokens"], json!(metadata.context.tokens));
        assert_eq!(store.context_history("side").unwrap().1, Some(11));
        assert!(history.items[1].metadata.is_none());
        let tool_preview = engine.call(Command::Context {
            session: None,
            input: "".into(),
            tools: true,
        });
        assert!(
            tool_preview.is_err(),
            "Fixed tool definitions must not fit this intentionally small allowance"
        );
    }
    #[async_trait]
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
            output: mpsc::Sender<String>,
            _: CancellationToken,
        ) -> Result<(), String> {
            for _ in 0..96 {
                output.send("你好!".into()).await.map_err(|_| "closed")?;
            }
            if self.fail {
                return Err("Fixture failed".into());
            }
            if self.hang {
                std::future::pending::<()>().await;
            }
            Ok(())
        }
        async fn stream_with_usage(
            &self,
            messages: Vec<Message>,
            output: mpsc::Sender<String>,
            cancel: CancellationToken,
        ) -> Result<Option<dolores_core::TokenUsage>, String> {
            self.stream(messages, output, cancel).await?;
            Ok(Some(dolores_core::TokenUsage {
                input_tokens: Some(0),
                output_tokens: Some(96),
                ..Default::default()
            }))
        }
    }
    #[test]
    fn context_preview_uses_latest_saved_history_and_does_not_create_or_mutate_sessions() {
        let store = Arc::new(SqliteStore::open(std::path::Path::new(":memory:")).unwrap());
        let engine = Engine::new(
            store.clone(),
            Arc::new(connection::testing::MemoryCredentials::default()),
        )
        .unwrap();
        let empty = engine
            .call(Command::Context {
                session: None,
                input: "".into(),
                tools: false,
            })
            .unwrap();
        assert_eq!(empty["savedTurns"], 0);
        assert!(store.list().unwrap().is_empty());
        store.create("long").unwrap();
        for n in 0..61 {
            store
                .commit_turn("long", &format!("u{n}"), "reply")
                .unwrap();
        }
        let context = engine
            .call(Command::Context {
                session: Some("long".into()),
                input: "你好".into(),
                tools: false,
            })
            .unwrap();
        assert_eq!(context["includedTurns"], 40);
        assert_eq!(context["savedTurns"], 61);
        assert_eq!(context["omittedTurns"], 21);
        let prepared = context["messages"].as_array().unwrap();
        assert_eq!(prepared.len(), 82);
        assert_eq!(prepared[1]["content"], "u21");
        assert_eq!(prepared.last().unwrap()["content"], "你好");
        assert_eq!(prepared[0]["role"], "system");
        assert_eq!(
            prepared
                .iter()
                .map(|m| m["content"].as_str().unwrap().len())
                .sum::<usize>(),
            context["textBytes"].as_u64().unwrap() as usize
        );
        assert!(engine
            .call(Command::Context {
                session: Some("missing".into()),
                input: "".into(),
                tools: false,
            })
            .is_err());
        assert!(engine
            .call(Command::Context {
                session: None,
                input: "x".repeat(dolores_core::MAX_INPUT_BYTES + 1),
                tools: false,
            })
            .is_err());
    }
    #[test]
    fn request_settings_command_is_validated_and_visible_before_connecting() {
        let store = Arc::new(SqliteStore::open(std::path::Path::new(":memory:")).unwrap());
        let engine = Engine::new(
            store,
            Arc::new(connection::testing::MemoryCredentials::default()),
        )
        .unwrap();
        assert_eq!(
            engine.call(Command::Bootstrap).unwrap()["requestSettings"]["maxOutputTokens"],
            2048
        );
        let settings = RequestSettings {
            max_output_tokens: 4096,
            timeout_seconds: 300,
        };
        engine
            .call(Command::SetRequestSettings { settings })
            .unwrap();
        let state = engine.call(Command::Bootstrap).unwrap();
        assert_eq!(state["requestSettings"]["timeoutSeconds"], 300);
        assert_eq!(state["configured"], false);
        assert!(engine
            .call(Command::SetRequestSettings {
                settings: RequestSettings {
                    max_output_tokens: 0,
                    ..settings
                }
            })
            .is_err());
        assert_eq!(
            engine.call(Command::Bootstrap).unwrap()["requestSettings"]["maxOutputTokens"],
            4096
        );
    }
    #[test]
    fn approval_commands_reject_stale_wrong_call_and_repeated_decisions() {
        let engine = Engine::new(
            Arc::new(SqliteStore::open(std::path::Path::new(":memory:")).unwrap()),
            Arc::new(connection::testing::MemoryCredentials::default()),
        )
        .unwrap();
        let (reply, mut decision) = tokio::sync::oneshot::channel();
        let pending = Arc::new(Mutex::new(Some(approval::PendingApproval {
            call_id: "one".into(),
            reply,
        })));
        let (_output, events) = mpsc::channel(4);
        *engine.active.lock().unwrap() = Some(Run {
            id: 7,
            cancel: CancellationToken::new(),
            events,
            approvals: pending.clone(),
        });
        for (id, call_id) in [(6, "one"), (7, "wrong")] {
            assert!(engine
                .call(Command::ApproveTool {
                    id,
                    call_id: call_id.into(),
                    allow: true
                })
                .is_err());
            assert!(pending.lock().unwrap().is_some());
        }
        engine
            .call(Command::ApproveTool {
                id: 7,
                call_id: "one".into(),
                allow: false,
            })
            .unwrap();
        assert!(!decision.try_recv().unwrap());
        assert!(engine
            .call(Command::ApproveTool {
                id: 7,
                call_id: "one".into(),
                allow: true
            })
            .is_err());
    }
    #[test]
    fn bridge_null_and_invalid_json_return_owned_error_envelopes() {
        for (pointer, length) in [(std::ptr::null(), 0), (b"bad".as_ptr(), 3)] {
            let result = unsafe { dolores_call(pointer, length) };
            let text = unsafe { std::ffi::CStr::from_ptr(result) }
                .to_str()
                .unwrap();
            assert_eq!(serde_json::from_str::<Value>(text).unwrap()["ok"], false);
            unsafe {
                dolores_free(result);
            }
        }
    }
    #[tokio::test]
    async fn stream_deadline_still_expires_when_flutter_stops_draining_events() {
        let store = Arc::new(SqliteStore::open(std::path::Path::new(":memory:")).unwrap());
        store.create("paused-window").unwrap();
        let saved = store.clone();
        let (output, mut events) = mpsc::channel(1);
        let task = tokio::spawn(async move {
            execute(
                store,
                Arc::new(Fixture {
                    hang: false,
                    fail: false,
                }),
                TurnRequest {
                    id: 1,
                    session: Some("paused-window".into()),
                    input: "unsent".into(),
                    model: "fixture".into(),
                    settings: Some(RequestSettings {
                        max_output_tokens: 2048,
                        timeout_seconds: 1,
                    }),
                    tools: vec![],
                    approval: None,
                },
                CancellationToken::new(),
                &output,
            )
            .await
        });
        assert_eq!(events.recv().await.unwrap()["type"], "started");
        tokio::time::sleep(std::time::Duration::from_millis(1300)).await;
        let finished = task.is_finished();
        if !finished {
            task.abort();
        }
        assert!(
            finished,
            "A full Flutter event queue must not suspend the request deadline"
        );
        assert_eq!(
            task.await.unwrap().unwrap_err(),
            "Model request timed out. Adjust the request timeout or try again."
        );
        assert!(saved.messages("paused-window").unwrap().is_empty());
    }
    #[test]
    fn backpressure_cancel_failure_and_success_preserve_atomic_turns() {
        for (hang, fail) in [(false, false), (true, false), (false, true)] {
            let directory = tempfile::tempdir().unwrap();
            let store = Arc::new(SqliteStore::open(&directory.path().join("fixture.db")).unwrap());
            let engine = Engine::new(
                store.clone(),
                Arc::new(connection::testing::MemoryCredentials::default()),
            )
            .unwrap();
            engine.connection.lock().unwrap().provider = Some(Arc::new(Fixture { hang, fail }));
            store
                .save_preferences(&ConnectionPreferences {
                    base_url: "http://localhost/v1".into(),
                    model: "fixture".into(),
                })
                .unwrap();
            store
                .create_workspace_session("side", &dolores_core::SessionWorkspace::default())
                .unwrap();
            engine
                .call(Command::Start {
                    id: 7,
                    session: Some("side".into()),
                    input: "user".into(),
                    workspace: None,
                })
                .unwrap();
            assert!(engine
                .call(Command::Delete {
                    session: "anything".into()
                })
                .is_err());
            assert!(engine
                .call(Command::Context {
                    session: None,
                    input: "".into(),
                    tools: false,
                })
                .is_err());
            assert_eq!(engine.call(Command::Poll { id: 6 }).unwrap(), json!([]));
            assert!(engine
                .call(Command::SetRequestSettings {
                    settings: RequestSettings::default()
                })
                .is_err());
            assert!(engine
                .call(Command::SelectModel {
                    model: "other".into()
                })
                .is_err());
            assert!(engine
                .call(Command::ListModels {
                    base_url: "http://localhost:19421/v1".into(),
                    api_key: None
                })
                .is_err());
            assert!(engine
                .call(Command::MessagesPage {
                    session: "anything".into(),
                    cursor: None,
                    newer: false
                })
                .is_err());
            assert!(engine
                .call(Command::Export {
                    session: "anything".into(),
                    path: directory.path().join("busy.json"),
                    format: dolores_core::ExportFormat::Json
                })
                .is_err());
            // Give the producer time to fill its bounded queue, then cancel it.
            std::thread::sleep(std::time::Duration::from_millis(60));
            if hang {
                engine.call(Command::Cancel { id: 7 }).unwrap();
            }
            let mut answer = String::new();
            let deadline = std::time::Instant::now() + std::time::Duration::from_secs(3);
            let done = loop {
                assert!(
                    std::time::Instant::now() < deadline,
                    "missing terminal event"
                );
                let events = engine.call(Command::Poll { id: 7 }).unwrap();
                let mut done = None;
                for event in events.as_array().unwrap() {
                    if event["type"] == "delta" {
                        answer.push_str(event["text"].as_str().unwrap());
                    }
                    if event["type"] == "done" {
                        done = Some(event.clone());
                    }
                }
                if let Some(done) = done {
                    break done;
                }
                std::thread::sleep(std::time::Duration::from_millis(5));
            };
            let sessions = store.list().unwrap();
            let messages = store.messages(&sessions[0].id).unwrap();
            if hang || fail {
                assert!(done["error"].is_string());
                assert!(messages.is_empty());
            } else {
                assert_eq!(answer, "你好!".repeat(96));
                assert_eq!(done["answer"], answer);
                assert_eq!(messages.len(), 2);
                assert_eq!(messages[1].content, answer);
                let page = store
                    .messages_page(&sessions[0].id, None, false, 80)
                    .unwrap();
                let metadata = page.items[1].metadata.as_ref().unwrap();
                assert_eq!(metadata.model, "fixture");
                assert_eq!(metadata.usage.as_ref().unwrap().input_tokens, Some(0));
                assert_eq!(metadata.context.included_turns, 0);
            }
            assert!(engine.call(Command::Bootstrap).is_ok());
        }
    }
}
