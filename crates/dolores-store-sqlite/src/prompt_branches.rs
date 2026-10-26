use super::{now, storage_error, SqliteStore};
use dolores_core::{
    planning::{ContextReference, PlanningState},
    prompt_branches::*,
    Role, Session, SessionWorkspace, TurnMetadata, WorkspaceKind, MAX_DRAFT_ATTACHMENTS,
};
use rusqlite::{params, Connection, OptionalExtension, TransactionBehavior};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

#[cfg(test)]
#[path = "prompt_branch_tests.rs"]
mod tests;

#[derive(Serialize, Deserialize)]
struct Saved {
    request: Create,
    origin: Origin,
    workspace: SessionWorkspace,
    context: Vec<ContextReference>,
    source_origin: Option<serde_json::Value>,
    messages: Vec<ArchivedMessage>,
}
struct Source {
    title: String,
    workspace: SessionWorkspace,
    context: Vec<ContextReference>,
    source_origin: Option<serde_json::Value>,
    messages: Vec<ArchivedMessage>,
    selected: usize,
    revision: String,
    bytes: usize,
}
fn exists(c: &Connection, id: &str) -> Result<bool, String> {
    c.query_row(
        "SELECT EXISTS(SELECT 1 FROM sessions WHERE id=?1)",
        [id],
        |r| r.get(0),
    )
    .map_err(storage_error)
}
fn digest<T: Serialize>(value: &T) -> Result<String, String> {
    Ok(format!(
        "{:x}",
        Sha256::digest(serde_json::to_vec(value).map_err(storage_error)?)
    ))
}
fn target(
    c: &Connection,
    parent: &str,
    child: &str,
    expected: u32,
) -> Result<dolores_core::worktrees::ManagedWorktree, String> {
    use dolores_core::worktrees::{ManagedWorktree, WorktreeState};
    let raw: Option<Option<String>> = c.query_row("SELECT CASE WHEN length(CAST(data AS BLOB))<=32768 THEN data ELSE NULL END FROM managed_worktrees WHERE session=?1",[child],|r|r.get(0)).optional().map_err(storage_error)?;
    let record: ManagedWorktree = serde_json::from_str(&raw.flatten().ok_or(
        "Choose a retained Ready managed worktree; the original chat and edited prompt remain.",
    )?)
    .map_err(storage_error)?;
    record.validate()?;
    if record.session != child
        || record.state != WorktreeState::Ready
        || record.revision != expected
    {
        return Err("Managed worktree changed or is not Ready. Inspect it and choose its current revision before attaching the edited prompt.".into());
    }
    let parent_root: Option<String> = c
        .query_row(
            "SELECT root FROM session_workspaces WHERE session_id=?1",
            [parent],
            |r| r.get(0),
        )
        .optional()
        .map_err(storage_error)?
        .flatten();
    let child_workspace: Option<(String, Option<String>)> = c
        .query_row(
            "SELECT kind,root FROM session_workspaces WHERE session_id=?1",
            [child],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )
        .optional()
        .map_err(storage_error)?;
    if parent_root.as_deref() != Some(record.source_root.as_str())
        || child_workspace != Some(("project".into(), Some(record.root.clone())))
    {
        return Err("Choose a managed checkout created from this source folder and still bound to its owning chat. Other checkouts remain unchanged.".into());
    }
    let occupied: bool = c.query_row("SELECT EXISTS(SELECT 1 FROM messages WHERE session_id=?1) OR EXISTS(SELECT 1 FROM session_drafts WHERE session_id=?1 AND length(text)>0) OR EXISTS(SELECT 1 FROM runs WHERE thread=?1) OR EXISTS(SELECT 1 FROM goals WHERE session=?1) OR EXISTS(SELECT 1 FROM goal_history WHERE session=?1) OR EXISTS(SELECT 1 FROM task_inputs WHERE session=?1) OR EXISTS(SELECT 1 FROM task_input_state WHERE session=?1) OR EXISTS(SELECT 1 FROM planning WHERE session=?1) OR EXISTS(SELECT 1 FROM composer_skills WHERE session=?1) OR EXISTS(SELECT 1 FROM scoped_settings WHERE scope='thread' AND scope_key=?1) OR EXISTS(SELECT 1 FROM draft_attachments WHERE session_id=?1 AND data!='[]') OR EXISTS(SELECT 1 FROM file_changes WHERE session_id=?1) OR EXISTS(SELECT 1 FROM thread_context WHERE session_id=?1 AND origin IS NOT NULL)",[child],|r|r.get(0)).map_err(storage_error)?;
    if occupied {
        return Err("This managed chat already has work, settings or a draft. Keep it and choose a new empty managed worktree; no text or history was replaced.".into());
    }
    Ok(record)
}
pub(super) fn inspect_target(
    store: &SqliteStore,
    parent: &str,
    child: &str,
    revision: u32,
) -> Result<dolores_core::worktrees::ManagedWorktree, String> {
    let c = store.lock()?;
    target(&c, parent, child, revision)
}
fn source(c: &Connection, parent: &str, message: i64) -> Result<Source, String> {
    if !exists(c, parent)? {
        return Err("Original chat is unavailable. Keep the edit and use a selected-text side discussion or another saved chat.".into());
    }
    let (count, raw_bytes): (usize, usize) = c.query_row("SELECT COUNT(*),COALESCE(SUM(length(CAST(m.content AS BLOB))+COALESCE(length(CAST(t.data AS BLOB)),0)+COALESCE(length(CAST(p.data AS BLOB)),0)),0) FROM messages m LEFT JOIN turn_metadata t ON t.message_id=m.id LEFT JOIN message_parts p ON p.message_id=m.id WHERE m.session_id=?1", [parent], |r| Ok((r.get(0)?,r.get(1)?))).map_err(storage_error)?;
    if count == 0 || count > MAX_MESSAGES || !count.is_multiple_of(2) || raw_bytes > MAX_TRANSCRIPT
    {
        return Err("A prompt branch retains up to 100 completed exchanges and 1 MiB of original history/receipts. Keep this chat; narrow the history or use a selected-text side discussion.".into());
    }
    let title: String = c
        .query_row("SELECT title FROM sessions WHERE id=?1", [parent], |r| {
            r.get(0)
        })
        .map_err(storage_error)?;
    let title = title.chars().take(120).collect::<String>();
    let workspace = c.query_row("SELECT kind,root FROM session_workspaces WHERE session_id=?1", [parent], |r| Ok((r.get::<_,String>(0)?,r.get::<_,Option<String>>(1)?))).optional().map_err(storage_error)?.map(|(kind,root)| {
        let kind = match kind.as_str() { "project" => WorkspaceKind::Project, "temporary" => WorkspaceKind::Temporary, "side" => WorkspaceKind::Side, _ => return Err("Original workspace is invalid. Inspect the original chat; no branch was created.".to_string()) };
        Ok(SessionWorkspace {kind,root})
    }).transpose()?.unwrap_or(SessionWorkspace {kind:WorkspaceKind::Side, root:None});
    let context = super::planning::read(c, parent)?.references;
    let raw_origin: Option<String> = c
        .query_row(
            "SELECT origin FROM thread_context WHERE session_id=?1",
            [parent],
            |r| r.get(0),
        )
        .optional()
        .map_err(storage_error)?
        .flatten();
    if raw_origin.as_ref().is_some_and(|s| s.len() > 32768) {
        return Err("Original ancestry exceeds 32 KiB. Keep the original chat and use a selected-text side discussion.".into());
    }
    let source_origin = raw_origin
        .map(|s| serde_json::from_str(&s))
        .transpose()
        .map_err(storage_error)?;
    let mut query = c.prepare("SELECT m.id,m.role,m.content,t.data,p.data,d.saved_at FROM messages m LEFT JOIN turn_metadata t ON t.message_id=m.id LEFT JOIN message_parts p ON p.message_id=m.id LEFT JOIN message_timestamps d ON d.message_id=m.id WHERE m.session_id=?1 ORDER BY m.id LIMIT 201").map_err(storage_error)?;
    let mut messages = Vec::with_capacity(count);
    let mut rows = query.query([parent]).map_err(storage_error)?;
    while let Some(r) = rows.next().map_err(storage_error)? {
        let role: String = r.get(1).map_err(storage_error)?;
        let role = match role.as_str() {"user"=>Role::User,"assistant"=>Role::Assistant,_=>return Err("Original history has an incomplete or invalid turn. Keep the original chat and choose another saved prompt.".into())};
        let metadata: Option<String> = r.get(3).map_err(storage_error)?;
        if let Some(raw) = &metadata {
            serde_json::from_str::<TurnMetadata>(raw).map_err(storage_error)?;
        }
        let parts: Option<String> = r.get(4).map_err(storage_error)?;
        let parts: Vec<dolores_core::AttachmentRef> = parts
            .map(|s| serde_json::from_str(&s))
            .transpose()
            .map_err(storage_error)?
            .unwrap_or_default();
        if parts.len() > MAX_DRAFT_ATTACHMENTS {
            return Err("Original prompt attachments exceed four. Inspect the original chat; nothing was copied.".into());
        }
        for part in &parts {
            part.validate()?;
        }
        messages.push(ArchivedMessage {
            id: r.get(0).map_err(storage_error)?,
            role,
            content: r.get(2).map_err(storage_error)?,
            parts,
            metadata: metadata
                .map(|s| serde_json::from_str(&s))
                .transpose()
                .map_err(storage_error)?,
            saved_at: r.get(5).map_err(storage_error)?,
        });
    }
    if messages.len() != count
        || messages
            .as_chunks::<2>()
            .0
            .iter()
            .any(|p| p[0].role != Role::User || p[1].role != Role::Assistant)
    {
        return Err("Choose a saved user prompt in complete user/assistant history. The original chat remains.".into());
    }
    let selected = messages
        .iter()
        .position(|m| m.id == message && m.role == Role::User)
        .ok_or("Select a saved user prompt belonging to this chat; no other chat was changed.")?;
    let bytes = serde_json::to_vec(&messages).map_err(storage_error)?.len();
    if bytes > MAX_TRANSCRIPT {
        return Err("Encoded original transcript exceeds 1 MiB. Keep the original chat and use a narrower side discussion; your edit remains.".into());
    }
    let revision = digest(&(
        parent,
        &title,
        &workspace,
        &messages,
        &context,
        &source_origin,
    ))?;
    Ok(Source {
        title,
        workspace,
        context,
        source_origin,
        messages,
        selected,
        revision,
        bytes,
    })
}
fn saved(c: &Connection, child: &str) -> Result<Option<Saved>, String> {
    let raw: Option<Option<String>> = c.query_row("SELECT CASE WHEN length(CAST(data AS BLOB))<=2097152 THEN data ELSE NULL END FROM prompt_branches WHERE session=?1", [child], |r|r.get(0)).optional().map_err(storage_error)?;
    let Some(raw) = raw else { return Ok(None) };
    let value: Saved = serde_json::from_str(&raw.ok_or(
        "Branch archive exceeds its allowance. Keep the chat and inspect its saved evidence.",
    )?)
    .map_err(storage_error)?;
    value.request.validate()?;
    dolores_core::side_conversations::identity(&value.origin.parent)?;
    match (&value.origin.managed,value.request.folder) {
        (Some(record),Folder::Managed) if record.session==child && Some(record.revision)==value.request.target_revision => record.validate()?,
        (None,Folder::Shared|Folder::Side) => (),
        _ => return Err("Managed branch target provenance is invalid. Keep the retained chat and inspect its archive.".into()),
    }
    if value.request.id != child
        || value.origin.kind != "editedPrompt"
        || value.origin.basis != value.request.basis
        || value.origin.folder != value.request.folder
        || value.messages.len() > MAX_MESSAGES
        || !value.messages.len().is_multiple_of(2)
        || value.origin.archived_turns != value.messages.len() / 2
        || value.origin.parent_title.len() > 512
        || value
            .messages
            .as_chunks::<2>()
            .0
            .iter()
            .any(|pair| pair[0].role != Role::User || pair[1].role != Role::Assistant)
        || value
            .origin
            .copied_turns
            .checked_mul(2)
            .and_then(|i| value.messages.get(i))
            .is_none_or(|m| m.id != value.request.basis.message || m.role != Role::User)
        || digest(&(
            &value.origin.parent,
            &value.origin.parent_title,
            &value.workspace,
            &value.messages,
            &value.context,
            &value.source_origin,
        ))? != value.request.basis.revision
    {
        return Err("Branch provenance changed or is invalid. Preserve the original and inspect the retained archive before continuing.".into());
    }
    Ok(Some(value))
}
fn info(c: &Connection, value: Saved) -> Result<Info, String> {
    let parent_available = exists(c, &value.origin.parent)?;
    let source_available = parent_available
        && source(c, &value.origin.parent, value.request.basis.message)
            .is_ok_and(|s| s.revision == value.request.basis.revision);
    Ok(Info {
        origin: value.origin,
        request: value.request,
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
pub(super) fn preview(store: &SqliteStore, parent: &str, message: i64) -> Result<Preview, String> {
    let c = store.lock()?;
    let s = source(&c, parent, message)?;
    let selected = &s.messages[s.selected];
    Ok(Preview {
        basis: Basis {
            message,
            revision: s.revision,
        },
        original_prompt: selected.content.clone(),
        attachments: selected.parts.clone(),
        context: s.context,
        workspace: s.workspace,
        prior_turns: s.selected / 2,
        later_turns: (s.messages.len() - s.selected) / 2,
        archive_bytes: s.bytes,
        effect_turns: s
            .messages
            .iter()
            .filter(|m| {
                m.metadata
                    .as_ref()
                    .and_then(|v| v.get("agent"))
                    .and_then(|v| v.get("tools"))
                    .and_then(|v| v.as_array())
                    .is_some_and(|v| !v.is_empty())
            })
            .count(),
    })
}
pub(super) fn read(store: &SqliteStore, child: &str) -> Result<Option<Info>, String> {
    let c = store.lock()?;
    saved(&c, child)?.map(|v| info(&c, v)).transpose()
}
pub(super) fn archive(
    store: &SqliteStore,
    child: &str,
    offset: usize,
) -> Result<ArchivePage, String> {
    let c = store.lock()?;
    let value = saved(&c, child)?.ok_or("This chat has no original prompt snapshot.")?;
    let text = serde_json::to_string_pretty(&serde_json::json!({"sourceOrigin":value.source_origin,"contextSnapshots":value.context,"managedTarget":value.origin.managed,"messages":value.messages})).map_err(storage_error)?;
    if offset > text.len() || !text.is_char_boundary(offset) {
        return Err(
            "Snapshot page is unavailable. Start at the beginning; the retained snapshot remains."
                .into(),
        );
    }
    let mut end = offset.saturating_add(PAGE_BYTES).min(text.len());
    while !text.is_char_boundary(end) {
        end -= 1;
    }
    Ok(ArchivePage {
        text: text[offset..end].into(),
        offset,
        next_offset: (end < text.len()).then_some(end),
        total_bytes: text.len(),
    })
}
pub(super) fn create(
    store: &SqliteStore,
    parent: &str,
    request: &Create,
) -> Result<Created, String> {
    request.validate()?;
    dolores_core::side_conversations::identity(parent)?;
    if parent == request.id {
        return Err(
            "An edited branch needs a separate chat identity. The original stays in place.".into(),
        );
    }
    let mut c = store.lock()?;
    let tx = c
        .transaction_with_behavior(TransactionBehavior::Immediate)
        .map_err(storage_error)?;
    let existing = exists(&tx, &request.id)?;
    if let Some(old) = saved(&tx, &request.id)? {
        if old.origin.parent != parent || old.request != *request {
            return Err("Branch identity already used for another edit. Inspect the saved branch; no newer draft was replaced.".into());
        }
        return Ok(Created {
            session: session(&tx, &request.id)?,
            info: info(&tx, old)?,
        });
    }
    if existing && request.folder != Folder::Managed {
        return Err("That identity belongs to another chat. Inspect the saved chat instead of replacing it.".into());
    }
    let managed = if request.folder == Folder::Managed {
        Some(target(
            &tx,
            parent,
            &request.id,
            request.target_revision.unwrap(),
        )?)
    } else {
        None
    };
    let s = source(&tx, parent, request.basis.message)?;
    if request.basis.revision != s.revision {
        return Err("Original conversation changed after preview. Refresh the preview while keeping your edited text; nothing was branched.".into());
    }
    let selected_parts = if request.include_attachments {
        s.messages[s.selected].parts.clone()
    } else {
        vec![]
    };
    if request.edited.trim().is_empty() && selected_parts.is_empty() {
        return Err("Add edited text or include an existing prompt attachment. Your edit and original chat remain.".into());
    }
    let workspace = match request.folder {
        Folder::Shared if s.workspace.root.is_some() => s.workspace.clone(),
        Folder::Shared => return Err("The original chat has no working folder. Choose No working folder; your edited text remains.".into()),
        Folder::Side => SessionWorkspace {kind:WorkspaceKind::Side,root:None},
        Folder::Managed => SessionWorkspace {kind:WorkspaceKind::Project,root:Some(managed.as_ref().unwrap().root.clone())},
    };
    let child = Session {
        id: request.id.clone(),
        title: format!("Edit: {}", s.title),
        updated_at: now(),
    };
    let origin = Origin {
        kind: "editedPrompt".into(),
        parent: parent.into(),
        parent_title: s.title,
        basis: request.basis.clone(),
        folder: request.folder,
        created_at: child.updated_at,
        copied_turns: s.selected / 2,
        archived_turns: s.messages.len() / 2,
        managed,
    };
    let value = Saved {
        request: request.clone(),
        origin,
        workspace: s.workspace,
        context: s.context,
        source_origin: s.source_origin,
        messages: s.messages,
    };
    let encoded = serde_json::to_string(&value).map_err(storage_error)?;
    if encoded.len() > MAX_ARCHIVE {
        return Err("Encoded branch archive exceeds its allowance. Narrow the edit or keep this conversation and use a side discussion.".into());
    }
    if existing {
        tx.execute(
            "UPDATE sessions SET title=?2,updated_at=?3 WHERE id=?1",
            params![child.id, child.title, child.updated_at],
        )
        .map_err(storage_error)?;
    } else {
        tx.execute(
            "INSERT INTO sessions(id,title,updated_at) VALUES(?1,?2,?3)",
            params![child.id, child.title, child.updated_at],
        )
        .map_err(storage_error)?;
        tx.execute(
            "INSERT INTO session_workspaces(session_id,kind,root) VALUES(?1,?2,?3)",
            params![
                child.id,
                match workspace.kind {
                    WorkspaceKind::Project => "project",
                    WorkspaceKind::Temporary => "temporary",
                    WorkspaceKind::Side => "side",
                },
                workspace.root
            ],
        )
        .map_err(storage_error)?;
    }
    tx.execute(
        "INSERT INTO prompt_branches(session,data) VALUES(?1,?2)",
        params![child.id, encoded],
    )
    .map_err(storage_error)?;
    for message in &value.messages {
        let mut retained_parts = message.parts.clone();
        if let Some(metadata) = &message.metadata {
            let metadata: TurnMetadata =
                serde_json::from_value(metadata.clone()).map_err(storage_error)?;
            if let Some(agent) = metadata.agent {
                retained_parts.extend(agent.tools.into_iter().flat_map(|tool| tool.parts));
            }
        }
        for part in &retained_parts {
            part.validate()?;
            tx.execute("INSERT OR IGNORE INTO prompt_branch_attachments(session,digest,reference) VALUES(?1,?2,?3)",params![child.id,part.digest,serde_json::to_string(part).map_err(storage_error)?]).map_err(storage_error)?;
        }
    }
    for message in &value.messages[..s.selected] {
        tx.execute(
            "INSERT INTO messages(session_id,role,content) VALUES(?1,?2,?3)",
            params![
                child.id,
                if message.role == Role::User {
                    "user"
                } else {
                    "assistant"
                },
                message.content
            ],
        )
        .map_err(storage_error)?;
        let id = tx.last_insert_rowid();
        if !message.parts.is_empty() {
            tx.execute(
                "INSERT INTO message_parts(message_id,data) VALUES(?1,?2)",
                params![
                    id,
                    serde_json::to_string(&message.parts).map_err(storage_error)?
                ],
            )
            .map_err(storage_error)?;
        }
        if let Some(at) = message.saved_at {
            tx.execute(
                "INSERT INTO message_timestamps(message_id,saved_at) VALUES(?1,?2)",
                params![id, at],
            )
            .map_err(storage_error)?;
        }
        if let Some(metadata) = &message.metadata {
            let mut metadata: TurnMetadata =
                serde_json::from_value(metadata.clone()).map_err(storage_error)?;
            metadata.paused = None;
            tx.execute(
                "INSERT INTO turn_metadata(message_id,data) VALUES(?1,?2)",
                params![id, serde_json::to_string(&metadata).map_err(storage_error)?],
            )
            .map_err(storage_error)?;
        }
    }
    tx.execute(
        "INSERT INTO session_drafts(session_id,text) VALUES(?1,?2) ON CONFLICT(session_id) DO UPDATE SET text=excluded.text",
        params![child.id, request.edited],
    )
    .map_err(storage_error)?;
    if !selected_parts.is_empty() {
        tx.execute(
            "INSERT INTO draft_attachments(session_id,data) VALUES(?1,?2) ON CONFLICT(session_id) DO UPDATE SET data=excluded.data",
            params![
                child.id,
                serde_json::to_string(&selected_parts).map_err(storage_error)?
            ],
        )
        .map_err(storage_error)?;
    }
    tx.execute(
        "INSERT INTO thread_context(session_id,auto_compact,origin) VALUES(?1,1,?2) ON CONFLICT(session_id) DO UPDATE SET auto_compact=1,origin=excluded.origin",
        params![
            child.id,
            serde_json::to_string(&value.origin).map_err(storage_error)?
        ],
    )
    .map_err(storage_error)?;
    let planning = PlanningState {
        references: if request.include_context {
            value.context.clone()
        } else {
            vec![]
        },
        ..Default::default()
    };
    planning.validate()?;
    tx.execute(
        "INSERT INTO planning(session,data) VALUES(?1,?2)",
        params![
            child.id,
            serde_json::to_string(&planning).map_err(storage_error)?
        ],
    )
    .map_err(storage_error)?;
    tx.commit().map_err(storage_error)?;
    Ok(Created {
        session: child,
        info: Info {
            origin: value.origin,
            request: value.request,
            parent_available: true,
            source_available: true,
        },
    })
}
