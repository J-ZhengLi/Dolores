use super::*;
use dolores_core::{ToolCall, ToolPlugin, ToolRequest, ToolSpec};
use std::io::Read;

#[path = "source_bundle.rs"]
pub(super) mod bundle;
const ALIASES: &[(&str, &str)] = &[
    ("core", "crates/dolores-core/src/lib.rs"),
    ("agent", "crates/dolores-core/src/agent.rs"),
    ("host", "crates/dolores-flutter-bridge/src/lib.rs"),
    ("files", "crates/dolores-tools-fs/src/lib.rs"),
    ("provider", "crates/dolores-provider-openai/src/agent.rs"),
    (
        "subagents",
        "crates/dolores-flutter-bridge/src/subagents.rs",
    ),
    (
        "provider_stream",
        "crates/dolores-provider-openai/src/agent_stream.rs",
    ),
    (
        "provider_settings",
        "crates/dolores-provider-openai/src/lib.rs",
    ),
    ("recovery", "crates/dolores-flutter-bridge/src/recovery.rs"),
    (
        "attachments",
        "crates/dolores-flutter-bridge/src/attachments.rs",
    ),
    ("task_budget", "crates/dolores-core/src/task_budget.rs"),
    (
        "failure_watchdog",
        "crates/dolores-core/src/agent_watchdog.rs",
    ),
];
pub(super) fn resolve(name: &str) -> Result<&'static bundle::Source, String> {
    bundle::find(
        ALIASES
            .iter()
            .find(|(key, _)| *key == name)
            .map_or(name, |(_, path)| path),
    )
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
        description: "Diagnose Dolores read-only, without changing the user's project. Empty arguments show this build's inventory and private-data-free failure evidence. action=list pages the matching source manifest; action=search finds literal text or identifier tokens (symbol=true), with nextCursor continuation; source plus a startLine/lineCount reads up to 2048 lines with nextLine continuation. Use sourceId/bundleId to refuse stale navigation. Source comes from the running build, not the project. Returned JSON fits the current tool-result allowance. Historical build or missing telemetry stays unknown. Requires host approval; no edit or update authority.".into(),
        parameters: json!({"type":"object","properties":{"action":{"type":"string","enum":["list","search","read"]},"source":{"type":"string","description":"Bundled relative path or legacy source alias"},"startLine":{"type":"integer","minimum":1},"lineCount":{"type":"integer","minimum":1,"maximum":2048},"query":{"type":"string","maxLength":128},"symbol":{"type":"boolean"},"cursor":{"type":"integer","minimum":0},"sourceId":{"type":"string"},"bundleId":{"type":"string"},"maxBytes":{"type":"integer","minimum":1024,"maximum":32768}},"additionalProperties":false}),
    }
}
#[derive(Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Query {
    action: Option<String>,
    source: Option<String>,
    query: Option<String>,
    source_id: Option<String>,
    bundle_id: Option<String>,
    #[serde(default)]
    cursor: usize,
    #[serde(default)]
    symbol: bool,
    #[serde(default = "one")]
    start_line: usize,
    #[serde(default = "default_lines")]
    line_count: usize,
    #[serde(default = "default_bytes")]
    max_bytes: usize,
}
fn one() -> usize {
    1
}
fn default_lines() -> usize {
    256
}
fn default_bytes() -> usize {
    32768
}
fn parse(text: &str) -> Result<Query, String> {
    let q: Query =
        serde_json::from_str(text).map_err(|_| "Invalid harness inspection arguments.")?;
    if q.start_line == 0
        || !(1..=2048).contains(&q.line_count)
        || !(1024..=32768).contains(&q.max_bytes)
        || q.query
            .as_ref()
            .is_some_and(|s| s.is_empty() || s.len() > 128 || s.chars().any(char::is_control))
        || q.action
            .as_ref()
            .is_some_and(|s| !matches!(s.as_str(), "list" | "search" | "read"))
    {
        return Err(
            "Use 1–2048 lines, 1024–32768 bytes and a nonempty search up to 128 bytes.".into(),
        );
    }
    if q.bundle_id.as_deref().is_some_and(|id| id != bundle::ID) {
        return Err(
            "Source bundle changed. Keep the diagnosis; list the running build before continuing."
                .into(),
        );
    }
    if let Some(name) = &q.source {
        let source = resolve(name)?;
        if q.source_id.as_deref().is_some_and(|id| id != source.id) {
            return Err(
                "Source identity changed. Keep the diagnosis and request fresh matching source."
                    .into(),
            );
        }
    } else if q.source_id.is_some() {
        return Err("Choose a source for its identity check.".into());
    }
    if q.action.as_deref() == Some("search") && q.query.is_none() {
        return Err("Choose text or a symbol to search.".into());
    }
    Ok(q)
}
fn bound(q: &Query) -> usize {
    q.max_bytes.min(dolores_core::MAX_TOOL_BYTES)
}
fn clipped(text: &str) -> &str {
    let mut end = text.len().min(256);
    while !text.is_char_boundary(end) {
        end -= 1;
    }
    &text[..end]
}
pub fn source(query: &str, checkout: Option<&std::path::Path>) -> Result<Value, String> {
    let q = parse(query)?;
    match q.action.as_deref() {
        Some("list") => {
            let files: Vec<_> = bundle::SOURCES
                .iter()
                .filter(|s| {
                    q.query
                        .as_deref()
                        .is_none_or(|filter| s.path.contains(filter))
                })
                .collect();
            if q.cursor > files.len() {
                return Err("Source list cursor is unavailable. Start with cursor 0.".into());
            }
            let mut entries = vec![];
            let mut used = 1024;
            for s in files.iter().skip(q.cursor).take(100) {
                let item = bundle::entry(s);
                let bytes = item.to_string().len() + 1;
                if used + bytes > bound(&q) {
                    break;
                }
                entries.push(item);
                used += bytes;
            }
            if entries.is_empty() && q.cursor < files.len() {
                return Err(
                    "Source page cannot fit. Increase maxBytes and retry; evidence remains.".into(),
                );
            }
            let next = q.cursor + entries.len();
            Ok(
                json!({"bundle":bundle::summary(),"entries":entries,"nextCursor":if next<files.len(){Some(next)}else{None},"total":files.len(),"deliveryBytes":bound(&q)}),
            )
        }
        Some("search") => {
            let needle = q.query.as_deref().unwrap();
            let mut hits = vec![];
            let mut scanned = 0;
            let mut used = 1024;
            let mut more = false;
            'files: for s in bundle::SOURCES.iter().filter(|s| {
                q.source
                    .as_deref()
                    .is_none_or(|name| resolve(name).is_ok_and(|chosen| chosen.path == s.path))
            }) {
                if scanned + s.lines <= q.cursor {
                    scanned += s.lines;
                    continue;
                }
                let text = bundle::text(s)?;
                for (index, line) in text.lines().enumerate() {
                    scanned += 1;
                    if scanned <= q.cursor {
                        continue;
                    }
                    let matched = if q.symbol {
                        line.split(|c: char| !c.is_alphanumeric() && c != '_')
                            .any(|token| token == needle)
                    } else {
                        line.contains(needle)
                    };
                    if !matched {
                        continue;
                    }
                    let item = json!({"source":s.path,"sourceId":s.id,"line":index+1,"text":clipped(line),"shortened":line.len()>256});
                    let bytes = item.to_string().len() + 1;
                    if used + bytes > bound(&q) || hits.len() == 64 {
                        scanned -= 1;
                        more = true;
                        break 'files;
                    }
                    used += bytes;
                    hits.push(item);
                }
            }
            if more && hits.is_empty() {
                return Err(
                    "Search match cannot fit. Increase maxBytes and retry; diagnosis remains."
                        .into(),
                );
            }
            Ok(
                json!({"bundleId":bundle::ID,"matches":hits,"nextCursor":if more{Some(scanned)}else{None},"searchKind":if q.symbol{"identifier token; not an AST or call graph"}else{"literal text"},"deliveryBytes":bound(&q)}),
            )
        }
        _ => {
            let s = resolve(
                q.source
                    .as_deref()
                    .ok_or("Choose a bundled source name or list action.")?,
            )?;
            let text = bundle::text(s)?;
            let checkout_status = checkout.map(|root| {
                let candidate = root.join(s.path);
                match (root.canonicalize(), candidate.canonicalize()) {
                    (Ok(root), Ok(path)) if path.starts_with(&root) => {
                        if checkout_matches(&path, &text) {
                            "matches"
                        } else {
                            "mismatch"
                        }
                    }
                    _ => "unavailable",
                }
            });
            let mut lines = String::new();
            let mut next = q.start_line;
            let mut used = 1024;
            for (n, line) in text
                .lines()
                .enumerate()
                .skip(q.start_line - 1)
                .take(q.line_count)
            {
                let item = format!("{}\t{line}\n", n + 1);
                let bytes = serde_json::to_string(&item).unwrap().len() - 2;
                if used + bytes > bound(&q) {
                    break;
                }
                used += bytes;
                lines.push_str(&item);
                next = n + 2;
            }
            if lines.is_empty() {
                return Err("Source range cannot fit or is unavailable. Choose an earlier range or increase maxBytes; evidence remains.".into());
            }
            Ok(
                json!({"source":q.source,"path":s.path,"sourceId":s.id,"bundleId":bundle::ID,"origin":"bundled with running build","checkoutStatus":checkout_status,"startLine":q.start_line,"nextLine":next,"hasMore":next<=s.lines,"totalLines":s.lines,"text":lines,"deliveryBytes":bound(&q)}),
            )
        }
    }
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
            let matching = events
                .iter()
                .find(|e| e.kind == "implementation")
                .and_then(|e| e.data["bundleId"].as_str());
            let measurements = events
                .iter()
                .rev()
                .filter(|e| e.kind == "modelTelemetry")
                .find_map(|e| {
                    serde_json::from_value::<dolores_core::StreamMeasurements>(
                        e.data["measurements"].clone(),
                    )
                    .ok()
                });
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
                "The older transport guard counted repeated provider metadata toward 2 MiB. The record does not contain decoded output or stream activity measurements. The current build instead bounds decoded fields and traffic without progress."
            } else if error.starts_with("Provider stream sent 2 MiB without new response data") {
                "The provider sent excessive traffic without adding reply, reasoning or tool-call data. No incomplete tool call was executed."
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
                "sourceMatch":match matching { Some(id) if id==bundle::ID=>"matches", Some(_)=>"mismatch",None=>"unknown; older build has no bundle identity" },
                "latestMeasurements":measurements,
                "telemetryCoverage":"Final counters for completed or failed instrumented tool-model requests. Older adapters and externally interrupted requests may have no counters; absent values are unknown, never zero.",
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
            if self.workspace_directory.is_some() {
                tools.push(crate::harness_repair::spec());
                tools.push(crate::repair_evaluation::spec());
            }
            #[cfg(windows)]
            if self.workspace_directory.is_some() {
                tools.push(crate::native_build::spec());
            }
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
        let sources: Vec<_> = ALIASES
            .iter()
            .map(|(key, path)| {
                let mut entry = bundle::entry(resolve(path).unwrap());
                entry["id"] = json!(key);
                entry
            })
            .collect();
        let images = configured
            && self
                .store
                .image_models(&preferences.base_url)?
                .contains(&preferences.model);
        let recent = self.recent_harness_failures(session)?;
        let mut inventory = json!({"version":env!("CARGO_PKG_VERSION"),"revision":env!("DOLORES_BUILD_REVISION"),"sourceIdentity":"bundled file identities; revision may include local source changes","workspace":workspace.map(|w| w.kind),"configured":configured,"model":preferences.model,"modelCapabilities":{"input":if images {vec!["text","image"]} else {vec!["text"]},"tools":"adapter supports function calls; selected model support not established","images":if images {"image input configured; comprehension not established"} else {"disabled; enable a capable model in Model connection"}},"attachments":{"formats":["text/plain","image/png","image/jpeg"],"draftFiles":dolores_core::MAX_DRAFT_ATTACHMENTS,"textBytes":dolores_core::MAX_TEXT_ATTACHMENT_BYTES,"imageBytes":dolores_core::MAX_IMAGE_ATTACHMENT_BYTES,"storeBytes":dolores_core::MAX_ATTACHMENT_STORE_BYTES},"approval":format!("{:?}",effective.permissions.mode),"containment":"file tools use folder capabilities; commands/MCP have user-account permissions","selfUpdate":"not available","contextWindowTokens":capacity.unwrap_or(dolores_core::DEFAULT_CONTEXT_WINDOW_TOKENS),"contextOrigin":if capacity.is_some(){"model override"}else{"128K default"},"effectiveSettings":effective,"requestSettings":effective.request,"automaticCheckpoints":{"modelCalls":effective.task.model_limit(),"toolOperations":effective.task.tool_limit(),"repeatedFailures":2,"consecutiveFailures":6},"limits":{"modelCalls":effective.task.model_calls,"toolOperations":effective.task.tool_calls,"taskSegments":effective.task.segments,"taskDeadlineSeconds":effective.task.elapsed_seconds,"inputBytes":dolores_core::MAX_INPUT_BYTES,"contextBytes":dolores_core::MAX_CONTEXT_BYTES,"maxSourceLines":2048},"extensionApi":dolores_core::HOST_EXTENSION_API,"extensions":registry.entries,"hookOrder":registry.order,"adapters":[self.store.descriptor(),connection.descriptor(),self.mcp_credentials.descriptor()],"tools":catalog,"unavailableReason":if working {""}else{"No working folder. Start a project or temporary working session for tool use."},"sources":sources});
        inventory["sourceBundle"] = bundle::summary();
        #[cfg(windows)]
        {
            inventory["selfUpdate"] = json!("Reviewed Windows Rust build; installation and Restore require separate direct user review in the idle normal app.");
            inventory["nativeRepair"] = json!({"buildAvailable":working && self.workspace_directory.is_some(),"scope":"provider implementation or core command-outcome/task-budget handling; unchanged existing tests","qualification":"complete frozen baseline failure, candidate success and library regressions","installation":"Settings > Advanced > Native repairs; exact user review","unattended":false,"osContainment":false});
        }
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
        let value = if q.source.is_some() || q.action.is_some() {
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
        assert!(stream["text"]
            .as_str()
            .unwrap()
            .contains("MAX_IDLE_WIRE_BYTES"));
        assert!(!stream["text"].as_str().unwrap().contains("MAX_FRAMES"));
    }
    #[test]
    fn bundled_source_ranges_are_bounded_and_checkout_drift_is_explicit() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join(ALIASES[0].1);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(&path, bundle::text(resolve("core").unwrap()).unwrap()).unwrap();
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
        assert!(source(r#"{"source":"core","lineCount":2049}"#, None).is_err());
        assert!(source(r#"{"source":"core","startLine":999999}"#, None).is_err());
        assert!(source(r#"{"source":"core","startLine":0}"#, None).is_err());
    }
    #[test]
    fn source_navigation_pages_searches_and_refuses_stale_identity_without_project_access() {
        let listing = source(
            r#"{"action":"list","query":"dolores-core","maxBytes":2048}"#,
            None,
        )
        .unwrap();
        assert!(listing.to_string().len() <= 2048);
        assert!(listing["nextCursor"].is_number());
        let module = resolve("core").unwrap();
        let read = source(
            &json!({"source":"core","lineCount":2048,"sourceId":module.id,"bundleId":bundle::ID})
                .to_string(),
            None,
        )
        .unwrap();
        assert!(read["nextLine"].as_u64().unwrap() > 120);
        assert!(read.to_string().len() <= dolores_core::MAX_TOOL_BYTES);
        let next = source(
            &json!({"source":"core","startLine":read["nextLine"],"lineCount":30}).to_string(),
            None,
        )
        .unwrap();
        assert_eq!(next["startLine"], read["nextLine"]);
        let search = source(
            r#"{"action":"search","source":"core","query":"ModelProvider","symbol":true}"#,
            None,
        )
        .unwrap();
        assert!(!search["matches"].as_array().unwrap().is_empty());
        assert!(search["matches"]
            .as_array()
            .unwrap()
            .iter()
            .any(|h| h["line"].as_u64().unwrap() > 120));
        assert!(source(r#"{"source":"core","sourceId":"stale"}"#, None)
            .unwrap_err()
            .contains("identity changed"));
        assert!(source(
            r#"{"action":"search","query":"ModelProvider","bundleId":"stale"}"#,
            None
        )
        .unwrap_err()
        .contains("bundle changed"));
        assert!(source(r#"{"source":"C:/example/project/game.html"}"#, None).is_err());
        assert!(source(r#"{"source":"core","maxBytes":1024}"#, None)
            .unwrap_err()
            .contains("evidence remains"));
    }
    #[test]
    fn search_continuation_is_complete_and_corrupt_source_is_never_delivered() {
        let mut cursor = 0;
        let mut matches = vec![];
        loop {
            let page=source(&json!({"action":"search","source":"core","query":"fn ","cursor":cursor,"maxBytes":2048}).to_string(),None).unwrap();
            assert!(page.to_string().len() <= 2048);
            matches.extend(
                page["matches"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(|m| m["line"].as_u64().unwrap()),
            );
            match page["nextCursor"].as_u64() {
                Some(next) => {
                    assert!(next > cursor);
                    cursor = next;
                }
                None => break,
            }
        }
        let text = bundle::text(resolve("core").unwrap()).unwrap();
        let expected: Vec<_> = text
            .lines()
            .enumerate()
            .filter(|(_, line)| line.contains("fn "))
            .map(|(n, _)| (n + 1) as u64)
            .collect();
        assert_eq!(matches, expected);
        let bad = bundle::Source {
            path: "bad",
            id: "sha256:bad",
            lines: 1,
            bytes: 3,
            compressed: b"bad",
        };
        assert!(bundle::text(&bad).is_err());
        assert!(bundle::SOURCES
            .iter()
            .all(|s| !s.path.contains("output/") && !s.path.contains(".env")));
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
    #[cfg(windows)]
    #[test]
    fn working_inventory_reports_reviewed_build_without_installation_authority() {
        let profile = tempfile::tempdir().unwrap();
        let project = tempfile::tempdir().unwrap();
        let store = Arc::new(SqliteStore::open(std::path::Path::new(":memory:")).unwrap());
        let mut engine = Engine::new(
            store,
            Arc::new(connection::testing::MemoryCredentials::default()),
        )
        .unwrap();
        engine.workspace_directory = Some(profile.path().join("workspaces"));
        let session = engine
            .call(
                serde_json::from_value(
                    json!({"command":"createSession","kind":"project","path":project.path()}),
                )
                .unwrap(),
            )
            .unwrap();
        let value = engine
            .harness_inventory(session["session"]["id"].as_str())
            .unwrap();
        assert!(value["tools"]
            .as_array()
            .unwrap()
            .iter()
            .any(|t| t["id"] == "build_harness_repair"));
        assert_eq!(value["nativeRepair"]["buildAvailable"], true);
        assert_eq!(value["nativeRepair"]["unattended"], false);
        assert!(value["selfUpdate"]
            .as_str()
            .unwrap()
            .contains("direct user review"));
        assert!(!value
            .to_string()
            .contains(&project.path().display().to_string()));
    }
}
