# Codex setup compatibility

Dolores can reuse extension setup written for Codex without invoking the Codex
agent or importing its account, model, approval or sandbox settings.

## Profile and source contract

Settings → Connections → Codex setup selects an absolute Codex home. Reuse an
existing profile read-only, or choose the separate Dolores setup folder. Enable
project setup only for projects whose executable integrations you intend to use.
This selection establishes the source boundary once; individual skills need no
review. The selection is stored in Dolores's data directory, separately from
Codex files. Changes take effect on the next task.

The loader reads user `config.toml` and `hooks.json`, then enabled project
`.codex/` layers from repository root to the working directory. Nearer MCP
entries replace an entire named server, including disable flags. Removing a source
removes its entries on the next task. Invalid/unsupported entries expose diagnostics
without printing configuration values. Auth files and model/provider settings
are never read. No real Codex configuration is written.

Instructions use nonempty `AGENTS.override.md` before `AGENTS.md` in each scope.
The optional standalone `@file` reader is a Dolores extension for generated
instruction references; it resolves relative paths against the source, canonicalizes
identities and skips repeated/cyclic sources. It does not evaluate commands.
Ordinary skills use the existing selection/version machinery with live text and
resolved-source identity. Profile `skills/`, project `.agents/skills/` and legacy
project `.codex/skills/` are discovered; modern project sources win. Existing
Dolores global skills remain available. Explicit opt-outs and managed package
ownership remain effective. Removed sources disappear; changed sources invalidate
stale composer selections.

## Implementation batches

- Profile/configuration, instruction and skill loading: implemented.
- Configured MCP stdio runtime: implemented.
- Synchronous Codex command hooks and shell translation: implemented.
- Setup terminal and fixture update/removal qualification: implemented. Installed
  third-party setup commands and live-model integration remain unqualified.

The supported public contracts are
[MCP configuration](https://learn.chatgpt.com/docs/extend/mcp),
[instruction discovery](https://learn.chatgpt.com/docs/agent-configuration/agents-md)
and [command hooks](https://learn.chatgpt.com/docs/hooks).
Compatibility does not imply Codex CLI, plugin marketplace, login, managed policy
or sandbox parity. Unsupported fields/events/handlers are diagnosed explicitly.

## Configured MCP execution

Enabled stdio entries use their literal command/arguments, working directory,
explicit environment and inherited `env_vars`; `CODEX_HOME` is scoped to the child.
Credential-named and long configured environment values are redacted from server
results; ordinary short flags are not treated as secrets inside every RPC field.
PATH and supported Windows Node launchers resolve through the ordinary command
launcher. Startup and tool timeouts and enabled/disabled tool lists are honored.
Configured metadata is validated independently of the legacy inline-catalog size
limits. A large single schema uses the same saved-result pages, so discovery does
not return an empty page with an unchanged cursor.
An unavailable variable or launcher produces a diagnostic, never an installer or
implicit download. No server is started by opening Settings or context preview.

A run owns each connection on one supervised worker. Tools are discovered through
the paged `codex_mcp` catalog and called by stable canonical aliases. This avoids
registering every server schema in the fixed built-in catalog and loading every
schema into every model request. Server workflow instructions accompany the run.
Stop, completion and lost ownership close the worker/process tree. A protocol
failure ends the connection; an executed action is never automatically reconnected
or replayed. Config/source updates and removal apply on the next task.

Large redacted results spill into Dolores-owned, workspace-scoped result files.
The returned result ID and byte offsets let `codex_mcp` read subsequent pages
without another server call. These retained files contain tool evidence, not
authority or executable requests. Source profiles are never used for output.

Compatibility currently supports text MCP results and stdio transport. HTTP and
OAuth entries expose a diagnostic and remain available through Dolores's existing
manual Connections flow. Sampling, elicitation, resources, prompts and async tasks
are not advertised. Compatibility execution is excluded from scheduled and
computer-use runs; planning does not start external servers.

## Command hooks

Synchronous `command` handlers support `SessionStart`, `UserPromptSubmit`,
`PreToolUse`, `PostToolUse` and observational `Stop`. Regex matchers apply to
session sources and canonical tool names; prompt/Stop matchers are ignored.
Windows-specific `commandWindows`/`command_windows` overrides are supported.
One JSON object goes to stdin, including session/turn/model/cwd/permission mode;
no transcript file is synthesized. Handlers use the selected project cwd and
child-scoped `CODEX_HOME`. The default shell is `cmd /d /s /c` on Windows and
`/bin/sh -c` elsewhere; `DOLORES_CODEX_SHELL` can select an installed shell.
POSIX hook scripts on Windows should explicitly invoke an installed POSIX shell.
This environment choice is not copied into the machine's environment.

`run_command` retains literal `program`/`args`. When hooks are active it also
accepts an explicit `command` string in the disclosed shell. This form matches
`Bash` and supplies `tool_input.command`; native argv requests match
`run_command` with their original structured input. Arbitrary argv is never joined
to manufacture shell source. MCP calls match the stable returned `mcp__...` alias
and send the server's arguments, without gateway metadata.

`PreToolUse` applies object `updatedInput` only with
`permissionDecision: "allow"`. The effective input is then validated and prepared
by the actual tool and passes normal task approval and rechecks. Hook approval
does not grant access. Deny/block decisions and exit 2 prevent dispatch. Exit 0
with empty output succeeds; supported additional context is delivered to the model.
Other failures expose diagnostics and retain normal processing. Receipts bind
original/effective inputs and handler sources; changed source files apply on the
next task, not midway through a pinned run. No handler or action is retried.

`SessionStart` fires once per chat per app lifetime when compatibility is used;
`UserPromptSubmit` fires once for a new human turn. Compaction does not replay
either. Post-tool feedback preserves completed-action receipts and cannot undo
effects. `Stop` runs after a completed model turn; Stop-driven continuation is
diagnosed as unsupported and does not create a new task. Interrupting a task
cancels hook children without running additional handlers.

Async, MCP/prompt/agent hook handlers, permission escalation, managed hooks,
plugin hooks, output suppression and output replacement are outside this subset.
Unsupported events/options/response forms expose diagnostics. Existing reviewed
Dolores lifecycle scripts remain independent and keep their own semantics.

## Setup handoff and recovery

Save the selected profile, then use **Open setup terminal**. The ordinary human
terminal opens in the working folder with `CODEX_HOME` set only in that child.
It creates a missing selected profile directory but writes no source configuration
or installer command on its own. Run the package's Codex setup command there, then
Reload this page or start a new task to load changes. Older installers that ignore
`CODEX_HOME` require their documented project-local option or an existing profile;
Dolores does not impersonate a Codex CLI or redirect HOME.

The setup terminal follows normal terminal ownership, Stop and close behavior;
it receives no model input. Closing Settings preserves other settings drafts.
Real-profile setup, installation or upgrade is never performed by automated tests.
Selecting a profile does not copy its credentials into a model request.

Configuration is read fresh for each task. Updates, disables and removal therefore
do not reuse a saved imported catalog. Active tasks pin their original definitions
and do not replay initialization or actions after a source edit. Corrupt Dolores
compatibility settings disable this adapter and display recovery guidance without
blocking chat history or replacing the corrupt file until the user saves again.
Fixture checks cover the ordinary Start path, MCP process reuse and large-result
reads, rewritten command approval, Stop/denial, source lifecycle and child-only
setup environment. They do not establish live-model compliance, released
installer interoperability, HTTP/OAuth reuse or Codex feature parity.
