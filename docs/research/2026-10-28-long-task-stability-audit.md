# Long-task stability and execution-limit audit

This follows the [token audit](2026-10-27-token-consumption-audit.md) and
[pinned upstream source review](2026-10-28-long-task-harness-research.md).
Private transcript bodies and credentials are excluded from this report.

## Observed failure

The supplied `99696da3-6982-48dc-a8ba-278e187123c5` export records one
`kimi-k3.1-pro` run: 17 generation attempts, 14 reported successes and three
attempts with unknown usage. Known usage is 203,590 tokens: 197,999 input and
5,591 output; 146,688 cached input tokens are a subset of input, not an extra
chargeable total. These counters do not establish the three failed attempts'
bill or permit a controlled comparison with DeepSeek.

The first failing stream ended after 30.244 seconds, with reasoning activity
at 30.084 seconds, no completed tool call and no finish marker. The host then
selected complete-response mode twice; the saved final error reports timeout,
with a configured request deadline of 180 seconds. The run's recorded duration
is about ten minutes. The export cannot establish why the upstream connection
ended, whether a provider gateway imposed a deadline, or whether the Flutter
thread deadlocked. Those remain distinct from the observable silent wait and
poor history presentation.

There are 41 accepted tool-result events. An execution checkpoint after 12 model
calls archived 33 tools. That checkpoint is local evidence segmentation, not
proof that model-context summarization occurred. The earlier receipts remain in
the export. Its UI entry previously exposed raw event/JSON fragments; the model
had no general tool for retrieving an archived original receipt after projection
or compaction.

## Prioritized repairs

| Priority | Confirmed seam | Implemented change |
| --- | --- | --- |
| P0 | Default 8 KiB command capture killed verbose commands; supervised sessions did the same | Capture bounds inline memory. Excess stdout/stderr spool to workspace files while the process runs. Zero exit, cleanup, missing output and shortened preview remain separate. A command with complete saved output does not fail solely because its preview is short. |
| P0 | Repeated checkpoint snapshots retained all large historic bodies, eventually exceeding checkpoint storage before context pressure | Release older successful working bodies only after the complete segment is durably sealed. Full accepted receipts remain local. An 800-step roomy-window regression reproduces the original failure and passes the repair. |
| P1 | A failed complete-response fallback remained selected for another silent retry | The remaining retry returns to streaming. A successful complete-response route remains selected to avoid repeated failed-stream charges. The UI exposes that wait, elapsed time and deadline. Saved attempt records include transport, duration and outcome. |
| P1 | “Earlier progress” hid ordinary outputs inside machine-formatted fragments; optional checkpoint failure could hide fetched evidence | Reconstruct readable segment timelines, retain paging errors without dropping the last page, and keep technical events separate. Opening history sends no model request. |
| P1 | The model could not recover original archived output | Add deferred `read_task_history`: owning-chat scope, receipt-reference listing and UTF-8 byte pages by call ID. Projection and restored-context notices point to retrieval rather than repeated effects. |
| P1 | Output above 16 KiB could be discarded and abort a run after execution | Save complete output fragments before returning a compact retrieval-linked preview. Original output survives reopening SQLite and is retrievable without reexecution. |
| P1 | File creation and review shared a 16 KiB result-preview cap; a long source line was unreadable | Separate request, review and preview allocations. Creation uses the existing 64 KiB argument allowance; complete review diffs allow 256 KiB. Large reads page automatically; byte cursors reconstruct long Unicode lines exactly. Snapshot/stale-file checks remain. |
| P1 | Five-minute command-session ceiling and one-second paid polling | Explicit reviewed sessions permit six hours; output reads wait up to 30 seconds and remain cancelable. Defaults and run-owned cleanup remain. |
| P1 | Context overflow below the local estimate ended a productive run | One compaction before a strictly smaller retry; repeated rejection or no reduction saves a checkpoint. Completed effects do not replay, and missing usage stays unknown. |
| P2 | Registration count was tied to an obsolete 24-tool inventory | Separate the allocation guard from inventory count. Deferred schemas and actual request capacity govern provider payloads. |
| P2 | Eager instruction scanning blocked large/deep repositories and loaded unrelated guidance | Read only applicable ancestors on demand, require current guidance before a file write, and retain exact scoped instructions through context maintenance. |
| P2 | Initial history still stopped at 40 turns/128 KiB independently of model capacity | Select complete turns against the configured input allowance, followed by the actual token estimate. A separate 32-MiB allocation guard remains. |
| P2 | Child allowances and a separate two-follow-up cutoff prevented sustained scoped implementation | Use cumulative 32-call/64-file child allowances within shared parent limits; follow-ups keep identity, scope and usage. |

Compaction is already mandatory in normal saved chats: the persistent store
ignores legacy disabled values, rejects disabling, and the UI has no opt-out.
The earlier suggestion that users could disable it was too broad. Local evidence
segmentation is now also independent of whether a summary provider is attached.

## Validation boundaries

Credential-free checks cover 800 tool steps, repeated maintenance, exact saved
receipts, context rejection, stalled fallback recovery, zero-exit verbose
commands above 512 KiB, Unicode paging, stale writes, Stop and owned cleanup.
Flutter checks cover visible silent waits, automatic terminal delivery without
Stop, readable history and page failures in compact/wide Light/Dark views.
The native soak helper combines file creation, original-receipt retrieval after
compaction, injected provider faults, draft preservation and a real supervised
child. Usage from scripted providers is synthetic.

The final short native fixture completed 160 tool steps, 40 verified files and
172 scripted provider requests in about 25 seconds, including a ten-second real
child, context rejection, a stalled buffered retry, original-receipt retrieval,
applicable nested guidance and full oversized command output. Its two-hour gate
is explicitly false. The separate 800-step roomy-context regression passed.

The token fixture's built-in baseline estimates 738 system tokens and 3,225 tool
schema tokens with nine initial tools. Loading two tools increases the catalogue
to eleven; unloading restores nine. On its fixed 39-request trace, projected tool
bodies total 2,899,245 bytes against 8,935,265 bytes without projection, a 67.55%
reduction in those bytes. These estimates and synthetic usage do not establish
an equivalent reduction in billed tokens or a cross-harness performance ratio.

`scripts/test-long-task-soak.py --seconds 7200` is the separate two-hour
wall-clock gate. A short fixture does not establish that gate. No paid/live
Kimi or DeepSeek run is implied by these checks; provider-specific feature
delivery and clean-machine acceptance remain unverified.

## Further limits identified

The remaining audit repairs replace eager nested-instruction scanning with
ancestor-only reads and exact scoped guidance retained through maintenance.
Unrelated directory count and depth no longer block context preparation.
Initial saved history uses the model allowance rather than a fixed 40-turn/
128-KiB cutoff, with a separate 32-MiB allocation guard. Reads above 1 MiB stream
and hash incrementally, including byte continuation through long UTF-8 lines;
the 1 MiB atomic editing snapshot remains a separate allocation. Explicit scoped
discovery can inspect source directories named `build` or `target`.

Child agents use cumulative allowances up to 32 model calls/64 file operations,
bounded by shared parent limits, instead of four/four and two follow-up messages.
They retain ownership, no-recursion and parent-verification boundaries. This is
not a claim that every finite allocation can or should be removed. Actual
provider capacity, explicit user budgets, cancellation and failed-persistence
boundaries remain authoritative. The user explicitly excluded an actual
two-hour soak; that gate remains unrun.
