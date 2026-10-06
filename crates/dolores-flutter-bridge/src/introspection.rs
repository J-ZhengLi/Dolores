use super::*;
use dolores_core::{ToolCall, ToolPlugin, ToolRequest, ToolSpec};
use std::io::Read;

const SOURCES: &[(&str, &str)] = &[
    ("core", include_str!("../../dolores-core/src/lib.rs")),
    ("agent", include_str!("../../dolores-core/src/agent.rs")),
    ("host", include_str!("lib.rs")),
    ("files", include_str!("../../dolores-tools-fs/src/lib.rs")),
    (
        "provider",
        include_str!("../../dolores-provider-openai/src/agent.rs"),
    ),
    ("subagents", include_str!("subagents.rs")),
    (
        "provider_stream",
        include_str!("../../dolores-provider-openai/src/agent_stream.rs"),
    ),
    (
        "provider_settings",
        include_str!("../../dolores-provider-openai/src/lib.rs"),
    ),
    ("recovery", include_str!("recovery.rs")),
    ("attachments", include_str!("attachments.rs")),
];
const PATHS: &[&str] = &[
    "crates/dolores-core/src/lib.rs",
    "crates/dolores-core/src/agent.rs",
    "crates/dolores-flutter-bridge/src/lib.rs",
    "crates/dolores-tools-fs/src/lib.rs",
    "crates/dolores-provider-openai/src/agent.rs",
    "crates/dolores-flutter-bridge/src/subagents.rs",
    "crates/dolores-provider-openai/src/agent_stream.rs",
    "crates/dolores-provider-openai/src/lib.rs",
    "crates/dolores-flutter-bridge/src/recovery.rs",
    "crates/dolores-flutter-bridge/src/attachments.rs",
];

fn source_id(text: &str) -> String {
    // Diagnostic identity only; actual checkout matching compares complete bytes.
    let hash = text.bytes().fold(0xcbf29ce484222325u64, |h, b| {
        (h ^ u64::from(b)).wrapping_mul(0x100000001b3)
    });
    format!("fnv1a64:{hash:016x}")
}
fn checkout_matches(path: &std::path::Path, text: &str) -> bool {
    let Ok(file) = std::fs::File::open(path) else {
        return false;
    };
    if !file
        .metadata()
        .is_ok_and(|m| m.is_file() && m.len() == text.len() as u64)
    {
        return false;
    }
    let mut bytes = Vec::new();
    file.take(text.len() as u64 + 1)
        .read_to_end(&mut bytes)
        .is_ok()
        && bytes == text.as_bytes()
}
pub fn spec() -> ToolSpec {
    ToolSpec {
        name: "inspect_harness".into(),
        description: "Diagnose Dolores using current capabilities, limits, recent failures in this chat and version-matched bundled source. Empty arguments show inventory and diagnostics. Use its source IDs to read exact code. Read-only; requires host approval; never edits the harness or grants permissions. Historical failure evidence may describe an older build, not the current source.".into(),
        parameters: json!({"type":"object","properties":{"source":{"type":"string","enum":SOURCES.iter().map(|(key,_)|key).collect::<Vec<_>>()},"startLine":{"type":"integer","minimum":1},"lineCount":{"type":"integer","minimum":1,"maximum":120}},"additionalProperties":false}),
    }
}
#[derive(Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Query {
    source: Option<String>,
    #[serde(default = "one")]
    start_line: usize,
    #[serde(default = "default_lines")]
    line_count: usize,
}
fn one() -> usize {
    1
}
fn default_lines() -> usize {
    60
}
fn parse(text: &str) -> Result<Query, String> {
    let q: Query =
        serde_json::from_str(text).map_err(|_| "Invalid harness inspection arguments.")?;
    if q.start_line == 0 || !(1..=120).contains(&q.line_count) {
        return Err("Use a positive start line and 1–120 lines.".into());
    }
    if let Some(name) = &q.source {
        if !SOURCES.iter().any(|(key, _)| key == name) {
            return Err(
                "Source is unavailable. Inspect inventory for supported source names.".into(),
            );
        }
    }
    Ok(q)
}
pub fn source(query: &str, checkout: Option<&std::path::Path>) -> Result<Value, String> {
    let q = parse(query)?;
    let index = SOURCES
        .iter()
        .position(|(key, _)| Some(*key) == q.source.as_deref())
        .ok_or("Choose a bundled source name.")?;
    let text = SOURCES[index].1;
    let mut lines = String::new();
    let mut next = q.start_line;
    for (n, line) in text
        .lines()
        .enumerate()
        .skip(q.start_line - 1)
        .take(q.line_count)
    {
        let item = format!("{}\t{line}\n", n + 1);
        if lines.len() + item.len() > 12 * 1024 {
            break;
        }
        lines.push_str(&item);
        next = n + 2;
    }
    if lines.is_empty() {
        return Err("Source range is unavailable. Choose an earlier or smaller range.".into());
    }
    let checkout_status = checkout.map(|root| {
        let candidate = root.join(PATHS[index]);
        match (root.canonicalize(), candidate.canonicalize()) {
            (Ok(root), Ok(path)) if path.starts_with(&root) => {
                if checkout_matches(&path, text) {
                    "matches"
                } else {
                    "mismatch"
                }
            }
            _ => "unavailable",
        }
    });
    Ok(
        json!({"source":q.source,"path":PATHS[index],"sourceId":source_id(text),"origin":"bundled with running build","checkoutStatus":checkout_status,"startLine":q.start_line,"nextLine":next,"totalLines":text.lines().count(),"text":lines}),
    )
}
impl Engine {
    fn recent_harness_failures(&self, session: Option<&str>) -> Result<Value, String> {
        let Some(session) = session else {
            return Ok(json!([]));
        };
        let mut failures = vec![];
        for run in self
            .store
            .runs(session)?
            .into_iter()
            .filter(|r| {
                matches!(
                    r.state,
                    dolores_core::RunState::Failed
                        | dolores_core::RunState::Paused
                        | dolores_core::RunState::Interrupted
                )
            })
            .take(3)
        {
            let events = self.store.run_events(session, &run.id)?;
            let terminal = events.iter().rev().find(|e| e.kind == "finished");
            let error = terminal
                .and_then(|e| e.data["message"].as_str())
                .unwrap_or("");
            let recovery = crate::recovery::advice(error);
            // Never expose arbitrary plugin messages, prompts, paths, arguments
            // or transcripts. Categories and explanations are host-authored.
            let evidence = if error == "Streamed tool response exceeds its frame limit." {
                "Legacy stream frame guard rejected the response. That build bounded both event count and individual event size. The saved record does not contain counts, so it cannot establish which bound fired."
            } else if error.starts_with("Provider sent an oversized stream event") {
                "A single provider stream event exceeded 256 KiB. No incomplete tool call was executed."
            } else if error == "Streamed tool response exceeds the 2 MiB wire limit." {
                "The streamed response exceeded the total wire byte allowance."
            } else if error == "Streamed tool field exceeds its byte limit." {
                "An assembled response field exceeded its byte allowance."
            } else if run.state == dolores_core::RunState::Paused {
                "This task paused. Inspect the saved pause reason and completed work before continuing."
            } else if run.state == dolores_core::RunState::Interrupted {
                "Execution was interrupted. Effects may already exist; inspect receipts before retrying."
            } else {
                recovery.guidance
            };
            failures.push(json!({"runId":run.id,"state":run.state,"model":run.model,
                "build":run.build,"createdAt":run.created_at,"requestSettings":run.settings,
                "category":recovery.kind,"evidence":evidence,
                "savedTurn":terminal.and_then(|e|e.data["savedTurn"].as_bool()),
                "pauseReason":terminal.and_then(|e|e.data.get("pauseReason")),
                "sourceHints":["provider_stream","provider_settings","agent","host","recovery"],
                "certainty":"Historical evidence only. Source is from the current running build. Completed effects are not automatically undone."}));
        }
        Ok(json!(failures))
    }
    pub(super) fn harness_inventory(&self, session: Option<&str>) -> Result<Value, String> {
        let effective = self.effective_settings(session)?;
        let registry = self.extension_registry(session)?;
        let workspace = session.map(|id| self.store.workspace(id)).transpose()?;
        let working = workspace.as_ref().is_some_and(|w| w.root.is_some());
        let connection = self
            .connection
            .lock()
            .map_err(|_| "Connection unavailable.")?;
        let configured = connection.provider.is_some();
        let preferences = self.store.preferences()?;
        let contexts = self.store.model_contexts(&preferences.base_url)?;
        let capacity = contexts.get(&preferences.model).copied().flatten();
        let mut tools = if working {
            dolores_tools_fs::folder_tool_specs()
        } else {
            vec![]
        };
        if working {
            tools.push(dolores_tools_command::command_spec());
            tools.push(spec());
            tools.push(crate::subagents::spec());
            tools.extend(dolores_tools_web::specs(&self.store.web_configuration()?));
            if self.browser_runtime().is_ok() {
                tools.push(dolores_tools_browser::spec());
            }
            for c in self
                .store
                .mcp_connections(workspace.as_ref().unwrap().root.as_ref().unwrap())?
            {
                tools.extend(c.specs());
            }
        }
        let catalog: Vec<_> = tools.iter().map(|tool| json!({"id":tool.name,"enabled":true,"available":configured,"permission":"host checks current task mode and exact grants","empiricallyTested":"unknown","source": if tool.name == "inspect_harness" { "host" } else if tool.name == "delegate_tasks" {"subagents"} else if tool.name.starts_with("mcp_tool_") { "external server; no bundled source" } else if matches!(tool.name.as_str(), "web_search" | "read_web_page") { "web adapter; no bundled source" } else if tool.name == "run_command" { "command adapter; no bundled source" } else { "files" }})).collect();
        let sources: Vec<_> = SOURCES.iter().enumerate().map(|(n, (key, text))| json!({"id":key,"path":PATHS[n],"sourceId":source_id(text),"lines":text.lines().count()})).collect();
        let images = configured
            && self
                .store
                .image_models(&preferences.base_url)?
                .contains(&preferences.model);
        let recent = self.recent_harness_failures(session)?;
        let mut inventory = json!({"version":env!("CARGO_PKG_VERSION"),"revision":env!("DOLORES_BUILD_REVISION"),"sourceIdentity":"bundled file identities; revision may include local source changes","workspace":workspace.map(|w| w.kind),"configured":configured,"model":preferences.model,"modelCapabilities":{"input":if images {vec!["text","image"]} else {vec!["text"]},"tools":"adapter supports function calls; selected model support not established","images":if images {"image input configured; comprehension not established"} else {"disabled; enable a capable model in Model connection"}},"attachments":{"formats":["text/plain","image/png","image/jpeg"],"draftFiles":dolores_core::MAX_DRAFT_ATTACHMENTS,"textBytes":dolores_core::MAX_TEXT_ATTACHMENT_BYTES,"imageBytes":dolores_core::MAX_IMAGE_ATTACHMENT_BYTES,"storeBytes":dolores_core::MAX_ATTACHMENT_STORE_BYTES},"approval":format!("{:?}",effective.permissions.mode),"containment":"file tools use folder capabilities; commands/MCP have user-account permissions","selfUpdate":"not available","contextWindowTokens":capacity.unwrap_or(dolores_core::DEFAULT_CONTEXT_WINDOW_TOKENS),"contextOrigin":if capacity.is_some(){"model override"}else{"128K default"},"effectiveSettings":effective,"requestSettings":effective.request,"limits":{"modelCalls":effective.task.model_calls,"toolOperations":effective.task.tool_calls,"taskSegments":effective.task.segments,"taskDeadlineSeconds":effective.task.deadline(effective.request.timeout_seconds),"inputBytes":dolores_core::MAX_INPUT_BYTES,"contextBytes":dolores_core::MAX_CONTEXT_BYTES,"maxSourceLines":120},"extensionApi":dolores_core::HOST_EXTENSION_API,"extensions":registry.entries,"hookOrder":registry.order,"adapters":[self.store.descriptor(),connection.descriptor(),self.mcp_credentials.descriptor()],"tools":catalog,"unavailableReason":if working {""}else{"No working folder. Start a project or temporary working session for tool use."},"sources":sources});
        inventory["recentFailures"] = recent;
        inventory["diagnosticCoverage"] = json!("Latest 20 runs in this chat, at most 3 failed/paused/interrupted runs; no private transcripts, file paths, tool arguments or credentials.");
        Ok(inventory)
    }
}
pub struct InspectHarness(pub Value);
#[async_trait::async_trait]
impl ToolPlugin for InspectHarness {
    fn spec(&self) -> ToolSpec {
        spec()
    }
    fn prepare(&self, call: &ToolCall) -> Result<ToolRequest, String> {
        if call.name != "inspect_harness" {
            return Err("Invalid inspection tool.".into());
        }
        let q = parse(&call.arguments)?;
        Ok(ToolRequest {
            call_id: call.id.clone(),
            name: call.name.clone(),
            target: "running Dolores harness".into(),
            query: Some(serde_json::to_string(&q).map_err(|_| "Invalid inspection request.")?),
            diff: None,
            command: None,
            mcp: None,
        })
    }
    async fn invoke(
        &self,
        request: &ToolRequest,
        cancel: CancellationToken,
    ) -> Result<String, String> {
        if cancel.is_cancelled() {
            return Err(stopped());
        }
        if request.name != "inspect_harness"
            || request.target != "running Dolores harness"
            || request.diff.is_some()
            || request.command.is_some()
            || request.mcp.is_some()
        {
            return Err("Inspection request changed.".into());
        }
        let query = request
            .query
            .as_deref()
            .ok_or("Inspection query missing.")?;
        let q = parse(query)?;
        let value = if q.source.is_some() {
            source(query, None)?
        } else {
            self.0.clone()
        };
        serde_json::to_string(&value).map_err(|_| "Inspection result unavailable.".into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn diagnostics_are_chat_scoped_bounded_and_never_echo_private_plugin_data() {
        use dolores_core::{RunSnapshot, RunState};
        let store = Arc::new(SqliteStore::open(std::path::Path::new(":memory:")).unwrap());
        store.create("a").unwrap();
        store.create("b").unwrap();
        for n in 0..5 {
            let id = uuid::Uuid::new_v4().to_string();
            let run = RunSnapshot {
                id: id.clone(),
                thread: if n == 4 { "b" } else { "a" }.into(),
                parent_run: None,
                segments: 1,
                model: "fixture".into(),
                settings: Default::default(),
                input: "PRIVATE_PROMPT_SENTINEL".into(),
                state: RunState::Prepared,
                sequence: 0,
                created_at: n,
                build: "older-build".into(),
                tools: vec![],
                extensions: vec![],
                effective_settings: None,
            };
            store.begin_run(&run).unwrap();
            store.append_run_event(&id, 0, Some(RunState::Failed), "finished",
                &json!({"savedTurn":false,"message":if n == 0 {"Streamed tool response exceeds its frame limit."} else {"PRIVATE_ERROR_SENTINEL /private/path secret-key"},"transcript":"PRIVATE_TRANSCRIPT_SENTINEL"})).unwrap();
        }
        let engine = Engine::new(
            store,
            Arc::new(connection::testing::MemoryCredentials::default()),
        )
        .unwrap();
        let a = engine.recent_harness_failures(Some("a")).unwrap();
        assert_eq!(a.as_array().unwrap().len(), 3);
        assert!(!a.to_string().contains("PRIVATE"));
        assert!(!a.to_string().contains("secret-key"));
        assert!(engine
            .recent_harness_failures(None)
            .unwrap()
            .as_array()
            .unwrap()
            .is_empty());
        let b = engine.recent_harness_failures(Some("b")).unwrap();
        assert_eq!(b.as_array().unwrap().len(), 1);
        assert_ne!(b[0]["runId"], a[0]["runId"]);
        let stream = source(r#"{"source":"provider_stream","lineCount":120}"#, None).unwrap();
        assert!(stream["text"].as_str().unwrap().contains("MAX_WIRE_BYTES"));
        assert!(!stream["text"].as_str().unwrap().contains("MAX_FRAMES"));
    }
    #[test]
    fn bundled_source_ranges_are_bounded_and_checkout_drift_is_explicit() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join(PATHS[0]);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(&path, SOURCES[0].1).unwrap();
        let q = r#"{"source":"core","lineCount":3}"#;
        assert_eq!(
            source(q, Some(dir.path())).unwrap()["checkoutStatus"],
            "matches"
        );
        std::fs::write(&path, "changed").unwrap();
        let drift = source(q, Some(dir.path())).unwrap();
        assert_eq!(drift["checkoutStatus"], "mismatch");
        assert!(drift["text"].as_str().unwrap().contains("1\t"));
        assert!(source(r#"{"source":"../secret"}"#, None).is_err());
        assert!(source(r#"{"source":"core","lineCount":121}"#, None).is_err());
        assert!(source(r#"{"source":"core","startLine":999999}"#, None).is_err());
        assert!(source(r#"{"source":"core","startLine":0}"#, None).is_err());
    }
    #[test]
    fn local_inventory_needs_no_connection_and_hides_private_roots() {
        let store = Arc::new(SqliteStore::open(std::path::Path::new(":memory:")).unwrap());
        store.create("side").unwrap();
        let engine = Engine::new(
            store,
            Arc::new(connection::testing::MemoryCredentials::default()),
        )
        .unwrap();
        let value = engine
            .call(Command::HarnessInventory {
                session: Some("side".into()),
            })
            .unwrap();
        assert_eq!(value["configured"], false);
        assert!(value["tools"].as_array().unwrap().is_empty());
        assert_eq!(value["contextWindowTokens"], 131072);
        assert!(!value.to_string().contains("apiKey"));
    }
}
