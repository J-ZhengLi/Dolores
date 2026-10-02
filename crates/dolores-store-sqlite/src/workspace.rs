use super::{now, storage_error, SqliteStore};
use dolores_core::{Project, Session, SessionWorkspace, WorkspaceKind};
use rusqlite::{params, OptionalExtension};

impl SqliteStore {
    pub(super) fn read_workspace(&self, id: &str) -> Result<SessionWorkspace, String> {
        let connection = self.lock()?;
        let exists: bool = connection
            .query_row(
                "SELECT EXISTS(SELECT 1 FROM sessions WHERE id=?1)",
                [id],
                |r| r.get(0),
            )
            .map_err(storage_error)?;
        if !exists {
            return Err("Conversation no longer exists.".into());
        }
        let data: Option<(String, Option<String>)> = connection
            .query_row(
                "SELECT kind,root FROM session_workspaces WHERE session_id=?1",
                [id],
                |r| Ok((r.get(0)?, r.get(1)?)),
            )
            .optional()
            .map_err(storage_error)?;
        let Some((kind, root)) = data else {
            return Ok(SessionWorkspace::default());
        };
        let kind = match kind.as_str() {
            "project" => WorkspaceKind::Project,
            "temporary" => WorkspaceKind::Temporary,
            "side" => WorkspaceKind::Side,
            _ => return Err("Saved working folder could not be read.".into()),
        };
        if (kind == WorkspaceKind::Side) != root.is_none() {
            return Err("Saved working folder could not be read.".into());
        }
        Ok(SessionWorkspace { kind, root })
    }

    pub(super) fn read_projects(&self) -> Result<Vec<Project>, String> {
        let connection = self.lock()?;
        let mut statement = connection
            .prepare("SELECT root,name FROM projects ORDER BY updated_at DESC,root LIMIT 50")
            .map_err(storage_error)?;
        let rows = statement
            .query_map([], |r| {
                Ok(Project {
                    root: r.get(0)?,
                    name: r.get(1)?,
                })
            })
            .map_err(storage_error)?;
        rows.collect::<Result<Vec<_>, _>>().map_err(storage_error)
    }

    pub(super) fn save_workspace_session(
        &self,
        id: &str,
        workspace: &SessionWorkspace,
    ) -> Result<Session, String> {
        let kind = match workspace.kind {
            WorkspaceKind::Project => "project",
            WorkspaceKind::Temporary => "temporary",
            WorkspaceKind::Side => "side",
        };
        if (workspace.kind == WorkspaceKind::Side) != workspace.root.is_none() {
            return Err("Invalid working folder.".into());
        }
        let title = match workspace.kind {
            WorkspaceKind::Project => "New project chat",
            WorkspaceKind::Temporary => "Temporary chat",
            WorkspaceKind::Side => "Side chat",
        };
        let session = Session {
            id: id.into(),
            title: title.into(),
            updated_at: now(),
        };
        let mut connection = self.lock()?;
        let tx = connection.transaction().map_err(storage_error)?;
        tx.execute(
            "INSERT INTO sessions(id,title,updated_at) VALUES(?1,?2,?3)",
            params![id, title, session.updated_at],
        )
        .map_err(storage_error)?;
        tx.execute(
            "INSERT INTO session_workspaces(session_id,kind,root) VALUES(?1,?2,?3)",
            params![id, kind, workspace.root],
        )
        .map_err(storage_error)?;
        if workspace.kind == WorkspaceKind::Project {
            let root = workspace.root.as_ref().unwrap();
            let name = std::path::Path::new(root)
                .file_name()
                .and_then(|s| s.to_str())
                .filter(|s| !s.is_empty())
                .unwrap_or(root);
            tx.execute("INSERT INTO projects(root,name,updated_at) VALUES(?1,?2,?3) ON CONFLICT(root) DO UPDATE SET updated_at=excluded.updated_at", params![root,name,session.updated_at]).map_err(storage_error)?;
        }
        tx.commit().map_err(storage_error)?;
        Ok(session)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use dolores_core::{ExportFormat, SessionStore};
    #[test]
    fn working_folder_is_atomic_private_per_session_and_survives_restart() {
        let directory = tempfile::tempdir().unwrap();
        let db = directory.path().join("state.db");
        let project = directory
            .path()
            .join("project")
            .to_string_lossy()
            .into_owned();
        let temporary = directory
            .path()
            .join("temporary")
            .to_string_lossy()
            .into_owned();
        {
            let store = SqliteStore::open(&db).unwrap();
            store.create("legacy").unwrap();
            assert_eq!(store.workspace("legacy").unwrap().kind, WorkspaceKind::Side);
            assert!(store.workspace("missing").is_err());
            for (id, kind, root) in [
                ("p", WorkspaceKind::Project, Some(project.clone())),
                ("t", WorkspaceKind::Temporary, Some(temporary.clone())),
                ("s", WorkspaceKind::Side, None),
            ] {
                store
                    .create_workspace_session(id, &SessionWorkspace { kind, root })
                    .unwrap();
            }
            store.commit_turn("p", "hello", "world").unwrap();
            for format in [ExportFormat::Json, ExportFormat::Markdown] {
                let mut output = vec![];
                store.export_conversation("p", format, &mut output).unwrap();
                assert!(!String::from_utf8(output).unwrap().contains(&project));
            }
            assert!(store
                .create_workspace_session("p", &SessionWorkspace::default())
                .is_err());
            assert_eq!(store.workspace("p").unwrap().root, Some(project.clone()));
        }
        let store = SqliteStore::open(&db).unwrap();
        assert_eq!(store.workspace("t").unwrap().root, Some(temporary));
        assert_eq!(store.workspace("s").unwrap(), SessionWorkspace::default());
        assert_eq!(store.projects().unwrap().len(), 1);
        store.delete("p").unwrap();
        assert!(store.workspace("p").is_err());
        assert_eq!(store.projects().unwrap()[0].root, project);
    }
}
