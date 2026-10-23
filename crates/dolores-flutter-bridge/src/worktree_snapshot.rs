//! Host-owned snapshots for reviewed transfers and archive. A temporary index
//! keeps staged state untouched; raw file bytes are retained separately because
//! Git attributes can normalize text in the object tree.
use super::{git_process, worktree_git::Git, worktree_identity::Pin};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    collections::BTreeSet,
    fs::File,
    io::{Read, Write},
    path::{Component, Path},
};
use tokio_util::sync::CancellationToken;

pub(super) const MAX_PATHS: usize = 256;
pub(super) const MAX_BYTES: usize = 64 * 1024 * 1024;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub(super) struct Snapshot {
    pub head: String,
    pub tree: String,
    pub index_tree: String,
    pub index_digest: String,
    pub configuration: String,
}

pub(super) fn path(value: &str) -> Result<(), String> {
    let windows_alias = cfg!(windows)
        && value.split('/').any(|part| {
            let base = part.split('.').next().unwrap_or("").to_ascii_uppercase();
            part.ends_with(['.', ' '])
                || part.chars().any(|c| c < ' ')
                || matches!(base.as_str(), "CON" | "PRN" | "AUX" | "NUL")
                || base
                    .strip_prefix("COM")
                    .or_else(|| base.strip_prefix("LPT"))
                    .is_some_and(|n| {
                        matches!(n, "1" | "2" | "3" | "4" | "5" | "6" | "7" | "8" | "9")
                    })
        });
    if value.is_empty()
        || windows_alias
        || value.len() > 4096
        || value.contains(['\0', '\r', '\n', '\\', ':'])
        || Path::new(value).is_absolute()
        || value.split('/').any(|part| {
            part.is_empty() || part == "." || part == ".." || part.eq_ignore_ascii_case(".git")
        })
        || Path::new(value)
            .components()
            .any(|c| !matches!(c, Component::Normal(_)))
    {
        return Err(
            "Unsupported transfer path. Choose a direct repository file; no file was changed."
                .into(),
        );
    }
    Ok(())
}

fn checked(output: git_process::Output) -> Result<String, String> {
    if output.code != 0 {
        return Err(format!("Could not snapshot checkout: {}. Existing files and staging remain; inspect before retrying.",String::from_utf8_lossy(&output.stderr).chars().take(1000).collect::<String>()));
    }
    let text =
        String::from_utf8(output.stdout).map_err(|_| "Git snapshot response is not Unicode.")?;
    Ok(text.trim().into())
}

pub(super) fn capture(
    git: &Git,
    root: &Path,
    scratch: &Path,
    cancel: &CancellationToken,
) -> Result<Snapshot, String> {
    let _pin = Pin::open(root)?;
    let head = git.line(&["rev-parse", "--verify", "HEAD"], cancel)?;
    let index_path = git.line(
        &["rev-parse", "--path-format=absolute", "--git-path", "index"],
        cancel,
    )?;
    let index = Path::new(&index_path);
    super::worktree_identity::no_alias(index)?;
    let before = super::source_control::read_bounded(index, 16 * 1024 * 1024)?;
    let index_digest = format!("{:x}", Sha256::digest(&before));
    let temporary = tempfile::tempdir_in(scratch)
        .map_err(|_| "Could not reserve snapshot storage. Check disk space and retry.")?;
    let working = temporary.path().join("working-index");
    let staged = temporary.path().join("staged-index");
    std::fs::write(&staged, &before)
        .map_err(|_| "Could not retain private index bytes. Check disk space.")?;
    let index_tree = checked(git.with_index(&["write-tree"], &staged, cancel)?)?;
    checked(git.with_index(&["read-tree", &head], &working, cancel)?)?;
    let staged_entries = git.command(&["ls-files", "--stage", "-z"], cancel)?;
    if staged_entries
        .split(|b| *b == 0)
        .any(|entry| entry.starts_with(b"160000 "))
    {
        return Err("Snapshots cannot yet include submodules. Keep this checkout and inspect with external Git.".into());
    }
    // Explicitly bound the inventory before asking Git to read file contents.
    let raw = git.command(
        &[
            "ls-files",
            "--cached",
            "--others",
            "--exclude-standard",
            "-z",
        ],
        cancel,
    )?;
    let names = std::str::from_utf8(&raw)
        .map_err(|_| "Snapshot contains a non-Unicode path. Inspect with external Git.")?;
    let mut bytes = 0u64;
    let paths = names
        .split('\0')
        .filter(|s| !s.is_empty())
        .collect::<BTreeSet<_>>();
    if paths.len() > 20000 {
        return Err("Checkout snapshot exceeds 20,000 files. Narrow the checkout or inspect with external Git; files remain.".into());
    }
    for name in paths {
        path(name)?;
        let file = root.join(name);
        match std::fs::symlink_metadata(&file) {
            Ok(meta) => {
                super::worktree_identity::no_alias(&file)?;
                if meta.is_dir() {
                    continue;
                }
                if !meta.is_file() {
                    return Err("Snapshots cannot yet include submodules or directory entries. Keep this checkout and inspect with external Git.".into());
                }
                bytes = bytes
                    .checked_add(meta.len())
                    .ok_or("Snapshot byte count overflow.")?;
                if bytes > MAX_BYTES as u64 {
                    return Err("Checkout snapshot exceeds 64 MiB of saved files. Narrow the checkout before retrying; no files were removed.".into());
                }
            }
            Err(error)
                if matches!(
                    error.kind(),
                    std::io::ErrorKind::NotFound | std::io::ErrorKind::NotADirectory
                ) => {}
            Err(_) => {
                return Err("Snapshot file is unavailable. Restore access before retrying.".into())
            }
        }
        if cancel.is_cancelled() {
            return Err("Snapshot stopped. No working file was changed.".into());
        }
    }
    checked(git.with_index(&["add", "--all", "--", "."], &working, cancel)?)?;
    let tree = checked(git.with_index(&["write-tree"], &working, cancel)?)?;
    if super::source_control::read_bounded(index, 16 * 1024 * 1024)? != before {
        return Err("Staging changed during snapshot. Review again; no staging was reset.".into());
    }
    if git.line(&["rev-parse", "--verify", "HEAD"], cancel)? != head {
        return Err(
            "Checkout commit changed during snapshot. Review again; no files were reset.".into(),
        );
    }
    Ok(Snapshot {
        head,
        tree,
        index_tree,
        index_digest,
        configuration: Git::new(root)?.configuration(cancel)?,
    })
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub(super) struct RawFile {
    pub path: String,
    pub exists: bool,
    pub digest: String,
    pub bytes: usize,
    #[serde(default)]
    pub readonly: bool,
    #[serde(default)]
    pub unix_mode: Option<u32>,
}

/// Numbered, host-owned backup names never use repository paths as destinations.
/// Preserve the manifest and copied bytes even when a later transfer fails.
pub(super) fn retain_raw(
    root: &Path,
    paths: &[String],
    target: &Path,
) -> Result<Vec<RawFile>, String> {
    if paths.is_empty() || paths.len() > MAX_PATHS {
        return Err("Choose one to 256 files for a transfer.".into());
    }
    retain_archive_raw(root, paths, target)
}

pub(super) fn retain_archive_raw(
    root: &Path,
    paths: &[String],
    target: &Path,
) -> Result<Vec<RawFile>, String> {
    if paths.len() > 20000 {
        return Err("Archive exceeds 20,000 files. Keep the checkout and narrow it or back it up externally.".into());
    }
    let _root = Pin::open(root)?;
    std::fs::create_dir(target).map_err(|_| {
        "Could not reserve retained snapshot storage. Check disk space; source files remain."
    })?;
    let _target = Pin::open(target)?;
    let mut manifest = vec![];
    let mut total = 0usize;
    for (index, name) in paths.iter().enumerate() {
        path(name)?;
        let file = root.join(name);
        // A deleted file can have missing parent components. Check every existing
        // component before reading; never follow a junction into another folder.
        for ancestor in file.ancestors().take_while(|p| p.starts_with(root)) {
            if ancestor.exists() {
                super::worktree_identity::no_alias(ancestor)?;
            }
        }
        let source=match File::open(&file) {
            Ok(source)=>Some(source),
            Err(error) if error.kind()==std::io::ErrorKind::NotFound=>None,
            Err(_)=>return Err("Could not retain a selected file. Restore access and retry; no transfer was applied.".into()),
        };
        let mut bytes = vec![];
        if let Some(source) = source {
            let metadata = source
                .metadata()
                .map_err(|_| "Snapshot file metadata is unavailable.")?;
            if !metadata.is_file() {
                return Err("Selected snapshot entry is not a regular file.".into());
            }
            source
                .take((MAX_BYTES - total + 1) as u64)
                .read_to_end(&mut bytes)
                .map_err(|_| "Could not read retained snapshot bytes.")?;
            total = total
                .checked_add(bytes.len())
                .ok_or("Snapshot byte count overflow.")?;
            if total > MAX_BYTES {
                return Err("Selected snapshot exceeds 64 MiB. Choose fewer files; no transfer was applied.".into());
            }
            let mut saved = File::options()
                .write(true)
                .create_new(true)
                .open(target.join(format!("{index}.bin")))
                .map_err(|_| {
                    "Retained snapshot could not be written. Check disk space before retrying."
                })?;
            saved
                .write_all(&bytes)
                .and_then(|_| saved.sync_all())
                .map_err(|_| {
                    "Retained snapshot could not be flushed. No transfer was applied.".to_string()
                })?;
            manifest.push(RawFile {
                path: name.clone(),
                exists: true,
                digest: format!("{:x}", Sha256::digest(&bytes)),
                bytes: bytes.len(),
                readonly: metadata.permissions().readonly(),
                unix_mode: {
                    #[cfg(unix)]
                    {
                        use std::os::unix::fs::PermissionsExt;
                        Some(metadata.permissions().mode())
                    }
                    #[cfg(not(unix))]
                    {
                        None
                    }
                },
            });
        } else {
            manifest.push(RawFile {
                path: name.clone(),
                exists: false,
                digest: format!("{:x}", Sha256::digest([])),
                bytes: 0,
                readonly: false,
                unix_mode: None,
            });
        }
    }
    let mut file = File::options()
        .write(true)
        .create_new(true)
        .open(target.join("manifest.json"))
        .map_err(|_| "Snapshot manifest could not be created.")?;
    file.write_all(&serde_json::to_vec(&manifest).map_err(|_| "Snapshot manifest is invalid.")?)
        .and_then(|_| file.sync_all())
        .map_err(|_| {
            "Snapshot manifest could not be saved. No transfer was applied.".to_string()
        })?;
    Ok(manifest)
}

pub(super) fn verify_raw(root: &Path, manifest: &[RawFile]) -> Result<(), String> {
    for record in manifest {
        path(&record.path)?;
        let file = root.join(&record.path);
        for ancestor in file.ancestors().take_while(|p| p.starts_with(root)) {
            if ancestor.exists() {
                super::worktree_identity::no_alias(ancestor)?;
            }
        }
        match std::fs::File::open(&file) {
            Ok(file)=> {
                if !record.exists || !file.metadata().map_err(|_|"Selected file metadata is unavailable.")?.is_file() {return Err("Selected files changed. Review transfer again; both checkouts remain.".into());}
                let mut bytes=vec![];
                file.take(MAX_BYTES as u64+1).read_to_end(&mut bytes).map_err(|_|"Selected file could not be rechecked.")?;
                if bytes.len()!=record.bytes || format!("{:x}",Sha256::digest(&bytes))!=record.digest {return Err("Selected files changed. Review transfer again; both checkouts remain.".into());}
            }
            Err(error) if error.kind()==std::io::ErrorKind::NotFound && !record.exists=>(),
            Err(_)=>return Err("Selected file disappeared or is unavailable. Review transfer again; retained versions remain.".into()),
        }
    }
    Ok(())
}
