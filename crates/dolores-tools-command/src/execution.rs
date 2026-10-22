use super::*;
use serde::Serialize;

pub(super) fn describe(mut spec: ToolSpec) -> ToolSpec {
    spec.parameters["properties"]["isolation"] =
        json!({"type":"string","enum":["native","isolated"]});
    spec.parameters["properties"]["readonly_roots"] =
        json!({"type":"array","items":{"type":"string"},"maxItems":8});
    spec.description.push_str(" Native execution is the default. Optional isolation=isolated uses Windows LPAC for direct Node commands only: writable working folder, at most eight explicit absolute readonly_roots, a read-only runtime copy and no network capability. Each isolated launch needs fresh review of its grants, including under Full access. Use explicit --preserve-symlinks and --preserve-symlinks-main Node arguments for local module/script entry points. Unsupported runtimes, aliases, stale grant roots, pending cleanup and setup failures are refused without native fallback. File effects remain after Stop; inspect Commands and Retry cleanup when requested. Browser, human terminals and repair helpers use their separately labeled execution modes.");
    spec
}

#[derive(Default, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub(crate) enum Isolation {
    #[default]
    Native,
    Isolated,
}

pub(crate) enum Launch {
    Native,
    #[cfg(windows)]
    Isolated {
        host: Arc<isolation::Host>,
        review: isolation::Review,
    },
}
impl Launch {
    pub fn label(&self) -> &'static str {
        match self {
            Self::Native => "native; not sandboxed",
            #[cfg(windows)]
            Self::Isolated { .. } => "isolated; Windows LPAC; network denied",
        }
    }
    pub fn preview(&self) -> Option<dolores_core::CommandIsolationPreview> {
        match self {
            Self::Native => None,
            #[cfg(windows)]
            Self::Isolated { review, .. } => Some(dolores_core::CommandIsolationPreview {
                workspace: review.workspace().to_string_lossy().into_owned(),
                readonly_roots: review
                    .readonly()
                    .iter()
                    .map(|p| p.to_string_lossy().into_owned())
                    .collect(),
            }),
        }
    }
    pub fn cleanup_complete(&self) -> bool {
        match self {
            Self::Native => true,
            #[cfg(windows)]
            Self::Isolated { host, .. } => host.pending_cleanup().is_ok_and(|n| n == 0),
        }
    }
    pub fn spawn(
        &self,
        plan: &Plan,
        root: &Path,
        cancel: &CancellationToken,
    ) -> Result<(platform::Running, File, File, File), String> {
        plan.validate(root, cancel)?;
        match self {
            Self::Native => platform::spawn_stdio(
                &plan.executable,
                &plan.invocation.args,
                root,
                &environment(),
            ),
            #[cfg(windows)]
            Self::Isolated { host, review } => {
                let lease = host.prepare_reviewed(&plan.executable, review, cancel)?;
                plan.validate(root, cancel)?;
                isolation::spawn_cancellable(lease, &plan.invocation.args, &environment(), cancel)
            }
        }
    }
}
impl RunCommand {
    #[cfg(windows)]
    pub fn with_isolation(mut self, host: Arc<isolation::Host>) -> Self {
        self.isolation = Some(host);
        self
    }
    pub(super) fn launch(
        &self,
        mode: &Isolation,
        program: &str,
        readonly: &[PathBuf],
    ) -> Result<Launch, String> {
        if *mode == Isolation::Native {
            if !readonly.is_empty() {
                return Err("Read-only roots require isolated execution; native commands use account permissions.".into());
            }
            return Ok(Launch::Native);
        }
        #[cfg(windows)]
        {
            let host = self.isolation.as_ref().ok_or("OS isolation is unavailable for this launch. Nothing was started; no native fallback was used.")?;
            if program != "node" {
                return Err("Isolated execution currently supports direct Node commands. This runtime is unsupported; nothing was started and no native fallback was used.".into());
            }
            if host.pending_cleanup()? != 0 {
                return Err("Isolation cleanup is pending. Inspect Commands and Retry cleanup before reviewing another isolated command.".into());
            }
            let review = host.review(&self.root, readonly, &CancellationToken::new())?;
            Ok(Launch::Isolated {
                host: host.clone(),
                review,
            })
        }
        #[cfg(not(windows))]
        {
            let _ = program;
            Err("OS isolation is unavailable on this platform. Nothing was started; no native fallback was used.".into())
        }
    }
}
