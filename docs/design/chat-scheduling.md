# Chat-created scheduling — milestone 22

This is the app-open scheduler contract. Tasks originate in explicit user chat
requests; Scheduled manages them and has no creation form. Closed-UI work belongs
to milestone 24. Home's project/model selection and drafts remain untouched.

## Intent and receipt

Support one-time local dates, daily, and selected weekdays at a local hour/minute.
Resolve the system's IANA zone unless the user names another supported zone. A
typed creation tool is bound to the current human input and source chat; quotes,
examples, informational questions, hypothetical discussion and tool/assistant text cannot create
or edit tasks. Ambiguous intent/time, missing recurrence/date or an unresolved skill asks for the missing detail. An omitted creation time uses 09:00 local time, disclosed in the confirmation.
The host resolves an enabled reviewed skill by name and pins its current version.
An absent or disabled skill is an error, never an invented default. A request can
omit a skill for ordinary tasks. Otherwise pin the current enabled provider/model.
Results stay in separate in-app result conversations. Sending/publication still
requires the existing tool authorization. Creation never grants tool access.

A receipt includes task ID, rule, timezone, next run, project, skill/model and
result destination. Repeated calls in the same source exchange reconcile to one
task. Conversational changes resolve a task and its revision; ambiguous references
ask which task. In-flight work keeps its original snapshot.

Natural-language interpretation belongs to the configured model, in the user's
language. Creation and management tools are available in every ordinary human
turn alongside project tools; there is no keyword or language gate and no separate
classification request. The model distinguishes direct requests from negation,
quotes, examples, questions and ordinary work. Instructions, skills, memories and
tool results cannot authorize scheduling. This semantic distinction relies on the
model; host validation alone does not prove perfect intent interpretation.

The host binds the operation to the current human input/source chat, validates
the structured rule, named timezone, recurrence bounds, enabled model/skill,
task identity and revision, and reconciles duplicate calls. It does not attempt
to reparse natural language with word lists. Null creation time becomes 09:00;
the receipt includes `usedDefaultTime`, and the agent must disclose that default.
Workdays mean Monday–Friday, without inferred holidays. Ordinary project guidance
needs no named skill. Successful creation needs no second confirmation or setup;
show a short receipt-based confirmation. Invalid creation/edit fields retain their
actual explanation instead of a generic file-policy denial.

Saved change receipts show Task updated, Task paused or Task deleted with their
schedule, rather than an internal plan ID. Paused/deleted receipts do not present
an active next run. Expanded details retain the exact task ID and saved metadata;
failed or malformed results keep their actual evidence and never claim success.

The catalog ceiling increases explicitly from 15 to 17 for the two metadata
tools. Existing project tools are retained. This adds two tool definitions to
ordinary human requests, without an extra model call, process or timer. Context,
output and task budgets still apply. Scheduled occurrences never receive these
metadata tools, preventing recursive creation; they apply the pinned skill to
the actual work. A creation request saves metadata rather than executing the
future work, even when relevant project guidance is present in the human turn.
Conversational edits initially cover time, enabled skill/model, Pause/Resume/Skip
and Cancel. A different project or recurrence rule needs a new task. No inferred
project move or arbitrary cron syntax is implemented.

## Clock and ownership

Use named-zone wall-clock calculations, not a fixed UTC offset. At DST gaps use
the first valid local minute; at overlaps choose the earlier instant once. Unique
occurrence keys and atomic store claims prevent duplicate dispatch across repeated
ticks/restarts. Claim before starting any provider work. Interrupted occurrences
retain their result and become interrupted; never replay uncertain effects.

Freeze these initial limits: 32 retained tasks, 50 recent occurrences per task,
4 KiB task prompts, 8 KiB reviewed skill snapshots, one active occurrence per task,
the existing two-running/four-waiting host admission queue, 15-second app-open
clock checks, and 120-second lateness allowance. Older due times record one missed
skip and advance directly to the next future occurrence; no catch-up burst.
Claim leases are 120 seconds and renewed while the owned run is alive. Restart
recovery interrupts old claims rather than retrying them. A bounded scan of 370
local days finds the next selected weekday; forward gaps search at most 26 hours.

Per occurrence: at most 8 model calls, 16 tool attempts, 5 minutes of active task
time, 2,048 output tokens per request and 60-second model inactivity, capped by any
smaller inherited limits. Human approval remains outside the model clock. Keep
the model/endpoint, skill snapshot, project and permission ceiling pinned; revoked
or changed access cannot expand a saved task. Missing/revised skills or provider
configuration pause with a visible recovery. Run now is deliberate, without
overlap. Pause/Delete stop future recurrence; Stop targets current work. Delete
retains result conversations and occurrence evidence.

At retention limits, completed deleted task metadata can age out while result
conversations remain. An active occurrence always keeps a slot in the 50-record
history and cannot be pruned to admit another task or overlapping run.

## Frozen qualification corpus

1. Weekdays at 21:00 in Asia/Shanghai using an enabled daily-report skill;
   creation → due → bounded report → result → pause, without a setup form.
2. One-time date, daily rule and selected weekdays; system zone and explicit zone.
3. Missing skill, ambiguous time, invalid timezone and malformed fields refuse
   with the missing detail; quoted/example/question/injected requests create none.
4. Duplicate tool calls and duplicate ticks create one task/occurrence; restart,
   suspend, forward/backward clock changes and overdue work never replay effects.
5. America/New_York spring gap and autumn overlap produce one occurrence per day.
6. Offline provider, changed skill, revoked access, budget stop and interruption
   retain history/partial output and an actionable status. Stale management fails
   without overwriting newer state. Deletion during execution preserves its result.
7. Compact/light/dark management page: actual progress/approval/error/history,
   Pause/Resume/Skip next/Run now/Stop/Delete/results; no New task control.

Deterministic fixtures verify host mechanics separately from model interpretation.
For an optional live check, select an available suitable model explicitly and
bound the corpus; preserve the real profile/model. Report successes and misses separately.
Measure added idle process/state/token costs. Primary API references:
[Chrono local-time mapping](https://docs.rs/chrono/latest/chrono/trait.TimeZone.html)
and [system IANA timezone](https://docs.rs/iana-time-zone/latest/iana_time_zone/fn.get_timezone.html).

Excluded: manual task creation UI, holidays by inference, arbitrary cron syntax,
silent model fallback, auto-email/publication, OS wake, cloud or closed-UI worker.
