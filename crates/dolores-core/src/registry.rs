//! Compiled registrations and metadata only; this is not an executable module loader.
use crate::{ToolCall, ToolPlugin, ToolRequest, ToolSpec};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeMap, BTreeSet},
    sync::Arc,
};
use tokio_util::sync::CancellationToken;
pub const HOST_EXTENSION_API: u32 = 1;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum ExtensionKind {
    Compiled,
    ExternalMcp,
    RestrictedWasm,
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ExtensionDescriptor {
    pub id: String,
    pub version: String,
    pub api_min: u32,
    pub api_max: u32,
    pub kind: ExtensionKind,
    pub config_revision: u32,
    pub entry_identity: String,
    pub dependencies: Vec<String>,
    pub capabilities: Vec<String>,
    pub tools: Vec<String>,
    pub enabled: bool,
    pub available: bool,
    pub unavailable_reason: String,
    pub health: String,
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ResolvedExtension {
    pub descriptor: ExtensionDescriptor,
    pub active: bool,
    pub reason: String,
}
/// Synchronous compiled hooks must be pure and bounded. External MCP cannot register hooks.
pub trait ToolProposalHook: Send + Sync {
    fn transform(&self, proposal: &ToolRequest) -> Result<ToolRequest, String>;
}
pub struct ExtensionRegistration {
    pub descriptor: ExtensionDescriptor,
    pub hooks: Vec<Arc<dyn ToolProposalHook>>,
}
#[derive(Default)]
pub struct ExtensionRegistry {
    registrations: BTreeMap<String, Arc<ExtensionRegistration>>,
    generation: u64,
}
#[derive(Clone)]
pub struct RegistrySnapshot {
    pub generation: u64,
    pub entries: Vec<ResolvedExtension>,
    pub order: Vec<String>,
    registrations: Vec<Arc<ExtensionRegistration>>,
}
impl ExtensionRegistry {
    pub fn register(&mut self, registration: ExtensionRegistration) -> Result<(), String> {
        let d = &registration.descriptor;
        let identifier = |s: &str| {
            !s.is_empty()
                && s.len() <= 128
                && s.bytes()
                    .all(|b| b.is_ascii_alphanumeric() || b"/-_.".contains(&b))
        };
        if !identifier(&d.id)
            || d.version.is_empty()
            || d.version.len() > 128
            || d.config_revision == 0
            || d.dependencies.len() > 16
            || d.capabilities.len() > 16
            || d.tools.len() > 9
            || d.dependencies
                .iter()
                .chain(&d.capabilities)
                .chain(&d.tools)
                .any(|s| !identifier(s))
            || d.entry_identity.len() > 512
            || d.health.len() > 128
            || d.unavailable_reason.len() > 512
            || registration.hooks.len() > 4
            || (d.kind == ExtensionKind::ExternalMcp && !registration.hooks.is_empty())
        {
            return Err("Invalid or unsupported extension registration.".into());
        }
        if self.registrations.contains_key(&d.id) || self.registrations.len() >= 32 {
            return Err("Extension ID is already registered or the registry is full.".into());
        }
        self.registrations
            .insert(d.id.clone(), Arc::new(registration));
        self.generation += 1;
        Ok(())
    }
    pub fn unregister(&mut self, id: &str) {
        if self.registrations.remove(id).is_some() {
            self.generation += 1;
        }
    }
    pub fn snapshot(&self) -> RegistrySnapshot {
        let mut reasons = BTreeMap::<String, String>::new();
        for (id, r) in &self.registrations {
            let d = &r.descriptor;
            let reason = if !d.enabled {
                "Disabled"
            } else if d.api_min > HOST_EXTENSION_API
                || d.api_max < HOST_EXTENSION_API
                || d.api_min > d.api_max
            {
                "Unsupported host API; update or disable this extension"
            } else if !d.available {
                &d.unavailable_reason
            } else {
                ""
            };
            if !d.available && reason.is_empty() {
                reasons.insert(id.clone(), "Extension unavailable".into());
            } else if !reason.is_empty() {
                reasons.insert(id.clone(), reason.into());
            }
        }
        loop {
            let mut changed = false;
            for (id, r) in &self.registrations {
                if !reasons.contains_key(id)
                    && r.descriptor.dependencies.iter().any(|dep| {
                        !self.registrations.contains_key(dep) || reasons.contains_key(dep)
                    })
                {
                    reasons.insert(
                        id.clone(),
                        "Dependency unavailable; inspect or restore its registration".into(),
                    );
                    changed = true;
                }
            }
            if !changed {
                break;
            }
        }
        let mut remaining: BTreeSet<_> = self
            .registrations
            .keys()
            .filter(|id| !reasons.contains_key(*id))
            .cloned()
            .collect();
        let mut order = vec![];
        while let Some(id) = remaining
            .iter()
            .find(|id| {
                self.registrations[*id]
                    .descriptor
                    .dependencies
                    .iter()
                    .all(|dep| order.contains(dep))
            })
            .cloned()
        {
            remaining.remove(&id);
            order.push(id);
        }
        for id in remaining {
            reasons.insert(
                id,
                "Dependency cycle; disable or repair the affected extension".into(),
            );
        }
        let entries = self
            .registrations
            .iter()
            .map(|(id, r)| ResolvedExtension {
                descriptor: r.descriptor.clone(),
                active: !reasons.contains_key(id),
                reason: reasons.get(id).cloned().unwrap_or_default(),
            })
            .collect();
        let registrations = order
            .iter()
            .map(|id| self.registrations[id].clone())
            .collect();
        RegistrySnapshot {
            generation: self.generation,
            entries,
            order,
            registrations,
        }
    }
}
impl RegistrySnapshot {
    pub fn pin_tool(
        self: &Arc<Self>,
        inner: Arc<dyn ToolPlugin>,
    ) -> Result<Arc<dyn ToolPlugin>, String> {
        let name = inner.spec().name;
        let owners: Vec<_> = self
            .entries
            .iter()
            .filter(|e| e.active && e.descriptor.tools.contains(&name))
            .collect();
        if owners.len() != 1 {
            return Err("Tool registration has no unique active owner. Inspect extensions.".into());
        }
        Ok(Arc::new(RegistryTool {
            inner,
            snapshot: self.clone(),
        }))
    }
    pub fn transform_proposal(&self, proposal: &ToolRequest) -> Result<ToolRequest, String> {
        let mut transformed = proposal.clone();
        for registration in &self.registrations {
            for hook in &registration.hooks {
                transformed=hook.transform(&transformed).map_err(|_|"Extension policy hook failed. No operation was dispatched; inspect its registration.".to_string())?;
            }
        }
        // Host authority is outside hooks. Reviewed adapter plans cannot be silently rewritten.
        if &transformed != proposal {
            return Err(
                "Extension changed a prepared tool plan. Prepare and review a fresh proposal."
                    .into(),
            );
        }
        Ok(transformed)
    }
}
struct RegistryTool {
    inner: Arc<dyn ToolPlugin>,
    snapshot: Arc<RegistrySnapshot>,
}
#[async_trait]
impl ToolPlugin for RegistryTool {
    fn spec(&self) -> ToolSpec {
        self.inner.spec()
    }
    fn prepare(&self, call: &ToolCall) -> Result<ToolRequest, String> {
        let proposal = self.inner.prepare(call)?;
        self.snapshot.transform_proposal(&proposal)
    }
    async fn invoke(
        &self,
        request: &ToolRequest,
        cancel: CancellationToken,
    ) -> Result<String, String> {
        if cancel.is_cancelled() {
            return Err("Response stopped. Your message was not saved.".into());
        }
        self.snapshot.transform_proposal(request)?;
        self.inner.invoke(request, cancel).await
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    fn registration(id: &str, deps: &[&str]) -> ExtensionRegistration {
        ExtensionRegistration {
            descriptor: ExtensionDescriptor {
                id: id.into(),
                version: "1".into(),
                api_min: 1,
                api_max: 1,
                kind: ExtensionKind::Compiled,
                config_revision: 1,
                entry_identity: "fixture".into(),
                dependencies: deps.iter().map(|s| (*s).into()).collect(),
                capabilities: vec![],
                tools: vec![],
                enabled: true,
                available: true,
                unavailable_reason: "".into(),
                health: "ready".into(),
            },
            hooks: vec![],
        }
    }
    #[test]
    fn dependency_order_api_cycles_and_removal_are_local_and_pinned() {
        let mut registry = ExtensionRegistry::default();
        registry.register(registration("z", &["a"])).unwrap();
        registry.register(registration("a", &[])).unwrap();
        registry
            .register(registration("cycle1", &["cycle2"]))
            .unwrap();
        registry
            .register(registration("cycle2", &["cycle1"]))
            .unwrap();
        let mut bad = registration("bad", &[]);
        bad.descriptor.api_min = 2;
        registry.register(bad).unwrap();
        let old = registry.snapshot();
        assert_eq!(old.order, vec!["a", "z"]);
        assert!(
            !old.entries
                .iter()
                .find(|e| e.descriptor.id == "bad")
                .unwrap()
                .active
        );
        registry.unregister("a");
        let new = registry.snapshot();
        assert!(new.order.is_empty());
        assert_eq!(old.order, vec!["a", "z"]);
        assert!(new.generation > old.generation);
    }
    struct Broken(bool);
    impl ToolProposalHook for Broken {
        fn transform(&self, p: &ToolRequest) -> Result<ToolRequest, String> {
            if self.0 {
                return Err("fixture".into());
            }
            let mut changed = p.clone();
            changed.target = "unreviewed".into();
            Ok(changed)
        }
    }
    #[test]
    fn failed_or_retargeting_hook_refuses_and_external_hooks_are_not_registered() {
        let proposal = ToolRequest {
            call_id: "one".into(),
            name: "read_text_file".into(),
            target: "reviewed".into(),
            query: None,
            diff: None,
            command: None,
            mcp: None,
        };
        for failing in [true, false] {
            let mut registry = ExtensionRegistry::default();
            let mut entry = registration("policy", &[]);
            entry.hooks.push(Arc::new(Broken(failing)));
            registry.register(entry).unwrap();
            assert!(registry.snapshot().transform_proposal(&proposal).is_err());
        }
        let mut external = registration("external", &[]);
        external.descriptor.kind = ExtensionKind::ExternalMcp;
        external.hooks.push(Arc::new(Broken(true)));
        assert!(ExtensionRegistry::default().register(external).is_err());
    }

    #[test]
    fn retired_registration_resources_live_only_until_the_last_snapshot_drops() {
        use std::sync::atomic::{AtomicUsize, Ordering};
        struct Owned(Arc<AtomicUsize>);
        impl ToolProposalHook for Owned {
            fn transform(&self, p: &ToolRequest) -> Result<ToolRequest, String> {
                Ok(p.clone())
            }
        }
        impl Drop for Owned {
            fn drop(&mut self) {
                self.0.fetch_add(1, Ordering::SeqCst);
            }
        }
        let drops = Arc::new(AtomicUsize::new(0));
        let mut registry = ExtensionRegistry::default();
        let mut entry = registration("owned", &[]);
        entry.hooks.push(Arc::new(Owned(drops.clone())));
        registry.register(entry).unwrap();
        let pinned = registry.snapshot();
        registry.unregister("owned");
        assert!(registry.snapshot().entries.is_empty());
        assert_eq!(drops.load(Ordering::SeqCst), 0);
        drop(registry);
        assert_eq!(drops.load(Ordering::SeqCst), 0);
        drop(pinned);
        assert_eq!(drops.load(Ordering::SeqCst), 1);
    }

    struct FixtureTool(&'static str, bool);
    #[async_trait]
    impl ToolPlugin for FixtureTool {
        fn spec(&self) -> ToolSpec {
            ToolSpec {
                name: self.0.into(),
                description: "fixture".into(),
                parameters: serde_json::json!({}),
            }
        }
        fn prepare(&self, call: &ToolCall) -> Result<ToolRequest, String> {
            Ok(ToolRequest {
                call_id: call.id.clone(),
                name: self.0.into(),
                target: "fixture".into(),
                query: None,
                diff: None,
                command: None,
                mcp: None,
            })
        }
        async fn invoke(&self, _: &ToolRequest, _: CancellationToken) -> Result<String, String> {
            if self.1 {
                Err("Server unavailable. Inspect the connection again.".into())
            } else {
                Ok("usable".into())
            }
        }
    }
    #[tokio::test]
    async fn failed_external_adapter_keeps_other_tools_usable_and_removed_tools_unavailable() {
        let mut registry = ExtensionRegistry::default();
        for id in ["external", "files"] {
            let mut entry = registration(id, &[]);
            entry.descriptor.tools = vec![id.into()];
            if id == "external" {
                entry.descriptor.kind = ExtensionKind::ExternalMcp;
            }
            registry.register(entry).unwrap();
        }
        let old = Arc::new(registry.snapshot());
        let external = old
            .pin_tool(Arc::new(FixtureTool("external", true)))
            .unwrap();
        let files = old.pin_tool(Arc::new(FixtureTool("files", false))).unwrap();
        let call = ToolCall {
            id: "one".into(),
            name: "external".into(),
            arguments: "{}".into(),
        };
        let request = external.prepare(&call).unwrap();
        assert!(external
            .invoke(&request, CancellationToken::new())
            .await
            .is_err());
        let request = files
            .prepare(&ToolCall {
                name: "files".into(),
                ..call
            })
            .unwrap();
        assert_eq!(
            files
                .invoke(&request, CancellationToken::new())
                .await
                .unwrap(),
            "usable"
        );
        registry.unregister("external");
        assert!(Arc::new(registry.snapshot())
            .pin_tool(Arc::new(FixtureTool("external", true)))
            .is_err());
        assert!(old
            .entries
            .iter()
            .any(|e| e.active && e.descriptor.id == "external"));
    }
}
