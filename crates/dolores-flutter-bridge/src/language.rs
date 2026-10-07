//! Lazy, human-requested LSP processes. The model receives no additional tool.
use crate::Engine;
use dolores_tools_command::process::{environment, read_available, spawn_stdio};
use serde::Deserialize;
use serde_json::{json, Value};
use std::{
    collections::{HashMap, VecDeque},
    io::Write,
    path::{Path, PathBuf},
    sync::{
        atomic::{AtomicBool, Ordering},
        mpsc::{sync_channel, SyncSender},
        Arc, Mutex,
    },
    time::{Duration, Instant},
};

#[derive(Deserialize)]
#[serde(tag = "action", rename_all = "camelCase", deny_unknown_fields)]
pub(crate) enum Request {
    Feature {
        session: String,
        document: String,
        version: u64,
        feature: String,
        #[serde(default)]
        position: Value,
        #[serde(default)]
        name: String,
    },
    Poll {
        id: String,
    },
    Stop,
    InstallInfo,
    Install {
        language: String,
    },
    InstallPoll {
        id: String,
    },
    CancelInstall {
        id: String,
    },
}
struct Work {
    id: String,
    snapshot: Value,
    feature: String,
    position: Value,
    name: String,
}
struct Job {
    root: String,
    document: String,
    version: u64,
    result: Option<Result<Value, String>>,
}
struct Server {
    stop: Arc<AtomicBool>,
    sender: SyncSender<Work>,
    thread: Option<std::thread::JoinHandle<()>>,
}
#[derive(Default)]
pub(crate) struct Registry {
    install: Option<crate::language_tools::Install>,
    servers: HashMap<String, Server>,
    jobs: Arc<Mutex<HashMap<String, Job>>>,
}
impl Registry {
    pub(crate) fn stop_all(&mut self) -> Result<(), String> {
        if let Some(i) = &self.install {
            i.cancel.cancel();
            let until = Instant::now() + Duration::from_secs(2);
            while !i.task.is_finished() && Instant::now() < until {
                std::thread::sleep(Duration::from_millis(10));
            }
            if !i.task.is_finished() {
                return Err(
                    "Language installation is canceling. Keep Dolores open and retry closing."
                        .into(),
                );
            }
        }
        for s in self.servers.values() {
            s.stop.store(true, Ordering::Release);
        }
        let until = Instant::now() + Duration::from_secs(2);
        while self
            .servers
            .values()
            .any(|s| s.thread.as_ref().is_some_and(|t| !t.is_finished()))
            && Instant::now() < until
        {
            std::thread::sleep(Duration::from_millis(10));
        }
        if self
            .servers
            .values()
            .any(|s| s.thread.as_ref().is_some_and(|t| !t.is_finished()))
        {
            return Err(
                "Language services are stopping. Keep Dolores open and retry closing.".into(),
            );
        }
        for (_, mut s) in self.servers.drain() {
            if let Some(t) = s.thread.take() {
                let _ = t.join();
            }
        }
        self.jobs
            .lock()
            .map_err(|_| "Language state unavailable.")?
            .clear();
        Ok(())
    }
    fn submit(
        &mut self,
        root: String,
        snapshot: Value,
        feature: String,
        position: Value,
        name: String,
        managed: &Path,
    ) -> Result<Value, String> {
        if ![
            "diagnostics",
            "completion",
            "hover",
            "definition",
            "references",
            "formatting",
            "rename",
        ]
        .contains(&feature.as_str())
        {
            return Err("This language feature is unavailable.".into());
        }
        let path = snapshot["snapshot"]["path"]
            .as_str()
            .ok_or("Document path unavailable.")?;
        let language = language(path)
            .ok_or("Language services support TypeScript, JavaScript and Rust files.")?;
        if feature == "rename"
            && (name.is_empty() || name.len() > 128 || name.chars().any(char::is_control))
        {
            return Err("Enter a symbol name within 128 bytes.".into());
        }
        if feature != "diagnostics" && feature != "formatting" {
            position_byte(snapshot["text"].as_str().unwrap(), &position)?;
        }
        let key = format!("{root}:{language}");
        if !self.servers.contains_key(&key) {
            if self.servers.len() >= 2 {
                return Err("Two language servers are retained. Stop language services from Editor actions before opening another project/language.".into());
            }
            let (executable, args) = discover(language, managed)?;
            let (tx, rx) = sync_channel::<Work>(8);
            let stop = Arc::new(AtomicBool::new(false));
            let cancel = stop.clone();
            let jobs = self.jobs.clone();
            let folder = PathBuf::from(&root);
            let thread = std::thread::spawn(move || {
                let result = run(&executable, &args, &folder, rx, cancel, jobs.clone());
                if let Err(error) = result {
                    finish_all(&jobs, &folder.to_string_lossy(), &error);
                }
            });
            self.servers.insert(
                key.clone(),
                Server {
                    stop,
                    sender: tx,
                    thread: Some(thread),
                },
            );
        }
        let mut jobs = self
            .jobs
            .lock()
            .map_err(|_| "Language state unavailable.")?;
        if jobs.len() >= 16 {
            return Err("Sixteen language requests are retained. Finish or stop language services before retrying.".into());
        }
        let id = uuid::Uuid::new_v4().to_string();
        let document = snapshot["document"].as_str().unwrap().into();
        let version = snapshot["version"].as_u64().unwrap();
        jobs.insert(
            id.clone(),
            Job {
                root,
                document,
                version,
                result: None,
            },
        );
        if self.servers[&key]
            .sender
            .try_send(Work {
                id: id.clone(),
                snapshot,
                feature,
                position,
                name,
            })
            .is_err()
        {
            jobs.remove(&id);
            return Err("Language service stopped or its request queue is full. Stop services and retry; edits remain.".into());
        }
        Ok(json!({"id":id,"state":"pending"}))
    }
}
impl Drop for Registry {
    fn drop(&mut self) {
        let _ = self.stop_all();
    }
}
fn language(path: &str) -> Option<&'static str> {
    match path.rsplit('.').next()? {
        "rs" => Some("rust"),
        "ts" | "tsx" | "js" | "jsx" | "mjs" | "cjs" => Some("typescript"),
        _ => None,
    }
}
fn discover(language: &str, managed: &Path) -> Result<(PathBuf, Vec<String>), String> {
    let installed = crate::language_tools::installed(managed, language)?;
    let managed = installed.as_deref().unwrap_or(managed);
    if language == "rust" {
        let pinned = managed.join("rust-analyzer.exe");
        let exe = if pinned.is_file() {
            Some(pinned)
        } else {
            crate::terminal::executable(&[if cfg!(windows) {
                "rust-analyzer.exe"
            } else {
                "rust-analyzer"
            }])
        };
        return exe.map(|e|(e,vec![])).ok_or("Rust language server is unavailable. Use Install language tools in Editor actions; files remain editable.".into());
    }
    let script = managed.join("node_modules/typescript-language-server/lib/cli.mjs");
    if script.is_file() {
        let node = crate::terminal::executable(&[if cfg!(windows) { "node.exe" } else { "node" }])
            .ok_or("Install Node.js before using TypeScript language tools.")?;
        return Ok((
            node,
            vec![script.to_string_lossy().into(), "--stdio".into()],
        ));
    }
    // npm's command wrappers cannot be passed to CreateProcess as executables.
    if let Some(wrapper) = crate::terminal::executable(&[
        "typescript-language-server.cmd",
        "typescript-language-server",
    ]) {
        let script = wrapper
            .parent()
            .unwrap()
            .join("node_modules/typescript-language-server/lib/cli.mjs");
        if script.is_file() {
            let node = crate::terminal::executable(&["node.exe", "node"])
                .ok_or("Node.js is unavailable.")?;
            return Ok((
                node,
                vec![script.to_string_lossy().into(), "--stdio".into()],
            ));
        }
        if !cfg!(windows) {
            return Ok((wrapper, vec!["--stdio".into()]));
        }
    }
    Err("TypeScript language server is unavailable. Use Install language tools in Editor actions; files remain editable.".into())
}
pub(super) fn position_byte(text: &str, position: &Value) -> Result<usize, String> {
    let line = position["line"]
        .as_u64()
        .ok_or("Invalid language position.")? as usize;
    let column = position["character"]
        .as_u64()
        .ok_or("Invalid language position.")? as usize;
    let mut offset = 0;
    for (n, part) in text.split('\n').enumerate() {
        if n == line {
            let mut units = 0;
            for (b, c) in part.char_indices() {
                if units == column {
                    return Ok(offset + b);
                }
                units += c.len_utf16();
                if units > column {
                    return Err("Language position splits a Unicode character.".into());
                }
            }
            return if units == column {
                Ok(offset + part.len())
            } else {
                Err("Language position is outside the line.".into())
            };
        }
        offset += part.len() + 1;
    }
    Err("Language position is outside the file.".into())
}
#[derive(Default)]
struct Frames {
    bytes: Vec<u8>,
}
impl Frames {
    fn push(&mut self, chunk: &[u8]) -> Result<Vec<Value>, String> {
        self.bytes.extend_from_slice(chunk);
        if self.bytes.len() > 4 * 1024 * 1024 + 8192 {
            return Err("Language server frame exceeded 4 MiB. Stop services and retry.".into());
        }
        let mut values = vec![];
        while let Some(end) = self.bytes.windows(4).position(|s| s == b"\r\n\r\n") {
            if end > 8192 {
                return Err("Language server header exceeded 8 KiB.".into());
            }
            let header = std::str::from_utf8(&self.bytes[..end])
                .map_err(|_| "Invalid language server header.")?;
            let lengths = header
                .lines()
                .filter_map(|l| l.split_once(':'))
                .filter(|(k, _)| k.eq_ignore_ascii_case("Content-Length"))
                .map(|(_, v)| v.trim().parse::<usize>())
                .collect::<Vec<_>>();
            if lengths.len() != 1 {
                return Err("Language server frame has invalid Content-Length.".into());
            }
            let length = *lengths[0]
                .as_ref()
                .map_err(|_| "Invalid language server length.")?;
            if length > 4 * 1024 * 1024 {
                return Err("Language server frame exceeded 4 MiB.".into());
            }
            if self.bytes.len() < end + 4 + length {
                break;
            }
            values.push(
                serde_json::from_slice(&self.bytes[end + 4..end + 4 + length])
                    .map_err(|_| "Malformed language server JSON.")?,
            );
            self.bytes.drain(..end + 4 + length);
        }
        if self.bytes.len() > 8192 && !self.bytes.windows(4).any(|s| s == b"\r\n\r\n") {
            return Err("Language server header exceeded 8 KiB.".into());
        }
        Ok(values)
    }
}
fn send(tx: &SyncSender<Value>, value: Value) -> Result<(), String> {
    tx.try_send(value)
        .map_err(|_| "Language input queue is full. Stop services and retry.".into())
}
fn finish(jobs: &Arc<Mutex<HashMap<String, Job>>>, id: &str, result: Result<Value, String>) {
    if let Ok(mut all) = jobs.lock() {
        if let Some(j) = all.get_mut(id) {
            j.result = Some(result);
        }
    }
}
fn finish_all(jobs: &Arc<Mutex<HashMap<String, Job>>>, root: &str, error: &str) {
    if let Ok(mut all) = jobs.lock() {
        for j in all
            .values_mut()
            .filter(|j| j.root == root && j.result.is_none())
        {
            j.result = Some(Err(error.into()));
        }
    }
}
fn uri(path: &Path) -> Result<String, String> {
    let ordinary = path
        .to_string_lossy()
        .trim_start_matches(r"\\?\")
        .to_string();
    url::Url::from_file_path(&ordinary)
        .map(|u| u.into())
        .map_err(|_| "File URI is unavailable.".into())
}
fn run(
    exe: &Path,
    args: &[String],
    root: &Path,
    rx: std::sync::mpsc::Receiver<Work>,
    stop: Arc<AtomicBool>,
    jobs: Arc<Mutex<HashMap<String, Job>>>,
) -> Result<(), String> {
    let (mut process, mut input, mut stdout, mut stderr) =
        spawn_stdio(exe, args, root, &environment())?;
    let (tx, writer_rx) = sync_channel::<Value>(16);
    let writer = std::thread::spawn(move || -> std::io::Result<()> {
        for v in writer_rx {
            let bytes = serde_json::to_vec(&v)?;
            write!(input, "Content-Length: {}\r\n\r\n", bytes.len())?;
            input.write_all(&bytes)?;
            input.flush()?;
        }
        Ok(())
    });
    let result = (|| {
        let root_uri = uri(root)?;
        send(
            &tx,
            json!({"jsonrpc":"2.0","id":0,"method":"initialize","params":{"processId":std::process::id(),"rootUri":root_uri,"workspaceFolders":[{"uri":root_uri,"name":"Project"}],"capabilities":{"general":{"positionEncodings":["utf-16"]},"textDocument":{"publishDiagnostics":{"versionSupport":true},"completion":{"completionItem":{"snippetSupport":false}},"synchronization":{"dynamicRegistration":false}},"workspace":{"configuration":true,"applyEdit":false}},"initializationOptions":{"cargo":{"buildScripts":{"enable":false}},"procMacro":{"enable":false},"checkOnSave":false}}}),
        )?;
        let start = Instant::now();
        let mut ready = false;
        let mut frames = Frames::default();
        let mut queue = VecDeque::<Work>::new();
        let mut pending = HashMap::<u64, (Work, Instant, String)>::new();
        let mut next = 1u64;
        let mut opened = HashMap::<String, u64>::new();
        let mut diagnostics = HashMap::<String, Value>::new();
        let mut bytes = [0u8; 65536];
        loop {
            if stop.load(Ordering::Acquire) {
                return Err("Language services stopped. Editor buffers remain.".into());
            }
            if process
                .try_exit()
                .map_err(|_| "Language process status unavailable.")?
                .is_some()
            {
                return Err("Language server exited. Check installation, stop services and retry; buffers remain.".into());
            }
            if !ready && start.elapsed() > Duration::from_secs(15) {
                return Err("Language server initialization timed out after 15 seconds. Stop services and retry.".into());
            }
            while let Ok(w) = rx.try_recv() {
                queue.push_back(w);
            }
            if ready {
                while let Some(w) = queue.pop_front() {
                    if let Some(paths) = w.snapshot["openPaths"].as_array() {
                        let keep = paths
                            .iter()
                            .filter_map(|p| p.as_str())
                            .map(|p| uri(&root.join(p)))
                            .collect::<Result<std::collections::HashSet<_>, _>>()?;
                        for old in opened
                            .keys()
                            .filter(|u| !keep.contains(*u))
                            .cloned()
                            .collect::<Vec<_>>()
                        {
                            send(
                                &tx,
                                json!({"jsonrpc":"2.0","method":"textDocument/didClose","params":{"textDocument":{"uri":old}}}),
                            )?;
                            opened.remove(&old);
                            diagnostics.remove(&old);
                        }
                    }
                    let path = w.snapshot["snapshot"]["path"].as_str().unwrap();
                    let file_uri = uri(&root.join(path))?;
                    let version = w.snapshot["version"].as_u64().unwrap();
                    let language_id = if path.ends_with(".rs") {
                        "rust"
                    } else if path.ends_with(".js")
                        || path.ends_with(".mjs")
                        || path.ends_with(".cjs")
                    {
                        "javascript"
                    } else if path.ends_with(".jsx") {
                        "javascriptreact"
                    } else if path.ends_with(".tsx") {
                        "typescriptreact"
                    } else {
                        "typescript"
                    };
                    if !opened.contains_key(&file_uri) {
                        send(
                            &tx,
                            json!({"jsonrpc":"2.0","method":"textDocument/didOpen","params":{"textDocument":{"uri":file_uri,"languageId":language_id,"version":version,"text":w.snapshot["text"]}}}),
                        )?;
                    } else if opened[&file_uri] != version {
                        send(
                            &tx,
                            json!({"jsonrpc":"2.0","method":"textDocument/didChange","params":{"textDocument":{"uri":file_uri,"version":version},"contentChanges":[{"text":w.snapshot["text"]}]}}),
                        )?;
                    }
                    opened.insert(file_uri.clone(), version);
                    let method = if w.feature == "diagnostics" {
                        "documentSymbol"
                    } else {
                        w.feature.as_str()
                    };
                    let mut params = json!({"textDocument":{"uri":file_uri},"position":w.position});
                    if w.feature == "references" {
                        params["context"] = json!({"includeDeclaration":true});
                    }
                    if w.feature == "formatting" {
                        params["options"] = json!({"tabSize":4,"insertSpaces":true});
                    }
                    if w.feature == "rename" {
                        params["newName"] = json!(w.name);
                    }
                    send(
                        &tx,
                        json!({"jsonrpc":"2.0","id":next,"method":format!("textDocument/{method}"),"params":params}),
                    )?;
                    pending.insert(next, (w, Instant::now(), file_uri));
                    next += 1;
                }
            }
            match read_available(&mut stdout, &mut bytes) {
                Ok(0) => return Err("Language output closed. Stop services and retry.".into()),
                Ok(n) => {
                    for value in frames.push(&bytes[..n])? {
                        if value.get("method").is_some() && value.get("id").is_some() {
                            let method = value["method"].as_str().unwrap_or("");
                            let result = if method == "workspace/configuration" {
                                json!(value["params"]["items"].as_array().map(|a|a.iter().map(|_|json!({"cargo":{"buildScripts":{"enable":false}},"procMacro":{"enable":false},"checkOnSave":false})).collect::<Vec<_>>()).unwrap_or_default())
                            } else if method == "workspace/applyEdit" {
                                json!({"applied":false,"failureReason":"User-reviewed workspace edits only."})
                            } else {
                                Value::Null
                            };
                            send(
                                &tx,
                                json!({"jsonrpc":"2.0","id":value["id"],"result":result}),
                            )?;
                        } else if value["method"] == "textDocument/publishDiagnostics" {
                            let p = &value["params"];
                            if let Some(uri) = p["uri"].as_str() {
                                if opened
                                    .get(uri)
                                    .is_some_and(|v| p["version"].as_u64() == Some(*v))
                                {
                                    let items = p["diagnostics"]
                                        .as_array()
                                        .map(|a| a.iter().take(200).cloned().collect::<Vec<_>>())
                                        .unwrap_or_default();
                                    let shown = json!(items);
                                    if serde_json::to_vec(&shown).unwrap_or_default().len()
                                        <= 256 * 1024
                                    {
                                        diagnostics.insert(uri.into(), shown.clone());
                                        for id in pending
                                            .iter()
                                            .filter(|(_, (w, _, u))| {
                                                w.feature == "diagnostics"
                                                    && u == uri
                                                    && w.snapshot["version"] == p["version"]
                                            })
                                            .map(|(id, _)| *id)
                                            .collect::<Vec<_>>()
                                        {
                                            if let Some((w, _, _)) = pending.remove(&id) {
                                                finish(
                                                    &jobs,
                                                    &w.id,
                                                    Ok(
                                                        json!({"items":shown,"notice":"Diagnostics match this editor version."}),
                                                    ),
                                                );
                                            }
                                        }
                                    }
                                }
                            }
                        } else if value["id"] == 0 {
                            if value.get("error").is_some() {
                                return Err("Language server refused initialization.".into());
                            }
                            if value["result"]["capabilities"]["positionEncoding"]
                                .as_str()
                                .is_some_and(|s| s != "utf-16")
                            {
                                return Err(
                                    "Language server did not negotiate UTF-16 positions.".into()
                                );
                            }
                            ready = true;
                            send(
                                &tx,
                                json!({"jsonrpc":"2.0","method":"initialized","params":{}}),
                            )?;
                        } else if let Some(id) = value["id"].as_u64() {
                            if let Some((w, _, file_uri)) = pending.remove(&id) {
                                if w.feature == "diagnostics"
                                    && !diagnostics.contains_key(&file_uri)
                                    && value.get("error").is_none()
                                {
                                    pending.insert(id, (w, Instant::now(), file_uri));
                                    continue;
                                }
                                let result = if value.get("error").is_some() {
                                    Err("Language server could not provide this feature. Check project setup and retry; buffers remain.".into())
                                } else if w.feature == "diagnostics" {
                                    Ok(
                                        json!({"items":diagnostics.remove(&file_uri).unwrap_or(json!([])),"notice":"Only diagnostics carrying the current editor version are displayed."}),
                                    )
                                } else {
                                    Ok(value["result"].clone())
                                };
                                finish(&jobs, &w.id, result);
                            }
                        }
                    }
                }
                Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {}
                Err(_) => return Err("Language output failed. Stop services and retry.".into()),
            }
            // Drain stderr without retaining private project text or blocking stdout.
            let _ = read_available(&mut stderr, &mut bytes);
            for id in pending
                .iter()
                .filter(|(_, (_, t, _))| t.elapsed() > Duration::from_secs(15))
                .map(|(id, _)| *id)
                .collect::<Vec<_>>()
            {
                if let Some((w, _, _)) = pending.remove(&id) {
                    send(
                        &tx,
                        json!({"jsonrpc":"2.0","method":"$/cancelRequest","params":{"id":id}}),
                    )?;
                    finish(&jobs,&w.id,Err("Language request timed out after 15 seconds. Retry or stop services; edits remain.".into()));
                }
            }
            std::thread::sleep(Duration::from_millis(20));
        }
    })();
    process.stop();
    drop(tx);
    let _ = writer.join();
    result
}
impl Engine {
    pub(crate) fn language_call(&self, request: Request) -> Result<Value, String> {
        match request {
            Request::InstallInfo => Ok(crate::language_tools::info()),
            Request::Install { language } => {
                let mut state = self
                    .languages
                    .lock()
                    .map_err(|_| "Language state unavailable.")?;
                if state
                    .install
                    .as_ref()
                    .is_some_and(|i| !i.task.is_finished())
                {
                    return Err("An installation is running. Finish or cancel it first.".into());
                }
                if !state.servers.is_empty() {
                    return Err(
                        "Stop language services before installing tools. Buffers remain.".into(),
                    );
                }
                let managed = self
                    .workspace_directory
                    .as_ref()
                    .and_then(|p| p.parent())
                    .ok_or("Private language storage unavailable.")?
                    .join("language-tools");
                let id = uuid::Uuid::new_v4().to_string();
                let cancel = tokio_util::sync::CancellationToken::new();
                let progress = Arc::new(Mutex::new(json!({"state":"starting","bytes":0})));
                let c = cancel.clone();
                let p = progress.clone();
                let task = self.runtime.spawn(async move {
                    let result =
                        crate::language_tools::install(managed, language, c, p.clone()).await;
                    if let Ok(mut s) = p.lock() {
                        *s = match result {
                            Ok(()) => json!({"state":"done"}),
                            Err(e) => json!({"state":"failed","error":e}),
                        };
                    }
                });
                state.install = Some(crate::language_tools::Install {
                    id: id.clone(),
                    cancel,
                    state: progress,
                    task,
                });
                Ok(json!({"id":id}))
            }
            Request::InstallPoll { id } => {
                let state = self
                    .languages
                    .lock()
                    .map_err(|_| "Language state unavailable.")?;
                let i = state
                    .install
                    .as_ref()
                    .filter(|i| i.id == id)
                    .ok_or("Installation expired. Review setup again.")?;
                let value = i
                    .state
                    .lock()
                    .map_err(|_| "Install status unavailable.")?
                    .clone();
                Ok(value)
            }
            Request::CancelInstall { id } => {
                let state = self
                    .languages
                    .lock()
                    .map_err(|_| "Language state unavailable.")?;
                let i = state
                    .install
                    .as_ref()
                    .filter(|i| i.id == id)
                    .ok_or("Installation expired.")?;
                i.cancel.cancel();
                Ok(json!({"canceling":true}))
            }
            Request::Feature {
                session,
                document,
                version,
                feature,
                position,
                name,
            } => {
                let root = self
                    .store
                    .workspace(&session)?
                    .root
                    .ok_or("Select a project conversation on Home.")?;
                let snapshot = self
                    .editor
                    .lock()
                    .map_err(|_| "Editor unavailable.")?
                    .language_snapshot(&root, &document, version)?;
                let managed = self
                    .workspace_directory
                    .as_ref()
                    .and_then(|p| p.parent())
                    .ok_or("Private profile directory unavailable.")?
                    .join("language-tools");
                self.languages
                    .lock()
                    .map_err(|_| "Language state unavailable.")?
                    .submit(root, snapshot, feature, position, name, &managed)
            }
            Request::Poll { id } => {
                let state = self
                    .languages
                    .lock()
                    .map_err(|_| "Language state unavailable.")?;
                let mut jobs = state
                    .jobs
                    .lock()
                    .map_err(|_| "Language jobs unavailable.")?;
                let job = jobs
                    .get(&id)
                    .ok_or("Language request expired. Request the feature again.")?;
                if job.result.is_none() {
                    return Ok(json!({"state":"pending"}));
                }
                let job = jobs.remove(&id).unwrap();
                self.editor
                    .lock()
                    .map_err(|_| "Editor unavailable.")?
                    .language_snapshot(&job.root, &job.document, job.version)?;
                Ok(
                    json!({"state":"done","document":job.document,"version":job.version,"result":job.result.unwrap()?}),
                )
            }
            Request::Stop => {
                self.languages
                    .lock()
                    .map_err(|_| "Language state unavailable.")?
                    .stop_all()?;
                Ok(json!({"stopped":true}))
            }
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn background_server_sees_unsaved_unicode_and_malformed_output_is_recoverable() {
        let root = tempfile::tempdir().unwrap();
        let script = root
            .path()
            .join("node_modules/typescript-language-server/lib/cli.mjs");
        std::fs::create_dir_all(script.parent().unwrap()).unwrap();
        std::fs::write(&script,r#"
let buffer=Buffer.alloc(0), document;
function send(v){const b=Buffer.from(JSON.stringify({jsonrpc:'2.0',...v}));process.stdout.write(`Content-Length: ${b.length}\r\n\r\n`);process.stdout.write(b);}
process.stdin.on('data',chunk=>{buffer=Buffer.concat([buffer,chunk]);for(;;){const end=buffer.indexOf('\r\n\r\n');if(end<0)return;const n=Number(/Content-Length: (\d+)/i.exec(buffer.subarray(0,end).toString())[1]);if(buffer.length<end+4+n)return;const v=JSON.parse(buffer.subarray(end+4,end+4+n));buffer=buffer.subarray(end+4+n);
 if(v.method==='initialize')send({id:v.id,result:{capabilities:{positionEncoding:'utf-16'}}});
 if(v.method==='textDocument/didOpen'){document=v.params.textDocument;send({method:'textDocument/publishDiagnostics',params:{uri:document.uri,version:document.version,diagnostics:[{message:'fixture',range:{start:{line:0,character:0},end:{line:0,character:1}}}]}});}
 if(v.method==='textDocument/completion')send({id:v.id,result:[{label:document.text}]});
 if(v.method==='textDocument/documentSymbol')send({id:v.id,result:[]});
 if(v.method==='textDocument/hover')process.stdout.write('Content-Length: 1\r\n\r\nx');
}});
"#).unwrap();
        let folder = root.path().to_string_lossy().into_owned();
        let snap =
            json!({"document":"d","version":0,"snapshot":{"path":"a.ts"},"text":"unsaved 😀 世界"});
        let mut r = Registry::default();
        let job = r
            .submit(
                folder.clone(),
                snap.clone(),
                "completion".into(),
                json!({"line":0,"character":0}),
                String::new(),
                root.path(),
            )
            .unwrap();
        let wait = |r: &Registry, id: &str| {
            let until = Instant::now() + Duration::from_secs(10);
            loop {
                if let Some(v) = r.jobs.lock().unwrap().get(id).unwrap().result.clone() {
                    return v;
                }
                assert!(Instant::now() < until, "fixture request hung");
                std::thread::sleep(Duration::from_millis(20));
            }
        };
        assert_eq!(
            wait(&r, job["id"].as_str().unwrap()).unwrap()[0]["label"],
            snap["text"]
        );
        let job = r
            .submit(
                folder.clone(),
                snap.clone(),
                "diagnostics".into(),
                Value::Null,
                String::new(),
                root.path(),
            )
            .unwrap();
        assert_eq!(
            wait(&r, job["id"].as_str().unwrap()).unwrap()["items"][0]["message"],
            "fixture"
        );
        let job = r
            .submit(
                folder,
                snap,
                "hover".into(),
                json!({"line":0,"character":0}),
                String::new(),
                root.path(),
            )
            .unwrap();
        assert!(wait(&r, job["id"].as_str().unwrap())
            .unwrap_err()
            .contains("Malformed"));
        r.stop_all().unwrap();
        assert!(r.servers.is_empty());
    }
    #[test]
    fn framing_and_utf16_positions_reject_malformed_or_split_data() {
        let mut f = Frames::default();
        assert!(f.push(b"Content-Length: 2\r\n\r\n{").unwrap().is_empty());
        assert_eq!(f.push(b"}").unwrap(), vec![json!({})]);
        assert!(Frames::default()
            .push(b"Content-Length: 99999999\r\n\r\n")
            .is_err());
        assert!(Frames::default()
            .push(b"Content-Length: 1\r\n\r\nx")
            .is_err());
        assert_eq!(
            position_byte("a😀b\n世界", &json!({"line":0,"character":3})).unwrap(),
            5
        );
        assert!(position_byte("a😀b", &json!({"line":0,"character":2})).is_err());
    }
}
