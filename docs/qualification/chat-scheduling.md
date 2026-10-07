# Chat-created scheduling qualification — 2026-10-07

Milestone 22 implements app-open scheduling and its management page. This report
separates host mechanics, model interpretation, rendered appearance and normal
desktop integration. The [contract](../design/chat-scheduling.md) owns numerical
bounds; the [roadmap](../ROADMAP.md) retains companionship and closed-UI work as
separate milestones.

## Implemented behavior

A direct chat request creates a durable local task with an exact named-zone rule,
project, enabled model, current reviewed skill version and smaller task budgets.
One-time YYYY-MM-DD dates, daily and selected weekdays are supported. The receipt
shows its next run and result destination. Quotes, discussion, assistant/tool
text, invented times or unavailable skills cannot authorize task creation.

One conditional 15-second app-open clock shares the existing chat queue/owners.
Occurrence claims and next-run updates are atomic. Results have separate saved
conversations; approvals and Stop use the ordinary host path. Creation grants no
future tool access. Restart interrupts unfinished work without replay. Overdue
work records one missed run and advances without a catch-up burst.

Scheduled lists and filters existing tasks with one actions menu, real progress,
approval, errors and history. Pause/Resume/Skip/Run now/Stop/Delete and result/source
links are present; there is no creation form. Chat changes time/skill/model or
pauses/resumes/skips/cancels a resolved task. Active work keeps its prior snapshot.

## Deterministic evidence

All 288 core/store/bridge library tests, 326 Flutter tests, analyzer and strict
Clippy pass. Focused cases cover named-zone spring gaps/autumn overlap, weekdays,
literal-time/source refusal, duplicate calls/ticks, backward clock, missed-run
advance, atomic failed writes, stale changes, pronoun ambiguity, paused clocks,
Home draft/model/project preservation and retained lists after refresh failure.
Sixty deliberate skips retain the active occurrence within the 50-record history
and still refuse another Run now.

`scripts/test-scheduling.py` drives the normal packaged native C ABI in a public
disposable profile. It creates a weekday task, checks its metadata-only tool
catalog, injects a due instant, executes the pinned report, approves one file read,
stops while awaiting approval, refuses overlap and recovers after an offline model.
It also checks Pause/Skip/Run now, stale writes, disabled-skill refusal and explicit
conversational skill-version recovery. A separate process verifies interrupted
claims are not replayed and completed results survive reopening.

The due-clock injection is a fixture. It does not establish that a physical
desktop fired at 21:00, nor that Windows sleep or app closure permits execution.

Five Scheduled page cases and three host cases cover narrow/wide light/dark,
action-menu Pause, result opening, stale/failed action recovery and no creation
control. Saved renders were inspected with view_image. They are diagnostic
appearance evidence, not physical keyboard, hover or accessibility acceptance.

## Model evidence

`scripts/qualify-scheduling-live.py` copies configuration only into a history-free
profile and uses the configured vault key in memory. Original profile hashes are
checked before and after. Public prompts and short synthetic report guidance are
the only task data in this corpus. No model or setting is changed in the original
profile; execution is an explicit Run now, separately from the due-clock fixture.

Earlier Qwen direct-creation probes asked unnecessary questions instead of
saving the fully specified weekday task. The first exposed future skill guidance
being loaded during creation; metadata turns now omit that guidance and use only
their scheduling tool. Another probe still asked for report/model details. These
misses count as model reliability gaps, not successful task creation.

Final bounded live results:

| Case | Evidence |
| --- | --- |
| Qwen direct weekday creation | Miss: unavailable tool after earlier unnecessary-detail responses. No task falsely counted as created. |
| DeepSeek creation with explicit Qwen execution model | Pass: one task, exact weekday/time, named skill/model and receipt. |
| Qwen pinned report | Miss: requested file access despite the public skill's no-tool instruction; driver cancelled that request. |
| DeepSeek creation and pinned report | Pass: one task and the actual PUBLIC-SCHEDULE-REPORT result in its separate saved conversation. |
| DeepSeek conversational time change | Miss in two bounded probes: adapter refused incomplete/invalid arguments; revision/time stayed unchanged. Page management and deterministic typed conversational changes pass separately. |
| DeepSeek quoted example | Pass: no task created in all four probes. |

Four isolated runs used 13 chat/run turns in total. Provider-reported token totals
were unavailable and are not reported as zero. Each live request had a 1,024-token
output ceiling, 20-second model inactivity and a 35-second driver deadline. Future
tool effects were not approved in live probes. General natural-language scheduling
and conversational-edit reliability remain **unaccepted**. There was no silent
model fallback: the separate DeepSeek creation explicitly named Qwen for its run.
The later direct DeepSeek case pinned DeepSeek throughout.

Final receipts: `output/scheduling-live-03/receipt.json` and
`output/scheduling-live-04/receipt.json`; earlier misses are retained in runs 01/02.

## Costs and limits

Clock scans load task metadata and pending occurrences, not all completed skill
snapshots. Management responses omit occurrence snapshots and skill text. Each
task pins only the current reviewed skill version, capped at 8 KiB; historical
skill versions are retained by their original skill service, not copied here.
No scheduler process, idle model polling or closed-UI worker is added. When no
unpaused recurrence or active occurrence needs a clock, its timer is absent.

`scripts/qualify-scheduling-resources.py` seeds 32 tasks and 1,600 public terminal
occurrences with maximum prompt/skill sizes and samples list/tick costs. Those
short measurements do not qualify sustained idle CPU, memory on lower-end hosts
or other platforms. Physical timing, sleep/wake, accessibility and broad natural
language reliability remain open. Arbitrary recurrence text, holidays, manual
creation forms, external delivery and OS wake are excluded.

Normal packaged fixture: 11 local provider requests, zero live requests; reopening
made zero provider requests. Ten tick samples had a 0.429 ms maximum and its
management summary was 3,295 bytes. At maximum synthetic retention, stored JSON
was 22,791,996 bytes, the list response 238,232 bytes, tick maximum 76.319 ms and
list maximum 27.123 ms across ten samples. This is a state/latency sample, not a
claim of bounded sustained process RSS or idle CPU. Receipts:
`output/scheduling-fixture-03/{save,reopen}.json` and
`output/scheduling-resources-01/receipt.json`.

## Normal desktop and preservation

Normal Windows main-entry release build passes. The owned preview was restored
using the desktop launcher, which reported a visible native main window; physical
foreground interaction was not inspected. Schema 36 adds the two scheduler tables.
All 45 original table hashes are unchanged, including provider settings, model,
history, drafts and memory. No schedule was inserted in that profile. Private
hash/identity receipts stay under ignored output; the original database backup
also remains there. `output/scheduling-normal-handoff.json` records preservation.

## Corrections found during qualification

The schema migration now tolerates existing scheduler tables when an older-schema
fixture reopens. The first normal build could not replace a DLL held by the owned
preview; its identity and inactive saved runs were checked, its database backed
up, and only that preview was stopped. A real Start exposed the 14-tool catalog
limit; metadata turns now omit future project tools without increasing the limit.
Newest-first history uses insertion order so same-second manual runs are stable.
Retention keeps active owners visible even after repeated skips. Explicit model
selection uses that model's request profile and source-scoped settings.

Reproduction scripts require a new disposable output directory; save/reopen must
run in separate processes. Native artifacts come from the normal `desktop.py
build` path. Ignored output receipts hold measurements; credentials/private
conversations are never exported into this report.
