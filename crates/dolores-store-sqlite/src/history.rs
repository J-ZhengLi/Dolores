use super::{storage_error, SqliteStore};
use dolores_core::{ExportFormat, HistoryPage, Role, Session, SessionCursor, StoredMessage};
use rusqlite::{params, OptionalExtension};
use std::io::Write;

fn checked_limit(limit: usize, max: usize) -> Result<i64, String> {
    if limit == 0 || limit > max {
        return Err(format!("History page size must be between 1 and {max}."));
    }
    Ok(limit as i64)
}

fn stored_message(row: &rusqlite::Row<'_>) -> rusqlite::Result<StoredMessage> {
    let role: String = row.get(1)?;
    Ok(StoredMessage {
        parts: row
            .get::<_, Option<String>>(5)?
            .map(|v| {
                serde_json::from_str(&v).map_err(|e| {
                    rusqlite::Error::FromSqlConversionFailure(
                        5,
                        rusqlite::types::Type::Text,
                        Box::new(e),
                    )
                })
            })
            .transpose()?
            .unwrap_or_default(),
        id: row.get(0)?,
        role: match role.as_str() {
            "user" => Role::User,
            "assistant" => Role::Assistant,
            _ => return Err(rusqlite::Error::InvalidQuery),
        },
        content: row.get(2)?,
        metadata: row
            .get::<_, Option<String>>(3)?
            .map(|data| {
                serde_json::from_str(&data).map_err(|error| {
                    rusqlite::Error::FromSqlConversionFailure(
                        3,
                        rusqlite::types::Type::Text,
                        Box::new(error),
                    )
                })
            })
            .transpose()?,
        feedback: row
            .get::<_, Option<String>>(4)?
            .map(|v| {
                serde_json::from_str(&v).map_err(|error| {
                    rusqlite::Error::FromSqlConversionFailure(
                        4,
                        rusqlite::types::Type::Text,
                        Box::new(error),
                    )
                })
            })
            .transpose()?,
    })
}

fn markdown_block(output: &mut dyn Write, text: &str) -> std::io::Result<()> {
    let fence = "`".repeat(
        text.split(|c| c != '`')
            .map(str::len)
            .max()
            .unwrap_or(0)
            .max(2)
            + 1,
    );
    writeln!(output, "{fence}text\n{text}\n{fence}\n")
}
impl SqliteStore {
    pub(super) fn read_sessions_page(
        &self,
        cursor: Option<SessionCursor>,
        newer: bool,
        limit: usize,
    ) -> Result<HistoryPage<Session>, String> {
        let limit = checked_limit(limit, 100)?;
        if newer && cursor.is_none() {
            return Err("Newer history requires a cursor.".into());
        }
        let connection = self.lock()?;
        let snapshot = connection.unchecked_transaction().map_err(storage_error)?;
        let (predicate, order) = if newer {
            (
                "(updated_at>?1 OR (updated_at=?1 AND id<?2))",
                "updated_at ASC,id DESC",
            )
        } else {
            (
                "(?1 IS NULL OR updated_at<?1 OR (updated_at=?1 AND id>?2))",
                "updated_at DESC,id ASC",
            )
        };
        let mut statement = snapshot.prepare(&format!("SELECT id,title,updated_at FROM sessions WHERE {predicate} ORDER BY {order} LIMIT ?3")).map_err(storage_error)?;
        let mut items = statement
            .query_map(
                params![
                    cursor.as_ref().map(|c| c.updated_at),
                    cursor.as_ref().map(|c| &c.id),
                    limit
                ],
                |row| {
                    Ok(Session {
                        id: row.get(0)?,
                        title: row.get(1)?,
                        updated_at: row.get(2)?,
                    })
                },
            )
            .map_err(storage_error)?
            .collect::<Result<Vec<_>, _>>()
            .map_err(storage_error)?;
        if newer {
            items.reverse();
        }
        let has_older = if let Some(last) = items.last() {
            snapshot.query_row("SELECT EXISTS(SELECT 1 FROM sessions WHERE updated_at<?1 OR (updated_at=?1 AND id>?2))", params![last.updated_at,last.id], |row| row.get(0)).map_err(storage_error)?
        } else {
            false
        };
        let has_newer = if let Some(first) = items.first() {
            snapshot.query_row("SELECT EXISTS(SELECT 1 FROM sessions WHERE updated_at>?1 OR (updated_at=?1 AND id<?2))", params![first.updated_at,first.id], |row| row.get(0)).map_err(storage_error)?
        } else {
            cursor.is_some()
        };
        Ok(HistoryPage {
            items,
            has_older,
            has_newer,
        })
    }
    pub(super) fn read_messages_page(
        &self,
        id: &str,
        cursor: Option<i64>,
        newer: bool,
        limit: usize,
    ) -> Result<HistoryPage<StoredMessage>, String> {
        let limit = checked_limit(limit, 80)?;
        if cursor.is_some_and(|c| c <= 0) || (newer && cursor.is_none()) {
            return Err("Invalid message history cursor.".into());
        }
        let connection = self.lock()?;
        let snapshot = connection.unchecked_transaction().map_err(storage_error)?;
        let exists: bool = snapshot
            .query_row(
                "SELECT EXISTS(SELECT 1 FROM sessions WHERE id=?1)",
                [id],
                |row| row.get(0),
            )
            .map_err(storage_error)?;
        if !exists {
            return Err("Conversation no longer exists.".into());
        }
        let (predicate, order) = if newer {
            ("id>?2", "ASC")
        } else {
            ("(?2 IS NULL OR id<?2)", "DESC")
        };
        let mut statement = snapshot.prepare(&format!("SELECT id,role,content,(SELECT data FROM turn_metadata WHERE message_id=messages.id),(SELECT data FROM task_feedback WHERE message_id=messages.id),(SELECT data FROM message_parts WHERE message_id=messages.id) FROM messages WHERE session_id=?1 AND {predicate} ORDER BY id {order} LIMIT ?3")).map_err(storage_error)?;
        let mut items = statement
            .query_map(params![id, cursor, limit], stored_message)
            .map_err(storage_error)?
            .collect::<Result<Vec<_>, _>>()
            .map_err(storage_error)?;
        if !newer {
            items.reverse();
        }
        let has_older = if let Some(first) = items.first() {
            snapshot
                .query_row(
                    "SELECT EXISTS(SELECT 1 FROM messages WHERE session_id=?1 AND id<?2)",
                    params![id, first.id],
                    |row| row.get(0),
                )
                .map_err(storage_error)?
        } else {
            false
        };
        let has_newer = if let Some(last) = items.last() {
            snapshot
                .query_row(
                    "SELECT EXISTS(SELECT 1 FROM messages WHERE session_id=?1 AND id>?2)",
                    params![id, last.id],
                    |row| row.get(0),
                )
                .map_err(storage_error)?
        } else {
            cursor.is_some()
        };
        Ok(HistoryPage {
            items,
            has_older,
            has_newer,
        })
    }
    pub(super) fn write_export(
        &self,
        id: &str,
        format: ExportFormat,
        output: &mut dyn Write,
    ) -> Result<u64, String> {
        let connection = self.lock()?;
        let snapshot = connection.unchecked_transaction().map_err(storage_error)?;
        let session = snapshot
            .query_row(
                "SELECT id,title,updated_at FROM sessions WHERE id=?1",
                [id],
                |row| {
                    Ok(Session {
                        id: row.get(0)?,
                        title: row.get(1)?,
                        updated_at: row.get(2)?,
                    })
                },
            )
            .optional()
            .map_err(storage_error)?
            .ok_or("Conversation no longer exists.")?;
        let error = |_| "Could not write the conversation export.".to_string();
        match format {
            ExportFormat::Json => {
                output
                    .write_all(b"{\"exportVersion\":1,\"session\":")
                    .map_err(error)?;
                serde_json::to_writer(&mut *output, &session).map_err(storage_error)?;
                output.write_all(b",\"messages\":[").map_err(error)?;
            }
            ExportFormat::Markdown => {
                // Plain title, fenced message bodies: model Markdown/HTML stays inert.
                writeln!(output, "# Dolores conversation\n\nSession: {}\nUpdated: {} (Unix milliseconds)\n\nTitle:\n", session.id, session.updated_at).map_err(error)?;
                markdown_block(output, &session.title).map_err(error)?;
            }
        }
        let mut statement = snapshot
            .prepare("SELECT id,role,content,(SELECT data FROM turn_metadata WHERE message_id=messages.id),(SELECT data FROM task_feedback WHERE message_id=messages.id),(SELECT data FROM message_parts WHERE message_id=messages.id) FROM messages WHERE session_id=?1 ORDER BY id ASC")
            .map_err(storage_error)?;
        let mut rows = statement.query([id]).map_err(storage_error)?;
        let mut count = 0;
        while let Some(row) = rows.next().map_err(storage_error)? {
            let message = stored_message(row).map_err(storage_error)?;
            match format {
                ExportFormat::Json => {
                    if count > 0 {
                        output.write_all(b",").map_err(error)?;
                    }
                    serde_json::to_writer(&mut *output, &message).map_err(storage_error)?;
                }
                ExportFormat::Markdown => {
                    let role = if message.role == Role::User {
                        "You"
                    } else {
                        "Dolores"
                    };
                    writeln!(output, "## {role}\n").map_err(error)?;
                    markdown_block(output, &message.content).map_err(error)?;
                    if !message.parts.is_empty() {
                        writeln!(
                            output,
                            "Local attachment references (export snapshot bytes separately):\n"
                        )
                        .map_err(error)?;
                        markdown_block(
                            output,
                            &serde_json::to_string(&message.parts).map_err(storage_error)?,
                        )
                        .map_err(error)?;
                    }
                    if let Some(feedback) = &message.feedback {
                        writeln!(output, "Local user feedback (not verification):\n")
                            .map_err(error)?;
                        markdown_block(
                            output,
                            &serde_json::to_string(feedback).map_err(storage_error)?,
                        )
                        .map_err(error)?;
                    }
                    if let Some(metadata) = &message.metadata {
                        writeln!(output, "Request usage and context:\n").map_err(error)?;
                        markdown_block(
                            output,
                            &serde_json::to_string(metadata).map_err(storage_error)?,
                        )
                        .map_err(error)?;
                    }
                }
            }
            count += 1;
        }
        if matches!(format, ExportFormat::Json) {
            output.write_all(b"],\"comparisons\":[").map_err(error)?;
        }
        let mut comparisons = snapshot
            .prepare("SELECT data FROM context_comparisons WHERE session_id=?1 ORDER BY id ASC")
            .map_err(storage_error)?;
        let mut rows = comparisons.query([id]).map_err(storage_error)?;
        let mut first = true;
        while let Some(row) = rows.next().map_err(storage_error)? {
            let run: dolores_core::ContextComparison =
                serde_json::from_str(&row.get::<_, String>(0).map_err(storage_error)?)
                    .map_err(storage_error)?;
            run.validate()?;
            let receipt = serde_json::json!({"record":run,"summary":run.summary()});
            match format {
                ExportFormat::Json => {
                    if !first {
                        output.write_all(b",").map_err(error)?;
                    }
                    serde_json::to_writer(&mut *output, &receipt).map_err(storage_error)?;
                }
                ExportFormat::Markdown => {
                    writeln!(output, "## Frozen context comparison\n").map_err(error)?;
                    markdown_block(
                        output,
                        &serde_json::to_string(&receipt).map_err(storage_error)?,
                    )
                    .map_err(error)?;
                }
            }
            first = false;
        }
        if matches!(format, ExportFormat::Json) {
            output.write_all(b"]}\n").map_err(error)?;
        }
        Ok(count)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use dolores_core::SessionStore;

    #[test]
    fn tied_session_pages_cover_all_rows_and_reverse_without_duplicates() {
        let directory = tempfile::tempdir().unwrap();
        let store = SqliteStore::open(&directory.path().join("history.db")).unwrap();
        for index in 0..137 {
            store.create(&format!("session-{index:03}")).unwrap();
        }
        store
            .lock()
            .unwrap()
            .execute("UPDATE sessions SET updated_at=42", [])
            .unwrap();
        let first = store.sessions_page(None, false, 50).unwrap();
        assert!(first.has_older && !first.has_newer);
        let cursor = |s: &Session| SessionCursor {
            id: s.id.clone(),
            updated_at: s.updated_at,
        };
        let second = store
            .sessions_page(Some(cursor(first.items.last().unwrap())), false, 50)
            .unwrap();
        assert!(second.has_older && second.has_newer);
        let third = store
            .sessions_page(Some(cursor(second.items.last().unwrap())), false, 50)
            .unwrap();
        assert_eq!(third.items.len(), 37);
        assert!(!third.has_older && third.has_newer);
        let backwards = store
            .sessions_page(Some(cursor(third.items.first().unwrap())), true, 50)
            .unwrap();
        assert_eq!(
            backwards.items.iter().map(|s| &s.id).collect::<Vec<_>>(),
            second.items.iter().map(|s| &s.id).collect::<Vec<_>>()
        );
        let ids = first
            .items
            .iter()
            .chain(&second.items)
            .chain(&third.items)
            .map(|s| &s.id)
            .collect::<std::collections::BTreeSet<_>>();
        assert_eq!(ids.len(), 137);
        store.delete(&second.items.last().unwrap().id).unwrap();
        assert_eq!(
            store
                .sessions_page(Some(cursor(second.items.last().unwrap())), false, 50)
                .unwrap()
                .items
                .len(),
            37
        );
        assert!(store.sessions_page(None, false, 101).is_err());
    }

    #[test]
    fn long_history_browses_both_directions_and_exports_every_message_after_restart() {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("history.db");
        let store = SqliteStore::open(&path).unwrap();
        store.create("long").unwrap();
        for turn in 0..123 {
            store
                .commit_turn(
                    "long",
                    &format!("你好 {turn}"),
                    &format!("answer {turn}\n```\n~~~~\n<script>text</script>"),
                )
                .unwrap();
        }
        drop(store);
        let store = SqliteStore::open(&path).unwrap();
        assert_eq!(store.messages("long").unwrap().len(), 80); // Context stays bounded.
        let latest = store.messages_page("long", None, false, 80).unwrap();
        assert!(latest.has_older && !latest.has_newer);
        let middle = store
            .messages_page("long", Some(latest.items[0].id), false, 80)
            .unwrap();
        let newer = store
            .messages_page("long", Some(middle.items.last().unwrap().id), true, 80)
            .unwrap();
        assert_eq!(newer.items[0].id, latest.items[0].id);
        let mut cursor = None;
        let mut seen = std::collections::BTreeSet::new();
        loop {
            let page = store.messages_page("long", cursor, false, 80).unwrap();
            for item in &page.items {
                assert!(seen.insert(item.id));
            }
            cursor = page.items.first().map(|m| m.id);
            if !page.has_older {
                break;
            }
        }
        assert_eq!(seen.len(), 246);
        let mut json = Vec::new();
        assert_eq!(
            store
                .export_conversation("long", ExportFormat::Json, &mut json)
                .unwrap(),
            246
        );
        let value: serde_json::Value = serde_json::from_slice(&json).unwrap();
        assert_eq!(value["messages"].as_array().unwrap().len(), 246);
        assert_eq!(value["messages"][0]["content"], "你好 0");
        assert!(value["messages"][245]["content"]
            .as_str()
            .unwrap()
            .starts_with("answer 122"));
        assert!(value.get("preferences").is_none());
        let mut markdown = Vec::new();
        assert_eq!(
            store
                .export_conversation("long", ExportFormat::Markdown, &mut markdown)
                .unwrap(),
            246
        );
        let markdown = String::from_utf8(markdown).unwrap();
        assert_eq!(markdown.matches("## You\n").count(), 123);
        assert!(markdown.contains("````text\nanswer 122"));
        assert!(store.messages_page("deleted", None, false, 80).is_err());
        assert!(store.messages_page("long", None, false, 0).is_err());
        struct BrokenWriter;
        impl Write for BrokenWriter {
            fn write(&mut self, _: &[u8]) -> std::io::Result<usize> {
                Err(std::io::Error::other("fixture"))
            }
            fn flush(&mut self) -> std::io::Result<()> {
                Ok(())
            }
        }
        assert!(store
            .export_conversation("long", ExportFormat::Json, &mut BrokenWriter)
            .is_err());
    }
}
