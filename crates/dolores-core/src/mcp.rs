use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct McpLaunch {
    pub label: String,
    pub executable: String,
    pub args: Vec<String>,
}
impl McpLaunch {
    pub fn validate(&self) -> Result<(), String> {
        if self.label.trim().is_empty()
            || self.label.len() > 128
            || self.label.chars().any(char::is_control)
            || !std::path::Path::new(&self.executable).is_absolute()
            || self.executable.len() > 32768
            || self.executable.contains('\0')
            || self.args.len() > 32
            || self.args.iter().any(|s| s.len() > 4096 || s.contains('\0'))
            || serde_json::to_vec(self).map_or(true, |v| v.len() > 48 * 1024)
        {
            return Err("Use a short server name, an absolute direct executable and at most 32 literal arguments.".into());
        }
        #[cfg(windows)]
        if !std::path::Path::new(&self.executable)
            .extension()
            .is_some_and(|s| s.eq_ignore_ascii_case("exe"))
        {
            return Err(
                "Choose a direct .exe program; shell and batch launching is not supported.".into(),
            );
        }
        Ok(())
    }
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct McpFingerprint {
    pub path: String,
    pub sha256: String,
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct McpTool {
    pub name: String,
    pub description: String,
    pub input_schema: Value,
}
impl McpTool {
    pub fn validate(&self) -> Result<(), String> {
        if self.name.is_empty()
            || self.name.len() > 128
            || self.name.chars().any(char::is_control)
            || self.description.len() > 2048
            || self.description.contains('\0')
            || !self.input_schema.is_object()
            || self.input_schema["type"] != "object"
            || serde_json::to_vec(self).map_or(true, |v| v.len() > 8192)
        {
            return Err("MCP tools need bounded names, descriptions and object input schemas within 8 KiB each.".into());
        }
        Ok(())
    }
}
pub const MAX_MCP_CONNECTIONS: usize = 4;
pub const MAX_ACTIVE_MCP_TOOLS: usize = 2;
pub fn legacy_mcp_id() -> String {
    "legacy".into()
}
pub fn is_legacy_mcp_id(id: &String) -> bool {
    id == "legacy"
}
pub fn valid_mcp_id(id: &str) -> bool {
    id == "legacy" || valid_credential_id(id)
}

pub fn check_mcp_capacity(
    connections: &[McpConnection],
    proposed: &McpConnection,
) -> Result<(), String> {
    let others = connections
        .iter()
        .filter(|c| c.id != proposed.id)
        .collect::<Vec<_>>();
    if others.len() >= MAX_MCP_CONNECTIONS {
        return Err("This folder already has four MCP connections. Forget a saved server before adding another.".into());
    }
    let active: usize = others
        .iter()
        .filter(|c| c.enabled)
        .map(|c| c.tools.len())
        .sum();
    if active + proposed.tools.len() > MAX_ACTIVE_MCP_TOOLS {
        return Err("Only two external tools can be enabled per folder. Disable another server or select fewer tools, then retry Enable. Your review is preserved.".into());
    }
    Ok(())
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct McpConnection {
    #[serde(default = "legacy_mcp_id", skip_serializing_if = "is_legacy_mcp_id")]
    pub id: String,
    pub revision: u32,
    pub enabled: bool,
    pub launch: McpLaunch,
    pub fingerprints: Vec<McpFingerprint>,
    pub protocol_version: String,
    pub server_name: String,
    pub server_version: String,
    pub tools: Vec<McpTool>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub credentials: Vec<McpCredentialBinding>,
    // Recoverable cleanup references; never values. Missing entries are harmless.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub retired_credentials: Vec<String>,
}
impl McpConnection {
    pub fn validate(&self) -> Result<(), String> {
        self.launch.validate()?;
        validate_mcp_credential_names(
            &self
                .credentials
                .iter()
                .map(|b| b.name.clone())
                .collect::<Vec<_>>(),
        )?;
        if self.retired_credentials.len() > 32
            || self
                .credentials
                .iter()
                .map(|b| &b.credential_id)
                .chain(self.retired_credentials.iter())
                .any(|id| !valid_credential_id(id))
        {
            return Err(
                "MCP credential references are invalid. Review the connection again.".into(),
            );
        }
        let ids = self
            .credentials
            .iter()
            .map(|b| &b.credential_id)
            .chain(self.retired_credentials.iter())
            .collect::<Vec<_>>();
        if ids.iter().enumerate().any(|(i, id)| ids[..i].contains(id)) {
            return Err(
                "MCP credential references are duplicated. Review the connection again.".into(),
            );
        }
        if !valid_mcp_id(&self.id)
            || self.revision == 0
            || self.tools.is_empty()
            || self.tools.len() > 2
            || !matches!(
                self.protocol_version.as_str(),
                "2025-11-25" | "2025-06-18" | "2025-03-26" | "2024-11-05"
            )
            || self.server_name.is_empty()
            || self.server_name.len() > 128
            || self.server_version.len() > 128
            || self.server_name.chars().any(char::is_control)
            || self.server_version.chars().any(char::is_control)
            || self.fingerprints.is_empty()
            || self.fingerprints.len() > 33
            || self.fingerprints[0].path != self.launch.executable
            || self.fingerprints.iter().any(|f| {
                !std::path::Path::new(&f.path).is_absolute()
                    || f.path.len() > 32768
                    || f.path.contains('\0')
                    || f.sha256.len() != 64
                    || !f.sha256.bytes().all(|b| b.is_ascii_hexdigit())
            })
            || self
                .tools
                .iter()
                .enumerate()
                .any(|(i, t)| self.tools[..i].iter().any(|other| other.name == t.name))
            || serde_json::to_vec(self).map_or(true, |v| v.len() > 96 * 1024)
        {
            return Err("Saved MCP connection is invalid. Review the server again.".into());
        }
        for tool in &self.tools {
            tool.validate()?;
        }
        Ok(())
    }
    pub fn specs(&self) -> Vec<crate::ToolSpec> {
        if !self.enabled {
            return vec![];
        }
        self.tools.iter().enumerate().map(|(index, tool)| crate::ToolSpec {
            name: if self.id == "legacy" { format!("mcp_tool_{}", index + 1) } else { format!("mcp_tool_{}_{}", self.id.replace('-', ""), index + 1) },
            description: format!("External MCP tool {} from {}. Requires approval and starts the reviewed server. {}", tool.name, self.launch.label, tool.description),
            parameters: tool.input_schema.clone(),
        }).collect()
    }
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct McpCallPreview {
    #[serde(default = "legacy_mcp_id", skip_serializing_if = "is_legacy_mcp_id")]
    pub connection_id: String,
    pub server: String,
    pub tool: String,
    pub arguments: String,
    pub revision: u32,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub credential_names: Vec<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct McpCredentialBinding {
    pub name: String,
    pub credential_id: String,
}
fn valid_credential_id(id: &str) -> bool {
    id.len() == 36
        && id.bytes().enumerate().all(|(i, b)| {
            if [8, 13, 18, 23].contains(&i) {
                b == b'-'
            } else {
                b.is_ascii_hexdigit()
            }
        })
}
pub fn validate_mcp_credential_names(names: &[String]) -> Result<(), String> {
    const RESERVED: &[&str] = &[
        "PATH",
        "HOME",
        "USERPROFILE",
        "SYSTEMROOT",
        "WINDIR",
        "TEMP",
        "TMP",
        "APPDATA",
        "LOCALAPPDATA",
        "LANG",
        "TZ",
        "CI",
        "TERM",
        "NO_COLOR",
    ];
    if names.len() > 8
        || names.iter().enumerate().any(|(i, name)| {
            name.is_empty()
                || name.len() > 64
                || !name.bytes().enumerate().all(|(i, b)| {
                    b == b'_' || b.is_ascii_uppercase() || (i > 0 && b.is_ascii_digit())
                })
                || names[..i].contains(name)
                || RESERVED.contains(&name.as_str())
                || [
                    "LC_",
                    "LD_",
                    "DYLD_",
                    "NODE_",
                    "PYTHON",
                    "RUBY",
                    "PERL",
                    "BASH",
                    "ENV",
                    "COMSPEC",
                    "PATHEXT",
                    "PSMODULEPATH",
                    "JAVA",
                    "JDK_",
                    "DOTNET_",
                    "COR_",
                    "GIT_",
                    "SSH_",
                ]
                .iter()
                .any(|prefix| name.starts_with(prefix))
        })
    {
        return Err("Use up to eight unique uppercase credential names (letters, digits, underscore). Process and runtime settings cannot be overridden.".into());
    }
    Ok(())
}
