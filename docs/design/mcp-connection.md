# Reviewed MCP connections

Brick 6.1 introduced local stdio MCP tools; brick 6.3 supports up to four saved servers per working folder, sharing two active external tool slots through a compiled `ToolPlugin` adapter. Flutter remains the desktop shell; the Rust bridge coordinates reviews and storage, while `dolores-tools-mcp` owns the protocol and process lifetime. No server, runtime or package is bundled, installed or downloaded.

The Windows run catalog allows at most 24 registrations: up to 21 current host tools,
one optional browser tool and the existing two external slots. This catalog
bound does not increase per-turn calls, task budgets,
external slots, argument sizes or process lifetime. Duplicate aliases and excess
registrations are refused before model dispatch.

## Desktop flow

In a saved Project or Temporary chat, open **Settings → Connections**. Select a saved connection or choose **Add connection**. For a **Local program**, enter a short name, choose an installed direct executable and add its literal arguments individually. On Windows choose an `.exe`; for a JavaScript server, choose an installed Node executable and put the existing server script path in the first argument. A shell command or `npx.cmd` is not a direct executable. No variable expansion or shell fallback occurs. Remote endpoints are described below.

**Inspect server** is an explicit launch action. Its disclosure explains that the program runs with user OS permissions, may access files/network or change them during startup, and is not sandboxed by tool approval. Inspection initializes the server, discovers a bounded catalog, then closes it. Select one or two tools and choose **Enable** to save the reviewed connection for that folder. Editing launch fields discards the review; inspecting never enables tools. A previously enabled connection remains unchanged until a successful replacement, Disable or Forget. Side chats have no MCP setup or tools.

Each chat operation shows the server label, original tool name, revision and complete selectable JSON arguments, with **Run once** and **Deny**. Approval starts a new server, rechecks its reviewed metadata, sends exactly one call and closes it. Deny starts nothing. Results stay literal in chat/trajectory, including explicit `isError` display. Arguments, provenance and results persist with a completed reply and exports. Effects can remain after Stop or later reply failure; external actions are outside the file-change journal.

## Protocol and lifetime

The adapter uses newline-delimited UTF-8 JSON-RPC over a supervised child's stdio, following the official [transport specification](https://modelcontextprotocol.io/specification/2025-11-25/basic/transports). It sends `initialize`, checks negotiated version and tools capability, then sends `notifications/initialized`. Supported versions are 2025-11-25, 2025-06-18, 2025-03-26 and 2024-11-05. Graceful close drops stdin, waits briefly and terminates remaining supervised processes, consistent with the [lifecycle](https://modelcontextprotocol.io/specification/2025-11-25/basic/lifecycle).

`tools/list` follows bounded pagination. `tools/call` retains text blocks and the error flag; this follows the ordinary synchronous [tools contract](https://modelcontextprotocol.io/specification/2025-11-25/server/tools). Task-required tools are omitted. Server instructions/icons, tool annotations and unknown metadata do not become host instructions or permissions. Descriptions, schemas and results are untrusted model data. The model receives stable connection-scoped `mcp_tool_<32-hex-id>_<slot>` aliases; migrated legacy connections retain `mcp_tool_1` / `mcp_tool_2`. It receives original names in descriptions, and the selected object schemas. It receives neither launch paths nor launch arguments. The server validates semantic input arguments; Dolores validates their bounded JSON object shape and binds the exact approved value.

Local client capabilities are empty. Ping requests are answered; sampling, roots, elicitation and other server requests receive method-not-supported. There is no general environment editor, resource/image/audio loading or task execution. Invocation follows the chat's access policy; enabling a connection alone grants no tool access. Changed-tool notifications require another inspection. Structured-only results are not exposed as structured model data in this adapter.

Processes start only during an explicit inspection or approved invocation. Specification lookup, request preparation, context inspection, enable/disable, restart and idle chat do not launch a process. Connections are intentionally stateless across calls: servers requiring a persistent session are not accepted by this first flow. Repeated startup costs and external-server resource use remain server dependent.

Windows reuses the command tool's isolated inherited-handle list, suspended-start job assignment and hidden child process. Unix uses a process group and nonblocking output. These supervise lifetime, not filesystem/network access. A server can escape intended application behavior with user privileges; Unix processes can also leave their process group. No adversarial process sandbox or total resource quota is claimed.

## Bounds and freshness

| Item | Limit |
| --- | --- |
| Connection/catalog | Four saved connections per folder, two active external tool slots shared across all enabled servers; 32 discovered tools, four pages, 32 KiB catalog |
| Tool definition | 128-byte name, 2048-byte description, object input schema; 8 KiB combined |
| Launch | 128-byte label, absolute executable, 32 literal arguments of at most 4096 bytes; 48 KiB total |
| Operation | 30 seconds including hashing, startup, discovery and call; no retry |
| Protocol | 64 KiB frame, 256 KiB stdout, 64 KiB discarded stderr, 64 server requests/notifications |
| Invocation/result | 4 KiB argument JSON, 8 KiB combined text; 16 KiB encoded host result |
| Default turn limits | Four tool operations / four model calls; task settings and continuation accounting remain independent of catalog registration capacity |

SHA-256 covers the executable (up to 128 MiB) and each existing direct file argument (up to 4 MiB). Files are checked before/after inspection, before saving and before invocation. The approved call also checks protocol, server name/version and all selected tool names/descriptions/schemas before `tools/call`. Changed launch files or selected metadata require review. This is freshness detection, not atomic executable binding: it cannot prevent a change between checking and launch, and does not hash package dependencies, dynamically loaded files, wrappers' indirect inputs or the entire runtime environment. Trust the installed server and its dependencies.

A five-minute opaque review binds the originating saved chat, folder, previous connection snapshot and inspected launch/catalog. Saving accepts only one or two distinct reviewed names; a successful compare-and-swap consumes the review. Failed writes preserve the review for an explicit retry. Settings refresh, edits, Stop, discard and shutdown invalidate it. Capacity refusal preserves it: use the inline **Disable** beside another enabled server to free a slot, then retry **Enable** without switching forms or inspecting again. Disable/Forget of that other server preserves the pending review; changing the reviewed server invalidates it. Disable advances the revision and removes advertised tools; Forget revokes saved credential entries and deletes the local connection, leaving external files intact.

SQLite schema 15 introduced `mcp_connections(root,data)`. Schema 16 atomically moves that configuration into `mcp_connections(root,id,data)` with a composite primary key, copying original JSON unchanged under `legacy`. Other tables, legacy aliases, credential references and historic provenance remain intact. Saved launch fields and tool metadata are unencrypted alongside application preferences; keep secrets out of them. Provider credentials retain their separate native-vault flow. Brick 6.2 adds the explicit MCP credential flow below without a schema migration.

## Explicit credentials (brick 6.2)

The official [authorization specification](https://modelcontextprotocol.io/specification/2025-11-25/basic/authorization) uses environment credentials for stdio servers; HTTP OAuth is a separate future transport. Add up to eight **Credentials** rows before inspection. Names use uppercase letters/digits/underscore, at most 64 bytes; duplicate names and process/runtime settings such as PATH, HOME, LD_PRELOAD and NODE_OPTIONS are refused. Values are masked, nonempty, at most 4 KiB each / 16 KiB total (the OS vault may impose smaller limits). No host API-key variables are discovered or inherited. Values are sent to the explicitly selected local program during **Inspect server** and each approved call; trust that program and its dependencies.

Inspection holds values in an opaque, short-lived review; it does not write the vault. **Enable** writes generated entries through the existing OS credential plugin, then saves names and opaque UUID references in SQLite. No credential value is returned in settings/review/approval, persisted in launch fields/history/exports, or sent to the model as configuration. Fields containing an exact configured key are refused before launch. Native vault records bind each key to the canonical folder, connection identity, exact launch fields, reviewed file hashes and environment name. Missing connection identity in a legacy vault record defaults to `legacy`. Provider keys retain separate identities and cannot be reused as server credentials. No plaintext fallback, installation, idle process, automatic retry or wider permission is introduced.

For a saved name, leave its value blank to reuse it; a new/renamed row needs a value. Changed executable, arguments, label or file hashes require entering the key again before inspection. Missing/locked/misbound keys fail before startup; unlock storage or enter a replacement, then inspect and enable. Context/specification/preparation/restart do not read MCP vault entries. Approval lists the credential names that the server receives, binding those names to the exact one-use request. **Disable** retains keys for a future review. Remove a row and enable a fresh review to revoke that binding. **Forget** disables tools first, deletes active/retired keys, then removes configuration; on vault failure it keeps a disabled configuration with references and a clear retry message.

Vault and SQLite cannot commit atomically. A failed save deletes newly written entries and retains the review for an explicit retry. Rotation saves old opaque IDs as bounded cleanup references before retiring them, so cleanup failures remain recoverable through another explicit Enable or Forget. Already deleted IDs are harmless; up to 32 retirement references are retained. If rollback cleanup itself fails while SQLite is unavailable, unused OS-vault entries may remain and the error reports this; no distributed transaction or guaranteed cleanup after a process crash is claimed. Removing the application data directory outside Dolores also cannot revoke its vault entries automatically.

Raw protocol errors and stderr remain private. Initialization/catalog JSON containing any exact configured value is refused, including nested schema keys. Exact credential text echoed in tool-result text is replaced before it reaches the model/history/export. This is bounded exact-value containment, not a sandbox or general data-loss prevention: a trusted server can transform, encode, split or transmit a key using its account permissions. Native/Dart process memory necessarily holds values while configuring/reviewing/running; secure erasure is not claimed. Closing/editing/Stop invalidates review; the review expires for Enable after five minutes. Ordinary saved launch/tool metadata remains unencrypted; never put unrelated secrets there.

## Multiple connections (brick 6.3)

Each saved connection has an independent UUID and revision, credential scope and one-use approval. Identical original tool names across servers are supported. Disabling or forgetting one server never renumbers another server's aliases. A failing invocation returns a tool error and the agent can continue with another server within its existing four-call/four-operation bounds. No automatic retry or limit increase is added.

The server list shows each enabled/disabled state and the shared slot count. Add server refuses the fifth saved entry; tool-capacity errors explain how to free a slot and retain the current review. Disabled entries retain configuration/keys and require a fresh inspection to enable. Forget removes only that connection and its credential entries. An enabled sibling is unaffected.

## Remote endpoints (EH09.2)

### OAuth protocol foundation

The internal OAuth layer now supports protected-resource metadata, OAuth/OIDC
issuer discovery, public-client registration, S256 PKCE, callback validation and
token exchange/refresh. It follows the current
[discovery requirements](https://modelcontextprotocol.io/specification/2026-07-28/basic/authorization/authorization-server-discovery),
[registration choices](https://modelcontextprotocol.io/specification/2026-07-28/basic/authorization/client-registration)
and [authorization security requirements](https://modelcontextprotocol.io/specification/2026-07-28/basic/authorization/security-considerations).
On Windows, Connections also offers **Browser sign-in (OAuth)**. **Sign in**
discovers the resource's challenged or well-known metadata and opens the default
external browser. One foreground attempt owns a dynamic IPv4 loopback callback
listener for at most five minutes. It accepts at most eight bounded requests,
with a three-second read deadline per request, exact callback path/Host and state/
issuer validation. Stop, disposal and shutdown release the listener; late replies
cannot save or enable a connection. Callback pages contain only static text.

Discovery checks the resource and exact issuer identity, supports challenged
metadata and well-known fallback, and requires advertised S256/public-client
support. Registration prefers issuer-bound pre-registration, then a configured
hosted client metadata document, then compatible native dynamic registration.
No Dolores client document is published or assumed. Native redirects use an
IP-literal HTTP loopback address; hosted client metadata permits its port to vary
while preserving the address and callback path. Remote authorization endpoints
require HTTPS; HTTP authorization fixtures require an HTTP loopback MCP target.

Each network phase has one 30-second deadline, cancellation, twelve requests,
32 KiB per JSON document and 128 KiB cumulative response bytes. It follows no
redirects and retries no request. Callback URLs are limited to 8 KiB, each token
to 4 KiB, scopes to 2 KiB/32 names and advertised issuers to eight. Sign-in
state/verifier remain ephemeral for at
most five minutes; a code-exchange attempt is consumed once. Callback state,
duplicate parameters and exact decoded issuer are checked before success or
error handling. Authorization and token requests carry the resource identifier.
Tokens and raw authorization errors have no diagnostic serialization. A failed
refresh preserves the prior in-memory tokens without replaying an MCP tool;
it cannot undo a rotation already performed by the server. Token persistence
stays inside the MCP credential boundary and native vault, bound to canonical
folder, connection, exact target, issuer, resource, public client and purpose.
SQLite contains only opaque credential references; browser URLs, callback data
and token values never enter foreground responses, history or model context.
Vault payloads retain the existing 16 KiB bound; native storage may impose a
smaller limit and returns an actionable refusal without plaintext fallback.

Credential resolution and refresh serialize per saved credential. Before a
refresh request, save a durable pending marker; save the rotated tokens before
using them. A crash, cancelled/revoked refresh or failed rotation save retains
inspectable login uncertainty and requires **Sign in again**. Nothing repeats a
dispatched refresh or business tool to recover output. A successful refresh can
remain saved after Stop; it does not enable a new catalog or broaden approval.
Refresh uses the retained protected-resource metadata address, exact issuer,
client and token endpoint. Changed metadata requires fresh sign-in. Config edits
use the existing foreground owner/CAS boundary; distributed atomicity is not claimed.

### Available remote controls

Choose **Add connection → Remote endpoint**, enter a connection name and its MCP
URL, then choose **No authentication**, **Bearer key** or **Browser sign-in (OAuth)**. Keep a key in the masked
field, never the name or URL. Use **Inspect endpoint** to contact it and review
its bounded tool catalog. Inspection sends the key to that endpoint but does not
save it or enable a tool. Search filters names/descriptions locally; selected
tools remain selected when hidden by a filter. The Enable button shows the
selected count and saves only those reviewed names. Local and remote connections
share the existing four saved connections and two enabled tool slots per folder.
Saved connection type is fixed in the form; add a connection for another type.

Remote URLs use HTTPS, with HTTP available only for explicit loopback addresses.
URL credentials/fragments and redirects are refused. Loopback traffic bypasses
proxies to address this computer directly. The current `2026-07-28` Streamable
HTTP binding uses stateless discovery and mirrored request metadata; handshake
bindings `2025-11-25`, `2025-06-18` and `2025-03-26` retain transient sessions.
JSON and request-scoped SSE responses are supported. Deprecated two-endpoint
HTTP+SSE, background subscriptions, sampling, roots and MRTR continuation are not
enabled. Unsupported header annotations exclude that tool with an inspectable
reason. Server metadata remains untrusted data, not host instructions.

Loading settings makes no network request and shows **Not checked**. **Recheck
connection** reports the latest inspection in the current window, including
failure or cleanup warnings; it is not an uptime monitor. A failed or stopped
inspection keeps the previous saved connection. Expired keys require a new key
and explicit inspection. Leave a saved bearer field blank to reuse it only for
the exact saved target. Changing the endpoint or label requires entering the key
again. Enable sends a token and selected names, not another copy of the key.
Both transports share the vault-write/configuration-CAS recovery path: failed
saves preserve the review and old connection, while successful rotation retires
the old key. Disable/Forget retain the existing recoverable cleanup behavior.

For OAuth, Sign in produces the same tool review; **Enable** separately saves the
login and selected catalog. Failed Enable retains the review for explicit retry.
**Recheck connection** reuses the saved login and refreshes it once if expiring,
before any tool call. It never opens a browser automatically. **Sign in again**
starts a fresh grant and retains the old saved configuration until Enable succeeds.
Sign-in options accept an exact issuer and supplied public client ID, or a real
hosted client metadata URL; otherwise compatible dynamic registration is used.
Multiple advertised issuers require an explicit choice. Supplied registration
values must be entered again for fresh sign-in; no hosted Dolores URL is invented.
Sign-in itself neither enables tools nor changes their permissions. Other
platforms have no native browser sign-in qualification.

Remote bearer records bind the canonical folder, connection identity, exact
target and credential purpose. Values allow at most 4 KiB of visible ASCII;
canonical root bindings allow 4 KiB of UTF-8 and vault payloads 16 KiB. Task access
still governs every invocation. A one-use approval shows the exact endpoint and
arguments. The driver reads the key after approval, pins the protocol and checks
server identity and every selected definition before one call. It never replays
a business call after transport/output failure. The 30-second operation deadline
includes credential resolution, discovery, call and close. Wire/message/catalog
and text/encoded-result bounds remain those recorded in transport acceptance.
Successful bounded results can carry a cleanup warning if session deletion is
unconfirmed; output failures explain possible remaining effects and ask for
inspection instead of repeating an operation for its output.

Connected OAuth qualification and broader lifecycle integration are tracked by
[EH09](../plans/everyday-harness/eh09.md). Bearer setup does not complete H19.
Dropping a client, failed initialization or Stop does not guarantee deletion of a
remote legacy session. The native fixture qualifies the declared public-loopback
flow; third-party compatibility and real-model reliability remain separate.

## Verification boundary

`scripts/mock-mcp.mjs` is a synthetic local stdio diagnostic, never installed or launched by normal startup. Rust tests exercise framing, negotiation, pagination, unsupported server requests, approval binding, changed files/metadata, cancellation, time/output limits, text-only results, `isError` and descendant cleanup. Store/bridge tests cover scope, revisions, expiry, stale state, failed writes and replay. Flutter tests exercise compact light/dark review, selection, explicit enable/disable/forget, Stop and approval controls.

`scripts/test-mcp.py save --directory <fresh-output-directory>` and then `restore` in a separate process exercise the bundled FFI, SQLite, actual child server and loopback model fixture. They verify preview/send parity, Deny without startup, exact invocation, tool errors, Stop without partial history, restart without startup and retained provenance/export. Add `--credentials` to both stages to exercise masked setup, actual native-vault save/rotation, restart reuse, changed-launch refusal and credential-free model/history/export/database text. On Windows an independent credential-existence probe verifies rotation and Forget remove the actual entries without reading their blobs. No live provider quota is used. `scripts/test-mcp-multiple.py save --directory <fresh-output-directory>` then `restore` exercises two same-named echo tools through real FFI/SQLite/stdio and a loopback provider. It checks capacity refusal followed by sibling Disable and same-review Enable, independent aliases/provenance, one server crashing while another completes, Forget and restart without startup. The Windows smoke entry also drives rendered multi-server inspection and enable/disable/forget; these are controller/widget checks, not OS pointer/IME acceptance. Third-party server compatibility, macOS/Linux runtime, persistent cross-call sessions and OAuth and representative low-end measurements remain open.

### Session cleanup ownership

Each transient legacy session has one cleanup-only DELETE, including after
failed initialization, cancellation, deadlines and dropped invocation futures.
Cleanup has an independent two-second ceiling and cannot call a tool, refresh a
credential or retry effects. Normal task owners join their cleanup jobs before
terminal run evidence, retaining mcpCleanup receipts and warnings when termination
is unsupported or unconfirmed. A 404 confirms the session is already gone; 405
records unsupported termination rather than claiming the server released it.
A returned tool result survives cleanup failure. Never repeat a tool to clean up
its session. Stateless modern connections have no session to delete. Abrupt
process loss cannot confirm remote cleanup; submitted external effects may remain.
Inspection-only clients also attempt bounded cleanup on drop; interrupted
inspection does not establish that a server-side effect/session was undone.
