# Stream generation, image chat and harness diagnosis

2026-10-06. The developer-workspace roadmap remains paused for these fixes.
All model requests use disposable profiles and synthetic prompts. The original
provider selection, response settings and history are preserved. Private keys,
transcripts and raw run records stay outside tracked artifacts.

The subsequent [fragmented reasoning/progress follow-up](reasoning-streams.md)
reproduces and replaces the cumulative 2 MiB guard described below, qualifies
DeepSeek with default reasoning and adds visible request activity. This document
records the initial fix and its passing/failed settings, not that later boundary.

## Stream diagnosis and bounds

A deterministic HTML file call split into more than 4096 small SSE events
reproduced `Streamed tool response exceeds its frame limit.` before the fix.
The same response now completes with its exact arguments. The old guard counted
provider fragments, even when the generated file and wire payload were small.
Its historical error also covered oversized individual events, so the user's
old banner alone cannot prove which of those bounds fired.

The initial fix removed the event-count guard but bounded total wire data to 2 MiB,
individual agent events to 256 KiB, assembled fields to 128 KiB, plus existing
complete-call validation, file limits, cancellation and deadlines. Oversized
events/total wire data and malformed or truncated calls still refuse dispatch.
Recovery identifies the stream limit, retains completed work and suggests
inspection or a smaller first file. It never silently replays the request.

Tool-response reasoning is preserved exactly in bounded transient adapter state
for providers requiring it on the next tool-result request. It is matched to
exact assistant text and tool calls, excluded from UI/journals/exports and
cleared for fresh runs/final answers. New model projections use independent
state. Both cached keys and reasoning count toward 128 KiB, with at most 16
entries; serialized request limits still apply. This is protocol continuation,
not durable agent memory. Context-preview token estimates do not separately
itemize this private protocol field.

The agent sees its effective output/deadline allowance and guidance to build a
small usable baseline before polish. Limits and user-selected defaults are not
automatically raised. GLM Low is an explicit reasoning option using `max_tokens`,
`thinking.type=enabled` and `reasoning_effort=low`. See the primary
[GLM thinking documentation](https://docs.z.ai/guides/capabilities/thinking-mode)
and [DeepSeek protocol](https://api-docs.deepseek.com/guides/thinking_mode/).

## Bounded live HTML checks

Exact prompt: `Build me a Super Mario clone and output it as an HTML file`.
Each task had four model calls, six tool operations, one segment, a 240-second
task deadline and 180-second response deadline. Only local HTML file operations
and folder listing were approved; model-proposed commands were denied.

| Model | Output allowance | Explicit reasoning | Result |
| --- | --- | --- | --- |
| DeepSeek V4.1 Flash | 8192 tokens | DeepSeek thinking off | Completed; one 14,152-byte standalone HTML file; JavaScript syntax PASS |
| GLM-5.3-Flash | 16384 tokens | GLM Low | Completed; one 6,517-byte standalone HTML file; JavaScript syntax PASS |

Both generated files were opened in an isolated local browser and their canvas
games rendered. Movement/jump keys were exercised; GLM's score changed from 0 to
100. No script runtime errors were reported; DeepSeek's only console error was
the local server's missing favicon. These checks establish transport, artifact
creation, rendering and basic input, not game completeness or broad coding
reliability. They do not establish success at every output allowance.

Earlier disposable provider-default runs exhausted DeepSeek output allowances
and timed out with GLM. Those failures are retained as qualification evidence;
thinking-off/low success is not a provider-default pass. Continue/changed response
settings remain explicit user actions. No incomplete file call was executed.

Reproduce with `scripts/qualify-html-generation.py --help`, a fresh absolute
directory below ignored `output/` and credentials supplied through the documented
test environment variables in that script. Do not use a normal data profile.

## Harness diagnosis

`inspect_harness` now exposes streaming/settings/recovery/attachment source IDs
and at most three failures from the current chat's latest 20 runs. Summaries
contain saved build, model, frozen limits, host-authored category/evidence and
saved pause metadata. They omit prompts, paths, tool arguments, arbitrary plugin
errors and transcripts. Current bundled source is explicitly distinguished from
historical builds. Inspection remains read-only and reviewed.

The live DeepSeek diagnosis case first injected a deterministic oversized event,
then asked the real model to diagnose it without replay. PASS: the model read
inventory and `provider_stream`, identified the 256-KiB bound and stated that an
incomplete call did not run. The follow-up used 4096 output tokens, thinking off,
a 120-second response deadline and four reviewed inspection operations.
`scripts/qualify-harness-diagnosis.py` reproduces this bounded case. This result
does not establish arbitrary self-diagnosis or grant self-modification authority.

## Images and recovery

Widget checks cover image-only paste despite an empty text clipboard, text
fallback across Markdown blocks, immediate pending/saved thumbnails, preserved
text and earlier attachments after storage failure, oversized-image refusal,
session reservation during asynchronous paste and temporary-copy cleanup.
Windows BMP-to-PNG checks cover actual decoding plus malformed/oversized headers.
Existing IME, Shift+Enter, selection and undo tests remain required.

The normal Windows release was visually checked with a synthetic image copied
from Paint. Shift+Insert created a draft thumbnail without replacing text.
Sending to a delayed local fixture immediately displayed that thumbnail in the
right-aligned user bubble, while Stop and the working indicator remained visible.
The fixture received exactly one image; the saved bubble retained its preview
after completion and normal-app restart. Right-Control+V also created a new
thumbnail in a fresh empty temporary chat without sending a model request.

The native helper's injected left-Control chord did not paste. The already
documented [input check](../design/native-input-verification.md) records its
modifier being released before the letter event; this does not establish a
physical-keyboard failure. Right-Control passed; physical-keyboard acceptance
remains separate. Shift+Insert exercised the actual native clipboard conversion.

The clipboard dependency is native `pasteboard` 0.5.0, with no embedded browser
or idle clipboard polling. The Windows payload allowlist now requires its DLL;
four packaging integrity/refusal tests PASS and Flutter's compressed notices
contain the package attribution. Native launcher preflight PASSes refusal of
missing/empty clipboard DLLs, restoration and retention of synthetic history.
The rebuilt normal app was visibly inspected and left open with the original
profile; all 38 history/settings tables are unchanged.
Full separate inventory collection currently
refuses missing `wasmi` notice text. That release gap is not waived.

Remaining boundaries: other OS clipboard paths, physical keyboard/IME combinations,
large native bitmap allocation before adapter checks, interrupted temporary-file
cleanup, repeated model stress runs, general visual comprehension and complete
game quality are unqualified. Platform CI 8.4 stays deferred.
