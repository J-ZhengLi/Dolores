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
    if range.is_none() && text.len() <= MAX_TOOL_BYTES {
        return Ok(text.into());
    }
    let range = range.unwrap_or("1:120");
    if let Some(bytes) = range.strip_prefix("bytes:") {
        let (offset, count) = bytes
            .split_once(':')
            .ok_or("Invalid approved byte range.")?;
        let offset: usize = offset.parse().map_err(|_| "Invalid approved byte range.")?;
        let count: usize = count.parse().map_err(|_| "Invalid approved byte range.")?;
        return render_bytes(text, offset, count);
    }
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
        return render_bytes(text, lines.iter().take(s - 1).map(|s| s.len()).sum(), 8192);
    }
    let result=serde_json::json!({"snapshot":digest(text),"startLine":s,"lineCount":used,"totalLines":lines.len(),"bytes":text.len(),"text":part,"hasMore":s-1+used<lines.len()}).to_string();
    if result.len() > MAX_TOOL_BYTES {
        return render_bytes(text, lines.iter().take(s - 1).map(|s| s.len()).sum(), 8192);
    }
    Ok(result)
}

pub(super) fn byte_range(offset: Option<usize>, count: Option<usize>) -> Result<String, String> {
    let count = count.unwrap_or(8192);
    if !(1..=65536).contains(&count) {
        return Err("Use max_bytes 1–65536 and start_byte from nextByte.".into());
    }
    Ok(format!("bytes:{}:{count}", offset.unwrap_or(0)))
}
fn render_bytes(text: &str, offset: usize, count: usize) -> Result<String, String> {
    if offset > text.len() || !text.is_char_boundary(offset) || !(1..=65536).contains(&count) {
        return Err("Invalid UTF-8 byte cursor. Use nextByte from the previous read.".into());
    }
    let mut next = offset.saturating_add(count).min(text.len());
    while !text.is_char_boundary(next) {
        next -= 1;
    }
    if next == offset && offset < text.len() {
        next = offset + text[offset..].chars().next().unwrap().len_utf8();
    }
    loop {
        let value = serde_json::json!({"snapshot":digest(text),"startByte":offset,"nextByte":next,"bytes":text.len(),"text":&text[offset..next],"hasMore":next<text.len(),"partialLine":next<text.len() && !text[..next].ends_with('\n')});
        let encoded = value.to_string();
        if encoded.len() <= MAX_TOOL_BYTES {
            return Ok(encoded);
        }
        next = offset + (next - offset) / 2;
        while !text.is_char_boundary(next) {
            next -= 1;
        }
    }
}

/// Large-file reads scan/hash incrementally, retaining only a response page.
pub(super) fn stream_page(
    mut file: cap_std::fs::File,
    range: Option<&str>,
    cancel: &tokio_util::sync::CancellationToken,
) -> Result<String, String> {
    use std::io::Read;
    let mut start_line = 1usize;
    let mut line_count = 120usize;
    let mut start_byte = None;
    let mut max_bytes = 8192usize;
    if let Some(range) = range {
        if let Some(range) = range.strip_prefix("bytes:") {
            let (s, c) = range.split_once(':').ok_or("Invalid byte range.")?;
            start_byte = Some(s.parse::<usize>().map_err(|_| "Invalid byte range.")?);
            max_bytes = c
                .parse::<usize>()
                .map_err(|_| "Invalid byte range.")?
                .min(8192);
        } else {
            let (s, c) = range.split_once(':').ok_or("Invalid line range.")?;
            start_line = s.parse().map_err(|_| "Invalid line range.")?;
            line_count = c.parse().map_err(|_| "Invalid line range.")?;
        }
    }
    let mut digest = Sha256::new();
    let mut buffer = [0u8; 65536];
    let mut carry = vec![];
    let mut text = String::new();
    let (mut bytes, mut line, mut selected_start, mut selected_end) =
        (0usize, 1usize, None, 0usize);
    let mut last_newline = false;
    loop {
        if cancel.is_cancelled() {
            return Err("Response stopped.".into());
        }
        let n = file
            .read(&mut buffer)
            .map_err(|_| "File could not be read.")?;
        if n == 0 {
            break;
        }
        if buffer[..n].contains(&0) {
            return Err("File is binary.".into());
        }
        digest.update(&buffer[..n]);
        carry.extend_from_slice(&buffer[..n]);
        let valid = match std::str::from_utf8(&carry) {
            Ok(_) => carry.len(),
            Err(e) if e.error_len().is_none() => e.valid_up_to(),
            Err(_) => return Err("File is not UTF-8 text.".into()),
        };
        for c in std::str::from_utf8(&carry[..valid]).unwrap().chars() {
            let width = c.len_utf8();
            if start_byte.is_some_and(|s| s > bytes && s < bytes + width) {
                return Err("Invalid UTF-8 byte cursor. Use nextByte.".into());
            }
            let selected = match start_byte {
                Some(start) => bytes >= start,
                None => line >= start_line && line < start_line.saturating_add(line_count),
            };
            if selected
                && (text.len() + width <= max_bytes || selected_start.is_none())
                && (selected_start.is_none() || selected_end == bytes)
            {
                selected_start.get_or_insert(bytes);
                text.push(c);
                selected_end = bytes + width;
            }
            bytes += width;
            last_newline = c == '\n';
            if last_newline {
                line += 1;
            }
        }
        carry.drain(..valid);
    }
    if !carry.is_empty() {
        return Err("File is not UTF-8 text.".into());
    }
    let start = selected_start.unwrap_or(bytes);
    if start_byte.is_some_and(|s| s > bytes) || start_byte.is_none() && start_line > line {
        return Err("Range is beyond the file. Read earlier lines or bytes.".into());
    }
    if text.is_empty() {
        selected_end = bytes;
    }
    let snapshot = format!("{:x}", digest.finalize());
    loop {
        let result = serde_json::json!({"snapshot":snapshot,"startByte":start,"nextByte":selected_end,"bytes":bytes,"totalLines":line-usize::from(last_newline),"text":text,"hasMore":selected_end<bytes,"partialLine":!text.ends_with('\n')});
        let encoded = result.to_string();
        if encoded.len() <= MAX_TOOL_BYTES {
            return Ok(encoded);
        }
        let mut end = text.len() / 2;
        while !text.is_char_boundary(end) {
            end -= 1;
        }
        text.truncate(end);
        selected_end = start + end;
    }
}
