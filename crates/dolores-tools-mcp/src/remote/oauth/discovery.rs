use super::{
    challenge, issuer_url, oauth_url, scopes, strings, text, Failure, Kind, Operation, Result,
};
use reqwest::Url;
use serde_json::{json, Value};

#[derive(Clone)]
pub struct ResourceMetadata {
    pub(super) resource: String,
    pub(super) issuers: Vec<String>,
    pub(super) scopes: Vec<String>,
    pub(super) local: bool,
}
impl ResourceMetadata {
    pub fn resource(&self) -> &str {
        &self.resource
    }
    pub fn issuers(&self) -> &[String] {
        &self.issuers
    }
    pub fn scopes(&self) -> &[String] {
        &self.scopes
    }
}

#[derive(Clone)]
pub struct AuthorizationServer {
    pub(super) resource: String,
    pub(super) issuer: String,
    pub(super) authorization: Url,
    pub(super) token: Url,
    registration: Option<Url>,
    pub(super) scopes: Vec<String>,
    pub(super) issuer_in_callback: bool,
    client_metadata: bool,
}
impl AuthorizationServer {
    pub fn issuer(&self) -> &str {
        &self.issuer
    }
    pub fn resource(&self) -> &str {
        &self.resource
    }
    pub fn scopes(&self) -> &[String] {
        &self.scopes
    }
}

#[derive(Default)]
pub struct ClientConfiguration {
    pub pre_registered: Option<PreRegisteredClient>,
    /// A real, externally hosted metadata document. No built-in URL is assumed.
    pub metadata_document: Option<String>,
}
pub struct PreRegisteredClient {
    pub issuer: String,
    pub client_id: String,
}
#[derive(Clone)]
pub struct RegisteredClient {
    pub(super) issuer: String,
    pub(super) resource: String,
    pub(super) client_id: String,
    pub(super) redirect: String,
}
impl RegisteredClient {
    pub fn client_id(&self) -> &str {
        &self.client_id
    }
    pub fn issuer(&self) -> &str {
        &self.issuer
    }
}

fn well_known(base: &Url, suffix: &str, append: bool) -> Url {
    let mut url = base.clone();
    url.set_query(None);
    let path = base.path();
    let path = if path == "/" { "" } else { path };
    url.set_path(&if append {
        format!("{}/.well-known/{suffix}", path.trim_end_matches('/'))
    } else {
        format!("/.well-known/{suffix}{path}")
    });
    url
}
fn boolean(value: &Value, name: &str) -> Result<bool> {
    value
        .get(name)
        .map(|v| v.as_bool().ok_or(Failure(Kind::Metadata)))
        .unwrap_or(Ok(false))
}
fn metadata_scopes(value: &Value) -> Result<Vec<String>> {
    let names = strings(value.get("scopes_supported"), 32)?;
    if names.iter().any(|s| s.contains(' ')) {
        return Err(Failure(Kind::Metadata));
    }
    scopes(&names.join(" "))
}
fn endpoint(value: &Value, name: &str, local: bool) -> Result<Url> {
    oauth_url(value[name].as_str().ok_or(Failure(Kind::Metadata))?, local)
}

impl Operation {
    pub async fn discover_resource(
        &mut self,
        resource: &str,
        challenge: Option<&str>,
    ) -> Result<ResourceMetadata> {
        self.active()?;
        let resource = oauth_url(resource, true)?;
        // Insecure authorization fixtures cannot be introduced by a public MCP
        // endpoint's metadata. Core validation already restricts HTTP to loopback.
        let local = resource.scheme() == "http";
        let challenge = challenge::parse(challenge)?;
        let urls = if let Some(metadata) = challenge.metadata {
            vec![oauth_url(&metadata, local)?]
        } else {
            let scoped = well_known(&resource, "oauth-protected-resource", false);
            let mut root = resource.clone();
            root.set_path("/");
            let root = well_known(&root, "oauth-protected-resource", false);
            if scoped == root {
                vec![root]
            } else {
                vec![scoped, root]
            }
        };
        let value = self.first_metadata(&urls).await?;
        let identified = oauth_url(
            value["resource"].as_str().ok_or(Failure(Kind::Resource))?,
            local,
        )?;
        if identified != resource {
            return Err(Failure(Kind::Resource));
        }
        let issuers = strings(value.get("authorization_servers"), 8)?;
        if issuers.is_empty() {
            return Err(Failure(Kind::Metadata));
        }
        for issuer in &issuers {
            issuer_url(issuer, local)?;
        }
        let methods = strings(value.get("bearer_methods_supported"), 8)?;
        if value.get("bearer_methods_supported").is_some() && !methods.iter().any(|m| m == "header")
        {
            return Err(Failure(Kind::Metadata));
        }
        Ok(ResourceMetadata {
            resource: resource.to_string(),
            issuers,
            scopes: challenge.scopes.unwrap_or(metadata_scopes(&value)?),
            local,
        })
    }
    async fn first_metadata(&mut self, urls: &[Url]) -> Result<Value> {
        for url in urls {
            match self.get(url).await {
                Err(Failure(Kind::Http(404 | 405))) => continue,
                value => return value,
            }
        }
        Err(Failure(Kind::Metadata))
    }
    pub async fn discover_server(
        &mut self,
        resource: &ResourceMetadata,
        issuer: &str,
    ) -> Result<AuthorizationServer> {
        self.active()?;
        if !resource.issuers.iter().any(|s| s == issuer) {
            return Err(Failure(Kind::Issuer));
        }
        let base = issuer_url(issuer, resource.local)?;
        let mut urls = vec![
            well_known(&base, "oauth-authorization-server", false),
            well_known(&base, "openid-configuration", false),
        ];
        let appended = well_known(&base, "openid-configuration", true);
        if !urls.contains(&appended) {
            urls.push(appended);
        }
        let value = self.first_metadata(&urls).await?;
        // String comparison is deliberate: URL normalization hides mix-up
        // differences such as trailing slash, host case and explicit default port.
        if value["issuer"].as_str() != Some(issuer) {
            return Err(Failure(Kind::Issuer));
        }
        if !strings(value.get("code_challenge_methods_supported"), 8)?
            .iter()
            .any(|m| m == "S256")
        {
            return Err(Failure(Kind::Pkce));
        }
        if !strings(value.get("response_types_supported"), 16)?
            .iter()
            .any(|m| m == "code")
            || (value.get("grant_types_supported").is_some()
                && !strings(value.get("grant_types_supported"), 16)?
                    .iter()
                    .any(|m| m == "authorization_code"))
            || !strings(value.get("token_endpoint_auth_methods_supported"), 16)?
                .iter()
                .any(|m| m == "none")
        {
            return Err(Failure(Kind::Registration));
        }
        Ok(AuthorizationServer {
            resource: resource.resource.clone(),
            issuer: issuer.to_owned(),
            authorization: endpoint(&value, "authorization_endpoint", resource.local)?,
            token: endpoint(&value, "token_endpoint", resource.local)?,
            registration: value
                .get("registration_endpoint")
                .map(|_| endpoint(&value, "registration_endpoint", resource.local))
                .transpose()?,
            scopes: resource.scopes.clone(),
            issuer_in_callback: boolean(&value, "authorization_response_iss_parameter_supported")?,
            client_metadata: boolean(&value, "client_id_metadata_document_supported")?,
        })
    }
    pub async fn register_client(
        &mut self,
        server: &AuthorizationServer,
        redirect: &str,
        configuration: &ClientConfiguration,
    ) -> Result<RegisteredClient> {
        self.active()?;
        loopback_redirect(redirect)?;
        let client_id = if let Some(client) = &configuration.pre_registered {
            if client.issuer != server.issuer {
                return Err(Failure(Kind::Issuer));
            }
            if !text(&client.client_id, 2048) {
                return Err(Failure(Kind::Registration));
            }
            client.client_id.clone()
        } else if let Some(document) = configuration
            .metadata_document
            .as_ref()
            .filter(|_| server.client_metadata)
        {
            let url = oauth_url(document, false)?;
            if url.path() == "/" || url.query().is_some() {
                return Err(Failure(Kind::Registration));
            }
            let value = self.get(&url).await?;
            validate_client_document(&value, document, redirect)?;
            document.clone()
        } else if let Some(url) = &server.registration {
            let value = self.json(Self::client(url)?.post(url.clone()).json(&json!({
                "client_name":"Dolores", "application_type":"native", "redirect_uris":[redirect],
                "grant_types":["authorization_code","refresh_token"], "response_types":["code"],
                "token_endpoint_auth_method":"none"
            }))).await?;
            // A public native app cannot keep a server-issued client secret.
            if value.get("client_secret").is_some()
                || value["token_endpoint_auth_method"] != "none"
                || !strings(value.get("redirect_uris"), 8)?
                    .iter()
                    .any(|uri| uri == redirect)
            {
                return Err(Failure(Kind::Registration));
            }
            value["client_id"]
                .as_str()
                .filter(|s| text(s, 2048))
                .ok_or(Failure(Kind::Registration))?
                .to_owned()
        } else {
            return Err(Failure(Kind::Registration));
        };
        Ok(RegisteredClient {
            issuer: server.issuer.clone(),
            resource: server.resource.clone(),
            client_id,
            redirect: redirect.to_owned(),
        })
    }
}

pub(super) fn loopback_redirect(value: &str) -> Result<Url> {
    let url = oauth_url(value, true)?;
    let ip = url
        .host_str()
        .unwrap_or("")
        .trim_start_matches('[')
        .trim_end_matches(']')
        .parse::<std::net::IpAddr>();
    if url.scheme() != "http"
        || !ip.is_ok_and(|ip| ip.is_loopback())
        || url.query().is_some()
        || url.port() == Some(0)
        || url.path() == "/"
    {
        return Err(Failure(Kind::Callback));
    }
    Ok(url)
}
fn redirect_matches(registered: &str, actual: &str) -> bool {
    let (Ok(mut registered), Ok(mut actual)) =
        (loopback_redirect(registered), loopback_redirect(actual))
    else {
        return false;
    };
    // RFC 8252: only the port may vary for an IP-literal loopback redirect.
    let _ = registered.set_port(None);
    let _ = actual.set_port(None);
    registered == actual
}

pub(super) fn validate_client_document(
    value: &Value,
    document: &str,
    redirect: &str,
) -> Result<()> {
    if value["client_id"].as_str() != Some(document)
        || !value["client_name"].as_str().is_some_and(|s| text(s, 128))
        || value["token_endpoint_auth_method"] != "none"
        || !strings(value.get("grant_types"), 8)?
            .iter()
            .any(|v| v == "authorization_code")
        || !strings(value.get("response_types"), 8)?
            .iter()
            .any(|v| v == "code")
        || !strings(value.get("redirect_uris"), 8)?
            .iter()
            .any(|uri| redirect_matches(uri, redirect))
    {
        return Err(Failure(Kind::Registration));
    }
    Ok(())
}
