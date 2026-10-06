//! Explicitly reviewed native tests. Process supervision is not OS containment.
#[cfg(test)]
#[path = "repair_evaluation_tests.rs"]
mod tests;
use crate::{
    harness_repair::{artifact_matches, identity, is_alias, matching},
    introspection::bundle,
};
use dolores_core::{
    NativeTestRun, RepairEvaluation, RepairWorkspace, SessionStore, ToolCall, ToolPlugin,
    ToolRequest, ToolSpec,
};
use quote::ToTokens;
use serde::Deserialize;
use serde_json::{json, Value};
use std::{
    collections::HashMap,
    fs,
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
};
use syn::visit::Visit;
use tokio_util::sync::CancellationToken;

const ARGS: &[&str] = &[
    "test",
    "--offline",
    "--locked",
    "--workspace",
    "--lib",
    "--",
    "--test-threads=1",
];
const SECONDS: u64 = 300;
const CAPTURE: usize = 262144;
const CRITERIA:&str="native-reproduction-and-regressions-v1; identical reviewed reproduction; unchanged test syntax; offline locked; serial tests; at most 3 commands; 300s/262144 bytes each; baseline test failure, candidate reproduction and workspace library regression passes required; no installation";

#[derive(Default)]
struct TestSyntax(Vec<String>);
fn test_attribute(attrs: &[syn::Attribute]) -> bool {
    attrs.iter().any(|a| {
        a.path()
            .segments
            .iter()
            .any(|s| s.ident == "test" || s.ident == "ignore")
            || (a.path().is_ident("cfg") || a.path().is_ident("cfg_attr"))
                && a.meta
                    .to_token_stream()
                    .to_string()
                    .split(|c: char| !c.is_alphanumeric() && c != '_')
                    .any(|s| s == "test")
    })
}
impl<'ast> Visit<'ast> for TestSyntax {
    fn visit_item(&mut self, item: &'ast syn::Item) {
        let attrs = match item {
            syn::Item::Const(i) => &i.attrs,
            syn::Item::Enum(i) => &i.attrs,
            syn::Item::Fn(i) => &i.attrs,
            syn::Item::Impl(i) => &i.attrs,
            syn::Item::Macro(i) => &i.attrs,
            syn::Item::Mod(i) => &i.attrs,
            syn::Item::Static(i) => &i.attrs,
            syn::Item::Struct(i) => &i.attrs,
            syn::Item::Trait(i) => &i.attrs,
            syn::Item::Type(i) => &i.attrs,
            syn::Item::Use(i) => &i.attrs,
            _ => {
                syn::visit::visit_item(self, item);
                return;
            }
        };
        if test_attribute(attrs) {
            self.0.push(item.to_token_stream().to_string());
        } else {
            syn::visit::visit_item(self, item);
        }
    }
}
fn tests(text: &str) -> Result<Vec<String>, String> {
    let ast=syn::parse_file(text).map_err(|_|"Candidate Rust syntax could not be parsed. Keep the proposal and fix its source before requesting native execution.")?;
    let mut visitor = TestSyntax::default();
    visitor.visit_file(&ast);
    Ok(visitor.0)
}
fn eligible(state: &RepairWorkspace) -> Result<(), String> {
    matching(state)?;
    if !state.files.iter().any(|f| f.before != f.after) {
        return Err("Prepare a changed candidate before requesting native tests; retained baseline remains.".into());
    }
    for f in state.files.iter().filter(|f| f.before != f.after) {
        if !f.path.starts_with("crates/")
            || !f.path.ends_with(".rs")
            || f.path.contains("test")
            || f.path.ends_with("build.rs")
            || f.path.contains("repair_evaluation")
            || f.path.contains("harness_repair")
            || f.path.contains("source_bundle")
            || tests(&f.before)? != tests(&f.after)?
        {
            return Err("This evaluator accepts Rust implementation changes with unchanged qualifying tests only. Test, manifest, build-script, evaluator or other-language changes need a separate reviewed workflow; the proposal remains.".into());
        }
    }
    Ok(())
}
fn criterion(state: &RepairWorkspace, package: &str, reproduction: &str) -> String {
    // Pin the suite and candidate separately. No provider-chosen argv or grading exists.
    identity(&format!(
        "{CRITERIA}\n{}\n{package}\n{reproduction}",
        state.bundle_id
    ))
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Query {
    repair_id: String,
    revision: u32,
    package: String,
    reproduction: String,
}
struct Plan {
    request: ToolRequest,
    state: RepairWorkspace,
    receipt: RepairEvaluation,
    command: dolores_core::CommandPreview,
    reproduction_command: dolores_core::CommandPreview,
    cargo_id: String,
}
pub(super) struct EvaluationTool {
    store: Arc<dyn SessionStore>,
    session: String,
    directory: PathBuf,
    plans: Mutex<HashMap<String, Plan>>,
}
pub(super) fn spec() -> ToolSpec {
    ToolSpec {name:"test_harness_repair".into(),description:"Separately review native Rust evaluation for exact repairId/revision. package is dolores-core or dolores-provider-openai; reproduction is a complete Rust integration test (4096 UTF-8 bytes maximum) using that crate's public API. Review this exact test and candidate diff. Host freezes the same reproduction for baseline/candidate, then runs existing candidate workspace library regressions. Installed Cargo, offline/locked, 300 seconds and 256 KiB capture per command, at most three commands. Build scripts and native code have account permissions, NOT OS containment. Baseline already passing skips candidate execution; incomplete evidence cannot qualify. Existing test definitions cannot change. No installation/restart/task replay/retry. Full access is not authority.".into(),parameters:json!({"type":"object","properties":{"repairId":{"type":"string"},"revision":{"type":"integer","minimum":1},"package":{"type":"string","enum":["dolores-core","dolores-provider-openai"]},"reproduction":{"type":"string","maxLength":4096}},"required":["repairId","revision","package","reproduction"],"additionalProperties":false})}
}
impl EvaluationTool {
    pub fn new(store: Arc<dyn SessionStore>, session: String, directory: PathBuf) -> Self {
        Self {
            store,
            session,
            directory,
            plans: Mutex::new(HashMap::new()),
        }
    }
    fn current(&self, id: &str, revision: u32) -> Result<RepairWorkspace, String> {
        if !dolores_core::harness_repair::valid_repair_id(id) {
            return Err("Invalid repair ID; use this chat's retained repair.".into());
        }
        let state = self.store.repair_workspace(&self.session, id)?;
        if state.revision != revision {
            return Err("Repair changed since review. Inspect the current revision before requesting native tests; previous work remains.".into());
        }
        eligible(&state)?;
        artifact_matches(&self.directory, &state)?;
        Ok(state)
    }
}
fn command_call(id: &str) -> ToolCall {
    ToolCall {
        id: id.into(),
        name: "run_command".into(),
        arguments:
            json!({"program":"cargo","args":ARGS,"timeout_seconds":SECONDS,"capture_bytes":CAPTURE})
                .to_string(),
    }
}
fn cargo_identity(command: &dolores_core::CommandPreview) -> Result<String, String> {
    use std::io::Read;
    let file = fs::File::open(&command.executable)
        .map_err(|_| "Installed Cargo unavailable; request a fresh native review.")?;
    let size = file
        .metadata()
        .map_err(|_| "Installed Cargo unavailable.")?
        .len();
    if size > 64 * 1024 * 1024 {
        return Err(
            "Installed Cargo exceeds its inspection allowance; no native trial ran.".into(),
        );
    }
    let mut bytes = vec![];
    file.take(64 * 1024 * 1024 + 1)
        .read_to_end(&mut bytes)
        .map_err(|_| "Installed Cargo could not be inspected.")?;
    if bytes.len() > 64 * 1024 * 1024 {
        return Err("Installed Cargo changed during inspection.".into());
    }
    Ok(format!("sha256:{:x}", sha2::Sha256::digest(&bytes)))
}
async fn checked_variant(
    root: &Path,
    command: &dolores_core::CommandPreview,
    cargo_id: &str,
    cancel: CancellationToken,
) -> Result<NativeTestRun, String> {
    if cargo_identity(command)? != cargo_id {
        return Err("Installed Cargo changed since review. Retain the proposal and request fresh native execution review.".into());
    }
    variant(root, command, cancel).await
}
fn materialize(
    root: &Path,
    state: &RepairWorkspace,
    cancel: &CancellationToken,
) -> Result<(), String> {
    for variant in ["baseline", "candidate"] {
        for source in bundle::SOURCES {
            if cancel.is_cancelled() {
                return Err("Stopped while preparing native tests; proposal remains.".into());
            }
            let text = if variant == "candidate" {
                state
                    .files
                    .iter()
                    .find(|f| f.path == source.path)
                    .map(|f| f.after.clone())
                    .map(Ok)
                    .unwrap_or_else(|| bundle::text(source))?
            } else {
                bundle::text(source)?
            };
            let file = root.join(variant).join(source.path);
            fs::create_dir_all(file.parent().unwrap())
                .map_err(|_| "Evaluation source could not be staged; proposal remains.")?;
            use std::io::Write;
            let mut out = fs::OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(file)
                .map_err(|_| "Evaluation file could not be staged.")?;
            out.write_all(text.as_bytes())
                .and_then(|_| out.sync_all())
                .map_err(|_| "Evaluation source could not be saved.")?;
        }
    }
    Ok(())
}
fn verify_sources(root: &Path, state: &RepairWorkspace) -> Result<(), String> {
    if is_alias(&fs::symlink_metadata(root).map_err(|_| "Native evaluation storage unavailable.")?)
    {
        return Err("Native evaluation storage changed; no improvement qualified.".into());
    }
    for variant in ["baseline", "candidate"] {
        for source in bundle::SOURCES {
            let path = root.join(variant).join(source.path);
            let mut check = root.to_path_buf();
            for part in format!("{variant}/{}", source.path).split('/') {
                check.push(part);
                if is_alias(
                    &fs::symlink_metadata(&check)
                        .map_err(|_| "Evaluation source unavailable; no improvement qualified.")?,
                ) {
                    return Err("Evaluation file alias refused; retain evidence and start a fresh reviewed trial.".into());
                }
            }
            let mut file = fs::File::open(path).map_err(|_| "Evaluation source unavailable.")?;
            if file
                .metadata()
                .map_err(|_| "Evaluation source unavailable.")?
                .len()
                > dolores_core::MAX_FILE_SNAPSHOT_BYTES as u64
            {
                return Err(
                    "Evaluation source exceeds its frozen allowance; no improvement qualified."
                        .into(),
                );
            }
            use std::io::Read;
            let mut bytes = vec![];
            file.by_ref()
                .take(dolores_core::MAX_FILE_SNAPSHOT_BYTES as u64 + 1)
                .read_to_end(&mut bytes)
                .map_err(|_| "Evaluation source unavailable.")?;
            let expected = if variant == "candidate" {
                state
                    .files
                    .iter()
                    .find(|f| f.path == source.path)
                    .map(|f| f.candidate_id.as_str())
                    .unwrap_or(source.id)
            } else {
                source.id
            };
            if bytes.len() > dolores_core::MAX_FILE_SNAPSHOT_BYTES
                || format!("sha256:{:x}", sha2::Sha256::digest(&bytes)) != expected
            {
                return Err("Evaluation source or frozen tests changed during execution. No improvement qualified; retained proposal remains.".into());
            }
        }
    }
    Ok(())
}
use sha2::Digest;
fn reproduction_path(root: &Path, variant: &str, receipt: &RepairEvaluation) -> PathBuf {
    root.join(variant)
        .join("crates")
        .join(&receipt.package)
        .join("tests/dolores_repair_reproduction.rs")
}
fn stage_reproduction(root: &Path, receipt: &RepairEvaluation) -> Result<(), String> {
    use std::io::Write;
    for variant in ["baseline", "candidate"] {
        let path = reproduction_path(root, variant, receipt);
        fs::create_dir_all(path.parent().unwrap())
            .map_err(|_| "Frozen reproduction could not be staged.")?;
        let mut file = fs::OpenOptions::new()
            .create_new(true)
            .write(true)
            .open(path)
            .map_err(|_| "Frozen reproduction conflicts with retained source; no command ran.")?;
        file.write_all(receipt.reproduction.as_bytes())
            .and_then(|_| file.sync_all())
            .map_err(|_| "Frozen reproduction could not be saved.")?;
    }
    Ok(())
}
fn verify_reproduction(root: &Path, receipt: &RepairEvaluation) -> Result<(), String> {
    for variant in ["baseline", "candidate"] {
        let path = reproduction_path(root, variant, receipt);
        for p in [path.parent().unwrap(), path.as_path()] {
            let metadata = fs::symlink_metadata(p)
                .map_err(|_| "Frozen reproduction unavailable; no improvement qualified.")?;
            if is_alias(&metadata) || (p == path && (!metadata.is_file() || metadata.len() > 4096))
            {
                return Err("Frozen reproduction changed; no improvement qualified.".into());
            }
        }
        if fs::read_to_string(path).map_err(|_| "Frozen reproduction unreadable.")?
            != receipt.reproduction
        {
            return Err("Frozen reproduction changed during execution. Retain evidence; do not install or replay.".into());
        }
    }
    Ok(())
}
fn output_text(root: &Path, value: &Value) -> Result<String, String> {
    if value["previewTruncated"] == true {
        let name = value["localLog"]
            .as_str()
            .ok_or("Native test log unavailable; verification incomplete.")?;
        if !name.starts_with("dolores-command-log-")
            || !name.ends_with(".txt")
            || name.contains(['/', '\\', ':'])
        {
            return Err("Invalid native log identity; verification incomplete.".into());
        }
        let path = root.join(name);
        let meta = fs::symlink_metadata(&path).map_err(|_| "Native test log unavailable.")?;
        if is_alias(&meta) || !meta.is_file() || meta.len() > CAPTURE as u64 + 2048 {
            return Err(
                "Native log changed or exceeded its allowance; verification incomplete.".into(),
            );
        }
        return fs::read_to_string(path)
            .map_err(|_| "Native test log unreadable; verification incomplete.".into());
    }
    Ok(format!(
        "{}\n{}",
        value["stdout"].as_str().unwrap_or(""),
        value["stderr"].as_str().unwrap_or("")
    ))
}
fn measured(root: &Path, value: &Value) -> Result<NativeTestRun, String> {
    let text = output_text(root, value)?;
    let mut passed = vec![];
    let mut failed = vec![];
    for line in text.lines() {
        if let Some((name, status)) = line
            .strip_prefix("test ")
            .and_then(|s| s.rsplit_once(" ... "))
        {
            if name.len() > 512 || passed.len() + failed.len() >= 4096 {
                return Err(
                    "Native test evidence exceeds its allowance; no improvement qualified.".into(),
                );
            }
            if status == "ok" {
                passed.push(name.into());
            } else if status == "FAILED" {
                failed.push(name.into());
            }
        }
    }
    Ok(NativeTestRun {
        reason: value["reason"].as_str().unwrap_or("unknown").into(),
        exit_code: value["exitCode"]
            .as_i64()
            .and_then(|n| i32::try_from(n).ok()),
        complete: value["reason"] == "completed"
            && value["truncated"] == false
            && value["outputError"] == false
            && value["lossyUtf8"] == false
            && text.contains("test result:")
            && (!passed.is_empty() || !failed.is_empty()),
        passed,
        failed,
    })
}
async fn variant(
    root: &Path,
    preview: &dolores_core::CommandPreview,
    cancel: CancellationToken,
) -> Result<NativeTestRun, String> {
    let command = dolores_tools_command::RunCommand::new(root)?;
    let request = command.prepare(&ToolCall{id:"native-test".into(),name:"run_command".into(),arguments:json!({"program":"cargo","args":preview.invocation.args,"timeout_seconds":SECONDS,"capture_bytes":CAPTURE}).to_string()})?;
    if request.command.as_ref() != Some(preview) {
        return Err("Installed Cargo changed since review. Retain the proposal and request fresh native execution review.".into());
    }
    let result = command.invoke(&request, cancel).await?;
    use std::io::Write;
    let log_name = if preview.invocation.args.iter().any(|s| s == "--test") {
        "reproduction-command.json"
    } else {
        "native-command.json"
    };
    let mut log = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(root.join(log_name))
        .map_err(|_| "Native test evidence could not be retained; no improvement qualified.")?;
    log.write_all(result.as_bytes())
        .and_then(|_| log.sync_all())
        .map_err(|_| "Native test evidence could not be saved; no improvement qualified.")?;
    let value: Value =
        serde_json::from_str(&result).map_err(|_| "Native trial returned invalid evidence.")?;
    measured(root, &value)
}
#[async_trait::async_trait]
impl ToolPlugin for EvaluationTool {
    fn spec(&self) -> ToolSpec {
        spec()
    }
    fn prepare(&self, call: &ToolCall) -> Result<ToolRequest, String> {
        if call.name != "test_harness_repair" || call.arguments.len() > 8192 {
            return Err("Invalid native trial arguments; use repairId, revision, package and bounded reproduction source.".into());
        }
        let q: Query = serde_json::from_str(&call.arguments).map_err(|_| {
            "Use repairId, current revision, supported package and bounded reproduction source."
        })?;
        if !["dolores-core", "dolores-provider-openai"].contains(&q.package.as_str())
            || q.reproduction.is_empty()
            || q.reproduction.len() > 4096
            || q.reproduction.contains('\0')
            || tests(&q.reproduction)?.is_empty()
        {
            return Err("Choose a supported package and a complete Rust reproduction test within 4096 bytes; original proposal remains.".into());
        }
        let state = self.current(&q.repair_id, q.revision)?;
        if self
            .store
            .repair_evaluations(&self.session, &state.id)?
            .len()
            >= 4
        {
            return Err("This repair has four retained native trials. Inspect them or explicitly prepare a separate repair; no trial was replayed.".into());
        }
        let command = dolores_tools_command::RunCommand::new(&self.directory)?
            .prepare(&command_call("native-probe"))?
            .command
            .unwrap();
        let reproduction_command=dolores_tools_command::RunCommand::new(&self.directory)?.prepare(&ToolCall{id:"reproduction-probe".into(),name:"run_command".into(),arguments:json!({"program":"cargo","args":["test","--offline","--locked","-p",q.package,"--test","dolores_repair_reproduction","--","--test-threads=1"],"timeout_seconds":SECONDS,"capture_bytes":CAPTURE}).to_string()})?.command.unwrap();
        let id = uuid::Uuid::new_v4().to_string();
        let cargo_id = cargo_identity(&command)?;
        let receipt=RepairEvaluation{id:id.clone(),session:self.session.clone(),repair_id:state.id.clone(),revision:state.revision,bundle_id:state.bundle_id.clone(),candidate_ids:state.files.iter().map(|f|f.candidate_id.clone()).collect(),criteria_id:criterion(&state,&q.package,&q.reproduction),cargo_id:cargo_id.clone(),package:q.package,reproduction:q.reproduction,status:"started".into(),artifact:format!("evaluation-{id}"),baseline:None,candidate:None,regression:None,note:"No installation authority. An unfinished receipt requires a fresh reviewed trial; nothing resumes automatically.".into()};
        let mut diff = state
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
        diff.push_str(&format!(
            "\nFrozen reproduction, identical in baseline and candidate:\n{}",
            dolores_tools_fs::change_diff("", &receipt.reproduction)
        ));
        if diff.len() > 12288 {
            return Err("Combined native review diff exceeds 12 KiB. Retain the proposal and prepare a smaller repair before execution.".into());
        }
        let request=ToolRequest{call_id:call.id.clone(),name:call.name.clone(),target:"Dolores native evaluation".into(),query:Some(json!({"repairId":state.id,"revision":state.revision,"evaluationId":id,"bundleId":state.bundle_id,"criteriaId":receipt.criteria_id,"cargoId":cargo_id,"commands":{"baselineReproduction":reproduction_command,"candidateReproduction":reproduction_command,"candidateRegressions":command},"candidateRunsOnlyAfter":"complete baseline test failure","storage":"separate owned evaluation workspace","authority":"account permissions; not OS sandboxed; no installation"}).to_string()),diff:Some(diff),command:None,mcp:None};
        let mut plans = self
            .plans
            .lock()
            .map_err(|_| "Native review unavailable.")?;
        if plans.len() >= dolores_core::MAX_TOOL_CALLS || plans.contains_key(&call.id) {
            return Err("Native review allowance reached; retained proposal remains.".into());
        }
        plans.insert(
            call.id.clone(),
            Plan {
                request: request.clone(),
                state,
                receipt,
                command,
                reproduction_command,
                cargo_id,
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
            .map_err(|_| "Native review unavailable.")?
            .remove(&request.call_id)
            .ok_or("Native review expired; request a fresh trial.")?;
        if &plan.request != request || cancel.is_cancelled() {
            return Err(
                "Native review changed or stopped. Proposal remains; nothing executed.".into(),
            );
        }
        let current = self.current(&plan.state.id, plan.state.revision)?;
        if current != plan.state {
            return Err("Candidate changed since native review; nothing executed.".into());
        }
        let mut receipt = plan.receipt;
        self.store.save_repair_evaluation(&receipt)?;
        let root = self.directory.join(&receipt.artifact);
        let result=async {
            fs::create_dir(&root).map_err(|_|"Native evaluation storage unavailable; no command ran.")?;
            let state=plan.state.clone();let staging=root.clone();let token=cancel.clone();
            tokio::task::spawn_blocking(move||materialize(&staging,&state,&token)).await.map_err(|_|"Native staging worker failed.")??;
            stage_reproduction(&root,&receipt)?;
            verify_sources(&root,&plan.state)?;
            verify_reproduction(&root,&receipt)?;
            receipt.baseline=Some(checked_variant(&root.join("baseline"),&plan.reproduction_command,&plan.cargo_id,cancel.clone()).await?);
            self.store.save_repair_evaluation(&receipt)?;
            verify_sources(&root,&plan.state)?;
            verify_reproduction(&root,&receipt)?;
            let baseline=receipt.baseline.as_ref().unwrap();
            if !baseline.complete || baseline.failed.is_empty() || baseline.exit_code==Some(0) {
                receipt.status="withheld".into();
                receipt.note=match baseline.reason.as_str() {
                    "timedOut"=>"Baseline reached the 300-second execution limit. Candidate did not run. Source and available logs remain; prepare a smaller reproduction before a fresh bounded review.",
                    "outputLimit"=>"Baseline reached the 256 KiB output limit. Candidate did not run. Source and bounded logs remain; reduce reproduction output before a fresh review.",
                    _ if !baseline.complete=>"Baseline compilation, startup or evidence was incomplete. Candidate did not run. Inspect retained baseline logs and fix the toolchain or reproduction before a fresh review; no retry or installation occurred.",
                    _=>"Baseline did not reproduce a complete test failure. Candidate was not executed. Keep evidence; reproduce the actual fault with separately reviewed frozen criteria before claiming a repair.",
                }.into();
                return Ok::<(),String>(());
            }
            receipt.candidate=Some(checked_variant(&root.join("candidate"),&plan.reproduction_command,&plan.cargo_id,cancel.clone()).await?);
            self.store.save_repair_evaluation(&receipt)?;
            verify_sources(&root,&plan.state)?;
            verify_reproduction(&root,&receipt)?;
            if receipt.candidate.as_ref().is_some_and(|c|c.complete && c.exit_code==Some(0) && c.failed.is_empty()) {
                receipt.regression=Some(checked_variant(&root.join("candidate"),&plan.command,&plan.cargo_id,cancel.clone()).await?);
            }
            verify_sources(&root,&plan.state)?;
            verify_reproduction(&root,&receipt)?;
            self.current(&plan.state.id,plan.state.revision)?;
            receipt.status=if receipt.improved(){"qualified"}else{"withheld"}.into();
            receipt.note="Reviewed Rust library evidence only; no OS containment, general reliability, installation or task replay. Installation requires a separate reviewed build/restart/restore workflow.".into();Ok(())
        }.await;
        if let Err(error) = result {
            receipt.status = if cancel.is_cancelled() {
                "stopped"
            } else {
                "failed"
            }
            .into();
            receipt.note = error;
        }
        if let Err(_error) = self.store.save_repair_evaluation(&receipt) {
            return Err("Native trial finished but its final receipt could not be saved. Owned artifacts and proposal remain; inspect the unfinished receipt and local logs. Do not install or automatically replay.".into());
        }
        // The on-disk receipt is convenience evidence; SQLite remains authoritative.
        let _ = fs::write(
            root.join("receipt.json"),
            serde_json::to_vec(&receipt).unwrap(),
        );
        let value = json!({"evaluation":receipt,"note":"Native execution was independently reviewed. Passing tests grants no installation authority."});
        if value.to_string().len() > dolores_core::MAX_TOOL_BYTES {
            return Ok(json!({"evaluationId":receipt.id,"status":receipt.status,"note":receipt.note,"baselinePassed":receipt.baseline.as_ref().map(|r|r.passed.len()),"baselineFailed":receipt.baseline.as_ref().map(|r|r.failed.len()),"candidatePassed":receipt.candidate.as_ref().map(|r|r.passed.len()),"candidateFailed":receipt.candidate.as_ref().map(|r|r.failed.len()),"evidence":"Full typed receipt and bounded logs retained locally."}).to_string());
        }
        Ok(value.to_string())
    }
}
