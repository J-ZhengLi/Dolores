# Compatibility with an existing harness setup target

Researched 2026-10-27. Recommendation and implementation proposal; this is not
delivered compatibility. No installer was run against a real agent profile.

## Selected target: Codex (user decision, 2026-10-27)

**This decision supersedes the Claude recommendation and Claude-specific batches
below.** Those sections retain the comparison evidence, not the active plan.
The user already configures Codex integrations, making existing setup a useful
read-only test source. Rust supports TOML through established crates; JSON already
has good Rust support too. TOML parsing is straightforward but does not implement
MCP sessions, hook execution or tool-input translation.

The active contract is:

- Read `[mcp_servers.<name>]` from the selected Codex home's `config.toml` and
  trusted project `.codex/config.toml`. Translate supported transport, arguments,
  environment and tool controls into Dolores's MCP runtime. Unsupported options
  need diagnostics rather than silent partial activation.
  ([Official MCP configuration](https://learn.chatgpt.com/docs/extend/mcp?surface=cli))
- Discover Codex `AGENTS.md`/`AGENTS.override.md` guidance and ordinary skills using
  source provenance and the documented scope chain. Handle installer-generated
  file references explicitly; do not assume Claude's `@` expansion is a Codex
  contract. ([Official instruction discovery](https://learn.chatgpt.com/docs/agent-configuration/agents-md))
- Read `hooks.json` and supported inline TOML `[hooks]` definitions at active
  layers. Implement stdin JSON, tool-name aliases, matchers and command handlers.
  Codex shell hooks match `Bash` and use `tool_input.command`; `PreToolUse` rewrite
  requires `permissionDecision: "allow"` plus `updatedInput`. Apply the rewritten
  input before normal Dolores validation/permission checks. Codex's unsupported
  `ask` response is not a Claude-compatible decision to copy blindly.
  ([Official hook contract](https://learn.chatgpt.com/docs/hooks))
- Support an explicitly selected existing Codex profile as a read-only source.
  New setup can use a Dolores-owned `CODEX_HOME` when the installer respects it.
  Never change the real Codex profile, credentials, selected model or global
  environment. Existing Codex authentication is not part of configuration reuse.

Local read-only checks found `codegraph` and `node_repl` MCP entries in the active
Codex config, with no hook events in its existing hooks file. Only names and field
keys were inspected; private arguments/environment values were not emitted.
RTK 0.42.4 offers `init --codex` but its `hook --help` has no `codex` processor.
Its existing installation can test instruction-based use, not transparent Codex
rewriting. Use a generic hook fixture and later a compatible installed RTK release
for that acceptance case; do not install or upgrade tools as part of this plan.

Installed CodeGraph 1.6.0 hardcodes global `~/.codex`; current pinned upstream
supports `CODEX_HOME`. Reading the existing configuration works independently of
that installer difference. Isolated setup needs a verified compatible installer
or its documented project-local mode.
([Pinned installer](https://github.com/colbymchenry/codegraph/blob/69af19ace270f2e940444d514de6dde971df12df/src/installer/targets/codex.ts),
[RTK Codex integration](https://github.com/rtk-ai/rtk/blob/develop/hooks/codex/README.md))

Active batches, each committed separately:

1. Generic Codex config/profile loader, instruction and skill discovery; remove
   package-name discovery. Test precedence, missing/malformed config, source
   changes and read-only reuse of isolated copies of installed configuration.
2. Configured MCP runtime with launcher/environment translation, stable names,
   server lifecycle and missing/disabled/removal handling. Test a generic fixture
   server before testing installed CodeGraph in an isolated indexed project.
3. Codex command-hook protocol and shell-tool translation. Test rewrite, denial,
   context delivery, Stop and post-rewrite policy with a package Dolores does not
   recognize. Report the supported event/field subset and unsupported options.
4. Generic setup-profile handoff plus installer/update/uninstall qualification.
   Test existing instruction-only RTK separately from hook-capable versions;
   preserve real Codex settings and verify no tool replay or duplicate activation.

Acceptance targets setup files and public extension contracts, not full Codex CLI,
marketplace, App Server, account/authentication, managed-policy or sandbox parity.
Full implementation and live integration acceptance remain outstanding.

## Recommendation

Implement a **Claude Code configuration and command-hook compatibility mode**.
Third-party setup commands can target `claude`; Dolores consumes their resulting
configuration through an adapter. Integrations are selected by configuration,
not executable names. Replace the CodeGraph/RTK-specific discovery rules added
in `96943de`; retain the general Windows Node launcher fixes.

This is a compatibility surface for installers, not a replacement Claude CLI,
Claude model dependency, or promise to implement every Claude feature. The
initial useful contract is settings, MCP connections, instruction imports,
ordinary skills, and command hooks around prompt/tool execution. Claude documents
hooks as externally invoked handlers with JSON stdin and event-specific output,
so they can be adapted without embedding another agent runtime.
([Hook contract](https://code.claude.com/docs/en/hooks))

## Why this target

| Candidate | Existing integration surface | Dolores implementation consequence |
| --- | --- | --- |
| Claude Code — recommended | JSON settings and MCP configuration, Markdown guidance/skills, executable command hooks | Implement file discovery and a JSON process protocol; reuse Rust process supervision and MCP client. |
| Codex | TOML MCP configuration and AGENTS.md; current RTK also targets its hook format | Viable later adapter. The locally installed RTK's `--codex` setup is instruction-based; do not assume newer hook behavior from current upstream. |
| OpenCode | JSON/JSONC configuration plus executable JavaScript/TypeScript plugins | Reproducing plugin behavior requires a JavaScript runtime and parts of the OpenCode client/Bun shell context, not just a JSON parser. |

These are engineering judgments. The underlying interfaces are documented in
[CodeGraph's Codex installer](https://github.com/colbymchenry/codegraph/blob/69af19ace270f2e940444d514de6dde971df12df/src/installer/targets/codex.ts),
[RTK's current Codex integration](https://github.com/rtk-ai/rtk/blob/develop/hooks/codex/README.md),
and [OpenCode's plugin API](https://opencode.ai/docs/plugins/). OpenCode supplies
plugins with a client and shell API, and supports mutation before tool execution;
implementing arbitrary plugins therefore has a larger initial runtime contract.

## What setup actually produces

### MCP and instructions

CodeGraph's verified installer ID is `claude`, not `claude-code`. Its local setup
command shape is `codegraph install --target claude --location local`. It writes
`.mcp.json`, `.claude/CLAUDE.md`, and optionally `.claude/settings.json` permissions
and a prompt hook. The MCP entry starts `codegraph serve --mcp`. Dolores must load
the server entry and instruction files, rather than special-case this program.
([Pinned installer source](https://github.com/colbymchenry/codegraph/blob/69af19ace270f2e940444d514de6dde971df12df/src/installer/targets/claude.ts))

Claude's project MCP convention uses an `mcpServers` object; its global/default
scope differs from project configuration. Dolores already has an MCP client and
a manual JSON importer, but importing a file is not automatic discovery or a
compatible runtime lifecycle. Configured servers need stable server/tool naming,
startup, tool discovery and removal handling. MCP session behavior and server
guidance also need explicit tests; merely reading the JSON proves neither.
([MCP configuration and lifecycle](https://code.claude.com/docs/en/mcp))

### Command interception

The installed RTK 0.42.4 reports `rtk init -g --agent claude` as its global setup
shape. A dry run redirected by `CLAUDE_CONFIG_DIR` targets that directory's
`settings.json`, `CLAUDE.md`, and `RTK.md`. Its local init dry run only adds
instructions and a filters template; it does not establish the global hook.
Current source retains that distinction.
([RTK installer](https://github.com/rtk-ai/rtk/blob/e05c7bdfacfcfe1f845d11f47339e4eae63eb3f5/src/hooks/init/claude.rs),
[profile resolution](https://github.com/rtk-ai/rtk/blob/e05c7bdfacfcfe1f845d11f47339e4eae63eb3f5/src/hooks/init/mod.rs))

A direct local protocol probe sent a synthetic `PreToolUse` event to the installed
`rtk hook claude`. It returned exit 0 with:

```json
{"hookSpecificOutput":{"hookEventName":"PreToolUse","permissionDecisionReason":"RTK auto-rewrite","updatedInput":{"command":"rtk git status"}}}
```

The payload used `tool_name: "Bash"` and
`tool_input: {"command":"git status"}`. No Git command was executed. A compatible
host must apply `updatedInput`, not merely display hook output or tell the model
to prefix commands. Support updates without a permission decision, as this real
version emits them. Rewrite processing must precede validation, preparation and
permission checks on the resulting command.

### Instructions and skills

Support `CLAUDE.md`, `.claude/CLAUDE.md`, profile guidance and their `@file`
imports with source-relative resolution and cycle handling. This is necessary
for generated references such as `@RTK.md`; Dolores currently labels references
literal instead of loading them. Claude's documented imports can be relative or
absolute and recurse up to four hops.
([Instruction imports](https://code.claude.com/docs/en/memory#import-additional-files))

Discover ordinary `.claude/skills/<name>/SKILL.md` and profile skills through the
existing skill machinery. Standard Agent Skills content is portable; Claude-only
dynamic command expansion, skill hooks and subagent declarations require separate
support. Report unsupported executable features rather than claiming the whole
skill format works.
([Skill contract](https://code.claude.com/docs/en/skills))

## A separate profile is possible, with a version boundary

Use `<Dolores data>/compat/claude/` as the proposed user-profile root and scope
`CLAUDE_CONFIG_DIR` to setup and compatible hook/server children. Do not change
the machine environment, redirect HOME, overwrite `claude`, or write into the
user's existing Claude/Codex profiles. The profile is configuration storage, not
a sandbox for arbitrary installers.

Installed CodeGraph **1.6.0** uses `os.homedir()` directly for global Claude paths;
it ignores `CLAUDE_CONFIG_DIR`. Current pinned upstream explicitly respects the
override, including the profile's `.claude.json`. Consequently, isolated global
setup cannot be promised for the installed version. Project-local setup is a
separate path; old installers need that documented alternative or a verified
upstream version supporting custom profiles, not tool-name-specific redirection.
([Upstream profile support](https://github.com/colbymchenry/codegraph/blob/69af19ace270f2e940444d514de6dde971df12df/src/installer/targets/claude.ts#L56-L80))

Dolores should expose a generic setup terminal/runner that shows its compatibility
profile and runs the user's chosen setup command with that scoped environment.
The program's own `--target`/`--agent` options still select `claude`; supporting
the files/protocol does not make upstream installers recognize `dolores` as a new
target. Installers that invoke `claude mcp add`, a marketplace command, or private
Claude APIs need a separately implemented API/CLI compatibility surface. The two
inspected setup paths write configuration directly and do not require a fake CLI.

## Gaps in current Dolores

| Area | Existing behavior | Needed adapter behavior |
| --- | --- | --- |
| Discovery | Named CodeGraph/RTK PATH hints | Read compatible configuration without recognizing package names. |
| Instructions | Reviewed AGENTS snapshots; literal imports | Load compatible guidance/imports with source provenance and documented precedence. |
| MCP | Manual setup/import; small selected catalogue; per-call processes | Automatic configured-server lifecycle in a chosen profile/project; usable advertised names and honest unsupported capability diagnostics. |
| Hooks | Reviewed Node/Python scripts, event JSON in argv; allow/message only | JSON stdin, event/matcher selection, native executable or declared shell launch, updated input and additional context. |
| Tool execution | `run_command` uses literal program/args | Provide a real compatible shell-command surface for `Bash` hooks; never reconstruct arbitrary shell strings by joining argv. |

Owning code: `crates/dolores-flutter-bridge/src/instructions.rs`,
`crates/dolores-core/src/instructions.rs`,
`apps/dolores_flutter/lib/mcp_import.dart`,
`crates/dolores-flutter-bridge/src/lifecycle_hooks/runtime.rs`,
`crates/dolores-tools-command/src/developer_tools.rs`.

## Implementation batches

1. **Configuration and setup profile.** Implement compatible file discovery,
   source provenance, profile/project precedence, instruction imports and ordinary
   skills. Reuse existing settings views to show loaded sources and errors. Remove
   package-name discovery guidance. Do not introduce a new per-package registry.
2. **Configured MCP runtime.** Translate compatible entries into the existing MCP
   client, including PATH/Node launch resolution and configured environment. Test
   initialization, usable tools, session ownership, removal, shutdown and failures.
   Disclose unsupported fields; avoid turning server metadata into permissions.
3. **Command-hook ABI.** Start with synchronous `SessionStart`,
   `UserPromptSubmit`, `PreToolUse`, `PostToolUse` and `Stop` command handlers.
   Define session/turn boundaries explicitly. Support matcher selection, stdin
   JSON, `updatedInput`, `additionalContext`, deny/ask and exit statuses. Add a
   compatible shell tool backed by an available declared shell; show a useful
   missing-shell error on Windows. Keep native argv commands available.
4. **Installer interoperability.** Exercise setup, reload, update and uninstall
   in isolated fixtures. Add a generic setup runner/profile handoff. Use the two
   named tools as acceptance examples, plus an unrelated fixture integration whose
   name is unknown to Dolores, proving there are no package-specific branches.

Rewritten calls must pass normal policy after transformation; neither an imported
allow rule nor hook output expands the chat's authority. Bind both original and
effective input in evidence. Stop cancels hook children and no interrupted action
is replayed. Establish trust for the compatibility profile/project once, rather
than adding a skill-review prerequisite or asking approval for every hook action.

Acceptance must prove model-visible MCP calls, actual hook-rewritten execution,
instruction imports, process cleanup, unchanged real harness profiles, stable
draft/results, malformed-config recovery and a second generic fixture package.
Unsupported plugin marketplaces, in-process JS mods, agent/prompt/HTTP hooks,
managed enterprise policies and Claude login/model APIs are outside the initial
claim. Persisted process/session, notification and permissions semantics require
qualification; naming the mode Claude-compatible alone establishes none of them.

## Evidence and status

- Inspected installed CodeGraph 1.6.0 launcher/package/installer source and RTK
  0.42.4 help, dry-run output and hook response. All setup probes were read-only
  or dry-run. No live model request or real-profile setup was performed.
- Pinned upstream snapshots: CodeGraph
  `69af19ace270f2e940444d514de6dde971df12df`; RTK
  `e05c7bdfacfcfe1f845d11f47339e4eae63eb3f5`. They differ from installed versions;
  released-version support must be checked before documenting working commands.
- Raw downloaded sources and probe helpers remain ignored under
  `output/harness-compatibility-research/`.
- This research does not implement the compatibility mode. Existing special-case
  discovery remains until the replacement batch is implemented.
