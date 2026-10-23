use super::{now, storage_error, SqliteStore};
use dolores_core::{
    harness_task::InputEdit,
    planning::{ContextReference, PlanningState},
    side_conversations::*,
    Session,
};
use rusqlite::{params, Connection, OptionalExtension, TransactionBehavior};
use sha2::{Digest, Sha256};
#[cfg(test)]
#[path = "side_conversation_tests.rs"]
mod tests;

fn hash(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}
fn exists(c: &Connection, session: &str) -> Result<bool, String> {
    c.query_row(
        "SELECT EXISTS(SELECT 1 FROM sessions WHERE id=?1)",
        [session],
        |r| r.get(0),
    )
    .map_err(storage_error)
}
fn source(c: &Connection, session: &str, message: i64) -> Result<(String, String), String> {
    let row:Option<(String,String)>=c.query_row("SELECT role,CASE WHEN length(CAST(content AS BLOB))<=?3 THEN content ELSE NULL END FROM messages WHERE session_id=?1 AND id=?2",params![session,message,MAX_SOURCE as i64],|r|Ok((r.get(0)?,r.get(1)?))).optional().map_err(|_|"Selected message exceeds the 1 MiB inspection limit or is unreadable. Choose a smaller source; nothing was shared.".to_string())?;
    row.ok_or_else(|| {
        "Selected source is unavailable. Keep the frozen excerpt and choose a saved message again."
            .into()
    })
}
fn revision(session: &str, message: i64, role: &str, content: &str) -> Result<String, String> {
    Ok(hash(
        &serde_json::to_vec(&(session, message, role, content)).map_err(storage_error)?,
    ))
}
fn checked(c: &Connection, session: &str, quote: &Quote, assistant: bool) -> Result<(), String> {
    quote.validate()?;
    let (role, body) = source(c, session, quote.message)?;
    if assistant && role != "assistant" {
        return Err(
            "Select a saved assistant conclusion to bring back. No parent input was added.".into(),
        );
    }
    if !matches!(role.as_str(), "user" | "assistant")
        || quote.revision != revision(session, quote.message, &role, &body)?
    {
        return Err("Source changed since preview. Preview and select again; conversations and drafts remain.".into());
    }
    Ok(())
}
fn origin(c: &Connection, session: &str) -> Result<Option<Origin>, String> {
    if !exists(c, session)? {
        return Err("Conversation no longer exists. Other chats and working files remain.".into());
    }
    let raw: Option<String> = c
        .query_row(
            "SELECT origin FROM thread_context WHERE session_id=?1",
            [session],
            |r| r.get(0),
        )
        .optional()
        .map_err(storage_error)?
        .flatten();
    let Some(raw) = raw else {
        return Ok(None);
    };
    if raw.len() > MAX_ORIGIN {
        return Err("Side-discussion provenance exceeds its inspection bound. Keep the saved chat and inspect its export.".into());
    }
    let value: serde_json::Value = serde_json::from_str(&raw).map_err(storage_error)?;
    if value["kind"] != "sideSelection" {
        return Ok(None);
    }
    let value: Origin = serde_json::from_value(value).map_err(storage_error)?;
    value.validate()?;
    Ok(Some(value))
}
fn info(c: &Connection, value: Origin) -> Result<Info, String> {
    let parent_available = exists(c, &value.parent)?;
    let source_available =
        parent_available && checked(c, &value.parent, &value.quote, false).is_ok();
    Ok(Info {
        origin: value,
        parent_available,
        source_available,
    })
}
fn session(c: &Connection, id: &str) -> Result<Session, String> {
    c.query_row(
        "SELECT title,updated_at FROM sessions WHERE id=?1",
        [id],
        |r| {
            Ok(Session {
                id: id.into(),
                title: r.get(0)?,
                updated_at: r.get(1)?,
            })
        },
    )
    .map_err(storage_error)
}
pub(super) fn preview(
    store: &SqliteStore,
    s: &str,
    message: i64,
    text: &str,
) -> Result<Quote, String> {
    excerpt(text)?;
    let c = store.lock()?;
    let (role, body) = source(&c, s, message)?;
    if !matches!(role.as_str(), "user" | "assistant") {
        return Err("Select text from a saved user or assistant message.".into());
    }
    Ok(Quote {
        message,
        revision: revision(s, message, &role, &body)?,
        excerpt: text.into(),
    })
}
pub(super) fn read(store: &SqliteStore, s: &str) -> Result<Option<Info>, String> {
    let c = store.lock()?;
    origin(&c, s)?.map(|v| info(&c, v)).transpose()
}
pub(super) fn create(
    store: &SqliteStore,
    parent: &str,
    request: &Create,
) -> Result<Created, String> {
    identity(parent)?;
    identity(&request.id)?;
    request.quote.validate()?;
    question(&request.question)?;
    if parent == request.id {
        return Err("A side conversation needs a new identity. Keep the parent and retry with a new discussion.".into());
    }
    let mut c = store.lock()?;
    let tx = c
        .transaction_with_behavior(TransactionBehavior::Immediate)
        .map_err(storage_error)?;
    if exists(&tx, &request.id)? {
        let old=origin(&tx,&request.id)?.ok_or("That identity belongs to another conversation. Open the saved chat instead of replacing it.")?;
        if old.parent != parent || old.quote != request.quote || old.question != request.question {
            return Err("Side conversation identity already used. Inspect the saved discussion; no draft was overwritten.".into());
        }
        return Ok(Created {
            session: session(&tx, &request.id)?,
            info: info(&tx, old)?,
        });
    }
    checked(&tx, parent, &request.quote, false)?;
    let parent_title = session(&tx, parent)?
        .title
        .chars()
        .take(120)
        .collect::<String>();
    let value = Origin {
        kind: "sideSelection".into(),
        parent: parent.into(),
        parent_title,
        quote: request.quote.clone(),
        question: request.question.clone(),
        returns: vec![],
    };
    value.validate()?;
    let child = Session {
        id: request.id.clone(),
        title: format!(
            "Side: {}",
            value
                .quote
                .excerpt
                .split_whitespace()
                .collect::<Vec<_>>()
                .join(" ")
                .chars()
                .take(42)
                .collect::<String>()
        ),
        updated_at: now(),
    };
    tx.execute(
        "INSERT INTO sessions(id,title,updated_at) VALUES(?1,?2,?3)",
        params![child.id, child.title, child.updated_at],
    )
    .map_err(storage_error)?;
    tx.execute(
        "INSERT INTO session_workspaces(session_id,kind,root) VALUES(?1,'side',NULL)",
        [&child.id],
    )
    .map_err(storage_error)?;
    tx.execute(
        "INSERT INTO thread_context(session_id,auto_compact,origin) VALUES(?1,1,?2)",
        params![
            child.id,
            serde_json::to_string(&value).map_err(storage_error)?
        ],
    )
    .map_err(storage_error)?;
    let state=PlanningState{references:vec![ContextReference{kind:"sideExcerpt".into(),source:serde_json::json!({"parent":parent,"message":value.quote.message,"selection":"user-selected rendered text"}).to_string(),revision:value.quote.revision.clone(),text:value.quote.excerpt.clone(),omissions:"Only selected rendered text. Canonical Markdown byte offsets, other messages, attachments and parent task authority are not shared.".into()}],..Default::default()};
    state.validate()?;
    tx.execute(
        "INSERT INTO planning(session,data) VALUES(?1,?2)",
        params![
            child.id,
            serde_json::to_string(&state).map_err(storage_error)?
        ],
    )
    .map_err(storage_error)?;
    if !request.question.is_empty() {
        tx.execute(
            "INSERT INTO session_drafts(session_id,text) VALUES(?1,?2)",
            params![child.id, request.question],
        )
        .map_err(storage_error)?;
    }
    tx.commit().map_err(storage_error)?;
    Ok(Created {
        session: child,
        info: Info {
            origin: value,
            parent_available: true,
            source_available: true,
        },
    })
}
pub(super) fn bring_back(
    store: &SqliteStore,
    child: &str,
    request: &Return,
) -> Result<Info, String> {
    identity(&request.id)?;
    request.quote.validate()?;
    let mut c = store.lock()?;
    let tx = c
        .transaction_with_behavior(TransactionBehavior::Immediate)
        .map_err(storage_error)?;
    let mut value = origin(&tx, child)?.ok_or(
        "This is not a linked side conversation. Use explicit context selection to share its text.",
    )?;
    let receipt = ReturnReceipt {
        id: request.id.clone(),
        message: request.quote.message,
        revision: request.quote.revision.clone(),
        excerpt_digest: hash(request.quote.excerpt.as_bytes()),
        mode: request.mode,
    };
    if let Some(old) = value.returns.iter().find(|r| r.id == request.id) {
        if *old != receipt {
            return Err("Return identity already used for another selection. Inspect the parent queue before retrying.".into());
        }
        return info(&tx, value);
    }
    if value.returns.len() >= MAX_RETURNS {
        return Err("32 returned conclusions are retained. Continue in the parent or start a new side discussion; existing receipts and drafts remain.".into());
    }
    if !exists(&tx, &value.parent)? {
        return Err("Parent conversation is unavailable. Keep this discussion and use explicit context selection in another chat; nothing was queued.".into());
    }
    checked(&tx, child, &request.quote, true)?;
    let used: bool = tx
        .query_row(
            "SELECT EXISTS(SELECT 1 FROM task_inputs WHERE id=?1)",
            [&request.id],
            |r| r.get(0),
        )
        .map_err(storage_error)?;
    if used {
        return Err("Return identity belongs to another instruction. Inspect the parent queue and use a fresh return identity.".into());
    }
    let text = conclusion(child, &request.quote)?;
    value.returns.push(receipt);
    value.validate()?;
    super::task_inputs::edit_in_transaction(
        &tx,
        &value.parent,
        &InputEdit::Enqueue {
            id: request.id.clone(),
            text,
            mode: request.mode,
        },
        false,
    )?;
    tx.execute(
        "UPDATE thread_context SET origin=?2 WHERE session_id=?1",
        params![child, serde_json::to_string(&value).map_err(storage_error)?],
    )
    .map_err(storage_error)?;
    tx.commit().map_err(storage_error)?;
    let c = &*c;
    info(c, value)
}
