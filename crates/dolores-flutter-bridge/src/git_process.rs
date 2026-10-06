//! Supervised binary Git transport; no shell or persistent idle process.
use dolores_tools_command::process;
use std::{
    io,
    path::{Path, PathBuf},
    time::{Duration, Instant},
};
use tokio_util::sync::CancellationToken;

pub(super) struct Output {
    pub code: i32,
    pub stdout: Vec<u8>,
    pub stderr: Vec<u8>,
}
pub(super) fn executable(root: &Path) -> Result<PathBuf, String> {
    for directory in std::env::split_paths(&std::env::var_os("PATH").unwrap_or_default()) {
        if !directory.is_absolute() {
            continue;
        }
        let candidate = directory.join(if cfg!(windows) { "git.exe" } else { "git" });
        if let Ok(file) = candidate.canonicalize() {
            if file.is_file() && !file.starts_with(root) {
                return Ok(file);
            }
        }
    }
    Err(
        "Git is unavailable. Install Git outside the project, then restart Dolores and Refresh."
            .into(),
    )
}
pub(super) fn run(
    executable: &Path,
    root: &Path,
    args: &[String],
    cancel: &CancellationToken,
    remote: bool,
) -> Result<Output, String> {
    let mut env = process::environment();
    env.extend([
        ("GIT_OPTIONAL_LOCKS".into(), "0".into()),
        ("GIT_TERMINAL_PROMPT".into(), "0".into()),
        ("GCM_INTERACTIVE".into(), "never".into()),
        ("GIT_LITERAL_PATHSPECS".into(), "1".into()),
    ]);
    let mut argv = vec![
        "--no-pager".into(),
        "-c".into(),
        "color.ui=false".into(),
        "-c".into(),
        "core.quotepath=false".into(),
    ];
    argv.extend_from_slice(args);
    if cancel.is_cancelled() {
        return Err(
            "Git operation stopped. Refresh to inspect any completed effects before retrying."
                .into(),
        );
    }
    let (mut process, input, mut out, mut err) =
        process::spawn_stdio(executable, &argv, root, &env)?;
    drop(input);
    let deadline = Instant::now();
    let mut stdout = vec![];
    let mut stderr = vec![];
    let mut eof = [false, false];
    let mut exit = None;
    loop {
        for (i, pipe) in [&mut out, &mut err].into_iter().enumerate() {
            if eof[i] {
                continue;
            }
            let mut buffer = [0; 8192];
            match process::read_available(pipe, &mut buffer) {
                Ok(0) => eof[i] = true,
                Ok(n) => {
                    if stdout.len() + stderr.len() + n > 512 * 1024 {
                        process.stop();
                        return Err("Git output exceeds 512 KiB. Partial results were refused; inspect a smaller repository with external Git, then Refresh.".into());
                    }
                    if i == 0 {
                        stdout.extend_from_slice(&buffer[..n]);
                    } else {
                        stderr.extend_from_slice(&buffer[..n]);
                    }
                }
                Err(e) if e.kind() == io::ErrorKind::WouldBlock => {}
                Err(_) => {
                    process.stop();
                    return Err("Git output could not be read. Refresh before retrying.".into());
                }
            }
        }
        if exit.is_none() {
            exit = process
                .try_exit()
                .map_err(|_| "Git process state unavailable.")?;
        }
        if let Some(code) = exit {
            if eof.iter().all(|x| *x) {
                process.stop();
                return Ok(Output {
                    code,
                    stdout,
                    stderr,
                });
            }
        }
        if cancel.is_cancelled()
            || deadline.elapsed() > Duration::from_secs(if remote { 60 } else { 30 })
        {
            process.stop();
            return Err("Git stopped or reached its deadline. Effects may remain; Refresh and reconcile before retrying.".into());
        }
        // Reap helpers after their parent completes, even if they retain a pipe.
        if exit.is_some() {
            process.stop();
        }
        std::thread::sleep(Duration::from_millis(2));
    }
}
