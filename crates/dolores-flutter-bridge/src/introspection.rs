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
];
const PATHS: &[&str] = &[
    "crates/dolores-core/src/lib.rs",
    "crates/dolores-core/src/agent.rs",
    "crates/dolores-flutter-bridge/src/lib.rs",
    "crates/dolores-tools-fs/src/lib.rs",
    "crates/dolores-provider-openai/src/agent.rs",
    "crates/dolores-flutter-bridge/src/subagents.rs",
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
        description: "Inspect the running Dolores capabilities, limits and version-matched bundled source. Empty arguments show inventory; source is one of core, agent, host, files, provider, subagents. Read-only, bounded, requires approval; never edits the harness or grants permissions.".into(),
        parameters: json!({"type":"object","properties":{"source":{"type":"string","enum":["core","agent","host","files","provider","subagents"]},"startLine":{"type":"integer","minimum":1},"lineCount":{"type":"integer","minimum":1,"maximum":120}},"additionalProperties":false}),
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
            for c in self
                .store
                .mcp_connections(workspace.as_ref().unwrap().root.as_ref().unwrap())?
            {
                tools.extend(c.specs());
            }
        }
        let catalog: Vec<_> = tools.iter().map(|tool| json!({"id":tool.name,"enabled":true,"available":configured,"permission":"host checks current task mode and exact grants","empiricallyTested":"unknown","source": if tool.name == "inspect_harness" { "host" } else if tool.name == "delegate_tasks" {"subagents"} else if tool.name.starts_with("mcp_tool_") { "external server; no bundled source" } else if tool.name == "run_command" { "command adapter; no bundled source" } else { "files" }})).collect();
        let sources: Vec<_> = SOURCES.iter().enumerate().map(|(n, (key, text))| json!({"id":key,"path":PATHS[n],"sourceId":source_id(text),"lines":text.lines().count()})).collect();
        let images = configured
            && self
                .store
                .image_models(&preferences.base_url)?
                .contains(&preferences.model);
        Ok(
            json!({"version":env!("CARGO_PKG_VERSION"),"revision":env!("DOLORES_BUILD_REVISION"),"sourceIdentity":"bundled file identities; revision may include local source changes","workspace":workspace.map(|w| w.kind),"configured":configured,"model":preferences.model,"modelCapabilities":{"input":if images {vec!["text","image"]} else {vec!["text"]},"tools":"adapter supports function calls; selected model support not established","images":if images {"image input configured; comprehension not established"} else {"disabled; enable a capable model in Model connection"}},"attachments":{"formats":["text/plain","image/png","image/jpeg"],"draftFiles":dolores_core::MAX_DRAFT_ATTACHMENTS,"textBytes":dolores_core::MAX_TEXT_ATTACHMENT_BYTES,"imageBytes":dolores_core::MAX_IMAGE_ATTACHMENT_BYTES,"storeBytes":dolores_core::MAX_ATTACHMENT_STORE_BYTES},"approval":format!("{:?}",effective.permissions.mode),"containment":"file tools use folder capabilities; commands/MCP have user-account permissions","selfUpdate":"not available","contextWindowTokens":capacity.unwrap_or(dolores_core::DEFAULT_CONTEXT_WINDOW_TOKENS),"contextOrigin":if capacity.is_some(){"model override"}else{"128K default"},"effectiveSettings":effective,"requestSettings":effective.request,"limits":{"modelCalls":effective.task.model_calls,"toolOperations":effective.task.tool_calls,"taskSegments":effective.task.segments,"taskDeadlineSeconds":effective.task.deadline(effective.request.timeout_seconds),"inputBytes":dolores_core::MAX_INPUT_BYTES,"contextBytes":dolores_core::MAX_CONTEXT_BYTES,"maxSourceLines":120},"extensionApi":dolores_core::HOST_EXTENSION_API,"extensions":registry.entries,"hookOrder":registry.order,"adapters":[self.store.descriptor(),connection.descriptor(),self.mcp_credentials.descriptor()],"tools":catalog,"unavailableReason":if working {""}else{"No working folder. Start a project or temporary working session for tool use."},"sources":sources}),
        )
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
