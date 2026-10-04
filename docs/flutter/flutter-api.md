# Flutter/Rust bridge contract

Scoped settings accept the optional [task allowance](../design/task-budgets.md) group. Started events include `taskBudget`; paused metadata includes backward-readable `segments`. Effective inventory/context/run snapshots report resolved task values. Continue refuses exhausted segments before issuing another model request.

This is an internal in-process integration, not an HTTP API. The Dart UI and bundled Rust bridge are built together; there is no independently versioned external client compatibility promise. See [module notes](../../apps/dolores_flutter/README.md) and [architecture](../ARCHITECTURE.md).

## Ownership and loading

The Rust library exports `dolores_call(const uint8_t* input, size_t length)` and `dolores_free(char* value)`. Input is UTF-8 JSON, at most 128 KiB, readable for the call duration. The result is an owned NUL-terminated UTF-8 JSON allocation. Copy it, then free exactly once through `dolores_free`; do not use the Dart allocator to free Rust results. Invalid pointers remain caller errors; Rust's error/panic envelope is not a memory sandbox.

Every reply is `{"ok":true,"result":...}` or `{"ok":false,"error":"..."}`. Successful commands may return null. Dart `NativeBridge` serializes calls through a worker isolate; runtime network work occurs on Rust's bounded async runtime. Library loading/worker failure resolves pending UI calls with recovery guidance.

| Platform | Library path relative to executable |
| --- | --- |
| Windows | `dolores_flutter_bridge.dll` |
| Linux | `lib/libdolores_flutter_bridge.so` |
| macOS | `../Frameworks/libdolores_flutter_bridge.dylib` |

`DOLORES_DATA_DIR` and `DOLORES_GLOBAL_SKILLS_DIR` are optional absolute host overrides read at initialization. Defaults use the account application-data/home paths. One engine is initialized per process; shutdown cancels current work but is not an in-process switch to another data directory.

## Command coordination

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
| Local evaluation | `saveTaskFeedback`, `comparisonSources`, `comparisonsPage`, `comparison`, `deleteComparison`, `startComparison` | Local feedback CAS; exact frozen comparison requests/results and bounded storage. |

Run IDs are supplied by the host. `poll` drains a bounded batch and is the active-run event transport; consume the `done` event before starting another run. `cancel` affects the matching run, and each approval binds a waiting call ID. Configuration and most session/inspection mutations refuse while work is active. Review tokens/revisions are separate from run IDs and never grant reusable tool permission.

Common event kinds include lifecycle/start, text deltas/public model commentary, tool approval/results, usage/progress, automatic-memory activity and `done`. Detailed event fields depend on the operation; see the Rust emitter and corresponding Dart consumer rather than assuming all operations share a reply shape. Comparison `done` includes a full receipt, `persisted` flag and optional save error; a volatile receipt must not be shown as durably complete.

Transport/Stop/malformed ordinary responses do not publish an incomplete chat turn; explicit output/step limits can save a paused reply with completed evidence. Continue requires the latest saved source and a fresh bounded run. File/command effects are independent of turn persistence. Feedback/comparison exports remain separate from provider-visible history.
