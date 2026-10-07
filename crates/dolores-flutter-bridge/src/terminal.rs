//! Human-only PTY sessions. No model tool is registered for this registry.
use crate::Engine;
use base64::{engine::general_purpose::STANDARD, Engine as _};
use portable_pty::{native_pty_system, Child, CommandBuilder, MasterPty, PtySize};
use serde::Deserialize;
use serde_json::{json, Value};
use std::{
    collections::{HashMap, VecDeque},
    io::{Read, Write},
    path::{Path, PathBuf},
    sync::{
        atomic::{AtomicBool, Ordering},
        mpsc::{sync_channel, SyncSender},
        Arc, Condvar, Mutex,
    },
    time::{Duration, Instant},
};
#[path = "terminal_recovery.rs"]
mod recovery;
#[path = "terminal_tree.rs"]
mod tree;
const QUEUE_BYTES: usize = 1024 * 1024;
#[derive(Deserialize)]
#[serde(tag = "action", rename_all = "camelCase")]
pub(crate) enum Request {
    Create {
        session: Option<String>,
        #[serde(default)]
        home: bool,
        shell: Option<String>,
    },
    List,
    Restore,
    Checkpoint {
        value: Value,
    },
    Share {
        session: String,
        text: String,
    },
    Poll {
        id: String,
    },
    Input {
        id: String,
        text: String,
    },
    Resize {
        id: String,
        rows: u16,
        cols: u16,
    },
    Stop {
        id: String,
    },
    Close {
        id: String,
    },
}
#[derive(Default)]
struct Output {
    bytes: VecDeque<u8>,
    eof: bool,
    error: bool,
    paused: bool,
}
type Queue = Arc<(Mutex<Output>, Condvar)>;
struct Session {
    id: String,
    cwd: PathBuf,
    shell: PathBuf,
    pid: u32,
    state: &'static str,
    master: Option<Box<dyn MasterPty + Send>>,
    child: Box<dyn Child + Send + Sync>,
    writer: Option<SyncSender<Vec<u8>>>,
    output: Queue,
    stopping: Arc<AtomicBool>,
    tree: Option<tree::Tree>,
    threads: Vec<std::thread::JoinHandle<()>>,
}
#[derive(Default)]
pub(crate) struct Registry {
    sessions: HashMap<String, Session>,
    closed: bool,
}
fn size(rows: u16, cols: u16) -> Result<PtySize, String> {
    if !(2..=500).contains(&rows) || !(2..=500).contains(&cols) {
        return Err("Terminal size must be between 2 and 500 rows/columns.".into());
    }
    Ok(PtySize {
        rows,
        cols,
        pixel_width: 0,
        pixel_height: 0,
    })
}
pub(crate) fn executable(names: &[&str]) -> Option<PathBuf> {
    let paths = std::env::var_os("PATH")?;
    for name in names {
        for dir in std::env::split_paths(&paths).filter(|p| p.is_absolute()) {
            let file = dir.join(name);
            if file.is_file() {
                return file.canonicalize().ok();
            }
        }
    }
    None
}
fn shell(choice: Option<&str>) -> Result<PathBuf, String> {
    #[cfg(windows)]
    let names: &[&str] = match choice {
        Some("cmd") => &["cmd.exe"],
        None | Some("powershell") => &["pwsh.exe", "powershell.exe"],
        _ => return Err("Choose PowerShell or Command Prompt.".into()),
    };
    #[cfg(not(windows))]
    let names: &[&str] = match choice {
        None | Some("sh") => &["bash", "sh"],
        _ => return Err("Choose an installed shell.".into()),
    };
    executable(names).ok_or_else(|| "No supported shell was found. Install a shell, then Retry. Home and editing remain available.".into())
}
impl Session {
    fn view(&self) -> Value {
        json!({"id":self.id,"cwd":self.cwd,"shell":self.shell,"pid":self.pid,"state":self.state})
    }
    fn stop(&mut self) -> Result<(), String> {
        self.stopping.store(true, Ordering::Release);
        self.output.1.notify_all();
        self.writer.take();
        self.tree.take();
        let _ = self.child.kill();
        self.master.take();
        let start = Instant::now();
        while start.elapsed() < Duration::from_secs(2) {
            if self.child.try_wait().ok().flatten().is_some()
                && self.threads.iter().all(|t| t.is_finished())
            {
                for t in self.threads.drain(..) {
                    let _ = t.join();
                }
                if self.state != "exited" {
                    self.state = "stopped";
                }
                return Ok(());
            }
            std::thread::sleep(Duration::from_millis(10));
        }
        self.state = "unknown";
        Err("Terminal cleanup is still pending. Retained output is available; Retry Stop before closing.".into())
    }
}
impl Drop for Session {
    fn drop(&mut self) {
        let _ = self.stop();
    }
}
impl Registry {
    pub(crate) fn live(&self) -> bool {
        self.sessions
            .values()
            .any(|s| matches!(s.state, "running" | "unknown"))
    }
    fn create(&mut self, cwd: &Path, choice: Option<&str>) -> Result<Value, String> {
        if self.closed {
            return Err("Terminals are closing. Restart Dolores.".into());
        }
        if self.sessions.len() >= 8 {
            return Err(
                "Eight terminal sessions are retained. Close one before opening another.".into(),
            );
        }
        let cwd = cwd.canonicalize().map_err(|_| {
            "Terminal folder is unavailable. Choose a folder, Open at home, or Retry."
        })?;
        if !cwd.is_dir() {
            return Err(
                "Terminal folder is not a directory. Choose a folder or Open at home.".into(),
            );
        }
        let shell = shell(choice)?;
        let pair = native_pty_system()
            .openpty(size(24, 80)?)
            .map_err(|_| "PTY initialization failed. Home/editing remain available; Retry.")?;
        let mut command = CommandBuilder::new(&shell);
        command.cwd(cwd.to_string_lossy().trim_start_matches(r"\\?\"));
        command.env_clear();
        for (key, value) in dolores_tools_command::process::environment() {
            if !matches!(
                key.as_str(),
                "CI" | "NO_COLOR" | "TERM" | "GIT_TERMINAL_PROMPT"
            ) {
                command.env(key, value);
            }
        }
        command.env("TERM", "xterm-256color");
        #[cfg(windows)]
        if !shell
            .file_name()
            .unwrap()
            .to_string_lossy()
            .eq_ignore_ascii_case("cmd.exe")
        {
            command.args(["-NoLogo", "-NoProfile"]);
        }
        #[cfg(windows)]
        if shell
            .file_name()
            .unwrap()
            .to_string_lossy()
            .eq_ignore_ascii_case("cmd.exe")
        {
            command.arg("/d");
        }
        let mut child = pair
            .slave
            .spawn_command(command)
            .map_err(|_| "The shell could not start. Check its installation and Retry.")?;
        drop(pair.slave);
        let pid = child
            .process_id()
            .ok_or("Terminal process identity is unavailable.")?;
        let tree = match tree::Tree::attach(pid) {
            Ok(t) => t,
            Err(e) => {
                let _ = child.kill();
                return Err(e);
            }
        };
        let mut reader = pair
            .master
            .try_clone_reader()
            .map_err(|_| "Terminal output is unavailable.")?;
        let mut writer = pair
            .master
            .take_writer()
            .map_err(|_| "Terminal input is unavailable.")?;
        let output: Queue = Arc::new((Mutex::new(Output::default()), Condvar::new()));
        let stopping = Arc::new(AtomicBool::new(false));
        let q = output.clone();
        let stop = stopping.clone();
        let read = std::thread::spawn(move || {
            let mut bytes = [0u8; 8192];
            loop {
                let count = match reader.read(&mut bytes) {
                    Ok(0) => break,
                    Ok(n) => n,
                    Err(_) => {
                        if !stop.load(Ordering::Acquire) {
                            q.0.lock().unwrap().error = true;
                        }
                        break;
                    }
                };
                let mut state = q.0.lock().unwrap();
                while state.bytes.len() + count > QUEUE_BYTES && !stop.load(Ordering::Acquire) {
                    state.paused = true;
                    state = q.1.wait(state).unwrap();
                }
                if stop.load(Ordering::Acquire) {
                    break;
                }
                state.bytes.extend(&bytes[..count]);
            }
            q.0.lock().unwrap().eof = true;
        });
        let (input, receive) = sync_channel::<Vec<u8>>(64);
        let q = output.clone();
        let write = std::thread::spawn(move || {
            while let Ok(bytes) = receive.recv() {
                if writer
                    .write_all(&bytes)
                    .and_then(|_| writer.flush())
                    .is_err()
                {
                    q.0.lock().unwrap().error = true;
                    break;
                }
            }
        });
        let id = uuid::Uuid::new_v4().to_string();
        let session = Session {
            id: id.clone(),
            cwd,
            shell,
            pid,
            state: "running",
            master: Some(pair.master),
            child,
            writer: Some(input),
            output,
            stopping,
            tree: Some(tree),
            threads: vec![read, write],
        };
        let value = session.view();
        self.sessions.insert(id, session);
        Ok(value)
    }
    fn request(&mut self, request: Request) -> Result<Value, String> {
        let id = match &request {
            Request::Poll { id }
            | Request::Input { id, .. }
            | Request::Resize { id, .. }
            | Request::Stop { id }
            | Request::Close { id } => id,
            _ => return Err("Invalid terminal operation.".into()),
        };
        let s = self
            .sessions
            .get_mut(id)
            .ok_or("Terminal is no longer available. Open another tab.")?;
        match request {
            Request::Poll { .. } => {
                if s.state == "running"
                    && s.child
                        .try_wait()
                        .map_err(|_| "Shell state unavailable. Retry Stop.")?
                        .is_some()
                {
                    s.state = "exited";
                    s.stop()?;
                }
                let mut output = s
                    .output
                    .0
                    .lock()
                    .map_err(|_| "Terminal output unavailable.")?;
                let count = output.bytes.len().min(64 * 1024);
                let bytes = output.bytes.drain(..count).collect::<Vec<_>>();
                let paused = output.paused;
                output.paused = false;
                s.output.1.notify_all();
                Ok(
                    json!({"session":s.view(),"bytes":STANDARD.encode(bytes),"backpressure":paused,"outputError":output.error,"eof":output.eof && output.bytes.is_empty()}),
                )
            }
            Request::Input { text, .. } => {
                if s.state != "running" {
                    return Err(
                        "This shell stopped. Open another terminal; retained output remains."
                            .into(),
                    );
                }
                if text.len() > 16 * 1024 {
                    return Err("Paste exceeds 16 KiB. Paste smaller sections.".into());
                }
                s.writer
                    .as_ref()
                    .ok_or("Terminal input is closed.")?
                    .try_send(text.into_bytes())
                    .map_err(|_| "Terminal input is busy. Wait, then retry the input.")?;
                Ok(Value::Null)
            }
            Request::Resize { rows, cols, .. } => {
                let size = size(rows, cols)?;
                s.master
                    .as_ref()
                    .ok_or("Terminal stopped.")?
                    .resize(size)
                    .map_err(|_| "Terminal resize failed. Retry.")?;
                Ok(Value::Null)
            }
            Request::Stop { .. } => {
                s.stop()?;
                Ok(s.view())
            }
            Request::Close { id } => {
                s.stop()?;
                self.sessions.remove(&id);
                Ok(Value::Null)
            }
            _ => unreachable!(),
        }
    }
    pub(crate) fn stop_all(&mut self) -> Result<(), String> {
        self.closed = true;
        for session in self.sessions.values_mut() {
            session.stop()?;
        }
        Ok(())
    }
}
impl Engine {
    pub(crate) fn terminal_call(&self, request: Request) -> Result<Value, String> {
        match &request {
            Request::Restore => {
                let mut value = self.store.editor_state(recovery::KEY)?;
                if !value.is_null() {
                    recovery::validate(&value)?;
                    for s in value["sessions"].as_array_mut().unwrap() {
                        s["state"] = json!("stopped");
                    }
                }
                return Ok(value);
            }
            Request::Checkpoint { value } => {
                recovery::validate(value)?;
                self.store.save_editor_state(recovery::KEY, value)?;
                return Ok(json!({"saved":true}));
            }
            Request::Share { session, text } => {
                use sha2::{Digest, Sha256};
                if text.is_empty() || text.len() > 8192 || text.contains('\0') {
                    return Err(
                        "Select up to 8 KiB of terminal output to attach. Nothing was shared."
                            .into(),
                    );
                }
                let data = dolores_core::AttachmentData {
                    reference: dolores_core::AttachmentRef {
                        digest: format!("{:x}", Sha256::digest(text.as_bytes())),
                        name: "Terminal output.txt".into(),
                        mime: "text/plain".into(),
                        bytes: text.len(),
                    },
                    data: text.as_bytes().to_vec(),
                };
                self.store.add_attachment(session, &data)?;
                return Ok(json!(self.store.draft_attachments(session)?));
            }
            _ => {}
        }
        let mut state = self
            .terminals
            .lock()
            .map_err(|_| "Terminal state is unavailable.")?;
        match request {
            Request::Create {
                session,
                home,
                shell,
            } => {
                let root = if home {
                    None
                } else {
                    session
                        .map(|s| self.store.workspace(&s))
                        .transpose()?
                        .and_then(|w| w.root)
                };
                let cwd = root
                    .map(PathBuf::from)
                    .or_else(|| directories::BaseDirs::new().map(|b| b.home_dir().to_owned()))
                    .ok_or("OS home directory is unavailable.")?;
                state.create(&cwd, shell.as_deref())
            }
            Request::List => Ok(json!(state
                .sessions
                .values()
                .map(|s| s.view())
                .collect::<Vec<_>>())),
            operation => state.request(operation),
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn display_checkpoint_and_explicit_selection_share_survive_restart() {
        use dolores_core::SessionStore;
        let dir = tempfile::tempdir().unwrap();
        let store =
            Arc::new(dolores_store_sqlite::SqliteStore::open(&dir.path().join("test.db")).unwrap());
        store.create("chosen").unwrap();
        store.create("other").unwrap();
        let e = Engine::new(store.clone(), Arc::new(crate::connection::testing::MemoryCredentials::default())).unwrap();
        let v = json!({"version":1,"sessions":[{"id":"old","cwd":"C:/","shell":"cmd","title":"Shell","output":"kept 世界"}],"groups":[{"id":"g","tabs":["old"],"active":"old"}],"activeGroup":"g","tree":{"group":"g"}});
        e.terminal_call(Request::Checkpoint { value: v.clone() })
            .unwrap();
        let mut bad = v.clone();
        bad["sessions"][0]["output"] = json!("x".repeat(8193));
        assert!(e.terminal_call(Request::Checkpoint { value: bad }).is_err());
        assert_eq!(store.editor_state(recovery::KEY).unwrap(), v);
        let parts = e
            .terminal_call(Request::Share {
                session: "chosen".into(),
                text: "selected 世界".into(),
            })
            .unwrap();
        assert!(store.draft_attachments("other").unwrap().is_empty());
        assert_eq!(
            store
                .attachment_data("chosen", parts[0]["digest"].as_str().unwrap())
                .unwrap()
                .data,
            "selected 世界".as_bytes()
        );
        assert!(e
            .terminal_call(Request::Share {
                session: "chosen".into(),
                text: "x".repeat(8193)
            })
            .is_err());
        drop(e);
        let e = Engine::new(store, Arc::new(crate::connection::testing::MemoryCredentials::default())).unwrap();
        assert_eq!(
            e.terminal_call(Request::Restore).unwrap()["sessions"][0]["state"],
            "stopped"
        );
        assert_eq!(e.terminal_call(Request::List).unwrap(), json!([]));
    }
    #[test]
    fn real_pty_unicode_input_resize_missing_root_and_cleanup() {
        let folder = tempfile::tempdir().unwrap();
        let mut r = Registry::default();
        assert!(r.create(&folder.path().join("missing"), None).is_err());
        #[cfg(windows)]
        let choice = Some("cmd");
        #[cfg(not(windows))]
        let choice = None;
        let value = r.create(folder.path(), choice).unwrap();
        let id = value["id"].as_str().unwrap().to_string();
        r.request(Request::Resize {
            id: id.clone(),
            rows: 40,
            cols: 100,
        })
        .unwrap();
        r.request(Request::Input {
            id: id.clone(),
            text: "echo DOL_PTY_READY\r\n".into(),
        })
        .unwrap();
        let started = Instant::now();
        let mut seen = vec![];
        while started.elapsed() < Duration::from_secs(10) {
            let v = r.request(Request::Poll { id: id.clone() }).unwrap();
            let chunk = STANDARD.decode(v["bytes"].as_str().unwrap()).unwrap();
            if String::from_utf8_lossy(&chunk).contains("\u{1b}[6n") {
                r.request(Request::Input {
                    id: id.clone(),
                    text: "\u{1b}[1;1R".into(),
                })
                .unwrap();
            }
            seen.extend(chunk);
            if String::from_utf8_lossy(&seen).contains("DOL_PTY_READY") {
                break;
            }
            std::thread::sleep(Duration::from_millis(20));
        }
        assert!(
            String::from_utf8_lossy(&seen).contains("DOL_PTY_READY"),
            "{}",
            String::from_utf8_lossy(&seen)
        );
        assert!(r
            .request(Request::Resize {
                id: id.clone(),
                rows: 0,
                cols: 100
            })
            .is_err());
        assert!(r
            .request(Request::Input {
                id: id.clone(),
                text: "x".repeat(17000)
            })
            .is_err());
        r.request(Request::Stop { id: id.clone() }).unwrap();
        assert!(!r.live());
        assert!(r
            .request(Request::Input {
                id: id.clone(),
                text: "echo no".into()
            })
            .is_err());
        r.request(Request::Close { id }).unwrap();
        assert!(r.sessions.is_empty());
    }
}
