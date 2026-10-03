//! Credential values live only in the native vault and short-lived process memory.
use dolores_core::{CredentialStore, McpConnection, McpFingerprint, McpLaunch};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::path::Path;

pub const VAULT_ERROR: &str = "MCP secure storage is unavailable or the key is missing. Unlock storage or enter the key again, then inspect and enable the connection.";

// Intentionally no Debug or Serialize: this is an inbound, ephemeral value.
#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CredentialInput {
    pub name: String,
    pub value: Option<String>,
}
#[derive(Clone)]
pub struct Credentials(pub(crate) Vec<(String, String)>);
impl Credentials {
    pub fn empty() -> Self {
        Self(vec![])
    }
    pub fn names(&self) -> Vec<String> {
        self.0.iter().map(|(n, _)| n.clone()).collect()
    }
    pub fn value(&self, name: &str) -> Option<String> {
        self.0
            .iter()
            .find(|(n, _)| n == name)
            .map(|(_, v)| v.clone())
    }
    pub fn new(values: Vec<(String, String)>) -> Result<Self, String> {
        dolores_core::validate_mcp_credential_names(
            &values.iter().map(|(n, _)| n.clone()).collect::<Vec<_>>(),
        )?;
        if values
            .iter()
            .any(|(_, v)| v.is_empty() || v.len() > 4096 || v.contains('\0'))
            || values.iter().map(|(_, v)| v.len()).sum::<usize>() > 16384
        {
            return Err(
                "MCP credentials need nonempty values within 4 KiB each and 16 KiB total.".into(),
            );
        }
        Ok(Self(values))
    }
    pub fn validate_launch(&self, launch: &McpLaunch) -> Result<(), String> {
        if self.0.iter().any(|(_, value)| {
            launch.label.contains(value)
                || launch.executable.contains(value)
                || launch.args.iter().any(|arg| arg.contains(value))
        }) {
            return Err(
                "Keep credential values out of the server name, executable and arguments.".into(),
            );
        }
        Ok(())
    }
    pub(crate) fn reject_metadata(&self, value: &Value) -> Result<(), String> {
        fn contains(value: &Value, key: &str) -> bool {
            match value {
                Value::String(s) => s.contains(key),
                Value::Array(a) => a.iter().any(|v| contains(v, key)),
                Value::Object(o) => o.iter().any(|(k, v)| k.contains(key) || contains(v, key)),
                Value::Number(n) => n.to_string().contains(key),
                _ => false,
            }
        }
        if self.0.iter().any(|(_, key)| contains(value, key)) {
            return Err("MCP server exposed a credential in metadata. Nothing was shared.".into());
        }
        Ok(())
    }
    pub(crate) fn redact(&self, text: &str) -> String {
        // Work on original text; never re-scan a replacement marker. A bounded
        // byte mask unions overlapping secrets without quadratic rescanning.
        let mut covered = vec![false; text.len()];
        for (_, key) in &self.0 {
            for (start, _) in text.match_indices(key) {
                covered[start..start + key.len()].fill(true);
            }
        }
        let mut result = String::new();
        let mut offset = 0;
        while offset < text.len() {
            let masked = covered[offset];
            let end = covered[offset..]
                .iter()
                .position(|v| *v != masked)
                .map_or(text.len(), |n| offset + n);
            if masked {
                result.push_str("[redacted]");
            } else {
                result.push_str(&text[offset..end]);
            }
            offset = end;
        }
        result
    }
    pub fn encoded(
        &self,
        name: &str,
        root: &Path,
        launch: &McpLaunch,
        fingerprints: &[McpFingerprint],
    ) -> Result<String, String> {
        self.encoded_for("legacy", name, root, launch, fingerprints)
    }
    pub fn encoded_for(
        &self,
        connection_id: &str,
        name: &str,
        root: &Path,
        launch: &McpLaunch,
        fingerprints: &[McpFingerprint],
    ) -> Result<String, String> {
        let value = self
            .0
            .iter()
            .find(|(n, _)| n == name)
            .ok_or(VAULT_ERROR)?
            .1
            .clone();
        serde_json::to_string(&StoredCredential {
            connection_id: connection_id.into(),
            root: root
                .canonicalize()
                .map_err(|_| "Working folder is unavailable.")?
                .to_string_lossy()
                .into_owned(),
            launch: launch.clone(),
            fingerprints: fingerprints.to_vec(),
            name: name.into(),
            value,
        })
        .map_err(|_| VAULT_ERROR.into())
    }
}
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct StoredCredential {
    #[serde(default = "dolores_core::legacy_mcp_id")]
    connection_id: String,
    root: String,
    launch: McpLaunch,
    fingerprints: Vec<McpFingerprint>,
    name: String,
    value: String,
}
pub fn resolve(
    root: &Path,
    connection: &McpConnection,
    vault: &dyn CredentialStore,
) -> Result<Credentials, String> {
    let root = root
        .canonicalize()
        .map_err(|_| "Working folder is unavailable.")?;
    let mut values = vec![];
    for binding in &connection.credentials {
        let raw = vault
            .read(&binding.credential_id)
            .map_err(|_| VAULT_ERROR)?
            .ok_or(VAULT_ERROR)?;
        let stored: StoredCredential = serde_json::from_str(&raw).map_err(|_| VAULT_ERROR)?;
        if stored.connection_id != connection.id
            || stored.root != root.to_string_lossy()
            || stored.launch != connection.launch
            || stored.fingerprints != connection.fingerprints
            || stored.name != binding.name
        {
            return Err("MCP credential binding changed. Enter the key again, then inspect and enable this connection.".into());
        }
        values.push((binding.name.clone(), stored.value));
    }
    let credentials = Credentials::new(values)?;
    credentials.validate_launch(&connection.launch)?;
    Ok(credentials)
}
