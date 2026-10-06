//! Protected local native-update protocol. No provider-selected paths or commands.
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    fs,
    io::{Read, Write},
    path::{Path, PathBuf},
};
pub const LAUNCHER: &str = "dolores-update-launcher.exe";
pub const APP: &str = "dolores_flutter.exe";
pub const LIBRARY: &str = "dolores_flutter_bridge.dll";
pub const MAX_MEMBER: u64 = 128 * 1024 * 1024;
pub const MAX_BUNDLE: u64 = 512 * 1024 * 1024;
pub const MAX_MEMBERS: usize = 2048;
pub fn id_valid(id: &str) -> bool {
    uuid::Uuid::parse_str(id).is_ok_and(|u| u.to_string() == id)
}
pub fn hash_valid(id: &str) -> bool {
    id.len() == 71
        && id.starts_with("sha256:")
        && id[7..]
            .bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
}
pub fn identity(bytes: &[u8]) -> String {
    format!("sha256:{:x}", Sha256::digest(bytes))
}
pub fn alias(meta: &fs::Metadata) -> bool {
    #[cfg(windows)]
    {
        use std::os::windows::fs::MetadataExt;
        meta.file_type().is_symlink() || meta.file_attributes() & 0x400 != 0
    }
    #[cfg(not(windows))]
    {
        meta.file_type().is_symlink()
    }
}
pub fn safe_path(path: &Path) -> Result<(), String> {
    if !path.is_absolute() {
        return Err("Native storage must be absolute.".into());
    }
    for part in path.ancestors() {
        if alias(
            &fs::symlink_metadata(part)
                .map_err(|_| "Native storage unavailable; retained work remains.")?,
        ) {
            return Err(
                "Native storage is linked or changed. Retain evidence and request fresh review."
                    .into(),
            );
        }
    }
    Ok(())
}
pub fn read(path: &Path, limit: u64) -> Result<Vec<u8>, String> {
    safe_path(path)?;
    let meta = fs::metadata(path).map_err(|_| "Native artifact unavailable.")?;
    if !meta.is_file() || meta.len() > limit {
        return Err("Native artifact exceeds its file allowance.".into());
    }
    let mut bytes = vec![];
    fs::File::open(path)
        .map_err(|_| "Native artifact unavailable.")?
        .take(limit + 1)
        .read_to_end(&mut bytes)
        .map_err(|_| "Native artifact unreadable.")?;
    if bytes.len() as u64 > limit {
        return Err("Native artifact grew beyond its allowance.".into());
    }
    Ok(bytes)
}
pub fn file_hash(path: &Path) -> Result<String, String> {
    Ok(identity(&read(path, MAX_MEMBER)?))
}
pub fn json<T: for<'a> Deserialize<'a>>(path: &Path) -> Result<T, String> {
    serde_json::from_slice(&read(path, 1024 * 1024)?)
        .map_err(|_| "Native receipt is invalid; do not replay or install.".to_owned())
}
pub fn save<T: Serialize>(path: &Path, value: &T) -> Result<(), String> {
    safe_path(
        path.parent()
            .ok_or("Native receipt location unavailable.")?,
    )?;
    if path.exists() {
        safe_path(path)?;
    }
    let bytes = serde_json::to_vec(value).map_err(|_| "Native receipt could not be encoded.")?;
    if bytes.len() > 1024 * 1024 {
        return Err("Native receipt exceeded its allowance.".into());
    }
    let temporary = path.with_extension(format!("{}.pending", uuid::Uuid::new_v4()));
    let mut file = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&temporary)
        .map_err(|_| "Native receipt could not be retained.")?;
    file.write_all(&bytes)
        .and_then(|_| file.sync_all())
        .map_err(|_| "Native receipt could not be synced.")?;
    fs::rename(&temporary, path).map_err(|_| {
        "Native receipt could not be published. Retain its pending file; no replay.".to_owned()
    })
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Member {
    pub path: String,
    pub id: String,
    pub bytes: u64,
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Bundle {
    pub id: String,
    pub files: Vec<Member>,
}
fn relative(path: &str) -> bool {
    !path.is_empty()
        && path.len() <= 512
        && !path.contains(['\\', ':'])
        && !path.chars().any(char::is_control)
        && path.split('/').all(|p| !matches!(p, "" | "." | ".."))
}
impl Bundle {
    pub fn validate(&self) -> Result<(), String> {
        if self.files.is_empty()
            || self.files.len() > MAX_MEMBERS
            || self
                .files
                .iter()
                .any(|m| !relative(&m.path) || !hash_valid(&m.id) || m.bytes > MAX_MEMBER)
            || self.files.windows(2).any(|w| w[0].path >= w[1].path)
            || self.files.iter().map(|m| m.bytes).sum::<u64>() > MAX_BUNDLE
            || identity(&serde_json::to_vec(&self.files).unwrap()) != self.id
        {
            return Err("Native bundle receipt invalid or oversized.".into());
        }
        Ok(())
    }
}
pub fn manifest(root: &Path) -> Result<Bundle, String> {
    safe_path(root)?;
    fn walk(
        root: &Path,
        dir: &Path,
        files: &mut Vec<Member>,
        total: &mut u64,
    ) -> Result<(), String> {
        for entry in fs::read_dir(dir).map_err(|_| "Native bundle unreadable.")? {
            let entry = entry.map_err(|_| "Native bundle entry unavailable.")?;
            let meta = fs::symlink_metadata(entry.path())
                .map_err(|_| "Native bundle entry unavailable.")?;
            if alias(&meta) {
                return Err("Linked native bundle members are refused.".into());
            }
            if meta.is_dir() {
                walk(root, &entry.path(), files, total)?;
            } else if meta.is_file() {
                *total = total.saturating_add(meta.len());
                if files.len() >= MAX_MEMBERS || meta.len() > MAX_MEMBER || *total > MAX_BUNDLE {
                    return Err(
                        "Native bundle exceeds 2048 files or 512 MiB; retain the build for review."
                            .into(),
                    );
                }
                let path = entry
                    .path()
                    .strip_prefix(root)
                    .unwrap()
                    .to_str()
                    .ok_or("Native bundle path is not UTF-8.")?
                    .replace('\\', "/");
                files.push(Member {
                    path,
                    id: file_hash(&entry.path())?,
                    bytes: meta.len(),
                });
            } else {
                return Err("Unsupported native bundle member.".into());
            }
        }
        Ok(())
    }
    let mut files = vec![];
    walk(root, root, &mut files, &mut 0)?;
    files.sort_by(|a, b| a.path.cmp(&b.path));
    let bundle = Bundle {
        id: identity(&serde_json::to_vec(&files).unwrap()),
        files,
    };
    bundle.validate()?;
    Ok(bundle)
}
pub fn verify(root: &Path, expected: &Bundle) -> Result<(), String> {
    expected.validate()?;
    if &manifest(root)? != expected {
        return Err("Native bundle changed since review; nothing installed.".into());
    }
    Ok(())
}
pub fn copy_bundle(source: &Path, destination: &Path, bundle: &Bundle) -> Result<(), String> {
    verify(source, bundle)?;
    fs::create_dir(destination).map_err(|_| "Separate native bundle could not be created.")?;
    safe_path(destination)?;
    for m in &bundle.files {
        let dest = destination.join(&m.path);
        fs::create_dir_all(dest.parent().unwrap())
            .map_err(|_| "Native bundle could not be staged.")?;
        safe_path(dest.parent().unwrap())?;
        let bytes = read(&source.join(&m.path), MAX_MEMBER)?;
        if identity(&bytes) != m.id {
            return Err("Native source bundle changed during copying.".into());
        }
        let mut f = fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(dest)
            .map_err(|_| "Native bundle member already exists or could not be staged.")?;
        f.write_all(&bytes)
            .and_then(|_| f.sync_all())
            .map_err(|_| "Native bundle member could not be retained.")?;
    }
    verify(destination, bundle)
}
/// Replace only the DLL in a previously staged, separate bundle. Flush the
/// writable handle: Windows rejects FlushFileBuffers on a read-only handle.
pub fn write_library(root: &Path, bytes: &[u8]) -> Result<(), String> {
    let path = root.join(LIBRARY);
    safe_path(&path)?;
    if bytes.len() as u64 > MAX_MEMBER || !path.is_file() {
        return Err("Candidate DLL is invalid or oversized.".into());
    }
    let mut file = fs::OpenOptions::new()
        .write(true)
        .truncate(true)
        .open(&path)
        .map_err(|_| "Candidate DLL could not be opened for staging.")?;
    file.write_all(bytes)
        .and_then(|_| file.sync_all())
        .map_err(|_| "Candidate DLL could not be retained and synced.".to_owned())
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Build {
    pub id: String,
    pub session: String,
    pub repair_id: String,
    pub revision: u32,
    pub source_id: String,
    pub candidate_source_id: String,
    pub evaluation_id: String,
    pub cargo_id: String,
    pub status: String,
    pub note: String,
    pub previous: Option<Bundle>,
    pub candidate: Option<Bundle>,
    pub schema: i64,
}
impl Build {
    pub fn validate(&self) -> Result<(), String> {
        if !id_valid(&self.id)
            || !id_valid(&self.session)
            || !id_valid(&self.repair_id)
            || !id_valid(&self.evaluation_id)
            || self.revision == 0
            || !hash_valid(&self.source_id)
            || !hash_valid(&self.candidate_source_id)
            || !hash_valid(&self.cargo_id)
            || self.note.len() > 2048
            || !matches!(
                self.status.as_str(),
                "started" | "ready" | "failed" | "stopped"
            )
        {
            return Err("Native build receipt invalid.".into());
        }
        if let Some(b) = &self.previous {
            b.validate()?;
        }
        if let Some(b) = &self.candidate {
            b.validate()?;
        }
        if self.status == "ready" {
            let old = self.previous.as_ref().ok_or("Previous bundle missing.")?;
            let new = self.candidate.as_ref().ok_or("Candidate bundle missing.")?;
            if old.files.len() != new.files.len()
                || old
                    .files
                    .iter()
                    .zip(&new.files)
                    .any(|(a, b)| a.path != b.path || (a.path != LIBRARY && a != b))
                || old.files.iter().find(|f| f.path == LIBRARY)
                    == new.files.iter().find(|f| f.path == LIBRARY)
                || [APP, LAUNCHER, LIBRARY]
                    .iter()
                    .any(|n| !old.files.iter().any(|f| &f.path == n))
            {
                return Err(
                    "Only the qualified Rust DLL may differ in a complete native bundle.".into(),
                );
            }
        }
        Ok(())
    }
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Process {
    pub pid: u32,
    pub created: u64,
    pub executable: PathBuf,
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Handoff {
    pub id: String,
    pub build: Build,
    pub profile: PathBuf,
    pub build_root: PathBuf,
    pub old: Process,
    pub child: Option<Process>,
    pub helper: Option<Process>,
    pub operation: String,
    pub stage: String,
    pub note: String,
    pub database_id: Option<String>,
    pub backup_id: Option<String>,
    pub expected_source: String,
}
impl Handoff {
    pub fn validate(&self) -> Result<(), String> {
        self.build.validate()?;
        if !id_valid(&self.id)
            || self.build.status != "ready"
            || !matches!(self.operation.as_str(), "install" | "restore")
            || !matches!(
                self.stage.as_str(),
                "waiting"
                    | "previousStopped"
                    | "snapshotSaved"
                    | "starting"
                    | "rollingBack"
                    | "applied"
                    | "restored"
                    | "rolledBack"
                    | "failed"
                    | "cancelled"
                    | "recoveryRequired"
            )
            || self.expected_source != self.build.source_id
                && self.expected_source != self.build.candidate_source_id
            || !self.profile.is_absolute()
            || self.build_root
                != self
                    .profile
                    .join("repairs")
                    .join(format!("build-{}", self.build.id))
            || self.note.len() > 2048
            || self.database_id.as_ref().is_some_and(|id| !hash_valid(id))
            || self.backup_id.as_ref().is_some_and(|id| !hash_valid(id))
            || std::iter::once(&self.old)
                .chain(self.child.iter())
                .chain(self.helper.iter())
                .any(|p| p.pid == 0 || p.created == 0 || !p.executable.is_absolute())
        {
            return Err(
                "Native handoff identity invalid; retain both versions without replay.".into(),
            );
        }
        safe_path(&self.profile)?;
        safe_path(&self.build_root)?;
        Ok(())
    }
    pub fn target(&self) -> PathBuf {
        self.build_root.join(if self.operation == "restore" {
            "previous"
        } else {
            "bundle"
        })
    }
    pub fn target_bundle(&self) -> &Bundle {
        if self.operation == "restore" {
            self.build.previous.as_ref().unwrap()
        } else {
            self.build.candidate.as_ref().unwrap()
        }
    }
    pub fn previous(&self) -> PathBuf {
        self.build_root.join(if self.operation == "restore" {
            "bundle"
        } else {
            "previous"
        })
    }
    pub fn previous_bundle(&self) -> &Bundle {
        if self.operation == "restore" {
            self.build.candidate.as_ref().unwrap()
        } else {
            self.build.previous.as_ref().unwrap()
        }
    }
    pub fn terminal(&self) -> bool {
        matches!(
            self.stage.as_str(),
            "applied" | "restored" | "rolledBack" | "cancelled" | "failed"
        )
    }
}
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Health {
    pub job: String,
    pub source: String,
    pub schema: i64,
    pub database: String,
    pub process: Process,
}
/// Claim only while holding the intent's launcher lock. Re-read after acquiring
/// that lock so an interrupted owner cannot be replaced using an earlier read.
pub fn claim_handoff(
    receipt: &Path,
    expected: &Handoff,
    helper: Process,
) -> Result<Handoff, String> {
    let mut current: Handoff = json(receipt)?;
    current.validate()?;
    if &current != expected || current.stage != "waiting" || current.helper.is_some() {
        return Err("Native handoff changed or already started. Inspect it; do not replay.".into());
    }
    current.helper = Some(helper);
    current.validate()?;
    save(receipt, &current)?;
    Ok(current)
}
pub fn database_identity(path: &Path) -> Result<String, String> {
    safe_path(path)?;
    if fs::metadata(path)
        .map_err(|_| "History database unavailable.")?
        .len()
        > MAX_MEMBER
    {
        return Err("History snapshot exceeds its allowance; keep the current app.".into());
    }
    let db =
        rusqlite::Connection::open_with_flags(path, rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY)
            .map_err(|_| "History database could not be inspected.")?;
    let mut hash = Sha256::new();
    let mut stmt = db
        .prepare("SELECT name,sql FROM sqlite_master WHERE type='table' ORDER BY name")
        .map_err(|_| "History schema unreadable.")?;
    let names = stmt
        .query_map([], |r| Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?)))
        .map_err(|_| "History schema unreadable.")?;
    let mut used = 0usize;
    for item in names {
        let (name, sql) = item.map_err(|_| "History schema unreadable.")?;
        hash.update(sql.as_bytes());
        hash.update([0]);
        let mut stmt = db
            .prepare(&format!("SELECT * FROM \"{}\"", name.replace('"', "\"\"")))
            .map_err(|_| "History table unreadable.")?;
        let cols = stmt.column_count();
        let mut rows = stmt.query([]).map_err(|_| "History rows unreadable.")?;
        let mut digests = vec![];
        while let Some(row) = rows.next().map_err(|_| "History row unreadable.")? {
            let mut h = Sha256::new();
            for i in 0..cols {
                use rusqlite::types::ValueRef;
                let v = row.get_ref(i).map_err(|_| "History value unreadable.")?;
                let (tag, bytes) = match v {
                    ValueRef::Null => (0, vec![]),
                    ValueRef::Integer(n) => (1, n.to_le_bytes().to_vec()),
                    ValueRef::Real(n) => (2, n.to_bits().to_le_bytes().to_vec()),
                    ValueRef::Text(b) => (3, b.to_vec()),
                    ValueRef::Blob(b) => (4, b.to_vec()),
                };
                used = used.saturating_add(bytes.len());
                if used > MAX_MEMBER as usize || digests.len() >= 500000 {
                    return Err(
                        "History verification exceeds its allowance; keep the current app.".into(),
                    );
                }
                h.update([tag]);
                h.update((bytes.len() as u64).to_le_bytes());
                h.update(bytes);
            }
            digests.push(h.finalize().to_vec());
        }
        digests.sort();
        hash.update((digests.len() as u64).to_le_bytes());
        for d in digests {
            hash.update(d);
        }
    }
    let version: i64 = db
        .query_row("PRAGMA user_version", [], |r| r.get(0))
        .map_err(|_| "History schema version unreadable.")?;
    hash.update(version.to_le_bytes());
    Ok(format!("sha256:{:x}", hash.finalize()))
}
pub fn snapshot_database(profile: &Path, dest: &Path) -> Result<String, String> {
    safe_path(profile)?;
    safe_path(dest.parent().ok_or("Snapshot location unavailable.")?)?;
    if dest.exists() {
        return Err("History snapshot already exists; do not overwrite it.".into());
    }
    let db = rusqlite::Connection::open_with_flags(
        profile.join("dolores.db"),
        rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY,
    )
    .map_err(|_| "History snapshot source unavailable.")?;
    db.backup(rusqlite::DatabaseName::Main, dest, None)
        .map_err(|_| "Consistent history snapshot could not be saved.")?;
    file_hash(dest)
}
pub fn database_schema(path: &Path) -> Result<i64, String> {
    safe_path(path)?;
    rusqlite::Connection::open_with_flags(path, rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY)
        .and_then(|db| db.query_row("PRAGMA user_version", [], |r| r.get(0)))
        .map_err(|_| "History schema could not be inspected without migration.".into())
}
#[cfg(windows)]
pub mod process;
#[cfg(test)]
mod tests;
