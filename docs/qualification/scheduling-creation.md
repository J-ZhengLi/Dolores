# Scheduling creation correction — 2026-10-08

The reported request asked for a previous-day report at 09:00 on each workday,
using the selected repository's guidance. The exported conversation records a
real blocked `schedule_task` call. Its later assistant explanation incorrectly
denied that the call had happened. The private export is not copied into Git.

## Cause and correction

The host recognized a scheduling request but required a recurrence phrase
allowlist. “Each workday” failed. The agent loop then masked the scheduling error
as a file-policy denial. A public normal-bridge regression reproduced zero saved
tasks and that exact misleading error. Focused Rust regressions also failed first.

Further checks exposed English/Chinese keyword gates in routing and task edits.
Adding synonyms would retain that design flaw. Both scheduling tools now coexist
with ordinary work tools in human turns, and the configured model interprets
intent and schedule wording in any language. No classifier call is added. The
registration ceiling explicitly rises from 15 to 17 for these two definitions.
Scheduled occurrences omit them, so executing a saved task cannot recursively
create one. Host checks retain schema/rule bounds, current source chat, task
identity/revisions, enabled model/skill, cancellation and duplicate reconciliation.
Creation grants no future tool authority and requires no extra user approval.

Null model pins the current model; current/default aliases do too. An unrequested
different model produces an actionable correction. Ordinary repo guidance needs
no named skill. Null creation time uses 09:00 local time and returns
`usedDefaultTime` so the agent discloses the default. The user explicitly accepted
a default instead of a missing-time question. Ambiguous time, missing recurrence
or date, and unresolved task identity still need a brief clarification.

## Iteration evidence

An initial isolated live DeepSeek V4.1 Flash probe passed the original request,
missing-time clarification and quoted-example refusal, but its worded-time/range
case supplied an unrequested model and unnecessarily asked for its ID. Evidence
remains under ignored `output/scheduling-creation-live-02/receipt.json`.
The next probe created both valid tasks but chose and disclosed 17:00 for an
omitted time (`output/scheduling-creation-live-03/receipt.json`). The user then
accepted defaults; final policy uses the explicit 09:00 default above. An interim
missing-time refusal was removed rather than retained against that preference.

The multilingual probe (`output/scheduling-creation-live-04/receipt.json`) saved
all six requested tasks and left all three negative/ordinary cases unscheduled.
It used 17 model calls (96,607 reported tokens, including cached input). Two cases
corrected malformed fields within the existing budget without a user question.
Manual receipt review still found a confirmation claiming that Monday's
“previous day” report meant Friday. The stored prompt was unchanged, but that
claim was unsupported. Final confirmation guidance limits responses to two short
receipt-based sentences, omits internal details, and forbids that reinterpretation.

The shortened-confirmation repeat (`output/scheduling-creation-live-05/receipt.json`)
passed seven of nine mechanical cases. The model unnecessarily asked for a time
on the omitted-time request, and its Spanish call had invalid/incomplete provider
arguments before any task was saved. Arabic creation saved the correct clock but
incorrectly described it as a default. These misses are retained. Final guidance
places the automatic default after other context guidance, explicitly disallows a
missing-time question, requests object arguments and requires accurate default
receipt reporting. This corrects guidance; it does not remove model uncertainty.

## Verification boundary

Rust validation passes 76 core and 159 bridge tests, with one intentional bridge
ignore; formatting and strict Clippy pass. The normal Windows main-entry build
passes. The native creation corpus covers nine requests across six languages,
duplicate prevention, preserved prompt/project/model and no extra creation
approval. Null time defaults to 09:00 with an explicit receipt flag. Malformed
fields create nothing and retain their actionable error; an ordinary file read
keeps its work tools and creates nothing. The corpus uses 22 local HTTP requests
and no live credentials. Its catalog contains 16 tools; the initial two scheduling
definitions occupy 2,899 serialized UTF-8 bytes. No new idle process, timer or
separate classification call is added.

The scheduler save/reopen corpus passes with 11 local HTTP requests: due
execution, pinned skill/result, reviewed reads, Stop while waiting, overlap,
offline recovery, stale edits, disabled-skill correction and interrupted restart
without replay. Its due instant is injected, not a physical clock acceptance.

The final normal main-entry build passes. The bounded DeepSeek V4.1 Flash corpus
(`output/scheduling-creation-live-06/receipt.json`) passes **9/9**: original
workday wording, worded time/range, omitted-time default, Spanish, Japanese and
Arabic creation, quoted example, Spanish negation and ordinary work. Six tasks
are saved without an approval or follow-up question; all confirmations are short
and preserve the requested work. Only the omitted-time case has
`usedDefaultTime:true`, matching its disclosed default; explicit times are not
described as defaults. No tasks are saved for the three negative/ordinary cases.
There are 15 model calls, with 84,607 input and 1,755 output tokens reported
(86,362 total, including cached input). The earlier misses remain evidence of
model variability; this final pass is not a general reliability claim.

The final native fixture repeat passes all nine creation cases, malformed fields
and ordinary file work in 22 local requests. The maintained normal app is reopened
using the original profile, with Windows window presence confirmed. All 49 table
hashes remain unchanged. The reported failed task was not silently created in that
profile; resending the original instruction uses the corrected flow.
Public synthetic prompts and isolated configuration-only profiles are used;
credentials stay in memory and original conversations are not copied. The original
profile is compared against all 49 saved table hashes. A window's presence on
reopen does not establish physical UX, real clock timing or sleep/wake behavior.

Host checks validate structured metadata, not multilingual semantics. Negation,
quotes, time interpretation and task intent depend on the model; neither fixtures
nor a bounded live corpus prove arbitrary-language reliability. Future execution
still uses existing tool permissions, interruption and recovery controls.
