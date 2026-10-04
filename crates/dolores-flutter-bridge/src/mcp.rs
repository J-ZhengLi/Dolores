use super::{Engine, Run};
use dolores_core::{McpConnection, McpCredentialBinding, McpLaunch};
use dolores_tools_mcp::credentials::{self, CredentialInput, Credentials};
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
    connection_id: String,
    inspection: Inspection,
    credentials: Credentials,
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
        Ok(
            json!({"connection":self.store.mcp_connection(&root)?,"connections":self.store.mcp_connections(&root)?,"directory":root,"maxConnections":dolores_core::MAX_MCP_CONNECTIONS,"maxActiveTools":dolores_core::MAX_ACTIVE_MCP_TOOLS}),
        )
    }
    pub(super) fn inspect_mcp(
        &self,
        active: &mut crate::run_journal::RunCoordinator,
        id: u64,
        session: String,
        connection_id: String,
        launch: McpLaunch,
        inputs: Vec<CredentialInput>,
    ) -> Result<Value, String> {
        launch.validate()?;
        let root = self.mcp_root(&session)?;
        let connection_id = if connection_id.is_empty() {
            uuid::Uuid::new_v4().to_string()
        } else {
            connection_id
        };
        if !dolores_core::valid_mcp_id(&connection_id) {
            return Err("MCP connection identity is invalid.".into());
        }
        let all = self.store.mcp_connections(&root)?;
        let previous = all.iter().find(|c| c.id == connection_id).cloned();
        if previous.is_none() && all.len() >= dolores_core::MAX_MCP_CONNECTIONS {
            return Err("This folder already has four MCP connections. Forget a saved server before adding another.".into());
        }
        self.clear_mcp_review()?;
        let cancel = CancellationToken::new();
        let (output, events) = mpsc::channel(4);
        active.reserve(Run {
            thread: None,
            id,
            cancel: cancel.clone(),
            events,
            approvals: Arc::new(Mutex::new(None)),
        })?;
        let store = self.store.clone();
        let slot = self.mcp_review.clone();
        let vault = self.mcp_credentials.clone();
        self.runtime.spawn(async move {
            let result = async {
                let check_root = root.clone();
                let token_cancel = cancel.clone();
                let saved = previous.clone();
                let (inspection, credentials) = super::blocking(move || {
                    dolores_core::validate_mcp_credential_names(&inputs.iter().map(|i| i.name.clone()).collect::<Vec<_>>())?;
                    let reuse = inputs.iter().filter(|i| i.value.is_none()).map(|i| i.name.clone()).collect::<Vec<_>>();
                    let existing = if reuse.is_empty() { Credentials::empty() } else {
                        let mut saved = saved.ok_or(credentials::VAULT_ERROR)?;
                        if saved.launch != launch { return Err("MCP launch changed. Enter the key again before inspecting this program.".into()); }
                        dolores_tools_mcp::verify_launch(Path::new(&check_root), &inspection_for(&saved), &token_cancel)?;
                        saved.credentials.retain(|b| reuse.contains(&b.name));
                        credentials::resolve(Path::new(&check_root), &saved, vault.as_ref())?
                    };
                    let mut values = vec![];
                    for input in inputs {
                        let value = match input.value {
                            Some(value) => value,
                            None => existing.value(&input.name).ok_or(credentials::VAULT_ERROR)?,
                        };
                        values.push((input.name, value));
                    }
                    let credentials = Credentials::new(values)?;
                    let inspection = dolores_tools_mcp::inspect_with_credentials(Path::new(&check_root), launch, token_cancel, credentials.clone())?;
                    Ok::<_, String>((inspection, credentials))
                }).await?;
                if store.workspace(&session)?.root.as_ref() != Some(&root) || store.mcp_connection_by_id(&root, &connection_id)? != previous { return Err(STALE.into()); }
                let token = uuid::Uuid::new_v4().to_string();
                let result = json!({"token":token,"tools":inspection.tools,"protocolVersion":inspection.protocol_version,"serverName":inspection.server_name,"serverVersion":inspection.server_version,"launch":inspection.launch,"credentialNames":credentials.names(),"connectionId":connection_id});
                let mut review = slot.lock().map_err(|_| STALE)?;
                if cancel.is_cancelled() { return Err("MCP inspection stopped. Nothing was saved.".into()); }
                *review = Some(McpReview { token, session, root, previous, connection_id, inspection, credentials, created: Instant::now() });
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
            || self
                .store
                .mcp_connection_by_id(&review.root, &review.connection_id)?
                != review.previous
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
        let proposed = McpConnection {
            id: review.connection_id.clone(),
            revision: 1,
            enabled: true,
            launch: review.inspection.launch.clone(),
            fingerprints: review.inspection.fingerprints.clone(),
            protocol_version: review.inspection.protocol_version.clone(),
            server_name: review.inspection.server_name.clone(),
            server_version: review.inspection.server_version.clone(),
            tools: tools.clone(),
            credentials: vec![],
            retired_credentials: vec![],
        };
        dolores_core::check_mcp_capacity(&self.store.mcp_connections(&review.root)?, &proposed)?;
        // Keep old opaque references until Forget; cleanup failures remain retryable.
        let mut retired = review
            .previous
            .as_ref()
            .map(|p| p.retired_credentials.clone())
            .unwrap_or_default();
        retired.retain(|id| self.mcp_credentials.delete(id).is_err());
        if let Some(previous) = &review.previous {
            retired.extend(previous.credentials.iter().map(|b| b.credential_id.clone()));
        }
        retired.sort();
        retired.dedup();
        if retired.len() > 32 {
            return Err("MCP credential cleanup is pending. Unlock secure storage and retry, or Forget this connection first.".into());
        }
        let mut prepared = vec![];
        for name in review.credentials.names() {
            let id = uuid::Uuid::new_v4().to_string();
            let encoded = review.credentials.encoded_for(
                &review.connection_id,
                &name,
                Path::new(&review.root),
                &review.inspection.launch,
                &review.inspection.fingerprints,
            )?;
            prepared.push((
                McpCredentialBinding {
                    name,
                    credential_id: id,
                },
                encoded,
            ));
        }
        let mut bindings: Vec<McpCredentialBinding> = vec![];
        for (binding, encoded) in prepared {
            let id = &binding.credential_id;
            if self.mcp_credentials.write(id, &encoded).is_err() {
                let mut cleanup_failed = self.mcp_credentials.delete(id).is_err();
                for binding in &bindings {
                    cleanup_failed |= self.mcp_credentials.delete(&binding.credential_id).is_err();
                }
                if cleanup_failed {
                    return Err("MCP credentials were not saved and secure storage cleanup failed. Unlock storage before retrying; unused vault entries may remain.".into());
                }
                return Err(credentials::VAULT_ERROR.into());
            }
            bindings.push(binding);
        }
        let value = McpConnection {
            id: review.connection_id.clone(),
            revision: 1,
            enabled: true,
            launch: review.inspection.launch.clone(),
            fingerprints: review.inspection.fingerprints.clone(),
            protocol_version: review.inspection.protocol_version.clone(),
            server_name: review.inspection.server_name.clone(),
            server_version: review.inspection.server_version.clone(),
            tools,
            credentials: bindings,
            retired_credentials: retired,
        };
        let persistence = if review.created.elapsed() > Duration::from_secs(300) {
            Err(STALE.into())
        } else {
            self.store.save_mcp_connection(
                &review.root,
                &value,
                review.previous.as_ref().map(|c| c.revision),
            )
        };
        let saved = match persistence {
            Ok(saved) => saved,
            Err(error) => {
                let failed = value
                    .credentials
                    .iter()
                    .filter(|b| self.mcp_credentials.delete(&b.credential_id).is_err())
                    .count();
                if failed > 0 {
                    return Err("MCP configuration was not saved and secure storage cleanup failed. Unlock storage before retrying; unused vault entries may remain.".into());
                }
                return Err(error);
            }
        };
        let mut cleanup_pending = false;
        for id in &saved.retired_credentials {
            cleanup_pending |= self.mcp_credentials.delete(id).is_err();
        }
        slot.take();
        let mut result = json!(saved);
        if cleanup_pending {
            result["warning"] = json!("Tools enabled, but old credential cleanup is pending. Unlock storage and use Forget or inspect and enable again to retry.");
        }
        Ok(result)
    }
    pub(super) fn mutate_mcp(
        &self,
        session: &str,
        connection_id: &str,
        revision: u32,
        forget: bool,
    ) -> Result<Value, String> {
        let mut review = self.mcp_review.lock().map_err(|_| STALE)?;
        if review
            .as_ref()
            .is_some_and(|r| r.connection_id == connection_id)
        {
            review.take();
        }
        drop(review);
        let root = self.mcp_root(session)?;
        let saved = self
            .store
            .mcp_connection_by_id(&root, connection_id)?
            .ok_or("MCP connection is missing.")?;
        if saved.revision != revision {
            return Err(STALE.into());
        }
        if forget && (!saved.credentials.is_empty() || !saved.retired_credentials.is_empty()) {
            // Disable first, retaining references until every vault deletion succeeds.
            let disabled = if saved.enabled {
                self.store
                    .mutate_mcp_connection_by_id(&root, connection_id, revision, false)?;
                self.store
                    .mcp_connection_by_id(&root, connection_id)?
                    .ok_or(STALE)?
            } else {
                saved
            };
            let mut failed = false;
            for id in disabled
                .credentials
                .iter()
                .map(|b| &b.credential_id)
                .chain(disabled.retired_credentials.iter())
            {
                failed |= self.mcp_credentials.delete(id).is_err();
            }
            if failed {
                return Err("MCP tools are disabled, but credential removal failed. Unlock secure storage and press Forget again.".into());
            }
            self.store.mutate_mcp_connection_by_id(
                &root,
                connection_id,
                disabled.revision,
                true,
            )?;
        } else {
            self.store
                .mutate_mcp_connection_by_id(&root, connection_id, revision, forget)?;
        }
        Ok(json!({"connections":self.store.mcp_connections(&root)?}))
    }
}

fn inspection_for(connection: &McpConnection) -> Inspection {
    Inspection {
        launch: connection.launch.clone(),
        fingerprints: connection.fingerprints.clone(),
        protocol_version: connection.protocol_version.clone(),
        server_name: connection.server_name.clone(),
        server_version: connection.server_version.clone(),
        tools: connection.tools.clone(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use dolores_core::{SessionStore, SessionWorkspace, WorkspaceKind};
    use dolores_store_sqlite::SqliteStore;
    #[test]
    fn desktop_json_routes_named_connections_and_defaults_legacy_requests() {
        for name in ["inspectMcp", "disableMcp", "forgetMcp"] {
            for identity in [None, Some(""), Some("63ed0154-e98f-4547-91cb-184971cdb922")] {
                let mut request = json!({"command":name,"session":"work","id":1,"revision":1,
                    "launch":{"label":"Fixture","executable":"/fixture/node","args":[]}});
                if let Some(id) = identity {
                    request["connectionId"] = json!(id);
                }
                let command: crate::Command = serde_json::from_value(request).unwrap();
                let actual = match command {
                    crate::Command::InspectMcp { connection_id, .. }
                    | crate::Command::DisableMcp { connection_id, .. }
                    | crate::Command::ForgetMcp { connection_id, .. } => connection_id,
                    _ => unreachable!(),
                };
                assert_eq!(actual, identity.unwrap_or("legacy"));
            }
        }
    }
    #[test]
    fn credential_rotation_failed_save_and_locked_forget_are_recoverable() {
        use std::sync::atomic::Ordering::Relaxed;
        let folder = tempfile::tempdir().unwrap();
        let root = folder
            .path()
            .canonicalize()
            .unwrap()
            .to_str()
            .unwrap()
            .to_owned();
        let store = Arc::new(SqliteStore::open(&folder.path().join("fixture.db")).unwrap());
        store
            .create_workspace_session(
                "work",
                &SessionWorkspace {
                    kind: WorkspaceKind::Project,
                    root: Some(root.clone()),
                },
            )
            .unwrap();
        let vault = Arc::new(crate::connection::testing::MemoryCredentials::default());
        let engine = Engine::new(store.clone(), vault.clone()).unwrap();
        let filename = if cfg!(windows) { "node.exe" } else { "node" };
        let exe = std::env::split_paths(&std::env::var_os("PATH").unwrap())
            .map(|p| p.join(filename))
            .find(|p| p.is_file())
            .unwrap();
        let launch = McpLaunch {
            label: "Fixture".into(),
            executable: exe.to_str().unwrap().into(),
            args: vec![
                Path::new(env!("CARGO_MANIFEST_DIR"))
                    .join("../../scripts/mock-mcp.mjs")
                    .to_str()
                    .unwrap()
                    .into(),
                "--mode=credential-echo".into(),
            ],
        };
        let credentials = Credentials::new(vec![(
            "DOLORES_MCP_TEST_TOKEN".into(),
            "synthetic-bridge-credential".into(),
        )])
        .unwrap();
        let inspection = dolores_tools_mcp::inspect_with_credentials(
            Path::new(&root),
            launch,
            CancellationToken::new(),
            credentials.clone(),
        )
        .unwrap();
        let review = |previous| {
            *engine.mcp_review.lock().unwrap() = Some(McpReview {
                token: "review".into(),
                connection_id: "legacy".into(),
                session: "work".into(),
                root: root.clone(),
                previous,
                inspection: inspection.clone(),
                credentials: credentials.clone(),
                created: Instant::now(),
            });
        };
        review(None);
        vault.locked.store(true, Relaxed);
        assert!(engine
            .enable_mcp("work", "review", vec!["echo".into()])
            .is_err());
        assert!(store.mcp_connection(&root).unwrap().is_none());
        vault.locked.store(false, Relaxed);
        let db = rusqlite::Connection::open(folder.path().join("fixture.db")).unwrap();
        db.execute_batch("CREATE TRIGGER refuse_mcp BEFORE INSERT ON mcp_connections BEGIN SELECT RAISE(ABORT,'fixture'); END;").unwrap();
        assert!(engine
            .enable_mcp("work", "review", vec!["echo".into()])
            .is_err());
        assert!(vault.values.lock().unwrap().is_empty());
        assert!(engine.mcp_review.lock().unwrap().is_some());
        db.execute_batch("DROP TRIGGER refuse_mcp").unwrap();
        engine
            .enable_mcp("work", "review", vec!["echo".into()])
            .unwrap();
        let first = store.mcp_connection(&root).unwrap().unwrap();
        assert_eq!(first.credentials.len(), 1);
        let raw: String = db
            .query_row("SELECT data FROM mcp_connections", [], |row| row.get(0))
            .unwrap();
        assert!(!raw.contains("synthetic-bridge-credential"));
        review(Some(first.clone()));
        engine
            .enable_mcp("work", "review", vec!["echo".into()])
            .unwrap();
        let rotated = store.mcp_connection(&root).unwrap().unwrap();
        assert_ne!(first.credentials, rotated.credentials);
        assert!(!vault
            .values
            .lock()
            .unwrap()
            .contains_key(&first.credentials[0].credential_id));
        assert_eq!(vault.values.lock().unwrap().len(), 1);
        vault.locked.store(true, Relaxed);
        let error = engine
            .mutate_mcp("work", "legacy", rotated.revision, true)
            .unwrap_err();
        assert!(error.contains("disabled") && error.contains("Forget again"));
        let disabled = store.mcp_connection(&root).unwrap().unwrap();
        assert!(!disabled.enabled && !disabled.credentials.is_empty());
        assert!(engine
            .mutate_mcp("work", "legacy", rotated.revision, true)
            .is_err());
        vault.locked.store(false, Relaxed);
        engine
            .mutate_mcp("work", "legacy", disabled.revision, true)
            .unwrap();
        assert!(store.mcp_connection(&root).unwrap().is_none());
        assert!(vault.values.lock().unwrap().is_empty());
    }
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
                connection_id: "legacy".into(),
                session: "work".into(),
                root: root.clone(),
                previous: None,
                inspection: inspection.clone(),
                credentials: Credentials::empty(),
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
        engine.mutate_mcp("work", "legacy", 1, false).unwrap();
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
