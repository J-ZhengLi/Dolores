//! Only an explicit foreground Sign in reaches this boundary. No URL is logged.
pub(crate) fn open(url: &str) -> Result<(), String> {
    let parsed = url::Url::parse(url)
        .map_err(|_| "Could not open the sign-in browser. Try Sign in again.")?;
    if url.len() > 16 * 1024 || url.chars().any(char::is_control) || parsed.fragment().is_some() {
        return Err("The sign-in browser address is invalid. Try Sign in again.".into());
    }
    let mut endpoint = parsed.clone();
    endpoint.set_query(None);
    dolores_core::validate_mcp_endpoint(endpoint.as_str())?;
    #[cfg(windows)]
    {
        use std::os::windows::ffi::OsStrExt;
        let operation: Vec<u16> = std::ffi::OsStr::new("open")
            .encode_wide()
            .chain(Some(0))
            .collect();
        let target: Vec<u16> = std::ffi::OsStr::new(url)
            .encode_wide()
            .chain(Some(0))
            .collect();
        // ShellExecute takes a document URL directly, never a shell command.
        let result = unsafe {
            windows_sys::Win32::UI::Shell::ShellExecuteW(
                std::ptr::null_mut(),
                operation.as_ptr(),
                target.as_ptr(),
                std::ptr::null(),
                std::ptr::null(),
                1,
            )
        };
        if result as isize > 32 {
            return Ok(());
        }
        Err("Could not open the default browser. Check Windows default apps, then try Sign in again.".into())
    }
    #[cfg(not(windows))]
    Err("Native MCP sign-in is currently supported on Windows. Use an explicitly configured bearer connection on this platform.".into())
}
