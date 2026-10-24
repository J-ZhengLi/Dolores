use super::*;
use crate::*;
use dolores_core::skill_packages::{Installed, GLOBAL_ROOT};
use dolores_core::{ProjectSkill, SkillScope};
use serde::Deserialize;
use std::{
    io::Write,
    path::PathBuf,
    time::{Duration, Instant},
};

#[derive(Deserialize)]
#[serde(tag = "action", rename_all = "camelCase")]
pub enum Request {
    List,
    Inspect {
        name: String,
        path: PathBuf,
    },
    InspectInstalled {
        id: String,
    },
    InspectRetained {
        entry: String,
    },
    Install {
        token: String,
    },
    Activate {
        token: String,
        #[serde(default)]
        rollback: Option<u32>,
    },
    Export {
        token: String,
        path: PathBuf,
    },
    Remove {
        token: String,
    },
    ForgetVersion {
        token: String,
        version: u32,
    },
    Cancel {
        token: String,
    },
}

#[cfg(all(test, windows))]
mod tests {
    use super::*;
    use std::{io::Cursor, path::Path};
    fn archive(reference: &str) -> Vec<u8> {
        let mut zip = zip::ZipWriter::new(Cursor::new(Vec::new()));
        let options = zip::write::SimpleFileOptions::default()
            .compression_method(zip::CompressionMethod::Deflated);
        zip.add_directory("assets/empty/", options).unwrap();
        for (path,bytes) in [("SKILL.md",b"---\nname: review\ndescription: Review code\n---\nRead references/check.md. Inspect scripts/check.py before using a separately approved command.".as_slice()),("references/check.md",reference.as_bytes()),("scripts/check.py",b"raise RuntimeError('must never run during package management')".as_slice()),("assets/icon.bin",b"\0\x01\xff".as_slice())] {
            zip.start_file(path,options).unwrap();zip.write_all(bytes).unwrap();
        }
        zip.finish().unwrap().into_inner()
    }
    fn engine(profile: &Path) -> (Engine, Arc<SqliteStore>) {
        let store = Arc::new(SqliteStore::open(&profile.join("state.db")).unwrap());
        if !store
            .sessions_page(None, false, 50)
            .unwrap()
            .items
            .iter()
            .any(|s| s.id == "task")
        {
            store.create("task").unwrap();
            store.save_draft("task", "Keep unsent 世界").unwrap();
        }
        let mut engine = Engine::new(
            store.clone(),
            Arc::new(crate::connection::testing::MemoryCredentials::default()),
        )
        .unwrap();
        engine.workspace_directory = Some(profile.join("workspaces"));
        engine.global_skills_directory = Some(profile.join("source-skills"));
        (engine, store)
    }
    fn call(e: &Engine, r: Request) -> Result<Value, String> {
        e.call(Command::SkillPackages {
            session: "task".into(),
            scope: SkillScope::Global,
            request: r,
        })
    }
    fn review(e: &Engine, profile: &Path, text: &str) -> String {
        let path = profile.join("input.zip");
        std::fs::write(&path, archive(text)).unwrap();
        call(
            e,
            Request::Inspect {
                name: "review".into(),
                path,
            },
        )
        .unwrap()["token"]
            .as_str()
            .unwrap()
            .into()
    }
    fn inspect_id(e: &Engine, id: &str) -> String {
        call(e, Request::InspectInstalled { id: id.into() }).unwrap()["token"]
            .as_str()
            .unwrap()
            .into()
    }
    #[test]
    fn failed_install_save_retries_one_complete_bundle_without_activating_and_rollback_keeps_resources(
    ) {
        let profile = tempfile::tempdir().unwrap();
        let (e, s) = engine(profile.path());
        let token = review(&e, profile.path(), "First public reference 世界");
        let first = call(
            &e,
            Request::Install {
                token: token.clone(),
            },
        )
        .unwrap();
        let id = first["installed"]["package"]["id"]
            .as_str()
            .unwrap()
            .to_owned();
        let root = PathBuf::from(first["installed"]["package"]["root"].as_str().unwrap());
        assert!(root.join("assets/empty").is_dir());
        assert_eq!(
            std::fs::read(root.join("assets/icon.bin")).unwrap(),
            b"\0\x01\xff"
        );
        assert!(s.global_skills().unwrap().is_empty());
        let cancel = CancellationToken::new();
        e.active
            .lock()
            .unwrap()
            .reserve(Run {
                thread: Some("task".into()),
                id: 720,
                cancel: cancel.clone(),
                events: tokio::sync::mpsc::channel(1).1,
                approvals: Arc::new(Mutex::new(None)),
            })
            .unwrap();
        assert!(call(
            &e,
            Request::Activate {
                token: token.clone(),
                rollback: None
            }
        )
        .unwrap_err()
        .contains("Finish or stop"));
        assert!(call(
            &e,
            Request::Remove {
                token: token.clone()
            }
        )
        .unwrap_err()
        .contains("Finish or stop"));
        assert!(e.active.lock().unwrap().get(720).is_some());
        assert!(!cancel.is_cancelled());
        e.active.lock().unwrap().remove(720);
        call(
            &e,
            Request::Activate {
                token,
                rollback: None,
            },
        )
        .unwrap();
        let before = s.global_skills().unwrap();
        let (context, sources) = dolores_core::prepare_skill_context(
            dolores_core::preview_context(vec![], "Review this public task").unwrap(),
            &before,
        )
        .unwrap();
        assert_eq!(sources[0].package.as_ref().unwrap().id, id);
        assert_eq!(sources[0].source, format!("{}/SKILL.md", root.display()));
        assert!(!context[0]
            .content
            .contains("must never run during package management"));
        let token = review(&e, profile.path(), "Second public reference");
        let db = rusqlite::Connection::open(profile.path().join("state.db")).unwrap();
        db.execute_batch("CREATE TRIGGER refuse_package BEFORE INSERT ON skill_packages BEGIN SELECT RAISE(ABORT,'fixture'); END;").unwrap();
        let error = call(
            &e,
            Request::Install {
                token: token.clone(),
            },
        )
        .unwrap_err();
        assert!(error.contains("Retry Install"));
        assert_eq!(s.global_skills().unwrap(), before);
        assert_eq!(e.package_cache().unwrap().entries().unwrap().len(), 2);
        assert_eq!(s.installed_skill_packages(GLOBAL_ROOT).unwrap().len(), 1);
        db.execute_batch("DROP TRIGGER refuse_package").unwrap();
        call(
            &e,
            Request::Install {
                token: token.clone(),
            },
        )
        .unwrap();
        assert_eq!(e.package_cache().unwrap().entries().unwrap().len(), 2);
        assert_eq!(s.global_skills().unwrap(), before);
        call(
            &e,
            Request::Activate {
                token,
                rollback: None,
            },
        )
        .unwrap();
        std::fs::write(
            root.join("references/check.md"),
            "Changed first-version resource",
        )
        .unwrap();
        let old_review = e
            .review_scoped_skill("task", "review", Some(1), SkillScope::Global)
            .unwrap();
        assert!(e
            .activate_skill("task", old_review["token"].as_str().unwrap())
            .is_err());
        assert_ne!(
            s.global_skills().unwrap()[0]
                .current()
                .package
                .as_ref()
                .unwrap()
                .id,
            id
        );
        std::fs::write(
            root.join("references/check.md"),
            "First public reference 世界",
        )
        .unwrap();
        let token = inspect_id(&e, &id);
        let restored = call(
            &e,
            Request::Activate {
                token,
                rollback: Some(1),
            },
        )
        .unwrap();
        assert_eq!(
            restored["versions"].as_array().unwrap().last().unwrap()["package"]["id"],
            id
        );
        assert_eq!(
            std::fs::read_to_string(root.join("references/check.md")).unwrap(),
            "First public reference 世界"
        );
        assert_eq!(s.saved_draft("task").unwrap(), "Keep unsent 世界");
        drop(e);
        let (e, s) = engine(profile.path());
        assert_eq!(
            s.global_skills().unwrap()[0]
                .current()
                .package
                .as_ref()
                .unwrap()
                .id,
            id
        );
        assert!(
            call(&e, Request::List).unwrap()["installed"]
                .as_array()
                .unwrap()
                .len()
                == 2
        );
    }
    #[test]
    fn stale_scope_tampering_export_and_partial_cleanup_keep_foreign_work_and_allow_explicit_recovery(
    ) {
        let profile = tempfile::tempdir().unwrap();
        let (e, s) = engine(profile.path());
        let token = review(&e, profile.path(), "Public reference");
        let wrong = e
            .call(Command::SkillPackages {
                session: "another-task".into(),
                scope: SkillScope::Global,
                request: Request::Install {
                    token: token.clone(),
                },
            })
            .unwrap_err();
        assert!(wrong.contains("scope/version changed"));
        let installed = call(
            &e,
            Request::Install {
                token: token.clone(),
            },
        )
        .unwrap();
        let id = installed["installed"]["package"]["id"]
            .as_str()
            .unwrap()
            .to_owned();
        let root = PathBuf::from(installed["installed"]["package"]["root"].as_str().unwrap());
        let path = profile.path().join("export.zip");
        call(
            &e,
            Request::Export {
                token: token.clone(),
                path: path.clone(),
            },
        )
        .unwrap();
        assert_eq!(
            std::fs::read(&path).unwrap(),
            std::fs::read(profile.path().join("input.zip")).unwrap()
        );
        assert!(call(
            &e,
            Request::Export {
                token: token.clone(),
                path: path.clone()
            }
        )
        .is_err());
        std::fs::write(root.join("references/check.md"), "User-changed resource").unwrap();
        assert!(call(
            &e,
            Request::Activate {
                token: token.clone(),
                rollback: None
            }
        )
        .is_err());
        assert!(s.global_skills().unwrap().is_empty());
        assert!(call(
            &e,
            Request::Remove {
                token: token.clone()
            }
        )
        .is_err());
        assert!(root.parent().unwrap().join("source.zip").is_file());
        assert_eq!(
            std::fs::read_to_string(root.join("references/check.md")).unwrap(),
            "User-changed resource"
        );
        std::fs::write(root.join("references/check.md"), "Public reference").unwrap();
        std::fs::remove_file(root.join("scripts/check.py")).unwrap();
        call(&e, Request::Remove { token }).unwrap();
        assert!(!root.parent().unwrap().exists());
        assert!(s.installed_skill_packages(GLOBAL_ROOT).unwrap().is_empty());
        assert!(path.exists()); // A chosen export is not owned cache data.
        assert_eq!(s.saved_draft("task").unwrap(), "Keep unsent 世界");
        let token = review(&e, profile.path(), "Recovery reference");
        e.package_review.lock().unwrap().as_mut().unwrap().created =
            Instant::now() - Duration::from_secs(301);
        assert!(call(&e, Request::Install { token }).is_err());
        assert!(e.package_cache().unwrap().entries().unwrap().is_empty());
        let _ = id;
    }
    #[test]
    fn native_cache_links_existing_destinations_and_capacity_refuse_before_writing() {
        use std::os::windows::process::CommandExt;
        let profile = tempfile::tempdir().unwrap();
        let cache = bundles::Cache::open(profile.path()).unwrap();
        let bytes = archive("Public reference");
        let id = inspect("review", &bytes).unwrap().archive_digest;
        let outside = tempfile::tempdir().unwrap();
        std::fs::write(outside.path().join("keep.txt"), "Foreign work").unwrap();
        let destination = cache.root.join(&id);
        assert!(std::process::Command::new("cmd")
            .creation_flags(0x08000000)
            .args(["/c", "mklink", "/J"])
            .arg(&destination)
            .arg(outside.path())
            .output()
            .unwrap()
            .status
            .success());
        assert!(cache.publish("review", &bytes).is_err());
        assert_eq!(
            std::fs::read_to_string(outside.path().join("keep.txt")).unwrap(),
            "Foreign work"
        );
        assert!(!outside.path().join("source.zip").exists());
        std::fs::remove_dir(&destination).unwrap(); // Known isolated junction only.
        std::fs::create_dir(&destination).unwrap();
        assert!(cache.publish("review", &bytes).is_err());
        assert!(destination.is_dir());
        std::fs::remove_dir(&destination).unwrap();
        for n in 0..dolores_core::skill_packages::MAX_INSTALLED_BUNDLES {
            std::fs::create_dir(cache.root.join(format!("{n:064x}"))).unwrap();
        }
        assert!(cache
            .publish("review", &bytes)
            .unwrap_err()
            .contains("sixty"));
        assert_eq!(
            std::fs::read_dir(&cache.root).unwrap().count(),
            dolores_core::skill_packages::MAX_INSTALLED_BUNDLES
        );
        assert_eq!(
            std::fs::read_to_string(outside.path().join("keep.txt")).unwrap(),
            "Foreign work"
        );
    }
    #[test]
    fn older_activation_recovery_retries_failed_saves_and_a_sixth_install_without_changing_the_current_package(
    ) {
        for scope in [SkillScope::Project, SkillScope::Global] {
            let profile = tempfile::tempdir().unwrap();
            let (e, s) = engine(profile.path());
            let project = profile.path().join("project");
            std::fs::create_dir(&project).unwrap();
            let session = if scope == SkillScope::Project {
                s.create_workspace_session(
                    "project-task",
                    &dolores_core::SessionWorkspace {
                        kind: dolores_core::WorkspaceKind::Project,
                        root: Some(
                            std::fs::canonicalize(project)
                                .unwrap()
                                .to_string_lossy()
                                .into_owned(),
                        ),
                    },
                )
                .unwrap();
                "project-task"
            } else {
                "task"
            };
            s.save_draft(session, "Retained current draft 世界")
                .unwrap();
            let scoped = |request| {
                e.call(Command::SkillPackages {
                    session: session.into(),
                    scope,
                    request,
                })
            };
            let root = e.package_root(session, scope).unwrap();
            let mut installed = vec![];
            let input = profile.path().join("input.zip");
            for n in 1..=5 {
                std::fs::write(&input, archive(&format!("Public version {n}"))).unwrap();
                let token = scoped(Request::Inspect {
                    name: "review".into(),
                    path: input.clone(),
                })
                .unwrap()["token"]
                    .as_str()
                    .unwrap()
                    .to_owned();
                let result = scoped(Request::Install {
                    token: token.clone(),
                })
                .unwrap();
                installed.push(
                    serde_json::from_value::<Installed>(result["installed"].clone()).unwrap(),
                );
                scoped(Request::Activate {
                    token,
                    rollback: None,
                })
                .unwrap();
            }
            let before = e.package_skills(&root).unwrap();
            let current = before[0].current().clone();
            std::fs::write(&input, archive("Public version 6")).unwrap();
            let token = scoped(Request::Inspect {
                name: "review".into(),
                path: input.clone(),
            })
            .unwrap()["token"]
                .as_str()
                .unwrap()
                .to_owned();
            assert!(scoped(Request::Install { token })
                .unwrap_err()
                .contains("five installed"));
            assert_eq!(e.package_skills(&root).unwrap(), before);
            assert_eq!(e.package_cache().unwrap().entries().unwrap().len(), 6);
            let token = scoped(Request::InspectInstalled {
                id: installed[4].package.id.clone(),
            })
            .unwrap()["token"]
                .as_str()
                .unwrap()
                .to_owned();
            assert!(scoped(Request::ForgetVersion { token, version: 5 })
                .unwrap_err()
                .contains("current saved version"));
            let token = scoped(Request::InspectInstalled {
                id: installed[0].package.id.clone(),
            })
            .unwrap()["token"]
                .as_str()
                .unwrap()
                .to_owned();
            assert!(scoped(Request::ForgetVersion {
                token: token.clone(),
                version: 2
            })
            .unwrap_err()
            .contains("does not match"));
            let cancel = CancellationToken::new();
            e.active
                .lock()
                .unwrap()
                .reserve(Run {
                    thread: Some(session.into()),
                    id: 731,
                    cancel: cancel.clone(),
                    events: tokio::sync::mpsc::channel(1).1,
                    approvals: Arc::new(Mutex::new(None)),
                })
                .unwrap();
            assert!(scoped(Request::ForgetVersion {
                token: token.clone(),
                version: 1
            })
            .unwrap_err()
            .contains("Finish or stop"));
            assert!(!cancel.is_cancelled());
            e.active.lock().unwrap().remove(731);
            let first_file = Path::new(&installed[0].package.root).join("references/check.md");
            std::fs::write(&first_file, "Changed cache bytes must be preserved").unwrap();
            let token = scoped(Request::InspectRetained {
                entry: installed[0].package.id.clone(),
            })
            .unwrap()["token"]
                .as_str()
                .unwrap()
                .to_owned();
            let db = rusqlite::Connection::open(profile.path().join("state.db")).unwrap();
            db.execute_batch("CREATE TRIGGER refuse_old_version BEFORE UPDATE ON project_skills BEGIN SELECT RAISE(ABORT,'fixture'); END;").unwrap();
            assert!(scoped(Request::ForgetVersion {
                token: token.clone(),
                version: 1
            })
            .unwrap_err()
            .contains("Retry Forget saved version"));
            assert_eq!(e.package_skills(&root).unwrap(), before);
            db.execute_batch("DROP TRIGGER refuse_old_version;")
                .unwrap();
            let result = scoped(Request::ForgetVersion {
                token: token.clone(),
                version: 1,
            })
            .unwrap();
            assert_eq!(result["active"]["revision"], 6);
            assert_eq!(e.package_skills(&root).unwrap()[0].current(), &current);
            assert!(Path::new(&installed[0].package.root)
                .join("references/check.md")
                .exists());
            assert!(scoped(Request::Remove { token })
                .unwrap_err()
                .contains("retained data remains"));
            assert_eq!(e.package_skills(&root).unwrap()[0].current(), &current);
            assert_eq!(
                std::fs::read_to_string(&first_file).unwrap(),
                "Changed cache bytes must be preserved"
            );
            let token = scoped(Request::Inspect {
                name: "review".into(),
                path: input,
            })
            .unwrap()["token"]
                .as_str()
                .unwrap()
                .to_owned();
            scoped(Request::Install {
                token: token.clone(),
            })
            .unwrap();
            assert_eq!(e.package_cache().unwrap().entries().unwrap().len(), 6);
            assert_eq!(e.package_skills(&root).unwrap()[0].current(), &current);
            let active = scoped(Request::Activate {
                token,
                rollback: None,
            })
            .unwrap();
            assert_eq!(
                active["versions"].as_array().unwrap().last().unwrap()["version"],
                6
            );
            std::fs::write(&first_file, "Public version 1").unwrap();
            let cleanup = scoped(Request::InspectRetained {
                entry: installed[0].package.id.clone(),
            })
            .unwrap()["token"]
                .as_str()
                .unwrap()
                .to_owned();
            scoped(Request::Remove { token: cleanup }).unwrap();
            assert_eq!(e.package_cache().unwrap().entries().unwrap().len(), 5);
            let token = scoped(Request::InspectInstalled {
                id: installed[1].package.id.clone(),
            })
            .unwrap()["token"]
                .as_str()
                .unwrap()
                .to_owned();
            s.forget_skill_package_version(&root, "review", 7, 3, &installed[2].package.id)
                .unwrap();
            assert!(scoped(Request::ForgetVersion { token, version: 2 })
                .unwrap_err()
                .contains("scope/version changed"));
            assert_eq!(e.package_skills(&root).unwrap()[0].current().version, 6);
            assert_eq!(
                s.saved_draft(session).unwrap(),
                "Retained current draft 世界"
            );
        }
    }
    #[test]
    fn interrupted_publication_reimports_source_then_removes_only_staging() {
        let profile = tempfile::tempdir().unwrap();
        let (e, s) = engine(profile.path());
        let cache = e.package_cache().unwrap();
        let bytes = archive("Public reference");
        let published = cache.publish("review", &bytes).unwrap();
        let id = published.package.id;
        let stage = format!("stage-{}", uuid::Uuid::new_v4());
        let stage_path = cache.root.join(&stage);
        std::fs::rename(cache.root.join(&id), &stage_path).unwrap();
        std::fs::write(stage_path.join("source.zip"), &bytes[..20]).unwrap();
        std::fs::write(stage_path.join("payload/references/check.md"), b"Public").unwrap();
        let token = call(
            &e,
            Request::InspectRetained {
                entry: stage.clone(),
            },
        )
        .unwrap()["token"]
            .as_str()
            .unwrap()
            .to_owned();
        assert!(call(&e, Request::Remove { token })
            .unwrap_err()
            .contains("Reimport its original ZIP"));
        assert!(stage_path.is_dir());
        assert!(s.installed_skill_packages(GLOBAL_ROOT).unwrap().is_empty());
        let input = profile.path().join("recovery.zip");
        std::fs::write(&input, &bytes).unwrap();
        let token = call(
            &e,
            Request::Inspect {
                name: "review".into(),
                path: input,
            },
        )
        .unwrap()["token"]
            .as_str()
            .unwrap()
            .to_owned();
        call(&e, Request::Install { token }).unwrap();
        let before = s.installed_skill_packages(GLOBAL_ROOT).unwrap();
        let token = call(&e, Request::InspectRetained { entry: stage }).unwrap()["token"]
            .as_str()
            .unwrap()
            .to_owned();
        call(&e, Request::Remove { token }).unwrap();
        assert!(!stage_path.exists());
        assert_eq!(s.installed_skill_packages(GLOBAL_ROOT).unwrap(), before);
        assert_eq!(cache.checked(&id).unwrap().1, bytes);
        assert!(s.global_skills().unwrap().is_empty());
    }
}
#[derive(Clone)]
pub(crate) struct Review {
    token: String,
    session: String,
    scope: SkillScope,
    root: String,
    cache: PathBuf,
    package: Installed,
    bytes: Option<Vec<u8>>,
    entry: String,
    catalogue: Vec<Installed>,
    saved: Option<ProjectSkill>,
    created: Instant,
}
impl Engine {
    pub(crate) fn verify_skill_package(
        &self,
        document: &SkillDocument,
        package: &Snapshot,
    ) -> Result<(), String> {
        let (checked, _) = self.package_cache()?.checked(&package.id)?;
        if checked.package != *package || checked.document != *document {
            return Err(
                "Retained package changed. Reinstall or review its original ZIP before activation."
                    .into(),
            );
        }
        Ok(())
    }
    fn package_cache(&self) -> Result<bundles::Cache, String> {
        bundles::Cache::open(
            self.workspace_directory
                .as_ref()
                .and_then(|p| p.parent())
                .ok_or("Owned package storage is unavailable in this host.")?,
        )
    }
    fn package_root(&self, session: &str, scope: SkillScope) -> Result<String, String> {
        match scope {
            SkillScope::Global => Ok(GLOBAL_ROOT.into()),
            SkillScope::Project => self
                .store
                .workspace(session)?
                .root
                .ok_or("Choose a working project or Global packages.".into()),
        }
    }
    fn package_skills(&self, root: &str) -> Result<Vec<ProjectSkill>, String> {
        if root == GLOBAL_ROOT {
            self.store.global_skills()
        } else {
            self.store.project_skills(root)
        }
    }
    fn fresh_package(
        &self,
        r: &Review,
        session: &str,
        scope: SkillScope,
        token: &str,
        cache: &bundles::Cache,
    ) -> Result<(), String> {
        if r.token != token
            || r.session != session
            || r.scope != scope
            || r.root != self.package_root(session, scope)?
            || r.cache != cache.root
            || r.created.elapsed() > Duration::from_secs(300)
            || self.store.installed_skill_packages(&r.root)? != r.catalogue
            || self
                .package_skills(&r.root)?
                .into_iter()
                .find(|s| s.name == r.package.document.name)
                != r.saved
        {
            return Err("Package review expired or its scope/version changed. Refresh Packages and inspect it again; active skills and retained files remain.".into());
        }
        Ok(())
    }
    fn review_package(
        &self,
        session: &str,
        scope: SkillScope,
        cache: &bundles::Cache,
        package: Installed,
        bytes: Option<Vec<u8>>,
        entry: String,
    ) -> Result<Value, String> {
        let root = self.package_root(session, scope)?;
        let catalogue = self.store.installed_skill_packages(&root)?;
        let saved = self
            .package_skills(&root)?
            .into_iter()
            .find(|s| s.name == package.document.name);
        let token = uuid::Uuid::new_v4().to_string();
        let registered = catalogue.iter().any(|p| p.package.id == package.package.id);
        let value = json!({"token":token,"package":package,"registered":registered,"scope":scope,"entry":entry,"active":saved,"installationOnly":true});
        *self
            .package_review
            .lock()
            .map_err(|_| "Package review is unavailable.")? = Some(Review {
            token,
            session: session.into(),
            scope,
            root,
            cache: cache.root.clone(),
            package,
            bytes,
            entry,
            catalogue,
            saved,
            created: Instant::now(),
        });
        Ok(value)
    }
    pub(crate) fn package_call(
        &self,
        session: &str,
        scope: SkillScope,
        request: Request,
        active: &run_journal::RunCoordinator,
    ) -> Result<Value, String> {
        if matches!(
            request,
            Request::Activate { .. } | Request::Remove { .. } | Request::ForgetVersion { .. }
        ) && active.is_some()
        {
            return Err("Finish or stop running tasks before changing saved package versions or removing resources. Installation and inspection remain available.".into());
        }
        let _pins = if scope == SkillScope::Project {
            Some(self.pin_worktree(session)?)
        } else {
            None
        };
        let root = self.package_root(session, scope)?;
        let cache = self.package_cache()?;
        match request {
            Request::List => {
                let installed = self.store.installed_skill_packages(&root)?;
                let retained=cache.entries()?.into_iter().map(|entry| {
                    match cache.receipt(&entry) {
                        Ok(p)=>{let referenced=self.store.skill_package_referenced(&p.package.id)?;Ok(json!({"entry":entry,"id":p.package.id,"name":p.document.name,"version":p.package.manifest.version,"referenced":referenced}))},
                        Err(error)=>Ok(json!({"entry":entry,"problem":error})),
                    }
                }).collect::<Result<Vec<Value>,String>>()?;
                Ok(
                    json!({"installed":installed,"retained":retained,"scope":scope,"maxBundles":dolores_core::skill_packages::MAX_INSTALLED_BUNDLES}),
                )
            }
            Request::Inspect { name, path } => {
                if !path.is_absolute()
                    || path
                        .extension()
                        .and_then(|s| s.to_str())
                        .is_none_or(|s| !s.eq_ignore_ascii_case("zip"))
                {
                    return Err("Choose an absolute .zip skill package path.".into());
                }
                let file = std::fs::File::open(&path)
                    .map_err(|_| "Package ZIP could not be opened. Choose a readable file.")?;
                if !file
                    .metadata()
                    .map_err(|_| "Package ZIP is unavailable.")?
                    .is_file()
                {
                    return Err("Choose a regular package ZIP file.".into());
                }
                let mut bytes = Vec::new();
                file.take((MAX_ARCHIVE_BYTES + 1) as u64)
                    .read_to_end(&mut bytes)
                    .map_err(|_| "Package ZIP could not be read.")?;
                let p = inspect(&name, &bytes)?;
                let package = Installed {
                    document: p.document.clone(),
                    package: cache.snapshot(&p),
                    installed_at: 0,
                };
                self.review_package(
                    session,
                    scope,
                    &cache,
                    package,
                    Some(bytes),
                    p.archive_digest,
                )
            }
            Request::InspectInstalled { id } => {
                let package = self
                    .store
                    .installed_skill_packages(&root)?
                    .into_iter()
                    .find(|p| p.package.id == id)
                    .ok_or("Package is no longer installed in this scope. Refresh Packages.")?;
                let (checked, _) = cache.checked(&id)?;
                if checked != package {
                    return Err("Saved package does not match its retained files. Reinstall the original ZIP.".into());
                }
                self.review_package(session, scope, &cache, package, None, id)
            }
            Request::InspectRetained { entry } => {
                let package = cache.receipt(&entry)?;
                self.review_package(session, scope, &cache, package, None, entry)
            }
            Request::Cancel { token } => {
                let mut slot = self
                    .package_review
                    .lock()
                    .map_err(|_| "Package review unavailable.")?;
                if slot.as_ref().is_some_and(|r| r.token == token) {
                    slot.take();
                }
                Ok(Value::Null)
            }
            action => {
                let token = match &action {
                    Request::Install { token }
                    | Request::Activate { token, .. }
                    | Request::Export { token, .. }
                    | Request::ForgetVersion { token, .. }
                    | Request::Remove { token } => token,
                    _ => unreachable!(),
                };
                let mut slot = self
                    .package_review
                    .lock()
                    .map_err(|_| "Package review unavailable.")?;
                let r = slot
                    .as_mut()
                    .ok_or("Inspect a package before choosing this action.")?;
                self.fresh_package(r, session, scope, token, &cache)?;
                match action {
                    Request::ForgetVersion { version, .. } => {
                        let saved = r.saved.as_ref().ok_or("This package has no saved activations in this scope. Refresh Packages.")?;
                        if !saved.versions.iter().any(|v| {
                            v.version == version && v.package.as_ref() == Some(&r.package.package)
                        }) {
                            return Err("This activation does not match the reviewed package. Refresh Packages and inspect the intended version.".into());
                        }
                        let skill = self.store.forget_skill_package_version(&root, &saved.name, saved.revision, version, &r.package.package.id)
                            .map_err(|e|format!("{e} Retry Forget saved version from this review, or refresh if it changed. No package files were removed."))?;
                        r.saved = Some(skill.clone());
                        Ok(
                            json!({"active":skill,"versionForgotten":version,"token":r.token,"filesChanged":false}),
                        )
                    }
                    Request::Install { .. } => {
                        let bytes=r.bytes.as_ref().ok_or("Choose a source ZIP to install, or inspect its already installed version.")?;
                        let retained = cache.publish(&r.package.document.name, bytes)?;
                        if retained.package != r.package.package
                            || retained.document != r.package.document
                        {
                            return Err("Publication did not match the reviewed package. Retained files remain for inspection.".into());
                        }
                        self.store.retain_skill_package(&root,&retained).map_err(|e|format!("{e} The complete bundle remains at {}. Retry Install with this review; no active skill changed.",cache.root.join(&retained.package.id).display()))?;
                        r.catalogue = self.store.installed_skill_packages(&root)?;
                        r.package = retained.clone();
                        r.bytes = None;
                        Ok(json!({"installed":retained,"activeSkillChanged":false,"token":r.token}))
                    }
                    Request::Activate { rollback, .. } => {
                        let installed = self
                            .store
                            .installed_skill_packages(&root)?
                            .into_iter()
                            .find(|p| p.package.id == r.package.package.id)
                            .ok_or("Install this package before activating it.")?;
                        let (checked, _) = cache.checked(&installed.package.id)?;
                        if checked != installed
                            || installed.package != r.package.package
                            || installed.document != r.package.document
                        {
                            return Err("Installed resources changed after review. Reinstall or inspect another retained version.".into());
                        }
                        let skill = self.store.activate_skill_package(
                            &root,
                            &installed.document,
                            &installed.package,
                            r.saved.as_ref().map(|s| s.revision),
                            rollback,
                        )?;
                        slot.take();
                        Ok(json!(skill))
                    }
                    Request::Export { path, .. } => {
                        let (installed, bytes) = cache.checked_archive(&r.package.package.id)?;
                        if installed.package != r.package.package
                            || installed.document != r.package.document
                        {
                            return Err(
                                "Retained package changed. Inspect it again before exporting."
                                    .into(),
                            );
                        }
                        if !path.is_absolute()
                            || path
                                .extension()
                                .and_then(|s| s.to_str())
                                .is_none_or(|s| !s.eq_ignore_ascii_case("zip"))
                            || path.symlink_metadata().is_ok()
                        {
                            return Err("Choose a new absolute .zip filename; existing files are never overwritten.".into());
                        }
                        let mut file = tempfile::NamedTempFile::new_in(
                            path.parent().ok_or("Choose an export folder.")?,
                        )
                        .map_err(|_| "Choose an existing writable export folder.")?;
                        file.write_all(&bytes)
                            .and_then(|()| file.as_file().sync_all())
                            .map_err(|_| {
                                "Could not finish the package export; retained files remain."
                            })?;
                        file.persist_noclobber(&path).map_err(|_|"Could not publish the package export. Choose another new filename and retry.")?;
                        Ok(json!({"bytes":bytes.len(),"archiveDigest":installed.package.id}))
                    }
                    Request::Remove { .. } => {
                        if r.entry.starts_with("stage-") {
                            cache.remove(&r.entry)?;
                            slot.take();
                            return Ok(json!({"stagingRemoved":true,"registrationUnchanged":true}));
                        }
                        self.store
                            .forget_skill_package(&root, &r.package.package.id)?;
                        r.catalogue = self.store.installed_skill_packages(&root)?;
                        if self.store.skill_package_referenced(&r.package.package.id)? {
                            slot.take();
                            return Ok(
                                json!({"registrationRemoved":true,"filesRetained":true,"note":"Another scope or retained skill version still uses this bundle."}),
                            );
                        }
                        cache.remove(&r.entry).map_err(|e|format!("{e} This scoped registration was removed; retained data remains at {}. Retry Remove from this review or inspect retained data again.",cache.root.join(&r.entry).display()))?;
                        slot.take();
                        Ok(json!({"registrationRemoved":true,"filesRemoved":true}))
                    }
                    _ => unreachable!(),
                }
            }
        }
    }
}
