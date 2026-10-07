# Automatic memory — milestone 21

## 21.0 Audit and frozen evaluation

The baseline captures the latest user message after a successful, unpaused turn.
`automatic_memory::learn` claims a durable message watermark, obtains the saved
policy and scoped preferences, then uses literal response-style extraction or a
tool-free model request. It currently waits before sending `done`. Sources belong
to completed user/assistant pairs. Atomic publication rechecks source bytes,
policy revision, scope, cancellation and the existing record set.

Baseline eligibility requires explicit preference phrases. Its six topics exclude
ordinary project facts, decisions, outcomes and open work, which are intentionally
skipped. This is a source diagnosis, not a diagnosis of the private user profile.
The no-policy-row default enables capture; recall ignores the capture switch.
Both must change for the master-switch contract.

The isolated SQLite reproduction retains a completed decision in conversation
history while rejecting it for automatic capture. Disabled policy prevents claiming;
malformed model output is separately rejected.

Frozen public corpus: [memory corpus](../fixtures/memory-corpus.json). Deterministic
targets: all eligible exact-source candidates accepted; negatives/unsupported
candidates refused atomically; zero scope leaks, stale publications or resurrection
from forgotten sources. Matching facts must rank first for fixed cues. Corrections
replace current recall; manual corrections remain protected. Missing evidence is
disclosed and excluded. Fixture targets do not prove model reliability.

Capture initially retains existing bounds: 8 KiB source, three candidates,
512-byte exact quotes, ten-second deadline. No tools, silent retries or startup
backfill. One maintenance request at a time; waiting work coalesces per chat.
Output is bounded separately from foreground requests. Recall freezes eight index
candidates, three source excerpts, at most 2,000 estimated inserted tokens, reduced
to fit the current provider context. Existing 4 KiB memory text is a tighter bound.
Storage caps/migration costs are frozen in 21.1 before adoption.

Conversation history remains the source archive. Forget removes derived records
and tombstones sources without deleting conversations. Off stops capture and
learned recall. Re-enable does not replay history. No ambient screen/audio/image
collection. Supplied-image handling is a separate 21.5 qualification.

Live evaluation uses public bounded prompts with configured Qwen3.5-2B and harder
DeepSeek V4.1 Flash cases, preserving original selection. Capture/recall hits,
unsupported claims, scope leaks and usage are reported separately from fixtures.
Memory Off is the baseline. Universal reliability is not claimed.


## 21.1 Records, index and master switch

Schema 34 adds an atomic source lookup with insert/update triggers and cascade deletion. Existing records and saved policy are retained; a fresh profile defaults Off. Recall consults the same switch as capture. UI needs no setup form; optional legacy manual editing remains available.

Caps: 128 automatic records plus the existing 12 manual records per scope; at most 280 scoped/global candidates read. Each record retains the existing 1 KiB text, 512-byte source quote, 80-character title and bounded IDs/model. The maximum-field UTF-8 fixture is 2407 bytes per JSON record (308096 bytes for 128); source index duplicates only bounded references/quotes. SQLite page overhead is separate. Full-scope behavior skips new additions with an actionable Forget instruction; no silent eviction. Recall retains its 4 KiB ceiling. No retrieval dependency is added.

Tests cover fresh Off, saved On/Off and history across restart, transaction failure without history/schema loss, recall Off with retained inspectable data, compact light/dark policy and activity/failure recovery.


## 21.2 Useful capture and maintenance

Completed user statements can supply exact facts, decisions, reported outcomes and open work without a remember phrase. The bounded tool-free extractor emits stable typed titles and exact standalone excerpts; partial qualifiers, speculation, quoted third-party text, credentials and invented candidates are refused. Reported outcomes retain user attribution, not independent task-completion proof. Legacy literal response preferences still avoid a request.

A lazy worker runs one maintenance request at a time and queues at most 16 distinct chats, coalescing waiting work per chat. Reply completion does not wait for the model. Jobs pin source/policy and re-read current scoped records before extraction. Publication remains atomic; Off/Shutdown cancels requests and clears waiting work. Restart reports interrupted attempts without replay. Queue/full-scope failures preserve replies and explain the next action. Existing extraction bounds remain 512 output tokens, 8 KiB source, three candidates and ten seconds or lower configured limits. Separate reported usage is stored.

Global automatic storage is capped at 8192 records (maximum-field JSON payload about 19 MiB, plus bounded source index and SQLite overhead). Worst-case queued snapshots remain bounded by 17 active/waiting jobs times 280 source-linked records; no idle polling service or new retrieval dependency. Multi-project foreground requests may run concurrently with the one tool-free memory worker.

Frozen positive/negative/partial-qualifier/atomic-validation cases pass. Hung-provider admission completes within 500 ms, one request runs, 16 waiting chats coalesce, overflow skips with guidance, and Off stops without publication while replies remain saved. Full-suite legacy migration tests exposed artificially rolled-back schema fixtures; idempotent index migration preserves them, and a genuinely incompatible destination still rolls back without losing history.


## 21.3 Cue-led recall and source opening

Scoped lexical ranking uses English word cues and Chinese bigrams, with stable scope/time/ID tie-breaking. Typed factual/episodic records require cue overlap; durable preferences remain eligible. At most eight compact index entries and three exact source excerpts are inserted, with a 4 KiB text ceiling and at most 2000 estimated tokens including memory framing. Baseline context is prepared first; memory consumes only its remaining allowance. Small windows omit memory instead of making an otherwise valid question fail. Omitted counts remain inspectable.

Source availability is rechecked from retained conversation bytes before recall. Deleted/changed evidence is excluded while its historical record remains inspectable. View source performs a local scoped/revisioned read of the exact excerpt, not a model request or an unbounded conversation fetch. Its unavailable/stale error offers inspect/forget/refresh without losing the card.

All six frozen English/Chinese cues rank their matching record first; four-token memory allowance leaves the question unchanged. A new chat in project A recalls Cedar, B recalls Maple without cross-scope leakage; local source opening succeeds, cross-project opening is refused, and deleting the source excludes Cedar with an honest unavailable state. Twelve compact memory UI tests pass, including local source opening and missing-evidence recovery.
