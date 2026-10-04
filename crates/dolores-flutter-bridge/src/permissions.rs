use crate::*;
use dolores_core::{PermissionMode, PermissionPolicy, SettingsScope, ToolRequest};
use std::time::{SystemTime, UNIX_EPOCH};
pub(super) fn now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}
pub(super) struct PermissionGuard {
    pub store: Arc<dyn SessionStore>,
    pub session: String,
    pub revision: u32,
    pub policy: PermissionPolicy,
}
impl PermissionGuard {
    pub fn recheck(&self) -> Result<(), String> {
        let record = self
            .store
            .scoped_settings(SettingsScope::Thread, &self.session)?;
        if record.revision != self.revision
            || record.patch.permissions.clone().unwrap_or_default() != self.policy
        {
            return Err("Task permissions changed. No pending operation was dispatched. Inspect saved progress and prepare a fresh run.".into());
        }
        if self.policy.expired(now()) {
            return Err("Task permission grant expired. Review permissions and start a fresh run; pending operations were not dispatched.".into());
        }
        Ok(())
    }
    pub fn automatic(&self, request: &ToolRequest) -> Result<bool, String> {
        self.recheck()?;
        Ok(self.policy.automatic(request, now()))
    }
}
impl Engine {
    pub(super) fn permission_view(&self, session: &str) -> Result<Value, String> {
        let workspace = self.store.workspace(session)?;
        let record = self.store.scoped_settings(SettingsScope::Thread, session)?;
        let policy = record.patch.permissions.unwrap_or_default();
        let mut mcp_tools = vec![];
        if let Some(root) = &workspace.root {
            for connection in self.store.mcp_connections(root)? {
                for (spec, tool) in connection.specs().iter().zip(&connection.tools) {
                    mcp_tools.push(json!({"alias":spec.name,"tool":tool.name,"label":connection.launch.label,"connectionId":connection.id,"revision":connection.revision}));
                }
            }
        }
        Ok(
            json!({"revision":record.revision,"policy":policy,"expired":policy.expired(now()),"mcpTools":mcp_tools,"working":workspace.root.is_some(),"scope":"This chat and its saved working folder","containment":"File tools retain folder/secret/link restrictions. Commands and MCP run with your OS account permissions, including outside the folder. This is not an OS sandbox.","adaptation":"Task access cannot enable self-updates. Budgets, Stop, fresh snapshots and reviewed MCP metadata remain enforced."}),
        )
    }
    pub(super) fn set_permissions(
        &self,
        session: &str,
        revision: u32,
        policy: PermissionPolicy,
    ) -> Result<Value, String> {
        policy.validate()?;
        let workspace = self.store.workspace(session)?;
        if workspace.root.is_none() && policy.mode != PermissionMode::Review {
            return Err(
                "Choose a project or temporary working chat before granting tool access.".into(),
            );
        }
        if policy.mode != PermissionMode::Review && policy.expired(now()) {
            return Err("Choose a future expiry or no expiry before granting access.".into());
        }
        let mut record = self.store.scoped_settings(SettingsScope::Thread, session)?;
        if record.revision != revision {
            return Err(
                "Permissions changed. Refresh and review your choices before saving.".into(),
            );
        }
        record.patch.permissions = Some(policy);
        self.store
            .save_scoped_settings(SettingsScope::Thread, session, revision, &record.patch)?;
        self.permission_view(session)
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn revocation_and_revision_changes_invalidate_a_prepared_grant() {
        let store = Arc::new(SqliteStore::open(std::path::Path::new(":memory:")).unwrap());
        store.create("a").unwrap();
        store.create("b").unwrap();
        let guard = PermissionGuard {
            store: store.clone(),
            session: "a".into(),
            revision: 0,
            policy: Default::default(),
        };
        assert!(guard.recheck().is_ok());
        let record = store.scoped_settings(SettingsScope::Thread, "a").unwrap();
        store
            .save_scoped_settings(SettingsScope::Thread, "a", 0, &record.patch)
            .unwrap();
        assert!(guard.recheck().unwrap_err().contains("changed"));
        assert_eq!(
            store
                .scoped_settings(SettingsScope::Thread, "b")
                .unwrap()
                .revision,
            0
        );
    }
}
