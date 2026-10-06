# Fresh-session handoff — 2026-10-06

## Start here

Continue the existing implementation in a fresh chat. The immediate order is
long-task reliability qualification, then **milestone 20 — Harness self-repair**.
Milestones 16–19 (developer workspace) remain the later, paused track. Platform
CI 8.4 remains deferred until the user explicitly requests it.

1. Read the repository's `AGENTS.md`, this guide, the newest sections of
   [acceptance](ACCEPTANCE.md), [task-loop qualification](qualification/task-loop.md)
   and the [self-repair specification](design/harness-self-repair.md).
2. Check current Git status/build/process ownership. This guide records a
   snapshot; commit IDs and local process state must be verified afresh.
3. Preserve the exact task's retained HTML/scripts and useful qualification below;
   end-to-end completion remains unqualified. Repair reproduced harness faults
   and document actual model failures separately.
4. 20.1–20.2 now provide matching-source inspection and reviewed native proposals.
   The user selected native-first; 20.3's proposed broader Wasm hook is not adopted.
   Prepare dependent native evaluation/install/restore when prerequisites pass.
   Commit each completed brick. Discuss material
   architecture/authority changes before adopting them.
5. At milestone 20 exit, report actual repair/restore evidence and remaining
   gaps. The later workspace track retains its reviewed prototype and component
   feasibility gate; a plan's presence does not make it implemented or accepted.

## Latest delivered work and unresolved task

Implementation baseline: `007b60ea56d78c50e68acb4f11a3d23191f28b98`
(`fix: [agent] Continue ordinary tasks beyond four tool calls`). Earlier response
recovery: `b7ba04c721aac90854055b5436952b1a6f399c46`.

Newer source/evidence brick: `8dd1b5f3961ac13d68fc90084e52e61ea92fccb0`.
20.2 adds scoped native proposals, schema 32 and separate baseline/candidate
snapshots. Verify its latest commit from Git rather than this guide. Rust 336
library tests (one ignored), Clippy, 250 Flutter tests/analysis, normal-native
fixture and bounded five-step DeepSeek proposal handling pass. Candidate code
was not executed; a complete build workspace/evaluation remains unimplemented.
The original normal profile is preserved. Foreground visual verification awaits
Windows unlock; do not interact with the locked desktop.

- Blank output delegates the response maximum to the provider. Existing numeric
  settings remain. Model inactivity is separate from optional task time; human
  approval does not expire on the model clock. Stop remains available.
- Provider thinking is visible as a collapsed rolling preview, not a generated
  Codex-style reasoning summary. Wire accounting bounds traffic without decoded
  progress rather than total useful stream length; malformed/oversized individual
  fields and incomplete calls still refuse execution.
- Fresh task counts are Automatic: checkpoint at 64 model calls / 128 tool attempts
  per segment. Existing numeric counts, including old four-call settings, remain.
  Clear optional count fields explicitly to use Automatic. Continue saves evidence
  and uses fresh requests; it does not replay pending calls.
- Repeated failed requests pause before a third identical attempt, or after six
  consecutive failures. Success clears the streak. Declined plans release review
  slots. Journals retain 2048 entries plus a terminal marker. Command validation is
  aligned at 8 KiB per argument / 16 KiB aggregate / 32 KiB JSON.
- Image clipboard paste and pending/saved sent-bubble previews were implemented;
  image capability is explicit per model. Other-platform/physical-input acceptance
  remains separate from Windows fixture results.

**Exact unresolved case:** configured **DeepSeek V4.1 Flash**, project
`D:\Workspace\mario_clone`, instruction
`Build me a Super Mario clone and output it as an HTML file`.
The earlier fresh native run created `mario-clone.html` (14,604 bytes), with
independent syntax/rendering/movement/jump/restart checks passing. A reviewed
approval survived a measured 190-second wait. The agent then hit command bounds
and its old four-call allowance before qualified final validation. The file and
receipts remain. Preserve them; inspect current files before another creation.

A useful bounded follow-up used eight reviewed list/read/create/command steps
and created two validation scripts. Rendering/gameplay evidence is retained;
the HTML stayed byte-identical. The driver reached its 600-second bound before
the pending bot-test edit and final response. No edit was consumed after timeout.
Command execution was exercised; successful completion/winning was not qualified.

Latest evidence: live Qwen six-file task passes with seven model calls and six
reviewed reads. A normal native fixture preserves 128 reads, a checkpoint and all
514 activity entries; repeated missing-file recovery and explicit continuation
restart pass. Rust library suites, Clippy, 248 Flutter tests and analysis pass.
Four DeepSeek comment-heavy command probes remain **unqualified** (driver denials
or adapter shape/size refusal). They do not establish a failing Mario regression
or successful end-to-end repair. Avoid repeated synthetic padding probes; test the
actual useful task and its validation. See the linked qualification reports for
precise settings, evidence and exclusions.

## Self-repair: foundation versus missing implementation

Flutter desktop hosts the Rust plugin-based engine, SQLite and native credentials.
Early desktop experiments were removed. Source inspection and recent failure
summaries exist, alongside skills and restricted ABI 1 recovery-hint mods with
trials/activation/restore. They **cannot repair the native parser/agent/UI**.

Milestone 20's full conversation-level repair workflow is not implemented:

| Brick | Next deliverable |
| --- | --- |
| 20.1 | Implemented matching-source navigation and typed failed-stream evidence; visual/platform/general-model gaps remain. |
| 20.2 | Implemented scoped selected-file snapshots and visible reviewed native proposals; no generated execution or complete build checkout. |
| 20.3 | Broader Wasm seam not adopted; reviewed native pipeline prioritized by the user. |
| 20.4 | Independent baseline reproduction, candidate and frozen regression trials. |
| 20.5 | Qualified extension activation or reviewed native build/install/restart, with restore. |
| 20.6 | Real repair, withheld non-improvement and rollback qualification. |

The [specification](design/harness-self-repair.md) owns authority, source-bundle,
state-machine and test/install contracts. Native/core replacement stays reviewed;
low-risk qualified extensions may activate under the user's enabled policy with
rollback. Compilation or inspection alone is not repair acceptance.

## Reference map: load by task

| Task | Authoritative references |
| --- | --- |
| Priority/order and planned exit gates | [Roadmap](ROADMAP.md); [self-repair](design/harness-self-repair.md) |
| Current implementation and bridge | [Architecture](ARCHITECTURE.md); [Flutter API](flutter/flutter-api.md) |
| Long generation, timeouts and thinking | [Response recovery](qualification/response-recovery.md); [reasoning streams](qualification/reasoning-streams.md); [recovery design](design/long-task-recovery.md) |
| Task/checkpoint/command qualification | [Task-loop evidence](qualification/task-loop.md); [budgets](design/task-budgets.md); [commands](design/approved-commands.md) |
| Image/paste fixes | [Stream and image evidence](qualification/stream-and-image-recovery.md); [attachments](design/attachments.md) |
| Original evolving-harness intent | [Vision discussion](design/dolores-vision-discussion.md); [target architecture](design/evolving-harness-architecture.md); [behavior](design/dolores-behavior.md); [DSH/Claude research](research/evolving-harnesses.md) |
| Prior maintenance gates | [Remaining work](qualification/remaining-work.md); [learning](qualification/learning-followup.md); [computer use](qualification/computer-use.md); [idle resources](qualification/idle-resources.md); [everyday UX](qualification/everyday-ux.md) |
| Paused developer workspace and prior handoff | [Workspace specification](design/developer-workspace.md); [earlier handoff](design/developer-workspace-handoff.md) |
| UI changes | [UI contract](UI.md); [UX simplification](design/ux-simplification.md); [control map](design/ux-control-map.md) |
| Historical completed bricks | [Implementation history](IMPLEMENTATION_HISTORY.md); dated sections of [acceptance](ACCEPTANCE.md) |

Older handoffs and acceptance entries are historical snapshots. This guide sets
the current starting order; specifications own the design and newer acceptance
entries own demonstrated behavior. Remaining gates include general computer-use,
useful automatic learning/restore, GLM default reasoning, representative idle/low-end
resources and other-platform/accessibility checks. Retain failures as failures.

## Working agreement and local verification

The user wants a lean, modular, calm and helpful harness with concise controls,
familiar icons, the original infinity logo, theme previews and no decorative
colored left status stripe. Follow `docs/UI.md`. Discuss questionable assumptions
before substantial implementation; use evidence rather than repeated apologies.

Follow `AGENTS.md`: CodeGraph first for indexed code; RTK for agent shell commands;
English commits per brick; basic plus one or two realistic edge/recovery cases;
record acceptance gaps. Runtime changes require the normal Python build/visible
launch, preserving configuration/history. Documentation-only changes use document
checks and a commit, without relaunch. Keep services lazy and measure new overhead.

The maintained commands are `python scripts/desktop.py build` and
`python scripts/desktop.py launch`. The original local preview profile is
`output/model-picker/preview/data`, with owned-process record
`output/maintained-preview.json`. Use an absolute profile path when launching and
verify ownership before replacement. The maintained launcher reports the normal
window present, but foreground inspection is pending desktop unlock. Verify
current process/window state rather than trusting old process IDs.

Routine live tests use configured Qwen/Qwen3.5-2B; harder cases use configured
DeepSeek V4.1 Flash. Preserve selected settings and bound test spending. Retrieve
credentials locally without printing them; keep private transcripts/captures and
test profiles under ignored output, and record public evidence only. Existing
local helpers under output are conveniences, not portable checked-in tests.

Native UI verification may use the installed computer-use skill. Its Windows
automation rules prohibit approving terminal commands through UI automation;
use a separately reviewed, tightly scoped native-host test path for command
qualification instead. A blocked automation action is not a product defect.
