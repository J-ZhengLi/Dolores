//! Pair a selected slice without retaining the rest of a replacement run.
use super::PAGE_BYTES;
use serde_json::{json, Value};
use std::collections::BTreeMap;

struct Row {
    value: Value,
    bytes: usize,
    warning: bool,
}
#[derive(Default)]
struct Side {
    total: usize,
    bytes: usize,
    closed: bool,
    selected: BTreeMap<usize, Row>,
}
#[derive(Default)]
struct Changes {
    removed: Side,
    added: Side,
}
pub(super) struct Collector {
    cursor: usize,
    limit: usize,
    total: usize,
    bytes: usize,
    next: Option<usize>,
    rows: Vec<Value>,
    changes: Option<Changes>,
    warning: bool,
}
impl Collector {
    pub(super) fn new(cursor: usize, limit: usize) -> Self {
        Self {
            cursor,
            limit,
            total: 0,
            bytes: 0,
            next: None,
            rows: vec![],
            changes: None,
            warning: false,
        }
    }
    pub(super) fn push(&mut self, value: Value, bytes: usize, warning: bool) {
        let kind = value["kind"].as_str().unwrap();
        if kind != "remove" && kind != "add" {
            self.flush();
            let row = Row {
                value,
                bytes,
                warning,
            };
            self.emit(Some(&row), Some(&row));
            self.total += 1;
            return;
        }
        if kind == "remove" && self.changes.as_ref().is_some_and(|c| c.added.total > 0) {
            self.flush();
        }
        let changes = self.changes.get_or_insert_with(Changes::default);
        let side = if kind == "remove" {
            &mut changes.removed
        } else {
            &mut changes.added
        };
        let index = side.total;
        side.total += 1;
        let absolute = self.total.saturating_add(index);
        if self.next.is_none()
            && !side.closed
            && absolute >= self.cursor
            && absolute.saturating_sub(self.cursor) < self.limit
        {
            // Each side gets half the remaining budget. The first missing
            // counterpart ends this page; it is never silently dropped.
            if side.bytes + bytes <= (PAGE_BYTES - self.bytes) / 2 {
                side.bytes += bytes;
                side.selected.insert(
                    index,
                    Row {
                        value,
                        bytes,
                        warning,
                    },
                );
            } else {
                side.closed = true;
            }
        }
    }
    fn emit(&mut self, left: Option<&Row>, right: Option<&Row>) {
        if self.total < self.cursor || self.next.is_some() {
            return;
        }
        let bytes = left.map_or(0, |r| r.bytes) + right.map_or(0, |r| r.bytes);
        if self.rows.len() >= self.limit || self.bytes + bytes > PAGE_BYTES {
            self.next = Some(self.total);
            return;
        }
        self.bytes += bytes;
        self.warning |= left.is_some_and(|r| r.warning) || right.is_some_and(|r| r.warning);
        self.rows
            .push(json!({"left":left.map(|r| &r.value),"right":right.map(|r| &r.value)}));
    }
    fn flush(&mut self) {
        let Some(changes) = self.changes.take() else {
            return;
        };
        let count = changes.removed.total.max(changes.added.total);
        let start = self.total;
        if self.next.is_none() {
            let first = self.cursor.saturating_sub(start).min(count);
            for index in first..count {
                self.total = start + index;
                let left = changes.removed.selected.get(&index);
                let right = changes.added.selected.get(&index);
                if (index < changes.removed.total && left.is_none())
                    || (index < changes.added.total && right.is_none())
                {
                    self.next = Some(self.total);
                    break;
                }
                self.emit(left, right);
                if self.next.is_some() {
                    break;
                }
            }
        }
        self.total = start + count;
    }
    pub(super) fn finish(mut self) -> Result<Value, String> {
        self.flush();
        if self.cursor > self.total {
            return Err("Diff page is outside these changes. Refresh the comparison.".into());
        }
        Ok(
            json!({"sideRows":self.rows,"rows":[],"cursor":self.cursor,"next":self.next,
            "totalRows":self.total,"sideBySide":true,"encodingWarning":self.warning,"displayBytes":self.bytes}),
        )
    }
}
