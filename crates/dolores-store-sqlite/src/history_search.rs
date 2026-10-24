use super::*;
use dolores_core::history_search::*;
use dolores_core::{SessionWorkspace, WorkspaceKind};
use rusqlite::{named_params, OptionalExtension, TransactionBehavior};
use std::time::{Duration, Instant};

struct Budget<'a>(&'a Connection);
impl<'a> Budget<'a> {
    fn new(c: &'a Connection, limit: Duration) -> Self {
        let started = Instant::now();
        c.progress_handler(
            if limit.is_zero() { 1 } else { 1000 },
            Some(move || started.elapsed() >= limit),
        );
        Self(c)
    }
}
impl Drop for Budget<'_> {
    fn drop(&mut self) {
        self.0.progress_handler(0, None::<fn() -> bool>);
    }
}

pub(super) fn migrate(c: &Connection) -> Result<(), String> {
    let _budget = Budget::new(c, Duration::from_secs(30));
    c.execute_batch(include_str!("history_search_schema.sql"))
        .map_err(storage_error)
}
fn organization(c: &Connection, session: &str) -> Result<Organization, String> {
    let exists: bool = c
        .query_row(
            "SELECT EXISTS(SELECT 1 FROM sessions WHERE id=?1)",
            [session],
            |r| r.get(0),
        )
        .map_err(storage_error)?;
    if !exists {
        return Err(
            "Conversation no longer exists. Refresh history; no other chat was changed.".into(),
        );
    }
    Ok(c.query_row(
        "SELECT revision,pinned,archived FROM history_organization WHERE session=?1",
        [session],
        |r| {
            Ok(Organization {
                revision: r.get(0)?,
                pinned: r.get(1)?,
                archived: r.get(2)?,
            })
        },
    )
    .optional()
    .map_err(storage_error)?
    .unwrap_or_default())
}
pub(super) fn read(store: &SqliteStore, session: &str) -> Result<Organization, String> {
    organization(&*store.lock()?, session)
}
pub(super) fn edit(
    store: &SqliteStore,
    session: &str,
    edit: &Organization,
) -> Result<Organization, String> {
    let mut c = store.lock()?;
    let tx = c
        .transaction_with_behavior(TransactionBehavior::Immediate)
        .map_err(storage_error)?;
    let mut current = organization(&tx, session)?;
    if current.revision != edit.revision {
        return Err("Conversation organization changed. Refresh its pin/archive state and retry; drafts and task input remain.".into());
    }
    if current.pinned == edit.pinned && current.archived == edit.archived {
        return Ok(current);
    }
    current = Organization {
        revision: current
            .revision
            .checked_add(1)
            .ok_or("Organization revision exhausted; history remains available.")?,
        pinned: edit.pinned,
        archived: edit.archived,
    };
    tx.execute("INSERT INTO history_organization(session,revision,pinned,archived) VALUES(?1,?2,?3,?4) ON CONFLICT(session) DO UPDATE SET revision=excluded.revision,pinned=excluded.pinned,archived=excluded.archived",params![session,current.revision,current.pinned,current.archived]).map_err(storage_error)?;
    tx.commit().map_err(storage_error)?;
    Ok(current)
}
pub(super) fn rebuild(store: &SqliteStore) -> Result<(), String> {
    rebuild_limited(store, Duration::from_secs(30))
}
pub(super) fn rebuild_limited(store: &SqliteStore, limit: Duration) -> Result<(), String> {
    let mut c = store.lock()?;
    let tx = c
        .transaction_with_behavior(TransactionBehavior::Immediate)
        .map_err(storage_error)?;
    let budget = Budget::new(&tx, limit);
    tx.execute(
        "INSERT INTO history_titles(history_titles) VALUES('rebuild')",
        [],
    )
    .map_err(|error| rebuild_error(error, limit))?;
    tx.execute(
        "INSERT INTO history_messages(history_messages) VALUES('rebuild')",
        [],
    )
    .map_err(|error| rebuild_error(error, limit))?;
    drop(budget);
    tx.commit().map_err(storage_error)
}
pub(super) fn search(store: &SqliteStore, query: &Query) -> Result<Page, String> {
    search_limited(store, query, Duration::from_secs(1))
}
pub(super) fn search_limited(
    store: &SqliteStore,
    query: &Query,
    limit: Duration,
) -> Result<Page, String> {
    query.validate()?;
    let c = store.lock()?;
    let tx = c.unchecked_transaction().map_err(storage_error)?;
    let _budget = Budget::new(&tx, limit);
    let text = query.filter.text.trim();
    // Trigrams cover multilingual literal substrings. One/two characters use
    // canonical text directly instead of silently producing an empty FTS result.
    let indexed = text.chars().count() >= 3;
    let phrase = format!("\"{}\"", text.replace('"', "\"\""));
    let candidate = if indexed {
        "s.rowid IN (SELECT rowid FROM history_titles WHERE history_titles MATCH :phrase) OR s.id IN (SELECT m.session_id FROM history_messages f JOIN messages m ON m.id=f.rowid WHERE history_messages MATCH :phrase)"
    } else {
        "instr(lower(s.title),lower(:text))>0 OR EXISTS(SELECT 1 FROM messages m WHERE m.session_id=s.id AND instr(lower(m.content),lower(:text))>0)"
    };
    // All values are parameters. FTS syntax is escaped as a single literal phrase.
    let sql=format!("SELECT s.id,substr(s.title,1,160),s.updated_at,COALESCE(w.kind,'side'),w.root,COALESCE(o.revision,0),COALESCE(o.pinned,0),COALESCE(o.archived,0) FROM sessions s LEFT JOIN session_workspaces w ON w.session_id=s.id LEFT JOIN history_organization o ON o.session=s.id WHERE (:empty OR ({candidate})) AND (:archive='all' OR COALESCE(o.archived,0)=(:archive='archived')) AND (:pinned IS NULL OR COALESCE(o.pinned,0)=:pinned) AND (:kind IS NULL OR COALESCE(w.kind,'side')=:kind) AND (:root IS NULL OR w.root=:root) AND (:cursor IS NULL OR COALESCE(o.pinned,0)<:cpin OR (COALESCE(o.pinned,0)=:cpin AND (s.updated_at<:ctime OR (s.updated_at=:ctime AND s.id>:cursor)))) ORDER BY COALESCE(o.pinned,0) DESC,s.updated_at DESC,s.id ASC LIMIT 51");
    let mut statement = tx.prepare(&sql).map_err(search_failure)?;
    let archive = match query.filter.archive {
        ArchiveFilter::All => "all",
        ArchiveFilter::Archived => "archived",
        ArchiveFilter::Visible => "visible",
    };
    // Only bind names present in the chosen query; SQLite refuses unused names.
    let mut parameters: Vec<(&str, &dyn rusqlite::ToSql)> = vec![];
    let empty = text.is_empty();
    let cp = query.cursor.as_ref().map(|v| v.pinned);
    let ct = query.cursor.as_ref().map(|v| v.updated_at);
    let ci = query.cursor.as_ref().map(|v| v.id.as_str());
    parameters.extend_from_slice(&[
        (":empty", &empty),
        (":archive", &archive),
        (":pinned", &query.filter.pinned),
        (":kind", &query.filter.kind),
        (":root", &query.filter.root),
        (":cursor", &ci),
        (":cpin", &cp),
        (":ctime", &ct),
    ]);
    if indexed {
        parameters.push((":phrase", &phrase));
    } else {
        parameters.push((":text", &text));
    }
    let mut items = statement
        .query_map(parameters.as_slice(), |r| {
            Ok(Hit {
                session: Session {
                    id: r.get(0)?,
                    title: r.get(1)?,
                    updated_at: r.get(2)?,
                },
                workspace: SessionWorkspace {
                    kind: match r.get::<_, String>(3)?.as_str() {
                        "project" => WorkspaceKind::Project,
                        "temporary" => WorkspaceKind::Temporary,
                        _ => WorkspaceKind::Side,
                    },
                    root: r.get(4)?,
                },
                organization: Organization {
                    revision: r.get(5)?,
                    pinned: r.get(6)?,
                    archived: r.get(7)?,
                },
                excerpt: None,
            })
        })
        .map_err(search_failure)?
        .collect::<Result<Vec<_>, _>>()
        .map_err(search_failure)?;
    let more = items.len() > 50;
    items.truncate(50);
    for item in &mut items {
        if !text.is_empty() {
            item.excerpt=tx.query_row("SELECT id,substr(content,max(1,instr(lower(content),lower(:text))-60),240) FROM messages WHERE session_id=:session AND instr(lower(content),lower(:text))>0 ORDER BY id DESC LIMIT 1",named_params!{":session":item.session.id,":text":text},|r|Ok(Excerpt{message:r.get(0)?,text:r.get(1)?})).optional().map_err(storage_error)?;
        }
    }
    let next_cursor = if more {
        items.last().map(|v| Cursor {
            filter: query.filter.clone(),
            pinned: v.organization.pinned,
            updated_at: v.session.updated_at,
            id: v.session.id.clone(),
        })
    } else {
        None
    };
    Ok(Page { items, next_cursor })
}
fn search_error() -> String {
    "Local search index is unavailable. Rebuild search or browse saved conversations; history and drafts remain.".into()
}
fn rebuild_error(error: rusqlite::Error, limit: Duration) -> String {
    if matches!(error,rusqlite::Error::SqliteFailure(e,_) if e.code==rusqlite::ErrorCode::OperationInterrupted)
    {
        format!("Search rebuild reached its {}-second allowance. The previous index and history remain; browse saved conversations and retry rebuilding when fewer tasks are active.",limit.as_secs())
    } else {
        "Could not rebuild local search. The previous index and history remain; restore storage access and retry, or browse saved conversations.".into()
    }
}
fn search_failure(error: rusqlite::Error) -> String {
    if matches!(error,rusqlite::Error::SqliteFailure(e,_) if e.code==rusqlite::ErrorCode::OperationInterrupted)
    {
        "Local history search reached its one-second allowance. Narrow the text or workspace filters and retry; saved history and drafts remain.".into()
    } else {
        search_error()
    }
}
