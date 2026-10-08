# Automatic memory, scheduled work and companionship — 2026-10-07

**Milestones 21 and 22 are implemented; 23–24 remain planning only.** This contract extends existing history/preferences/context/run
services rather than replacing them. [ROADMAP](../ROADMAP.md) assigns milestones
21–24; earlier numbers and delivered repair evidence are preserved. Implement and
commit one brick at a time. [Memory qualification](../qualification/automatic-memory.md)
records delivered behavior, bounded live evidence and remaining reliability gaps.
[Scheduling contract](chat-scheduling.md) and
[qualification](../qualification/chat-scheduling.md) record app-open behavior and
model/platform gaps. Product docs must not advertise companionship or closed-UI
execution as delivered.
The design adopts automatic useful facts/decisions with inspect/forget,
and opt-in occasional in-app companionship during chosen hours with a daily cap.

## Memory: a useful index to retained evidence

The human-memory analogy guides design rather than proving a biological equivalent.
Memory is selective and reconstructive; humans do not retain a complete recording
of every conversation or sensation. Working context, durable knowledge and cue-led
episode retrieval are useful separate layers. Scientific findings and engineering
inferences are cited in [memory foundations](../research/memory-foundations.md).

| Layer | Contents | Use |
| --- | --- | --- |
| Working context | Current bounded messages/tools and retrieved excerpts | Immediate task; summaries/checkpoints handle context limits |
| Episode index | Short account of useful task/conversation events, source IDs, time, project and outcome | Locate evidence without loading the entire history |
| Facts/decisions/preferences | Small revisable statements with sources, scope and confidence | Durable recall; distinguish explicit user statement from inference |
| Source evidence | Existing retained message/run/artifact references | Fetch bounded relevant passages when needed; never fabricate unavailable evidence |

Do not copy entire histories into prompts or make a summary the sole truth. Use
immutable source/version references, typed scope and timestamps. Retrieval first
finds compact candidates, then fetches bounded excerpts; lexical retrieval is the
first baseline. Embeddings/vector services are optional only after a measured
retrieval gap, not a mandatory setup. Recollection is quoted/summarized as evidence,
not a tool instruction or a grant. Assistant assertions alone are not facts.

### One-switch user experience

Settings → Memory exposes **Memory** On/Off as the primary control. No template,
folder setup, manual entry or per-item review is required for eligible automatic
capture. Preserve existing stored entries/policy choices during migration; do not
silently enable a previously disabled feature. With no stored choice, start Off.
The user can opt in once with a concise explanation of local storage and provider
use. Optional advanced controls must not become prerequisites.

On: extract useful facts, preferences, decisions, important outcomes and open-work
references from eligible completed interactions. Off: stop new capture/consolidation
and learned-memory retrieval; retain existing data for deliberate Forget/Delete.
Re-enabling does not silently mine old history; an optional bounded Catch up action
can index older eligible retained material. Scope preserves project-specific data;
do not send project A's private facts into B without an explicit sharing policy.

Memory shows recent automatic additions/updates, last attempt, source and honest
Skipped/Failed reason. Empty memory explains whether capture is Off, no eligible
source exists, or extraction failed. Ordinary chat succeeds even if memory fails.
The 21.0 audit traced the earlier narrow explicit-preference system. Milestone 21
now captures exact useful user statements and explicitly shared images. The
private-profile report was not itself a diagnosed live failure. Extraction remains
conservative and model-dependent for statements outside the small literal grammar.

### Capture, consolidation and correction

Durable post-turn watermarks make capture incremental and restartable. Run a bounded
tool-free extraction on new eligible evidence, outside the UI and without delaying
the completed answer. Structured candidates cite real source spans and are validated
before atomic publication. Store provenance, model/policy version and separate usage.
Skip credentials, grants, quoted third-party instructions and unsupported sensitive
inferences. No ambient screen/audio/sensor collection is introduced.

Task receipts can support outcomes; an intention is not a completed task. Inferred
habits remain low-confidence candidates and do not override explicit preferences.
Consolidation links/deduplicates related records; explicit corrections supersede
earlier facts while retaining historical attribution. Opposing statements need
context/time resolution, not blind latest-write replacement. Manual corrections
take priority and automatic maintenance cannot recreate a deliberately forgotten
entry from the same source. Forget applies to index/summaries/caches/embeddings and
pending jobs; explain that original conversation retention is a separate action.
Deletion of source evidence makes recall unavailable/tombstoned, never an invented
replacement. Explicit open-work state becomes resolved when completion is evidenced.

Frozen implemented bounds: 8 KiB source, three candidates, 512-byte exact quotes,
512 output tokens/ten seconds per tool-free request, subject to lower provider
limits. One active maintenance job, sixteen waiting chats, coalesced per chat;
text extraction and optional image caption run sequentially. Recall uses at most
eight index entries, three source excerpts, 4 KiB memory text and 2,000 estimated
inserted tokens, reduced to fit the current context. Automatic records are capped
at 128 per scope and 8192 globally; manual records retain their separate allowance.
No idle reflection, startup replay or silent retries. Older-history Catch up remains
optional future work, not a delivered control.

Images explicitly supplied in chat have source-linked captions/indexes in 21.5;
preserve uncertainty and require eligible image-model handling. Retrieval fetches
only retained allowed assets when relevant. No claim to remember pixels the model
never received or sensory data never captured. Missing/unsupported images keep text
memory useful. At most one image is indexed per completed interaction and one
relevant asset is reopened per request, within existing image/context limits.

## Scheduled: created through ordinary conversation

Example: “Dolores, at 9 pm every weekday, write my daily report using skill X.”
The user-authored request is the intent to create one recurring task. Dolores
resolves the skill/project/model/destination and local timezone, creates a durable
task when those are unambiguous, then replies with a plain-language receipt showing
weekdays/time/timezone, next run, source chat, skill and where the result appears.
No mandatory setup form or redundant confirmation for a fully specified benign
in-app task. A vague mention, quotation, suggestion or model-generated instruction
does not create a task. Missing skill/project or ambiguous time asks only the
necessary question before activation; no guessed command strings or arbitrary tools.

Initial report destination is an in-app task result/thread, not an external message.
External sending/publication requires explicit destination/effect authorization.
Repeated phrasing in the same creation exchange cannot duplicate the schedule.
“Change it to 8 pm”, “skip tomorrow”, “pause this” and “cancel that report” operate
on an identified task; ambiguity asks which one and stale edits recheck revision.

Scheduled lists auto-created tasks with next run, Running/Queued/Waiting for approval/
Succeeded/Failed/Paused status. Task details show progress, artifacts, errors, retry
conditions and run history. Actions: Pause/Resume, Stop current run, Skip next,
Run now, inspect results, edit existing details or continue the change in chat,
and delete with an understandable effect. No Create/New task page: indefinitely
deferred. Proposed task counts/progress come from actual work; no fabricated percent.

### Durable clock and execution

Persist recurrence with named timezone, wall-clock rule, revision and next due time;
never substitute the machine's UTC offset for the chosen zone. Initial rule scope:
one-time, daily and selected weekdays at a local time. Holidays are not implicitly
excluded by “weekdays.” The frozen policy uses the first valid local minute for
gaps and the earlier instant once for repeated times, shown in
details. Clock jumps, restart and suspend reconcile unique occurrence IDs.

Each occurrence has an atomic claim/lease and unique result/run identity. One active
occurrence per task, same-project mutation queue, user-visible status and cancellation.
Background jobs share the host scheduler rather than duplicating profile recovery.
Missed runs initially skip with a recorded reason; Run now is explicit. No burst
catch-up or rerun of uncertain side effects. Failed/offline jobs retain partial
work and the next action. A bounded retry is allowed only for known safe operations;
exactly-once external effects are not promised by a database lease.

Execution snapshots task project, skill version, model and a reviewed-effects ceiling.
Revoked grants, missing project/model or incompatible skill changes pause with a
specific recovery. Pin the approved skill; compatible updates need a deliberate
policy, not surprise substitution. Human approval remains visible and does not
expire on the model clock. A scheduled read/write/task does not bypass the host
approval or protected native-repair boundary. User-requested report generation can
run automatically under its configured scope; future tool effects require review.

Use a task-specific model override when supplied, otherwise pin the current enabled
selection at task creation. Do not change Home's selected model or silently substitute
a model after a failure. Initial per-occurrence budgets inherit existing host bounds;
freeze dedicated smaller defaults with the report corpus in 22.0. All background
usage appears separately. The page explains whether it needs the app open.

Milestone 22 runs only while the application host is alive. Closing Windows/app,
sleeping or being offline cannot honestly promise a 9 pm result. Milestone 24 separately
qualifies opt-in work while the UI is closed and delivery on reopen. No cloud service,
OS wake/security bypass or always-running worker is silently installed.

## Companionship: occasional, warm and grounded

Settings → Companionship has an opt-in switch, allowed hours, daily cap and an enabled
configured weaker model. The design adopts **in-app messages**, not desktop notifications.
Defaults: Off initially; after opt-in, 09:00–21:00 local time, at most 2 attempts/day.
The [frequency policy](companionship.md#frequency-policy-revised-2026-10-08) now
uses a Quiet–Chatty slider from 0 to 100, with pacing based on the chosen hours
and cap. Default low-frequency pacing retains the 3-hour gap. No silent model fallback or foreground
model change. A short generated message uses a separately recorded bounded request.

Candidates include light chat, a checked fun fact, a relevant remembered conversation
or one genuinely unresolved task. Vary timing inside allowed windows with persisted
cooldowns and deterministic test clocks; never call a model repeatedly just to
decide whether to speak. When absent, busy, muted or outside hours, remain quiet;
do not accumulate a greeting flood. Queue at most one candidate, expire it and
revalidate source/task state before delivery. Memory Off prevents memory-based
prompts; companionship can still offer generic chat if enabled.

Deliver as an ordinary labeled initiated conversation in Home; do not force focus,
interrupt a current draft or create a blocking modal. “Not now”, dismiss, mute,
fewer reminders and corrected preferences affect future eligibility. Questions
cannot presume a forgotten task is unfinished or a remembered event occurred
without evidence. Fun facts use a bounded verified source or known validated
content; do not invent a recent factual claim to make a message interesting.

Aim for calm companionship inspired by the documented interface design, without
pretending to have human feelings/consciousness, fabricating personal memories,
guilt, exclusivity or pressure to engage. No task creation/tool execution emerges
from a proactive message without user intent. Preference learning is inspectable,
scoped and respects Off/Forget. Evaluate whether messages are useful and welcome,
not merely whether generation succeeded. “Meta Muse-level” is a capability aspiration,
not a claim of verified parity with an unspecified product.

## Qualification and implementation boundaries

Memory: fixed multi-session explicit facts/decisions/corrections corpus, negative
facts/prompt injection/Forget/Off, scope, restart, byte/token costs and bounded
real recall. Schedule: fake clock/DST/suspend/duplicate/lease/crash fixtures plus
one bounded real report using a configured skill. Companion: fake time/cooldown/
cap/mute/completed-work fixtures plus bounded configured weaker-model checks and
user feedback. Closed-UI worker: process/profile ownership and unavailable-host
tests; never infer it from app-open scheduling. Record fixtures/live results/gaps
separately. Scientific analogy and a paper's simulated agents do not prove reliability.

Visual work follows the revised AGENTS rules: saved renders inspected with
`view_image` first; computer-use only for a relevant UX/native interaction. No
per-task launch or visual ritual. Preserve real profile/settings and secrets;
planning-only work does not activate memory, schedules or proactive messages.
