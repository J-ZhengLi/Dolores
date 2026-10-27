# Brick 3.4 — Streaming agent replies

Working chats stream each model call's public assistant text, including commentary before approval. Earlier commentary moves into expandable Agent progress; successful replies retain optional `agent.steps` metadata separately from final text. Chat, trajectory and exports preserve this distinction. Provider-supplied reasoning has a separate optional channel: a collapsed Thinking panel and local `agent.thinking` metadata retain at most a 16-KiB latest preview per model call. Full protocol reasoning stays in the provider's bounded continuation cache and does not enter public answers or diagnostic journals.

The provider port adds optional `stream_tool_turn`, defaulting to existing `tool_turn` with one completed-text emission. The OpenAI-compatible adapter requests streaming Chat Completions, function specs, `tool_choice:auto`, `parallel_tool_calls:false` and an optional per-call output allowance. Blank output omits both `max_tokens` and `max_completion_tokens`; explicit settings use the appropriate provider field. It assembles tool deltas by unsigned index label, binds stable IDs/names (including repeated identical metadata), and concatenates fragmented arguments using the [official contract](https://developers.openai.com/api/docs/guides/function-calling).

Public text and the separate bounded thinking preview cross progress channels; only public text enters assistant answers. Calls become eligible for local validation after a matching finish reason, complete object arguments, unambiguous index-to-call identity, function types, valid names/IDs and unique IDs. `[DONE]` without valid finish fails; clean EOF after valid finish is accepted. Unfinished frames, interruptions and explicit errors fail. Partial arguments never request approval or invoke tools. Registry, immutable folder scope and single-use approval remain authoritative.

Streaming limits: 2 MiB of wire traffic without decoded progress and 256 KiB per completed frame; there is no cumulative event-count limit. The shared decoder retains a 1-MiB incomplete-frame buffer cap. Tool indexes need not start at zero or be contiguous and are never used for array allocation. There is no four-call parser cap in either transport. Actual aggregate tool fields plus per-slot accounting fit 128 KiB; per-field, ID/name, context, tool-result and explicit task budgets remain. Calls in a completed batch execute sequentially under normal policy. A response stays within its 128-KiB public-text budget; durable task segments archive earlier text independently of model compaction. Request time now bounds connection startup and inactivity without decoded growth, rather than total active generation. Provider metadata and UI heartbeat ticks do not reset inactivity. Delivery stalls remain bounded and Stop is always available. Blank task time adds no overall deadline; explicit task clocks exclude approval review. No worker/dependency is added.

Each call keeps its last valid usage snapshot, including empty-choices chunks, without summing duplicates. An explicit pre-stream unsupported-usage-option rejection permits one compatibility retry without `stream_options`. The agent supervisor separately supports the bounded generation-only recovery below; it never falls back to plain chat or switches models. Side chat remains an explicit user choice.

Final text, commentary, tools and usage commit atomically in schema-6 metadata JSON. Old summaries without `steps` decode to an empty list; no migration/history rewrite occurs. Unhandled failures and Stop restore the draft with transient progress; recoverable generation failures can save a paused checkpoint as described below. Public commentary returns to the provider with its corresponding calls.

Recovery refinement (2026-10-08): already delivered public text, including an incomplete current response, remains selectable in the existing Not saved progress panel until another chat/run. Incomplete text is labeled Partial; it is never inserted into saved history or sent in a new request. The latest response replaces the oldest transient step when the three-step display bound is full; the existing public-text budget bounds retained bytes. This does not persist interrupted text across restart or claim task completion.

Live model compatibility, OS input/folder-picker UAT, other platforms, accessibility and resource targets remain separate acceptance work.

Protocol reasoning is bounded at 1 MiB per turn. Retained reasoning and public messages share the selected model input allowance for follow-up checks and compaction pressure. The cache allocation bound follows that allowance with one response of headroom; it is not a fixed 2-MiB context window. UI previews retain the latest 16 KiB with an omission label. Automatic output uses a context-planning reserve of one quarter of the window, capped at 32768 tokens; that reserve is not sent as a generation limit.

## Generation-only automatic retry

Tool-enabled agent runs can retry adapter-classified recoverable model failures
before any tool from that response is prepared or dispatched. The OpenAI-compatible
adapter opts in for disconnects/unfinished or malformed streams, incomplete object
arguments, inactivity, rate limiting and HTTP 408/500/502/503/504. Authentication,
configuration, context/allocation guards, conflicting identities and unknown
adapter errors do not retry. Tool invocation failures never enter this path.
Legacy adapters default to no retry unless they explicitly classify their errors.

At most two retries follow consecutive failed attempts. Delays are 1 then 2 seconds,
or 5 then 10 for rate limiting. Eligible interrupted/malformed tool streams can
switch to the adapter's complete-response transport after a 5-second wait (10 on
another failure). The same model, tools, reasoning and output settings are sent;
there is no fallback to tool-free chat. Successful recovery retains that transport
for this run only, with text arriving once the complete response validates.
Saved connection settings are unchanged. A gateway that cuts off both transports
can still fail; no client retry can extend its server-side deadline.
All waits are cancelable and count toward task time.
Every attempt reserves the shared/persistent model allowance; it is never refunded.
Success clears the retry streak. Stop, steering and policy/budget checks remain at
the next normal dispatch boundary. Exhaustion saves a paused checkpoint with
partial text labeled as interrupted, completed receipts and unknown failed-attempt
usage. Failed calls never enter model context; only earlier completed tool results
are retained. No task restart, tool replay, approval reuse or application-restart
retry is introduced. Tool-free side-chat and strict structured-consumer requests
retain their existing explicit recovery behavior.

`modelRetry {number,attempt,delayMs,buffered}` keeps one visible task/timer/draft and shows
Reconnecting to the model. The durable journal records retry counters without raw
response bodies and identifies complete-response recovery. Earlier partial text is
evidence, never proof an action ran. Structured summary requests keep the bounded
summary retry path; transport fallback here applies to tool-enabled generation.
