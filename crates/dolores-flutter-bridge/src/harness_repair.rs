//! Managed native patch proposals. This module never compiles, runs or installs them.
use super::*;
use dolores_core::{RepairFile, RepairWorkspace, ToolCall, ToolPlugin, ToolRequest, ToolSpec};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::io::{Read, Write};

fn identity(text: &str) -> String {
    format!("sha256:{:x}", Sha256::digest(text.as_bytes()))
}
fn is_alias(metadata: &std::fs::Metadata) -> bool {
    #[cfg(windows)]
    {
        use std::os::windows::fs::MetadataExt;
        if metadata.file_attributes() & 0x400 != 0 {
            return true;
        }
    }
    metadata.file_type().is_symlink()
}
fn checkpoint(cancel: &CancellationToken) -> Result<(), String> {
    if cancel.is_cancelled() {
        Err("Repair preparation stopped. Retained snapshots and the original task remain; nothing was installed.".into())
    } else {
        Ok(())
    }
}
fn check(state: &RepairWorkspace) -> Result<(), String> {
    state.validate()?;
    if state
        .files
        .iter()
        .any(|f| identity(&f.before) != f.source_id || identity(&f.after) != f.candidate_id)
    {
        return Err("Repair snapshot identity changed. Keep the evidence; do not test or install this candidate.".into());
    }
    Ok(())
}
fn matching(state: &RepairWorkspace) -> Result<(), String> {
    check(state)?;
    if state.bundle_id != introspection::bundle::ID {
        return Err("Running source changed. Keep this repair and its diff; prepare a new repair from the current bundle before changing it.".into());
    }
    if state
        .files
        .iter()
        .any(|f| !introspection::bundle::find(&f.path).is_ok_and(|s| s.id == f.source_id))
    {
        return Err("Repair baseline differs from the pinned source bundle. Keep the evidence and prepare a fresh matching repair.".into());
    }
    Ok(())
}
fn file(source: &str, source_id: &str, bundle_id: &str) -> Result<RepairFile, String> {
    if bundle_id != introspection::bundle::ID {
        return Err("Source bundle changed. Inspect the current bundle before preparing a repair; original task remains.".into());
    }
    let entry = introspection::resolve(source)?;
    if source_id != entry.id {
        return Err("Source changed. Read its current identity and prepare a fresh repair; original task remains.".into());
    }
    let text = introspection::bundle::text(entry)?;
    Ok(RepairFile {
        path: entry.path.into(),
        source_id: entry.id.into(),
        candidate_id: entry.id.into(),
        before: text.clone(),
        after: text,
    })
}
fn protected(path: &str) -> bool {
    path.ends_with("Cargo.toml")
        || path.ends_with("Cargo.lock")
        || path.ends_with("build.rs")
        || path.contains("test")
        || path.starts_with("scripts/")
        || [
            "credentials",
            "permissions",
            "registry",
            "harness_repair",
            "run_journal",
            "task_budget",
        ]
        .iter()
        .any(|name| path.contains(name))
}
fn summary(state: &RepairWorkspace) -> Value {
    json!({"repairId":state.id,"revision":state.revision,"status":state.status,"bundleId":state.bundle_id,
        "build":state.build,"sourceMatch":if state.bundle_id==introspection::bundle::ID && state.files.iter().all(|f|introspection::bundle::find(&f.path).is_ok_and(|s|s.id==f.source_id)){"matches"}else{"mismatch"},
        "files":state.files.iter().map(|f| json!({"path":f.path,"sourceId":f.source_id,"candidateId":f.candidate_id,"changed":f.before!=f.after,"protectedReview":protected(&f.path)})).collect::<Vec<_>>(),
        "limits":{"sourceFiles":8,"snapshotBytes":2*1024*1024,"revisions":16,"workspacesPerChat":4},
        "note":"Managed matching-source snapshots only. Nothing compiled, executed or installed. Tests and native installation require separate review; the user's project and failed task were not replayed."})
}
fn artifact_receipt(state: &RepairWorkspace) -> Value {
    json!({"repairId":state.id,"session":state.session,"revision":state.revision,"bundleId":state.bundle_id,"build":state.build,"artifact":state.artifact,"status":state.status,
        "files":state.files.iter().map(|f|json!({"path":f.path,"sourceId":f.source_id,"candidateId":f.candidate_id})).collect::<Vec<_>>()})
}
fn direct_file(root: &std::path::Path, path: &str) -> Result<String, String> {
    let mut candidate = root.to_path_buf();
    for part in path.split('/') {
        candidate.push(part);
        if is_alias(
            &std::fs::symlink_metadata(&candidate)
                .map_err(|_| "Repair artifact is unavailable; database snapshots remain.")?,
        ) {
            return Err("Repair artifact contains a file alias. Retained snapshots remain; no proposal ran.".into());
        }
    }
    let file = std::fs::File::open(candidate).map_err(|_| "Repair file is unavailable.")?;
    if !file
        .metadata()
        .is_ok_and(|m| m.is_file() && m.len() <= dolores_core::MAX_FILE_SNAPSHOT_BYTES as u64)
    {
        return Err("Repair artifact is not bounded text.".into());
    }
    let mut text = String::new();
    file.take(dolores_core::MAX_FILE_SNAPSHOT_BYTES as u64 + 1)
        .read_to_string(&mut text)
        .map_err(|_| "Repair file could not be read.")?;
    if text.len() > dolores_core::MAX_FILE_SNAPSHOT_BYTES {
        return Err("Repair file exceeds its bound.".into());
    }
    Ok(text)
}
fn artifact_matches(root: &std::path::Path, state: &RepairWorkspace) -> Result<(), String> {
    check(state)?;
    if is_alias(
        &std::fs::symlink_metadata(root)
            .map_err(|_| "Repair storage is unavailable; retained snapshots remain.")?,
    ) {
        return Err("Repair storage alias refused.".into());
    }
    let directory = root.join(&state.artifact);
    if is_alias(
        &std::fs::symlink_metadata(&directory)
            .map_err(|_| "Repair artifact is missing; inspect its retained database snapshots.")?,
    ) {
        return Err("Repair artifact alias refused.".into());
    }
    let manifest = direct_file(&directory, "manifest.json")?;
    let saved: Value =
        serde_json::from_str(&manifest).map_err(|_| "Repair artifact manifest changed.")?;
    if saved != artifact_receipt(state) {
        return Err("Repair artifact manifest changed; retained database snapshots remain.".into());
    }
    for f in &state.files {
        if direct_file(&directory, &format!("baseline/{}", f.path))? != f.before
            || direct_file(&directory, &format!("candidate/{}", f.path))? != f.after
        {
            return Err("Repair artifact changed. Inspect retained snapshots and prepare a separate repair; nothing was installed.".into());
        }
    }
    Ok(())
}
fn publish(
    root: &std::path::Path,
    store: &dyn SessionStore,
    mut state: RepairWorkspace,
    expected: Option<u32>,
    cancel: &CancellationToken,
) -> Result<RepairWorkspace, String> {
    check(&state)?;
    checkpoint(cancel)?;
    std::fs::create_dir_all(root)
        .map_err(|_| "Repair storage could not be prepared. Original task remains.")?;
    if is_alias(&std::fs::symlink_metadata(root).map_err(|_| "Repair storage is unavailable.")?) {
        return Err("Repair storage alias refused.".into());
    }
    let root = root
        .canonicalize()
        .map_err(|_| "Repair storage is unavailable.")?;
    let directory = tempfile::Builder::new()
        .prefix("repair-")
        .tempdir_in(&root)
        .map_err(|_| "Repair artifact could not be staged.")?;
    state.artifact = directory
        .path()
        .file_name()
        .unwrap()
        .to_str()
        .ok_or("Repair artifact name unavailable.")?
        .into();
    let mut receipt = state.clone();
    receipt.revision = state
        .revision
        .checked_add(1)
        .ok_or("Repair revision exhausted.")?;
    for f in &state.files {
        for (variant, text) in [("baseline", &f.before), ("candidate", &f.after)] {
            checkpoint(cancel)?;
            let path = directory.path().join(variant).join(&f.path);
            std::fs::create_dir_all(path.parent().unwrap())
                .map_err(|_| "Repair artifact could not be staged.")?;
            let mut output = std::fs::OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(path)
                .map_err(|_| "Repair artifact could not be staged.")?;
            output
                .write_all(text.as_bytes())
                .and_then(|_| output.sync_all())
                .map_err(|_| "Repair artifact could not be saved.")?;
        }
    }
    let bytes = serde_json::to_vec(&artifact_receipt(&receipt))
        .map_err(|_| "Repair receipt unavailable.")?;
    let mut manifest = std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(directory.path().join("manifest.json"))
        .map_err(|_| "Repair receipt could not be staged.")?;
    manifest
        .write_all(&bytes)
        .and_then(|_| manifest.sync_all())
        .map_err(|_| "Repair receipt could not be saved.")?;
    drop(manifest);
    checkpoint(cancel)?;
    let saved = store.save_repair_workspace(&state, expected)?;
    let _ = directory.keep(); // Immutable version retained only after the authoritative transaction succeeds.
    Ok(saved)
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Query {
    action: String,
    repair_id: Option<String>,
    revision: Option<u32>,
    bundle_id: Option<String>,
    source: Option<String>,
    source_id: Option<String>,
    old_text: Option<String>,
    new_text: Option<String>,
    #[serde(default = "one")]
    start_line: usize,
    #[serde(default = "lines")]
    line_count: usize,
}
fn one() -> usize {
    1
}
fn lines() -> usize {
    180
}
pub(super) fn spec() -> ToolSpec {
    ToolSpec {name:"harness_repair".into(),description:"Prepare a reviewed native repair proposal in Dolores-owned storage, without changing the project or replaying its task. action=prepare requires source, exact sourceId and bundleId copied from inspect_harness. action=add includes repairId and current revision plus another exact source identity. action=propose requires repairId, revision, source, sourceId (current candidateId), oldText and newText: one unique exact replacement, each at most 4096 UTF-8 bytes. action=inspect reads a saved repair (optional source/startLine/lineCount); action=list lists this chat's retained repairs. Review the exact candidate diff. No compile, test execution, install or update authority. Stop/stale source keeps existing work; native build and execution need separate review.".into(),
        parameters:json!({"type":"object","properties":{"action":{"type":"string","enum":["prepare","add","propose","inspect","list"]},"repairId":{"type":"string"},"revision":{"type":"integer","minimum":1},"bundleId":{"type":"string"},"source":{"type":"string"},"sourceId":{"type":"string"},"oldText":{"type":"string","maxLength":4096},"newText":{"type":"string","maxLength":4096},"startLine":{"type":"integer","minimum":1},"lineCount":{"type":"integer","minimum":1,"maximum":2048}},"required":["action"],"additionalProperties":false}) }
}
struct Plan {
    request: ToolRequest,
    query: Query,
    next: Option<RepairWorkspace>,
    expected: Option<u32>,
}
pub(super) struct RepairTool {
    pub store: Arc<dyn SessionStore>,
    pub session: String,
    pub directory: PathBuf,
    plans: Mutex<HashMap<String, Plan>>,
}
impl RepairTool {
    pub fn new(store: Arc<dyn SessionStore>, session: String, directory: PathBuf) -> Self {
        Self {
            store,
            session,
            directory,
            plans: Mutex::new(HashMap::new()),
        }
    }
    fn state(&self, q: &Query) -> Result<RepairWorkspace, String> {
        let id = q
            .repair_id
            .as_deref()
            .ok_or("Choose a repairId from this chat's retained repairs.")?;
        if !dolores_core::harness_repair::valid_repair_id(id) {
            return Err("Invalid repair identity; no project path is accepted.".into());
        }
        let state = self.store.repair_workspace(&self.session, id)?;
        check(&state)?;
        Ok(state)
    }
    fn view(&self, q: &Query) -> Result<Value, String> {
        if q.action == "list" {
            return Ok(
                json!({"repairIds":self.store.repair_ids(&self.session)?,"note":"Read-only list; no task replay or installation."}),
            );
        }
        let state = self.state(q)?;
        let mut view = summary(&state);
        view["artifactIntegrity"] = json!(artifact_matches(&self.directory, &state)
            .err()
            .unwrap_or_else(|| "matches".into()));
        if let Some(source) = &q.source {
            let path = introspection::resolve(source)
                .map(|s| s.path)
                .unwrap_or(source);
            let f = state
                .files
                .iter()
                .find(|f| f.path == path)
                .ok_or("Source was not captured in this repair. Existing snapshots remain.")?;
            let mut text = String::new();
            let mut next = q.start_line;
            for (index, line) in f
                .after
                .lines()
                .enumerate()
                .skip(q.start_line - 1)
                .take(q.line_count)
            {
                let item = format!("{}\t{line}\n", index + 1);
                if serde_json::to_string(&text).unwrap().len()
                    + serde_json::to_string(&item).unwrap().len()
                    > 8192
                {
                    break;
                }
                text.push_str(&item);
                next = index + 2;
            }
            if text.is_empty() {
                return Err(
                    "Repair range unavailable. Choose an earlier line; snapshots remain.".into(),
                );
            }
            view["source"] = json!({"path":f.path,"sourceId":f.source_id,"candidateId":f.candidate_id,"text":text,"nextLine":next,"hasMore":next<=f.after.lines().count()});
            let diff = dolores_tools_fs::change_diff(&f.before, &f.after);
            if diff.len() <= 4096 {
                view["diff"] = json!(diff);
            } else {
                view["diffNotice"]=json!("Full diff retained locally. Inspect smaller source ranges; no test or installation is qualified.");
            }
        }
        if view.to_string().len() > dolores_core::MAX_TOOL_BYTES {
            return Err("Repair view exceeds the current tool allowance. Inspect a smaller lineCount; snapshots remain.".into());
        }
        Ok(view)
    }
}
#[async_trait::async_trait]
impl ToolPlugin for RepairTool {
    fn spec(&self) -> ToolSpec {
        spec()
    }
    fn prepare(&self, call: &ToolCall) -> Result<ToolRequest, String> {
        if call.name != "harness_repair" || call.arguments.len() > 16384 {
            return Err("Invalid repair arguments; use a smaller explicit proposal.".into());
        }
        let q: Query = serde_json::from_str(&call.arguments)
            .map_err(|_| "Invalid repair arguments. Use only fields needed by the action.")?;
        if q.start_line == 0 || !(1..=2048).contains(&q.line_count) {
            return Err("Use a positive startLine and 1–2048 lines.".into());
        }
        let mut expected = None;
        let mut diff = None;
        let next=match q.action.as_str(){
            "prepare"=>{
                if q.repair_id.is_some() || q.revision.is_some() || q.old_text.is_some() || q.new_text.is_some(){return Err("Prepare uses source and copied sourceId/bundleId only.".into());}
                if self.store.repair_ids(&self.session)?.len()>=4{return Err("Repair workspace allowance reached. Retained repairs remain; inspect them or explicitly start another chat.".into());}
                let f=file(q.source.as_deref().ok_or("Choose a source from inspect_harness.")?,q.source_id.as_deref().ok_or("Copy the sourceId from inspection.")?,q.bundle_id.as_deref().ok_or("Copy the bundleId from inspection.")?)?;
                Some(RepairWorkspace{id:uuid::Uuid::new_v4().to_string(),session:self.session.clone(),revision:0,bundle_id:introspection::bundle::ID.into(),build:env!("DOLORES_BUILD_REVISION").into(),artifact:"repair-pending".into(),status:"prepared".into(),files:vec![f]})
            }
            "add"|"propose"=>{
                let mut state=self.state(&q)?;matching(&state)?;artifact_matches(&self.directory,&state)?;
                if Some(state.revision)!=q.revision{return Err("Repair changed. Inspect its revision and prepare a fresh proposal; old work remains.".into());}
                if state.revision>=16{return Err("Repair proposal allowance reached. Snapshots remain; review this candidate or explicitly prepare a separate repair.".into());}
                expected=Some(state.revision);
                let source=q.source.as_deref().ok_or("Choose a captured source.")?;
                if q.action=="add" {
                    if q.old_text.is_some() || q.new_text.is_some(){return Err("Add captures source; it does not edit text.".into());}
                    let f=file(source,q.source_id.as_deref().ok_or("Copy the sourceId.")?,q.bundle_id.as_deref().ok_or("Copy the bundleId.")?)?;
                    if state.files.iter().any(|old|old.path==f.path){return Err("Source already captured. Inspect it before proposing a patch.".into());}
                    state.files.push(f);
                }else{
                    if q.bundle_id.as_deref().is_some_and(|id|id!=state.bundle_id){return Err("Repair bundle changed; retained candidate remains.".into());}
                    let path=introspection::resolve(source)?.path;
                    let f=state.files.iter_mut().find(|f|f.path==path).ok_or("Source was not captured. Add matching source before editing; no project file is accepted.")?;
                    if q.source_id.as_deref()!=Some(&f.candidate_id){return Err("Candidate changed. Inspect its candidateId and prepare a fresh patch; retained work remains.".into());}
                    let old=q.old_text.as_deref().ok_or("Propose requires oldText.")?;let new=q.new_text.as_deref().ok_or("Propose requires newText.")?;
                    if old.is_empty() || old==new || old.len()>4096 || new.len()>4096 || new.contains('\0') || f.after.match_indices(old).count()!=1{return Err("Use one unique exact oldText and different newText, each within 4096 bytes. Read the retained candidate and prepare a smaller patch.".into());}
                    f.after=f.after.replacen(old,new,1);f.candidate_id=identity(&f.after);
                    let preview=dolores_tools_fs::change_diff(&f.before,&f.after);
                    if preview.len()>8192{return Err("Candidate diff exceeds 8 KiB. Retained patch remains; prepare a smaller separate repair.".into());}
                    diff=Some(preview);state.status="proposed".into();
                }
                check(&state)?;Some(state)
            }
            "inspect"|"list"=>{self.view(&q)?;None}
            _=>return Err("Use prepare, add, propose, inspect or list; no execution/installation action exists.".into()),
        };
        let repair_id = next
            .as_ref()
            .map(|s| s.id.as_str())
            .or(q.repair_id.as_deref());
        let request=ToolRequest{call_id:call.id.clone(),name:call.name.clone(),target:"Dolores managed repair".into(),query:Some(json!({"action":q.action,"repairId":repair_id,"source":q.source,"revision":expected,"execution":"none; native proposal only"}).to_string()),diff,command:None,mcp:None};
        let mut plans = self
            .plans
            .lock()
            .map_err(|_| "Repair review unavailable.")?;
        if plans.len() >= dolores_core::MAX_TOOL_CALLS || plans.contains_key(&call.id) {
            return Err("Repair review allowance reached. Retained work remains.".into());
        }
        plans.insert(
            call.id.clone(),
            Plan {
                request: request.clone(),
                query: q,
                next,
                expected,
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
            .map_err(|_| "Repair review unavailable.")?
            .remove(&request.call_id)
            .ok_or("Repair review expired. Prepare a fresh proposal; retained work remains.")?;
        if &plan.request != request {
            return Err("Repair review changed. No candidate was saved.".into());
        }
        checkpoint(&cancel)?;
        let result = if let Some(state) = plan.next {
            matching(&state)?;
            if let Some(revision) = plan.expected {
                let old = self.store.repair_workspace(&self.session, &state.id)?;
                if old.revision != revision {
                    return Err("Repair changed during review. Inspect retained work and prepare a fresh patch.".into());
                }
                artifact_matches(&self.directory, &old)?;
            }
            summary(&publish(
                &self.directory,
                self.store.as_ref(),
                state,
                plan.expected,
                &cancel,
            )?)
        } else {
            self.view(&plan.query)?
        };
        Ok(result.to_string())
    }
}

impl Engine {
    pub(super) fn repair_view(
        &self,
        session: &str,
        id: Option<&str>,
        source: Option<String>,
    ) -> Result<Value, String> {
        self.store.workspace(session)?;
        let directory = self
            .workspace_directory
            .as_ref()
            .and_then(|p| p.parent())
            .ok_or("Managed repair storage unavailable.")?
            .join("repairs");
        RepairTool::new(self.store.clone(), session.into(), directory).view(&Query {
            action: if id.is_some() { "inspect" } else { "list" }.into(),
            repair_id: id.map(str::to_owned),
            revision: None,
            bundle_id: None,
            source,
            source_id: None,
            old_text: None,
            new_text: None,
            start_line: 1,
            line_count: 180,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn tool(root: &std::path::Path) -> RepairTool {
        let store = Arc::new(SqliteStore::open(&root.join("state.db")).unwrap());
        store.create("a").unwrap();
        store.create("b").unwrap();
        RepairTool::new(store, "a".into(), root.join("repairs"))
    }
    fn prepare(tool: &RepairTool, id: &str, args: Value) -> ToolRequest {
        tool.prepare(&ToolCall {
            id: id.into(),
            name: "harness_repair".into(),
            arguments: args.to_string(),
        })
        .unwrap()
    }
    fn baseline() -> Value {
        json!({"action":"prepare","source":"failure_watchdog","bundleId":introspection::bundle::ID,
            "sourceId":introspection::resolve("failure_watchdog").unwrap().id})
    }
    #[tokio::test]
    async fn reviewed_matching_proposal_has_owned_files_visible_diff_and_single_use() {
        let root = tempfile::tempdir().unwrap();
        let project = root.path().join("unrelated-project");
        std::fs::create_dir(&project).unwrap();
        std::fs::write(project.join("keep.txt"), "preserved").unwrap();
        let tool = tool(root.path());
        let token = CancellationToken::new();
        let req = prepare(&tool, "one", baseline());
        assert!(!tool.directory.exists()); // Preview cannot change project or publish a workspace.
        let result: Value =
            serde_json::from_str(&tool.invoke(&req, token.clone()).await.unwrap()).unwrap();
        assert!(tool.invoke(&req, token.clone()).await.is_err());
        let id = result["repairId"].as_str().unwrap();
        let original = tool.store.repair_workspace("a", id).unwrap();
        let f = &original.files[0];
        let req = prepare(
            &tool,
            "patch",
            json!({"action":"propose","repairId":id,"revision":1,"source":"failure_watchdog","sourceId":f.candidate_id,"oldText":"pub(crate) struct FailureWatchdog","newText":"// Retained native proposal\npub(crate) struct FailureWatchdog"}),
        );
        assert!(req
            .diff
            .as_ref()
            .unwrap()
            .contains("+// Retained native proposal"));
        let result: Value =
            serde_json::from_str(&tool.invoke(&req, token.clone()).await.unwrap()).unwrap();
        assert_eq!(result["revision"], 2);
        assert_eq!(result["status"], "proposed");
        let state = tool.store.repair_workspace("a", id).unwrap();
        artifact_matches(&tool.directory, &state).unwrap();
        assert_eq!(state.files[0].before, original.files[0].before);
        assert!(state.files[0].after.contains("Retained native proposal"));
        let req = prepare(
            &tool,
            "view",
            json!({"action":"inspect","repairId":id,"source":"failure_watchdog"}),
        );
        let view: Value = serde_json::from_str(&tool.invoke(&req, token).await.unwrap()).unwrap();
        assert_eq!(view["artifactIntegrity"], "matches");
        assert!(view["diff"]
            .as_str()
            .unwrap()
            .contains("Retained native proposal"));
        assert_eq!(
            std::fs::read_to_string(project.join("keep.txt")).unwrap(),
            "preserved"
        );
    }
    #[tokio::test]
    async fn stale_source_stop_cross_chat_and_changed_artifact_keep_retained_work() {
        let root = tempfile::tempdir().unwrap();
        let tool = tool(root.path());
        let token = CancellationToken::new();
        let mut stale = baseline();
        stale["bundleId"] = json!("stale");
        assert!(tool
            .prepare(&ToolCall {
                id: "stale".into(),
                name: "harness_repair".into(),
                arguments: stale.to_string()
            })
            .unwrap_err()
            .contains("Source bundle changed"));
        let req = prepare(&tool, "one", baseline());
        let result: Value =
            serde_json::from_str(&tool.invoke(&req, token.clone()).await.unwrap()).unwrap();
        let id = result["repairId"].as_str().unwrap();
        let before = tool.store.repair_workspace("a", id).unwrap();
        let patch = json!({"action":"propose","repairId":id,"revision":1,"source":"failure_watchdog","sourceId":before.files[0].candidate_id,"oldText":"pub(crate) struct FailureWatchdog","newText":"// Proposed\npub(crate) struct FailureWatchdog"});
        let req = prepare(&tool, "stopped", patch.clone());
        let cancel = CancellationToken::new();
        cancel.cancel();
        assert!(tool
            .invoke(&req, cancel)
            .await
            .unwrap_err()
            .contains("Retained snapshots"));
        assert_eq!(tool.store.repair_workspace("a", id).unwrap(), before);
        let other = RepairTool::new(tool.store.clone(), "b".into(), tool.directory.clone());
        assert!(other
            .prepare(&ToolCall {
                id: "cross".into(),
                name: "harness_repair".into(),
                arguments: json!({"action":"inspect","repairId":id}).to_string()
            })
            .unwrap_err()
            .contains("another chat"));
        let req = prepare(&tool, "reviewed", patch);
        std::fs::write(
            tool.directory
                .join(&before.artifact)
                .join("candidate")
                .join(&before.files[0].path),
            "changed externally",
        )
        .unwrap();
        assert!(tool
            .invoke(&req, token)
            .await
            .unwrap_err()
            .contains("artifact changed"));
        assert_eq!(tool.store.repair_workspace("a", id).unwrap(), before);
        let view = tool
            .view(&Query {
                action: "inspect".into(),
                repair_id: Some(id.into()),
                revision: None,
                bundle_id: None,
                source: Some("failure_watchdog".into()),
                source_id: None,
                old_text: None,
                new_text: None,
                start_line: 1,
                line_count: 180,
            })
            .unwrap();
        assert!(view["artifactIntegrity"]
            .as_str()
            .unwrap()
            .contains("artifact changed"));
        assert!(view["source"]["text"]
            .as_str()
            .unwrap()
            .contains("FailureWatchdog"));
    }
}
