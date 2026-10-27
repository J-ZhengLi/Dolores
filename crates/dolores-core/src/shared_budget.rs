use crate::TaskBudget;
use serde::Serialize;
use std::sync::Mutex;

/// Reservations count attempts, including blocked/failed work. Never refunded.
pub struct SharedTaskBudget {
    automatic: std::sync::atomic::AtomicBool,
    limit: TaskBudget,
    used: Mutex<SharedTaskUsage>,
    ledger: Option<std::sync::Arc<dyn BudgetLedger>>,
    error: Mutex<Option<String>>,
}
pub trait BudgetLedger: Send + Sync {
    fn reserve(&self, models: usize, tools: usize) -> Result<bool, String>;
}
#[derive(Clone, Copy, Default, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SharedTaskUsage {
    pub model_calls: usize,
    pub tool_calls: usize,
}
impl SharedTaskBudget {
    pub fn new(limit: TaskBudget) -> Self {
        Self {
            automatic: std::sync::atomic::AtomicBool::new(false),
            limit,
            used: Mutex::new(SharedTaskUsage::default()),
            ledger: None,
            error: Mutex::new(None),
        }
    }
    pub fn usage(&self) -> SharedTaskUsage {
        *self.used.lock().unwrap_or_else(|e| e.into_inner())
    }
    pub fn enable_automatic(&self) {
        self.automatic
            .store(true, std::sync::atomic::Ordering::SeqCst);
    }
    pub fn with_ledger(limit: TaskBudget, ledger: std::sync::Arc<dyn BudgetLedger>) -> Self {
        Self {
            ledger: Some(ledger),
            ..Self::new(limit)
        }
    }
    pub fn error(&self) -> Option<String> {
        self.error.lock().unwrap_or_else(|e| e.into_inner()).clone()
    }
    fn charge(&self, models: usize, tools: usize) -> bool {
        match self
            .ledger
            .as_ref()
            .map(|l| l.reserve(models, tools))
            .transpose()
        {
            Ok(Some(allowed)) => allowed,
            Ok(None) => true,
            Err(e) => {
                *self.error.lock().unwrap_or_else(|e| e.into_inner()) = Some(e);
                false
            }
        }
    }
    pub fn reserve_model(&self, child: bool) -> bool {
        let mut used = self.used.lock().unwrap_or_else(|e| e.into_inner());
        let limit = if self.automatic.load(std::sync::atomic::Ordering::SeqCst)
            && self.limit.model_calls.is_none()
        {
            usize::MAX
        } else {
            self.limit.model_limit()
        }
        .saturating_sub(usize::from(child));
        if used.model_calls >= limit || !self.charge(1, 0) {
            return false;
        }
        used.model_calls += 1;
        true
    }
    pub fn reserve_tools(&self, count: usize) -> bool {
        let mut used = self.used.lock().unwrap_or_else(|e| e.into_inner());
        let limit = if self.automatic.load(std::sync::atomic::Ordering::SeqCst)
            && self.limit.tool_calls.is_none()
        {
            usize::MAX
        } else {
            self.limit.tool_limit()
        };
        if count > limit.saturating_sub(used.tool_calls) || !self.charge(0, count) {
            return false;
        }
        used.tool_calls += count;
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn children_cannot_spend_parent_report_or_race_past_total() {
        let shared = std::sync::Arc::new(SharedTaskBudget::new(TaskBudget {
            model_calls: Some(4),
            tool_calls: Some(4),
            ..Default::default()
        }));
        let threads: Vec<_> = (0..12)
            .map(|_| {
                let shared = shared.clone();
                std::thread::spawn(move || (shared.reserve_model(true), shared.reserve_tools(1)))
            })
            .collect();
        let results: Vec<_> = threads.into_iter().map(|t| t.join().unwrap()).collect();
        assert_eq!(results.iter().filter(|v| v.0).count(), 3);
        assert_eq!(results.iter().filter(|v| v.1).count(), 4);
        assert!(shared.reserve_model(false));
        assert!(!shared.reserve_model(false));
        assert!(!shared.reserve_tools(1));
    }
}
