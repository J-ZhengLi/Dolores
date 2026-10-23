use super::{worktree_git, Engine};
use dolores_core::worktrees::*;
use serde::Deserialize;
use serde_json::{json, Value};
use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};
use tokio_util::sync::CancellationToken;

#[derive(Deserialize)]
#[serde(tag = "action", rename_all = "camelCase", deny_unknown_fields)]
pub(crate) enum Request {
    List {
        cursor: Option<i64>,
    },
    Inspect,
    ReconcileCreation {
        revision: u32,
    },
    ReviewCreate {
        base: String,
        setup: WorktreeSetup,
    },
    Create {
        token: String,
    },
    DiscardReview {
        token: String,
    },
    Poll {
        job: String,
    },
    Cancel {
        job: String,
    },
    CompareTransfer,
    ReviewTransfer {
        destination: String,
        paths: Vec<String>,
    },
    ApplyTransfer {
        token: String,
    },
    ListTransfers,
    ReviewArchive,
    Archive {
        token: String,
    },
    ReviewRestore,
    Restore {
        token: String,
    },
    InspectLifecycle {
        revision: u32,
    },
}

struct Job {
    session: String,
    cancel: CancellationToken,
    result: Option<Result<Value, String>>,
}

#[derive(Default)]
pub(super) struct Registry {
    jobs: BTreeMap<String, Job>,
    reviews: BTreeMap<String, worktree_git::CreationReview>,
    transfers: BTreeMap<String, super::worktree_transfer::Review>,
    lifecycle: BTreeMap<String, super::worktree_archive::Review>,
    closed: bool,
}

impl Engine {
    pub(super) fn worktree_call(
        &self,
        session: &str,
        request: Request,
        active: &super::run_journal::RunCoordinator,
    ) -> Result<Value, String> {
        if let Request::List { cursor } = request {
            return Ok(json!(self.store.worktrees(cursor)?));
        }
        if matches!(request, Request::Inspect) {
            return Ok(json!(record_for_chat(self.store.as_ref(), session)?));
        }
        if matches!(request, Request::ListTransfers) {
            return super::worktree_transfer::list(&self.transfer_directory()?, session);
        }
        let mut state = self
            .worktrees
            .lock()
            .map_err(|_| "Worktree operations are unavailable.")?;
        match request {
            Request::Poll { job } => {
                let owned = state.jobs.get_mut(&job).filter(|j|j.session==session).ok_or("Worktree job belongs to another chat or has expired. Inspect saved worktree details.")?;
                if let Some(result) = owned.result.take() {
                    state.jobs.remove(&job);
                    Ok(match result {
                        Ok(value) => json!({"done":true,"value":value}),
                        Err(error) => json!({"done":true,"error":error}),
                    })
                } else {
                    Ok(json!({"done":false}))
                }
            }
            Request::Cancel { job } => {
                state
                    .jobs
                    .get(&job)
                    .filter(|j| j.session == session)
                    .ok_or("This chat no longer owns that worktree job.")?
                    .cancel
                    .cancel();
                Ok(Value::Null)
            }
            Request::DiscardReview { token } => {
                if state
                    .lifecycle
                    .get(&token)
                    .is_some_and(|r| r.owner == session)
                {
                    state.lifecycle.remove(&token);
                }
                if state
                    .transfers
                    .get(&token)
                    .is_some_and(|r| r.receipt.source_session == session)
                {
                    state.transfers.remove(&token);
                }
                if state
                    .reviews
                    .get(&token)
                    .is_some_and(|r| r.source_session == session)
                {
                    state.reviews.remove(&token);
                }
                Ok(Value::Null)
            }
            operation => {
                if state.closed {
                    return Err(
                        "Worktree operations are closing. Restart Dolores before continuing."
                            .into(),
                    );
                }
                state
                    .reviews
                    .retain(|_, r| r.created.elapsed() < Duration::from_secs(600));
                state
                    .transfers
                    .retain(|_, r| r.created.elapsed() < Duration::from_secs(600));
                state
                    .lifecycle
                    .retain(|_, r| r.created.elapsed() < Duration::from_secs(600));
                if state.jobs.len() >= 8
                    || state.jobs.values().filter(|j| j.result.is_none()).count() >= 2
                {
                    return Err("Worktree operations are busy. Inspect pending results before retrying; your choices remain.".into());
                }
                let id = uuid::Uuid::new_v4().to_string();
                let cancel = CancellationToken::new();
                let work: Box<dyn FnOnce() -> Result<Value, String> + Send> = match operation {
                    Request::ReviewArchive
                    | Request::Archive { .. }
                    | Request::ReviewRestore
                    | Request::Restore { .. }
                    | Request::InspectLifecycle { .. } => {
                        let record = self.store.worktree(session)?.ok_or(
                            "Open the owning managed worktree details before archive or restore.",
                        )?;
                        if active.has_repository(Path::new(&record.common_directory))? {
                            return Err("Finish or stop repository tasks before archive/restore. Drafts and retained work remain.".into());
                        }
                        {
                            let terminals = self
                                .terminals
                                .lock()
                                .map_err(|_| "Terminal ownership is unavailable.")?;
                            if terminals.repository_busy(Path::new(&record.common_directory))?
                                || terminals.checkout_pinned(Path::new(&record.root))
                            {
                                return Err("Stop or close terminals in this checkout/repository before archive/restore. Retained terminal output remains.".into());
                            }
                        }
                        if self
                            .languages
                            .lock()
                            .map_err(|_| "Language ownership is unavailable.")?
                            .checkout_pinned(Path::new(&record.root))
                        {
                            return Err("Stop language services from Editor actions before archiving this checkout. Files and buffers remain.".into());
                        }
                        self.editor
                            .lock()
                            .map_err(|_| "Editor state is unavailable.")?
                            .ensure_git_clean(Path::new(&record.root), &[])?;
                        let data = self
                            .workspace_directory
                            .as_ref()
                            .and_then(|p| p.parent())
                            .ok_or("Managed worktree storage is unavailable.")?;
                        let managed = data.join("managed-worktrees");
                        let archives = data.join("worktree-archives");
                        if !archives.exists() {
                            std::fs::create_dir(&archives).map_err(|_|"Archive storage could not be created. Check disk space and access; checkout remains.")?;
                        }
                        let _archives = super::worktree_identity::Pin::open(&archives)?;
                        let store = self.store.clone();
                        let owner = session.to_string();
                        let token = cancel.clone();
                        let roots = vec![
                            PathBuf::from(&record.root),
                            PathBuf::from(&record.common_directory),
                        ];
                        let applying = matches!(
                            &operation,
                            Request::Archive { .. } | Request::Restore { .. }
                        );
                        let restoring =
                            matches!(&operation, Request::Restore { .. } | Request::ReviewRestore);
                        let task: Box<dyn FnOnce() -> Result<Value, String> + Send> =
                            match operation {
                                Request::ReviewArchive | Request::ReviewRestore => {
                                    if state.lifecycle.len() >= 8 {
                                        return Err("Close another archive/restore review before opening a new one.".into());
                                    }
                                    let restore = restoring;
                                    let registry = self.worktrees.clone();
                                    Box::new(move || {
                                        let (review, value) = if restore {
                                            super::worktree_archive::review_restore(
                                                &owner, &record, &managed, &archives, &token,
                                            )
                                        } else {
                                            super::worktree_archive::review_archive(
                                                &owner, &record, &managed, &archives, &token,
                                            )
                                        }?;
                                        if token.is_cancelled() {
                                            return Err("Archive/restore review stopped. Retained evidence remains; no lifecycle action was applied.".into());
                                        }
                                        let mut registry = registry
                                            .lock()
                                            .map_err(|_| "Lifecycle review is unavailable.")?;
                                        if registry.closed || registry.lifecycle.len() >= 8 {
                                            return Err("Lifecycle review could not be retained. Inspect saved evidence and review again.".into());
                                        }
                                        registry.lifecycle.insert(review.token.clone(), review);
                                        Ok(value)
                                    })
                                }
                                Request::Archive {
                                    token: review_token,
                                }
                                | Request::Restore {
                                    token: review_token,
                                } => {
                                    let restore = restoring;
                                    let reviewed=state.lifecycle.get(&review_token).filter(|r|r.owner==session&&r.restore==restore).ok_or("Archive/restore review expired or belongs to another chat. Review again.")?;
                                    if reviewed.record != record {
                                        return Err("Checkout lifecycle changed. Refresh and review again; no files were removed.".into());
                                    }
                                    self.git
                                        .lock()
                                        .map_err(|_| "Repository ownership is unavailable.")?
                                        .reserve_worktree(&id, &roots)?;
                                    let review = state.lifecycle.remove(&review_token).unwrap();
                                    Box::new(move || {
                                        if restore {
                                            super::worktree_restore::apply(
                                                store.as_ref(),
                                                review,
                                                &managed,
                                                &token,
                                            )
                                        } else {
                                            super::worktree_archive::apply_archive(
                                                store.as_ref(),
                                                review,
                                                &managed,
                                                &token,
                                            )
                                        }
                                    })
                                }
                                Request::InspectLifecycle { revision } => Box::new(move || {
                                    super::worktree_restore::inspect(
                                        store.as_ref(),
                                        &owner,
                                        revision,
                                        &managed,
                                        &archives,
                                        &token,
                                    )
                                }),
                                _ => unreachable!(),
                            };
                        if !applying {
                            self.git
                                .lock()
                                .map_err(|_| "Repository ownership is unavailable.")?
                                .reserve_worktree(&id, &roots)?;
                        }
                        task
                    }
                    Request::CompareTransfer
                    | Request::ReviewTransfer { .. }
                    | Request::ApplyTransfer { .. } => {
                        let source=record_for_chat(self.store.as_ref(),session)?.ok_or("Open a managed worktree chat before comparing or transferring changes.")?;
                        self.require_worktree_ready(session)?;
                        if active.has_repository(Path::new(&source.common_directory))? {
                            return Err("Finish or stop tasks in this repository before reviewing transfer. Files and drafts remain.".into());
                        }
                        if self
                            .terminals
                            .lock()
                            .map_err(|_| "Terminal ownership is unavailable.")?
                            .repository_busy(Path::new(&source.common_directory))?
                        {
                            return Err("Stop terminals in this repository before transferring changes. Their output and drafts remain.".into());
                        }
                        let parent = self.transfer_directory()?;
                        if !parent.exists() {
                            std::fs::create_dir(&parent).map_err(|_|"Transfer evidence storage is unavailable. Check disk space and access.")?;
                        }
                        let _parent = super::worktree_identity::Pin::open(&parent)?;
                        let roots = vec![
                            PathBuf::from(&source.root),
                            PathBuf::from(&source.common_directory),
                        ];
                        let applying = matches!(&operation, Request::ApplyTransfer { .. });
                        let task: Box<dyn FnOnce() -> Result<Value, String> + Send> =
                            match operation {
                                Request::CompareTransfer => {
                                    let token = cancel.clone();
                                    Box::new(move || {
                                        super::worktree_transfer::compare(&source, &parent, &token)
                                    })
                                }
                                Request::ReviewTransfer { destination, paths } => {
                                    if state.transfers.len() >= 8 {
                                        return Err("Close another transfer review before opening a new one.".into());
                                    }
                                    let target = self
                                        .store
                                        .workspace(&destination)?
                                        .root
                                        .ok_or("Choose a destination project chat.")?;
                                    self.require_worktree_ready(&destination)?;
                                    self.editor
                                        .lock()
                                        .map_err(|_| "Editor state is unavailable.")?
                                        .ensure_git_clean(Path::new(&source.root), &paths)?;
                                    self.editor
                                        .lock()
                                        .map_err(|_| "Editor state is unavailable.")?
                                        .ensure_git_clean(Path::new(&target), &paths)?;
                                    let registry = self.worktrees.clone();
                                    let owner = session.to_string();
                                    let token = cancel.clone();
                                    Box::new(move || {
                                        let (review, value) = super::worktree_transfer::review(
                                            &owner,
                                            &source,
                                            &destination,
                                            Path::new(&target),
                                            &parent,
                                            paths,
                                            &token,
                                        )?;
                                        if token.is_cancelled() {
                                            return Err("Transfer review stopped. Retained snapshots remain; no transfer was applied.".into());
                                        }
                                        let mut state = registry.lock().map_err(|_| {
                                            "Transfer review could not be retained."
                                        })?;
                                        if state.closed || state.transfers.len() >= 8 {
                                            return Err("Transfer review could not be retained. Evidence remains; compare again later.".into());
                                        }
                                        state.transfers.insert(review.token.clone(), review);
                                        Ok(value)
                                    })
                                }
                                Request::ApplyTransfer { token } => {
                                    let reviewed=state.transfers.get(&token).filter(|r|r.receipt.source_session==session).ok_or("Transfer review expired or belongs to another chat. Compare again.")?;
                                    self.editor
                                        .lock()
                                        .map_err(|_| "Editor state is unavailable.")?
                                        .ensure_git_clean(
                                            Path::new(&source.root),
                                            &reviewed.receipt.paths,
                                        )?;
                                    self.editor
                                        .lock()
                                        .map_err(|_| "Editor state is unavailable.")?
                                        .ensure_git_clean(
                                            Path::new(&reviewed.receipt.destination_root),
                                            &reviewed.receipt.paths,
                                        )?;
                                    // Reserve before consuming the one-use review. Admission, editor,
                                    // language and terminal paths all check this common boundary.
                                    self.git
                                        .lock()
                                        .map_err(|_| "Repository ownership is unavailable.")?
                                        .reserve_worktree(&id, &roots)?;
                                    let review = state.transfers.remove(&token).unwrap();
                                    let store = self.store.clone();
                                    let token = cancel.clone();
                                    Box::new(move || {
                                        super::worktree_transfer::apply(
                                            store.as_ref(),
                                            review,
                                            &token,
                                        )
                                    })
                                }
                                _ => unreachable!(),
                            };
                        // Apply reserved above before review consumption; comparison/review
                        // need the same owner so queued model work cannot alter their basis.
                        if !applying {
                            self.git
                                .lock()
                                .map_err(|_| "Repository ownership is unavailable.")?
                                .reserve_worktree(&id, &roots)?;
                        }
                        task
                    }
                    Request::ReconcileCreation { revision } => {
                        if active.has_session(session) {
                            return Err(
                                "Finish this chat's task before inspecting recovery.".into()
                            );
                        }
                        let store = self.store.clone();
                        let session = session.to_owned();
                        let token = cancel.clone();
                        Box::new(move || {
                            worktree_git::reconcile_creation(
                                store.as_ref(),
                                &session,
                                revision,
                                &token,
                            )
                            .map(|record| json!({"record":record,"session":record.session}))
                        })
                    }
                    Request::ReviewCreate { base, setup } => {
                        if state.reviews.len() >= 8 {
                            return Err(
                                "Close an existing worktree review before opening another.".into(),
                            );
                        }
                        let root = self
                            .store
                            .workspace(session)?
                            .root
                            .ok_or("Open a project chat before creating a worktree.")?;
                        self.require_worktree_ready(session)?;
                        let parent = self
                            .workspace_directory
                            .as_ref()
                            .and_then(|p| p.parent())
                            .ok_or("Managed worktree storage is unavailable.")?
                            .join("managed-worktrees");
                        let registry = self.worktrees.clone();
                        let source_session = session.to_owned();
                        let token = cancel.clone();
                        Box::new(move || {
                            let review = worktree_git::review(
                                &source_session,
                                Path::new(&root),
                                &parent,
                                &base,
                                setup,
                                &token,
                            )?;
                            let value = json!({"token":review.token,"record":review.record,"notice":"Create an independent detached checkout at this exact commit. Original dirty files stay in their original checkout. Automatic setup, Git hooks and filters are skipped; filtered files may require manual setup."});
                            if token.is_cancelled() {
                                return Err(
                                    "Worktree review stopped. No checkout was created.".into()
                                );
                            }
                            let mut state = registry
                                .lock()
                                .map_err(|_| "Worktree review could not be retained.")?;
                            if state.closed || state.reviews.len() >= 8 {
                                return Err("Worktree review could not be retained. Close another review and try again.".into());
                            }
                            state.reviews.insert(review.token.clone(), review);
                            Ok(value)
                        })
                    }
                    Request::Create { token } => {
                        let review=state.reviews.get(&token).filter(|r|r.source_session==session).ok_or("Creation review expired or belongs to another chat. Review creation again.")?;
                        let root = self
                            .store
                            .workspace(session)?
                            .root
                            .ok_or("Source chat no longer has a working folder.")?;
                        if !super::source_control::within(
                            Path::new(&root),
                            Path::new(&review.record.source_root),
                        ) || !super::source_control::within(
                            Path::new(&review.record.source_root),
                            Path::new(&root),
                        ) {
                            return Err(
                                "Source chat changed after review. Review creation again.".into()
                            );
                        }
                        if review.roots.iter().any(|root| active.has_repo(root))
                            || active.has_repository(Path::new(&review.record.common_directory))?
                        {
                            return Err("Finish or stop tasks in this repository before creating its worktree. Your creation review remains.".into());
                        }
                        self.git
                            .lock()
                            .map_err(|_| "Repository ownership is unavailable.")?
                            .reserve_worktree(&id, &review.roots)?;
                        let review = state.reviews.remove(&token).unwrap();
                        let store = self.store.clone();
                        let token = cancel.clone();
                        Box::new(move || {
                            worktree_git::create(store.as_ref(), &review, &token)
                                .map(|record| json!({"record":record,"session":record.session}))
                        })
                    }
                    _ => unreachable!(),
                };
                state.jobs.insert(
                    id.clone(),
                    Job {
                        session: session.into(),
                        cancel,
                        result: None,
                    },
                );
                let registry = self.worktrees.clone();
                let git = self.git.clone();
                let owned = id.clone();
                // A failed thread spawn must release the repository reservation too.
                if let Err(error)=std::thread::Builder::new().name("dolores-worktree".into()).spawn(move || {
                    let result=std::panic::catch_unwind(std::panic::AssertUnwindSafe(work)).unwrap_or_else(|_|Err("Worktree operation stopped unexpectedly. Inspect its saved intent and folder before retrying.".into()));
                    if let Ok(mut state)=git.lock() {state.release_worktree(&owned);}
                    finish(&registry,&owned,result);
                }) {
                    state.jobs.remove(&id);
                    self.git.lock().map_err(|_|"Repository ownership is unavailable.")?.release_worktree(&id);
                    return Err(format!("Could not start worktree operation: {error}. Review again."));
                }
                Ok(json!({"job":id}))
            }
        }
    }

    pub(super) fn require_worktree_ready(&self, session: &str) -> Result<(), String> {
        self.pin_worktree(session).map(|_| ())
    }

    pub(super) fn pin_worktree(
        &self,
        session: &str,
    ) -> Result<Option<worktree_git::ReadyPins>, String> {
        if let Some(record) = record_for_chat(self.store.as_ref(), session)? {
            let workspace = self.store.workspace(session)?;
            if !workspace.root.as_ref().is_some_and(|root| {
                super::source_control::within(Path::new(root), Path::new(&record.root))
            }) {
                return Err("Chat checkout binding differs from its managed worktree. Inspect recovery details.".into());
            }
            return worktree_git::validate_ready(&record).map(Some);
        }
        Ok(None)
    }

    pub(super) fn stop_worktrees(&self) -> Result<(), String> {
        {
            let mut state = self
                .worktrees
                .lock()
                .map_err(|_| "Worktree operations are unavailable.")?;
            state.closed = true;
            state.reviews.clear();
            state.transfers.clear();
            state.lifecycle.clear();
            for job in state.jobs.values() {
                job.cancel.cancel();
            }
        }
        let started = Instant::now();
        loop {
            if self
                .worktrees
                .lock()
                .map_err(|_| "Worktree operations are unavailable.")?
                .jobs
                .values()
                .all(|j| j.result.is_some())
            {
                return Ok(());
            }
            if started.elapsed() > Duration::from_secs(5) {
                return Err("Worktree cleanup is still running. Wait before quitting; the saved operation intent remains recoverable.".into());
            }
            std::thread::sleep(Duration::from_millis(10));
        }
    }
    fn transfer_directory(&self) -> Result<PathBuf, String> {
        self.workspace_directory
            .as_ref()
            .and_then(|p| p.parent())
            .map(|p| p.join("worktree-transfers"))
            .ok_or("Transfer evidence storage is unavailable.".into())
    }
}

fn finish(registry: &Arc<Mutex<Registry>>, id: &str, result: Result<Value, String>) {
    if let Ok(mut state) = registry.lock() {
        if let Some(job) = state.jobs.get_mut(id) {
            job.result = Some(result);
        }
    }
}

pub(super) fn record_for_chat(
    store: &dyn dolores_core::SessionStore,
    session: &str,
) -> Result<Option<ManagedWorktree>, String> {
    if let Some(record) = store.worktree(session)? {
        return Ok(Some(record));
    }
    match store.workspace(session)?.root {
        Some(root) => store.worktree_for_root(&root),
        None => Ok(None),
    }
}

#[cfg(test)]
#[path = "worktree_tests.rs"]
mod tests;
