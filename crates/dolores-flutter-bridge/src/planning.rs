use super::*;
use dolores_core::planning::*;
use serde::Serialize;
use sha2::{Digest, Sha256};
#[derive(Clone, Deserialize, Serialize)]
#[serde(tag = "kind", rename_all = "camelCase", deny_unknown_fields)]
pub(super) enum Source {
    File { path: String },
    Folder { path: String },
    Code { digest: String },
    Terminal { digest: String },
    Message { session: String, message: i64 },
    Conversation { session: String },
}
#[derive(Deserialize)]
#[serde(tag = "action", rename_all = "camelCase", deny_unknown_fields)]
pub(super) enum Edit {
    Mode { enabled: bool },
    Draft { text: String, decisions: String },
    Accept,
    Add { source: Source, expected: String },
    Remove { digest: String },
}
pub(super) fn check_goal(
    store: &dyn SessionStore,
    s: &str,
    state: &PlanningState,
) -> Result<(), String> {
    if let Some(p) = &state.accepted {
        if let Some(id) = &p.goal {
            let g = store
                .goal(s)?
                .ok_or("Accepted plan's goal is missing. Review and accept the plan again.")?;
            if &g.id != id
                || g.steps
                    .iter()
                    .map(|s| s.criterion.clone())
                    .collect::<Vec<_>>()
                    != p.criteria
            {
                return Err("Goal/checklist changed since plan acceptance. Review and accept again; nothing started.".into());
            }
        }
    }
    Ok(())
}
pub(super) fn save_result(
    store: &dyn SessionStore,
    s: &str,
    state: &PlanningState,
    text: &str,
) -> Result<(), String> {
    if !state.planning {
        return Ok(());
    }
    let mut updated = state.clone();
    updated.draft = text.into();
    updated.accepted = None;
    store.save_planning(s, state.revision, &updated)?;
    Ok(())
}
impl Engine {
    pub(super) fn reference_choices(
        &self,
        s: &str,
        kind: &str,
        owner: Option<&str>,
        path: &str,
        cursor: usize,
    ) -> Result<Value, String> {
        let scope = self.store.workspace(s)?.root;
        if matches!(kind, "file" | "folder") {
            let root = scope.ok_or("Choose a working chat before browsing files.")?;
            let tree = dolores_tools_fs::EditorFolder::new(std::path::Path::new(&root))?
                .tree(path, cursor)?;
            let mut choices = vec![];
            if kind == "folder" {
                choices.push(json!({"title":format!("Share listing: {path}"),"source":{"kind":"folder","path":path}}));
            }
            for e in tree["entries"].as_array().ok_or("Directory unavailable.")? {
                if e["directory"] == true {
                    choices.push(json!({"title":e["name"],"folder":e["path"]}));
                } else if kind == "file" {
                    choices
                        .push(json!({"title":e["name"],"source":{"kind":"file","path":e["path"]}}));
                }
            }
            return Ok(json!({"choices":choices,"cursor":tree["cursor"]}));
        }
        if matches!(kind, "code" | "terminal") {
            return Ok(
                json!({"choices":self.store.draft_attachments(s)?.into_iter().filter(|p|!p.is_image() && if kind=="code"{p.name.starts_with("Selection v")}else{p.name.to_lowercase().contains("terminal")}).map(|p|json!({"title":p.name,"source":{"kind":kind,"digest":p.digest}})).collect::<Vec<_>>(),"notice":"Use Folders or Terminal Attach selection first, then Refresh. Only explicit text excerpts are listed."}),
            );
        }
        if !matches!(kind, "message" | "conversation") {
            return Err("Unknown context kind.".into());
        }
        if kind == "message" {
            if let Some(other) = owner {
                if other != s && (scope.is_none() || scope != self.store.workspace(other)?.root) {
                    return Err("Choose a chat in this working folder.".into());
                }
                let p = self.store.messages_page(other, None, false, 80)?;
                return Ok(
                    json!({"choices":p.items.iter().map(|m|json!({"title":format!("{:?}: {}",m.role,m.content.chars().take(100).collect::<String>()),"source":{"kind":"message","session":other,"message":m.id}})).collect::<Vec<_>>(),"notice":"Latest 80 messages; attachments omitted."}),
                );
            }
        }
        let p = self.store.sessions_page(None, false, 80)?;
        let mut choices = vec![];
        for chat in p.items {
            if chat.id == s || (scope.is_some() && self.store.workspace(&chat.id)?.root == scope) {
                choices.push(if kind == "message" {
                    json!({"title":chat.title,"owner":chat.id})
                } else {
                    json!({"title":chat.title,"source":{"kind":"conversation","session":chat.id}})
                });
            }
        }
        Ok(
            json!({"choices":choices,"notice":"Latest 80 chats filtered to this working folder; choose a narrower excerpt if needed."}),
        )
    }
    pub(super) fn reference_preview(
        &self,
        s: &str,
        source: &Source,
    ) -> Result<ContextReference, String> {
        let workspace = self.store.workspace(s)?;
        let mut omissions = String::new();
        let (kind, label, text) = match source {
            Source::File { path } | Source::Folder { path } => {
                let root = workspace
                    .root
                    .ok_or("Choose a working chat before selecting files.")?;
                let fs = dolores_tools_fs::EditorFolder::new(std::path::Path::new(&root))?;
                if matches!(source, Source::Folder { .. }) {
                    let tree = fs.tree(path, 0)?;
                    omissions="One directory page only (up to 200 entries); file contents and subfolders omitted.".into();
                    if !tree["cursor"].is_null() {
                        omissions.push_str(" More entries omitted; select a narrower folder.");
                    }
                    (
                        "folder",
                        path.clone(),
                        serde_json::to_string_pretty(&tree)
                            .map_err(|_| "Could not preview folder.")?,
                    )
                } else {
                    let snapshot = fs.snapshot(path)?;
                    if snapshot.reason.is_some() {
                        return Err(
                            "Select a readable UTF-8 file within the source limit; nothing shared."
                                .into(),
                        );
                    }
                    ("file", path.clone(), snapshot.text)
                }
            }
            Source::Code { digest } | Source::Terminal { digest } => {
                let asset = self.store.attachment_data(s, digest)?;
                if asset.reference.is_image() {
                    return Err("Choose an explicitly attached text excerpt.".into());
                }
                let code = matches!(source, Source::Code { .. });
                if !(if code {
                    asset.reference.name.starts_with("Selection v")
                } else {
                    asset.reference.name.to_lowercase().contains("terminal")
                }) {
                    return Err("Use Folders' Attach selection or Terminal's Attach selection first, then refresh this picker.".into());
                }
                (
                    if code { "code" } else { "terminal" },
                    asset.reference.name,
                    String::from_utf8(asset.data).map_err(|_| "Excerpt is not UTF-8.")?,
                )
            }
            Source::Message { session, .. } | Source::Conversation { session } => {
                let other = self.store.workspace(session)?;
                if session != s && (workspace.root.is_none() || workspace.root != other.root) {
                    return Err("Select a conversation in this working folder. Other chat scopes are not shared.".into());
                }
                let page = self.store.messages_page(session, None, false, 80)?;
                let text = if let Source::Message { message, .. } = source {
                    let m=page.items.iter().find(|m|m.id==*message).ok_or("Message deleted or outside the latest 80. Refresh or choose a recent message.")?;
                    format!("Message {} ({:?}): {}", m.id, m.role, m.content)
                } else {
                    omissions =
                        "Latest 80 messages only; attachments and earlier history omitted.".into();
                    serde_json::to_string(
                        &page
                            .items
                            .iter()
                            .map(|m| json!({"id":m.id,"role":m.role,"text":m.content}))
                            .collect::<Vec<_>>(),
                    )
                    .map_err(|_| "Could not preview messages.")?
                };
                (
                    if matches!(source, Source::Message { .. }) {
                        "message"
                    } else {
                        "conversation"
                    },
                    serde_json::to_string(source).map_err(|_| "Source unavailable.")?,
                    text,
                )
            }
        };
        let r = ContextReference {
            kind: kind.into(),
            source: label.clone(),
            revision: format!(
                "{:x}",
                Sha256::digest(
                    serde_json::to_vec(&(kind, &label, &text, &omissions))
                        .map_err(|_| "Could not identify the context snapshot.")?
                )
            ),
            text,
            omissions,
        };
        PlanningState {
            references: vec![r.clone()],
            ..Default::default()
        }
        .validate()?;
        Ok(r)
    }
    pub(super) fn edit_planning(
        &self,
        s: &str,
        revision: u32,
        edit: Edit,
    ) -> Result<Value, String> {
        let mut state = self.store.planning(s)?;
        if state.revision != revision {
            return Err(
                "Plan or context changed. Refresh before saving; your edits remain.".into(),
            );
        }
        match edit {
            Edit::Mode { enabled } => state.planning = enabled,
            Edit::Draft { text, decisions } => {
                state.draft = text;
                state.decisions = decisions;
                state.accepted = None;
            }
            Edit::Accept => {
                if state.draft.trim().is_empty() {
                    return Err("Save a concrete plan before accepting it.".into());
                }
                let goal = self.store.goal(s)?.filter(|g| {
                    !matches!(
                        g.state,
                        dolores_core::goal::GoalState::Completed
                            | dolores_core::goal::GoalState::Cleared
                    )
                });
                state.accepted = Some(AcceptedPlan {
                    revision: revision.checked_add(1).ok_or("Plan revision exhausted.")?,
                    text: state.draft.clone(),
                    decisions: state.decisions.clone(),
                    goal: goal.as_ref().map(|g| g.id.clone()),
                    criteria: goal.map_or(vec![], |g| {
                        g.steps.into_iter().map(|s| s.criterion).collect()
                    }),
                });
                state.planning = true;
            }
            Edit::Add { source, expected } => {
                let r = self.reference_preview(s, &source)?;
                if r.revision != expected {
                    return Err("Source changed since preview. Preview again and reselect; your draft remains.".into());
                }
                if !state.references.contains(&r) {
                    state.references.push(r);
                }
                state.accepted = None;
            }
            Edit::Remove { digest } => {
                state.references.retain(|r| r.revision != digest);
                state.accepted = None;
            }
        }
        Ok(json!(self.store.save_planning(s, revision, &state)?))
    }
}
