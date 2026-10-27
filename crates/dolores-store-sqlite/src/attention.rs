use super::*;
use dolores_core::attention::*;
use rusqlite::{OptionalExtension, TransactionBehavior};
use serde_json::{json, Value};

pub(super) fn migrate(c: &Connection) -> Result<(), String> {
    c.execute_batch(include_str!("attention_schema.sql"))
        .map_err(storage_error)
}
fn prefs(c: &Connection) -> Result<Preferences, String> {
    let raw: Option<String> = c
        .query_row(
            "SELECT data FROM attention_preferences WHERE id=1",
            [],
            |r| r.get(0),
        )
        .optional()
        .map_err(storage_error)?;
    raw.map(|v| serde_json::from_str(&v).map_err(storage_error))
        .transpose()
        .map(|v| v.unwrap_or_default())
}
fn row(r: &rusqlite::Row<'_>) -> rusqlite::Result<Notice> {
    let kind: String = r.get(4)?;
    let delivery: String = r.get(7)?;
    let parse = |v: &str| {
        serde_json::from_str(&format!("\"{v}\"")).map_err(|e| {
            rusqlite::Error::FromSqlConversionFailure(4, rusqlite::types::Type::Text, Box::new(e))
        })
    };
    Ok(Notice {
        id: r.get(0)?,
        title: r.get(9)?,
        details: r.get(10)?,
        severity: r.get(11)?,
        session: r.get::<_, Option<String>>(1)?.unwrap_or_default(),
        run: r.get::<_, Option<String>>(2)?.unwrap_or_default(),
        sequence: r.get(3)?,
        kind: parse(&kind)?,
        created_at: r.get(5)?,
        seen: r.get(6)?,
        delivery: serde_json::from_str(&format!("\"{delivery}\"")).map_err(|e| {
            rusqlite::Error::FromSqlConversionFailure(7, rusqlite::types::Type::Text, Box::new(e))
        })?,
    })
}
fn inspect(c: &Connection, id: &str) -> Result<Notice, String> {
    if id.len() != 36 {
        return Err(
            "Attention target is invalid. Open Attention and choose a retained entry.".into(),
        );
    }
    c.query_row("SELECT id,session,run,sequence,kind,created_at,seen,delivery,rowid,title,details,severity FROM attention_notices WHERE id=?1",[id],row).optional().map_err(storage_error)?.ok_or("This attention entry is no longer retained. Search the conversation or inspect its run history; no decision was applied.".into())
}
fn label<T: serde::Serialize>(v: T) -> Result<String, String> {
    Ok(serde_json::to_value(v)
        .map_err(storage_error)?
        .as_str()
        .ok_or("Invalid attention value.")?
        .into())
}
/// Scan saved events independently: a broken notification subsystem must never
/// fail a turn save, overwrite a result or consume approval authority.
fn collect(c: &Connection, foreground_session: Option<&str>) -> Result<bool, String> {
    let preferences = prefs(c)?;
    let mut st=c.prepare("SELECT r.id,r.thread,COALESCE(a.sequence,0) FROM runs r LEFT JOIN attention_sources a ON a.run=r.id WHERE json_extract(r.data,'$.sequence')>COALESCE(a.sequence,0) ORDER BY r.rowid LIMIT 16").map_err(storage_error)?;
    let owners = st
        .query_map([], |r| {
            Ok((
                r.get::<_, String>(0)?,
                r.get::<_, String>(1)?,
                r.get::<_, u32>(2)?,
            ))
        })
        .map_err(storage_error)?
        .collect::<Result<Vec<_>, _>>()
        .map_err(storage_error)?;
    for (run, session, from) in &owners {
        let mut events=c.prepare("SELECT sequence,json_extract(data,'$.kind'),json_extract(data,'$.state') FROM run_events WHERE run_id=?1 AND sequence>?2 ORDER BY sequence LIMIT 256").map_err(storage_error)?;
        let rows = events
            .query_map(params![run, from], |r| {
                Ok((
                    r.get::<_, u32>(0)?,
                    r.get::<_, String>(1)?,
                    r.get::<_, String>(2)?,
                ))
            })
            .map_err(storage_error)?
            .collect::<Result<Vec<_>, _>>()
            .map_err(storage_error)?;
        let mut sequence = *from;
        for (seq, event, state) in rows {
            sequence = seq;
            // A foreground event is already visible in its task. Advance the
            // cursor without retaining a notice that could surface after blur.
            if foreground_session == Some(session.as_str()) {
                continue;
            }
            if let Some(kind) = Kind::from_event(&event, &state) {
                let delivery = if preferences.permits(kind) {
                    "pending"
                } else {
                    "suppressed"
                };
                c.execute("INSERT OR IGNORE INTO attention_notices(id,session,run,sequence,kind,created_at,seen,delivery) SELECT ?1,?2,?3,?4,?5,?6,0,?7 WHERE EXISTS(SELECT 1 FROM sessions WHERE id=?2)",params![uuid::Uuid::new_v4().to_string(),session,run,seq,label(kind)?,now(),delivery]).map_err(storage_error)?;
            }
        }
        c.execute("INSERT INTO attention_sources(run,sequence) VALUES(?1,?2) ON CONFLICT(run) DO UPDATE SET sequence=excluded.sequence",params![run,sequence]).map_err(storage_error)?;
    }
    // Keep 256 newest attention entries. Source cursors prevent pruned notices
    // from returning; the underlying run evidence remains available.
    let removed=c.execute("DELETE FROM attention_notices WHERE kind!='app' AND rowid NOT IN (SELECT rowid FROM attention_notices WHERE kind!='app' ORDER BY rowid DESC LIMIT 256)",[]).map_err(storage_error)?;
    if removed > 0 {
        c.execute("INSERT INTO attention_preferences(id,data,omitted) VALUES(1,?1,?2) ON CONFLICT(id) DO UPDATE SET omitted=omitted+excluded.omitted",params![serde_json::to_string(&preferences).map_err(storage_error)?,removed]).map_err(storage_error)?;
    }
    c.query_row("SELECT EXISTS(SELECT 1 FROM runs r LEFT JOIN attention_sources a ON a.run=r.id WHERE json_extract(r.data,'$.sequence')>COALESCE(a.sequence,0))",[],|r|r.get(0)).map_err(storage_error)
}
pub(super) fn recover(store: &SqliteStore) -> Result<(), String> {
    store
        .lock()?
        .execute(
            "UPDATE attention_notices SET delivery='uncertain' WHERE delivery='claimed'",
            [],
        )
        .map_err(storage_error)?;
    Ok(())
}
pub(super) fn call(store: &SqliteStore, request: &Request) -> Result<Value, String> {
    let mut c = store.lock()?;
    let tx = c
        .transaction_with_behavior(TransactionBehavior::Immediate)
        .map_err(storage_error)?;
    let result = match request {
        Request::Post {
            source,
            title,
            details,
            severity,
        } => {
            if source.is_empty()
                || source.len() > 80
                || !source
                    .bytes()
                    .all(|b| b.is_ascii_alphanumeric() || b"-_.".contains(&b))
                || title.trim().is_empty()
                || title.len() > 160
                || details.len() > 4096
                || title.chars().any(char::is_control)
                || details.contains('\0')
                || !["info", "success", "warning", "error"].contains(&severity.as_str())
                || dolores_core::credential_like(title)
                || dolores_core::credential_like(details)
            {
                return Err(
                    "Notification content is invalid. Keep a brief nonsecret summary.".into(),
                );
            }
            let recent:Option<String>=tx.query_row("SELECT id FROM attention_notices WHERE kind='app' AND source_key=?1 AND created_at>?2 ORDER BY rowid DESC LIMIT 1",params![source,now()-5000],|r|r.get(0)).optional().map_err(storage_error)?;
            let id = recent.unwrap_or_else(|| uuid::Uuid::new_v4().to_string());
            tx.execute("INSERT INTO attention_notices(id,sequence,kind,created_at,seen,delivery,title,details,severity,source_key) VALUES(?1,0,'app',?2,0,'suppressed',?3,?4,?5,?6) ON CONFLICT(id) DO UPDATE SET created_at=excluded.created_at,seen=0,title=excluded.title,details=excluded.details,severity=excluded.severity",params![id,now(),title,details,severity,source]).map_err(storage_error)?;
            tx.execute("DELETE FROM attention_notices WHERE kind='app' AND rowid NOT IN(SELECT rowid FROM attention_notices WHERE kind='app' ORDER BY created_at DESC,rowid DESC LIMIT 100)",[]).map_err(storage_error)?;
            json!(inspect(&tx, &id)?)
        }
        Request::ClearRead => {
            tx.execute("DELETE FROM attention_notices WHERE seen=1", [])
                .map_err(storage_error)?;
            Value::Null
        }
        Request::SeenAll => {
            tx.execute("UPDATE attention_notices SET seen=1,delivery=CASE WHEN delivery='pending' THEN 'suppressed' ELSE delivery END",[]).map_err(storage_error)?;
            Value::Null
        }
        Request::Preferences => json!(prefs(&tx)?),
        Request::SavePreferences {
            preferences,
            foreground_session,
        } => {
            // Advance cursors under the old policy before changing opt-in.
            // New opt-in never retroactively delivers old suppressed notices.
            if collect(&tx, foreground_session.as_deref())? {
                return Err("Attention is catching up. Refresh it before changing notification preferences; the current choice remains.".into());
            }
            let old = prefs(&tx)?;
            if old.revision != preferences.revision {
                return Err("Notification preferences changed. Refresh Attention and retry; task state remains.".into());
            }
            let mut value = preferences.clone();
            value.revision = old
                .revision
                .checked_add(1)
                .ok_or("Notification preference revision exhausted.")?;
            tx.execute("INSERT INTO attention_preferences(id,data) VALUES(1,?1) ON CONFLICT(id) DO UPDATE SET data=excluded.data",[serde_json::to_string(&value).map_err(storage_error)?]).map_err(storage_error)?;
            if !value.enabled {
                tx.execute(
                    "UPDATE attention_notices SET delivery='suppressed' WHERE delivery='pending'",
                    [],
                )
                .map_err(storage_error)?;
            }
            json!(value)
        }
        Request::Poll {
            cursor,
            foreground_session,
        } => {
            if cursor.is_some_and(|v| v <= 0) {
                return Err("Attention page is invalid. Refresh from the beginning.".into());
            }
            let more_sources = collect(&tx, foreground_session.as_deref())?;
            let mut st=tx.prepare("SELECT id,session,run,sequence,kind,created_at,seen,delivery,rowid,title,details,severity FROM attention_notices WHERE (?1 IS NULL OR rowid<?1) ORDER BY rowid DESC LIMIT 51").map_err(storage_error)?;
            let mut rows = st
                .query_map([cursor], |r| Ok((row(r)?, r.get::<_, i64>(8)?)))
                .map_err(storage_error)?
                .collect::<Result<Vec<_>, _>>()
                .map_err(storage_error)?;
            let more = rows.len() > 50;
            rows.truncate(50);
            for (notice, _) in &mut rows {
                if notice.kind == Kind::App {
                    continue;
                }
                notice.title = tx
                    .query_row(
                        "SELECT substr(title,1,160) FROM sessions WHERE id=?1",
                        [&notice.session],
                        |r| r.get(0),
                    )
                    .optional()
                    .map_err(storage_error)?;
            }
            let next_cursor = if more {
                rows.last().map(|(_, v)| *v)
            } else {
                None
            };
            let omitted = tx
                .query_row(
                    "SELECT omitted FROM attention_preferences WHERE id=1",
                    [],
                    |r| r.get(0),
                )
                .optional()
                .map_err(storage_error)?
                .unwrap_or(0);
            let mut pending=tx.prepare("SELECT id,session,run,sequence,kind,created_at,seen,delivery,rowid,title,details,severity FROM attention_notices WHERE delivery='pending' AND seen=0 ORDER BY rowid LIMIT 2").map_err(storage_error)?;
            let pending_delivery = pending
                .query_map([], row)
                .map_err(storage_error)?
                .collect::<Result<Vec<_>, _>>()
                .map_err(storage_error)?;
            json!(Page {
                preferences: prefs(&tx)?,
                items: rows.into_iter().map(|(v, _)| v).collect(),
                next_cursor,
                more_sources,
                omitted,
                unread: tx
                    .query_row(
                        "SELECT count(*) FROM attention_notices WHERE seen=0",
                        [],
                        |r| r.get(0)
                    )
                    .map_err(storage_error)?,
                pending_delivery
            })
        }
        Request::Inspect { id } => json!(inspect(&tx, id)?),
        Request::Resolve { id } => {
            let notice = inspect(&tx, id)?;
            if notice.kind == Kind::App {
                return Err("Open this notification's details in Notifications.".into());
            }
            let state: String = tx
                .query_row(
                    "SELECT json_extract(data,'$.state') FROM runs WHERE id=?1 AND thread=?2",
                    params![notice.run, notice.session],
                    |r| r.get(0),
                )
                .map_err(storage_error)?;
            let last:Option<u32>=tx.query_row("SELECT MAX(sequence) FROM run_events WHERE run_id=?1 AND json_extract(data,'$.kind') IN ('approvalRequested','approvalDecided','questionAsked','questionAnswered','finished','restart')",[&notice.run],|r|r.get(0)).map_err(storage_error)?;
            json!({"notice":notice,"currentState":state,"sameDecision":state=="waitingForApproval"&&last==Some(notice.sequence),"note":"Opening is navigation only. Decide in the current task; older notices retain evidence and cannot approve or answer."})
        }
        Request::Seen { id } => {
            inspect(&tx, id)?;
            tx.execute("UPDATE attention_notices SET seen=1,delivery=CASE WHEN delivery='pending' THEN 'suppressed' ELSE delivery END WHERE id=?1",[id]).map_err(storage_error)?;
            json!(inspect(&tx, id)?)
        }
        Request::Claim { id } => {
            let n = inspect(&tx, id)?;
            let stale_decision = if matches!(n.kind, Kind::Question | Kind::Approval) {
                let current: String = tx
                    .query_row(
                        "SELECT json_extract(data,'$.state') FROM runs WHERE id=?1",
                        [&n.run],
                        |r| r.get(0),
                    )
                    .map_err(storage_error)?;
                let last:Option<u32>=tx.query_row("SELECT MAX(sequence) FROM run_events WHERE run_id=?1 AND json_extract(data,'$.kind') IN ('approvalRequested','approvalDecided','questionAsked','questionAnswered','finished','restart')",[&n.run],|r|r.get(0)).map_err(storage_error)?;
                current != "waitingForApproval" || last != Some(n.sequence)
            } else {
                false
            };
            if n.seen
                || n.delivery != Delivery::Pending
                || !prefs(&tx)?.permits(n.kind)
                || stale_decision
            {
                if n.delivery == Delivery::Pending {
                    tx.execute(
                        "UPDATE attention_notices SET delivery='suppressed' WHERE id=?1",
                        [id],
                    )
                    .map_err(storage_error)?;
                }
                Value::Null
            } else {
                tx.execute(
                    "UPDATE attention_notices SET delivery='claimed' WHERE id=?1",
                    [id],
                )
                .map_err(storage_error)?;
                json!(inspect(&tx, id)?)
            }
        }
        Request::Finish { id, delivery } => {
            if !matches!(
                delivery,
                Delivery::Accepted | Delivery::Failed | Delivery::Suppressed | Delivery::Uncertain
            ) {
                return Err(
                    "Invalid notification delivery outcome; the in-app task remains available."
                        .into(),
                );
            }
            let n = inspect(&tx, id)?;
            if n.delivery == *delivery {
                json!(n)
            } else {
                if n.delivery != Delivery::Claimed {
                    return Err("Notification handoff is no longer claimed. Inspect its delivery receipt; do not resend it automatically.".into());
                }
                tx.execute(
                    "UPDATE attention_notices SET delivery=?2 WHERE id=?1",
                    params![id, label(delivery)?],
                )
                .map_err(storage_error)?;
                json!(inspect(&tx, id)?)
            }
        }
    };
    tx.commit().map_err(storage_error)?;
    Ok(result)
}
