//! No scripts, shell, clipboard or global shortcuts. One checked action per process.
use crate::{
    capture::{self, Target},
    desktop_input::Input,
};
use serde_json::{json, Value};
use std::io::{BufRead, Write};
use windows_sys::Win32::{
    Foundation::{POINT, RECT},
    Graphics::Gdi::ClientToScreen,
    UI::{Input::KeyboardAndMouse::*, WindowsAndMessaging::*},
};

fn check(request: &Value) -> Result<(Target, Input), String> {
    let target: Target =
        serde_json::from_value(request["target"].clone()).map_err(|_| "Invalid input target.")?;
    capture::recheck(&target)?;
    if capture::geometry(&target)? != request["observation"]["geometry"] {
        return Err(
            "Window moved, resized, or focus changed. No input dispatched; capture again.".into(),
        );
    }
    let action: Input =
        serde_json::from_value(request["action"].clone()).map_err(|_| "Invalid input action.")?;
    let image = &request["observation"];
    action.validate(
        image["width"].as_u64().unwrap_or(0) as u32,
        image["height"].as_u64().unwrap_or(0) as u32,
    )?;
    if request["automatic"] == true && !action.ordinary() {
        return Err("This operation requires fresh review. No input dispatched.".into());
    }
    unsafe {
        let foreground = GetForegroundWindow();
        let mut pid = 0;
        GetWindowThreadProcessId(foreground, &mut pid);
        if foreground.is_null()
            || (foreground as usize as u64 != target.handle
                && Some(u64::from(pid)) != request["hostPid"].as_u64())
        {
            return Err("Another application took focus. No input dispatched; select the target and capture again.".into());
        }
        if request["automatic"] == true {
            let mut last: LASTINPUTINFO = std::mem::zeroed();
            last.cbSize = std::mem::size_of_val(&last) as u32;
            if GetLastInputInfo(&mut last) == 0
                || Some(u64::from(last.dwTime)) != image["lastInput"].as_u64()
            {
                return Err("User input changed since observation. No input dispatched; inspect and capture again.".into());
            }
        }
        let desktop = windows_sys::Win32::System::StationsAndDesktops::OpenInputDesktop(
            0,
            0,
            windows_sys::Win32::System::StationsAndDesktops::DESKTOP_READOBJECTS,
        );
        if desktop.is_null() {
            return Err(
                "Desktop is unavailable or locked. No input dispatched; unlock and capture again."
                    .into(),
            );
        }
        let mut name = [0u16; 128];
        let mut needed = 0;
        let named = windows_sys::Win32::System::StationsAndDesktops::GetUserObjectInformationW(
            desktop,
            windows_sys::Win32::System::StationsAndDesktops::UOI_NAME,
            name.as_mut_ptr() as *mut _,
            std::mem::size_of_val(&name) as u32,
            &mut needed,
        );
        windows_sys::Win32::System::StationsAndDesktops::CloseDesktop(desktop);
        let end = name.iter().position(|c| *c == 0).unwrap_or(name.len());
        if named == 0 || String::from_utf16_lossy(&name[..end]) != "Default" {
            return Err("Secure or locked desktop is unavailable. No input dispatched; return to your normal desktop and capture again.".into());
        }
    }
    Ok((target, action))
}
fn keyboard(vk: u16, scan: u16, flags: u32) -> INPUT {
    INPUT {
        r#type: INPUT_KEYBOARD,
        Anonymous: INPUT_0 {
            ki: KEYBDINPUT {
                wVk: vk,
                wScan: scan,
                dwFlags: flags,
                time: 0,
                dwExtraInfo: 0,
            },
        },
    }
}
fn mouse(dx: i32, dy: i32, data: u32, flags: u32) -> INPUT {
    INPUT {
        r#type: INPUT_MOUSE,
        Anonymous: INPUT_0 {
            mi: MOUSEINPUT {
                dx,
                dy,
                mouseData: data,
                dwFlags: flags,
                time: 0,
                dwExtraInfo: 0,
            },
        },
    }
}
fn point(request: &Value, target: &Target, x: u32, y: u32) -> Result<(i32, i32), String> {
    let o = &request["observation"];
    let g = &o["geometry"];
    let left = g["left"].as_i64().ok_or("Invalid geometry.")? as i32;
    let top = g["top"].as_i64().ok_or("Invalid geometry.")? as i32;
    let width = g["right"].as_i64().ok_or("Invalid geometry.")? as i32 - left;
    let height = g["bottom"].as_i64().ok_or("Invalid geometry.")? as i32 - top;
    if width <= 0
        || height <= 0
        || (width - i32::try_from(o["originalWidth"].as_u64().unwrap_or(0)).unwrap_or(0)).abs() > 2
        || (height - i32::try_from(o["originalHeight"].as_u64().unwrap_or(0)).unwrap_or(0)).abs()
            > 2
    {
        return Err("Capture coordinates do not match the physical window. No input dispatched; capture again.".into());
    }
    let px = left
        + ((u64::from(x) * width as u64) / o["width"].as_u64().ok_or("Invalid image size.")?)
            as i32;
    let py = top
        + ((u64::from(y) * height as u64) / o["height"].as_u64().ok_or("Invalid image size.")?)
            as i32;
    unsafe {
        let hwnd = target.handle as usize as *mut _;
        let mut client: RECT = std::mem::zeroed();
        let mut origin = POINT { x: 0, y: 0 };
        if GetClientRect(hwnd, &mut client) == 0
            || ClientToScreen(hwnd, &mut origin) == 0
            || px < origin.x
            || py < origin.y
            || px >= origin.x + client.right
            || py >= origin.y + client.bottom
        {
            return Err(
                "Coordinate is outside the selected window's client area. No input dispatched."
                    .into(),
            );
        }
        if GetAncestor(WindowFromPoint(POINT { x: px, y: py }), GA_ROOT) != hwnd {
            return Err("Another window covers that coordinate. No input dispatched; uncover and capture again.".into());
        }
        let vx = GetSystemMetrics(SM_XVIRTUALSCREEN);
        let vy = GetSystemMetrics(SM_YVIRTUALSCREEN);
        let vw = GetSystemMetrics(SM_CXVIRTUALSCREEN);
        let vh = GetSystemMetrics(SM_CYVIRTUALSCREEN);
        if vw <= 1 || vh <= 1 {
            return Err("Desktop geometry unavailable.".into());
        }
        Ok((
            ((i64::from(px - vx) * 65535) / i64::from(vw - 1)) as i32,
            ((i64::from(py - vy) * 65535) / i64::from(vh - 1)) as i32,
        ))
    }
}
pub fn run<R: BufRead>(request: Value, reader: &mut R) -> Result<Value, String> {
    check(&request)?;
    println!("{}", json!({"ready":true}));
    std::io::stdout()
        .flush()
        .map_err(|_| "Input handshake unavailable.")?;
    let mut ack = String::new();
    std::io::Read::take(&mut *reader, 128)
        .read_line(&mut ack)
        .map_err(|_| "Input handshake unavailable.")?;
    if serde_json::from_str::<Value>(&ack)
        .ok()
        .is_none_or(|v| v != json!({"dispatch":true}))
    {
        return Err("Input was not committed. No input dispatched.".into());
    }
    let (target, action) = check(&request)?;
    unsafe {
        let hwnd = target.handle as usize as *mut _;
        if GetForegroundWindow() != hwnd && SetForegroundWindow(hwnd) == 0 {
            return Err("Selected window could not take focus. No input dispatched.".into());
        }
        if GetForegroundWindow() != hwnd {
            return Err("Target focus unavailable. No input dispatched.".into());
        }
    }
    let mut events = Vec::new();
    let move_flag = MOUSEEVENTF_MOVE | MOUSEEVENTF_ABSOLUTE | MOUSEEVENTF_VIRTUALDESK;
    match action {
        Input::Type { text } => {
            for c in text.encode_utf16() {
                events.push(keyboard(0, c, KEYEVENTF_UNICODE));
                events.push(keyboard(0, c, KEYEVENTF_UNICODE | KEYEVENTF_KEYUP));
            }
        }
        Input::Key { key } => {
            let (modifier, vk) = match key.as_str() {
                "Shift+Tab" => (Some(VK_SHIFT), VK_TAB),
                "Ctrl+A" => (Some(VK_CONTROL), 0x41),
                "Tab" => (None, VK_TAB),
                "Left" => (None, VK_LEFT),
                "Right" => (None, VK_RIGHT),
                "Up" => (None, VK_UP),
                "Down" => (None, VK_DOWN),
                "Home" => (None, VK_HOME),
                "End" => (None, VK_END),
                "Enter" => (None, VK_RETURN),
                "Escape" => (None, VK_ESCAPE),
                "Delete" => (None, VK_DELETE),
                "Backspace" => (None, VK_BACK),
                _ => return Err("Unsupported key.".into()),
            };
            if let Some(m) = modifier {
                events.push(keyboard(m, 0, 0));
            }
            events.push(keyboard(vk, 0, 0));
            events.push(keyboard(vk, 0, KEYEVENTF_KEYUP));
            if let Some(m) = modifier {
                events.push(keyboard(m, 0, KEYEVENTF_KEYUP));
            }
        }
        Input::Click { x, y } | Input::DoubleClick { x, y } => {
            let (dx, dy) = point(&request, &target, x, y)?;
            events.push(mouse(dx, dy, 0, move_flag));
            let count = if matches!(action, Input::DoubleClick { .. }) {
                2
            } else {
                1
            };
            for _ in 0..count {
                events.push(mouse(0, 0, 0, MOUSEEVENTF_LEFTDOWN));
                events.push(mouse(0, 0, 0, MOUSEEVENTF_LEFTUP));
            }
        }
        Input::Scroll { x, y, delta } => {
            let (dx, dy) = point(&request, &target, x, y)?;
            events.push(mouse(dx, dy, 0, move_flag));
            events.push(mouse(0, 0, delta as u32, MOUSEEVENTF_WHEEL));
        }
        Input::Drag { x, y, end_x, end_y } => {
            let (sx, sy) = point(&request, &target, x, y)?;
            let (ex, ey) = point(&request, &target, end_x, end_y)?;
            // Refuse a drag crossing any window occlusion before dispatch.
            for i in 1..=8 {
                point(
                    &request,
                    &target,
                    (i64::from(x) + (i64::from(end_x) - i64::from(x)) * i / 8) as u32,
                    (i64::from(y) + (i64::from(end_y) - i64::from(y)) * i / 8) as u32,
                )?;
            }
            events.push(mouse(sx, sy, 0, move_flag));
            events.push(mouse(0, 0, 0, MOUSEEVENTF_LEFTDOWN));
            for i in 1..=8 {
                events.push(mouse(
                    sx + (ex - sx) * i / 8,
                    sy + (ey - sy) * i / 8,
                    0,
                    move_flag,
                ));
            }
            events.push(mouse(0, 0, 0, MOUSEEVENTF_LEFTUP));
        }
    }
    unsafe {
        for vk in [
            VK_SHIFT, VK_CONTROL, VK_MENU, VK_LWIN, VK_RWIN, VK_LBUTTON, VK_RBUTTON,
        ] {
            if GetAsyncKeyState(vk as i32) < 0 {
                return Err("A user key or mouse button is held. No input dispatched; release it and capture again.".into());
            }
        }
        capture::recheck(&target)?;
        if GetForegroundWindow() as usize as u64 != target.handle
            || capture::geometry(&target)? != request["observation"]["geometry"]
        {
            return Err("Focus or geometry changed before dispatch. No input dispatched.".into());
        }
        let sent = SendInput(
            events.len() as u32,
            events.as_ptr(),
            std::mem::size_of::<INPUT>() as i32,
        );
        if sent != events.len() as u32 {
            // Release only keys/buttons this batch could own. Never replay the action.
            let release = [
                keyboard(VK_SHIFT, 0, KEYEVENTF_KEYUP),
                keyboard(VK_CONTROL, 0, KEYEVENTF_KEYUP),
                mouse(0, 0, 0, MOUSEEVENTF_LEFTUP),
            ];
            SendInput(
                release.len() as u32,
                release.as_ptr(),
                std::mem::size_of::<INPUT>() as i32,
            );
            return Err("Input may have partially occurred. Inspect the window before continuing; nothing will be replayed.".into());
        }
        Ok(
            json!({"dispatched":true,"events":sent,"verification":"Input was inserted, not verified as an application effect. Observe again before any further input."}),
        )
    }
}
