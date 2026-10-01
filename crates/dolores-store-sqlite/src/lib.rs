use dolores_core::{
    ConnectionPreferences, Message, PluginDescriptor, RememberedConnection, Role, Session,
    SessionStore, TurnMetadata, HISTORY_LIMIT,
};
use rusqlite::{params, Connection, OptionalExtension};
mod history;
use std::{
    path::Path,
    sync::{Mutex, MutexGuard},
    time::{SystemTime, UNIX_EPOCH},
};

pub struct SqliteStore {
    connection: Mutex<Connection>,
}
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
            CREATE TABLE IF NOT EXISTS model_choices (id INTEGER PRIMARY KEY CHECK(id=1), base_url TEXT NOT NULL, models TEXT NOT NULL);").map_err(storage_error)?;
        let version: i64 = connection
            .query_row("PRAGMA user_version", [], |row| row.get(0))
            .map_err(storage_error)?;
        if version < 4 {
            connection
                .pragma_update(None, "user_version", 4)
                .map_err(storage_error)?;
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
        let mut statement = snapshot.prepare("SELECT role,content FROM (SELECT id,role,content FROM messages WHERE session_id=?1 ORDER BY id DESC LIMIT ?2) ORDER BY id ASC").map_err(storage_error)?;
        let rows = statement
            .query_map(params![id, HISTORY_LIMIT as i64], |row| {
                let role: String = row.get(0)?;
                let role = match role.as_str() {
                    "user" => Role::User,
                    "assistant" => Role::Assistant,
                    _ => return Err(rusqlite::Error::InvalidQuery),
                };
                Ok(Message {
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
        self.save_turn(id, user, assistant, None)
    }
    fn commit_turn_metadata(
        &self,
        id: &str,
        user: &str,
        assistant: &str,
        metadata: &TurnMetadata,
    ) -> Result<(), String> {
        self.save_turn(id, user, assistant, Some(metadata))
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
        let mut connection = self.lock()?;
        let transaction = connection.transaction().map_err(storage_error)?;
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
    ) -> Result<(), String> {
        let mut connection = self.lock()?;
        let transaction = connection.transaction().map_err(storage_error)?;
        transaction.execute("INSERT INTO messages(session_id,role,content) VALUES(?1,'user',?2),(?1,'assistant',?3)", params![id,user,assistant]).map_err(storage_error)?;
        if let Some(metadata) = metadata {
            let data = serde_json::to_string(metadata).map_err(storage_error)?;
            transaction
                .execute(
                    "INSERT INTO turn_metadata(message_id,data) VALUES(?1,?2)",
                    params![transaction.last_insert_rowid(), data],
                )
                .map_err(storage_error)?;
        }
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
    fn usage_and_context_survive_restart_export_and_atomic_failure() {
        use dolores_core::{ContextSummary, ExportFormat, TokenUsage};
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("usage.db");
        let store = SqliteStore::open(&path).unwrap();
        store.create("session").unwrap();
        store.commit_turn("session", "legacy", "no usage").unwrap();
        let metadata = TurnMetadata {
            model: "fixture".into(),
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
        assert_eq!(version, 4);
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
