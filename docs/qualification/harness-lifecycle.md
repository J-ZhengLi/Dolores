# Workflow lifecycle qualification

EH09.5 qualifies the following Windows x64 host boundaries with credential-free
loopback fixtures. This closes EH09 under those bounds; EH10 still owns broader
daily-use, native input, migration, resource and live-model qualification.

## Connected boundary

`scripts/test-harness-lifecycle.py --directory <fresh-output-directory>` uses the
normal Windows bridge, real SQLite/files, installed Node and public HTTP fixtures.
Its save and restore stages run in separate processes with isolated profiles.

- A reviewed packaged workflow reads its verified resource, calls a reviewed MCP
  connector, delegates a frozen resource read and independently inspects a file.
  Child model/tool costs charge the same durable goal ledger. Children receive
  only scoped file and frozen package tools, without commands/MCP/delegation.
- Parent/child hooks share one serialized allowance. A malformed post-tool
  observer preserves file/resource/connector receipts and the saved reply;
  child-attributed hook results remain in the parent journal. Hook output is
  absent from provider context.
- A scheduled occurrence reads its pinned resource under its reviewed hook
  snapshot. Disabling the current hooks refuses another occurrence before model
  work. Changing the active package and forgetting its old activation cannot
  delete resources still pinned by the future task. Deleting recurrence releases
  that pin once no active occurrence needs it; prior receipts remain inspectable.
  Changed/disabled source skills still require explicit task update.
- Safe-boundary steering adds the exact selected package to the parent reader;
  queued execution binds its selected version and acknowledges the correct input.
  Existing children keep their earlier resource snapshot. Compaction retains host
  bindings and shared counters rather than accepting authority from summaries.
- Stop during a legacy remote call joins cleanup before terminal evidence. One
  external call and one cleanup DELETE occur; no tool is replayed. Restart retains
  child context/evidence and public files without resuming completed or uncertain
  work or reopening the connector.

## Focused failures and limits

Store tests cover scheduled/active-occurrence pins across deletion and reopen,
unresolved queue pins and explicit release. Reader tests cover preflight-only
steering, conflicting replacement, frozen child copies and verified UTF-8 reads.
Hook tests cover blocking failures, Stop, serialization and the shared cap.
Remote tests cover cancelled/dropped invocations, dropped initialization, failed
cleanup and the independent two-second cleanup deadline. A returned result remains
available when cleanup fails. Unsupported termination is recorded without claiming
that the server released its session; abrupt process loss remains uncertain.

`python scripts/qualify-native-ui.py --case subagents --case hooks --case scheduled`
exercises the rendered controls with the normal bridge. Flutter checks retain
pending decisions through cleanup warnings and disclose reviewed scheduled hooks.
These are synthetic rendered/control journeys, not physical input or general
provider/server competence. Raw receipts, public profiles, logs and images remain
in ignored `output/`; no real credentials or live provider quota are required.

Package installation and richer supervision still grant no child command/MCP
capability. H21 remote supervision remains deferred. Third-party OAuth behavior,
other platforms, clean-machine support and broad migration/performance evidence
retain their existing qualification boundaries.
