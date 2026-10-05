use dolores_core::{
    ConnectionPreferences, Message, ModelContexts, PluginDescriptor, RememberedConnection,
    RequestSettings, Role, Session, SessionStore, TurnMetadata, HISTORY_LIMIT,
};
use rusqlite::{params, Connection, OptionalExtension};
mod automatic_memory;
#[cfg(test)]
mod change_tests;
mod changes;
mod comparison;
mod feedback;
mod generation_profiles;
mod history;
mod instructions;
mod mcp;
mod memory;
mod skills;
mod summaries;
mod workspace;
use std::{
    path::Path,
    sync::{Mutex, MutexGuard},
    time::{SystemTime, UNIX_EPOCH},
};

pub const SCHEMA_VERSION: i64 = 28;
pub struct SqliteStore {
    connection: Mutex<Connection>,
}
mod attachments;
mod drafts;
mod knowledge;
#[cfg(test)]
mod message_timestamp_tests;
mod runs;
mod settings;
mod threads;
mod web;
fn storage_error(_: impl std::fmt::Display) -> String {
    "Could not read or save local conversation data.".into()
}
fn now() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as i64
}

impl SqliteStore {
    pub fn open(path: &Path) -> Result<Self, String> {
        let connection = Connection::open(path).map_err(storage_error)?;
        connection
            .busy_timeout(std::time::Duration::from_secs(5))
            .map_err(storage_error)?;
        connection.execute_batch("PRAGMA foreign_keys = ON; PRAGMA journal_mode = WAL;
            CREATE TABLE IF NOT EXISTS sessions (id TEXT PRIMARY KEY, title TEXT NOT NULL, updated_at INTEGER NOT NULL);
            CREATE TABLE IF NOT EXISTS messages (id INTEGER PRIMARY KEY, session_id TEXT NOT NULL REFERENCES sessions(id) ON DELETE CASCADE, role TEXT NOT NULL, content TEXT NOT NULL);
            CREATE INDEX IF NOT EXISTS messages_session ON messages(session_id, id);
            CREATE TABLE IF NOT EXISTS turn_metadata (message_id INTEGER PRIMARY KEY REFERENCES messages(id) ON DELETE CASCADE, data TEXT NOT NULL);
            CREATE INDEX IF NOT EXISTS sessions_order ON sessions(updated_at DESC, id ASC);
            CREATE TABLE IF NOT EXISTS preferences (id INTEGER PRIMARY KEY CHECK(id=1), base_url TEXT NOT NULL, model TEXT NOT NULL);
            CREATE TABLE IF NOT EXISTS remembered_connection (id INTEGER PRIMARY KEY CHECK(id=1), data TEXT NOT NULL);
            CREATE TABLE IF NOT EXISTS model_choices (id INTEGER PRIMARY KEY CHECK(id=1), base_url TEXT NOT NULL, models TEXT NOT NULL);
            CREATE TABLE IF NOT EXISTS model_contexts (id INTEGER PRIMARY KEY CHECK(id=1), base_url TEXT NOT NULL, data TEXT NOT NULL);
            CREATE TABLE IF NOT EXISTS workspace_instructions (root TEXT PRIMARY KEY, data TEXT NOT NULL);
            CREATE TABLE IF NOT EXISTS project_skills (root TEXT NOT NULL, name TEXT NOT NULL, data TEXT NOT NULL, PRIMARY KEY(root,name));
            CREATE TABLE IF NOT EXISTS mcp_connections (root TEXT PRIMARY KEY, data TEXT NOT NULL);
            CREATE TABLE IF NOT EXISTS memory_preferences (id TEXT PRIMARY KEY, root TEXT NOT NULL, data TEXT NOT NULL);
            CREATE INDEX IF NOT EXISTS memory_scope ON memory_preferences(root,id);
            CREATE TABLE IF NOT EXISTS session_summaries (session_id TEXT PRIMARY KEY REFERENCES sessions(id) ON DELETE CASCADE, data TEXT NOT NULL);
            CREATE TABLE IF NOT EXISTS automatic_memory_policy (id INTEGER PRIMARY KEY CHECK(id=1), data TEXT NOT NULL);
            CREATE TABLE IF NOT EXISTS automatic_memory_attempts (session_id TEXT PRIMARY KEY REFERENCES sessions(id) ON DELETE CASCADE, message_id INTEGER NOT NULL, data TEXT NOT NULL);
            CREATE TABLE IF NOT EXISTS request_settings (id INTEGER PRIMARY KEY CHECK(id=1), data TEXT NOT NULL);
            CREATE TABLE IF NOT EXISTS session_workspaces (session_id TEXT PRIMARY KEY REFERENCES sessions(id) ON DELETE CASCADE, kind TEXT NOT NULL, root TEXT);
            CREATE TABLE IF NOT EXISTS projects (root TEXT PRIMARY KEY, name TEXT NOT NULL, updated_at INTEGER NOT NULL);
            CREATE TABLE IF NOT EXISTS file_changes (id INTEGER PRIMARY KEY, root TEXT NOT NULL, session_id TEXT NOT NULL, target TEXT NOT NULL, created_at INTEGER NOT NULL, status TEXT NOT NULL CHECK(status IN ('pending','applied','notApplied','reverted')), reverts INTEGER REFERENCES file_changes(id), before_text TEXT NOT NULL, after_text TEXT NOT NULL);
            CREATE INDEX IF NOT EXISTS file_changes_folder ON file_changes(root, id DESC);
            PRAGMA synchronous = FULL;").map_err(storage_error)?;
        let version: i64 = connection
            .query_row("PRAGMA user_version", [], |row| row.get(0))
            .map_err(storage_error)?;
        if version < 8 {
            for column in ["before_exists", "after_exists"] {
                let exists: bool = connection.query_row("SELECT EXISTS(SELECT 1 FROM pragma_table_info('file_changes') WHERE name=?1)", [column], |row| row.get(0)).map_err(storage_error)?;
                if !exists {
                    connection.execute_batch(&format!("ALTER TABLE file_changes ADD COLUMN {column} INTEGER NOT NULL DEFAULT 1 CHECK({column} IN (0,1));")).map_err(storage_error)?;
                }
            }
            connection
                .pragma_update(None, "user_version", 8)
                .map_err(storage_error)?;
        }
        if version < 9 {
            connection
                .pragma_update(None, "user_version", 9)
                .map_err(storage_error)?;
        }
        if version < 10 {
            connection
                .pragma_update(None, "user_version", 10)
                .map_err(storage_error)?;
        }
        if version < 11 {
            connection
                .pragma_update(None, "user_version", 11)
                .map_err(storage_error)?;
        }
        if version < 15 {
            connection
                .pragma_update(None, "user_version", 15)
                .map_err(storage_error)?;
        }
        if version < 16 {
            connection.execute_batch("BEGIN IMMEDIATE;
                CREATE TABLE mcp_connections_v16 (root TEXT NOT NULL, id TEXT NOT NULL, data TEXT NOT NULL, PRIMARY KEY(root,id));
                INSERT INTO mcp_connections_v16(root,id,data) SELECT root,'legacy',data FROM mcp_connections;
                DROP TABLE mcp_connections;
                ALTER TABLE mcp_connections_v16 RENAME TO mcp_connections;
                PRAGMA user_version=16;
                COMMIT;").map_err(storage_error)?;
        }
        if version < 17 {
            connection.execute_batch("BEGIN IMMEDIATE;
                CREATE TABLE model_request_settings (base_url TEXT NOT NULL, model TEXT NOT NULL, data TEXT NOT NULL, PRIMARY KEY(base_url,model));
                PRAGMA user_version=17; COMMIT;").map_err(storage_error)?;
        }
        if version < 18 {
            connection.execute_batch("BEGIN IMMEDIATE;
                CREATE TABLE task_feedback (message_id INTEGER PRIMARY KEY REFERENCES messages(id) ON DELETE CASCADE, data TEXT NOT NULL);
                PRAGMA user_version=18; COMMIT;").map_err(storage_error)?;
        }
        if version < 19 {
            connection.execute_batch("BEGIN IMMEDIATE;
                CREATE TABLE context_comparisons (id INTEGER PRIMARY KEY AUTOINCREMENT, session_id TEXT NOT NULL REFERENCES sessions(id) ON DELETE CASCADE, data TEXT NOT NULL);
                CREATE INDEX comparisons_session ON context_comparisons(session_id,id);
                PRAGMA user_version=19; COMMIT;").map_err(storage_error)?;
        }
        if version < 20 {
            connection.execute_batch("BEGIN IMMEDIATE;
                CREATE TABLE IF NOT EXISTS runs(id TEXT PRIMARY KEY, thread TEXT NOT NULL REFERENCES sessions(id) ON DELETE CASCADE, data TEXT NOT NULL);
                CREATE INDEX IF NOT EXISTS runs_thread ON runs(thread);
                CREATE TABLE IF NOT EXISTS run_events(run_id TEXT NOT NULL REFERENCES runs(id) ON DELETE CASCADE, sequence INTEGER NOT NULL, data TEXT NOT NULL, PRIMARY KEY(run_id,sequence));
                PRAGMA user_version=20; COMMIT;").map_err(storage_error)?;
        }
        if version < 21 {
            connection.execute_batch("BEGIN IMMEDIATE; CREATE TABLE IF NOT EXISTS scoped_settings(scope TEXT NOT NULL,scope_key TEXT NOT NULL,revision INTEGER NOT NULL,data TEXT NOT NULL,PRIMARY KEY(scope,scope_key));
            CREATE TRIGGER IF NOT EXISTS delete_thread_settings AFTER DELETE ON sessions BEGIN DELETE FROM scoped_settings WHERE scope='thread' AND scope_key=OLD.id; END;
            PRAGMA user_version=21; COMMIT;").map_err(storage_error)?;
        }
        if version < 22 {
            connection.execute_batch("BEGIN IMMEDIATE; CREATE TABLE IF NOT EXISTS session_drafts(session_id TEXT PRIMARY KEY REFERENCES sessions(id) ON DELETE CASCADE,text TEXT NOT NULL); PRAGMA user_version=22; COMMIT;").map_err(storage_error)?;
        }
        if version < 23 {
            connection.execute_batch("BEGIN IMMEDIATE; CREATE TABLE IF NOT EXISTS thread_context(session_id TEXT PRIMARY KEY REFERENCES sessions(id) ON DELETE CASCADE,auto_compact INTEGER NOT NULL DEFAULT 0,origin TEXT); PRAGMA user_version=23; COMMIT;").map_err(storage_error)?;
        }
        if version < 24 {
            connection.execute_batch("BEGIN IMMEDIATE;
            CREATE TABLE IF NOT EXISTS attachment_assets(digest TEXT PRIMARY KEY,data BLOB NOT NULL);
            CREATE TABLE IF NOT EXISTS draft_attachments(session_id TEXT PRIMARY KEY REFERENCES sessions(id) ON DELETE CASCADE,data TEXT NOT NULL);
            CREATE TABLE IF NOT EXISTS message_parts(message_id INTEGER PRIMARY KEY REFERENCES messages(id) ON DELETE CASCADE,data TEXT NOT NULL);
            CREATE TABLE IF NOT EXISTS model_images(base_url TEXT PRIMARY KEY,data TEXT NOT NULL);
            PRAGMA user_version=24;COMMIT;").map_err(storage_error)?;
        }
        if version < 25 {
            connection.execute_batch("BEGIN IMMEDIATE;
            CREATE TABLE IF NOT EXISTS message_timestamps(message_id INTEGER PRIMARY KEY REFERENCES messages(id) ON DELETE CASCADE,saved_at INTEGER NOT NULL);
            PRAGMA user_version=25;COMMIT;").map_err(storage_error)?;
        }
        if version < 26 {
            connection.execute_batch("BEGIN IMMEDIATE;
            CREATE TABLE IF NOT EXISTS web_configuration(id INTEGER PRIMARY KEY CHECK(id=1),data TEXT NOT NULL);
            PRAGMA user_version=26;COMMIT;").map_err(storage_error)?;
        }
        if version < 27 {
            connection.execute_batch("BEGIN IMMEDIATE;
            CREATE TABLE IF NOT EXISTS appearance(id INTEGER PRIMARY KEY CHECK(id=1),theme TEXT NOT NULL CHECK(theme IN ('system','light','dark')));
            PRAGMA user_version=27;COMMIT;").map_err(storage_error)?;
        }
        if version < 28 {
            connection.execute_batch("BEGIN IMMEDIATE; CREATE TABLE IF NOT EXISTS project_knowledge(root TEXT PRIMARY KEY,data TEXT NOT NULL); PRAGMA user_version=28; COMMIT;").map_err(storage_error)?;
        }
        Ok(Self {
            connection: Mutex::new(connection),
        })
    }
    fn lock(&self) -> Result<MutexGuard<'_, Connection>, String> {
        self.connection.lock().map_err(storage_error)
    }
}

impl SessionStore for SqliteStore {
    fn knowledge(&self, root: &str) -> Result<dolores_core::KnowledgeState, String> {
        self.read_knowledge(root)
    }
    fn save_knowledge(
        &self,
        root: &str,
        revision: u32,
        state: &dolores_core::KnowledgeState,
    ) -> Result<dolores_core::KnowledgeState, String> {
        self.write_knowledge(root, revision, state)
    }
    fn appearance(&self) -> Result<dolores_core::Appearance, String> {
        let theme: Option<String> = self
            .lock()?
            .query_row("SELECT theme FROM appearance WHERE id=1", [], |row| {
                row.get(0)
            })
            .optional()
            .map_err(storage_error)?;
        match theme.as_deref() {
            None | Some("system") => Ok(dolores_core::Appearance::System),
            Some("light") => Ok(dolores_core::Appearance::Light),
            Some("dark") => Ok(dolores_core::Appearance::Dark),
            _ => Err("Saved theme is invalid. Choose a theme in Settings.".into()),
        }
    }
    fn save_appearance(&self, theme: dolores_core::Appearance) -> Result<(), String> {
        let theme = match theme {
            dolores_core::Appearance::System => "system",
            dolores_core::Appearance::Light => "light",
            dolores_core::Appearance::Dark => "dark",
        };
        self.lock()?.execute("INSERT INTO appearance(id,theme) VALUES(1,?1) ON CONFLICT(id) DO UPDATE SET theme=excluded.theme", [theme]).map_err(storage_error)?;
        Ok(())
    }
    fn web_configuration(&self) -> Result<dolores_core::WebConfiguration, String> {
        self.read_web_configuration()
    }
    fn save_web_configuration(
        &self,
        revision: u32,
        configuration: &dolores_core::WebConfiguration,
    ) -> Result<dolores_core::WebConfiguration, String> {
        self.write_web_configuration(revision, configuration)
    }
    fn draft_attachments(&self, id: &str) -> Result<Vec<dolores_core::AttachmentRef>, String> {
        let c = self.lock()?;
        attachments::draft(&c, id)
    }
    fn add_attachment(&self, id: &str, data: &dolores_core::AttachmentData) -> Result<(), String> {
        self.save_attachment(id, data)
    }
    fn remove_attachment(&self, id: &str, digest: &str) -> Result<(), String> {
        self.drop_attachment(id, digest)
    }
    fn attachment_data(
        &self,
        id: &str,
        digest: &str,
    ) -> Result<dolores_core::AttachmentData, String> {
        self.read_attachment(id, digest)
    }
    fn cleanup_attachments(&self) -> Result<usize, String> {
        let c = self.lock()?;
        attachments::cleanup(&c)
    }
    fn image_models(&self, base: &str) -> Result<Vec<String>, String> {
        let c = self.lock()?;
        attachments::image_models(&c, base)
    }
    fn save_image_models(&self, base: &str, models: &[String]) -> Result<(), String> {
        let c = self.lock()?;
        attachments::save_image_models(&c, base, models)
    }

    fn fork_session(&self, source: &str, through: i64, id: &str) -> Result<Session, String> {
        self.fork_thread(source, through, id)
    }
    fn auto_compact(&self, id: &str) -> Result<bool, String> {
        self.read_auto_compact(id)
    }
    fn set_auto_compact(&self, id: &str, enabled: bool) -> Result<(), String> {
        self.write_auto_compact(id, enabled)
    }

    fn saved_draft(&self, id: &str) -> Result<String, String> {
        self.read_draft(id)
    }
    fn save_draft(&self, id: &str, text: &str) -> Result<(), String> {
        self.write_draft(id, text)
    }
    fn clear_draft_if(&self, id: &str, text: &str) -> Result<(), String> {
        self.lock()?
            .execute(
                "DELETE FROM session_drafts WHERE session_id=?1 AND text=?2",
                params![id, text],
            )
            .map_err(storage_error)?;
        Ok(())
    }
    fn scoped_settings(
        &self,
        scope: dolores_core::SettingsScope,
        key: &str,
    ) -> Result<dolores_core::ScopedSettings, String> {
        self.read_scoped_settings(scope, key)
    }
    fn save_scoped_settings(
        &self,
        scope: dolores_core::SettingsScope,
        key: &str,
        revision: u32,
        patch: &dolores_core::SettingsPatch,
    ) -> Result<dolores_core::ScopedSettings, String> {
        self.write_scoped_settings(scope, key, revision, patch)
    }

    fn begin_run(&self, run: &dolores_core::RunSnapshot) -> Result<(), String> {
        self.insert_run(run)
    }
    fn append_run_event(
        &self,
        id: &str,
        expected: u32,
        state: Option<dolores_core::RunState>,
        kind: &str,
        data: &serde_json::Value,
    ) -> Result<u32, String> {
        self.append_event(id, expected, state, kind, data)
    }
    fn runs(&self, thread: &str) -> Result<Vec<dolores_core::RunSnapshot>, String> {
        self.read_runs(thread)
    }
    fn run_events(&self, thread: &str, id: &str) -> Result<Vec<dolores_core::RunEvent>, String> {
        self.read_run_events(thread, id)
    }
    fn interrupt_runs(&self) -> Result<(), String> {
        self.mark_interrupted()
    }
    fn create_comparison(
        &self,
        run: &dolores_core::ContextComparison,
    ) -> Result<dolores_core::ContextComparison, String> {
        self.insert_comparison(run)
    }
    fn update_comparison(
        &self,
        run: &dolores_core::ContextComparison,
    ) -> Result<dolores_core::ContextComparison, String> {
        self.advance_comparison(run)
    }
    fn comparisons_page(
        &self,
        session: &str,
        cursor: Option<i64>,
    ) -> Result<dolores_core::HistoryPage<dolores_core::ComparisonSummary>, String> {
        self.read_comparisons(session, cursor)
    }
    fn comparison(
        &self,
        session: &str,
        id: i64,
    ) -> Result<dolores_core::ContextComparison, String> {
        self.read_comparison(session, id)
    }
    fn delete_comparison(&self, session: &str, id: i64, revision: u32) -> Result<(), String> {
        self.remove_comparison(session, id, revision)
    }
    fn save_task_feedback(
        &self,
        session: &str,
        draft: &dolores_core::FeedbackDraft,
    ) -> Result<dolores_core::TaskFeedback, String> {
        self.write_task_feedback(session, draft)
    }
    fn mcp_connection(&self, root: &str) -> Result<Option<dolores_core::McpConnection>, String> {
        self.read_mcp(root)
    }
    fn mcp_connections(&self, root: &str) -> Result<Vec<dolores_core::McpConnection>, String> {
        self.read_mcps(root)
    }
    fn mutate_mcp_connection_by_id(
        &self,
        root: &str,
        id: &str,
        revision: u32,
        forget: bool,
    ) -> Result<(), String> {
        self.change_mcp(root, id, revision, forget)
    }
    fn save_mcp_connection(
        &self,
        root: &str,
        connection: &dolores_core::McpConnection,
        expected: Option<u32>,
    ) -> Result<dolores_core::McpConnection, String> {
        self.write_mcp(root, connection, expected)
    }
    fn mutate_mcp_connection(&self, root: &str, revision: u32, forget: bool) -> Result<(), String> {
        self.change_mcp(root, "legacy", revision, forget)
    }
    fn promote_skill(
        &self,
        promotion: &dolores_core::SkillPromotion,
    ) -> Result<dolores_core::ProjectSkill, String> {
        self.promote_reviewed_skill(promotion)
    }
    fn global_skills(&self) -> Result<Vec<dolores_core::ProjectSkill>, String> {
        self.read_skills(skills::GLOBAL_ROOT)
    }
    fn activate_global_skill(
        &self,
        document: &dolores_core::SkillDocument,
        revision: Option<u32>,
        rollback: Option<u32>,
    ) -> Result<dolores_core::ProjectSkill, String> {
        self.activate_skill(skills::GLOBAL_ROOT, document, revision, rollback)
    }
    fn disable_global_skill(&self, name: &str, revision: u32) -> Result<(), String> {
        self.mutate_skill(skills::GLOBAL_ROOT, name, revision, false)
    }
    fn forget_global_skill(&self, name: &str, revision: u32) -> Result<(), String> {
        self.mutate_skill(skills::GLOBAL_ROOT, name, revision, true)
    }
    fn project_skills(&self, root: &str) -> Result<Vec<dolores_core::ProjectSkill>, String> {
        if !std::path::Path::new(root).is_absolute() {
            return Err("Project skills need an absolute working folder.".into());
        }
        self.read_skills(root)
    }
    fn activate_project_skill(
        &self,
        root: &str,
        document: &dolores_core::SkillDocument,
        revision: Option<u32>,
        rollback: Option<u32>,
    ) -> Result<dolores_core::ProjectSkill, String> {
        if !std::path::Path::new(root).is_absolute() {
            return Err("Project skills need an absolute working folder.".into());
        }
        self.activate_skill(root, document, revision, rollback)
    }
    fn disable_project_skill(&self, root: &str, name: &str, revision: u32) -> Result<(), String> {
        if !std::path::Path::new(root).is_absolute() {
            return Err("Project skills need an absolute working folder.".into());
        }
        self.mutate_skill(root, name, revision, false)
    }
    fn forget_project_skill(&self, root: &str, name: &str, revision: u32) -> Result<(), String> {
        if !std::path::Path::new(root).is_absolute() {
            return Err("Project skills need an absolute working folder.".into());
        }
        self.mutate_skill(root, name, revision, true)
    }
    fn automatic_memory_policy(&self) -> Result<dolores_core::AutomaticMemoryPolicy, String> {
        self.auto_policy()
    }
    fn set_automatic_memory_policy(
        &self,
        enabled: bool,
        revision: u32,
    ) -> Result<dolores_core::AutomaticMemoryPolicy, String> {
        self.write_auto_policy(enabled, revision)
    }
    fn automatic_memory_attempt(
        &self,
        session: &str,
    ) -> Result<Option<dolores_core::AutomaticMemoryAttempt>, String> {
        self.auto_attempt(session)
    }
    fn claim_automatic_memory(
        &self,
        session: &str,
        source: &dolores_core::MemoryMessage,
        revision: u32,
    ) -> Result<bool, String> {
        self.claim_auto(session, source, revision)
    }
    fn finish_automatic_memory(
        &self,
        update: &dolores_core::AutomaticMemoryUpdate,
        cancel: &tokio_util::sync::CancellationToken,
    ) -> Result<dolores_core::AutomaticMemoryAttempt, String> {
        self.finish_auto(update, cancel)
    }
    fn session_summary(
        &self,
        session: &str,
    ) -> Result<Option<dolores_core::SessionSummary>, String> {
        self.read_summary(session)
    }
    fn review_summary_batch(&self, session: &str) -> Result<dolores_core::SummaryBatch, String> {
        self.summary_batch(session)
    }
    fn save_session_summary(
        &self,
        session: &str,
        batch: &dolores_core::SummaryBatch,
        text: &str,
        model: &str,
    ) -> Result<dolores_core::SessionSummary, String> {
        self.write_summary(session, batch, text, model)
    }
    fn correct_session_summary(
        &self,
        session: &str,
        revision: u32,
        text: &str,
    ) -> Result<dolores_core::SessionSummary, String> {
        self.correct_summary(session, revision, text)
    }
    fn delete_session_summary(&self, session: &str, revision: u32) -> Result<(), String> {
        self.remove_summary(session, revision)
    }
    fn summary_context_history(
        &self,
        session: &str,
    ) -> Result<dolores_core::SummaryHistory, String> {
        self.summary_history(session)
    }
    fn memory_source_messages(
        &self,
        session: &str,
    ) -> Result<dolores_core::HistoryPage<dolores_core::MemoryMessage>, String> {
        self.read_memory_sources(session)
    }
    fn memory_source_message(
        &self,
        session: &str,
        id: i64,
    ) -> Result<Option<dolores_core::MemoryMessage>, String> {
        self.read_memory_source(session, id)
    }
    fn memory_preferences(
        &self,
        root: Option<&str>,
    ) -> Result<Vec<dolores_core::MemoryPreference>, String> {
        self.read_memory(root)
    }
    fn save_memory_preference(
        &self,
        root: Option<&str>,
        draft: &dolores_core::MemoryDraft,
    ) -> Result<dolores_core::MemoryPreference, String> {
        self.write_memory(root, draft)
    }
    fn save_suggested_memory_preference(
        &self,
        root: Option<&str>,
        draft: &dolores_core::MemoryDraft,
        sources: &[dolores_core::MemoryMessage],
    ) -> Result<dolores_core::MemoryPreference, String> {
        self.write_suggested_memory(root, draft, sources)
    }
    fn delete_memory_preference(
        &self,
        root: Option<&str>,
        id: &str,
        revision: u32,
    ) -> Result<(), String> {
        self.remove_memory(root, id, revision)
    }
    fn workspace_instructions(
        &self,
        root: &str,
    ) -> Result<Option<dolores_core::WorkspaceInstructions>, String> {
        self.read_instructions(root)
    }
    fn save_workspace_instructions(
        &self,
        root: &str,
        value: Option<&dolores_core::WorkspaceInstructions>,
    ) -> Result<(), String> {
        self.save_instructions(root, value)
    }
    fn begin_change(&self, draft: &dolores_core::ChangeDraft) -> Result<i64, String> {
        self.insert_change(draft)
    }
    fn finish_change(&self, id: i64, applied: bool) -> Result<(), String> {
        self.complete_change(id, applied)
    }
    fn changes_page(
        &self,
        root: &str,
        cursor: Option<i64>,
    ) -> Result<dolores_core::HistoryPage<dolores_core::FileChange>, String> {
        self.read_changes(root, cursor)
    }
    fn change_snapshot(&self, id: i64) -> Result<dolores_core::ChangeSnapshot, String> {
        self.read_change(id)
    }
    fn workspace(&self, id: &str) -> Result<dolores_core::SessionWorkspace, String> {
        self.read_workspace(id)
    }
    fn projects(&self) -> Result<Vec<dolores_core::Project>, String> {
        self.read_projects()
    }
    fn create_workspace_session(
        &self,
        id: &str,
        workspace: &dolores_core::SessionWorkspace,
    ) -> Result<Session, String> {
        self.save_workspace_session(id, workspace)
    }
    fn request_settings(&self) -> Result<RequestSettings, String> {
        let data: Option<String> = self
            .lock()?
            .query_row("SELECT data FROM request_settings WHERE id=1", [], |row| {
                row.get(0)
            })
            .optional()
            .map_err(storage_error)?;
        let settings = data
            .map(|data| serde_json::from_str::<RequestSettings>(&data))
            .transpose()
            .map_err(|_| {
                "Saved request settings could not be read. Reset them in Request settings."
                    .to_string()
            })?
            .unwrap_or_default();
        settings.validate()?;
        Ok(settings)
    }
    fn model_request_settings(
        &self,
        base_url: &str,
    ) -> Result<std::collections::BTreeMap<String, RequestSettings>, String> {
        self.read_generation_profiles(base_url)
    }
    fn save_model_request_settings(
        &self,
        preferences: &ConnectionPreferences,
        settings: Option<&RequestSettings>,
    ) -> Result<(), String> {
        self.write_generation_profile(preferences, settings)
    }
    fn save_request_settings(&self, settings: &RequestSettings) -> Result<(), String> {
        settings.validate()?;
        let data = serde_json::to_string(settings).map_err(storage_error)?;
        self.lock()?.execute("INSERT INTO request_settings(id,data) VALUES(1,?1) ON CONFLICT(id) DO UPDATE SET data=excluded.data", [data]).map_err(storage_error)?;
        Ok(())
    }
    fn sessions_page(
        &self,
        cursor: Option<dolores_core::SessionCursor>,
        newer: bool,
        limit: usize,
    ) -> Result<dolores_core::HistoryPage<Session>, String> {
        self.read_sessions_page(cursor, newer, limit)
    }
    fn messages_page(
        &self,
        id: &str,
        cursor: Option<i64>,
        newer: bool,
        limit: usize,
    ) -> Result<dolores_core::HistoryPage<dolores_core::StoredMessage>, String> {
        self.read_messages_page(id, cursor, newer, limit)
    }
    fn export_conversation(
        &self,
        id: &str,
        format: dolores_core::ExportFormat,
        output: &mut dyn std::io::Write,
    ) -> Result<u64, String> {
        self.write_export(id, format, output)
    }
    fn descriptor(&self) -> PluginDescriptor {
        PluginDescriptor {
            id: "dolores.store.sqlite",
            kind: "storage",
            api_version: 1,
        }
    }
    fn list(&self) -> Result<Vec<Session>, String> {
        let connection = self.lock()?;
        let mut statement = connection
            .prepare(
                "SELECT id,title,updated_at FROM sessions ORDER BY updated_at DESC,id LIMIT 100",
            )
            .map_err(storage_error)?;
        let rows = statement
            .query_map([], |row| {
                Ok(Session {
                    id: row.get(0)?,
                    title: row.get(1)?,
                    updated_at: row.get(2)?,
                })
            })
            .map_err(storage_error)?;
        rows.collect::<Result<Vec<_>, _>>().map_err(storage_error)
    }
    fn create(&self, id: &str) -> Result<Session, String> {
        let session = Session {
            id: id.into(),
            title: "New conversation".into(),
            updated_at: now(),
        };
        self.lock()?
            .execute(
                "INSERT INTO sessions(id,title,updated_at) VALUES(?1,?2,?3)",
                params![session.id, session.title, session.updated_at],
            )
            .map_err(storage_error)?;
        Ok(session)
    }
    fn messages(&self, id: &str) -> Result<Vec<Message>, String> {
        self.context_history(id).map(|(messages, _)| messages)
    }
    fn context_history(&self, id: &str) -> Result<(Vec<Message>, Option<u64>), String> {
        let connection = self.lock()?;
        let snapshot = connection.unchecked_transaction().map_err(storage_error)?;
        let exists: bool = snapshot
            .query_row(
                "SELECT EXISTS(SELECT 1 FROM sessions WHERE id=?1)",
                [id],
                |row| row.get(0),
            )
            .map_err(storage_error)?;
        if !exists {
            return Err("Conversation no longer exists.".into());
        }
        let count: u64 = snapshot
            .query_row(
                "SELECT COUNT(*) FROM messages WHERE session_id=?1",
                [id],
                |row| row.get(0),
            )
            .map_err(storage_error)?;
        if !count.is_multiple_of(2) {
            return Err("Stored conversation has an incomplete turn.".into());
        }
        let mut statement = snapshot.prepare("SELECT role,content,id FROM (SELECT id,role,content FROM messages WHERE session_id=?1 ORDER BY id DESC LIMIT ?2) ORDER BY id ASC").map_err(storage_error)?;
        let rows = statement
            .query_map(params![id, HISTORY_LIMIT as i64], |row| {
                let role: String = row.get(0)?;
                let role = match role.as_str() {
                    "user" => Role::User,
                    "assistant" => Role::Assistant,
                    _ => return Err(rusqlite::Error::InvalidQuery),
                };
                Ok(Message {
                    parts: attachments::parts(&snapshot, row.get(2)?).map_err(|e| {
                        rusqlite::Error::ToSqlConversionFailure(Box::new(std::io::Error::other(e)))
                    })?,
                    role,
                    content: row.get(1)?,
                })
            })
            .map_err(storage_error)?;
        Ok((
            rows.collect::<Result<Vec<_>, _>>().map_err(storage_error)?,
            Some(count / 2),
        ))
    }
    fn delete(&self, id: &str) -> Result<(), String> {
        self.lock()?
            .execute("DELETE FROM sessions WHERE id=?1", [id])
            .map_err(storage_error)?;
        Ok(())
    }
    fn commit_turn(&self, id: &str, user: &str, assistant: &str) -> Result<(), String> {
        self.save_turn(id, user, assistant, None, None)
    }
    fn commit_turn_metadata(
        &self,
        id: &str,
        user: &str,
        assistant: &str,
        metadata: &TurnMetadata,
    ) -> Result<(), String> {
        self.save_turn(id, user, assistant, Some(metadata), None)
    }
    fn commit_continuation(
        &self,
        id: &str,
        expected: i64,
        user: &str,
        assistant: &str,
        metadata: &TurnMetadata,
    ) -> Result<(), String> {
        self.save_turn(id, user, assistant, Some(metadata), Some(expected))
    }
    fn preferences(&self) -> Result<ConnectionPreferences, String> {
        self.read_preferences()
    }

    fn save_preferences(&self, preferences: &ConnectionPreferences) -> Result<(), String> {
        self.lock()?.execute("INSERT INTO preferences(id,base_url,model) VALUES(1,?1,?2) ON CONFLICT(id) DO UPDATE SET base_url=excluded.base_url,model=excluded.model", params![preferences.base_url,preferences.model]).map_err(storage_error)?;
        Ok(())
    }
    fn remembered_connection(&self) -> Result<Option<RememberedConnection>, String> {
        let data: Option<String> = self
            .lock()?
            .query_row(
                "SELECT data FROM remembered_connection WHERE id=1",
                [],
                |row| row.get(0),
            )
            .optional()
            .map_err(storage_error)?;
        data.map(|data| serde_json::from_str(&data).map_err(storage_error))
            .transpose()
    }
    fn save_connection(
        &self,
        preferences: &ConnectionPreferences,
        remembered: Option<&RememberedConnection>,
    ) -> Result<(), String> {
        self.save_connection_models(
            preferences,
            remembered,
            std::slice::from_ref(&preferences.model),
        )
    }
    fn model_choices(&self, base_url: &str) -> Result<Vec<String>, String> {
        let data: Option<String> = self
            .lock()?
            .query_row(
                "SELECT models FROM model_choices WHERE id=1 AND base_url=?1",
                [base_url],
                |row| row.get(0),
            )
            .optional()
            .map_err(storage_error)?;
        data.map(|data| serde_json::from_str(&data).map_err(storage_error))
            .transpose()
            .map(|models| models.unwrap_or_default())
    }
    fn save_connection_models(
        &self,
        preferences: &ConnectionPreferences,
        remembered: Option<&RememberedConnection>,
        models: &[String],
    ) -> Result<(), String> {
        self.save_model_configuration(preferences, remembered, models, None)
    }
    fn model_contexts(&self, base_url: &str) -> Result<ModelContexts, String> {
        let data: Option<String> = self
            .lock()?
            .query_row(
                "SELECT data FROM model_contexts WHERE id=1 AND base_url=?1",
                [base_url],
                |row| row.get(0),
            )
            .optional()
            .map_err(storage_error)?;
        data.map(|data| serde_json::from_str(&data).map_err(storage_error))
            .transpose()
            .map(|contexts| contexts.unwrap_or_default())
    }
    fn save_connection_model_contexts(
        &self,
        preferences: &ConnectionPreferences,
        remembered: Option<&RememberedConnection>,
        models: &[String],
        contexts: &ModelContexts,
    ) -> Result<(), String> {
        dolores_core::validate_model_contexts(contexts, models)?;
        self.save_model_configuration(preferences, remembered, models, Some(contexts))
    }
}

impl SqliteStore {
    fn save_model_configuration(
        &self,
        preferences: &ConnectionPreferences,
        remembered: Option<&RememberedConnection>,
        models: &[String],
        contexts: Option<&ModelContexts>,
    ) -> Result<(), String> {
        let mut connection = self.lock()?;
        let transaction = connection.transaction().map_err(storage_error)?;
        let contexts = match contexts {
            Some(contexts) => contexts.clone(),
            None => {
                let data: Option<String> = transaction
                    .query_row(
                        "SELECT data FROM model_contexts WHERE id=1 AND base_url=?1",
                        [&preferences.base_url],
                        |row| row.get(0),
                    )
                    .optional()
                    .map_err(storage_error)?;
                let mut contexts: ModelContexts = data
                    .map(|data| serde_json::from_str(&data).map_err(storage_error))
                    .transpose()?
                    .unwrap_or_default();
                contexts.retain(|id, _| models.contains(id));
                contexts
            }
        };
        let data = serde_json::to_string(&contexts).map_err(storage_error)?;
        // Disabled models lose their overrides in the same configuration transaction.
        let mut statement = transaction
            .prepare("SELECT model FROM model_request_settings WHERE base_url=?1")
            .map_err(storage_error)?;
        let saved = statement
            .query_map([&preferences.base_url], |r| r.get::<_, String>(0))
            .map_err(storage_error)?
            .collect::<Result<Vec<_>, _>>()
            .map_err(storage_error)?;
        drop(statement);
        for model in saved.into_iter().filter(|model| !models.contains(model)) {
            transaction
                .execute(
                    "DELETE FROM model_request_settings WHERE base_url=?1 AND model=?2",
                    params![preferences.base_url, model],
                )
                .map_err(storage_error)?;
        }
        transaction.execute("INSERT INTO model_contexts(id,base_url,data) VALUES(1,?1,?2) ON CONFLICT(id) DO UPDATE SET base_url=excluded.base_url,data=excluded.data", params![preferences.base_url, data]).map_err(storage_error)?;
        transaction.execute("INSERT INTO preferences(id,base_url,model) VALUES(1,?1,?2) ON CONFLICT(id) DO UPDATE SET base_url=excluded.base_url,model=excluded.model", params![preferences.base_url, preferences.model]).map_err(storage_error)?;
        let data = serde_json::to_string(models).map_err(storage_error)?;
        transaction.execute("INSERT INTO model_choices(id,base_url,models) VALUES(1,?1,?2) ON CONFLICT(id) DO UPDATE SET base_url=excluded.base_url,models=excluded.models", params![preferences.base_url, data]).map_err(storage_error)?;
        match remembered {
            Some(record) => {
                let data = serde_json::to_string(record).map_err(storage_error)?;
                transaction.execute("INSERT INTO remembered_connection(id,data) VALUES(1,?1) ON CONFLICT(id) DO UPDATE SET data=excluded.data", [data]).map_err(storage_error)?;
            }
            None => {
                transaction
                    .execute("DELETE FROM remembered_connection WHERE id=1", [])
                    .map_err(storage_error)?;
            }
        }
        transaction.commit().map_err(storage_error)
    }
}

impl SqliteStore {
    fn save_turn(
        &self,
        id: &str,
        user: &str,
        assistant: &str,
        metadata: Option<&TurnMetadata>,
        expected: Option<i64>,
    ) -> Result<(), String> {
        let mut connection = self.lock()?;
        let transaction = connection.transaction().map_err(storage_error)?;
        if let Some(expected) = expected {
            let latest: Option<i64> = transaction
                .query_row(
                    "SELECT MAX(id) FROM messages WHERE session_id=?1",
                    [id],
                    |r| r.get(0),
                )
                .map_err(storage_error)?;
            if latest != Some(expected) {
                return Err("The chat changed during continuation. Saved tool effects remain; review the latest chat before continuing.".into());
            }
        }
        transaction.execute("INSERT INTO messages(session_id,role,content) VALUES(?1,'user',?2),(?1,'assistant',?3)", params![id,user,assistant]).map_err(storage_error)?;
        let assistant_id = transaction.last_insert_rowid();
        attachments::commit_draft(&transaction, id, assistant_id - 1)?;
        if let Some(metadata) = metadata {
            let data = serde_json::to_string(metadata).map_err(storage_error)?;
            transaction
                .execute(
                    "INSERT INTO turn_metadata(message_id,data) VALUES(?1,?2)",
                    params![assistant_id, data],
                )
                .map_err(storage_error)?;
        }
        // Both timestamps are local save times, not guessed send/receive times.
        let saved_at = now();
        transaction
            .execute(
                "INSERT INTO message_timestamps(message_id,saved_at) VALUES(?1,?3),(?2,?3)",
                params![assistant_id - 1, assistant_id, saved_at],
            )
            .map_err(storage_error)?;
        let title: String = user
            .split_whitespace()
            .collect::<Vec<_>>()
            .join(" ")
            .chars()
            .take(48)
            .collect();
        transaction.execute("UPDATE sessions SET title=CASE WHEN (SELECT COUNT(*) FROM messages WHERE session_id=?1)=2 THEN ?2 ELSE title END,updated_at=?3 WHERE id=?1", params![id,title,now()]).map_err(storage_error)?;
        transaction.commit().map_err(storage_error)
    }
    fn read_preferences(&self) -> Result<ConnectionPreferences, String> {
        self.lock()?
            .query_row(
                "SELECT base_url,model FROM preferences WHERE id=1",
                [],
                |row| {
                    Ok(ConnectionPreferences {
                        base_url: row.get(0)?,
                        model: row.get(1)?,
                    })
                },
            )
            .optional()
            .map_err(storage_error)
            .map(|p| p.unwrap_or_default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn paused_continuation_is_atomic_persistent_and_refuses_reused_source() {
        let directory = tempfile::tempdir().unwrap();
        let file = directory.path().join("paused.db");
        let store = SqliteStore::open(&file).unwrap();
        store.create("work").unwrap();
        let metadata = TurnMetadata {
            model: "fixture".into(),
            usage: None,
            context: dolores_core::ContextSummary::from_messages(&[], None),
            request_settings: None,
            agent: None,
            paused: Some(dolores_core::PausedTask {
                segments: 1,
                reason: dolores_core::PauseReason::OutputLimit,
                task: "Keep 世界".into(),
                receipts: vec![],
            }),
        };
        store
            .commit_turn_metadata("work", "Keep 世界", "Saved partial 世界", &metadata)
            .unwrap();
        let source = store
            .messages_page("work", None, false, 2)
            .unwrap()
            .items
            .last()
            .unwrap()
            .id;
        drop(store);
        let store = SqliteStore::open(&file).unwrap();
        assert_eq!(
            store
                .messages_page("work", None, false, 2)
                .unwrap()
                .items
                .last()
                .unwrap()
                .metadata
                .as_ref()
                .unwrap()
                .paused,
            metadata.paused
        );
        store.lock().unwrap().execute_batch("CREATE TRIGGER reject_continue BEFORE INSERT ON messages BEGIN SELECT RAISE(ABORT,'fixture'); END;").unwrap();
        assert!(store
            .commit_continuation("work", source, "Continue", "still partial", &metadata)
            .is_err());
        assert_eq!(store.messages("work").unwrap().len(), 2);
        store
            .lock()
            .unwrap()
            .execute_batch("DROP TRIGGER reject_continue;")
            .unwrap();
        store
            .commit_continuation("work", source, "Continue", "still partial", &metadata)
            .unwrap();
        assert!(store
            .commit_continuation("work", source, "Continue", "duplicate", &metadata)
            .is_err());
        assert_eq!(store.messages("work").unwrap().len(), 4);
    }
    #[test]
    fn request_settings_are_nonsecret_persistent_and_failed_writes_leave_previous_settings() {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("settings.db");
        let store = SqliteStore::open(&path).unwrap();
        assert_eq!(
            store.request_settings().unwrap(),
            RequestSettings::default()
        );
        store.create("preserved").unwrap();
        store
            .commit_turn("preserved", "question", "answer")
            .unwrap();
        let settings = RequestSettings {
            max_output_tokens: 4096,
            timeout_seconds: 300,
            reasoning: Default::default(),
        };
        store.save_request_settings(&settings).unwrap();
        assert!(store
            .save_request_settings(&RequestSettings {
                max_output_tokens: 0,
                timeout_seconds: 300,
                reasoning: Default::default()
            })
            .is_err());
        store.lock().unwrap().execute_batch("CREATE TRIGGER reject_settings BEFORE UPDATE ON request_settings BEGIN SELECT RAISE(ABORT,'fixture'); END;").unwrap();
        assert!(store
            .save_request_settings(&RequestSettings::default())
            .is_err());
        assert_eq!(store.request_settings().unwrap(), settings);
        drop(store);
        let store = SqliteStore::open(&path).unwrap();
        assert_eq!(store.request_settings().unwrap(), settings);
        assert_eq!(store.messages("preserved").unwrap().len(), 2);
        // Old reply metadata decodes without pretending a historical setting.
        let old = r#"{"model":"old","usage":null,"context":{"includedTurns":0,"savedTurns":0,"omittedTurns":0,"textBytes":200,"maxTextBytes":131072,"maxTurns":40}}"#;
        let metadata: TurnMetadata = serde_json::from_str(old).unwrap();
        assert!(metadata.request_settings.is_none());
    }
    #[test]
    fn usage_and_context_survive_restart_export_and_atomic_failure() {
        use dolores_core::{ContextSummary, ExportFormat, TokenUsage};
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("usage.db");
        let store = SqliteStore::open(&path).unwrap();
        store.create("session").unwrap();
        store.commit_turn("session", "legacy", "no usage").unwrap();
        let metadata = TurnMetadata {
            paused: None,
            model: "fixture".into(),
            request_settings: None,
            agent: None,
            usage: Some(TokenUsage {
                input_tokens: Some(0),
                output_tokens: Some(4),
                total_tokens: None,
                ..TokenUsage::default()
            }),
            context: ContextSummary::from_messages(
                &dolores_core::prepare_context(store.messages("session").unwrap(), "next").unwrap(),
                Some(1),
            ),
        };
        store.lock().unwrap().execute_batch("CREATE TRIGGER reject_usage BEFORE INSERT ON turn_metadata BEGIN SELECT RAISE(ABORT,'fixture'); END;").unwrap();
        assert!(store
            .commit_turn_metadata("session", "next", "reply", &metadata)
            .is_err());
        assert_eq!(store.messages("session").unwrap().len(), 2);
        store
            .lock()
            .unwrap()
            .execute_batch("DROP TRIGGER reject_usage;")
            .unwrap();
        store
            .commit_turn_metadata("session", "next", "reply", &metadata)
            .unwrap();
        drop(store);
        let store = SqliteStore::open(&path).unwrap();
        let page = store.messages_page("session", None, false, 80).unwrap();
        assert!(page.items[1].metadata.is_none());
        assert_eq!(page.items[3].metadata.as_ref(), Some(&metadata));
        let mut output = Vec::new();
        store
            .export_conversation("session", ExportFormat::Json, &mut output)
            .unwrap();
        let json: serde_json::Value = serde_json::from_slice(&output).unwrap();
        assert!(json["messages"][1].get("metadata").is_none());
        assert_eq!(json["messages"][3]["metadata"]["usage"]["inputTokens"], 0);
        assert!(json["messages"][3]["metadata"]["usage"]["totalTokens"].is_null());
        let mut markdown = Vec::new();
        store
            .export_conversation("session", ExportFormat::Markdown, &mut markdown)
            .unwrap();
        let markdown = String::from_utf8(markdown).unwrap();
        assert!(
            markdown.contains("Request usage and context:")
                && markdown.contains("\"inputTokens\":0")
        );
        store.delete("session").unwrap();
        let count: i64 = store
            .lock()
            .unwrap()
            .query_row("SELECT COUNT(*) FROM turn_metadata", [], |row| row.get(0))
            .unwrap();
        assert_eq!(count, 0);
    }
    #[test]
    fn version_three_history_migrates_without_rewriting_legacy_turns() {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("legacy.db");
        let old = Connection::open(&path).unwrap();
        old.execute_batch("CREATE TABLE sessions(id TEXT PRIMARY KEY,title TEXT NOT NULL,updated_at INTEGER NOT NULL); CREATE TABLE messages(id INTEGER PRIMARY KEY,session_id TEXT NOT NULL REFERENCES sessions(id) ON DELETE CASCADE,role TEXT NOT NULL,content TEXT NOT NULL); INSERT INTO sessions VALUES('legacy','Original title',1); INSERT INTO messages VALUES(1,'legacy','user','Original question'),(2,'legacy','assistant','Original answer'); PRAGMA user_version=3;").unwrap();
        drop(old);
        let store = SqliteStore::open(&path).unwrap();
        let page = store.messages_page("legacy", None, false, 80).unwrap();
        assert_eq!(page.items.len(), 2);
        assert_eq!(page.items[1].id, 2);
        assert_eq!(page.items[1].content, "Original answer");
        assert!(page.items[1].metadata.is_none());
        let version: i64 = store
            .lock()
            .unwrap()
            .query_row("PRAGMA user_version", [], |row| row.get(0))
            .unwrap();
        assert_eq!(version, SCHEMA_VERSION);
        assert_eq!(store.list().unwrap()[0].title, "Original title");
    }
    #[test]
    fn context_snapshot_counts_all_saved_turns_without_loading_all_history() {
        let store = SqliteStore::open(Path::new(":memory:")).unwrap();
        store.create("long").unwrap();
        for i in 0..63 {
            store
                .commit_turn("long", &format!("u{i}"), "reply")
                .unwrap();
        }
        let (history, count) = store.context_history("long").unwrap();
        assert_eq!(history.len(), 80);
        assert_eq!(count, Some(63));
        assert_eq!(history[0].content, "u23");
    }
    #[test]
    fn appearance_survives_restart_and_failed_save_without_changing_chat() {
        use dolores_core::Appearance;
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("appearance.db");
        let store = SqliteStore::open(&path).unwrap();
        assert_eq!(store.appearance().unwrap(), Appearance::System);
        store.create("chat").unwrap();
        store
            .commit_turn("chat", "Original question", "Original reply")
            .unwrap();
        let history = store.messages("chat").unwrap();
        let preferences = store.preferences().unwrap();
        store.save_appearance(Appearance::Dark).unwrap();
        store.lock().unwrap().execute_batch("CREATE TRIGGER reject_theme BEFORE UPDATE ON appearance BEGIN SELECT RAISE(ABORT,'fixture'); END;").unwrap();
        assert!(store.save_appearance(Appearance::Light).is_err());
        assert_eq!(store.appearance().unwrap(), Appearance::Dark);
        drop(store);
        let store = SqliteStore::open(&path).unwrap();
        assert_eq!(store.appearance().unwrap(), Appearance::Dark);
        assert_eq!(store.preferences().unwrap(), preferences);
        assert_eq!(store.messages("chat").unwrap().len(), history.len());
        assert_eq!(
            store.messages("chat").unwrap()[0].content,
            history[0].content
        );
        store
            .lock()
            .unwrap()
            .execute_batch("DROP TRIGGER reject_theme;")
            .unwrap();
        store.save_appearance(Appearance::System).unwrap();
        assert_eq!(store.appearance().unwrap(), Appearance::System);
    }
    #[test]
    fn connection_metadata_and_preferences_commit_together_and_migrate_legacy_database() {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("legacy.db");
        let legacy = Connection::open(&path).unwrap();
        legacy.execute_batch("CREATE TABLE preferences(id INTEGER PRIMARY KEY CHECK(id=1),base_url TEXT NOT NULL,model TEXT NOT NULL); INSERT INTO preferences VALUES(1,'http://localhost:1234/v1','old'); PRAGMA user_version=1;").unwrap();
        drop(legacy);
        let store = SqliteStore::open(&path).unwrap();
        assert_eq!(store.preferences().unwrap().model, "old");
        assert!(store.remembered_connection().unwrap().is_none());
        let preferences = ConnectionPreferences {
            base_url: "https://example.com/v1".into(),
            model: "new".into(),
        };
        let saved = RememberedConnection {
            preferences: preferences.clone(),
            credential_id: None,
        };
        store.lock().unwrap().execute_batch("CREATE TRIGGER reject_remember BEFORE INSERT ON remembered_connection BEGIN SELECT RAISE(ABORT,'fixture'); END;").unwrap();
        assert!(store.save_connection(&preferences, Some(&saved)).is_err());
        assert_eq!(store.preferences().unwrap().model, "old");
        store
            .lock()
            .unwrap()
            .execute_batch("DROP TRIGGER reject_remember;")
            .unwrap();
        store.save_connection(&preferences, Some(&saved)).unwrap();
        drop(store);
        let reopened = SqliteStore::open(&path).unwrap();
        assert_eq!(reopened.preferences().unwrap(), preferences);
        assert_eq!(
            reopened
                .remembered_connection()
                .unwrap()
                .unwrap()
                .preferences,
            preferences
        );
    }
    #[test]
    fn turn_is_atomic_and_survives_restart_and_delete_cascades() {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("chat.db");
        {
            let store = SqliteStore::open(&path).unwrap();
            store.create("session").unwrap();
            store
                .commit_turn("session", "a question", "an answer")
                .unwrap();
            assert!(store.commit_turn("missing", "orphan", "reply").is_err());
            store
                .save_preferences(&ConnectionPreferences {
                    base_url: "http://localhost:1234/v1".into(),
                    model: "test".into(),
                })
                .unwrap();
        }
        let store = SqliteStore::open(&path).unwrap();
        assert_eq!(store.list().unwrap()[0].title, "a question");
        assert_eq!(store.messages("session").unwrap().len(), 2);
        assert_eq!(store.preferences().unwrap().model, "test");
        store.delete("session").unwrap();
        assert!(store.messages("session").is_err());
        let count: i64 = store
            .lock()
            .unwrap()
            .query_row("SELECT COUNT(*) FROM messages", [], |row| row.get(0))
            .unwrap();
        assert_eq!(count, 0);
    }
    #[test]
    fn history_keeps_newest_complete_turns() {
        let store = SqliteStore::open(Path::new(":memory:")).unwrap();
        store.create("s").unwrap();
        for i in 0..50 {
            store.commit_turn("s", &i.to_string(), "reply").unwrap();
        }
        let messages = store.messages("s").unwrap();
        assert_eq!(messages.len(), HISTORY_LIMIT);
        assert_eq!(messages[0].content, "10");
        assert_eq!(messages.last().unwrap().role, Role::Assistant);
    }
}
