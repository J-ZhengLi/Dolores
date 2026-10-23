//! Resume only byte-checked restore steps. Restart itself never executes them.
use super::{
    worktree_archive::{self as archive, Manifest, Review},
    worktree_git,
    worktree_identity::Pin,
    worktree_snapshot::{self, RawFile},
};
use dolores_core::{worktrees::*, SessionStore};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::{
    collections::BTreeMap,
    fs::File,
    io::{Read, Write},
    path::{Path, PathBuf},
    time::Duration,
};
use tokio_util::sync::CancellationToken;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Bytes {
    digest: String,
    length: usize,
    readonly: bool,
    unix_mode: Option<u32>,
}
#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Step {
    path: String,
    before: Option<Bytes>,
    before_directory: bool,
    after: Option<Bytes>,
    backup: Option<usize>,
}
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Plan {
    snapshot: String,
    root_identity: String,
    steps: Vec<Step>,
    extra_directories: Vec<String>,
    index_before: Option<Bytes>,
}
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Ownership {
    snapshot: String,
    root: String,
    identity: String,
    start_revision: u32,
}

fn bytes(path: &Path) -> Result<Option<Bytes>, String> {
    match std::fs::symlink_metadata(path) {
        Err(error) if matches!(error.kind(),std::io::ErrorKind::NotFound|std::io::ErrorKind::NotADirectory)=>return Ok(None),
        Err(_)=>return Err("Restore path is unavailable. Keep current files and inspect the retained archive.".into()),
        Ok(meta) if !meta.is_file()=>return Err("Restore path became a directory or alias. Keep both versions and inspect before continuing.".into()),
        Ok(_)=>(),
    }
    super::worktree_identity::no_alias(path)?;
    let file = File::open(path)
        .map_err(|_| "Restore file is unavailable. Restore access before continuing.")?;
    let meta = file
        .metadata()
        .map_err(|_| "Restore file metadata is unavailable.")?;
    let mut data = vec![];
    file.take(worktree_snapshot::MAX_BYTES as u64 + 1)
        .read_to_end(&mut data)
        .map_err(|_| "Restore file could not be read.")?;
    if data.len() > worktree_snapshot::MAX_BYTES {
        return Err("Changed restore file exceeds 64 MiB. Keep it and inspect manually; it was not overwritten.".into());
    }
    #[cfg(unix)]
    let unix_mode = {
        use std::os::unix::fs::PermissionsExt;
        Some(meta.permissions().mode())
    };
    #[cfg(not(unix))]
    let unix_mode = None;
    Ok(Some(Bytes {
        digest: format!("{:x}", Sha256::digest(&data)),
        length: data.len(),
        readonly: meta.permissions().readonly(),
        unix_mode,
    }))
}

fn after(file: &RawFile) -> Bytes {
    Bytes {
        digest: file.digest.clone(),
        length: file.bytes,
        readonly: file.readonly,
        unix_mode: file.unix_mode,
    }
}
fn same_bytes(a: &Option<Bytes>, b: &Option<Bytes>) -> bool {
    match (a, b) {
        (None, None) => true,
        (Some(a), Some(b)) => a.digest == b.digest && a.length == b.length,
        _ => false,
    }
}
fn permissions(path: &Path, value: &Bytes) -> Result<(), String> {
    let mut permissions = std::fs::metadata(path)
        .map_err(|_| "Restored file metadata is unavailable.")?
        .permissions();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        if let Some(mode) = value.unix_mode {
            permissions.set_mode(mode);
        }
    }
    #[cfg(windows)]
    permissions.set_readonly(value.readonly);
    std::fs::set_permissions(path,permissions).map_err(|_|"File bytes were restored but permissions could not be set. Inspect and explicitly continue; retained bytes remain.".into())
}

fn ensure_directory(root: &Path, relative: &str) -> Result<(), String> {
    worktree_snapshot::path(relative)?;
    let mut current = root.to_path_buf();
    for part in relative.split('/') {
        let _parent = Pin::open(&current)?;
        current.push(part);
        match std::fs::symlink_metadata(&current) {
            Ok(_) => {
                Pin::open(&current)?;
            }
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                std::fs::create_dir(&current).map_err(|_|"Could not recreate retained directories. Check disk space/access and explicitly continue.")?;
                Pin::open(&current)?;
            }
            Err(_) => {
                return Err(
                    "Restore directory is unavailable. Keep it and inspect before continuing."
                        .into(),
                )
            }
        }
    }
    Ok(())
}

fn load<T: serde::de::DeserializeOwned>(path: &Path) -> Result<T, String> {
    let (_pin, data) = archive::pin_file(path, archive::MANIFEST_LIMIT)?;
    serde_json::from_slice(&data).map_err(|_|"Restore evidence is malformed. Inspect the retained plan and files; nothing was repeated.".into())
}

fn ready_identity(
    record: &ManagedWorktree,
    manifest: &Manifest,
    cancel: &CancellationToken,
) -> Result<(WorktreeIdentity, worktree_git::ReadyPins), String> {
    let root = Pin::open(Path::new(&record.root))?;
    let git = archive::safe_git(&root.path, Path::new(&record.common_directory), cancel)?;
    if git.line(&["rev-parse", "--verify", "HEAD"], cancel)? != manifest.snapshot.head {
        return Err("Restoring checkout commit changed. Keep current work and inspect the retained snapshot; no reset was performed.".into());
    }
    let metadata = Pin::open(Path::new(
        &git.line(&["rev-parse", "--absolute-git-dir"], cancel)?,
    ))?;
    let common = Pin::matches(Path::new(&record.common_directory), &record.common_identity)?;
    if !metadata.path.starts_with(common.path.join("worktrees")) {
        return Err("Restore Git registration is outside the owned repository. Keep both folders and inspect.".into());
    }
    let identity = WorktreeIdentity {
        filesystem: root.identity,
        git_directory: metadata.text()?,
        git_identity: metadata.identity.clone(),
    };
    let mut checked = record.clone();
    checked.state = WorktreeState::Ready;
    checked.recovery_from = None;
    checked.identity = Some(identity.clone());
    let pins = worktree_git::validate_ready(&checked)?;
    Ok((identity, pins))
}

fn make_plan(
    record: &ManagedWorktree,
    manifest: &Manifest,
    directory: &Path,
    identity: &str,
    cancel: &CancellationToken,
) -> Result<Plan, String> {
    let root = Path::new(&record.root);
    let git = archive::safe_git(root, directory, cancel)?;
    if !git
        .command(
            &["status", "--porcelain=v1", "-z", "--untracked-files=all"],
            cancel,
        )?
        .is_empty()
        || !git
            .command(
                &[
                    "ls-files",
                    "--others",
                    "--ignored",
                    "--exclude-standard",
                    "-z",
                ],
                cancel,
            )?
            .is_empty()
    {
        return Err("Restore has no saved publication plan and its initial checkout changed. Keep its work and archive; inspect manually before continuing.".into());
    }
    let (initial, directories) = archive::inventory(root, cancel)?;
    let original = manifest
        .files
        .iter()
        .enumerate()
        .map(|(i, f)| (f.path.clone(), (i, f)))
        .collect::<BTreeMap<_, _>>();
    let paths = initial
        .into_iter()
        .chain(original.keys().cloned())
        .collect::<std::collections::BTreeSet<_>>();
    let mut steps = vec![];
    for path in paths {
        let before_directory = root.join(&path).is_dir();
        if before_directory {
            Pin::open(&root.join(&path))?;
        }
        let before = if before_directory {
            None
        } else {
            bytes(&root.join(&path))?
        };
        let (after, backup) = match original.get(&path) {
            Some((i, f)) => (Some(after(f)), Some(*i)),
            None => (None, None),
        };
        steps.push(Step {
            path,
            before,
            before_directory,
            after,
            backup,
        });
    }
    let git_directory = git.line(&["rev-parse", "--absolute-git-dir"], cancel)?;
    let mut extra_directories = directories
        .into_iter()
        .filter(|d| {
            !manifest
                .directories
                .iter()
                .any(|wanted| wanted.path == d.path)
        })
        .map(|d| d.path)
        .collect::<Vec<_>>();
    extra_directories.sort_by_key(|p| std::cmp::Reverse(p.split('/').count()));
    Ok(Plan {
        snapshot: record.snapshot.clone().ok_or("Restore snapshot missing.")?,
        root_identity: identity.into(),
        steps,
        extra_directories,
        index_before: bytes(&Path::new(&git_directory).join("index"))?,
    })
}

struct Staged {
    path: PathBuf,
}
impl Drop for Staged {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.path);
    }
}

fn publish(
    target: &Path,
    before: &Option<Bytes>,
    after: &Option<Bytes>,
    backup: Option<&Path>,
    cancel: &CancellationToken,
) -> Result<(), String> {
    if cancel.is_cancelled() {
        return Err("Restore stopped. Completed files and retained snapshot remain; inspect before an explicit continuation.".into());
    }
    let current = bytes(target)?;
    if same_bytes(&current, after) {
        if let Some(after) = after {
            permissions(target, after)?;
        }
        return Ok(());
    }
    if &current != before {
        return Err(format!("Restore target changed: {}. Keep both versions and inspect the retained snapshot; it was not overwritten.",target.display()));
    }
    let parent = Pin::open(target.parent().ok_or("Restore path has no parent.")?)?;
    let staged = if let Some(after) = after {
        let backup = backup.ok_or("Restore bytes are not retained.")?;
        let (_pin, data) = archive::pin_file(backup, worktree_snapshot::MAX_BYTES)?;
        if data.len() != after.length || format!("{:x}", Sha256::digest(&data)) != after.digest {
            return Err(
                "Retained restore bytes changed. Inspect the owned archive before continuing."
                    .into(),
            );
        }
        let temporary = parent
            .path
            .join(format!(".dolores-restore-{}", uuid::Uuid::new_v4()));
        let mut file=File::options().write(true).create_new(true).open(&temporary).map_err(|_|"Could not stage restore bytes. Check disk space/access, then explicitly continue; existing files remain.")?;
        let staged = Staged { path: temporary };
        file.write_all(&data).and_then(|_|file.sync_all()).map_err(|_|"Could not flush restore bytes. Check disk space, inspect retained work and explicitly continue.".to_string())?;
        drop(file);
        Some(staged)
    } else {
        None
    };
    if cancel.is_cancelled() || bytes(target)? != *before {
        return Err("Restore target changed or Stop was requested before publication. Inspect both versions before continuing.".into());
    }
    if let Some(staged) = &staged {
        std::fs::rename(&staged.path,target).map_err(|_|"Could not publish restored bytes. Existing file and archive remain; inspect access/space, then explicitly continue.".to_string())?;
        permissions(
            target,
            after.as_ref().ok_or("Restore permissions missing.")?,
        )?;
    } else {
        std::fs::remove_file(target).map_err(|_|"Could not remove an initial checkout file during restore. Inspect access and explicitly continue; archive remains.")?;
    }
    Ok(())
}

pub(super) fn apply(
    store: &dyn SessionStore,
    review: Review,
    managed: &Path,
    cancel: &CancellationToken,
) -> Result<Value, String> {
    if !review.restore || review.created.elapsed() > Duration::from_secs(600) {
        return Err("Restore review expired. Inspect and review again.".into());
    }
    let record = store
        .worktree(&review.record.session)?
        .ok_or("Worktree lifecycle evidence unavailable.")?;
    if record != review.record {
        return Err("Restore lifecycle changed after review. Inspect and review again; no file was changed.".into());
    }
    let parent = archive::owned_target(&record, managed)?;
    let (manifest, _backups) = archive::load(&record, &review.snapshot, &review.directory, cancel)?;
    if parent.identity != manifest.parent_identity {
        return Err("Restore parent folder changed. Inspect it before continuing.".into());
    }
    if record.state == WorktreeState::Archived && Path::new(&record.root).exists() {
        return Err("Restore destination became occupied. Keep both folders and review again; nothing was overwritten.".into());
    }
    if cancel.is_cancelled() {
        return Err("Restore stopped before its intent. Snapshot remains archived.".into());
    }
    let intent=archive::save_state(store,&record,WorktreeState::Restoring,"Restore intent saved. Setup stays skipped; inspect before explicitly resuming any interruption.")?;
    match restore_files(store, &intent, &manifest, &review.directory, cancel) {
        Ok(record) => {
            Ok(json!({"record":record,"directory":review.directory,"snapshot":review.snapshot}))
        }
        Err(error) => Err(archive::recover(store, &intent, &error)),
    }
}

fn restore_files(
    store: &dyn SessionStore,
    record: &ManagedWorktree,
    manifest: &Manifest,
    directory: &Path,
    cancel: &CancellationToken,
) -> Result<ManagedWorktree, String> {
    let root = Path::new(&record.root);
    let ownership_path = directory.join("restore-owner.json");
    let ownership: Ownership = if ownership_path.exists() {
        let proof: Ownership = load(&ownership_path)?;
        if proof.snapshot
            != record
                .snapshot
                .as_deref()
                .ok_or("Restore snapshot missing.")?
            || proof.root != record.root
            || proof.start_revision > record.revision
        {
            return Err(
                "Restore ownership evidence changed. Keep current work and inspect manually."
                    .into(),
            );
        }
        Pin::matches(root, &proof.identity)?;
        proof
    } else {
        std::fs::create_dir(root).map_err(|_|"Could not reserve restore destination. Check disk space/access and retain any existing folder; it was not overwritten.")?;
        let root = Pin::open(root)?;
        let proof = Ownership {
            snapshot: record.snapshot.clone().ok_or("Restore snapshot missing.")?,
            root: record.root.clone(),
            identity: root.identity.clone(),
            start_revision: record.revision,
        };
        archive::save(&ownership_path, &proof)?;
        proof
    };
    let root_pin = Pin::matches(root, &ownership.identity)?;
    let common = Pin::matches(Path::new(&record.common_directory), &record.common_identity)?;
    let git = archive::safe_git(&common.path, directory, cancel)?;
    if !root.join(".git").exists() {
        if std::fs::read_dir(root)
            .map_err(|_| "Restore folder is unavailable.")?
            .next()
            .is_some()
        {
            return Err("Reserved restore folder has unrecognized files before Git setup. Keep them and inspect manually; no files were reset.".into());
        }
        git.command(
            &[
                "worktree",
                "add",
                "--detach",
                "--lock",
                "--reason",
                "Dolores managed checkout",
                "--",
                &record.root,
                &manifest.snapshot.head,
            ],
            cancel,
        )?;
    }
    let (identity, _ready_pins) = ready_identity(record, manifest, cancel)?;
    if identity.filesystem != ownership.identity {
        return Err(
            "Restore folder was replaced. Keep both folders and inspect the retained archive."
                .into(),
        );
    }
    let plan_path = directory.join("restore-plan.json");
    let plan: Plan = if plan_path.exists() {
        load(&plan_path)?
    } else {
        let plan = make_plan(record, manifest, directory, &root_pin.identity, cancel)?;
        archive::save(&plan_path, &plan)?;
        archive::save(
            &directory.join("restore-plan-digest.json"),
            &format!(
                "{:x}",
                Sha256::digest(
                    serde_json::to_vec(&plan).map_err(|_| "Restore plan could not be encoded.")?
                )
            ),
        )?;
        plan
    };
    let expected: String = load(&directory.join("restore-plan-digest.json"))?;
    if plan.snapshot
        != record
            .snapshot
            .as_deref()
            .ok_or("Restore snapshot missing.")?
        || plan.root_identity != root_pin.identity
        || format!(
            "{:x}",
            Sha256::digest(serde_json::to_vec(&plan).map_err(|_| "Restore plan invalid.")?)
        ) != expected
    {
        return Err(
            "Restore plan changed. Inspect its retained versions before continuing.".into(),
        );
    }
    if plan.steps.len() > 40000 {
        return Err(
            "Restore plan exceeds its entry bound. Inspect manually; no files were reset.".into(),
        );
    }
    // Clear only validated base files absent from the archive before making
    // retained directories. This handles a saved file-to-directory transition.
    for step in plan
        .steps
        .iter()
        .filter(|s| s.after.is_none() && !s.before_directory)
    {
        worktree_snapshot::path(&step.path)?;
        let path = root.join(&step.path);
        if !path.is_dir() {
            publish(&path, &step.before, &None, None, cancel)?;
        }
    }
    // Conversely, a base directory can become a retained file. Its children must
    // already have been validated/removed, and remove_dir refuses foreign work.
    for path in &plan.extra_directories {
        worktree_snapshot::path(path)?;
        let path = root.join(path);
        if path.is_dir() {
            let owned = Pin::open(&path)?;
            drop(owned);
            std::fs::remove_dir(&path).map_err(|_|"Initial directory is no longer empty or removable. Keep its contents and inspect before continuing.")?;
        }
    }
    for d in &manifest.directories {
        worktree_snapshot::path(&d.path)?;
        let path = root.join(&d.path);
        if !path.exists() {
            ensure_directory(root, &d.path)?;
        }
        Pin::open(&path)?;
    }
    let mut seen = std::collections::BTreeSet::new();
    for step in &plan.steps {
        worktree_snapshot::path(&step.path)?;
        if !seen.insert(&step.path) {
            return Err("Restore plan contains duplicate paths. Inspect manually.".into());
        }
        let backup = if let Some(i) = step.backup {
            let file = manifest
                .files
                .get(i)
                .ok_or("Restore plan backup is unavailable.")?;
            if file.path != step.path || step.after != Some(after(file)) {
                return Err(
                    "Restore plan differs from the retained snapshot. Inspect manually.".into(),
                );
            }
            Some(directory.join(format!("files/{i}.bin")))
        } else {
            if step.after.is_some() {
                return Err("Restore plan has unretained bytes. Inspect manually.".into());
            }
            None
        };
        if step.after.is_none()
            && root.join(&step.path).is_dir()
            && manifest.directories.iter().any(|d| d.path == step.path)
        {
            continue;
        }
        publish(
            &root.join(&step.path),
            &step.before,
            &step.after,
            backup.as_deref(),
            cancel,
        )?;
    }
    let index = Path::new(&identity.git_directory).join("index");
    let (_, index_bytes) = archive::pin_file(&directory.join("index.bin"), 16 * 1024 * 1024)?;
    let index_after = Some(Bytes {
        digest: manifest.snapshot.index_digest.clone(),
        length: index_bytes.len(),
        readonly: false,
        unix_mode: None,
    });
    publish(
        &index,
        &plan.index_before,
        &index_after,
        Some(&directory.join("index.bin")),
        cancel,
    )?;
    for d in manifest.directories.iter().rev() {
        permissions(
            &root.join(&d.path),
            &Bytes {
                digest: String::new(),
                length: 0,
                readonly: d.readonly,
                unix_mode: d.unix_mode,
            },
        )?;
    }
    verify(record, manifest, cancel)?;
    let mut ready = record.clone();
    ready.revision = ready
        .revision
        .checked_add(1)
        .ok_or("Worktree revision exhausted.")?;
    ready.state = WorktreeState::Ready;
    ready.recovery_from = None;
    ready.identity = Some(identity);
    ready.notice="Restored exact saved files, original staging, ignored files and empty folders. Setup was skipped. Inspect changes before continuing the task.".into();
    store.save_worktree(&ready)?;
    Ok(ready)
}

pub(super) fn verify(
    record: &ManagedWorktree,
    manifest: &Manifest,
    cancel: &CancellationToken,
) -> Result<(), String> {
    let root = Path::new(&record.root);
    let (paths, dirs) = archive::inventory(root, cancel)?;
    if paths
        != manifest
            .files
            .iter()
            .map(|f| f.path.clone())
            .collect::<Vec<_>>()
        || dirs != manifest.directories
    {
        return Err("Restored inventory differs from the retained archive. Keep current work and inspect both versions.".into());
    }
    worktree_snapshot::verify_raw(root, &manifest.files)?;
    for f in &manifest.files {
        if bytes(&root.join(&f.path))? != Some(after(f)) {
            return Err(
                "Restored file bytes or permissions differ. Inspect and explicitly continue."
                    .into(),
            );
        }
    }
    let (identity, _pins) = ready_identity(record, manifest, cancel)?;
    let index = bytes(&Path::new(&identity.git_directory).join("index"))?
        .ok_or("Restored index missing.")?;
    if index.digest != manifest.snapshot.index_digest {
        return Err("Restored staging differs from the retained index. Keep both versions and inspect manually.".into());
    }
    Ok(())
}

pub(super) fn inspect(
    store: &dyn SessionStore,
    session: &str,
    revision: u32,
    managed: &Path,
    archives: &Path,
    cancel: &CancellationToken,
) -> Result<Value, String> {
    let record = store
        .worktree(session)?
        .ok_or("Worktree recovery evidence is unavailable.")?;
    if record.revision != revision || record.state != WorktreeState::Recovery {
        return Err(
            "Worktree recovery changed. Refresh before inspecting; no operation was repeated."
                .into(),
        );
    }
    let _parent = archive::owned_target(&record, managed)?;
    let directory = archive::directory(archives, &record)?;
    let (manifest, _backups) = archive::load(
        &record,
        record.snapshot.as_deref().ok_or("Snapshot missing.")?,
        &directory,
        cancel,
    )?;
    match record.recovery_from {
        Some(WorktreeState::Archiving) => {
            if !Path::new(&record.root).exists()
                && record
                    .identity
                    .as_ref()
                    .is_some_and(|i| !Path::new(&i.git_directory).exists())
            {
                let saved=archive::save_state(store,&record,WorktreeState::Archived,"Archive removal verified after interruption. Snapshot retained; Restore is explicit.")?;
                return Ok(json!({"record":saved,"directory":directory}));
            }
            let mut checked = record.clone();
            checked.state = WorktreeState::Ready;
            checked.recovery_from = None;
            worktree_git::validate_ready(&checked)?;
            let saved=archive::save_state(store,&record,WorktreeState::Ready,"Owned checkout still exists; its current work and retained archive are kept. Removal was not repeated. Review again to archive current work.")?;
            Ok(json!({"record":saved,"directory":directory}))
        }
        Some(WorktreeState::Restoring) => {
            if !directory.join("restore-owner.json").exists() {
                if !Path::new(&record.root).exists() {
                    let saved=archive::save_state(store,&record,WorktreeState::Archived,"Restore stopped before reserving its target. Snapshot remains archived; review restore again.")?;
                    return Ok(json!({"record":saved,"directory":directory}));
                }
                return Err("Restore folder exists without saved ownership. Keep it and inspect the target/retained archive manually; it will not be adopted or removed.".into());
            }
            let proof: Ownership = load(&directory.join("restore-owner.json"))?;
            Pin::matches(Path::new(&record.root), &proof.identity)?;
            if verify(&record, &manifest, cancel).is_ok() {
                let (identity, _pins) = ready_identity(&record, &manifest, cancel)?;
                let mut saved = record.clone();
                saved.state = WorktreeState::Ready;
                saved.recovery_from = None;
                saved.revision += 1;
                saved.identity = Some(identity);
                saved.notice="Completed restore inspected and reattached. No file copy, Git checkout or setup was repeated.".into();
                store.save_worktree(&saved)?;
                Ok(json!({"record":saved,"directory":directory}))
            } else {
                Ok(
                    json!({"record":record,"directory":directory,"notice":"Restore is incomplete. Review an explicit continuation; matching completed bytes will be skipped and unrelated bytes will refuse. Inspect retained originals before any manual change."}),
                )
            }
        }
        _ => Err(
            "This lifecycle needs creation recovery or manual inspection. No file was changed."
                .into(),
        ),
    }
}
