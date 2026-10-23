//! Retain exact bytes before removing only a reviewed, owned managed checkout.
use super::{
    worktree_git::{self, Git},
    worktree_identity::Pin,
    worktree_snapshot::{self, RawFile, Snapshot},
};
use dolores_core::{worktrees::*, SessionStore};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::{
    fs::File,
    io::{Read, Write},
    path::{Path, PathBuf},
    time::{Duration, Instant},
};
use tokio_util::sync::CancellationToken;

pub(super) const MANIFEST_LIMIT: usize = 16 * 1024 * 1024;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(super) struct Directory {
    pub path: String,
    pub readonly: bool,
    pub unix_mode: Option<u32>,
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(super) struct Manifest {
    pub version: u32,
    pub record: ManagedWorktree,
    pub parent_identity: String,
    pub directory_identity: String,
    pub snapshot: Snapshot,
    pub files: Vec<RawFile>,
    pub directories: Vec<Directory>,
}

pub(super) struct Review {
    pub token: String,
    pub owner: String,
    pub record: ManagedWorktree,
    pub snapshot: String,
    pub directory: PathBuf,
    pub restore: bool,
    pub created: Instant,
}

pub(super) fn save(path: &Path, value: &impl Serialize) -> Result<(), String> {
    let bytes = serde_json::to_vec(value).map_err(|_| "Archive evidence could not be encoded.")?;
    if bytes.len() > MANIFEST_LIMIT {
        return Err(
            "Archive evidence exceeds 16 MiB. Keep the checkout and narrow the inventory.".into(),
        );
    }
    let mut file=File::options().write(true).create_new(true).open(path).map_err(|_|"Archive evidence could not be created. Check disk space and inspect the retained folder before retrying.")?;
    file.write_all(&bytes).and_then(|_|file.sync_all()).map_err(|_|"Archive evidence could not be flushed. Retained data remains; no operation will be replayed.".into())
}

pub(super) fn pin_file(path: &Path, limit: usize) -> Result<(File, Vec<u8>), String> {
    super::worktree_identity::no_alias(path)?;
    let mut options = File::options();
    options.read(true);
    #[cfg(windows)]
    {
        use std::os::windows::fs::OpenOptionsExt;
        use windows_sys::Win32::Storage::FileSystem::*;
        options
            .share_mode(FILE_SHARE_READ)
            .custom_flags(FILE_FLAG_OPEN_REPARSE_POINT);
    }
    let file = options.open(path).map_err(|_| {
        "Retained archive file is unavailable or in use. Inspect the snapshot before retrying."
    })?;
    if !file
        .metadata()
        .map_err(|_| "Archive file metadata is unavailable.")?
        .is_file()
    {
        return Err("Archive evidence must be a regular file.".into());
    }
    let mut bytes = vec![];
    (&file)
        .take(limit as u64 + 1)
        .read_to_end(&mut bytes)
        .map_err(|_| "Archive bytes could not be read.")?;
    if bytes.len() > limit {
        return Err("Archive evidence exceeds its retained byte bound. Inspect it manually; no files were changed.".into());
    }
    Ok((file, bytes))
}

pub(super) fn safe_git(
    root: &Path,
    scratch: &Path,
    cancel: &CancellationToken,
) -> Result<Git, String> {
    let mut git = Git::new(root)?;
    git.materialization_options(scratch, cancel)?;
    Ok(git)
}

fn permissions(path: &Path) -> Result<(bool, Option<u32>), String> {
    let meta = std::fs::metadata(path).map_err(|_| "Archive entry metadata is unavailable.")?;
    let readonly = meta.permissions().readonly();
    #[cfg(unix)]
    let mode = {
        use std::os::unix::fs::PermissionsExt;
        Some(meta.permissions().mode())
    };
    #[cfg(not(unix))]
    let mode = None;
    Ok((readonly, mode))
}

/// Enumerate the actual directory, including ignored files and empty folders.
pub(super) fn inventory(
    root: &Path,
    cancel: &CancellationToken,
) -> Result<(Vec<String>, Vec<Directory>), String> {
    let _root = Pin::open(root)?;
    let mut pending = vec![String::new()];
    let mut files = vec![];
    let mut directories = vec![];
    let mut total = 0u64;
    let started = Instant::now();
    while let Some(relative) = pending.pop() {
        if cancel.is_cancelled() || started.elapsed() > Duration::from_secs(120) {
            return Err("Archive inspection stopped or reached two minutes. Checkout files remain; inspect before retrying.".into());
        }
        let folder = root.join(&relative);
        let _folder = Pin::open(&folder)?;
        for entry in std::fs::read_dir(&folder).map_err(|_| {
            "Archive folder could not be listed. Restore access; the checkout remains."
        })? {
            let entry = entry.map_err(|_| "Archive entry is unavailable.")?;
            let name = entry
                .file_name()
                .to_str()
                .ok_or("Archive paths must be Unicode. Back up this checkout externally.")?
                .to_string();
            if relative.is_empty() && name == ".git" {
                continue;
            }
            let path = if relative.is_empty() {
                name
            } else {
                format!("{relative}/{name}")
            };
            worktree_snapshot::path(&path)?;
            super::worktree_identity::no_alias(&entry.path())?;
            let meta = std::fs::symlink_metadata(entry.path())
                .map_err(|_| "Archive entry changed during inspection.")?;
            if meta.is_dir() {
                let (readonly, unix_mode) = permissions(&entry.path())?;
                directories.push(Directory {
                    path: path.clone(),
                    readonly,
                    unix_mode,
                });
                pending.push(path);
            } else if meta.is_file() {
                total = total
                    .checked_add(meta.len())
                    .ok_or("Archive byte count overflow.")?;
                if total > worktree_snapshot::MAX_BYTES as u64 {
                    return Err("Archive exceeds 64 MiB including ignored files. Keep the checkout and move disposable data or back it up externally before a new review.".into());
                }
                files.push(path);
            } else {
                return Err("Archive cannot remove a checkout containing special files, aliases or nested repositories. Keep it and inspect externally.".into());
            }
            if files.len() + directories.len() > 20000 {
                return Err("Archive exceeds 20,000 entries including ignored files. Keep the checkout and narrow it or back it up externally.".into());
            }
        }
    }
    files.sort();
    directories.sort_by(|a, b| a.path.cmp(&b.path));
    if cfg!(windows) {
        let mut unique = std::collections::BTreeSet::new();
        for path in files.iter().chain(directories.iter().map(|d| &d.path)) {
            if !unique.insert(path.to_lowercase()) {
                return Err("Archive contains paths differing only by case. Keep this checkout and back it up externally; restore would need case-sensitive directory support.".into());
            }
        }
    }
    Ok((files, directories))
}

pub(super) fn owned_target(record: &ManagedWorktree, parent: &Path) -> Result<Pin, String> {
    record.validate()?;
    let parent = Pin::open(parent)?;
    let expected = parent.path.join(&record.id);
    if super::source_control::path_key(&expected)
        != super::source_control::path_key(Path::new(&record.root))
    {
        return Err("Managed removal/restore target is outside its owned UUID folder. Inspect the saved record; no folder was removed.".into());
    }
    Ok(parent)
}

pub(super) fn archive_ref(record: &ManagedWorktree, snapshot: &str) -> String {
    format!("refs/dolores/worktrees/{}/archives/{snapshot}", record.id)
}

fn index_bytes(git: &Git, cancel: &CancellationToken) -> Result<Vec<u8>, String> {
    let path = git.line(
        &["rev-parse", "--path-format=absolute", "--git-path", "index"],
        cancel,
    )?;
    super::source_control::read_bounded(Path::new(&path), 16 * 1024 * 1024)
}

fn checked(output: super::git_process::Output) -> Result<String, String> {
    if output.code != 0 {
        return Err(format!(
            "Archive Git snapshot failed: {}. Checkout files remain.",
            String::from_utf8_lossy(&output.stderr)
                .chars()
                .take(1400)
                .collect::<String>()
        ));
    }
    String::from_utf8(output.stdout)
        .map(|s| s.trim().to_string())
        .map_err(|_| "Git snapshot identity is not Unicode.".into())
}

fn commit(
    git: &Git,
    tree: &str,
    parents: &[&str],
    cancel: &CancellationToken,
) -> Result<String, String> {
    let mut args = vec![
        "-c",
        "user.name=Dolores recovery",
        "-c",
        "user.email=recovery@dolores.invalid",
        "-c",
        "commit.gpgSign=false",
        "commit-tree",
        tree,
    ];
    for parent in parents {
        args.extend(["-p", parent]);
    }
    args.extend(["-m", "Dolores recoverable worktree snapshot"]);
    git.line(&args, cancel)
}

fn snapshot_objects(
    git: &Git,
    directory: &Path,
    manifest: &Manifest,
    cancel: &CancellationToken,
) -> Result<String, String> {
    let temporary = tempfile::tempdir_in(directory)
        .map_err(|_| "Snapshot index storage is unavailable. Check disk space.")?;
    let index = temporary.path().join("index");
    checked(git.with_index(&["read-tree", "--empty"], &index, cancel)?)?;
    let mut files = manifest
        .files
        .iter()
        .enumerate()
        .map(|(i, _)| format!("files/{i}.bin"))
        .collect::<Vec<_>>();
    files.extend(["index.bin".into(), "manifest.json".into()]);
    let started = Instant::now();
    for chunk in files.chunks(16) {
        if cancel.is_cancelled() || started.elapsed() > Duration::from_secs(120) {
            return Err("Saving archive objects stopped or reached two minutes. The checkout was not removed.".into());
        }
        let names = chunk
            .iter()
            .map(|s| {
                directory
                    .join(s)
                    .to_str()
                    .ok_or("Archive path is not Unicode.")
                    .map(str::to_string)
            })
            .collect::<Result<Vec<_>, _>>()?;
        let mut args = vec!["hash-object", "-w", "--no-filters", "--"];
        args.extend(names.iter().map(String::as_str));
        let ids = git
            .line(&args, cancel)?
            .lines()
            .map(str::to_string)
            .collect::<Vec<_>>();
        if ids.len() != chunk.len()
            || ids
                .iter()
                .any(|s| !matches!(s.len(), 40 | 64) || !s.bytes().all(|b| b.is_ascii_hexdigit()))
        {
            return Err(
                "Git returned invalid archive object identities. Checkout files remain.".into(),
            );
        }
        let mut update = vec!["update-index".to_string(), "--add".into()];
        for (name, id) in chunk.iter().zip(ids) {
            update.extend(["--cacheinfo".into(), format!("100644,{id},{name}")]);
        }
        checked(git.with_index(
            &update.iter().map(String::as_str).collect::<Vec<_>>(),
            &index,
            cancel,
        )?)?;
    }
    let tree = checked(git.with_index(&["write-tree"], &index, cancel)?)?;
    let working = commit(
        git,
        &manifest.snapshot.tree,
        &[&manifest.snapshot.head],
        cancel,
    )?;
    let staged = commit(
        git,
        &manifest.snapshot.index_tree,
        &[&manifest.snapshot.head],
        cancel,
    )?;
    commit(git, &tree, &[&working, &staged], cancel)
}

pub(super) fn review_archive(
    owner: &str,
    record: &ManagedWorktree,
    managed: &Path,
    archives: &Path,
    cancel: &CancellationToken,
) -> Result<(Review, Value), String> {
    let _pins = worktree_git::validate_ready(record)?;
    let parent = owned_target(record, managed)?;
    let shelf = archives.join(&record.id);
    if !shelf.exists() {
        std::fs::create_dir(&shelf).map_err(|_| {
            "Could not reserve archive storage. Check disk space; checkout remains."
        })?;
    }
    let _shelf = Pin::open(&shelf)?;
    if std::fs::read_dir(&shelf)
        .map_err(|_| "Archive inventory is unavailable.")?
        .take(129)
        .count()
        >= 128
    {
        return Err("128 retained archive snapshots or drafts need inspection. Check the archive folder before another review; no evidence was removed.".into());
    }
    let directory = shelf.join(format!("pending-{}", uuid::Uuid::new_v4()));
    std::fs::create_dir(&directory)
        .map_err(|_| "Could not reserve snapshot storage. Check disk space; checkout remains.")?;
    let directory_pin = Pin::open(&directory)?;
    let root = Path::new(&record.root);
    let git = safe_git(root, &directory, cancel)?;
    if !git
        .line(&["rev-parse", "--shared-index-path"], cancel)?
        .is_empty()
    {
        return Err("Split indexes need external backup or explicit conversion to a normal index before archive. Checkout and staging remain.".into());
    }
    let (paths, directories) = inventory(root, cancel)?;
    let snapshot = worktree_snapshot::capture(&git, root, &directory, cancel)?;
    let files = worktree_snapshot::retain_archive_raw(root, &paths, &directory.join("files"))?;
    let index = index_bytes(&git, cancel)?;
    let mut saved = File::options()
        .write(true)
        .create_new(true)
        .open(directory.join("index.bin"))
        .map_err(|_| "Could not retain archive index. Checkout remains.")?;
    saved
        .write_all(&index)
        .and_then(|_| saved.sync_all())
        .map_err(|_| "Could not flush retained archive index. Checkout remains.")?;
    drop(saved);
    let manifest = Manifest {
        version: 1,
        record: record.clone(),
        parent_identity: parent.identity,
        directory_identity: directory_pin.identity.clone(),
        snapshot,
        files,
        directories,
    };
    save(&directory.join("manifest.json"), &manifest)?;
    verify_source(record, &manifest, &git, &directory, cancel)?;
    let snapshot = snapshot_objects(&git, &directory, &manifest, cancel)?;
    git.command(
        &[
            "update-ref",
            &archive_ref(record, &snapshot),
            &snapshot,
            &"0".repeat(snapshot.len()),
        ],
        cancel,
    )?;
    let final_directory = shelf.join(&snapshot);
    drop(directory_pin);
    std::fs::rename(&directory,&final_directory).map_err(|_|"Archive objects were retained but evidence could not be finalized. Checkout remains; inspect the pending archive folder.")?;
    let token = uuid::Uuid::new_v4().to_string();
    let value = json!({"token":token,"kind":"archive","record":record,"snapshot":snapshot,"directory":final_directory,"files":manifest.files.len(),"directories":manifest.directories.len(),"bytes":manifest.files.iter().map(|f|f.bytes).sum::<usize>(),"notice":"All saved files, including ignored files, raw staging and empty folders are retained. Archive removes only this managed checkout; the original project and chat history remain. Setup will not run on restore."});
    Ok((
        Review {
            token,
            owner: owner.into(),
            record: record.clone(),
            snapshot,
            directory: final_directory,
            restore: false,
            created: Instant::now(),
        },
        value,
    ))
}

fn verify_source(
    record: &ManagedWorktree,
    manifest: &Manifest,
    git: &Git,
    directory: &Path,
    cancel: &CancellationToken,
) -> Result<(), String> {
    if worktree_snapshot::capture(git, Path::new(&record.root), directory, cancel)?
        != manifest.snapshot
    {
        return Err("Checkout or staging changed during archive review. Review again; no folder was removed.".into());
    }
    let (paths, dirs) = inventory(Path::new(&record.root), cancel)?;
    if paths
        != manifest
            .files
            .iter()
            .map(|f| f.path.clone())
            .collect::<Vec<_>>()
        || dirs != manifest.directories
    {
        return Err("Checkout inventory changed during archive review. Include the new work in a fresh review.".into());
    }
    worktree_snapshot::verify_raw(Path::new(&record.root), &manifest.files)?;
    for file in &manifest.files {
        let (readonly, mode) = permissions(&Path::new(&record.root).join(&file.path))?;
        if readonly != file.readonly || mode != file.unix_mode {
            return Err("Checkout permissions changed after archive review. Review again; no folder was removed.".into());
        }
    }
    Ok(())
}

pub(super) fn load(
    record: &ManagedWorktree,
    snapshot: &str,
    directory: &Path,
    cancel: &CancellationToken,
) -> Result<(Manifest, Vec<File>), String> {
    let common = Pin::matches(Path::new(&record.common_directory), &record.common_identity)?;
    let directory_pin = Pin::open(directory)?;
    let git = safe_git(&common.path, directory, cancel)?;
    if git.line(
        &["rev-parse", "--verify", &archive_ref(record, snapshot)],
        cancel,
    )? != snapshot
    {
        return Err("Retained archive ref changed. Keep both folders and inspect Git recovery refs before restoring.".into());
    }
    let (file, bytes) = pin_file(&directory.join("manifest.json"), MANIFEST_LIMIT)?;
    let manifest: Manifest = serde_json::from_slice(&bytes).map_err(|_| {
        "Archive manifest is malformed. Retained Git objects remain; inspect before restoring."
    })?;
    manifest.record.validate()?;
    if manifest.version != 1
        || manifest.record.id != record.id
        || manifest.record.root != record.root
        || manifest.record.common_identity != record.common_identity
        || directory_pin.identity != manifest.directory_identity
        || manifest.files.len() + manifest.directories.len() > 20000
    {
        return Err(
            "Archive identity changed. Inspect retained evidence; no checkout was restored.".into(),
        );
    }
    if git.line(
        &[
            "hash-object",
            "--no-filters",
            "--",
            directory
                .join("manifest.json")
                .to_str()
                .ok_or("Archive manifest path is not Unicode.")?,
        ],
        cancel,
    )? != git.line(&["rev-parse", &format!("{snapshot}:manifest.json")], cancel)?
    {
        return Err(
            "Archive manifest changed after retention. Inspect the owned Git ref before restoring."
                .into(),
        );
    }
    let mut pins = vec![file];
    let mut total = 0usize;
    let mut seen = std::collections::BTreeSet::new();
    for (i, entry) in manifest.files.iter().enumerate() {
        worktree_snapshot::path(&entry.path)?;
        if !entry.exists || !seen.insert(&entry.path) {
            return Err(
                "Archive file inventory is malformed. Inspect its retained versions.".into(),
            );
        }
        let (pin, bytes) = pin_file(
            &directory.join(format!("files/{i}.bin")),
            worktree_snapshot::MAX_BYTES,
        )?;
        total = total
            .checked_add(bytes.len())
            .ok_or("Archive byte count overflow.")?;
        if total > worktree_snapshot::MAX_BYTES
            || bytes.len() != entry.bytes
            || format!("{:x}", Sha256::digest(&bytes)) != entry.digest
        {
            return Err("Retained archive bytes changed. Inspect Git snapshot and raw copies; no folder was removed or restored.".into());
        }
        pins.push(pin);
    }
    let (pin, index) = pin_file(&directory.join("index.bin"), 16 * 1024 * 1024)?;
    if format!("{:x}", Sha256::digest(&index)) != manifest.snapshot.index_digest {
        return Err("Retained staging bytes changed. Inspect archive before restoring.".into());
    }
    pins.push(pin);
    for d in &manifest.directories {
        worktree_snapshot::path(&d.path)?;
    }
    Ok((manifest, pins))
}

pub(super) fn save_state(
    store: &dyn SessionStore,
    record: &ManagedWorktree,
    state: WorktreeState,
    notice: &str,
) -> Result<ManagedWorktree, String> {
    let mut next = record.clone();
    next.revision = next
        .revision
        .checked_add(1)
        .ok_or("Worktree revision exhausted.")?;
    next.state = state;
    if state != WorktreeState::Recovery {
        next.recovery_from = None;
    }
    next.notice = notice.into();
    store.save_worktree(&next)?;
    Ok(next)
}

pub(super) fn recover(store: &dyn SessionStore, record: &ManagedWorktree, error: &str) -> String {
    match record.interrupted() {
        Ok(mut saved) => {
            saved.notice=format!("{} Retained snapshot: {}. Inspect the worktree and archive before an explicit recovery action; nothing will be replayed.",error.chars().take(1600).collect::<String>(),record.snapshot.as_deref().unwrap_or("not finalized"));
            match store.save_worktree(&saved) {Ok(())=>saved.notice,Err(failure)=>format!("{error} Recovery state could not be saved: {failure}. Saved intent and archive remain; inspect before retrying.")}
        }
        Err(failure) => format!("{error} Recovery state unavailable: {failure}"),
    }
}

pub(super) fn apply_archive(
    store: &dyn SessionStore,
    review: Review,
    managed: &Path,
    cancel: &CancellationToken,
) -> Result<Value, String> {
    if review.restore || review.created.elapsed() > Duration::from_secs(600) {
        return Err("Archive review expired. Review again; the checkout remains.".into());
    }
    let record = store
        .worktree(&review.record.session)?
        .ok_or("Managed checkout evidence is unavailable.")?;
    if record != review.record {
        return Err(
            "Worktree state changed after review. Inspect and review archive again.".into(),
        );
    }
    let pins = worktree_git::validate_ready(&record)?;
    let parent = owned_target(&record, managed)?;
    let (manifest, _backups) = load(&record, &review.snapshot, &review.directory, cancel)?;
    if manifest.parent_identity != parent.identity {
        return Err("Managed worktree storage was replaced. Keep both folders and inspect recovery details.".into());
    }
    let git = safe_git(Path::new(&record.root), &review.directory, cancel)?;
    verify_source(&record, &manifest, &git, &review.directory, cancel)?;
    let mut intent = record.clone();
    intent.snapshot = Some(review.snapshot.clone());
    intent.revision = intent
        .revision
        .checked_add(1)
        .ok_or("Worktree revision exhausted.")?;
    intent.state = WorktreeState::Archiving;
    intent.notice="Archive removal intent saved; all file/index bytes and Git snapshot are retained. Inspect after interruption; never repeat removal automatically.".into();
    if cancel.is_cancelled() {
        return Err("Archive stopped before removal. Checkout remains.".into());
    }
    store.save_worktree(&intent)?;
    // Git needs delete access to the verified target; retain the parent and common
    // identity, release only Ready pins immediately before this explicit removal.
    drop(pins);
    let common = Pin::matches(Path::new(&record.common_directory), &record.common_identity)?;
    let git = safe_git(&common.path, &review.directory, cancel)?;
    let result = (|| {
        let last = Pin::matches(
            Path::new(&record.root),
            &record
                .identity
                .as_ref()
                .ok_or("Checkout identity unavailable.")?
                .filesystem,
        )?;
        drop(last);
        let lock = Path::new(
            &record
                .identity
                .as_ref()
                .ok_or("Checkout identity unavailable.")?
                .git_directory,
        )
        .join("locked");
        if lock.exists() {
            let reason = super::source_control::read_bounded(&lock, 8192)?;
            if String::from_utf8_lossy(&reason).trim() != "Dolores managed checkout" {
                return Err("Checkout has a different Git lock reason. Keep it and inspect that lock before a fresh archive review.".into());
            }
            git.command(&["worktree", "unlock", "--", &record.root], cancel)?;
        }
        git.command(
            &["worktree", "remove", "--force", "--", &record.root],
            cancel,
        )?;
        if Path::new(&record.root).exists() {
            return Err("Archive did not remove the owned checkout. Retained snapshot remains; inspect it before retrying.".into());
        }
        save_state(store,&intent,WorktreeState::Archived,"Archived with exact file/index bytes, ignored files and empty folders retained. Restore is explicit and never runs setup.")
    })();
    match result {
        Ok(record) => {
            Ok(json!({"record":record,"directory":review.directory,"snapshot":review.snapshot}))
        }
        Err(error) => Err(recover(store, &intent, &error)),
    }
}

pub(super) fn directory(archives: &Path, record: &ManagedWorktree) -> Result<PathBuf, String> {
    record.validate()?;
    Ok(archives
        .join(&record.id)
        .join(record.snapshot.as_deref().ok_or(
            "No retained archive snapshot is available. Keep the checkout and inspect its details.",
        )?))
}

pub(super) fn review_restore(
    owner: &str,
    record: &ManagedWorktree,
    managed: &Path,
    archives: &Path,
    cancel: &CancellationToken,
) -> Result<(Review, Value), String> {
    if record.state != WorktreeState::Archived
        && !(record.state == WorktreeState::Recovery
            && record.recovery_from == Some(WorktreeState::Restoring))
    {
        return Err("Inspect interrupted lifecycle state before reviewing restore.".into());
    }
    let parent = owned_target(record, managed)?;
    let directory = directory(archives, record)?;
    let snapshot = record
        .snapshot
        .as_deref()
        .ok_or("Archive snapshot is unavailable.")?;
    let (manifest, _pins) = load(record, snapshot, &directory, cancel)?;
    if parent.identity != manifest.parent_identity {
        return Err(
            "Managed storage was replaced. Inspect the original parent before restoring.".into(),
        );
    }
    if record.state == WorktreeState::Archived && Path::new(&record.root).exists() {
        return Err("Restore destination already exists. Keep its files and move or inspect them before reviewing again; nothing was overwritten.".into());
    }
    let token = uuid::Uuid::new_v4().to_string();
    let value = json!({"token":token,"kind":"restore","record":record,"snapshot":snapshot,"directory":directory,"files":manifest.files.len(),"directories":manifest.directories.len(),"bytes":manifest.files.iter().map(|f|f.bytes).sum::<usize>(),"notice":if record.state==WorktreeState::Archived {"Restore the retained checkout at its original path, preserving staged/unstaged and ignored files. Setup will stay skipped."}else{"Continue only validated restore steps. Completed bytes are skipped; unrelated bytes refuse. Setup will not be repeated."}});
    Ok((
        Review {
            token,
            owner: owner.into(),
            record: record.clone(),
            snapshot: snapshot.into(),
            directory,
            restore: true,
            created: Instant::now(),
        },
        value,
    ))
}
