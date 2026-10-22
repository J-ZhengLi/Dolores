use super::*;
use std::os::windows::{
    fs::{MetadataExt, OpenOptionsExt},
    io::AsRawHandle,
};
use windows_sys::Win32::{Security::Authorization::*, Storage::FileSystem::*};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub(super) struct Identity {
    volume: u32,
    high: u32,
    low: u32,
}
pub(super) struct Root {
    pub path: PathBuf,
    pub file: File,
    pub identity: Identity,
}

fn open(path: &Path, directory: bool) -> Result<File, String> {
    std::fs::OpenOptions::new()
        .access_mode(READ_CONTROL | WRITE_DAC | FILE_READ_ATTRIBUTES)
        // Pin the root namespace while its child runs; do not inherit this handle.
        .share_mode(FILE_SHARE_READ | FILE_SHARE_WRITE)
        .custom_flags(FILE_FLAG_OPEN_REPARSE_POINT | if directory {FILE_FLAG_BACKUP_SEMANTICS} else {0})
        .open(path).map_err(|_| "Isolation cannot inspect/update this folder's permissions. Choose an owned local folder.".into())
}
fn info(file: &File) -> Result<BY_HANDLE_FILE_INFORMATION, String> {
    let mut result = unsafe { std::mem::zeroed() };
    if unsafe { GetFileInformationByHandle(file.as_raw_handle(), &mut result) } == 0 {
        return Err("Isolation file identity is unavailable.".into());
    }
    Ok(result)
}
fn identity(info: &BY_HANDLE_FILE_INFORMATION) -> Identity {
    Identity {
        volume: info.dwVolumeSerialNumber,
        high: info.nFileIndexHigh,
        low: info.nFileIndexLow,
    }
}

pub(super) fn no_alias(path: &Path) -> Result<(), String> {
    for part in path.ancestors() {
        let metadata = std::fs::symlink_metadata(part).map_err(|_| {
            "Isolation folder changed or is unavailable. Review its current path.".to_string()
        })?;
        if metadata.file_attributes() & FILE_ATTRIBUTE_REPARSE_POINT != 0 {
            return Err(
                "Isolation refuses reparse/symlink paths. Choose a local folder without aliases."
                    .into(),
            );
        }
    }
    Ok(())
}
pub(super) fn root(path: &Path) -> Result<Root, String> {
    if !path.is_absolute() || !path.is_dir() || path.components().count() < 3 {
        return Err("Isolation needs an owned local folder, not a drive root.".into());
    }
    let canonical = path
        .canonicalize()
        .map_err(|_| "Isolation folder is unavailable.")?;
    if !matches!(canonical.components().next(),Some(std::path::Component::Prefix(p)) if matches!(p.kind(),std::path::Prefix::VerbatimDisk(_)|std::path::Prefix::Disk(_)))
    {
        return Err(
            "Isolation currently requires a local drive; UNC/device roots are unqualified.".into(),
        );
    }
    no_alias(path)?;
    let file = open(&canonical, true)?;
    let id = info(&file)?;
    if id.dwFileAttributes & FILE_ATTRIBUTE_REPARSE_POINT != 0 {
        return Err("Isolation root changed into a reparse path.".into());
    }
    Ok(Root {
        path: canonical,
        file,
        identity: identity(&id),
    })
}

/// Validate before and after adding an inheritable package ACE. No untrusted
/// child is resumed while this scan/setup is in progress.
pub(super) fn check_tree(root: &Root) -> Result<(), String> {
    check_tree_cancel(root, &CancellationToken::new())
}
pub(super) fn check_tree_cancel(root: &Root, cancel: &CancellationToken) -> Result<(), String> {
    stopped(cancel)?;
    no_alias(&root.path)?;
    let current = open(&root.path, true)?;
    if identity(&info(&current)?) != root.identity {
        return Err("Isolation root changed after review.".into());
    }
    let mut pending = vec![(root.path.clone(), 0)];
    let mut entries = 0usize;
    while let Some((path, depth)) = pending.pop() {
        stopped(cancel)?;
        for entry in std::fs::read_dir(path).map_err(|_| {
            "Isolation cannot validate every granted entry. Choose a narrower owned folder."
        })? {
            let entry =
                entry.map_err(|_| "Isolation folder changed while validating its entries.")?;
            stopped(cancel)?;
            entries += 1;
            if entries > 65536 || depth >= 64 {
                return Err("Isolation validation reached 65,536 entries or 64 folder levels. Choose a narrower working folder.".into());
            }
            let metadata = std::fs::symlink_metadata(entry.path())
                .map_err(|_| "Isolation folder changed during validation.")?;
            if metadata.file_attributes() & FILE_ATTRIBUTE_REPARSE_POINT != 0 {
                return Err("Isolation refuses a workspace containing reparse/symlink entries. Remove the alias or choose a narrower folder.".into());
            }
            if metadata.is_dir() {
                pending.push((entry.path(), depth + 1));
            } else {
                let file = open(&entry.path(), false)?;
                if info(&file)?.nNumberOfLinks != 1 {
                    return Err("Isolation refuses hard-linked files. Use an independent copy in this workspace.".into());
                }
            }
        }
    }
    Ok(())
}

pub(super) fn change(file: &File, sid: PSID, write: bool, revoke: bool) -> Result<(), String> {
    let mut old = null_mut();
    let mut descriptor = null_mut();
    let result = unsafe {
        GetSecurityInfo(
            file.as_raw_handle(),
            SE_FILE_OBJECT,
            DACL_SECURITY_INFORMATION,
            null_mut(),
            null_mut(),
            &mut old,
            null_mut(),
            &mut descriptor,
        )
    };
    if result != 0 {
        return Err("Isolation permissions could not be inspected.".into());
    }
    let descriptor = Local(descriptor);
    if old.is_null() {
        return Err("Isolation refuses a null/unrestricted folder ACL. Keep its permissions unchanged and choose another folder.".into());
    }
    let entry = EXPLICIT_ACCESS_W {
        grfAccessPermissions: FILE_GENERIC_READ
            | FILE_GENERIC_EXECUTE
            | if write {
                FILE_GENERIC_WRITE | DELETE
            } else {
                0
            },
        grfAccessMode: if revoke { REVOKE_ACCESS } else { GRANT_ACCESS },
        grfInheritance: if revoke {
            0
        } else {
            SUB_CONTAINERS_AND_OBJECTS_INHERIT
        },
        Trustee: TRUSTEE_W {
            pMultipleTrustee: null_mut(),
            MultipleTrusteeOperation: NO_MULTIPLE_TRUSTEE,
            TrusteeForm: TRUSTEE_IS_SID,
            TrusteeType: TRUSTEE_IS_UNKNOWN,
            ptstrName: sid.cast(),
        },
    };
    let mut new = null_mut();
    if unsafe { SetEntriesInAclW(1, &entry, old, &mut new) } != 0 {
        return Err("Isolation permission change could not be prepared.".into());
    }
    let new = Local(new.cast());
    let result = unsafe {
        SetSecurityInfo(
            file.as_raw_handle(),
            SE_FILE_OBJECT,
            DACL_SECURITY_INFORMATION,
            null_mut(),
            null_mut(),
            new.0.cast(),
            null_mut(),
        )
    };
    drop((new, descriptor));
    if result != 0 {
        return Err("Isolation permissions could not be saved. Inspect retained cleanup evidence; no native fallback was used.".into());
    }
    Ok(())
}

pub(super) fn revoke_tree(root: &Root, sid: PSID) -> Result<(), String> {
    // Inheritable ACL changes propagate. Validate the complete tree before that
    // propagation so a junction/hard link created during execution cannot make
    // cleanup touch a different resource. Retain the journal for explicit repair.
    check_tree(root)?;
    // Remove the parent's inherited ACE first. Then remove any explicit owned
    // SID entries on files created by the child, preserving all other trustees.
    change(&root.file, sid, false, true)?;
    let mut pending = vec![(root.path.clone(), 0)];
    let mut count = 0;
    while let Some((path, depth)) = pending.pop() {
        for entry in std::fs::read_dir(path).map_err(|_| {
            "Isolation cleanup could not enumerate the granted folder. Its journal is retained."
        })? {
            let entry = entry.map_err(|_| "Isolation cleanup could not inspect an entry.")?;
            count += 1;
            if count > 65536 || depth >= 64 {
                return Err("Isolation cleanup reached its traversal bound. Its journal is retained; inspect the granted folder before retrying cleanup.".into());
            }
            let metadata = std::fs::symlink_metadata(entry.path())
                .map_err(|_| "Isolation cleanup entry changed.")?;
            if metadata.file_attributes() & FILE_ATTRIBUTE_REPARSE_POINT != 0 {
                return Err("Isolation cleanup found a reparse path and did not follow it. Remove that alias and retry cleanup; its journal remains.".into());
            }
            let file = open(&entry.path(), metadata.is_dir())?;
            change(&file, sid, false, true)?;
            if metadata.is_dir() {
                pending.push((entry.path(), depth + 1));
            }
        }
    }
    Ok(())
}
