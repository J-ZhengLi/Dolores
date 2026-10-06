use std::{collections::VecDeque, sync::{Arc, Mutex, atomic::{AtomicBool, Ordering}}};
use tokio::sync::Notify;
use tokio_util::sync::CancellationToken;

pub(super) const MAX_PRIMARY: usize = 2;
pub(super) const MAX_QUEUED: usize = 4;

struct Entry { id: u64, scope: String, running: bool }
#[derive(Default)]
pub(super) struct Admission { entries: Mutex<VecDeque<Entry>>, changed: Notify, closed: AtomicBool }
pub(super) struct Permit { owner: Arc<Admission>, id: u64 }
impl Drop for Permit { fn drop(&mut self) { self.owner.remove(self.id); } }

impl Admission {
    pub fn close(&self) {
        let _entries = self.entries.lock().unwrap_or_else(|e| e.into_inner());
        self.closed.store(true, Ordering::SeqCst);
        self.changed.notify_waiters();
    }
    pub fn check(&self, id:u64, scope:&str) -> Result<(),String> {
        let entries=self.entries.lock().map_err(|_| "Run queue is unavailable.")?;
        self.check_entries(&entries,id,scope)
    }
    fn check_entries(&self,entries:&VecDeque<Entry>,id:u64,scope:&str)->Result<(),String>{
        if self.closed.load(Ordering::SeqCst){return Err(crate::stopped());}
        let running=entries.iter().filter(|e|e.running).count();
        let immediate=running<MAX_PRIMARY&&!entries.iter().any(|e|e.running&&e.scope==scope);
        if entries.len()>=MAX_PRIMARY+MAX_QUEUED||(!immediate&&entries.iter().filter(|e|!e.running).count()>=MAX_QUEUED){
            return Err("The run queue is full (two active and four waiting). Stop or cancel a task before sending; your draft remains.".into());
        }
        if entries.iter().any(|e|e.id==id){return Err("This run ID is already owned.".into());}
        Ok(())
    }
    pub fn insert(&self, id: u64, scope: String) -> Result<(), String> {
        let mut entries = self.entries.lock().map_err(|_| "Run queue is unavailable.")?;
        self.check_entries(&entries,id,&scope)?;
        let running = entries.iter().filter(|e| e.running).count();
        let immediate = running < MAX_PRIMARY && !entries.iter().any(|e| e.running && e.scope == scope);
        entries.push_back(Entry { id, scope, running: immediate });
        self.changed.notify_waiters();
        Ok(())
    }
    pub fn remove(&self, id: u64) {
        if let Ok(mut entries) = self.entries.lock() { entries.retain(|e| e.id != id); }
        self.changed.notify_waiters();
    }
    pub async fn acquire(self: &Arc<Self>, id: u64, cancel: &CancellationToken) -> Result<Permit, String> {
        loop {
            let notified = self.changed.notified();
            tokio::pin!(notified);
            notified.as_mut().enable();
            if cancel.is_cancelled() { self.remove(id); return Err(crate::stopped()); }
            {
                let mut entries = self.entries.lock().map_err(|_| "Run queue is unavailable.")?;
                if self.closed.load(Ordering::SeqCst){return Err(crate::stopped());}
                if entries.iter().any(|e| e.id == id && e.running) {
                    return Ok(Permit { owner: self.clone(), id });
                }
                let running = entries.iter().filter(|e| e.running).count();
                let next = entries.iter().position(|e| !e.running &&
                    !entries.iter().any(|r| r.running && r.scope == e.scope));
                if running < MAX_PRIMARY && next.is_some_and(|i| entries[i].id == id) {
                    entries[next.unwrap()].running = true;
                    self.changed.notify_waiters();
                    return Ok(Permit { owner: self.clone(), id });
                }
                if !entries.iter().any(|e| e.id == id) { return Err(crate::stopped()); }
            }
            tokio::select! { _ = cancel.cancelled() => {
                self.remove(id); return Err(crate::stopped());
            }, _ = notified => {} }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[tokio::test]
    async fn shutdown_cannot_admit_queued_work_after_a_slot_is_released(){
        let owner=Arc::new(Admission::default());let cancel=CancellationToken::new();
        owner.insert(1,"A".into()).unwrap();let first=owner.acquire(1,&cancel).await.unwrap();
        for id in 2..=5{owner.insert(id,"A".into()).unwrap();}
        assert!(owner.check(6,"A").unwrap_err().contains("queue is full"));
        let waiting=owner.clone();let task=tokio::spawn(async move{waiting.acquire(2,&cancel).await});
        tokio::task::yield_now().await;owner.close();drop(first);
        assert!(tokio::time::timeout(std::time::Duration::from_secs(1),task).await.unwrap().unwrap().is_err());
        assert!(owner.insert(6,"B".into()).is_err());
    }
    #[tokio::test]
    async fn two_roots_run_while_same_root_waits_and_cancel_preserves_other_owners() {
        let owner = Arc::new(Admission::default());
        let cancel = CancellationToken::new();
        owner.insert(1, "A".into()).unwrap();
        let first = owner.acquire(1, &cancel).await.unwrap();
        owner.insert(2, "A".into()).unwrap();
        owner.insert(3, "B".into()).unwrap();
        let third = owner.acquire(3, &cancel).await.unwrap();
        let stopped = CancellationToken::new(); stopped.cancel();
        assert!(owner.acquire(2, &stopped).await.is_err());
        assert_eq!(owner.entries.lock().unwrap().iter().filter(|e|e.running).count(), 2);
        owner.insert(4, "A".into()).unwrap();
        drop(first);
        let fourth = owner.acquire(4, &cancel).await.unwrap();
        drop(third); drop(fourth);
        assert!(owner.entries.lock().unwrap().is_empty());
    }
    #[tokio::test]
    async fn queue_is_bounded_and_slot_release_wakes_a_waiter() {
        let owner = Arc::new(Admission::default());
        let cancel = CancellationToken::new();
        for id in 1..=6 { owner.insert(id, format!("root-{id}")).unwrap(); }
        assert!(owner.insert(7, "other".into()).unwrap_err().contains("queue is full"));
        let one = owner.acquire(1, &cancel).await.unwrap();
        let two = owner.acquire(2, &cancel).await.unwrap();
        let waiting = owner.clone();
        let task = tokio::spawn(async move { waiting.acquire(3, &cancel).await });
        tokio::task::yield_now().await;
        assert!(!task.is_finished());
        drop(one);
        let three = tokio::time::timeout(std::time::Duration::from_secs(1),task).await.unwrap().unwrap().unwrap();
        drop(two); drop(three);
    }
}
