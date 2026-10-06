# Fragmented reasoning streams and visible progress

2026-10-06 follow-up to [stream and image recovery](stream-and-image-recovery.md).
Developer-workspace milestones remain paused. These results supersede the earlier
cumulative wire limit; they do not erase the earlier failures.

## Reproduction and change

The earlier fix removed an event-count guard but retained a cumulative 2 MiB HTTP
body guard. Repeated metadata around tiny reasoning/tool deltas could exhaust it
even when the actual decoded answer and file were small. A real adapter fixture
with 12,000 one-character reasoning deltas and a complete small HTML call reproduced
the exact wire-limit error. It now completes with identical arguments.

The adapter bounds decoded fields, individual events, tool arguments and the whole
request deadline. The 2 MiB allowance now applies to traffic **without decoded
progress**, resetting when reply, reasoning or tool-call data grows. Excessive
padding/idle events still fail explicitly. There is no automatic retry and an
incomplete call never runs. The decoder also avoids repeatedly scanning the rest
of CRLF batches and draining the front buffer for every event. The 38-test provider
suite completed in 2.64 seconds locally; the earlier single reproduction took
about 20 seconds. These fixture timings are not a universal performance claim.

During a model call the host publishes Waiting for model, Thinking, Preparing tool
call or Responding, with elapsed seconds and the response deadline on hover. A
request-scoped pulse runs about once a second even when no public text arrives;
there is no idle polling timer. Private reasoning words and partial arguments
are not published. Stop remains available. An elapsed counter shows time, not a
promise of progress or completion.

## Basic and failure checks

- Core/provider/bridge: 192 tests PASS; Flutter: 245 tests PASS; analysis: no issues.
- Fragmented valid reasoning/HTML beyond 2 MiB passes; excessive no-progress
  traffic and oversized individual events fail without dispatch.
- A delayed real TCP fixture publishes Reasoning before completion, emits no
  private reasoning text and cancels promptly. Existing malformed/truncated-call,
  output-limit and whole-request deadline cases pass.
- Widget check: reasoning-only activity is visible; stale model-step activity is
  ignored; Stop restores the draft. The normal native release was checked with a
  disposable keyless delayed provider: waiting/thinking/tool preparation appeared,
  Stop interrupted incomplete arguments and restored the draft without approval.
  Native injected Unicode text was not accepted by the verification helper; the
  check used a literal key instead. Physical IME coverage is not claimed.

## Configured-provider checks

Exact prompt: `Build me a Super Mario clone and output it as an HTML file`.
Fresh disposable profiles used 32,768 output tokens, a 180-second response deadline,
240-second task deadline, four model calls, six tool operations and one segment.
Approvals allowed folder listing and local HTML file operations only. Proposed
commands were denied. Saved user model/settings/history were not changed.

| Model / reasoning | Result |
| --- | --- |
| DeepSeek V4.1 Flash / provider default | Completed without pause; one 13,686-byte standalone HTML file; JavaScript syntax PASS. 156 activity updates covered waiting, reasoning, tool arguments and responding; maximum gap 1.03 seconds. |
| GLM-5.3-Flash / provider default | Reached the 180-second response deadline while reasoning; no file or tool call. 181 activity updates, maximum gap 1.03 seconds. This is a failed completion, not a wire-limit regression. |
| GLM-5.3-Flash / explicit Low | Completed without pause; one 9,076-byte standalone HTML file; JavaScript syntax PASS. 74 activity updates across all four phases; maximum gap 1.03 seconds. |

DeepSeek's canvas game rendered in an isolated local browser; restart, movement
and jump keys were exercised and a post-input screenshot showed the player.
No script errors occurred; the local server reported a missing favicon.
This is transport/artifact/basic-input evidence, not complete game qualification
or a guarantee of dependable general coding. GLM default reasoning remains an
acceptance gap. A separate explicit Low run completed; that is not a default-mode
pass. Lower reasoning or a different model is an explicit choice,
not a hidden retry or automatic settings change.

## Self-repair boundary

Current source inspection and ABI 1 recovery-hint mods cannot fix arbitrary native
runtime code. The 120-line inspection setting is a per-read page, but navigation
is incomplete. [Milestone 20](../design/harness-self-repair.md) plans matching
source/search, actual failure measurements, managed patches, independent trials,
broader extension seams, reviewed native installation and rollback. It is planned,
not delivered by this runtime fix. Platform CI 8.4 remains deferred.
