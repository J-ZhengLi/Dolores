//! Cleanup is a separate, bounded operation. It can never dispatch tools/call.
use super::*;
use std::sync::{Arc, Mutex};
use tokio::sync::watch;

#[derive(Clone, Copy)]
pub(super) enum Outcome {
    Confirmed,
    Unsupported,
    Unconfirmed,
}
pub(super) struct Job {
    connection: String,
    result: watch::Receiver<Option<Outcome>>,
}
impl Job {
    pub async fn wait(&self) -> Outcome {
        let mut result = self.result.clone();
        loop {
            if let Some(confirmed) = *result.borrow() {
                return confirmed;
            }
            if result.changed().await.is_err() {
                return Outcome::Unconfirmed;
            }
        }
    }
}
#[derive(Default)]
pub struct CleanupOwner {
    jobs: Mutex<Vec<Arc<Job>>>,
}
impl CleanupOwner {
    /// Called after the owning invocation futures have settled or been dropped.
    /// Jobs already run concurrently and have a two-second independent deadline.
    pub async fn finish(&self) -> Vec<Value> {
        let jobs = std::mem::take(&mut *self.jobs.lock().unwrap_or_else(|e| e.into_inner()));
        let mut receipts = Vec::with_capacity(jobs.len());
        for job in jobs {
            let status = job.wait().await;
            receipts.push(json!({"connectionId":job.connection,"confirmed":matches!(status,Outcome::Confirmed),"status":match status {Outcome::Confirmed=>"confirmed",Outcome::Unsupported=>"unsupported",Outcome::Unconfirmed=>"unconfirmed"},"operation":"sessionCleanup","toolReplayed":false}));
        }
        receipts
    }
}
pub(super) fn start(client: &RemoteClient, session: String) -> Arc<Job> {
    let (result, receiver) = watch::channel(None);
    let job = Arc::new(Job {
        connection: client
            .cleanup_owner
            .as_ref()
            .map(|(_, id)| id.clone())
            .unwrap_or_default(),
        result: receiver,
    });
    if let Some((owner, _)) = &client.cleanup_owner {
        owner
            .jobs
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .push(job.clone());
    }
    let http = client.client.clone();
    let url = client.url.clone();
    let bearer = client.bearer.clone();
    let protocol = client.protocol.clone();
    if let Ok(runtime) = tokio::runtime::Handle::try_current() {
        runtime.spawn(async move {
            let cleanup = async {
                let mut header = HeaderValue::from_str(&session).map_err(|_| ())?;
                header.set_sensitive(true);
                let mut request = http
                    .delete(url)
                    .header("mcp-session-id", header)
                    .header("mcp-protocol-version", protocol);
                if let Some(bearer) = bearer {
                    request = request.bearer_auth(bearer);
                }
                let response = request.send().await.map_err(|_| ())?;
                Ok::<Outcome, ()>(
                    if response.status().is_success() || response.status().as_u16() == 404 {
                        Outcome::Confirmed
                    } else if response.status().as_u16() == 405 {
                        Outcome::Unsupported
                    } else {
                        Outcome::Unconfirmed
                    },
                )
            };
            let confirmed = tokio::time::timeout(Duration::from_secs(2), cleanup)
                .await
                .ok()
                .and_then(Result::ok)
                .unwrap_or(Outcome::Unconfirmed);
            let _ = result.send(Some(confirmed));
        });
    } else {
        let _ = result.send(Some(Outcome::Unconfirmed));
    }
    job
}
