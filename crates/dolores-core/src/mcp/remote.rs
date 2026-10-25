use serde::{Deserialize, Serialize};

/// Credential purpose within a remote target, never an environment override.
pub const MCP_REMOTE_BEARER: &str = "MCP_BEARER_TOKEN";
pub const MCP_REMOTE_OAUTH: &str = "MCP_OAUTH_GRANT";

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum McpRemoteAuthentication {
    None,
    Bearer,
    Oauth,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct McpRemoteTarget {
    pub label: String,
    pub endpoint: String,
    pub authentication: McpRemoteAuthentication,
}
impl McpRemoteTarget {
    pub fn validate(&self) -> Result<(), String> {
        if self.label.trim().is_empty()
            || self.label.len() > 128
            || self.label.chars().any(char::is_control)
        {
            return Err(
                "Choose a short remote MCP connection name without control characters.".into(),
            );
        }
        validate_mcp_endpoint(&self.endpoint).map(|_| ())
    }
}

pub fn validate_mcp_endpoint(text: &str) -> Result<url::Url, String> {
    const ERROR:&str="Choose an HTTPS MCP endpoint without URL credentials or fragments. Plain HTTP is available only for loopback endpoints.";
    if text.len() > 2048 || text.chars().any(char::is_control) {
        return Err(ERROR.into());
    }
    let url = url::Url::parse(text).map_err(|_| ERROR.to_owned())?;
    let loopback = match url.host() {
        Some(url::Host::Ipv4(ip)) => ip.is_loopback(),
        Some(url::Host::Ipv6(ip)) => ip.is_loopback(),
        Some(url::Host::Domain(host)) => matches!(host, "localhost" | "localhost."),
        _ => false,
    };
    if url.host().is_none()
        || !url.username().is_empty()
        || url.password().is_some()
        || url.fragment().is_some()
        || !(url.scheme() == "https" || url.scheme() == "http" && loopback)
    {
        return Err(ERROR.into());
    }
    Ok(url)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{McpConnection, McpCredentialBinding, McpFingerprint, McpLaunch, McpTool};
    use serde_json::json;
    fn local() -> McpConnection {
        let executable = if cfg!(windows) {
            "C:/public/server.exe"
        } else {
            "/public/server.exe"
        };
        McpConnection {
            id: "legacy".into(),
            revision: 1,
            enabled: true,
            launch: Some(McpLaunch {
                label: "Public local".into(),
                executable: executable.into(),
                args: vec![],
                isolation: None,
            }),
            remote: None,
            fingerprints: vec![McpFingerprint {
                path: executable.into(),
                sha256: "a".repeat(64),
            }],
            protocol_version: "2025-11-25".into(),
            server_name: "Public".into(),
            server_version: "1".into(),
            tools: vec![McpTool {
                name: "echo".into(),
                description: "".into(),
                input_schema: json!({"type":"object"}),
            }],
            credentials: vec![],
            retired_credentials: vec![],
        }
    }
    #[test]
    fn legacy_shape_is_unchanged_and_targets_are_exclusive() {
        let mut value = local();
        value.validate().unwrap();
        let old = json!({"revision":1,"enabled":true,"launch":{"label":"Public local","executable":value.local_launch().unwrap().executable,"args":[]},"fingerprints":[{"path":value.local_launch().unwrap().executable,"sha256":"a".repeat(64)}],"protocolVersion":"2025-11-25","serverName":"Public","serverVersion":"1","tools":[{"name":"echo","description":"","inputSchema":{"type":"object"}}]});
        assert!(old.get("remote").is_none());
        assert!(old["launch"].is_object());
        assert!(old.get("id").is_none());
        assert_eq!(
            serde_json::from_value::<McpConnection>(old.clone()).unwrap(),
            value
        );
        value.launch = None;
        assert!(value.validate().is_err());
        value.remote = Some(McpRemoteTarget {
            label: "Remote".into(),
            endpoint: "https://example.test/mcp".into(),
            authentication: McpRemoteAuthentication::None,
        });
        assert!(value.validate().is_err()); // Legacy identity and local fingerprints cannot become a remote grant.
        value.id = "00000000-0000-4000-8000-000000000001".into();
        value.fingerprints.clear();
        value.protocol_version = "2026-07-28".into();
        value.validate().unwrap();
        assert!(value.local_launch().is_err());
        assert!(value.specs()[0].description.contains("reviewed endpoint"));
        value.launch = local().launch;
        assert!(value.validate().is_err());
        assert_eq!(serde_json::to_value(local()).unwrap(), old);
    }
    #[test]
    fn remote_authentication_requires_only_its_bound_purpose() {
        let mut value = local();
        value.id = "00000000-0000-4000-8000-000000000001".into();
        value.launch = None;
        value.fingerprints.clear();
        value.remote = Some(McpRemoteTarget {
            label: "Remote".into(),
            endpoint: "http://[::1]/mcp".into(),
            authentication: McpRemoteAuthentication::Bearer,
        });
        assert!(value.validate().is_err());
        value.credentials.push(McpCredentialBinding {
            name: MCP_REMOTE_BEARER.into(),
            credential_id: "00000000-0000-4000-8000-000000000002".into(),
        });
        value.validate().unwrap();
        value.credentials[0].name = "SERVICE_API_KEY".into();
        assert!(value.validate().is_err());
        value.credentials[0].name = MCP_REMOTE_BEARER.into();
        value.remote.as_mut().unwrap().authentication = McpRemoteAuthentication::None;
        assert!(value.validate().is_err());
        value.credentials.clear();
        value.remote.as_mut().unwrap().endpoint = "http://example.test/mcp".into();
        assert!(value.validate().is_err());
    }
}
