use super::Engine;
use dolores_core::{ProjectSkill, SessionStore, SkillDocument, SkillScope};
use serde_json::{json, Value};
use std::{
    collections::BTreeSet,
    path::{Path, PathBuf},
    time::{Duration, Instant},
};

pub(super) struct SkillReview {
    token: String,
    session: String,
    root: String,
    scope: SkillScope,
    target: Option<PathBuf>,
    document: SkillDocument,
    saved: Option<ProjectSkill>,
    rollback: Option<u32>,
    created: Instant,
}
pub(super) fn for_session(
    store: &dyn SessionStore,
    session: Option<&str>,
) -> Result<Vec<ProjectSkill>, String> {
    let root = session
        .map(|s| store.workspace(s))
        .transpose()?
        .and_then(|w| w.root);
    let mut skills = root.map_or(Ok(vec![]), |root| store.project_skills(&root))?;
    skills.extend(store.global_skills()?);
    Ok(skills)
}
fn read_document(root: &str, name: &str, scope: SkillScope) -> Result<SkillDocument, String> {
    match scope {
        SkillScope::Project => dolores_tools_fs::read_project_skill(Path::new(root), name),
        SkillScope::Global => dolores_tools_fs::read_global_skill(Path::new(root), name),
    }
}
impl Engine {
    fn skill_root(&self, session: &str, scope: SkillScope) -> Result<String, String> {
        let workspace = self.store.workspace(session)?;
        match scope {
            SkillScope::Project => workspace
                .root
                .ok_or("Side chats do not use project skills.".into()),
            SkillScope::Global => self
                .global_skills_directory
                .as_ref()
                .and_then(|p| p.to_str())
                .map(str::to_owned)
                .ok_or("Global skills directory is unavailable.".into()),
        }
    }
    fn saved_skills(&self, root: &str, scope: SkillScope) -> Result<Vec<ProjectSkill>, String> {
        match scope {
            SkillScope::Project => self.store.project_skills(root),
            SkillScope::Global => self.store.global_skills(),
        }
    }
    #[cfg(test)]
    fn project_skills(&self, session: &str) -> Result<Value, String> {
        self.scoped_skills(session, SkillScope::Project)
    }
    pub(super) fn scoped_skills(&self, session: &str, scope: SkillScope) -> Result<Value, String> {
        self.skill_review
            .lock()
            .map_err(|_| "Skills are unavailable.")?
            .take();
        let root = self.skill_root(session, scope)?;
        let saved = self.saved_skills(&root, scope)?;
        let catalog = match scope {
            SkillScope::Project => dolores_tools_fs::project_skill_catalog(Path::new(&root)),
            SkillScope::Global => dolores_tools_fs::global_skill_catalog(Path::new(&root)),
        };
        let project_names: BTreeSet<_> = if scope == SkillScope::Global {
            for_session(self.store.as_ref(), Some(session))?
                .into_iter()
                .filter(|s| s.scope == SkillScope::Project && s.enabled)
                .map(|s| s.name)
                .collect()
        } else {
            BTreeSet::new()
        };
        let mut names: BTreeSet<_> = saved.iter().map(|s| s.name.clone()).collect();
        if let Ok(catalog) = &catalog {
            names.extend(catalog.names.iter().cloned());
        }
        let items:Vec<_> = names.into_iter().map(|name| {
            let record = saved.iter().find(|s| s.name == name);
            json!({"name":name,"scope":scope,"generated":record.is_some_and(|s| s.current().evaluation.is_some()),"overridden":project_names.contains(&name),"enabled":record.is_some_and(|s| s.enabled),"revision":record.map(|s| s.revision),"version":record.map(|s| s.current().version),"description":record.map(|s| &s.current().document.description),"sourceAvailable":catalog.as_ref().is_ok_and(|c| c.sources.contains(&name))})
        }).collect();
        Ok(
            json!({"items":items,"scope":scope,"directory":root,"partial":catalog.as_ref().is_ok_and(|c| c.partial),"problem":catalog.err()}),
        )
    }
    #[cfg(test)]
    fn review_skill(
        &self,
        session: &str,
        name: &str,
        version: Option<u32>,
    ) -> Result<Value, String> {
        self.review_scoped_skill(session, name, version, SkillScope::Project)
    }
    pub(super) fn review_scoped_skill(
        &self,
        session: &str,
        name: &str,
        version: Option<u32>,
        scope: SkillScope,
    ) -> Result<Value, String> {
        self.skill_review
            .lock()
            .map_err(|_| "Skills are unavailable.")?
            .take();
        let root = self.skill_root(session, scope)?;
        let saved = self
            .saved_skills(&root, scope)?
            .into_iter()
            .find(|s| s.name == name);
        let target = if scope == SkillScope::Global {
            dolores_tools_fs::global_skills_target(Path::new(&root)).ok()
        } else {
            None
        };
        let current = read_document(&root, name, scope);
        if scope == SkillScope::Global
            && current.is_ok()
            && dolores_tools_fs::global_skills_target(Path::new(&root)).ok() != target
        {
            return Err("Global skills directory changed; review it again.".into());
        }
        let document = if let Some(version) = version {
            saved
                .as_ref()
                .and_then(|s| s.versions.iter().find(|v| v.version == version))
                .map(|v| v.document.clone())
                .ok_or("Skill version is no longer retained. Refresh Skills.")?
        } else {
            current.clone()?
        };
        let token = uuid::Uuid::new_v4().to_string();
        let evaluation = version
            .and_then(|number| {
                saved
                    .as_ref()?
                    .versions
                    .iter()
                    .find(|v| v.version == number)
            })
            .and_then(|v| v.evaluation.as_ref());
        let versions:Vec<_> = saved.iter().flat_map(|s| &s.versions).map(|v| json!({"version":v.version,"reviewedAt":v.reviewed_at,"rollbackFrom":v.rollback_from})).collect();
        let generated = evaluation.is_some();
        let response = json!({"scope":scope,"token":token,"document":document,"evaluation":evaluation,"generated":generated,"enabled":saved.as_ref().is_some_and(|s| s.enabled),"revision":saved.as_ref().map(|s| s.revision),"activeVersion":saved.as_ref().map(|s| s.current().version),"versions":versions,"reviewVersion":version,"sourceMatches":current.as_ref().ok()==Some(&document),"alreadyActive":saved.as_ref().is_some_and(|s| s.enabled && s.current().document==document),"problem":if generated {None} else {current.err()}});
        *self
            .skill_review
            .lock()
            .map_err(|_| "Skills are unavailable.")? = Some(SkillReview {
            token,
            session: session.into(),
            root,
            scope,
            target,
            document,
            saved,
            rollback: version,
            created: Instant::now(),
        });
        Ok(response)
    }
    pub(super) fn activate_skill(&self, session: &str, token: &str) -> Result<Value, String> {
        let review = self
            .skill_review
            .lock()
            .map_err(|_| "Skills are unavailable.")?
            .take()
            .ok_or("Skill review expired. Refresh Skills and review it again.")?;
        if review.token != token
            || review.session != session
            || review.root != self.skill_root(session, review.scope)?
            || review.created.elapsed() > Duration::from_secs(300)
        {
            return Err("Skill review expired. Refresh Skills and review it again.".into());
        }
        let saved = self
            .saved_skills(&review.root, review.scope)?
            .into_iter()
            .find(|s| s.name == review.document.name);
        if saved != review.saved {
            return Err("Skill changed after review. Refresh Skills.".into());
        }
        if review.rollback.is_none()
            && (read_document(&review.root, &review.document.name, review.scope)?
                != review.document
                || (review.scope == SkillScope::Global
                    && Some(dolores_tools_fs::global_skills_target(Path::new(
                        &review.root,
                    ))?) != review.target))
        {
            return Err(
                "SKILL.md changed after review. Refresh Skills and review it again.".into(),
            );
        }
        let revision = review.saved.as_ref().map(|s| s.revision);
        let value = match review.scope {
            SkillScope::Project => self.store.activate_project_skill(
                &review.root,
                &review.document,
                revision,
                review.rollback,
            ),
            SkillScope::Global => {
                self.store
                    .activate_global_skill(&review.document, revision, review.rollback)
            }
        }?;
        Ok(json!(value))
    }
    #[cfg(test)]
    fn mutate_skill(
        &self,
        session: &str,
        name: &str,
        revision: u32,
        forget: bool,
    ) -> Result<Value, String> {
        self.mutate_scoped_skill(session, name, revision, forget, SkillScope::Project)
    }
    pub(super) fn mutate_scoped_skill(
        &self,
        session: &str,
        name: &str,
        revision: u32,
        forget: bool,
        scope: SkillScope,
    ) -> Result<Value, String> {
        let root = self.skill_root(session, scope)?;
        self.skill_review
            .lock()
            .map_err(|_| "Skills are unavailable.")?
            .take();
        match (scope, forget) {
            (SkillScope::Project, true) => self.store.forget_project_skill(&root, name, revision),
            (SkillScope::Project, false) => self.store.disable_project_skill(&root, name, revision),
            (SkillScope::Global, true) => self.store.forget_global_skill(name, revision),
            (SkillScope::Global, false) => self.store.disable_global_skill(name, revision),
        }?;
        Ok(Value::Null)
    }
    pub(super) fn cancel_skill_review(&self, token: &str) -> Result<Value, String> {
        let mut slot = self
            .skill_review
            .lock()
            .map_err(|_| "Skills are unavailable.")?;
        if slot.as_ref().is_some_and(|r| r.token == token) {
            slot.take();
        }
        Ok(Value::Null)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use dolores_core::{SessionWorkspace, WorkspaceKind};
    use dolores_store_sqlite::SqliteStore;
    use std::sync::Arc;
    #[cfg(windows)]
    #[test]
    fn retargeting_the_global_boundary_requires_another_review_even_with_identical_text() {
        use std::os::windows::process::CommandExt;
        let temp = tempfile::tempdir().unwrap();
        let store = Arc::new(SqliteStore::open(&temp.path().join("state.db")).unwrap());
        store.create("side").unwrap();
        let mut engine = Engine::new(
            store.clone(),
            Arc::new(crate::connection::testing::MemoryCredentials::default()),
        )
        .unwrap();
        let boundary = temp.path().join("skills");
        for name in ["first", "second"] {
            let directory = temp.path().join(name).join("review");
            std::fs::create_dir_all(&directory).unwrap();
            std::fs::write(
                directory.join("SKILL.md"),
                "---\nname: review\ndescription: Review\n---\nSame text",
            )
            .unwrap();
        }
        let link = |target: &Path| {
            assert!(std::process::Command::new("cmd")
                .creation_flags(0x08000000)
                .args(["/c", "mklink", "/J"])
                .arg(&boundary)
                .arg(target)
                .output()
                .unwrap()
                .status
                .success());
        };
        link(&temp.path().join("first"));
        engine.global_skills_directory = Some(boundary.clone());
        let preview = engine
            .review_scoped_skill("side", "review", None, SkillScope::Global)
            .unwrap();
        // Remove only this known isolated junction, never its target contents.
        std::fs::remove_dir(&boundary).unwrap();
        link(&temp.path().join("second"));
        assert!(engine
            .activate_skill("side", preview["token"].as_str().unwrap())
            .is_err());
        assert!(store.global_skills().unwrap().is_empty());
        let preview = engine
            .review_scoped_skill("side", "review", None, SkillScope::Global)
            .unwrap();
        engine
            .activate_skill("side", preview["token"].as_str().unwrap())
            .unwrap();
    }
    #[test]
    fn global_review_is_explicit_available_in_side_chats_and_scoped() {
        let temp = tempfile::tempdir().unwrap();
        let root = crate::workspace::canonical_folder(temp.path()).unwrap();
        let store = Arc::new(SqliteStore::open(&temp.path().join("state.db")).unwrap());
        store.create("side").unwrap();
        store
            .create_workspace_session(
                "project",
                &SessionWorkspace {
                    kind: WorkspaceKind::Project,
                    root: Some(root.clone()),
                },
            )
            .unwrap();
        let mut engine = Engine::new(
            store.clone(),
            Arc::new(crate::connection::testing::MemoryCredentials::default()),
        )
        .unwrap();
        let directory = temp.path().join("global");
        engine.global_skills_directory = Some(directory.clone());
        std::fs::create_dir_all(directory.join("review")).unwrap();
        let file = directory.join("review/SKILL.md");
        let text = "---\nname: review\ndescription: Review\n---\nGLOBAL_ONLY";
        std::fs::write(&file, text).unwrap();
        assert_eq!(
            engine.scoped_skills("side", SkillScope::Global).unwrap()["items"][0]["enabled"],
            false
        );
        assert!(for_session(store.as_ref(), Some("side"))
            .unwrap()
            .is_empty());
        let preview = engine
            .review_scoped_skill("side", "review", None, SkillScope::Global)
            .unwrap();
        assert_eq!(preview["document"]["text"], text);
        let result = engine
            .activate_skill("side", preview["token"].as_str().unwrap())
            .unwrap();
        assert_eq!(result["scope"], "global");
        assert_eq!(
            for_session(store.as_ref(), Some("project")).unwrap().len(),
            1
        );
        assert_eq!(for_session(store.as_ref(), Some("side")).unwrap().len(), 1);
        assert_eq!(for_session(store.as_ref(), None).unwrap().len(), 1);
        let doc = SkillDocument {
            name: "review".into(),
            description: "Review".into(),
            text: "PROJECT_ONLY".into(),
        };
        store
            .activate_project_skill(&root, &doc, None, None)
            .unwrap();
        assert_eq!(
            engine.scoped_skills("project", SkillScope::Global).unwrap()["items"][0]["overridden"],
            true
        );
        let context = engine
            .call(crate::Command::Context {
                session: Some("side".into()),
                input: "draft".into(),
                tools: false,
            })
            .unwrap();
        assert_eq!(context["skills"][0]["scope"], "global");
        assert_eq!(context["skillEntries"][0]["scope"], "global");
        assert!(context["tools"].as_array().unwrap().is_empty());
        assert!(!context["messages"][0]["content"]
            .as_str()
            .unwrap()
            .contains(&root));
        std::fs::remove_file(file).unwrap();
        let preview = engine
            .review_scoped_skill("side", "review", Some(1), SkillScope::Global)
            .unwrap();
        engine
            .mutate_scoped_skill("side", "review", 1, false, SkillScope::Global)
            .unwrap();
        assert!(engine
            .activate_skill("side", preview["token"].as_str().unwrap())
            .is_err());
        assert!(store.project_skills(&root).unwrap()[0].enabled);
        let preview = engine
            .review_scoped_skill("side", "review", Some(1), SkillScope::Global)
            .unwrap();
        engine
            .activate_skill("side", preview["token"].as_str().unwrap())
            .unwrap();
        engine
            .mutate_scoped_skill("side", "review", 3, true, SkillScope::Global)
            .unwrap();
        assert!(store.global_skills().unwrap().is_empty());
    }
    #[test]
    fn reviews_bind_source_session_expiry_saved_state_and_never_auto_activate() {
        let temp = tempfile::tempdir().unwrap();
        let root = crate::workspace::canonical_folder(temp.path()).unwrap();
        let store = Arc::new(SqliteStore::open(&temp.path().join("state.db")).unwrap());
        for id in ["first", "second"] {
            store
                .create_workspace_session(
                    id,
                    &SessionWorkspace {
                        kind: WorkspaceKind::Project,
                        root: Some(root.clone()),
                    },
                )
                .unwrap();
        }
        store.create("side").unwrap();
        let engine = Engine::new(
            store.clone(),
            Arc::new(crate::connection::testing::MemoryCredentials::default()),
        )
        .unwrap();
        let path = temp.path().join(".agents/skills/review");
        std::fs::create_dir_all(&path).unwrap();
        let file = path.join("SKILL.md");
        std::fs::write(
            &file,
            "---\nname: review\ndescription: Code review\n---\nCheck focused tests.",
        )
        .unwrap();
        assert_eq!(
            engine.project_skills("first").unwrap()["items"][0]["enabled"],
            false
        );
        assert!(for_session(store.as_ref(), Some("first"))
            .unwrap()
            .is_empty());
        let preview = engine.review_skill("first", "review", None).unwrap();
        assert!(engine
            .activate_skill("second", preview["token"].as_str().unwrap())
            .is_err());
        assert!(engine
            .activate_skill("first", preview["token"].as_str().unwrap())
            .is_err());
        let preview = engine.review_skill("first", "review", None).unwrap();
        std::fs::write(
            &file,
            "---\nname: review\ndescription: Code review\n---\nChanged.",
        )
        .unwrap();
        assert!(engine
            .activate_skill("first", preview["token"].as_str().unwrap())
            .is_err());
        let preview = engine.review_skill("first", "review", None).unwrap();
        engine
            .skill_review
            .lock()
            .unwrap()
            .as_mut()
            .unwrap()
            .created = Instant::now() - Duration::from_secs(301);
        assert!(engine
            .activate_skill("first", preview["token"].as_str().unwrap())
            .is_err());
        let preview = engine.review_skill("first", "review", None).unwrap();
        engine
            .activate_skill("first", preview["token"].as_str().unwrap())
            .unwrap();
        std::fs::remove_file(&file).unwrap();
        assert!(for_session(store.as_ref(), Some("second")).unwrap()[0].enabled);
        let preview = engine.review_skill("first", "review", Some(1)).unwrap();
        engine.mutate_skill("second", "review", 1, false).unwrap();
        assert!(engine
            .activate_skill("first", preview["token"].as_str().unwrap())
            .is_err());
        assert!(engine.project_skills("side").is_err());
        assert!(for_session(store.as_ref(), Some("side"))
            .unwrap()
            .is_empty());
        let preview = engine.review_skill("first", "review", Some(1)).unwrap();
        engine
            .activate_skill("first", preview["token"].as_str().unwrap())
            .unwrap();
        assert_eq!(
            store.project_skills(&root).unwrap()[0]
                .current()
                .rollback_from,
            Some(1)
        );
    }
}
