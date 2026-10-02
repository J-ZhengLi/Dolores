use serde::Serialize;

/// Local snapshots are never included in conversation exports or model context.
pub struct ChangeDraft {
    pub root: String,
    pub session: String,
    pub target: String,
    pub before: String,
    pub after: String,
    pub reverts: Option<i64>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FileChange {
    pub id: i64,
    pub created_at: i64,
    pub target: String,
    pub status: String,
    pub reverts: Option<i64>,
    pub bytes_before: usize,
    pub bytes_after: usize,
}

pub struct ChangeSnapshot {
    pub change: FileChange,
    pub root: String,
    pub before: String,
    pub after: String,
}

/// Host-bound journal, independent of model arguments and conversation saving.
pub trait ChangeJournal: Send + Sync {
    fn begin(&self, target: &str, before: &str, after: &str) -> Result<i64, String>;
    fn finish(&self, id: i64, applied: bool) -> Result<(), String>;
}

pub struct WorkspaceJournal {
    pub store: std::sync::Arc<dyn crate::SessionStore>,
    pub root: String,
    pub session: String,
    pub reverts: Option<i64>,
}
impl ChangeJournal for WorkspaceJournal {
    fn begin(&self, target: &str, before: &str, after: &str) -> Result<i64, String> {
        self.store.begin_change(&ChangeDraft {
            root: self.root.clone(),
            session: self.session.clone(),
            target: target.into(),
            before: before.into(),
            after: after.into(),
            reverts: self.reverts,
        })
    }
    fn finish(&self, id: i64, applied: bool) -> Result<(), String> {
        self.store.finish_change(id, applied)
    }
}
