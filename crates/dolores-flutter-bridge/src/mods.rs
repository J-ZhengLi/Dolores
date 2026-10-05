use crate::Engine;
use dolores_core::{ModManifest, ModState, ModVersion, SessionStore};
use dolores_mod_runtime::{digest, evaluate, RecoveryMod, BASELINE};
use serde_json::{json, Value};
use tokio_util::sync::CancellationToken;

fn root(store: &dyn SessionStore, session: &str) -> Result<String, String> {
    store
        .workspace(session)?
        .root
        .ok_or("Open a working chat for project-scoped mods.".into())
}
pub(super) fn identity(manifest: &ModManifest, source: &str) -> String {
    digest(&format!(
        "{}\n{source}",
        serde_json::to_string(manifest).expect("manifest serialization")
    ))
}
fn check_version(v: &ModVersion) -> Result<RecoveryMod, String> {
    v.manifest.validate()?;
    if identity(&v.manifest, &v.source) != v.identity {
        return Err(
            "Mod source or manifest changed. Retest exact bytes; baseline retained.".into(),
        );
    }
    RecoveryMod::compile(&v.source)
}
pub(super) fn stage(
    store: &dyn SessionStore,
    session: &str,
    revision: u32,
    manifest: ModManifest,
    source: String,
    model: String,
    cancel: &CancellationToken,
) -> Result<ModState, String> {
    manifest.validate()?;
    let compiled = RecoveryMod::compile(&source)?;
    let root = root(store, session)?;
    let mut s = store.mod_state(&root)?;
    if s.revision != revision || s.pending.is_some() {
        return Err("Mods changed or activation is pending. Refresh and test again.".into());
    }
    if s.events.len() > 26 || s.versions.len() >= 8 {
        return Err("Mod history is full. Baseline retained; use another working folder.".into());
    }
    let id = identity(&manifest, &source);
    if s.versions.iter().any(|v| v.identity == id) {
        return Err("This exact version is already retained, including quarantine. Inspect it; no implicit reactivation.".into());
    }
    let baseline = s
        .active_version()
        .map(check_version)
        .transpose()?
        .unwrap_or(RecoveryMod::compile(BASELINE)?);
    let baseline_results = evaluate(&baseline, cancel)?;
    let results = evaluate(&compiled, cancel)?;
    let mut v = ModVersion { identity: id, manifest, source, baseline: s.active.clone(), results, baseline_results, status: "review".into(), reason: "Fixed recovery-suite-v1; six host-owned categories with equal runtime allowances. Review activation separately.".into(), model };
    if !v.improved() {
        v.status = "rejected".into();
        v.reason = "Candidate failed fixed criteria or did not strictly improve the baseline. No activation; retained for inspection.".into();
    }
    s.events
        .push(format!("{}: {}", v.status, &v.identity[..12]));
    s.versions.push(v);
    if cancel.is_cancelled() {
        return Err("Mod testing stopped; baseline retained.".into());
    }
    store.save_mod_state(&root, revision, &s)
}
pub(super) fn activate(
    store: &dyn SessionStore,
    session: &str,
    revision: u32,
    id: &str,
    cancel: &CancellationToken,
) -> Result<ModState, String> {
    let root = root(store, session)?;
    let mut s = store.mod_state(&root)?;
    if s.revision != revision || s.pending.is_some() {
        return Err("Mods changed. Refresh and retest before activation.".into());
    }
    let v = s
        .versions
        .iter()
        .find(|v| v.identity == id)
        .ok_or("Mod version unavailable.")?;
    if v.status != "review" || !v.improved() || v.baseline != s.active {
        return Err(
            "Mod is stale, unqualified or quarantined. Baseline retained; test a new candidate."
                .into(),
        );
    }
    if evaluate(&check_version(v)?, cancel)? != v.results {
        return Err("Mod health/criteria changed. Baseline retained.".into());
    }
    if s.events.len() > 28 {
        return Err("Mod event history is full; baseline retained.".into());
    }
    s.pending = Some(id.into());
    s.events.push(format!("Activation intent: {}", &id[..12]));
    s = store.save_mod_state(&root, revision, &s)?;
    if cancel.is_cancelled() {
        return Err(
            "Activation interrupted; baseline retained. Restart reconciles the pending intent."
                .into(),
        );
    }
    let old = s.active.replace(id.into());
    s.previous = old.clone();
    s.pending = None;
    for v in &mut s.versions {
        if v.identity == id {
            v.status = "active".into();
        } else if old.as_ref() == Some(&v.identity) {
            v.status = "retired".into();
        }
    }
    s.events.push(format!(
        "Activated {} at a safe boundary; restore remains available.",
        &id[..12]
    ));
    store.save_mod_state(&root, s.revision, &s)
}
pub(super) fn restore(
    store: &dyn SessionStore,
    session: &str,
    revision: u32,
) -> Result<ModState, String> {
    let root = root(store, session)?;
    let mut s = store.mod_state(&root)?;
    if s.revision != revision || s.pending.is_some() {
        return Err(
            "Mods changed or activation is pending. Restart/refresh before restore.".into(),
        );
    }
    if s.events.len() >= 32 {
        return Err("Mod recovery history is full. Baseline retained.".into());
    }
    let active = s.active.take().ok_or("No active mod to restore.")?;
    if let Some(old) = &s.previous {
        let v = s
            .versions
            .iter()
            .find(|v| &v.identity == old)
            .ok_or("Retained baseline missing.")?;
        check_version(v)?;
    }
    s.active = s.previous.take();
    for v in &mut s.versions {
        if v.identity == active {
            v.status = "quarantined".into();
        } else if s.active.as_ref() == Some(&v.identity) {
            v.status = "active".into();
        }
    }
    s.events.push(format!(
        "Restored baseline; quarantined {}. Task history unchanged.",
        &active[..12]
    ));
    store.save_mod_state(&root, revision, &s)
}
pub(super) fn hint(
    version: Option<&ModVersion>,
    category: i32,
    cancel: &CancellationToken,
) -> Result<Value, String> {
    let result = version
        .map(check_version)
        .transpose()?
        .map(|m| m.invoke(category, cancel))
        .transpose()?
        .unwrap_or(0);
    let (action, text) = match result {
        1 => (
            "continue",
            "Use the saved reply’s explicit Continue action. Each segment remains bounded.",
        ),
        2 => (
            "context",
            "Review selected context and model window; compact or reduce input explicitly.",
        ),
        3 => (
            "limits",
            "Review task allowances and retained tool progress before an explicit continuation.",
        ),
        4 => (
            "permissions",
            "Review the denied operation and your current grants. A mod cannot expand access.",
        ),
        5 => (
            "checkpoint",
            "Inspect the saved checkpoint and uncertain effects before preparing a resume draft.",
        ),
        _ => (
            "inspect",
            "Inspect task evidence before retrying. No action has been executed.",
        ),
    };
    Ok(
        json!({"action":action,"text":text,"identity":version.map(|v| &v.identity),"title":version.map(|v| v.manifest.title.as_str()).unwrap_or("Built-in recovery guidance")}),
    )
}
impl Engine {
    pub(super) fn mod_view(&self, session: &str, category: i32) -> Result<Value, String> {
        if !(0..=5).contains(&category) {
            return Err("Unsupported recovery category.".into());
        }
        let root = root(self.store.as_ref(), session)?;
        let mut state = self.store.mod_state(&root)?;
        let card = match hint(state.active_version(), category, &CancellationToken::new()) {
            Ok(card) => card,
            Err(error) => {
                if state.active.is_some() {
                    state = restore(self.store.as_ref(), session, state.revision)?;
                }
                let mut fallback =
                    hint(state.active_version(), category, &CancellationToken::new())?;
                fallback["notice"] = json!(error);
                fallback
            }
        };
        Ok(
            json!({"state":state,"card":card,"boundary":"ABI 1: stateless hints; no file/network/process/credential imports; 10000 fuel; project scope only"}),
        )
    }
    pub(super) fn test_mod(
        &self,
        session: &str,
        revision: u32,
        manifest: ModManifest,
        source: String,
    ) -> Result<Value, String> {
        stage(
            self.store.as_ref(),
            session,
            revision,
            manifest,
            source,
            "local source".into(),
            &CancellationToken::new(),
        )?;
        self.mod_view(session, 1)
    }
    pub(super) fn activate_mod(
        &self,
        session: &str,
        revision: u32,
        identity: &str,
    ) -> Result<Value, String> {
        activate(
            self.store.as_ref(),
            session,
            revision,
            identity,
            &CancellationToken::new(),
        )?;
        self.mod_view(session, 1)
    }
    pub(super) fn restore_mod(&self, session: &str, revision: u32) -> Result<Value, String> {
        restore(self.store.as_ref(), session, revision)?;
        self.mod_view(session, 1)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use dolores_store_sqlite::SqliteStore;
    fn fixture() -> (SqliteStore, tempfile::TempDir) {
        let folder = tempfile::tempdir().unwrap();
        let store = SqliteStore::open(&folder.path().join("state.db")).unwrap();
        store
            .create_workspace_session(
                "test",
                &dolores_core::SessionWorkspace {
                    kind: dolores_core::WorkspaceKind::Project,
                    root: Some(folder.path().to_string_lossy().into_owned()),
                },
            )
            .unwrap();
        (store, folder)
    }
    #[test]
    fn improvement_activation_pinning_restore_and_quarantine() {
        let (store, folder) = fixture();
        let token = CancellationToken::new();
        let s = stage(
            &store,
            "test",
            0,
            ModManifest::default(),
            dolores_mod_runtime::REPAIRED.into(),
            "fixture".into(),
            &token,
        )
        .unwrap();
        let id = s.versions[0].identity.clone();
        let s = activate(&store, "test", s.revision, &id, &token).unwrap();
        let pinned = s.active_version().unwrap().clone();
        assert_eq!(
            hint(Some(&pinned), 1, &token).unwrap()["action"],
            "continue"
        );
        let s = restore(&store, "test", s.revision).unwrap();
        assert!(s.active.is_none());
        assert_eq!(s.versions[0].status, "quarantined");
        assert!(activate(&store, "test", s.revision, &id, &token).is_err());
        assert_eq!(
            hint(Some(&pinned), 1, &token).unwrap()["action"],
            "continue"
        );
        drop(store);
        let reopened = SqliteStore::open(&folder.path().join("state.db")).unwrap();
        assert_eq!(
            reopened.mod_state(folder.path().to_str().unwrap()).unwrap(),
            s
        );
    }
    #[test]
    fn stale_source_bad_candidate_and_interrupted_intent_keep_baseline() {
        let (store, folder) = fixture();
        let root = folder.path().to_str().unwrap();
        let token = CancellationToken::new();
        let s = stage(
            &store,
            "test",
            0,
            ModManifest::default(),
            BASELINE.into(),
            "fixture".into(),
            &token,
        )
        .unwrap();
        assert_eq!(s.versions[0].status, "rejected");
        let mut s = stage(
            &store,
            "test",
            s.revision,
            ModManifest::default(),
            dolores_mod_runtime::REPAIRED.into(),
            "fixture".into(),
            &token,
        )
        .unwrap();
        let id = s.versions[1].identity.clone();
        assert!(activate(&store, "test", 0, &id, &token).is_err());
        s.versions[1].source = BASELINE.into();
        let s = store.save_mod_state(root, s.revision, &s).unwrap();
        assert!(activate(&store, "test", s.revision, &id, &token).is_err());
        let mut s = s;
        s.pending = Some(id);
        let s = store.save_mod_state(root, s.revision, &s).unwrap();
        store.recover_mod_activations().unwrap();
        let recovered = store.mod_state(root).unwrap();
        assert!(recovered.pending.is_none() && recovered.active.is_none());
        assert!(recovered.revision > s.revision);
    }
    #[test]
    fn failed_activation_receipt_keeps_old_pointer_and_recovers() {
        let (store, folder) = fixture();
        let root = folder.path().to_str().unwrap();
        let token = CancellationToken::new();
        let s = stage(
            &store,
            "test",
            0,
            ModManifest::default(),
            dolores_mod_runtime::REPAIRED.into(),
            "fixture".into(),
            &token,
        )
        .unwrap();
        let conn = rusqlite::Connection::open(folder.path().join("state.db")).unwrap();
        conn.execute_batch("CREATE TRIGGER reject_receipt BEFORE UPDATE ON project_mods WHEN json_extract(NEW.data,'$.pending') IS NULL BEGIN SELECT RAISE(ABORT,'fixture'); END;").unwrap();
        assert!(activate(&store, "test", s.revision, &s.versions[0].identity, &token).is_err());
        let interrupted = store.mod_state(root).unwrap();
        assert!(interrupted.active.is_none());
        assert!(interrupted.pending.is_some());
        conn.execute_batch("DROP TRIGGER reject_receipt;").unwrap();
        store.recover_mod_activations().unwrap();
        let recovered = store.mod_state(root).unwrap();
        assert!(recovered.active.is_none() && recovered.pending.is_none());
        let active = activate(
            &store,
            "test",
            recovered.revision,
            &s.versions[0].identity,
            &token,
        )
        .unwrap();
        assert_eq!(active.active.as_ref(), Some(&s.versions[0].identity));
    }
}
