//! Private host lifecycle evidence. Absolute checkout paths must not enter model context.
use serde::{Deserialize, Serialize};
use std::path::Path;

pub const WORKTREE_PAGE_SIZE: usize = 20;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum WorktreeState {
    Creating,
    Ready,
    Archiving,
    Archived,
    Restoring,
    Recovery,
}

impl WorktreeState {
    pub fn in_flight(self) -> bool {
        matches!(self, Self::Creating | Self::Archiving | Self::Restoring)
    }
}

/// Neither policy executes repository content on creation or restart.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum WorktreeSetup {
    Skip,
    Manual,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorktreeIdentity {
    /// Host filesystem identity (volume/file ID on Windows), not a path hash.
    pub filesystem: String,
    pub git_directory: String,
    pub git_identity: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ManagedWorktree {
    pub id: String,
    pub session: String,
    pub revision: u32,
    pub source_root: String,
    pub common_directory: String,
    pub common_identity: String,
    pub root: String,
    pub base_ref: String,
    pub base_commit: String,
    pub setup: WorktreeSetup,
    pub state: WorktreeState,
    pub identity: Option<WorktreeIdentity>,
    /// Retained commit must have an owned Git ref before removing the checkout.
    pub snapshot: Option<String>,
    pub recovery_from: Option<WorktreeState>,
    pub notice: String,
}

fn absolute(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 4096
        && !value.contains(['\0', '\r', '\n'])
        && Path::new(value).is_absolute()
        && !Path::new(value)
            .components()
            .any(|part| matches!(part, std::path::Component::ParentDir))
}

fn oid(value: &str) -> bool {
    matches!(value.len(), 40 | 64)
        && value
            .bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
}

impl ManagedWorktree {
    pub fn validate(&self) -> Result<(), String> {
        if uuid::Uuid::parse_str(&self.id).is_err()
            || uuid::Uuid::parse_str(&self.session).is_err()
            || !absolute(&self.source_root)
            || !absolute(&self.common_directory)
            || self.common_identity.is_empty()
            || self.common_identity.len() > 256
            || self.common_identity.contains(['\0', '\r', '\n'])
            || !absolute(&self.root)
            || self.source_root == self.root
            || self.base_ref.is_empty()
            || self.base_ref.len() > 256
            || self.base_ref.starts_with('-')
            || self.base_ref.contains(['\0', '\r', '\n'])
            || !oid(&self.base_commit)
            || self.snapshot.as_ref().is_some_and(|s| !oid(s))
            || self.notice.len() > 4096
            || self.notice.contains('\0')
        {
            return Err("Invalid managed worktree identity or base. Inspect the saved record before continuing.".into());
        }
        if let Some(identity) = &self.identity {
            if identity.filesystem.is_empty()
                || identity.filesystem.len() > 256
                || identity.filesystem.contains(['\0', '\r', '\n'])
                || !(absolute(&identity.git_directory)
                    || identity.git_directory.is_empty()
                        && (self.state == WorktreeState::Creating
                            || self.recovery_from == Some(WorktreeState::Creating)))
                || identity.git_identity.len() > 256
                || identity.git_identity.contains(['\0', '\r', '\n'])
                || identity.git_directory.is_empty() != identity.git_identity.is_empty()
            {
                return Err("Managed checkout identity is unavailable. Inspect its folder and Git registration.".into());
            }
        }
        if (self.state == WorktreeState::Recovery) != self.recovery_from.is_some()
            || self.recovery_from == Some(WorktreeState::Recovery)
            || matches!(
                self.state,
                WorktreeState::Ready
                    | WorktreeState::Archiving
                    | WorktreeState::Archived
                    | WorktreeState::Restoring
            ) && self.identity.is_none()
            || matches!(
                self.state,
                WorktreeState::Archived | WorktreeState::Restoring
            ) && self.snapshot.is_none()
        {
            return Err("Incomplete worktree lifecycle evidence. Keep the checkout and inspect recovery details.".into());
        }
        Ok(())
    }

    pub fn validate_creation(&self) -> Result<(), String> {
        self.validate()?;
        if self.revision != 0
            || self.state != WorktreeState::Creating
            || self.identity.is_some()
            || self.snapshot.is_some()
        {
            return Err("New worktrees must start with a saved creation intent.".into());
        }
        Ok(())
    }

    pub fn validate_update(&self, old: &Self) -> Result<(), String> {
        self.validate()?;
        if old.revision.checked_add(1) != Some(self.revision)
            || self.id != old.id
            || self.session != old.session
            || self.source_root != old.source_root
            || self.common_directory != old.common_directory
            || self.common_identity != old.common_identity
            || self.root != old.root
            || self.base_ref != old.base_ref
            || self.base_commit != old.base_commit
            || self.setup != old.setup
        {
            return Err(
                "Worktree revision or ownership changed. Refresh; no operation was repeated."
                    .into(),
            );
        }
        use WorktreeState::*;
        let allowed = matches!(
            (old.state, self.state),
            (Creating, Creating | Ready | Recovery)
                | (Ready, Archiving | Recovery)
                | (Archiving, Archiving | Archived | Ready | Recovery)
                | (Archived, Restoring | Recovery)
                | (Restoring, Ready | Archived | Recovery)
        ) || old.state == Recovery
            && (self.state == Recovery
                || matches!(
                    (old.recovery_from, self.state),
                    (Some(Creating), Creating | Ready)
                        | (Some(Ready), Ready | Archiving)
                        | (Some(Archiving), Ready | Archiving | Archived)
                        | (Some(Archived), Archived | Restoring)
                        | (Some(Restoring), Ready | Archived | Restoring)
                ));
        if !allowed
            || self.state == Recovery
                && self.recovery_from != Some(old.recovery_from.unwrap_or(old.state))
            || self.snapshot != old.snapshot
                && !(self.state == Archiving && self.snapshot.is_some())
            || self.identity != old.identity
                && !((old.state == Creating || old.recovery_from == Some(Creating))
                    && matches!(self.state, Creating | Ready)
                    && self.identity.as_ref().is_some_and(|new| {
                        old.identity
                            .as_ref()
                            .is_none_or(|old| old.filesystem == new.filesystem)
                    })
                    || (old.state == Restoring || old.recovery_from == Some(Restoring))
                        && self.state == Ready
                        && self.identity.is_some())
        {
            return Err("Worktree transition would discard recovery evidence. Inspect the retained checkout and snapshot.".into());
        }
        Ok(())
    }

    /// Startup records uncertainty only. Filesystem inspection and retry are explicit host actions.
    pub fn interrupted(&self) -> Result<Self, String> {
        self.validate()?;
        if !self.state.in_flight() {
            return Ok(self.clone());
        }
        let mut recovered = self.clone();
        recovered.revision = self
            .revision
            .checked_add(1)
            .ok_or("Worktree revision exhausted.")?;
        recovered.state = WorktreeState::Recovery;
        recovered.recovery_from = Some(self.state);
        recovered.notice = "The worktree operation was interrupted. Inspect the folder, Git registration and retained snapshot before explicitly continuing. No Git or setup operation was repeated.".into();
        recovered.validate_update(self)?;
        Ok(recovered)
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorktreePage {
    pub items: Vec<ManagedWorktree>,
    pub next_cursor: Option<i64>,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn record() -> ManagedWorktree {
        let base = std::env::temp_dir();
        ManagedWorktree {
            id: uuid::Uuid::new_v4().to_string(),
            session: uuid::Uuid::new_v4().to_string(),
            revision: 0,
            source_root: base.join("source").to_string_lossy().into_owned(),
            common_directory: base.join("source/.git").to_string_lossy().into_owned(),
            common_identity: "volume:repository".into(),
            root: base.join("owned").to_string_lossy().into_owned(),
            base_ref: "main".into(),
            base_commit: "a".repeat(40),
            setup: WorktreeSetup::Manual,
            state: WorktreeState::Creating,
            identity: None,
            snapshot: None,
            recovery_from: None,
            notice: String::new(),
        }
    }

    #[test]
    fn worktree_interruption_retains_identity_and_requires_explicit_recovery() {
        let mut old = record();
        old.validate_creation().unwrap();
        let interrupted = old.interrupted().unwrap();
        assert_eq!(interrupted.state, WorktreeState::Recovery);
        assert_eq!(interrupted.recovery_from, Some(WorktreeState::Creating));
        assert_eq!(interrupted.interrupted().unwrap(), interrupted);
        old = interrupted;
        let mut ready = old.clone();
        ready.revision += 1;
        ready.state = WorktreeState::Ready;
        ready.recovery_from = None;
        assert!(ready.validate_update(&old).is_err());
        ready.identity = Some(WorktreeIdentity {
            filesystem: "volume:file".into(),
            git_directory: format!("{}/worktrees/owned", ready.common_directory),
            git_identity: "volume:registration".into(),
        });
        ready.validate_update(&old).unwrap();
        let mut archive = ready.clone();
        archive.revision += 1;
        archive.state = WorktreeState::Archiving;
        archive.validate_update(&ready).unwrap();
        let mut saved = archive.clone();
        saved.revision += 1;
        saved.snapshot = Some("b".repeat(40));
        saved.validate_update(&archive).unwrap();
        let recovered = saved.interrupted().unwrap();
        assert_eq!(recovered.snapshot, saved.snapshot);
        assert_eq!(recovered.identity, saved.identity);
        let mut wrong_phase = recovered.clone();
        wrong_phase.revision += 1;
        wrong_phase.state = WorktreeState::Creating;
        wrong_phase.recovery_from = None;
        assert!(wrong_phase.validate_update(&recovered).is_err());
        let mut replaced_root = recovered.clone();
        replaced_root.revision += 1;
        replaced_root.state = WorktreeState::Ready;
        replaced_root.recovery_from = None;
        replaced_root.identity.as_mut().unwrap().filesystem = "different:file".into();
        assert!(replaced_root.validate_update(&recovered).is_err());
        let mut invalid = recovered.clone();
        invalid.revision += 1;
        invalid.snapshot = None;
        assert!(invalid.validate_update(&recovered).is_err());
    }

    #[test]
    fn worktree_rejects_stale_revision_retargeting_and_missing_archive_snapshot() {
        let old = record();
        let mut next = old.interrupted().unwrap();
        next.root = old.source_root.clone();
        assert!(next.validate_update(&old).is_err());
        next = old.interrupted().unwrap();
        next.base_commit = "b".repeat(40);
        assert!(next.validate_update(&old).is_err());
        next = old.interrupted().unwrap();
        next.revision = 0;
        assert!(next.validate_update(&old).is_err());
        next.state = WorktreeState::Archived;
        next.recovery_from = None;
        assert!(next.validate().is_err());
    }
}
