//! Run-local resource authority comes only from the prepared skill sources.
use super::bundles::Cache;
use dolores_core::{
    skill_packages::{valid_digest, validate_path, Snapshot},
    SkillScope, SkillSource, ToolCall, ToolPlugin, ToolRequest, ToolSpec,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::{
    path::{Path, PathBuf},
    sync::Mutex,
};
use tokio_util::sync::CancellationToken;

pub(crate) fn spec() -> ToolSpec {
    ToolSpec {
        name: "read_skill_resource".into(),
        description: "Read verified resources from a skill package selected in this request. Use the host-provided skill, scope and packageId. action=list returns paged file metadata; action=read requires one exact portable relative path and returns UTF-8 text with nextOffset byte continuation. Binary files return metadata only. offset must be a UTF-8 boundary. No arbitrary root, script execution, dependency installation or tool grant. Host approval/access policy still applies. Changed resources refuse without sharing bytes; inspect Settings → Skills → Packages for recovery.".into(),
        parameters: json!({"type":"object","properties":{"skill":{"type":"string"},"scope":{"type":"string","enum":["project","global"]},"packageId":{"type":"string"},"action":{"type":"string","enum":["list","read"]},"path":{"type":"string"},"offset":{"type":"integer","minimum":0},"cursor":{"type":"integer","minimum":0},"maxBytes":{"type":"integer","minimum":1024,"maximum":16384}},"required":["skill","scope","packageId","action"],"additionalProperties":false}),
    }
}
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Query {
    skill: String,
    scope: SkillScope,
    package_id: String,
    action: String,
    path: Option<String>,
    #[serde(default)]
    offset: usize,
    #[serde(default)]
    cursor: usize,
    #[serde(default = "default_bytes")]
    max_bytes: usize,
}
fn default_bytes() -> usize {
    8192
}
fn parse(text: &str) -> Result<Query, String> {
    if text.len() > 4096 {
        return Err("Resource arguments exceed 4 KiB. Narrow the request.".into());
    }
    let q: Query = serde_json::from_str(text).map_err(|_| {
        "Invalid package-resource arguments. Use a selected skill, scope and exact package ID."
    })?;
    if !dolores_core::valid_skill_name(&q.skill)
        || !valid_digest(&q.package_id)
        || !(1024..=16384).contains(&q.max_bytes)
        || !matches!(q.action.as_str(), "list" | "read")
        || (q.action == "read") != q.path.is_some()
        || (q.action == "list" && q.offset != 0)
        || (q.action == "read" && q.cursor != 0)
    {
        return Err(
            "Invalid package-resource range or identity. List the selected package first.".into(),
        );
    }
    if let Some(path) = &q.path {
        validate_path(path)?;
    }
    Ok(q)
}
pub(crate) struct Reader {
    profile: PathBuf,
    sources: Mutex<Option<Vec<SkillSource>>>,
}
impl Reader {
    pub(crate) fn new(profile: &Path) -> Self {
        Self {
            profile: profile.into(),
            sources: Mutex::new(None),
        }
    }
    pub(crate) fn bind(&self, sources: &[SkillSource]) -> Result<(), String> {
        let packages = sources
            .iter()
            .filter(|s| s.package.is_some())
            .cloned()
            .collect::<Vec<_>>();
        for s in &packages {
            s.package.as_ref().unwrap().validate()?;
        }
        let mut slot = self
            .sources
            .lock()
            .map_err(|_| "Selected package sources unavailable.")?;
        if slot.as_ref().is_some_and(|old| old != &packages) {
            return Err("Selected skill packages changed during this run. Keep progress and start a fresh request.".into());
        }
        *slot = Some(packages);
        Ok(())
    }
    fn selected(&self, q: &Query) -> Result<Snapshot, String> {
        self.sources.lock().map_err(|_|"Selected packages unavailable.")?.as_ref()
            .and_then(|s|s.iter().find(|s|s.name==q.skill && s.scope==q.scope && s.package.as_ref().is_some_and(|p|p.id==q.package_id)))
            .and_then(|s|s.package.clone()).ok_or("Package is not selected in this request. Select its reviewed skill in the composer and send a fresh message.".into())
    }
    fn checked(&self, q: &Query) -> Result<(Cache, Snapshot), String> {
        let snapshot = self.selected(q)?;
        let cache = Cache::open(&self.profile)?;
        let (saved, _) = cache.checked(&snapshot.id)?;
        if saved.package != snapshot || saved.document.name != q.skill {
            return Err("Retained package differs from this request's snapshot. Inspect Packages and select a healthy version.".into());
        }
        if q.path
            .as_ref()
            .is_some_and(|path| !snapshot.resources.iter().any(|r| &r.path == path))
        {
            return Err(
                "This file is not in the selected package. List its resources first.".into(),
            );
        }
        Ok((cache, snapshot))
    }
    fn target(q: &Query) -> String {
        format!(
            "{} skill {} · {} · package {}",
            if q.scope == SkillScope::Global {
                "Global"
            } else {
                "Project"
            },
            q.skill,
            q.path.as_deref().unwrap_or("list resources"),
            q.package_id
        )
    }
    fn result(&self, q: &Query) -> Result<Value, String> {
        let (cache, snapshot) = self.checked(q)?;
        let mut value = json!({"skill":q.skill,"scope":q.scope,"packageId":q.package_id,"contentDigest":snapshot.content_digest});
        let limit = q.max_bytes.min(dolores_core::MAX_TOOL_BYTES);
        if q.action == "list" {
            if q.cursor > snapshot.resources.len() {
                return Err("Resource cursor is unavailable. List with cursor 0.".into());
            }
            let mut entries = vec![];
            let mut next = q.cursor;
            for resource in snapshot.resources.iter().skip(q.cursor) {
                entries.push(json!(resource));
                value["resources"] = json!(entries);
                value["nextCursor"] = json!(next + 1);
                if value.to_string().len() > limit {
                    entries.pop();
                    break;
                }
                next += 1;
            }
            if entries.is_empty() && next < snapshot.resources.len() {
                return Err("Resource metadata needs a larger maxBytes allowance.".into());
            }
            value["resources"] = json!(entries);
            value["nextCursor"] = if next < snapshot.resources.len() {
                json!(next)
            } else {
                Value::Null
            };
        } else {
            let path = q.path.as_ref().unwrap();
            let bytes = cache.resource(&snapshot, path)?;
            let resource = snapshot.resources.iter().find(|r| &r.path == path).unwrap();
            value["resource"] = json!(resource);
            let text = std::str::from_utf8(&bytes)
                .ok()
                .filter(|t| !t.contains('\0'));
            if let Some(text) = text {
                if q.offset > text.len() || !text.is_char_boundary(q.offset) {
                    return Err("Resource offset must be an available UTF-8 byte boundary. Use the returned nextOffset.".into());
                }
                value["format"] = json!("utf8");
                value["offset"] = json!(q.offset);
                let candidates = text[q.offset..]
                    .char_indices()
                    .map(|(i, c)| q.offset + i + c.len_utf8())
                    .take_while(|end| *end - q.offset <= limit)
                    .collect::<Vec<_>>();
                let (mut low, mut high) = (0, candidates.len());
                while low < high {
                    let middle = low + (high - low) / 2;
                    let end = candidates[middle];
                    value["text"] = json!(&text[q.offset..end]);
                    value["nextOffset"] = if end < text.len() {
                        json!(end)
                    } else {
                        Value::Null
                    };
                    if value.to_string().len() <= limit {
                        low = middle + 1;
                    } else {
                        high = middle;
                    }
                }
                let end = low.checked_sub(1).map_or(q.offset, |i| candidates[i]);
                value["text"] = json!(&text[q.offset..end]);
                value["nextOffset"] = if end < text.len() {
                    json!(end)
                } else {
                    Value::Null
                };
                if end == q.offset && end < text.len() {
                    return Err("Resource text needs a larger maxBytes allowance.".into());
                }
            } else {
                if q.offset != 0 {
                    return Err("Binary resources have metadata only; use offset 0.".into());
                }
                value["format"] = json!("binary");
                value["note"]=json!("Verified metadata only. Export the original ZIP to inspect this asset with an appropriate, separately authorized tool.");
            }
        }
        if value.to_string().len() > limit {
            return Err("Resource metadata exceeds maxBytes. Increase the allowance.".into());
        }
        Ok(value)
    }
}
#[async_trait::async_trait]
impl ToolPlugin for Reader {
    fn spec(&self) -> ToolSpec {
        spec()
    }
    fn prepare(&self, call: &ToolCall) -> Result<ToolRequest, String> {
        if call.name != "read_skill_resource" {
            return Err("Invalid package resource tool.".into());
        }
        let q = parse(&call.arguments)?;
        self.checked(&q)?;
        Ok(ToolRequest {
            call_id: call.id.clone(),
            name: call.name.clone(),
            target: Self::target(&q),
            query: Some(serde_json::to_string(&q).map_err(|_| "Resource query unavailable.")?),
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
            return Err(crate::stopped());
        }
        let q = parse(
            request
                .query
                .as_deref()
                .ok_or("Resource query unavailable.")?,
        )?;
        if request.name != "read_skill_resource"
            || request.target != Self::target(&q)
            || request.diff.is_some()
            || request.command.is_some()
            || request.mcp.is_some()
        {
            return Err("Package resource request changed. Prepare it again.".into());
        }
        let result = self.result(&q)?.to_string();
        if cancel.is_cancelled() {
            return Err(crate::stopped());
        }
        Ok(result)
    }
}

#[cfg(all(test, windows))]
mod tests {
    use super::*;
    use dolores_core::{skill_packages::Installed, ProjectSkill, SessionStore, SkillVersion};
    use std::{
        io::{Cursor, Write},
        sync::Arc,
    };
    fn install(profile: &Path, text: &str) -> Installed {
        let mut writer = zip::ZipWriter::new(Cursor::new(Vec::new()));
        let options = zip::write::SimpleFileOptions::default()
            .compression_method(zip::CompressionMethod::Deflated);
        for (path,bytes) in [("SKILL.md",b"---\nname: review\ndescription: Review public code\n---\nRead references/check.md.".as_slice()),("references/check.md",text.as_bytes()),("assets/image.bin",b"\0\xff\x01".as_slice()),("scripts/check.py",b"raise RuntimeError('must not execute')".as_slice())] {
            writer.start_file(path,options).unwrap();writer.write_all(bytes).unwrap();
        }
        Cache::open(profile)
            .unwrap()
            .publish("review", &writer.finish().unwrap().into_inner())
            .unwrap()
    }
    fn skill(package: &Installed, scope: SkillScope) -> ProjectSkill {
        ProjectSkill {
            scope,
            name: "review".into(),
            revision: 1,
            enabled: true,
            versions: vec![SkillVersion {
                package: Some(package.package.clone()),
                evaluation: None,
                version: 1,
                reviewed_at: 1,
                document: package.document.clone(),
                rollback_from: None,
            }],
        }
    }
    fn sources(packages: &[ProjectSkill], input: &str) -> Vec<SkillSource> {
        dolores_core::prepare_selected_skill_context(
            dolores_core::preview_context(vec![], input).unwrap(),
            packages,
            &[],
        )
        .unwrap()
        .1
    }
    fn query(p: &Installed, scope: SkillScope, path: Option<&str>) -> Value {
        let mut q = json!({"skill":"review","scope":scope,"packageId":p.package.id,"action":"list","maxBytes":1024});
        if let Some(path) = path {
            q["action"] = json!("read");
            q["path"] = json!(path);
        }
        q
    }
    fn prepare(reader: &Reader, q: &Value) -> Result<ToolRequest, String> {
        reader.prepare(&ToolCall {
            id: "read-1".into(),
            name: "read_skill_resource".into(),
            arguments: q.to_string(),
        })
    }
    async fn invoke(reader: &Reader, q: &Value) -> Result<String, String> {
        reader
            .invoke(&prepare(reader, q)?, CancellationToken::new())
            .await
    }
    #[tokio::test]
    async fn selected_scope_identity_and_compaction_binding_cannot_substitute_another_package() {
        let profile = tempfile::tempdir().unwrap();
        let global = install(profile.path(), "Global public reference 世界");
        let project = install(profile.path(), "Project public reference");
        let reader = Reader::new(profile.path());
        let selected = sources(
            &[
                skill(&global, SkillScope::Global),
                skill(&project, SkillScope::Project),
            ],
            "Use review",
        );
        assert_eq!(selected.len(), 1);
        reader.bind(&selected).unwrap();
        reader.bind(&selected).unwrap();
        assert!(reader
            .bind(&sources(
                &[skill(&global, SkillScope::Global)],
                "Use review"
            ))
            .unwrap_err()
            .contains("changed during"));
        assert!(prepare(&reader, &query(&global, SkillScope::Global, None))
            .unwrap_err()
            .contains("not selected"));
        let result = invoke(
            &reader,
            &query(&project, SkillScope::Project, Some("references/check.md")),
        )
        .await
        .unwrap();
        assert!(result.contains("Project public reference"));
        assert!(!result.contains("Global public reference"));
        let empty = Reader::new(profile.path());
        empty
            .bind(&sources(
                &[skill(&global, SkillScope::Global)],
                "Compose a poem",
            ))
            .unwrap();
        assert!(prepare(&empty, &query(&global, SkillScope::Global, None))
            .unwrap_err()
            .contains("not selected"));
        assert!(prepare(
            &reader,
            &query(&project, SkillScope::Project, Some("../outside.txt"))
        )
        .is_err());
        assert!(prepare(
            &reader,
            &query(&project, SkillScope::Project, Some("C:/outside.txt"))
        )
        .is_err());
        assert!(prepare(
            &reader,
            &query(&project, SkillScope::Project, Some("references/missing.md"))
        )
        .unwrap_err()
        .contains("not in the selected"));
        let mut unknown = query(&project, SkillScope::Project, None);
        unknown["root"] = json!("C:/private");
        assert!(prepare(&reader, &unknown).is_err());
    }
    #[tokio::test]
    async fn utf8_and_escaped_text_and_metadata_page_without_exceeding_the_result_allowance() {
        let profile = tempfile::tempdir().unwrap();
        let text = "世界🙂\n\"\\\u{1}\t".repeat(150);
        let package = install(profile.path(), &text);
        let reader = Reader::new(profile.path());
        reader
            .bind(&sources(
                &[skill(&package, SkillScope::Global)],
                "Use review",
            ))
            .unwrap();
        let mut q = query(&package, SkillScope::Global, Some("references/check.md"));
        let mut reconstructed = String::new();
        let mut steps = 0;
        loop {
            let result = invoke(&reader, &q).await.unwrap();
            assert!(result.len() <= 1024);
            let result: Value = serde_json::from_str(&result).unwrap();
            reconstructed.push_str(result["text"].as_str().unwrap());
            steps += 1;
            if result["nextOffset"].is_null() {
                break;
            }
            let offset = result["nextOffset"].as_u64().unwrap();
            assert!(offset > q["offset"].as_u64().unwrap_or(0));
            q["offset"] = json!(offset);
            assert!(steps < 32);
        }
        assert_eq!(reconstructed, text);
        assert!(steps > 1);
        q["offset"] = json!(1);
        assert!(invoke(&reader, &q)
            .await
            .unwrap_err()
            .contains("UTF-8 byte boundary"));
        let mut listing = query(&package, SkillScope::Global, None);
        let mut found = vec![];
        loop {
            let page = invoke(&reader, &listing).await.unwrap();
            assert!(page.len() <= 1024);
            let page: Value = serde_json::from_str(&page).unwrap();
            found.extend(
                page["resources"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(|r| r["path"].as_str().unwrap().to_owned()),
            );
            if page["nextCursor"].is_null() {
                break;
            }
            listing["cursor"] = page["nextCursor"].clone();
        }
        assert_eq!(found.len(), package.package.resources.len());
        let binary: Value = serde_json::from_str(
            &invoke(
                &reader,
                &query(&package, SkillScope::Global, Some("assets/image.bin")),
            )
            .await
            .unwrap(),
        )
        .unwrap();
        assert_eq!(binary["format"], "binary");
        assert!(binary.get("text").is_none());
        let mut wrong = query(&package, SkillScope::Global, None);
        wrong["maxBytes"] = json!(512);
        assert!(prepare(&reader, &wrong).is_err());
        let mut large = query(&package, SkillScope::Global, None);
        large["extra"] = json!("x".repeat(4096));
        assert!(prepare(&reader, &large).unwrap_err().contains("4 KiB"));
    }
    #[tokio::test]
    async fn changed_resources_after_approval_cancel_and_tampered_request_preserve_recovery() {
        let profile = tempfile::tempdir().unwrap();
        let package = install(profile.path(), "Public original");
        let reader = Reader::new(profile.path());
        reader
            .bind(&sources(
                &[skill(&package, SkillScope::Project)],
                "Use review",
            ))
            .unwrap();
        let q = query(&package, SkillScope::Project, Some("references/check.md"));
        let request = prepare(&reader, &q).unwrap();
        let file = Path::new(&package.package.root).join("references/check.md");
        std::fs::write(&file, "Changed private bytes must not reach the model").unwrap();
        let error = reader
            .invoke(&request, CancellationToken::new())
            .await
            .unwrap_err();
        assert!(!error.contains("private bytes"));
        assert!(Cache::open(profile.path())
            .unwrap()
            .checked_archive(&package.package.id)
            .is_ok());
        std::fs::write(&file, "Public original").unwrap();
        assert!(reader
            .invoke(&request, CancellationToken::new())
            .await
            .unwrap()
            .contains("Public original"));
        let cancel = CancellationToken::new();
        cancel.cancel();
        assert!(reader
            .invoke(&request, cancel)
            .await
            .unwrap_err()
            .contains("stopped"));
        let mut bad = request.clone();
        bad.target = "Other package".into();
        assert!(reader
            .invoke(&bad, CancellationToken::new())
            .await
            .unwrap_err()
            .contains("request changed"));
        let policy = dolores_core::PermissionPolicy::default();
        assert!(!policy.automatic(&request, 1));
        let policy = dolores_core::PermissionPolicy {
            mode: dolores_core::PermissionMode::FullAccess,
            ..Default::default()
        };
        assert!(policy.automatic(&request, 1));
    }
    #[test]
    fn planning_and_registry_keep_resource_reads_separate_from_execution_authority() {
        assert!(dolores_core::planning::inspection_tool(
            "read_skill_resource"
        ));
        assert!(!dolores_core::planning::inspection_tool("run_command"));
        let profile = tempfile::tempdir().unwrap();
        let store = Arc::new(crate::SqliteStore::open(&profile.path().join("state.db")).unwrap());
        store.create("task").unwrap();
        let mut engine = crate::Engine::new(
            store,
            Arc::new(crate::connection::testing::MemoryCredentials::default()),
        )
        .unwrap();
        engine.workspace_directory = Some(profile.path().join("workspaces"));
        engine
            .extension_registry(Some("task"))
            .unwrap()
            .pin_tool(Arc::new(Reader::new(profile.path())))
            .unwrap();
    }
}
