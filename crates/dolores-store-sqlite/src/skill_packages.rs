use super::*;
use dolores_core::skill_packages::{valid_digest, Installed, Snapshot};
use rusqlite::TransactionBehavior;

pub(super) fn root_valid(root: &str) -> Result<(), String> {
    if root != skills::GLOBAL_ROOT && !Path::new(root).is_absolute() {
        return Err("Package scope needs an absolute project folder or Global.".into());
    }
    Ok(())
}
pub(super) fn read_record(
    c: &Connection,
    root: &str,
    name: &str,
    id: &str,
) -> Result<Option<Installed>, String> {
    let raw: Option<String> = c
        .query_row(
            "SELECT data FROM skill_packages WHERE root=?1 AND name=?2 AND id=?3",
            params![root, name, id],
            |r| r.get(0),
        )
        .optional()
        .map_err(storage_error)?;
    raw.map(|raw| {
        let record: Installed = serde_json::from_str(&raw).map_err(storage_error)?;
        record.validate()?;
        if record.document.name != name || record.package.id != id {
            return Err("Saved package identity is inconsistent. Retained files remain.".into());
        }
        Ok(record)
    })
    .transpose()
}
fn version_ref(c: &Connection, root: Option<&str>, id: &str) -> Result<bool, String> {
    c.query_row("SELECT EXISTS(SELECT 1 FROM project_skills s,json_each(s.data,'$.versions') v WHERE (?1 IS NULL OR s.root=?1) AND json_extract(v.value,'$.package.id')=?2)",params![root,id],|r|r.get(0)).map_err(storage_error)
}
// Keep package operations in the store's existing mutex/transaction boundary.
impl SqliteStore {
    pub(super) fn package_list(&self, root: &str) -> Result<Vec<Installed>, String> {
        root_valid(root)?;
        let c = self.lock()?;
        let mut st=c.prepare("SELECT name,id FROM skill_packages WHERE root=?1 ORDER BY name,rowid DESC LIMIT 61").map_err(storage_error)?;
        let keys = st
            .query_map([root], |r| {
                Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?))
            })
            .map_err(storage_error)?
            .collect::<Result<Vec<_>, _>>()
            .map_err(storage_error)?;
        if keys.len() > 60 {
            return Err(
                "Saved package catalogue exceeds its allowance. Retained files remain.".into(),
            );
        }
        keys.into_iter()
            .map(|(name, id)| {
                read_record(&c, root, &name, &id)?
                    .ok_or_else(|| "Package catalogue changed. Refresh Packages.".into())
            })
            .collect()
    }
    pub(super) fn package_retain(&self, root: &str, record: &Installed) -> Result<(), String> {
        root_valid(root)?;
        record.validate()?;
        let mut c = self.lock()?;
        let tx = c
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(storage_error)?;
        if let Some(old) = read_record(&tx, root, &record.document.name, &record.package.id)? {
            if old != *record {
                return Err("This package identity already has different saved metadata. Earlier data was retained.".into());
            }
            return Ok(());
        }
        let versions: usize = tx
            .query_row(
                "SELECT COUNT(*) FROM skill_packages WHERE root=?1 AND name=?2",
                params![root, record.document.name],
                |r| r.get(0),
            )
            .map_err(storage_error)?;
        let names: usize = tx
            .query_row(
                "SELECT COUNT(DISTINCT name) FROM skill_packages WHERE root=?1",
                [root],
                |r| r.get(0),
            )
            .map_err(storage_error)?;
        if versions >= dolores_core::MAX_SKILL_VERSIONS
            || (versions == 0 && names >= dolores_core::MAX_SAVED_SKILLS)
        {
            return Err("Packages retain five installed versions per skill and twelve names per scope. Remove an unused package version, then retry; active skills and files remain.".into());
        }
        let raw = serde_json::to_string(record).map_err(storage_error)?;
        if raw.len() > 512 * 1024 {
            return Err("Package metadata exceeds 512 KiB. Reduce its resource catalogue.".into());
        }
        tx.execute(
            "INSERT INTO skill_packages(root,name,id,data) VALUES(?1,?2,?3,?4)",
            params![root, record.document.name, record.package.id, raw],
        )
        .map_err(storage_error)?;
        tx.commit().map_err(storage_error)
    }
    pub(super) fn package_forget(&self, root: &str, id: &str) -> Result<(), String> {
        root_valid(root)?;
        if !valid_digest(id) {
            return Err("Choose a retained package identity.".into());
        }
        let mut c = self.lock()?;
        let tx = c
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(storage_error)?;
        if version_ref(&tx, Some(root), id)? {
            return Err("A retained skill version still uses this package. Forget older activations in Packages, or keep the current package; its resources remain.".into());
        }
        tx.execute(
            "DELETE FROM skill_packages WHERE root=?1 AND id=?2",
            params![root, id],
        )
        .map_err(storage_error)?;
        tx.commit().map_err(storage_error)
    }
    pub(super) fn package_forget_version(
        &self,
        root: &str,
        name: &str,
        revision: u32,
        version: u32,
        id: &str,
    ) -> Result<dolores_core::ProjectSkill, String> {
        root_valid(root)?;
        if !dolores_core::valid_skill_name(name) || !valid_digest(id) {
            return Err("Choose an exact saved package version.".into());
        }
        let mut c = self.lock()?;
        let tx = c
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(storage_error)?;
        let mut saved = skills::read(&tx, root)?
            .into_iter()
            .find(|s| s.name == name)
            .ok_or("Saved skill is no longer available. Refresh Packages and inspect it again.")?;
        if saved.revision != revision {
            return Err("Saved skill changed after review. Refresh Packages and inspect it again; current skills and files remain.".into());
        }
        let index = saved.versions.iter().position(|v|v.version == version && v.package.as_ref().is_some_and(|p|p.id == id))
            .ok_or("This saved activation no longer matches the reviewed package. Refresh Packages and inspect it again.")?;
        if index + 1 == saved.versions.len() {
            return Err("The current saved version cannot be forgotten here. Choose an older activation; the current skill and its resources remain.".into());
        }
        saved.versions.remove(index);
        saved.revision = saved
            .revision
            .checked_add(1)
            .ok_or("Skill revision limit reached. Current skills and files remain.")?;
        saved.validate()?;
        tx.execute(
            "UPDATE project_skills SET data=?3 WHERE root=?1 AND name=?2",
            params![
                root,
                name,
                serde_json::to_string(&saved).map_err(storage_error)?
            ],
        )
        .map_err(storage_error)?;
        tx.commit().map_err(storage_error)?;
        Ok(saved)
    }
    pub(super) fn package_referenced(&self, id: &str) -> Result<bool, String> {
        if !valid_digest(id) {
            return Err("Choose a retained package identity.".into());
        }
        let c = self.lock()?;
        let installed: bool = c
            .query_row(
                "SELECT EXISTS(SELECT 1 FROM skill_packages WHERE id=?1)",
                [id],
                |r| r.get(0),
            )
            .map_err(storage_error)?;
        Ok(installed || version_ref(&c, None, id)?)
    }
    pub(super) fn package_activate(
        &self,
        root: &str,
        document: &dolores_core::SkillDocument,
        package: &Snapshot,
        revision: Option<u32>,
        rollback: Option<u32>,
    ) -> Result<dolores_core::ProjectSkill, String> {
        self.activate_skill_reviewed(root, document, revision, rollback, None, Some(package))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use dolores_core::{
        skill_packages::{Manifest, Resource},
        SkillDocument,
    };
    use sha2::{Digest, Sha256};
    fn package(folder: &Path, n: usize) -> Installed {
        let document = SkillDocument {
            name: "review".into(),
            description: "Review code".into(),
            text: "---\nname: review\ndescription: Review code\n---\nPublic package instructions"
                .into(),
        };
        Installed {
            package: Snapshot {
                id: format!("{n:064x}"),
                content_digest: format!("{:064x}", n + 100),
                manifest: Manifest {
                    version: format!("1.0.{n}"),
                    ..Default::default()
                },
                resources: vec![Resource {
                    path: "SKILL.md".into(),
                    bytes: document.text.len(),
                    digest: format!("{:x}", Sha256::digest(document.text.as_bytes())),
                }],
                directories: vec![],
                root: folder
                    .join(format!("package-{n}"))
                    .to_string_lossy()
                    .into_owned(),
            },
            document,
            installed_at: 1,
        }
    }
    #[test]
    fn installations_are_separate_and_rollback_retains_the_exact_package_across_reopen() {
        let folder = tempfile::tempdir().unwrap();
        let path = folder.path().join("state.db");
        let s = SqliteStore::open(&path).unwrap();
        let root = skills::GLOBAL_ROOT;
        s.create("task").unwrap();
        s.save_draft("task", "Unsent 世界").unwrap();
        let first = package(folder.path(), 1);
        let second = package(folder.path(), 2);
        s.retain_skill_package(root, &first).unwrap();
        assert!(s.global_skills().unwrap().is_empty());
        s.activate_skill_package(root, &first.document, &first.package, None, None)
            .unwrap();
        s.retain_skill_package(root, &second).unwrap();
        assert_eq!(
            s.global_skills().unwrap()[0].current().package.as_ref(),
            Some(&first.package)
        );
        s.activate_skill_package(root, &second.document, &second.package, Some(1), None)
            .unwrap();
        assert!(s
            .activate_skill_package(root, &first.document, &first.package, Some(1), None)
            .is_err());
        let restored = s
            .activate_global_skill(&first.document, Some(2), Some(1))
            .unwrap();
        assert_eq!(restored.current().package.as_ref(), Some(&first.package));
        assert_eq!(restored.current().rollback_from, Some(1));
        assert!(s.forget_skill_package(root, &first.package.id).is_err());
        assert_eq!(s.saved_draft("task").unwrap(), "Unsent 世界");
        drop(s);
        let s = SqliteStore::open(&path).unwrap();
        assert_eq!(s.global_skills().unwrap()[0], restored);
        s.retain_skill_package(folder.path().to_str().unwrap(), &first)
            .unwrap();
        s.forget_global_skill("review", 3).unwrap();
        s.forget_skill_package(root, &first.package.id).unwrap();
        assert!(s.skill_package_referenced(&first.package.id).unwrap());
        s.forget_skill_package(folder.path().to_str().unwrap(), &first.package.id)
            .unwrap();
        assert!(!s.skill_package_referenced(&first.package.id).unwrap());
    }
    #[test]
    fn failed_registration_and_capacity_preserve_installed_versions_and_active_state() {
        let folder = tempfile::tempdir().unwrap();
        let s = SqliteStore::open(&folder.path().join("state.db")).unwrap();
        let root = skills::GLOBAL_ROOT;
        let first = package(folder.path(), 1);
        s.retain_skill_package(root, &first).unwrap();
        let active = s
            .activate_skill_package(root, &first.document, &first.package, None, None)
            .unwrap();
        s.lock().unwrap().execute_batch("CREATE TRIGGER refuse_package BEFORE INSERT ON skill_packages BEGIN SELECT RAISE(ABORT,'fixture'); END;").unwrap();
        assert!(s
            .retain_skill_package(root, &package(folder.path(), 2))
            .is_err());
        assert_eq!(s.global_skills().unwrap(), vec![active.clone()]);
        assert_eq!(
            s.installed_skill_packages(root).unwrap(),
            vec![first.clone()]
        );
        s.lock()
            .unwrap()
            .execute_batch("DROP TRIGGER refuse_package")
            .unwrap();
        for n in 2..=5 {
            s.retain_skill_package(root, &package(folder.path(), n))
                .unwrap();
        }
        assert!(s
            .retain_skill_package(root, &package(folder.path(), 6))
            .unwrap_err()
            .contains("five installed"));
        assert_eq!(s.global_skills().unwrap(), vec![active]);
        let mut invalid = package(folder.path(), 6);
        invalid.package.root = "../outside".into();
        assert!(s.retain_skill_package(root, &invalid).is_err());
        assert_eq!(s.installed_skill_packages(root).unwrap().len(), 5);
    }
    #[test]
    fn five_activated_installations_refuse_a_sixth_without_clearing_the_current_version() {
        let folder = tempfile::tempdir().unwrap();
        let s = SqliteStore::open(&folder.path().join("state.db")).unwrap();
        let root = skills::GLOBAL_ROOT;
        let mut previous = None;
        for n in 1..=5 {
            let p = package(folder.path(), n);
            s.retain_skill_package(root, &p).unwrap();
            let active = s
                .activate_skill_package(root, &p.document, &p.package, previous, None)
                .unwrap();
            previous = Some(active.revision);
        }
        let before = s.global_skills().unwrap();
        assert_eq!(before[0].current().version, 5);
        assert_eq!(before[0].versions.len(), 5);
        assert!(s
            .retain_skill_package(root, &package(folder.path(), 6))
            .unwrap_err()
            .contains("five installed"));
        for n in 1..=5 {
            assert!(s
                .forget_skill_package(root, &package(folder.path(), n).package.id)
                .unwrap_err()
                .contains("retained skill version"));
        }
        assert_eq!(s.global_skills().unwrap(), before);
        assert_eq!(s.installed_skill_packages(root).unwrap().len(), 5);
    }
    #[test]
    fn forgetting_an_old_activation_keeps_current_scope_files_and_versions_and_unblocks_the_sixth_install(
    ) {
        let folder = tempfile::tempdir().unwrap();
        let path = folder.path().join("state.db");
        let s = SqliteStore::open(&path).unwrap();
        for root in [folder.path().to_str().unwrap(), skills::GLOBAL_ROOT] {
            let mut revision = None;
            for n in 1..=5 {
                let p = package(folder.path(), n);
                s.retain_skill_package(root, &p).unwrap();
                revision = Some(
                    s.activate_skill_package(root, &p.document, &p.package, revision, None)
                        .unwrap()
                        .revision,
                );
            }
            let before = s.read_skills(root).unwrap().remove(0);
            let first = package(folder.path(), 1);
            let current = before.current().clone();
            assert!(s
                .forget_skill_package_version(
                    root,
                    "review",
                    5,
                    5,
                    &current.package.as_ref().unwrap().id
                )
                .unwrap_err()
                .contains("current saved version"));
            assert!(s
                .forget_skill_package_version(root, "review", 4, 1, &first.package.id)
                .unwrap_err()
                .contains("changed after review"));
            assert!(s
                .forget_skill_package_version(
                    root,
                    "review",
                    5,
                    1,
                    &current.package.as_ref().unwrap().id
                )
                .unwrap_err()
                .contains("no longer matches"));
            s.lock().unwrap().execute_batch("CREATE TRIGGER refuse_version_forget BEFORE UPDATE ON project_skills BEGIN SELECT RAISE(ABORT,'fixture'); END;").unwrap();
            assert!(s
                .forget_skill_package_version(root, "review", 5, 1, &first.package.id)
                .is_err());
            assert_eq!(s.read_skills(root).unwrap(), vec![before.clone()]);
            s.lock()
                .unwrap()
                .execute_batch("DROP TRIGGER refuse_version_forget;")
                .unwrap();
            let result = s
                .forget_skill_package_version(root, "review", 5, 1, &first.package.id)
                .unwrap();
            assert_eq!(result.current(), &current);
            assert_eq!(result.enabled, before.enabled);
            assert_eq!(result.scope, before.scope);
            assert_eq!(result.revision, 6);
            assert_eq!(
                result
                    .versions
                    .iter()
                    .map(|v| v.version)
                    .collect::<Vec<_>>(),
                vec![2, 3, 4, 5]
            );
            assert!(s
                .forget_skill_package_version(root, "review", 6, 1, &first.package.id)
                .is_err());
            assert_eq!(s.installed_skill_packages(root).unwrap().len(), 5);
            s.forget_skill_package(root, &first.package.id).unwrap();
            let sixth = package(folder.path(), 6);
            s.retain_skill_package(root, &sixth).unwrap();
            assert_eq!(s.read_skills(root).unwrap()[0].current(), &current);
            let activated = s
                .activate_skill_package(root, &sixth.document, &sixth.package, Some(6), None)
                .unwrap();
            assert_eq!(activated.current().version, 6);
            assert_eq!(activated.revision, 7);
            assert_eq!(activated.versions.len(), 5);
        }
        drop(s);
        let s = SqliteStore::open(&path).unwrap();
        assert_eq!(s.global_skills().unwrap()[0].current().version, 6);
        assert_eq!(
            s.project_skills(folder.path().to_str().unwrap()).unwrap()[0]
                .current()
                .version,
            6
        );
    }
    #[test]
    fn v48_migration_keeps_legacy_skills_and_drafts_without_creating_installations() {
        let folder = tempfile::tempdir().unwrap();
        let file = folder.path().join("state.db");
        let s = SqliteStore::open(&file).unwrap();
        s.create("task").unwrap();
        s.save_draft("task", "Legacy unsent draft").unwrap();
        let document = package(folder.path(), 1).document;
        let legacy = s.activate_global_skill(&document, None, None).unwrap();
        s.lock()
            .unwrap()
            .execute_batch("DROP TABLE skill_packages; PRAGMA user_version=48;")
            .unwrap();
        drop(s);
        let s = SqliteStore::open(&file).unwrap();
        assert_eq!(s.global_skills().unwrap(), vec![legacy]);
        assert_eq!(s.saved_draft("task").unwrap(), "Legacy unsent draft");
        assert!(s
            .installed_skill_packages(skills::GLOBAL_ROOT)
            .unwrap()
            .is_empty());
        assert_eq!(
            s.lock()
                .unwrap()
                .query_row("PRAGMA user_version", [], |r| r.get::<_, i64>(0))
                .unwrap(),
            super::super::SCHEMA_VERSION
        );
    }
}
