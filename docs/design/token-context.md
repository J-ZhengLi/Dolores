# Token context and model capacity

Context previews and the composer ring use estimated tokens, with a saved context-window override for each enabled model at the current endpoint. Blank capacity uses the user-requested **128K (131,072 tokens)** default; this is a local setting, not inferred provider metadata. Model IDs and the standard `/models` response do not imply a capacity. Model settings save atomically with connection settings. Existing connections and saved replies remain readable.

The lightweight counter follows Codex's UTF-8 byte heuristic, rounded upward at four bytes per token, with approximate message framing and serialized function definitions/calls included. It runs locally without a tokenizer service or model download. Provider-reported usage stays separate and authoritative for the completed request. The ring shows the last call's reported total when available, otherwise the saved initial estimate; opening the inspector prepares a fresh next-message estimate. Never add usage across agent calls to measure one context window.

The ring and inspector compare tokens with the configured raw context window. Preparation leaves 5% headroom, then reserves the configured maximum output tokens. Both preview and sending retain the newest complete turns that fit this estimated input allowance. The draft, system instructions and tool definitions are never silently cut; if they alone exceed the allowance, preparation fails before a request. Each subsequent agent call checks its expanded context and stops locally when it exceeds the allowance, preserving call/result pairs and existing tool effects. These are heuristic checks, not a guarantee against every provider's context limit.

The input, tool-result and output byte guards remain memory/resource limits. The
agent's 128-KiB public working-set target triggers compaction; it is not a second
provider context window. Streaming and non-streaming tool requests use the same
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

Automatic context compaction can repeat throughout a task; there is no two-summary
ceiling. Each summary still reserves a model attempt from the existing shared task
budget. Compaction does not reset usage, re-run effects, change permissions or
override explicit task limits. Provider-private reasoning continuation is bounded
by bytes rather than a sixteen-entry counter. Its pressure can trigger the same
safe-boundary compaction before the private allocation guard is reached, without
exposing reasoning text. If recovery is unavailable, save a resumable checkpoint
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

Checkpoint summaries still require valid JSON with the known string-array
fields and an 8-KiB input bound. Item-count and individual-string targets are
writing guidance, not independent failure gates. Blank entries are ignored.
An empty remaining list becomes a host reminder to verify the original objective
and unresolved outcomes; it never establishes task completion. Malformed JSON,
unknown fields and actual oversized input remain explicit recovery cases.
