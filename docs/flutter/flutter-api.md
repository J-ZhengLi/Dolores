# Flutter bridge contract

## History pagination and complete export (brick 2.2)

Bootstrap's `sessions` and additive `sessionPage` expose the same first 50 entries. `sessionsPage` returns `{items,hasOlder,hasNewer}` using a cursor `{updatedAt,id}` from the first or last displayed row. Order is `updatedAt DESC, id ASC`, including deterministic timestamp ties. Older is the default; newer requires a cursor and returns the preceding page in the same display order. Boundaries are exclusive. A deleted boundary still works. Session updates can reorder entries: this is live browsing, not a multi-request snapshot; refresh returns the newest page.

`messagesPage` returns chronological `{id,role,content}` entries, up to 80. With no cursor it loads the latest page. A positive message ID selects older (`id < cursor`) or newer (`id > cursor`) rows; newer requires a cursor. Missing sessions return `Conversation no longer exists.` UI pages replace each other and expose both direction flags, while `messages` retains its legacy bounded context behavior. No messages are deleted by paging. The default SQLite plugin supports page limits 1–100 sessions / 1–80 messages internally; the Flutter commands always use 50 / 80. Queries use a consistent read transaction for items and direction flags. The session ordering index is additive; the schema remains version 3.

`export` writes the whole saved conversation, independent of the displayed page or provider context. Formats are `markdown` and `json`; the absolute destination must end in `.md` or `.json` respectively (case-insensitive). The destination must not exist. Export streams rows from a consistent SQLite read transaction through a buffered writer into a temporary file in the selected folder, syncs it, then publishes without replacing an existing file. Failed writes drop the temporary file and leave no partial destination. A crash may leave an unreferenced temporary file; directory durability and no-clobber publication atomicity are platform/filesystem dependent. A raced/existing destination is never overwritten. Folder/permission/I/O errors are user-facing and exclude secrets.

JSON shape: `{exportVersion:1,session:{id,title,updatedAt},messages:[{id,role,content}]}`. `updatedAt` is Unix milliseconds; messages are chronological. This is an export format, not an implemented import API. It contains no connection metadata or credentials. Markdown uses role headings with literal text fences sized to preserve embedded backticks, including the title. Saved conversations may themselves contain sensitive text; the user chooses their output file. All history/export commands share the generation exclusion rule. Canceling the native Save dialog sends no export command. UI view state keeps only the 20 most recently visited drafts/scroll positions in process memory, not across restart.

The selected Flutter shell loads the bundled Rust library in the same process with `dart:ffi`. There is no HTTP server, Node sidecar, or third-party plugin ABI. All calls run in one Dart worker isolate. SQLite and secure storage stay off the UI isolate; Rust runs generation on two Tokio workers with at most two blocking workers.

## C ABI v1 and memory ownership

`dolores_call(const uint8_t *input, size_t length) -> char *` accepts UTF-8 JSON, at most 128 KiB. A null pointer, oversized input or invalid request returns an error envelope. The caller must keep the input allocation readable for the call; this trusted in-process ABI cannot validate arbitrary pointers. The result is owned, NUL-terminated UTF-8 JSON: `{ "ok": true, "result": ... }` or `{ "ok": false, "error": "..." }`. No Rust panic intentionally crosses the C ABI.

`dolores_free(char *result)` frees a result **exactly once**, in the same library that allocated it. Null is allowed. Dart frees its own request allocation separately. Never pass another allocation or an already-freed pointer.

The engine initializes once per process, on the first valid command. It uses an absolute `DOLORES_DATA_DIR` override or the shared `dev.dolores.desktop` application-data directory and `dolores.db`. Initialization attempts remembered-connection recovery. Failure produces a warning rather than blocking history. `shutdown` cancels/releases the active run; it does not destroy the process-wide store/runtime, forget saved credentials, or clear configuration.

## JSON commands

| `command` | Additional fields | Successful `result` |
| --- | --- | --- |
| `bootstrap` | None | `{sessions, sessionPage, preferences: {baseUrl, model}, enabledModels, configured, rememberConnection, hasSavedKey, connectionWarning, plugins}`; sessions/sessionPage contain the initial 50-row page; plugins lists storage and credentials |
| `messages` | `session: string` | Chronological array of `{role, content}`, at most 80 messages |
| `sessionsPage` | `cursor: {updatedAt: i64, id: string} or null` (optional), `newer: bool` (default false) | `{items: Session[0..50], hasOlder, hasNewer}` |
| `messagesPage` | `session: string`, `cursor: positive i64 or null` (optional), `newer: bool` (default false) | `{items: [{id, role, content}][0..80], hasOlder, hasNewer}` |
| `export` | `session: string`, `path: absolute string`, `format: markdown or json` | `{messageCount: u64}`; complete saved conversation in a new file |
| `delete` | `session: string` | `null`; cascades saved messages |
| `configure` | `preferences: {baseUrl, model}`, `apiKey: string or null` (optional), `rememberConnection: bool` (optional, default false), `enabledModels: string[]` (optional, defaults to active model) | `null`; validates, saves preferences/choices and optionally a secure key; no network request |
| `listModels` | `baseUrl: string`, `apiKey: string or null` (optional) | Sorted unique model ID array; draft discovery makes a GET request without changing configuration |
| `selectModel` | `model: string` | `null`; switches to an enabled model, saving active model/metadata together without rotating the key |
| `recoverConnection` | None | `null`; reloads saved settings/key; failures clear the active connection and set a warning |
| `forgetConnection` | None | `null`; removes the saved key/reference and active connection, preserving preferences/history |
| `start` | `id: u64`, `session: string or null`, `input: string` | `null`; reserves the run before asynchronous preparation |
| `poll` | `id: u64` | At most 32 run events; stale/missing run returns `[]` |
| `cancel` | `id: u64` | `null`; stale/missing run is a no-op |
| `shutdown` | None | `null`; cancels the current run and drops its receiver |

Dart uses small, monotonically increasing run IDs. One run is active at a time, including the interval until its terminal event is drained. While active, only `poll`, `cancel` and `shutdown` are accepted; other commands fail with `Stop the current response first.`

## Remembering and recovery

An explicit string replaces the key; `""` selects a keyless connection. Null/omitted `apiKey` reuses an active or saved key only for the exact same base URL. If none exists it selects an empty key; it never retrieves a key from another endpoint. `configured` means locally configured, not network-tested. `rememberConnection` means metadata exists; `hasSavedKey` means this process successfully saved/restored a secure key. The key and opaque credential ID are never included in responses.

With remembering enabled, SQLite schema version 3 holds nonsecret preferences, model choices and an opaque UUID. Migration adds the model-choice table without discarding earlier metadata/history. The credentials plugin uses explicit keyring 3.6.3 native backends: Windows Credential Manager, macOS Keychain, Linux Secret Service. Credential names include a hash of the canonical data directory; copying a DB to a different directory does not reuse its key. Key material includes an endpoint binding inside secure storage. Keyless local connections can be remembered without accessing secure storage. There is no plaintext or mock-backend fallback. If secure storage is locked/unavailable, unlock and retry, re-enter the key, or choose a connection for this launch only.

SQLite atomically updates preferences and their reference. Vault and SQLite cannot share a transaction: a new key is written first, failed metadata writes attempt deletion, and a successful replacement attempts deletion of the previous key. A crash or failed cleanup can leave an unreferenced secure entry; cleanup failure is reported without exposing the secret. Forget deletes the saved key before clearing metadata; a locked vault rejects forgetting, while a later metadata failure leaves the active connection cleared and reports the failure. These operations are serialized in the worker. History is not encrypted by this feature. Changes made by another shell invalidate restoration when preferences differ.

## Model discovery and selection

listModels sends GET <baseUrl>/models using validated draft settings and an explicit or same-endpoint active/saved key. It does not save settings or require a model name. Unavailable, unsupported, malformed or empty listings return sanitized errors; manual IDs remain supported. The request is bounded to 20 seconds, 1 MiB and 512 rows. IDs are nonblank, at most 200 UTF-8 bytes, with no control characters; duplicates are removed and results sorted. Listing does not guarantee text-chat capability for every model.

enabledModels accepts 1–32 IDs, including the active model. Choices are saved atomically with preferences/reference, even for launch-only connections. selectModel requires a configured provider and an enabled ID. It reuses the provider client/key and updates preferences/remembered metadata together, without rotating the vault entry or changing history/draft. Old records without choices expose their active model. Connection changes replace choices for the endpoint. Both commands are excluded during generation; there is no idle fetching.

## Events and completion

- `{type: "started", id, session}` after session/history preparation.
- `{type: "delta", id, text}` for streamed Unicode text.
- `{type: "done", id, answer}` after a successful atomic complete-turn transaction, or `{type: "done", id, error}` on failure/cancellation.

The event queue and text channels are bounded to 32 entries. Dart drains only while a run is active, scheduling the next poll 25 ms after the previous one resolves; there is no idle polling. Cancellation interrupts a producer blocked on forwarding. Deltas are drained before completion. A late Stop after the atomic transaction starts loses to successful completion. Failures restore the unsaved draft; partial turns never become durable history.

The core's 16 KiB input, 128 KiB context/output and 40-turn history limits still apply. Provider URL/model/key validation and sanitized errors are shared with the other shells. Native pointer/IME interaction, accessibility and macOS/Linux packaging are separate acceptance checks.
