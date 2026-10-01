use dolores_core::{
    ConnectionPreferences, Message, PluginDescriptor, Role, Session, SessionStore, HISTORY_LIMIT,
};
use rusqlite::{params, Connection, OptionalExtension};
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
            CREATE TABLE IF NOT EXISTS preferences (id INTEGER PRIMARY KEY CHECK(id=1), base_url TEXT NOT NULL, model TEXT NOT NULL);
            PRAGMA user_version = 1;").map_err(storage_error)?;
        Ok(Self {
            connection: Mutex::new(connection),
        })
    }
    fn lock(&self) -> Result<MutexGuard<'_, Connection>, String> {
        self.connection.lock().map_err(storage_error)
    }
}

impl SessionStore for SqliteStore {
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
        let connection = self.lock()?;
        let exists: bool = connection
            .query_row(
                "SELECT EXISTS(SELECT 1 FROM sessions WHERE id=?1)",
                [id],
                |row| row.get(0),
            )
            .map_err(storage_error)?;
        if !exists {
            return Err("Conversation no longer exists.".into());
        }
        let mut statement = connection.prepare("SELECT role,content FROM (SELECT id,role,content FROM messages WHERE session_id=?1 ORDER BY id DESC LIMIT ?2) ORDER BY id ASC").map_err(storage_error)?;
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
        rows.collect::<Result<Vec<_>, _>>().map_err(storage_error)
    }
    fn delete(&self, id: &str) -> Result<(), String> {
        self.lock()?
            .execute("DELETE FROM sessions WHERE id=?1", [id])
            .map_err(storage_error)?;
        Ok(())
    }
    fn commit_turn(&self, id: &str, user: &str, assistant: &str) -> Result<(), String> {
        let mut connection = self.lock()?;
        let transaction = connection.transaction().map_err(storage_error)?;
        transaction.execute("INSERT INTO messages(session_id,role,content) VALUES(?1,'user',?2),(?1,'assistant',?3)", params![id,user,assistant]).map_err(storage_error)?;
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
    fn preferences(&self) -> Result<ConnectionPreferences, String> {
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
    fn save_preferences(&self, preferences: &ConnectionPreferences) -> Result<(), String> {
        self.lock()?.execute("INSERT INTO preferences(id,base_url,model) VALUES(1,?1,?2) ON CONFLICT(id) DO UPDATE SET base_url=excluded.base_url,model=excluded.model", params![preferences.base_url,preferences.model]).map_err(storage_error)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
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
