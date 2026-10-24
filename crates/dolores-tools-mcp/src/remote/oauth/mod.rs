//! OAuth protocol boundary for a host-owned, explicit MCP sign-in.
//!
//! This module does not open browsers, persist tokens or dispatch business tools.
//! The host must separately own those lifetimes and bind any saved credentials.
mod challenge;
mod discovery;
mod grant;
#[cfg(test)]
mod tests;

pub use discovery::{
    AuthorizationServer, ClientConfiguration, PreRegisteredClient, RegisteredClient,
    ResourceMetadata,
};
pub use grant::{PendingAuthorization, Tokens};
use reqwest::{header::CONTENT_TYPE, RequestBuilder, Url};
use serde_json::Value;
use std::{fmt, time::Duration};
use tokio::time::Instant;
use tokio_util::sync::CancellationToken;

const DOCUMENT: usize = 32 * 1024;
const TOTAL: usize = 128 * 1024;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    Endpoint,
    Challenge,
    Metadata,
    Resource,
    Issuer,
    Pkce,
    Registration,
    Callback,
    AttemptExpired,
    Denied,
    SignInRequired,
    Tokens,
    Redirect,
    Http(u16),
    Network,
    Limit,
    Cancelled,
    Deadline,
}

/// Never embeds an endpoint, callback, credential or remote error description.
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Failure(pub Kind);
impl fmt::Debug for Failure {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(self, f)
    }
}
impl fmt::Display for Failure {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self.0 {
            Kind::Endpoint => "OAuth requires HTTPS endpoints without URL credentials or fragments. HTTP authorization is available only for an explicitly selected loopback MCP resource.",
            Kind::Challenge => "The MCP authentication challenge is malformed or ambiguous. Ask the server owner to fix it, then inspect again.",
            Kind::Metadata => "OAuth discovery returned invalid or unsupported metadata. Check the server's authorization configuration and inspect again.",
            Kind::Resource => "OAuth metadata identifies a different MCP resource. Verify the endpoint before signing in again.",
            Kind::Issuer => "The authorization server changed or its issuer did not match. Review the connection and sign in again; no credentials were reused.",
            Kind::Pkce => "The authorization server does not advertise S256 PKCE. Ask its owner to enable secure native sign-in.",
            Kind::Registration => "This server needs a registered public client with a loopback redirect. Supply its issuer-bound client ID or configure compatible client registration, then sign in again.",
            Kind::Callback => "The sign-in response did not match this attempt. Start a fresh sign-in; no authorization code was exchanged.",
            Kind::AttemptExpired => "This sign-in attempt exceeded five minutes. Start sign-in again; the previous connection is retained.",
            Kind::Denied => "Sign-in was declined or failed. The previous connection is retained; start sign-in again when ready.",
            Kind::SignInRequired => "The saved OAuth login expired or was revoked. Sign in again; no tool call was retried.",
            Kind::Tokens => "The authorization server returned invalid or unsupported tokens. The previous credentials are retained; sign in again.",
            Kind::Redirect => "An OAuth endpoint redirected. Review its configured address before signing in again; credentials were not forwarded.",
            Kind::Http(_) => "The OAuth endpoint refused the request. Check its configuration and sign in again explicitly.",
            Kind::Network => "The OAuth connection failed. Check connectivity and sign in again explicitly; no request was retried.",
            Kind::Limit => "OAuth exceeded its response or request limit. Ask the server owner for smaller metadata and retry sign-in explicitly.",
            Kind::Cancelled => "OAuth sign-in stopped. The previous connection is retained.",
            Kind::Deadline => "OAuth exceeded its 30-second network allowance. The previous connection is retained; retry sign-in explicitly.",
        })
    }
}
impl std::error::Error for Failure {}
type Result<T> = std::result::Result<T, Failure>;

/// One bounded network phase. Browser/user waiting must use a separate host-owned
/// lifetime; constructing another phase never authorizes a business-call replay.
pub struct Operation {
    cancel: CancellationToken,
    deadline: Instant,
    bytes: usize,
    requests: usize,
}
impl Operation {
    pub fn new(cancel: CancellationToken) -> Self {
        Self::until(cancel, Instant::now() + Duration::from_secs(30))
    }
    pub fn until(cancel: CancellationToken, deadline: Instant) -> Self {
        Self {
            cancel,
            deadline: deadline.min(Instant::now() + Duration::from_secs(30)),
            bytes: 0,
            requests: 0,
        }
    }
    fn active(&self) -> Result<()> {
        if self.cancel.is_cancelled() {
            return Err(Failure(Kind::Cancelled));
        }
        if Instant::now() >= self.deadline {
            return Err(Failure(Kind::Deadline));
        }
        Ok(())
    }
    fn client(url: &Url) -> Result<reqwest::Client> {
        super::http_client(url, reqwest::Client::builder()).map_err(|_| Failure(Kind::Network))
    }
    async fn get(&mut self, url: &Url) -> Result<Value> {
        self.json(Self::client(url)?.get(url.clone())).await
    }
    async fn json(&mut self, request: RequestBuilder) -> Result<Value> {
        self.active()?;
        if self.requests >= 12 {
            return Err(Failure(Kind::Limit));
        }
        self.requests += 1;
        let read = async {
            let mut response = request
                .header(reqwest::header::ACCEPT, "application/json")
                .send()
                .await
                .map_err(|_| Failure(Kind::Network))?;
            if response.status().is_redirection() {
                return Err(Failure(Kind::Redirect));
            }
            let status = response.status().as_u16();
            // Token errors need a bounded parsed body; other HTTP failures don't.
            if !response.status().is_success() && status != 400 && status != 401 {
                return Err(Failure(Kind::Http(status)));
            }
            let media = response
                .headers()
                .get(CONTENT_TYPE)
                .and_then(|v| v.to_str().ok())
                .unwrap_or("")
                .split(';')
                .next()
                .unwrap_or("")
                .trim()
                .to_ascii_lowercase();
            if !(media == "application/json"
                || (media.starts_with("application/") && media.ends_with("+json")))
            {
                return Err(Failure(Kind::Metadata));
            }
            if response
                .content_length()
                .is_some_and(|n| n > DOCUMENT as u64)
            {
                return Err(Failure(Kind::Limit));
            }
            let mut data = Vec::new();
            while let Some(chunk) = response.chunk().await.map_err(|_| Failure(Kind::Network))? {
                self.bytes = self.bytes.saturating_add(chunk.len());
                if data.len().saturating_add(chunk.len()) > DOCUMENT || self.bytes > TOTAL {
                    return Err(Failure(Kind::Limit));
                }
                data.extend_from_slice(&chunk);
            }
            let value: Value =
                serde_json::from_slice(&data).map_err(|_| Failure(Kind::Metadata))?;
            if !value.is_object() {
                return Err(Failure(Kind::Metadata));
            }
            if status == 400 || status == 401 {
                return Err(Failure(if value["error"] == "invalid_grant" {
                    Kind::SignInRequired
                } else {
                    Kind::Http(status)
                }));
            }
            Ok(value)
        };
        tokio::select! {
            biased;
            _ = self.cancel.cancelled() => Err(Failure(Kind::Cancelled)),
            _ = tokio::time::sleep_until(self.deadline) => Err(Failure(Kind::Deadline)),
            result = read => result,
        }
    }
}

fn text(value: &str, limit: usize) -> bool {
    !value.is_empty() && value.len() <= limit && !value.chars().any(char::is_control)
}
fn oauth_url(value: &str, local: bool) -> Result<Url> {
    let url = dolores_core::validate_mcp_endpoint(value).map_err(|_| Failure(Kind::Endpoint))?;
    if url.scheme() != "https" && !local {
        return Err(Failure(Kind::Endpoint));
    }
    Ok(url)
}
fn issuer_url(value: &str, local: bool) -> Result<Url> {
    let url = oauth_url(value, local)?;
    if url.query().is_some() {
        return Err(Failure(Kind::Issuer));
    }
    Ok(url)
}
fn scopes(value: &str) -> Result<Vec<String>> {
    if value.len() > 2048
        || value.bytes().any(|c| {
            !(c == b' ' || c == 0x21 || (0x23..=0x5b).contains(&c) || (0x5d..=0x7e).contains(&c))
        })
    {
        return Err(Failure(Kind::Metadata));
    }
    let mut names: Vec<_> = value
        .split(' ')
        .filter(|s| !s.is_empty())
        .map(str::to_owned)
        .collect();
    names.sort();
    names.dedup();
    if names.len() > 32 {
        return Err(Failure(Kind::Limit));
    }
    Ok(names)
}
fn strings(value: Option<&Value>, max: usize) -> Result<Vec<String>> {
    let Some(value) = value else {
        return Ok(vec![]);
    };
    let array = value
        .as_array()
        .filter(|v| v.len() <= max)
        .ok_or(Failure(Kind::Metadata))?;
    array
        .iter()
        .map(|v| {
            v.as_str()
                .filter(|s| text(s, 2048))
                .map(str::to_owned)
                .ok_or(Failure(Kind::Metadata))
        })
        .collect()
}
