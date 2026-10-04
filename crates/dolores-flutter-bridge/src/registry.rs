use crate::Engine;
use dolores_core::{
    ExtensionDescriptor, ExtensionKind, ExtensionRegistration, ExtensionRegistry, RegistrySnapshot,
    ToolProposalHook, ToolRequest,
};
use std::sync::Arc;
struct ValidateProposal;
impl ToolProposalHook for ValidateProposal {
    fn transform(&self, request: &ToolRequest) -> Result<ToolRequest, String> {
        if request.call_id.is_empty()
            || request.name.is_empty()
            || request.target.len() > 1024
            || request.query.as_ref().is_some_and(|q| {
                q.len()
                    > if request.name == "delegate_tasks" {
                        4096
                    } else {
                        256
                    }
                    || q.chars().any(char::is_control)
            })
            || request
                .diff
                .as_ref()
                .is_some_and(|d| d.len() > dolores_core::MAX_TOOL_BYTES)
        {
            return Err("Invalid tool proposal.".into());
        }
        Ok(request.clone())
    }
}
impl Engine {
    pub(super) fn extension_registry(
        &self,
        session: Option<&str>,
    ) -> Result<Arc<RegistrySnapshot>, String> {
        let workspace = session.map(|s| self.store.workspace(s)).transpose()?;
        let root = workspace.and_then(|w| w.root);
        let working = root.is_some();
        let configured = self
            .connection
            .lock()
            .map_err(|_| "Connection unavailable.")?
            .provider
            .is_some();
        let mut registry = ExtensionRegistry::default();
        let mut add = |id: &str,
                       enabled: bool,
                       available: bool,
                       capabilities: Vec<String>,
                       tools: Vec<String>| {
            registry.register(ExtensionRegistration {
                descriptor: ExtensionDescriptor {
                    id: id.into(),
                    version: env!("CARGO_PKG_VERSION").into(),
                    api_min: 1,
                    api_max: 1,
                    kind: ExtensionKind::Compiled,
                    config_revision: 1,
                    entry_identity: env!("DOLORES_BUILD_REVISION").into(),
                    dependencies: vec![],
                    capabilities,
                    tools,
                    enabled,
                    available,
                    unavailable_reason: if available {
                        ""
                    } else if id == "provider/openai" {
                        "Connect a model"
                    } else {
                        "Start a project or temporary working chat"
                    }
                    .into(),
                    health: if available {
                        "compiled; no dynamic loader"
                    } else {
                        "unavailable"
                    }
                    .into(),
                },
                hooks: if id == "host/proposal" {
                    vec![Arc::new(ValidateProposal)]
                } else {
                    vec![]
                },
            })
        };
        add(
            "storage/sqlite",
            true,
            true,
            vec!["local-history".into()],
            vec![],
        )?;
        add(
            "credentials/os",
            true,
            true,
            vec!["credential-storage".into()],
            vec![],
        )?;
        add(
            "provider/openai",
            true,
            configured,
            vec!["network-model".into()],
            vec![],
        )?;
        add(
            "context/default",
            true,
            true,
            vec!["context-preparation".into()],
            vec![],
        )?;
        add(
            "learning/preferences",
            self.store.automatic_memory_policy()?.enabled,
            configured,
            vec!["preference-learning".into()],
            vec![],
        )?;
        add(
            "host/proposal",
            true,
            true,
            vec!["proposal-validation".into()],
            vec![],
        )?;
        add(
            "files",
            true,
            working,
            vec!["folder-read".into(), "reviewed-folder-write".into()],
            dolores_tools_fs::folder_tool_specs()
                .into_iter()
                .map(|s| s.name)
                .collect(),
        )?;
        add(
            "commands",
            true,
            working,
            vec!["user-account-process".into()],
            vec!["run_command".into()],
        )?;
        add(
            "introspection",
            true,
            working,
            vec!["read-bundled-source".into()],
            vec!["inspect_harness".into()],
        )?;
        add(
            "subagents",
            true,
            working,
            vec!["bounded-scoped-children".into()],
            vec!["delegate_tasks".into()],
        )?;
        if let Some(root) = root {
            for c in self.store.mcp_connections(&root)? {
                let mut advertised = c.clone();
                advertised.enabled = true;
                registry.register(ExtensionRegistration {
                    descriptor: ExtensionDescriptor {
                        id: format!("mcp/{}", c.id),
                        version: if c.server_version.is_empty() {
                            "unknown".into()
                        } else {
                            c.server_version.clone()
                        },
                        api_min: 1,
                        api_max: 1,
                        kind: ExtensionKind::ExternalMcp,
                        config_revision: c.revision,
                        entry_identity: c
                            .fingerprints
                            .iter()
                            .map(|f| f.sha256.as_str())
                            .collect::<Vec<_>>()
                            .join(":"),
                        dependencies: vec![],
                        capabilities: vec!["user-account-external-server".into()],
                        tools: advertised.specs().into_iter().map(|s| s.name).collect(),
                        enabled: c.enabled,
                        available: true,
                        unavailable_reason: "".into(),
                        health: "lazy startup; reviewed manifest, current server health unprobed"
                            .into(),
                    },
                    hooks: vec![],
                })?;
            }
        }
        Ok(Arc::new(registry.snapshot()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use dolores_core::{SessionStore, ToolCall};
    #[test]
    fn scoped_host_registry_pins_valid_inspection_and_reports_side_tool_unavailability() {
        let directory = tempfile::tempdir().unwrap();
        let store = Arc::new(
            dolores_store_sqlite::SqliteStore::open(std::path::Path::new(":memory:")).unwrap(),
        );
        store
            .create_workspace_session(
                "working",
                &dolores_core::SessionWorkspace {
                    kind: dolores_core::WorkspaceKind::Project,
                    root: Some(directory.path().to_string_lossy().into_owned()),
                },
            )
            .unwrap();
        store.create("side").unwrap();
        let engine = Engine::new(
            store,
            Arc::new(crate::connection::testing::MemoryCredentials::default()),
        )
        .unwrap();
        let registry = engine.extension_registry(Some("working")).unwrap();
        let tool = registry
            .pin_tool(Arc::new(crate::introspection::InspectHarness(
                serde_json::json!({}),
            )))
            .unwrap();
        assert!(tool
            .prepare(&ToolCall {
                id: "one".into(),
                name: "inspect_harness".into(),
                arguments: "{}".into()
            })
            .is_ok());
        assert!(tool
            .prepare(&ToolCall {
                id: "two".into(),
                name: "inspect_harness".into(),
                arguments: r#"{"source":"inventory"}"#.into()
            })
            .is_err());
        let side = engine.extension_registry(Some("side")).unwrap();
        assert!(
            !side
                .entries
                .iter()
                .find(|e| e.descriptor.id == "files")
                .unwrap()
                .active
        );
    }
}
