use super::{
    edit::{snapshot, EditTextFile},
    ReadTextFile,
};
use cap_std::fs::{Dir, Permissions};
use dolores_core::{ChangeJournal, MAX_TOOL_BYTES};
use serde_json::json;
use std::sync::Arc;

const CHANGED: &str = "File changed since preview. No file was removed.";

/// Local-only reversal of a recorded creation, never a model deletion tool.
pub struct RemoveCreatedPlan {
    read: ReadTextFile,
    parent: Arc<Dir>,
    filename: String,
    target: String,
    expected: String,
    permissions: Permissions,
    diff: String,
}
impl RemoveCreatedPlan {
    pub fn preview(root: &std::path::Path, target: &str, expected: &str) -> Result<Self, String> {
        if expected.len() > MAX_TOOL_BYTES || expected.contains('\0') {
            return Err("Invalid removal snapshot.".into());
        }
        let read = ReadTextFile::new(root)?;
        EditTextFile::new(read.clone()).direct_path(target)?;
        let (parent, filename) = super::create::direct_parent(&read, target)?;
        let (current, permissions) = snapshot(&parent, &filename)?;
        if current != expected {
            return Err("File changed after creation. Removal was not prepared.".into());
        }
        let diff = super::file_change_diff(Some(expected), None);
        if diff.len() > MAX_TOOL_BYTES {
            return Err("Removal diff exceeds the 16 KiB limit.".into());
        }
        Ok(Self {
            read,
            parent,
            filename,
            target: target.into(),
            expected: expected.into(),
            permissions,
            diff,
        })
    }
    pub fn diff(&self) -> &str {
        &self.diff
    }
    fn check(&self) -> Result<(), String> {
        EditTextFile::new(self.read.clone())
            .direct_path(&self.target)
            .map_err(|_| CHANGED)?;
        let (current, permissions) = snapshot(&self.parent, &self.filename).map_err(|_| CHANGED)?;
        if current != self.expected || permissions != self.permissions {
            return Err(CHANGED.into());
        }
        Ok(())
    }
    pub fn apply(self, journal: Arc<dyn ChangeJournal>) -> Result<String, String> {
        self.check()?;
        let id = journal.begin_file_change(&self.target, Some(&self.expected), None)?;
        let publish: Result<(), String> = (|| {
            self.check()?;
            self.parent
                .remove_file(&self.filename)
                .map_err(|_| "Could not remove the created file.".into())
        })();
        let saved = journal.finish(id, publish.is_ok()).is_ok();
        publish?;
        Ok(json!({"applied":true,"removed":true,"bytesBefore":self.expected.len(),"bytesAfter":0,"changeId":id,"journalStatus":if saved {"applied"} else {"pending"}}).to_string())
    }
}
