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
                    > if matches!(
                        request.name.as_str(),
                        "delegate_tasks" | "browser" | "desktop_control" | "inspect_harness" | "harness_repair" | "test_harness_repair"
                    ) {
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
        add("harness/repair",true,working && self.workspace_directory.is_some(),vec!["managed-native-proposal".into()],vec!["harness_repair".into()])?;
        add("harness/native-tests",true,working && self.workspace_directory.is_some(),vec!["reviewed-native-tests".into()],vec!["test_harness_repair".into()])?;
        add(
            "desktop/observation",
            true,
            working,
            vec!["selected-screenshot-sharing".into()],
            vec![
                "inspect_desktop_capture".into(),
                "desktop_control".into(),
                "request_desktop_access".into(),
            ],
        )?;
        add(
            "subagents",
            true,
            working,
            vec!["bounded-scoped-children".into()],
            vec!["delegate_tasks".into()],
        )?;
        add(
            "browser",
            true,
            working && self.browser_runtime().is_ok(),
            vec!["owned-browser-session".into()],
            vec!["browser".into()],
        )?;
        {
            let web = self.store.web_configuration()?;
            registry.register(ExtensionRegistration {
                descriptor: ExtensionDescriptor {
                    id: "web".into(),
                    version: env!("CARGO_PKG_VERSION").into(),
                    api_min: 1,
                    api_max: 1,
                    kind: dolores_core::ExtensionKind::Compiled,
                    entry_identity: format!("{}:web", env!("DOLORES_BUILD_REVISION")),
                    config_revision: web.revision.saturating_add(1),
                    dependencies: vec![],
                    capabilities: vec!["public-network-research".into()],
                    tools: dolores_tools_web::specs(&web)
                        .into_iter()
                        .map(|s| s.name)
                        .collect(),
                    enabled: web.enabled,
                    available: working,
                    unavailable_reason: if working {
                        String::new()
                    } else {
                        "Start a project or temporary working chat".into()
                    },
                    health: "on-demand; service availability not probed".into(),
                },
                hooks: vec![],
            })?;
        }
        if let Some(root) = root {
            if let Ok(state) = self.store.mod_state(&root) {
                for version in &state.versions {
                    registry.register(ExtensionRegistration {
                        descriptor: ExtensionDescriptor {
                            id: format!("mods/{}", &version.identity[..12]),
                            version: version.identity.clone(),
                            api_min: 1,
                            api_max: 1,
                            kind: ExtensionKind::RestrictedWasm,
                            config_revision: state.revision.max(1),
                            entry_identity: version.identity.clone(),
                            dependencies: vec![],
                            capabilities: vec!["stateless-recovery-hint".into()],
                            tools: vec![],
                            enabled: state.active.as_ref() == Some(&version.identity),
                            available: true,
                            unavailable_reason: String::new(),
                            health: version.status.clone(),
                        },
                        hooks: vec![],
                    })?;
                }
            }
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
        assert!(tool.prepare(&ToolCall {
            id: "pinned-range".into(), name: "inspect_harness".into(),
            arguments: serde_json::json!({"source":"core","startLine":150,"lineCount":180,
                "bundleId":crate::introspection::bundle::ID,
                "sourceId":crate::introspection::bundle::find("crates/dolores-core/src/lib.rs").unwrap().id}).to_string(),
        }).is_ok());
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
