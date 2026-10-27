# Long-task recovery — brick 6.4

The selected Flutter shell saves progress when the provider explicitly reports its output limit, the agent reaches its model/tool step limit, a tool-enabled model stream stalls, or an explicit active-task clock expires. The response is labeled paused rather than complete. It retains text, usage, public model steps, bounded provider thinking previews and completed tool receipts; unfinished calls are discarded. A model that uses its output allowance only for reasoning gets an honest placeholder instead of invented visible work. Stop, permanent/ambiguous protocol errors and tool execution failures retain explicit recovery. Adapter-classified transient generation failures can retry before tool dispatch; exhaustion saves a paused checkpoint. File effects may survive these failures independently in Changes.

## Generation recovery

Working agent requests and internal context-summary requests can retry a recoverable provider response twice with cancelable backoff, without repeating completed tools. Every attempt uses the same task and cumulative budgets. Partial text stays labeled as interrupted; incomplete calls are discarded. Authentication/configuration errors, allocation guards and tool execution failures do not auto-retry. See [streaming and retry contract](streaming-agent.md#generation-only-automatic-retry).

For eligible tool-stream interruptions, recovery waits and requests a complete
non-streaming response with the same model/tools/settings, retaining that transport
for the current run. This can bypass a broken SSE route; it does not lengthen a
provider/gateway deadline. Failure of both routes still saves a checkpoint after
three generation attempts. No completed tool is dispatched by transport recovery.

Compaction begins near 80% of the model input allowance after output headroom,
counting public context and retained provider reasoning. A fixed public-byte
threshold, tool count or model-call count does not force a summary. Evidence
pagination remains local: sealing older receipts/text preserves the active model
context and spends no model request. Actual context pressure still compacts at a
settled boundary, preserving instructions and archived full receipts.

## Continue

Continue appears on the latest saved paused assistant message. Send or clear an existing draft first. The host validates the source ID and exact recovery input before starting and again during execution. The continuation's message pair is appended only if that source remains latest inside the same SQLite transaction. Old IDs, another session and modified recovery input are refused. A failed or cancelled continuation leaves the original saved segment available; it does not restore the synthetic recovery input into the composer.

The next run uses the currently selected model, request settings, working folder and enabled guidance/skills/tools. The last user context message carries JSON containing the original task, saved partial response and accumulated completed tool receipts. The host labels these fields untrusted and instructs the model to preserve completed effects, inspect current files when needed and request fresh tools/approvals. No pending call is resumed and no old approval is reused. This instruction helps models avoid duplicate work; it does not guarantee every model will follow it. Each new tool invocation still needs a separate decision.

A continuation is its own bounded turn. Repeated pauses retain the original task and accumulated receipts, while each visible response stays in history. After a new ordinary message, the old paused card remains visible but cannot be continued using its stale ID. A completed continuation removes the latest Continue action. Automatic preference learning skips paused replies, avoiding an extra request after incomplete work.

## Budgets and compatibility

In-run context checkpoints accept completed prose, Markdown or legacy JSON
summaries as untrusted evidence. JSON shape is not a continuation requirement.
Literal user instructions and full archived receipts remain independent of the
model summary. Empty, interrupted, oversized or failed summary generation still
preserves an inspectable checkpoint without replay or an automatic retry loop.

Output defaults to **provider-selected allowance**; existing numeric settings remain unchanged. Defaults retain **180 seconds without decoded progress**, with **Automatic model/tool cycles** instead of four-call/four-operation stops. Automatic root tasks with durable recovery refresh context and roll internal evidence segments without ending the visible task. They no longer pause at the old 64-call/128-tool defaults; explicit numeric settings still apply. Failed compaction, storage failure and genuine context insufficiency still save an inspectable checkpoint. Explicit Continue starts a fresh segment without switching models or replaying calls. In provider-default output mode only, one truncated response with useful public text can continue within the same run. Its partial text is retained, truncated calls are discarded, and fresh calls receive normal approval. Explicit output caps, reasoning-only exhaustion, unresolved commands/children and repeated truncation pause. This is not a transport retry or an indefinite background loop. Recovery prompt construction retains the **96 KiB** last-resort bound for the original task plus bounded previews; large receipt collections alone are no longer a refusal reason; normal context token/byte guards can refuse sooner. Refusal keeps the saved response and suggests a reviewed summary in a new chat. Repeated tool failures pause separately with saved evidence; see [task budgets](task-budgets.md).

The provider's chat-specific outcome distinguishes explicit `length` from interruption and preserves trailing usage. Both streamed tool arguments and nonstreamed pending calls are discarded on `length`. Strict structured consumers such as skill drafts still reject truncated output instead of accepting partial JSON. Existing provider plugins inherit a strict default; storage plugins must implement atomic continuation to support it. Optional paused metadata fits existing JSON storage and exports with **schema 16 unchanged**. No new runtime, dependency, resident process or background retry exists. Flutter is the maintained desktop host.

## Checks and limits

`scripts/test-continuation.py` uses isolated synthetic data, the bundled bridge, SQLite and its own loopback HTTP fixture. Run `save`, then `restore` in separate processes with the same fresh absolute directory under `output/`. It covers repeated output limits, stale/wrong-session sources, changed recovery input, cancellation after visible text, three approved file creations followed by a step-limit pause, explicit continuation that reads saved work, unchanged files, restart and JSON export provenance. Unit checks add atomic failed commits, source reuse refusal, bounded untrusted prompts and incomplete streamed arguments with trailing usage. Compact light/dark widget checks cover drafts, failed retry, duplicate exclusion and missing connection.

Windows release diagnostics exercise the rendered Continue action at wide and compact sizes. Authorized Qwen and DeepSeek probes intentionally use small limits on synthetic text tasks; both retain two paused segments after explicit continuation. They establish recovery behavior, not completion of those tasks or general coding competence. The exact game-building failure has since been reproduced in the normal app; see [response recovery qualification](../qualification/response-recovery.md). Native keyboard/IME UAT, macOS/Linux runtime and representative low-end resource acceptance remain open.

Internal segments do not call Continue, create synthetic user turns, consume manual segments or resume after restart. See [automatic continuation](automatic-task-continuation.md).
