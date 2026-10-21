# Everyday agent harness gaps: Dolores compared with Codex

Date: 2026-10-21 (Asia/Shanghai)

Status: findings based on a source and documentation comparison. On 2026-10-21,
the user deferred **H21: remote task supervision** and requested an umbrella
implementation plan for **H01–H20**. See the
[implementation plan](../design/everyday-harness-implementation-plan.md) for
delivery batches, dependencies, and acceptance gates. This findings document
does not establish runtime acceptance.

## Finding

Dolores has many underlying tools, but lacks several controls needed to direct,
organize, and review ongoing agent work. The largest gaps are instruction
queueing, steering an active conversation, a persistent goal lifecycle, and
discoverable command and skill invocation.

The next development focus should connect these controls to the existing run,
history, context, permission, and recovery machinery. Existing features should
be extended where they already provide part of the required behavior.

## Scope and evidence

The comparison inspected the current Flutter/Rust source, user guide,
architecture, and acceptance records, together with official OpenAI documentation.
No app launch, live-provider check, or runtime modification was performed for
this analysis. An implementation observed in source is not a claim of dependable
real-model behavior; existing qualification gaps remain recorded in
[ACCEPTANCE.md](../ACCEPTANCE.md).

Important findings from the current implementation:

- The [Home composer](../../apps/dolores_flutter/lib/main.dart) becomes read-only
  while a run is busy. The [chat controller](../../apps/dolores_flutter/lib/chat.dart)
  also refuses a new send while busy. Queueing and steering need runtime support
  as well as composer changes.
- [Skill selection](../../crates/dolores-core/src/relevant_skills.rs) already
  matches activated skills by name and description keywords. Mentioning a skill
  name explicitly selects its reviewed snapshot. The token matching also
  recognizes the name in `/skill-name`; this is ordinary text matching, without
  a command picker or structured invocation.
- The current [run admission controller](../../crates/dolores-flutter-bridge/src/run_admission.rs)
  permits two active runs across separate roots and four waiting runs. This is a
  host run queue, not a queue of follow-up instructions in one conversation.
  Parallel execution is therefore not wholly missing.
- [Conversation forks](../design/managed-threads.md) already exist but share the
  original working folder. A conversation fork does not isolate file changes.
- [Task checkpoints](../design/task-checkpoints.md),
  [continuation](../design/long-task-recovery.md), summaries, and preflight
  compaction already exist. These provide useful foundations for durable goals.

Some descriptive documents retain earlier limits. For this comparison, the
current source takes precedence for the run admission and skill matching findings
above. Any implementation plan should recheck current contracts before editing.

## P0: establish everyday task control

Codex distinguishes Queue, which sends a follow-up after the current turn, from
Steer, which directs work already in progress. Dolores should make both actions
explicit rather than treating every follow-up as a stop and restart.
[Official reference](https://developers.openai.com/blog/mastering-codex-remote-for-engineering).

| ID | Feature | Current Dolores behavior | Proposed addition |
| --- | --- | --- | --- |
| H01 | Instruction queue | No same-chat follow-up queue while running. Existing host admission queue has a different purpose. | Keep the composer editable; queue multiple instructions; inspect, edit, reorder, cancel, or send one immediately. Persist pending instructions. |
| H02 | Steer conversation | Stop and send another message is the available workaround. | Add **Steer now** at a safe execution boundary, with pending/applied status. Preserve the original objective unless the user explicitly replaces it. |
| H03 | Goal mode | Automatic tool cycles and bounded continuation exist, without a durable goal lifecycle. | Persist the objective, completion criteria, lifecycle, progress, and budget accounting. Support pause/resume/clear and evidence-based completion across turns. |
| H04 | `/` command and skill picker | Skill names select activated snapshots; picker and autocomplete are missing. | Searchable commands and skills, keyboard navigation, scope/description preview, and a visible selected-skill chip. Bind explicit selection to the intended reviewed version. |
| H05 | Plan mode | Planning can be requested in prose; there is no dedicated mode. | Inspect and propose a plan, collect decisions, retain the approved plan, and explicitly switch into execution. |
| H06 | `@` context references | File and image attachments exist. | Search and reference project files, folders, selected code, terminal output, prior messages, and other chats. Preview the content that will be shared. |
| H07 | Persistent task checklist | Activity and run history expose execution evidence. | Show pending/in-progress/completed steps, the current blocker, verification results, and the next action. Preserve this state across continuation and compaction. |
| H08 | Agent-owned command sessions | The agent command tool is bounded and noninteractive; the separate terminal UI supports sessions. | Let the agent start a command, receive a session ID, inspect incremental output, send input, and stop it. Support builds, servers, and interactive development workflows. |

Goals should persist as thread state and continue only within explicit bounds;
completion should depend on evidence rather than a model's assertion. This
extends Dolores's continuation machinery instead of simply increasing tool
budgets. [Codex Goals reference](https://developers.openai.com/cookbook/examples/codex/using_goals_in_codex).

Codex documents a filtered `/` command menu that also includes enabled skills.
Dolores can adopt this interaction while preserving its reviewed skill snapshots
and permission rules. Invocation syntax varies between clients; this proposal
specifically chooses `/` as the Dolores discovery surface.
[Slash command reference](https://learn.chatgpt.com/docs/reference/slash-commands).

## P1: support sustained work and review

| ID | Feature | Existing foundation | Missing capability |
| --- | --- | --- | --- |
| H09 | Managed worktrees per task | Projects, Git views, and conversation forks. | Create isolated task checkouts; associate them with chats; compare, transfer, archive, and restore their work. |
| H10 | Review-to-agent feedback loop | Changes and Git diffs. | Select diff lines, attach comments, submit selected feedback to the agent, and track whether each finding was addressed. |
| H11 | Linked side conversations | Side chats and forks. | Open **Ask about this** from selected text with parent context and a return link. Explicitly bring useful conclusions into the main task. |
| H12 | Completion and attention notifications | Background scheduling and saved results. | Notify when work finishes, fails, needs approval, or asks a question. Open the exact pending decision from the notification. |
| H13 | Searchable, organized history | Project grouping and paginated conversation history. | Full-text search, pinning, archive/unarchive, filters, and quick switching between active tasks. |
| H14 | Edit and rerun earlier prompts | Draft recovery, retry, and forks. | Edit an earlier instruction and create a clearly identified alternative continuation while preserving the original conversation. |
| H15 | Richer subagent supervision | Bounded delegation. | Follow up with a particular child, inspect its context, interrupt it independently, and optionally select its model. Retain shared budgets and explicit ownership. |
| H16 | Recovery during long conversations | Summaries, preflight compaction, checkpoints, and Continue. | Preserve goals, pending instructions, decisions, and task state through compaction. Recover from mid-task context exhaustion without manually reconstructing the task. |
| H17 | Enforced execution isolation | File capabilities and Review/Auto/Full access. | Optional OS-enforced filesystem and network boundaries for commands and external tools, with scoped escalation where needed. |

Codex's managed worktrees provide a reference for task isolation and retained-work
recovery. Dolores's existing Git support should be extended with an explicit
worktree lifecycle. [Worktrees reference](https://learn.chatgpt.com/docs/environments/git-worktrees).

Codex also demonstrates linked side chats and inline feedback sent back to an
agent. Those interactions are useful models for H10 and H11.
[Workflow reference](https://developers.openai.com/blog/mastering-codex-remote-for-engineering).

H17 is an execution-boundary addition. Dolores's current tool review and access
modes already provide application policy, while commands and MCP processes run
with the user's account permissions. See [task permissions](../design/task-permissions.md)
and [privacy](../PRIVACY.md).

## P2: expand extensibility

| ID | Feature | Proposed addition |
| --- | --- | --- |
| H18 | Complete skill package management | Import, install, update, and export a skill together with scripts, references, and assets. Inspect dependencies and version changes. |
| H19 | Broader connector management | Remote MCP connections, authentication flows, connection health, and searchable discovery of relevant tools as the catalog grows. |
| H20 | User-configurable lifecycle hooks | Reviewed hooks for session start, tool execution, compaction, and task completion, supporting project setup, checks, and integrations. |

Skills can package instructions and supporting resources; this is broader than
managing a retained `SKILL.md` snapshot alone.
[Skills reference](https://learn.chatgpt.com/docs/build-skills).
Codex lifecycle hooks provide a reference for H20.
[Hooks reference](https://learn.chatgpt.com/docs/hooks).

## Deferred

| ID | Feature | Disposition |
| --- | --- | --- |
| H21 | Remote task supervision | Deferred by the user on 2026-10-21. Cross-device progress, queueing, steering, questions, and approvals are outside the current implementation plan. Revisit only when the user brings this scope back. |

H21 is a gap identifier, not roadmap milestone 21 (automatic useful memory).
Remote MCP connections in H19 remain in scope; they do not require remote control
of Dolores.

## Recommended implementation order

1. **H01: instruction queue.** Establish durable pending input and delivery rules.
2. **H02: steering.** Apply new instructions to active work at safe boundaries.
3. **H04: command and skill picker.** Make invocation explicit and discoverable.
4. **H03: goal mode.** Add bounded continuation that respects queued and steering
   input, pending decisions, and the existing run owner.
5. **H05: plan mode.** Retain decisions and make the plan-to-execution transition
   explicit.
6. **H06: context references.** Make the working context easy to select and inspect.

H07 and H16 should connect to the same durable task state rather than maintaining
separate copies of goals, progress, and pending instructions. H08 can be planned
as a separate execution brick. H09 becomes especially valuable as users run more
independent coding tasks in parallel.

This section preserves the initial recommendation. The
[umbrella implementation plan](../design/everyday-harness-implementation-plan.md)
now defines the complete sequence for H01–H20. Existing milestone numbers in
[ROADMAP.md](../ROADMAP.md) remain unchanged.

## Proposed acceptance journeys

These are future acceptance criteria, not exercised results.

| Journey | Basic flow | Realistic failure and recovery |
| --- | --- | --- |
| Queue | While a task runs, queue two instructions, edit their order, and deliver them in the displayed order. | Reopen the app with pending instructions. Preserve their identity and state; show what needs resumption without duplicating delivered work. |
| Steer | Redirect an active task and show when the instruction becomes effective. | Steer while a command is running. Preserve the command's actual outcome, apply guidance before subsequent actions, and avoid replaying completed effects. |
| Skill invocation | Select a skill through `/`, inspect its scope/version, and confirm that the intended snapshot reaches context. | A selected skill is disabled or changes before dispatch. Explain the stale selection and offer reselection while retaining the draft. Ordinary slash-containing text remains usable. |
| Goal | Define an objective and verification criteria, then continue across turns until the evidence supports completion. | Exhaust a budget or encounter repeated failures. Retain progress, show the actual stopping reason, and provide explicit recovery without marking the goal complete. |
| Plan to execution | Inspect, revise, and accept a plan before starting implementation. | Interrupt or reopen during planning. Preserve decisions and avoid starting edits merely because the app resumes. |
| Combined daily workflow | Start a task, queue follow-up work, steer its direction, invoke a skill, reach a checkpoint, reopen Dolores, and continue with the goal and pending work intact. | Preserve usable files and receipts through interruption; distinguish completed actions, pending input, and uncertain outcomes. |

For implemented bricks, add focused tests and realistic recovery cases under the
repository guidance, then record exercised behavior and remaining gaps in
[ACCEPTANCE.md](../ACCEPTANCE.md). A fixture pass alone does not establish
real-model reliability.
