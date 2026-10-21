# Everyday harness delivery contracts

Frozen EH00 contract, 2026-10-21. Implementation follows the
[batch tasks](../plans/everyday-harness/README.md). This specification does not
expose controls by itself. H21 remains deferred.

## Baseline and bounds

The source baseline is `9d33e9f`. SQLite is version 38. The composer refuses
submission while busy; drafts are persisted separately. Host run admission allows
two primaries and four waiting runs, serializing equal workspace scopes. These
are host runs, not pending instructions. Existing durable run/checkpoint evidence,
settings, history, manual continuation and optional scheduling remain authoritative.

The core input limit is 16 KiB UTF-8 and context limit is 128 KiB. EH01 starts
with eight pending instructions per conversation: at most 128 KiB of text, matching
one existing context envelope. Reference payloads retain their existing separate
limits. Reject an over-limit enqueue without clearing its draft; never silently
evict pending work. This new queue bound is a conservative storage ceiling, not
an inference about model capacity. Do not raise existing run or context defaults.

The command baseline retains 30 seconds and 8 KiB capture by default; explicit
reviewed requests allow 1–300 seconds and 1–256 KiB capture. Human PTY output is
bounded at 1 MiB. EH05 initially limits agent sessions to two per active task and
four per host, at most 256 KiB retained output per session, reusing reviewed command
deadlines before allowing an explicit longer-lived session policy. These proposed
ceilings fit existing capture/admission bounds; EH05 measures them before enabling
new paths. A session deadline is not a goal or tool-budget reset.

Existing automatic run ceilings are 64 model calls, 128 tools and four segments;
saved explicit limits remain authoritative. Goal accounting must accumulate across
segments and children. The current ceilings are not evidence that every model can
finish a real task within them.

## Logical records and migration ownership

| Record | Required fields and ownership |
| --- | --- |
| Task | Stable UUID, conversation FK, revision, suspended/recovery flags, run ancestry; one current task per chat in EH01. Retain prior run evidence. |
| Input | Stable ID, task/conversation identity, monotonic display position, Queue/Steer mode, text, revision, delivery state and target run/boundary receipt. EH01 binds delivery times through its run; structured reference snapshots belong to EH02/EH04. |
| Goal | Task FK, revision, user-set objective/criteria/constraints, lifecycle, cumulative budget and evidence references; owned by EH03. |
| Plan/checklist | Task FK, separate accepted-plan and progress revisions, decisions and evidence links; owned by EH03/EH04. |
| Explicit context | Immutable selected identity/revision, scope, sharing preview and bounded resolved content; owned by EH02/EH04. |

EH01 uses the existing conversation UUID as current task identity. It adds tables
through the current transactional migration mechanism; do not
rewrite applied migrations or eagerly create empty goal/plan tables. Version 38
profiles must reopen without changing messages, settings, attachments, schedules
or working files. Index pending input by task, state and order. Foreign keys and
explicit session-deletion cleanup remove logical task/input rows. Deleting a chat
does not remove working files or replay/undo effects. Refuse deletion while the
existing owner cannot safely release the chat. Export includes input text and
delivery metadata as user data when that record ships; secret material follows
existing export rules. Receipt retention follows conversation history; bound
enumeration with paging, rather than pruning unresolved delivery.

## Input states and revision rules

Acceptance means a successful atomic save of a **pending** input; there is no
separate durable "accepted" state. Draft clearing is conditional on the exact
submitted draft and succeeds only after acceptance. An enqueue retry uses the
same UUID; identical submission is idempotent, differing content is refused.

| From | Allowed next state | Meaning |
| --- | --- | --- |
| Pending | Delivering | Persist intent and bind run/boundary before submission. |
| Pending | Cancelled | User cancels before delivery intent. |
| Delivering | Applied | Owner durably acknowledges insertion at the safe boundary. This is not task completion. |
| Delivering | Failed | Owner proves it did not apply the input; show error and explicit new-input recovery. |
| Delivering | Uncertain | Cancellation/crash may have crossed insertion; inspect before new submission. |
| Applied/Cancelled/Failed/Uncertain | None | Terminal receipt; no historical effect replay. |

Edit, reorder and cancel require the current revision and Pending state, under
the same transaction. Increment revisions with checked arithmetic. Reorder validates
the entire pending list; never partially reorder stale input. An active receipt
cannot be edited. A failed/uncertain retry is a new explicit input with ancestry,
not a mutation back to Pending. At restart Delivering becomes Uncertain, Pending
remains Pending, and the task requires inspection/resume before any dispatch.

## Dispatch arbitration

The tested [public corpus](../fixtures/harness-dispatch.json) and
[core contract](../../crates/dolores-core/src/harness_task.rs) define precedence:
Stop → suspended/recovery/decision wait → safe-boundary steering → pending user
turn → eligible goal continuation. Approval/question responses use the existing
decision channel and remain deliverable while the task queue waits.

Evaluate and claim under the existing task owner; host admission remains the only
way to start a primary. A pending instruction does not hold an admission slot.
Equal-root chats still serialize, and each follow-up reacquires admission rather
than holding the root indefinitely. Ordered user input takes precedence over goal
continuation; pending steers retain their own submission order and take precedence
over ordinary follow-ups. Do not reorder ordinary inputs as a steering side effect.

A safe boundary is after the current provider generation/tool operation has
settled and before dispatch of the next model request or not-yet-started tool.
Apply steering as a new user instruction alongside the retained original request,
not as a replacement objective. Already dispatched effects and output remain.
After a pending approval resolves, recheck Stop, steering and the reviewed target
before starting its operation. A correction that changes a target invalidates that
review and requires a fresh one. Providers need no in-flight injection capability.
Late steering after run completion becomes the next user turn with its visible
Steer mode retained. Stop wins races with both paths.

Stop marks the task suspended and cancels its run, retaining pending input and
completed effects. Explicit Resume clears suspension after inspection and only
dispatches safe pending input. Reopen also requires Resume. Clearing a goal,
asking a side question, deleting pending input and cancelling a run are separate
actions. No queue operation grants tools, copies approval tokens or changes a
goal without explicit intent.

## Bridge and UX contract

EH01 operations: list, enqueue, edit, reorder, cancel and resume with conversation,
task revision/input revision and input UUID. Mutations return the saved record
and current task revision. Poll includes task state, pending counts and receipt
changes. Stale/failed saves return actionable errors without consuming the draft
or attachments. Dart may display optimistic editing but never Applied before the
owner's durable receipt. Bridge names and generated bindings are fixed during
EH01 implementation; events are additive to existing run polling.

Keep Home/navigation and native text editing. Busy composer stays editable;
Queue and Steer now are deliberate actions, with Stop reachable. Enter uses the
visible submit mode; it must not silently switch to Steer. During IME composition,
Enter confirms composition and does not submit/select a command. Escape dismisses
the picker first, preserving text/selection; it does not stop work. Arrow keys only
navigate an open picker when composition is inactive. `/` opens at a command
boundary, not inside paths/code blocks; selecting a skill creates a reviewed
reference chip and an inspectable snapshot. Pending instructions have Edit,
Move up/down and Cancel keyboard/menu alternatives. Compact layouts wrap actions.

Saved static sketches were inspected in all four variants. They are design
evidence only, not rendered Flutter/native acceptance:

- [Light wide](../images/eh00-composer-light-wide.png)
- [Dark wide](../images/eh00-composer-dark-wide.png)
- [Light compact](../images/eh00-composer-light-compact.png)
- [Dark compact](../images/eh00-composer-dark-compact.png)

## Windows command and isolation feasibility

Existing `process::spawn_stdio` already supplies owned stdin/stdout/stderr and
kill-on-close job cleanup. A real Node fixture received Unicode input, remained
alive awaiting input and was stopped with the receipt preserved. Existing command
tests exercise timeout, output flooding and child/grandchild cleanup. Reuse this
for EH05; human terminal tabs are not implicitly shared with the model. Production
session handles, lifetime policy and input review remain EH05 work.

| Candidate | Probe or source result | Decision |
| --- | --- | --- |
| Existing job objects and filtered environment | Process cleanup succeeds; commands intentionally can read sibling paths. | Use for ownership, never claim filesystem/network isolation. |
| AppContainer with no network capabilities and scoped folder ACL | Child token is AppContainer. Granted folder read/write succeeds, sibling read/write denied with error 5. Local loopback connection succeeds unexpectedly; control succeeds too. Profile and owned ACL grants removed. | Filesystem candidate only; reject as a complete network-isolation backend on this host. |
| Experimental CreateProcessInSandbox API | Microsoft labels it experimental and currently disallows inherited handles. | Do not select a volatile API for the existing stdio protocol without a separate compatibility design/probe. |
| Windows Sandbox | Requires supported Windows edition/virtualization and a separate guest environment. Not installed/enabled by this batch. | Potential stronger alternative; no product compatibility or cleanup claim. |

Revised EH05 design: use existing process ownership plus an explicit isolation
capability check; pursue AppContainer/LPAC with a verified network enforcement path
or a guest-backed runner. Freeze the backend only after filesystem **and** network
negative controls pass. Until then requested isolation fails closed. H17 stays open;
neither a UI permission label nor a successful container token is sufficient.
EH01–EH04 may proceed because they do not promise OS isolation. No admin setup,
firewall change, loopback exemption or feature installation was performed here.

The probe uses a static-CRT Rust child with public files and a local listener.
This does not qualify Node/Python/Cargo/Dart, MCP/browser/helper isolation,
Internet/intranet policy, toolchain caches, reparse points or ACL crash recovery.
Initial low-integrity-label setup failed with access denied and was removed from
the probe after confirming AppContainer's granted folder already supports writing.
Failed and subsequent results remain separate local receipts.

Primary references checked 2026-10-21:
[AppContainer launch and capability model](https://learn.microsoft.com/en-us/windows/win32/secauthz/implementing-an-appcontainer),
[AppContainer isolation](https://learn.microsoft.com/en-us/windows/win32/secauthz/appcontainer-isolation),
[experimental sandbox process API](https://learn.microsoft.com/en-us/windows/win32/secauthz/createprocessinsandbox),
[Windows Sandbox prerequisites](https://learn.microsoft.com/en-us/windows/security/application-security/application-isolation/windows-sandbox/windows-sandbox-install).

## Baseline evidence and limits

The existing normal build opened an isolated empty profile with a visible window.
A ten-second idle sample recorded 0.078 CPU seconds (0.78% of one core), approximately
229 MiB working set and 237 MiB private bytes; no desktop helpers were observed.
Concurrent tests/build work and uncontrolled focus make this a reference sample,
not a performance acceptance threshold. Launch-to-visible timing, active-delivery
resources, native IME and low-end machines remain unmeasured. The sampler's generic
resource label mentions a Python C ABI host; this run's target was the recorded
Flutter executable, as verified by `measure-desktop.py` identity checks.

Existing source/fixture checks establish admission cancellation and durable draft
reopen, not a new native runtime journey. Keep the October priority-audit deferrals
in [ACCEPTANCE.md](../ACCEPTANCE.md). Queue/steer UI, durable migration and actual
dispatcher integration remain EH01; goals/plans remain their owning batches.
