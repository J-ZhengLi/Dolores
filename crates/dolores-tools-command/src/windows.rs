use std::{
    ffi::OsStr,
    fs::File,
    io::{self, Read},
    mem::{size_of, size_of_val, ManuallyDrop},
    os::windows::{
        ffi::OsStrExt,
        io::{AsRawHandle, FromRawHandle},
    },
    path::Path,
    ptr::{null, null_mut},
};
use windows_sys::Win32::{
    Foundation::*,
    Security::SECURITY_ATTRIBUTES,
    System::{
        JobObjects::*,
        Pipes::{CreatePipe, PeekNamedPipe},
        Threading::*,
    },
};

struct Handle(HANDLE);
impl Drop for Handle {
    fn drop(&mut self) {
        if !self.0.is_null() && self.0 != INVALID_HANDLE_VALUE {
            unsafe {
                CloseHandle(self.0);
            }
        }
    }
}
impl Handle {
    fn file(self) -> File {
        let owned = ManuallyDrop::new(self);
        unsafe { File::from_raw_handle(owned.0) }
    }
}
pub(super) struct Running {
    process: Handle,
    job: Option<Handle>,
}
impl Running {
    pub fn try_exit(&mut self) -> io::Result<Option<i32>> {
        unsafe {
            match WaitForSingleObject(self.process.0, 0) {
                WAIT_TIMEOUT => Ok(None),
                WAIT_OBJECT_0 => {
                    let mut code = 0;
                    if GetExitCodeProcess(self.process.0, &mut code) == 0 {
                        Err(io::Error::last_os_error())
                    } else {
                        Ok(Some(code as i32))
                    }
                }
                _ => Err(io::Error::last_os_error()),
            }
        }
    }
    pub fn stop(&mut self) {
        if let Some(job) = self.job.take() {
            unsafe {
                TerminateJobObject(job.0, 1);
            };
            drop(job);
            unsafe {
                WaitForSingleObject(self.process.0, 200);
            };
        }
    }
}
impl Drop for Running {
    fn drop(&mut self) {
        self.stop();
    }
}
struct Attributes {
    storage: Vec<usize>,
}
impl Attributes {
    fn pointer(&mut self) -> LPPROC_THREAD_ATTRIBUTE_LIST {
        self.storage.as_mut_ptr() as LPPROC_THREAD_ATTRIBUTE_LIST
    }
    fn new(handles: &[HANDLE]) -> Result<Self, String> {
        let mut size = 0;
        unsafe {
            InitializeProcThreadAttributeList(null_mut(), 1, 0, &mut size);
        };
        if size == 0 {
            return Err("Command handle isolation is unavailable.".into());
        }
        let mut storage = vec![0usize; size.div_ceil(size_of::<usize>())];
        let pointer = storage.as_mut_ptr() as LPPROC_THREAD_ATTRIBUTE_LIST;
        if unsafe { InitializeProcThreadAttributeList(pointer, 1, 0, &mut size) } == 0 {
            return Err("Command handle isolation is unavailable.".into());
        }
        let mut list = Self { storage };
        if unsafe {
            UpdateProcThreadAttribute(
                list.pointer(),
                0,
                PROC_THREAD_ATTRIBUTE_HANDLE_LIST as usize,
                handles.as_ptr().cast(),
                size_of_val(handles),
                null_mut(),
                null(),
            )
        } == 0
        {
            return Err("Command handle isolation is unavailable.".into());
        }
        Ok(list)
    }
}
impl Drop for Attributes {
    fn drop(&mut self) {
        unsafe {
            DeleteProcThreadAttributeList(self.pointer());
        }
    }
}
fn wide(value: &OsStr) -> Vec<u16> {
    value.encode_wide().chain(Some(0)).collect()
}
// Windows CRT argument quoting: every argument remains one literal value.
fn quote(value: &str) -> String {
    let mut result = String::from("\"");
    let mut slashes = 0;
    for character in value.chars() {
        if character == '\\' {
            slashes += 1;
            continue;
        }
        if character == '"' {
            result.push_str(&"\\".repeat(slashes * 2 + 1));
        } else {
            result.push_str(&"\\".repeat(slashes));
        }
        slashes = 0;
        result.push(character);
    }
    result.push_str(&"\\".repeat(slashes * 2));
    result.push('"');
    result
}
fn pipe() -> Result<(Handle, Handle), String> {
    let mut read = null_mut();
    let mut write = null_mut();
    let attributes = SECURITY_ATTRIBUTES {
        nLength: size_of::<SECURITY_ATTRIBUTES>() as u32,
        lpSecurityDescriptor: null_mut(),
        bInheritHandle: 1,
    };
    if unsafe { CreatePipe(&mut read, &mut write, &attributes, 4096) } == 0 {
        return Err("Command pipes are unavailable.".into());
    }
    Ok((Handle(read), Handle(write)))
}
pub(super) fn spawn(
    executable: &Path,
    args: &[String],
    root: &Path,
    env: &[(String, String)],
) -> Result<(Running, File, File), String> {
    let (stdout, out_write) = pipe()?;
    let (stderr, err_write) = pipe()?;
    let (stdin, in_write) = pipe()?;
    drop(in_write); // Closed input: the child receives EOF, never an interactive prompt.
    for handle in [&stdout, &stderr] {
        if unsafe { SetHandleInformation(handle.0, HANDLE_FLAG_INHERIT, 0) } == 0 {
            return Err("Command pipes are unavailable.".into());
        }
    }
    let handles = [stdin.0, out_write.0, err_write.0];
    let mut attributes = Attributes::new(&handles)?;
    let job = Handle(unsafe { CreateJobObjectW(null(), null()) });
    if job.0.is_null() {
        return Err("Command process control is unavailable.".into());
    }
    let mut limits: JOBOBJECT_EXTENDED_LIMIT_INFORMATION = unsafe { std::mem::zeroed() };
    limits.BasicLimitInformation.LimitFlags = JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE;
    if unsafe {
        SetInformationJobObject(
            job.0,
            JobObjectExtendedLimitInformation,
            (&limits as *const JOBOBJECT_EXTENDED_LIMIT_INFORMATION).cast(),
            size_of_val(&limits) as u32,
        )
    } == 0
    {
        return Err("Command process control is unavailable.".into());
    }
    let executable_wide = wide(executable.as_os_str());
    let directory = wide(root.as_os_str());
    let command = std::iter::once(executable.to_string_lossy().into_owned())
        .chain(args.iter().cloned())
        .map(|s| quote(&s))
        .collect::<Vec<_>>()
        .join(" ");
    let mut command = wide(OsStr::new(&command));
    let mut environment = Vec::new();
    for (key, value) in env {
        environment.extend(format!("{key}={value}").encode_utf16());
        environment.push(0);
    }
    environment.push(0);
    let mut startup: STARTUPINFOEXW = unsafe { std::mem::zeroed() };
    startup.StartupInfo.cb = size_of::<STARTUPINFOEXW>() as u32;
    startup.StartupInfo.dwFlags = STARTF_USESTDHANDLES;
    startup.StartupInfo.hStdInput = stdin.0;
    startup.StartupInfo.hStdOutput = out_write.0;
    startup.StartupInfo.hStdError = err_write.0;
    startup.lpAttributeList = attributes.pointer();
    let mut info: PROCESS_INFORMATION = unsafe { std::mem::zeroed() };
    if unsafe {
        CreateProcessW(
            executable_wide.as_ptr(),
            command.as_mut_ptr(),
            null(),
            null(),
            1,
            CREATE_SUSPENDED
                | CREATE_NO_WINDOW
                | CREATE_UNICODE_ENVIRONMENT
                | EXTENDED_STARTUPINFO_PRESENT,
            environment.as_ptr().cast(),
            directory.as_ptr(),
            &startup.StartupInfo,
            &mut info,
        )
    } == 0
    {
        return Err("Command could not be started.".into());
    }
    let process = Handle(info.hProcess);
    let thread = Handle(info.hThread);
    if unsafe { AssignProcessToJobObject(job.0, process.0) } == 0 {
        unsafe {
            TerminateProcess(process.0, 1);
            WaitForSingleObject(process.0, 200);
        };
        return Err("Command process control is unavailable. Nothing was run.".into());
    }
    let mut running = Running {
        process,
        job: Some(job),
    };
    if unsafe { ResumeThread(thread.0) } == u32::MAX {
        running.stop();
        return Err("Command could not be started.".into());
    }
    drop((thread, stdin, out_write, err_write, attributes));
    Ok((running, stdout.file(), stderr.file()))
}
pub(super) fn read_available(pipe: &mut File, buffer: &mut [u8]) -> io::Result<usize> {
    let mut available = 0;
    if unsafe {
        PeekNamedPipe(
            pipe.as_raw_handle(),
            null_mut(),
            0,
            null_mut(),
            &mut available,
            null_mut(),
        )
    } == 0
    {
        let error = io::Error::last_os_error();
        if error.raw_os_error() == Some(ERROR_BROKEN_PIPE as i32) {
            return Ok(0);
        }
        return Err(error);
    }
    if available == 0 {
        return Err(io::ErrorKind::WouldBlock.into());
    }
    let count = buffer.len().min(available as usize);
    pipe.read(&mut buffer[..count])
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn quotes_empty_spaces_quotes_and_trailing_slashes() {
        assert_eq!(quote(""), "\"\"");
        assert_eq!(quote("a b"), "\"a b\"");
        assert_eq!(quote("a\"b"), "\"a\\\"b\"");
        assert_eq!(quote("a\\"), "\"a\\\\\"");
    }
}
