use crate::{public_url, MAX_EXTRACT_BYTES};
use dolores_core::SearchProvider;
use scraper::{Html, Node, Selector};
use serde_json::{json, Value};
fn shortened(text: &str, max: usize) -> String {
    let mut end = text.len().min(max);
    while !text.is_char_boundary(end) {
        end -= 1;
    }
    text[..end].to_owned()
}
fn segments(value: &Value) -> Option<String> {
    value
        .as_array()?
        .iter()
        .map(|v| v.get("value")?.as_str())
        .collect::<Option<Vec<_>>>()
        .map(|v| v.concat())
}
pub(crate) fn results(value: &Value, provider: SearchProvider) -> Result<Vec<Value>, String> {
    let raw = match provider {
        SearchProvider::Mwmbl => value.as_array(),
        SearchProvider::Brave => value.pointer("/web/results").and_then(Value::as_array),
        SearchProvider::Searxng => value.get("results").and_then(Value::as_array),
    }.ok_or("Search response has no valid results list. Open Web search settings or use a public source URL.")?;
    let mut results = vec![];
    for item in raw.iter().take(100) {
        let raw_url = item
            .get("url")
            .and_then(Value::as_str)
            .ok_or("Search result URL is malformed.")?;
        if public_url(raw_url).is_err() {
            continue;
        }
        let (title, text) = if provider == SearchProvider::Mwmbl {
            (
                segments(&item["title"]).ok_or("Search title is malformed.")?,
                segments(&item["extract"]).ok_or("Search snippet is malformed.")?,
            )
        } else {
            (
                item["title"]
                    .as_str()
                    .ok_or("Search title is malformed.")?
                    .into(),
                item[if provider == SearchProvider::Brave {
                    "description"
                } else {
                    "content"
                }]
                .as_str()
                .unwrap_or("")
                .into(),
            )
        };
        results.push(
            json!({"url":raw_url,"title":shortened(&title,256),"snippet":shortened(&text,768)}),
        );
        if results.len() == 5 {
            break;
        }
    }
    Ok(results)
}
pub(crate) fn page(text: &str, kind: &str) -> Result<(String, String), String> {
    let kind = kind
        .split(';')
        .next()
        .unwrap_or("")
        .trim()
        .to_ascii_lowercase();
    if kind == "text/plain" {
        return Ok((String::new(), text.to_owned()));
    }
    if kind != "text/html" && kind != "application/xhtml+xml" {
        return Err("This web format is unsupported. Use an HTML/plain-text page or a local attachment; PDF, images and script-only pages need another adapter.".into());
    }
    // A byte ceiling alone permits deeply nested input with quadratic ancestry
    // work. Refuse excessive markup before parsing and cap extraction work too.
    const COMPLEX: &str = "This page exceeds the bounded reader's markup/depth limit. Choose a simpler public page or attach converted text; no partial evidence was returned.";
    const MAX_NODES: usize = 32_768;
    const MAX_DEPTH: usize = 128;
    if text.matches('<').take(MAX_NODES + 1).count() > MAX_NODES {
        return Err(COMPLEX.into());
    }
    let document = Html::parse_document(text);
    let title = document
        .select(&Selector::parse("title").unwrap())
        .next()
        .map(|e| e.text().collect::<String>())
        .unwrap_or_default();
    let mut content = String::new();
    for (index, node) in document.root_element().descendants().enumerate() {
        if index >= MAX_NODES {
            return Err(COMPLEX.into());
        }
        if let Node::Text(text) = node.value() {
            let mut hidden = false;
            for (depth, parent) in node.ancestors().enumerate() {
                if depth >= MAX_DEPTH {
                    return Err(COMPLEX.into());
                }
                if matches!(parent.value(),Node::Element(e) if ["script","style","noscript","template","svg","head"].contains(&e.name()) || e.attr("hidden").is_some() || e.attr("aria-hidden")==Some("true"))
                {
                    hidden = true;
                    break;
                }
            }
            if hidden {
                continue;
            }
            let text = text.split_whitespace().collect::<Vec<_>>().join(" ");
            if !text.is_empty() {
                content.push_str(&text);
                content.push('\n');
            }
        }
    }
    if content.trim().is_empty() {
        return Err("No readable page text was returned. This may need a browser or login; provide another public source. No browser was started.".into());
    }
    Ok((shortened(&title, 256), content))
}
pub(crate) fn excerpt(text: &str, start: usize) -> Result<(String, Option<usize>), String> {
    let byte_start = text
        .char_indices()
        .nth(start)
        .map(|(i, _)| i)
        .unwrap_or(text.len());
    if byte_start == text.len() && start != 0 {
        return Err("Web offset is beyond this page. Read from startCharacter 0; the page may have changed.".into());
    }
    let slice = shortened(&text[byte_start..], MAX_EXTRACT_BYTES);
    let next = (byte_start + slice.len() < text.len()).then(|| start + slice.chars().count());
    Ok((slice, next))
}
