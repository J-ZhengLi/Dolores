//! Pin local directory identities while a reviewed worktree operation owns them.
use std::{
    fs::File,
    path::{Path, PathBuf},
};

pub(super) struct Pin {
    pub path: PathBuf,
    pub identity: String,
    _file: File,
}

pub(super) fn no_alias(path: &Path) -> Result<(), String> {
    if !path.is_absolute()
        || path
            .components()
            .any(|p| matches!(p, std::path::Component::ParentDir))
    {
        return Err("Worktrees need an absolute local path without parent traversal.".into());
    }
    for part in path.ancestors() {
        let meta = std::fs::symlink_metadata(part)
            .map_err(|_| "Worktree folder is missing. Restore its location and inspect again.")?;
        #[cfg(windows)]
        {
            use std::os::windows::fs::MetadataExt;
            if meta.file_attributes() & 0x400 != 0 {
                return Err("Worktree paths cannot contain junctions or symbolic links. Inspect the original folder.".into());
            }
        }
        if meta.file_type().is_symlink() {
            return Err("Worktree paths cannot contain symbolic links.".into());
        }
    }
    Ok(())
}

impl Pin {
    pub fn open(path: &Path) -> Result<Self, String> {
        no_alias(path)?;
        let path = path
            .canonicalize()
            .map_err(|_| "Worktree folder is unavailable.")?;
        if !path.is_dir() {
            return Err("Worktree ownership requires a folder.".into());
        }
        #[cfg(windows)]
        let (file, identity) = {
            use std::os::windows::{fs::OpenOptionsExt, io::AsRawHandle};
            use windows_sys::Win32::Storage::FileSystem::*;
            if !matches!(path.components().next(),Some(std::path::Component::Prefix(p)) if matches!(p.kind(),std::path::Prefix::VerbatimDisk(_)|std::path::Prefix::Disk(_)))
            {
                return Err("Managed worktrees require a local drive.".into());
            }
            let file = std::fs::OpenOptions::new().access_mode(FILE_READ_ATTRIBUTES)
                .share_mode(FILE_SHARE_READ | FILE_SHARE_WRITE)
                .custom_flags(FILE_FLAG_BACKUP_SEMANTICS | FILE_FLAG_OPEN_REPARSE_POINT)
                .open(&path).map_err(|_| "Could not pin the worktree folder. Close conflicting file operations and retry.")?;
            let mut info: BY_HANDLE_FILE_INFORMATION = unsafe { std::mem::zeroed() };
            if unsafe { GetFileInformationByHandle(file.as_raw_handle(), &mut info) } == 0
                || info.dwFileAttributes & FILE_ATTRIBUTE_REPARSE_POINT != 0
            {
                return Err("Worktree filesystem identity changed or is unavailable.".into());
            }
            let id = format!(
                "{}:{}:{}",
                info.dwVolumeSerialNumber, info.nFileIndexHigh, info.nFileIndexLow
            );
            (file, id)
        };
        #[cfg(unix)]
        let (file, identity) = {
            use std::os::unix::fs::MetadataExt;
            let file =
                File::open(&path).map_err(|_| "Could not inspect the worktree directory.")?;
            let meta = file
                .metadata()
                .map_err(|_| "Worktree identity unavailable.")?;
            let id = format!("{}:{}", meta.dev(), meta.ino());
            (file, id)
        };
        Ok(Self {
            path,
            identity,
            _file: file,
        })
    }

    pub fn matches(path: &Path, expected: &str) -> Result<Self, String> {
        let pinned = Self::open(path)?;
        if pinned.identity != expected {
            return Err("The worktree or repository folder was replaced. Keep both versions and inspect its saved recovery details.".into());
        }
        Ok(pinned)
    }

    pub fn text(&self) -> Result<String, String> {
        let text = self.path.to_str().ok_or("Worktree path must be Unicode.")?;
        #[cfg(windows)]
        let text = text.strip_prefix(r"\\?\").unwrap_or(text);
        Ok(text.into())
    }
}
