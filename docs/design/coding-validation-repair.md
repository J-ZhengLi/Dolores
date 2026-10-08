# Coding validation and repair — brick 6.7

The previous loop preserved command output but called every returned command
receipt completed, including nonzero exits. A model could then report success
without an actionable repair path. Keep the existing four-model-call/four-tool
budget and use the actual command evidence to retain unfinished work.

## Evidence and recovery

- A normally completed command with a nonzero integer exit is failed. A timeout,
  unavailable exit, malformed receipt, shortened/unreadable output or lossy UTF-8
  is incomplete. Complete zero-exit evidence is labeled Exited 0; it establishes
  only that invocation's result.
- The model receives updated remaining-call/tool counts in its system context.
  The first note is included in the ordinary context preview, trimming and token
  accounting; later calls replace it rather than accumulating reminders.
  Guidance asks the model to reserve validation capacity and preserve tests.
  This is guidance, not an extra execution budget or guaranteed task planner.
- If a final answer leaves a failed/incomplete invocation unresolved, save a
  commandReview pause even when the model claims success. Existing output/step
  pauses retain their reason and the same receipts. Process errors also need
  review; denied/blocked proposals do not imply executed commands.
- Repair and verify uses the existing latest-message-bound Continue protocol.
  Its visible request and saved prompt identify repair as the user's new intent,
  including after a read-only check request. This permits proposing a repair;
  every read, edit and rerun still requires its own fresh operation review.
  Ordinary resource/output continuation does not imply a repair request.
  It starts a separate run with current model settings, exact saved output and
  fresh per-operation approvals. Completed files and earlier segments remain.
  The repair prompt asks for current-file inspection, implementation repair and
  rerunning the same literal checks without weakening tests merely to pass.
- Match program and every literal argument. A later complete zero-exit receipt
  for that invocation clears its failure; a different successful command or
  a model's prose cannot. The Flutter host reconciles prior and new receipts
  before saving, so restart, no-tool replies and unrelated checks retain failure.
  Older completed-status receipts are interpreted using their actual JSON.
- A commandReview card exposes saved command receipts, including inherited ones
  absent from the latest segment's tool list. Original failures remain in history
  after a successful repair. Draft, Stop, timeout and stale-source behavior stay
  as defined in the long-task recovery design. Paused replies skip learning.

## Boundaries

No dependency, schema migration, resident process, automatic retry, budget
increase or reusable approval is added. Existing 96-KiB/16-receipt recovery and
token/byte guards remain; oversized progress needs a reviewed summary/new chat.
Commands retain the 30-second/8-KiB limits and user-account permissions, and their
effects remain outside the file journal. Arbitrary command meaning cannot be
inferred from exit code. Zero exit does not prove complete project correctness,
and tests can be inadequate or changed by separately approved actions.

This is recovery for attempted checks, not automatic test discovery. Files
written without a command are not retrospectively certified, and a later edit
can invalidate an earlier successful check. Model compliance and exact-edit
line-ending handling remain limitations. The live DeepSeek case required two
explicit continuations after multiline edits failed against CRLF source; Qwen
never reached validation. See ACCEPTANCE.md for precise evidence.
