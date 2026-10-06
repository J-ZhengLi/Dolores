use dolores_native_update::*;
use std::{
    fs,
    path::Path,
    time::{Duration, Instant},
};
fn pause() {
    std::thread::sleep(Duration::from_millis(100));
}
fn lock_profile(profile: &Path) -> Result<fs::File, String> {
    safe_path(profile)?;
    let path = profile.join("dolores.lock");
    if path.exists() {
        safe_path(&path)?;
    }
    let lock = fs::OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(false)
        .open(path)
        .map_err(|_| "History lock unavailable.")?;
    lock.try_lock().map_err(|_| {
        "Another process owns history; retain both bundles and inspect before recovery."
    })?;
    Ok(lock)
}
fn wait_exit(p: &Process) -> Result<(), String> {
    let deadline = Instant::now() + Duration::from_secs(30);
    loop {
        match process::identity(p.pid)? {
            None => return Ok(()),
            Some(current) if &current != p => {
                return Err("Process identity changed; no unrelated process was stopped.".into())
            }
            _ => {}
        }
        if Instant::now() >= deadline {
            return Err("The app did not close within 30 seconds. No busy app was forcibly stopped; retained bundle remains.".into());
        }
        pause();
    }
}
fn expect_health(job: &Handoff, folder: &Path) -> Result<(), String> {
    let deadline = Instant::now() + Duration::from_secs(30);
    loop {
        let child = job
            .child
            .as_ref()
            .ok_or("Candidate process receipt missing.")?;
        if process::identity(child.pid)?.as_ref() != Some(child) {
            return Err("Candidate exited or changed before healthy startup.".into());
        }
        let path = folder.join("health.json");
        if path.exists() {
            let h: Health = json(&path)?;
            if h.job != job.id
                || h.source != job.expected_source
                || h.schema != job.build.schema
                || &h.process != child
                || Some(&h.database) != job.database_id.as_ref()
                || database_identity(&job.profile.join("dolores.db"))? != h.database
            {
                return Err("Candidate startup/source/schema/history check failed; no healthy handoff recorded.".into());
            }
            let expected = if job.expected_source == job.build.source_id {
                job.build.previous.as_ref().unwrap()
            } else {
                job.build.candidate.as_ref().unwrap()
            };
            verify(
                child
                    .executable
                    .parent()
                    .ok_or("Started bundle unavailable.")?,
                expected,
            )?;
            return Ok(());
        }
        if Instant::now() >= deadline {
            return Err("Candidate startup exceeded 30 seconds; retained previous bundle will be recovered.".into());
        }
        pause();
    }
}
fn restore_snapshot(job: &Handoff, folder: &Path) -> Result<(), String> {
    let backup = folder.join("profile-before.sqlite");
    if Some(file_hash(&backup)?) != job.backup_id {
        return Err(
            "History backup changed; automatic recovery withheld. Keep both copies for inspection."
                .into(),
        );
    }
    let _lock = lock_profile(&job.profile)?;
    let temporary = job
        .profile
        .join(format!("native-restore-{}.sqlite", job.id));
    if temporary.exists() {
        return Err("History restore intent already exists; inspect without replay.".into());
    }
    fs::copy(&backup, &temporary)
        .map_err(|_| "History recovery copy failed; retained backup remains.")?;
    if database_identity(&temporary)?
        != *job.database_id.as_ref().ok_or("History receipt missing.")?
    {
        return Err("History backup identity failed verification.".into());
    }
    for suffix in ["dolores.db-wal", "dolores.db-shm"] {
        let p = job.profile.join(suffix);
        if p.exists() {
            safe_path(&p)?;
            fs::remove_file(p)
                .map_err(|_| "History journal could not be recovered; retained backup remains.")?;
        }
    }
    let db = job.profile.join("dolores.db");
    safe_path(&db)?;
    fs::rename(temporary, db)
        .map_err(|_| "History snapshot could not be restored; retain its copy.")?;
    Ok(())
}
pub fn run() -> Result<(), String> {
    let args: Vec<_> = std::env::args_os().collect();
    if args.len() != 3 || args[1] != "--handoff" {
        return Err("Native launcher requires a host-reviewed handoff receipt.".into());
    }
    let receipt = std::path::PathBuf::from(&args[2]);
    let mut job: Handoff = json(&receipt)?;
    job.validate()?;
    let folder = job
        .profile
        .join("native-updates")
        .join(format!("job-{}", job.id));
    if receipt != folder.join("handoff.json") || job.stage != "waiting" || job.helper.is_some() {
        return Err("Native handoff changed or already started. Inspect it; do not replay.".into());
    }
    let own = std::env::current_exe().map_err(|_| "Launcher identity unavailable.")?;
    let expected = job
        .build
        .previous
        .as_ref()
        .unwrap()
        .files
        .iter()
        .find(|f| f.path == LAUNCHER)
        .ok_or("Protected launcher missing.")?;
    if file_hash(&own)? != expected.id {
        return Err("Protected launcher changed; native update refused.".into());
    }
    let gate = folder.join("launcher.lock");
    if gate.exists() {
        safe_path(&gate)?;
    }
    let lock = fs::OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(false)
        .open(gate)
        .map_err(|_| "Native launcher lock unavailable.")?;
    lock.try_lock()
        .map_err(|_| "A launcher already owns this intent.")?;
    job = claim_handoff(
        &receipt,
        &job,
        process::identity(std::process::id())?.ok_or("Launcher process unavailable.")?,
    )?;
    let outcome = (|| {
        verify(
            job.old
                .executable
                .parent()
                .ok_or("Old bundle unavailable.")?,
            job.previous_bundle(),
        )?;
        verify(&job.target(), job.target_bundle())?;
        verify(&job.previous(), job.previous_bundle())?;
        if process::identity(job.old.pid)?
            .as_ref()
            .is_some_and(|p| p != &job.old)
        {
            return Err("Old app identity changed; no process was stopped.".into());
        }
        wait_exit(&job.old)?;
        job.stage = "previousStopped".into();
        save(&receipt, &job)?;
        {
            let _profile = lock_profile(&job.profile)?;
            job.database_id = Some(database_identity(&job.profile.join("dolores.db"))?);
            job.backup_id = Some(snapshot_database(
                &job.profile,
                &folder.join("profile-before.sqlite"),
            )?);
        }
        job.stage = "snapshotSaved".into();
        save(&receipt, &job)?;
        let target = job.target();
        verify(&target, job.target_bundle())?;
        process::launch_recorded(&mut job, &receipt, &target)?;
        expect_health(&job, &folder)?;
        job.stage = if job.operation == "install" {
            "applied"
        } else {
            "restored"
        }
        .into();
        job.note="Healthy normal startup verified. History preserved; no task was replayed. Restore needs a fresh review.".into();
        save(&receipt, &job)
    })();
    if let Err(error) = outcome {
        job.note = error.clone();
        if job.child.is_some() && job.backup_id.is_some() {
            // UI remains blocked until the healthy receipt is acknowledged. No post-handoff user work exists here.
            let recovery = (|| {
                let failed = job.child.clone().unwrap();
                process::stop_failed_startup(&failed)?;
                wait_exit(&failed)?;
                restore_snapshot(&job, &folder)?;
                job.stage = "rollingBack".into();
                save(&receipt, &job)?;
                let old = job.previous();
                verify(&old, job.previous_bundle())?;
                // Retain the failed health receipt without allowing it to qualify the rollback.
                let health = folder.join("health.json");
                if health.exists() {
                    safe_path(&health)?;
                    fs::rename(&health, folder.join("failed-health.json"))
                        .map_err(|_| "Failed startup evidence could not be retained.")?;
                }
                job.expected_source = if job.operation == "install" {
                    job.build.source_id.clone()
                } else {
                    job.build.candidate_source_id.clone()
                };
                process::launch_recorded(&mut job, &receipt, &old)?;
                expect_health(&job, &folder)?;
                job.stage = "rolledBack".into();
                job.note = format!(
                    "{error} Previous version and pre-startup history restored; no task replay."
                );
                save(&receipt, &job)
            })();
            if let Err(recovery) = recovery {
                job.stage = "recoveryRequired".into();
                job.note=format!("{error} Recovery incomplete: {recovery} Both bundles and history backup remain; inspect before a separately reviewed Restore.");
                let _ = save(&receipt, &job);
                return Err(job.note);
            }
        } else {
            job.stage = if process::identity(job.old.pid)?.as_ref() == Some(&job.old) {
                "failed"
            } else {
                "recoveryRequired"
            }
            .into();
            let _ = save(&receipt, &job);
        }
        return Err(error);
    }
    Ok(())
}
