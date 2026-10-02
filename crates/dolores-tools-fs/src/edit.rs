use super::ReadTextFile;
use async_trait::async_trait;
use cap_std::fs::{Dir, OpenOptions, Permissions};
use dolores_core::{ChangeJournal, ToolCall, ToolPlugin, ToolRequest, ToolSpec, MAX_TOOL_BYTES};
use serde::Deserialize;
use serde_json::json;
use std::{
    collections::HashMap,
    io::{Read, Write},
    sync::{
        atomic::{AtomicU64, Ordering},
        Arc, Mutex,
    },
};
use tokio_util::sync::CancellationToken;

const CHANGED: &str = "File changed since preview. No edit was applied.";
pub(super) static NEXT_TEMP: AtomicU64 = AtomicU64::new(0);

pub struct EditTextFile {
    read: ReadTextFile,
    plans: Mutex<HashMap<String, Plan>>,
    journal: Option<Arc<dyn ChangeJournal>>,
}
struct Plan {
    request: ToolRequest,
    parent: Arc<Dir>,
    filename: String,
    before: String,
    after: String,
    permissions: Permissions,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Arguments {
    path: String,
    old_text: String,
    new_text: String,
}

pub(super) fn checkpoint(cancel: &CancellationToken) -> Result<(), String> {
    if cancel.is_cancelled() {
        Err("Response stopped.".into())
    } else {
        Ok(())
    }
}
pub(super) fn snapshot(directory: &Dir, path: &str) -> Result<(String, Permissions), String> {
    let mut options = OpenOptions::new();
    options.read(true);
    #[cfg(unix)]
    {
        use cap_std::fs::OpenOptionsExt;
        options.custom_flags(libc::O_NONBLOCK);
    }
    let file = directory
        .open_with(path, &options)
        .map_err(|_| "File is unavailable.")?;
    let metadata = file.metadata().map_err(|_| "File is unavailable.")?;
    if !metadata.is_file() || metadata.permissions().readonly() {
        return Err("Only writable regular text files can be edited.".into());
    }
    let mut bytes = Vec::new();
    file.take((MAX_TOOL_BYTES + 1) as u64)
        .read_to_end(&mut bytes)
        .map_err(|_| "File could not be read.")?;
    if bytes.len() > MAX_TOOL_BYTES || bytes.contains(&0) {
        return Err("File exceeds the text limit or is binary.".into());
    }
    Ok((
        String::from_utf8(bytes).map_err(|_| "File is not UTF-8 text.")?,
        metadata.permissions(),
    ))
}

// One hunk contains the changed lines and three context lines on each side.
// Long unchanged files do not inflate the approval card or saved metadata.
pub fn change_diff(before: &str, after: &str) -> String {
    let old: Vec<_> = before.split_inclusive('\n').collect();
    let new: Vec<_> = after.split_inclusive('\n').collect();
    let prefix = old.iter().zip(&new).take_while(|(a, b)| a == b).count();
    let suffix = old[prefix..]
        .iter()
        .rev()
        .zip(new[prefix..].iter().rev())
        .take_while(|(a, b)| a == b)
        .count();
    let start = prefix.saturating_sub(3);
    let old_end = (old.len() - suffix + 3).min(old.len());
    let new_end = (new.len() - suffix + 3).min(new.len());
    let line = |index: usize, count: usize| if count == 0 { index } else { index + 1 };
    let mut result = format!(
        "--- before\n+++ after\n@@ -{},{} +{},{} @@\n",
        line(start, old_end - start),
        old_end - start,
        line(start, new_end - start),
        new_end - start
    );
    let mut append = |marker: char, text: &str| {
        result.push(marker);
        result.push_str(text);
        if !text.ends_with('\n') {
            result.push_str("\n\\ No newline at end of file\n");
        }
    };
    for text in &old[start..prefix] {
        append(' ', text);
    }
    for text in &old[prefix..old.len() - suffix] {
        append('-', text);
    }
    for text in &new[prefix..new.len() - suffix] {
        append('+', text);
    }
    for text in &old[old.len() - suffix..old_end] {
        append(' ', text);
    }
    result
}

pub fn file_change_diff(before: Option<&str>, after: Option<&str>) -> String {
    let diff = change_diff(before.unwrap_or_default(), after.unwrap_or_default());
    let diff = if before.is_none() {
        diff.replacen("--- before", "--- /dev/null", 1)
    } else {
        diff
    };
    if after.is_none() {
        diff.replacen("+++ after", "+++ /dev/null", 1)
    } else {
        diff
    }
}

pub(super) struct TempFile<'a> {
    pub(super) parent: &'a Dir,
    pub(super) name: String,
}
impl Drop for TempFile<'_> {
    fn drop(&mut self) {
        let _ = self.parent.remove_file(&self.name);
    }
}
impl EditTextFile {
    pub fn new(read: ReadTextFile) -> Self {
        Self {
            read,
            plans: Mutex::new(HashMap::new()),
            journal: None,
        }
    }
    pub(super) fn direct_path(&self, path: &str) -> Result<(), String> {
        if self.read.resolve(path)? != path {
            return Err("Only direct file paths can be edited.".into());
        }
        let mut component = String::new();
        for part in path.split('/') {
            if !component.is_empty() {
                component.push('/');
            }
            component.push_str(part);
            if self
                .read
                .directory
                .symlink_metadata(&component)
                .map_err(|_| "File is unavailable.")?
                .file_type()
                .is_symlink()
            {
                return Err("File aliases cannot be edited.".into());
            }
        }
        Ok(())
    }
    pub(super) fn journaled(read: ReadTextFile, journal: Arc<dyn ChangeJournal>) -> Self {
        Self {
            journal: Some(journal),
            ..Self::new(read)
        }
    }
    fn apply(
        read: ReadTextFile,
        plan: Plan,
        cancel: CancellationToken,
        journal: Option<Arc<dyn ChangeJournal>>,
    ) -> Result<String, String> {
        checkpoint(&cancel)?;
        Self::new(read.clone())
            .direct_path(&plan.request.target)
            .map_err(|_| CHANGED)?;
        let (current, permissions) = snapshot(&plan.parent, &plan.filename).map_err(|_| CHANGED)?;
        if current != plan.before || permissions != plan.permissions {
            return Err(CHANGED.into());
        }
        checkpoint(&cancel)?;
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
            .map_err(|_| "Could not stage the file edit.")?;
        let temp = TempFile {
            parent: &plan.parent,
            name,
        };
        file.write_all(plan.after.as_bytes())
            .map_err(|_| "Could not stage the file edit.")?;
        file.set_permissions(plan.permissions.clone())
            .map_err(|_| "Could not preserve file permissions.")?;
        file.sync_all()
            .map_err(|_| "Could not flush the file edit.")?;
        drop(file);
        // Recheck after staging, immediately before the short publish operation.
        Self::new(read.clone())
            .direct_path(&plan.request.target)
            .map_err(|_| CHANGED)?;
        let (current, permissions) = snapshot(&plan.parent, &plan.filename).map_err(|_| CHANGED)?;
        if current != plan.before || permissions != plan.permissions {
            return Err(CHANGED.into());
        }
        checkpoint(&cancel)?;
        // Durably save intent before publishing. Failure here prevents the write.
        let receipt = journal
            .as_ref()
            .map(|j| j.begin(&plan.request.target, &plan.before, &plan.after))
            .transpose()?;
        let publish = (|| {
            checkpoint(&cancel)?;
            // Persistence may block; check the file again after it completes.
            Self::new(read)
                .direct_path(&plan.request.target)
                .map_err(|_| CHANGED)?;
            let (current, permissions) =
                snapshot(&plan.parent, &plan.filename).map_err(|_| CHANGED)?;
            if current != plan.before || permissions != plan.permissions {
                return Err(CHANGED.into());
            }
            checkpoint(&cancel)?;
            plan.parent
                .rename(&temp.name, &plan.parent, &plan.filename)
                .map_err(|_| "Could not replace the file.".to_string())
        })();
        let saved = match (&journal, receipt) {
            (Some(journal), Some(id)) => journal.finish(id, publish.is_ok()).is_ok(),
            _ => true,
        };
        publish?;
        let mut result =
            json!({"applied":true,"bytesBefore":plan.before.len(),"bytesAfter":plan.after.len()});
        if let Some(id) = receipt {
            result["changeId"] = json!(id);
            result["journalStatus"] = json!(if saved { "applied" } else { "pending" });
        }
        Ok(result.to_string())
    }
}
#[async_trait]
impl ToolPlugin for EditTextFile {
    fn spec(&self) -> ToolSpec {
        ToolSpec {
            name:"edit_text_file".into(),
            description:"Replace one exact, unique old_text occurrence with new_text in an existing writable UTF-8 file under the working folder. Relative direct paths only, files up to 16 KiB. The user must review the local diff and approve once before writing. No creation, deletion, shell, aliases or VCS/credential files. Changed files require a fresh preview. Applied edits remain if a later reply fails.".into(),
            parameters:json!({"type":"object","properties":{"path":{"type":"string"},"old_text":{"type":"string","minLength":1},"new_text":{"type":"string"}},"required":["path","old_text","new_text"],"additionalProperties":false}),
        }
    }
    fn prepare(&self, call: &ToolCall) -> Result<ToolRequest, String> {
        if call.name != "edit_text_file" || call.arguments.len() > 4096 {
            return Err("Invalid edit arguments.".into());
        }
        let args: Arguments =
            serde_json::from_str(&call.arguments).map_err(|_| "Invalid edit arguments.")?;
        if args.old_text.is_empty()
            || args.old_text == args.new_text
            || args.new_text.contains('\0')
        {
            return Err("Edit needs different text and a nonempty match.".into());
        }
        self.direct_path(&args.path)?;
        let (parent, filename) = args.path.rsplit_once('/').unwrap_or((".", &args.path));
        let parent = Arc::new(
            self.read
                .directory
                .open_dir(parent)
                .map_err(|_| "File folder is unavailable.")?,
        );
        let (before, permissions) = snapshot(&parent, filename)?;
        let mut positions = before.match_indices(&args.old_text);
        let (position, _) = positions.next().ok_or("Exact edit text was not found.")?;
        if positions.next().is_some() {
            return Err("Edit text occurs more than once. Use a larger unique match.".into());
        }
        // Reject overlapping matches too (for example aa in aaa).
        let next = position + args.old_text.chars().next().unwrap().len_utf8();
        if before[next..].contains(&args.old_text) {
            return Err("Edit text occurs more than once.".into());
        }
        let after = before.replacen(&args.old_text, &args.new_text, 1);
        if after.len() > MAX_TOOL_BYTES {
            return Err("Edited file exceeds the 16 KiB limit.".into());
        }
        let preview = change_diff(&before, &after);
        if preview.len() > MAX_TOOL_BYTES {
            return Err("Edit diff exceeds the 16 KiB limit. Use a smaller edit.".into());
        }
        let request = ToolRequest {
            call_id: call.id.clone(),
            name: call.name.clone(),
            target: args.path.clone(),
            query: None,
            diff: Some(preview),
        };
        let mut plans = self
            .plans
            .lock()
            .map_err(|_| "Edit preview is unavailable.")?;
        if plans.len() >= dolores_core::MAX_TOOL_CALLS || plans.contains_key(&call.id) {
            return Err("Edit preview limit reached.".into());
        }
        plans.insert(
            call.id.clone(),
            Plan {
                request: request.clone(),
                parent,
                filename: filename.into(),
                before,
                after,
                permissions,
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
            .map_err(|_| "Edit preview is unavailable.")?
            .remove(&request.call_id)
            .ok_or("Edit preview expired.")?;
        if request.name != plan.request.name
            || request.target != plan.request.target
            || request.query.is_some()
            || request.diff != plan.request.diff
        {
            return Err("Approved edit changed.".into());
        }
        let read = self.read.clone();
        let journal = self.journal.clone();
        // Stop is checked inside the worker, including just before publishing.
        // Once the atomic replacement starts, finish it and report its outcome.
        tokio::task::spawn_blocking(move || Self::apply(read, plan, cancel, journal))
            .await
            .map_err(|_| "File edit task failed.")?
    }
}

/// Opaque, local-only plan. The bridge binds it to a chat and a one-use token.
pub struct RevertPlan {
    read: ReadTextFile,
    plan: Plan,
}
impl RevertPlan {
    pub fn preview(
        root: &std::path::Path,
        target: &str,
        expected: &str,
        restored: &str,
    ) -> Result<Self, String> {
        if expected == restored
            || expected.len() > MAX_TOOL_BYTES
            || restored.len() > MAX_TOOL_BYTES
            || restored.contains('\0')
        {
            return Err("Invalid revert snapshot.".into());
        }
        let read = ReadTextFile::new(root)?;
        EditTextFile::new(read.clone()).direct_path(target)?;
        let (parent, filename) = target.rsplit_once('/').unwrap_or((".", target));
        let parent = Arc::new(
            read.directory
                .open_dir(parent)
                .map_err(|_| "File folder is unavailable.")?,
        );
        let (before, permissions) = snapshot(&parent, filename)?;
        if before != expected {
            return Err("File changed after this edit. Revert was not prepared.".into());
        }
        let preview = change_diff(&before, restored);
        if preview.len() > MAX_TOOL_BYTES {
            return Err("Revert diff exceeds the 16 KiB limit.".into());
        }
        Ok(Self {
            read,
            plan: Plan {
                request: ToolRequest {
                    call_id: String::new(),
                    name: "revert".into(),
                    target: target.into(),
                    query: None,
                    diff: Some(preview),
                },
                parent,
                filename: filename.into(),
                before,
                after: restored.into(),
                permissions,
            },
        })
    }
    pub fn diff(&self) -> &str {
        self.plan.request.diff.as_deref().unwrap()
    }
    pub fn apply(self, journal: Arc<dyn ChangeJournal>) -> Result<String, String> {
        EditTextFile::apply(
            self.read,
            self.plan,
            CancellationToken::new(),
            Some(journal),
        )
    }
}

#[cfg(test)]
fn diff(before: &str, after: &str) -> String {
    change_diff(before, after)
}

#[cfg(test)]
#[path = "edit_tests.rs"]
mod tests;
