//! Remote bearer values are bound vault payloads, never saved target fields.
use crate::credentials::VAULT_ERROR;
use dolores_core::{
    CredentialStore, McpConnection, McpRemoteAuthentication, McpRemoteTarget, MCP_REMOTE_BEARER,
};
use serde::{Deserialize, Serialize};
use std::path::Path;

// Deliberately no Debug or Serialize on the inbound credential.
pub struct BearerCredential(String);
impl BearerCredential {
    pub fn new(value: String) -> Result<Self, String> {
        if value.is_empty()
            || value.len() > 4096
            || !value.bytes().all(|b| (0x21..=0x7e).contains(&b))
        {
            return Err("Use a nonempty MCP bearer value within 4 KiB, without whitespace or control characters.".into());
        }
        Ok(Self(value))
    }
    pub fn validate_target(&self, target: &McpRemoteTarget) -> Result<(), String> {
        target.validate()?;
        let url = dolores_core::validate_mcp_endpoint(&target.endpoint)?;
        if target.authentication != McpRemoteAuthentication::Bearer
            || target.label.contains(&self.0)
            || url.as_str().contains(&self.0)
            || url
                .query_pairs()
                .any(|(k, v)| k.contains(&self.0) || v.contains(&self.0))
        {
            return Err("Keep the bearer value in the separate authentication field, not the connection name or endpoint.".into());
        }
        Ok(())
    }
    pub fn encoded_for(
        &self,
        root: &Path,
        connection_id: &str,
        target: &McpRemoteTarget,
    ) -> Result<String, String> {
        self.validate_target(target)?;
        if connection_id == "legacy" || !dolores_core::valid_mcp_id(connection_id) {
            return Err("Remote MCP connection identity is invalid.".into());
        }
        let value = serde_json::to_string(&StoredRemote::BearerV1 {
            connection_id: connection_id.into(),
            root: canonical_root(root)?,
            target: target.clone(),
            value: self.0.clone(),
        })
        .map_err(|_| VAULT_ERROR)?;
        if value.len() > 16 * 1024 {
            return Err(VAULT_ERROR.into());
        }
        Ok(value)
    }
    pub(super) fn expose(&self) -> &str {
        &self.0
    }
}
#[derive(Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "camelCase", deny_unknown_fields)]
enum StoredRemote {
    #[serde(rename_all = "camelCase")]
    BearerV1 {
        connection_id: String,
        root: String,
        target: McpRemoteTarget,
        value: String,
    },
}
fn canonical_root(root: &Path) -> Result<String, String> {
    let root = root
        .canonicalize()
        .map_err(|_| "Working folder is unavailable.")?;
    let text = root
        .to_str()
        .filter(|s| s.len() <= 4096)
        .ok_or("Working folder path is unsupported.")?;
    Ok(text.into())
}
pub fn resolve_bearer(
    root: &Path,
    connection: &McpConnection,
    vault: &dyn CredentialStore,
) -> Result<BearerCredential, String> {
    connection.validate()?;
    let target = connection
        .remote
        .as_ref()
        .ok_or("This is not a remote MCP connection.")?;
    if target.authentication != McpRemoteAuthentication::Bearer
        || connection.credentials.len() != 1
        || connection.credentials[0].name != MCP_REMOTE_BEARER
    {
        return Err(VAULT_ERROR.into());
    }
    let raw = vault
        .read(&connection.credentials[0].credential_id)
        .map_err(|_| VAULT_ERROR)?
        .ok_or(VAULT_ERROR)?;
    if raw.len() > 16 * 1024 {
        return Err(VAULT_ERROR.into());
    }
    let StoredRemote::BearerV1 {
        connection_id,
        root: saved_root,
        target: saved_target,
        value,
    } = serde_json::from_str(&raw).map_err(|_| VAULT_ERROR)?;
    if connection_id != connection.id
        || saved_root != canonical_root(root)?
        || &saved_target != target
    {
        return Err("Remote MCP credential binding changed. Enter the key again, then inspect and enable this endpoint.".into());
    }
    let credential = BearerCredential::new(value)?;
    credential.validate_target(target)?;
    Ok(credential)
}
