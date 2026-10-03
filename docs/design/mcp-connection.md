# Reviewed local MCP connection

Brick 6.1 adds one local stdio MCP server per working folder through a compiled `ToolPlugin` adapter. Flutter remains the desktop shell; the Rust bridge coordinates reviews and storage, while `dolores-tools-mcp` owns the protocol and process lifetime. No server, runtime or package is bundled, installed or downloaded.

## Desktop flow

In a saved Project or Temporary chat, open **Chat actions → External tools (MCP)**. Enter a short name, choose an installed direct executable and add its literal arguments individually. On Windows choose an `.exe`; for a JavaScript server, choose an installed Node executable and put the existing server script path in the first argument. A shell command or `npx.cmd` is not a direct executable. No variable expansion or shell fallback occurs.

**Inspect server** is an explicit launch action. Its disclosure explains that the program runs with user OS permissions, may access files/network or change them during startup, and is not sandboxed by tool approval. Inspection initializes the server, discovers a bounded catalog, then closes it. Select one or two tools and choose **Enable selected tools** to save the reviewed connection for that folder. Editing launch fields discards the review; inspecting never enables tools. A previously enabled connection remains unchanged until a successful replacement, Disable or Forget. Side chats have no MCP setup or tools.

Each chat operation shows the server label, original tool name, revision and complete selectable JSON arguments, with **Run once** and **Deny**. Approval starts a new server, rechecks its reviewed metadata, sends exactly one call and closes it. Deny starts nothing. Results stay literal in chat/trajectory, including explicit `isError` display. Arguments, provenance and results persist with a completed reply and exports. Effects can remain after Stop or later reply failure; external actions are outside the file-change journal.

## Protocol and lifetime

The adapter uses newline-delimited UTF-8 JSON-RPC over a supervised child's stdio, following the official [transport specification](https://modelcontextprotocol.io/specification/2025-11-25/basic/transports). It sends `initialize`, checks negotiated version and tools capability, then sends `notifications/initialized`. Supported versions are 2025-11-25, 2025-06-18, 2025-03-26 and 2024-11-05. Graceful close drops stdin, waits briefly and terminates remaining supervised processes, consistent with the [lifecycle](https://modelcontextprotocol.io/specification/2025-11-25/basic/lifecycle).

`tools/list` follows bounded pagination. `tools/call` retains text blocks and the error flag; this follows the ordinary synchronous [tools contract](https://modelcontextprotocol.io/specification/2025-11-25/server/tools). Task-required tools are omitted. Server instructions/icons, tool annotations and unknown metadata do not become host instructions or permissions. Descriptions, schemas and results are untrusted model data. The model receives stable `mcp_tool_1` / `mcp_tool_2` aliases, original names in descriptions, and the selected object schemas. It receives neither launch paths nor launch arguments. The server validates semantic input arguments; Dolores validates their bounded JSON object shape and binds the exact approved value.

Client capabilities are empty. Ping requests are answered; sampling, roots, elicitation and other server requests receive method-not-supported. There is no remote HTTP transport, environment/credential editor, resource/image/audio loading, task execution, automatic approval or retry. Changed-tool notifications require another inspection. Structured-only results are not exposed as structured model data in this first adapter.

Processes start only during an explicit inspection or approved invocation. Specification lookup, request preparation, context inspection, enable/disable, restart and idle chat do not launch a process. Connections are intentionally stateless across calls: servers requiring a persistent session are not accepted by this first flow. Repeated startup costs and external-server resource use remain server dependent.

Windows reuses the command tool's isolated inherited-handle list, suspended-start job assignment and hidden child process. Unix uses a process group and nonblocking output. These supervise lifetime, not filesystem/network access. A server can escape intended application behavior with user privileges; Unix processes can also leave their process group. No adversarial process sandbox or total resource quota is claimed.

## Bounds and freshness

| Item | Limit |
| --- | --- |
| Connection/catalog | One connection per folder, up to two selected tools; 32 discovered tools, four pages, 32 KiB catalog |
| Tool definition | 128-byte name, 2048-byte description, object input schema; 8 KiB combined |
| Launch | 128-byte label, absolute executable, 32 literal arguments of at most 4096 bytes; 48 KiB total |
| Operation | 30 seconds including hashing, startup, discovery and call; no retry |
| Protocol | 64 KiB frame, 256 KiB stdout, 64 KiB discarded stderr, 64 server requests/notifications |
| Invocation/result | 4 KiB argument JSON, 8 KiB combined text; 16 KiB encoded host result |
| Existing agent loop | Eight advertised tools including six built-ins; four tool operations / four model calls |

SHA-256 covers the executable (up to 128 MiB) and each existing direct file argument (up to 4 MiB). Files are checked before/after inspection, before saving and before invocation. The approved call also checks protocol, server name/version and all selected tool names/descriptions/schemas before `tools/call`. Changed launch files or selected metadata require review. This is freshness detection, not atomic executable binding: it cannot prevent a change between checking and launch, and does not hash package dependencies, dynamically loaded files, wrappers' indirect inputs or the entire runtime environment. Trust the installed server and its dependencies.

A five-minute opaque review binds the originating saved chat, folder, previous connection snapshot and inspected launch/catalog. Saving accepts only one or two distinct reviewed names; a successful compare-and-swap consumes the review. Failed writes preserve the review for an explicit retry. Settings refresh, edits, Stop, discard and shutdown invalidate it. Disable advances the revision and removes advertised tools; Forget deletes only the local connection, leaving external files intact.

SQLite schema 15 adds `mcp_connections(root,data)` without rewriting existing tables. Saved launch fields and tool metadata are unencrypted alongside application preferences; keep secrets out of them. Provider credentials retain their separate native-vault flow. MCP credentials are future scope.

## Verification boundary

`scripts/mock-mcp.mjs` is a synthetic local stdio diagnostic, never installed or launched by normal startup. Rust tests exercise framing, negotiation, pagination, unsupported server requests, approval binding, changed files/metadata, cancellation, time/output limits, text-only results, `isError` and descendant cleanup. Store/bridge tests cover scope, revisions, expiry, stale state, failed writes and replay. Flutter tests exercise compact light/dark review, selection, explicit enable/disable/forget, Stop and approval controls.

`scripts/test-mcp.py save --directory <fresh-output-directory>` and then `restore` in a separate process exercise the bundled FFI, SQLite, actual child server and loopback model fixture. They verify preview/send parity, Deny without startup, exact invocation, tool errors, Stop without partial history, restart without startup and retained provenance/export. No live provider quota is used. The Windows smoke entry also drives rendered inspection and enable/disable/forget; these are controller/widget checks, not OS pointer/IME acceptance. Third-party server compatibility, macOS/Linux runtime, persistent/remote transports and representative low-end measurements remain open.
