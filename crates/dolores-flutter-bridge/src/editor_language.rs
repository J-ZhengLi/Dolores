//! Language edits are one checked buffer transaction, never implicit disk writes.
use super::*;
#[derive(Deserialize)]
#[serde(tag = "action", rename_all = "camelCase", deny_unknown_fields)]
pub(crate) enum Request {
    Preview {
        document: String,
        version: u64,
        edit: Value,
    },
    Apply {
        token: String,
    },
    Undo {
        token: String,
    },
}
pub(super) struct Review {
    token: String,
    root: String,
    created: std::time::Instant,
    applied: bool,
    before: Vec<Document>,
    after: Vec<Document>,
    existed: Vec<bool>,
}
fn path(root: &str, uri: &str) -> Result<String, String> {
    let uri = url::Url::parse(uri).map_err(|_| "Language edit has an invalid file URI.")?;
    let full = uri
        .to_file_path()
        .map_err(|_| "Language edit must target a project file.")?;
    let base = std::path::PathBuf::from(root.trim_start_matches(r"\\?\"));
    if !crate::source_control::within(&full, &base) {
        return Err("Language edit is outside the selected project. Nothing changed.".into());
    }
    let relative = full
        .strip_prefix(&base)
        .map_err(|_| "Language edit path has a different project identity.")?;
    relative
        .to_str()
        .map(|s| s.replace('\\', "/"))
        .ok_or("Language edit path is not Unicode.".into())
}
fn replacements(text: &str, edits: &Value) -> Result<String, String> {
    let edits = edits.as_array().ok_or("Language edits are invalid.")?;
    if edits.len() > 1000 {
        return Err("Language edit exceeds 1,000 replacements. Narrow the change.".into());
    }
    let mut changes = Vec::new();
    for e in edits {
        let start = crate::language::position_byte(text, &e["range"]["start"])?;
        let end = crate::language::position_byte(text, &e["range"]["end"])?;
        let new = e["newText"]
            .as_str()
            .ok_or("Language replacement text unavailable.")?;
        changes.push((start, end, new));
    }
    changes.sort_by_key(|e| e.0);
    let mut output = String::new();
    let mut last = 0;
    for (start, end, new) in changes {
        if start < last || end < start {
            return Err("Language edits overlap. Nothing changed.".into());
        }
        output.push_str(&text[last..start]);
        output.push_str(new);
        last = end;
    }
    output.push_str(&text[last..]);
    if output.len() > 1024 * 1024 {
        return Err("Language result exceeds the 1 MiB editor limit. Nothing changed.".into());
    }
    Ok(output)
}
impl Engine {
    pub(crate) fn language_edits_call(
        &self,
        session: &str,
        request: Request,
    ) -> Result<Value, String> {
        let root = self
            .store
            .workspace(session)?
            .root
            .ok_or("Choose the owning project conversation on Home.")?;
        if self
            .git
            .lock()
            .map_err(|_| "Source Control unavailable.")?
            .mutation_overlaps(std::path::Path::new(&root))
        {
            return Err("Finish the Git action before applying language edits.".into());
        }
        let fs = EditorFolder::new(std::path::Path::new(&root))?;
        let mut state = self
            .editor
            .lock()
            .map_err(|_| "Editor state unavailable.")?;
        let undo_requested = matches!(&request, Request::Undo { .. });
        match request {
            Request::Preview {
                document,
                version,
                edit,
            } => {
                state.language_snapshot(&root, &document, version)?;
                if serde_json::to_vec(&edit)
                    .map_err(|_| "Invalid workspace edit.")?
                    .len()
                    > 4 * 1024 * 1024
                {
                    return Err("Language edit exceeds 4 MiB. Narrow the change.".into());
                }
                let mut paths = BTreeMap::<String, (Option<u64>, Value)>::new();
                if let Some(changes) = edit["changes"].as_object() {
                    for (uri, edits) in changes {
                        if paths
                            .insert(path(&root, uri)?, (None, edits.clone()))
                            .is_some()
                        {
                            return Err("Duplicate language paths are refused.".into());
                        }
                    }
                }
                if let Some(changes) = edit["documentChanges"].as_array() {
                    for change in changes {
                        if change.get("kind").is_some() {
                            return Err("File creation, deletion and rename operations from a language server are unsupported. Nothing changed.".into());
                        }
                        let uri = change["textDocument"]["uri"]
                            .as_str()
                            .ok_or("Language edit URI unavailable.")?;
                        if paths
                            .insert(
                                path(&root, uri)?,
                                (
                                    change["textDocument"]["version"].as_u64(),
                                    change["edits"].clone(),
                                ),
                            )
                            .is_some()
                        {
                            return Err("Duplicate language paths are refused.".into());
                        }
                    }
                }
                if paths.is_empty() || paths.len() > 4 {
                    return Err(
                        "Choose a language edit affecting one to four files. Nothing changed."
                            .into(),
                    );
                }
                let mut before = vec![];
                let mut after = vec![];
                let mut existed = vec![];
                for (path, (expected, edits)) in paths {
                    let old = state
                        .docs
                        .values()
                        .find(|d| d.root == root && d.snapshot.path == path);
                    let was_open = old.is_some();
                    let d = if let Some(d) = old {
                        d.clone()
                    } else {
                        let snapshot = fs.snapshot(&path)?;
                        Document {
                            id: uuid::Uuid::new_v4().to_string(),
                            root: root.clone(),
                            project: project_id(&root),
                            text: snapshot.text.clone(),
                            snapshot,
                            version: 0,
                        }
                    };
                    if d.snapshot.readonly
                        || expected.is_some_and(|v| v != d.version)
                        || fs.snapshot(&path)?.revision != d.snapshot.revision
                    {
                        return Err("Language target is stale or read-only. Compare files and request a fresh preview; nothing changed.".into());
                    }
                    let mut next = d.clone();
                    next.text = replacements(&d.text, &edits)?;
                    validate_text(&next.text, &next.snapshot)?;
                    next.next()?;
                    before.push(d);
                    after.push(next);
                    existed.push(was_open);
                }
                let token = uuid::Uuid::new_v4().to_string();
                let views=before.iter().zip(&after).map(|(a,b)|json!({"path":a.snapshot.path,"document":a.id,"version":a.version,"revision":a.snapshot.revision,"before":a.text,"after":b.text})).collect::<Vec<_>>();
                state.language_review = Some(Review {
                    token: token.clone(),
                    root,
                    created: std::time::Instant::now(),
                    applied: false,
                    before,
                    after,
                    existed,
                });
                Ok(
                    json!({"token":token,"files":views,"note":"Apply changes all reviewed buffers together. Files remain unsaved; Undo language edit restores them together."}),
                )
            }
            Request::Apply { token } | Request::Undo { token } => {
                let review = state
                    .language_review
                    .as_ref()
                    .filter(|r| r.token == token && r.root == root)
                    .ok_or("Language preview expired. Request it again; buffers remain.")?;
                if review.applied != undo_requested {
                    return Err("This language token has already advanced. Use Undo after Apply, or request a new preview.".into());
                }
                if !review.applied && review.created.elapsed().as_secs() > 300 {
                    return Err(
                        "Language preview expired after five minutes. Request it again.".into(),
                    );
                }
                let undo = review.applied;
                // A token advances preview -> applied -> undone exactly once.
                let expected = if undo { &review.after } else { &review.before };
                let target = if undo { &review.before } else { &review.after };
                let mut docs = state.docs.clone();
                for ((old, new), existed) in expected.iter().zip(target).zip(&review.existed) {
                    let current = docs.get(&old.id);
                    if (undo || *existed)
                        && current.is_none_or(|d| {
                            d.root != root || d.version != old.version || d.text != old.text
                        })
                        || (!undo
                            && !*existed
                            && docs
                                .values()
                                .any(|d| d.root == root && d.snapshot.path == old.snapshot.path))
                        || fs.snapshot(&old.snapshot.path)?.revision != old.snapshot.revision
                    {
                        return Err("A reviewed buffer or disk file changed. Request a fresh language preview; all edits remain.".into());
                    }
                    let mut next = new.clone();
                    if undo {
                        next.version = old.version;
                        next.next()?;
                    }
                    docs.insert(next.id.clone(), next);
                }
                if docs.len() > 4
                    || docs
                        .values()
                        .map(|d| d.text.len() + d.snapshot.text.len())
                        .sum::<usize>()
                        > 4 * 1024 * 1024
                {
                    return Err("Language change exceeds four open files or 4 MiB resident text. Close an inactive file and preview again.".into());
                }
                let staged = Registry {
                    docs,
                    ..Default::default()
                };
                self.editor_checkpoint(&staged, &root)?;
                let views = target
                    .iter()
                    .map(|d| staged.docs[&d.id].view())
                    .collect::<Vec<_>>();
                state.docs = staged.docs;
                if undo {
                    state.language_review = None;
                } else {
                    state.language_review.as_mut().unwrap().applied = true;
                }
                Ok(json!({"token":token,"undone":undo,"documents":views}))
            }
        }
    }
}
