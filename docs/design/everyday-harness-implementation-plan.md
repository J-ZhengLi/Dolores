# Everyday agent harness: umbrella implementation plan

Date: 2026-10-21 (Asia/Shanghai)

Status: implementation authorized 2026-10-21; EH00–EH08 complete; EH09 in progress; EH10 pending.
Executable task lists and current status live in the
[batch index](../plans/everyday-harness/README.md).

Scope: **H01–H20** from the
[gap analysis](../research/agent-harness-gap-analysis.md). **H21: remote task
supervision is deferred by the user** and excluded from every delivery gate.
These H identifiers are independent of existing roadmap milestone numbers;
milestone 21 remains automatic useful memory.

## Outcome and delivery policy

Dolores should support a complete daily workflow: choose context and a skill,
agree on a plan, start work toward a goal, queue more instructions, steer ongoing
work, inspect progress and changes, recover from interruption, and review the
result without reconstructing the task in each message.

Build on the current Flutter/Rust harness. Reuse run admission, durable evidence,
checkpoints, attachments, reviewed skills, scoped settings, Git, terminals,
subagents, scheduling, and memory. No wholesale harness replacement is planned.
Codex supplies interaction references, not a requirement to use an OpenAI-only
backend or reproduce every Codex capability.

Execute **one batch at a time**, with an English commit after each completed
batch, as requested by the user. Before each batch, recheck current source and existing contracts, record
the exact owned files and tests, and resolve its design decisions. Preserve
unrelated work. The user's 2026-10-21 follow-up authorizes implementation in this
order; the plan records scope and gates.

Use EH00–EH10 as batch identifiers without renumbering historical milestones.
EH00 contracts/probes, EH01 live text controls, EH02 command/skill discovery, EH03 goals/context recovery, EH04 planning/context references and EH05 command sessions/isolation are complete. EH06 completes the bounded Windows creation/ownership/transfer/archive/restore lifecycle.
EH07 completes review findings, linked side chats and edited-prompt branching with explicit folder choices.
EH08 completes history search, organization and retained attention with opt-in
Windows notifications under its declared qualification bounds. EH09–EH10 retain
their open gates. Dates and effort
estimates should follow the EH00 baseline rather than precede it.

## Architecture and invariants

| Area | Planned approach and invariant |
| --- | --- |
| Durable state | Store task identity, instruction entries, goal, plan/checklist revisions, selected context, and applied-input receipts through existing storage ports. Keep logical records separate from run snapshots and derived summaries. Exact schema changes belong to brick designs. |
| Run ownership | Keep the existing host admission controller and scope serialization. One conversation has one active primary execution; unrelated roots retain current concurrency. New features must not create an alternate dispatcher. |
| Input | Give every submitted instruction a stable ID, target conversation/task, order, mode, and revision. Track accepted, pending, applied, cancelled, failed, or uncertain delivery explicitly. Acknowledgement is not proof of application. |
| Goal | Persist objective, verification criteria, constraints, lifecycle, and cumulative accounting across runs. Keep completion evidence separate from the assistant's final text. Budget exhaustion is not completion. |
| Plan | Persist the user's accepted plan revision separately from model progress updates. Approval to execute one plan does not approve new tools, unrelated work, or a revised scope. |
| Context | Bind explicit skills and references to reviewed identities and revisions. Preserve source provenance and sharing scope. Summaries cannot replace user instructions or permission checks. |
| Effects | Never replay a command, MCP call, browser action, or approval from historical receipts. Crashes after effect intent retain uncertainty and an inspection path. |
| Platform | Windows x64 is the first supported target. Native commands, notifications, isolation, and process cleanup need Windows evidence; other platforms remain separately qualified. |
| UI | Follow [UI.md](../UI.md), preserving the Home layout, native text editing and IME, compact/light/dark behavior, accessible controls, drafts, and reachable Stop. |

Current integration entry points include
[ChatController](../../apps/dolores_flutter/lib/chat.dart),
[Home](../../apps/dolores_flutter/lib/main.dart),
[run admission](../../crates/dolores-flutter-bridge/src/run_admission.rs),
[agent execution](../../crates/dolores-core/src/agent.rs),
[SQLite storage](../../crates/dolores-store-sqlite/src/lib.rs), and
[skill selection](../../crates/dolores-core/src/relevant_skills.rs).
These are starting points, not a fixed list of files to modify.

### Dispatch and lifecycle rules to freeze in EH00

- Process Stop, cancellation, permission revocation, and shutdown promptly.
  Keep user-question and approval delivery available while input is pending.
- Steering becomes effective at a documented safe boundary before subsequent
  model/tool work. It does not undo output or effects already dispatched.
- Deliver queued follow-ups in their displayed order before automatic goal
  continuation. Preserve host fairness for other conversations and schedules.
- A normal follow-up keeps the active goal unless the user explicitly replaces
  or clears it. Side questions must not silently become replacement objectives.
- Stop suspends automatic dispatch for that task, including its goal and queued
  work. Retain pending input; explicit resume determines whether work proceeds.
- Reopening after interruption restores inspectable state and offers resume;
  it does not silently resend input or repeat effects with uncertain outcomes.
- Goal continuation initially runs while the normal app owner is available.
  Closing behavior continues to follow existing ordinary-run/background-task
  contracts. Background goal execution is not implied by background scheduling.

## Sequence and feature coverage

Each H feature has one owning batch. Other batches may establish prerequisites
or exercise it without claiming a second implementation of that feature.

| Batch | Deliverable | Owned gap IDs | Dependencies | Batch exit |
| --- | --- | --- | --- | --- |
| EH00 | Contracts, baseline, and feasibility | Shared prerequisites | None | State transitions, dispatch ordering, migrations, UX sketches, and feasibility decisions are documented and testable. |
| EH01 | Live instruction control | H01, H02 | EH00 | Queue and steer work in a normal chat with durable delivery and interruption recovery. |
| EH02 | Discoverable commands and skills | H04 | EH01 | `/` selects the intended command or reviewed skill without breaking ordinary text or IME. |
| EH03 | Durable goals and progress | H03, H07, H16 | EH01, EH02 | Goal, checklist, and pending input survive continuation, compaction, and restart with honest stopping states. |
| EH04 | Planning and explicit context | H05, H06 | EH02, EH03 | A retained plan can move into execution with inspectable, bounded context references. |
| EH05 | Command sessions and execution isolation | H08, H17 | EH00, EH03 | Agent-owned sessions have controlled lifetime and output; the selected Windows isolation mode proves its advertised boundary. |
| EH06 | Managed task worktrees | H09 | EH03, EH05 | Isolated task work survives archive/restore and never overwrites unrelated edits during transfer. |
| EH07 | Review and conversation branches | H10, H11, H14 | EH04, EH06 | Review findings, side conversations, and edited-prompt branches preserve provenance and original work. |
| EH08 | History and attention | H12, H13 | EH03, EH07 | Search and organization retain data, and notifications open the current relevant task or decision. |
| EH09 | Reusable workflows and supervised delegation | H15, H18, H19, H20 | EH04, EH05, EH08 | Skills, connectors, hooks, and child agents share explicit authority, lifecycle, and recovery rules. |
| EH10 | Integrated daily-use qualification | Cross-feature acceptance | EH01–EH09 | Complete the declared Windows journeys and report remaining model/platform/resource gaps. |

The first usable slice is EH01–EH04. EH05–EH09 broaden sustained engineering
workflows; EH10 is the umbrella exit. Do not call the entire plan complete when
only the first slice is delivered.

## EH00: contracts, baseline, and feasibility

**Bricks**

1. Audit source against the gap findings and current contracts. Capture baseline
   draft/history/settings behavior, admission limits, cancellation, idle CPU and
   memory, startup, and active delivery. Preserve existing acceptance gaps.
2. Specify the minimal durable task/input/goal/plan records, state transition
   tables, revision checks, retention/deletion, dispatch arbitration, and bridge
   events. Define Stop, reopen, uncertain delivery, and same-root contention.
3. Save and inspect compact/light/dark composer sketches for queue/steer, `/`,
   plan, goal, and progress. Resolve interaction and IME rules without replacing
   the established workspace navigation.
4. Run bounded feasibility probes for command-session ownership and Windows
   filesystem/network isolation. Compare candidate backends against explicit
   denied-access and cleanup cases before selecting one. Record unsupported
   executable/tool combinations and required installation behavior.

**Exit:** freeze contracts and a small public synthetic task corpus; specify
numeric storage, queue, process, output, and context bounds using baseline
evidence. Do not change defaults merely to make the corpus pass. A failed
isolation probe leaves H17 open and leads to a revised implementation design;
it does not remove H17 from scope or substitute application policy for isolation.

## EH01: instruction queue and steering

**Bricks**

1. Add durable instruction entries and enqueue/edit/reorder/cancel operations,
   with stable identities, optimistic revisions, and additive migrations.
2. Keep the composer editable during work; separate **Queue**, **Steer now**,
   and **Stop**. Display pending instructions and save errors without consuming
   the current draft or attachments.
3. Integrate queue dispatch and steering into the existing owner. Use a
   provider-independent safe model/tool boundary first; native provider steering
   can be an optional adapter capability. Do not require every provider to
   support injection into an in-flight generation.
4. Persist delivery receipts and restore pending/uncertain entries after restart.
   Define queued input handling on Stop, permission review, session deletion,
   schedule admission, and a run finishing while the user submits new guidance.

**Exit:** queue two follow-ups, edit their order, and apply a steering correction
without losing the original task. Test crash between acceptance and application,
and steer during a running command or pending review. Completed effects remain;
the UI distinguishes pending from applied and offers explicit recovery.

## EH02: command and skill discovery

**Bricks**

1. Add a local command registry and filtered `/` picker with accessible keyboard
   selection. Recognized commands are host operations, not arbitrary prompt
   text interpreted as privileged actions.
2. Represent explicit skill selections as structured references to reviewed
   snapshots. Show scope and version; retain existing implicit name/keyword
   matching for ordinary requests.
3. Connect the picker to queue/steer and context preview. Add goal and plan
   commands when their handlers ship; unavailable handlers must be clearly
   unavailable rather than silently treated as implemented.

**Exit:** choose a skill and verify the exact selected snapshot reaches the
request. Test stale/disabled selection and a failed save while preserving the
draft; test slash-containing paths, code blocks, and IME composition.

## EH03: goals, checklist, and long-context recovery

**Bricks**

1. Persist a task-scoped goal with objective, success criteria, constraints,
   lifecycle, cumulative budgets, and run ancestry. Add set/view/pause/resume/
   clear controls. Do not infer authorization to create a goal from every prompt.
2. Add bounded automatic continuation through the existing dispatcher only when
   idle, with no pending user input or decision. Account for retries,
   compaction, children, and continuation segments; preserve configured limits.
3. Add a revisioned checklist and progress panel. Link completed steps and goal
   completion to concrete evidence; show blocked, paused, interrupted, and
   budget-limited outcomes without presenting them as success.
4. Preserve goals, accepted decisions, checklist, and pending input independently
   of summaries. Implement a bounded mid-task context recovery path that keeps
   completed tool results, refuses malformed summaries, and does not replay tools.
5. Restore inspectable task state on reopen and add explicit resume. Prevent
   no-progress continuation loops using bounded detection and actionable stops.
6. Add calm, host-provided context-capacity awareness with byte/token estimates,
   output reserve and separate cumulative task limits. Ship it with a usable
   safe-boundary recovery path; model-dependent premature stopping remains a
   qualification concern, not an assumption that prompting alone prevents it.

**Exit:** complete a multi-turn fixture goal with a verifiable artifact. Test
budget/context exhaustion and repeated failures, plus Stop/reopen with queued
input. A truncated or malformed provider response retains useful work and cannot
mark the goal complete. Real-model reliability remains a separate gate.

## EH04: plan mode and context references

**Bricks**

1. Add a dedicated planning state with a versioned plan and user decisions. Limit
   planning to an explicit inspection tool set; generic commands are not made
   read-only by their labels. Transition to execution on the user's concrete
   action, using existing access rules.
2. Add an `@` picker and structured references for files/folders, selected code,
   terminal excerpts, messages, and conversations. Preview sharing and size;
   snapshot or bind revisions so dispatch matches what the user selected.
3. Resolve references through existing attachments/context ports. Bound directory
   expansion and cross-chat excerpts, honor source scope, and expose omissions.
4. Connect accepted plan revisions to the goal/checklist. New plans or references
   cannot silently broaden grants or supersede direct user instructions.

**Exit:** plan a small change, select context, and explicitly start execution.
Test stale/deleted references and over-budget context; recover by reselection or
narrowing with the draft intact. Reopening a plan never starts edits by itself.

## EH05: agent command sessions and enforced isolation

**Bricks**

1. Introduce supervised command-session handles with read/poll, input, interrupt,
   and close operations. Reuse process/terminal ownership where appropriate
   without exposing all human terminal tabs automatically to the model.
2. Add bounded incremental output, deadlines, process counts, stdin review, and
   run/task-linked receipts. Define which sessions may outlive a turn and keep
   their owner, Stop, and exit state visible.
3. Implement the EH00-selected Windows isolation backend and explicit filesystem/
   network grants. Route command and supported external-tool launches through
   it; reject or clearly label unsupported execution paths. Optional isolation
   does not silently fall back to unrestricted execution when requested.
4. Add scoped escalation and crash/quit cleanup. Lost handles require inspection
   and a new explicit command, not an assumed reattach or repeated execution.

**Exit:** run a bounded build/server fixture, inspect output, send input, and
stop its owned process tree. Test output flooding and interrupted/lost sessions.
Prove actual denied filesystem/network access for the advertised isolated mode,
including a failed isolation setup; UI policy checks alone do not pass H17.

## EH06: managed task worktrees

**Bricks**

1. Add managed worktree creation and chat association with an explicit base ref,
   root identity, setup policy, and owned lifecycle metadata.
2. Bind run/file/Git/command ownership to the actual checkout; serialize conflicting
   repository actions and preserve the original checkout and dirty files.
3. Add compare and reviewed transfer of changes. Handle conflicts without
   resetting either side; show the source/destination and retained work.
4. Archive with a recoverable snapshot and restore flow. Verify absolute owned
   paths before removal; handle missing roots, setup failure, and disk exhaustion.

**Exit:** run independent tasks in separate worktrees, transfer a selected change,
archive and restore it. Test conflicting dirty work and failed setup/restore;
preserve both versions and provide manual recovery details.

## EH07: review, side conversations, and edited prompts

**Bricks**

1. Bind inline review findings to file/diff revision, side, and location. Submit
   selected findings as ordinary queued or steering input and track resolution
   against evidence from a later revision.
2. Add linked side conversations from selected text with bounded parent excerpts,
   return navigation, and an explicit action to bring a conclusion back.
3. Add edit-and-rerun as a new conversation branch. Preserve original messages,
   ancestry, and external-effect evidence; do not imply file rollback or copy
   active approvals, goals, or pending queue authority automatically.
4. Offer shared-folder versus managed-worktree behavior explicitly where relevant.
   Rebase or mark comments stale when their referenced diff changes.

**Exit:** review a change, ask a linked side question, and submit chosen findings
for repair. Test stale diff locations and editing an old prompt after files have
changed. The original conversation/files remain usable; branching never replays
historical tool effects.

## EH08: history and attention

**Bricks**

1. Add full-text history search, filters, pinning, archive/unarchive, and quick
   navigation. Preserve existing deletion semantics and pending task ownership;
   archiving is not deletion or implicit permission to cancel a goal.
2. Add opt-in Windows notifications for completion, failure, approval, or a
   question, with minimal private content, deduplication, and exact task targets.
3. Resolve notification clicks against current state. A stale notification opens
   evidence instead of acting on an expired approval. Notification delivery
   failure leaves the same decision reachable inside Dolores.

**Exit:** find and restore an archived conversation, then navigate from a live
attention notification. Test notification denial/stale targets and search index
rebuild/interrupted updates. Preserve history, drafts, and pending input.

## EH09: workflows, connectors, hooks, and subagents

**Bricks**

1. **H18:** import/install/export complete skill packages with resources, paths,
   provenance, versions, dependency inspection, and rollback. Reject escaping
   paths and retain the previous usable version on partial install. Installation
   does not execute scripts or approve their tools.
2. **H19:** add remote MCP transport/authentication, connection health, and bounded
   tool discovery. Keep secrets in the credential system, pin the run's reviewed
   catalog, and recover explicitly from expired credentials or server failure.
   This is outbound tool connectivity, not remote control of Dolores.
3. **H20:** expose reviewed lifecycle hooks through host-owned events. Separate
   observational hooks from blocking checks. Define timeout/malformed-result
   behavior: failed blocking checks refuse the affected action with recovery;
   observational failures surface without silently changing task authority.
   Bound output, depth, recursion, and context contribution.
4. **H15:** add child-specific follow-up, context inspection, interruption, and
   optional model selection. Freeze child scope/permissions/model snapshots,
   retain parent cancellation and shared budgets, and verify child reports.
5. Integrate package resources, discovered tools, hooks, and delegation with
   goal/queue/context ownership. Keep command/MCP access unavailable to children
   unless separately designed, supported, and qualified; richer supervision
   alone does not expand child capabilities.

**Exit:** install a skill with resources, invoke a connected fixture tool under
its workflow, supervise a child, and observe a bounded hook. Test package update
failure, expired connector auth, hook timeout, and parent cancellation while a
child is pending. Keep earlier versions and completed results inspectable.

## EH10: integrated qualification and release readiness

Freeze observable criteria before running these journeys. Use public synthetic
projects and isolated data; preserve the user's provider settings and history.

| Journey | Required observable outcome |
| --- | --- |
| Daily task control | Plan a small feature, select a skill/context, execute toward a goal, queue follow-ups, steer scope, and finish with verified files/tests. |
| Long task recovery | Reach context/output/tool limits, preserve effects and pending input, reopen, inspect, resume, and complete without duplicate operations. |
| Engineering workflow | Run a supervised command, isolate a task in a worktree, review inline, repair selected findings, and retain recoverable archived work. |
| Attention and branching | Find prior work, ask a linked side question, edit an earlier prompt into a new branch, and open a current decision from a notification. |
| Extensibility and delegation | Use a complete skill package, connector, bounded hook, and supervised child while retaining scope, budgets, and cancellation. |
| Preservation | Migrate/reopen an older profile; preserve chats, drafts, settings, skills, memories, scheduled tasks, and unrelated filesystem edits. |

For each implemented brick, run focused deterministic tests plus one or two
realistic failure/recovery cases where applicable. Use a normal Windows build
when compilation/packaging changes require it. Use saved renders inspected with
`view_image` for affected UX; perform native interaction checks when keyboard,
IME, process, window, notification, or OS boundaries need them. Use
`python scripts/desktop.py build` and `python scripts/desktop.py launch` when
needed, following contributor/release guidance.

Optional live checks explicitly select a suitable available model, bounded
prompts and usage, and isolated output. Record fixture, rendered/native,
live-model, resource, and platform evidence separately in
[ACCEPTANCE.md](../ACCEPTANCE.md). Fixtures do not prove general model reliability.

**Umbrella exit:** all H01–H20 have implementation and the declared acceptance
evidence, or a clearly recorded unresolved gate. An unresolved required gate
keeps the umbrella incomplete; it is not a silent scope reduction. H21 does not
block completion because it is explicitly deferred.

## Change tracking and next action

Every brick records its design decisions, files, basic flow, failure/recovery
checks, evidence paths, remaining gaps, and English commit. Update this plan's
batch statuses and acceptance records as actual work progresses. Preserve old
evidence as historical rather than rewriting it into a pass.

Continue from the [batch task index](../plans/everyday-harness/README.md).
EH00 owns the [frozen contracts](everyday-harness-contracts.md), source audit,
credential-free state-machine corpus, UX sketches and bounded Windows probes.
Keep its diagnostic evidence separate from user-visible EH01–EH09 acceptance.
