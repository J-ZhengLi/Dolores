use super::{storage_error, SqliteStore};
use dolores_core::McpConnection;
use rusqlite::params;

fn root_valid(root: &str) -> Result<(), String> {
    if !std::path::Path::new(root).is_absolute() || root.contains('\0') {
        return Err("MCP connections need an absolute working folder.".into());
    }
    Ok(())
}

impl SqliteStore {
    pub(super) fn read_mcps(&self, root: &str) -> Result<Vec<McpConnection>, String> {
        root_valid(root)?;
        let db = self.lock()?;
        read_connections(&db, root)
    }
    pub(super) fn read_mcp(&self, root: &str) -> Result<Option<McpConnection>, String> {
        Ok(self.read_mcps(root)?.into_iter().find(|c| c.id == "legacy"))
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
        let connections = read_connections(&tx, root)?;
        let previous = connections.iter().find(|c| c.id == connection.id);
        if previous.map(|c| c.revision) != expected {
            return Err("MCP connection changed. Inspect and review it again.".into());
        }
        let mut value = connection.clone();
        value.revision = expected
            .unwrap_or(0)
            .checked_add(1)
            .ok_or("MCP revision limit reached.")?;
        value.enabled = true;
        dolores_core::check_mcp_capacity(&connections, &value)?;
        value.validate()?;
        tx.execute("INSERT INTO mcp_connections(root,id,data) VALUES(?1,?2,?3) ON CONFLICT(root,id) DO UPDATE SET data=excluded.data", params![root, value.id, serde_json::to_string(&value).map_err(storage_error)?]).map_err(storage_error)?;
        tx.commit().map_err(storage_error)?;
        Ok(value)
    }
    pub(super) fn change_mcp(
        &self,
        root: &str,
        id: &str,
        revision: u32,
        forget: bool,
    ) -> Result<(), String> {
        root_valid(root)?;
        if !dolores_core::valid_mcp_id(id) {
            return Err("MCP connection identity is invalid.".into());
        }
        let mut db = self.lock()?;
        let tx = db.transaction().map_err(storage_error)?;
        let connections = read_connections(&tx, root)?;
        let mut value = connections
            .into_iter()
            .find(|c| c.id == id)
            .ok_or("MCP connection is unavailable. Refresh.")?;
        if value.revision != revision {
            return Err("MCP connection changed. Refresh.".into());
        }
        if forget {
            tx.execute(
                "DELETE FROM mcp_connections WHERE root=?1 AND id=?2",
                params![root, id],
            )
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
                "UPDATE mcp_connections SET data=?3 WHERE root=?1 AND id=?2",
                params![
                    root,
                    id,
                    serde_json::to_string(&value).map_err(storage_error)?
                ],
            )
            .map_err(storage_error)?;
        }
        tx.commit().map_err(storage_error)?;
        Ok(())
    }
}
fn read_connections(db: &rusqlite::Connection, root: &str) -> Result<Vec<McpConnection>, String> {
    let mut statement = db
        .prepare("SELECT id,data FROM mcp_connections WHERE root=?1 ORDER BY id LIMIT 5")
        .map_err(storage_error)?;
    let rows = statement
        .query_map([root], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
        })
        .map_err(storage_error)?;
    let mut result = vec![];
    for row in rows {
        let (id, text) = row.map_err(storage_error)?;
        if text.len() > 96 * 1024 {
            return Err("Saved MCP connection exceeds its limit.".into());
        }
        let value: McpConnection = serde_json::from_str(&text).map_err(storage_error)?;
        value.validate()?;
        if value.id != id {
            return Err(
                "Saved MCP identity does not match its record. Review local data recovery.".into(),
            );
        }
        result.push(value);
    }
    if result.len() > dolores_core::MAX_MCP_CONNECTIONS
        || result
            .iter()
            .filter(|c| c.enabled)
            .map(|c| c.tools.len())
            .sum::<usize>()
            > dolores_core::MAX_ACTIVE_MCP_TOOLS
    {
        return Err(
            "Saved MCP connections exceed the folder limits. Review local data recovery.".into(),
        );
    }
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    use dolores_core::{McpFingerprint, McpLaunch, McpTool, SessionStore};
    use serde_json::json;
    #[test]
    fn migration_preserves_legacy_configuration_and_credential_references() {
        let d = tempfile::tempdir().unwrap();
        let file = d.path().join("old.db");
        let root = d.path().join("project");
        let mut original = value(&root);
        original.credentials = vec![dolores_core::McpCredentialBinding {
            name: "SERVICE_API_KEY".into(),
            credential_id: "00000000-0000-4000-8000-000000000001".into(),
        }];
        original.retired_credentials = vec!["00000000-0000-4000-8000-000000000002".into()];
        let text = serde_json::to_string(&original).unwrap();
        let old = rusqlite::Connection::open(&file).unwrap();
        old.execute_batch("CREATE TABLE mcp_connections(root TEXT PRIMARY KEY,data TEXT NOT NULL); PRAGMA user_version=15;").unwrap();
        old.execute(
            "INSERT INTO mcp_connections VALUES(?1,?2)",
            params![root.to_str().unwrap(), text],
        )
        .unwrap();
        drop(old);
        let store = SqliteStore::open(&file).unwrap();
        assert_eq!(
            store.mcp_connections(root.to_str().unwrap()).unwrap(),
            vec![original]
        );
        let retained: String = store
            .lock()
            .unwrap()
            .query_row(
                "SELECT data FROM mcp_connections WHERE id='legacy'",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(retained, text); // No serialization rewrite or changed legacy aliases.
        drop(store);
        let reopened = SqliteStore::open(&file).unwrap();
        assert_eq!(
            reopened
                .mcp_connections(root.to_str().unwrap())
                .unwrap()
                .len(),
            1
        );
    }
    #[test]
    fn shared_tool_budget_refusal_is_atomic_and_independent_revisions_are_retained() {
        let d = tempfile::tempdir().unwrap();
        let root = d.path().to_str().unwrap();
        let store = SqliteStore::open(&d.path().join("fixture.db")).unwrap();
        let mut a = value(d.path());
        a.id = "00000000-0000-4000-8000-000000000001".into();
        let mut other = a.tools[0].clone();
        other.name = "second".into();
        a.tools.push(other);
        let a = store.save_mcp_connection(root, &a, None).unwrap();
        let mut b = value(d.path());
        b.id = "00000000-0000-4000-8000-000000000002".into();
        let refusal = store.save_mcp_connection(root, &b, None).unwrap_err();
        assert!(refusal.contains("select fewer") && refusal.contains("review is preserved"));
        assert_eq!(store.mcp_connections(root).unwrap(), vec![a.clone()]);
        store
            .mutate_mcp_connection_by_id(root, &a.id, 1, false)
            .unwrap();
        let b = store.save_mcp_connection(root, &b, None).unwrap();
        let mut c = value(d.path());
        c.id = "00000000-0000-4000-8000-000000000003".into();
        store.save_mcp_connection(root, &c, None).unwrap();
        assert_eq!(
            store.mcp_connection_by_id(root, &b.id).unwrap(),
            Some(b.clone())
        );
        store
            .mutate_mcp_connection_by_id(root, &c.id, 1, true)
            .unwrap();
        assert_eq!(
            store.mcp_connection_by_id(root, &b.id).unwrap(),
            Some(b.clone())
        );
        assert!(store
            .mutate_mcp_connection_by_id(root, &a.id, 1, true)
            .is_err());
        assert_eq!(store.mcp_connection_by_id(root, &b.id).unwrap(), Some(b));
        for n in 3..=4 {
            let mut v = value(d.path());
            v.id = format!("00000000-0000-4000-8000-{n:012}");
            store.save_mcp_connection(root, &v, None).unwrap();
            store
                .mutate_mcp_connection_by_id(root, &v.id, 1, false)
                .unwrap();
        }
        let mut fifth = value(d.path());
        fifth.id = "00000000-0000-4000-8000-000000000005".into();
        assert!(store
            .save_mcp_connection(root, &fifth, None)
            .unwrap_err()
            .contains("four MCP connections"));
        assert_eq!(store.mcp_connections(root).unwrap().len(), 4);
    }
    fn value(root: &std::path::Path) -> McpConnection {
        let exe = root.join("server.exe").to_str().unwrap().to_owned();
        McpConnection {
            id: "legacy".into(),
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
            21
        );
    }
}
