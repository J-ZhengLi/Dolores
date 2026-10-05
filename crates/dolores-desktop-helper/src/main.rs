use std::io::{self, BufRead, Read};

#[cfg(windows)]
mod capture;
#[cfg(windows)]
#[path = "../../dolores-core/src/desktop_input.rs"]
mod desktop_input;
#[cfg(windows)]
mod input;

fn main() {
    let result = (|| -> Result<serde_json::Value, String> {
        #[cfg(windows)]
        unsafe {
            windows_sys::Win32::UI::HiDpi::SetProcessDpiAwarenessContext(
                windows_sys::Win32::UI::HiDpi::DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2,
            );
        }
        if std::env::args().nth(1).as_deref() == Some("--input") {
            let stdin = io::stdin();
            let mut reader = stdin.lock();
            let mut line = String::new();
            reader
                .by_ref()
                .take(16385)
                .read_line(&mut line)
                .map_err(|_| "Input request unavailable.")?;
            if line.len() > 16384 {
                return Err("Input request exceeds its limit.".into());
            }
            let request: serde_json::Value =
                serde_json::from_str(&line).map_err(|_| "Invalid input request.")?;
            #[cfg(windows)]
            {
                return input::run(request, &mut reader);
            }
            #[cfg(not(windows))]
            {
                let _ = request;
                return Err("Desktop input is only qualified on Windows.".into());
            }
        }
        let mut bytes = Vec::new();
        io::stdin()
            .take(16385)
            .read_to_end(&mut bytes)
            .map_err(|_| "Observation request unavailable.")?;
        if bytes.len() > 16384 {
            return Err("Observation request exceeds its limit.".into());
        }
        let request = serde_json::from_slice(&bytes).map_err(|_| "Invalid observation request.")?;
        #[cfg(windows)]
        {
            capture::run(request)
        }
        #[cfg(not(windows))]
        {
            let _: serde_json::Value = request;
            Err("Desktop observation is only qualified on Windows.".into())
        }
    })();
    let value = match result {
        Ok(value) => serde_json::json!({"ok":true,"result":value}),
        Err(error) => serde_json::json!({"ok":false,"error":error}),
    };
    println!("{value}");
}
