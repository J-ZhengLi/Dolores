use super::{now, storage_error, SqliteStore};
use dolores_core::Session;
use rusqlite::params;
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
        // Compaction is a context safeguard, including for legacy opt-out rows.
        Ok(true)
    }
    pub(super) fn write_auto_compact(&self, id: &str, enabled: bool) -> Result<(), String> {
        if !enabled {
            return Err("Automatic compaction is always enabled.".into());
        }
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
        let completed: bool = tx.query_row(
            "SELECT EXISTS(SELECT 1 FROM messages WHERE session_id=?1 AND id=?2 AND role='assistant')",
            params![source,through], |r| r.get(0),
        ).map_err(storage_error)?;
        if !completed {
            return Err("Choose a completed assistant reply in this chat. A branch cannot copy a partial turn.".into());
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
        {
            // Keep only the cursor's ID/role and metadata kind in Rust. SQLite copies text and
            // metadata one message at a time; total history is never buffered.
            let mut select = tx
                .prepare("SELECT m.id,m.role,json_type(t.data) FROM messages m LEFT JOIN turn_metadata t ON t.message_id=m.id WHERE m.session_id=?1 AND m.id<=?2 ORDER BY m.id")
                .map_err(storage_error)?;
            let mut rows = select
                .query(params![source, through])
                .map_err(storage_error)?;
            let mut expected = "user";
            let mut last = None;
            while let Some(row) = rows.next().map_err(storage_error)? {
                let source_id: i64 = row.get(0).map_err(storage_error)?;
                let role: String = row.get(1).map_err(storage_error)?;
                let metadata_kind: Option<String> = row.get(2).map_err(storage_error)?;
                if metadata_kind
                    .as_deref()
                    .is_some_and(|kind| kind != "object")
                {
                    return Err("Saved turn metadata is invalid. The original chat remains; no branch was created.".into());
                }
                if role != expected {
                    return Err("Saved history contains an incomplete turn. The original chat remains; no branch was created.".into());
                }
                expected = if expected == "user" {
                    "assistant"
                } else {
                    "user"
                };
                tx.execute("INSERT INTO messages(session_id,role,content) SELECT ?1,role,content FROM messages WHERE id=?2 AND session_id=?3",params![id,source_id,source]).map_err(storage_error)?;
                let copied_id = tx.last_insert_rowid();
                tx.execute("INSERT INTO message_parts(message_id,data) SELECT ?1,data FROM message_parts WHERE message_id=?2",params![copied_id,source_id]).map_err(storage_error)?;
                tx.execute("INSERT INTO message_timestamps(message_id,saved_at) SELECT ?1,saved_at FROM message_timestamps WHERE message_id=?2",params![copied_id,source_id]).map_err(storage_error)?;
                // Historical receipts remain evidence, without parent resume
                // authority. Unknown metadata fields survive the SQL projection.
                tx.execute("INSERT INTO turn_metadata(message_id,data) SELECT ?1,json_remove(data,'$.paused') FROM turn_metadata WHERE message_id=?2",params![copied_id,source_id]).map_err(storage_error)?;
                last = Some(source_id);
            }
            if last != Some(through) || expected != "user" {
                return Err(
                    "Choose a completed assistant reply. No partial branch was created.".into(),
                );
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
    fn invalid_saved_metadata_cannot_publish_an_unreadable_branch() {
        for invalid in ["[]", "{"] {
            let s = SqliteStore::open(std::path::Path::new(":memory:")).unwrap();
            s.create("parent").unwrap();
            s.commit_turn("parent", "Question", "Answer").unwrap();
            let through = s.messages_page("parent", None, false, 2).unwrap().items[1].id;
            s.lock()
                .unwrap()
                .execute(
                    "INSERT INTO turn_metadata(message_id,data) VALUES(?1,?2)",
                    params![through, invalid],
                )
                .unwrap();
            assert!(s.fork_session("parent", through, "fork").is_err());
            assert_eq!(
                s.lock()
                    .unwrap()
                    .query_row::<i64, _, _>(
                        "SELECT count(*) FROM sessions WHERE id='fork'",
                        [],
                        |r| r.get(0)
                    )
                    .unwrap(),
                0
            );
        }
    }
    #[test]
    fn fork_copies_history_beyond_one_mib_without_a_total_size_cutoff() {
        let s = SqliteStore::open(std::path::Path::new(":memory:")).unwrap();
        s.create("large-parent").unwrap();
        let answer = "Evidence 世界\n".repeat(7000);
        for n in 0..12 {
            s.commit_turn("large-parent", &format!("Question {n}"), &answer)
                .unwrap();
        }
        let through = s
            .messages_page("large-parent", None, false, 2)
            .unwrap()
            .items
            .last()
            .unwrap()
            .id;
        let result = s.fork_session("large-parent", through, "large-fork");
        assert!(
            result.is_ok(),
            "Large saved history must remain branchable: {}",
            result.unwrap_err()
        );
        let c = s.lock().unwrap();
        let (count, bytes): (i64,i64) = c.query_row("SELECT count(*),sum(length(CAST(content AS BLOB))) FROM messages WHERE session_id='large-fork'", [], |r|Ok((r.get(0)?,r.get(1)?))).unwrap();
        assert_eq!(count, 24);
        assert!(bytes > 1024 * 1024);
    }
    #[test]
    fn fork_copies_more_than_one_hundred_complete_turns() {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("long-fork.db");
        let s = SqliteStore::open(&path).unwrap();
        s.create("long-parent").unwrap();
        for n in 0..140 {
            s.commit_turn(
                "long-parent",
                &format!("Question {n}"),
                &format!("Answer {n}"),
            )
            .unwrap();
        }
        let through = s
            .messages_page("long-parent", None, false, 2)
            .unwrap()
            .items
            .last()
            .unwrap()
            .id;
        let result = s.fork_session("long-parent", through, "long-fork");
        assert!(
            result.is_ok(),
            "Long saved history must remain branchable: {}",
            result.unwrap_err()
        );
        assert_eq!(
            s.lock()
                .unwrap()
                .query_row::<i64, _, _>(
                    "SELECT count(*) FROM messages WHERE session_id='long-fork'",
                    [],
                    |r| r.get(0)
                )
                .unwrap(),
            280
        );
        s.delete("long-parent").unwrap();
        drop(s);
        let s = SqliteStore::open(&path).unwrap();
        let c = s.lock().unwrap();
        let mut query = c
            .prepare("SELECT role,content FROM messages WHERE session_id='long-fork' ORDER BY id")
            .unwrap();
        let copied = query
            .query_map([], |r| Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?)))
            .unwrap()
            .collect::<Result<Vec<_>, _>>()
            .unwrap();
        let expected: Vec<_> = (0..140)
            .flat_map(|n| {
                [
                    ("user".to_owned(), format!("Question {n}")),
                    ("assistant".to_owned(), format!("Answer {n}")),
                ]
            })
            .collect();
        assert_eq!(copied, expected);
    }
    #[test]
    fn fork_copies_large_tool_metadata_exactly_without_parent_resume_authority() {
        let s = SqliteStore::open(std::path::Path::new(":memory:")).unwrap();
        s.create("parent").unwrap();
        s.commit_turn("parent", "Original question 世界", "Completed report")
            .unwrap();
        let through = s.messages_page("parent", None, false, 2).unwrap().items[1].id;
        let receipts: Vec<_> = (0..128).map(|n|serde_json::json!({"callId":format!("evidence-{n}"),"name":"read_text_file","target":format!("public-{n}.txt"),"status":"read","content":"Complete receipt 世界\n".repeat(600)})).collect();
        let mut metadata = serde_json::json!({
            "model":"fixture", "usage":{"inputTokens":100,"outputTokens":20},
            "context":dolores_core::ContextSummary::from_messages(&[],None),
            "agent":{"modelCalls":111,"tools":receipts,"usageByCall":[null]},
            "paused":{"reason":"checkpoint","task":"Original question 世界","receipts":[],"segments":2},
            "futureProvenance":{"literal":"Retain unknown fields 世界"}
        });
        let original = metadata.to_string();
        assert!(original.len() > 1024 * 1024);
        s.lock()
            .unwrap()
            .execute(
                "INSERT INTO turn_metadata(message_id,data) VALUES(?1,?2)",
                params![through, original],
            )
            .unwrap();
        s.commit_turn("parent", "Later question", "Excluded later answer")
            .unwrap();
        s.fork_session("parent", through, "fork").unwrap();
        metadata.as_object_mut().unwrap().remove("paused");
        let c = s.lock().unwrap();
        let copied: String = c.query_row("SELECT t.data FROM turn_metadata t JOIN messages m ON m.id=t.message_id WHERE m.session_id='fork'",[],|r|r.get(0)).unwrap();
        assert_eq!(
            serde_json::from_str::<serde_json::Value>(&copied).unwrap(),
            metadata
        );
        let decoded: dolores_core::TurnMetadata = serde_json::from_str(&copied).unwrap();
        assert!(decoded.paused.is_none());
        assert_eq!(decoded.agent.unwrap().tools.len(), 128);
        assert_eq!(
            c.query_row::<String, _, _>(
                "SELECT data FROM turn_metadata WHERE message_id=?1",
                [through],
                |r| r.get(0)
            )
            .unwrap(),
            original
        );
        assert_eq!(
            c.query_row::<i64, _, _>(
                "SELECT count(*) FROM messages WHERE session_id='fork'",
                [],
                |r| r.get(0)
            )
            .unwrap(),
            2
        );
    }
    #[test]
    fn failed_large_fork_rolls_back_every_row_and_retries_without_partial_history() {
        let s = SqliteStore::open(std::path::Path::new(":memory:")).unwrap();
        s.create("parent").unwrap();
        for n in 0..140 {
            s.commit_turn("parent", &format!("Question {n}"), "Saved answer")
                .unwrap();
        }
        let through = s
            .messages_page("parent", None, false, 2)
            .unwrap()
            .items
            .last()
            .unwrap()
            .id;
        s.save_draft("parent", "Keep original draft").unwrap();
        s.lock().unwrap().execute_batch("CREATE TRIGGER fail_large_fork BEFORE INSERT ON messages WHEN NEW.session_id='fork' AND (SELECT count(*) FROM messages WHERE session_id='fork')>=220 BEGIN SELECT RAISE(ABORT,'fixture storage failure'); END;").unwrap();
        assert!(s.fork_session("parent", through, "fork").is_err());
        {
            let c = s.lock().unwrap();
            for table in ["sessions", "messages"] {
                let column = if table == "sessions" {
                    "id"
                } else {
                    "session_id"
                };
                assert_eq!(
                    c.query_row::<i64, _, _>(
                        &format!("SELECT count(*) FROM {table} WHERE {column}='fork'"),
                        [],
                        |r| r.get(0)
                    )
                    .unwrap(),
                    0
                );
            }
            assert_eq!(
                c.query_row::<i64, _, _>(
                    "SELECT count(*) FROM messages WHERE session_id='parent'",
                    [],
                    |r| r.get(0)
                )
                .unwrap(),
                280
            );
            assert!(c
                .prepare("PRAGMA foreign_key_check")
                .unwrap()
                .query([])
                .unwrap()
                .next()
                .unwrap()
                .is_none());
            c.execute_batch("DROP TRIGGER fail_large_fork;").unwrap();
        }
        assert_eq!(s.saved_draft("parent").unwrap(), "Keep original draft");
        s.fork_session("parent", through, "fork").unwrap();
        assert_eq!(
            s.lock()
                .unwrap()
                .query_row::<i64, _, _>(
                    "SELECT count(*) FROM messages WHERE session_id='fork'",
                    [],
                    |r| r.get(0)
                )
                .unwrap(),
            280
        );
    }
    #[test]
    fn malformed_earlier_turn_rolls_back_the_branch_at_a_valid_assistant_boundary() {
        let s = SqliteStore::open(std::path::Path::new(":memory:")).unwrap();
        s.create("parent").unwrap();
        for _ in 0..3 {
            s.commit_turn("parent", "Question", "Answer").unwrap();
        }
        let through = s.messages_page("parent", None, false, 2).unwrap().items[1].id;
        s.lock().unwrap().execute("UPDATE messages SET role='assistant' WHERE id=(SELECT min(id) FROM messages WHERE session_id='parent')",[]).unwrap();
        assert!(s
            .fork_session("parent", through, "fork")
            .unwrap_err()
            .contains("incomplete turn"));
        assert_eq!(
            s.lock()
                .unwrap()
                .query_row::<i64, _, _>("SELECT count(*) FROM sessions WHERE id='fork'", [], |r| r
                    .get(0))
                .unwrap(),
            0
        );
    }
    #[test]
    fn compaction_is_always_enabled_for_new_and_legacy_chats_after_restart() {
        let dir = tempfile::tempdir().unwrap();
        let file = dir.path().join("context.db");
        let s = SqliteStore::open(&file).unwrap();
        s.create("new").unwrap();
        s.create("legacy").unwrap();
        s.lock()
            .unwrap()
            .execute(
                "INSERT INTO thread_context(session_id,auto_compact) VALUES('legacy',0)",
                [],
            )
            .unwrap();
        assert!(s.auto_compact("new").unwrap());
        assert!(s.auto_compact("legacy").unwrap());
        assert!(s
            .set_auto_compact("legacy", false)
            .unwrap_err()
            .contains("always enabled"));
        drop(s);
        let s = SqliteStore::open(&file).unwrap();
        assert!(s.auto_compact("legacy").unwrap());
        assert!(s.auto_compact("missing").is_err());
    }
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
        assert!(s.auto_compact("fork").unwrap());
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
