use super::{git_process, worktree_identity::Pin};
use dolores_core::{worktrees::*, SessionStore};
use sha2::{Digest, Sha256};
use std::{
    path::{Path, PathBuf},
    time::Instant,
};
use tokio_util::sync::CancellationToken;

pub(super) struct CreationReview {
    pub token: String,
    pub source_session: String,
    pub record: ManagedWorktree,
    pub roots: Vec<PathBuf>,
    pub created: Instant,
    source_identity: String,
    parent_identity: String,
    configuration: String,
    registrations: String,
}

pub(super) struct Git {
    executable: PathBuf,
    root: PathBuf,
    options: Vec<String>,
}

impl Git {
    pub(super) fn new(root: &Path) -> Result<Self, String> {
        Ok(Self {
            executable: git_process::executable(root)?,
            root: root.into(),
            options: vec![
                "-c".into(),
                "core.fsmonitor=false".into(),
                "-c".into(),
                "submodule.recurse=false".into(),
            ],
        })
    }
    pub(super) fn run(
        &self,
        args: &[&str],
        cancel: &CancellationToken,
    ) -> Result<git_process::Output, String> {
        let mut argv = self.options.clone();
        argv.extend(args.iter().map(|v| v.to_string()));
        git_process::run(&self.executable, &self.root, &argv, cancel, false)
    }
    pub(super) fn with_index(
        &self,
        args: &[&str],
        index: &Path,
        cancel: &CancellationToken,
    ) -> Result<git_process::Output, String> {
        let mut argv = self.options.clone();
        argv.extend(args.iter().map(|v| v.to_string()));
        git_process::run_with_index(
            &self.executable,
            &self.root,
            &argv,
            cancel,
            false,
            Some(index),
        )
    }
    pub(super) fn command(
        &self,
        args: &[&str],
        cancel: &CancellationToken,
    ) -> Result<Vec<u8>, String> {
        let output = self.run(args, cancel)?;
        if output.code != 0 {
            let detail = String::from_utf8_lossy(&output.stderr);
            // Local UI evidence only; never include these absolute paths in model input.
            return Err(format!("Git worktree operation failed ({}): {}. Inspect the retained folder before retrying.",output.code,detail.chars().take(1800).collect::<String>()));
        }
        Ok(output.stdout)
    }
    pub(super) fn line(&self, args: &[&str], cancel: &CancellationToken) -> Result<String, String> {
        String::from_utf8(self.command(args, cancel)?)
            .map(|s| s.trim().to_owned())
            .map_err(|_| {
                "Git returned a non-Unicode path or revision. Use external Git to inspect it."
                    .into()
            })
    }
    pub(super) fn configuration(&self, cancel: &CancellationToken) -> Result<String, String> {
        Ok(format!(
            "{:x}",
            Sha256::digest(self.command(&["config", "--null", "--list"], cancel)?)
        ))
    }
    pub(super) fn materialization_options(
        &mut self,
        parent: &Path,
        cancel: &CancellationToken,
    ) -> Result<(), String> {
        // Hooks and filters are executable repository configuration, not setup approval.
        let hooks = parent.join(format!("disabled-hooks-{}", uuid::Uuid::new_v4()));
        if hooks.exists() {
            return Err("Hook exclusion path already exists. Review again.".into());
        }
        self.options.extend([
            "-c".into(),
            format!(
                "core.hooksPath={}",
                hooks.to_str().ok_or("Invalid hook path.")?
            ),
            "-c".into(),
            "core.sparseCheckout=false".into(),
            "-c".into(),
            "core.sparseCheckoutCone=false".into(),
        ]);
        let filters = self.run(
            &[
                "config",
                "--null",
                "--name-only",
                "--get-regexp",
                r"^filter\..*\.(clean|smudge|process|required)$",
            ],
            cancel,
        )?;
        if filters.code != 0 && filters.code != 1 {
            return Err("Could not inspect Git filters. No checkout was created.".into());
        }
        let keys = std::str::from_utf8(&filters.stdout)
            .map_err(|_| "Unsupported Git filter configuration.")?
            .split('\0')
            .filter(|k| !k.is_empty())
            .collect::<Vec<_>>();
        if keys.len() > 64 {
            return Err("This repository has more than 64 Git filter settings. Inspect setup with external Git before creating a managed checkout.".into());
        }
        for key in keys {
            if key.len() > 1024 || key.contains(['\n', '\r']) {
                return Err(
                    "Unsupported Git filter name. Inspect repository configuration.".into(),
                );
            }
            let value = if key.to_ascii_lowercase().ends_with(".required") {
                "false"
            } else {
                ""
            };
            self.options.extend(["-c".into(), format!("{key}={value}")]);
        }
        Ok(())
    }
}

pub(super) fn review(
    source_session: &str,
    source: &Path,
    parent: &Path,
    base: &str,
    setup: WorktreeSetup,
    cancel: &CancellationToken,
) -> Result<CreationReview, String> {
    if base.is_empty()
        || base.len() > 256
        || base.starts_with('-')
        || base.contains(['\0', '\r', '\n'])
    {
        return Err("Choose an explicit base branch, tag or commit within 256 bytes.".into());
    }
    let source_pin = Pin::open(source)?;
    let git = Git::new(&source_pin.path)?;
    let top = Pin::open(Path::new(
        &git.line(&["rev-parse", "--show-toplevel"], cancel)?,
    ))?;
    if top.path != source_pin.path {
        return Err("Open the repository root before creating a managed worktree.".into());
    }
    let common = Pin::open(Path::new(&git.line(
        &["rev-parse", "--path-format=absolute", "--git-common-dir"],
        cancel,
    )?))?;
    let base_commit = git.line(
        &[
            "rev-parse",
            "--verify",
            "--end-of-options",
            &format!("{base}^{{commit}}"),
        ],
        cancel,
    )?;
    // This host-owned parent is created only in the app data location, never inside the repository.
    let ancestor = parent
        .parent()
        .ok_or("Managed worktree storage is unavailable.")?;
    let ancestor_pin = Pin::open(ancestor)?;
    if ancestor_pin.path.starts_with(&top.path) || ancestor_pin.path.starts_with(&common.path) {
        return Err("Managed worktree storage must be outside the source repository. Choose a separate Dolores data location.".into());
    }
    if !parent.exists() {
        std::fs::create_dir(parent).map_err(|_| {
            "Could not create managed worktree storage. Check disk space and access, then retry."
        })?;
    }
    let parent_pin = Pin::open(parent)?;
    let id = uuid::Uuid::new_v4().to_string();
    let root = parent_pin.path.join(&id);
    let root_text = root
        .to_str()
        .ok_or("Managed worktree path must be Unicode.")?;
    #[cfg(windows)]
    let root_text = root_text.strip_prefix(r"\\?\").unwrap_or(root_text);
    let record = ManagedWorktree {
        id: id.clone(),
        session: id,
        revision: 0,
        source_root: top.text()?,
        common_directory: common.text()?,
        common_identity: common.identity.clone(),
        root: root_text.into(),
        base_ref: base.into(),
        base_commit,
        setup,
        state: WorktreeState::Creating,
        identity: None,
        snapshot: None,
        recovery_from: None,
        notice: String::new(),
    };
    record.validate_creation()?;
    let raw = git.command(&["worktree", "list", "--porcelain", "-z"], cancel)?;
    let mut roots = vec![];
    for field in std::str::from_utf8(&raw)
        .map_err(|_| "Worktree registrations contain a non-Unicode path.")?
        .split('\0')
    {
        if let Some(root) = field.strip_prefix("worktree ") {
            roots.push(PathBuf::from(root));
        }
    }
    if roots.is_empty() || roots.len() > 128 {
        return Err("Cannot review more than 128 registered checkouts. Inspect the repository with external Git.".into());
    }
    roots.push(PathBuf::from(&record.root));
    roots.push(common.path.clone());
    Ok(CreationReview {
        token: uuid::Uuid::new_v4().to_string(),
        source_session: source_session.into(),
        record,
        roots,
        created: Instant::now(),
        source_identity: top.identity,
        parent_identity: parent_pin.identity,
        configuration: git.configuration(cancel)?,
        registrations: format!("{:x}", Sha256::digest(&raw)),
    })
}

pub(super) fn create(
    store: &dyn SessionStore,
    review: &CreationReview,
    cancel: &CancellationToken,
) -> Result<ManagedWorktree, String> {
    let source = Pin::matches(
        Path::new(&review.record.source_root),
        &review.source_identity,
    )?;
    let common = Pin::matches(
        Path::new(&review.record.common_directory),
        &review.record.common_identity,
    )?;
    let parent = Path::new(&review.record.root)
        .parent()
        .ok_or("Invalid worktree target.")?;
    let parent_pin = Pin::matches(parent, &review.parent_identity)?;
    if Path::new(&review.record.root)
        .file_name()
        .and_then(|p| p.to_str())
        != Some(review.record.id.as_str())
    {
        return Err("Worktree target changed after review.".into());
    }
    let mut git = Git::new(&source.path)?;
    if git.configuration(cancel)? != review.configuration {
        return Err("Repository configuration changed after review. Review creation again; no checkout was created.".into());
    }
    let actual_common = Pin::open(Path::new(&git.line(
        &["rev-parse", "--path-format=absolute", "--git-common-dir"],
        cancel,
    )?))?;
    if actual_common.identity != common.identity {
        return Err("Repository changed after review. No checkout was created.".into());
    }
    if format!(
        "{:x}",
        Sha256::digest(git.command(&["worktree", "list", "--porcelain", "-z"], cancel)?)
    ) != review.registrations
    {
        return Err("Registered worktrees changed after review. Review creation again; no checkout was created.".into());
    }
    git.materialization_options(&parent_pin.path, cancel)?;
    if cancel.is_cancelled() {
        return Err(
            "Worktree creation stopped before saving an intent. No checkout was created.".into(),
        );
    }
    store.create_worktree(&review.record)?;
    let result = create_saved(store, &git, review.record.clone(), cancel);
    match result {
        Ok(record) => Ok(record),
        Err(error) => {
            let saved = store.worktree(&review.record.session)?
                .ok_or("Creation evidence is unavailable. Inspect the requested folder manually before retrying.")?;
            let mut recovery = saved.interrupted()?;
            recovery.notice = format!("{error} Retained target: {}. Inspect it before retrying; the original checkout was not reset or removed.",saved.root);
            if recovery.notice.len() > 4096 {
                recovery.notice = format!("Creation failed. Inspect retained target {} and its Git registration before retrying.",saved.root);
            }
            if let Err(save_error) = store.save_worktree(&recovery) {
                return Err(format!("{error} Recovery could not be saved: {save_error} The creation intent and folder {} remain. Inspect before retrying.",saved.root));
            }
            Err(recovery.notice)
        }
    }
}

fn create_saved(
    store: &dyn SessionStore,
    git: &Git,
    mut record: ManagedWorktree,
    cancel: &CancellationToken,
) -> Result<ManagedWorktree, String> {
    std::fs::create_dir(&record.root).map_err(|_|"Could not reserve a new worktree folder. Check disk space and inspect any existing target; it was not overwritten.")?;
    let pin = Pin::open(Path::new(&record.root))?;
    record.identity = Some(WorktreeIdentity {
        filesystem: pin.identity.clone(),
        git_directory: String::new(),
        git_identity: String::new(),
    });
    record.revision += 1;
    store.save_worktree(&record)?;
    git.command(
        &[
            "worktree",
            "add",
            "--detach",
            "--lock",
            "--reason",
            "Dolores managed checkout",
            "--",
            &record.root,
            &record.base_commit,
        ],
        cancel,
    )?;
    let mut checkout = Git::new(&pin.path)?;
    checkout.options = git.options.clone();
    if checkout.line(&["rev-parse", "--verify", "HEAD"], cancel)? != record.base_commit {
        return Err(
            "Created checkout does not match the reviewed commit. Inspect it before running tasks."
                .into(),
        );
    }
    let git_dir = Pin::open(Path::new(
        &checkout.line(&["rev-parse", "--absolute-git-dir"], cancel)?,
    ))?;
    let common = Pin::open(Path::new(&record.common_directory))?;
    if !git_dir.path.starts_with(common.path.join("worktrees")) {
        return Err("Created Git registration is outside the reviewed repository. Inspect it before using the checkout.".into());
    }
    record.identity.as_mut().unwrap().git_directory = git_dir.text()?;
    record.identity.as_mut().unwrap().git_identity = git_dir.identity;
    record.revision += 1;
    record.state = WorktreeState::Ready;
    record.notice = match record.setup {
        WorktreeSetup::Skip => "Checkout ready. Automatic setup, Git hooks and filters were skipped. Files using filters may need manual setup.".into(),
        WorktreeSetup::Manual => "Checkout ready for manual setup. Run and review setup commands in this chat; failure keeps the checkout and command receipt.".into(),
    };
    store.save_worktree(&record)?;
    Ok(record)
}

pub(super) struct ReadyPins {
    _directories: Vec<Pin>,
    _pointers: Vec<std::fs::File>,
}

pub(super) fn validate_ready(record: &ManagedWorktree) -> Result<ReadyPins, String> {
    record.validate()?;
    if record.state != WorktreeState::Ready {
        return Err("This managed worktree is not ready. Open Worktree details and inspect or restore it before starting a task; your draft remains.".into());
    }
    let identity = record
        .identity
        .as_ref()
        .ok_or("Worktree identity is unavailable.")?;
    let root = Pin::matches(Path::new(&record.root), &identity.filesystem)?;
    let common = Pin::matches(Path::new(&record.common_directory), &record.common_identity)?;
    // Pin both namespace and pointer contents for the lifetime of an admitted run.
    let (root_pointer, pointer) = read_pointer(&root.path.join(".git"))?;
    let path = PathBuf::from(
        pointer
            .trim()
            .strip_prefix("gitdir: ")
            .ok_or("Worktree Git pointer is malformed.")?,
    );
    let target = if path.is_absolute() {
        path
    } else {
        root.path.join(path)
    };
    let git_dir = Pin::matches(&target, &identity.git_identity)?;
    let saved = Pin::matches(Path::new(&identity.git_directory), &identity.git_identity)?;
    if git_dir.path != saved.path {
        return Err("Worktree Git registration changed. Inspect recovery details.".into());
    }
    let (common_pointer, common_text) = read_pointer(&git_dir.path.join("commondir"))?;
    let actual_common = pointer_path(&common_text, &git_dir.path)?;
    if actual_common != common.path {
        return Err("Worktree common Git directory changed. Inspect recovery details.".into());
    }
    let (back_file, back_text) = read_pointer(&git_dir.path.join("gitdir"))?;
    let back_pointer = pointer_path(&back_text, &git_dir.path)?;
    if back_pointer != root.path.join(".git") {
        return Err("Git registration points to another checkout. Keep both folders and inspect recovery details.".into());
    }
    Ok(ReadyPins {
        _directories: vec![root, common, git_dir, saved],
        _pointers: vec![root_pointer, common_pointer, back_file],
    })
}

fn read_pointer(path: &Path) -> Result<(std::fs::File, String), String> {
    use std::io::Read;
    super::worktree_identity::no_alias(path)?;
    let mut options = std::fs::OpenOptions::new();
    options.read(true);
    #[cfg(windows)]
    {
        use std::os::windows::fs::OpenOptionsExt;
        use windows_sys::Win32::Storage::FileSystem::*;
        options
            .share_mode(FILE_SHARE_READ)
            .custom_flags(FILE_FLAG_OPEN_REPARSE_POINT);
    }
    let file = options
        .open(path)
        .map_err(|_| "Git registration is missing, changed or in use. Inspect recovery details.")?;
    if !file
        .metadata()
        .map_err(|_| "Git registration is unavailable.")?
        .is_file()
    {
        return Err("Git registration must be a regular file.".into());
    }
    let mut bytes = vec![];
    (&file)
        .take(8193)
        .read_to_end(&mut bytes)
        .map_err(|_| "Git registration is unreadable.")?;
    if bytes.len() > 8192 {
        return Err("Git registration exceeds its 8 KiB inspection bound.".into());
    }
    let text =
        String::from_utf8(bytes).map_err(|_| "Git registration contains a non-Unicode path.")?;
    Ok((file, text))
}

fn pointer_path(text: &str, base: &Path) -> Result<PathBuf, String> {
    let text = text.trim();
    if text.is_empty() || text.contains(['\0', '\r', '\n']) {
        return Err("Git registration path is malformed.".into());
    }
    let path = PathBuf::from(text);
    // Git's own commondir normally contains ../..; resolve it, then inspect every
    // component of the actual target and compare with the pinned common directory.
    let resolved = if path.is_absolute() {
        path
    } else {
        base.join(path)
    };
    let resolved = resolved
        .canonicalize()
        .map_err(|_| "Git registration target is missing. Inspect recovery details.")?;
    super::worktree_identity::no_alias(&resolved)?;
    Ok(resolved)
}

pub(super) fn reconcile_creation(
    store: &dyn SessionStore,
    session: &str,
    revision: u32,
    cancel: &CancellationToken,
) -> Result<ManagedWorktree, String> {
    let mut record = store
        .worktree(session)?
        .ok_or("Managed worktree evidence is unavailable.")?;
    if record.revision != revision
        || record.state != WorktreeState::Recovery
        || record.recovery_from != Some(WorktreeState::Creating)
    {
        return Err(
            "Creation recovery changed. Refresh its details before inspecting again.".into(),
        );
    }
    let identity=record.identity.as_ref().ok_or("No owned folder identity was saved. Inspect the target manually; it will not be adopted or removed automatically.")?;
    let root = Pin::matches(Path::new(&record.root), &identity.filesystem)?;
    let common = Pin::matches(Path::new(&record.common_directory), &record.common_identity)?;
    let git = Git::new(&root.path)?;
    if git.line(&["rev-parse", "--verify", "HEAD"], cancel)? != record.base_commit {
        return Err("Recovered checkout no longer has the reviewed base commit. Keep it and inspect with external Git; no files were reset.".into());
    }
    let metadata = Pin::open(Path::new(
        &git.line(&["rev-parse", "--absolute-git-dir"], cancel)?,
    ))?;
    if !metadata.path.starts_with(common.path.join("worktrees")) {
        return Err("Recovered checkout is not registered in the reviewed repository. It was left untouched.".into());
    }
    record.identity = Some(WorktreeIdentity {
        filesystem: root.identity,
        git_directory: metadata.text()?,
        git_identity: metadata.identity,
    });
    record.state = WorktreeState::Ready;
    record.recovery_from = None;
    record.revision = record
        .revision
        .checked_add(1)
        .ok_or("Worktree revision exhausted.")?;
    validate_ready(&record)?;
    if cancel.is_cancelled() {
        return Err("Inspection stopped. Retained work is unchanged.".into());
    }
    record.notice="Existing checkout inspected and reattached. No checkout or setup was repeated; inspect any existing file changes before continuing.".into();
    store.save_worktree(&record)?;
    Ok(record)
}
