use super::*;
use base64::Engine as _;
#[derive(Default)]
pub(super) struct Setup {
    phase: &'static str,
    note: String,
    pub(super) cancel: CancellationToken,
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) enum SetupAction {
    Start,
    Status,
    Cancel,
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) enum Prerequisite {
    Node,
    Browser,
}
pub(super) fn open_prerequisite(kind: Prerequisite) -> Result<Value, String> {
    mcp::open_browser(match kind {
        Prerequisite::Node => "https://nodejs.org/en/download",
        Prerequisite::Browser => "https://www.microsoft.com/edge/download",
    })?;
    Ok(Value::Null)
}
impl Engine {
    fn browser_adapter(&self) -> Result<PathBuf, String> {
        Ok(self
            .browser_captures()?
            .parent()
            .ok_or("Browser storage unavailable.")?
            .join("browser-adapters")
            .join(dolores_tools_browser::setup::VERSION))
    }
    pub(super) fn browser_setup_action(&self, action: SetupAction) -> Result<Value, String> {
        let mut state = self
            .browser_setup
            .lock()
            .map_err(|_| "Browser setup is unavailable.")?;
        match action {
            SetupAction::Start if state.phase != "installing" => {
                dolores_tools_browser::setup::node_tools()?;
                if !dolores_tools_browser::setup::browser_installed() {
                    return Err("Install Microsoft Edge first, then check again.".into());
                }
                let destination = self.browser_adapter()?;
                state.phase = "installing";
                state.note = "Setting up browser…".into();
                state.cancel = CancellationToken::new();
                let token = state.cancel.clone();
                let shared = self.browser_setup.clone();
                std::thread::Builder::new()
                    .name("browser-setup".into())
                    .spawn(move || {
                        let result = dolores_tools_browser::setup::install(&destination, &token);
                        if let Ok(mut state) = shared.lock() {
                            match result {
                                Ok(()) => {
                                    state.phase = "ready";
                                    state.note = "Browser is ready.".into();
                                }
                                Err(note) => {
                                    state.phase = "stopped";
                                    state.note = note;
                                }
                            }
                        }
                    })
                    .map_err(|_| {
                        state.phase = "stopped";
                        "Could not start browser setup. Try again."
                    })?;
            }
            SetupAction::Cancel => state.cancel.cancel(),
            _ => {}
        }
        Ok(json!({"phase":state.phase,"note":state.note}))
    }

    fn browser_captures(&self) -> Result<PathBuf, String> {
        Ok(self
            .workspace_directory
            .as_ref()
            .and_then(|p| p.parent())
            .ok_or("Browser storage unavailable.")?
            .join("browser-captures"))
    }
    pub(super) fn browser_runtime(&self) -> Result<dolores_tools_browser::BrowserRuntime, String> {
        dolores_tools_browser::BrowserRuntime::discover_with(
            self.browser_captures()?,
            Some(self.browser_adapter()?),
        )
    }
    pub(super) fn browser_settings(&self) -> Result<Value, String> {
        let runtime = self.browser_runtime();
        let prerequisites = dolores_tools_browser::setup::prerequisites();
        Ok(
            json!({"prerequisites":prerequisites,"setupState":self.browser_setup_action(SetupAction::Status)?,"setupFolder":self.browser_adapter()?.to_string_lossy(),"available":runtime.is_ok(),"reason":runtime.err(),"captureFolder":self.browser_captures().ok().map(|p|p.to_string_lossy().into_owned()),"adapter":"Playwright 1.63.0 · Node · installed Edge (Windows) / Chrome (Linux/macOS)","profile":"Fresh visible browser per parent run; no existing profile or saved login", "bounds":"One page · 20 seconds per operation · 60 controls · 8 KiB text · 512 KiB viewport screenshot · 128 saved captures", "setup":"Install Node and Edge/Chrome, then follow docs/USER_GUIDE.md → Browser setup. No browser download or installer runs automatically."}),
        )
    }
    pub(super) fn browser_capture(&self, capture: &str) -> Result<Value, String> {
        let id = uuid::Uuid::parse_str(capture).map_err(|_| "Invalid browser capture.")?;
        if id.to_string() != capture {
            return Err("Invalid browser capture.".into());
        }
        let root = self
            .browser_captures()?
            .canonicalize()
            .map_err(|_| "Browser capture is unavailable.")?;
        let path = root.join(format!("{id}.jpg"));
        let metadata =
            std::fs::symlink_metadata(&path).map_err(|_| "Browser capture is unavailable.")?;
        if !metadata.is_file()
            || metadata.file_type().is_symlink()
            || metadata.len() > 512 * 1024
            || path.canonicalize().ok().as_ref() != Some(&path)
        {
            return Err("Browser capture changed or exceeds its limit.".into());
        }
        use std::io::Read;
        let mut bytes = Vec::new();
        std::fs::File::open(&path)
            .map_err(|_| "Browser capture is unavailable.")?
            .take(512 * 1024 + 1)
            .read_to_end(&mut bytes)
            .map_err(|_| "Browser capture could not be read.")?;
        if bytes.len() > 512 * 1024 || !bytes.starts_with(&[0xff, 0xd8, 0xff]) {
            return Err("Browser capture is not a supported JPEG.".into());
        }
        Ok(json!({"data":base64::engine::general_purpose::STANDARD.encode(bytes)}))
    }
}
