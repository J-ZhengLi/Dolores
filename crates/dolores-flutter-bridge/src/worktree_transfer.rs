//! Explicit, single-use worktree transfers with durable local recovery artifacts.
//! Neither listing nor restart applies a saved intent.
use super::{
    worktree_git::{self, Git},
    worktree_identity::Pin,
    worktree_snapshot::{self, RawFile, Snapshot},
};
use dolores_core::{worktrees::ManagedWorktree, SessionStore};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::{
    fs::File,
    io::Write,
    path::{Path, PathBuf},
    time::{Duration, Instant},
};
use tokio_util::sync::CancellationToken;

#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(super) struct Receipt {
    pub id: String,
    pub source_session: String,
    pub destination_session: String,
    pub source: ManagedWorktree,
    pub destination_root: String,
    pub destination_identity: String,
    pub directory_identity: String,
    pub source_snapshot: Snapshot,
    pub destination_snapshot: Snapshot,
    pub paths: Vec<String>,
    pub source_files: Vec<RawFile>,
    pub destination_files: Vec<RawFile>,
    pub patch_digest: String,
    pub applicable: bool,
    pub notice: String,
}

pub(super) struct Review {
    pub token: String,
    pub receipt: Receipt,
    pub directory: PathBuf,
    pub created: Instant,
}

fn save(path: &Path, value: &impl Serialize) -> Result<(), String> {
    let bytes = serde_json::to_vec(value).map_err(|_| "Transfer evidence could not be encoded.")?;
    let mut file=File::options().write(true).create_new(true).open(path).map_err(|_|"Transfer evidence could not be created. Check disk space and inspect retained artifacts before retrying.")?;
    file.write_all(&bytes)
        .and_then(|_| file.sync_all())
        .map_err(|_| {
            "Transfer evidence could not be flushed. Inspect retained artifacts before retrying."
                .into()
        })
}

fn pin_bytes(path: &Path, limit: usize) -> Result<(File, Vec<u8>), String> {
    use std::io::Read;
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
        "Retained transfer file is missing, changed or in use. Review transfer again."
    })?;
    if !file
        .metadata()
        .map_err(|_| "Retained transfer file metadata is unavailable.")?
        .is_file()
    {
        return Err("Retained transfer file must be a regular file.".into());
    }
    let mut bytes = vec![];
    (&file)
        .take(limit as u64 + 1)
        .read_to_end(&mut bytes)
        .map_err(|_| "Retained transfer file could not be read.")?;
    if bytes.len() > limit {
        return Err(format!(
            "Retained transfer file exceeds its {limit}-byte bound. Review a smaller selection."
        ));
    }
    Ok((file, bytes))
}

fn pin_backups(directory: &Path, receipt: &Receipt) -> Result<Vec<File>, String> {
    let mut pins = vec![];
    for (folder, manifest) in [
        ("source-files", &receipt.source_files),
        ("destination-files", &receipt.destination_files),
    ] {
        for (index, record) in manifest.iter().enumerate().filter(|(_, r)| r.exists) {
            let (file, bytes) = pin_bytes(
                &directory.join(folder).join(format!("{index}.bin")),
                worktree_snapshot::MAX_BYTES,
            )?;
            if bytes.len() != record.bytes
                || format!("{:x}", Sha256::digest(&bytes)) != record.digest
            {
                return Err("Retained backup changed. Review transfer again before applying; no working file was changed.".into());
            }
            pins.push(file);
        }
    }
    for (name, snapshot) in [
        ("source", &receipt.source_snapshot),
        ("destination", &receipt.destination_snapshot),
    ] {
        let (file, bytes) = pin_bytes(
            &directory.join(format!("{name}-index.bin")),
            16 * 1024 * 1024,
        )?;
        if format!("{:x}", Sha256::digest(&bytes)) != snapshot.index_digest {
            return Err(
                "Retained index backup changed. Review transfer again before applying.".into(),
            );
        }
        pins.push(file);
    }
    Ok(pins)
}

fn safe_git(root: &Path, scratch: &Path, cancel: &CancellationToken) -> Result<Git, String> {
    let mut git = Git::new(root)?;
    git.materialization_options(scratch, cancel)?;
    Ok(git)
}

pub(super) fn compare(
    source: &ManagedWorktree,
    parent: &Path,
    cancel: &CancellationToken,
) -> Result<Value, String> {
    let _pins = worktree_git::validate_ready(source)?;
    let temporary = tempfile::tempdir_in(parent)
        .map_err(|_| "Comparison storage is unavailable. Check disk space and retry.")?;
    let root = Path::new(&source.root);
    let git = safe_git(root, temporary.path(), cancel)?;
    let snapshot = worktree_snapshot::capture(&git, root, temporary.path(), cancel)?;
    let paths = changed(&git, &source.base_commit, &snapshot.tree, cancel)?;
    Ok(
        json!({"source":source.root,"base":source.base_commit,"paths":paths,"snapshot":snapshot,"note":"Saved files only, including committed changes and non-ignored untracked files. Unsaved editor buffers and ignored files are not transferred."}),
    )
}

fn changed(
    git: &Git,
    base: &str,
    tree: &str,
    cancel: &CancellationToken,
) -> Result<Vec<String>, String> {
    let bytes = git.command(
        &[
            "diff",
            "--no-ext-diff",
            "--no-textconv",
            "--no-renames",
            "--name-only",
            "-z",
            base,
            tree,
            "--",
        ],
        cancel,
    )?;
    let paths = std::str::from_utf8(&bytes)
        .map_err(|_| "Changed paths are not Unicode. Inspect with external Git.")?
        .split('\0')
        .filter(|s| !s.is_empty())
        .map(str::to_string)
        .collect::<Vec<_>>();
    if paths.len() > 2000 {
        return Err("Comparison exceeds 2,000 changed paths. Narrow the task before transferring; both checkouts remain.".into());
    }
    for path in &paths {
        worktree_snapshot::path(path)?;
    }
    Ok(paths)
}

fn retain_index(git: &Git, target: &Path, cancel: &CancellationToken) -> Result<(), String> {
    let path = git.line(
        &["rev-parse", "--path-format=absolute", "--git-path", "index"],
        cancel,
    )?;
    let bytes = super::source_control::read_bounded(Path::new(&path), 16 * 1024 * 1024)?;
    let mut file = File::options()
        .create_new(true)
        .write(true)
        .open(target)
        .map_err(|_| "Could not retain index bytes. No transfer was applied.")?;
    file.write_all(&bytes)
        .and_then(|_| file.sync_all())
        .map_err(|_| "Could not flush retained index bytes. No transfer was applied.".into())
}

pub(super) fn review(
    source_session: &str,
    source: &ManagedWorktree,
    destination_session: &str,
    destination_root: &Path,
    parent: &Path,
    paths: Vec<String>,
    cancel: &CancellationToken,
) -> Result<(Review, Value), String> {
    if paths.is_empty() || paths.len() > worktree_snapshot::MAX_PATHS {
        return Err("Choose one to 256 displayed files for transfer.".into());
    }
    let unique = paths.iter().collect::<std::collections::BTreeSet<_>>();
    if unique.len() != paths.len() {
        return Err("Choose each transfer path once.".into());
    }
    for path in &paths {
        worktree_snapshot::path(path)?;
    }
    let _source_pins = worktree_git::validate_ready(source)?;
    let destination = Pin::open(destination_root)?;
    if super::source_control::path_key(&destination.path)
        == super::source_control::path_key(Path::new(&source.root))
    {
        return Err("Choose a different destination checkout. No file was changed.".into());
    }
    let common = Pin::matches(Path::new(&source.common_directory), &source.common_identity)?;
    if !super::repository_scope::matches(&destination.path, &common.path)? {
        return Err(
            "Transfer needs a destination in the same Git repository. Choose its project chat."
                .into(),
        );
    }
    let id = uuid::Uuid::new_v4().to_string();
    let directory = parent.join(&id);
    std::fs::create_dir(&directory)
        .map_err(|_| "Could not reserve transfer storage. Check disk space and retry.")?;
    let _directory = Pin::open(&directory)?;
    let source_git = safe_git(Path::new(&source.root), &directory, cancel)?;
    let destination_git = safe_git(&destination.path, &directory, cancel)?;
    if super::source_control::path_key(Path::new(
        &destination_git.line(&["rev-parse", "--show-toplevel"], cancel)?,
    )) != super::source_control::path_key(&destination.path)
    {
        return Err("Open the destination repository root before transferring changes. Subfolder chats keep their existing files.".into());
    }
    let source_snapshot =
        worktree_snapshot::capture(&source_git, Path::new(&source.root), &directory, cancel)?;
    let destination_snapshot =
        worktree_snapshot::capture(&destination_git, &destination.path, &directory, cancel)?;
    let available = changed(
        &source_git,
        &source.base_commit,
        &source_snapshot.tree,
        cancel,
    )?;
    if paths.iter().any(|path| !available.contains(path)) {
        return Err("Selected changes disappeared. Compare again; both checkouts remain.".into());
    }
    let mut args = vec![
        "diff",
        "--binary",
        "--full-index",
        "--no-ext-diff",
        "--no-textconv",
        "--no-renames",
        &source.base_commit,
        &source_snapshot.tree,
        "--",
    ];
    args.extend(paths.iter().map(String::as_str));
    let patch = source_git.command(&args, cancel)?;
    if patch.is_empty() {
        return Err("No selected changes remain. Compare again.".into());
    }
    // Shared transport bounds the patch to 512 KiB, including binary payloads.
    let mut patch_file = File::options()
        .write(true)
        .create_new(true)
        .open(directory.join("selected.patch"))
        .map_err(|_| "Could not save the transfer patch. No transfer was applied.")?;
    patch_file
        .write_all(&patch)
        .and_then(|_| patch_file.sync_all())
        .map_err(|_| "Could not flush the transfer patch. No transfer was applied.")?;
    let source_files = worktree_snapshot::retain_raw(
        Path::new(&source.root),
        &paths,
        &directory.join("source-files"),
    )?;
    let destination_files = worktree_snapshot::retain_raw(
        &destination.path,
        &paths,
        &directory.join("destination-files"),
    )?;
    retain_index(&source_git, &directory.join("source-index.bin"), cancel)?;
    retain_index(
        &destination_git,
        &directory.join("destination-index.bin"),
        cancel,
    )?;
    for (name, snapshot) in [
        ("source", &source_snapshot),
        ("destination", &destination_snapshot),
    ] {
        for (kind, tree) in [("files", &snapshot.tree), ("index", &snapshot.index_tree)] {
            source_git.command(
                &[
                    "update-ref",
                    &format!("refs/dolores/transfers/{id}/{name}-{kind}"),
                    tree,
                    &"0".repeat(tree.len()),
                ],
                cancel,
            )?;
        }
    }
    let patch_path = directory.join("selected.patch");
    let check = destination_git.run(
        &[
            "apply",
            "--check",
            "--whitespace=nowarn",
            patch_path
                .to_str()
                .ok_or("Transfer patch path is not Unicode.")?,
        ],
        cancel,
    )?;
    // Ensure files and staging did not move while raw/object evidence was saved.
    if worktree_snapshot::capture(&source_git, Path::new(&source.root), &directory, cancel)?
        != source_snapshot
        || worktree_snapshot::capture(&destination_git, &destination.path, &directory, cancel)?
            != destination_snapshot
    {
        return Err("Files or staging changed while preparing transfer. Retained snapshots remain; compare again.".into());
    }
    worktree_snapshot::verify_raw(Path::new(&source.root), &source_files)?;
    worktree_snapshot::verify_raw(&destination.path, &destination_files)?;
    let receipt = Receipt {
        id,
        source_session: source_session.into(),
        destination_session: destination_session.into(),
        source: source.clone(),
        destination_root: destination.text()?,
        destination_identity: destination.identity,
        directory_identity: _directory.identity.clone(),
        source_snapshot,
        destination_snapshot,
        paths,
        source_files,
        destination_files,
        patch_digest: format!("{:x}", Sha256::digest(&patch)),
        applicable: check.code == 0,
        notice: if check.code == 0 {
            "Ready to transfer selected saved changes. Destination staging and unrelated files remain.".into()
        } else {
            format!("Transfer conflicts with the destination. Neither checkout was changed. Keep both versions, resolve deliberately, then compare again. {}",String::from_utf8_lossy(&check.stderr).chars().take(1600).collect::<String>())
        },
    };
    save(&directory.join("review.json"), &receipt)?;
    let token = uuid::Uuid::new_v4().to_string();
    let response = json!({"token":token,"receipt":receipt,"directory":directory,"patch":String::from_utf8_lossy(&patch),"note":"Only Apply transfers files. A conflict or interruption never resets either checkout."});
    Ok((
        Review {
            token,
            receipt,
            directory,
            created: Instant::now(),
        },
        response,
    ))
}

pub(super) fn apply(
    store: &dyn SessionStore,
    review: Review,
    cancel: &CancellationToken,
) -> Result<Value, String> {
    if review.created.elapsed() > Duration::from_secs(600) || !review.receipt.applicable {
        return Err(
            "Transfer review expired or has conflicts. Compare again; retained versions remain."
                .into(),
        );
    }
    let r = &review.receipt;
    let saved = store
        .worktree(&r.source.session)?
        .ok_or("Managed checkout ownership is unavailable.")?;
    if saved != r.source {
        return Err("Source checkout state changed. Review transfer again.".into());
    }
    let _source = worktree_git::validate_ready(&saved)?;
    let _destination_managed = super::worktrees::record_for_chat(store, &r.destination_session)?
        .map(|record| worktree_git::validate_ready(&record))
        .transpose()?;
    let destination = Pin::matches(Path::new(&r.destination_root), &r.destination_identity)?;
    let _directory = Pin::matches(&review.directory, &r.directory_identity)?;
    let _backup_pins = pin_backups(&review.directory, r)?;
    let bound = store
        .workspace(&r.destination_session)?
        .root
        .ok_or("Destination chat no longer has a working folder.")?;
    if super::source_control::path_key(Path::new(&bound))
        != super::source_control::path_key(&destination.path)
    {
        return Err("Destination chat folder changed. Review transfer again.".into());
    }
    if !super::repository_scope::matches(&destination.path, Path::new(&saved.common_directory))? {
        return Err(
            "Destination repository changed. Keep both folders and inspect before retrying.".into(),
        );
    }
    let source_git = safe_git(Path::new(&saved.root), &review.directory, cancel)?;
    let destination_git = safe_git(&destination.path, &review.directory, cancel)?;
    if worktree_snapshot::capture(
        &source_git,
        Path::new(&saved.root),
        &review.directory,
        cancel,
    )? != r.source_snapshot
        || worktree_snapshot::capture(
            &destination_git,
            &destination.path,
            &review.directory,
            cancel,
        )? != r.destination_snapshot
    {
        return Err(
            "Source or destination changed after review. Compare again; no transfer was applied."
                .into(),
        );
    }
    worktree_snapshot::verify_raw(Path::new(&saved.root), &r.source_files)?;
    worktree_snapshot::verify_raw(&destination.path, &r.destination_files)?;
    let patch_path = review.directory.join("selected.patch");
    let (_patch_pin, patch) = pin_bytes(&patch_path, 512 * 1024)?;
    if format!("{:x}", Sha256::digest(&patch)) != r.patch_digest {
        return Err("Retained patch changed. Review transfer again; no file was changed.".into());
    }
    let patch_path = patch_path
        .to_str()
        .ok_or("Transfer patch path is not Unicode.")?;
    destination_git.command(
        &["apply", "--check", "--whitespace=nowarn", patch_path],
        cancel,
    )?;
    if cancel.is_cancelled() {
        return Err("Transfer stopped before publication. Both checkouts remain.".into());
    }
    save(
        &review.directory.join("intent.json"),
        &json!({"operation":"apply","id":r.id,"certainty":"Unknown until result is saved. Inspect files before any new transfer."}),
    )?;
    let result = destination_git.run(&["apply", "--whitespace=nowarn", patch_path], cancel);
    let (applied,notice)=match result {
        Ok(output) if output.code==0=>(true,"Selected changes transferred. Source checkout and destination staging are retained.".to_string()),
        Ok(output)=>(false,format!("Transfer did not complete ({}). Inspect both folders and retained backups before reviewing again. {}",output.code,String::from_utf8_lossy(&output.stderr).chars().take(1600).collect::<String>())),
        Err(error)=>(false,format!("{error} Transfer outcome is uncertain. Inspect both folders and retained backups; no retry was started.")),
    };
    let value = json!({"id":r.id,"applied":applied,"state":if applied {"applied"}else{"recovery"},"notice":notice,"directory":review.directory,"source":saved.root,"destination":r.destination_root});
    save(&review.directory.join("result.json"),&value).map_err(|error|format!("{error} Publication may have finished. Inspect transfer {} at {} before any new review.",r.id,review.directory.display()))?;
    Ok(value)
}

pub(super) fn list(parent: &Path, source_session: &str) -> Result<Value, String> {
    if !parent.exists() {
        return Ok(json!({"items":[]}));
    }
    let _parent = Pin::open(parent)?;
    let mut items = vec![];
    for (number, entry) in std::fs::read_dir(parent)
        .map_err(|_| "Transfer evidence is unavailable. Restore access and retry.")?
        .enumerate()
    {
        if number >= 4096 {
            return Err("Transfer evidence inventory exceeds 4,096 entries. Inspect the evidence folder; no files were removed.".into());
        }
        let entry = entry.map_err(|_| "Transfer evidence could not be listed.")?;
        let id = entry
            .file_name()
            .to_str()
            .ok_or("Transfer evidence has an invalid name.")?
            .to_string();
        if uuid::Uuid::parse_str(&id).is_err() {
            continue;
        }
        if items.len() >= 128 {
            return Err("More than 128 retained transfers need inspection. Use the transfer evidence folder; no evidence was removed.".into());
        }
        let directory = entry.path();
        let _directory = Pin::open(&directory)?;
        if !directory.join("review.json").exists() {
            continue;
        }
        let raw = super::source_control::read_bounded(&directory.join("review.json"), 256 * 1024)?;
        let receipt:Receipt=serde_json::from_slice(&raw).map_err(|_|"A retained transfer record is malformed. Inspect its evidence folder; nothing was replayed.")?;
        if receipt.id != id {
            return Err(
                "Retained transfer identity changed. Inspect its folder; nothing was replayed."
                    .into(),
            );
        }
        if receipt.directory_identity != _directory.identity {
            return Err("Retained transfer folder was replaced. Inspect both folders before any new transfer.".into());
        }
        if receipt.source_session != source_session && receipt.destination_session != source_session
        {
            continue;
        }
        receipt.source.validate()?;
        let result = if directory.join("result.json").exists() {
            let value = serde_json::from_slice::<Value>(&super::source_control::read_bounded(
                &directory.join("result.json"),
                8192,
            )?)
            .map_err(|_| "Transfer result is malformed. Inspect the retained files.")?;
            if value["id"] != receipt.id
                || !matches!(value["state"].as_str(), Some("applied" | "recovery"))
                || !value["notice"].is_string()
            {
                return Err("Transfer result identity or state is malformed. Inspect the retained evidence folder; nothing was replayed.".into());
            }
            value
        } else {
            json!({"state":if directory.join("intent.json").exists(){"uncertain"}else if receipt.applicable{"reviewed"}else{"conflict"},"notice":if directory.join("intent.json").exists(){"Publication was interrupted. Inspect both checkouts and backups. This intent will not be replayed."}else{&receipt.notice}})
        };
        items.push(json!({"receipt":receipt,"result":result,"directory":directory}));
    }
    items.sort_by(|a, b| {
        a["receipt"]["id"]
            .as_str()
            .cmp(&b["receipt"]["id"].as_str())
    });
    Ok(json!({"items":items}))
}
