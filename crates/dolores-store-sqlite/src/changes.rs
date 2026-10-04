use super::{now, storage_error, SqliteStore};
use dolores_core::{ChangeDraft, ChangeSnapshot, FileChange, HistoryPage};
use rusqlite::{params, OptionalExtension, Row};

fn summary(row: &Row<'_>) -> rusqlite::Result<FileChange> {
    Ok(FileChange {
        id: row.get(0)?,
        created_at: row.get(1)?,
        target: row.get(2)?,
        status: row.get(3)?,
        reverts: row.get(4)?,
        bytes_before: row.get(5)?,
        bytes_after: row.get(6)?,
        before_exists: row.get(7)?,
        after_exists: row.get(8)?,
    })
}
const COLUMNS: &str = "id,created_at,target,status,reverts,length(CAST(before_text AS BLOB)),length(CAST(after_text AS BLOB)),before_exists,after_exists";

impl SqliteStore {
    pub(super) fn insert_change(&self, draft: &ChangeDraft) -> Result<i64, String> {
        if (draft.before == draft.after && draft.before_exists == draft.after_exists)
            || (!draft.before_exists && !draft.before.is_empty())
            || (!draft.after_exists && !draft.after.is_empty())
            || (!draft.before_exists && !draft.after_exists)
            || (!draft.after_exists && draft.reverts.is_none())
            || draft.before.len() > dolores_core::MAX_FILE_SNAPSHOT_BYTES
            || draft.after.len() > dolores_core::MAX_FILE_SNAPSHOT_BYTES
            || draft.root.len() > 32768
            || draft.target.is_empty()
            || draft.target.len() > 1024
            || draft.before.contains('\0')
            || draft.after.contains('\0')
        {
            return Err("Invalid change journal entry.".into());
        }
        let mut db = self.lock()?;
        let tx = db.transaction().map_err(storage_error)?;
        let bound: bool = tx
            .query_row(
                "SELECT EXISTS(SELECT 1 FROM session_workspaces WHERE session_id=?1 AND root=?2)",
                params![draft.session, draft.root],
                |r| r.get(0),
            )
            .map_err(storage_error)?;
        if !bound {
            return Err("Working folder no longer belongs to this chat.".into());
        }
        if let Some(id) = draft.reverts {
            let matches: bool = tx.query_row("SELECT EXISTS(SELECT 1 FROM file_changes WHERE id=?1 AND root=?2 AND target=?3 AND reverts IS NULL AND status IN ('pending','applied') AND before_text=?4 AND after_text=?5 AND before_exists=?6 AND after_exists=?7)", params![id,draft.root,draft.target,draft.after,draft.before,draft.after_exists,draft.before_exists], |r| r.get(0)).map_err(storage_error)?;
            if !matches {
                return Err("This change can no longer be reverted.".into());
            }
        }
        tx.execute("INSERT INTO file_changes(root,session_id,target,created_at,status,reverts,before_text,after_text,before_exists,after_exists) VALUES(?1,?2,?3,?4,'pending',?5,?6,?7,?8,?9)", params![draft.root,draft.session,draft.target,now(),draft.reverts,draft.before,draft.after,draft.before_exists,draft.after_exists]).map_err(storage_error)?;
        let id = tx.last_insert_rowid();
        tx.commit().map_err(storage_error)?;
        Ok(id)
    }
    pub(super) fn complete_change(&self, id: i64, applied: bool) -> Result<(), String> {
        let mut db = self.lock()?;
        let tx = db.transaction().map_err(storage_error)?;
        let original: Option<Option<i64>> = tx
            .query_row(
                "SELECT reverts FROM file_changes WHERE id=?1 AND status='pending'",
                [id],
                |r| r.get(0),
            )
            .optional()
            .map_err(storage_error)?;
        let original = original.ok_or("Change receipt is no longer pending.")?;
        tx.execute(
            "UPDATE file_changes SET status=?2 WHERE id=?1",
            params![id, if applied { "applied" } else { "notApplied" }],
        )
        .map_err(storage_error)?;
        if applied {
            if let Some(original) = original {
                let count = tx.execute("UPDATE file_changes SET status='reverted' WHERE id=?1 AND status IN ('pending','applied')", [original]).map_err(storage_error)?;
                if count != 1 {
                    return Err("Original change receipt is unavailable.".into());
                }
            }
        }
        tx.commit().map_err(storage_error)
    }
    pub(super) fn read_changes(
        &self,
        root: &str,
        cursor: Option<i64>,
    ) -> Result<HistoryPage<FileChange>, String> {
        let db = self.lock()?;
        let mut query = db.prepare(&format!("SELECT {COLUMNS} FROM file_changes WHERE root=?1 AND (?2 IS NULL OR id<?2) ORDER BY id DESC LIMIT 21")).map_err(storage_error)?;
        let mut items = query
            .query_map(params![root, cursor], summary)
            .map_err(storage_error)?
            .collect::<Result<Vec<_>, _>>()
            .map_err(storage_error)?;
        let has_older = items.len() > 20;
        items.truncate(20);
        Ok(HistoryPage {
            items,
            has_older,
            has_newer: cursor.is_some(),
        })
    }
    pub(super) fn read_change(&self, id: i64) -> Result<ChangeSnapshot, String> {
        self.lock()?
            .query_row(
                &format!(
                    "SELECT {COLUMNS},root,before_text,after_text FROM file_changes WHERE id=?1"
                ),
                [id],
                |r| {
                    Ok(ChangeSnapshot {
                        change: summary(r)?,
                        root: r.get(9)?,
                        before: r.get(10)?,
                        after: r.get(11)?,
                    })
                },
            )
            .map_err(storage_error)
    }
}
