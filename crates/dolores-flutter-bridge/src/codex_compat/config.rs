use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::{
    collections::{BTreeMap, BTreeSet},
    io::Read,
    path::{Path, PathBuf},
};

#[derive(Clone, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct Profile {
    pub enabled: bool,
    pub home: Option<PathBuf>,
    #[serde(default)]
    pub project: bool,
    #[serde(default)]
    pub referenced_instructions: bool,
}
impl Profile {
    pub fn validate(&self) -> Result<(), String> {
        if let Some(path) = &self.home {
            if !path.is_absolute() || path.to_string_lossy().contains('\0') {
                return Err("Choose an absolute Codex profile folder.".into());
            }
        }
        if self.enabled && self.home.is_none() {
            return Err("Choose a Codex profile before enabling compatibility.".into());
        }
        Ok(())
    }
}
pub(super) fn default_home() -> Option<PathBuf> {
    std::env::var_os("CODEX_HOME")
        .map(PathBuf::from)
        .filter(|p| p.is_absolute())
        .or_else(|| directories::BaseDirs::new().map(|p| p.home_dir().join(".codex")))
}
pub(crate) struct Snapshot {
    pub sessions: std::sync::Arc<std::sync::Mutex<BTreeSet<String>>>,
    pub runtime_guidance: std::sync::Mutex<String>,
    pub evidence_directory: Option<PathBuf>,
    pub home: Option<PathBuf>,
    pub root: Option<PathBuf>,
    pub project: bool,
    pub sources: Vec<String>,
    pub diagnostics: Vec<String>,
    pub servers: BTreeMap<String, Server>,
    pub hooks: Vec<Hook>,
    pub instructions: String,
    pub skill_collections: Vec<(PathBuf, dolores_core::SkillScope)>,
}
#[derive(Clone, Deserialize)]
pub(crate) struct Server {
    pub command: Option<String>,
    #[serde(default)]
    pub args: Vec<String>,
    pub cwd: Option<PathBuf>,
    #[serde(default)]
    pub env: BTreeMap<String, String>,
    #[serde(default)]
    pub env_vars: Vec<String>,
    pub url: Option<String>,
    #[serde(default = "yes")]
    pub enabled: bool,
    pub enabled_tools: Option<Vec<String>>,
    #[serde(default)]
    pub disabled_tools: Vec<String>,
    #[serde(default = "startup")]
    pub startup_timeout_sec: f64,
    #[serde(default = "tool_timeout")]
    pub tool_timeout_sec: f64,
}
fn yes() -> bool {
    true
}
fn startup() -> f64 {
    10.0
}
fn tool_timeout() -> f64 {
    60.0
}
#[derive(Clone)]
pub(crate) struct Hook {
    pub event: String,
    pub matcher: regex::Regex,
    pub command: String,
    pub timeout: u64,
    pub source: PathBuf,
}
impl Snapshot {
    pub fn owns_project_guidance(&self) -> bool {
        self.home.is_some() && self.root.is_some() && self.project
    }
    pub fn report(&self) -> Value {
        json!({"home":self.home, "sources":self.sources, "diagnostics":self.diagnostics, "servers":self.servers.iter().map(|(name,s)|json!({"name":name,"enabled":s.enabled,"transport":if s.url.is_some(){"http"}else{"stdio"}})).collect::<Vec<_>>(), "hookCount":self.hooks.len(), "instructionBytes":self.instructions.len(), "skillCollections":self.skill_collections.iter().map(|(p,_)|p.to_string_lossy()).collect::<Vec<_>>()})
    }
    pub fn context(&self, messages: &mut [dolores_core::Message]) {
        if self.home.is_none() {
            return;
        }
        if let Some(first) = messages.first_mut() {
            first.content.push_str("\n\nSelected Codex setup guidance (source files are read-only; host tool policy and direct human requests take precedence):\n");
            first.content.push_str(&self.instructions);
            if let Ok(guidance) = self.runtime_guidance.lock() {
                first.content.push_str(&guidance);
            }
            if !self.diagnostics.is_empty() {
                first.content.push_str(&format!("\nCompatibility diagnostics: {}\nDo not claim unavailable integrations worked.", self.diagnostics.join("; ")));
            }
        }
    }
}
pub(super) fn read(path: &Path) -> Result<Option<String>, String> {
    let file = match std::fs::File::open(path) {
        Ok(f) => f,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(_) => return Err(format!("Cannot read {}.", path.display())),
    };
    if !file.metadata().is_ok_and(|m| m.is_file()) {
        return Err(format!("{} must be a regular file.", path.display()));
    }
    let mut bytes = vec![];
    file.take(1024 * 1024 + 1)
        .read_to_end(&mut bytes)
        .map_err(|_| format!("Cannot read {}.", path.display()))?;
    if bytes.len() > 1024 * 1024 || bytes.contains(&0) {
        return Err(format!(
            "{} is not supported UTF-8 text within 1 MiB.",
            path.display()
        ));
    }
    String::from_utf8(bytes)
        .map(Some)
        .map_err(|_| format!("{} is not UTF-8 text.", path.display()))
}
pub(super) fn load(profile: &Profile, root: Option<&Path>) -> Snapshot {
    let mut snapshot = Snapshot {
        sessions: Default::default(),
        runtime_guidance: std::sync::Mutex::new(String::new()),
        evidence_directory: None,
        home: None,
        root: root.map(Path::to_path_buf),
        project: profile.project,
        sources: vec![],
        diagnostics: vec![],
        servers: BTreeMap::new(),
        hooks: vec![],
        instructions: String::new(),
        skill_collections: vec![],
    };
    if !profile.enabled {
        return snapshot;
    }
    if let Err(e) = profile.validate() {
        snapshot.diagnostics.push(e);
        return snapshot;
    }
    let home = profile.home.as_ref().unwrap();
    snapshot.home = Some(home.clone());
    if !home.is_dir() {
        snapshot.diagnostics.push(format!("Codex profile {} is missing. Create it with your integration setup command or choose an existing profile.",home.display()));
    }
    let mut layers = vec![home.clone()];
    if profile.project {
        if let Some(root) = root {
            let mut chain = vec![root.to_path_buf()];
            let mut parent = root;
            while !parent.join(".git").exists() {
                let Some(next) = parent.parent() else { break };
                chain.push(next.to_path_buf());
                parent = next;
            }
            if !parent.join(".git").exists() {
                chain = vec![root.to_path_buf()];
            }
            chain.reverse();
            layers.extend(chain.into_iter().map(|p| p.join(".codex")));
        }
    }
    let mut seen = BTreeSet::new();
    for layer in layers {
        if !seen.insert(layer.clone()) {
            continue;
        }
        let file = layer.join("config.toml");
        match read(&file) {
            Ok(Some(text)) => {
                snapshot.sources.push(file.to_string_lossy().into());
                match toml_edit::de::from_str::<Value>(&text) {
                    Ok(value) => merge(&mut snapshot, &value, &file),
                    Err(_) => snapshot.diagnostics.push(format!(
                        "Malformed TOML in {}. Fix the file; its entries were not loaded.",
                        file.display()
                    )),
                }
            }
            Ok(None) => {}
            Err(e) => snapshot.diagnostics.push(e),
        }
        let file = layer.join("hooks.json");
        match read(&file) {
            Ok(Some(text)) => {
                snapshot.sources.push(file.to_string_lossy().into());
                match serde_json::from_str::<Value>(&text) {
                    Ok(v) => hooks(&mut snapshot, &v["hooks"], &file),
                    Err(_) => snapshot.diagnostics.push(format!(
                        "Malformed JSON in {}. Its hooks were not loaded.",
                        file.display()
                    )),
                }
            }
            Ok(None) => {}
            Err(e) => snapshot.diagnostics.push(e),
        }
    }
    super::sources::load(&mut snapshot, profile, root);
    snapshot
}
fn merge(snapshot: &mut Snapshot, value: &Value, file: &Path) {
    if let Some(servers) = value.get("mcp_servers") {
        if let Some(entries) = servers.as_object() {
            for (name, value) in entries {
                // A nearer entry replaces the whole server, so removed fields cannot leak
                // from a previous layer. Malformed overrides disable the inherited entry.
                snapshot.servers.remove(name);
                let unsupported = value
                    .as_object()
                    .map(|v| {
                        v.keys()
                            .filter(|k| {
                                ![
                                    "command",
                                    "args",
                                    "cwd",
                                    "env",
                                    "env_vars",
                                    "url",
                                    "enabled",
                                    "enabled_tools",
                                    "disabled_tools",
                                    "startup_timeout_sec",
                                    "tool_timeout_sec",
                                ]
                                .contains(&k.as_str())
                            })
                            .cloned()
                            .collect::<Vec<_>>()
                    })
                    .unwrap_or_default();
                match serde_json::from_value::<Server>(value.clone()) {
                    Ok(server) if unsupported.is_empty() && (!server.enabled || server.command.is_some()!=server.url.is_some()) && server.startup_timeout_sec>0.0 && server.tool_timeout_sec>0.0=>{
                        snapshot.servers.insert(name.clone(),server);
                    }
                    _=>snapshot.diagnostics.push(format!("MCP server '{name}' in {} has invalid or unsupported options [{}]; it was not activated.",file.display(),unsupported.join(", "))),
                }
            }
        } else {
            snapshot.diagnostics.push(format!(
                "mcp_servers in {} must be a table.",
                file.display()
            ));
        }
    }
    if let Some(value) = value.get("hooks") {
        hooks(snapshot, value, file);
    }
}
fn hooks(snapshot: &mut Snapshot, value: &Value, file: &Path) {
    let Some(events) = value.as_object() else {
        snapshot
            .diagnostics
            .push(format!("hooks in {} must be an object.", file.display()));
        return;
    };
    for (event, groups) in events {
        if ![
            "SessionStart",
            "UserPromptSubmit",
            "PreToolUse",
            "PostToolUse",
            "Stop",
        ]
        .contains(&event.as_str())
        {
            snapshot.diagnostics.push(format!(
                "Unsupported Codex hook event '{event}' in {}.",
                file.display()
            ));
            continue;
        }
        let Some(groups) = groups.as_array() else {
            snapshot.diagnostics.push(format!(
                "Invalid hook groups for {event} in {}.",
                file.display()
            ));
            continue;
        };
        for group in groups {
            let matcher = group.get("matcher").and_then(Value::as_str).unwrap_or("");
            let matcher = if matcher.is_empty() {
                "(?s).*"
            } else {
                matcher
            };
            let (Ok(matcher), Some(handlers)) =
                (regex::Regex::new(matcher), group["hooks"].as_array())
            else {
                snapshot.diagnostics.push(format!(
                    "Invalid matcher/handlers for {event} in {}.",
                    file.display()
                ));
                continue;
            };
            for handler in handlers {
                let valid = handler["type"] == "command"
                    && handler.get("async").is_none_or(|v| v == false)
                    && handler.as_object().is_some_and(|o| {
                        o.keys().all(|k| {
                            [
                                "type",
                                "command",
                                "commandWindows",
                                "command_windows",
                                "timeout",
                                "statusMessage",
                                "async",
                            ]
                            .contains(&k.as_str())
                        })
                    });
                let command = if cfg!(windows) {
                    handler
                        .get("commandWindows")
                        .or_else(|| handler.get("command_windows"))
                        .unwrap_or(&handler["command"])
                } else {
                    &handler["command"]
                };
                match (valid,command.as_str(),handler.get("timeout").map_or(Some(600),Value::as_u64)) {
                    (true,Some(command),Some(timeout)) if !command.trim().is_empty() && (1..=3600).contains(&timeout)=>{
                        snapshot.hooks.push(Hook{event:event.clone(),matcher:matcher.clone(),command:command.into(),timeout,source:file.into()});
                    }
                    _=>snapshot.diagnostics.push(format!("Unsupported command hook for {event} in {}. Only synchronous command handlers are supported.",file.display())),
                }
            }
        }
    }
}
