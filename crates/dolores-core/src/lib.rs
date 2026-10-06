use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc;
use tokio_util::sync::CancellationToken;
mod accounting;
mod attachments;
pub use attachments::*;
mod comparison;
mod experience;
pub use experience::*;
mod adaptation;
pub use adaptation::*;
mod feedback;
pub use comparison::*;
pub use feedback::*;
mod agent;
pub use agent::ModelActivity;
mod web;
pub use web::*;
mod runs;
pub use runs::*;
mod registry;
pub use registry::*;
mod mods;
pub use mods::*;
mod task_budget;
pub use task_budget::*;
mod shared_budget;
pub use shared_budget::*;
mod settings;
pub use settings::*;
mod permissions;
pub use permissions::*;
mod command_outcome;
mod continuation;
pub use command_outcome::{unresolved_commands, CommandOutcome};
pub use continuation::PausedTask;
mod mcp;
pub use mcp::*;
mod change;
mod request_settings;
mod token_context;
mod workspace;
pub use accounting::{ContextSummary, Reply, StreamOutcome, TokenUsage, TurnMetadata};
mod automatic_memory;
mod instructions;
mod relevant_skills;
mod skill_drafts;
mod skills;
pub use relevant_skills::prepare_relevant_skill_context;
pub use skill_drafts::*;
pub use skills::{
    effective_skills, prepare_skill_context, valid_skill_name, ProjectSkill, SkillDocument,
    SkillScope, SkillSource, SkillVersion, MAX_ACTIVE_SKILLS, MAX_SAVED_SKILLS, MAX_SKILL_BYTES,
    MAX_SKILL_VERSIONS,
};
mod knowledge;
mod memory;
pub use knowledge::*;
mod memory_suggestions;
mod summary;
pub use agent::*;
pub use automatic_memory::*;
pub use change::*;
pub use instructions::{
    prepare_instruction_context, InstructionSource, WorkspaceInstructions, MAX_INSTRUCTION_BYTES,
};
pub use memory::*;
pub use memory_suggestions::*;
pub use request_settings::{ReasoningControl, RequestSettings};
pub use summary::*;
pub use token_context::*;
pub use workspace::{Project, SessionWorkspace, WorkspaceKind};

pub const MAX_INPUT_BYTES: usize = 16 * 1024;
pub const MAX_CONTEXT_BYTES: usize = 128 * 1024;
pub const MAX_OUTPUT_BYTES: usize = 128 * 1024;
pub const HISTORY_LIMIT: usize = 80;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PluginDescriptor {
    pub id: &'static str,
    pub kind: &'static str,
    pub api_version: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Role {
    User,
    Assistant,
    System,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Message {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub parts: Vec<AttachmentRef>,
    pub role: Role,
    pub content: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Session {
    pub id: String,
    pub title: String,
    pub updated_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionCursor {
    pub updated_at: i64,
    pub id: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HistoryPage<T> {
    pub items: Vec<T>,
    pub has_older: bool,
    pub has_newer: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct StoredMessage {
    /// Local commit time; legacy messages have no timestamp to infer.
    #[serde(rename = "savedAt", skip_serializing_if = "Option::is_none")]
    pub saved_at: Option<i64>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub parts: Vec<AttachmentRef>,
    pub id: i64,
    pub role: Role,
    pub content: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<TurnMetadata>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub feedback: Option<TaskFeedback>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ExportFormat {
    Markdown,
    Json,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ConnectionPreferences {
    pub base_url: String,
    pub model: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ModelDetails {
    pub context_window_tokens: Option<u32>,
    pub image_input: bool,
    pub request_settings: Option<RequestSettings>,
}

/// Nonsecret metadata. The optional ID references a key in a credential plugin.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RememberedConnection {
    pub preferences: ConnectionPreferences,
    pub credential_id: Option<String>,
}

pub trait CredentialStore: Send + Sync {
    fn descriptor(&self) -> PluginDescriptor;
    fn read(&self, id: &str) -> Result<Option<String>, String>;
    fn write(&self, id: &str, secret: &str) -> Result<(), String>;
    fn delete(&self, id: &str) -> Result<(), String>;
}

impl Default for ConnectionPreferences {
    fn default() -> Self {
        Self {
            base_url: "http://localhost:11434/v1".into(),
            model: String::new(),
        }
    }
}

pub mod desktop_input;
/// Host-owned evidence lookup. Never accepts a model path or raw image bytes.
pub trait AttachmentResolver: Send + Sync {
    fn resolve(&self, reference: &AttachmentRef) -> Result<AttachmentData, String>;
}
#[async_trait]
pub trait ModelProvider: Send + Sync {
    /// Providers returning true enforce decoded-progress inactivity during chat streams.
    /// Legacy adapters retain the host's fallback bound until they implement this contract.
    fn manages_stream_inactivity(&self) -> bool {
        false
    }
    fn with_attachment_resolver(
        &self,
        _: std::sync::Arc<dyn AttachmentResolver>,
    ) -> Result<std::sync::Arc<dyn ModelProvider>, String> {
        Err("This provider cannot resolve fresh desktop evidence.".into())
    }
    fn with_attachment_assets(
        &self,
        _: Vec<AttachmentData>,
        _: bool,
    ) -> Result<std::sync::Arc<dyn ModelProvider>, String> {
        Err("This provider has no image adapter. Choose an image-capable model/adapter or remove the image; your draft is retained.".into())
    }

    /// Chat-only outcome; strict structured consumers keep using stream_with_usage.
    async fn stream_chat_outcome(
        &self,
        messages: Vec<Message>,
        output: mpsc::Sender<String>,
        cancel: CancellationToken,
    ) -> Result<StreamOutcome, String> {
        Ok(StreamOutcome {
            usage: self.stream_with_usage(messages, output, cancel).await?,
            output_limit: false,
        })
    }
    /// Optional activity reporting; existing provider plugins keep their text contract.
    async fn stream_tool_turn_with_thinking(
        &self,
        messages: &[AgentMessage],
        tools: &[ToolSpec],
        output: mpsc::Sender<String>,
        activity: mpsc::Sender<agent::ModelActivity>,
        _thinking: mpsc::Sender<String>,
        cancel: CancellationToken,
    ) -> Result<AgentTurn, String> {
        self.stream_tool_turn_with_activity(messages, tools, output, activity, cancel)
            .await
    }
    async fn stream_tool_turn_with_activity(
        &self,
        messages: &[AgentMessage],
        tools: &[ToolSpec],
        output: mpsc::Sender<String>,
        _activity: mpsc::Sender<agent::ModelActivity>,
        cancel: CancellationToken,
    ) -> Result<AgentTurn, String> {
        self.stream_tool_turn(messages, tools, output, cancel).await
    }
    /// Optional streaming tool capability; existing provider plugins still work.
    async fn stream_tool_turn(
        &self,
        messages: &[AgentMessage],
        tools: &[ToolSpec],
        output: mpsc::Sender<String>,
        cancel: CancellationToken,
    ) -> Result<AgentTurn, String> {
        let turn = self.tool_turn(messages, tools, cancel.clone()).await?;
        if !turn.content.is_empty() {
            tokio::select! { biased;
                _ = cancel.cancelled() => return Err("Response stopped.".into()),
                result = output.send(turn.content.clone()) => result.map_err(|_| "Conversation window closed.")?,
            }
        }
        Ok(turn)
    }
    async fn tool_turn(
        &self,
        _: &[AgentMessage],
        _: &[ToolSpec],
        _: CancellationToken,
    ) -> Result<AgentTurn, String> {
        Err(
            "This model connection does not support tool calls. Start a side chat to chat without tools."
                .into(),
        )
    }
    fn request_settings(&self) -> Option<RequestSettings> {
        None
    }
    fn context_window_tokens(&self) -> Option<u32> {
        None
    }
    fn descriptor(&self) -> PluginDescriptor;
    async fn list_models(&self) -> Result<Vec<String>, String> {
        Err("This provider does not support model discovery. Add a model manually.".into())
    }
    fn with_model(&self, _: &str) -> Result<std::sync::Arc<dyn ModelProvider>, String> {
        Err("This provider does not support switching models.".into())
    }
    async fn stream(
        &self,
        messages: Vec<Message>,
        output: mpsc::Sender<String>,
        cancel: CancellationToken,
    ) -> Result<(), String>;
    /// Backward-compatible optional accounting capability for provider plugins.
    async fn stream_with_usage(
        &self,
        messages: Vec<Message>,
        output: mpsc::Sender<String>,
        cancel: CancellationToken,
    ) -> Result<Option<TokenUsage>, String> {
        self.stream(messages, output, cancel).await?;
        Ok(None)
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Appearance {
    #[default]
    System,
    Light,
    Dark,
}

pub trait SessionStore: Send + Sync {
    fn mod_state(&self, _: &str) -> Result<ModState, String> {
        Ok(Default::default())
    }
    fn save_mod_state(&self, _: &str, _: u32, _: &ModState) -> Result<ModState, String> {
        Err("Executable mod storage is unavailable.".into())
    }
    fn recover_mod_activations(&self) -> Result<(), String> {
        Ok(())
    }
    fn interrupt_adaptations(&self) -> Result<(), String> {
        Ok(())
    }
    fn restore_adaptation(
        &self,
        _: &str,
        _: u32,
        _: &str,
        _: bool,
    ) -> Result<AdaptationState, String> {
        Err("Learning restore is unavailable in this store.".into())
    }
    fn adaptation(&self, _: &str) -> Result<AdaptationState, String> {
        Ok(Default::default())
    }
    fn save_adaptation(
        &self,
        _: &str,
        _: u32,
        _: &AdaptationState,
    ) -> Result<AdaptationState, String> {
        Err("Skill learning unavailable.".into())
    }
    fn activate_adaptation(&self, _: &str, _: u32, _: &str) -> Result<AdaptationState, String> {
        Err("Skill learning activation unavailable.".into())
    }
    fn experience_trials(&self, _: &str) -> Result<Vec<ExperienceTrial>, String> {
        Ok(vec![])
    }
    fn create_experience_trial(&self, _: &ExperienceTrial) -> Result<ExperienceTrial, String> {
        Err("Tool trials unavailable.".into())
    }
    fn save_experience_trial(&self, _: &ExperienceTrial) -> Result<ExperienceTrial, String> {
        Err("Tool trials unavailable.".into())
    }
    fn knowledge(&self, _: &str) -> Result<KnowledgeState, String> {
        Ok(Default::default())
    }
    fn save_knowledge(
        &self,
        _: &str,
        _: u32,
        _: &KnowledgeState,
    ) -> Result<KnowledgeState, String> {
        Err("Project knowledge is unavailable in this store.".into())
    }
    fn appearance(&self) -> Result<Appearance, String> {
        Ok(Appearance::System)
    }
    fn save_appearance(&self, _: Appearance) -> Result<(), String> {
        Err("Appearance settings are unavailable in this store.".into())
    }
    fn web_configuration(&self) -> Result<WebConfiguration, String> {
        Ok(Default::default())
    }
    fn save_web_configuration(
        &self,
        _: u32,
        _: &WebConfiguration,
    ) -> Result<WebConfiguration, String> {
        Err("Web settings are unavailable in this store.".into())
    }
    fn draft_attachments(&self, _: &str) -> Result<Vec<AttachmentRef>, String> {
        Ok(vec![])
    }
    fn add_attachment(&self, _: &str, _: &AttachmentData) -> Result<(), String> {
        Err("Attachments are unavailable.".into())
    }
    fn remove_attachment(&self, _: &str, _: &str) -> Result<(), String> {
        Err("Attachments are unavailable.".into())
    }
    fn attachment_data(&self, _: &str, _: &str) -> Result<AttachmentData, String> {
        Err("Attachment snapshot is unavailable. Reattach it or remove its reference.".into())
    }
    fn cleanup_attachments(&self) -> Result<usize, String> {
        Err("Attachment cleanup is unavailable.".into())
    }
    fn image_models(&self, _: &str) -> Result<Vec<String>, String> {
        Ok(vec![])
    }
    fn save_image_models(&self, _: &str, _: &[String]) -> Result<(), String> {
        Err("Image configuration is unavailable.".into())
    }

    fn fork_session(&self, _: &str, _: i64, _: &str) -> Result<Session, String> {
        Err("Thread forks are unavailable.".into())
    }
    fn auto_compact(&self, _: &str) -> Result<bool, String> {
        Ok(false)
    }
    fn set_auto_compact(&self, _: &str, _: bool) -> Result<(), String> {
        Err("Managed context is unavailable.".into())
    }

    fn saved_draft(&self, _: &str) -> Result<String, String> {
        Ok(String::new())
    }
    fn save_draft(&self, _: &str, _: &str) -> Result<(), String> {
        Err("Persistent drafts are unavailable in this store.".into())
    }
    fn clear_draft_if(&self, _: &str, _: &str) -> Result<(), String> {
        Ok(())
    }
    fn scoped_settings(&self, _scope: SettingsScope, _key: &str) -> Result<ScopedSettings, String> {
        Ok(Default::default())
    }
    fn save_scoped_settings(
        &self,
        _scope: SettingsScope,
        _key: &str,
        _revision: u32,
        _patch: &SettingsPatch,
    ) -> Result<ScopedSettings, String> {
        Err("Scoped settings are unavailable.".into())
    }
    fn begin_run(&self, _: &RunSnapshot) -> Result<(), String> {
        Err("Run evidence is unavailable in this store.".into())
    }
    fn append_run_event(
        &self,
        _: &str,
        _: u32,
        _: Option<RunState>,
        _: &str,
        _: &serde_json::Value,
    ) -> Result<u32, String> {
        Err("Run evidence is unavailable in this store.".into())
    }
    fn runs(&self, _: &str) -> Result<Vec<RunSnapshot>, String> {
        Ok(vec![])
    }
    fn run_events(&self, _: &str, _: &str) -> Result<Vec<RunEvent>, String> {
        Err("Run evidence is unavailable in this store.".into())
    }
    fn interrupt_runs(&self) -> Result<(), String> {
        Ok(())
    }
    fn create_comparison(&self, _: &ContextComparison) -> Result<ContextComparison, String> {
        Err("Context comparisons are unavailable in this store.".into())
    }
    fn update_comparison(&self, _: &ContextComparison) -> Result<ContextComparison, String> {
        Err("Context comparisons are unavailable in this store.".into())
    }
    fn comparisons_page(
        &self,
        _: &str,
        _: Option<i64>,
    ) -> Result<HistoryPage<ComparisonSummary>, String> {
        Err("Context comparisons are unavailable in this store.".into())
    }
    fn comparison(&self, _: &str, _: i64) -> Result<ContextComparison, String> {
        Err("Context comparisons are unavailable in this store.".into())
    }
    fn delete_comparison(&self, _: &str, _: i64, _: u32) -> Result<(), String> {
        Err("Context comparisons are unavailable in this store.".into())
    }
    fn save_task_feedback(&self, _: &str, _: &FeedbackDraft) -> Result<TaskFeedback, String> {
        Err("This storage plugin does not support task feedback.".into())
    }
    fn commit_continuation(
        &self,
        _: &str,
        _: i64,
        _: &str,
        _: &str,
        _: &TurnMetadata,
    ) -> Result<(), String> {
        Err("This storage plugin does not support atomic continuation.".into())
    }
    fn mcp_connection(&self, _root: &str) -> Result<Option<McpConnection>, String> {
        Ok(None)
    }
    fn mcp_connections(&self, root: &str) -> Result<Vec<McpConnection>, String> {
        Ok(self.mcp_connection(root)?.into_iter().collect())
    }
    fn mcp_connection_by_id(&self, root: &str, id: &str) -> Result<Option<McpConnection>, String> {
        Ok(self.mcp_connections(root)?.into_iter().find(|c| c.id == id))
    }
    fn mutate_mcp_connection_by_id(
        &self,
        root: &str,
        id: &str,
        revision: u32,
        forget: bool,
    ) -> Result<(), String> {
        if id != "legacy" {
            return Err("Named MCP connections are unavailable in this store.".into());
        }
        self.mutate_mcp_connection(root, revision, forget)
    }
    fn save_mcp_connection(
        &self,
        _root: &str,
        _connection: &McpConnection,
        _expected: Option<u32>,
    ) -> Result<McpConnection, String> {
        Err("This storage plugin does not support MCP connections.".into())
    }
    fn mutate_mcp_connection(
        &self,
        _root: &str,
        _revision: u32,
        _forget: bool,
    ) -> Result<(), String> {
        Err("This storage plugin does not support MCP connections.".into())
    }
    fn promote_skill(&self, _: &SkillPromotion) -> Result<ProjectSkill, String> {
        Err("This storage plugin does not support evaluated skill promotion.".into())
    }
    fn global_skills(&self) -> Result<Vec<ProjectSkill>, String> {
        Ok(vec![])
    }
    fn activate_global_skill(
        &self,
        _: &SkillDocument,
        _: Option<u32>,
        _: Option<u32>,
    ) -> Result<ProjectSkill, String> {
        Err("This storage plugin does not support global skills.".into())
    }
    fn disable_global_skill(&self, _: &str, _: u32) -> Result<(), String> {
        Err("This storage plugin does not support global skills.".into())
    }
    fn forget_global_skill(&self, _: &str, _: u32) -> Result<(), String> {
        Err("This storage plugin does not support global skills.".into())
    }
    fn project_skills(&self, _: &str) -> Result<Vec<ProjectSkill>, String> {
        Ok(vec![])
    }
    fn activate_project_skill(
        &self,
        _: &str,
        _: &SkillDocument,
        _: Option<u32>,
        _: Option<u32>,
    ) -> Result<ProjectSkill, String> {
        Err("This storage plugin does not support project skills.".into())
    }
    fn disable_project_skill(&self, _: &str, _: &str, _: u32) -> Result<(), String> {
        Err("This storage plugin does not support project skills.".into())
    }
    fn forget_project_skill(&self, _: &str, _: &str, _: u32) -> Result<(), String> {
        Err("This storage plugin does not support project skills.".into())
    }
    fn automatic_memory_policy(&self) -> Result<AutomaticMemoryPolicy, String> {
        Ok(AutomaticMemoryPolicy {
            enabled: false,
            revision: 1,
        })
    }
    fn set_automatic_memory_policy(
        &self,
        _: bool,
        _: u32,
    ) -> Result<AutomaticMemoryPolicy, String> {
        Err("This storage plugin does not support automatic memory.".into())
    }
    fn automatic_memory_attempt(&self, _: &str) -> Result<Option<AutomaticMemoryAttempt>, String> {
        Ok(None)
    }
    fn claim_automatic_memory(&self, _: &str, _: &MemoryMessage, _: u32) -> Result<bool, String> {
        Ok(false)
    }
    fn finish_automatic_memory(
        &self,
        _: &AutomaticMemoryUpdate,
        _: &CancellationToken,
    ) -> Result<AutomaticMemoryAttempt, String> {
        Err("This storage plugin does not support automatic memory.".into())
    }
    fn session_summary(&self, _: &str) -> Result<Option<SessionSummary>, String> {
        Ok(None)
    }
    fn review_summary_batch(&self, _: &str) -> Result<SummaryBatch, String> {
        Err("This storage plugin does not support session summaries.".into())
    }
    fn save_session_summary(
        &self,
        _: &str,
        _: &SummaryBatch,
        _: &str,
        _: &str,
    ) -> Result<SessionSummary, String> {
        Err("This storage plugin does not support session summaries.".into())
    }
    fn correct_session_summary(&self, _: &str, _: u32, _: &str) -> Result<SessionSummary, String> {
        Err("This storage plugin does not support session summaries.".into())
    }
    fn delete_session_summary(&self, _: &str, _: u32) -> Result<(), String> {
        Err("This storage plugin does not support session summaries.".into())
    }
    fn summary_context_history(&self, session: &str) -> Result<SummaryHistory, String> {
        let (messages, count) = self.context_history(session)?;
        Ok((messages, count, None))
    }
    fn memory_source_messages(&self, _: &str) -> Result<HistoryPage<MemoryMessage>, String> {
        Err("This storage plugin does not support memory sources.".into())
    }
    fn memory_source_message(&self, _: &str, _: i64) -> Result<Option<MemoryMessage>, String> {
        Err("This storage plugin does not support memory sources.".into())
    }
    fn memory_preferences(&self, _: Option<&str>) -> Result<Vec<MemoryPreference>, String> {
        Ok(vec![])
    }
    fn save_memory_preference(
        &self,
        _: Option<&str>,
        _: &MemoryDraft,
    ) -> Result<MemoryPreference, String> {
        Err("This storage plugin does not support memory preferences.".into())
    }
    fn save_suggested_memory_preference(
        &self,
        _: Option<&str>,
        _: &MemoryDraft,
        _: &[MemoryMessage],
    ) -> Result<MemoryPreference, String> {
        Err("This storage plugin does not support reviewed memory suggestions.".into())
    }
    fn delete_memory_preference(&self, _: Option<&str>, _: &str, _: u32) -> Result<(), String> {
        Err("This storage plugin does not support memory preferences.".into())
    }
    fn workspace_instructions(&self, _: &str) -> Result<Option<WorkspaceInstructions>, String> {
        Ok(None)
    }
    fn save_workspace_instructions(
        &self,
        _: &str,
        _: Option<&WorkspaceInstructions>,
    ) -> Result<(), String> {
        Err("This storage plugin does not support workspace instructions.".into())
    }
    fn begin_change(&self, _: &ChangeDraft) -> Result<i64, String> {
        Err("This storage plugin does not support the change journal.".into())
    }
    fn finish_change(&self, _: i64, _: bool) -> Result<(), String> {
        Err("This storage plugin does not support the change journal.".into())
    }
    fn changes_page(&self, _: &str, _: Option<i64>) -> Result<HistoryPage<FileChange>, String> {
        Err("This storage plugin does not support the change journal.".into())
    }
    fn change_snapshot(&self, _: i64) -> Result<ChangeSnapshot, String> {
        Err("This storage plugin does not support the change journal.".into())
    }
    fn workspace(&self, _: &str) -> Result<SessionWorkspace, String> {
        Ok(SessionWorkspace::default())
    }
    fn projects(&self) -> Result<Vec<Project>, String> {
        Ok(vec![])
    }
    fn create_workspace_session(&self, _: &str, _: &SessionWorkspace) -> Result<Session, String> {
        Err("This storage plugin does not support working folders.".into())
    }
    fn descriptor(&self) -> PluginDescriptor;
    fn list(&self) -> Result<Vec<Session>, String>;
    fn create(&self, id: &str) -> Result<Session, String>;
    fn messages(&self, id: &str) -> Result<Vec<Message>, String>;
    fn delete(&self, id: &str) -> Result<(), String>;
    fn commit_turn(&self, id: &str, user: &str, assistant: &str) -> Result<(), String>;
    fn context_history(&self, id: &str) -> Result<(Vec<Message>, Option<u64>), String> {
        Ok((self.messages(id)?, None))
    }
    fn commit_turn_metadata(
        &self,
        id: &str,
        user: &str,
        assistant: &str,
        _: &TurnMetadata,
    ) -> Result<(), String> {
        self.commit_turn(id, user, assistant)
    }
    fn preferences(&self) -> Result<ConnectionPreferences, String>;
    fn commit_turn_preserving_draft(
        &self,
        _: &str,
        _: &str,
        _: &str,
        _: &TurnMetadata,
    ) -> Result<(), String> {
        Err("This store cannot preserve a separate draft during window sharing.".into())
    }
    fn save_preferences(&self, preferences: &ConnectionPreferences) -> Result<(), String>;
    fn request_settings(&self) -> Result<RequestSettings, String> {
        Ok(RequestSettings::default())
    }
    fn save_request_settings(&self, _: &RequestSettings) -> Result<(), String> {
        Err("This storage plugin does not support request settings.".into())
    }
    fn model_request_settings(
        &self,
        _: &str,
    ) -> Result<std::collections::BTreeMap<String, RequestSettings>, String> {
        Ok(Default::default())
    }
    fn save_model_request_settings(
        &self,
        _: &ConnectionPreferences,
        _: Option<&RequestSettings>,
    ) -> Result<(), String> {
        Err("This storage plugin does not support model generation profiles.".into())
    }
    fn effective_request_settings(
        &self,
        preferences: &ConnectionPreferences,
    ) -> Result<RequestSettings, String> {
        match self
            .model_request_settings(&preferences.base_url)?
            .get(&preferences.model)
        {
            Some(settings) => Ok(*settings),
            None => self.request_settings(),
        }
    }
    fn sessions_page(
        &self,
        _: Option<SessionCursor>,
        _: bool,
        _: usize,
    ) -> Result<HistoryPage<Session>, String> {
        Err("This storage plugin does not support history browsing.".into())
    }
    fn messages_page(
        &self,
        _: &str,
        _: Option<i64>,
        _: bool,
        _: usize,
    ) -> Result<HistoryPage<StoredMessage>, String> {
        Err("This storage plugin does not support history browsing.".into())
    }
    fn export_conversation(
        &self,
        _: &str,
        _: ExportFormat,
        _: &mut dyn std::io::Write,
    ) -> Result<u64, String> {
        Err("This storage plugin does not support conversation export.".into())
    }
    fn remembered_connection(&self) -> Result<Option<RememberedConnection>, String> {
        Ok(None)
    }
    fn model_choices(&self, _: &str) -> Result<Vec<String>, String> {
        Ok(Vec::new())
    }
    fn model_contexts(&self, _: &str) -> Result<ModelContexts, String> {
        Ok(ModelContexts::new())
    }
    fn model_details(&self, preferences: &ConnectionPreferences) -> Result<ModelDetails, String> {
        Ok(ModelDetails {
            context_window_tokens: self
                .model_contexts(&preferences.base_url)?
                .get(&preferences.model)
                .copied()
                .flatten(),
            image_input: self
                .image_models(&preferences.base_url)?
                .contains(&preferences.model),
            request_settings: self
                .model_request_settings(&preferences.base_url)?
                .get(&preferences.model)
                .copied(),
        })
    }
    fn save_model_details(
        &self,
        _: &ConnectionPreferences,
        _: &ModelDetails,
        _: &ModelDetails,
    ) -> Result<(), String> {
        Err("This storage plugin cannot save model details atomically.".into())
    }
    fn save_connection_model_contexts(
        &self,
        _: &ConnectionPreferences,
        _: Option<&RememberedConnection>,
        _: &[String],
        _: &ModelContexts,
    ) -> Result<(), String> {
        Err("This storage plugin does not support model context windows.".into())
    }
    fn save_connection_models(
        &self,
        _: &ConnectionPreferences,
        _: Option<&RememberedConnection>,
        _: &[String],
    ) -> Result<(), String> {
        Err("This storage plugin does not support model choices.".into())
    }
    fn save_connection(
        &self,
        _: &ConnectionPreferences,
        _: Option<&RememberedConnection>,
    ) -> Result<(), String> {
        Err("This storage plugin does not support remembered connections.".into())
    }
}

pub fn prepare_context(history: Vec<Message>, input: &str) -> Result<Vec<Message>, String> {
    if input.trim().is_empty() {
        return Err("Write a message first.".into());
    }
    preview_context(history, input)
}

/// Same context selection as sending, but an empty draft can be inspected.
pub fn preview_context(history: Vec<Message>, input: &str) -> Result<Vec<Message>, String> {
    if input.len() > MAX_INPUT_BYTES {
        return Err("Message exceeds the 16 KiB limit.".into());
    }
    if !history.len().is_multiple_of(2) {
        return Err("Stored conversation has an incomplete turn.".into());
    }
    let system = Message { parts: vec![], role: Role::System, content: "You are Dolores, a thoughtful, precise assistant. Be candid about uncertainty. Use only the tools and saved context supplied for this request; do not claim access or memories that were not supplied.".into() };
    let mut budget = MAX_CONTEXT_BYTES - input.len() - system.content.len();
    // Keep newest complete turns. Never start context with an orphaned assistant reply.
    let mut pairs = Vec::new();
    for pair in history.chunks_exact(2).rev().take(HISTORY_LIMIT / 2) {
        if pair[0].role != Role::User || pair[1].role != Role::Assistant {
            return Err("Stored conversation has an invalid turn.".into());
        }
        let bytes = pair[0].content.len() + pair[1].content.len();
        if bytes > budget {
            break;
        }
        budget -= bytes;
        pairs.push(pair.to_vec());
    }
    let mut messages = vec![system];
    for pair in pairs.into_iter().rev() {
        messages.extend(pair);
    }
    messages.push(Message {
        parts: vec![],
        role: Role::User,
        content: input.to_owned(),
    });
    Ok(messages)
}

pub async fn stream_reply(
    provider: &dyn ModelProvider,
    messages: Vec<Message>,
    output: mpsc::Sender<String>,
    cancel: CancellationToken,
) -> Result<String, String> {
    Ok(stream_reply_with_usage(provider, messages, output, cancel)
        .await?
        .answer)
}

pub async fn stream_reply_with_usage(
    provider: &dyn ModelProvider,
    messages: Vec<Message>,
    output: mpsc::Sender<String>,
    cancel: CancellationToken,
) -> Result<Reply, String> {
    collect_reply(provider, messages, output, cancel, false).await
}
pub async fn stream_chat_reply(
    provider: &dyn ModelProvider,
    messages: Vec<Message>,
    output: mpsc::Sender<String>,
    cancel: CancellationToken,
) -> Result<Reply, String> {
    collect_reply(provider, messages, output, cancel, true).await
}
async fn collect_reply(
    provider: &dyn ModelProvider,
    messages: Vec<Message>,
    output: mpsc::Sender<String>,
    cancel: CancellationToken,
    recover: bool,
) -> Result<Reply, String> {
    let (sender, mut receiver) = mpsc::channel(32);
    let request = async {
        if recover {
            provider
                .stream_chat_outcome(messages, sender, cancel.clone())
                .await
        } else {
            Ok(StreamOutcome {
                usage: provider
                    .stream_with_usage(messages, sender, cancel.clone())
                    .await?,
                output_limit: false,
            })
        }
    };
    tokio::pin!(request);
    let mut finished = false;
    let mut answer = String::new();
    let mut outcome = StreamOutcome {
        usage: None,
        output_limit: false,
    };
    loop {
        tokio::select! {
            biased;
            _ = cancel.cancelled() => return Err("Response stopped. Your message was not saved.".into()),
            result = &mut request, if !finished => { outcome = result?; finished = true; }
            delta = receiver.recv() => match delta {
                Some(delta) => {
                    if answer.len() + delta.len() > MAX_OUTPUT_BYTES { return Err("Response exceeds the 128 KiB limit.".into()); }
                    answer.push_str(&delta);
                    tokio::select! {
                        _ = cancel.cancelled() => return Err("Response stopped. Your message was not saved.".into()),
                        result = output.send(delta) => result.map_err(|_| "Conversation window closed.".to_string())?,
                    }
                }
                None => { if !finished { outcome = request.await?; } break; }
            }
        }
    }
    if answer.trim().is_empty() {
        if outcome.output_limit {
            answer = "The model reached its output limit before producing visible text.".into();
        } else {
            return Err("The model returned no text.".into());
        }
    }
    Ok(Reply {
        answer,
        usage: outcome.usage,
        output_limit: outcome.output_limit,
    })
}

/// Local file snapshots are larger than model-facing tool results.
pub const MAX_FILE_SNAPSHOT_BYTES: usize = 1024 * 1024;
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn bounds_context_without_orphaning_turns() {
        let history = (0..100)
            .flat_map(|_| {
                [
                    Message {
                        parts: vec![],
                        role: Role::User,
                        content: "u".repeat(4096),
                    },
                    Message {
                        parts: vec![],
                        role: Role::Assistant,
                        content: "a".repeat(4096),
                    },
                ]
            })
            .collect();
        let context = prepare_context(history, "next").unwrap();
        assert!(context.iter().map(|m| m.content.len()).sum::<usize>() <= MAX_CONTEXT_BYTES);
        assert_eq!(context[1].role, Role::User);
        assert_eq!(context.last().unwrap().content, "next");
        assert_eq!(context.len() % 2, 0);
    }
    #[test]
    fn rejects_empty_and_oversized_input() {
        assert!(prepare_context(vec![], "  ").is_err());
        assert!(prepare_context(vec![], &"x".repeat(MAX_INPUT_BYTES + 1)).is_err());
    }

    struct FixtureProvider {
        chunks: Vec<String>,
        hang: bool,
    }
    #[async_trait]
    impl ModelProvider for FixtureProvider {
        fn descriptor(&self) -> PluginDescriptor {
            PluginDescriptor {
                id: "test",
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
            for chunk in &self.chunks {
                output
                    .send(chunk.clone())
                    .await
                    .map_err(|_| "closed".to_string())?;
            }
            if self.hang {
                std::future::pending::<()>().await;
            }
            Ok(())
        }
    }
    #[tokio::test]
    async fn drains_all_deltas_before_success() {
        let provider = FixtureProvider {
            chunks: vec!["one".into(), "two".into()],
            hang: false,
        };
        let (tx, mut rx) = mpsc::channel(32);
        let answer = stream_reply(&provider, vec![], tx, CancellationToken::new())
            .await
            .unwrap();
        assert_eq!(answer, "onetwo");
        assert_eq!(rx.recv().await.as_deref(), Some("one"));
        assert_eq!(rx.recv().await.as_deref(), Some("two"));
        assert!(rx.recv().await.is_none());
    }
    #[tokio::test]
    async fn rejects_output_over_budget_and_empty_replies() {
        for chunks in [vec![], vec!["x".repeat(MAX_OUTPUT_BYTES), "x".into()]] {
            let provider = FixtureProvider {
                chunks,
                hang: false,
            };
            let (tx, _receiver) = mpsc::channel(32);
            assert!(
                stream_reply(&provider, vec![], tx, CancellationToken::new())
                    .await
                    .is_err()
            );
        }
    }
    #[tokio::test]
    async fn cancels_a_hung_provider_and_a_blocked_consumer() {
        for provider in [
            FixtureProvider {
                chunks: vec![],
                hang: true,
            },
            FixtureProvider {
                chunks: vec!["text".into(); 100],
                hang: false,
            },
        ] {
            let cancel = CancellationToken::new();
            let trigger = cancel.clone();
            tokio::spawn(async move {
                tokio::time::sleep(std::time::Duration::from_millis(10)).await;
                trigger.cancel();
            });
            let (tx, _receiver) = mpsc::channel(1);
            let result = tokio::time::timeout(
                std::time::Duration::from_millis(200),
                stream_reply(&provider, vec![], tx, cancel),
            )
            .await
            .unwrap();
            assert!(result.unwrap_err().contains("stopped"));
        }
    }
}
