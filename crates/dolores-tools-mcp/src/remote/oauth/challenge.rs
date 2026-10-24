use super::{scopes, Failure, Kind, Result};
use std::collections::BTreeMap;

pub(super) struct Challenge {
    pub metadata: Option<String>,
    pub scopes: Option<Vec<String>>,
}
fn token(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || b"!#$%&'*+-.^_`|~".contains(&byte)
}
fn parameter(value: &str) -> Result<(String, String)> {
    let (name, value) = value.split_once('=').ok_or(Failure(Kind::Challenge))?;
    let name = name.trim();
    if name.is_empty() || !name.bytes().all(token) {
        return Err(Failure(Kind::Challenge));
    }
    let value = value.trim();
    let parsed = if let Some(inner) = value.strip_prefix('"').and_then(|v| v.strip_suffix('"')) {
        let mut result = String::new();
        let mut escaped = false;
        for c in inner.chars() {
            if escaped {
                result.push(c);
                escaped = false;
            } else if c == '\\' {
                escaped = true;
            } else if c == '"' {
                return Err(Failure(Kind::Challenge));
            } else {
                result.push(c);
            }
        }
        if escaped {
            return Err(Failure(Kind::Challenge));
        }
        result
    } else if !value.is_empty() && value.bytes().all(token) {
        value.to_owned()
    } else {
        return Err(Failure(Kind::Challenge));
    };
    Ok((name.to_ascii_lowercase(), parsed))
}

/// Split only commas outside quoted strings; recognize challenge schemes instead
/// of mistaking another scheme's parameters for Bearer discovery instructions.
pub(super) fn parse(header: Option<&str>) -> Result<Challenge> {
    let mut answer = Challenge {
        metadata: None,
        scopes: None,
    };
    let Some(header) = header else {
        return Ok(answer);
    };
    if header.len() > 8192
        || !header
            .bytes()
            .all(|b| (0x20..=0x7e).contains(&b) || b == b'\t')
    {
        return Err(Failure(Kind::Challenge));
    }
    let (mut quoted, mut escaped, mut start) = (false, false, 0);
    let mut parts = vec![];
    for (i, byte) in header.bytes().enumerate() {
        if escaped {
            escaped = false;
            continue;
        }
        if quoted && byte == b'\\' {
            escaped = true;
        } else if byte == b'"' {
            quoted = !quoted;
        } else if !quoted && byte == b',' {
            parts.push(&header[start..i]);
            start = i + 1;
        }
    }
    if quoted || escaped {
        return Err(Failure(Kind::Challenge));
    }
    parts.push(&header[start..]);
    if parts.len() > 32 {
        return Err(Failure(Kind::Challenge));
    }
    let (mut bearer, mut found) = (false, false);
    let mut fields = BTreeMap::new();
    for part in parts {
        let part = part.trim();
        let n = part.bytes().take_while(|b| token(*b)).count();
        if n == 0 {
            return Err(Failure(Kind::Challenge));
        }
        let after = part[n..].trim_start();
        let parameter_text = if !after.starts_with('=') {
            bearer = part[..n].eq_ignore_ascii_case("Bearer");
            if bearer && found {
                return Err(Failure(Kind::Challenge));
            }
            found |= bearer;
            after
        } else {
            part
        };
        if !bearer || parameter_text.is_empty() {
            continue;
        }
        let (name, value) = parameter(parameter_text)?;
        if fields.insert(name, value).is_some() {
            return Err(Failure(Kind::Challenge));
        }
    }
    answer.metadata = fields.remove("resource_metadata");
    answer.scopes = fields.remove("scope").map(|s| scopes(&s)).transpose()?;
    Ok(answer)
}
