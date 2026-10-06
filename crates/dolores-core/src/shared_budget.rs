use crate::TaskBudget;
use serde::Serialize;
use std::sync::Mutex;

/// Reservations count attempts, including blocked/failed work. Never refunded.
pub struct SharedTaskBudget {
    limit: TaskBudget,
    used: Mutex<SharedTaskUsage>,
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
            limit,
            used: Mutex::new(SharedTaskUsage::default()),
        }
    }
    pub fn usage(&self) -> SharedTaskUsage {
        *self.used.lock().unwrap_or_else(|e| e.into_inner())
    }
    pub fn reserve_model(&self, child: bool) -> bool {
        let mut used = self.used.lock().unwrap_or_else(|e| e.into_inner());
        let limit = self.limit.model_limit().saturating_sub(usize::from(child));
        if used.model_calls >= limit {
            return false;
        }
        used.model_calls += 1;
        true
    }
    pub fn reserve_tools(&self, count: usize) -> bool {
        let mut used = self.used.lock().unwrap_or_else(|e| e.into_inner());
        if count > self.limit.tool_limit().saturating_sub(used.tool_calls) {
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
