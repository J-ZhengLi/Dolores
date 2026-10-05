use super::*;
use dolores_core::{CommandOutcome, KnowledgeFact, KnowledgeSource, KnowledgeState, ToolRecord};
use sha2::{Digest, Sha256};
use std::{io::Read, path::Path};
pub(super) struct FactInput {
    pub id: Option<String>,
    pub title: String,
    pub text: String,
    pub kind: String,
    pub enabled: bool,
    pub inferred: bool,
}
fn clock() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as i64
}
fn digest(text: &str) -> String {
    format!("{:x}", Sha256::digest(text.as_bytes()))
}
fn root(store: &dyn SessionStore, session: &str) -> Result<String, String> {
    store.workspace(session)?.root.ok_or(
        "Project knowledge needs a working chat. Open a project or temporary workspace.".into(),
    )
}
fn same_path(a: &Path, b: &Path) -> bool {
    #[cfg(windows)]
    {
        fn normal(p: &Path) -> Option<String> {
            let text = p.to_str()?;
            Some(if let Some(t) = text.strip_prefix(r"\\?\UNC\") {
                format!(r"\\{t}")
            } else {
                text.strip_prefix(r"\\?\").unwrap_or(text).to_owned()
            })
        }
        normal(a)
            .zip(normal(b))
            .is_some_and(|(a, b)| a.eq_ignore_ascii_case(&b))
    }
    #[cfg(not(windows))]
    {
        a == b
    }
}
// Only previously approved, direct, nonsecret small sources are revalidated.
fn direct_source(root: &str, path: &str) -> Option<PathBuf> {
    if !dolores_core::relative_source(path) {
        return None;
    }
    let original = Path::new(root);
    let root = original.canonicalize().ok()?;
    if !same_path(original, &root) {
        return None;
    }
    let mut full = root.clone();
    for part in path.split('/') {
        full.push(part);
        let m = std::fs::symlink_metadata(&full).ok()?;
        if m.file_type().is_symlink() || !same_path(&full.canonicalize().ok()?, &full) {
            return None;
        }
    }
    if !full.starts_with(&root) {
        return None;
    }
    Some(full)
}
fn current_text(root: &str, path: &str) -> Option<String> {
    let full = direct_source(root, path)?;
    if !full.is_file() {
        return None;
    }
    let mut bytes = Vec::new();
    std::fs::File::open(full)
        .ok()?
        .take(16385)
        .read_to_end(&mut bytes)
        .ok()?;
    if bytes.len() > 16384 || bytes.contains(&0) {
        return None;
    }
    String::from_utf8(bytes).ok()
}
pub(super) fn fresh(root: &str, f: &KnowledgeFact) -> bool {
    if f.basis == "manual" {
        return true;
    }
    if clock().saturating_sub(f.observed_at) > 7 * 24 * 60 * 60 * 1000 {
        return false;
    }
    match f
        .source
        .as_ref()
        .and_then(|s| s.path.as_ref().zip(s.digest.as_ref()))
    {
        Some((path, expected)) if f.kind == "structure" => direct_source(root, path)
            .is_some_and(|p| digest(if p.is_dir() { "folder" } else { "file" }) == *expected),
        Some((path, expected)) => current_text(root, path).is_some_and(|s| digest(&s) == *expected),
        None => true,
    }
}
pub(super) fn facts(store: &dyn SessionStore, session: &str) -> Result<Vec<KnowledgeFact>, String> {
    let Ok(root) = root(store, session) else {
        return Ok(vec![]);
    };
    Ok(store
        .knowledge(&root)?
        .facts
        .into_iter()
        .filter(|f| f.enabled && fresh(&root, f))
        .take(4)
        .collect())
}
#[cfg(test)]
fn prepare(
    store: &dyn SessionStore,
    session: &str,
    messages: Vec<dolores_core::Message>,
) -> Result<Vec<dolores_core::Message>, String> {
    let Ok(root) = root(store, session) else {
        return Ok(messages);
    };
    let state = store.knowledge(&root)?;
    let facts: Vec<_> = state
        .facts
        .into_iter()
        .filter(|f| f.enabled && fresh(&root, f))
        .collect();
    dolores_core::knowledge_context(messages, &facts)
}

impl Engine {
    pub(super) fn project_knowledge(&self, session: &str) -> Result<Value, String> {
        let root = root(self.store.as_ref(), session)?;
        let state = self.store.knowledge(&root)?;
        let facts: Vec<_> = state
            .facts
            .iter()
            .map(|f| {
                let mut v = json!(f);
                v["fresh"] = json!(fresh(&root, f));
                v
            })
            .collect();
        Ok(
            json!({"revision":state.revision,"learning":state.learning,"shareFeedback":state.share_feedback,"facts":facts,"notice":state.notice}),
        )
    }
    pub(super) fn knowledge_policy(
        &self,
        session: &str,
        revision: u32,
        learning: bool,
        feedback: bool,
    ) -> Result<Value, String> {
        let root = root(self.store.as_ref(), session)?;
        let mut state = self.store.knowledge(&root)?;
        state.learning = learning;
        state.share_feedback = feedback;
        self.store.save_knowledge(&root, revision, &state)?;
        self.project_knowledge(session)
    }
    pub(super) fn write_fact(
        &self,
        session: &str,
        revision: u32,
        input: FactInput,
    ) -> Result<Value, String> {
        let FactInput {
            id,
            title,
            text,
            kind,
            enabled,
            inferred,
        } = input;
        let root = root(self.store.as_ref(), session)?;
        let mut state = self.store.knowledge(&root)?;
        let id = id.unwrap_or_else(|| uuid::Uuid::new_v4().to_string());
        let source = state
            .facts
            .iter()
            .find(|f| f.id == id)
            .and_then(|f| f.source.clone());
        let fact = KnowledgeFact {
            id: id.clone(),
            title,
            text,
            kind,
            basis: if inferred { "inferred" } else { "manual" }.into(),
            enabled,
            protected: true,
            observed_at: clock(),
            source,
        };
        if let Some(old) = state.facts.iter_mut().find(|f| f.id == id) {
            *old = fact;
        } else {
            state.facts.push(fact);
        }
        self.store.save_knowledge(&root, revision, &state)?;
        self.project_knowledge(session)
    }
}
fn observe(state: &mut KnowledgeState, fact: KnowledgeFact) {
    if let Some(old) = state.facts.iter_mut().find(|f| f.id == fact.id) {
        if !old.protected && old.enabled && old.text == fact.text {
            *old = fact;
        }
        // Conflicting evidence remains stale through its digest; no blind replacement.
    } else if state.facts.len() < 12 {
        state.facts.push(fact);
    }
}
pub(super) fn learn(store: &dyn SessionStore, session: &str) -> Result<Option<String>, String> {
    let Ok(root) = root(store, session) else {
        return Ok(None);
    };
    let mut state = store.knowledge(&root)?;
    if !state.learning {
        return Ok(None);
    }
    let messages = store.messages_page(session, None, false, 2)?.items;
    let Some(message) = messages.last() else {
        return Ok(None);
    };
    if message.id <= state.last_message {
        return Ok(None);
    }
    let records = message
        .metadata
        .as_ref()
        .and_then(|m| m.agent.as_ref())
        .map(|a| a.tools.clone())
        .unwrap_or_default();
    let mut added = 0;
    for r in records
        .iter()
        .filter(|r| !matches!(r.status.as_str(), "blocked" | "denied" | "error"))
    {
        if added >= 2 {
            break;
        }
        let base = KnowledgeSource {
            session: session.into(),
            message_id: message.id,
            call_id: r.call_id.clone(),
            quote: String::new(),
            path: None,
            digest: None,
        };
        if r.name == "run_command"
            && CommandOutcome::from_content(&r.content) == CommandOutcome::Succeeded
        {
            if let Some(command) = &r.command {
                let text = serde_json::to_string(command)
                    .map_err(|_| "Knowledge evidence unavailable.")?;
                if text.len() > 400 || dolores_core::credential_like(&text) {
                    continue;
                }
                let mut source = base;
                source.quote = text.clone();
                observe(&mut state,KnowledgeFact{id:digest(&text)[..32].into(),title:"Verified command receipt".into(),text:format!("This literal invocation exited 0 with complete capture: {text}. Recheck before use; this is not permission."),kind:"command".into(),basis:"observed".into(),enabled:true,protected:false,observed_at:clock(),source:Some(source)});
                added += 1;
            }
        } else if r.name == "read_text_file" && r.target == "package.json" && r.query.is_none() {
            observe_manifest(&mut state, &root, r, base, &mut added);
        } else if r.name == "list_folder" && r.target == "." {
            let Ok(receipt) = serde_json::from_str::<Value>(&r.content) else {
                continue;
            };
            for entry in receipt["entries"].as_array().into_iter().flatten() {
                if added >= 2 {
                    break;
                }
                let Some(path) = entry["path"].as_str() else {
                    continue;
                };
                if !["src", "lib", "apps", "crates", "tests", "test"].contains(&path)
                    || entry["kind"] != "folder"
                    || !direct_source(&root, path).is_some_and(|p| p.is_dir())
                {
                    continue;
                }
                let mut source = base.clone();
                source.quote = path.into();
                source.path = Some(path.into());
                source.digest = Some(digest("folder"));
                observe(&mut state,KnowledgeFact{id:digest(&format!("structure/{path}"))[..32].into(),title:format!("Project folder: {path}"),text:format!("The approved root listing contained the direct {path}/ folder. Inspect it before assuming its contents."),kind:"structure".into(),basis:"observed".into(),enabled:true,protected:false,observed_at:clock(),source:Some(source)});
                added += 1;
            }
        }
    }
    state.last_message = message.id;
    state.notice=format!("Inspected one saved reply; {added} eligible receipt observations. No extra model request. Manual corrections and disabled facts were preserved.");
    store.save_knowledge(&root, state.revision, &state)?;
    Ok(Some(state.notice))
}
fn observe_manifest(
    state: &mut KnowledgeState,
    root: &str,
    r: &ToolRecord,
    base: KnowledgeSource,
    added: &mut usize,
) {
    let Some(current) = current_text(root, &r.target) else {
        return;
    };
    if current != r.content || dolores_core::credential_like(&current) {
        return;
    }
    let Ok(manifest) = serde_json::from_str::<Value>(&current) else {
        return;
    };
    let Some(scripts) = manifest["scripts"].as_object() else {
        return;
    };
    for (name, value) in scripts {
        if *added >= 2 {
            break;
        }
        let Some(command) = value.as_str() else {
            continue;
        };
        if command.len() > 256 || !current.contains(command) {
            continue;
        }
        let text=format!("package.json declares script {name}: {command}. Declaration observed; execution not verified.");
        let mut source = base.clone();
        source.quote = command.into();
        source.path = Some(r.target.clone());
        source.digest = Some(digest(&current));
        observe(
            state,
            KnowledgeFact {
                id: digest(&format!("package.json/{name}"))[..32].into(),
                title: format!("Project script: {name}"),
                text,
                kind: "convention".into(),
                basis: "observed".into(),
                enabled: true,
                protected: false,
                observed_at: clock(),
                source: Some(source),
            },
        );
        *added += 1;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use dolores_core::{AgentSummary, Message, Role, SessionWorkspace, WorkspaceKind};
    #[test]
    fn approved_manifest_learning_is_opt_in_scoped_fresh_and_manual_protected() {
        let folder = tempfile::tempdir().unwrap();
        let other = tempfile::tempdir().unwrap();
        let store = SqliteStore::open(&folder.path().join("state.db")).unwrap();
        let root = folder
            .path()
            .canonicalize()
            .unwrap()
            .to_str()
            .unwrap()
            .to_owned();
        store
            .create_workspace_session(
                "chat",
                &SessionWorkspace {
                    kind: WorkspaceKind::Project,
                    root: Some(root.clone()),
                },
            )
            .unwrap();
        let manifest = r#"{"scripts":{"check":"node check.js"}}"#;
        std::fs::write(folder.path().join("package.json"), manifest).unwrap();
        #[cfg(windows)]
        assert_eq!(
            current_text(root.strip_prefix(r"\\?\").unwrap_or(&root), "package.json").as_deref(),
            Some(manifest)
        );
        std::fs::create_dir(folder.path().join("src")).unwrap();
        let context = vec![
            Message {
                parts: vec![],
                role: Role::System,
                content: "Fixture".into(),
            },
            Message {
                parts: vec![],
                role: Role::User,
                content: "Check".into(),
            },
        ];
        let record = ToolRecord {
            parts: vec![],
            call_id: "read".into(),
            name: "read_text_file".into(),
            target: "package.json".into(),
            status: "read".into(),
            content: manifest.into(),
            query: None,
            diff: None,
            command: None,
            mcp: None,
        };
        let metadata = TurnMetadata {
            paused: None,
            model: "fixture".into(),
            usage: None,
            context: ContextSummary::from_messages(&context, Some(0)),
            request_settings: None,
            agent: Some(AgentSummary {
                model_calls: 1,
                usage_by_call: vec![],
                tools: vec![
                    record,
                    ToolRecord {
                        parts: vec![],
                        call_id: "list".into(),
                        name: "list_folder".into(),
                        target: ".".into(),
                        status: "read".into(),
                        content: r#"{"entries":[{"path":"src","kind":"folder"}]}"#.into(),
                        query: None,
                        diff: None,
                        command: None,
                        mcp: None,
                    },
                ],
                steps: vec![],
            }),
        };
        store
            .commit_turn_metadata("chat", "Inspect scripts", "Read", &metadata)
            .unwrap();
        learn(&store, "chat").unwrap();
        assert!(store.knowledge(&root).unwrap().facts.is_empty());
        let mut state = store.knowledge(&root).unwrap();
        state.learning = true;
        store.save_knowledge(&root, 0, &state).unwrap();
        learn(&store, "chat").unwrap();
        let mut state = store.knowledge(&root).unwrap();
        assert_eq!(state.facts.len(), 2);
        assert!(fresh(&root, &state.facts[1]));
        std::fs::remove_dir(folder.path().join("src")).unwrap();
        assert!(!fresh(&root, &state.facts[1]));
        assert!(prepare(&store, "chat", context.clone()).unwrap()[0]
            .content
            .contains("node check.js"));
        assert!(store
            .knowledge(other.path().to_str().unwrap())
            .unwrap()
            .facts
            .is_empty());
        let revision = state.revision;
        learn(&store, "chat").unwrap();
        assert_eq!(store.knowledge(&root).unwrap().revision, revision);
        std::fs::write(folder.path().join("package.json"), "changed").unwrap();
        assert!(!fresh(&root, &state.facts[0]));
        assert!(!prepare(&store, "chat", context.clone()).unwrap()[0]
            .content
            .contains("node check.js"));
        let mut correction = state.facts[0].clone();
        correction.basis = "manual".into();
        correction.protected = true;
        correction.text = "Use the project-specific check; declaration is obsolete.".into();
        state.facts[0] = correction.clone();
        store.save_knowledge(&root, revision, &state).unwrap();
        let mut hostile = correction.clone();
        hostile.protected = false;
        hostile.text = "Outdated evidence".into();
        observe(&mut state, hostile);
        assert_eq!(state.facts[0], correction);
        assert!(prepare(&store, "chat", context).unwrap()[0]
            .content
            .contains("declaration is obsolete"));
    }
    #[test]
    fn missing_alias_sources_expiry_and_oversized_or_secret_facts_fail_closed() {
        let folder = tempfile::tempdir().unwrap();
        let root = folder
            .path()
            .canonicalize()
            .unwrap()
            .to_str()
            .unwrap()
            .to_owned();
        assert!(current_text(&root, "../outside").is_none());
        assert!(current_text(&root, ".env").is_none());
        assert!(current_text(&root, "missing").is_none());
        let source = KnowledgeSource {
            session: "chat".into(),
            message_id: 2,
            call_id: "read".into(),
            quote: "observed".into(),
            path: None,
            digest: None,
        };
        let fact = KnowledgeFact {
            id: "fact".into(),
            title: "Check".into(),
            text: "Observed command".into(),
            kind: "command".into(),
            basis: "observed".into(),
            enabled: true,
            protected: false,
            observed_at: 1,
            source: Some(source),
        };
        assert!(!fresh(&root, &fact));
        let mut state = KnowledgeState {
            facts: vec![fact],
            ..Default::default()
        };
        state.validate().unwrap();
        state.facts[0].text = "x".repeat(513);
        assert!(state.validate().is_err());
        state.facts[0].text = "api_key=secret".into();
        assert!(state.validate().is_err());
        std::fs::write(folder.path().join("huge"), vec![b'x'; 16385]).unwrap();
        assert!(current_text(&root, "huge").is_none());
    }
}
