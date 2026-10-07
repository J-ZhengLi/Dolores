//! Private display checkpoints never restore processes or replay shell input.
use serde_json::Value;
pub(super) const KEY: &str = "$dolores:terminals:v1";
pub(super) fn validate(value: &Value) -> Result<(), String> {
    let bad = || {
        "Terminal recovery is invalid. Retained shells and the previous checkpoint remain."
            .to_string()
    };
    if serde_json::to_vec(value).map_err(|_| bad())?.len() > 96 * 1024 || value["version"] != 1 {
        return Err(bad());
    }
    let sessions = value["sessions"].as_array().ok_or_else(bad)?;
    if sessions.len() > 8 {
        return Err(bad());
    }
    let mut ids = std::collections::HashSet::new();
    for s in sessions {
        for (field, max) in [
            ("id", 80),
            ("cwd", 4096),
            ("shell", 4096),
            ("title", 480),
            ("output", 8192),
        ] {
            let text = s[field].as_str().ok_or_else(bad)?;
            if text.len() > max || text.contains('\0') {
                return Err(bad());
            }
        }
        if !ids.insert(s["id"].as_str().unwrap()) {
            return Err(bad());
        }
    }
    let groups = value["groups"].as_array().ok_or_else(bad)?;
    if groups.is_empty() || groups.len() > 4 {
        return Err(bad());
    }
    let mut group_ids = std::collections::HashSet::new();
    let mut placed = std::collections::HashSet::new();
    for g in groups {
        let id = g["id"].as_str().ok_or_else(bad)?;
        if id.len() > 80 || !group_ids.insert(id) {
            return Err(bad());
        }
        let tabs = g["tabs"].as_array().ok_or_else(bad)?;
        for tab in tabs {
            let id = tab.as_str().ok_or_else(bad)?;
            if !ids.contains(id) || !placed.insert(id) {
                return Err(bad());
            }
        }
        if !g["active"].is_null() && !tabs.contains(&g["active"]) {
            return Err(bad());
        }
    }
    if ids != placed || !group_ids.contains(value["activeGroup"].as_str().ok_or_else(bad)?) {
        return Err(bad());
    }
    fn tree<'a>(
        v: &'a Value,
        depth: usize,
        leaves: &mut std::collections::HashSet<&'a str>,
    ) -> bool {
        if depth > 3 {
            return false;
        }
        if let Some(g) = v["group"].as_str() {
            return leaves.insert(g);
        }
        matches!(v["axis"].as_str(), Some("horizontal" | "vertical"))
            && v["ratio"]
                .as_f64()
                .is_some_and(|r| (0.15..=0.85).contains(&r))
            && tree(&v["first"], depth + 1, leaves)
            && tree(&v["second"], depth + 1, leaves)
    }
    let mut leaves = std::collections::HashSet::new();
    if !tree(&value["tree"], 0, &mut leaves) || leaves != group_ids {
        return Err(bad());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    #[test]
    fn recovery_is_bounded_and_cannot_duplicate_process_owners() {
        let v = json!({"version":1,"sessions":[{"id":"one","cwd":"C:/","shell":"cmd","title":"Shell","output":"kept 世界"}],"groups":[{"id":"g","tabs":["one"],"active":"one"}],"activeGroup":"g","tree":{"group":"g"}});
        validate(&v).unwrap();
        let mut bad = v.clone();
        bad["sessions"][0]["output"] = json!("x".repeat(8193));
        assert!(validate(&bad).is_err());
        let mut bad = v.clone();
        bad["groups"][0]["tabs"] = json!(["one", "one"]);
        assert!(validate(&bad).is_err());
        let mut bad = v;
        bad["tree"] = json!({"group":"missing"});
        assert!(validate(&bad).is_err());
    }
}
