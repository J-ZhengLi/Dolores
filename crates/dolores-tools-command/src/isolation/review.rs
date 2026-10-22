use super::*;
use tokio_util::sync::CancellationToken;

/// Immutable grant identities captured before asking the human. Folder contents
/// can evolve, but a replacement folder never inherits an earlier approval.
#[derive(Clone, Debug)]
pub struct Review {
    pub(super) grants: Vec<Grant>,
}
impl Review {
    pub fn workspace(&self) -> &Path {
        &self.grants[0].path
    }
    pub fn readonly(&self) -> Vec<PathBuf> {
        self.grants.iter().skip(1).map(|g| g.path.clone()).collect()
    }
    fn pins(&self, cancel: &CancellationToken) -> Result<Vec<acl::Root>, String> {
        let mut pins = vec![];
        for grant in &self.grants {
            stopped(cancel)?;
            let root = acl::root(&grant.path)?;
            if root.identity != grant.identity {
                return Err("Isolation grant root changed after review. Review the current folder; no command was launched.".into());
            }
            acl::check_tree_cancel(&root, cancel)?;
            pins.push(root);
        }
        Ok(pins)
    }
}
impl Host {
    pub fn review(
        &self,
        workspace: &Path,
        readonly: &[PathBuf],
        cancel: &CancellationToken,
    ) -> Result<Review, String> {
        if readonly.len() > 8 {
            return Err(
                "Isolation supports at most eight explicit read-only roots. Nothing was launched."
                    .into(),
            );
        }
        let mut grants: Vec<Grant> = vec![];
        for (i, path) in std::iter::once(workspace)
            .chain(readonly.iter().map(PathBuf::as_path))
            .enumerate()
        {
            stopped(cancel)?;
            let root = acl::root(path)?;
            let label = root.path.to_string_lossy();
            if label.len() > 1024 || label.chars().any(char::is_control) {
                return Err("Isolation root exceeds the review display limit or contains control characters.".into());
            }
            if self.root.starts_with(&root.path) || root.path.starts_with(&self.root) {
                return Err("Isolation grants cannot include private isolation state.".into());
            }
            if grants
                .iter()
                .any(|g| g.path.starts_with(&root.path) || root.path.starts_with(&g.path))
            {
                return Err(
                    "Isolation grants overlap. Review disjoint writable/read-only roots.".into(),
                );
            }
            acl::check_tree_cancel(&root, cancel)?;
            grants.push(Grant {
                path: root.path,
                identity: root.identity,
                write: i == 0,
            });
        }
        Ok(Review { grants })
    }
    pub fn prepare_reviewed(
        self: &Arc<Self>,
        executable: &Path,
        review: &Review,
        cancel: &CancellationToken,
    ) -> Result<Lease, String> {
        let pins = review.pins(cancel)?;
        let readonly = review.readonly();
        let result = self.prepare_with_cancel(executable, &pins[0].path, &readonly, None, cancel);
        // Pins remain live until the new lease owns its own identical roots.
        drop(pins);
        result
    }
    /// Read-only status; listing/refreshing never retries cleanup or execution.
    pub fn pending_cleanup(&self) -> Result<usize, String> {
        let active = self
            .active
            .lock()
            .map_err(|_| "Isolation owner state unavailable.")?;
        let mut pending = 0;
        for entry in
            std::fs::read_dir(&self.root).map_err(|_| "Isolation cleanup state unavailable.")?
        {
            let entry = entry.map_err(|_| "Isolation cleanup entry unavailable.")?;
            let name = entry.file_name().to_string_lossy().into_owned();
            if name != "owner.lock" && !active.contains(&name) {
                pending += 1;
            }
            if pending > 64 {
                break;
            }
        }
        Ok(pending)
    }
}
pub(super) fn stopped(cancel: &CancellationToken) -> Result<(), String> {
    if cancel.is_cancelled() {
        Err("Isolation setup stopped. No child was launched.".into())
    } else {
        Ok(())
    }
}
