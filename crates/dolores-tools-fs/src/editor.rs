//! Human editor operations use the same directory capability as model file tools.
use super::{
    create::direct_parent,
    edit::{EditTextFile, TempFile, NEXT_TEMP},
    ReadTextFile,
};
use cap_std::fs::OpenOptions;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::{
    io::{Read, Write},
    path::Path,
    sync::atomic::Ordering,
};

pub const EDITOR_BYTES: usize = 1024 * 1024;
#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EditorSnapshot {
    pub path: String,
    pub text: String,
    pub revision: String,
    pub bom: bool,
    pub newline: String,
    pub readonly: bool,
    pub reason: Option<String>,
    pub bytes: u64,
}
pub struct EditorFolder {
    read: ReadTextFile,
}
fn digest(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}
impl EditorFolder {
    pub fn new(root: &Path) -> Result<Self, String> {
        Ok(Self {
            read: ReadTextFile::new(root)?,
        })
    }
    fn direct(&self, path: &str) -> Result<(), String> {
        EditTextFile::new(self.read.clone()).direct_path(path)
    }
    fn bytes(&self, path: &str) -> Result<(Vec<u8>, cap_std::fs::Metadata), String> {
        self.direct(path)?;
        let mut options = OpenOptions::new();
        options.read(true);
        #[cfg(unix)]
        {
            use cap_std::fs::OpenOptionsExt;
            options.custom_flags(libc::O_NONBLOCK);
        }
        let file = self
            .read
            .directory
            .open_with(path, &options)
            .map_err(|_| "File could not be opened. Retry or choose another file.")?;
        let metadata = file.metadata().map_err(|_| "File is unavailable.")?;
        if !metadata.is_file() {
            return Err("Only regular files can be opened.".into());
        }
        let mut bytes = Vec::new();
        file.take((EDITOR_BYTES + 1) as u64)
            .read_to_end(&mut bytes)
            .map_err(|_| "File could not be read.")?;
        Ok((bytes, metadata))
    }
    pub fn snapshot(&self, path: &str) -> Result<EditorSnapshot, String> {
        let (bytes, metadata) = self.bytes(path)?;
        let bom = bytes.starts_with(&[0xef, 0xbb, 0xbf]);
        let raw = if bom { &bytes[3..] } else { &bytes };
        let strict = std::str::from_utf8(raw).ok();
        let text = strict.unwrap_or("");
        let crlf = text.contains("\r\n");
        let without = text.replace("\r\n", "");
        let reason = if metadata.len() > EDITOR_BYTES as u64 {
            Some("File exceeds the 1 MiB editing limit.")
        } else if bytes.contains(&0) || strict.is_none() {
            Some("Binary or non-UTF-8 file. Original bytes are unchanged.")
        } else if without.contains('\r') || (crlf && without.contains('\n')) {
            Some("Mixed or unsupported line endings. Original bytes are unchanged.")
        } else if text.lines().any(|s| s.len() > 8192) {
            Some("A line exceeds the 8 KiB editing limit.")
        } else if metadata.permissions().readonly() {
            Some("File is read-only.")
        } else {
            None
        };
        let shown = if reason.is_some() {
            let preview = &raw[..raw.len().min(64 * 1024)];
            String::from_utf8_lossy(preview)
                .replace('\0', "�")
                .lines()
                .map(|line| line.chars().take(2000).collect::<String>())
                .collect::<Vec<_>>()
                .join("\n")
        } else {
            text.replace("\r\n", "\n")
        };
        Ok(EditorSnapshot {
            path: path.into(),
            text: shown,
            revision: digest(&bytes),
            bom,
            newline: if crlf { "crlf" } else { "lf" }.into(),
            readonly: reason.is_some(),
            reason: reason.map(str::to_string),
            bytes: metadata.len(),
        })
    }
    pub fn tree(&self, path: &str, cursor: usize) -> Result<Value, String> {
        if cursor > 100_000 {
            return Err("Directory page limit reached. Open a narrower folder.".into());
        }
        if path != "." {
            self.direct(path)?;
        }
        let mut entries = Vec::with_capacity(200);
        let mut scanned = 0usize;
        let mut more = false;
        for item in self
            .read
            .directory
            .read_dir(path)
            .map_err(|_| "Folder is unavailable. Refresh or choose another folder.")?
        {
            let item = item.map_err(|_| "Folder entry is unavailable. Refresh this folder.")?;
            let at = scanned;
            scanned += 1;
            if at < cursor {
                continue;
            }
            // Bound each page's work even when most entries are excluded.
            if scanned > cursor + 1000 || entries.len() == 200 {
                more = true;
                break;
            }
            let name = match item.file_name().into_string() {
                Ok(n) => n,
                Err(_) => continue,
            };
            let target = if path == "." {
                name.clone()
            } else {
                format!("{path}/{name}")
            };
            if !super::valid_path(&target)
                || matches!(
                    name.as_str(),
                    "node_modules" | "target" | "build" | ".dart_tool"
                )
            {
                continue;
            }
            let kind = item
                .file_type()
                .map_err(|_| "Folder entry could not be checked.")?;
            if kind.is_symlink() || (!kind.is_dir() && !kind.is_file()) {
                continue;
            }
            entries.push(json!({"name":name,"path":target,"directory":kind.is_dir()}));
        }
        // Offset addresses the next raw entry. No recursive traversal or whole-directory sort.
        let next = more.then_some(scanned - 1);
        Ok(json!({"entries":entries,"cursor":next,"pageSize":200}))
    }
    pub fn save(
        &self,
        path: &str,
        expected: &str,
        text: &str,
        bom: bool,
        newline: &str,
    ) -> Result<EditorSnapshot, String> {
        let bytes = encode(text, bom, newline)?;
        let (before, metadata) = self.bytes(path)?;
        if metadata.len() > EDITOR_BYTES as u64 || digest(&before) != expected {
            return Err(
                "File changed on disk. Compare before saving; your edits are retained.".into(),
            );
        }
        if metadata.permissions().readonly() {
            return Err("File is read-only. Keep edits or Save as.".into());
        }
        if before == bytes {
            return self.snapshot(path);
        }
        let (parent, filename) = direct_parent(&self.read, path)?;
        let name = format!(
            ".dolores-edit-{}-{}",
            std::process::id(),
            NEXT_TEMP.fetch_add(1, Ordering::Relaxed)
        );
        let temp = TempFile {
            parent: &parent,
            name,
        };
        let mut options = OpenOptions::new();
        options.write(true).create_new(true);
        let mut file = parent
            .open_with(&temp.name, &options)
            .map_err(|_| "Could not stage Save. Your edits are retained; Retry or Save as.")?;
        file.write_all(&bytes)
            .map_err(|_| "Save failed. Your edits are retained.")?;
        file.set_permissions(metadata.permissions().clone())
            .map_err(|_| "Could not preserve file permissions.")?;
        file.sync_all()
            .map_err(|_| "Could not flush Save. Your edits are retained.")?;
        drop(file);
        let (current, current_meta) = self.bytes(path)?;
        if digest(&current) != expected || current_meta.permissions() != metadata.permissions() {
            return Err("File changed during Save. Compare before retrying.".into());
        }
        parent.rename(&temp.name, &parent, &filename).map_err(|_| {
            "Could not replace the file. Your edits are retained; Retry or Save as."
        })?;
        self.snapshot(path)
    }
    pub fn create(
        &self,
        path: &str,
        text: &str,
        bom: bool,
        newline: &str,
    ) -> Result<EditorSnapshot, String> {
        let bytes = encode(text, bom, newline)?;
        let (parent, filename) = direct_parent(&self.read, path)?;
        match parent.symlink_metadata(&filename) {
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
            _ => return Err("Target exists or could not be checked. Choose another name.".into()),
        }
        let temp = TempFile {
            parent: &parent,
            name: format!(
                ".dolores-edit-{}-{}",
                std::process::id(),
                NEXT_TEMP.fetch_add(1, Ordering::Relaxed)
            ),
        };
        let mut options = OpenOptions::new();
        options.write(true).create_new(true);
        let mut file = parent
            .open_with(&temp.name, &options)
            .map_err(|_| "Could not stage the new file.")?;
        file.write_all(&bytes)
            .map_err(|_| "Could not write the new file.")?;
        file.sync_all()
            .map_err(|_| "Could not flush the new file.")?;
        drop(file);
        direct_parent(&self.read, path)?;
        parent
            .hard_link(&temp.name, &parent, &filename)
            .map_err(|_| {
                "Could not create the file without replacement. Choose another name or Retry."
            })?;
        self.snapshot(path)
    }
    pub fn rename(
        &self,
        path: &str,
        target: &str,
        expected: &str,
    ) -> Result<EditorSnapshot, String> {
        let (bytes, metadata) = self.bytes(path)?;
        if metadata.len() > EDITOR_BYTES as u64 || digest(&bytes) != expected {
            return Err("File changed. Refresh before renaming.".into());
        }
        let (parent, name) = direct_parent(&self.read, path)?;
        let (dest, new_name) = direct_parent(&self.read, target)?;
        self.direct(path)?;
        parent.hard_link(&name, &dest, &new_name).map_err(|_| {
            "Destination exists or rename is unavailable. Original file is retained."
        })?;
        if let Err(_) = parent.remove_file(&name) {
            return Err("The new name was created, but the original could not be removed. Refresh both entries.".into());
        }
        self.snapshot(target)
    }
    pub fn delete(&self, path: &str, expected: &str) -> Result<(), String> {
        let (bytes, metadata) = self.bytes(path)?;
        if metadata.len() > EDITOR_BYTES as u64 || digest(&bytes) != expected {
            return Err("File changed. Refresh before deleting.".into());
        }
        self.direct(path)?;
        self.read
            .directory
            .remove_file(path)
            .map_err(|_| "Could not delete the file. The document remains open.".into())
    }
}
pub fn encode(text: &str, bom: bool, newline: &str) -> Result<Vec<u8>, String> {
    if !matches!(newline, "lf" | "crlf")
        || text.contains(['\0', '\r'])
        || text.lines().any(|s| s.len() > 8192)
    {
        return Err("Edit exceeds the line limit or contains unsupported text.".into());
    }
    let mut bytes = if bom {
        vec![0xef, 0xbb, 0xbf]
    } else {
        Vec::new()
    };
    if newline == "crlf" {
        bytes.extend_from_slice(text.replace('\n', "\r\n").as_bytes());
    } else {
        bytes.extend_from_slice(text.as_bytes());
    }
    if bytes.len() > EDITOR_BYTES {
        return Err("Edit exceeds the 1 MiB encoded-file limit. Split the change; your previous buffer is retained.".into());
    }
    Ok(bytes)
}
#[cfg(test)]
#[path = "editor_tests.rs"]
mod tests;
