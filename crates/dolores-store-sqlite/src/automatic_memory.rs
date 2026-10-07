use super::{
    memory::{decode, scope_key},
    now, storage_error, SqliteStore,
};
use dolores_core::*;
use rusqlite::{params, OptionalExtension};

fn policy(conn: &rusqlite::Connection) -> Result<AutomaticMemoryPolicy, String> {
    let data: Option<String> = conn
        .query_row(
            "SELECT data FROM automatic_memory_policy WHERE id=1",
            [],
            |r| r.get(0),
        )
        .optional()
        .map_err(storage_error)?;
    let p: AutomaticMemoryPolicy = data
        .map(|d| serde_json::from_str(&d).map_err(storage_error))
        .transpose()?
        .unwrap_or_default();
    if p.revision == 0 {
        return Err("Automatic memory policy is invalid.".into());
    }
    Ok(p)
}

fn scope(conn: &rusqlite::Connection, session: &str) -> Result<String, String> {
    let exists: bool = conn
        .query_row(
            "SELECT EXISTS(SELECT 1 FROM sessions WHERE id=?1)",
            [session],
            |r| r.get(0),
        )
        .map_err(storage_error)?;
    if !exists {
        return Err("Conversation no longer exists.".into());
    }
    let root: Option<String> = conn
        .query_row(
            "SELECT root FROM session_workspaces WHERE session_id=?1",
            [session],
            |r| r.get(0),
        )
        .optional()
        .map_err(storage_error)?
        .flatten();
    Ok(scope_key(root.as_deref())?.into())
}
fn source_matches(
    conn: &rusqlite::Connection,
    session: &str,
    source: &MemoryMessage,
) -> Result<bool, String> {
    let text:Option<String>=conn.query_row("SELECT m.content FROM messages m WHERE m.session_id=?1 AND m.id=?2 AND m.role='user' AND EXISTS(SELECT 1 FROM messages a WHERE a.session_id=m.session_id AND a.id=m.id+1 AND a.role='assistant')",params![session,source.message_id],|r|r.get(0)).optional().map_err(storage_error)?;
    Ok(text.as_deref() == Some(&source.text))
}
fn forgotten(
    conn: &rusqlite::Connection,
    root: &str,
    session: &str,
    message_id: i64,
) -> Result<bool, String> {
    conn.query_row("SELECT EXISTS(SELECT 1 FROM memory_forget_watermarks WHERE root=?1 AND session=?2 AND message_id>=?3)",params![root,session,message_id],|r|r.get(0)).map_err(storage_error)
}
fn preferences(conn: &rusqlite::Connection, root: &str) -> Result<Vec<MemoryPreference>, String> {
    let mut stmt = conn
        .prepare(
            "SELECT data FROM memory_preferences WHERE root='' OR root=?1 ORDER BY id LIMIT 281",
        )
        .map_err(storage_error)?;
    let mut values = stmt
        .query_map([root], |r| r.get::<_, String>(0))
        .map_err(storage_error)?
        .map(|r| decode(r.map_err(storage_error)?))
        .collect::<Result<Vec<_>, _>>()?;
    if values.len() > MAX_MEMORY_RECORDS_PER_SCOPE * 2 {
        return Err("Saved memory exceeds the scope limit.".into());
    }
    values.sort_by(|a, b| a.id.cmp(&b.id));
    Ok(values)
}
impl SqliteStore {
    pub(super) fn auto_policy(&self) -> Result<AutomaticMemoryPolicy, String> {
        let conn = self.lock()?;
        policy(&conn)
    }
    pub(super) fn write_auto_policy(
        &self,
        enabled: bool,
        revision: u32,
    ) -> Result<AutomaticMemoryPolicy, String> {
        let mut conn = self.lock()?;
        let tx = conn.transaction().map_err(storage_error)?;
        let p = policy(&tx)?;
        if p.revision != revision {
            return Err("Memory policy changed. Refresh Memory.".into());
        }
        let value = AutomaticMemoryPolicy {
            enabled,
            revision: revision
                .checked_add(1)
                .ok_or("Memory policy revision limit reached.")?,
        };
        tx.execute("INSERT INTO automatic_memory_policy(id,data) VALUES(1,?1) ON CONFLICT(id) DO UPDATE SET data=excluded.data",[serde_json::to_string(&value).map_err(storage_error)?]).map_err(storage_error)?;
        tx.commit().map_err(storage_error)?;
        Ok(value)
    }
    pub(super) fn auto_attempt(
        &self,
        session: &str,
    ) -> Result<Option<AutomaticMemoryAttempt>, String> {
        let conn = self.lock()?;
        scope(&conn, session)?;
        let data: Option<String> = conn
            .query_row(
                "SELECT data FROM automatic_memory_attempts WHERE session_id=?1",
                [session],
                |r| r.get(0),
            )
            .optional()
            .map_err(storage_error)?;
        data.map(|d| serde_json::from_str(&d).map_err(storage_error))
            .transpose()
    }
    pub(super) fn claim_auto(
        &self,
        session: &str,
        source: &MemoryMessage,
        revision: u32,
    ) -> Result<bool, String> {
        let mut conn = self.lock()?;
        let tx = conn.transaction().map_err(storage_error)?;
        let root = scope(&tx, session)?;
        let p = policy(&tx)?;
        if !p.enabled
            || p.revision != revision
            || !source_matches(&tx, session, source)?
            || forgotten(&tx, &root, session, source.message_id)?
        {
            return Ok(false);
        }
        let last: i64 = tx
            .query_row(
                "SELECT message_id FROM automatic_memory_attempts WHERE session_id=?1",
                [session],
                |r| r.get(0),
            )
            .optional()
            .map_err(storage_error)?
            .unwrap_or(0);
        // No startup replay, backlog extraction, or automatic retry of failed attempts.
        if last >= source.message_id {
            return Ok(false);
        }
        let latest: i64 = tx
            .query_row(
                "SELECT MAX(id) FROM messages WHERE session_id=?1 AND role='user'",
                [session],
                |r| r.get(0),
            )
            .map_err(storage_error)?;
        if latest != source.message_id {
            return Ok(false);
        }
        let report = AutomaticMemoryAttempt {
            message_id: source.message_id,
            status: "updating".into(),
            note: "Memory is updating in the background; the completed reply is saved.".into(),
            updated_at: now(),
            saved: 0,
            skipped: 0,
            usage: None,
        };
        tx.execute("INSERT INTO automatic_memory_attempts(session_id,message_id,data) VALUES(?1,?2,?3) ON CONFLICT(session_id) DO UPDATE SET message_id=excluded.message_id,data=excluded.data",params![session,source.message_id,serde_json::to_string(&report).map_err(storage_error)?]).map_err(storage_error)?;
        tx.commit().map_err(storage_error)?;
        Ok(true)
    }
    pub(super) fn finish_auto(
        &self,
        update: &AutomaticMemoryUpdate,
        cancel: &tokio_util::sync::CancellationToken,
    ) -> Result<AutomaticMemoryAttempt, String> {
        let mut conn = self.lock()?;
        let tx = conn.transaction().map_err(storage_error)?;
        let root = scope(&tx, &update.session)?;
        let pending: Option<String> = tx
            .query_row(
                "SELECT data FROM automatic_memory_attempts WHERE session_id=?1 AND message_id=?2",
                params![update.session, update.source.message_id],
                |r| r.get(0),
            )
            .optional()
            .map_err(storage_error)?;
        let pending: AutomaticMemoryAttempt =
            serde_json::from_str(&pending.ok_or("Automatic memory attempt changed.")?)
                .map_err(storage_error)?;
        if pending.status != "updating" {
            return Err("Automatic memory attempt already finished.".into());
        }
        let p = policy(&tx)?;
        let mut current = preferences(&tx, &root)?;
        let mut expected = update.existing.clone();
        expected.sort_by(|a, b| a.id.cmp(&b.id));
        let fresh = p.enabled
            && p.revision == update.policy_revision
            && source_matches(&tx, &update.session, &update.source)?
            && !forgotten(&tx, &root, &update.session, update.source.message_id)?
            && current == expected;
        let mut report = AutomaticMemoryAttempt {
            message_id: update.source.message_id,
            status: update.status.clone(),
            note: update.note.clone(),
            updated_at: now(),
            saved: 0,
            skipped: 0,
            usage: update.usage.clone(),
        };
        if cancel.is_cancelled() {
            report.status = "stopped".into();
            report.note = "Learning stopped; the completed reply is saved.".into();
        } else if !fresh {
            report.status = "changed".into();
            report.note = "Sources, preferences or policy changed. Nothing was saved.".into();
        } else if update.status == "completed" {
            // Revalidate model JSON/evidence at the atomic publication boundary.
            let answer = serde_json::json!({"suggestions":update.candidates}).to_string();
            let candidates = if update.candidates.is_empty() {
                Vec::new()
            } else {
                parse_automatic_memories(&answer, &update.source)?
            };
            let desired_scope = if root.is_empty() {
                MemoryScope::All
            } else {
                MemoryScope::Folder
            };
            for c in candidates {
                let normalized = |s: &str| {
                    s.split_whitespace()
                        .collect::<Vec<_>>()
                        .join(" ")
                        .to_lowercase()
                };
                let subject = |title: &str| {
                    normalized(title.split_once(':').map_or(title, |(_, subject)| subject))
                };
                let same = current
                    .iter()
                    .find(|m| m.scope == desired_scope && subject(&m.title) == subject(&c.title));
                if current
                    .iter()
                    .any(|m| normalized(&m.text) == normalized(&c.text))
                {
                    report.skipped += 1;
                    continue;
                }
                let previous = match same {
                    Some(m)
                        if m.source == "automatic"
                            && m.auto_update
                            && m.enabled
                            && (explicit_correction(&c.quote)
                                || (m.title.starts_with("Open work:")
                                    && c.title.starts_with("Outcome:"))) =>
                    {
                        Some(m.clone())
                    }
                    Some(_) => {
                        report.skipped += 1;
                        continue;
                    }
                    None => None,
                };
                if previous.is_none()
                    && current
                        .iter()
                        .filter(|m| m.scope == desired_scope && m.source == "automatic")
                        .count()
                        >= MAX_AUTOMATIC_MEMORIES_PER_SCOPE
                {
                    report.skipped += 1;
                    report.note = "Memory reached its 128 automatic-record limit for this scope. Forget an unused memory, then finish a new interaction; no records were silently removed.".into();
                    continue;
                }
                if previous.is_none() {
                    let count: usize = tx.query_row("SELECT count(*) FROM memory_preferences WHERE json_extract(data,'$.source')='automatic'", [], |r|r.get(0)).map_err(storage_error)?;
                    if count >= 8192 {
                        report.skipped += 1;
                        report.note = "Memory reached its 8192 automatic-record application limit. Forget unused memories before another interaction; existing work is retained.".into();
                        continue;
                    }
                }
                let timestamp = now();
                if let Some(origin) = previous.as_ref().and_then(|m| m.origin.as_ref()) {
                    super::memory::mark_source(&tx, &root, origin, false)?;
                }
                let value = MemoryPreference {
                    id: previous
                        .as_ref()
                        .map(|p| p.id.clone())
                        .unwrap_or_else(|| format!("auto-{}", uuid::Uuid::new_v4().simple())),
                    revision: previous.as_ref().map_or(Ok(1), |p| {
                        p.revision
                            .checked_add(1)
                            .ok_or("Memory revision limit reached.")
                    })?,
                    title: c.title.clone(),
                    text: c.text.clone(),
                    scope: desired_scope,
                    source: "automatic".into(),
                    enabled: true,
                    created_at: previous.as_ref().map_or(timestamp, |p| p.created_at),
                    updated_at: timestamp,
                    origin: Some(c.origin(&update.session, &update.model, timestamp)),
                    auto_update: true,
                    image: None,
                };
                value.validate()?;
                tx.execute("INSERT INTO memory_preferences(id,root,data) VALUES(?1,?2,?3) ON CONFLICT(id) DO UPDATE SET data=excluded.data",params![value.id,root,serde_json::to_string(&value).map_err(storage_error)?]).map_err(storage_error)?;
                current.retain(|m| m.id != value.id);
                current.push(value);
                report.saved += 1;
            }
            if report.note.is_empty() {
                report.note = format!(
                    "{} saved · {} duplicates, conflicts or protected entries skipped",
                    report.saved, report.skipped
                );
            }
            if let Some(caption) = &update.image {
                caption.validate()?;
                let parts = super::attachments::parts(&tx, update.source.message_id)?;
                use sha2::{Digest, Sha256};
                let bytes: Option<Vec<u8>> = tx
                    .query_row(
                        "SELECT data FROM attachment_assets WHERE digest=?1",
                        [&caption.asset.digest],
                        |r| r.get(0),
                    )
                    .optional()
                    .map_err(storage_error)?;
                let available = parts.contains(&caption.asset)
                    && bytes.is_some_and(|b| {
                        b.len() == caption.asset.bytes
                            && format!("{:x}", Sha256::digest(&b)) == caption.asset.digest
                    });
                let count:usize=tx.query_row("SELECT count(*) FROM memory_preferences WHERE root=?1 AND json_extract(data,'$.source')='automatic'",[&root],|r|r.get(0)).map_err(storage_error)?;
                let total:usize=tx.query_row("SELECT count(*) FROM memory_preferences WHERE json_extract(data,'$.source')='automatic'",[],|r|r.get(0)).map_err(storage_error)?;
                if available
                    && count < MAX_AUTOMATIC_MEMORIES_PER_SCOPE
                    && total < 8192
                    && !current.iter().any(|m| {
                        m.scope == desired_scope
                            && m.image
                                .as_ref()
                                .is_some_and(|a| a.digest == caption.asset.digest)
                    })
                {
                    let timestamp = now();
                    let value = MemoryPreference {
                        id: format!("auto-{}", uuid::Uuid::new_v4().simple()),
                        revision: 1,
                        title: caption.title.clone(),
                        text: format!(
                            "{}\nUncertainty: {}",
                            caption.description, caption.uncertainty
                        ),
                        scope: desired_scope,
                        source: "automatic".into(),
                        enabled: true,
                        created_at: timestamp,
                        updated_at: timestamp,
                        origin: Some(MemoryOrigin {
                            session: update.session.clone(),
                            message_id: update.source.message_id,
                            quote: format!(
                                "Shared image: {} · asset {}",
                                caption.asset.name, caption.asset.digest
                            ),
                            model: update.model.clone(),
                            reviewed_at: timestamp,
                        }),
                        auto_update: true,
                        image: Some(caption.asset.clone()),
                    };
                    value.validate()?;
                    tx.execute(
                        "INSERT INTO memory_preferences(id,root,data) VALUES(?1,?2,?3)",
                        params![
                            value.id,
                            root,
                            serde_json::to_string(&value).map_err(storage_error)?
                        ],
                    )
                    .map_err(storage_error)?;
                    report.saved += 1;
                } else {
                    report.skipped += 1;
                    report.note.push_str(" Image duplicate, unavailable or memory full; inspect Memory, reattach or forget an unused record before a new interaction.");
                }
            }
        }
        if cancel.is_cancelled() && report.saved > 0 {
            return Err("Automatic memory stopped before saving.".into());
        }
        tx.execute(
            "UPDATE automatic_memory_attempts SET data=?1 WHERE session_id=?2",
            params![
                serde_json::to_string(&report).map_err(storage_error)?,
                update.session
            ],
        )
        .map_err(storage_error)?;
        tx.commit().map_err(storage_error)?;
        Ok(report)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio_util::sync::CancellationToken;
    fn prepare(store: &SqliteStore, session: &str, text: &str) -> AutomaticMemoryUpdate {
        // Existing explicit On choice from the earlier preference system.
        store
            .lock()
            .unwrap()
            .execute(
                "INSERT OR IGNORE INTO automatic_memory_policy VALUES(1,?1)",
                [r#"{"enabled":true,"revision":1}"#],
            )
            .unwrap();
        store
            .commit_turn(session, text, "ASSISTANT_EXCLUDED")
            .unwrap();
        let source = store.memory_source_messages(session).unwrap().items[0].clone();
        let p = store.automatic_memory_policy().unwrap();
        assert!(store
            .claim_automatic_memory(session, &source, p.revision)
            .unwrap());
        let root = store.workspace(session).unwrap().root;
        AutomaticMemoryUpdate {
            image: None,
            session: session.into(),
            source: source.clone(),
            policy_revision: p.revision,
            existing: store.memory_preferences(root.as_deref()).unwrap(),
            candidates: vec![MemorySuggestion {
                title: "Response style".into(),
                text: text.into(),
                message_id: source.message_id,
                quote: text.into(),
            }],
            model: "fixture".into(),
            status: "completed".into(),
            note: String::new(),
            usage: None,
        }
    }
    #[test]
    fn automatic_create_duplicate_explicit_replace_manual_protection_delete_and_restart() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("test.db");
        let store = SqliteStore::open(&path).unwrap();
        let root = dir.path().to_str().unwrap();
        store
            .create_workspace_session(
                "project",
                &SessionWorkspace {
                    kind: WorkspaceKind::Project,
                    root: Some(root.into()),
                },
            )
            .unwrap();
        store.create("side").unwrap();
        let cancel = CancellationToken::new();
        let initial = prepare(&store, "project", "I prefer concise replies.");
        assert_eq!(
            store
                .finish_automatic_memory(&initial, &cancel)
                .unwrap()
                .saved,
            1
        );
        assert!(!store
            .claim_automatic_memory("project", &initial.source, 1)
            .unwrap());
        assert!(store.finish_automatic_memory(&initial, &cancel).is_err());
        assert!(store.memory_preferences(None).unwrap().is_empty());
        let original = store.memory_preferences(Some(root)).unwrap()[0].clone();
        assert_eq!(original.source, "automatic");
        assert!(original.auto_update);
        let duplicate = prepare(&store, "project", "I prefer concise replies.");
        assert_eq!(
            store
                .finish_automatic_memory(&duplicate, &cancel)
                .unwrap()
                .skipped,
            1
        );
        assert_eq!(store.memory_preferences(Some(root)).unwrap()[0].revision, 1);
        let conflict = prepare(&store, "project", "I prefer detailed replies.");
        assert_eq!(
            store
                .finish_automatic_memory(&conflict, &cancel)
                .unwrap()
                .saved,
            0
        );
        let correction = prepare(
            &store,
            "project",
            "From now on I prefer detailed replies instead.",
        );
        assert_eq!(
            store
                .finish_automatic_memory(&correction, &cancel)
                .unwrap()
                .saved,
            1
        );
        let corrected = store.memory_preferences(Some(root)).unwrap()[0].clone();
        assert_eq!(corrected.id, original.id);
        assert_eq!(corrected.revision, 2);
        store
            .save_memory_preference(
                Some(root),
                &MemoryDraft {
                    id: corrected.id.clone(),
                    revision: Some(2),
                    title: corrected.title.clone(),
                    text: "USER_CORRECTED_STYLE".into(),
                    enabled: true,
                    origin: None,
                },
            )
            .unwrap();
        let protected = prepare(
            &store,
            "project",
            "From now on I prefer concise replies instead.",
        );
        assert_eq!(
            store
                .finish_automatic_memory(&protected, &cancel)
                .unwrap()
                .skipped,
            1
        );
        let edited = store.memory_preferences(Some(root)).unwrap()[0].clone();
        assert!(!edited.auto_update);
        assert_eq!(edited.origin, corrected.origin);
        store
            .delete_memory_preference(Some(root), &edited.id, edited.revision)
            .unwrap();
        assert!(!store
            .claim_automatic_memory("project", &protected.source, 1)
            .unwrap());
        // A newly stated preference can be learned again; deleted evidence is never replayed.
        let fresh = prepare(&store, "project", "I prefer concise replies.");
        assert_eq!(
            store
                .finish_automatic_memory(&fresh, &cancel)
                .unwrap()
                .saved,
            1
        );
        let global = prepare(&store, "side", "I prefer clear examples.");
        assert_eq!(
            store
                .finish_automatic_memory(&global, &cancel)
                .unwrap()
                .saved,
            1
        );
        drop(store);
        let store = SqliteStore::open(&path).unwrap();
        assert_eq!(store.memory_preferences(None).unwrap().len(), 1);
        assert_eq!(store.memory_preferences(Some(root)).unwrap().len(), 2);
        assert_eq!(
            store
                .automatic_memory_attempt("project")
                .unwrap()
                .unwrap()
                .message_id,
            fresh.source.message_id
        );
        store.delete("project").unwrap();
        assert!(store.automatic_memory_attempt("project").is_err());
        assert_eq!(store.memory_preferences(Some(root)).unwrap().len(), 2);
    }
    #[test]
    fn changed_scope_source_policy_cancel_and_sql_failure_cannot_publish_partial_updates() {
        let dir = tempfile::tempdir().unwrap();
        let store = SqliteStore::open(&dir.path().join("test.db")).unwrap();
        store.create("chat").unwrap();
        let cancel = CancellationToken::new();
        let update = prepare(&store, "chat", "I prefer concise replies.");
        store.set_automatic_memory_policy(false, 1).unwrap();
        assert_eq!(
            store
                .finish_automatic_memory(&update, &cancel)
                .unwrap()
                .status,
            "changed"
        );
        assert!(store.memory_preferences(None).unwrap().is_empty());
        assert!(!store
            .claim_automatic_memory("chat", &update.source, 2)
            .unwrap());
        assert!(store.set_automatic_memory_policy(true, 1).is_err());
        store.set_automatic_memory_policy(true, 2).unwrap();
        let canceled = prepare(&store, "chat", "I prefer concise replies.");
        cancel.cancel();
        assert_eq!(
            store
                .finish_automatic_memory(&canceled, &cancel)
                .unwrap()
                .status,
            "stopped"
        );
        let cancel = CancellationToken::new();
        let changed = prepare(&store, "chat", "I prefer concise replies.");
        store
            .lock()
            .unwrap()
            .execute(
                "UPDATE messages SET content='changed' WHERE id=?1",
                [changed.source.message_id],
            )
            .unwrap();
        assert_eq!(
            store
                .finish_automatic_memory(&changed, &cancel)
                .unwrap()
                .status,
            "changed"
        );
        let stale = prepare(&store, "chat", "I prefer concise replies.");
        store
            .save_memory_preference(
                None,
                &MemoryDraft {
                    id: "manual".into(),
                    revision: None,
                    title: "Response style".into(),
                    text: "Manual style".into(),
                    enabled: true,
                    origin: None,
                },
            )
            .unwrap();
        assert_eq!(
            store
                .finish_automatic_memory(&stale, &cancel)
                .unwrap()
                .status,
            "changed"
        );
        let protected = prepare(
            &store,
            "chat",
            "From now on I prefer concise replies instead.",
        );
        assert_eq!(
            store
                .finish_automatic_memory(&protected, &cancel)
                .unwrap()
                .saved,
            0
        );
        store.delete_memory_preference(None, "manual", 1).unwrap();
        let mut failed = prepare(
            &store,
            "chat",
            "I prefer concise replies. I always run tests.",
        );
        failed.candidates.push(MemorySuggestion {
            title: "Testing".into(),
            text: "I always run tests.".into(),
            quote: "I always run tests.".into(),
            message_id: failed.source.message_id,
        });
        store.lock().unwrap().execute_batch("CREATE TRIGGER reject_second BEFORE INSERT ON memory_preferences WHEN NEW.data LIKE '%Testing%' BEGIN SELECT RAISE(ABORT,'fixture'); END;").unwrap();
        assert!(store.finish_automatic_memory(&failed, &cancel).is_err());
        assert!(store.memory_preferences(None).unwrap().is_empty());
        assert!(!store
            .claim_automatic_memory("chat", &failed.source, 3)
            .unwrap());
        assert_eq!(store.messages("chat").unwrap().len(), 12);
    }
}
