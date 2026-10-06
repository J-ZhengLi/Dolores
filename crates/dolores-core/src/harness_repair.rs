//! Durable managed snapshots. A proposal is never evidence of execution or improvement.
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct RepairFile {
    pub path: String,
    pub source_id: String,
    pub candidate_id: String,
    pub before: String,
    pub after: String,
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct RepairWorkspace {
    pub id: String,
    pub session: String,
    pub revision: u32,
    pub bundle_id: String,
    pub build: String,
    pub artifact: String,
    pub status: String,
    pub files: Vec<RepairFile>,
}
pub fn valid_repair_id(id: &str) -> bool {
    id.len() == 36
        && id.bytes().enumerate().all(|(i, c)| {
            if [8, 13, 18, 23].contains(&i) {
                c == b'-'
            } else {
                c.is_ascii_hexdigit()
            }
        })
}
pub fn valid_source_id(id: &str) -> bool {
    id.strip_prefix("sha256:")
        .is_some_and(|s| s.len() == 64 && s.bytes().all(|c| c.is_ascii_hexdigit()))
}
impl RepairWorkspace {
    pub fn validate(&self) -> Result<(), String> {
        let mut paths = std::collections::BTreeSet::new();
        if !valid_repair_id(&self.id)
            || self.session.is_empty()
            || self.session.len() > 128
            || !valid_source_id(&self.bundle_id)
            || self.build.len() > 64
            || !self.artifact.starts_with("repair-")
            || self.artifact.len() > 80
            || !self
                .artifact
                .bytes()
                .all(|c| c.is_ascii_alphanumeric() || c == b'-')
            || !["prepared", "proposed"].contains(&self.status.as_str())
            || self.files.is_empty()
            || self.files.len() > 8
            || self
                .files
                .iter()
                .map(|f| f.before.len() + f.after.len())
                .sum::<usize>()
                > 2 * 1024 * 1024
            || self.files.iter().any(|f| {
                f.path.is_empty()
                    || f.path.len() > 256
                    || f.path.contains(['\\', ':'])
                    || f.path.chars().any(char::is_control)
                    || f.path.split('/').any(|p| matches!(p, "" | "." | ".."))
                    || !paths.insert(&f.path)
                    || !valid_source_id(&f.source_id)
                    || !valid_source_id(&f.candidate_id)
                    || f.before.len() > crate::MAX_FILE_SNAPSHOT_BYTES
                    || f.after.len() > crate::MAX_FILE_SNAPSHOT_BYTES
                    || f.before.contains('\0')
                    || f.after.contains('\0')
            })
        {
            return Err("Invalid repair snapshot or workspace limit. Retained work remains; inspect the repair before preparing a smaller proposal.".into());
        }
        Ok(())
    }
}
