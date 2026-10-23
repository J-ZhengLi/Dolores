//! Read-only repository membership for host ownership checks. Never execute Git,
//! hooks or configuration here: admission holds the existing host coordinator.
use std::path::{Path, PathBuf};

fn local(path: &Path) -> Result<(), String> {
    #[cfg(windows)]
    if !matches!(path.components().next(), Some(std::path::Component::Prefix(p)) if matches!(p.kind(), std::path::Prefix::Disk(_)|std::path::Prefix::VerbatimDisk(_)))
    {
        return Err("Repository management requires inspectable local-drive ownership.".into());
    }
    #[cfg(not(windows))]
    let _ = path;
    Ok(())
}

fn pointer(path: &Path) -> Result<Option<String>, String> {
    use std::io::Read;
    match std::fs::symlink_metadata(path) {
        Ok(_) => crate::worktree_identity::no_alias(path)?,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(_) => return Err("Repository ownership metadata is unavailable.".into()),
    }
    let file = match std::fs::File::open(path) {
        Ok(file) => file,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(_) => {
            return Err(
                "Repository ownership metadata is unavailable. Inspect the folder and retry."
                    .into(),
            )
        }
    };
    let mut bytes = vec![];
    file.take(8193)
        .read_to_end(&mut bytes)
        .map_err(|_| "Repository ownership metadata is unreadable.")?;
    if bytes.len() > 8192 {
        return Err(
            "Repository ownership metadata exceeds 8 KiB. Inspect it with external Git.".into(),
        );
    }
    let value =
        String::from_utf8(bytes).map_err(|_| "Repository ownership metadata is not Unicode.")?;
    let value = value.trim();
    if value.is_empty() || value.contains(['\0', '\r', '\n']) {
        return Err(
            "Repository ownership metadata is malformed. Inspect it with external Git.".into(),
        );
    }
    Ok(Some(value.into()))
}

fn directory(base: &Path, text: &str) -> Result<PathBuf, String> {
    let value = Path::new(text);
    let path = if value.is_absolute() {
        value.to_path_buf()
    } else {
        base.join(value)
    };
    local(&path)?;
    let mut normalized = PathBuf::new();
    for component in path.components() {
        match component {
            std::path::Component::ParentDir => {
                if !normalized.pop() {
                    return Err("Repository ownership target is malformed.".into());
                }
            }
            std::path::Component::CurDir => (),
            other => normalized.push(other),
        }
    }
    Ok(crate::worktree_identity::Pin::open(&normalized)?.path)
}

pub(super) fn common_directory(root: &Path) -> Result<Option<PathBuf>, String> {
    // Session-only scopes are not filesystem roots.
    if !root.is_absolute() {
        return Ok(None);
    }
    local(root)?;
    let root = crate::worktree_identity::Pin::open(root)?.path;
    for ancestor in root.ancestors() {
        let marker = ancestor.join(".git");
        let metadata =
            match std::fs::symlink_metadata(&marker) {
                Ok(metadata) => metadata,
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => continue,
                Err(_) => return Err(
                    "Git ownership marker is unavailable. Inspect it before repository management."
                        .into(),
                ),
            };
        crate::worktree_identity::no_alias(&marker)?;
        let git = if metadata.is_dir() {
            marker
                .canonicalize()
                .map_err(|_| "Git directory is unavailable.")?
        } else {
            let value = pointer(&marker)?
                .ok_or("Git ownership marker disappeared. Refresh before retrying.")?;
            directory(
                ancestor,
                value
                    .strip_prefix("gitdir: ")
                    .ok_or("Git ownership marker is malformed. Inspect it with external Git.")?,
            )?
        };
        return match pointer(&git.join("commondir"))? {
            Some(common) => directory(&git, &common).map(Some),
            None => Ok(Some(git)),
        };
    }
    Ok(None)
}

pub(super) fn matches(root: &Path, common: &Path) -> Result<bool, String> {
    Ok(common_directory(root)?.is_some_and(|candidate| {
        crate::source_control::path_key(&candidate) == crate::source_control::path_key(common)
    }))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn nested_worktree_relative_common_pointer_and_malformed_metadata_are_bounded() {
        let temp = tempfile::tempdir().unwrap();
        let common = temp.path().join("repo/.git");
        let registration = common.join("worktrees/task");
        let checkout = temp.path().join("checkout");
        std::fs::create_dir_all(&registration).unwrap();
        std::fs::create_dir_all(checkout.join("nested")).unwrap();
        std::fs::write(
            checkout.join(".git"),
            format!("gitdir: {}\n", registration.display()),
        )
        .unwrap();
        std::fs::write(registration.join("commondir"), "../..\n").unwrap();
        assert!(matches(&checkout.join("nested"), &common.canonicalize().unwrap()).unwrap());
        assert!(matches(&temp.path().join("repo"), &common.canonicalize().unwrap()).unwrap());
        assert_eq!(common_directory(temp.path()).unwrap(), None);
        std::fs::write(registration.join("commondir"), "x".repeat(8193)).unwrap();
        assert!(common_directory(&checkout).unwrap_err().contains("8 KiB"));
        std::fs::write(checkout.join(".git"), "not a Git pointer").unwrap();
        assert!(common_directory(&checkout)
            .unwrap_err()
            .contains("malformed"));
    }
}
