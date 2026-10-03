use super::{storage_error, SqliteStore};
use dolores_core::McpConnection;
use rusqlite::{params, OptionalExtension};

fn root_valid(root: &str) -> Result<(), String> {
    if !std::path::Path::new(root).is_absolute() || root.contains('\0') {
        return Err("MCP connections need an absolute working folder.".into());
    }
    Ok(())
}

impl SqliteStore {
    pub(super) fn read_mcp(&self, root: &str) -> Result<Option<McpConnection>, String> {
        root_valid(root)?;
        let text: Option<String> = self
            .lock()?
            .query_row(
                "SELECT data FROM mcp_connections WHERE root=?1",
                [root],
                |row| row.get(0),
            )
            .optional()
            .map_err(storage_error)?;
        text.map(|text| {
            if text.len() > 96 * 1024 {
                return Err("Saved MCP connection exceeds its limit.".into());
            }
            let value: McpConnection = serde_json::from_str(&text).map_err(storage_error)?;
            value.validate()?;
            Ok(value)
        })
        .transpose()
    }
    pub(super) fn write_mcp(
        &self,
        root: &str,
        connection: &McpConnection,
        expected: Option<u32>,
    ) -> Result<McpConnection, String> {
        root_valid(root)?;
        connection.validate()?;
        let mut db = self.lock()?;
        let tx = db.transaction().map_err(storage_error)?;
        let previous: Option<String> = tx
            .query_row(
                "SELECT data FROM mcp_connections WHERE root=?1",
                [root],
                |row| row.get(0),
            )
            .optional()
            .map_err(storage_error)?;
        let previous: Option<McpConnection> = previous
            .map(|s| serde_json::from_str(&s).map_err(storage_error))
            .transpose()?;
        if previous.as_ref().map(|c| c.revision) != expected {
            return Err("MCP connection changed. Inspect and review it again.".into());
        }
        let mut value = connection.clone();
        value.revision = expected
            .unwrap_or(0)
            .checked_add(1)
            .ok_or("MCP revision limit reached.")?;
        value.enabled = true;
        value.validate()?;
        tx.execute("INSERT INTO mcp_connections(root,data) VALUES(?1,?2) ON CONFLICT(root) DO UPDATE SET data=excluded.data", params![root, serde_json::to_string(&value).map_err(storage_error)?]).map_err(storage_error)?;
        tx.commit().map_err(storage_error)?;
        Ok(value)
    }
    pub(super) fn change_mcp(&self, root: &str, revision: u32, forget: bool) -> Result<(), String> {
        root_valid(root)?;
        let mut db = self.lock()?;
        let tx = db.transaction().map_err(storage_error)?;
        let text: String = tx
            .query_row(
                "SELECT data FROM mcp_connections WHERE root=?1",
                [root],
                |row| row.get(0),
            )
            .optional()
            .map_err(storage_error)?
            .ok_or("MCP connection is unavailable. Refresh.")?;
        let mut value: McpConnection = serde_json::from_str(&text).map_err(storage_error)?;
        value.validate()?;
        if value.revision != revision {
            return Err("MCP connection changed. Refresh.".into());
        }
        if forget {
            tx.execute("DELETE FROM mcp_connections WHERE root=?1", [root])
                .map_err(storage_error)?;
        } else {
            if !value.enabled {
                return Err("MCP connection is already disabled.".into());
            }
            value.enabled = false;
            value.revision = value
                .revision
                .checked_add(1)
                .ok_or("MCP revision limit reached.")?;
            tx.execute(
                "UPDATE mcp_connections SET data=?2 WHERE root=?1",
                params![root, serde_json::to_string(&value).map_err(storage_error)?],
            )
            .map_err(storage_error)?;
        }
        tx.commit().map_err(storage_error)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use dolores_core::{McpFingerprint, McpLaunch, McpTool, SessionStore};
    use serde_json::json;
    fn value(root: &std::path::Path) -> McpConnection {
        let exe = root.join("server.exe").to_str().unwrap().to_owned();
        McpConnection {
            credentials: vec![],
            retired_credentials: vec![],
            revision: 1,
            enabled: true,
            launch: McpLaunch {
                label: "Fixture".into(),
                executable: exe.clone(),
                args: vec![],
            },
            fingerprints: vec![McpFingerprint {
                path: exe,
                sha256: "a".repeat(64),
            }],
            protocol_version: "2025-11-25".into(),
            server_name: "Fixture".into(),
            server_version: "1".into(),
            tools: vec![McpTool {
                name: "echo".into(),
                description: "Echo".into(),
                input_schema: json!({"type":"object"}),
            }],
        }
    }
    #[test]
    fn connection_cas_disable_forget_scope_restart_and_atomic_failure() {
        let d = tempfile::tempdir().unwrap();
        let file = d.path().join("store.db");
        let root = d.path().join("project");
        let key = root.to_str().unwrap();
        let store = SqliteStore::open(&file).unwrap();
        assert!(store.mcp_connection(key).unwrap().is_none());
        let one = store.save_mcp_connection(key, &value(&root), None).unwrap();
        assert_eq!(one.revision, 1);
        assert!(store.save_mcp_connection(key, &one, None).is_err());
        assert!(store.mutate_mcp_connection(key, 2, true).is_err());
        store.mutate_mcp_connection(key, 1, false).unwrap();
        let disabled = store.mcp_connection(key).unwrap().unwrap();
        assert!(!disabled.enabled);
        assert_eq!(disabled.revision, 2);
        assert!(disabled.specs().is_empty());
        assert!(store
            .mcp_connection(d.path().join("other").to_str().unwrap())
            .unwrap()
            .is_none());
        store.lock().unwrap().execute_batch("CREATE TRIGGER refuse_mcp BEFORE UPDATE ON mcp_connections BEGIN SELECT RAISE(ABORT,'synthetic failure'); END;").unwrap();
        assert!(store.save_mcp_connection(key, &one, Some(2)).is_err());
        assert_eq!(store.mcp_connection(key).unwrap(), Some(disabled.clone()));
        store
            .lock()
            .unwrap()
            .execute_batch("DROP TRIGGER refuse_mcp")
            .unwrap();
        let three = store.save_mcp_connection(key, &one, Some(2)).unwrap();
        assert_eq!(three.revision, 3);
        drop(store);
        let store = SqliteStore::open(&file).unwrap();
        assert_eq!(store.mcp_connection(key).unwrap(), Some(three));
        store.mutate_mcp_connection(key, 3, true).unwrap();
        assert!(store.mcp_connection(key).unwrap().is_none());
        assert_eq!(
            store
                .lock()
                .unwrap()
                .query_row("PRAGMA user_version", [], |row| row.get::<_, i64>(0))
                .unwrap(),
            15
        );
    }
}
