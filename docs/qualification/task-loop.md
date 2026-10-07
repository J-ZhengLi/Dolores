# Automatic task execution — 2026-10-06

## Change and boundaries

Fresh task settings use Automatic model/tool counts instead of four calls/four
operations. Useful work can continue within a 64-model-call / 128-tool resource
checkpoint per segment. Reaching a checkpoint saves the reply, receipts and
completed changes; Continue remains explicit. Saved numeric settings, including
legacy four-call settings, are preserved. Clear their optional count fields to
choose Automatic. No permissions or provider settings are increased implicitly.

The failure watchdog prevents a third identical failed request and pauses after
six consecutive failures. Successful operations clear its failure history. It
does not mistake verbose thinking or human review for a failed operation. It
does not detect every semantic failure or guarantee task completion. Context,
response, file, protocol and storage bounds remain; partial tool calls never run.
Native self-repair is still milestone 20, not delivered by this change.

Declined prepared plans are discarded through registry, logging and subagent
wrappers so they cannot exhaust the four outstanding-review slots. The core
command approval validator now matches the command adapter's advertised 8 KiB
per argument, 16 KiB aggregate and 32 KiB JSON bounds. Larger scripts should be
written to a file. Activity journals retain up to 2048 entries plus their reserved
terminal marker, and the SQLite reader uses the same bound.

## Deterministic qualification

- The new six-read regression failed against the old default, then passed with
  seven model calls and six reviewed operations.
- Repeated denied/blocked requests, six distinct failures, success resetting the
  streak and queued operations after repeated failure are covered. No later
  queued request executes after the watchdog pauses.
- Automatic model exhaustion and context growth preserve completed receipts.
  Initial oversized fixed context still refuses before contacting the provider.
- Six discarded file-creation previews cannot execute or retain authority; a
  seventh fresh approved preview succeeds.
- A long command argument reaches core approval without the former hidden 4 KiB
  approval limit. Single/aggregate overflow remains rejected by the adapter.
- `python scripts/test-task-budget.py` passes against the normal native release:
  128 successful reads, 33 model calls, saved checkpoint and all 514 activity
  entries readable. A separate missing-file run saves two blocked receipts and
  pauses before the third attempt, with no approvals or file effects. Explicit
  caps, refused continuation, output-limit no-dispatch and Stop also pass.
- `python scripts/test-continuation.py` save/restore passes across two independent
  processes. This checks saved explicit-limit continuation, not automatic replay.
- Rust workspace library suites, Clippy with warnings denied, 248 Flutter tests,
  Flutter analysis and Python compilation pass. Library tests were used after
  the workspace-wide invocation encountered a pre-existing duplicate `probe`
  example binary name in the browser and web crates.

## Live qualification

The configured Qwen/Qwen3.5-2B connection passes the synthetic six-file task:
seven model calls, six reviewed reads, correct final markers, no pause, 15.54
seconds. This uses an isolated profile, 1024 output tokens, a 90-second inactivity
setting and a bounded test driver. The original provider/history remain intact.

Four fresh DeepSeek V4.1 Flash command probes do **not** qualify command completion.
They use provider-default output and a synthetic comment-heavy Node script to
exercise arguments above the former 4 KiB approval bound. The driver approves
only one literal Node invocation with a harmless comment and the fixed public
marker statement; other requests are denied.

| Probe | Result |
| --- | --- |
| 1 | A 5706-byte argument reached approval; the driver denied a mismatch with the originally requested exact comment. |
| 2 | The generated 3978-byte argument was below the pressure criterion and was denied. |
| 3 | A 6894-byte argument reached approval but varied the exact comment, so the driver denied it. |
| 4 | With a letters/whitespace-only comment envelope, the model instead exceeded the adapter's supported shape/size and was blocked before approval. |

Thinking updates were observable (89, 65 and 104 events in the latter three runs).
No probe is counted as a passing command execution, and the comment criterion
changes do not retroactively qualify earlier runs. The opt-in script
`scripts/qualify-task-loop.py` records only counts/statuses, not credentials,
provider endpoints or transcripts; its command mode remains unqualified.

## Native verification and remaining gaps

Fresh useful Mario rerun (2026-10-06): configured `deepseek-v4.1-flash`, actual
`C:/example/project`, exact instruction `Build me a Super Mario clone and
output it as an HTML file`. The normal UI was restored from minimized state and
showed the preserved four-call paused chat. An isolated native-host test profile
used Automatic counts, provider-default output and 180-second inactivity; the
original profile, selected settings and chat were not changed.

The driver separately reviewed a folder listing, three 120-line game reads,
creation of `validate-mario.js` and `validate-bot.js`, and their literal Node
invocations. Eight model calls, eight completed operations and 862 thinking
updates were observed. Both commands returned exit 0 with complete captures.
The first simulated 4,830 frames: coins/score changed, lives depleted, game-over
appeared, and no runtime errors occurred. The heuristic bot reached tile 70 of
196, then lost its lives; that does not prove the level is unwinnable or won.
The agent proposed adjusting the bot's jump timing rather than the game.

The ten-minute driver deadline elapsed during that edit review. No adjustment
executed and no final answer was reached. This is **unqualified end-to-end
completion**, with a successful useful command path; it is a driver timeout, not
evidence of another product approval-timeout or parser fault. The original HTML
is byte-identical, and the two new validation scripts remain as usable work.
An independent fresh syntax check also passed. No synthetic padding retry or
silent default change was made. Full level completion, final reporting and a
new normal-UI end-to-end run remain gaps.

The normal release was built and visibly launched through `scripts/desktop.py`
with the original profile. The dark-theme Advanced → Task limits page shows
Automatic counts and optional blank fields; its draft override was closed without
saving. The original selected DeepSeek model, Review mode and history remain.

The earlier native Mario run created a working HTML game but paused at its saved
four-call setting; completing its command validation under Automatic is still
unqualified. Do not infer that the whole game task is repaired from the six-read
pass. GLM default-reasoning reliability, other-platform qualification and general
computer-use reliability remain open; platform CI 8.4 remains deferred.
