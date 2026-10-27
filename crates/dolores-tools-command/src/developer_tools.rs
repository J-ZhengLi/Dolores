//! Discover optional installed CLIs without executing installers or project code.
use super::*;

pub(super) fn normalize(root: &Path, invocation: CommandSpec) -> Result<CommandSpec, String> {
    #[cfg(windows)]
    {
        let file = resolve(root, &invocation.program)?;
        if file
            .extension()
            .is_some_and(|e| e.eq_ignore_ascii_case("cmd"))
        {
            let mut launch = node_launcher(root, &file, &std::env::var_os("PATH").unwrap_or_default())
                .ok_or("This wrapper is not a supported Node CLI launcher. Invoke its shell or interpreter explicitly; no command was run.")?;
            launch.args.extend(invocation.args);
            if !valid(&launch) {
                return Err("Expanded Node CLI arguments exceed the command allowance. Use fewer arguments or a script file.".into());
            }
            return Ok(launch);
        }
    }
    #[cfg(not(windows))]
    let _ = root;
    Ok(invocation)
}

#[cfg(windows)]
fn relative(shim: &Path, value: &str) -> Option<PathBuf> {
    let value = value
        .strip_prefix("%~dp0")
        .or_else(|| value.strip_prefix("%dp0%"))?;
    if value.contains(['%', '\0', '"']) {
        return None;
    }
    let path = shim.parent()?.join(value.trim_start_matches(['\\', '/']));
    path.canonicalize().ok().filter(|p| p.is_file())
}

#[cfg(windows)]
fn node_launcher(root: &Path, shim: &Path, search: &std::ffi::OsStr) -> Option<CommandSpec> {
    use std::io::Read;
    let mut bytes = vec![];
    File::open(shim)
        .ok()?
        .take(16 * 1024 + 1)
        .read_to_end(&mut bytes)
        .ok()?;
    if bytes.len() > 16 * 1024 {
        return None;
    }
    let source = String::from_utf8(bytes).ok()?.replace("\r\n", "\n");
    let source = source.trim();
    let (runtime, script, flags) = if let Some(line) =
        source.strip_prefix("@\"").filter(|s| !s.contains('\n'))
    {
        let (runtime, rest) = line.split_once('"')?;
        let (flags, rest) = rest.trim().split_once('"')?;
        let (script, rest) = rest.split_once('"')?;
        if rest.trim() != "%*" {
            return None;
        }
        let flags: Vec<_> = flags.split_whitespace().collect();
        if !flags.iter().all(|s| {
            s.starts_with("--")
                && s.chars()
                    .all(|c| c.is_ascii_alphanumeric() || "-=_".contains(c))
        }) {
            return None;
        }
        (
            relative(shim, runtime)?,
            relative(shim, script)?,
            flags.into_iter().map(str::to_owned).collect::<Vec<_>>(),
        )
    } else {
        // Stock npm Windows shim. Interpret only this known forwarding template;
        // other batch files retain the explicit-shell contract.
        let prefix = "@ECHO off\nGOTO start\n:find_dp0\nSET dp0=%~dp0\nEXIT /b\n:start\nSETLOCAL\nCALL :find_dp0\n\nIF EXIST \"%dp0%\\node.exe\" (\n  SET \"_prog=%dp0%\\node.exe\"\n) ELSE (\n  SET \"_prog=node\"\n  SET PATHEXT=%PATHEXT:;.JS;=;%\n)\n\nendLocal & goto #_undefined_# 2>NUL || title %COMSPEC% & \"%_prog%\"  \"";
        let script = source.strip_prefix(prefix)?.strip_suffix("\" %*")?;
        let script = relative(shim, script)?;
        let runtime = shim.parent()?.join("node.exe");
        let runtime = if runtime.is_file() {
            runtime.canonicalize().ok()?
        } else {
            resolve_from_path(root, "node", search).ok()?
        };
        (runtime, script, vec![])
    };
    if !runtime
        .file_stem()
        .is_some_and(|s| s.eq_ignore_ascii_case("node"))
        || !script.extension().is_some_and(|s| {
            ["js", "cjs", "mjs"]
                .iter()
                .any(|e| s.eq_ignore_ascii_case(e))
        })
    {
        return None;
    }
    let mut args = flags;
    // Node's main-module resolver misreads a verbatim drive as "C:". Keep
    // filesystem identity canonical, but pass its ordinary Win32 spelling.
    args.push(
        super::windows::child_path(&script)
            .to_string_lossy()
            .into_owned(),
    );
    Some(CommandSpec {
        program: runtime.to_string_lossy().into_owned(),
        args,
    })
}

pub(super) fn guidance(root: &Path) -> String {
    guidance_on_path(root, &std::env::var_os("PATH").unwrap_or_default())
}
fn guidance_on_path(root: &Path, search: &std::ffi::OsStr) -> String {
    let mut hints = Vec::new();
    if let Ok(rtk) = resolve_from_path(root, "rtk", search) {
        hints.push(format!("RTK is installed at {}. Prefer run_command with this program and literal args such as [\"git\",\"status\"] or [\"cargo\",\"test\"] for compact human-readable output. Consult its --help for supported commands. Use [\"proxy\",<program>,<args>...] when exact/unfiltered output is needed or the command has no RTK filter. Do not compress JSON/protocol evidence that must be parsed exactly; honor explicit raw-output requests. Never retry a command with effects just because its wrapper reported an error.", json!(rtk)));
    }
    if root.join(".codegraph").is_dir() {
        if let Ok(codegraph) = resolve_from_path(root, "codegraph", search) {
            #[cfg(windows)]
            let usable = !codegraph
                .extension()
                .is_some_and(|e| e.eq_ignore_ascii_case("cmd"))
                || node_launcher(root, &codegraph, search).is_some();
            #[cfg(not(windows))]
            let usable = true;
            if usable {
                hints.push(format!("CodeGraph is installed at {} and this project already has a .codegraph index. For code structure, symbols, callers and relevant source, use an advertised CodeGraph MCP tool when available, otherwise run_command with this program and args [\"explore\",<question or symbols>], before broad file reads/searches. It runs in the selected project. Supported Node launchers are resolved automatically. Follow stale-index warnings and fall back to normal file tools if unavailable. Do not install, initialize, re-index, upgrade or start a resident service without a user request.", json!(codegraph)));
            }
        }
    }
    if hints.is_empty() {
        String::new()
    } else {
        format!(
            "\n\nInstalled developer tools (detected locally; normal task access applies):\n{}",
            hints.join("\n")
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[cfg(windows)]
    #[test]
    fn stock_npm_launcher_and_unsupported_wrappers_keep_literal_arguments() {
        let root = tempfile::tempdir().unwrap();
        std::fs::create_dir(root.path().join("module")).unwrap();
        std::fs::write(root.path().join("node.exe"), "runtime identity").unwrap();
        std::fs::write(root.path().join("module/cli.js"), "public fixture").unwrap();
        let shim = root.path().join("cli.cmd");
        std::fs::write(&shim, "@ECHO off\nGOTO start\n:find_dp0\nSET dp0=%~dp0\nEXIT /b\n:start\nSETLOCAL\nCALL :find_dp0\n\nIF EXIST \"%dp0%\\node.exe\" (\n  SET \"_prog=%dp0%\\node.exe\"\n) ELSE (\n  SET \"_prog=node\"\n  SET PATHEXT=%PATHEXT:;.JS;=;%\n)\n\nendLocal & goto #_undefined_# 2>NUL || title %COMSPEC% & \"%_prog%\"  \"%dp0%\\module\\cli.js\" %*\n").unwrap();
        let launch = node_launcher(root.path(), &shim, &std::ffi::OsString::new()).unwrap();
        assert!(launch.program.ends_with("node.exe"));
        assert_eq!(launch.args.len(), 1);
        assert!(launch.args[0].ends_with("cli.js"));
        for source in [
            "@echo Custom setup & node module/cli.js %*",
            "@\"%~dp0node.exe\" --eval=unsafe \"%~dp0module/cli.js\" %* & echo extra",
            "@\"%~dp0node.exe\" \"%~dp0missing.js\" %*",
        ] {
            std::fs::write(&shim, source).unwrap();
            assert!(node_launcher(root.path(), &shim, &std::ffi::OsString::new()).is_none());
        }
    }
    #[test]
    fn detection_requires_installed_tools_and_a_project_index_without_executing_them() {
        let root = tempfile::tempdir().unwrap();
        let bin = root.path().join("bin");
        std::fs::create_dir(&bin).unwrap();
        for name in ["rtk", "codegraph"] {
            let path = bin.join(if cfg!(windows) {
                format!("{name}.exe")
            } else {
                name.into()
            });
            std::fs::write(&path, "Not executed by detection").unwrap();
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o755)).unwrap();
            }
        }
        let search = std::env::join_paths([bin]).unwrap();
        let text = guidance_on_path(root.path(), &search);
        assert!(text.contains("RTK is installed"));
        assert!(!text.contains("CodeGraph is installed"));
        std::fs::create_dir(root.path().join(".codegraph")).unwrap();
        assert!(guidance_on_path(root.path(), &search).contains("CodeGraph is installed"));
        assert!(guidance_on_path(root.path(), &std::ffi::OsString::new()).is_empty());
    }
}
