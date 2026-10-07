use dolores_core::*;
use serde_json::{json, Value};
use std::{
    collections::VecDeque,
    sync::{Arc, Mutex},
};
use tokio_util::sync::CancellationToken;

const MAX_WAITING_CHATS: usize = 16;
struct Job {
    store: Arc<dyn SessionStore>,
    provider: Arc<dyn ModelProvider>,
    update: AutomaticMemoryUpdate,
    cancel: CancellationToken,
}
#[derive(Default)]
struct State {
    waiting: VecDeque<Job>,
    running: bool,
    closed: bool,
    current: Option<CancellationToken>,
}
#[derive(Default)]
pub(super) struct Maintenance {
    state: Mutex<State>,
}

impl Maintenance {
    pub fn stop(&self, close: bool) {
        let waiting = if let Ok(mut state) = self.state.lock() {
            if let Some(current) = state.current.take() {
                current.cancel();
            }
            state.closed |= close;
            state.waiting.drain(..).collect::<Vec<_>>()
        } else {
            Vec::new()
        };
        for job in waiting {
            job.cancel.cancel();
            let _ = job.store.finish_automatic_memory(&job.update, &job.cancel);
        }
    }

    pub async fn enqueue(
        self: &Arc<Self>,
        store: Arc<dyn SessionStore>,
        provider: Arc<dyn ModelProvider>,
        session: String,
        model: String,
    ) -> Option<Value> {
        let update = match super::automatic_memory::prepare(store.clone(), &session, &model).await {
            Ok(Some(update)) => update,
            Ok(None) => return None,
            Err(_) => {
                return Some(
                    json!({"status":"failed","note":"Memory source could not be prepared. Your reply is saved; inspect Memory and finish a new interaction to try again."}),
                )
            }
        };
        let report = json!({"messageId":update.source.message_id,"status":"queued","note":"Memory is updating in the background. Inspect Memory for the saved result and separate usage.","saved":0,"skipped":0});
        let mut rejected = None;
        let start = {
            let mut state = self.state.lock().ok()?;
            let job = Job {
                store: store.clone(),
                provider,
                update,
                cancel: CancellationToken::new(),
            };
            if state.closed
                || state.waiting.len() >= MAX_WAITING_CHATS
                    && !state.waiting.iter().any(|j| j.update.session == session)
            {
                rejected = Some(job);
                false
            } else {
                if let Some(pos) = state
                    .waiting
                    .iter()
                    .position(|j| j.update.session == session)
                {
                    if let Some(old) = state.waiting.remove(pos) {
                        old.cancel.cancel();
                    }
                }
                state.waiting.push_back(job);
                let start = !state.running;
                state.running = true;
                start
            }
        };
        if let Some(mut job) = rejected {
            job.update.status = "skipped".into();
            job.update.note="Memory queue is full or shutting down. Your reply is saved; finish another interaction after background work completes.".into();
            return super::blocking(move || {
                job.store.finish_automatic_memory(&job.update, &job.cancel)
            })
            .await
            .ok()
            .map(|r| json!(r));
        }
        if start {
            let queue = self.clone();
            tokio::spawn(async move {
                queue.drain().await;
            });
        }
        Some(report)
    }

    async fn drain(self: Arc<Self>) {
        loop {
            let mut job = {
                let Ok(mut state) = self.state.lock() else {
                    return;
                };
                match state.waiting.pop_front() {
                    Some(job) => {
                        state.current = Some(job.cancel.clone());
                        job
                    }
                    None => {
                        state.running = false;
                        state.current = None;
                        return;
                    }
                }
            };
            let reader = job.store.clone();
            let sid = job.update.session.clone();
            let revision = job.update.policy_revision;
            let existing = super::blocking(move || {
                let policy = reader.automatic_memory_policy()?;
                if !policy.enabled || policy.revision != revision {
                    return Err("Memory policy changed.".into());
                }
                super::memory::preferences_for_session(reader.as_ref(), Some(&sid))
            })
            .await;
            match existing {
                Ok(existing) => job.update.existing = existing,
                Err(_) => job.cancel.cancel(),
            }
            // No idle worker, reflection timer or automatic replay. Publication
            // rechecks the pinned source and policy even after a successful request.
            let _ = super::automatic_memory::learn(job.store, job.provider, job.update, job.cancel)
                .await;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use tokio::sync::{mpsc, Notify};
    struct Slow {
        entered: Notify,
        calls: AtomicUsize,
    }
    #[async_trait::async_trait]
    impl ModelProvider for Slow {
        fn descriptor(&self) -> PluginDescriptor {
            PluginDescriptor {
                id: "memory-fixture",
                kind: "provider",
                api_version: 1,
            }
        }
        async fn stream(
            &self,
            _: Vec<Message>,
            _: mpsc::Sender<String>,
            cancel: CancellationToken,
        ) -> Result<(), String> {
            self.calls.fetch_add(1, Ordering::SeqCst);
            self.entered.notify_one();
            cancel.cancelled().await;
            Err("Fixture stopped".into())
        }
    }
    #[tokio::test]
    async fn background_admission_is_prompt_single_worker_bounded_and_stoppable() {
        let temp = tempfile::tempdir().unwrap();
        let store = Arc::new(
            dolores_store_sqlite::SqliteStore::open(&temp.path().join("memory.db")).unwrap(),
        );
        store.set_automatic_memory_policy(true, 1).unwrap();
        let queue = Arc::new(Maintenance::default());
        let provider = Arc::new(Slow {
            entered: Notify::new(),
            calls: AtomicUsize::new(0),
        });
        store.create("active").unwrap();
        store
            .commit_turn("active", "We decided to use SQLite.", "Reply retained")
            .unwrap();
        let result = tokio::time::timeout(
            std::time::Duration::from_millis(500),
            queue.enqueue(
                store.clone(),
                provider.clone(),
                "active".into(),
                "fixture".into(),
            ),
        )
        .await
        .unwrap()
        .unwrap();
        assert_eq!(result["status"], "queued");
        tokio::time::timeout(
            std::time::Duration::from_secs(3),
            provider.entered.notified(),
        )
        .await
        .expect("The deliberately slow model path must have started");
        for n in 0..16 {
            let session = format!("waiting-{n}");
            store.create(&session).unwrap();
            store
                .commit_turn(&session, "We decided to use SQLite.", "Retained")
                .unwrap();
            queue
                .enqueue(store.clone(), provider.clone(), session, "fixture".into())
                .await
                .unwrap();
        }
        store
            .commit_turn("waiting-0", "We decided to use Rust.", "New retained reply")
            .unwrap();
        assert_eq!(
            queue
                .enqueue(
                    store.clone(),
                    provider.clone(),
                    "waiting-0".into(),
                    "fixture".into()
                )
                .await
                .unwrap()["status"],
            "queued"
        );
        assert_eq!(queue.state.lock().unwrap().waiting.len(), 16);
        store.create("overflow").unwrap();
        store
            .commit_turn("overflow", "Our project codename is Birch.", "Retained")
            .unwrap();
        let full = queue
            .enqueue(
                store.clone(),
                provider.clone(),
                "overflow".into(),
                "fixture".into(),
            )
            .await
            .unwrap();
        assert_eq!(full["status"], "skipped");
        assert!(full["note"].as_str().unwrap().contains("queue is full"));
        assert_eq!(provider.calls.load(Ordering::SeqCst), 1);
        store.set_automatic_memory_policy(false, 2).unwrap();
        queue.stop(false);
        for _ in 0..100 {
            if !queue.state.lock().unwrap().running {
                break;
            }
            tokio::time::sleep(std::time::Duration::from_millis(5)).await;
        }
        assert!(!queue.state.lock().unwrap().running);
        assert!(store.memory_preferences(None).unwrap().is_empty());
        assert_eq!(store.messages("active").unwrap().len(), 2);
        assert_eq!(
            store
                .automatic_memory_attempt("active")
                .unwrap()
                .unwrap()
                .status,
            "stopped"
        );
        assert_eq!(provider.calls.load(Ordering::SeqCst), 1);
    }
}
