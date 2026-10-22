use super::*;

#[derive(Clone, Default)]
pub struct ExecutionHost {
    #[cfg(windows)]
    host: Option<Arc<dolores_tools_command::isolation::Host>>,
}
pub(super) enum Reviewed {
    Native,
    #[cfg(windows)]
    Isolated {
        host: Arc<dolores_tools_command::isolation::Host>,
        review: dolores_tools_command::isolation::Review,
    },
}
impl ExecutionHost {
    #[cfg(windows)]
    pub fn new(host: Arc<dolores_tools_command::isolation::Host>) -> Self {
        Self { host: Some(host) }
    }
    pub(super) fn review(
        &self,
        root: &Path,
        launch: &McpLaunch,
        cancel: &CancellationToken,
    ) -> Result<Reviewed, String> {
        let Some(mode) = &launch.isolation else {
            return Ok(Reviewed::Native);
        };
        #[cfg(windows)]
        {
            let host = self
                .host
                .as_ref()
                .ok_or("MCP isolation is unavailable; no native fallback was used.")?;
            if !Path::new(&launch.executable)
                .file_name()
                .is_some_and(|name| name.eq_ignore_ascii_case("node.exe"))
            {
                return Err("Isolated MCP currently supports direct Node executables only; no native fallback was used.".into());
            }
            if host.pending_cleanup()? != 0 {
                return Err("Isolation cleanup is pending. Inspect Commands and Retry cleanup before launching.".into());
            }
            let roots = mode
                .readonly_roots
                .iter()
                .map(PathBuf::from)
                .collect::<Vec<_>>();
            let review = host.review(root, &roots, cancel)?;
            Ok(Reviewed::Isolated {
                host: host.clone(),
                review,
            })
        }
        #[cfg(not(windows))]
        {
            let _ = (root, mode, cancel);
            Err(
                "MCP isolation is unavailable on this platform; no native fallback was used."
                    .into(),
            )
        }
    }
}
impl Reviewed {
    pub(super) fn preview(&self) -> Option<dolores_core::CommandIsolationPreview> {
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
    pub(super) fn spawn(
        &self,
        root: &Path,
        launch: &McpLaunch,
        environment: &[(String, String)],
        cancel: &CancellationToken,
    ) -> Result<(process::Running, File, File, File), String> {
        match self {
            Self::Native => process::spawn_stdio(
                Path::new(&launch.executable),
                &launch.args,
                root,
                environment,
            ),
            #[cfg(windows)]
            Self::Isolated { host, review } => {
                let lease = host.prepare_reviewed(Path::new(&launch.executable), review, cancel)?;
                dolores_tools_command::isolation::spawn_cancellable(
                    lease,
                    &launch.args,
                    environment,
                    cancel,
                )
            }
        }
    }
}
