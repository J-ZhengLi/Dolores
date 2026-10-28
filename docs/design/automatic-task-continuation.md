# Automatic continuation for long tasks

Status: all three implementation batches are delivered. Fixture checks cover
one-start long execution, exact receipt retention, paged evidence, cumulative
caps, maintenance-time steering, Stop and failed saves. Live-model reliability
and clean-machine qualification remain separate from these checks.

## Intended behavior

In Automatic mode, the user starts one task and Dolores keeps working until it
finishes, needs input, encounters a genuine unresolved failure, or is stopped.
Context maintenance and internal accounting windows are invisible execution
boundaries, not reasons to display Continue. The running timer, task ownership,
draft and Stop action persist across these boundaries. A compact status such as
“Updating context” can explain maintenance without ending the response.

Explicit numeric model/tool/time budgets remain authoritative. Host recovery
must never reset their cumulative counters. Automatic does not bypass tool access
policy, retry uncertain operations, re-run completed actions, or resume work after
application restart without the user. Questions, denied access, repeated failures,
unresolved commands/children and provider outages remain distinct pause states.

## Current evidence and remaining seams

The observed context checkpoint followed two successful summaries in the older
run. The two-summary ceiling is removed: compaction now repeats within the same
task budget, and a credential-free 28-tool task exercises more than two summaries
without a Continue. The independent 128-KiB provider transport cutoff is removed;
both transport modes use the model's configured input allowance. Private
reasoning pressure can also trigger compaction before its allocation guard.

The following previous architectural limits are addressed by this implementation:

| Seam | Previous behavior | Implemented behavior |
| --- | --- | --- |
| Automatic execution | Paused at 64 model calls / 128 shared tool attempts | Internal execution windows roll forward automatically |
| Segment count | Four explicit continuations by default | Internal rollover does not consume human-continuation segments |
| Journal | 2,048 events per run plus terminal marker | Bounded journal segments seal under the same run ID |
| Live result state | Public output and receipts accumulated in one task summary | Complete segment results persist before pruning the active projection |
| Goal budget | Automatic was frozen into a numeric cap | New goals preserve Automatic; saved numeric totals remain authoritative |
| Images and recovery | Automatic restoration refused image-bearing context | Literal user attachments and the latest historical tool observation are retained; older tool images remain archived |

Raising default numbers, removing every allocation bound, or auto-clicking the
existing Continue button would move the next failure or change task authority.
The repair belongs in the task supervisor, with durable internal segments.

## Implementation batches

1. **Durable internal segments and paged evidence.** Introduce a host-owned
   execution-segment identity under the existing logical task. Seal its journal
   before the event limit, retain full receipt/text/usage evidence, and expose
   paged segment inspection. Keep existing run IDs and goal evidence references
   valid. Persist ownership and checkpoint atomically; interrupted rollover must
   recover to an inspectable stopped state. No model action follows a failed save.
   Implemented with schema 52's additive segment table. Run IDs and global event
   sequence references stay unchanged. The compatibility event view identifies
   omitted earlier events; `runEventsPage` reads the complete retained evidence
   in pages of at most 128 records / approximately 256 KiB (one whole record is
   always readable). Storage rotation alone does not dispatch work.
2. **Automatic supervisor rollover.** In Automatic mode, rotate the internal
   execution window and compact its bounded active context without completing the
   visible task or synthesizing a user message. Reuse the task's selected model,
   scoped settings, working folder, grants, original objective and corrections.
   Recheck current policy before dispatch. Keep Stop and steering responsive while
   saving or compacting. Do not execute an unfinished call from the old window.
   The host now seals full JSON summary fragments and the next compact context
   atomically before pruning the active projection. Internal windows target 24
   model attempts / 32 active tool receipts or earlier context pressure; these
   are maintenance triggers, not task caps. Automatic root runs with durable
   recovery continue beyond the old 64/128 defaults. Legacy adapters retain
   bounded fallback behavior. Original user instructions and steering are kept
   separately from synthesized checkpoints to avoid recursive prompt growth.
  Completed model summaries accept prose, Markdown and legacy JSON as untrusted
  context; formatting alone cannot end productive work. Empty, interrupted or
  over-budget summary responses still preserve the last safe checkpoint.
3. **Cumulative controls and UI projection.** Share explicit counters and active
   clocks across parent/children and every internal segment. Keep manual recovery
   for real pauses and crashes; internal rollovers do not consume manual segments.
   Keep the conversation transcript independent of the bounded model working set.
   Preserve timer, scroll, composer draft and task identity. Disclose usage and
   maintenance in Run details without generating completion notifications.
   Implemented with an inline maintenance phase and chronological conversation
   output. The controller retains delivered live output; reopened responses load
   stored execution windows locally into the same transcript, deduplicating
   carried tool receipts. Consecutive tools share one collapsed row between
   public outputs, even across silent model calls or storage windows. Failed
   local reads retain loaded output and offer a retry without dispatching work.
   Separate read-only Previous/Next evidence pages remain in Activity for diagnostics.
   Usage details identify their
   recent-work projection, with older exact values in archived summary fragments.

## Multi-hour coding reliability

Sustained coding work must not fail merely because full historic tool output
accumulates in memory or is repeatedly copied into checkpoint snapshots. After
a durable segment is sealed, its full receipts remain in Run history while old
successful working results become bounded previews. Apply this only after a
successful save; failed saves preserve inspectable state and prevent dispatch.
Retain recent results, unresolved failures, exact user constraints and tool
call/result identities through repeated maintenance.

Evidence archiving is independent of model summarization. Execution segments
(normally 24 model attempts or 32 tool receipts, possibly earlier output
pressure) remain a storage detail. They never replace visible conversation
output with an “Earlier progress” button. Reopened conversations reconstruct
original model output and tool records inline; raw diagnostic events remain
optional. A failed recovery-metadata lookup cannot hide readable history.
Reading this local data makes no model request. The agent can load the deferred
`read_task_history` tool, list saved call references and fetch original receipts
in bounded byte pages. Ordinary context does not resend every archived body.

Tool output above the inline envelope is saved as complete UTF-8 fragments before
a retrieval-linked preview is returned. A large returned result is not itself a
reason to abort an executed operation or erase its evidence. Command output
capture similarly bounds inline memory; excess output goes to Dolores-managed log
files outside the working folder while the command continues. Exit/cleanup, output loss, and shortened
previews are distinct facts. Failed disk writes remain explicit.

Long builds use run-owned command sessions with an explicitly reviewed lifetime
of up to six hours (21,600 seconds); the saved 30-second default and short
one-shot command limits remain unchanged. Session reads can wait up to 30 seconds
for useful output with prompt cancellation, instead of requiring paid model
polling every second. Stop and run completion still clean up owned processes;
long lifetimes do not detach a command from its owning task.

A provider context rejection can occur below the local heuristic threshold.
Recovery must preserve effects, compact before retrying, and verify a smaller
request; repeating the rejected envelope or widening authority is not recovery.
Explicit budgets remain cumulative. Failed recovery leaves a saved checkpoint.
Restart remains explicit, without replaying uncertain writes.

Automatic model-context compaction remains always enabled for normal saved chats,
including legacy opt-out records; no UI switch or successful disable command is
offered. A failed complete-response transport recovery returns the remaining
retry to streaming, instead of repeating a silent stalled route. A successful
complete-response route stays selected for that run to avoid repeatedly paying
for a known-broken stream. Its wait phase shows elapsed time and the request
deadline. Saved attempt records include mode, elapsed time and a host-authored
outcome category without private response bodies.

The [upstream source review](../research/2026-10-28-long-task-harness-research.md)
informs these changes. Qualification separates accelerated long-history/fault
fixtures, native integration, wall-clock soak duration and real-model feature
delivery; none alone establishes that arbitrary multi-hour tasks cannot fail.
See the [stability audit](../research/2026-10-28-long-task-stability-audit.md)
for observed failures, removed limits and qualification boundaries.

## Required regression journeys

- A deterministic task completes beyond 64 model calls, 128 tool attempts, two
  compactions and one journal window with exactly one user Start and no Continue.
  Assert every effect executes once and every full receipt remains inspectable.
- Explicit numeric caps pause at the exact cumulative total, including summaries
  and delegated work. Stop during rollover, summary generation, tool review and
  execution prevents the next dispatch and preserves uncertain-effect evidence.
- Crash or failed storage between sealing and starting a segment cannot create
  two owners, auto-resume after restart, repeat an effect or erase a receipt.
- Steering during maintenance is applied once before the next model/tool call;
  queued work retains its original ownership and delivery state.
- A failed/incomplete summary, repeated tool failure, unresolved command or
  provider outage cannot become an endless automatic retry loop.
- Long tool text, images, Unicode instructions and multiple user corrections
  retain their provenance and constraints through repeated maintenance.
- Light/Dark and compact native views keep the task timer and Stop available,
  preserve scroll/draft, and show older evidence through bounded paging.

Fixture checks establish lifecycle and persistence behavior. A subsequent bounded
live-model check should use isolated public data and a named capable model; it
does not substitute for the exact-once and failure tests above.

Related contracts: [task budgets](task-budgets.md),
[context maintenance](token-context.md), [manual recovery](long-task-recovery.md)
and [developer workspace](developer-workspace.md).
