# Brick 3.4 — Streaming agent replies

Working chats stream each model call's public assistant text, including commentary before approval. Earlier commentary moves into expandable Agent progress; successful replies retain optional `agent.steps` metadata separately from final text. Chat, trajectory and exports preserve this distinction. Hidden reasoning fields are neither displayed nor retained.

The provider port adds optional `stream_tool_turn`, defaulting to existing `tool_turn` with one completed-text emission. The OpenAI-compatible adapter requests streaming Chat Completions, function specs, `tool_choice:auto`, `parallel_tool_calls:false` and per-call `max_tokens`. It assembles indexed tool deltas and fragmented IDs/names/arguments using the [official contract](https://developers.openai.com/api/docs/guides/function-calling).

Only public text crosses the progress channel. Calls become eligible for local validation after a matching finish reason, complete object arguments, contiguous bounded indexes, function types, valid names/IDs and unique IDs. `[DONE]` without valid finish fails; clean EOF after valid finish is accepted. Unfinished frames, interruptions and explicit errors fail. Partial arguments never request approval or invoke tools. Registry, immutable folder scope and single-use approval remain authoritative.

Per-response limits: 2 MiB wire, 4096 data frames, 256 KiB per completed frame; the shared decoder retains a 1-MiB incomplete-frame buffer cap. Existing four-call/four-tool, 4-KiB argument, 128-byte ID/name, 128-KiB context, 16-KiB tool-result and whole-run deadlines remain. Public text across all calls shares one 128-KiB budget. Bounded-channel delivery is included in deadlines and Stop. No worker/dependency is added.

Each call keeps its last valid usage snapshot, including empty-choices chunks, without summing duplicates. Only explicit pre-stream unsupported-usage-option rejection permits one retry without `stream_options`. Generic rejection, interruption and unsupported format never fall back to plain chat. Side chat remains an explicit user choice.

Final text, commentary, tools and usage commit atomically in schema-6 metadata JSON. Old summaries without `steps` decode to an empty list; no migration/history rewrite occurs. Failure/Stop/exhaustion saves no partial turn and restores the draft. Completed earlier commentary may remain transiently inspectable as Not saved until another chat/run; incomplete current text is discarded. Public commentary returns to the provider with its corresponding calls.

Live model compatibility, OS input/folder-picker UAT, other platforms, accessibility and resource targets remain separate acceptance work.
