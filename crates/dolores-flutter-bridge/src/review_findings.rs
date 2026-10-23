//! Host capture and publication run inside Source Control's owned Git lane.
use super::*;
use dolores_core::{review_findings::*, SessionStore};

#[derive(Deserialize)]
#[serde(tag = "action", rename_all = "camelCase", deny_unknown_fields)]
pub(crate) enum Request {
    List {
        cursor: Option<i64>,
    },
    Get {
        id: String,
    },
    Submission {
        id: String,
    },
    Note {
        id: String,
        revision: u32,
        text: String,
    },
    Remove {
        id: String,
        revision: u32,
    },
    Reopen {
        id: String,
        revision: u32,
    },
}

#[cfg(test)]
#[path = "review_findings_tests.rs"]
mod tests;

#[derive(Deserialize)]
#[serde(tag = "action", rename_all = "camelCase", deny_unknown_fields)]
pub(crate) enum Operation {
    Capture {
        id: String,
        comparison: Comparison,
        cursor: usize,
        #[serde(default, rename = "sideBySide")]
        side_by_side: bool,
        side: Side,
        line: u32,
        text: String,
    },
    Resolve {
        id: String,
        revision: u32,
        comparison: Comparison,
        note: String,
    },
    Submit {
        submission: Submission,
    },
}

pub(crate) fn local(
    store: &dyn SessionStore,
    session: &str,
    request: Request,
) -> Result<Value, String> {
    let edit = match request {
        Request::List { cursor } => return Ok(json!(store.review_findings(session, cursor)?)),
        Request::Get { id } => return Ok(json!(store.review_finding(session, &id)?)),
        Request::Submission { id } => return Ok(json!(store.review_submission(session, &id)?)),
        Request::Note { id, revision, text } => Edit::Note { id, revision, text },
        Request::Remove { id, revision } => Edit::Remove { id, revision },
        Request::Reopen { id, revision } => Edit::Reopen { id, revision },
    };
    store.edit_review_finding(session, &edit)?;
    Ok(Value::Null)
}

fn comparison(
    repo: &Repo,
    expected: &Comparison,
    cursor: usize,
    side: bool,
    cancel: &CancellationToken,
) -> Result<Value, String> {
    expected.validate()?;
    if path_key(&repo.root) != path_key(Path::new(&expected.root)) {
        return Err("Finding belongs to a different checkout. Open its original chat or select a location in this checkout; saved notes remain.".into());
    }
    repo.check(&expected.repository)?;
    let read = if side {
        Repo::diff_side_page
    } else {
        Repo::diff_page
    };
    let diff = read(
        repo,
        &expected.revision,
        &expected.path,
        &expected.basis,
        expected.commit.as_deref(),
        (cursor, Some(&expected.digest)),
        cancel,
    )?;
    if diff["reason"].is_string() || diff["encodingWarning"] == true {
        return Err("This comparison cannot supply a complete UTF-8 review location. Inspect it externally or choose another file.".into());
    }
    Ok(diff)
}

fn captured_anchor(
    diff: &Value,
    expected: &Comparison,
    side: Side,
    line: u32,
    split: bool,
) -> Result<Anchor, String> {
    let key = if side == Side::Left {
        "oldLine"
    } else {
        "newLine"
    };
    let rows: Vec<&Value> = if split {
        diff["sideRows"]
            .as_array()
            .ok_or("Review comparison has no displayed rows.")?
            .iter()
            .map(|pair| &pair[if side == Side::Left { "left" } else { "right" }])
            .collect()
    } else {
        diff["rows"]
            .as_array()
            .ok_or("Review comparison has no displayed rows.")?
            .iter()
            .collect()
    };
    let row = rows
        .into_iter()
        .find(|row| row[key].as_u64() == Some(line as u64))
        .ok_or(
            "Selected review line is outside the displayed page. Refresh and select it again.",
        )?;
    let excerpt = row["text"]
        .as_str()
        .ok_or("Review line text is unavailable.")?;
    if row["endLine"] != true || row["continued"] == true || excerpt.len() > 2048 {
        return Err("This source line exceeds the 2 KiB review excerpt limit. Choose a smaller nearby line and describe the issue.".into());
    }
    Ok(Anchor {
        comparison: expected.clone(),
        side,
        line,
        excerpt: excerpt.into(),
    })
}

pub(super) fn execute(
    repo: &Repo,
    store: &dyn SessionStore,
    session: &str,
    operation: &Operation,
    cancel: &CancellationToken,
) -> Result<Value, String> {
    if cancel.is_cancelled() {
        return Err(
            "Review operation stopped. Inspect saved findings and the queue before retrying."
                .into(),
        );
    }
    match operation {
        Operation::Capture {
            id,
            comparison: expected,
            cursor,
            side_by_side,
            side,
            line,
            text,
        } => {
            identity(id)?;
            // Lost result inspection is read-only and does not require old files.
            if let Some(saved) = store.review_finding(session, id)? {
                if saved.anchor.comparison == *expected
                    && saved.anchor.side == *side
                    && saved.anchor.line == *line
                    && saved.text == *text
                {
                    return Ok(json!({"finding":saved,"retained":true}));
                }
                return Err(
                    "Finding identity already used. Inspect the saved note before adding another."
                        .into(),
                );
            }
            let diff = comparison(repo, expected, *cursor, *side_by_side, cancel)?;
            let anchor = captured_anchor(&diff, expected, *side, *line, *side_by_side)?;
            let finding = Finding {
                id: id.clone(),
                revision: 0,
                anchor,
                text: text.clone(),
                submissions: vec![],
                resolution: None,
            };
            finding.validate()?;
            if cancel.is_cancelled() {
                return Err(
                    "Review capture stopped before saving. Your note remains in the editor.".into(),
                );
            }
            store.edit_review_finding(
                session,
                &Edit::Add {
                    finding: Box::new(finding.clone()),
                },
            )?;
            Ok(json!({"finding":finding}))
        }
        Operation::Resolve {
            id,
            revision,
            comparison: expected,
            note,
        } => {
            let saved = store
                .review_finding(session, id)?
                .ok_or("Finding no longer exists. Refresh findings.")?;
            if saved.revision == revision.saturating_add(1)
                && saved
                    .resolution
                    .as_ref()
                    .is_some_and(|r| r.comparison == *expected && r.note == *note)
            {
                return Ok(json!({"finding":saved,"retained":true}));
            }
            saved.next(*revision)?;
            comparison(repo, expected, 0, false, cancel)?;
            let evidence = Resolution {
                comparison: expected.clone(),
                note: note.clone(),
            };
            if cancel.is_cancelled() {
                return Err(
                    "Resolution stopped before saving. Inspect the finding before retrying.".into(),
                );
            }
            store.edit_review_finding(
                session,
                &Edit::Resolve {
                    id: id.clone(),
                    revision: *revision,
                    evidence,
                },
            )?;
            Ok(json!({"finding":store.review_finding(session,id)?}))
        }
        Operation::Submit { submission } => {
            if let Some(previous) = store.review_submission(session, &submission.id)? {
                if previous != *submission {
                    return Err("Submission identity already used. Inspect its queue receipt before retrying.".into());
                }
                return Ok(
                    json!({"submission":previous,"inputs":store.task_inputs(session)?,"retained":true}),
                );
            }
            if submission.selected.is_empty() || submission.selected.len() > 8 {
                return Err("Select one to eight findings.".into());
            }
            for selection in &submission.selected {
                let finding = store
                    .review_finding(session, &selection.id)?
                    .ok_or("Selected finding no longer exists. Refresh findings.")?;
                finding.next(selection.revision)?;
                comparison(repo,&finding.anchor.comparison,0,false,cancel).map_err(|error|format!("Finding {} is stale or unavailable: {error} Refresh its diff, inspect the saved excerpt and select a new location before submitting.",finding.id))?;
            }
            if cancel.is_cancelled() {
                return Err("Review submission stopped before acceptance. Inspect the queue before retrying.".into());
            }
            let inputs = store.submit_review_findings(session, submission)?;
            Ok(json!({"submission":submission,"inputs":inputs}))
        }
    }
}
