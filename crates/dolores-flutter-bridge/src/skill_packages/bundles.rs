//! Owned profile storage. No archive path is ever joined to an ambient root.
use super::*;
use cap_std::fs::{Dir, OpenOptions};
use dolores_core::skill_packages::{Installed, Snapshot, MAX_INSTALLED_BUNDLES};
use serde::{Deserialize, Serialize};
use std::{
    io::Write,
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};

const RECEIPT: &str = "receipt.json";
const ARCHIVE: &str = "source.zip";
fn io_error(_: impl std::fmt::Display) -> String {
    "Package files are unavailable or changed. Retained data was preserved; inspect Packages and retry explicitly.".into()
}
fn strict(parent: &Dir, name: &str, create: bool) -> Result<Dir, String> {
    if create {
        match parent.create_dir(name) {
            Ok(()) => {}
            Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => {}
            Err(e) => return Err(io_error(e)),
        }
    }
    let m = parent.symlink_metadata(name).map_err(io_error)?;
    if !m.is_dir()
        || m.file_type().is_symlink()
        || parent.canonicalize(name).map_err(io_error)? != Path::new(name)
    {
        return Err(
            "Package directories must be direct directories; links and aliases are refused.".into(),
        );
    }
    parent.open_dir(name).map_err(io_error)
}
fn parent(root: &Dir, path: &str, create: bool) -> Result<(Dir, String), String> {
    validate_path(path)?;
    let mut parts = path.split('/').collect::<Vec<_>>();
    let leaf = parts.pop().ok_or("Choose a package file.")?.to_owned();
    let mut current = root.try_clone().map_err(io_error)?;
    for part in parts {
        current = strict(&current, part, create)?;
    }
    Ok((current, leaf))
}
fn read(root: &Dir, path: &str, limit: usize) -> Result<Vec<u8>, String> {
    let (dir, name) = parent(root, path, false)?;
    let m = dir.symlink_metadata(&name).map_err(io_error)?;
    if !m.is_file()
        || m.file_type().is_symlink()
        || dir.canonicalize(&name).map_err(io_error)? != Path::new(&name)
    {
        return Err("Package resource must be a regular file; links are refused.".into());
    }
    let file = dir.open(&name).map_err(io_error)?;
    if !file.metadata().map_err(io_error)?.is_file() {
        return Err("Package resource is no longer a file.".into());
    }
    let mut bytes = Vec::new();
    file.take((limit + 1) as u64)
        .read_to_end(&mut bytes)
        .map_err(io_error)?;
    if bytes.len() > limit {
        return Err("Package resource exceeds its saved byte allowance. Reinstall or choose another version.".into());
    }
    Ok(bytes)
}
fn write_new(root: &Dir, path: &str, bytes: &[u8]) -> Result<(), String> {
    let (dir, name) = parent(root, path, true)?;
    let mut options = OpenOptions::new();
    options.write(true).create_new(true);
    let mut file = dir.open_with(name, &options).map_err(io_error)?;
    file.write_all(bytes)
        .and_then(|()| file.sync_all())
        .map_err(io_error)
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Receipt {
    schema: u32,
    installed: Installed,
}
pub struct Cache {
    pub root: PathBuf,
    dir: Dir,
}
struct Stage {
    parent: Dir,
    name: String,
    published: bool,
}
impl Drop for Stage {
    fn drop(&mut self) {
        // This newly created UUID directory belongs solely to this operation.
        // The capability handle bounds cleanup even if an ambient path changes.
        if !self.published {
            let _ = self.parent.remove_dir_all(&self.name);
        }
    }
}
impl Cache {
    pub fn open(profile: &Path) -> Result<Self, String> {
        let profile = std::fs::canonicalize(profile).map_err(io_error)?;
        let owner =
            Dir::open_ambient_dir(&profile, cap_std::ambient_authority()).map_err(io_error)?;
        let dir = strict(&owner, "skill-packages", true)?;
        Ok(Self {
            root: profile.join("skill-packages"),
            dir,
        })
    }
    pub fn entries(&self) -> Result<Vec<String>, String> {
        let mut values = Vec::new();
        for item in self.dir.entries().map_err(io_error)? {
            if values.len() >= MAX_INSTALLED_BUNDLES {
                return Err("Package cache has sixty retained entries. Remove unused packages or inspect unfinished publications before installing more.".into());
            }
            let name = item
                .map_err(io_error)?
                .file_name()
                .into_string()
                .map_err(|_| "Package cache has an unreadable entry. Retained files remain.")?;
            values.push(name);
        }
        values.sort();
        Ok(values)
    }
    pub(super) fn snapshot(&self, p: &InspectedPackage) -> Snapshot {
        Snapshot {
            id: p.archive_digest.clone(),
            content_digest: p.content_digest.clone(),
            manifest: p.manifest.clone(),
            resources: p.resources.clone(),
            directories: p.directories.clone(),
            root: self
                .root
                .join(&p.archive_digest)
                .join("payload")
                .to_string_lossy()
                .into_owned(),
        }
    }
    fn owned_entry(entry: &str) -> bool {
        valid_digest(entry)
            || entry
                .strip_prefix("stage-")
                .is_some_and(|v| uuid::Uuid::parse_str(v).is_ok())
    }
    pub fn receipt(&self, entry: &str) -> Result<Installed, String> {
        if !Self::owned_entry(entry) {
            return Err(
                "Unknown package-cache entry was preserved. Inspect it manually before removal."
                    .into(),
            );
        }
        let dir = strict(&self.dir, entry, false)?;
        let r: Receipt =
            serde_json::from_slice(&read(&dir, RECEIPT, 512 * 1024)?).map_err(io_error)?;
        r.installed.validate()?;
        let expected = self.root.join(&r.installed.package.id).join("payload");
        if r.schema != 1
            || Path::new(&r.installed.package.root) != expected
            || (valid_digest(entry) && r.installed.package.id != entry)
        {
            return Err(
                "Package receipt does not match this owned cache. Files were preserved.".into(),
            );
        }
        Ok(r.installed)
    }
    pub fn checked(&self, id: &str) -> Result<(Installed, Vec<u8>), String> {
        let result = self.checked_archive(id)?;
        let dir = strict(&self.dir, id, false)?;
        self.verify_payload(&dir, &result.0, false)?;
        Ok(result)
    }
    /// Return actual payload bytes, never an unchecked ambient path or ZIP entry.
    pub(crate) fn resource(&self, snapshot: &Snapshot, path: &str) -> Result<Vec<u8>, String> {
        let (saved, _) = self.checked(&snapshot.id)?;
        if &saved.package != snapshot {
            return Err(
                "Selected package changed. Inspect Packages and select a healthy version.".into(),
            );
        }
        let resource = snapshot
            .resources
            .iter()
            .find(|r| r.path == path)
            .ok_or("This file is not in the selected package. List its resources first.")?;
        let bundle = strict(&self.dir, &snapshot.id, false)?;
        let payload = strict(&bundle, "payload", false)?;
        let bytes = read(&payload, path, resource.bytes)?;
        if bytes.len() != resource.bytes
            || format!("{:x}", Sha256::digest(&bytes)) != resource.digest
        {
            return Err("Package resource changed. No bytes were shared; inspect retained data or choose another healthy version.".into());
        }
        Ok(bytes)
    }
    pub fn checked_archive(&self, id: &str) -> Result<(Installed, Vec<u8>), String> {
        if !valid_digest(id) {
            return Err("Choose an exact retained package identity.".into());
        }
        let saved = self.receipt(id)?;
        let dir = strict(&self.dir, id, false)?;
        let zip = read(&dir, ARCHIVE, MAX_ARCHIVE_BYTES)?;
        let inspected = inspect(&saved.document.name, &zip)?;
        if self.snapshot(&inspected) != saved.package || inspected.document != saved.document {
            return Err("Retained package archive or receipt changed. Reinstall its original ZIP; no activation occurred.".into());
        }
        Ok((saved, zip))
    }
    fn verify_payload(&self, bundle: &Dir, saved: &Installed, partial: bool) -> Result<(), String> {
        let top = bundle
            .entries()
            .map_err(io_error)?
            .map(|e| {
                e.map_err(io_error).and_then(|e| {
                    e.file_name()
                        .into_string()
                        .map_err(|_| "Unknown package filename.".into())
                })
            })
            .collect::<Result<BTreeSet<_>, String>>()?;
        if top
            .iter()
            .any(|v| ![RECEIPT, ARCHIVE, "payload"].contains(&v.as_str()))
        {
            return Err(
                "Package contains unknown files. Preserve or move them before removal or reuse."
                    .into(),
            );
        }
        match strict(bundle, "payload", false) {
            Ok(payload) => {
                let mut found_files = BTreeSet::new();
                let mut found_dirs = BTreeSet::new();
                walk(&payload, "", &mut found_files, &mut found_dirs)?;
                let expected_files = saved
                    .package
                    .resources
                    .iter()
                    .map(|r| r.path.clone())
                    .collect::<BTreeSet<_>>();
                let expected_dirs = saved
                    .package
                    .directories
                    .iter()
                    .cloned()
                    .collect::<BTreeSet<_>>();
                if !found_files.is_subset(&expected_files)
                    || !found_dirs.is_subset(&expected_dirs)
                    || (!partial && (found_files != expected_files || found_dirs != expected_dirs))
                {
                    return Err("Package resources were added or removed. Keep the files, or reinstall from the retained ZIP.".into());
                }
                for resource in &saved.package.resources {
                    if partial && !found_files.contains(&resource.path) {
                        continue;
                    }
                    let bytes = read(&payload, &resource.path, resource.bytes)?;
                    if bytes.len() != resource.bytes
                        || format!("{:x}", Sha256::digest(&bytes)) != resource.digest
                    {
                        return Err(format!("Package resource {} changed. Its files were preserved; export or inspect them before reinstalling/removing.",resource.path));
                    }
                }
            }
            Err(e) if partial && !top.contains("payload") => {
                let _ = e;
            }
            Err(e) => return Err(e),
        }
        Ok(())
    }
    pub fn publish(&self, name: &str, bytes: &[u8]) -> Result<Installed, String> {
        if !cfg!(windows) {
            return Err(
                "Package publication currently needs the qualified Windows filesystem backend."
                    .into(),
            );
        }
        let p = inspect(name, bytes)?;
        if self.dir.symlink_metadata(&p.archive_digest).is_ok() {
            return self.checked(&p.archive_digest).map(|r| r.0);
        }
        if self.entries()?.len() >= MAX_INSTALLED_BUNDLES {
            return Err("Package cache has sixty retained entries. Remove unused packages before installing another bundle.".into());
        }
        let name = format!("stage-{}", uuid::Uuid::new_v4());
        self.dir.create_dir(&name).map_err(io_error)?;
        let mut cleanup = Stage {
            parent: self.dir.try_clone().map_err(io_error)?,
            name: name.clone(),
            published: false,
        };
        let stage = strict(&self.dir, &name, false)?;
        let installed = Installed {
            document: p.document.clone(),
            package: self.snapshot(&p),
            installed_at: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis() as i64,
        };
        installed.validate()?;
        write_new(
            &stage,
            RECEIPT,
            &serde_json::to_vec(&Receipt {
                schema: 1,
                installed: installed.clone(),
            })
            .map_err(io_error)?,
        )?;
        write_new(&stage, ARCHIVE, bytes)?;
        let payload = strict(&stage, "payload", true)?;
        for path in &p.directories {
            let (parent, name) = parent(&payload, path, true)?;
            strict(&parent, &name, true)?;
        }
        for (path, bytes) in &p.files {
            write_new(&payload, path, bytes)?;
        }
        self.verify_payload(&stage, &installed, false)?;
        drop(payload);
        drop(stage);
        // Windows directory rename refuses an existing destination directory.
        // Both parents are the same held capability; no ambient path is followed.
        self.dir
            .rename(&name, &self.dir, &p.archive_digest)
            .map_err(io_error)?;
        cleanup.published = true;
        self.checked(&p.archive_digest).map(|r| r.0)
    }
    /// Caller has removed scoped registration and proved no remaining references.
    /// Missing known files are accepted only here, for explicit partial cleanup.
    pub fn remove(&self, entry: &str) -> Result<(), String> {
        let saved = self.receipt(entry)?;
        let bundle = strict(&self.dir, entry, false)?;
        let staging = entry.starts_with("stage-");
        let source = if bundle.symlink_metadata(ARCHIVE).is_ok() {
            Some(read(&bundle, ARCHIVE, MAX_ARCHIVE_BYTES)?)
        } else {
            None
        };
        let original = if source
            .as_ref()
            .is_some_and(|b| format!("{:x}", Sha256::digest(b)) == saved.package.id)
        {
            source.clone()
        } else if staging {
            Some(self.checked(&saved.package.id).map_err(|_|"Unfinished publication has no complete source ZIP. Reimport its original ZIP, then retry removing this retained staging entry. Active skills and retained files remain.".to_string())?.1)
        } else if source.is_some() || bundle.symlink_metadata("payload").is_ok() {
            return Err("Retained package ZIP changed or is missing. Files were preserved; reimport its original ZIP before removal.".into());
        } else {
            None
        };
        let inspected = original
            .as_ref()
            .map(|zip| inspect(&saved.document.name, zip))
            .transpose()?;
        if inspected
            .as_ref()
            .is_some_and(|p| self.snapshot(p) != saved.package || p.document != saved.document)
        {
            return Err("Package receipt no longer matches its original ZIP. Retained files were preserved.".into());
        }
        if staging {
            if source
                .as_ref()
                .is_some_and(|b| !original.as_ref().is_some_and(|zip| zip.starts_with(b)))
            {
                return Err("Staging ZIP contains changed bytes. Retain or inspect it manually before removal.".into());
            }
            verify_staging(
                &bundle,
                &saved,
                inspected.as_ref().ok_or(
                    "Reimport the original package ZIP before removing this staging entry.",
                )?,
            )?;
        } else {
            self.verify_payload(&bundle, &saved, true)?;
        }
        if bundle.symlink_metadata("payload").is_ok() {
            let payload = strict(&bundle, "payload", false)?;
            for r in &saved.package.resources {
                if payload.symlink_metadata(&r.path).is_ok() {
                    let bytes = read(&payload, &r.path, r.bytes)?;
                    let matches = if staging {
                        inspected
                            .as_ref()
                            .and_then(|p| p.files.get(&r.path))
                            .is_some_and(|expected| expected.starts_with(&bytes))
                    } else {
                        format!("{:x}", Sha256::digest(&bytes)) == r.digest
                    };
                    if !matches {
                        return Err("Package changed during removal. Retained ZIP and receipt remain for recovery.".into());
                    }
                    let (dir, name) = parent(&payload, &r.path, false)?;
                    dir.remove_file(name).map_err(io_error)?;
                }
            }
            let mut dirs = saved.package.directories.clone();
            dirs.sort_by_key(|p| std::cmp::Reverse(p.split('/').count()));
            for p in dirs {
                if payload.symlink_metadata(&p).is_ok() {
                    let (dir, name) = parent(&payload, &p, false)?;
                    strict(&dir, &name, false)?;
                    dir.remove_dir(name).map_err(io_error)?;
                }
            }
            drop(payload);
            bundle.remove_dir("payload").map_err(io_error)?;
        }
        if bundle.symlink_metadata(ARCHIVE).is_ok() {
            bundle.remove_file(ARCHIVE).map_err(io_error)?;
        }
        bundle.remove_file(RECEIPT).map_err(io_error)?;
        drop(bundle);
        self.dir.remove_dir(entry).map_err(io_error)
    }
}
fn verify_staging(bundle: &Dir, saved: &Installed, p: &InspectedPackage) -> Result<(), String> {
    for item in bundle.entries().map_err(io_error)? {
        let name = item
            .map_err(io_error)?
            .file_name()
            .into_string()
            .map_err(|_| "Unknown staging filename.")?;
        if ![RECEIPT, ARCHIVE, "payload"].contains(&name.as_str()) {
            return Err(
                "Staging entry contains unknown files. Retained data was preserved.".into(),
            );
        }
    }
    if bundle.symlink_metadata("payload").is_err() {
        return Ok(());
    }
    let payload = strict(bundle, "payload", false)?;
    let mut files = BTreeSet::new();
    let mut dirs = BTreeSet::new();
    walk(&payload, "", &mut files, &mut dirs)?;
    let expected_dirs = saved
        .package
        .directories
        .iter()
        .cloned()
        .collect::<BTreeSet<_>>();
    if !dirs.is_subset(&expected_dirs) || files.iter().any(|f| !p.files.contains_key(f)) {
        return Err(
            "Staging entry contains unknown resources. Retained data was preserved.".into(),
        );
    }
    for path in files {
        let expected = &p.files[&path];
        let bytes = read(&payload, &path, expected.len())?;
        if !expected.starts_with(&bytes) {
            return Err(
                "Staging resource bytes changed. Preserve or inspect them manually before removal."
                    .into(),
            );
        }
    }
    Ok(())
}
fn walk(
    root: &Dir,
    prefix: &str,
    files: &mut BTreeSet<String>,
    dirs: &mut BTreeSet<String>,
) -> Result<(), String> {
    if prefix.split('/').count() > 13
        || files.len() > MAX_PACKAGE_FILES
        || dirs.len() > MAX_PACKAGE_ENTRIES * 12
    {
        return Err("Package directory exceeds its saved bound.".into());
    }
    for item in root.entries().map_err(io_error)? {
        let name = item
            .map_err(io_error)?
            .file_name()
            .into_string()
            .map_err(|_| "Package filename is invalid.")?;
        let path = if prefix.is_empty() {
            name.clone()
        } else {
            format!("{prefix}/{name}")
        };
        validate_path(&path)?;
        let m = root.symlink_metadata(&name).map_err(io_error)?;
        if m.file_type().is_symlink() {
            return Err("Package resources cannot be links.".into());
        }
        if m.is_dir() {
            dirs.insert(path.clone());
            walk(&strict(root, &name, false)?, &path, files, dirs)?;
        } else if m.is_file() {
            files.insert(path);
        } else {
            return Err("Package contains a special file.".into());
        }
        if files.len() > MAX_PACKAGE_FILES || dirs.len() > MAX_PACKAGE_ENTRIES * 12 {
            return Err("Package directory exceeds its saved bound.".into());
        }
    }
    Ok(())
}
