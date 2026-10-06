# Next roadmap handoff — developer workspace

**Superseded starting order — 2026-10-06.** Use the
[current session handoff](../HANDOFF.md) for immediate implementation. AI reliability
and milestone 20 self-repair take priority; milestones 16–19 remain paused. This
earlier handoff preserves the workspace requirements and prototype gate for when
that track resumes. Its baseline/process details are historical snapshots.

**2026-10-06. Planning only; runtime implementation has not started.** Use this
document when opening a fresh Dolores development chat. The new direction is
milestones 16–19 in [ROADMAP](../ROADMAP.md), specified in
[developer workspace](developer-workspace.md). Read those before proposing code.

## Product intent

Dolores combines an agent harness with a useful developer workspace, so the user
can monitor project A/B, inspect/edit files, review Git and use terminals without
opening a separate VS Code window for every project. Required arrangement: A chat
top left / A file top right / B chat bottom left / B file bottom right. Reorder and
split tabs first; dragging tabs to a detached native window remains a named
milestone, not an omitted feature. First LSP families are **TypeScript/JavaScript
and Rust**, confirmed by the user.

The left feature rail selects Chats / Files / Git, changing the adjacent resizable
panel. Keep Settings at the bottom left, familiar icons/hints, the original infinity
brand and the existing system/light/dark palette. Short contextual controls and
progressive disclosure remain important; avoid wordy routine pages or decorative
colored left edges. Discuss material assumptions before most implementation work
and show the workspace prototype before runtime UI changes.

Dolores's broader character is calm, kind, candid and questioning. Its self-evolving
vision remains: inspect capabilities/source, propose fixes to faulty skills or
extensions, test low-risk changes and activate with rollback. The protected host's
permissions, activation/evaluation boundary and write validation stay protected.
Do not treat an editor as a model permission expansion or claim consciousness.

## Current technical starting point

Flutter desktop plus bundled Rust host/typed plugins, SQLite and native credentials.
Early Iced/Tauri/Svelte experiments have been removed. The maintained local SDK is
Flutter 3.47.5 stable. UI is currently one ChatController selected session and one
Rust Engine active run. Multi-project concurrent views therefore require real host
ownership work, not just tab widgets. The proposed host owns runs/documents/Git/
PTY/LSP; windows own views and subscriptions. Detached windows cannot separately
recover/open the profile or shut down shared services.

Candidate editor/terminal/window libraries are researched options, not installed
dependencies or proven performance. Editor and numeric resource gates precede
selection; official Flutter windowing is experimental in the checked documentation.
Prefer native Flutter editing first, qualify detached-window overhead separately.

## Standing work rules

- Read applicable AGENTS.md and RTK guidance; use rtk for commands and CodeGraph
  before locating/understanding indexed source. Follow [UI](../UI.md).
- Use Python build/launch helpers; no PowerShell helper scripts. **8.4 platform CI
  is deferred until explicit user instruction.** Do not add it under another name.
- Batch authorized milestones, commit each completed brick in English, with hooks
  enabled. Do not push without an explicit request.
- Runtime tasks require the updated normal app built and visibly launched with
  original configuration/history preserved. Diagnostic builds do not satisfy it.
  Documentation-only tasks do not require relaunch.
- Verify the basic flow plus one or two realistic edge failures and recovery per
  brick. Record actual results/open gaps in [ACCEPTANCE](../ACCEPTANCE.md).
- Use configured Qwen3.5-2B for routine live tests; DeepSeek V4.1 Flash for harder
  ones. Keep usage bounded, selected provider/model unchanged and private data ignored.
- Never raise budgets silently to disguise model failures, replay uncertain effects
  blindly, lose dirty buffers, or count fixtures as real-model qualification.

## Open gates carried forward

[Remaining work](../qualification/remaining-work.md) is the current ledger:
sustained idle CPU, general computer-use reliability, broad model-based memory
extraction, useful automatic skill improvement/live regression restoration,
in-app browser installation and broader platform/accessibility qualification remain
open. Exact simple response preferences/corrections were locally fixed and live
checked; approved evidence survives command-review pauses. DeepSeek skill trials
tied and correctly did not activate. Do not report these as broad learning success.

The maintenance batch ended at commit
`bdfca2866f235b26a82525358d756e7940a153a9`. Later planning commits must be discovered
from Git; process IDs, build identity and dirty state are ephemeral and must be
checked afresh. The original preview data directory was
`output/model-picker/preview/data`; confirm ownership before replacing a preview.

## First action in the next chat

Discuss/review the 16.0 prototype and component feasibility scope. Preserve the
requested four-pane layout and the explicit project binding. Do not begin the
remaining runtime milestones merely because their plan exists. All milestones
have failure/recovery and resource gates; no timeline or full VS Code parity has
been promised.
