//! Read-only Codex setup compatibility. No Codex authentication is consumed.
#[cfg(test)]
mod agent_tests;
mod config;
#[cfg(test)]
mod hook_tests;
pub(crate) mod hooks;
#[cfg(test)]
mod journey_tests;
mod mcp;
mod process;
mod sources;
#[cfg(test)]
mod tests;

use crate::Engine;
pub(super) use config::{Profile, Snapshot};
pub(super) use mcp::activate as activate_mcp;
use serde_json::{json, Value};
pub(super) use sources::skills as add_skills;
use std::{
    path::{Path, PathBuf},
    sync::Mutex,
};

#[derive(Default)]
pub(crate) struct Settings {
    pub path: Option<PathBuf>,
    pub profile: Mutex<Profile>,
    pub sessions: std::sync::Arc<Mutex<std::collections::BTreeSet<String>>>,
    pub warning: Mutex<Option<String>>,
}
impl Settings {
    pub fn open(directory: &Path) -> Result<Self, String> {
        let path = directory.join("codex-compatibility.json");
        let (profile,warning) = match std::fs::read(&path) {
            Ok(bytes) => match serde_json::from_slice(&bytes){
                Ok(profile)=>(profile,None),
                Err(_)=>(Profile::default(),Some("Codex compatibility settings are malformed. Compatibility is disabled; select the profile again and Save to replace these settings. The source profiles and chat history remain.".into())),
            },
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => (Profile::default(),None),
            Err(_) => (Profile::default(),Some("Codex compatibility settings cannot be read. Compatibility is disabled; restore access and retry.".into())),
        };
        Ok(Self {
            path: Some(path),
            profile: Mutex::new(profile),
            sessions: Default::default(),
            warning: Mutex::new(warning),
        })
    }
    pub fn snapshot(&self, root: Option<&Path>) -> Result<Snapshot, String> {
        let profile = self
            .profile
            .lock()
            .map_err(|_| "Codex compatibility settings unavailable.")?
            .clone();
        let mut snapshot = config::load(&profile, root);
        snapshot.sessions = self.sessions.clone();
        if let Some(warning) = self
            .warning
            .lock()
            .map_err(|_| "Codex settings unavailable.")?
            .as_ref()
        {
            snapshot.diagnostics.push(warning.clone());
        }
        snapshot.evidence_directory = self
            .path
            .as_ref()
            .and_then(|p| p.parent())
            .map(|p| p.join("compat-results"));
        Ok(snapshot)
    }
}
impl Engine {
    pub(super) fn codex_snapshot(&self, session: Option<&str>) -> Result<Snapshot, String> {
        let root = session
            .map(|id| self.store.workspace(id))
            .transpose()?
            .and_then(|w| w.root);
        self.codex_compatibility
            .snapshot(root.as_deref().map(Path::new))
    }
    pub(super) fn codex_settings(&self, session: Option<&str>) -> Result<Value, String> {
        let profile = self
            .codex_compatibility
            .profile
            .lock()
            .map_err(|_| "Codex settings unavailable.")?
            .clone();
        let snapshot = self.codex_snapshot(session)?;
        Ok(
            json!({"profile":profile, "detectedHome":config::default_home(), "ownedHome":self.workspace_directory.as_ref().and_then(|p|p.parent()).map(|p|p.join("compat/codex")), "report":snapshot.report()}),
        )
    }
    pub(super) fn save_codex_settings(
        &self,
        session: Option<&str>,
        profile: Profile,
    ) -> Result<Value, String> {
        profile.validate()?;
        let mut current = self
            .codex_compatibility
            .profile
            .lock()
            .map_err(|_| "Codex settings unavailable.")?;
        if let Some(path) = &self.codex_compatibility.path {
            // Replace only the Dolores-owned setting; source profile files are never written.
            let bytes = serde_json::to_vec_pretty(&profile)
                .map_err(|_| "Codex settings cannot be encoded.")?;
            let mut temporary = tempfile::NamedTempFile::new_in(path.parent().unwrap())
                .map_err(|_| "Codex settings cannot be saved.")?;
            use std::io::Write;
            temporary
                .write_all(&bytes)
                .and_then(|_| temporary.as_file().sync_all())
                .map_err(|_| "Codex settings cannot be saved.")?;
            temporary
                .persist(path)
                .map_err(|_| "Codex settings cannot be replaced. Keep your draft and retry.")?;
        }
        *current = profile;
        self.codex_compatibility
            .warning
            .lock()
            .map_err(|_| "Codex settings unavailable.")?
            .take();
        drop(current);
        self.codex_settings(session)
    }
}
