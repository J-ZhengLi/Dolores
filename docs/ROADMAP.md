# Current roadmap

Use this page to locate active work. The selected batch owns its checklist and
status; this page owns priority and exclusions. Existing behavior belongs in the
user guide/architecture, not in repeated delivery summaries here.

## Active work

The [everyday harness batch index](plans/everyday-harness/README.md) is the source
of truth for batch status. EH00–EH08 have delivered their declared scopes.

- Continue [EH09](plans/everyday-harness/eh09.md): EH09.1–EH09.4 are delivered within
  declared Windows bounds. Continue with EH09.5 cross-feature lifecycle integration.
- After EH09's dependency/acceptance gates, run
  [EH10 integrated qualification](plans/everyday-harness/eh10.md).
- Read the [umbrella scope](design/everyday-harness-implementation-plan.md) only
  for cross-batch scope or dependency decisions; ordinary fixes use their feature
  specification and current code.

Implementation does not close a qualification gap automatically. Consult the
[acceptance limits](ACCEPTANCE.md) when a task or release depends on one.

## Scope and exclusions

Preserve existing Flutter/Rust features and user data. New requirements need an
explicit scope decision; a historical plan is not authorization to start work.
H21 remote task supervision remains deferred.

Separate qualification/authorization is required for non-Windows releases,
signing/public updater, cloud sync, a marketplace, bundled model hosting,
unattended native core replacement and unattended swarms. Outbound remote MCP
is included in EH09; that does not enable remote control of Dolores.

Detached developer views retain their held-backend/single-window fallback.
Manual scheduled-task creation UI remains deferred; chat is the creation surface.
Full VS Code extension compatibility, debugger, remote development, notebooks and
advanced destructive Git history rewriting remain outside the developer workspace.

## Historical plans

Earlier milestone numbers, layout explorations, completed brick sequences and
their original gates are retained in [Git history](IMPLEMENTATION_HISTORY.md#roadmap-and-delivery).
Use current feature specifications for behavior; historical layouts and old
“next” instructions do not override today's task.
