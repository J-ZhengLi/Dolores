use super::{valid_path, ReadTextFile};
use async_trait::async_trait;
use cap_std::fs::Dir;
use dolores_core::{ToolCall, ToolPlugin, ToolRequest, ToolSpec, MAX_TOOL_BYTES};
use serde::Deserialize;
use serde_json::json;
use std::{collections::VecDeque, io::Read};
use tokio_util::sync::CancellationToken;

const MAX_SEARCH_ENTRIES: usize = 256;
const MAX_SEARCH_FILES: usize = 64;
const MAX_SEARCH_BYTES: usize = 256 * 1024;
const MAX_MATCHES: usize = 30;
const MAX_DEPTH: usize = 4;
#[cfg(test)]
#[path = "discovery_tests.rs"]
mod tests;
pub struct Discover {
    read: ReadTextFile,
    search: bool,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Arguments {
    path: String,
    #[serde(default)]
    query: Option<String>,
}
fn eligible(path: &str) -> bool {
    valid_path(path)
        && !path.split('/').any(|part| {
            matches!(
                part.to_ascii_lowercase().as_str(),
                "node_modules"
                    | "target"
                    | "build"
                    | "dist"
                    | ".dart_tool"
                    | ".venv"
                    | "venv"
                    | "__pycache__"
            )
        })
}
fn valid_query(query: &str) -> bool {
    !query.trim().is_empty() && query.len() <= 256 && !query.chars().any(char::is_control)
}
fn checkpoint(cancel: &CancellationToken) -> Result<(), String> {
    if cancel.is_cancelled() {
        Err("Response stopped.".into())
    } else {
        Ok(())
    }
}
impl Discover {
    pub fn new(read: ReadTextFile, search: bool) -> Self {
        Self { read, search }
    }
    fn name(&self) -> &'static str {
        if self.search {
            "search_text"
        } else {
            "list_folder"
        }
    }
    fn resolve_scope(&self, path: &str) -> Result<String, String> {
        if path == "." {
            return Ok(".".into());
        }
        if !eligible(path) {
            return Err("Folder path is not allowed.".into());
        }
        let target = self.read.resolve(path)?;
        // Discovery never traverses aliases, even to another allowed directory.
        if target != path
            || !self
                .read
                .directory
                .metadata(&target)
                .map_err(|_| "Folder is unavailable.")?
                .is_dir()
        {
            return Err("Only direct folders can be scanned.".into());
        }
        Ok(target)
    }
    fn execute(&self, request: &ToolRequest, cancel: &CancellationToken) -> Result<String, String> {
        checkpoint(cancel)?;
        if request.name != self.name()
            || request.diff.is_some()
            || self.resolve_scope(&request.target)? != request.target
            || (self.search && !request.query.as_deref().is_some_and(valid_query))
            || (!self.search && request.query.is_some())
        {
            return Err("Approved folder request changed.".into());
        }
        let directory = &self.read.directory;
        checkpoint(cancel)?;
        if self.search {
            self.find(
                directory,
                &request.target,
                request.query.as_deref().unwrap(),
                cancel,
            )
        } else {
            self.list(directory, &request.target, cancel)
        }
    }
    fn list(
        &self,
        directory: &Dir,
        scope: &str,
        cancel: &CancellationToken,
    ) -> Result<String, String> {
        let mut inspected = 0;
        let mut truncated = false;
        let mut skipped = 0;
        let entries = entries(
            directory,
            scope,
            512,
            &mut inspected,
            &mut truncated,
            &mut skipped,
            cancel,
        )?;
        let mut result = json!({"entries":[],"inspectedEntries":inspected,"skippedEntries":skipped,"truncated":truncated});
        for (path, is_dir) in entries {
            let item = json!({"path":path,"kind":if is_dir { "folder" } else { "file" }});
            let list = result["entries"].as_array_mut().unwrap();
            if list.len() == 100 {
                result["truncated"] = json!(true);
                break;
            }
            list.push(item);
            if serde_json::to_vec(&result).unwrap().len() > MAX_TOOL_BYTES - 128 {
                result["entries"].as_array_mut().unwrap().pop();
                result["truncated"] = json!(true);
                break;
            }
        }
        Ok(result.to_string())
    }
    fn find(
        &self,
        directory: &Dir,
        scope: &str,
        query: &str,
        cancel: &CancellationToken,
    ) -> Result<String, String> {
        let mut queue = VecDeque::from([(scope.to_string(), 0usize)]);
        let mut inspected = 0;
        let mut truncated = false;
        let mut skipped_entries = 0;
        let mut skipped_files = 0;
        let mut scanned_files = 0;
        let mut attempted_files = 0;
        let mut bytes_read = 0;
        let mut matches = vec![];
        'walk: while let Some((path, depth)) = queue.pop_front() {
            checkpoint(cancel)?;
            let batch = match entries(
                directory,
                &path,
                MAX_SEARCH_ENTRIES,
                &mut inspected,
                &mut truncated,
                &mut skipped_entries,
                cancel,
            ) {
                Ok(batch) => batch,
                Err(error) if cancel.is_cancelled() => return Err(error),
                Err(_) => {
                    skipped_entries += 1;
                    continue;
                }
            };
            for (path, is_dir) in batch {
                checkpoint(cancel)?;
                if is_dir {
                    if depth < MAX_DEPTH {
                        queue.push_back((path, depth + 1));
                    } else {
                        truncated = true;
                    }
                    continue;
                }
                if attempted_files == MAX_SEARCH_FILES || bytes_read == MAX_SEARCH_BYTES {
                    truncated = true;
                    break 'walk;
                }
                // Revalidate relative names and links immediately before opening.
                let canonical = directory
                    .canonicalize(&path)
                    .ok()
                    .and_then(|p| p.to_str().map(|s| s.replace('\\', "/")));
                if canonical.as_deref() != Some(&path) {
                    skipped_files += 1;
                    continue;
                }
                let metadata = match directory.symlink_metadata(&path) {
                    Ok(m) => m,
                    Err(_) => {
                        skipped_files += 1;
                        continue;
                    }
                };
                if !metadata.is_file()
                    || metadata.file_type().is_symlink()
                    || metadata.len() > MAX_TOOL_BYTES as u64
                {
                    skipped_files += 1;
                    continue;
                }
                let mut options = cap_std::fs::OpenOptions::new();
                options.read(true);
                #[cfg(unix)]
                {
                    use cap_std::fs::OpenOptionsExt;
                    options.custom_flags(libc::O_NONBLOCK);
                }
                attempted_files += 1;
                let file = match directory.open_with(&path, &options) {
                    Ok(f) => f,
                    Err(_) => {
                        skipped_files += 1;
                        continue;
                    }
                };
                if !file.metadata().is_ok_and(|m| m.is_file()) {
                    skipped_files += 1;
                    continue;
                }
                let limit = (MAX_TOOL_BYTES + 1).min(MAX_SEARCH_BYTES - bytes_read);
                let mut bytes = vec![];
                let read = file.take(limit as u64).read_to_end(&mut bytes);
                bytes_read += bytes.len();
                checkpoint(cancel)?;
                if read.is_err() || bytes.len() > MAX_TOOL_BYTES || bytes.contains(&0) {
                    skipped_files += 1;
                    continue;
                }
                // An incomplete read at the aggregate boundary is never searched.
                if bytes.len() < metadata.len() as usize
                    || (limit <= MAX_TOOL_BYTES && bytes.len() == limit)
                {
                    truncated = true;
                    skipped_files += 1;
                    break 'walk;
                }
                let text = match String::from_utf8(bytes) {
                    Ok(t) => t,
                    Err(_) => {
                        skipped_files += 1;
                        continue;
                    }
                };
                scanned_files += 1;
                for (line, text) in text.lines().enumerate() {
                    checkpoint(cancel)?;
                    if let Some(at) = text.find(query) {
                        if matches.len() == MAX_MATCHES {
                            truncated = true;
                            break 'walk;
                        }
                        let before = text[..at].chars().count();
                        let start = before.saturating_sub(60);
                        let snippet: String = text.chars().skip(start).take(240).collect();
                        matches.push(json!({"path":path,"line":line+1,"text":snippet}));
                        if serde_json::to_vec(&matches).unwrap().len() > MAX_TOOL_BYTES - 512 {
                            matches.pop();
                            truncated = true;
                            break 'walk;
                        }
                    }
                }
            }
            if inspected == MAX_SEARCH_ENTRIES && !queue.is_empty() {
                truncated = true;
                break;
            }
        }
        Ok(json!({"matches":matches,"inspectedEntries":inspected,"scannedFiles":scanned_files,"attemptedFiles":attempted_files,"skippedFiles":skipped_files,
            "skippedEntries":skipped_entries,"bytesRead":bytes_read,"truncated":truncated}).to_string())
    }
}

/// Enumeration and sorting are bounded even if all names are excluded.
fn entries(
    directory: &Dir,
    path: &str,
    max: usize,
    inspected: &mut usize,
    truncated: &mut bool,
    skipped: &mut usize,
    cancel: &CancellationToken,
) -> Result<Vec<(String, bool)>, String> {
    let mut result = vec![];
    for entry in directory
        .read_dir(path)
        .map_err(|_| "Folder is unavailable.")?
    {
        checkpoint(cancel)?;
        if *inspected == max {
            *truncated = true;
            break;
        }
        *inspected += 1;
        let entry = match entry {
            Ok(e) => e,
            Err(_) => {
                *skipped += 1;
                continue;
            }
        };
        let name = entry.file_name();
        let Some(name) = name.to_str() else {
            *skipped += 1;
            continue;
        };
        let target = if path == "." {
            name.to_string()
        } else {
            format!("{path}/{name}")
        };
        if !eligible(&target) {
            *skipped += 1;
            continue;
        }
        let kind = match entry.file_type() {
            Ok(k) => k,
            Err(_) => {
                *skipped += 1;
                continue;
            }
        };
        if kind.is_symlink() || (!kind.is_dir() && !kind.is_file()) {
            *skipped += 1;
            continue;
        }
        // A directory can be replaced with a link during enumeration; the
        // capability still prevents escape, and this check avoids aliases.
        let canonical = directory
            .canonicalize(&target)
            .ok()
            .and_then(|p| p.to_str().map(|s| s.replace('\\', "/")));
        if canonical.as_deref() != Some(&target) {
            *skipped += 1;
            continue;
        }
        result.push((target, kind.is_dir()));
    }
    result.sort_by(|a, b| a.0.cmp(&b.0));
    Ok(result)
}
pub fn discovery_spec(search: bool) -> ToolSpec {
    let mut parameters = json!({"type":"object","properties":{"path":{"type":"string","description":"Relative folder, or '.' for the chosen root"}},"required":["path"],"additionalProperties":false});
    if search {
        parameters["properties"]["query"] = json!({"type":"string","description":"Case-sensitive literal single-line text, 1-256 UTF-8 bytes; no regex"});
        parameters["required"] = json!(["path", "query"]);
    }
    ToolSpec { name:if search { "search_text" } else { "list_folder" }.into(),parameters,description:if search {
            "Search bounded UTF-8 text files below an approved relative folder. Requires a separate user decision before scanning/sharing snippets. At most 4 levels, 256 entries, 64 files, 256 KiB read, 16 KiB per file and 30 matches. Results may be partial; full reads need another approval."
        } else { "List a single approved folder level. Requires user approval before sharing names. At most 512 inspected entries, 100 results and 16 KiB output; excludes links, common secret and generated/dependency paths. Results may be partial." }.into() }
}
#[async_trait]
impl ToolPlugin for Discover {
    fn spec(&self) -> ToolSpec {
        discovery_spec(self.search)
    }
    fn prepare(&self, call: &ToolCall) -> Result<ToolRequest, String> {
        if call.name != self.name() {
            return Err("Tool name does not match.".into());
        }
        let args: Arguments =
            serde_json::from_str(&call.arguments).map_err(|_| "Invalid folder arguments.")?;
        if (self.search && !args.query.as_deref().is_some_and(valid_query))
            || (!self.search && args.query.is_some())
        {
            return Err("Invalid folder query.".into());
        }
        Ok(ToolRequest {
            call_id: call.id.clone(),
            name: call.name.clone(),
            target: self.resolve_scope(&args.path)?,
            query: args.query,
            diff: None,
            command: None,
        })
    }
    async fn invoke(
        &self,
        request: &ToolRequest,
        cancel: CancellationToken,
    ) -> Result<String, String> {
        let tool = Self {
            read: self.read.clone(),
            search: self.search,
        };
        let request = request.clone();
        let worker_cancel = cancel.clone();
        let work = tokio::task::spawn_blocking(move || tool.execute(&request, &worker_cancel));
        tokio::select! { biased;
            _ = cancel.cancelled() => Err("Response stopped.".into()),
            result = work => result.map_err(|_| "Folder task failed.".to_string())?,
        }
    }
}
