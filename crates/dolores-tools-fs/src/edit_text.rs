use std::borrow::Cow;

#[derive(Clone, Copy)]
enum Ending {
    Lf,
    Crlf,
    Mixed,
}

fn lone_cr(text: &str) -> bool {
    text.as_bytes()
        .iter()
        .enumerate()
        .any(|(i, b)| *b == b'\r' && text.as_bytes().get(i + 1) != Some(&b'\n'))
}

fn ending(text: &str) -> Option<Ending> {
    if lone_cr(text) {
        return Some(Ending::Mixed);
    }
    let crlf = text.matches("\r\n").count();
    let lf = text.bytes().filter(|b| *b == b'\n').count();
    match (crlf, lf) {
        (_, 0) => None,
        (0, _) => Some(Ending::Lf),
        (a, b) if a == b => Some(Ending::Crlf),
        _ => Some(Ending::Mixed),
    }
}

fn adapt(text: &str, style: Ending) -> Cow<'_, str> {
    match style {
        Ending::Lf if text.contains("\r\n") => Cow::Owned(text.replace("\r\n", "\n")),
        Ending::Crlf if matches!(ending(text), Some(Ending::Lf | Ending::Mixed)) => {
            Cow::Owned(text.replace("\r\n", "\n").replace('\n', "\r\n"))
        }
        _ => Cow::Borrowed(text),
    }
}

/// Adapt only proposal newlines, never the source snapshot or other whitespace.
/// Homogeneous source endings define one deterministic, reviewed replacement.
pub(super) fn replace(before: &str, old: &str, new: &str) -> Result<String, String> {
    let style = ending(before);
    let (old, new) = match style {
        Some(style @ (Ending::Lf | Ending::Crlf)) if !lone_cr(old) && !lone_cr(new) => {
            (adapt(old, style), adapt(new, style))
        }
        _ => (Cow::Borrowed(old), Cow::Borrowed(new)),
    };
    if old.is_empty() || old == new {
        return Err("Edit needs different text and a nonempty match.".into());
    }
    let mut positions = before.match_indices(old.as_ref());
    let position = match positions.next() {
        Some((position, _)) => position,
        None if matches!(style, Some(Ending::Mixed))
            && ((lone_cr(before) && old.contains('\n'))
                || before
                    .replace("\r\n", "\n")
                    .contains(&old.replace("\r\n", "\n"))) =>
        {
            return Err("Line-ending adaptation is unavailable for mixed or lone-CR files. Use an exact single-line match or copy the original line endings.".into());
        }
        None => return Err("Exact edit text was not found.".into()),
    };
    if positions.next().is_some() {
        return Err("Edit text occurs more than once. Use a larger unique match.".into());
    }
    // Include overlapping matches, preserving the old exact-edit contract.
    let next = position + old.chars().next().unwrap().len_utf8();
    if before[next..].contains(old.as_ref()) {
        return Err("Edit text occurs more than once.".into());
    }
    Ok(before.replacen(old.as_ref(), new.as_ref(), 1))
}
