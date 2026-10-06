//! A qualified trial authorizes neither this build nor installation.
use super::{
    harness_repair::{artifact_matches, matching},
    introspection::bundle,
    repair_evaluation,
};
use dolores_core::{RepairWorkspace, SessionStore, ToolCall, ToolPlugin, ToolRequest, ToolSpec};
use dolores_native_update as native;
use serde::Deserialize;
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::{
    collections::HashMap,
    fs,
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
};
use tokio_util::sync::CancellationToken;
const ARGS: &[&str] = &[
    "build",
    "--offline",
    "--locked",
    "--release",
    "-p",
    "dolores-flutter-bridge",
];

// Keep account authority, persistence, schema, updater and evaluators out of repair scope.
pub(super) fn installable(state: &RepairWorkspace) -> Result<(), String> {
    repair_evaluation::eligible(state)?;
    for f in state.files.iter().filter(|f| f.before != f.after) {
        let allowed = f.path.starts_with("crates/dolores-provider-openai/src/")
            || matches!(
                f.path.as_str(),
                "crates/dolores-core/src/command_outcome.rs"
                    | "crates/dolores-core/src/task_budget.rs"
            );
        if !allowed {
            return Err("This first native path accepts provider implementation, command outcomes and task-budget repairs only. Approval, credentials, storage, host policy, evaluator, launcher, manifests and UI changes need a separate reviewed workflow. The proposal remains.".into());
        }
    }
    Ok(())
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn compact_compiler_intent_never_reuses_an_interrupted_directory() {
        let p = tempfile::tempdir().unwrap();
        let id = uuid::Uuid::new_v4().to_string();
        create_targets(p.path(), &id).unwrap();
        let root = target_dir(p.path(), &id, "c").unwrap();
        assert_eq!(root.parent().unwrap().file_name().unwrap(), &id[..8]);
        assert!(create_targets(p.path(), &id)
            .unwrap_err()
            .contains("fresh review"));
        assert_eq!(
            native::json::<String>(&root.parent().unwrap().join("owner.json")).unwrap(),
            id
        );
    }
    #[test]
    fn installation_scope_refuses_policy_changes_even_with_unchanged_tests() {
        let s = bundle::SOURCES
            .iter()
            .find(|s| s.path == "crates/dolores-core/src/permissions.rs")
            .unwrap();
        let before = bundle::text(s).unwrap();
        let after = format!("// scope refusal fixture\n{before}");
        let state = RepairWorkspace {
            id: uuid::Uuid::new_v4().to_string(),
            session: uuid::Uuid::new_v4().to_string(),
            revision: 1,
            bundle_id: bundle::ID.into(),
            build: env!("DOLORES_BUILD_REVISION").into(),
            artifact: "repair-fixture".into(),
            status: "proposed".into(),
            files: vec![dolores_core::harness_repair::RepairFile {
                path: s.path.into(),
                source_id: s.id.into(),
                candidate_id: super::super::harness_repair::identity(&after),
                before,
                after,
            }],
        };
        assert!(installable(&state)
            .unwrap_err()
            .contains("separate reviewed workflow"));
    }
    #[test]
    fn candidate_identity_pins_every_member_in_the_source_bundle() {
        let state = RepairWorkspace {
            id: uuid::Uuid::new_v4().to_string(),
            session: uuid::Uuid::new_v4().to_string(),
            revision: 1,
            bundle_id: bundle::ID.into(),
            build: env!("DOLORES_BUILD_REVISION").into(),
            artifact: "fixture".into(),
            status: "prepared".into(),
            files: vec![],
        };
        assert_eq!(candidate_id(&state), bundle::ID);
    }
}
pub(super) fn candidate_id(state: &RepairWorkspace) -> String {
    let mut h = Sha256::new();
    for s in bundle::SOURCES {
        let id = state
            .files
            .iter()
            .find(|f| f.path == s.path)
            .map(|f| f.candidate_id.as_str())
            .unwrap_or(s.id);
        h.update(s.path.as_bytes());
        h.update([0]);
        h.update(id.as_bytes());
        h.update([0]);
    }
    format!("sha256:{:x}", h.finalize())
}
pub(super) fn qualified(
    store: &dyn SessionStore,
    state: &RepairWorkspace,
    id: &str,
) -> Result<dolores_core::RepairEvaluation, String> {
    installable(state)?;
    let e = store
        .repair_evaluations(&state.session, &state.id)?
        .into_iter()
        .find(|e| e.id == id)
        .ok_or("Choose this repair's retained qualified trial.")?;
    e.validate()?;
    if e.status != "qualified"
        || !e.improved()
        || e.revision != state.revision
        || e.bundle_id != state.bundle_id
        || e.candidate_ids
            != state
                .files
                .iter()
                .map(|f| f.candidate_id.clone())
                .collect::<Vec<_>>()
    {
        return Err("Trial is unqualified or no longer matches the candidate. Retain it and request a fresh bounded trial; no build ran.".into());
    }
    Ok(e)
}
pub(super) fn build_root(profile: &Path, id: &str) -> Result<PathBuf, String> {
    if !native::id_valid(id) {
        return Err("Invalid retained build ID.".into());
    }
    Ok(profile.join("repairs").join(format!("build-{id}")))
}
pub(super) fn load(profile: &Path, id: &str) -> Result<native::Build, String> {
    let b: native::Build = native::json(&build_root(profile, id)?.join("build.json"))?;
    b.validate()?;
    if b.id != id {
        return Err("Native build identity changed.".into());
    }
    Ok(b)
}
pub(super) fn summary(b: &native::Build) -> Value {
    json!({"id":b.id,"session":b.session,"repairId":b.repair_id,"revision":b.revision,"evaluationId":b.evaluation_id,"sourceId":b.source_id,"candidateSourceId":b.candidate_source_id,"cargoId":b.cargo_id,"status":b.status,"note":b.note,"schema":b.schema,"previousId":b.previous.as_ref().map(|b|&b.id),"candidateId":b.candidate.as_ref().map(|b|&b.id)})
}

pub(super) fn bundle_directory() -> Result<PathBuf, String> {
    #[cfg(windows)]
    {
        use windows_sys::Win32::System::LibraryLoader::*;
        let mut module = std::ptr::null_mut();
        let mut path = vec![0u16; 32768];
        // Identify this loaded DLL, also when the fixed C ABI qualification driver is the host.
        unsafe {
            if GetModuleHandleExW(
                GET_MODULE_HANDLE_EX_FLAG_FROM_ADDRESS
                    | GET_MODULE_HANDLE_EX_FLAG_UNCHANGED_REFCOUNT,
                super::dolores_call as *const () as *const u16,
                &mut module,
            ) == 0
            {
                return Err("Loaded normal bridge identity unavailable.".into());
            }
            let n = GetModuleFileNameW(module, path.as_mut_ptr(), path.len() as u32);
            if n == 0 || n as usize >= path.len() {
                return Err("Loaded bridge path unavailable.".into());
            }
            use std::os::windows::ffi::OsStringExt;
            let p = PathBuf::from(std::ffi::OsString::from_wide(&path[..n as usize]));
            if p.file_name().is_none_or(|n| n != native::LIBRARY) {
                return Err("Build requires the packaged normal Windows bridge.".into());
            }
            Ok(p.parent().unwrap().to_path_buf())
        }
    }
    #[cfg(not(windows))]
    {
        Err("Native installation is currently qualified for Windows Rust bundles only.".into())
    }
}
pub(super) fn target_dir(profile: &Path, id: &str, phase: &str) -> Result<PathBuf, String> {
    if !native::id_valid(id) || !matches!(phase, "b" | "c" | "r") {
        return Err("Invalid native compiler ownership.".into());
    }
    let p = profile.join("nt").join(&id[..8]).join(phase);
    if cfg!(windows) && p.to_string_lossy().encode_utf16().count() > 150 {
        return Err("Profile path is too long for the Windows linker. Native work remains; use a separately reviewed shorter profile location before another build or trial.".into());
    }
    Ok(p)
}
pub(super) fn create_targets(profile: &Path, id: &str) -> Result<(), String> {
    let target = target_dir(profile, id, "b")?;
    let nt = profile.join("nt");
    fs::create_dir_all(&nt).map_err(|_| "Native compiler storage unavailable.")?;
    native::safe_path(&nt)?;
    let root = target.parent().unwrap();
    fs::create_dir(root).map_err(|_|"Native compiler intent already exists or is unavailable. Retain it and request a fresh review; no retry ran.")?;
    native::safe_path(root)?;
    native::save(&root.join("owner.json"), &id)?;
    for phase in ["b", "c", "r"] {
        fs::create_dir(root.join(phase))
            .map_err(|_| "Native compiler phase could not be retained.")?;
        native::safe_path(&root.join(phase))?;
    }
    Ok(())
}
fn build_call(id: &str, target: &Path) -> ToolCall {
    ToolCall {
        id: id.into(),
        name: "run_command".into(),
        arguments:
            json!({"program":"cargo","args":ARGS.iter().map(|s|s.to_string()).chain(["--target-dir".into(),target.to_string_lossy().into_owned()]).collect::<Vec<_>>(),"timeout_seconds":300,"capture_bytes":262144})
                .to_string(),
    }
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Query {
    repair_id: String,
    revision: u32,
    evaluation_id: String,
}
struct Plan {
    request: ToolRequest,
    state: RepairWorkspace,
    build: native::Build,
    command: dolores_core::CommandPreview,
    bundle_root: PathBuf,
    target: PathBuf,
}
pub(super) struct BuildTool {
    store: Arc<dyn SessionStore>,
    session: String,
    profile: PathBuf,
    plans: Mutex<HashMap<String, Plan>>,
}
pub(super) fn spec() -> ToolSpec {
    ToolSpec{name:"build_harness_repair".into(),description:"Separately review one Windows release DLL build for exact repairId/revision/evaluationId. Requires a complete qualified frozen native trial matching this candidate. Only provider, command-outcome or task-budget Rust implementation changes; existing tests unchanged. Fixed installed Cargo, offline/locked, 300 seconds and 256 KiB capture. Native compilation has account permissions, NOT OS containment. Full access grants no build authority. Retain the complete current bundle and change only its DLL in a separate owned directory. No install/restart/task replay. User separately reviews installation or Restore in Native repairs.".into(),parameters:json!({"type":"object","properties":{"repairId":{"type":"string"},"revision":{"type":"integer","minimum":1},"evaluationId":{"type":"string"}},"required":["repairId","revision","evaluationId"],"additionalProperties":false})}
}
impl BuildTool {
    pub fn new(store: Arc<dyn SessionStore>, session: String, profile: PathBuf) -> Self {
        Self {
            store,
            session,
            profile,
            plans: Mutex::new(HashMap::new()),
        }
    }
    fn current(
        &self,
        id: &str,
        revision: u32,
        evaluation: &str,
    ) -> Result<RepairWorkspace, String> {
        if !native::id_valid(id) {
            return Err("Choose this chat's retained repair ID.".into());
        }
        let s = self.store.repair_workspace(&self.session, id)?;
        if s.revision != revision {
            return Err("Repair changed since build review. Retained work remains; review the current revision.".into());
        }
        matching(&s)?;
        artifact_matches(&self.profile.join("repairs"), &s)?;
        qualified(self.store.as_ref(), &s, evaluation)?;
        Ok(s)
    }
}
#[async_trait::async_trait]
impl ToolPlugin for BuildTool {
    fn spec(&self) -> ToolSpec {
        spec()
    }
    fn prepare(&self, call: &ToolCall) -> Result<ToolRequest, String> {
        if call.name != "build_harness_repair" || call.arguments.len() > 4096 {
            return Err("Use only exact repairId, revision and evaluationId.".into());
        }
        let q: Query = serde_json::from_str(&call.arguments)
            .map_err(|_| "Use exact repairId, revision and evaluationId.")?;
        let state = self.current(&q.repair_id, q.revision, &q.evaluation_id)?;
        let eval = qualified(self.store.as_ref(), &state, &q.evaluation_id)?;
        let directory = self.profile.join("repairs");
        native::safe_path(&directory)?;
        let count = fs::read_dir(&directory)
            .map_err(|_| "Retained repairs unavailable.")?
            .filter_map(Result::ok)
            .filter(|e| e.file_name().to_string_lossy().starts_with("build-"))
            .filter_map(|e| {
                load(
                    &self.profile,
                    e.file_name().to_string_lossy().trim_start_matches("build-"),
                )
                .ok()
            })
            .filter(|b| b.repair_id == state.id)
            .count();
        if count >= 4 {
            return Err("Four native builds are retained for this repair. Inspect them or prepare a separate repair; no retry ran.".into());
        }
        let id = uuid::Uuid::new_v4().to_string();
        let target = target_dir(&self.profile, &id, "r")?;
        let command = dolores_tools_command::RunCommand::new(&directory)?
            .prepare(&build_call("build-probe", &target))?
            .command
            .unwrap();
        let cargo = repair_evaluation::cargo_identity(&command)?;
        if cargo != eval.cargo_id {
            return Err(
                "Installed Cargo changed since qualification. Request a fresh trial; no build ran."
                    .into(),
            );
        }
        let bundle_root = bundle_directory()?;
        let previous = native::manifest(&bundle_root)?;
        if ![native::APP, native::LIBRARY, native::LAUNCHER]
            .iter()
            .all(|n| previous.files.iter().any(|f| &f.path == n))
        {
            return Err("Normal application bundle is incomplete. Rebuild the maintained desktop before native repair.".into());
        }
        let build = native::Build {
            id,
            session: self.session.clone(),
            repair_id: state.id.clone(),
            revision: state.revision,
            source_id: state.bundle_id.clone(),
            candidate_source_id: candidate_id(&state),
            evaluation_id: eval.id,
            cargo_id: cargo,
            status: "started".into(),
            note: "Reviewed build only; installation needs a separate user review.".into(),
            previous: Some(previous),
            candidate: None,
            schema: dolores_store_sqlite::SCHEMA_VERSION,
        };
        build.validate()?;
        let diff = state
            .files
            .iter()
            .filter(|f| f.before != f.after)
            .map(|f| {
                format!(
                    "{}\n{}",
                    f.path,
                    dolores_tools_fs::change_diff(&f.before, &f.after)
                )
            })
            .collect::<Vec<_>>()
            .join("\n");
        if diff.len() > 8192 {
            return Err(
                "Build review exceeds 8 KiB. Prepare a smaller repair; retained proposal remains."
                    .into(),
            );
        }
        let request=ToolRequest{call_id:call.id.clone(),name:call.name.clone(),target:"Dolores native build".into(),query:Some(json!({"build":summary(&build),"command":command,"authority":"account permissions; no OS containment or installation","bundleId":build.previous.as_ref().unwrap().id}).to_string()),diff:Some(diff),command:None,mcp:None};
        let mut plans = self.plans.lock().map_err(|_| "Build review unavailable.")?;
        if plans.len() >= dolores_core::MAX_TOOL_CALLS || plans.contains_key(&call.id) {
            return Err(
                "Build review allowance reached; retain work and request a fresh review.".into(),
            );
        }
        plans.insert(
            call.id.clone(),
            Plan {
                request: request.clone(),
                state,
                build,
                command,
                bundle_root,
                target,
            },
        );
        Ok(request)
    }
    fn discard(&self, request: &ToolRequest) {
        if let Ok(mut plans) = self.plans.lock() {
            plans.remove(&request.call_id);
        }
    }
    async fn invoke(
        &self,
        request: &ToolRequest,
        cancel: CancellationToken,
    ) -> Result<String, String> {
        let plan = self
            .plans
            .lock()
            .map_err(|_| "Build review unavailable.")?
            .remove(&request.call_id)
            .ok_or("Build review expired; request a fresh review.")?;
        if &plan.request != request || cancel.is_cancelled() {
            return Err(
                "Build stopped or review changed. Nothing compiled; proposal remains.".into(),
            );
        }
        if self.current(
            &plan.state.id,
            plan.state.revision,
            &plan.build.evaluation_id,
        )? != plan.state
        {
            return Err("Candidate changed; no build ran.".into());
        }
        let root = build_root(&self.profile, &plan.build.id)?;
        fs::create_dir(&root)
            .map_err(|_| "Separate build storage unavailable; proposal remains.")?;
        native::safe_path(&root)?;
        let receipt = root.join("build.json");
        let mut build = plan.build;
        native::save(&receipt, &build)?;
        let result=async{
            create_targets(&self.profile,&build.id)?;
            native::verify(&plan.bundle_root,build.previous.as_ref().unwrap())?;
            let state=plan.state.clone();let staging=root.clone();let token=cancel.clone();let from=plan.bundle_root.clone();let previous=build.previous.clone().unwrap();
            tokio::task::spawn_blocking(move||{repair_evaluation::materialize(&staging,&state,&token)?;native::copy_bundle(&from,&staging.join("previous"),&previous)?;native::copy_bundle(&staging.join("previous"),&staging.join("bundle"),&previous)}).await.map_err(|_|"Build staging worker failed.")??;
            repair_evaluation::verify_sources(&root,&plan.state)?;
            let command=dolores_tools_command::RunCommand::new(&root.join("candidate"))?;let prepared=command.prepare(&build_call("release-build",&plan.target))?;
            if prepared.command.as_ref()!=Some(&plan.command)||repair_evaluation::cargo_identity(&plan.command)?!=build.cargo_id{return Err("Installed Cargo changed during build review; no command ran.".into());}
            let result=command.invoke(&prepared,cancel.clone()).await?;
            native::save(&root.join("command.json"),&serde_json::from_str::<Value>(&result).map_err(|_|"Build returned invalid evidence.")?)?;
            let value:Value=serde_json::from_str(&result).map_err(|_|"Build evidence invalid.")?;
            if cancel.is_cancelled(){return Err("Build stopped. Source and logs remain; fresh build review required.".into());}
            if value["reason"]!="completed"||value["exitCode"]!=0||value["truncated"]!=false||value["outputError"]!=false||value["lossyUtf8"]!=false{return Err("Release build did not complete within 300 seconds / 256 KiB or returned a compiler failure. Source, proposal and bounded logs remain. Inspect the build before requesting a fresh review; no installation occurred.".into());}
            repair_evaluation::verify_sources(&root,&plan.state)?;self.current(&plan.state.id,plan.state.revision,&build.evaluation_id)?;
            if repair_evaluation::cargo_identity(&plan.command)?!=build.cargo_id{return Err("Cargo changed during build. Installation withheld.".into());}
            native::verify(&root.join("previous"),build.previous.as_ref().unwrap())?;
            let bytes=native::read(&plan.target.join("release").join(native::LIBRARY),native::MAX_MEMBER)?;
            native::write_library(&root.join("bundle"),&bytes)?;
            build.candidate=Some(native::manifest(&root.join("bundle"))?);build.status="ready".into();build.note="Release DLL built from the qualified source. Installation and Restore each require a separate review; no restart or task replay occurred.".into();build.validate()?;Ok::<(),String>(())
        }.await;
        if let Err(error) = result {
            build.status = if cancel.is_cancelled() {
                "stopped"
            } else {
                "failed"
            }
            .into();
            build.note = error;
        }
        native::save(&receipt, &build)?;
        Ok(json!({"build":summary(&build)}).to_string())
    }
}
