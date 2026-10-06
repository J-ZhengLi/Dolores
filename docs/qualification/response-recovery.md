# Response recovery qualification — 2026-10-06

## Reproduced failures

The normal app, existing configured DeepSeek V4.1 Flash, the user's selected
project and the exact prompt `Build me a Super Mario clone and output it as an
HTML file` were used. The original long chat first reached a complete file
proposal near its inherited 180-second deadline. The approval disappeared when
that whole-task clock expired; no file was created and the draft returned.

After separating inactivity from task time, the same long chat paused at its
configured 32768-token response allowance instead. The provider reported 28400
reasoning tokens on the first response, and all 32768 as reasoning after an
explicit Continue. Neither run produced an executable tool call or saved file.
Those are failed task completions, despite the improved saved recovery state.

## Changes and deterministic checks

- Blank output omits the provider generation maximum. Explicit numeric settings
  remain valid and existing saved values are preserved. Local context planning
  reserves one quarter of the window, capped at 32768; it is not an API maximum.
- One useful truncated response in provider-default mode can continue within the
  same run. Explicit limits, empty public output, unresolved commands/children
  and repeated truncation pause. Truncated calls never run or enter history.
- Model time measures connection/inactivity, excluding human review. Blank task
  time adds no aggregate deadline; explicit task time excludes root/child review.
  A stalled tool-enabled response or expired active clock retains usable progress.
- Thinking has a separate collapsed preview, latest 16 KiB per model call, with
  an omission label. Actual provider reasoning is shown, not a generated summary.
  Elapsed phase and Stop remain visible after public text starts. Protocol
  reasoning has separate 1-MiB turn/2-MiB cache bounds and serialized accounting.

254 Rust tests passed across core/provider/bridge/store, 12 command-tool tests
passed, and 247 Flutter tests
passed. Workspace Clippy and Flutter analysis passed. Targeted checks include:
an active 1.8-second stream under a one-second inactivity setting; a human review
lasting longer than its one-second model setting; saved partial output on stall;
explicit task clock excluding review; Stop clearing pending work; default output
omitted in plain/streamed/strict tool requests; 198-KiB reasoning preserved on a
tool follow-up; rolling Unicode thinking; explicit cap/empty/repeated truncation
refusing automatic continuation; discarded partial calls with fresh approval.
Scoped-settings integration passed with four fixture requests. Separate skill-draft save/restore processes passed with 21 fixture requests, retained promotion evidence, Stop and reviewed rollback. The skill-draft
fixture now deliberately selects its reviewed skill and uses explicit limits
for truncation tests rather than relying on chat's previous default.

## Native follow-up

The updated normal desktop app was built and visibly launched using the
maintained Python launcher. The DeepSeek model editor saved blank output while
retaining provider-default reasoning, 180-second inactivity, configured context,
image capability and Review mode. Other models' settings were not changed.

A fresh chat in the same project used the exact prompt. It showed live rolling
thinking, public planning and tool preparation, then a valid folder-listing
approval after approximately three minutes. That approval remained usable after
a measured 190-second hold. Approving the folder listing and reviewed creation
produced a 14,604-byte standalone HTML game. The four model responses reported
28,561 / 2,953 / 1,978 / 3,769 output tokens, respectively; the first included
22,437 reasoning tokens. No response-limit or approval-timeout banner appeared.

The run then exposed a separate command defect: validation scripts longer than
1 KiB were refused with a misleading generic shape error, despite the model
correctly supplying literal arguments. Two refused commands consumed the four-call
run allowance, so the native task paused rather than reaching a qualified final
answer. The file and receipts remained after restart. This is artifact success,
not an end-to-end task-completion pass.

Command arguments now support 8 KiB each / 16 KiB total, within 32 KiB JSON;
larger checks receive an explicit instruction to create and run a script file.
Both the streamed-call validation and installed-executable adapter enforce these
bounds. Tests execute a reviewed script over 1 KiB containing quotes and Unicode,
refuse oversized single/aggregate arguments without effects, and check fragmented
command JSON plus overflow. Native model-command validation after this fix remains
an acceptance gap; Windows automation cannot approve terminal execution.

Independent JavaScript syntax and headed Edge rendering, right-arrow movement,
Space jump and R restart passed. Screenshots were inspected locally. Browser
console errors were limited to the missing favicon; no game script error occurred.
These are basic artifact checks, not a full game-quality assessment. The rebuilt
normal app is visibly relaunched with the same profile after these checks.

Computer-control text insertion did not reach Flutter on this run. Ordinary
key input did, and the exact final composer text was visually/accessibility
verified before sending. This is an automation limitation, not established
evidence of a user keyboard or paste defect.

## Comparison and remaining scope

OpenAI's [Codex agent-loop explanation](https://openai.com/index/unrolling-the-codex-agent-loop/)
describes repeated model/tool iterations within a turn, potentially hundreds of
tool calls, and context management as history grows. Its [reasoning guide](https://developers.openai.com/api/docs/guides/reasoning)
documents finite response maxima and shared reasoning/visible output allowances.
These establish the distinction between a long task and one long response; they
do not establish that Codex automatically salvages arbitrary truncated tool JSON.

Dolores still has explicit model/tool/segment allowances, provider-specific
behavior and bounded file/protocol storage. This change is not full Codex parity
or a general coding-competence qualification. Arbitrary native self-repair is
planned in milestone 20; inspection and recovery-hint mods cannot perform it.
Platform CI 8.4 remains deferred.
