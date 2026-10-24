use super::{fail, Failure, Kind};
use base64::{engine::general_purpose::STANDARD, Engine};
use reqwest::header::{HeaderMap, HeaderName, HeaderValue};
use serde_json::Value;
use std::collections::BTreeSet;

fn token(s: &str) -> bool {
    !s.is_empty()
        && s.len() <= 128
        && s.bytes()
            .all(|b| b.is_ascii_alphanumeric() || b"!#$%&'*+-.^_`|~".contains(&b))
}
pub(super) fn encoded(s: &str) -> String {
    if s.trim() == s
        && s.bytes().all(|b| (0x20..=0x7e).contains(&b) || b == b'\t')
        && !(s.starts_with("=?base64?") && s.ends_with("?="))
    {
        s.into()
    } else {
        format!("=?base64?{}?=", STANDARD.encode(s.as_bytes()))
    }
}
#[derive(Clone)]
struct Mapping {
    path: Vec<String>,
    name: String,
    kind: String,
}
fn walk(
    value: &Value,
    path: Option<&[String]>,
    names: &mut BTreeSet<String>,
    out: &mut Vec<Mapping>,
) -> Result<(), Failure> {
    match value {
        Value::Object(map) => {
            if let Some(name) = map.get("x-mcp-header") {
                let name = name
                    .as_str()
                    .filter(|s| token(s))
                    .ok_or_else(|| fail(Kind::HeaderSchema))?;
                let path = path
                    .filter(|p| !p.is_empty())
                    .ok_or_else(|| fail(Kind::HeaderSchema))?;
                let kind = map
                    .get("type")
                    .and_then(Value::as_str)
                    .filter(|t| matches!(*t, "integer" | "string" | "boolean"))
                    .ok_or_else(|| fail(Kind::HeaderSchema))?;
                if !names.insert(name.to_ascii_lowercase()) {
                    return Err(fail(Kind::HeaderSchema));
                }
                out.push(Mapping {
                    path: path.to_vec(),
                    name: name.into(),
                    kind: kind.into(),
                });
            }
            for (key, child) in map {
                if key == "properties" {
                    if let (Some(path), Some(properties)) = (path, child.as_object()) {
                        for (name, schema) in properties {
                            let mut next = path.to_vec();
                            next.push(name.clone());
                            walk(schema, Some(&next), names, out)?;
                        }
                        continue;
                    }
                }
                // These keywords contain instance data, not subschemas. A sample
                // object with an x-mcp-header key must not alter HTTP metadata.
                if !matches!(
                    key.as_str(),
                    "x-mcp-header" | "examples" | "default" | "enum" | "const"
                ) {
                    walk(child, None, names, out)?;
                }
            }
        }
        Value::Array(values) => {
            for child in values {
                walk(child, None, names, out)?;
            }
        }
        _ => {}
    }
    Ok(())
}
fn mappings(schema: &Value) -> Result<Vec<Mapping>, Failure> {
    let mut out = vec![];
    walk(schema, Some(&[]), &mut BTreeSet::new(), &mut out)?;
    Ok(out)
}
pub(super) fn validate(schema: &Value) -> Result<(), Failure> {
    mappings(schema).map(|_| ())
}
pub(super) fn tool_headers(name: &str, schema: &Value, args: &Value) -> Result<HeaderMap, Failure> {
    let mut headers = HeaderMap::new();
    headers.insert(
        "mcp-name",
        HeaderValue::from_str(&encoded(name)).map_err(|_| fail(Kind::Arguments))?,
    );
    for mapping in mappings(schema)? {
        let mut value = Some(args);
        for part in &mapping.path {
            value = value.and_then(|v| v.as_object()).and_then(|v| v.get(part));
        }
        let Some(value) = value.filter(|v| !v.is_null()) else {
            continue;
        };
        let value = match mapping.kind.as_str() {
            "string" => value.as_str().map(str::to_owned),
            "boolean" => value.as_bool().map(|v| v.to_string()),
            "integer" => value
                .as_f64()
                .filter(|n| n.is_finite() && n.fract() == 0.0 && n.abs() <= 9_007_199_254_740_991.0)
                .map(|v| (v as i64).to_string()),
            _ => None,
        }
        .ok_or_else(|| fail(Kind::Arguments))?;
        let key = HeaderName::from_bytes(format!("mcp-param-{}", mapping.name).as_bytes())
            .map_err(|_| fail(Kind::HeaderSchema))?;
        headers.insert(
            key,
            HeaderValue::from_str(&encoded(&value)).map_err(|_| fail(Kind::Arguments))?,
        );
    }
    Ok(headers)
}
