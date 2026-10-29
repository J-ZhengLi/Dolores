//! Supervised command-hook transport. JSON goes to stdin, never shell argv.
use dolores_core::CommandSpec;
use dolores_tools_command::process;
use std::{
    io::Write,
    path::{Path, PathBuf},
    time::{Duration, Instant},
};
use tokio_util::sync::CancellationToken;

#[derive(Clone)]
pub(super) struct Shell {
    pub program: String,
    pub args: Vec<String>,
}
impl Shell {
    pub fn detect(root: &Path) -> Result<Self, String> {
        let program = std::env::var("DOLORES_CODEX_SHELL").unwrap_or_else(|_| {
            if cfg!(windows) {
                "cmd".into()
            } else {
                "/bin/sh".into()
            }
        });
        let launch = dolores_tools_command::resolve_launcher(
            root,
            CommandSpec {
                program,
                args: vec![],
            },
            &std::env::var_os("PATH").unwrap_or_default(),
        )?;
        let stem = Path::new(&launch.program)
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("")
            .to_ascii_lowercase();
        let args=match stem.as_str(){
            "powershell"|"pwsh"=>vec!["-NoLogo".into(),"-NoProfile".into(),"-NonInteractive".into(),"-Command".into()],
            "bash"|"sh"|"zsh"=>vec!["-c".into()],
            "cmd"=>vec!["/d".into(),"/s".into(),"/c".into()],
            _=>return Err("Unsupported Codex compatibility shell. Set DOLORES_CODEX_SHELL to bash, sh, pwsh, powershell or cmd before starting Dolores.".into()),
        };
        Ok(Self {
            // Windows PowerShell's .NET initialization rejects verbatim
            // executable spellings; ownership still resolves canonically.
            program: ordinary_path(&launch.program),
            args,
        })
    }
    pub fn command(&self, text: &str) -> CommandSpec {
        let mut args = self.args.clone();
        args.push(text.into());
        CommandSpec {
            program: self.program.clone(),
            args,
        }
    }
}
fn ordinary_path(path: &str) -> String {
    #[cfg(windows)]
    {
        if let Some(rest) = path.strip_prefix(r"\\?\UNC\") {
            return format!(r"\\{rest}");
        }
        if let Some(rest) = path
            .strip_prefix(r"\\?\")
            .filter(|p| p.as_bytes().get(1) == Some(&b':'))
        {
            return rest.into();
        }
    }
    path.into()
}
pub(super) struct Output {
    pub code: i32,
    pub stdout: String,
    pub stderr: String,
}
struct CancelOnDrop(CancellationToken);
impl Drop for CancelOnDrop {
    fn drop(&mut self) {
        self.0.cancel();
    }
}
pub(super) async fn run(
    root: PathBuf,
    home: PathBuf,
    shell: Shell,
    command: String,
    input: serde_json::Value,
    seconds: u64,
    cancel: CancellationToken,
) -> Result<Output, String> {
    let child = cancel.child_token();
    let _guard = CancelOnDrop(child.clone());
    tokio::task::spawn_blocking(move || {
        let mut environment = process::environment();
        #[cfg(windows)]
        super::mcp::set_env(&mut environment, "PATHEXT", ".COM;.EXE;.BAT;.CMD".into());
        super::mcp::set_env(
            &mut environment,
            "CODEX_HOME",
            home.to_string_lossy().into(),
        );
        let launch = shell.command(&command);
        let (mut child_process, mut stdin, mut stdout, mut stderr) = process::spawn_stdio(
            Path::new(&launch.program),
            &launch.args,
            &root,
            &environment,
        )?;
        let input = serde_json::to_vec(&input).map_err(|_| "Hook input cannot be encoded.")?;
        let writer = std::thread::spawn(move || {
            let result = stdin.write_all(&input).and_then(|_| stdin.write_all(b"\n"));
            drop(stdin);
            result
        });
        let deadline = Instant::now() + Duration::from_secs(seconds);
        let mut out = vec![];
        let mut err = vec![];
        let result = (|| {
            loop {
                if child.is_cancelled() {
                    return Err(crate::stopped());
                }
                if Instant::now() >= deadline {
                    return Err("Codex hook timed out. It was not replayed.".into());
                }
                for (pipe, bytes) in [(&mut stdout, &mut out), (&mut stderr, &mut err)] {
                    let mut buffer = [0; 4096];
                    match process::read_available(pipe, &mut buffer) {
                        Ok(count) => bytes.extend_from_slice(&buffer[..count]),
                        Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {}
                        Err(_) => return Err("Codex hook output could not be read.".into()),
                    }
                    if bytes.len() > 256 * 1024 {
                        return Err(
                            "Codex hook output exceeds 256 KiB. Keep hook output concise.".into(),
                        );
                    }
                }
                if let Some(code) = child_process
                    .try_exit()
                    .map_err(|_| "Codex hook exit status unavailable.")?
                {
                    // Pipe reads above are intentionally incremental. Drain bytes
                    // left by a process that exited between reads.
                    for (pipe, bytes) in [(&mut stdout, &mut out), (&mut stderr, &mut err)] {
                        loop {
                            let mut buffer = [0; 4096];
                            match process::read_available(pipe, &mut buffer) {
                                Ok(0) => break,
                                Ok(n) => bytes.extend_from_slice(&buffer[..n]),
                                Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => break,
                                Err(_) => return Err("Codex hook output could not be read.".into()),
                            }
                            if bytes.len() > 256 * 1024 {
                                return Err("Codex hook output exceeds 256 KiB.".into());
                            }
                        }
                    }
                    return Ok(Output {
                        code,
                        stdout: String::from_utf8(out)
                            .map_err(|_| "Codex hook output is not UTF-8.")?,
                        stderr: String::from_utf8(err)
                            .map_err(|_| "Codex hook diagnostics are not UTF-8.")?,
                    });
                }
                std::thread::sleep(Duration::from_millis(5));
            }
        })();
        let cleaned = child_process.stop_checked();
        let _ = writer.join();
        if !cleaned {
            return Err(
                "Codex hook cleanup is incomplete. Inspect its external effects before retrying."
                    .into(),
            );
        }
        result
    })
    .await
    .map_err(|_| "Codex hook worker failed.")?
}
