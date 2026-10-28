# Token context and model capacity

Context previews and the composer ring use estimated tokens, with a saved context-window override for each enabled model at the current endpoint. Blank capacity uses the user-requested **128K (131,072 tokens)** default; this is a local setting, not inferred provider metadata. Model IDs and the standard `/models` response do not imply a capacity. Model settings save atomically with connection settings. Existing connections and saved replies remain readable.

The lightweight counter follows Codex's UTF-8 byte heuristic, rounded upward at four bytes per token, with approximate message framing and serialized function definitions/calls included. It runs locally without a tokenizer service or model download. Provider-reported usage stays separate and authoritative for the completed request. During a run, the ring and Context panel use the current projected request estimate, then the last call's reported total when available. Each new request replaces the previous reported value with its own estimate, including changed tool schemas and compacted history. Idle inspection prepares a fresh next-message estimate. Never add usage across agent calls to measure one context window.

Active inspection reads the locally delivered usage snapshot and never dispatches
a competing preparation/model request or changes the draft. The panel subscribes
to subsequent snapshots while open. Detailed active snapshots describe token
components and their basis; they do not claim to include a complete live provider
transcript. Unknown provider usage remains estimated rather than becoming zero.

The ring and inspector compare tokens with the configured raw context window. Preparation leaves 5% headroom, then reserves the configured maximum output tokens. Both preview and sending retain the newest complete turns that fit this estimated input allowance. The draft, system instructions and tool definitions are never silently cut; if they alone exceed the allowance, preparation fails before a request. Each subsequent agent call checks its expanded context and stops locally when it exceeds the allowance, preserving call/result pairs and existing tool effects. These are heuristic checks, not a guarantee against every provider's context limit.

Normal preview/send selects saved complete turns against the configured model
input allowance before the final token check, without a fixed 40-turn/128-KiB
cutoff. Preparation uses a separate 32-MiB allocation guard, not a model-context
default. Small models still trim complete turns after guidance, schemas and
output reserve are counted. Provider usage and configured capacity remain
distinct. The separate human-input, snapshot and output allocations are not
silently expanded by changing a model.

Inline result previews remain small; complete oversized plugin results are
stored as UTF-8 journal fragments with bounded receipt retrieval. This allocation
does not reject an already-executed operation merely for returning a large body.
In-run compaction uses the configured model input allowance, including retained
provider reasoning; a fixed 128-KiB public threshold does not trigger it. Local
evidence segments roll independently without a model summary or context loss. Streaming and non-streaming tool requests use the same
configured model input allowance, including tool definitions and image estimates,
rather than rejecting serialized messages at 128 KiB. Successful metadata snapshots
the capacity, output reservation, headroom and component estimates; legacy metadata
without this snapshot remains unavailable rather than being reconstructed. Changing
model, request settings, endpoint or draft invalidates stale preview accounting.

For an existing chat's next ordinary human request, preview includes the advertised scheduling tools, enabled project integrations, response-limit guidance and the same preparation used before the first model call. Preparation does not execute a tool or contact a model. External guidance is included once so agent entry cannot change the estimate by appending duplicates. Changed state, scheduled occurrences, continuations and screenshot-control requests require their own fresh preparation; a prior preview is not a guarantee of an identical later request.

Tool instructions follow the advertised catalogue. General guidance applies to
every agent run; file discovery requires the read catalogue, file-edit guidance
requires the write catalogue, and coding/command guidance requires the matching
capabilities. Side chats retain scheduling and questions without project-file
instructions. A delegated run with fewer tools removes inherited file/command
guidance that no longer applies. Preparation stays idempotent in preview and
sending. This changes instructions, never permissions or the actual tool registry;
unavailable calls still fail before execution, without automatic retries.

See [pinned upstream comparison](../research/token-context-accounting.md).
Token estimation requires no tokenizer download and grants no tool permissions.

## Saved tool evidence — audit repair, October 11

Next-request history includes textual receipts from recent included turns beside
the corresponding assistant message, in both preview and sending. Keep at most
eight recent receipts and 16 KiB of added context per history snapshot. Each
result preview uses at most 4 KiB of UTF-8 text, explicitly marking truncation.
Name the tool, target, recorded status and literal command when present; exclude
image assets, credential configuration and unrelated metadata. Older omitted evidence is
identified as omitted rather than represented as a complete history.

Label these receipts as historical, untrusted data: they show a past outcome,
not current file contents, a new verification or permission to repeat an action.
Context budgeting counts them before deciding which complete turns fit. A saved
summary excludes its covered turns and their receipts. Stored chat text, paged
conversation display, exports and summary-source provenance stay unchanged.
Malformed saved tool metadata must fail preparation with an actionable error;
it must not become invented evidence. This repair adds no tool execution,
automatic permission, schema migration or new background service.

## Checkpoints and continuation evidence

JSON and Markdown conversation exports include each owned run's snapshot and
ordered journal events, including archived summary/context fragments. They
also report cumulative known usage from saved message metadata and archived
agent summaries, with missing/partial usage counts. Cache/reasoning counters
remain subsets; no missing counter becomes zero. These are recorded subtotals,
not invoices: unsaved, delegated and auxiliary usage can remain unavailable.
Fragment reconstruction is bounded and rejects incomplete/malformed records
with an actionable error. Export reads one SQLite snapshot without changing
history or the database schema.

During an agent run, only the latest four tool results keep their full payload
in each model request. Older successful file reads, listings, searches and
commands above 2 KiB become explicitly labeled bounded historical previews;
identical older payloads reference a later receipt. Errors and unsuccessful
commands, other tool types, images, user constraints and call/result identities
remain intact. Full messages and receipts remain in local run evidence and
exports. Capacity checks and automatic summarization use this request projection;
the model can reread targeted source when a preview omits needed detail.

After successful durable sealing, the agent also releases those old result
bodies from its working set. Full archived receipts remain intact. Request
projection alone must not leave an ever-growing raw transcript that eventually
overflows the next checkpoint's byte/fragment limits.

Automatic context compaction can repeat throughout a task; there is no two-summary
ceiling. Each summary still reserves a model attempt from the existing shared task
budget. Compaction does not reset usage, re-run effects, change permissions or
override explicit task limits. Provider-private reasoning and public messages share the selected input token
allowance. The cache allocation bound follows that allowance with one response
of headroom, rather than a fixed 2-MiB ceiling; removed calls release cached
reasoning before the next request. This never exposes private reasoning text. If recovery is unavailable, save a resumable checkpoint
before the next request. Stop, genuine provider failures and no-progress detection
remain separate from context maintenance.

Continue and in-run context restoration use at most 16 KiB of tool-evidence
previews rather than embedding every full result, argument list and diff. Keep
unresolved command checks first, then recent receipts, retaining their original
order. Mark shortened fields and omitted records; keep short literal commands
exact. Oversized commands are explicitly marked previews and must not be
reconstructed or executed from truncated text. Large partial replies also become
marked previews. Original task constraints and full saved receipts stay intact;
conversation display, Run history and exports retain the complete evidence.

Checkpoint summaries accept complete prose, Markdown or legacy JSON as untrusted
evidence. Empty, interrupted or oversized output remains a failure; JSON shape
alone is not a failure. Recoverable summary-generation failures can retry twice
with cancelable backoff; every attempt reserves the shared/persistent budget.
Completed tool actions are never replayed by recovery. Persistent failure saves
an inspectable checkpoint. See [generation recovery](long-task-recovery.md#generation-recovery).
