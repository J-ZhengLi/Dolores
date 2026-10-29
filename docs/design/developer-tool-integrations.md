# Developer CLI launchers

All installed CLIs use the ordinary command tool without a program-name allowlist.
Dolores no longer injects guidance by recognizing particular package names.
Integration instructions and configuration come through the selected
[Codex setup](codex-compatibility.md). Restart Dolores after changing PATH.

For repository-wide literal searches, prefer the installed index or a reviewed
`run_command` invocation of `git grep`/`rg`. The built-in `search_text` is a
bounded fallback: it searches large UTF-8 files within a shared 256-KiB read
budget, up to 32 levels, 256 entries, 64 files and 30 matches. It explains partial
coverage and skipped files. Empty partial results never establish absence;
narrow the scope or switch to repository search instead of repeating the same
scan. Secret/generated/dependency exclusions and link checks still apply.

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
