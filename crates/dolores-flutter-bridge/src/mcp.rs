use super::{Engine, Run};
use dolores_core::{McpConnection, McpLaunch};
use dolores_tools_mcp::Inspection;
use serde_json::{json, Value};
use std::{
    path::Path,
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};
use tokio::sync::mpsc;
use tokio_util::sync::CancellationToken;

const STALE: &str = "MCP review expired or changed. Inspect and review the server again.";
pub(super) struct McpReview {
    token: String,
    session: String,
    root: String,
    previous: Option<McpConnection>,
    inspection: Inspection,
    created: Instant,
}

impl Engine {
    pub(super) fn clear_mcp_review(&self) -> Result<(), String> {
        self.mcp_review.lock().map_err(|_| STALE)?.take();
        Ok(())
    }
    pub(super) fn discard_mcp_review(&self, token: &str) -> Result<Value, String> {
        let mut slot = self.mcp_review.lock().map_err(|_| STALE)?;
        if slot.as_ref().is_some_and(|r| r.token == token) {
            slot.take();
        }
        Ok(Value::Null)
    }
    fn mcp_root(&self, session: &str) -> Result<String, String> {
        self.store
            .workspace(session)?
            .root
            .ok_or("MCP tools need a project or temporary working chat.".into())
    }
    pub(super) fn mcp_settings(&self, session: &str) -> Result<Value, String> {
        self.clear_mcp_review()?;
        let root = self.mcp_root(session)?;
        Ok(json!({"connection":self.store.mcp_connection(&root)?,"directory":root}))
    }
    pub(super) fn inspect_mcp(
        &self,
        active: &mut Option<Run>,
        id: u64,
        session: String,
        launch: McpLaunch,
    ) -> Result<Value, String> {
        launch.validate()?;
        let root = self.mcp_root(&session)?;
        let previous = self.store.mcp_connection(&root)?;
        self.clear_mcp_review()?;
        let cancel = CancellationToken::new();
        let (output, events) = mpsc::channel(4);
        *active = Some(Run {
            id,
            cancel: cancel.clone(),
            events,
            approvals: Arc::new(Mutex::new(None)),
        });
        let store = self.store.clone();
        let slot = self.mcp_review.clone();
        self.runtime.spawn(async move {
            let result = async {
                let check_root = root.clone();
                let token_cancel = cancel.clone();
                let inspection = super::blocking(move || dolores_tools_mcp::inspect(Path::new(&check_root), launch, token_cancel)).await?;
                if store.workspace(&session)?.root.as_ref() != Some(&root) || store.mcp_connection(&root)? != previous { return Err(STALE.into()); }
                let token = uuid::Uuid::new_v4().to_string();
                let result = json!({"token":token,"tools":inspection.tools,"protocolVersion":inspection.protocol_version,"serverName":inspection.server_name,"serverVersion":inspection.server_version,"launch":inspection.launch});
                let mut review = slot.lock().map_err(|_| STALE)?;
                if cancel.is_cancelled() { return Err("MCP inspection stopped. Nothing was saved.".into()); }
                *review = Some(McpReview { token, session, root, previous, inspection, created: Instant::now() });
                Ok::<_, String>(result)
            }.await;
            let event = match result {
                Ok(inspection) => json!({"type":"done","id":id,"mcpInspection":inspection}),
                Err(error) => json!({"type":"done","id":id,"error":error}),
            };
            let _ = output.send(event).await;
        });
        Ok(Value::Null)
    }
    pub(super) fn enable_mcp(
        &self,
        session: &str,
        token: &str,
        names: Vec<String>,
    ) -> Result<Value, String> {
        let mut slot = self.mcp_review.lock().map_err(|_| STALE)?;
        let review = slot.as_ref().ok_or(STALE)?;
        if review.token != token
            || review.session != session
            || review.root != self.mcp_root(session)?
            || review.created.elapsed() > Duration::from_secs(300)
            || self.store.mcp_connection(&review.root)? != review.previous
        {
            return Err(STALE.into());
        }
        if names.is_empty()
            || names.len() > 2
            || names
                .iter()
                .enumerate()
                .any(|(i, name)| names[..i].contains(name))
        {
            return Err("Choose one or two reviewed MCP tools.".into());
        }
        let mut tools = vec![];
        for name in names {
            tools.push(
                review
                    .inspection
                    .tools
                    .iter()
                    .find(|t| t.name == name)
                    .ok_or("Choose a tool from the reviewed server list.")?
                    .clone(),
            );
        }
        tools.sort_by(|a, b| a.name.cmp(&b.name));
        dolores_tools_mcp::verify_launch(
            Path::new(&review.root),
            &review.inspection,
            &CancellationToken::new(),
        )?;
        if review.created.elapsed() > Duration::from_secs(300) {
            return Err(STALE.into());
        }
        let value = McpConnection {
            revision: 1,
            enabled: true,
            launch: review.inspection.launch.clone(),
            fingerprints: review.inspection.fingerprints.clone(),
            protocol_version: review.inspection.protocol_version.clone(),
            server_name: review.inspection.server_name.clone(),
            server_version: review.inspection.server_version.clone(),
            tools,
        };
        let saved = self.store.save_mcp_connection(
            &review.root,
            &value,
            review.previous.as_ref().map(|c| c.revision),
        )?;
        slot.take();
        Ok(json!(saved))
    }
    pub(super) fn mutate_mcp(
        &self,
        session: &str,
        revision: u32,
        forget: bool,
    ) -> Result<Value, String> {
        self.clear_mcp_review()?;
        self.store
            .mutate_mcp_connection(&self.mcp_root(session)?, revision, forget)?;
        Ok(Value::Null)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use dolores_core::{SessionStore, SessionWorkspace, WorkspaceKind};
    use dolores_store_sqlite::SqliteStore;
    #[test]
    fn review_scope_expiry_replay_and_failed_save_preserve_explicit_activation() {
        let d = tempfile::tempdir().unwrap();
        let store = Arc::new(SqliteStore::open(&d.path().join("fixture.db")).unwrap());
        let root = d
            .path()
            .canonicalize()
            .unwrap()
            .to_str()
            .unwrap()
            .to_owned();
        for id in ["work", "other"] {
            store
                .create_workspace_session(
                    id,
                    &SessionWorkspace {
                        kind: WorkspaceKind::Project,
                        root: Some(root.clone()),
                    },
                )
                .unwrap();
        }
        store
            .create_workspace_session("side", &SessionWorkspace::default())
            .unwrap();
        let engine = Engine::new(
            store.clone(),
            Arc::new(crate::connection::testing::MemoryCredentials::default()),
        )
        .unwrap();
        assert!(engine.mcp_settings("side").is_err());
        let name = if cfg!(windows) { "node.exe" } else { "node" };
        let exe = std::env::split_paths(&std::env::var_os("PATH").unwrap())
            .map(|p| p.join(name))
            .find(|p| p.is_file())
            .unwrap();
        let launch = McpLaunch {
            label: "Fixture".into(),
            executable: exe.to_str().unwrap().into(),
            args: vec![Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("../../scripts/mock-mcp.mjs")
                .to_str()
                .unwrap()
                .into()],
        };
        let inspection =
            dolores_tools_mcp::inspect(Path::new(&root), launch, CancellationToken::new()).unwrap();
        let reset = |created: Instant| {
            *engine.mcp_review.lock().unwrap() = Some(McpReview {
                token: "one".into(),
                session: "work".into(),
                root: root.clone(),
                previous: None,
                inspection: inspection.clone(),
                created,
            });
        };
        reset(Instant::now());
        assert!(engine
            .enable_mcp("other", "one", vec!["echo".into()])
            .is_err());
        assert!(engine
            .enable_mcp("work", "wrong", vec!["echo".into()])
            .is_err());
        assert!(engine
            .enable_mcp("work", "one", vec!["echo".into(), "echo".into()])
            .is_err());
        assert!(engine
            .enable_mcp("work", "one", vec!["missing".into()])
            .is_err());
        reset(Instant::now() - Duration::from_secs(301));
        assert!(engine
            .enable_mcp("work", "one", vec!["echo".into()])
            .is_err());
        reset(Instant::now());
        // Write failure keeps the reviewed token usable for an explicit retry.
        let db = rusqlite::Connection::open(d.path().join("fixture.db")).unwrap();
        db.execute_batch("CREATE TRIGGER refuse_mcp BEFORE INSERT ON mcp_connections BEGIN SELECT RAISE(ABORT,'fixture failure'); END;").unwrap();
        assert!(engine
            .enable_mcp("work", "one", vec!["echo".into()])
            .is_err());
        assert!(engine.mcp_review.lock().unwrap().is_some());
        assert!(store.mcp_connection(&root).unwrap().is_none());
        db.execute_batch("DROP TRIGGER refuse_mcp").unwrap();
        let saved = engine
            .enable_mcp("work", "one", vec!["echo".into()])
            .unwrap();
        assert_eq!(saved["enabled"], true);
        assert!(engine
            .enable_mcp("work", "one", vec!["echo".into()])
            .is_err());
        engine.mutate_mcp("work", 1, false).unwrap();
        assert!(!store.mcp_connection(&root).unwrap().unwrap().enabled);
        reset(Instant::now());
        assert!(engine
            .enable_mcp("work", "one", vec!["echo".into()])
            .is_err());
        engine.discard_mcp_review("different").unwrap();
        assert!(engine.mcp_review.lock().unwrap().is_some());
        engine.discard_mcp_review("one").unwrap();
        assert!(engine.mcp_review.lock().unwrap().is_none());
    }
}
