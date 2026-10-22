//! Windows LPAC transport. Callers must review literal grants before allocation.
//! This module is not an advertised/qualified product mode merely by compiling.
use serde::{Deserialize, Serialize};
use std::{
    collections::HashSet,
    ffi::{c_void, OsStr},
    fs::File,
    io::{Read, Write},
    os::windows::{ffi::OsStrExt, fs::OpenOptionsExt},
    path::{Path, PathBuf},
    ptr::{null, null_mut},
    sync::{Arc, Mutex},
};
use windows_sys::Win32::{
    Foundation::*,
    Security::{Isolation::*, *},
};

mod acl;
mod review;
use review::stopped;
pub use review::Review;
use tokio_util::sync::CancellationToken;
#[cfg(test)]
mod tests;

fn wide(s: &OsStr) -> Vec<u16> {
    s.encode_wide().chain(Some(0)).collect()
}
struct Local(*mut c_void);
impl Drop for Local {
    fn drop(&mut self) {
        unsafe {
            LocalFree(self.0);
        }
    }
}
struct Sid(PSID);
impl Drop for Sid {
    fn drop(&mut self) {
        unsafe {
            FreeSid(self.0);
        }
    }
}
fn sid(name: &str) -> Result<Sid, String> {
    let mut value = null_mut();
    if unsafe {
        DeriveAppContainerSidFromAppContainerName(wide(OsStr::new(name)).as_ptr(), &mut value)
    } < 0
        || value.is_null()
    {
        return Err("Isolation package identity is unavailable. Nothing was launched.".into());
    }
    Ok(Sid(value))
}
struct Capabilities {
    groups: *mut PSID,
    group_count: u32,
    sids: *mut PSID,
    count: u32,
    entries: Vec<SID_AND_ATTRIBUTES>,
}
impl Capabilities {
    fn registry_read() -> Result<Self, String> {
        let mut this = Self {
            groups: null_mut(),
            group_count: 0,
            sids: null_mut(),
            count: 0,
            entries: vec![],
        };
        let ok = unsafe {
            DeriveCapabilitySidsFromName(
                wide(OsStr::new("registryRead")).as_ptr(),
                &mut this.groups,
                &mut this.group_count,
                &mut this.sids,
                &mut this.count,
            )
        };
        if ok == 0
            || this.count == 0
            || this.count > 16
            || this.group_count > 16
            || this.sids.is_null()
        {
            return Err(
                "Isolation's registryRead runtime capability is unavailable. Nothing was launched."
                    .into(),
            );
        }
        this.entries = (0..this.count as usize)
            .map(|i| SID_AND_ATTRIBUTES {
                Sid: unsafe { *this.sids.add(i) },
                Attributes: 4, /* SE_GROUP_ENABLED */
            })
            .collect();
        Ok(this)
    }
}
impl Drop for Capabilities {
    fn drop(&mut self) {
        unsafe {
            for (array, count) in [(self.groups, self.group_count), (self.sids, self.count)] {
                if !array.is_null() {
                    for i in 0..count as usize {
                        LocalFree(*array.add(i));
                    }
                    LocalFree(array.cast());
                }
            }
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Grant {
    path: PathBuf,
    identity: acl::Identity,
    write: bool,
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Journal {
    version: u32,
    id: String,
    profile: String,
    #[serde(default = "legacy_runtime")]
    runtime: String,
    grants: Vec<Grant>,
}
fn legacy_runtime() -> String {
    "runtime.exe".into()
}
fn runtime_name(name: &str) -> bool {
    name.len() <= 128
        && name.len() > 4
        && name.to_ascii_lowercase().ends_with(".exe")
        && name
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b == b'.' || b == b'_' || b == b'-')
        && !name.starts_with('.')
}

/// One exclusive host per private state directory. Live leases keep it alive;
/// a second process/host cannot recover another owner's grants.
pub struct Host {
    root: PathBuf,
    _lock: File,
    active: Mutex<HashSet<String>>,
    gate: Mutex<()>,
}
impl Host {
    pub fn open(path: &Path) -> Result<Arc<Self>, String> {
        if !path.is_absolute() {
            return Err("Isolation state needs an absolute private directory.".into());
        }
        std::fs::create_dir_all(path).map_err(|_| "Isolation state could not be created.")?;
        acl::no_alias(path)?;
        let root = path
            .canonicalize()
            .map_err(|_| "Isolation state is unavailable.")?;
        let lock = std::fs::OpenOptions::new()
            .create(true)
            .truncate(false)
            .read(true)
            .write(true)
            .share_mode(0)
            .open(root.join("owner.lock"))
            .map_err(|_| {
                "Another host owns isolation cleanup. Close it before opening this state directory."
            })?;
        Ok(Arc::new(Self {
            root,
            _lock: lock,
            active: Mutex::new(HashSet::new()),
            gate: Mutex::new(()),
        }))
    }

    pub fn recover(&self) -> Result<(), String> {
        let _gate = self
            .gate
            .lock()
            .map_err(|_| "Isolation cleanup state is unavailable.")?;
        self.recover_locked()
    }
    fn recover_locked(&self) -> Result<(), String> {
        let active = self
            .active
            .lock()
            .map_err(|_| "Isolation owner state is unavailable.")?;
        let mut count = 0;
        for entry in std::fs::read_dir(&self.root)
            .map_err(|_| "Isolation recovery directory is unavailable.")?
        {
            let entry = entry.map_err(|_| "Isolation recovery entry is unavailable.")?;
            let name = entry.file_name().to_string_lossy().into_owned();
            if name == "owner.lock" {
                continue;
            }
            if active.contains(&name) {
                continue;
            }
            count += 1;
            if count > 64 {
                return Err("Isolation recovery reached 64 retained launches. Inspect and recover existing evidence before starting more isolated work.".into());
            }
            if uuid::Uuid::parse_str(&name)
                .ok()
                .is_none_or(|id| id.to_string() != name)
            {
                return Err("Unexpected entry in private isolation state. Inspect it before retrying cleanup; no command was launched.".into());
            }
            acl::no_alias(&entry.path())?;
            if !entry.path().join("journal.json").exists() {
                if entry.path().join("revoked").is_file() {
                    cleanup_finished(&entry.path())?;
                    continue;
                }
                // A crash before the journal was saved cannot have created a
                // profile/grant. Only remove empty, owned preparation folders.
                if entry.path().join("bin").is_dir() {
                    acl::no_alias(&entry.path().join("bin"))?;
                    std::fs::remove_dir(entry.path().join("bin")).map_err(|_|"Unjournaled isolation runtime is not empty. Inspect it; nothing was replayed.")?;
                }
                std::fs::remove_dir(entry.path()).map_err(|_| {
                    "Unjournaled isolation state is not empty. Inspect it; nothing was replayed."
                })?;
                continue;
            }
            let journal = read_journal(&entry.path(), &name)?;
            cleanup(&self.root, &entry.path(), &journal)?;
        }
        Ok(())
    }

    /// No child runs here. A durable journal precedes profile creation and every
    /// grant. On any setup error Drop performs bounded, owned-SID-only cleanup.
    pub fn prepare(
        self: &Arc<Self>,
        executable: &Path,
        workspace: &Path,
        readonly: &[PathBuf],
    ) -> Result<Lease, String> {
        self.prepare_inner(executable, workspace, readonly, None)
    }
    fn prepare_inner(
        self: &Arc<Self>,
        executable: &Path,
        workspace: &Path,
        readonly: &[PathBuf],
        fault: Option<usize>,
    ) -> Result<Lease, String> {
        self.prepare_with_cancel(
            executable,
            workspace,
            readonly,
            fault,
            &CancellationToken::new(),
        )
    }
    fn prepare_with_cancel(
        self: &Arc<Self>,
        executable: &Path,
        workspace: &Path,
        readonly: &[PathBuf],
        fault: Option<usize>,
        cancel: &CancellationToken,
    ) -> Result<Lease, String> {
        stopped(cancel)?;
        let _gate = self
            .gate
            .lock()
            .map_err(|_| "Isolation state is unavailable.")?;
        if self.pending_cleanup()? != 0 {
            return Err("Isolation cleanup is pending. Inspect retained state and Retry cleanup before a new launch.".into());
        }
        stopped(cancel)?;
        if readonly.len() > 8 {
            return Err(
                "Isolation supports at most eight explicit read-only roots. Nothing was launched."
                    .into(),
            );
        }
        let mut roots = vec![acl::root(workspace)?];
        for path in readonly {
            roots.push(acl::root(path)?);
        }
        for root in &roots {
            if self.root.starts_with(&root.path) || root.path.starts_with(&self.root) {
                return Err("Isolation grants cannot include its private state directory. Choose a separate workspace.".into());
            }
            acl::check_tree_cancel(root, cancel)?;
        }
        for i in 0..roots.len() {
            for j in 0..i {
                if roots[i].path.starts_with(&roots[j].path)
                    || roots[j].path.starts_with(&roots[i].path)
                {
                    return Err(
                        "Isolation grants overlap. Review disjoint working/read-only roots.".into(),
                    );
                }
            }
        }
        let source =
            File::open(executable).map_err(|_| "Reviewed isolation executable is unavailable.")?;
        let before = source
            .metadata()
            .map_err(|_| "Reviewed executable identity is unavailable.")?;
        if before.len() > 256 * 1024 * 1024 {
            return Err("Isolation runtime copy exceeds 256 MiB. Use a qualified smaller runtime; nothing was launched.".into());
        }
        let runtime = executable.file_name().and_then(OsStr::to_str)
            .filter(|name| runtime_name(name))
            .ok_or("Isolation requires a direct executable with a simple .exe filename; nothing was launched.")?.to_string();
        let id = uuid::Uuid::new_v4().to_string();
        stopped(cancel)?;
        let directory = self.root.join(&id);
        std::fs::create_dir(&directory)
            .map_err(|_| "Isolation launch directory could not be created.")?;
        let bin = directory.join("bin");
        if std::fs::create_dir(&bin).is_err() {
            let _ = std::fs::remove_dir(&directory);
            return Err("Isolation runtime directory could not be created.".into());
        }
        roots.push(acl::root(&bin)?);
        let journal = Journal {
            version: 1,
            id: id.clone(),
            profile: format!("Dolores.Isolation.{}", id.replace('-', "")),
            runtime,
            grants: roots
                .iter()
                .enumerate()
                .map(|(i, r)| Grant {
                    path: r.path.clone(),
                    identity: r.identity.clone(),
                    write: i == 0,
                })
                .collect(),
        };
        // File creation uses a private unique directory, never an existing path.
        if let Err(error) = save_journal(&directory, &journal) {
            drop(roots);
            let _ = std::fs::remove_file(directory.join("journal.json"));
            let _ = std::fs::remove_dir(&bin);
            let _ = std::fs::remove_dir(&directory);
            return Err(error);
        }
        self.active
            .lock()
            .map_err(|_| "Isolation owner state unavailable.")?
            .insert(id);
        let mut lease = Lease {
            host: self.clone(),
            directory,
            journal,
            roots,
            sid: None,
            capabilities: None,
            cleaned: false,
        };
        // The launch is now journaled and registered as live. Release this
        // guard before any fallible setup so Lease::drop can acquire cleanup's
        // gate; acquire a later guard around the actual ACL changes below.
        drop(_gate);
        let mut destination = std::fs::OpenOptions::new()
            .create_new(true)
            .write(true)
            .open(lease.executable())
            .map_err(|_| "Isolation runtime copy could not be created.")?;
        let mut source = source.take(256 * 1024 * 1024 + 1);
        let mut copied = 0u64;
        let mut buffer = [0u8; 65536];
        loop {
            stopped(cancel)?;
            let count = source
                .read(&mut buffer)
                .map_err(|_| "Isolation runtime copy failed.")?;
            if count == 0 {
                break;
            }
            destination
                .write_all(&buffer[..count])
                .map_err(|_| "Isolation runtime copy failed.")?;
            copied += count as u64;
        }
        destination
            .sync_all()
            .map_err(|_| "Isolation runtime copy could not be saved.")?;
        let after = executable
            .metadata()
            .map_err(|_| "Reviewed executable changed during copying.")?;
        if copied != before.len()
            || after.len() != before.len()
            || after.modified().ok() != before.modified().ok()
        {
            return Err(
                "Reviewed executable changed during isolation setup. Nothing was launched.".into(),
            );
        }
        drop(destination);
        stopped(cancel)?;
        lease.sid = Some(sid(&lease.journal.profile)?);
        let name = wide(OsStr::new(&lease.journal.profile));
        let mut created = null_mut();
        let result = unsafe {
            CreateAppContainerProfile(
                name.as_ptr(),
                name.as_ptr(),
                name.as_ptr(),
                null(),
                0,
                &mut created,
            )
        };
        if result < 0 || created.is_null() {
            return Err(
                "LPAC profile creation failed. Nothing was launched; no native fallback was used."
                    .into(),
            );
        }
        let created = Sid(created);
        if unsafe { EqualSid(created.0, lease.sid.as_ref().unwrap().0) } == 0 {
            return Err("Isolation package identity changed. Nothing was launched.".into());
        }
        lease.capabilities = Some(Capabilities::registry_read()?);
        let _setup_gate = self
            .gate
            .lock()
            .map_err(|_| "Isolation permission state is unavailable.")?;
        for (i, root) in lease.roots.iter().enumerate() {
            stopped(cancel)?;
            acl::change(&root.file, lease.sid.as_ref().unwrap().0, i == 0, false)?;
            if fault == Some(i) {
                return Err("Injected isolation setup failure before child creation.".into());
            }
        }
        for root in &lease.roots {
            acl::check_tree_cancel(root, cancel)?;
        }
        stopped(cancel)?;
        Ok(lease)
    }
}

pub struct Lease {
    host: Arc<Host>,
    directory: PathBuf,
    journal: Journal,
    roots: Vec<acl::Root>,
    sid: Option<Sid>,
    capabilities: Option<Capabilities>,
    cleaned: bool,
}
// Windows owns the pointed-to SID allocations; fields are never concurrently
// mutated, and process creation borrows the lease until all attributes are copied.
unsafe impl Send for Lease {}
impl Lease {
    pub fn executable(&self) -> PathBuf {
        self.directory.join("bin").join(&self.journal.runtime)
    }
    pub fn workspace(&self) -> PathBuf {
        self.roots[0].path.clone()
    }
    pub(crate) fn validate(&self, cancel: &CancellationToken) -> Result<(), String> {
        stopped(cancel)?;
        if self.cleaned || self.roots.is_empty() {
            return Err(
                "Isolation lease already closed. Review a fresh command; nothing was launched."
                    .into(),
            );
        }
        for root in &self.roots {
            acl::check_tree_cancel(root, cancel)?;
        }
        Ok(())
    }
    pub(crate) fn security(&mut self) -> SECURITY_CAPABILITIES {
        let capabilities = self.capabilities.as_mut().unwrap();
        SECURITY_CAPABILITIES {
            AppContainerSid: self.sid.as_ref().unwrap().0,
            Capabilities: capabilities.entries.as_mut_ptr(),
            CapabilityCount: capabilities.entries.len() as u32,
            Reserved: 0,
        }
    }
    pub fn close(&mut self) -> Result<(), String> {
        let host = self.host.clone();
        let _gate = host
            .gate
            .lock()
            .map_err(|_| "Isolation cleanup state is unavailable.")?;
        if self.cleaned {
            return Ok(());
        }
        // Release namespace pins before deleting our private runtime directory.
        self.roots.clear();
        cleanup(&self.host.root, &self.directory, &self.journal)?;
        self.cleaned = true;
        self.host
            .active
            .lock()
            .map_err(|_| "Isolation owner state unavailable.")?
            .remove(&self.journal.id);
        Ok(())
    }
}
pub fn spawn(
    lease: Lease,
    args: &[String],
    environment: &[(String, String)],
) -> Result<(crate::process::Running, File, File, File), String> {
    spawn_cancellable(lease, args, environment, &CancellationToken::new())
}
pub fn spawn_cancellable(
    lease: Lease,
    args: &[String],
    environment: &[(String, String)],
    cancel: &CancellationToken,
) -> Result<(crate::process::Running, File, File, File), String> {
    lease.validate(cancel)?;
    // Descendants resolving the reviewed runtime by name need the owned copy,
    // rather than the ungranted installed runtime. Keep one PATH entry.
    let bin = lease
        .executable()
        .parent()
        .unwrap()
        .to_string_lossy()
        .into_owned();
    let mut environment = environment.to_vec();
    environment.retain(|(key, _)| !key.eq_ignore_ascii_case("PATH"));
    environment.push(("PATH".into(), bin));
    environment.sort_by_key(|(key, _)| key.to_ascii_uppercase());
    crate::windows::spawn_isolated(lease, args, &environment, cancel)
}
impl Drop for Lease {
    fn drop(&mut self) {
        let _ = self.close();
        // Failed cleanup is no longer live. A later explicit recovery must see
        // the durable journal and may never start another command in its place.
        self.host
            .active
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .remove(&self.journal.id);
    }
}

fn save_journal(directory: &Path, journal: &Journal) -> Result<(), String> {
    let bytes =
        serde_json::to_vec(journal).map_err(|_| "Isolation journal could not be prepared.")?;
    let mut file = std::fs::OpenOptions::new()
        .create_new(true)
        .write(true)
        .open(directory.join("journal.json"))
        .map_err(|_| "Isolation journal could not be created. No permissions were changed.")?;
    file.write_all(&bytes)
        .and_then(|_| file.sync_all())
        .map_err(|_| "Isolation journal could not be saved. No permissions were changed.".into())
}
fn read_journal(directory: &Path, id: &str) -> Result<Journal, String> {
    let path = directory.join("journal.json");
    acl::no_alias(&path)?;
    let metadata=path.metadata().map_err(|_|"Isolation recovery journal is missing. Inspect private isolation state; no command was launched.")?;
    if metadata.len() > 65536 {
        return Err(
            "Isolation recovery journal exceeds its bound. Inspect retained evidence.".into(),
        );
    }
    let journal: Journal = serde_json::from_slice(
        &std::fs::read(path).map_err(|_| "Isolation recovery journal unavailable.")?,
    )
    .map_err(|_| "Isolation recovery journal is malformed. Nothing was replayed.")?;
    if journal.version != 1
        || journal.id != id
        || journal.profile != format!("Dolores.Isolation.{}", id.replace('-', ""))
        || !runtime_name(&journal.runtime)
        || !(2..=10).contains(&journal.grants.len())
    {
        return Err("Isolation recovery identity is invalid. Nothing was replayed.".into());
    }
    Ok(journal)
}
fn cleanup(state: &Path, directory: &Path, journal: &Journal) -> Result<(), String> {
    if revoked_runtime(directory).ok().as_deref() == Some(&journal.runtime) {
        return cleanup_finished(directory);
    }
    let sid = sid(&journal.profile)?;
    let mut failure = None;
    for grant in &journal.grants {
        if (state.starts_with(&grant.path) || grant.path.starts_with(state))
            && grant.path != directory.join("bin").canonicalize().unwrap_or_default()
        {
            failure.get_or_insert_with(|| {
                "A cleanup grant overlaps private isolation state.".to_string()
            });
            continue;
        }
        match acl::root(&grant.path) {
            Ok(root) if root.identity == grant.identity => {
                if let Err(error) = acl::revoke_tree(&root, sid.0) {
                    failure.get_or_insert(error);
                }
            }
            Ok(_) => {
                failure.get_or_insert_with(|| "The original granted root was replaced; restore or inspect it before cleanup.".to_string());
            }
            Err(error) => {
                failure.get_or_insert(error);
            }
        }
    }
    let profile = wide(OsStr::new(&journal.profile));
    if unsafe { DeleteAppContainerProfile(profile.as_ptr()) } < 0 {
        failure.get_or_insert_with(|| {
            "The disposable profile could not be fully deleted.".to_string()
        });
    }
    if let Some(error) = failure {
        return Err(format!("Isolation cleanup is incomplete: {error} Its journal and unique SID remain available; inspect the granted paths and Retry cleanup before starting isolated work. No command was repeated."));
    }
    if directory.join("revoked").exists() {
        acl::no_alias(&directory.join("revoked"))?;
    }
    let mut marker = std::fs::OpenOptions::new()
        .create(true)
        .truncate(true)
        .write(true)
        .open(directory.join("revoked"))
        .map_err(|_| "Isolation cleanup completion could not be recorded; its journal remains.")?;
    marker
        .write_all(journal.runtime.as_bytes())
        .and_then(|_| marker.sync_all())
        .map_err(|_| "Isolation cleanup completion could not be saved; its journal remains.")?;
    drop(marker);
    cleanup_finished(directory)
}
fn revoked_runtime(directory: &Path) -> Result<String, String> {
    let path = directory.join("revoked");
    acl::no_alias(&path)?;
    if path
        .metadata()
        .map_err(|_| "Isolation cleanup marker unavailable.")?
        .len()
        > 128
    {
        return Err(
            "Isolation cleanup marker is malformed; retained state needs inspection.".into(),
        );
    }
    let name =
        std::fs::read_to_string(path).map_err(|_| "Isolation cleanup marker unavailable.")?;
    let name = if name.is_empty() {
        legacy_runtime()
    } else {
        name
    };
    if !runtime_name(&name) {
        return Err(
            "Isolation cleanup marker is malformed; retained state needs inspection.".into(),
        );
    }
    Ok(name)
}
fn cleanup_finished(directory: &Path) -> Result<(), String> {
    acl::no_alias(directory)?;
    let name = revoked_runtime(directory)?;
    for entry in
        std::fs::read_dir(directory).map_err(|_| "Isolation cleanup directory unavailable.")?
    {
        let entry = entry.map_err(|_| "Isolation cleanup entry unavailable.")?;
        if !["bin", "journal.json", "revoked"]
            .contains(&entry.file_name().to_string_lossy().as_ref())
        {
            return Err("Unexpected file in isolation state. It was retained with the journal; inspect it before retrying cleanup.".into());
        }
        acl::no_alias(&entry.path())?;
    }
    let bin = directory.join("bin");
    if bin.exists() {
        for entry in
            std::fs::read_dir(&bin).map_err(|_| "Isolation runtime cleanup unavailable.")?
        {
            let entry = entry.map_err(|_| "Isolation runtime entry unavailable.")?;
            if entry.file_name() != OsStr::new(&name) {
                return Err("Unexpected runtime file was retained. Inspect isolation state before retrying cleanup.".into());
            }
            acl::no_alias(&entry.path())?;
        }
    }
    let runtime = directory.join("bin").join(name);
    if runtime.exists() {
        std::fs::remove_file(runtime)
            .map_err(|_| "Isolation runtime cleanup failed; its journal remains.")?;
    }
    if directory.join("bin").exists() {
        std::fs::remove_dir(directory.join("bin"))
            .map_err(|_| "Isolation runtime directory cleanup failed; its journal remains.")?;
    }
    if directory.join("journal.json").exists() {
        std::fs::remove_file(directory.join("journal.json"))
            .map_err(|_| "Isolation journal cleanup failed.")?;
    }
    if directory.join("revoked").exists() {
        std::fs::remove_file(directory.join("revoked"))
            .map_err(|_| "Isolation cleanup marker could not be removed.")?;
    }
    std::fs::remove_dir(directory).map_err(|_| "Isolation launch directory cleanup failed.")?;
    Ok(())
}
