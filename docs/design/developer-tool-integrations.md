# Installed developer tools

Working chats discover optional CodeGraph and RTK launchers from the app's PATH
when preparing the command catalogue. Preview and execution use the same
project-specific specification. Detection reads paths and supported launcher
files only; it does not run commands, install packages or change project settings.
Restart Dolores after changing PATH. Other CLIs remain available through the
ordinary command tool without a program-name allowlist.

CodeGraph guidance appears when its launcher exists and the selected project
already contains `.codegraph/`. The model is directed to prefer an advertised
CodeGraph MCP tool, or `codegraph explore` through `run_command`, for relevant
source and call paths before broad searches. Missing/stale indexes retain the
ordinary file-tool fallback. Detection does not initialize, rebuild or upgrade an
index, start a service, or import another agent's MCP configuration.

RTK guidance directs supported human-readable checks through RTK and uses its
`proxy` path for exact output or unsupported commands. Structured protocol/JSON
evidence and explicit raw-output requests remain unfiltered. This is automatic
model guidance, not a shell-command rewriting hook; model compliance is not
guaranteed. Dolores does not import or execute Claude/Codex hooks.

On Windows, the command tool recognizes stock npm Node launchers and simple
`@"%~dp0...node.exe" ... "%~dp0...js" %*` forwarding launchers. It resolves the
runtime, supported literal runtime flags and entry script before review, then
starts Node directly with the original literal argument vector. The review shows
the actual runtime/script invocation; no shell expansion occurs. Node receives
the entry script in ordinary drive/UNC form because its module resolver rejects
Windows verbatim path prefixes. Filesystem identity checks remain canonical. Command sessions
share this preparation. Unknown batch wrappers require their explicit shell or
interpreter and produce an actionable error; they are never guessed or evaluated.

Normal task access, exact review, command capture, cancellation and ownership
continue to apply. Wrapper resolution grants no new permission, broadens no OS
isolation grant, and never retries an executed command. No storage migration or
bundled third-party runtime is introduced. Tests use credential-free launcher
fixtures; installed-tool and live-model checks must be identified separately.

Upstream interfaces: [CodeGraph CLI](https://colbymchenry.github.io/codegraph/reference/cli/)
and [RTK awareness and hooks](https://github.com/rtk-ai/rtk/blob/develop/hooks/README.md).
