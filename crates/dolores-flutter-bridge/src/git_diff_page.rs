//! Stream Git's changed sections into a bounded display page, never whole blobs.
use super::*;

pub(super) const PAGE_ROWS: usize = 256;
const PAGE_BYTES: usize = 256 * 1024;
const SEGMENT_BYTES: usize = 4096;

pub(super) struct Collector {
    cursor: usize,
    row_limit: usize,
    rows: Vec<Value>,
    patch: Vec<u8>,
    next: Option<usize>,
    total: usize,
    pending: Vec<u8>,
    first: bool,
    kind: &'static str,
    old: u64,
    new: u64,
    in_hunk: bool,
    digest: Sha256,
    encoding_warning: bool,
}

impl Collector {
    pub(super) fn new(cursor: usize, row_limit: usize) -> Self {
        Self {
            cursor,
            row_limit,
            rows: vec![],
            patch: vec![],
            next: None,
            total: 0,
            pending: vec![],
            first: true,
            kind: "meta",
            old: 0,
            new: 0,
            in_hunk: false,
            digest: Sha256::new(),
            encoding_warning: false,
        }
    }
    pub(super) fn push(&mut self, bytes: &[u8]) {
        self.digest.update(bytes);
        for byte in bytes {
            self.pending.push(*byte);
            if *byte == b'\n' {
                let part = std::mem::take(&mut self.pending);
                self.emit(&part, true);
            } else if self.pending.len() >= SEGMENT_BYTES + 4 {
                let mut end = SEGMENT_BYTES;
                // Keep a multi-byte UTF-8 character on one rendered segment.
                while end > 0 && self.pending[end] & 0xc0 == 0x80 {
                    end -= 1;
                }
                let remaining = self.pending.split_off(end);
                let part = std::mem::replace(&mut self.pending, remaining);
                self.emit(&part, false);
            }
        }
    }
    fn emit(&mut self, part: &[u8], end_line: bool) {
        let continued = !self.first;
        if self.first {
            if part.starts_with(b"diff --git ") {
                self.in_hunk = false;
            }
            if part.starts_with(b"@@ ") {
                let header = String::from_utf8_lossy(part);
                let mut ranges = header.split_whitespace().skip(1);
                let parse = |s: Option<&str>| {
                    s.and_then(|s| s.get(1..))
                        .and_then(|s| s.split(',').next())
                        .and_then(|s| s.parse::<u64>().ok())
                };
                if let (Some(old), Some(new)) = (parse(ranges.next()), parse(ranges.next())) {
                    self.old = old;
                    self.new = new;
                    self.in_hunk = true;
                }
                self.kind = "meta";
            } else {
                self.kind = if self.in_hunk {
                    match part.first() {
                        Some(b'-') => "remove",
                        Some(b'+') => "add",
                        Some(b' ') => "context",
                        _ => "meta",
                    }
                } else {
                    "meta"
                };
            }
        }
        let has_prefix = self.first && self.kind != "meta";
        let raw = if has_prefix { &part[1..] } else { part };
        let raw = raw.strip_suffix(b"\n").unwrap_or(raw);
        let raw = raw.strip_suffix(b"\r").unwrap_or(raw);
        let text = String::from_utf8_lossy(raw);
        let warning = std::str::from_utf8(raw).is_err();
        let old = (self.first && matches!(self.kind, "remove" | "context")).then_some(self.old);
        let new = (self.first && matches!(self.kind, "add" | "context")).then_some(self.new);
        if self.total >= self.cursor && self.next.is_none() {
            if self.rows.len() < self.row_limit && self.patch.len() + part.len() <= PAGE_BYTES {
                self.rows.push(json!({"kind":self.kind,"text":text,"oldLine":old,"newLine":new,"continued":continued,"endLine":end_line}));
                self.patch.extend_from_slice(part);
                self.encoding_warning |= warning;
            } else {
                self.next = Some(self.total);
            }
        }
        self.total += 1;
        if end_line {
            if matches!(self.kind, "remove" | "context") {
                self.old = self.old.saturating_add(1);
            }
            if matches!(self.kind, "add" | "context") {
                self.new = self.new.saturating_add(1);
            }
            self.first = true;
        } else {
            self.first = false;
        }
    }
    pub(super) fn finish(mut self) -> Result<Value, String> {
        if !self.pending.is_empty() {
            let part = std::mem::take(&mut self.pending);
            self.emit(&part, true);
        }
        if self.cursor > self.total {
            return Err("Diff page is outside these changes. Refresh the comparison.".into());
        }
        let side = |kind: &str| {
            self.rows
                .iter()
                .filter(|row| row["kind"] == kind || row["kind"] == "context")
                .map(|row| {
                    format!(
                        "{}{}",
                        row["text"].as_str().unwrap(),
                        if row["endLine"] == true { "\n" } else { "" }
                    )
                })
                .collect::<String>()
        };
        Ok(
            json!({"rows":self.rows,"patch":String::from_utf8_lossy(&self.patch),"left":side("remove"),"right":side("add"),"cursor":self.cursor,"next":self.next,"totalRows":self.total,"digest":format!("{:x}",self.digest.finalize()),"encodingWarning":self.encoding_warning}),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn pages_keep_line_numbers_utf8_and_bounded_long_line_segments() {
        let raw = format!(
            "diff --git a/a b/a\n@@ -20 +30 @@\n-old\n+{}\n context\n",
            "中".repeat(100000)
        );
        let mut cursor = 0;
        let mut combined = String::new();
        let mut fingerprint = None;
        loop {
            let mut collector = Collector::new(cursor, 256);
            for chunk in raw.as_bytes().chunks(97) {
                collector.push(chunk);
            }
            let page = collector.finish().unwrap();
            assert!(page["patch"].as_str().unwrap().len() <= PAGE_BYTES);
            assert_eq!(page["encodingWarning"], false);
            if let Some(previous) = &fingerprint {
                assert_eq!(&page["digest"], previous);
            }
            fingerprint = Some(page["digest"].clone());
            combined.push_str(page["patch"].as_str().unwrap());
            if cursor == 0 {
                assert_eq!(page["rows"][2]["oldLine"], 20);
                assert_eq!(page["rows"][3]["newLine"], 30);
            }
            if let Some(next) = page["next"].as_u64() {
                assert!(next as usize > cursor);
                cursor = next as usize;
            } else {
                break;
            }
        }
        assert_eq!(combined, raw);
    }
}
