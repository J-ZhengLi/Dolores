use super::*;
use base64::Engine as _;
impl Engine {
    fn browser_captures(&self) -> Result<PathBuf, String> {
        Ok(self
            .workspace_directory
            .as_ref()
            .and_then(|p| p.parent())
            .ok_or("Browser storage unavailable.")?
            .join("browser-captures"))
    }
    pub(super) fn browser_runtime(&self) -> Result<dolores_tools_browser::BrowserRuntime, String> {
        dolores_tools_browser::BrowserRuntime::discover(self.browser_captures()?)
    }
    pub(super) fn browser_settings(&self) -> Result<Value, String> {
        let runtime = self.browser_runtime();
        Ok(
            json!({"available":runtime.is_ok(),"reason":runtime.err(),"captureFolder":self.browser_captures().ok().map(|p|p.to_string_lossy().into_owned()),"adapter":"Playwright 1.63.0 · Node · installed Edge (Windows) / Chrome (Linux/macOS)","profile":"Fresh visible browser per parent run; no existing profile or saved login", "bounds":"One page · 20 seconds per operation · 60 controls · 8 KiB text · 512 KiB viewport screenshot · 128 saved captures", "setup":"Install Node and Edge/Chrome, then follow docs/USER_GUIDE.md → Browser setup. No browser download or installer runs automatically."}),
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
