use base64::{engine::general_purpose::STANDARD, Engine};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::{
    error::Error,
    sync::{Arc, Mutex},
};
use windows_capture::{
    capture::{Context, GraphicsCaptureApiHandler},
    frame::Frame,
    graphics_capture_api::InternalCaptureControl,
    settings::{
        ColorFormat, CursorCaptureSettings, DirtyRegionSettings, DrawBorderSettings,
        MinimumUpdateIntervalSettings, SecondaryWindowSettings, Settings,
    },
    window::Window,
};
use windows_sys::Win32::{
    Foundation::{CloseHandle, FILETIME},
    System::Threading::{GetProcessTimes, OpenProcess, PROCESS_QUERY_LIMITED_INFORMATION},
    UI::{
        HiDpi::GetDpiForWindow,
        WindowsAndMessaging::{GetWindowThreadProcessId, IsIconic, IsWindow, IsWindowVisible},
    },
};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Target {
    pub handle: u64,
    pub pid: u32,
    pub created: u64,
    pub title: String,
}

pub(crate) fn identity(handle: u64) -> Result<Target, String> {
    let hwnd = handle as usize as *mut std::ffi::c_void;
    unsafe {
        if IsWindow(hwnd) == 0 || IsWindowVisible(hwnd) == 0 || IsIconic(hwnd) != 0 {
            return Err(
                "Window unavailable or minimized. Restore it and refresh the window list.".into(),
            );
        }
        let mut pid = 0;
        GetWindowThreadProcessId(hwnd, &mut pid);
        let process = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, 0, pid);
        if process.is_null() {
            return Err("Window identity unavailable. Choose a non-elevated application.".into());
        }
        let mut created = FILETIME {
            dwLowDateTime: 0,
            dwHighDateTime: 0,
        };
        let mut exit = created;
        let mut kernel = created;
        let mut user = created;
        let valid = GetProcessTimes(process, &mut created, &mut exit, &mut kernel, &mut user);
        CloseHandle(process);
        if valid == 0 {
            return Err("Window identity unavailable. Refresh the window list.".into());
        }
        let mut title = Window::from_raw_hwnd(hwnd)
            .title()
            .map_err(|_| "Window title unavailable.")?;
        if title.len() > 1024 {
            let mut end = 1024;
            while !title.is_char_boundary(end) {
                end -= 1;
            }
            title.truncate(end);
        }
        Ok(Target {
            handle,
            pid,
            created: (u64::from(created.dwHighDateTime) << 32) | u64::from(created.dwLowDateTime),
            title,
        })
    }
}
pub(crate) fn recheck(target: &Target) -> Result<(), String> {
    if identity(target.handle)? != *target {
        return Err("Selected window changed. Refresh and capture it again.".into());
    }
    Ok(())
}

pub(crate) fn geometry(target: &Target) -> Result<Value, String> {
    use windows_sys::Win32::{
        Foundation::RECT,
        Graphics::Dwm::{DwmGetWindowAttribute, DWMWA_EXTENDED_FRAME_BOUNDS},
        UI::WindowsAndMessaging::{GetGUIThreadInfo, GUITHREADINFO},
    };
    let hwnd = target.handle as usize as *mut _;
    let mut rect: RECT = unsafe { std::mem::zeroed() };
    let mut info: GUITHREADINFO = unsafe { std::mem::zeroed() };
    info.cbSize = std::mem::size_of::<GUITHREADINFO>() as u32;
    unsafe {
        if DwmGetWindowAttribute(
            hwnd,
            DWMWA_EXTENDED_FRAME_BOUNDS as u32,
            &mut rect as *mut _ as *mut _,
            std::mem::size_of::<RECT>() as u32,
        ) != 0
        {
            return Err("Window geometry unavailable. Restore it and capture again.".into());
        }
        let thread = GetWindowThreadProcessId(hwnd, std::ptr::null_mut());
        if GetGUIThreadInfo(thread, &mut info) == 0 {
            return Err("Window focus unavailable. Focus its input and capture again.".into());
        }
        Ok(
            json!({"left":rect.left,"top":rect.top,"right":rect.right,"bottom":rect.bottom,"dpi":GetDpiForWindow(hwnd),"focus":info.hwndFocus as usize as u64}),
        )
    }
}

type Shared = Arc<Mutex<Option<Result<Value, String>>>>;
struct Capture {
    target: Target,
    result: Shared,
}
impl GraphicsCaptureApiHandler for Capture {
    type Flags = (Target, Shared);
    type Error = Box<dyn Error + Send + Sync>;
    fn new(ctx: Context<Self::Flags>) -> Result<Self, Self::Error> {
        Ok(Self {
            target: ctx.flags.0,
            result: ctx.flags.1,
        })
    }
    fn on_frame_arrived(
        &mut self,
        frame: &mut Frame,
        control: InternalCaptureControl,
    ) -> Result<(), Self::Error> {
        let result = (|| -> Result<Value, String> {
            recheck(&self.target)?;
            let geometry_before = geometry(&self.target)?;
            let (width, height) = (frame.width(), frame.height());
            if width == 0
                || height == 0
                || width > 4096
                || height > 4096
                || u64::from(width) * u64::from(height) > 4_000_000
            {
                return Err(
                    "Window exceeds the four-megapixel capture limit. Resize it and capture again."
                        .into(),
                );
            }
            let buffer = frame
                .buffer()
                .map_err(|_| "Window pixels unavailable. Restore it and capture again.")?;
            let mut padding = Vec::new();
            let rgba = image::RgbaImage::from_raw(
                width,
                height,
                buffer.as_nopadding_buffer(&mut padding).to_vec(),
            )
            .ok_or("Invalid capture surface.")?;
            let resized = image::DynamicImage::ImageRgba8(rgba)
                .resize(1024, 1024, image::imageops::FilterType::Triangle)
                .to_rgb8();
            let mut jpeg = Vec::new();
            image::codecs::jpeg::JpegEncoder::new_with_quality(&mut jpeg, 80)
                .encode_image(&resized)
                .map_err(|_| "Capture encoding unavailable.")?;
            if jpeg.len() > 512 * 1024 {
                return Err(
                    "Screenshot exceeds 512 KiB. Use a smaller window and capture again.".into(),
                );
            }
            recheck(&self.target)?;
            let geometry_after = geometry(&self.target)?;
            if geometry_before != geometry_after {
                return Err("Window moved or focus changed during capture. Capture again.".into());
            }
            let dpi = unsafe { GetDpiForWindow(self.target.handle as usize as *mut _) };
            let mut last_input: windows_sys::Win32::UI::Input::KeyboardAndMouse::LASTINPUTINFO =
                unsafe { std::mem::zeroed() };
            last_input.cbSize = std::mem::size_of_val(&last_input) as u32;
            unsafe {
                windows_sys::Win32::UI::Input::KeyboardAndMouse::GetLastInputInfo(&mut last_input);
            }
            Ok(
                json!({"target":self.target,"originalWidth":width,"originalHeight":height,
                "width":resized.width(),"height":resized.height(),"dpi":dpi,"mime":"image/jpeg",
                "geometry":geometry_after,"lastInput":last_input.dwTime,
                "imageBase64":STANDARD.encode(jpeg),"coordinateSpace":"image pixels; not desktop coordinates",
                "accessibility":"Unavailable in this adapter; screenshot evidence only."}),
            )
        })();
        *self
            .result
            .lock()
            .map_err(|_| "Capture state unavailable.")? = Some(result);
        control.stop();
        Ok(())
    }
    fn on_closed(&mut self) -> Result<(), Self::Error> {
        let mut result = self
            .result
            .lock()
            .map_err(|_| "Capture state unavailable.")?;
        if result.is_none() {
            *result = Some(Err(
                "Window closed. Refresh the list and capture again.".into()
            ));
        }
        Ok(())
    }
}
pub fn run(request: Value) -> Result<Value, String> {
    match request["operation"].as_str() {
        Some("list") => {
            let windows = Window::enumerate().map_err(|_| "Window list unavailable.")?;
            // Use the public raw handle conversion; never guess a window from its title.
            let targets: Vec<_> = windows
                .into_iter()
                .filter_map(|w| identity(w.as_raw_hwnd() as usize as u64).ok())
                .filter(|w| !w.title.is_empty())
                .take(64)
                .collect();
            Ok(json!({"windows":targets,"limit":64}))
        }
        Some("capture") => {
            let target: Target = serde_json::from_value(request["target"].clone())
                .map_err(|_| "Select a listed window first.")?;
            recheck(&target)?;
            let result: Shared = Arc::new(Mutex::new(None));
            let settings = Settings::new(
                Window::from_raw_hwnd(target.handle as usize as *mut _),
                CursorCaptureSettings::WithoutCursor,
                DrawBorderSettings::WithBorder,
                SecondaryWindowSettings::Exclude,
                MinimumUpdateIntervalSettings::Default,
                DirtyRegionSettings::Default,
                ColorFormat::Rgba8,
                (target, result.clone()),
            );
            Capture::start(settings).map_err(|_| {
                "Windows capture unavailable. Check Windows capture support and restore the target."
            })?;
            let value = result
                .lock()
                .map_err(|_| "Capture state unavailable.")?
                .take();
            value.unwrap_or_else(|| {
                Err("No frame arrived. Restore the window and capture again.".into())
            })
        }
        _ => Err("Unknown observation operation.".into()),
    }
}
