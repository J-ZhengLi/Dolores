use super::{
    discovery::loopback_redirect, scopes, text, AuthorizationServer, Failure, Kind, Operation,
    RegisteredClient, Result,
};
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
use reqwest::Url;
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::{
    collections::BTreeMap,
    time::{Duration, SystemTime, UNIX_EPOCH},
};
use tokio::time::Instant;

/// Ephemeral per-attempt secrets, deliberately neither Debug nor Serialize/Clone.
pub struct PendingAuthorization {
    server: AuthorizationServer,
    client: RegisteredClient,
    state: String,
    verifier: String,
    url: Url,
    expires: Instant,
}
fn random_secret() -> String {
    // Three OS-random UUIDs retain 366 random bits after fixed version/variant
    // bits, exceeding PKCE's 256-bit entropy recommendation. Encoding is unpadded.
    let mut bytes = [0_u8; 48];
    for part in bytes.as_chunks_mut::<16>().0 {
        part.copy_from_slice(uuid::Uuid::new_v4().as_bytes());
    }
    URL_SAFE_NO_PAD.encode(bytes)
}
impl PendingAuthorization {
    pub fn new(server: AuthorizationServer, client: RegisteredClient) -> Result<Self> {
        if client.issuer != server.issuer || client.resource != server.resource {
            return Err(Failure(Kind::Issuer));
        }
        loopback_redirect(&client.redirect)?;
        let verifier = random_secret();
        let state = random_secret();
        let challenge = URL_SAFE_NO_PAD.encode(Sha256::digest(verifier.as_bytes()));
        let mut url = server.authorization.clone();
        let reserved = [
            "response_type",
            "client_id",
            "redirect_uri",
            "resource",
            "scope",
            "state",
            "code_challenge",
            "code_challenge_method",
        ];
        if url
            .query_pairs()
            .any(|(key, _)| reserved.contains(&key.as_ref()))
        {
            return Err(Failure(Kind::Metadata));
        }
        {
            let mut params = url.query_pairs_mut();
            params
                .append_pair("response_type", "code")
                .append_pair("client_id", &client.client_id)
                .append_pair("redirect_uri", &client.redirect)
                .append_pair("resource", &server.resource)
                .append_pair("state", &state)
                .append_pair("code_challenge", &challenge)
                .append_pair("code_challenge_method", "S256");
            if !server.scopes.is_empty() {
                params.append_pair("scope", &server.scopes.join(" "));
            }
        }
        Ok(Self {
            server,
            client,
            state,
            verifier,
            url,
            expires: Instant::now() + Duration::from_secs(300),
        })
    }
    /// For the host's external browser only; do not log or store this attempt URL.
    pub fn authorization_url(&self) -> &str {
        self.url.as_str()
    }
    pub fn redirect_uri(&self) -> &str {
        &self.client.redirect
    }

    pub(super) fn code(&self, callback: &str) -> Result<String> {
        if Instant::now() >= self.expires {
            return Err(Failure(Kind::AttemptExpired));
        }
        if callback.len() > 8192 || !text(callback, 8192) {
            return Err(Failure(Kind::Callback));
        }
        let mut url = Url::parse(callback).map_err(|_| Failure(Kind::Callback))?;
        if url.fragment().is_some() {
            return Err(Failure(Kind::Callback));
        }
        // Url's query decoder is intentionally forgiving; the security boundary
        // rejects malformed escapes, replacement text and duplicate parameters.
        let query = url.query().ok_or(Failure(Kind::Callback))?.as_bytes();
        let mut offset = 0;
        while offset < query.len() {
            if query[offset] == b'%' {
                if offset + 2 >= query.len()
                    || !query[offset + 1..offset + 3]
                        .iter()
                        .all(u8::is_ascii_hexdigit)
                {
                    return Err(Failure(Kind::Callback));
                }
                offset += 3;
            } else {
                offset += 1;
            }
        }
        let mut fields = BTreeMap::new();
        for (key, value) in url.query_pairs() {
            if key.contains('\u{fffd}')
                || value.contains('\u{fffd}')
                || fields
                    .insert(key.into_owned(), value.into_owned())
                    .is_some()
            {
                return Err(Failure(Kind::Callback));
            }
        }
        url.set_query(None);
        if url.as_str() != self.client.redirect || fields.get("state") != Some(&self.state) {
            return Err(Failure(Kind::Callback));
        }
        // Validate issuer before even interpreting an error response. Never parse
        // the decoded issuer as a URL: exact issuer identity is required here.
        match fields.get("iss") {
            Some(issuer) if issuer != &self.server.issuer => return Err(Failure(Kind::Issuer)),
            None if self.server.issuer_in_callback => return Err(Failure(Kind::Issuer)),
            _ => {}
        }
        if fields.contains_key("error") {
            return Err(Failure(Kind::Denied));
        }
        fields
            .remove("code")
            .filter(|s| text(s, 4096))
            .ok_or(Failure(Kind::Callback))
    }
    /// Consumes one attempt, so the same authorization code cannot be exchanged
    /// twice through this API. The host must start a new attempt after failure.
    pub async fn complete(self, callback: &str, operation: &mut Operation) -> Result<Tokens> {
        operation.active()?;
        let code = self.code(callback)?;
        let value = operation
            .json(
                Operation::client(&self.server.token)?
                    .post(self.server.token.clone())
                    .form(&[
                        ("grant_type", "authorization_code"),
                        ("code", &code),
                        ("client_id", &self.client.client_id),
                        ("redirect_uri", &self.client.redirect),
                        ("code_verifier", &self.verifier),
                        ("resource", &self.server.resource),
                    ]),
            )
            .await?;
        Tokens::parse(value, &self.server, &self.client, None)
    }
}

/// Secret token result. Persistence and transport integration must remain inside
/// the MCP credential boundary; this type has no plaintext accessors or derives.
pub struct Tokens {
    pub(super) access: String,
    pub(super) refresh: Option<String>,
    pub(super) expires_at: Option<u64>,
    pub(super) scopes: Vec<String>,
    pub(super) issuer: String,
    pub(super) resource: String,
    pub(super) client_id: String,
}
fn token(value: &Value) -> Result<String> {
    value
        .as_str()
        .filter(|s| {
            !s.is_empty() && s.len() <= 4096 && s.bytes().all(|b| (0x21..=0x7e).contains(&b))
        })
        .map(str::to_owned)
        .ok_or(Failure(Kind::Tokens))
}
impl Tokens {
    fn parse(
        value: Value,
        server: &AuthorizationServer,
        client: &RegisteredClient,
        previous: Option<&Self>,
    ) -> Result<Self> {
        if !value["token_type"]
            .as_str()
            .is_some_and(|s| s.eq_ignore_ascii_case("Bearer"))
            || value.get("error").is_some()
        {
            return Err(Failure(Kind::Tokens));
        }
        let access = token(&value["access_token"])?;
        let refresh = value
            .get("refresh_token")
            .map(token)
            .transpose()?
            .or_else(|| previous.and_then(|t| t.refresh.clone()));
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|_| Failure(Kind::Tokens))?
            .as_secs();
        let expires_at = value
            .get("expires_in")
            .map(|v| {
                v.as_u64()
                    .filter(|n| *n > 0 && *n <= 31_536_000)
                    .and_then(|n| now.checked_add(n))
                    .ok_or(Failure(Kind::Tokens))
            })
            .transpose()?;
        let requested = previous.map(|v| &v.scopes).unwrap_or(&server.scopes);
        let granted = match value.get("scope") {
            Some(scope) => scopes(scope.as_str().ok_or(Failure(Kind::Tokens))?)
                .map_err(|_| Failure(Kind::Tokens))?,
            None => requested.clone(),
        };
        // With no requested scope, OAuth permits the server's documented default.
        // An explicit request still refuses unexpected additional authority.
        if !requested.is_empty() && granted.iter().any(|s| !requested.contains(s)) {
            return Err(Failure(Kind::Tokens));
        }
        Ok(Self {
            access,
            refresh,
            expires_at,
            scopes: granted,
            issuer: server.issuer.clone(),
            resource: server.resource.clone(),
            client_id: client.client_id.clone(),
        })
    }
    pub fn expires_at(&self) -> Option<u64> {
        self.expires_at
    }
    pub fn scopes(&self) -> &[String] {
        &self.scopes
    }
    pub fn can_refresh(&self) -> bool {
        self.refresh.is_some()
    }
    /// Known token text must not be echoed in metadata exposed to a model or UI.
    /// This supplements binding and secret-free diagnostics; it is not general DLP.
    pub fn contains_secret(&self, value: &Value) -> bool {
        super::super::contains(value, &self.access)
            || self
                .refresh
                .as_ref()
                .is_some_and(|s| super::super::contains(value, s))
    }
}
impl Operation {
    /// A failed or cancelled refresh leaves `previous` untouched. The caller must
    /// securely save a successful rotation before making it active. This method
    /// never reconnects to MCP or repeats a business request.
    pub async fn refresh(
        &mut self,
        server: &AuthorizationServer,
        client: &RegisteredClient,
        previous: &Tokens,
    ) -> Result<Tokens> {
        self.active()?;
        if server.issuer != client.issuer
            || server.resource != client.resource
            || previous.issuer != server.issuer
            || previous.resource != server.resource
            || previous.client_id != client.client_id
        {
            return Err(Failure(Kind::Issuer));
        }
        let refresh = previous
            .refresh
            .as_deref()
            .ok_or(Failure(Kind::SignInRequired))?;
        let mut form = vec![
            ("grant_type", "refresh_token"),
            ("refresh_token", refresh),
            ("client_id", &client.client_id),
            ("resource", &server.resource),
        ];
        let scope = previous.scopes.join(" ");
        if !scope.is_empty() {
            form.push(("scope", &scope));
        }
        let value = self
            .json(
                Self::client(&server.token)?
                    .post(server.token.clone())
                    .form(&form),
            )
            .await?;
        Tokens::parse(value, server, client, Some(previous))
    }
}
