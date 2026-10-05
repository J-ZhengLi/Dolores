use serde::{Deserialize, Serialize};

pub const MOD_ABI: u32 = 1;
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ModManifest {
    pub id: String,
    pub api: u32,
    pub state_schema: u32,
    pub hook: String,
    pub capabilities: Vec<String>,
    pub dependencies: Vec<String>,
    pub title: String,
    pub description: String,
}
impl Default for ModManifest {
    fn default() -> Self {
        Self { id:"recovery-hints".into(), api:1, state_schema:1, hook:"recovery_hint".into(), capabilities:vec![], dependencies:vec![], title:"Recovery guidance".into(), description:"A scoped mod suggests a host-owned recovery view. It never executes an action or changes a limit.".into() }
    }
}
impl ModManifest {
    pub fn validate(&self) -> Result<(), String> {
        if self.id != "recovery-hints"
            || self.api != MOD_ABI
            || self.state_schema != 1
            || self.hook != "recovery_hint"
            || !self.capabilities.is_empty()
            || !self.dependencies.is_empty()
            || self.title.is_empty()
            || self.title.len() > 80
            || self.description.len() > 512
            || self.title.chars().any(char::is_control)
        {
            return Err("Unsupported mod API, state, capability, dependency or card. ABI 1 allows only a stateless recovery hint; baseline retained.".into());
        }
        Ok(())
    }
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ModVersion {
    pub identity: String,
    pub manifest: ModManifest,
    pub source: String,
    pub baseline: Option<String>,
    pub results: Vec<bool>,
    pub baseline_results: Vec<bool>,
    pub status: String,
    pub reason: String,
    pub model: String,
}
impl ModVersion {
    pub fn improved(&self) -> bool {
        self.results.len() == 6
            && self.baseline_results.len() == 6
            && self.results.iter().all(|v| *v)
            && self.baseline_results.iter().any(|v| !*v)
    }
}
#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ModState {
    pub revision: u32,
    pub automatic: bool,
    pub active: Option<String>,
    pub previous: Option<String>,
    pub pending: Option<String>,
    pub versions: Vec<ModVersion>,
    pub events: Vec<String>,
    #[serde(default)]
    pub draft: String,
    #[serde(default)]
    pub draft_notice: String,
}
impl ModState {
    pub fn active_version(&self) -> Option<&ModVersion> {
        self.active
            .as_ref()
            .and_then(|id| self.versions.iter().find(|v| &v.identity == id))
    }
    pub fn validate(&self) -> Result<(), String> {
        if self.draft.len() > 8192
            || self.draft_notice.len() > 1024
            || self.versions.len() > 8
            || self.events.len() > 32
            || self.events.iter().any(|s| s.len() > 512)
        {
            return Err(
                "Mod history is full or invalid. Baseline retained; use another working folder."
                    .into(),
            );
        }
        let mut ids = std::collections::BTreeSet::new();
        for v in &self.versions {
            v.manifest.validate()?;
            if v.source.is_empty()
                || v.source.len() > 8192
                || v.identity.len() != 64
                || !v.identity.bytes().all(|b| b.is_ascii_hexdigit())
                || !ids.insert(v.identity.clone())
                || v.results.len() > 6
                || v.baseline_results.len() > 6
                || v.reason.len() > 512
                || v.model.len() > 128
                || ![
                    "staged",
                    "review",
                    "active",
                    "retired",
                    "rejected",
                    "quarantined",
                ]
                .contains(&v.status.as_str())
            {
                return Err("Invalid mod version. Baseline retained.".into());
            }
        }
        for id in [&self.active, &self.previous, &self.pending]
            .into_iter()
            .flatten()
        {
            if !ids.contains(id) {
                return Err(
                    "Mod recovery pointer is missing. Disable mods and retain history.".into(),
                );
            }
        }
        if self.active_version().is_some_and(|v| v.status != "active") {
            return Err("Active mod state is inconsistent.".into());
        }
        Ok(())
    }
}
