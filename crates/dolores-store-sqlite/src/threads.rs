use super::{now, storage_error, SqliteStore};
use dolores_core::{Session, TurnMetadata};
use rusqlite::{params, OptionalExtension};
impl SqliteStore {
    pub(super) fn read_auto_compact(&self, id: &str) -> Result<bool, String> {
        let c = self.lock()?;
        let exists: bool = c
            .query_row(
                "SELECT EXISTS(SELECT 1 FROM sessions WHERE id=?1)",
                [id],
                |r| r.get(0),
            )
            .map_err(storage_error)?;
        if !exists {
            return Err("Chat no longer exists.".into());
        }
        Ok(c.query_row(
            "SELECT auto_compact FROM thread_context WHERE session_id=?1",
            [id],
            |r| r.get(0),
        )
        .optional()
        .map_err(storage_error)?
        .unwrap_or(false))
    }
    pub(super) fn write_auto_compact(&self, id: &str, enabled: bool) -> Result<(), String> {
        self.lock()?.execute("INSERT INTO thread_context(session_id,auto_compact) VALUES(?1,?2) ON CONFLICT(session_id) DO UPDATE SET auto_compact=excluded.auto_compact",params![id,enabled]).map_err(storage_error)?;
        Ok(())
    }
    pub(super) fn fork_thread(
        &self,
        source: &str,
        through: i64,
        id: &str,
    ) -> Result<Session, String> {
        let mut c = self.lock()?;
        let tx = c.transaction().map_err(storage_error)?;
        let rows = {
            let mut s=tx.prepare("SELECT m.id,m.role,m.content,t.data FROM messages m LEFT JOIN turn_metadata t ON t.message_id=m.id WHERE m.session_id=?1 AND m.id<=?2 ORDER BY m.id LIMIT 201").map_err(storage_error)?;
            let rows = s
                .query_map(params![source, through], |r| {
                    Ok((
                        r.get::<_, i64>(0)?,
                        r.get::<_, String>(1)?,
                        r.get::<_, String>(2)?,
                        r.get::<_, Option<String>>(3)?,
                    ))
                })
                .map_err(storage_error)?
                .collect::<Result<Vec<_>, _>>()
                .map_err(storage_error)?;
            rows
        };
        if rows.is_empty()
            || rows.len() > 200
            || !rows.len().is_multiple_of(2)
            || rows.last().unwrap().0 != through
            || rows
                .chunks_exact(2)
                .any(|p| p[0].1 != "user" || p[1].1 != "assistant")
        {
            return Err("Choose a completed assistant turn within the first 100 turns. A fork cannot copy a partial boundary.".into());
        }
        if rows
            .iter()
            .map(|r| r.2.len() + r.3.as_ref().map_or(0, String::len))
            .sum::<usize>()
            > 1024 * 1024
        {
            return Err("Fork history exceeds 1 MiB. Export it or choose an earlier turn.".into());
        }
        let title = tx
            .query_row("SELECT title FROM sessions WHERE id=?1", [source], |r| {
                r.get::<_, String>(0)
            })
            .map_err(storage_error)?;
        let session = Session {
            id: id.into(),
            title: format!("Fork: {title}"),
            updated_at: now(),
        };
        tx.execute(
            "INSERT INTO sessions(id,title,updated_at) VALUES(?1,?2,?3)",
            params![id, session.title, session.updated_at],
        )
        .map_err(storage_error)?;
        tx.execute("INSERT INTO session_workspaces(session_id,kind,root) SELECT ?1,kind,root FROM session_workspaces WHERE session_id=?2",params![id,source]).map_err(storage_error)?;
        for (_, role, content, data) in rows {
            tx.execute(
                "INSERT INTO messages(session_id,role,content) VALUES(?1,?2,?3)",
                params![id, role, content],
            )
            .map_err(storage_error)?;
            if let Some(data) = data {
                let mut meta: TurnMetadata = serde_json::from_str(&data).map_err(storage_error)?;
                // Historical receipts remain evidence, but cannot resume a parent task.
                meta.paused = None;
                tx.execute(
                    "INSERT INTO turn_metadata(message_id,data) VALUES(?1,?2)",
                    params![
                        tx.last_insert_rowid(),
                        serde_json::to_string(&meta).map_err(storage_error)?
                    ],
                )
                .map_err(storage_error)?;
            }
        }
        tx.execute(
            "INSERT INTO thread_context(session_id,origin) VALUES(?1,?2)",
            params![
                id,
                serde_json::json!({"session":source,"through":through,"sharedFolder":true})
                    .to_string()
            ],
        )
        .map_err(storage_error)?;
        tx.commit().map_err(storage_error)?;
        Ok(session)
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use dolores_core::{SessionStore, SessionWorkspace, WorkspaceKind};
    #[test]
    fn forks_are_atomic_scoped_and_do_not_copy_drafts_permissions_or_policy() {
        let s = SqliteStore::open(std::path::Path::new(":memory:")).unwrap();
        s.create_workspace_session(
            "parent",
            &SessionWorkspace {
                kind: WorkspaceKind::Temporary,
                root: Some("shared".into()),
            },
        )
        .unwrap();
        s.commit_turn("parent", "Goal 世界", "done").unwrap();
        s.commit_turn("parent", "later", "later done").unwrap();
        s.save_draft("parent", "private draft").unwrap();
        s.set_auto_compact("parent", true).unwrap();
        let page = s.messages_page("parent", None, false, 80).unwrap();
        let user = page.items[0].id;
        let through = page.items[1].id;
        assert!(s.fork_session("parent", user, "bad").is_err());
        assert!(s.fork_session("other", through, "bad").is_err());
        s.fork_session("parent", through, "fork").unwrap();
        assert_eq!(s.messages("fork").unwrap().len(), 2);
        assert_eq!(s.workspace("fork").unwrap(), s.workspace("parent").unwrap());
        assert_eq!(s.saved_draft("fork").unwrap(), "");
        assert!(!s.auto_compact("fork").unwrap());
        assert!(s.runs("fork").unwrap().is_empty());
        assert_eq!(
            s.scoped_settings(dolores_core::SettingsScope::Thread, "fork")
                .unwrap()
                .revision,
            0
        );
        assert!(s.fork_session("parent", through, "fork").is_err());
        assert_eq!(s.messages("fork").unwrap().len(), 2);
        s.delete("parent").unwrap();
        assert_eq!(s.messages("fork").unwrap().len(), 2);
    }
}
