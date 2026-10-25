//! Explicit desktop setup. No shell, source checkout or privileged installation.
use dolores_tools_command::process;
use serde_json::{json, Value};
use std::{
    path::{Path, PathBuf},
    time::{Duration, Instant},
};
use tokio_util::sync::CancellationToken;

pub const VERSION: &str = "1.63.0";
const MANIFEST: &str = include_str!("../../../adapters/browser/package.json");
const LOCK: &str = include_str!("../../../adapters/browser/package-lock.json");

pub fn valid_adapter(path: &Path) -> bool {
    std::fs::read(path.join("node_modules/playwright-core/package.json"))
        .ok()
        .and_then(|v| serde_json::from_slice::<Value>(&v).ok())
        .is_some_and(|v| {
            v["version"] == VERSION && path.join("node_modules/playwright-core/index.js").is_file()
        })
}

fn find_node() -> Option<PathBuf> {
    let filename = if cfg!(windows) { "node.exe" } else { "node" };
    let mut paths = std::env::var_os("PATH")
        .map(|p| std::env::split_paths(&p).collect::<Vec<_>>())
        .unwrap_or_default();
    if cfg!(windows) {
        for key in ["ProgramFiles", "ProgramFiles(x86)"] {
            if let Some(p) = std::env::var_os(key) {
                paths.push(PathBuf::from(p).join("nodejs"));
            }
        }
    }
    paths
        .into_iter()
        .filter(|p| p.is_absolute())
        .map(|p| p.join(filename))
        .find(|p| p.is_file())
        .and_then(|p| p.canonicalize().ok())
}

pub fn browser_installed() -> bool {
    if cfg!(windows) {
        ["ProgramFiles", "ProgramFiles(x86)", "LOCALAPPDATA"]
            .iter()
            .filter_map(std::env::var_os)
            .any(|p| {
                PathBuf::from(p)
                    .join("Microsoft/Edge/Application/msedge.exe")
                    .is_file()
            })
    } else if cfg!(target_os = "macos") {
        Path::new("/Applications/Google Chrome.app/Contents/MacOS/Google Chrome").is_file()
    } else {
        ["/usr/bin/google-chrome", "/opt/google/chrome/chrome"]
            .iter()
            .any(|p| Path::new(p).is_file())
    }
}

pub fn node_tools() -> Result<(PathBuf, PathBuf), String> {
    let node = find_node().ok_or("Install Node.js, then check again.")?;
    let folder = node
        .parent()
        .ok_or("Node.js installation is unavailable.")?;
    let version = run(
        &node,
        &["--version".into()],
        folder,
        &CancellationToken::new(),
        Duration::from_secs(2),
    )?;
    if version
        .trim()
        .trim_start_matches('v')
        .split('.')
        .next()
        .and_then(|v| v.parse::<u32>().ok())
        .is_none_or(|v| v < 20)
    {
        return Err("Update Node.js to version 20 or newer, then check again.".into());
    }
    let candidates = [
        folder.join("node_modules/npm/bin/npm-cli.js"),
        folder.join("../lib/node_modules/npm/bin/npm-cli.js"),
        PathBuf::from("/usr/share/nodejs/npm/bin/npm-cli.js"),
    ];
    let npm = candidates
        .into_iter()
        .find(|p| p.is_file())
        .ok_or("Repair Node.js to include npm, then check again.")?;
    Ok((node, npm))
}

pub fn prerequisites() -> Value {
    let tools = node_tools();
    json!({"nodeReady":tools.is_ok(),"nodeReason":tools.err(),"browserReady":browser_installed()})
}

fn run(
    exe: &Path,
    args: &[String],
    folder: &Path,
    cancel: &CancellationToken,
    timeout: Duration,
) -> Result<String, String> {
    let mut env = process::environment();
    env.retain(|(k, _)| !k.to_uppercase().starts_with("NPM_CONFIG_") && k != "NODE_OPTIONS");
    let (mut child, input, mut out, mut err) = process::spawn_stdio(exe, args, folder, &env)?;
    drop(input);
    let end = Instant::now() + timeout;
    let mut kept = Vec::new();
    let mut buffer = [0u8; 4096];
    loop {
        if cancel.is_cancelled() {
            child.stop();
            return Err("Setup cancelled. Your previous installation remains.".into());
        }
        if Instant::now() >= end {
            child.stop();
            return Err("Setup timed out. Check your connection and try again.".into());
        }
        if let Ok(n) = process::read_available(&mut out, &mut buffer) {
            let count = n.min(128usize.saturating_sub(kept.len()));
            kept.extend_from_slice(&buffer[..count]);
        }
        let _ = process::read_available(&mut err, &mut buffer);
        if let Some(code) = child
            .try_exit()
            .map_err(|_| "Could not check setup progress.")?
        {
            // Drain the last short version response; installer output is never surfaced.
            if let Ok(n) = process::read_available(&mut out, &mut buffer) {
                let count = n.min(128usize.saturating_sub(kept.len()));
                kept.extend_from_slice(&buffer[..count]);
            }
            return if code == 0 {
                Ok(String::from_utf8_lossy(&kept).into_owned())
            } else {
                Err("Setup failed. Check your connection or repair Node.js, then try again. Your previous installation remains.".into())
            };
        }
        std::thread::sleep(Duration::from_millis(20));
    }
}

pub fn install(destination: &Path, cancel: &CancellationToken) -> Result<(), String> {
    let (node, npm) = node_tools()?;
    install_with(destination, cancel, |stage, config| {
        let args = vec![
            npm.to_string_lossy().into_owned(),
            "ci".into(),
            "--ignore-scripts".into(),
            "--no-audit".into(),
            "--no-fund".into(),
            "--registry=https://registry.npmjs.org".into(),
            format!("--userconfig={}", config.join("user.npmrc").display()),
            format!("--globalconfig={}", config.join("global.npmrc").display()),
            format!("--cache={}", config.join("cache").display()),
        ];
        run(&node, &args, stage, cancel, Duration::from_secs(90)).map(|_| ())
    })
}

fn install_with(
    destination: &Path,
    cancel: &CancellationToken,
    installer: impl FnOnce(&Path, &Path) -> Result<(), String>,
) -> Result<(), String> {
    if valid_adapter(destination) {
        return Ok(());
    }
    if destination.exists() {
        return Err("An incomplete adapter is retained in the setup folder. Move it aside using the folder shown in Details, then try again.".into());
    }
    let parent = destination
        .parent()
        .ok_or("Browser setup storage is unavailable.")?;
    std::fs::create_dir_all(parent).map_err(|_| "Could not create the browser setup folder.")?;
    let temp = tempfile::Builder::new()
        .prefix(".setup-")
        .tempdir_in(parent)
        .map_err(|_| "Could not stage browser setup.")?;
    let stage = temp.path().join("adapter");
    std::fs::create_dir(&stage).map_err(|_| "Could not stage browser setup.")?;
    for (name, body) in [("package.json", MANIFEST), ("package-lock.json", LOCK)] {
        std::fs::write(stage.join(name), body).map_err(|_| "Could not prepare browser setup.")?;
    }
    for name in ["user.npmrc", "global.npmrc"] {
        std::fs::write(temp.path().join(name), "")
            .map_err(|_| "Could not prepare browser setup.")?;
    }
    installer(&stage, temp.path())?;
    if cancel.is_cancelled() {
        return Err("Setup cancelled. Nothing was activated.".into());
    }
    if !valid_adapter(&stage) {
        return Err(
            "Browser adapter verification failed. Nothing was activated; try setup again.".into(),
        );
    }
    if destination.exists() {
        return Err("Setup folder changed. Your files were retained; check again.".into());
    }
    std::fs::rename(stage, destination)
        .map_err(|_| "Could not activate browser setup. Your previous files remain.")?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn staged_failure_cancel_and_verification_preserve_destination() {
        let temp = tempfile::tempdir().unwrap();
        let target = temp.path().join(VERSION);
        let cancel = CancellationToken::new();
        assert!(install_with(&target, &cancel, |_, _| Err(
            "fixture network failure".into()
        ))
        .is_err());
        assert!(!target.exists());
        assert!(install_with(&target, &cancel, |_, _| Ok(())).is_err());
        assert!(install_with(&target, &cancel, |stage, _| {
            let module = stage.join("node_modules/playwright-core");
            std::fs::create_dir_all(&module).unwrap();
            std::fs::write(
                module.join("package.json"),
                format!("{{\"version\":\"{VERSION}\"}}"),
            )
            .unwrap();
            std::fs::write(module.join("index.js"), "").unwrap();
            Ok(())
        })
        .is_ok());
        assert!(install_with(&target, &cancel, |_, _| panic!(
            "working adapter should not reinstall"
        ))
        .is_ok());
        let other = temp.path().join("other");
        let stopped = CancellationToken::new();
        stopped.cancel();
        assert!(install_with(&other, &stopped, |_, _| Ok(())).is_err());
        assert!(valid_adapter(&target));
        assert!(!other.exists());
        let unrelated = temp.path().join("unrelated");
        std::fs::create_dir(&unrelated).unwrap();
        std::fs::write(unrelated.join("keep"), "keep").unwrap();
        assert!(install_with(&unrelated, &cancel, |_, _| panic!()).is_err());
        assert!(unrelated.join("keep").is_file());
    }
}
