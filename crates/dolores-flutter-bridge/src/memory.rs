use super::Engine;
use dolores_core::{MemoryDraft, MemoryPreference, MemoryScope, SessionStore};
use serde_json::{json, Value};

pub(super) fn recall_for_session(
    store: &dyn SessionStore,
    session: Option<&str>,
) -> Result<Vec<MemoryPreference>, String> {
    let records = preferences_for_session(store, session)?;
    records
        .into_iter()
        .filter_map(|p| match p.origin.as_ref() {
            Some(origin) => match store.memory_source_message(&origin.session, origin.message_id) {
                Ok(Some(source)) if source.text.contains(&origin.quote) => Some(Ok(p)),
                Ok(_) => None,
                Err(error) => Some(Err(error)),
            },
            None => Some(Ok(p)),
        })
        .collect()
}

pub(super) fn prepare_recall(
    messages: Vec<dolores_core::Message>,
    memories: Vec<MemoryPreference>,
    specs: &[dolores_core::ToolSpec],
    window: Option<u32>,
    settings: dolores_core::RequestSettings,
) -> Result<
    (
        Vec<dolores_core::Message>,
        Option<dolores_core::MemoryContext>,
    ),
    String,
> {
    let (messages, baseline) =
        dolores_core::prepare_token_context(messages, specs, window, settings)?;
    let room = baseline
        .max_input_tokens
        .map_or(2000, |limit| limit.saturating_sub(baseline.input_tokens))
        .min(2000);
    dolores_core::prepare_memory_context_with_budget(messages, memories, room)
}

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
    if !store.automatic_memory_policy()?.enabled {
        return Ok(Vec::new());
    }
    let root = session
        .map(|id| store.workspace(id))
        .transpose()?
        .and_then(|w| w.root);
    store.memory_preferences(root.as_deref())
}
impl Engine {
    pub(super) fn memory_evidence(
        &self,
        session: Option<&str>,
        id: &str,
        revision: u32,
    ) -> Result<Value, String> {
        let root = session
            .map(|s| self.store.workspace(s))
            .transpose()?
            .and_then(|w| w.root);
        let memory = self
            .store
            .memory_preferences(root.as_deref())?
            .into_iter()
            .find(|p| p.id == id && p.revision == revision)
            .ok_or(
                "Memory changed or was forgotten. Refresh Memory to inspect its current source.",
            )?;
        let origin = memory
            .origin
            .ok_or("This memory was added manually and has no conversation source.")?;
        let source=self.store.memory_source_message(&origin.session,origin.message_id)?.filter(|s|s.text.contains(&origin.quote)).ok_or("Source evidence is unavailable or changed. The retained quote is historical only and is excluded from recall; inspect or forget this memory.")?;
        let start = source
            .text
            .find(&origin.quote)
            .ok_or("Source quote is unavailable.")?;
        let end = start + origin.quote.len();
        Ok(
            json!({"session":origin.session,"messageId":origin.message_id,"text":&source.text[start..end],"shortened":start>0 || end<source.text.len(),"note":"Exact retained source excerpt; opening it sends no model request."}),
        )
    }
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
        let preferences = self.store.memory_preferences(root.as_deref())?;
        let mut items = Vec::new();
        for preference in preferences {
            let mut item = json!(preference);
            item["kind"] = json!(preference.title.split(':').next().unwrap_or("Preference"));
            item["confidence"] = json!(if preference.origin.is_some() {
                "Source-linked; inspect the exact statement"
            } else {
                "Added or corrected by you"
            });
            if preference.source=="automatic" && !preference.auto_update { item["confidence"]=json!("Corrected by you; earlier source is historical"); }
            item["previous"] = json!(self.store.memory_versions(&preference.id)?);
            if let Some(origin) = &preference.origin {
                item["originAvailable"] = json!(self
                    .store
                    .memory_source_message(&origin.session, origin.message_id)?
                    .is_some_and(|m| m.text.contains(&origin.quote)));
            }
            items.push(item);
        }
        Ok(
            json!({"items":items,"folderAvailable":root.is_some(),"automaticPolicy":self.store.automatic_memory_policy()?,"automaticAttempt":session.map(|s|self.store.automatic_memory_attempt(s)).transpose()?.flatten()}),
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
            origin: None,
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
        self.memory_maintenance.stop(false);
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
        store.set_automatic_memory_policy(true, 1).unwrap();
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
        command(json!({"command":"setAutomaticMemory","enabled":false,"revision":2})).unwrap();
        let off =
            command(json!({"command":"context","session":"first","input":"hello","tools":false}))
                .unwrap();
        assert_eq!(off["memoryEntries"], json!([]));
        assert!(!off.to_string().contains("FOLDER_LITERAL"));
        assert_eq!(
            command(json!({"command":"memories","session":"first"})).unwrap()["items"]
                .as_array()
                .unwrap()
                .len(),
            2
        );
        command(json!({"command":"setAutomaticMemory","enabled":true,"revision":3})).unwrap();
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
