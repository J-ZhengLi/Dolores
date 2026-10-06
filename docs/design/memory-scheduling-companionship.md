# Automatic memory, scheduled work and companionship — 2026-10-07

**Planning only.** This contract extends existing history/preferences/context/run
services rather than replacing them. [ROADMAP](../ROADMAP.md) assigns milestones
21–24; earlier numbers and delivered repair evidence are preserved. Implement and
commit one brick at a time. Product/acceptance docs must not advertise these targets
as delivered. The user chose automatic useful facts/decisions with inspect/forget,
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
The current narrow system extracts explicit preference excerpts only; milestone
21.0 audits actual trigger/policy/scope/errors before changing that behavior. The
user's lack of observed updates is a reported gap, not a diagnosed live failure.

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

Initial proposed bounds: 16 KiB source excerpt per extraction, at most 8 candidates,
one tool-free request up to 1,024 output tokens/15 seconds, subject to lower provider
limits. Queue coalesces per chat; one maintenance request at a time. Recall proposes
8 index candidates, 3 source excerpts and 2,000 inserted tokens, reducing to fit the
current context. These are design proposals to freeze against fixed corpora before
implementation, not raised existing defaults or acceptance results. Queue/disk/index
caps must be measured/frozen in 21.1; no unbounded idle reflection or silent retries.

Images explicitly supplied in chat can later have source-linked captions/indexes;
preserve uncertainty and require eligible image-model handling. Retrieval fetches
only retained allowed assets when relevant. No claim to remember pixels the model
never received or sensory data never captured. Missing/unsupported images keep text
memory useful. This is a separate 21.5 brick after text recall works.

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
excluded by “weekdays.” Define DST gap/overlap behavior before activation. Suggested
policy: next valid instant for gaps and one occurrence for repeated times, shown in
details. Clock jumps, restart and suspend reconcile unique occurrence IDs.

Each occurrence has an atomic claim/lease and unique result/run identity. One active
occurrence per task, same-project mutation queue, user-visible status and cancellation.
Background jobs share the host scheduler rather than duplicating profile recovery.
Missed runs initially skip with a recorded reason; Run now is explicit. No burst
catch-up or rerun of uncertain side effects. Failed/offline jobs retain partial
work and the next action. A bounded retry is allowed only for known safe operations;
exactly-once external effects are not promised by a database lease.

Execution snapshots task project, skill version, model and granted capability scope.
Revoked grants, missing project/model or incompatible skill changes pause with a
specific recovery. Pin the approved skill; compatible updates need a deliberate
policy, not surprise substitution. Human approval remains visible and does not
expire on the model clock. A scheduled read/write/task does not bypass the host
approval or protected native-repair boundary. User-requested report generation can
run automatically under its configured scope; tool effects use existing grants.

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
configured weaker model. The user chose **in-app messages**, not desktop notifications.
Proposed defaults: Off initially; after opt-in, 09:00–21:00 local time, at most 2
messages/day with a 3-hour minimum gap. Chosen hours/cap always override defaults;
freeze these proposed limits before 23.0. No silent model fallback or foreground
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

Aim for calm companionship inspired by the user's Dolores reference, without
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
