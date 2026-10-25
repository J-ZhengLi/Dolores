//! Secret-bearing persistence stays inside the MCP credential boundary.
use super::{
    discovery::loopback_redirect, grant::Tokens, scopes, AuthorizationServer, ClientConfiguration,
    Operation, PreRegisteredClient, RegisteredClient,
};
use crate::remote::credentials::{canonical_root, BearerCredential};
use dolores_core::{
    CredentialStore, McpConnection, McpRemoteAuthentication, McpRemoteTarget, MCP_REMOTE_OAUTH,
};
use serde::{Deserialize, Serialize};
use std::{
    path::{Path, PathBuf},
    sync::{Arc, OnceLock, Weak},
    time::{SystemTime, UNIX_EPOCH},
};
use tokio::sync::Mutex;
use tokio_util::sync::CancellationToken;

const RECOVERY: &str = "MCP login expired, changed or has an uncertain refresh. Use Sign in again in Connections; no tool call was sent.";
const STORAGE: &str = "MCP login could not be read or saved securely. Unlock secure storage, then Sign in again. No tool call was sent.";
// Host admission serializes config edits against tools. This lock also serializes
// callers inside this library, including separate tools sharing one credential.
static OWNERS: OnceLock<std::sync::Mutex<std::collections::HashMap<String, Weak<Mutex<()>>>>> =
    OnceLock::new();

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Record {
    kind: String,
    root: String,
    connection_id: String,
    target: McpRemoteTarget,
    issuer: String,
    resource: String,
    metadata_url: String,
    client_id: String,
    redirect: String,
    token_endpoint: String,
    access: String,
    refresh: Option<String>,
    expires_at: Option<u64>,
    scopes: Vec<String>,
    refresh_pending: bool,
}
/// No Debug/Serialize/access-token accessor at the host boundary.
pub struct OAuthCredential(Record);
impl OAuthCredential {
    pub async fn connect(
        &self,
        target: &McpRemoteTarget,
        pinned: Option<&str>,
        cancel: CancellationToken,
        deadline: tokio::time::Instant,
    ) -> Result<crate::remote::RemoteClient, String> {
        if target.authentication != McpRemoteAuthentication::Oauth
            || dolores_core::validate_mcp_endpoint(&target.endpoint)?.as_str() != self.0.resource
        {
            return Err(RECOVERY.into());
        }
        crate::remote::RemoteClient::connect_target_until(
            target,
            Some(&self.bearer()?),
            pinned,
            cancel,
            deadline,
        )
        .await
        .map_err(|e| e.to_string())
    }
    pub fn new(
        resource: &super::ResourceMetadata,
        server: &AuthorizationServer,
        client: &RegisteredClient,
        tokens: Tokens,
    ) -> Result<Self, String> {
        if resource.resource() != server.resource()
            || tokens.issuer != server.issuer()
            || tokens.resource != server.resource()
            || tokens.client_id != client.client_id()
        {
            return Err(RECOVERY.into());
        }
        Ok(Self(Record {
            kind: "mcpOAuthV1".into(),
            root: String::new(),
            connection_id: String::new(),
            target: McpRemoteTarget {
                label: "Pending".into(),
                endpoint: server.resource().into(),
                authentication: McpRemoteAuthentication::Oauth,
            },
            issuer: tokens.issuer,
            resource: tokens.resource,
            metadata_url: resource.metadata_url.clone(),
            client_id: tokens.client_id,
            redirect: client.redirect.clone(),
            token_endpoint: server.token.to_string(),
            access: tokens.access,
            refresh: tokens.refresh,
            expires_at: tokens.expires_at,
            scopes: tokens.scopes,
            refresh_pending: false,
        }))
    }
    pub fn encoded_for(
        &self,
        root: &Path,
        connection_id: &str,
        target: &McpRemoteTarget,
    ) -> Result<String, String> {
        target.validate()?;
        if target.authentication != McpRemoteAuthentication::Oauth
            || dolores_core::validate_mcp_endpoint(&target.endpoint)?.as_str() != self.0.resource
            || connection_id == "legacy"
            || !dolores_core::valid_mcp_id(connection_id)
            || self.0.refresh_pending
            || self.contains_secret(&serde_json::to_value(target).map_err(|_| STORAGE)?)
        {
            return Err(RECOVERY.into());
        }
        let mut value = serde_json::to_value(&self.0).map_err(|_| STORAGE)?;
        value["root"] = canonical_root(root)?.into();
        value["connectionId"] = connection_id.into();
        value["target"] = serde_json::to_value(target).map_err(|_| STORAGE)?;
        encode(&value)
    }
    pub fn contains_secret(&self, value: &serde_json::Value) -> bool {
        super::super::contains(value, &self.0.access)
            || self
                .0
                .refresh
                .as_ref()
                .is_some_and(|s| super::super::contains(value, s))
    }
    pub(crate) fn bearer(&self) -> Result<BearerCredential, String> {
        BearerCredential::new(self.0.access.clone())
    }
    fn tokens(&self) -> Tokens {
        Tokens {
            access: self.0.access.clone(),
            refresh: self.0.refresh.clone(),
            expires_at: self.0.expires_at,
            scopes: self.0.scopes.clone(),
            issuer: self.0.issuer.clone(),
            resource: self.0.resource.clone(),
            client_id: self.0.client_id.clone(),
        }
    }
}
fn encode(value: &impl Serialize) -> Result<String, String> {
    let encoded = serde_json::to_string(value).map_err(|_| STORAGE)?;
    if encoded.len() > 16 * 1024 {
        return Err(STORAGE.into());
    }
    Ok(encoded)
}
async fn read(
    root: PathBuf,
    connection: McpConnection,
    vault: Arc<dyn CredentialStore>,
) -> Result<(OAuthCredential, String), String> {
    tokio::task::spawn_blocking(move || {
        connection.validate()?;
        let target = connection.remote.as_ref().ok_or(RECOVERY)?;
        if target.authentication != McpRemoteAuthentication::Oauth
            || connection.credentials.len() != 1
            || connection.credentials[0].name != MCP_REMOTE_OAUTH
        {
            return Err(RECOVERY.into());
        }
        let id = connection.credentials[0].credential_id.clone();
        let raw = vault.read(&id).map_err(|_| STORAGE)?.ok_or(RECOVERY)?;
        if raw.len() > 16 * 1024 {
            return Err(STORAGE.into());
        }
        let record: Record = serde_json::from_str(&raw).map_err(|_| STORAGE)?;
        if record.kind != "mcpOAuthV1"
            || record.root != canonical_root(&root)?
            || record.connection_id != connection.id
            || &record.target != target
            || record.resource != dolores_core::validate_mcp_endpoint(&target.endpoint)?.as_str()
            || record.refresh_pending
        {
            return Err(RECOVERY.into());
        }
        super::issuer_url(&record.issuer, record.resource.starts_with("http://"))
            .map_err(|_| RECOVERY)?;
        super::oauth_url(
            &record.token_endpoint,
            record.resource.starts_with("http://"),
        )
        .map_err(|_| RECOVERY)?;
        loopback_redirect(&record.redirect).map_err(|_| RECOVERY)?;
        if record.client_id.is_empty()
            || record.client_id.len() > 2048
            || record.client_id.chars().any(char::is_control)
            || scopes(&record.scopes.join(" ")).map_err(|_| RECOVERY)? != record.scopes
        {
            return Err(RECOVERY.into());
        }
        BearerCredential::new(record.access.clone())?;
        if let Some(refresh) = &record.refresh {
            BearerCredential::new(refresh.clone())?;
        }
        let result = OAuthCredential(record);
        if result.contains_secret(&serde_json::to_value(target).map_err(|_| STORAGE)?) {
            return Err(RECOVERY.into());
        }
        Ok((result, id))
    })
    .await
    .map_err(|_| STORAGE)?
}
fn write(vault: &dyn CredentialStore, id: &str, encoded: String) -> Result<(), String> {
    // Do not detach writes into cancellable blocking jobs. The owner must remain
    // held until this small native-vault transaction returns, even on Stop.
    vault.write(id, &encoded).map_err(|_| STORAGE.into())
}
/// Resolve after approval, or during explicit foreground Recheck. Refresh at most
/// once before any business call, never in response to a dispatched call's error.
pub async fn resolve(
    root: PathBuf,
    connection: McpConnection,
    vault: Arc<dyn CredentialStore>,
    cancel: CancellationToken,
) -> Result<OAuthCredential, String> {
    let key = connection
        .credentials
        .first()
        .ok_or(RECOVERY)?
        .credential_id
        .clone();
    let owner = {
        let mut owners = OWNERS
            .get_or_init(Default::default)
            .lock()
            .map_err(|_| RECOVERY)?;
        owners.retain(|_, v| v.strong_count() > 0);
        let owner = owners
            .get(&key)
            .and_then(Weak::upgrade)
            .unwrap_or_else(|| Arc::new(Mutex::new(())));
        owners.insert(key, Arc::downgrade(&owner));
        owner
    };
    let _owner = owner
        .try_lock_owned()
        .map_err(|_| "MCP login is already in use. Wait for that operation, then recheck.")?;
    if cancel.is_cancelled() {
        return Err(RECOVERY.into());
    }
    let deadline = tokio::time::Instant::now() + std::time::Duration::from_secs(30);
    let (mut credential, id) = tokio::select! {
        biased;
        _=cancel.cancelled()=>return Err(RECOVERY.into()),
        _=tokio::time::sleep_until(deadline)=>return Err(STORAGE.into()),
        result=read(root,connection,vault.clone())=>result?,
    };
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|_| RECOVERY)?
        .as_secs();
    if credential
        .0
        .expires_at
        .is_none_or(|expiry| expiry > now.saturating_add(30))
    {
        return Ok(credential);
    }
    if credential.0.refresh.is_none() {
        return Err(RECOVERY.into());
    }
    let mut operation = Operation::until(cancel.clone(), deadline);
    // Retain the exact discovered metadata address, including challenged paths.
    // Parsing it through the normal challenge boundary preserves URL policy.
    let challenge = format!(
        "Bearer resource_metadata={}",
        serde_json::to_string(&credential.0.metadata_url).map_err(|_| RECOVERY)?
    );
    let resource = operation
        .discover_resource(&credential.0.resource, Some(&challenge))
        .await
        .map_err(|e| e.to_string())?;
    let server = operation
        .discover_server(&resource, &credential.0.issuer)
        .await
        .map_err(|e| e.to_string())?;
    if server.token.as_str() != credential.0.token_endpoint {
        return Err(RECOVERY.into());
    }
    let client = RegisteredClient {
        issuer: credential.0.issuer.clone(),
        resource: credential.0.resource.clone(),
        client_id: credential.0.client_id.clone(),
        redirect: credential.0.redirect.clone(),
    };
    // A durable intent is required even if a refresh is cancelled at the server.
    credential.0.refresh_pending = true;
    write(vault.as_ref(), &id, encode(&credential.0)?)?;
    let rotated = operation
        .refresh(&server, &client, &credential.tokens())
        .await
        .map_err(|_| RECOVERY)?;
    credential.0.access = rotated.access;
    credential.0.refresh = rotated.refresh;
    credential.0.expires_at = rotated.expires_at;
    credential.0.scopes = rotated.scopes;
    credential.0.refresh_pending = false;
    // Persist before activation. A failed write leaves the pending marker (or an
    // uncertain successful write); neither path repeats a business operation.
    write(vault.as_ref(), &id, encode(&credential.0)?)?;
    if cancel.is_cancelled() {
        return Err(RECOVERY.into());
    }
    Ok(credential)
}

/// Optional issuer-bound registration inputs; never a client secret.
#[derive(Default, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SignInConfiguration {
    pub issuer: Option<String>,
    pub client_id: Option<String>,
    pub metadata_document: Option<String>,
}
impl SignInConfiguration {
    pub fn validate(&self) -> Result<(), String> {
        for value in [&self.issuer, &self.client_id, &self.metadata_document]
            .into_iter()
            .flatten()
        {
            if value.is_empty() || value.len() > 2048 || value.chars().any(char::is_control) {
                return Err(
                    "Use bounded issuer/client registration fields without control characters."
                        .into(),
                );
            }
        }
        if self.client_id.is_some() && self.issuer.is_none() {
            return Err("A supplied client ID requires its exact registered issuer.".into());
        }
        Ok(())
    }
    pub fn client_configuration(&self) -> ClientConfiguration {
        ClientConfiguration {
            pre_registered: self.client_id.as_ref().zip(self.issuer.as_ref()).map(
                |(client_id, issuer)| PreRegisteredClient {
                    issuer: issuer.clone(),
                    client_id: client_id.clone(),
                },
            ),
            metadata_document: self.metadata_document.clone(),
        }
    }
}
