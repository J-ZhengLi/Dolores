use super::{now, storage_error, SqliteStore};
use dolores_core::{
    valid_skill_name, ProjectSkill, SkillDocument, SkillScope, SkillVersion, MAX_ACTIVE_SKILLS,
    MAX_SAVED_SKILLS, MAX_SKILL_BYTES, MAX_SKILL_VERSIONS,
};
use rusqlite::{params, Connection};

// Disjoint from all absolute project roots. Global activation is local to this app's data directory.
pub(super) const GLOBAL_ROOT: &str = "@global-skills";

fn read(connection: &Connection, root: &str) -> Result<Vec<ProjectSkill>, String> {
    let mut query = connection
        .prepare("SELECT name,data FROM project_skills WHERE root=?1 ORDER BY name LIMIT 13")
        .map_err(storage_error)?;
    let values = query
        .query_map([root], |r| {
            Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?))
        })
        .map_err(storage_error)?;
    let mut result = vec![];
    for value in values {
        let (name, data) = value.map_err(storage_error)?;
        let skill: ProjectSkill = serde_json::from_str(&data)
            .map_err(|_| "Saved skills could not be read. Review Skills.")?;
        skill.validate()?;
        if name != skill.name
            || skill.scope
                != if root == GLOBAL_ROOT {
                    SkillScope::Global
                } else {
                    SkillScope::Project
                }
        {
            return Err("Saved skills have mismatched identities.".into());
        }
        result.push(skill);
    }
    if result.len() > MAX_SAVED_SKILLS {
        return Err("Saved skills exceed the limit for their scope.".into());
    }
    Ok(result)
}
fn validate_root(root: &str) -> Result<(), String> {
    if root != GLOBAL_ROOT && !std::path::Path::new(root).is_absolute() {
        return Err("Project skills need an absolute working folder.".into());
    }
    Ok(())
}
fn changed() -> String {
    "Skill changed after review. Refresh Skills and review it again.".into()
}
impl SqliteStore {
    pub(super) fn read_skills(&self, root: &str) -> Result<Vec<ProjectSkill>, String> {
        validate_root(root)?;
        let guard = self.lock()?;
        read(&guard, root)
    }
    pub(super) fn activate_skill(
        &self,
        root: &str,
        document: &SkillDocument,
        revision: Option<u32>,
        rollback: Option<u32>,
    ) -> Result<ProjectSkill, String> {
        validate_root(root)?;
        document.validate()?;
        let mut guard = self.lock()?;
        let transaction = guard.transaction().map_err(storage_error)?;
        let skills = read(&transaction, root)?;
        let previous = skills.iter().find(|s| s.name == document.name);
        if previous.map(|s| s.revision) != revision {
            return Err(changed());
        }
        if let Some(version) = rollback {
            if previous
                .and_then(|s| s.versions.iter().find(|v| v.version == version))
                .map(|v| &v.document)
                != Some(document)
            {
                return Err(changed());
            }
        }
        if previous.is_some_and(|s| s.enabled && &s.current().document == document) {
            return Err("This exact skill version is already active.".into());
        }
        if previous.is_none() && skills.len() >= MAX_SAVED_SKILLS {
            return Err(
                "Skills allow 12 saved entries per scope. Forget an unused entry first.".into(),
            );
        }
        let others: Vec<_> = skills
            .iter()
            .filter(|s| s.enabled && s.name != document.name)
            .collect();
        if others.len() >= MAX_ACTIVE_SKILLS
            || others
                .iter()
                .map(|s| s.current().document.text.len())
                .sum::<usize>()
                + document.text.len()
                > MAX_SKILL_BYTES
        {
            return Err("Skills allow three active entries and 8 KiB combined per scope. Disable or shorten a skill first.".into());
        }
        let mut versions = previous.map(|s| s.versions.clone()).unwrap_or_default();
        let version = versions
            .last()
            .map_or(Some(1), |v| v.version.checked_add(1))
            .ok_or_else(changed)?;
        versions.push(SkillVersion {
            version,
            reviewed_at: now(),
            document: document.clone(),
            rollback_from: rollback,
        });
        if versions.len() > MAX_SKILL_VERSIONS {
            versions.remove(0);
        }
        let value = ProjectSkill {
            scope: if root == GLOBAL_ROOT {
                SkillScope::Global
            } else {
                SkillScope::Project
            },
            name: document.name.clone(),
            revision: previous
                .map_or(Some(1), |s| s.revision.checked_add(1))
                .ok_or_else(changed)?,
            enabled: true,
            versions,
        };
        value.validate()?;
        transaction.execute("INSERT INTO project_skills(root,name,data) VALUES(?1,?2,?3) ON CONFLICT(root,name) DO UPDATE SET data=excluded.data", params![root, value.name, serde_json::to_string(&value).map_err(storage_error)?]).map_err(storage_error)?;
        transaction.commit().map_err(storage_error)?;
        Ok(value)
    }
    pub(super) fn mutate_skill(
        &self,
        root: &str,
        name: &str,
        revision: u32,
        forget: bool,
    ) -> Result<(), String> {
        validate_root(root)?;
        if !valid_skill_name(name) {
            return Err(changed());
        }
        let mut guard = self.lock()?;
        let transaction = guard.transaction().map_err(storage_error)?;
        let mut value = read(&transaction, root)?
            .into_iter()
            .find(|s| s.name == name && s.revision == revision)
            .ok_or_else(changed)?;
        if forget {
            transaction
                .execute(
                    "DELETE FROM project_skills WHERE root=?1 AND name=?2",
                    params![root, name],
                )
                .map_err(storage_error)?;
        } else {
            if !value.enabled {
                return Err("Skill is already disabled. Refresh Skills.".into());
            }
            value.enabled = false;
            value.revision = value.revision.checked_add(1).ok_or_else(changed)?;
            transaction
                .execute(
                    "UPDATE project_skills SET data=?3 WHERE root=?1 AND name=?2",
                    params![
                        root,
                        name,
                        serde_json::to_string(&value).map_err(storage_error)?
                    ],
                )
                .map_err(storage_error)?;
        }
        transaction.commit().map_err(storage_error)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn global_records_are_independent_persistent_and_use_the_same_version_guards() {
        use dolores_core::SessionStore;
        let temp = tempfile::tempdir().unwrap();
        let file = temp.path().join("state.db");
        let root = temp.path().to_str().unwrap();
        let doc = SkillDocument {
            name: "review".into(),
            description: "Review".into(),
            text: "Global text".into(),
        };
        {
            let store = SqliteStore::open(&file).unwrap();
            let global = store.activate_global_skill(&doc, None, None).unwrap();
            assert_eq!(global.scope, SkillScope::Global);
            assert!(store.activate_global_skill(&doc, None, None).is_err());
            let project = store
                .activate_project_skill(root, &doc, None, None)
                .unwrap();
            assert_eq!(project.scope, SkillScope::Project);
            assert!(store.project_skills(GLOBAL_ROOT).is_err());
            assert!(store
                .activate_project_skill(GLOBAL_ROOT, &doc, Some(1), None)
                .is_err());
            assert!(store
                .disable_project_skill(GLOBAL_ROOT, "review", 1)
                .is_err());
            assert!(store
                .forget_project_skill(GLOBAL_ROOT, "review", 1)
                .is_err());
            store.disable_global_skill("review", 1).unwrap();
            assert!(store.project_skills(root).unwrap()[0].enabled);
            assert!(store.forget_global_skill("review", 1).is_err());
        }
        let store = SqliteStore::open(&file).unwrap();
        let saved = store.global_skills().unwrap().remove(0);
        assert_eq!(saved.revision, 2);
        assert!(!saved.enabled);
        let restored = store.activate_global_skill(&doc, Some(2), Some(1)).unwrap();
        assert_eq!(restored.current().rollback_from, Some(1));
        assert_eq!(restored.current().version, 2);
        store.forget_global_skill("review", 3).unwrap();
        assert!(store.global_skills().unwrap().is_empty());
        assert_eq!(store.project_skills(root).unwrap().len(), 1);
    }
    use dolores_core::SessionStore;
    fn doc(name: &str, text: &str) -> SkillDocument {
        SkillDocument {
            name: name.into(),
            description: "Synthetic task".into(),
            text: text.into(),
        }
    }
    #[test]
    fn versions_cas_rollback_scope_restart_and_failed_write_preserve_state() {
        let temp = tempfile::tempdir().unwrap();
        let db = temp.path().join("state.db");
        let root = temp.path().to_str().unwrap();
        {
            let store = SqliteStore::open(&db).unwrap();
            store.create("chat").unwrap();
            let first = store
                .activate_project_skill(root, &doc("review", "first"), None, None)
                .unwrap();
            assert!(store
                .activate_project_skill(root, &doc("review", "stale"), None, None)
                .is_err());
            let second = store
                .activate_project_skill(root, &doc("review", "second"), Some(first.revision), None)
                .unwrap();
            assert!(store
                .disable_project_skill(root, "review", first.revision)
                .is_err());
            let third = store
                .activate_project_skill(
                    root,
                    &first.current().document,
                    Some(second.revision),
                    Some(1),
                )
                .unwrap();
            assert_eq!(third.current().document.text, "first");
            assert_eq!(third.current().rollback_from, Some(1));
            store.lock().unwrap().execute_batch("CREATE TRIGGER refuse_skill BEFORE UPDATE ON project_skills BEGIN SELECT RAISE(ABORT,'fixture'); END;").unwrap();
            assert!(store
                .activate_project_skill(root, &doc("review", "failed"), Some(third.revision), None)
                .is_err());
            assert_eq!(store.project_skills(root).unwrap(), vec![third]);
            assert!(store
                .project_skills(&format!("{root}/other"))
                .unwrap()
                .is_empty());
        }
        let store = SqliteStore::open(&db).unwrap();
        store
            .lock()
            .unwrap()
            .execute_batch("DROP TRIGGER refuse_skill")
            .unwrap();
        store.delete("chat").unwrap();
        assert_eq!(store.project_skills(root).unwrap()[0].current().version, 3);
        store.disable_project_skill(root, "review", 3).unwrap();
        assert!(!store.project_skills(root).unwrap()[0].enabled);
        for rev in 4..11 {
            store
                .activate_project_skill(
                    root,
                    &doc("review", &format!("version {rev}")),
                    Some(rev),
                    None,
                )
                .unwrap();
        }
        let value = store.project_skills(root).unwrap().remove(0);
        assert_eq!(value.versions.len(), 5);
        assert_eq!(value.current().version, 10);
        assert!(store
            .activate_project_skill(root, &doc("review", "first"), Some(11), Some(1))
            .is_err());
        store.forget_project_skill(root, "review", 11).unwrap();
        assert!(store.project_skills(root).unwrap().is_empty());
    }
    #[test]
    fn active_count_byte_and_saved_limits_refuse_without_partial_changes() {
        let temp = tempfile::tempdir().unwrap();
        let store = SqliteStore::open(&temp.path().join("state.db")).unwrap();
        let root = temp.path().to_str().unwrap();
        for name in ["a", "b", "c"] {
            store
                .activate_project_skill(root, &doc(name, "short"), None, None)
                .unwrap();
        }
        assert!(store
            .activate_project_skill(root, &doc("d", "short"), None, None)
            .is_err());
        store.disable_project_skill(root, "c", 1).unwrap();
        assert!(store
            .activate_project_skill(root, &doc("d", &"x".repeat(8192)), None, None)
            .is_err());
        for name in ["d", "e", "f", "g", "h", "i", "j", "k", "l"] {
            store
                .activate_project_skill(root, &doc(name, "short"), None, None)
                .unwrap();
            store.disable_project_skill(root, name, 1).unwrap();
        }
        assert!(store
            .activate_project_skill(root, &doc("m", "short"), None, None)
            .is_err());
        assert_eq!(store.project_skills(root).unwrap().len(), 12);
    }
}
