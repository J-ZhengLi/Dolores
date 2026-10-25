use crate::{
    run_journal::{RunCoordinator, RunLog},
    Engine,
};
use dolores_core::{
    lifecycle_hooks::{Configuration, Event, Hook, Mode, ReviewedHook},
    SessionStore, ToolApproval, ToolCall, ToolPlugin, ToolRequest,
};
use dolores_tools_command::RunCommand;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::{
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};
use tokio_util::sync::CancellationToken;
mod runtime;
pub(crate) use runtime::{HookedApproval, HookedTool, Runtime};
#[cfg(test)]
mod tests;
const STALE:&str="Hook review expired or changed. Review exact scripts and commands again; saved hooks remain unchanged.";
#[derive(Deserialize)]
#[serde(tag = "action", rename_all = "camelCase", deny_unknown_fields)]
pub(crate) enum Request {
    View,
    Review { revision: u32, hooks: Vec<Hook> },
    Enable { token: String },
    Disable { revision: u32 },
}
pub(crate) struct Review {
    token: String,
    session: String,
    root: String,
    created: Instant,
    configuration: Configuration,
}
fn scope(store: &dyn SessionStore, session: &str) -> Result<PathBuf, String> {
    let root = store
        .workspace(session)?
        .root
        .ok_or("Open a working chat to configure project hooks.")?;
    Path::new(&root)
        .canonicalize()
        .map_err(|_| "Hook working folder is unavailable.".into())
}
fn script_hash(root: &Path, hook: &Hook) -> Result<String, String> {
    Ok(format!("{:x}", Sha256::digest(script_bytes(root, hook)?)))
}
fn script_bytes(root: &Path, hook: &Hook) -> Result<Vec<u8>, String> {
    hook.validate()?;
    let mut target = root.to_path_buf();
    for part in hook.script.split('/') {
        target.push(part);
        let metadata = std::fs::symlink_metadata(&target)
            .map_err(|_| "Hook script is unavailable. Restore it or disable the hook.")?;
        #[cfg(windows)]
        {
            use std::os::windows::fs::MetadataExt;
            if metadata.file_attributes() & 0x400 != 0 {
                return Err("Hook scripts cannot use links or reparse points.".into());
            }
        }
        if metadata.file_type().is_symlink() {
            return Err("Hook scripts cannot use links.".into());
        }
    }
    if target
        .canonicalize()
        .map_err(|_| "Hook script is unavailable.")?
        != target
    {
        return Err("Hook script path changed. Review again.".into());
    }
    let metadata = target
        .metadata()
        .map_err(|_| "Hook script is unavailable.")?;
    if !metadata.is_file() || metadata.len() > 256 * 1024 {
        return Err("Use a hook script within 256 KiB.".into());
    }
    use std::io::Read;
    let mut bytes = vec![];
    let directory = cap_std::fs::Dir::open_ambient_dir(root, cap_std::ambient_authority())
        .map_err(|_| "Hook working folder is unavailable.")?;
    directory
        .open(&hook.script)
        .map_err(|_| "Hook script is unavailable.")?
        .take(256 * 1024 + 1)
        .read_to_end(&mut bytes)
        .map_err(|_| "Hook script could not be read.")?;
    if bytes.len() > 256 * 1024 {
        return Err("Hook script grew beyond 256 KiB. Review again.".into());
    }
    Ok(bytes)
}
fn prepare(root: &Path, hook: &Hook, event: Value) -> Result<(RunCommand, ToolRequest), String> {
    let runner = RunCommand::new(root)?;
    let mut args = vec![format!("./{}", hook.script)];
    args.extend(hook.args.clone());
    args.push(event.to_string());
    let request=runner.prepare(&ToolCall {id:uuid::Uuid::new_v4().to_string(),name:"run_command".into(),arguments:json!({"program":hook.program,"args":args,"timeout_seconds":hook.timeout_seconds,"capture_bytes":4096}).to_string()})?;
    Ok((runner, request))
}
fn inspect(root: &Path, hook: Hook) -> Result<ReviewedHook, String> {
    let script_sha256 = script_hash(root, &hook)?;
    let (runner, request) = prepare(root, &hook, json!({"schema":1,"event":hook.event}))?;
    let executable = request.command.as_ref().ok_or(STALE)?.executable.clone();
    runner.discard(&request);
    Ok(ReviewedHook {
        hook,
        script_sha256,
        executable,
    })
}
impl Engine {
    pub(crate) fn hooks_call(
        &self,
        session: &str,
        request: Request,
        active: &RunCoordinator,
    ) -> Result<Value, String> {
        let root = scope(self.store.as_ref(), session)?;
        let key = root.to_str().ok_or("Unsupported working folder.")?;
        if matches!(request, Request::View) {
            return Ok(json!({"configuration":self.store.lifecycle_hooks(key)?,"root":key}));
        }
        if active.is_some() {
            return Err(
                "Finish or stop running work before changing hooks. Your edits remain.".into(),
            );
        }
        match request {
            Request::View => unreachable!(),
            Request::Review { revision, hooks } => {
                if hooks.len() > 4 {
                    return Err("Choose at most four hooks.".into());
                }
                if self.store.lifecycle_hooks(key)?.revision != revision {
                    return Err(STALE.into());
                }
                let configuration = Configuration {
                    revision,
                    hooks: hooks
                        .into_iter()
                        .map(|h| inspect(&root, h))
                        .collect::<Result<_, _>>()?,
                };
                configuration.validate()?;
                let token = uuid::Uuid::new_v4().to_string();
                let mut scripts = Vec::new();
                for saved in &configuration.hooks {
                    let bytes = script_bytes(&root, &saved.hook)?;
                    if bytes.len() > 256 * 1024
                        || format!("{:x}", Sha256::digest(&bytes)) != saved.script_sha256
                    {
                        return Err(STALE.into());
                    }
                    let text = std::str::from_utf8(&bytes)
                        .map_err(|_| "Hook scripts must be UTF-8 text for review.")?;
                    let mut end = text.len().min(8192);
                    while !text.is_char_boundary(end) {
                        end -= 1;
                    }
                    scripts.push(json!({"name":saved.hook.name,"text":&text[..end],"omittedBytes":text.len()-end}));
                }
                let response = json!({"token":token,"configuration":configuration,"root":key,"scripts":scripts});
                *self.hook_review.lock().map_err(|_| STALE)? = Some(Review {
                    token,
                    session: session.into(),
                    root: key.into(),
                    created: Instant::now(),
                    configuration,
                });
                Ok(response)
            }
            Request::Enable { token } => {
                let mut slot = self.hook_review.lock().map_err(|_| STALE)?;
                let review = slot.as_ref().ok_or(STALE)?;
                if review.token != token
                    || review.session != session
                    || review.root != key
                    || review.created.elapsed() > Duration::from_secs(300)
                {
                    return Err(STALE.into());
                }
                for saved in &review.configuration.hooks {
                    if inspect(&root, saved.hook.clone())? != *saved {
                        return Err(STALE.into());
                    }
                }
                let saved = self
                    .store
                    .save_lifecycle_hooks(key, &review.configuration)?;
                slot.take();
                Ok(json!({"configuration":saved,"root":key}))
            }
            Request::Disable { revision } => {
                let saved = self.store.save_lifecycle_hooks(
                    key,
                    &Configuration {
                        revision,
                        hooks: vec![],
                    },
                )?;
                self.hook_review.lock().map_err(|_| STALE)?.take();
                Ok(json!({"configuration":saved,"root":key}))
            }
        }
    }
}
