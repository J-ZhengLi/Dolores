# Desktop API v1

All command payload keys use camelCase. Role values use lowercase. IDs are UUID strings. All commands are available only in the bundled local desktop frontend, without HTTP routes. Connection configuration authorizes the native host to send the user's conversation context to the chosen endpoint.

| Command | Arguments | Success result |
| --- | --- | --- |
| `bootstrap` | none | `{sessions, preferences, connected, plugins}` |
| `create_session` | none | `{id, title, updatedAt}` |
| `get_messages` | `{sessionId}` | `[{role, content}]` (newest 80 messages in chronological order) |
| `delete_session` | `{sessionId}` | null; cascade deletes messages |
| `configure_connection` | `{preferences: {baseUrl, model}, apiKey}` | null; validates and saves non-secret preferences; replaces session credential |
| `generate` | `{sessionId, runId, input, onEvent}` | null after streaming and atomic persistence |
| `cancel_run` | `{runId}` | null; cancels the matching active run, otherwise no-op |

`updatedAt` is Unix milliseconds. Session list is newest-first, capped at 100. `connected` means configured in this process, not network-tested. Every restart requires saving connection settings again; hosted providers need the key again. `apiKey` may be empty for a server that does not require authentication. It is never returned or saved. Saving connection settings replaces the previous key, including with an empty string.

`onEvent` is a Tauri `Channel<DeltaEvent>`; each event is `{runId, delta}`. The first event has an empty delta and acknowledges that the native run is reserved, allowing early cancellation before networking starts. Subsequent ordered deltas contain plain text. The invocation's resolution/rejection is the authoritative terminal signal. A failed or cancelled turn is not persisted. The UI may retain its partial text explicitly labelled unsaved and restore the user's draft. Successful history includes the complete turn exactly once.

One foreground run is allowed. Connection replacement/deletion are rejected while a run is active. An unknown/deleted session rejects history/generation. Delete of a syntactically valid nonexistent ID is a no-op. Empty or >16 KiB inputs reject. Requests use newest complete turns under a 128 KiB text budget. Answers cap at 128 KiB. There is no automatic retry or truncation of completed stored turns.

URL policy: HTTPS, or HTTP loopback (`localhost`, IPv4/IPv6 loopback). URL userinfo/query/fragment are rejected; redirects are disabled. API prefix is user-provided (`/v1` commonly). Model ID must be nonempty and ≤200 bytes. URLs cap at 2048 bytes, keys at 4096 bytes and reject CR/LF. Errors include invalid inputs, storage failure, unavailable connection, network error, HTTP denial/rate limit, malformed stream, missing completion, output limit, closed consumer, and cancellation. See the native implementation for current user-facing wording; do not parse error strings as stable codes.

Custom application commands are registered with Tauri's invoke handler. Current capabilities grant `core:default` to the local `main` window, with no remote origins. This is not an isolation boundary for the compiled Rust plugins.
