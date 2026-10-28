//! Capture bounds constrain memory, never the command's lifetime.
use std::{
    fs::{File, OpenOptions},
    io::Write,
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
};
pub(super) fn default_directory() -> PathBuf {
    static DIRECTORY: std::sync::OnceLock<PathBuf> = std::sync::OnceLock::new();
    DIRECTORY
        .get_or_init(|| {
            std::env::temp_dir()
                .join("dolores-command-logs")
                .join(format!(
                    "{}-{}",
                    std::process::id(),
                    std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap_or_default()
                        .as_nanos()
                ))
        })
        .clone()
}
pub(super) fn create(
    root: &Path,
    prefix: &str,
    extension: &str,
) -> std::io::Result<(String, File)> {
    static NEXT: AtomicU64 = AtomicU64::new(0);
    std::fs::create_dir_all(root)?;
    let root = root.canonicalize()?;
    let path = root.join(format!(
        "dolores-command-{prefix}-{}-{}.{extension}",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    ));
    let file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&path)?;
    Ok((path.to_string_lossy().into_owned(), file))
}
pub(super) struct OutputLog {
    root: PathBuf,
    pub name: Option<String>,
    file: Option<File>,
    pub failed: bool,
}
impl OutputLog {
    pub fn new(root: PathBuf) -> Self {
        Self {
            root,
            name: None,
            file: None,
            failed: false,
        }
    }
    pub fn append(&mut self, prefix: &[u8], bytes: &[u8], spill: bool) {
        if self.failed {
            return;
        }
        if self.file.is_none() {
            if !spill {
                return;
            }
            match create(&self.root, "output", "log") {
                Ok((name, mut file)) => {
                    self.name = Some(name);
                    if file.write_all(prefix).is_err() {
                        self.failed = true;
                        return;
                    }
                    self.file = Some(file);
                }
                Err(_) => {
                    self.failed = true;
                    return;
                }
            }
        }
        if self.file.as_mut().unwrap().write_all(bytes).is_err() {
            self.failed = true;
        }
    }
    pub fn finish(&mut self) {
        if self.file.as_mut().is_some_and(|f| f.sync_all().is_err()) {
            self.failed = true;
        }
    }
}
