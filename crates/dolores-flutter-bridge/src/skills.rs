use super::Engine;
use dolores_core::{ProjectSkill, SessionStore, SkillDocument};
use serde_json::{json, Value};
use std::{
    collections::BTreeSet,
    path::Path,
    time::{Duration, Instant},
};

pub(super) struct SkillReview {
    token: String,
    session: String,
    root: String,
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
    root.map_or(Ok(vec![]), |root| store.project_skills(&root))
}
impl Engine {
    fn skill_root(&self, session: &str) -> Result<String, String> {
        self.store
            .workspace(session)?
            .root
            .ok_or("Side chats do not use project skills.".into())
    }
    pub(super) fn project_skills(&self, session: &str) -> Result<Value, String> {
        self.skill_review
            .lock()
            .map_err(|_| "Skills are unavailable.")?
            .take();
        let root = self.skill_root(session)?;
        let saved = self.store.project_skills(&root)?;
        let catalog = dolores_tools_fs::project_skill_catalog(Path::new(&root));
        let mut names: BTreeSet<_> = saved.iter().map(|s| s.name.clone()).collect();
        if let Ok(catalog) = &catalog {
            names.extend(catalog.names.iter().cloned());
        }
        let items:Vec<_> = names.into_iter().map(|name| {
            let record = saved.iter().find(|s| s.name == name);
            json!({"name":name,"enabled":record.is_some_and(|s| s.enabled),"revision":record.map(|s| s.revision),"version":record.map(|s| s.current().version),"description":record.map(|s| &s.current().document.description),"sourceAvailable":catalog.as_ref().is_ok_and(|c| c.sources.contains(&name))})
        }).collect();
        Ok(
            json!({"items":items,"partial":catalog.as_ref().is_ok_and(|c| c.partial),"problem":catalog.err()}),
        )
    }
    pub(super) fn review_skill(
        &self,
        session: &str,
        name: &str,
        version: Option<u32>,
    ) -> Result<Value, String> {
        self.skill_review
            .lock()
            .map_err(|_| "Skills are unavailable.")?
            .take();
        let root = self.skill_root(session)?;
        let saved = self
            .store
            .project_skills(&root)?
            .into_iter()
            .find(|s| s.name == name);
        let current = dolores_tools_fs::read_project_skill(Path::new(&root), name);
        let document = if let Some(version) = version {
            saved
                .as_ref()
                .and_then(|s| s.versions.iter().find(|v| v.version == version))
                .map(|v| v.document.clone())
                .ok_or("Project skill version is no longer retained. Refresh Skills.")?
        } else {
            current.clone()?
        };
        let token = uuid::Uuid::new_v4().to_string();
        let versions:Vec<_> = saved.iter().flat_map(|s| &s.versions).map(|v| json!({"version":v.version,"reviewedAt":v.reviewed_at,"rollbackFrom":v.rollback_from})).collect();
        let response = json!({"token":token,"document":document,"enabled":saved.as_ref().is_some_and(|s| s.enabled),"revision":saved.as_ref().map(|s| s.revision),"activeVersion":saved.as_ref().map(|s| s.current().version),"versions":versions,"reviewVersion":version,"sourceMatches":current.as_ref().ok()==Some(&document),"alreadyActive":saved.as_ref().is_some_and(|s| s.enabled && s.current().document==document),"problem":current.err()});
        *self
            .skill_review
            .lock()
            .map_err(|_| "Skills are unavailable.")? = Some(SkillReview {
            token,
            session: session.into(),
            root,
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
            || review.root != self.skill_root(session)?
            || review.created.elapsed() > Duration::from_secs(300)
        {
            return Err("Skill review expired. Refresh Skills and review it again.".into());
        }
        let saved = self
            .store
            .project_skills(&review.root)?
            .into_iter()
            .find(|s| s.name == review.document.name);
        if saved != review.saved {
            return Err("Project skill changed after review. Refresh Skills.".into());
        }
        if review.rollback.is_none()
            && dolores_tools_fs::read_project_skill(Path::new(&review.root), &review.document.name)?
                != review.document
        {
            return Err(
                "SKILL.md changed after review. Refresh Skills and review it again.".into(),
            );
        }
        let value = self.store.activate_project_skill(
            &review.root,
            &review.document,
            review.saved.as_ref().map(|s| s.revision),
            review.rollback,
        )?;
        Ok(json!(value))
    }
    pub(super) fn mutate_skill(
        &self,
        session: &str,
        name: &str,
        revision: u32,
        forget: bool,
    ) -> Result<Value, String> {
        let root = self.skill_root(session)?;
        self.skill_review
            .lock()
            .map_err(|_| "Skills are unavailable.")?
            .take();
        if forget {
            self.store.forget_project_skill(&root, name, revision)?;
        } else {
            self.store.disable_project_skill(&root, name, revision)?;
        }
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
