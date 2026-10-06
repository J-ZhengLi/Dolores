use super::*;
use windows_sys::Win32::{Foundation::*, System::Threading::*};
fn wide(value: &std::ffi::OsStr) -> Vec<u16> {
    use std::os::windows::ffi::OsStrExt;
    value.encode_wide().chain(Some(0)).collect()
}
struct Handle(HANDLE);
impl Drop for Handle {
    fn drop(&mut self) {
        unsafe {
            CloseHandle(self.0);
        }
    }
}
fn from_handle(pid: u32, handle: HANDLE) -> Result<Process, String> {
    unsafe {
        let mut code = 0;
        if GetExitCodeProcess(handle, &mut code) == 0 || code != 259 {
            return Err("Native process has exited.".into());
        }
        let mut path = vec![0u16; 32768];
        let mut len = path.len() as u32;
        if QueryFullProcessImageNameW(handle, 0, path.as_mut_ptr(), &mut len) == 0 {
            return Err("Native executable identity unavailable.".into());
        }
        let mut times = [FILETIME {
            dwLowDateTime: 0,
            dwHighDateTime: 0,
        }; 4];
        if GetProcessTimes(
            handle,
            &mut times[0],
            &mut times[1],
            &mut times[2],
            &mut times[3],
        ) == 0
        {
            return Err("Native process creation identity unavailable.".into());
        }
        use std::os::windows::ffi::OsStringExt;
        Ok(Process {
            pid,
            created: ((times[0].dwHighDateTime as u64) << 32) | times[0].dwLowDateTime as u64,
            executable: PathBuf::from(std::ffi::OsString::from_wide(&path[..len as usize])),
        })
    }
}
pub fn identity(pid: u32) -> Result<Option<Process>, String> {
    unsafe {
        let handle = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, 0, pid);
        if handle.is_null() {
            return if GetLastError() == ERROR_INVALID_PARAMETER {
                Ok(None)
            } else {
                Err("Native process identity unavailable; no process was stopped.".into())
            };
        }
        let handle = Handle(handle);
        let mut code = 0;
        if GetExitCodeProcess(handle.0, &mut code) == 0 {
            return Err("Native process state unavailable.".into());
        }
        if code != 259 {
            return Ok(None);
        }
        Ok(Some(from_handle(pid, handle.0)?))
    }
}
pub fn stop_failed_startup(expected: &Process) -> Result<(), String> {
    unsafe {
        let handle = OpenProcess(
            PROCESS_QUERY_LIMITED_INFORMATION | PROCESS_TERMINATE,
            0,
            expected.pid,
        );
        if handle.is_null() {
            return if GetLastError() == ERROR_INVALID_PARAMETER {
                Ok(())
            } else {
                Err("Candidate identity unavailable; stop/recovery needs inspection.".into())
            };
        }
        let handle = Handle(handle);
        let mut code = 0;
        if GetExitCodeProcess(handle.0, &mut code) == 0 {
            return Err("Candidate process status unavailable.".into());
        }
        if code != 259 {
            return Ok(());
        }
        if &from_handle(expected.pid, handle.0)? != expected {
            return Err("Candidate process changed; it was not stopped.".into());
        }
        if TerminateProcess(handle.0, 1) == 0 {
            return Err("Failed-startup candidate could not be stopped.".into());
        }
        Ok(())
    }
}
pub fn launch_recorded(job: &mut Handoff, receipt: &Path, root: &Path) -> Result<(), String> {
    unsafe {
        let executable = root.join(APP);
        safe_path(&executable)?;
        let app = wide(executable.as_os_str());
        let mut command = wide(std::ffi::OsStr::new(&format!(
            "\"{}\"",
            executable.display()
        )));
        let cwd = wide(root.as_os_str());
        let allowed = [
            "PATH",
            "SystemRoot",
            "WINDIR",
            "USERPROFILE",
            "APPDATA",
            "LOCALAPPDATA",
            "TEMP",
            "TMP",
            "ProgramFiles",
            "ProgramFiles(x86)",
            "ProgramW6432",
            "ProgramData",
            "ALLUSERSPROFILE",
            "COMSPEC",
            "PROCESSOR_ARCHITECTURE",
            "PROCESSOR_ARCHITEW6432",
            "CARGO_HOME",
            "RUSTUP_HOME",
            "HTTP_PROXY",
            "HTTPS_PROXY",
            "NO_PROXY",
            "DOLORES_GLOBAL_SKILLS_DIR",
        ];
        let mut env: Vec<_> = std::env::vars_os()
            .filter(|(k, _)| {
                allowed
                    .iter()
                    .any(|a| k.to_string_lossy().eq_ignore_ascii_case(a))
            })
            .collect();
        env.push((
            "DOLORES_DATA_DIR".into(),
            job.profile.clone().into_os_string(),
        ));
        env.push(("DOLORES_NATIVE_HANDOFF".into(), job.id.clone().into()));
        env.sort_by_key(|(k, _)| k.to_string_lossy().to_uppercase());
        let mut block = vec![];
        use std::os::windows::ffi::OsStrExt;
        for (k, v) in env {
            block.extend(k.encode_wide());
            block.push('=' as u16);
            block.extend(v.encode_wide());
            block.push(0);
        }
        block.push(0);
        let mut startup: STARTUPINFOW = std::mem::zeroed();
        startup.cb = std::mem::size_of::<STARTUPINFOW>() as u32;
        let mut info: PROCESS_INFORMATION = std::mem::zeroed();
        if CreateProcessW(
            app.as_ptr(),
            command.as_mut_ptr(),
            std::ptr::null(),
            std::ptr::null(),
            0,
            CREATE_SUSPENDED | CREATE_UNICODE_ENVIRONMENT,
            block.as_ptr().cast(),
            cwd.as_ptr(),
            &startup,
            &mut info,
        ) == 0
        {
            return Err("Candidate could not start; previous bundle remains.".into());
        }
        let process = Handle(info.hProcess);
        let thread = Handle(info.hThread);
        let result = (|| {
            job.child = Some(from_handle(info.dwProcessId, process.0)?);
            job.stage = "starting".into();
            save(receipt, job)?;
            if ResumeThread(thread.0) == u32::MAX {
                return Err("Candidate could not resume; previous bundle remains.".into());
            }
            Ok(())
        })();
        if result.is_err() {
            TerminateProcess(process.0, 1);
        }
        result
    }
}
