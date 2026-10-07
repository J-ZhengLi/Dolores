use crate::Engine;
use dolores_tools_fs::{EditorFolder, EditorSnapshot};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;

#[derive(Deserialize)]
#[serde(tag = "action", rename_all = "camelCase", deny_unknown_fields)]
pub(crate) enum Request {
    Workspace,
    Attach {
        project: String,
        document: String,
        version: u64,
        start: usize,
        end: usize,
        target: String,
    },
    Refresh {
        project: String,
        document: String,
        version: u64,
    },
    Tree {
        project: String,
        path: String,
        cursor: usize,
    },
    Open {
        project: String,
        path: String,
    },
    Edit {
        project: String,
        document: String,
        version: u64,
        edits: Vec<Edit>,
    },
    Save {
        project: String,
        document: String,
        version: u64,
    },
    Compare {
        project: String,
        document: String,
        version: u64,
    },
    Reload {
        project: String,
        document: String,
        version: u64,
        revision: String,
    },
    Rebase {
        project: String,
        document: String,
        version: u64,
        revision: String,
    },
    SaveAs {
        project: String,
        document: String,
        version: u64,
        path: String,
    },
    Close {
        project: String,
        document: String,
        version: u64,
        discard: bool,
    },
    Create {
        project: String,
        path: String,
    },
    Rename {
        project: String,
        document: String,
        version: u64,
        path: String,
    },
    Delete {
        project: String,
        document: String,
        version: u64,
    },
    Checkpoint {
        project: String,
    },
    Layout {
        project: String,
        layout: Value,
    },
    Recover {
        project: String,
        path: String,
    },
}
#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Edit {
    start: usize,
    end: usize,
    text: String,
}
#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Document {
    id: String,
    root: String,
    project: String,
    snapshot: EditorSnapshot,
    text: String,
    version: u64,
}
impl Document {
    fn dirty(&self) -> bool {
        self.text != self.snapshot.text
    }
    fn view(&self) -> Value {
        json!({"document":self.id,"project":self.project,"snapshot":self.snapshot,"text":self.text,"version":self.version,"dirty":self.dirty()})
    }
    fn next(&mut self) -> Result<(), String> {
        self.version = self
            .version
            .checked_add(1)
            .ok_or("Document version exhausted.")?;
        Ok(())
    }
}
#[derive(Default)]
pub(crate) struct Registry {
    docs: BTreeMap<String, Document>,
    language_review: Option<language_edits::Review>,
}
#[path="editor_language.rs"]
pub(crate) mod language_edits;
fn project_id(root: &str) -> String {
    format!("{:x}", Sha256::digest(root.as_bytes()))
}
fn utf16_byte(text: &str, offset: usize) -> Result<usize, String> {
    let mut count = 0;
    for (byte, c) in text.char_indices() {
        if count == offset {
            return Ok(byte);
        }
        count += c.len_utf16();
        if count > offset {
            return Err("Edit splits a Unicode character. Previous buffer is retained.".into());
        }
    }
    if count == offset {
        Ok(text.len())
    } else {
        Err("Edit range is outside the document.".into())
    }
}
fn apply_edits(text: &str, edits: &[Edit]) -> Result<String, String> {
    if serde_json::to_vec(edits)
        .map_err(|_| "Invalid edit transaction.")?
        .len()
        > 512 * 1024
    {
        return Err("Serialized edit exceeds the 512 KiB limit.".into());
    }
    if edits.is_empty()
        || edits.len() > 16
        || edits.iter().map(|e| e.text.len()).sum::<usize>() > 64 * 1024
    {
        return Err("Edit exceeds the 64 KiB / 16 replacement limit. Split the paste; previous buffer is retained.".into());
    }
    let mut out = String::with_capacity(text.len());
    let mut last = 0;
    for e in edits {
        let start = utf16_byte(text, e.start)?;
        let end = utf16_byte(text, e.end)?;
        if start < last || end < start {
            return Err("Edits overlap or are out of order.".into());
        }
        out.push_str(&text[last..start]);
        out.push_str(&e.text);
        last = end;
    }
    out.push_str(&text[last..]);
    Ok(out)
}
impl Registry {
    pub(super) fn language_snapshot(&self, root: &str, id: &str, version: u64) -> Result<Value,String> {
        let d = self.docs.get(id).ok_or("Document closed. Reopen it before using language services.")?;
        if d.root != root || d.version != version || d.snapshot.readonly {
            return Err("Document changed or is read-only. Request the language feature again; edits remain.".into());
        }
        let mut view=d.view();
        view["openPaths"]=json!(self.docs.values().filter(|d|d.root==root).map(|d|d.snapshot.path.clone()).collect::<Vec<_>>());
        Ok(view)
    }
    pub(super) fn ensure_git_clean(&self,root:&std::path::Path,paths:&[String])->Result<(),String>{
        for d in self.docs.values().filter(|d|d.dirty()) {
            let full=std::path::Path::new(&d.root).join(&d.snapshot.path);
            if (paths.is_empty()&&crate::source_control::within(&full,root))||paths.iter().any(|p|crate::source_control::path_key(&full)==crate::source_control::path_key(&root.join(p))){return Err("Save or close unsaved editors for these Git paths, then review again. Their drafts remain.".into());}
        }Ok(())
    }
    fn replacement(&self,old:&Document,next:&Document)->Result<(),String>{
        if self.bytes()-old.snapshot.text.len()-old.text.len()+next.snapshot.text.len()+next.text.len()>4*1024*1024{
            return Err("Resident text limit reached. Save and close an inactive document; previous buffer is retained.".into());
        }Ok(())
    }
    fn bytes(&self) -> usize {
        self.docs
            .values()
            .map(|d| d.snapshot.text.len() + d.text.len())
            .sum()
    }
    fn admit(&self, d: &Document) -> Result<(), String> {
        if self.docs.len() >= 4
            || self.bytes() + d.snapshot.text.len() + d.text.len() > 4 * 1024 * 1024
        {
            return Err("Four documents or the 4 MiB resident-text limit are reached. Save and close an inactive document before opening another.".into());
        }
        Ok(())
    }
}
impl Engine {
    pub(super) fn editor_can_restart(&self)->Result<(),String>{
        if !self.git.lock().map_err(|_|"Source Control is unavailable.")?.active.is_empty(){return Err("Finish or stop Source Control before restarting native code.".into());}
        if self.editor.lock().map_err(|_|"Editor state is unavailable.")?.docs.values().any(Document::dirty){
            return Err("Save or close unsaved file editors before installing or restoring native code. Their edits are retained; no restart occurred.".into());
        }Ok(())
    }
    pub(crate) fn editor_call(&self, session: &str, request: Request) -> Result<Value, String> {
        let root = self
            .store
            .workspace(session)?
            .root
            .ok_or("Select a working project conversation on Home or open a folder.")?;
        let project = project_id(&root);
        if !matches!(&request,Request::Workspace|Request::Tree{..}|Request::Open{..}|Request::Compare{..}|Request::Checkpoint{..}|Request::Layout{..}|Request::Attach{..}|Request::Close{..})
            && self.git.lock().map_err(|_|"Source Control is unavailable.")?.mutation_overlaps(std::path::Path::new(&root)) {
            return Err("A Git mutation is running in this worktree. Finish it before editing or saving; drafts remain.".into());
        }
        let supplied = match &request {
            Request::Workspace => None,
            Request::Attach { project, .. } | Request::Refresh { project, .. } => Some(project),
            Request::Tree { project, .. }
            | Request::Open { project, .. }
            | Request::Edit { project, .. }
            | Request::Save { project, .. }
            | Request::Compare { project, .. }
            | Request::Reload { project, .. }
            | Request::Rebase { project, .. }
            | Request::SaveAs { project, .. }
            | Request::Close { project, .. }
            | Request::Create { project, .. }
            | Request::Rename { project, .. }
            | Request::Delete { project, .. }
            | Request::Checkpoint { project }
            | Request::Layout { project, .. }
            | Request::Recover { project, .. } => Some(project),
        };
        if supplied.is_some_and(|p| p != &project) {
            return Err("Selected project changed. Return to the owning conversation; documents are retained.".into());
        }
        let fs = EditorFolder::new(std::path::Path::new(&root))?;
        let mut registry = self
            .editor
            .lock()
            .map_err(|_| "Editor state is unavailable.")?;
        let create = matches!(&request, Request::Create { .. });
        let recover = matches!(&request, Request::Recover { .. });
        match request {
            Request::Workspace => {
                let saved = self.store.editor_state(&root)?;
                let recovery = saved["documents"]
                    .as_array()
                    .map(|a| {
                        a.iter()
                            .filter_map(|v| serde_json::from_value::<Document>(v.clone()).ok())
                            .filter(|d| d.root == root && d.project == project && d.dirty())
                            .map(|d| json!({"path":d.snapshot.path}))
                            .take(4)
                            .collect::<Vec<_>>()
                    })
                    .unwrap_or_default();
                Ok(
                    json!({"project":project,"root":root,"layout":saved["layout"],"recovery":recovery,"documents":registry.docs.values().filter(|d|d.project==project).map(Document::view).collect::<Vec<_>>()}),
                )
            }
            Request::Tree { path, cursor, .. } => fs.tree(&path, cursor),
            Request::Checkpoint { .. } => {
                self.editor_checkpoint(&registry, &root)?;
                Ok(Value::Null)
            }
            Request::Layout { layout, .. } => {
                validate_layout(&layout)?;
                let mut saved = self.store.editor_state(&root)?;
                if !saved.is_object() {
                    saved = json!({"version":1});
                }
                saved["layout"] = layout;
                self.store.save_editor_state(&root, &saved)?;
                Ok(Value::Null)
            }
            Request::Open { path, .. }
            | Request::Create { path, .. }
            | Request::Recover { path, .. } => {
                if let Some(d) = registry
                    .docs
                    .values()
                    .find(|d| d.project == project && d.snapshot.path == path)
                {
                    return Ok(d.view());
                }
                if registry.docs.len() >= 4 {
                    return Err("Four documents are retained. Save and close a document before opening another.".into());
                }
                let d = if recover {
                    let saved = self.store.editor_state(&root)?;
                    let mut d = saved["documents"]
                        .as_array()
                        .ok_or("No private recovery is available.")?
                        .iter()
                        .filter_map(|v| serde_json::from_value::<Document>(v.clone()).ok())
                        .find(|d| {
                            d.root == root
                                && d.project == project
                                && d.snapshot.path == path
                                && d.dirty()
                        })
                        .ok_or("This recovery is no longer available.")?;
                    if d.snapshot.readonly {
                        return Err("Invalid read-only recovery document.".into());
                    }
                    validate_text(&d.text, &d.snapshot)?;
                    validate_text(&d.snapshot.text, &d.snapshot)?;
                    d.id = uuid::Uuid::new_v4().to_string();
                    d.version = 0;
                    d
                } else {
                    let snapshot = if create {
                        fs.create(&path, "", false, "lf")?
                    } else {
                        fs.snapshot(&path)?
                    };
                    Document {
                        id: uuid::Uuid::new_v4().to_string(),
                        root: root.clone(),
                        project: project.clone(),
                        text: snapshot.text.clone(),
                        snapshot,
                        version: 0,
                    }
                };
                registry.admit(&d)?;
                let view = d.view();
                registry.docs.insert(d.id.clone(), d);
                Ok(view)
            }
            other => {
                let (id, version) = match &other {
                    Request::Attach {
                        document, version, ..
                    }
                    | Request::Refresh {
                        document, version, ..
                    } => (document, version),
                    Request::Edit {
                        document, version, ..
                    }
                    | Request::Save {
                        document, version, ..
                    }
                    | Request::Compare {
                        document, version, ..
                    }
                    | Request::Reload {
                        document, version, ..
                    }
                    | Request::Rebase {
                        document, version, ..
                    }
                    | Request::SaveAs {
                        document, version, ..
                    }
                    | Request::Close {
                        document, version, ..
                    }
                    | Request::Rename {
                        document, version, ..
                    }
                    | Request::Delete {
                        document, version, ..
                    } => (document, version),
                    _ => unreachable!(),
                };
                let d = registry.docs.get(id).ok_or(
                    "Document is no longer open. Reopen the file; local edits are retained.",
                )?;
                if d.project != project || d.root != root || d.version != *version {
                    return Err("Document version or project changed. Compare before continuing; local edits are retained.".into());
                }
                let id = id.clone();
                let mut next = d.clone();
                let reload = matches!(&other, Request::Reload { .. });
                match other {
                    Request::Refresh { .. } => {
                        let disk = fs.snapshot(&d.snapshot.path)?;
                        if disk.revision == d.snapshot.revision {
                            return Ok(json!({"changed":false}));
                        }
                        if d.dirty() {
                            return Ok(json!({"changed":true}));
                        }
                        next.text = disk.text.clone();
                        next.snapshot = disk;
                        registry.replacement(d,&next)?;
                        next.next()?;
                        let view = next.view();
                        registry.docs.insert(id, next);
                        Ok(view)
                    }
                    Request::Attach {
                        start, end, target, ..
                    } => {
                        if d.snapshot.readonly {
                            return Err(
                                "Select text from an editable UTF-8 document before attaching."
                                    .into(),
                            );
                        }
                        let a = utf16_byte(&d.text, start)?;
                        let b = utf16_byte(&d.text, end)?;
                        if b <= a || b - a > 60 * 1024 {
                            return Err(
                                "Select nonempty text within 60 KiB before attaching.".into()
                            );
                        }
                        let target_root = self.store.workspace(&target)?.root;
                        if target_root.as_deref() != Some(&root) {
                            return Err(
                                "Choose a conversation in this project for a source attachment."
                                    .into(),
                            );
                        }
                        let data=format!("Source file: {}\nEditor version: {} ({})\nSaved-byte revision: {}\nUTF-16 range: {}..{}\n\n{}",d.snapshot.path,d.version,if d.dirty(){"unsaved"}else{"saved"},d.snapshot.revision,start,end,&d.text[a..b]).into_bytes();
                        let reference = dolores_core::AttachmentRef {
                            digest: format!("{:x}", Sha256::digest(&data)),
                            name: format!(
                                "Selection v{} - {}",
                                d.version,
                                d.snapshot.path.rsplit('/').next().unwrap_or("source")
                            ),
                            mime: "text/plain".into(),
                            bytes: data.len(),
                        };
                        reference.validate()?;
                        self.store.add_attachment(
                            &target,
                            &dolores_core::AttachmentData { reference, data },
                        )?;
                        Ok(json!(self.store.draft_attachments(&target)?))
                    }
                    Request::Edit { edits, .. } => {
                        if d.snapshot.readonly {
                            return Err(
                                "This preview is read-only. Original bytes are unchanged.".into()
                            );
                        }
                        let text = apply_edits(&d.text, &edits)?;
                        validate_text(&text, &d.snapshot)?;
                        if registry.bytes() - d.text.len() + text.len() > 4 * 1024 * 1024 {
                            return Err("Resident text limit reached. Save and close an inactive document; previous buffer is retained.".into());
                        }
                        next.text = text;
                        next.next()?;
                        let ack = json!({"version":next.version,"dirty":next.dirty()});
                        registry.docs.insert(id, next);
                        Ok(ack)
                    }
                    Request::Compare { .. } => {
                        Ok(json!({"disk":fs.snapshot(&d.snapshot.path)?,"version":d.version}))
                    }
                    Request::Save { .. } => {
                        if d.snapshot.readonly {
                            return Err("This preview cannot be saved.".into());
                        }
                        if registry.bytes()-d.snapshot.text.len()+d.text.len()>4*1024*1024{
                            return Err("Resident text limit reached. Close an inactive document before saving.".into());
                        }
                        next.next()?;
                        next.snapshot = fs.save(
                            &d.snapshot.path,
                            &d.snapshot.revision,
                            &d.text,
                            d.snapshot.bom,
                            &d.snapshot.newline,
                        )?;
                        next.text = next.snapshot.text.clone();
                        let view = next.view();
                        registry.docs.insert(id, next);
                        if let Err(error) = self.editor_checkpoint(&registry, &root) {
                            return Ok(json!({"saved":view,"recoveryWarning":error}));
                        }
                        Ok(view)
                    }
                    Request::Reload { revision, .. } | Request::Rebase { revision, .. } => {
                        let disk = fs.snapshot(&d.snapshot.path)?;
                        if disk.revision != revision {
                            return Err("Disk changed again after comparison. Compare again; edits are retained.".into());
                        }
                        if disk.readonly {
                            return Err(
                                "Changed file is unsupported or read-only. Keep edits or Save as."
                                    .into(),
                            );
                        }
                        next.snapshot = disk;
                        if reload {
                            next.text = next.snapshot.text.clone();
                        } else {
                            validate_text(&next.text, &next.snapshot)?;
                        }
                        registry.replacement(d,&next)?;
                        next.next()?;
                        let view = next.view();
                        registry.docs.insert(id, next);
                        if let Err(error)=self.editor_checkpoint(&registry,&root){
                            return Ok(json!({"saved":view,"recoveryWarning":error}));
                        }
                        Ok(view)
                    }
                    Request::SaveAs { path, .. } => {
                        if d.snapshot.readonly {
                            return Err(
                                "Read-only previews cannot be converted through Save as.".into()
                            );
                        }
                        let old_path = d.snapshot.path.clone();
                        if registry.bytes()-d.snapshot.text.len()+d.text.len()>4*1024*1024{
                            return Err("Resident text limit reached. Close an inactive document before saving.".into());
                        }
                        next.next()?;
                        next.snapshot =
                            fs.create(&path, &d.text, d.snapshot.bom, &d.snapshot.newline)?;
                        let view = next.view();
                        registry.docs.insert(id, next);
                        if let Err(error) =
                            self.editor_checkpoint_forget(&registry, &root, Some(&old_path))
                        {
                            return Ok(json!({"saved":view,"recoveryWarning":error}));
                        }
                        Ok(view)
                    }
                    Request::Close { discard, .. } => {
                        if d.dirty() && !discard {
                            return Err(
                                "Save or explicitly discard this document before closing it."
                                    .into(),
                            );
                        }
                        let old = registry.docs.remove(&id).unwrap();
                        if let Err(e) = self.editor_checkpoint_forget(
                            &registry,
                            &root,
                            Some(&old.snapshot.path),
                        ) {
                            registry.docs.insert(id, old);
                            return Err(e);
                        }
                        Ok(Value::Null)
                    }
                    Request::Rename { path, .. } => {
                        if d.dirty() {
                            return Err("Save or close dirty views before renaming.".into());
                        }
                        if registry
                            .docs
                            .values()
                            .any(|x| x.project == project && x.snapshot.path == path)
                        {
                            return Err("Destination is already open.".into());
                        }
                        let old_path=d.snapshot.path.clone();
                        next.next()?;
                        next.snapshot = fs.rename(&d.snapshot.path, &path, &d.snapshot.revision)?;
                        next.text = next.snapshot.text.clone();
                        let view = next.view();
                        registry.docs.insert(id, next);
                        if let Err(error)=self.editor_checkpoint_forget(&registry,&root,Some(&old_path)){
                            return Ok(json!({"saved":view,"recoveryWarning":error}));
                        }
                        Ok(view)
                    }
                    Request::Delete { .. } => {
                        if d.dirty() {
                            return Err("Save or close dirty views before deleting.".into());
                        }
                        let path = d.snapshot.path.clone();
                        fs.delete(&path, &d.snapshot.revision)?;
                        registry.docs.remove(&id);
                        if let Err(error)=self.editor_checkpoint_forget(&registry,&root,Some(&path)){
                            return Ok(json!({"deleted":true,"recoveryWarning":error}));
                        }
                        Ok(Value::Null)
                    }
                    _ => unreachable!(),
                }
            }
        }
    }
    fn editor_checkpoint(&self, registry: &Registry, root: &str) -> Result<(), String> {
        self.editor_checkpoint_forget(registry, root, None)
    }
    fn editor_checkpoint_forget(
        &self,
        registry: &Registry,
        root: &str,
        forget: Option<&str>,
    ) -> Result<(), String> {
        let mut saved = self.store.editor_state(root)?;
        if !saved.is_object() {
            saved = json!({"version":1});
        }
        let mut docs = saved["documents"]
            .as_array()
            .map(|a| {
                a.iter()
                    .filter_map(|v| serde_json::from_value::<Document>(v.clone()).ok())
                    .filter(|d| {
                        d.root == root
                            && d.dirty()
                            && Some(d.snapshot.path.as_str()) != forget
                            && !registry
                                .docs
                                .values()
                                .any(|x| x.root == root && x.snapshot.path == d.snapshot.path)
                    })
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();
        docs.extend(
            registry
                .docs
                .values()
                .filter(|d| d.root == root && d.dirty())
                .cloned(),
        );
        if docs.len() > 4
            || docs
                .iter()
                .map(|d| d.text.len() + d.snapshot.text.len())
                .sum::<usize>()
                > 4 * 1024 * 1024
        {
            return Err("Private recovery limit reached. Recover and save or discard the earlier documents first; current edits remain open.".into());
        }
        saved["documents"] = json!(docs);
        self.store.save_editor_state(root, &saved)
    }
}
fn validate_text(text: &str, s: &EditorSnapshot) -> Result<(), String> {
    if text.contains(['\0', '\r']) || text.lines().any(|l| l.len() > 8192) {
        return Err("Edit exceeds the 8 KiB line limit or contains unsupported text.".into());
    }
    let size = text.len()
        + if s.bom { 3 } else { 0 }
        + if s.newline == "crlf" {
            text.bytes().filter(|b| *b == b'\n').count()
        } else {
            0
        };
    if size > 1024 * 1024 {
        return Err(
            "Edit exceeds the 1 MiB file limit. Split the change; previous buffer is retained."
                .into(),
        );
    }
    Ok(())
}
fn validate_layout(layout: &Value) -> Result<(), String> {
    if layout.to_string().len() > 8192
        || layout["version"] != 1
        || layout["groups"]
            .as_array()
            .is_none_or(|g| g.is_empty() || g.len() > 4)
    {
        return Err("Layout is invalid. Use the safe default layout.".into());
    }
    Ok(())
}
#[cfg(test)]
#[path = "editor_tests.rs"]
mod tests;
