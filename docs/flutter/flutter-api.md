# Flutter/Rust bridge contract

`draftAttachments`, `attachmentPreview`, `attachFile`, `removeAttachment`, `exportAttachments`, `cleanupAttachments` and `setImageModels` implement [attachment snapshots](../design/attachments.md). Bootstrap advertises `attachments` and configured `imageModels`; old hosts can omit them. Stored message pages/exports add optional `parts` with digest/name/MIME/bytes. Native previews alone may return bounded local `imageBase64`; raw images are not included in context reports or ordinary exports. Image sending uses an explicitly enabled provider adapter and refuses unsupported input without consuming the draft. Mutations are excluded during active execution.

`taskPermissions` reads a saved chat's exact local policy/revision and reviewed MCP choices. `setTaskPermissions` saves an explicit policy with an expected revision. During an active run only review/revocation is allowed; matching primary work is cancelled. The scoped patch/effective run snapshot has a backward-readable `permissions` group. See [task access](../design/task-permissions.md).

Scoped settings accept the optional [task allowance](../design/task-budgets.md) group. Started events include `taskBudget`; paused metadata includes backward-readable `segments`. Effective inventory/context/run snapshots report resolved task values. Continue refuses exhausted segments before issuing another model request.

This is an internal in-process integration, not an HTTP API. The Dart UI and bundled Rust bridge are built together; there is no independently versioned external client compatibility promise. See [module notes](../../apps/dolores_flutter/README.md) and [architecture](../ARCHITECTURE.md).

## Ownership and loading

`browserSettings` reads optional runtime discovery, literal bounds and the local
capture folder without launching Node or a browser. `browserCapture {capture}`
accepts only a canonical UUID and returns one bounded saved JPEG as
`data` (base64); missing, linked, oversized or invalid images refuse. These local
reads remain available while busy and never replay a browser action. Browser
tool results retain text/state/control refs and capture IDs, without automatic
image projection to the model. See [browser use](../design/browser-use.md).

`webSettings` reads the global revisioned connection while busy; `saveWebSettings`
accepts revision, enabled, provider (`mwmbl`/`brave`/`searxng`), optional endpoint,
optional `apiKey` and `clearKey`. Mutations refuse during a run. The result exposes
`hasSavedKey`, `cleanupPending` and a notice, never a key or vault identity.
Blank preserves a saved Brave key; replacement/removal are mutually exclusive.
Stale/invalid saves retain the caller's draft. See [web search](../design/web-search.md).

The Rust library exports `dolores_call(const uint8_t* input, size_t length)` and `dolores_free(char* value)`. Input is UTF-8 JSON, at most 128 KiB, readable for the call duration. The result is an owned NUL-terminated UTF-8 JSON allocation. Copy it, then free exactly once through `dolores_free`; do not use the Dart allocator to free Rust results. Invalid pointers remain caller errors; Rust's error/panic envelope is not a memory sandbox.

Every reply is `{"ok":true,"result":...}` or `{"ok":false,"error":"..."}`. Successful commands may return null. Dart `NativeBridge` serializes calls through a worker isolate; runtime network work occurs on Rust's bounded async runtime. Library loading/worker failure resolves pending UI calls with recovery guidance.

| Platform | Library path relative to executable |
| --- | --- |
| Windows | `dolores_flutter_bridge.dll` |
| Linux | `lib/libdolores_flutter_bridge.so` |
| macOS | `../Frameworks/libdolores_flutter_bridge.dylib` |

`DOLORES_DATA_DIR` and `DOLORES_GLOBAL_SKILLS_DIR` are optional absolute host overrides read at initialization. Defaults use the account application-data/home paths. One engine is initialized per process; shutdown cancels current work but is not an in-process switch to another data directory.

## Command coordination

History message items optionally include `savedAt` (Unix milliseconds): the local
atomic turn commit time, not an inferred send/receive time. Legacy messages omit
it; forks preserve the original value. These timestamps do not enter model context.

Requests use a camelCase `command` discriminator. [`Command`](../../crates/dolores-flutter-bridge/src/lib.rs) is the source of truth for exact fields/defaults; typed core structs define nested request/receipt validation. Changes must update Dart callers, tests and affected design contracts together. This document is an integration map, not a duplicate generated field schema.

| Family | Commands | Contract |
| --- | --- | --- |
| Bootstrap/workspaces | `bootstrap`, `createSession`, `workspace`, `sessionsPage`, `messagesPage`, `messages`, `delete`, `export` | Session identity, bounded paging, complete export; deletion preserves working files. |
| Connection/settings | `configure`, `listModels`, `selectModel`, `recoverConnection`, `forgetConnection`, `setRequestSettings`, `setModelRequestSettings` | Native-vault connection recovery and endpoint/model profiles; no key returned to UI. |
| Scoped settings | `scopedSettings`, `saveScopedSettings` | Effective user/project/chat values and origins; revisioned typed overrides, private root binding and frozen primary-run snapshots. Reads work while busy; mutations refuse. See [settings contract](../design/scoped-settings.md). |
| Durable run evidence | `runs`, `runEvents` | Latest 20 primary runs and ordered bounded literal events; safe reads while busy, interrupted status on restart and no effect replay. |
| Run/context | `start`, `poll`, `cancel`, `approveTool`, `shutdown`, `context` | One active run; context inspection is local. |
| Harness inspection | `harnessInventory`, `harnessSource` | Read-only local inspection works during an active run. Source IDs are allowlisted; ranges are bounded to 120 lines / 12 KiB. Optional explicitly selected checkout is compared, not used as running source; no provider request. |
| Instructions/changes | `reviewInstructions`, `enableInstructions`, `disableInstructions`, `cancelInstructionReview`, `changesPage`, `changeDetails`, `previewRevert`, `applyRevert`, `cancelRevert` | Exact one-use previews with scope/snapshot binding. |
| Memory | `memories`, `setAutomaticMemory`, `saveMemory`, `deleteMemory`, `reviewMemorySources`, `suggestMemories`, `saveMemorySuggestion`, `discardMemoryReview` | Revisioned preferences, quoted sources, scoped atomic updates. |
| Summary | `reviewSummary`, `generateSummary`, `saveSummary`, `correctSummary`, `deleteSummary`, `discardSummaryReview` | Exact contiguous sources and explicit Save. |
| Skills | `projectSkills`, `reviewSkill`, `activateSkill`, `exportSkill`, `disableSkill`, `forgetSkill`, `cancelSkillReview`, `reviewSkillExamples`, `generateSkillDraft`, `evaluateSkillDraft`, `promoteSkillDraft`, `discardSkillDraft` | Project/global snapshots, frozen evaluation and separate promotion. |
| MCP | `mcpSettings`, `inspectMcp`, `enableMcp`, `disableMcp`, `forgetMcp`, `discardMcpReview` | Folder/connection identity, credentials, reviewed metadata and on-demand execution. |
| Browser | `browserSettings`, `browserCapture` | Local optional-runtime discovery and bounded explicit capture preview; no automatic browser launch or action replay. |
| Local evaluation | `saveTaskFeedback`, `comparisonSources`, `comparisonsPage`, `comparison`, `deleteComparison`, `startComparison` | Local feedback CAS; exact frozen comparison requests/results and bounded storage. |

Run IDs are supplied by the host. `poll` drains a bounded batch and is the active-run event transport; consume the `done` event before starting another run. `cancel` affects the matching run, and each approval binds a waiting call ID. Configuration and most session/inspection mutations refuse while work is active. Review tokens/revisions are separate from run IDs and never grant reusable tool permission.

Common event kinds include lifecycle/start, text deltas/public model commentary, tool approval/results, usage/progress, automatic-memory activity and `done`. Detailed event fields depend on the operation; see the Rust emitter and corresponding Dart consumer rather than assuming all operations share a reply shape. Comparison `done` includes a full receipt, `persisted` flag and optional save error; a volatile receipt must not be shown as durably complete.

`subagent` events carry the primary client `id` and a `child` object with durable
`parentRunId`, `childId`, `goal`, `scope`, `readOnly` and `status`. Status updates
replace the matching child, never the parent's text stream. Completed
`delegate_tasks` tool results contain bounded `children` reports and
`sharedUsage` attempt counts when the batch returned. Parent `AgentSummary`
counts/usage remain parent-only; child usage is retained in run events. Child
file approval IDs use `child.<id>.<call>` and still bind one waiting decision.
`subagentReview` is an additional backward-readable pause reason; failed/blocked
children retain reports and require explicit review/continuation. Shared step
exhaustion uses `stepLimit`; neither automatically replays work.

Transport/Stop/malformed ordinary responses do not publish an incomplete chat turn; explicit output/step limits can save a paused reply with completed evidence. Continue requires the latest saved source and a fresh bounded run. File/command effects are independent of turn persistence. Feedback/comparison exports remain separate from provider-visible history.

`run_command` now accepts optional `timeout_seconds` and `capture_bytes`; its approval preview includes their resolved values. `read_text_file` accepts `start_line`/`line_count`; ranged results include `snapshot`, usable as edit `expected_snapshot`. See [practical tools](../design/practical-tools.md).

`savedDraft`, `saveDraft`, `runCheckpoint` and `checkpointDraft` are local APIs. `start.resumeRun` binds explicit recovery to the latest recoverable run; snapshots expose optional parentRun and accumulated segments. Old snapshot JSON defaults to the first segment. See [task checkpoints](../design/task-checkpoints.md).

orkSession {session, through} atomically copies complete turns and returns a new session/workspace with sharedFolder. setAutoCompact {session, enabled} controls only that chat; reviewSummary includes autoCompact. compacting/compacted events disclose a single preflight summary and reported usage; the durable run log retains these events.

### Desktop appearance

`bootstrap.appearance` is `system`, `light` or `dark` (absent legacy values use System). `saveAppearance {theme}` persists the typed local choice and returns the acknowledged theme. Invalid strings are rejected. It is allowed during a run because it changes presentation only, and does not change profiles, task authority, provider requests or conversation history. Storage failure leaves the previous acknowledged UI theme. Schema 27 adds an `appearance` singleton without rewriting existing tables.
