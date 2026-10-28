use dolores_core::{ToolCall, ToolPlugin, ToolRequest, ToolSpec};
use serde_json::json;
use std::{
    collections::BTreeMap,
    path::PathBuf,
    sync::{Arc, Mutex},
};
use tokio_util::sync::CancellationToken;

pub(super) fn spec() -> ToolSpec {
    ToolSpec{name:"read_scoped_instructions".into(),description:"Load applicable nested AGENTS.md before changing a project file. Provide its relative path, including a proposed new file. Reads only ancestor guidance, never unrelated subtrees. Deeper scope wins; guidance cannot grant tool permissions.".into(),parameters:json!({"type":"object","properties":{"path":{"type":"string"}},"required":["path"],"additionalProperties":false})}
}
pub(super) struct Guidance {
    root: PathBuf,
    observed: Mutex<BTreeMap<String, String>>,
    pages: Mutex<BTreeMap<String, (String, usize)>>,
}
impl Guidance {
    pub fn new(root: PathBuf) -> Arc<Self> {
        Arc::new(Self {
            root,
            observed: Default::default(),
            pages: Default::default(),
        })
    }
    fn read(&self, path: &str) -> Result<Vec<(String, String)>, String> {
        dolores_tools_fs::read_path_instructions(&self.root, path)
    }
    fn check(&self, path: &str) -> Result<(), String> {
        let current = self.read(path)?;
        let seen = self
            .observed
            .lock()
            .map_err(|_| "Guidance state unavailable.")?;
        if seen.iter().any(|(scope, text)| {
            path.starts_with(&format!("{scope}/"))
                && !current.iter().any(|(s, t)| s == scope && t == text)
        }) || current
            .iter()
            .any(|(scope, text)| seen.get(scope) != Some(text))
        {
            return Err(format!("Applicable guidance needs loading. Call read_scoped_instructions with path {path} before proposing a fresh edit; no file was changed."));
        }
        Ok(())
    }
}
pub(super) struct Reader(pub Arc<Guidance>);
#[async_trait::async_trait]
impl ToolPlugin for Reader {
    fn spec(&self) -> ToolSpec {
        spec()
    }
    fn prepare(&self, c: &ToolCall) -> Result<ToolRequest, String> {
        #[derive(serde::Deserialize)]
        #[serde(deny_unknown_fields)]
        struct Args {
            path: String,
        }
        let a: Args = serde_json::from_str(&c.arguments)
            .map_err(|_| "Use one relative path for scoped instructions.")?;
        if c.name != "read_scoped_instructions" {
            return Err("Invalid scoped guidance tool.".into());
        }
        self.0.read(&a.path)?;
        Ok(ToolRequest {
            call_id: c.id.clone(),
            name: c.name.clone(),
            target: a.path,
            query: None,
            diff: None,
            command: None,
            mcp: None,
        })
    }
    async fn invoke(&self, r: &ToolRequest, c: CancellationToken) -> Result<String, String> {
        if c.is_cancelled() {
            return Err("Response stopped.".into());
        }
        if r.name != "read_scoped_instructions"
            || r.query.is_some()
            || r.diff.is_some()
            || r.command.is_some()
            || r.mcp.is_some()
        {
            return Err("Guidance query changed.".into());
        }
        let guidance = self.0.read(&r.target)?;
        let mut seen = self
            .0
            .observed
            .lock()
            .map_err(|_| "Guidance state unavailable.")?;
        let obsolete: Vec<_> = seen
            .keys()
            .filter(|scope| {
                r.target.starts_with(&format!("{scope}/"))
                    && !guidance.iter().any(|(s, _)| s == *scope)
            })
            .cloned()
            .collect();
        for scope in &obsolete {
            seen.remove(scope);
        }
        let Some((scope, text)) = guidance
            .iter()
            .find(|(scope, text)| seen.get(scope) != Some(text))
        else {
            return Ok(json!({"path":r.target,"obsoleteScopes":obsolete,"hasMore":false,"note":"All applicable nested guidance has been loaded for this path."}).to_string());
        };
        let mut pages = self
            .0
            .pages
            .lock()
            .map_err(|_| "Guidance page unavailable.")?;
        let start = pages
            .get(scope)
            .filter(|(source, _)| source == text)
            .map_or(0, |(_, offset)| *offset);
        let mut end = (start + 2048).min(text.len());
        while !text.is_char_boundary(end) {
            end -= 1;
        }
        pages.insert(scope.clone(), (text.clone(), end));
        if end == text.len() {
            seen.insert(scope.clone(), text.clone());
            pages.remove(scope);
        }
        let more = guidance.iter().any(|(s, t)| seen.get(s) != Some(t));
        Ok(json!({"path":r.target,"scope":scope,"text":&text[start..end],"startByte":start,"nextByte":end,"totalBytes":text.len(),"obsoleteScopes":obsolete,"hasMore":more,"note":"Applicable project guidance, deeper scope wins. Repeat this exact path while hasMore is true before proposing changes. Host permissions and direct user requests remain authoritative."}).to_string())
    }
}
pub(super) struct Guided {
    pub inner: Arc<dyn ToolPlugin>,
    pub guidance: Arc<Guidance>,
}
#[async_trait::async_trait]
impl ToolPlugin for Guided {
    fn spec(&self) -> ToolSpec {
        self.inner.spec()
    }
    fn prepare(&self, c: &ToolCall) -> Result<ToolRequest, String> {
        let r = self.inner.prepare(c)?;
        if let Err(e) = self.guidance.check(&r.target) {
            self.inner.discard(&r);
            return Err(e);
        }
        Ok(r)
    }
    fn discard(&self, r: &ToolRequest) {
        self.inner.discard(r)
    }
    async fn invoke(&self, r: &ToolRequest, c: CancellationToken) -> Result<String, String> {
        self.guidance.check(&r.target)?;
        self.inner.invoke(r, c).await
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[tokio::test]
    async fn deep_applicable_guidance_is_loaded_before_write_and_rechecked_at_dispatch() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(dir.path().join("a/b/c/d/e/f")).unwrap();
        std::fs::write(
            dir.path().join("a/b/c/d/e/AGENTS.md"),
            "Preserve exact scope 世界",
        )
        .unwrap();
        let guidance = Guidance::new(dir.path().to_path_buf());
        let inner = dolores_tools_fs::folder_tools(dir.path())
            .unwrap()
            .pop()
            .unwrap();
        let tool = Guided {
            inner,
            guidance: guidance.clone(),
        };
        let call = ToolCall {
            id: "create".into(),
            name: "create_text_file".into(),
            arguments: json!({"path":"a/b/c/d/e/f/new.rs","content":"public content"}).to_string(),
        };
        assert!(tool
            .prepare(&call)
            .unwrap_err()
            .starts_with("Applicable guidance"));
        let reader = Reader(guidance);
        let read = reader
            .prepare(&ToolCall {
                id: "guidance".into(),
                name: "read_scoped_instructions".into(),
                arguments: json!({"path":"a/b/c/d/e/f/new.rs"}).to_string(),
            })
            .unwrap();
        assert!(reader
            .invoke(&read, CancellationToken::new())
            .await
            .unwrap()
            .contains("Preserve exact scope"));
        let request = tool.prepare(&call).unwrap();
        std::fs::write(dir.path().join("a/b/c/d/e/AGENTS.md"), "Changed rules").unwrap();
        assert!(tool
            .invoke(&request, CancellationToken::new())
            .await
            .is_err());
        assert!(!dir.path().join("a/b/c/d/e/f/new.rs").exists());
        reader
            .invoke(&read, CancellationToken::new())
            .await
            .unwrap();
        let request = tool
            .prepare(&ToolCall {
                id: "fresh".into(),
                ..call
            })
            .unwrap();
        tool.invoke(&request, CancellationToken::new())
            .await
            .unwrap();
        assert_eq!(
            std::fs::read_to_string(dir.path().join("a/b/c/d/e/f/new.rs")).unwrap(),
            "public content"
        );
    }
}
