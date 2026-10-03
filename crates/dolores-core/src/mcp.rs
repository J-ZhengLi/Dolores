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
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct McpConnection {
    pub revision: u32,
    pub enabled: bool,
    pub launch: McpLaunch,
    pub fingerprints: Vec<McpFingerprint>,
    pub protocol_version: String,
    pub server_name: String,
    pub server_version: String,
    pub tools: Vec<McpTool>,
}
impl McpConnection {
    pub fn validate(&self) -> Result<(), String> {
        self.launch.validate()?;
        if self.revision == 0
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
            name: format!("mcp_tool_{}", index + 1),
            description: format!("External MCP tool {} from {}. Requires approval and starts the reviewed server. {}", tool.name, self.launch.label, tool.description),
            parameters: tool.input_schema.clone(),
        }).collect()
    }
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct McpCallPreview {
    pub server: String,
    pub tool: String,
    pub arguments: String,
    pub revision: u32,
}
