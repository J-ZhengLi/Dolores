# Automatic memory — milestone 21

## 21.0 Audit and frozen evaluation

The baseline captures the latest user message after a successful, unpaused turn.
The pre-21 `automatic_memory::learn` claimed a durable message watermark, obtained the saved
policy and scoped preferences, then uses literal response-style extraction or a
tool-free model request. It waited before sending `done`. Sources belong
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


## 21.4 Correction and Forget

Schema 35 retains at most four previous versions per record, excluded from recall. Explicit corrections replace an automatic fact with the same subject; reported completion resolves matching open work. Manual corrections disable automatic replacement. Forget cancels active/waiting maintenance, deletes the record, source index and revision history, and clears displayed excerpts. Retained source-chat watermarks prevent older evidence from returning even through a future explicit catch-up; later statements remain eligible. Original conversations are retained separately. Watermarks contain IDs only and cascade on source-chat deletion; four maximum-field historical records add at most 9628 payload bytes per current record, plus SQLite overhead.

Five store audit tests cover correction provenance, pending publication after Forget, restart replay refusal, later fresh capture, open-work resolution and manual priority. All 281 core/store/bridge tests and 12 Flutter memory tests pass.


## 21.5 Explicit shared images

The existing selected image-capable model creates a bounded description and uncertainty label for at most one explicitly supplied image per completed interaction. Asset digest, original message/chat, model and scope are retained; captions are inferences, never exact user quotations. Attachment bytes/digest and policy/source/Forget state are revalidated at atomic publication. Images share the 128/8192 automatic-record caps; no duplicated image bytes or ambient capture is added. Optional asset metadata adds up to about 400 UTF-8 bytes per indexed image record.

One extra tool-free image request is allowed within 512 output tokens and ten seconds or lower configured limits, after text extraction, with the same single maintenance worker. Missing/unsupported/malformed image output preserves valid text capture and the completed reply. Relevant recall reopens at most one retained scoped image only with enabled image support and enough context; otherwise its uncertain caption stays useful. Source opening displays the actual retained image locally.

Two native recall tests cover malformed uncertainty, exact asset source, scoped recall, small-window omission, missing-source recovery and Forget. All 282 native tests pass. Normal Windows release build passes. Packaged save/restore fixture passes with six local requests and zero live requests: actual image wire input, later cross-chat pixels, local source opening, malformed caption preserving a valid text decision, restart, disabled image support, deleted source and Forget.


## 21.6 Final reliability qualification (2026-10-07)

21.0–21.6 are delivered as one authorized milestone batch with separate brick
commits. Final verification: 286 core/store/bridge tests, 310 Flutter tests,
analyzer, Clippy with warnings denied and maintained normal Windows release build
PASS. Four compact/wide light/dark history/Forget tests pass; saved widget renders
were inspected. Diagnostic renders do not establish physical native input.

Packaged save/restore fixtures pass in separate processes: capture uses 30 local
requests, six-cue recall uses 19, image qualification uses nine; restore uses zero.
Zero live requests in fixtures. Six correct first-ranked matches, zero negative
captures/project leaks and six deleted sources excluded. Failure/recovery includes
malformed/invented/oversized output, lower context/deadline, provider denial,
queue saturation, interruption/Off, restart, manual priority/Forget and missing
assets. Image tests exercise actual wire pixels, unused metadata, literal-shaped
fact plus image, and malformed captions retaining text and reported usage.
Requests already at 16 images or 8 MiB omit recalled pixels without losing the
caption/current input. Same typed folder subject supersedes its All-chats record;
disabled folder evidence does not shadow it. A queue fixture now has a timeout
and uses a model-path decision instead of a locally extracted fact.

### Configured live evidence

Public synthetic prompts use configured Qwen3.5-2B and DeepSeek V4.1 Flash through
the packaged host. Disposable profiles copy configuration only, not conversations
or assets. Vault credentials are read into memory, never persisted in artifacts.
Original configuration and table hashes remain unchanged. Foreground and memory
requests have 512-output-token/ten-second ceilings or lower configured limits;
no automatic retries or model substitution.

| Final case | Result | Meaning |
| --- | --- | --- |
| Qwen Memory Off | No Cedar-742 recalled | No source; not a positive capture hit |
| Qwen fact and fresh-chat recall | PASS | Exact local capture plus real Qwen recall, not model-based extraction proof |
| DeepSeek correction / projects A and B | PASS | Exact answers required; refusal mentioning a name is a miss; scope leak False |
| DeepSeek supplied image / fresh-chat recall | PASS | Requires saved caption and supported description; refusal/absence is a miss |

Final `output/m21-live-complete/receipt.json`: ten foreground turns, reported
2304 input / 394 output / 2698 total tokens.
Image maintenance: 283 input / 75 output /
358 total. Local facts consume no extraction request. Missing usage
is unknown, never zero. Earlier live responses exposed Qwen paraphrase/category
misses, unused DeepSeek caption metadata refusal and global/project conflict.
Quote-only extraction derives stored text from the exact validated quote. A tiny
identity grammar captures project codename/name, default branch and package name
locally; other prose remains model-dependent. Mixed text/image jobs retain selected
caption-model provenance. Only validated caption fields persist; bounded unused
metadata cannot affect source, scope, grants or tools. Explicit truncation fails.
Rejected captions now retain consumed reported usage and validation guidance.

Historical receipts stay separate: `m21-live-qualified` missed Qwen but passed
DeepSeek model-based fact/correction and image capture. `m21-live-final` and
`m21-live-trace` missed Qwen/image extraction. `m21-live-release` captured Qwen
locally but project B refused a conflicting global fact and image capture missed.
`m21-image-live-probe` identified unused `title_note` metadata. Earlier keyword-only
metrics counting refusals were corrected to misses. These failures are not erased
by the final recheck. Across qualification: 70 packaged live requests and three
direct format probes (diagnostic only). Some earlier failed-image usage was not
retained; no exact aggregate token total is claimed from incomplete counters.

### Cost and original profile

`output/m21-live-complete/cost.json`: forty local context calls per policy, zero
provider requests. 4 records / 2267 JSON
payload bytes; whole fixture database 475136 bytes
includes configuration and history. Off p50/p95
0.146/0.194 ms; On
0.406/0.644 ms. On uses 2
entries and 797 memory-text bytes. One public workload on this
laptop is not sustained/max-capacity performance proof. Queue/storage/context
ceilings stay frozen. Images add references, not duplicated pixels; their existing
approximate 4096-token allowance is separate from the 2000-token memory-text bound.

Normal main-entry launch: schema 35; all 42
original table hashes unchanged, only new tables memory_forget_watermarks, memory_source_index, memory_versions.
Receipt `output/m21-normal-handoff.json`. Provider selection/settings, history
and prior memory policy are preserved; receipts contain no private transcripts.

Remaining gaps: Qwen model-based extraction outside the tiny literal grammar is
unreliable in observed cases. Chinese/model-diverse/general task competence is not
proved by six deterministic cues. Captions remain uncertain inferences. Models
can also misstate that saving requires a memory-edit tool before background
capture finishes; the saved Memory activity is authoritative, not that acknowledgement.
Conservative credential/intent filters are not universal classifiers. No old-history Catch up
UI, embeddings, ambient recording, physical IME/accessibility, other-platform or
sustained/full-capacity qualification is claimed. Milestones 22–24 remain future.
