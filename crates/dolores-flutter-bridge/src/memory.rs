use super::Engine;
use dolores_core::{MemoryDraft, MemoryPreference, MemoryScope, SessionStore};
use serde_json::{json, Value};

#[derive(serde::Deserialize)]
pub(super) struct MemoryInput {
    pub(super) id: Option<String>,
    pub(super) revision: Option<u32>,
    pub(super) title: String,
    pub(super) text: String,
    pub(super) enabled: bool,
}

pub(super) fn preferences_for_session(
    store: &dyn SessionStore,
    session: Option<&str>,
) -> Result<Vec<MemoryPreference>, String> {
    let root = session
        .map(|id| store.workspace(id))
        .transpose()?
        .and_then(|w| w.root);
    store.memory_preferences(root.as_deref())
}
impl Engine {
    fn memory_root(
        &self,
        session: Option<&str>,
        scope: MemoryScope,
    ) -> Result<Option<String>, String> {
        if scope == MemoryScope::All {
            return Ok(None);
        }
        session
            .map(|id| self.store.workspace(id))
            .transpose()?
            .and_then(|w| w.root)
            .map(Some)
            .ok_or("Memory folder preferences need an existing working folder.".into())
    }
    pub(super) fn memories(&self, session: Option<&str>) -> Result<Value, String> {
        let root = session
            .map(|id| self.store.workspace(id))
            .transpose()?
            .and_then(|w| w.root);
        Ok(
            json!({"items":self.store.memory_preferences(root.as_deref())?,"folderAvailable":root.is_some()}),
        )
    }
    pub(super) fn save_memory(
        &self,
        session: Option<&str>,
        scope: MemoryScope,
        input: MemoryInput,
    ) -> Result<Value, String> {
        let MemoryInput {
            id,
            revision,
            title,
            text,
            enabled,
        } = input;
        if id.is_some() != revision.is_some() {
            return Err(
                "Memory preference identity is invalid. Refresh Memory and review it again.".into(),
            );
        }
        let root = self.memory_root(session, scope)?;
        let draft = MemoryDraft {
            id: id.unwrap_or_else(|| uuid::Uuid::new_v4().to_string()),
            revision,
            title,
            text,
            enabled,
        };
        Ok(json!(self
            .store
            .save_memory_preference(root.as_deref(), &draft)?))
    }
    pub(super) fn delete_memory(
        &self,
        session: Option<&str>,
        scope: MemoryScope,
        id: &str,
        revision: u32,
    ) -> Result<Value, String> {
        let root = self.memory_root(session, scope)?;
        self.store
            .delete_memory_preference(root.as_deref(), id, revision)?;
        Ok(Value::Null)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Command;
    use dolores_core::{SessionWorkspace, WorkspaceKind};
    use dolores_store_sqlite::SqliteStore;
    use std::sync::Arc;

    #[test]
    fn manual_commands_resolve_saved_scope_and_context_keeps_exact_provenance() {
        let folder = tempfile::tempdir().unwrap();
        let other = tempfile::tempdir().unwrap();
        let store = Arc::new(SqliteStore::open(&folder.path().join("state.db")).unwrap());
        for (id, root) in [
            ("first", Some(folder.path())),
            ("same", Some(folder.path())),
            ("other", Some(other.path())),
            ("side", None),
        ] {
            store
                .create_workspace_session(
                    id,
                    &SessionWorkspace {
                        kind: if root.is_some() {
                            WorkspaceKind::Project
                        } else {
                            WorkspaceKind::Side
                        },
                        root: root.map(|p| crate::workspace::canonical_folder(p).unwrap()),
                    },
                )
                .unwrap();
        }
        let engine = Engine::new(
            store,
            Arc::new(crate::connection::testing::MemoryCredentials::default()),
        )
        .unwrap();
        let command = |v| engine.call(serde_json::from_value::<Command>(v).unwrap());
        assert_eq!(
            command(json!({"command":"memories"})).unwrap()["items"],
            json!([])
        );
        let global = command(json!({"command":"saveMemory","scope":"all","title":"Style","text":"GLOBAL_LITERAL","enabled":true})).unwrap();
        let local = command(json!({"command":"saveMemory","scope":"folder","session":"first","title":"Tests","text":"FOLDER_LITERAL","enabled":true})).unwrap();
        for id in [None, Some("side"), Some("other")] {
            let items = command(json!({"command":"memories","session":id})).unwrap();
            assert_eq!(items["items"].as_array().unwrap().len(), 1);
            let preview =
                command(json!({"command":"context","session":id,"input":"hello","tools":false}))
                    .unwrap();
            assert!(preview["messages"][0]["content"]
                .as_str()
                .unwrap()
                .contains("GLOBAL_LITERAL"));
            assert!(!preview.to_string().contains("FOLDER_LITERAL"));
        }
        for id in ["first", "same"] {
            let preview =
                command(json!({"command":"context","session":id,"input":"hello","tools":false}))
                    .unwrap();
            assert_eq!(preview["memoryEntries"][0]["id"], local["id"]);
            assert_eq!(preview["memory"]["used"][0]["revision"], 1);
            assert_eq!(preview["memory"]["used"][1]["id"], global["id"]);
            assert!(!preview
                .to_string()
                .contains(folder.path().to_str().unwrap()));
            assert!(preview["tokens"]["inputTokens"].is_number());
        }
        for id in [None, Some("side")] {
            assert!(command(json!({"command":"saveMemory","scope":"folder","session":id,"title":"Wrong","text":"No folder","enabled":true})).is_err());
        }
        assert!(command(json!({"command":"deleteMemory","scope":"folder","session":"other","id":local["id"],"revision":1})).is_err());
        assert!(command(json!({"command":"deleteMemory","scope":"folder","session":"first","id":global["id"],"revision":1})).is_err());
        let disabled = command(json!({"command":"saveMemory","scope":"folder","session":"first","id":local["id"],"revision":1,"title":"Tests","text":"FOLDER_LITERAL","enabled":false})).unwrap();
        assert_eq!(disabled["revision"], 2);
        assert!(command(json!({"command":"deleteMemory","scope":"folder","session":"first","id":local["id"],"revision":1})).is_err());
        let preview =
            command(json!({"command":"context","session":"first","input":"hello","tools":false}))
                .unwrap();
        assert_eq!(preview["memoryEntries"].as_array().unwrap().len(), 1);
        command(json!({"command":"deleteMemory","scope":"all","id":global["id"],"revision":1}))
            .unwrap();
        let preview =
            command(json!({"command":"context","session":"first","input":"hello","tools":false}))
                .unwrap();
        assert!(preview.get("memory").is_none());
        assert_eq!(preview["memoryEntries"], json!([]));
    }
}
