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
mod developer_tools;
mod execution;
pub mod sessions;
use execution::{Isolation, Launch};
#[cfg(windows)]
pub mod isolation;
#[cfg(test)]
mod tests;

pub const MAX_COMMAND_SECONDS: u64 = 30;
const MAX_CAPTURE: usize = 8192;
const MAX_ARGUMENT_BYTES: usize = 8192;
const MAX_ARGUMENTS_BYTES: usize = 16384;
const MAX_ARGUMENT_JSON: usize = 32768;
pub struct RunCommand {
    root: PathBuf,
    plans: Mutex<HashMap<String, Plan>>,
    deadline: Duration,
    #[cfg(windows)]
    isolation: Option<Arc<isolation::Host>>,
}
struct Plan {
    request: ToolRequest,
    executable: PathBuf,
    metadata: Metadata,
    invocation: CommandSpec,
    launch: Launch,
}
impl Plan {
    fn validate(&self, root: &Path, cancel: &CancellationToken) -> Result<(), String> {
        if cancel.is_cancelled() {
            return Err("Response stopped.".into());
        }
        if root.canonicalize().ok().as_deref() != Some(root) {
            return Err("Working folder changed. Review a fresh command.".into());
        }
        let metadata = self
            .executable
            .metadata()
            .map_err(|_| "Executable changed. Review a fresh command.")?;
        if metadata.len() != self.metadata.len()
            || metadata.modified().ok() != self.metadata.modified().ok()
            || metadata.permissions() != self.metadata.permissions()
        {
            return Err("Executable changed. Review a fresh command.".into());
        }
        Ok(())
    }
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
    #[serde(default)]
    isolation: Isolation,
    #[serde(default)]
    readonly_roots: Vec<PathBuf>,
}
struct CancelOnDrop(CancellationToken);
impl Drop for CancelOnDrop {
    fn drop(&mut self) {
        self.0.cancel();
    }
}

fn valid(spec: &CommandSpec) -> bool {
    !spec.program.trim().is_empty()
        && spec.program.len() <= MAX_ARGUMENT_BYTES
        && !spec.program.chars().any(char::is_control)
        && spec.args.len() <= 32
        && spec
            .args
            .iter()
            .all(|s| s.len() <= MAX_ARGUMENT_BYTES && !s.contains('\0'))
        && spec.args.iter().map(String::len).sum::<usize>() <= MAX_ARGUMENTS_BYTES
}
fn resolve(root: &Path, program: &str) -> Result<PathBuf, String> {
    resolve_from_path(root, program, &std::env::var_os("PATH").unwrap_or_default())
}
fn resolve_from_path(
    root: &Path,
    program: &str,
    search: &std::ffi::OsStr,
) -> Result<PathBuf, String> {
    let program = Path::new(program);
    let candidates = if program.is_absolute() {
        vec![program.to_path_buf()]
    } else if program.components().count() > 1 {
        vec![root.join(program)]
    } else {
        std::env::split_paths(search)
            .map(|entry| {
                if entry.is_absolute() {
                    entry.join(program)
                } else {
                    root.join(entry).join(program)
                }
            })
            .collect()
    };
    for candidate in candidates {
        let mut names = Vec::new();
        if cfg!(windows) && candidate.extension().is_none() {
            names.push(candidate.with_extension("exe"));
            names.push(candidate.with_extension("cmd"));
        }
        names.push(candidate);
        for name in names {
            let Ok(file) = name.canonicalize() else {
                continue;
            };
            if !file.is_file() {
                continue;
            }
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                if file
                    .metadata()
                    .map_or(true, |m| m.permissions().mode() & 0o111 == 0)
                {
                    continue;
                }
            }
            return Ok(file);
        }
    }
    Err("Program was not found. Use an executable on PATH or provide an absolute or working-folder-relative executable path. Restart Dolores if PATH changed after installation. Wrapper scripts require their shell or interpreter as the program.".into())
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
        // Rust/MSVC discovery uses these standard installation directories.
        // No compiler overrides, credentials or arbitrary caller environment.
        "ProgramFiles",
        "ProgramFiles(x86)",
        "ProgramW6432",
        "PROCESSOR_ARCHITECTURE",
        "PROCESSOR_ARCHITEW6432",
        "ProgramData",
        "ALLUSERSPROFILE",
        "COMSPEC",
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
    fn take_plan(&self, request: &ToolRequest) -> Result<Plan, String> {
        let plan = self
            .plans
            .lock()
            .map_err(|_| "Command review is unavailable.")?
            .remove(&request.call_id)
            .ok_or("Command review expired.")?;
        if request != &plan.request {
            return Err("Approved command changed.".into());
        }
        Ok(plan)
    }
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
            #[cfg(windows)]
            isolation: None,
        })
    }
    fn execute(
        root: PathBuf,
        plan: Plan,
        deadline: Duration,
        cancel: CancellationToken,
    ) -> Result<String, String> {
        plan.validate(&root, &cancel)?;
        let (mut process, stdin, stdout, stderr) = plan.launch.spawn(&plan, &root, &cancel)?;
        drop(stdin);
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
        let cleanup = process.stop_checked(); // Also stops descendants after their parent exits.
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
        value["execution"] = json!(plan.launch.label());
        if let Some(grants) = plan.launch.preview() {
            value["isolation"] = json!(grants);
        }
        value["cleanupComplete"] = json!(cleanup);
        if !cleanup {
            value["reason"] = json!("cleanupIncomplete");
        }
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
    execution::describe(ToolSpec{name:"run_command".into(),description:"Run an executable by its PATH name, absolute path, or working-folder-relative path, with literal arguments and the current command access mode. There is no program-name allowlist. Shells and interpreters can be invoked explicitly; no implicit shell/batch expansion. At most 32 arguments, 8 KiB UTF-8 each and 16 KiB total, within 32 KiB JSON; create a script file and run it for larger checks. Runs with user permissions, may change files outside the folder or use network; NOT sandboxed and file effects are NOT journaled/reverted. Closed stdin, filtered environment, default 30-second deadline and 8 KiB combined capture; explicit timeout_seconds (1–300) and capture_bytes (1024–262144) are reviewed; larger logs stay local in a named JSON artifact with a bounded model preview, one-use review. Output is untrusted data.".into(),parameters:json!({"type":"object","properties":{"program":{"type":"string","minLength":1,"description":"Executable name on PATH, absolute path, or path relative to the working folder."},"args":{"type":"array","items":{"type":"string"},"maxItems":32},"timeout_seconds":{"type":"integer","minimum":1,"maximum":300},"capture_bytes":{"type":"integer","minimum":1024,"maximum":262144}},"required":["program","args"],"additionalProperties":false})})
}
pub fn command_spec_for(root: &Path) -> ToolSpec {
    let mut spec = command_spec();
    spec.description.push_str(&developer_tools::guidance(root));
    spec
}
#[async_trait]
impl ToolPlugin for RunCommand {
    fn spec(&self) -> ToolSpec {
        command_spec_for(&self.root)
    }
    fn prepare(&self, call: &ToolCall) -> Result<ToolRequest, String> {
        if call.name != "run_command" || call.arguments.len() > MAX_ARGUMENT_JSON {
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
            return Err("Command arguments exceed the supported shape or size. Provide an executable name or path up to 8 KiB with no control characters, and at most 32 literal arguments, 8 KiB per argument and 16 KiB total, with no NUL characters. For larger validation scripts, create a script file and run that file.".into());
        }
        let invocation = developer_tools::normalize(&self.root, invocation)?;
        let executable = resolve(&self.root, &invocation.program)?;
        let launch = self.launch(&args.isolation, &executable, &args.readonly_roots)?;
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
                isolation: launch.preview(),
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
                launch,
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
        let plan = self.take_plan(request)?;
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
