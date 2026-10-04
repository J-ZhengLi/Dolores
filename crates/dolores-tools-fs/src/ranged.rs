use dolores_core::MAX_TOOL_BYTES;
use sha2::{Digest, Sha256};
pub(super) fn digest(text: &str) -> String {
    format!("{:x}", Sha256::digest(text.as_bytes()))
}
pub(super) fn range(start: Option<usize>, count: Option<usize>) -> Result<Option<String>, String> {
    match (start, count) {
        (None, None) => Ok(None),
        (Some(s), c) if s > 0 && s <= 1_048_577 && (1..=120).contains(&c.unwrap_or(120)) => {
            Ok(Some(format!("{s}:{}", c.unwrap_or(120))))
        }
        _ => Err("Use start_line > 0 and line_count 1–120.".into()),
    }
}
pub(super) fn render(text: &str, range: Option<&str>) -> Result<String, String> {
    let Some(range) = range else {
        return if text.len() <= MAX_TOOL_BYTES {
            Ok(text.into())
        } else {
            Err("File exceeds 16 KiB. Use start_line and line_count for a ranged read.".into())
        };
    };
    let (s, c) = range.split_once(':').ok_or("Invalid approved range.")?;
    let s: usize = s.parse().map_err(|_| "Invalid approved range.")?;
    let c: usize = c.parse().map_err(|_| "Invalid approved range.")?;
    super::ranged::range(Some(s), Some(c))?;
    let lines: Vec<_> = text.split_inclusive('\n').collect();
    if s > lines.len().max(1) {
        return Err("Range is beyond the file. Read earlier lines.".into());
    }
    let mut part = String::new();
    let mut used = 0;
    for line in lines.iter().skip(s - 1).take(c) {
        if part.len() + line.len() > MAX_TOOL_BYTES / 2 {
            break;
        }
        part.push_str(line);
        used += 1;
    }
    if used == 0 && !text.is_empty() {
        return Err("One line exceeds the read allowance. Use a local editor to split it.".into());
    }
    let result=serde_json::json!({"snapshot":digest(text),"startLine":s,"lineCount":used,"totalLines":lines.len(),"bytes":text.len(),"text":part,"hasMore":s-1+used<lines.len()}).to_string();
    if result.len() > MAX_TOOL_BYTES {
        return Err("Encoded range exceeds the tool allowance. Request fewer lines.".into());
    }
    Ok(result)
}
