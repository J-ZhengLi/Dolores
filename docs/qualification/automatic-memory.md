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
