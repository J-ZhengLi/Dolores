use super::{
    edit::{checkpoint, EditTextFile, TempFile, NEXT_TEMP},
    ReadTextFile,
};
use async_trait::async_trait;
use cap_std::fs::{Dir, OpenOptions};
use dolores_core::{ChangeJournal, ToolCall, ToolPlugin, ToolRequest, ToolSpec, MAX_TOOL_BYTES};
use serde::Deserialize;
use serde_json::json;
use std::{
    collections::HashMap,
    io::Write,
    sync::{atomic::Ordering, Arc, Mutex},
};
use tokio_util::sync::CancellationToken;

pub(super) const EXISTS: &str = "Target already exists. No file was created.";
pub(super) struct CreateTextFile {
    read: ReadTextFile,
    journal: Option<Arc<dyn ChangeJournal>>,
    plans: Mutex<HashMap<String, Plan>>,
}
struct Plan {
    request: ToolRequest,
    parent: Arc<Dir>,
    filename: String,
    content: String,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Arguments {
    path: String,
    content: String,
}

pub(super) fn direct_parent(read: &ReadTextFile, path: &str) -> Result<(Arc<Dir>, String), String> {
    if !super::valid_path(path) {
        return Err("File path is not allowed.".into());
    }
    let (parent, filename) = path.rsplit_once('/').unwrap_or((".", path));
    if parent != "." {
        EditTextFile::new(read.clone()).direct_path(parent)?;
    }
    Ok((
        Arc::new(
            read.directory
                .open_dir(parent)
                .map_err(|_| "File folder is unavailable. Choose an existing folder.")?,
        ),
        filename.into(),
    ))
}
fn absent(parent: &Dir, filename: &str) -> Result<(), String> {
    match parent.symlink_metadata(filename) {
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Ok(_) => Err(EXISTS.into()),
        Err(_) => Err("Target could not be checked.".into()),
    }
}
impl CreateTextFile {
    pub(super) fn new(read: ReadTextFile, journal: Option<Arc<dyn ChangeJournal>>) -> Self {
        Self {
            read,
            journal,
            plans: Mutex::new(HashMap::new()),
        }
    }
    fn apply(
        read: ReadTextFile,
        plan: Plan,
        journal: Option<Arc<dyn ChangeJournal>>,
        cancel: CancellationToken,
    ) -> Result<String, String> {
        checkpoint(&cancel)?;
        direct_parent(&read, &plan.request.target)?;
        absent(&plan.parent, &plan.filename)?;
        let name = format!(
            ".dolores-edit-{}-{}",
            std::process::id(),
            NEXT_TEMP.fetch_add(1, Ordering::Relaxed)
        );
        let mut options = OpenOptions::new();
        options.write(true).create_new(true);
        let mut file = plan
            .parent
            .open_with(&name, &options)
            .map_err(|_| "Could not stage the new file.")?;
        let temp = TempFile {
            parent: &plan.parent,
            name,
        };
        file.write_all(plan.content.as_bytes())
            .map_err(|_| "Could not stage the new file.")?;
        file.sync_all()
            .map_err(|_| "Could not flush the new file.")?;
        drop(file);
        checkpoint(&cancel)?;
        let receipt = journal
            .as_ref()
            .map(|j| j.begin_file_change(&plan.request.target, None, Some(&plan.content)))
            .transpose()?;
        let publish: Result<(), String> = (|| {
            checkpoint(&cancel)?;
            direct_parent(&read, &plan.request.target)?;
            absent(&plan.parent, &plan.filename)?;
            checkpoint(&cancel)?;
            // Link publication is atomic and never replaces an existing entry.
            // Unsupported filesystems fail safely; there is no rename fallback.
            plan.parent
                .hard_link(&temp.name, &plan.parent, &plan.filename)
                .map_err(|error| {
                    if error.kind() == std::io::ErrorKind::AlreadyExists {
                        EXISTS.into()
                    } else {
                        "Could not publish the new file. The filesystem must support hard links."
                            .into()
                    }
                })
        })();
        let saved = match (&journal, receipt) {
            (Some(j), Some(id)) => j.finish(id, publish.is_ok()).is_ok(),
            _ => true,
        };
        publish?;
        let mut result =
            json!({"applied":true,"created":true,"bytesBefore":0,"bytesAfter":plan.content.len()});
        if let Some(id) = receipt {
            result["changeId"] = json!(id);
            result["journalStatus"] = json!(if saved { "applied" } else { "pending" });
        }
        Ok(result.to_string())
    }
}
pub fn create_spec() -> ToolSpec {
    ToolSpec{name:"create_text_file".into(),description:"Create one small UTF-8 text file at a direct relative path under an existing working-folder directory. Requires review of the complete addition and one user approval. Never overwrites existing files, directories or aliases. No directory creation, secret/VCS paths or shell. Content and JSON must fit the 4 KiB argument budget; file/diff cap 16 KiB. Empty files are allowed. Applied creation remains if the later reply fails.".into(),parameters:json!({"type":"object","properties":{"path":{"type":"string"},"content":{"type":"string"}},"required":["path","content"],"additionalProperties":false})}
}
#[async_trait]
impl ToolPlugin for CreateTextFile {
    fn spec(&self) -> ToolSpec {
        create_spec()
    }
    fn prepare(&self, call: &ToolCall) -> Result<ToolRequest, String> {
        if call.name != "create_text_file" || call.arguments.len() > 4096 {
            return Err("Invalid creation arguments.".into());
        }
        let args: Arguments =
            serde_json::from_str(&call.arguments).map_err(|_| "Invalid creation arguments.")?;
        if args.content.len() > MAX_TOOL_BYTES || args.content.contains('\0') {
            return Err("New file exceeds the UTF-8 text limit or contains NUL.".into());
        }
        let (parent, filename) = direct_parent(&self.read, &args.path)?;
        absent(&parent, &filename)?;
        let diff = super::file_change_diff(None, Some(&args.content));
        if diff.len() > MAX_TOOL_BYTES {
            return Err("New file diff exceeds the 16 KiB limit.".into());
        }
        let request = ToolRequest {
            call_id: call.id.clone(),
            name: call.name.clone(),
            target: args.path,
            query: None,
            diff: Some(diff),
            command: None,
        };
        let mut plans = self
            .plans
            .lock()
            .map_err(|_| "Creation preview is unavailable.")?;
        if plans.len() >= dolores_core::MAX_TOOL_CALLS || plans.contains_key(&call.id) {
            return Err("Creation preview limit reached.".into());
        }
        plans.insert(
            call.id.clone(),
            Plan {
                request: request.clone(),
                parent,
                filename,
                content: args.content,
            },
        );
        Ok(request)
    }
    async fn invoke(
        &self,
        request: &ToolRequest,
        cancel: CancellationToken,
    ) -> Result<String, String> {
        let plan = self
            .plans
            .lock()
            .map_err(|_| "Creation preview is unavailable.")?
            .remove(&request.call_id)
            .ok_or("Creation preview expired.")?;
        if request.name != plan.request.name
            || request.target != plan.request.target
            || request.diff != plan.request.diff
            || request.query.is_some()
        {
            return Err("Approved creation changed.".into());
        }
        let read = self.read.clone();
        let journal = self.journal.clone();
        tokio::task::spawn_blocking(move || Self::apply(read, plan, journal, cancel))
            .await
            .map_err(|_| "File creation task failed.")?
    }
}
