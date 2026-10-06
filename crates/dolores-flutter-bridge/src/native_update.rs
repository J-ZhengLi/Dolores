//! Direct user installation review; deliberately absent from provider tools.
use super::{introspection::bundle, native_build, Engine};
use dolores_native_update as native;
use native::Handoff;
use serde_json::{json, Value};
use std::{
    fs,
    path::{Path, PathBuf},
    time::Instant,
};
pub(super) struct Review {
    pub token: String,
    job: Handoff,
    created: Instant,
    current: Option<String>,
}
fn profile(engine: &Engine) -> Result<&Path, String> {
    engine
        .workspace_directory
        .as_ref()
        .and_then(|p| p.parent())
        .ok_or_else(|| "Native update requires the maintained profile.".into())
}
fn receipt(profile: &Path, id: &str) -> Result<PathBuf, String> {
    if !native::id_valid(id) {
        return Err("Native intent identity is invalid; no replay.".into());
    }
    Ok(profile
        .join("native-updates")
        .join(format!("job-{id}"))
        .join("handoff.json"))
}
fn current(profile: &Path) -> Result<Option<Handoff>, String> {
    let p = profile.join("native-updates/current.json");
    if !p.exists() {
        return Ok(None);
    }
    let id: String = native::json(&p)?;
    let j: Handoff = native::json(&receipt(profile, &id)?)?;
    j.validate()?;
    if j.id != id || j.profile != profile {
        return Err(
            "Native intent no longer belongs to this profile; retain it for inspection.".into(),
        );
    }
    Ok(Some(j))
}
fn check_bundles(job: &Handoff) -> Result<(), String> {
    native::verify(&job.target(), job.target_bundle())?;
    native::verify(&job.previous(), job.previous_bundle())?;
    native::verify(
        job.old
            .executable
            .parent()
            .ok_or("Current bundle unavailable.")?,
        job.previous_bundle(),
    )
}
#[cfg(windows)]
fn own() -> Result<native::Process, String> {
    let p =
        native::process::identity(std::process::id())?.ok_or("Normal app process unavailable.")?;
    if p.executable.file_name().is_none_or(|n| n != native::APP) {
        return Err("Installation is available only in the visibly running normal Dolores app. Test-driver processes cannot install or restart it.".into());
    }
    Ok(p)
}
#[cfg(not(windows))]
fn own() -> Result<native::Process, String> {
    Err("Reviewed native restart is currently supported on Windows only.".into())
}
fn check_prior(profile: &Path) -> Result<Option<String>, String> {
    let Some(job) = current(profile)? else {
        return Ok(None);
    };
    if !job.terminal() {
        #[cfg(windows)]
        for p in [&job.helper, &job.child].into_iter().flatten() {
            if native::process::identity(p.pid)?.as_ref() == Some(p) {
                return Err("A native handoff process still owns this intent. Inspect that app and wait for its result; no second update or replay can start.".into());
            }
        }
    }
    Ok(Some(job.id))
}
pub(super) fn preflight(profile: &Path) -> Result<Option<PathBuf>, String> {
    let Some(id) = std::env::var_os("DOLORES_NATIVE_HANDOFF") else {
        return Ok(None);
    };
    let id = id.to_str().ok_or("Native startup identity is invalid.")?;
    let p = receipt(profile, id)?;
    let job: Handoff = native::json(&p)?;
    job.validate()?;
    if job.profile != profile
        || job.id != id
        || job.stage != "starting"
        || job.child.as_ref() != Some(&own()?)
        || job.expected_source != bundle::ID
        || job.build.schema != dolores_store_sqlite::SCHEMA_VERSION
        || native::database_schema(&profile.join("dolores.db"))? != job.build.schema
        || Some(native::database_identity(&profile.join("dolores.db"))?) != job.database_id
    {
        return Err("Native source/schema/process/history startup check failed. Candidate remains blocked; the protected launcher will recover the previous version.".into());
    }
    Ok(Some(p))
}
impl Engine {
    pub(super) fn native_ready(&self) -> Result<bool, String> {
        let Some(path) = &self.native_startup else {
            return Ok(true);
        };
        let job: Handoff = native::json(path)?;
        job.validate()?;
        if matches!(job.stage.as_str(), "applied" | "restored" | "rolledBack") {
            if job.child.as_ref() != Some(&own()?) || job.expected_source != bundle::ID {
                return Err(
                    "Native startup receipt changed. Keep saved work and inspect recovery.".into(),
                );
            }
            return Ok(true);
        }
        if job.stage == "recoveryRequired" {
            return Err(job.note);
        }
        Ok(false)
    }
    pub(super) fn native_startup_ready(&self) -> Result<Value, String> {
        let Some(path) = &self.native_startup else {
            return Ok(json!({"ready":true}));
        };
        let job: Handoff = native::json(path)?;
        job.validate()?;
        let process = own()?;
        if job.child.as_ref() != Some(&process)
            || job.expected_source != bundle::ID
            || job.stage != "starting"
        {
            return Err(
                "Native startup no longer belongs to this process; no healthy receipt published."
                    .into(),
            );
        }
        let db = native::database_identity(&job.profile.join("dolores.db"))?;
        if Some(&db) != job.database_id.as_ref() {
            return Err("History changed during blocked startup. Automatic acceptance withheld; launcher recovery remains.".into());
        }
        native::save(
            &path.parent().unwrap().join("health.json"),
            &native::Health {
                job: job.id,
                source: bundle::ID.into(),
                schema: dolores_store_sqlite::SCHEMA_VERSION,
                database: db,
                process,
            },
        )?;
        Ok(
            json!({"ready":false,"note":"Normal chat initialization acknowledged; waiting for protected launcher verification."}),
        )
    }
    pub(super) fn native_repairs(&self, session: Option<&str>) -> Result<Value, String> {
        let profile = profile(self)?;
        let mut builds = vec![];
        let directory = profile.join("repairs");
        if directory.exists() {
            native::safe_path(&directory)?;
            for (count, entry) in fs::read_dir(directory)
                .map_err(|_| "Native builds unavailable.")?
                .enumerate()
            {
                if count >= 4096 {
                    return Err("Native storage inspection allowance reached. Retained artifacts remain; inspect storage locally.".into());
                }
                let entry = entry.map_err(|_| "Native build entry unavailable.")?;
                let name = entry.file_name();
                let name = name.to_string_lossy();
                if let Some(id) = name.strip_prefix("build-") {
                    let b = native_build::load(profile, id)?;
                    if session.is_none_or(|s| s == b.session) {
                        builds.push(native_build::summary(&b));
                    }
                }
            }
        }
        let job = current(profile)?;
        let intent=job.map(|j|json!({"id":j.id,"buildId":j.build.id,"session":j.build.session,"stage":j.stage,"operation":j.operation,"note":j.note,"sourceId":j.expected_source,"interrupted":!j.terminal(),"recovery":"Retain both bundles and history backup. Refresh inspects only; no native operation or task resumes automatically. A dead handoff can be superseded only by a fresh exact review from a verified retained normal bundle."}));
        builds.sort_by(|a, b| a["id"].as_str().cmp(&b["id"].as_str()));
        Ok(
            json!({"builds":builds,"intent":intent,"sourceId":bundle::ID,"startup":self.native_startup.is_some(),"note":"Build, installation and Restore have separate reviews. Native code has account permissions. Restore changes application code and preserves current history; it does not replay a task."}),
        )
    }
    pub(super) fn review_native(
        &self,
        session: &str,
        id: &str,
        restore: bool,
    ) -> Result<Value, String> {
        let profile = profile(self)?;
        let current = check_prior(profile)?;
        let build = native_build::load(profile, id)?;
        if build.session != session
            || build.status != "ready"
            || build.schema != dolores_store_sqlite::SCHEMA_VERSION
        {
            return Err("Choose this chat's ready compatible native build.".into());
        }
        if !restore {
            let s = self.store.repair_workspace(session, &build.repair_id)?;
            super::harness_repair::matching(&s)?;
            super::harness_repair::artifact_matches(&profile.join("repairs"), &s)?;
            let e = native_build::qualified(self.store.as_ref(), &s, &build.evaluation_id)?;
            if s.revision != build.revision
                || s.bundle_id != build.source_id
                || native_build::candidate_id(&s) != build.candidate_source_id
                || e.cargo_id != build.cargo_id
            {
                return Err("Ready build no longer matches its qualified candidate; request a fresh build review.".into());
            }
        }
        if bundle::ID
            != if restore {
                &build.candidate_source_id
            } else {
                &build.source_id
            }
        {
            return Err("Running source no longer matches this installation/Restore. Keep both versions and inspect the current normal bundle.".into());
        }
        let old = own()?;
        let job = Handoff {
            id: uuid::Uuid::new_v4().to_string(),
            build_root: native_build::build_root(profile, id)?,
            profile: profile.to_path_buf(),
            expected_source: if restore {
                build.source_id.clone()
            } else {
                build.candidate_source_id.clone()
            },
            build,
            old,
            child: None,
            helper: None,
            operation: if restore { "restore" } else { "install" }.into(),
            stage: "waiting".into(),
            note: "Waiting for separate exact user review. No task replay.".into(),
            database_id: None,
            backup_id: None,
        };
        job.validate()?;
        check_bundles(&job)?;
        let token = uuid::Uuid::new_v4().to_string();
        let result = json!({"token":token,"operation":job.operation,"build":native_build::summary(&job.build),"profile":profile,"currentBundle":job.previous_bundle().id,"targetBundle":job.target_bundle().id,"currentExecutable":job.old.executable,"restart":true,"shutdownSeconds":30,"startupSeconds":30,"note":"Save drafts, close this idle app and start the exact retained version. Native code has this account's permissions. Failed startup recovers the pre-startup history snapshot; later reviewed Restore preserves current history. No task is resumed or replayed."});
        *self
            .native_review
            .lock()
            .map_err(|_| "Native review unavailable.")? = Some(Review {
            token,
            job,
            created: Instant::now(),
            current,
        });
        Ok(result)
    }
    pub(super) fn discard_native(&self, token: &str) -> Result<Value, String> {
        let mut slot = self
            .native_review
            .lock()
            .map_err(|_| "Native review unavailable.")?;
        if slot.as_ref().is_some_and(|r| r.token == token) {
            slot.take();
        }
        Ok(Value::Null)
    }
    pub(super) fn apply_native(&self, token: &str) -> Result<Value, String> {
        self.editor_can_restart()?;
        let mut slot = self
            .native_review
            .lock()
            .map_err(|_| "Native review unavailable.")?;
        if slot.as_ref().is_none_or(|r| r.token != token) {
            return Err(
                "Native review expired. Request a fresh exact review; no restart occurred.".into(),
            );
        }
        let review = slot.take().unwrap();
        drop(slot);
        if review.created.elapsed().as_secs() > 300
            || own()? != review.job.old
            || check_prior(&review.job.profile)? != review.current
            || native_build::load(&review.job.profile, &review.job.build.id)? != review.job.build
        {
            return Err(
                "Native review or app changed. Drafts remain; request a fresh exact review.".into(),
            );
        }
        let job = review.job;
        check_bundles(&job)?;
        if job.operation == "install" {
            let s = self
                .store
                .repair_workspace(&job.build.session, &job.build.repair_id)?;
            native_build::qualified(self.store.as_ref(), &s, &job.build.evaluation_id)?;
            super::harness_repair::matching(&s)?;
            super::harness_repair::artifact_matches(&job.profile.join("repairs"), &s)?;
            if native_build::candidate_id(&s) != job.build.candidate_source_id
                || s.revision != job.build.revision
            {
                return Err("Candidate changed after review. No restart occurred.".into());
            }
        }
        let folder = job.profile.join("native-updates");
        fs::create_dir_all(&folder)
            .map_err(|_| "Native intent could not be retained; keep the current app.")?;
        native::safe_path(&folder)?;
        if let Some(mut previous) = current(&job.profile)? {
            if !previous.terminal() {
                previous.stage = "cancelled".into();
                previous.note="Interrupted intent superseded by a new exact review from a verified retained normal bundle; no operation or task replay.".into();
                native::save(&receipt(&job.profile, &previous.id)?, &previous)?;
            }
        }
        let path = receipt(&job.profile, &job.id)?;
        fs::create_dir(path.parent().unwrap())
            .map_err(|_| "Separate native intent unavailable.")?;
        native::save(&path, &job)?;
        native::save(&folder.join("current.json"), &job.id)?;
        #[cfg(windows)]
        {
            use std::os::windows::process::CommandExt;
            let result = std::process::Command::new(job.previous().join(native::LAUNCHER))
                .args([std::ffi::OsStr::new("--handoff"), path.as_os_str()])
                .creation_flags(0x08000000)
                .spawn();
            if result.is_err() {
                let mut job = job;
                job.stage = "failed".into();
                job.note="Protected launcher could not start. Current app and drafts remain; request a fresh review.".into();
                native::save(&path, &job)?;
                return Err(job.note);
            }
            self.native_leaving
                .store(true, std::sync::atomic::Ordering::SeqCst);
            Ok(
                json!({"restart":true,"id":job.id,"note":"Native handoff retained. Close this idle app now; protected launcher waits without force-stopping it."}),
            )
        }
        #[cfg(not(windows))]
        {
            Err("Native restart is currently supported on Windows only.".into())
        }
    }
}
