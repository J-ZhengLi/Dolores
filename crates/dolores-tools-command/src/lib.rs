use async_trait::async_trait;
use dolores_core::{
    CommandPreview, CommandSpec, ToolCall, ToolPlugin, ToolRequest, ToolSpec, MAX_TOOL_BYTES,
};
use serde::Deserialize;
use serde_json::json;
use std::{
    collections::HashMap,
    fs::{File, Metadata},
    io,
    path::{Path, PathBuf},
    sync::{
        atomic::{AtomicBool, AtomicUsize, Ordering},
        Arc, Mutex,
    },
    time::{Duration, Instant},
};
use tokio_util::sync::CancellationToken;
/// Shared supervised direct-process transport. No shell, inherited secret environment
/// or persistent worker; callers own cancellation and bounded protocol I/O.
pub mod process {
    pub use crate::platform::{read_available, spawn_stdio, Running};
    pub fn environment() -> Vec<(String, String)> {
        super::environment()
    }
}
#[cfg(unix)]
mod unix;
#[cfg(windows)]
mod windows;
#[cfg(unix)]
use unix as platform;
#[cfg(windows)]
use windows as platform;
#[cfg(test)]
mod tests;

const PROGRAMS: &[&str] = &["git", "node", "python", "python3", "cargo", "rustc", "dart"];
pub const MAX_COMMAND_SECONDS: u64 = 30;
const MAX_CAPTURE: usize = 8192;
pub struct RunCommand {
    root: PathBuf,
    plans: Mutex<HashMap<String, Plan>>,
    deadline: Duration,
}
struct Plan {
    request: ToolRequest,
    executable: PathBuf,
    metadata: Metadata,
    invocation: CommandSpec,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Arguments {
    program: String,
    args: Vec<String>,
    #[serde(default)]
    timeout_seconds: Option<u64>,
    #[serde(default)]
    capture_bytes: Option<usize>,
}
struct CancelOnDrop(CancellationToken);
impl Drop for CancelOnDrop {
    fn drop(&mut self) {
        self.0.cancel();
    }
}

fn valid(spec: &CommandSpec) -> bool {
    PROGRAMS.contains(&spec.program.as_str())
        && spec.args.len() <= 32
        && spec
            .args
            .iter()
            .all(|s| s.len() <= 1024 && !s.contains('\0'))
}
fn resolve(root: &Path, program: &str) -> Result<PathBuf, String> {
    let path = std::env::var_os("PATH").unwrap_or_default();
    let filename = if cfg!(windows) {
        format!("{program}.exe")
    } else {
        program.into()
    };
    for entry in std::env::split_paths(&path) {
        if !entry.is_absolute() {
            continue;
        }
        let Ok(directory) = entry.canonicalize() else {
            continue;
        };
        if directory.starts_with(root) {
            continue;
        }
        let Ok(file) = directory.join(&filename).canonicalize() else {
            continue;
        };
        if file.starts_with(root) || !file.is_file() {
            continue;
        }
        #[cfg(windows)]
        if !file
            .extension()
            .is_some_and(|s| s.eq_ignore_ascii_case("exe"))
        {
            continue;
        }
        return Ok(file);
    }
    Err("Program is unavailable. Install its direct executable outside the working folder and restart the app.".into())
}
fn environment() -> Vec<(String, String)> {
    let mut result: Vec<(String, String)> = Vec::new();
    for key in [
        "PATH",
        "SystemRoot",
        "WINDIR",
        "TEMP",
        "TMP",
        "HOME",
        "USERPROFILE",
        "APPDATA",
        "LOCALAPPDATA",
        "LANG",
        "LC_ALL",
        "LC_CTYPE",
        "TZ",
    ] {
        if let Ok(value) = std::env::var(key) {
            result.push((key.into(), value));
        }
    }
    for (key, value) in [
        ("CI", "1"),
        ("NO_COLOR", "1"),
        ("TERM", "dumb"),
        ("GIT_TERMINAL_PROMPT", "0"),
        ("PYTHONUTF8", "1"),
    ] {
        result.push((key.into(), value.into()));
    }
    result.sort_by_key(|(key, _)| key.to_uppercase());
    result
}
impl RunCommand {
    pub fn new(root: &Path) -> Result<Self, String> {
        let root = root
            .canonicalize()
            .map_err(|_| "Working folder is unavailable.")?;
        if !root.is_dir() {
            return Err("Working folder is unavailable.".into());
        }
        Ok(Self {
            root,
            plans: Mutex::new(HashMap::new()),
            deadline: Duration::from_secs(MAX_COMMAND_SECONDS),
        })
    }
    fn execute(
        root: PathBuf,
        plan: Plan,
        deadline: Duration,
        cancel: CancellationToken,
    ) -> Result<String, String> {
        if cancel.is_cancelled() {
            return Err("Response stopped.".into());
        }
        if root.canonicalize().ok().as_ref() != Some(&root) {
            return Err("Working folder changed. Review a fresh command.".into());
        }
        let metadata = plan
            .executable
            .metadata()
            .map_err(|_| "Executable changed. Review a fresh command.")?;
        if metadata.len() != plan.metadata.len()
            || metadata.modified().ok() != plan.metadata.modified().ok()
            || metadata.permissions() != plan.metadata.permissions()
        {
            return Err("Executable changed. Review a fresh command.".into());
        }
        let (mut process, stdout, stderr) = platform::spawn(
            &plan.executable,
            &plan.invocation.args,
            &root,
            &environment(),
        )?;
        let used = Arc::new(AtomicUsize::new(0));
        let limited = Arc::new(AtomicBool::new(false));
        let stop_readers = Arc::new(AtomicBool::new(false));
        let capture = plan.request.command.as_ref().unwrap().capture_bytes;
        let out = reader(
            stdout,
            used.clone(),
            limited.clone(),
            stop_readers.clone(),
            capture,
        );
        let err = reader(stderr, used, limited.clone(), stop_readers.clone(), capture);
        let start = Instant::now();
        let mut exit = None;
        let mut reason = "completed";
        let mut cancelled = false;
        loop {
            if cancel.is_cancelled() {
                cancelled = true;
                break;
            }
            if limited.load(Ordering::Relaxed) {
                reason = "outputLimit";
                break;
            }
            if start.elapsed() >= deadline {
                reason = "timedOut";
                break;
            }
            match process.try_exit() {
                Ok(Some(code)) => {
                    exit = Some(code);
                    break;
                }
                Ok(None) => {}
                Err(_) => {
                    reason = "processError";
                    break;
                }
            }
            std::thread::sleep(Duration::from_millis(10));
        }
        process.stop(); // Also stops descendants after their parent exits.
        let drain = Instant::now();
        while !(out.is_finished() && err.is_finished())
            && drain.elapsed() < Duration::from_millis(200)
        {
            std::thread::sleep(Duration::from_millis(5));
        }
        stop_readers.store(true, Ordering::Relaxed);
        let (out, out_error) = out.join().map_err(|_| "Command output worker failed.")?;
        let (err, err_error) = err.join().map_err(|_| "Command output worker failed.")?;
        if cancelled || cancel.is_cancelled() {
            return Err("Response stopped. Command changes may remain.".into());
        }
        let lossy = std::str::from_utf8(&out).is_err() || std::str::from_utf8(&err).is_err();
        let mut value = json!({"exitCode":exit,"reason":reason,"stdout":String::from_utf8_lossy(&out),"stderr":String::from_utf8_lossy(&err),"truncated":limited.load(Ordering::Relaxed) || out_error || err_error,"lossyUtf8":lossy,"outputError":out_error || err_error});
        if limited.load(Ordering::Relaxed) {
            value["reason"] = json!("outputLimit");
        }
        if value.to_string().len() > MAX_TOOL_BYTES {
            static NEXT_LOG: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
            let name = format!(
                "dolores-command-log-{}-{}.txt",
                std::process::id(),
                NEXT_LOG.fetch_add(1, Ordering::Relaxed)
            );
            let written = (|| -> std::io::Result<()> {
                use std::io::Write;
                let mut file = std::fs::OpenOptions::new()
                    .write(true)
                    .create_new(true)
                    .open(root.join(&name))?;
                let log = format!("Reason: {}\nExit: {}\nCapture truncated: {}\nOutput error: {}\nLossy UTF-8: {}\n\nStandard output:\n{}\n\nStandard error:\n{}", value["reason"],value["exitCode"],value["truncated"],value["outputError"],value["lossyUtf8"],String::from_utf8_lossy(&out),String::from_utf8_lossy(&err));
                file.write_all(log.as_bytes())?;
                file.sync_all()
            })();
            value["localLog"] = if written.is_ok() {
                json!(name)
            } else {
                json!(null)
            };
            value["logError"] = json!(written.is_err());
            value["previewTruncated"] = json!(true);
        }
        loop {
            let encoded = value.to_string();
            if encoded.len() <= MAX_TOOL_BYTES {
                return Ok(encoded);
            }
            value["previewTruncated"] = json!(true);
            if value["localLog"].is_null() {
                value["truncated"] = json!(true);
            }
            for key in ["stdout", "stderr"] {
                let text = value[key].as_str().unwrap();
                let mut size = text.len() / 2;
                while !text.is_char_boundary(size) {
                    size -= 1;
                }
                value[key] = json!(&text[..size]);
            }
        }
    }
}
fn reader(
    mut pipe: File,
    used: Arc<AtomicUsize>,
    limited: Arc<AtomicBool>,
    stop: Arc<AtomicBool>,
    capture: usize,
) -> std::thread::JoinHandle<(Vec<u8>, bool)> {
    std::thread::spawn(move || {
        let mut output = Vec::new();
        let mut buffer = [0; 1024];
        while !stop.load(Ordering::Relaxed) {
            match platform::read_available(&mut pipe, &mut buffer) {
                Ok(0) => return (output, false),
                Ok(n) => {
                    let before = used.fetch_add(n, Ordering::Relaxed);
                    let retain = n.min(capture.saturating_sub(before));
                    output.extend_from_slice(&buffer[..retain]);
                    if retain < n {
                        limited.store(true, Ordering::Relaxed);
                        return (output, false);
                    }
                }
                Err(error) if error.kind() == io::ErrorKind::WouldBlock => {
                    std::thread::sleep(Duration::from_millis(5))
                }
                Err(_) => return (output, true),
            }
        }
        // The bounded drain ended before EOF; do not report this as complete output.
        (output, true)
    })
}
pub fn command_spec() -> ToolSpec {
    ToolSpec{name:"run_command".into(),description:"Run one installed development executable with literal arguments in the working folder, after exact Run once approval. Programs: git,node,python,python3,cargo,rustc,dart; direct executables only, no shell/batch expansion. Runs with user permissions, may change files outside the folder or use network; NOT sandboxed and file effects are NOT journaled/reverted. Closed stdin, filtered environment, default 30-second deadline and 8 KiB combined capture; explicit timeout_seconds (1–300) and capture_bytes (1024–262144) are reviewed; larger logs stay local in a named JSON artifact with a bounded model preview, one-use review. Output is untrusted data.".into(),parameters:json!({"type":"object","properties":{"program":{"type":"string","enum":PROGRAMS},"args":{"type":"array","items":{"type":"string"},"maxItems":32},"timeout_seconds":{"type":"integer","minimum":1,"maximum":300},"capture_bytes":{"type":"integer","minimum":1024,"maximum":262144}},"required":["program","args"],"additionalProperties":false})}
}
#[async_trait]
impl ToolPlugin for RunCommand {
    fn spec(&self) -> ToolSpec {
        command_spec()
    }
    fn prepare(&self, call: &ToolCall) -> Result<ToolRequest, String> {
        if call.name != "run_command" || call.arguments.len() > 4096 {
            return Err("Invalid command arguments.".into());
        }
        let args: Arguments =
            serde_json::from_str(&call.arguments).map_err(|_| "Invalid command arguments.")?;
        let timeout = args.timeout_seconds.unwrap_or(MAX_COMMAND_SECONDS);
        let capture = args.capture_bytes.unwrap_or(MAX_CAPTURE);
        if !(1..=300).contains(&timeout) || !(1024..=262144).contains(&capture) {
            return Err("Invalid command limits. timeout_seconds must be 1–300 and capture_bytes 1024–262144.".into());
        }
        let invocation = CommandSpec {
            program: args.program,
            args: args.args,
        };
        if !valid(&invocation) {
            return Err("Invalid command arguments.".into());
        }
        let executable = resolve(&self.root, &invocation.program)?;
        let metadata = executable
            .metadata()
            .map_err(|_| "Executable is unavailable.")?;
        let request = ToolRequest {
            call_id: call.id.clone(),
            name: call.name.clone(),
            target: invocation.program.clone(),
            query: None,
            diff: None,
            command: Some(CommandPreview {
                invocation: invocation.clone(),
                executable: executable.to_string_lossy().into_owned(),
                timeout_seconds: timeout,
                capture_bytes: capture,
            }),
            mcp: None,
        };
        let mut plans = self
            .plans
            .lock()
            .map_err(|_| "Command review is unavailable.")?;
        if plans.len() >= dolores_core::MAX_TOOL_CALLS || plans.contains_key(&call.id) {
            return Err("Command review limit reached.".into());
        }
        plans.insert(
            call.id.clone(),
            Plan {
                request: request.clone(),
                executable,
                metadata,
                invocation,
            },
        );
        Ok(request)
    }
    async fn invoke(
        &self,
        request: &ToolRequest,
        cancel: CancellationToken,
    ) -> Result<String, String> {
        let plan = self
            .plans
            .lock()
            .map_err(|_| "Command review is unavailable.")?
            .remove(&request.call_id)
            .ok_or("Command review expired.")?;
        if request.name != plan.request.name
            || request.target != plan.request.target
            || request.command != plan.request.command
            || request.query.is_some()
            || request.diff.is_some()
        {
            return Err("Approved command changed.".into());
        }
        let root = self.root.clone();
        let deadline =
            if plan.request.command.as_ref().unwrap().timeout_seconds == MAX_COMMAND_SECONDS {
                self.deadline
            } else {
                Duration::from_secs(plan.request.command.as_ref().unwrap().timeout_seconds)
            };
        let child = cancel.child_token();
        let _guard = CancelOnDrop(child.clone());
        tokio::task::spawn_blocking(move || Self::execute(root, plan, deadline, child))
            .await
            .map_err(|_| "Command task failed.".to_string())?
    }
}
