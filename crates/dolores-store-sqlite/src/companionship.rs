use crate::{storage_error, SqliteStore};
use dolores_core::companionship::*;
use rusqlite::{params, Connection, OptionalExtension};
pub(super) fn read(c: &Connection) -> Result<CompanionState, String> {
    let data: Option<String> = c
        .query_row("SELECT data FROM companion_state WHERE id=1", [], |r| {
            r.get(0)
        })
        .optional()
        .map_err(storage_error)?;
    let state: CompanionState = data
        .map(|v| serde_json::from_str(&v).map_err(storage_error))
        .transpose()?
        .unwrap_or_default();
    state.validate()?;
    Ok(state)
}
pub(super) fn write(c: &Connection, state: &CompanionState) -> Result<(), String> {
    state.validate()?;
    c.execute("INSERT INTO companion_state(id,data) VALUES(1,?1) ON CONFLICT(id) DO UPDATE SET data=excluded.data",[serde_json::to_string(state).map_err(storage_error)?]).map_err(storage_error)?;
    Ok(())
}
pub(super) fn forget_source(c: &Connection, root: &str, id: &str) -> Result<(), String> {
    let mut s = read(c)?;
    let old = s.clone();
    let matches = |source: &CompanionSource| {
        source.memory == id && source.root.as_deref().unwrap_or("") == root
    };
    if s.pending
        .as_ref()
        .and_then(|p| p.source.as_ref())
        .is_some_and(matches)
    {
        s.pending = None;
    }
    for a in &mut s.activity {
        if a.source.as_ref().is_some_and(matches) {
            a.source = None;
            a.note = Some("Source forgotten.".into());
            if a.status == "generating" {
                a.status = "cancelled".into();
            }
        }
    }
    if old != s {
        s.revision += 1;
        write(c, &s)?;
    }
    Ok(())
}
impl SqliteStore {
    pub(super) fn companion_recover(&self) -> Result<(), String> {
        let mut c = self.lock()?;
        let tx = c.transaction().map_err(storage_error)?;
        let mut s = read(&tx)?;
        if let Some(p) = s.pending.take() {
            if let Some(a) = s.activity.iter_mut().find(|a| a.id == p.id) {
                a.status = "interrupted".into();
                a.note = Some("Generation stopped when Dolores closed.".into());
            }
            s.revision += 1;
            write(&tx, &s)?;
        }
        tx.commit().map_err(storage_error)
    }
    #[allow(clippy::too_many_arguments)]
    pub(super) fn companion_finish(
        &self,
        id: &str,
        body: Option<&str>,
        note: Option<&str>,
        usage: Option<dolores_core::TokenUsage>,
        now: i64,
        present: bool,
        busy: bool,
    ) -> Result<Option<String>, String> {
        let mut c = self.lock()?;
        let tx = c.transaction().map_err(storage_error)?;
        let mut s = read(&tx)?;
        let Some(p) = s.pending.clone().filter(|p| p.id == id) else {
            return Ok(None);
        };
        let (_, hours) = s.policy.local(now)?;
        let mut valid = s.policy.enabled
            && s.policy.revision == p.policy_revision
            && now >= p.created
            && now < p.expires
            && hours
            && s.snooze_until <= now
            && s.unread().is_none()
            && present
            && !busy;
        if let Some(source) = &p.source {
            let enabled: Option<String> = tx
                .query_row(
                    "SELECT data FROM automatic_memory_policy WHERE id=1",
                    [],
                    |r| r.get(0),
                )
                .optional()
                .map_err(storage_error)?;
            valid &= enabled
                .and_then(|d| serde_json::from_str::<dolores_core::AutomaticMemoryPolicy>(&d).ok())
                .is_some_and(|p| p.enabled);
            let data: Option<String> = tx
                .query_row(
                    "SELECT data FROM memory_preferences WHERE root=?1 AND id=?2",
                    params![source.root.as_deref().unwrap_or(""), source.memory],
                    |r| r.get(0),
                )
                .optional()
                .map_err(storage_error)?;
            valid &= data.and_then(|d| serde_json::from_str::<serde_json::Value>(&d).ok())
                == serde_json::from_str::<serde_json::Value>(&source.signature).ok();
            let text:Option<String>=tx.query_row("SELECT m.content FROM messages m WHERE m.session_id=?1 AND m.id=?2 AND m.role='user' AND EXISTS(SELECT 1 FROM messages a WHERE a.session_id=m.session_id AND a.id=m.id+1 AND a.role='assistant')",params![source.session,source.message_id],|r|r.get(0)).optional().map_err(storage_error)?;
            valid &= text.is_some_and(|t| !source.quote.is_empty() && t.contains(&source.quote));
            let root: Option<Option<String>> = tx
                .query_row(
                    "SELECT root FROM session_workspaces WHERE session_id=?1",
                    [&source.session],
                    |r| r.get(0),
                )
                .optional()
                .map_err(storage_error)?;
            valid &= root.flatten() == p.workspace.root
                && (source.root.is_none() || source.root == p.workspace.root);
            if source.kind == "openWork" {
                let latest:i64=tx.query_row("SELECT COALESCE(MAX(id),0) FROM messages WHERE session_id=?1 AND role='user'",[&source.session],|r|r.get(0)).map_err(storage_error)?;
                valid &= latest == source.message_id;
            }
        }
        let body = body.filter(|b| !b.is_empty() && b.len() <= 2048);
        let session = if let Some(body) = body.filter(|_| valid) {
            let session = format!("companion-{id}");
            let kind = match p.workspace.kind {
                dolores_core::WorkspaceKind::Project => "project",
                dolores_core::WorkspaceKind::Temporary => "temporary",
                dolores_core::WorkspaceKind::Side => "side",
            };
            tx.execute(
                "INSERT INTO sessions(id,title,updated_at) VALUES(?1,'From Dolores',?2)",
                params![session, crate::now()],
            )
            .map_err(storage_error)?;
            tx.execute(
                "INSERT INTO session_workspaces(session_id,kind,root) VALUES(?1,?2,?3)",
                params![session, kind, p.workspace.root],
            )
            .map_err(storage_error)?;
            tx.execute("INSERT INTO messages(session_id,role,content) VALUES(?1,'user','A note from Dolores'),(?1,'assistant',?2)",params![session,body]).map_err(storage_error)?;
            let last = tx.last_insert_rowid();
            tx.execute(
                "INSERT INTO message_timestamps(message_id,saved_at) VALUES(?1,?3),(?2,?3)",
                params![last - 1, last, crate::now()],
            )
            .map_err(storage_error)?;
            Some(session)
        } else {
            None
        };
        if let Some(a) = s.activity.iter_mut().find(|a| a.id == id) {
            a.status = if session.is_some() {
                "delivered"
            } else if body.is_none() {
                "failed"
            } else {
                "cancelled"
            }
            .into();
            a.session = session.clone();
            a.seen = session.is_none();
            a.note = note.map(str::to_string);
            a.usage = usage;
        }
        s.pending = None;
        s.revision += 1;
        write(&tx, &s)?;
        tx.commit().map_err(storage_error)?;
        Ok(session)
    }
    pub(super) fn companion_read(&self) -> Result<CompanionState, String> {
        let mut c = self.lock()?;
        let tx = c.transaction().map_err(storage_error)?;
        let mut s = read(&tx)?;
        let old = s.clone();
        for a in &mut s.activity {
            if !a.seen {
                if let Some(session) = &a.session {
                    let exists: bool = tx
                        .query_row(
                            "SELECT EXISTS(SELECT 1 FROM sessions WHERE id=?1)",
                            [session],
                            |r| r.get(0),
                        )
                        .map_err(storage_error)?;
                    if !exists {
                        a.seen = true;
                    }
                }
            }
        }
        if old != s {
            s.revision += 1;
            write(&tx, &s)?;
        }
        tx.commit().map_err(storage_error)?;
        Ok(s)
    }
    pub(super) fn companion_save(
        &self,
        state: &CompanionState,
        revision: u64,
    ) -> Result<CompanionState, String> {
        let mut c = self.lock()?;
        let tx = c.transaction().map_err(storage_error)?;
        let old = read(&tx)?;
        if old.revision != revision {
            return Err("Companionship changed. Refresh before saving.".into());
        }
        let mut next = state.clone();
        next.revision = revision
            .checked_add(1)
            .ok_or("Companion revision exhausted.")?;
        write(&tx, &next)?;
        tx.commit().map_err(storage_error)?;
        Ok(next)
    }
    pub(super) fn companion_claim(
        &self,
        now: i64,
        present: bool,
        busy: bool,
        jitter: u32,
        candidate: CompanionCandidate,
    ) -> Result<Option<CompanionCandidate>, String> {
        let mut c = self.lock()?;
        let tx = c.transaction().map_err(storage_error)?;
        let mut state = read(&tx)?;
        let old = state.clone();
        let claimed = state.reserve(now, present, busy, jitter, candidate.clone())?;
        if old != state {
            state.revision = state
                .revision
                .checked_add(1)
                .ok_or("Companion revision exhausted.")?;
            write(&tx, &state)?;
        }
        tx.commit().map_err(storage_error)?;
        Ok(claimed.then_some(candidate))
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use dolores_core::SessionStore;
    fn claimed(s: &SqliteStore, now: i64) -> CompanionCandidate {
        let mut state = s.companion_state().unwrap();
        state.policy.enabled = true;
        state.policy.model = "weak".into();
        state.policy.start = 0;
        state.policy.end = 1439;
        state.next_opportunity = Some(now);
        let rev = state.revision;
        let state = s.save_companion_state(&state, rev).unwrap();
        let c = CompanionCandidate {
            id: uuid::Uuid::new_v4().to_string(),
            policy_revision: state.policy.revision,
            created: now,
            expires: now + EXPIRY,
            workspace: Default::default(),
            kind: "chat".into(),
            body: String::new(),
            citation: None,
            source: None,
        };
        s.claim_companion(now, true, false, 0, c).unwrap().unwrap()
    }
    #[test]
    fn publication_is_atomic_and_disable_restart_and_duplicate_finish_stay_quiet() {
        let s = SqliteStore::open(std::path::Path::new(":memory:")).unwrap();
        let now = 1_800_000_000;
        let c = claimed(&s, now);
        let session = s
            .finish_companion(
                &c.id,
                Some("Would you like a break?"),
                None,
                None,
                now + 1,
                true,
                false,
            )
            .unwrap()
            .unwrap();
        assert_eq!(s.messages(&session).unwrap().len(), 2);
        assert!(s.companion_state().unwrap().unread().is_some());
        assert!(s
            .finish_companion(&c.id, Some("duplicate"), None, None, now + 2, true, false)
            .unwrap()
            .is_none());
        let s = SqliteStore::open(std::path::Path::new(":memory:")).unwrap();
        let c = claimed(&s, now);
        let mut state = s.companion_state().unwrap();
        state.policy.enabled = false;
        let rev = state.revision;
        s.save_companion_state(&state, rev).unwrap();
        assert!(s
            .finish_companion(&c.id, Some("late"), None, None, now + 1, true, false)
            .unwrap()
            .is_none());
        assert!(s.list().unwrap().is_empty());
        let s = SqliteStore::open(std::path::Path::new(":memory:")).unwrap();
        let c = claimed(&s, now);
        s.recover_companion().unwrap();
        assert!(s
            .finish_companion(&c.id, Some("late"), None, None, now + 1, true, false)
            .unwrap()
            .is_none());
        assert_eq!(s.companion_state().unwrap().attempts, 1);
    }
    #[test]
    fn failed_publication_rolls_back_conversation_and_retains_candidate() {
        let s = SqliteStore::open(std::path::Path::new(":memory:")).unwrap();
        let now = 1_800_000_000;
        let c = claimed(&s, now);
        s.lock().unwrap().execute_batch("CREATE TRIGGER deny_finish BEFORE UPDATE ON companion_state BEGIN SELECT RAISE(ABORT,'fixture'); END;").unwrap();
        assert!(s
            .finish_companion(
                &c.id,
                Some("Would you like a break?"),
                None,
                None,
                now + 1,
                true,
                false
            )
            .is_err());
        assert!(s.list().unwrap().is_empty());
        assert!(s.companion_state().unwrap().pending.is_some());
    }
    #[test]
    fn forgotten_disabled_or_updated_open_work_cannot_publish() {
        use dolores_core::{MemoryDraft, MemoryOrigin};
        for action in ["forget", "off", "newTurn", "valid"] {
            let s = SqliteStore::open(std::path::Path::new(":memory:")).unwrap();
            s.create("source").unwrap();
            s.commit_turn(
                "source",
                "We still need to test the report export.",
                "Okay.",
            )
            .unwrap();
            let m = s.memory_source_messages("source").unwrap().items[0].clone();
            let p = s
                .save_suggested_memory_preference(
                    None,
                    &MemoryDraft {
                        id: "open-export".into(),
                        revision: None,
                        title: "Open work: export".into(),
                        text: m.text.clone(),
                        enabled: true,
                        origin: Some(MemoryOrigin {
                            session: "source".into(),
                            message_id: m.message_id,
                            quote: m.text.clone(),
                            model: "fixture".into(),
                            reviewed_at: 1,
                        }),
                    },
                    std::slice::from_ref(&m),
                )
                .unwrap();
            s.set_automatic_memory_policy(true, 1).unwrap();
            let now = 1_800_000_000;
            let c = claimed(&s, now);
            let mut state = s.companion_state().unwrap();
            state.pending.as_mut().unwrap().source = Some(CompanionSource {
                root: None,
                memory: p.id.clone(),
                signature: serde_json::to_string(&p).unwrap(),
                session: "source".into(),
                message_id: m.message_id,
                quote: m.text.clone(),
                kind: "openWork".into(),
            });
            let rev = state.revision;
            s.save_companion_state(&state, rev).unwrap();
            match action {
                "forget" => s.delete_memory_preference(None, &p.id, p.revision).unwrap(),
                "off" => {
                    s.set_automatic_memory_policy(false, 2).unwrap();
                }
                "newTurn" => s
                    .commit_turn("source", "The report export tests passed.", "Done.")
                    .unwrap(),
                _ => {}
            }
            let published = s
                .finish_companion(
                    &c.id,
                    Some("Would you like to revisit this?"),
                    None,
                    None,
                    now + 1,
                    true,
                    false,
                )
                .unwrap();
            assert_eq!(published.is_some(), action == "valid");
            assert_eq!(
                s.list().unwrap().len(),
                if action == "valid" { 2 } else { 1 }
            );
        }
    }
    #[test]
    fn restart_retains_daily_cap_and_stale_or_failed_saves_preserve_policy() {
        let d = tempfile::tempdir().unwrap();
        let p = d.path().join("db");
        let s = SqliteStore::open(&p).unwrap();
        let mut state = s.companion_state().unwrap();
        assert!(!state.policy.enabled);
        state.policy.enabled = true;
        state.policy.model = "weak".into();
        state.attempts = 2;
        state.day = "2026-10-07".into();
        let saved = s.save_companion_state(&state, 0).unwrap();
        assert!(s.save_companion_state(&state, 0).is_err());
        s.lock().unwrap().execute_batch("CREATE TRIGGER deny_companion BEFORE UPDATE ON companion_state BEGIN SELECT RAISE(ABORT,'fixture'); END;").unwrap();
        state.policy.enabled = false;
        assert!(s.save_companion_state(&state, saved.revision).is_err());
        drop(s);
        assert_eq!(
            SqliteStore::open(&p).unwrap().companion_state().unwrap(),
            saved
        );
    }
}
