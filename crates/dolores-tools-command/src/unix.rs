use std::{
    fs::File,
    io::{self, Read},
    os::{
        fd::{AsRawFd, FromRawFd, IntoRawFd},
        unix::process::CommandExt,
    },
    path::Path,
    process::{Child, Command, Stdio},
};
pub(super) struct Running {
    child: Child,
    group: i32,
    stopped: bool,
}
impl Running {
    pub fn try_exit(&mut self) -> io::Result<Option<i32>> {
        Ok(self.child.try_wait()?.map(|s| s.code().unwrap_or(-1)))
    }
    pub fn stop(&mut self) {
        if !self.stopped {
            unsafe {
                libc::kill(-self.group, libc::SIGKILL);
            };
            let _ = self.child.wait();
            self.stopped = true;
        }
    }
}
impl Drop for Running {
    fn drop(&mut self) {
        self.stop();
    }
}
pub(super) fn spawn(
    executable: &Path,
    args: &[String],
    root: &Path,
    env: &[(String, String)],
) -> Result<(Running, File, File), String> {
    let mut child = Command::new(executable)
        .args(args)
        .current_dir(root)
        .env_clear()
        .envs(env.iter().cloned())
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .process_group(0)
        .spawn()
        .map_err(|_| "Command could not be started.")?;
    let group = child.id() as i32;
    let stdout = unsafe { File::from_raw_fd(child.stdout.take().unwrap().into_raw_fd()) };
    let stderr = unsafe { File::from_raw_fd(child.stderr.take().unwrap().into_raw_fd()) };
    let running = Running {
        child,
        group,
        stopped: false,
    };
    for pipe in [&stdout, &stderr] {
        let fd = pipe.as_raw_fd();
        let flags = unsafe { libc::fcntl(fd, libc::F_GETFL) };
        if flags < 0 || unsafe { libc::fcntl(fd, libc::F_SETFL, flags | libc::O_NONBLOCK) } < 0 {
            return Err("Command output could not be configured.".into());
        }
    }
    Ok((running, stdout, stderr))
}
pub(super) fn read_available(pipe: &mut File, buffer: &mut [u8]) -> io::Result<usize> {
    pipe.read(buffer)
}
