use super::Engine;
use dolores_core::{SessionWorkspace, WorkspaceKind};
use serde_json::{json, Value};
use std::path::{Path, PathBuf};

pub(super) fn canonical_folder(path: &Path) -> Result<String, String> {
    if !path.is_absolute() {
        return Err("Choose an absolute project folder.".into());
    }
    let root = path
        .canonicalize()
        .map_err(|_| "Working folder is unavailable. Restore it or start a new project chat.")?;
    if !root.is_dir() {
        return Err("Choose a folder, not a file.".into());
    }
    let root = root
        .to_str()
        .ok_or("Working folder path is not valid Unicode.")?;
    #[cfg(windows)]
    let root = if let Some(unc) = root.strip_prefix(r"\\?\UNC\") {
        format!(r"\\{unc}")
    } else {
        root.strip_prefix(r"\\?\").unwrap_or(root).to_owned()
    };
    #[cfg(not(windows))]
    let root = root.to_owned();
    dolores_tools_fs::folder_tools(Path::new(&root))?;
    Ok(root)
}

impl Engine {
    pub(super) fn create_working_session(
        &self,
        kind: WorkspaceKind,
        path: Option<PathBuf>,
    ) -> Result<Value, String> {
        let id = uuid::Uuid::new_v4().to_string();
        let mut created_folder = None;
        let root = match kind {
            WorkspaceKind::Project => {
                Some(canonical_folder(&path.ok_or("Choose a project folder.")?)?)
            }
            WorkspaceKind::Temporary => {
                if path.is_some() {
                    return Err("Temporary chats choose their own working folder.".into());
                }
                let base = self
                    .workspace_directory
                    .as_ref()
                    .ok_or("Temporary working folders are unavailable.")?;
                std::fs::create_dir_all(base)
                    .map_err(|_| "Could not create a temporary working folder.")?;
                let folder = base.join(&id);
                std::fs::create_dir(&folder)
                    .map_err(|_| "Could not create a temporary working folder.")?;
                let root = canonical_folder(&folder)?;
                created_folder = Some(folder);
                Some(root)
            }
            WorkspaceKind::Side => {
                if path.is_some() {
                    return Err("Side chats do not use a working folder.".into());
                }
                None
            }
        };
        let workspace = SessionWorkspace { kind, root };
        match self.store.create_workspace_session(&id, &workspace) {
            Ok(session) => Ok(json!({"session":session,"workspace":workspace})),
            Err(error) => {
                // Only our just-created, empty folder. Never recursively delete workspace data.
                if let Some(folder) = created_folder {
                    let _ = std::fs::remove_dir(folder);
                }
                Err(error)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Command;
    use dolores_core::SessionStore;
    use dolores_store_sqlite::SqliteStore;
    #[test]
    fn project_and_temporary_sessions_have_distinct_saved_folders() {
        let directory = tempfile::tempdir().unwrap();
        let store =
            std::sync::Arc::new(SqliteStore::open(&directory.path().join("state.db")).unwrap());
        let mut engine = Engine::new(
            store.clone(),
            std::sync::Arc::new(crate::connection::testing::MemoryCredentials::default()),
        )
        .unwrap();
        engine.workspace_directory = Some(directory.path().join("workspaces"));
        let first = engine
            .call(Command::CreateSession {
                kind: WorkspaceKind::Temporary,
                path: None,
            })
            .unwrap();
        let second = engine
            .call(Command::CreateSession {
                kind: WorkspaceKind::Temporary,
                path: None,
            })
            .unwrap();
        assert_ne!(first["workspace"]["root"], second["workspace"]["root"]);
        assert!(Path::new(first["workspace"]["root"].as_str().unwrap()).is_dir());
        let project = engine
            .call(Command::CreateSession {
                kind: WorkspaceKind::Project,
                path: Some(directory.path().to_owned()),
            })
            .unwrap();
        let id = project["session"]["id"].as_str().unwrap();
        assert_eq!(store.workspace(id).unwrap().kind, WorkspaceKind::Project);
        assert_eq!(store.projects().unwrap().len(), 1);
        let preview = engine
            .call(Command::Context {
                session: Some(id.into()),
                input: "hello".into(),
                tools: false,
            })
            .unwrap();
        assert!(preview["messages"]
            .as_array()
            .unwrap()
            .iter()
            .any(|m| m["role"] == "system"));
        assert!(!preview
            .to_string()
            .contains(&canonical_folder(directory.path()).unwrap()));
        assert!(engine
            .call(Command::CreateSession {
                kind: WorkspaceKind::Project,
                path: Some(directory.path().join("missing"))
            })
            .is_err());
        let side = engine
            .call(Command::CreateSession {
                kind: WorkspaceKind::Side,
                path: None,
            })
            .unwrap();
        assert!(side["workspace"]["root"].is_null());
        engine
            .call(Command::Configure {
                preferences: dolores_core::ConnectionPreferences {
                    base_url: "http://127.0.0.1:19421/v1".into(),
                    model: "fixture".into(),
                },
                api_key: Some(String::new()),
                remember: false,
                enabled_models: None,
                model_contexts: None,
            })
            .unwrap();
        assert!(engine
            .call(Command::Start {
                desktop_capture: None,
                desktop_grant: None,
                desktop_reconciled: false,
                observation_model: None,
                resume_run: None,
                continuation: None,
                id: 99,
                session: Some(id.into()),
                input: "hello".into(),
                workspace: Some(PathBuf::from(first["workspace"]["root"].as_str().unwrap()))
            })
            .unwrap_err()
            .contains("different working folder"));
        store.delete(id).unwrap();
        assert!(directory.path().is_dir());
    }
}
