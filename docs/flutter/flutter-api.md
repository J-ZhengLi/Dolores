# Flutter/Rust bridge contract

## Motion preference

`bootstrap.reduceMotion` is the local reduced-motion choice. The persisted
`experimentalPreferences.preferences.reduceMotion` boolean defaults to false for
older records; `saveExperimentalPreferences` retains its existing revision check
and returns the acknowledged preferences. The General motion control reads the
latest preferences and changes only this field, preserving the window and
keep-awake choices. Failed saves retain the displayed value. Flutter combines
this preference with OS reduced motion and shares it across conversation owners.
If the stored preference cannot be read at bootstrap, motion is reduced while
other app features remain available; an explicit settings save exposes the error.

## Planning and reference snapshots (EH04)

`bootstrap.planning` advertises `/plan` and `@` discovery. `planning {session}`
returns `{revision,planning,draft,decisions,accepted,references}`. `editPlanning
{session,revision,edit}` accepts `mode {enabled}`, `draft {text,decisions}`,
`accept`, `add {source,expected}` and `remove {digest}` actions. Revisions use
checked CAS; failed saves retain prior state. Drafts permit 8 KiB of plan and
2 KiB of decisions. Accept snapshots a concrete plan and optionally binds the
current goal ID and criterion strings, without starting execution. New drafts
or reference selections invalidate acceptance. `startPlan {session,revision,id}`
checks the accepted revision/goal, then uses normal Start/StartGoal admission and
permissions. Saving/accepting/reopening alone never dispatches work.

Planning filters both advertised and executable tools to local read/list/search
and human questions. No command, MCP, delegation, browser, schedule, repair or
write tool is available. Queued instructions freeze PlanningState, and steering
uses frozen reference text without changing the active run's tool policy. A
normal unpaused planning result saves its draft only against the original CAS;
a changed or failed save leaves the prior plan plus the saved reply/receipts.

`referenceChoices {session,kind,owner?,path,cursor}` enumerates bounded sources;
`referencePreview {session,source}` resolves a tagged file/folder path, code or
terminal attachment digest, message `{session,message}` or conversation session.
Preview uses existing scoped editor/attachment/history ports. Selecting rechecks
the digest and refuses changed/deleted sources. Saved snapshots remain literal
historical data: later source edits/deletion never substitute current content.
Four references allow 16 KiB each and 24 KiB total; a folder contains only a
200-entry first page with omissions; cross-chat sharing requires the same working
folder and latest 80 messages, excluding attachments. Code/terminal categories
require explicit text attachments from their existing selection commands.

Context preview and dispatch use the same core preparation, label snapshots as
untrusted data, and retain existing 128 KiB/model allowance checks. Run journals
retain PlanningState; queue/steer receipts snapshot it transactionally. Migration
42 adds chat-owned `planning` plus immutable `plan_versions` for accepted plans.
JSON/Markdown export includes current state, accepted versions and input receipts;
chat deletion cascades the records. Existing history/configuration is preserved.

## Goals and context recovery (EH03)

`bootstrap.goals` advertises working-chat goals. `goal {session}` returns the
current record or null. `editGoal {session,edit}` supports explicit human `set
{revision?,objective,criteria,constraints}`, `pause/resume/clear {revision}`.
Set pins the effective task limits; completion and clear retain receipts, and a
new Set archives the prior goal. SQLite migration 41 adds `goals` and
`goal_history`; JSON/Markdown exports retain both current and archived records.
Model progress has a separate `progressRevision`; human criteria remain immutable.

`startGoal {session,revision,id}` uses the normal primary admission route with a
fresh run ID, goal ancestry and durable claim. Pending, uncertain or suspended
inputs block the claim. A `goal_progress` tool records the unchanged criterion
list and successful `{run,sequence}` evidence; `inspect_goal` returns matching
receipt references. Completion is requested, then finalized only after a normal,
unpaused response and finished run evidence save. Failed/truncated responses,
Stop, exhausted allowances and missing receipts cannot complete it. Receipt
validation establishes provenance, not semantic correctness of model judgments.

All goal model/tool attempts are reserved transactionally before dispatch,
including failed attempts, children and compaction. Active time uses one-second
reservations excluding observed review waits; no refund or reset on restart.
Only explicit goal segments charge this goal, not unrelated queued human turns.
The frontend considers continuation after done/ownership release, yielding to
drafts, input and decisions; no idle timer or closed-app dispatcher is added.
Restart pauses active goals. Counters survive Resume; two segments without new
work/verification receipts block continuation. Existing task caps remain in force.

The core checks the tighter serialized public-byte/model input allowance. Goal
runs and auto-compaction chats receive calm host estimates at 60/80 percent
pressure, with output reserve and separate context/response/task limits. At a
safe boundary from 80 percent, up to two tool-free summary attempts can rebuild
context. Summary schema, size and useful headroom are validated; original user
instructions and durable host state remain separate from untrusted summary and
receipt previews. Historical tool calls/approval tokens are never reconstructed.
Images retain their existing explicit recovery path. Protocol fixtures do not
qualify model-dependent premature stopping, optimal thresholds or live reliability.

## Command and skill discovery (EH02)

`bootstrap.commandPicker` advertises `/` discovery. `composerCommands {session?}`
returns fixed host command descriptors (`id/title/description/available/reason`),
effective enabled reviewed skill entries (`selection/description/text`) and saved
`selected` references. This read does not scan disk or consume Settings reviews.
Commands and skills remain separate types even when names collide. Stop and
Resume queue use the existing host operations; Goal opens its EH03 controls.
Plan opens EH04 review controls.

`saveComposerSkills {session,selections}` persists at most three distinct
`{scope,name,revision,version}` references. New references require exact effective,
enabled reviewed snapshots; stale chips can always be removed. Selections persist
until removed and are included in context preview and normal request preparation.
Scheduled occurrences retain their own pinned skill. SQLite migration 40 adds
per-chat selections without rewriting version 39 input receipts.

Enqueue snapshots the selected references transactionally in `selectedSkills`.
Changing composer chips or editing queued text does not rewrite those snapshots.
Queued start and steering resolve them again before delivery, refusing changed,
disabled or overridden identities with draft/queue retained. Cancel and requeue
explicitly to choose a new version. Steering adds reviewed skill instructions
only at the same safe boundary as its user input, with existing context limits.
Normal request context sources and instruction receipts retain scope/version
provenance. Failed saves preserve the previous selection and draft.

## Live instruction control (EH01)

`bootstrap.instructionQueue` advertises durable text input. `taskInputs {session}`
returns task revision, suspension/recovery flags, unresolved entries and the most
recent 50 terminal receipts. `editTaskInputs {session,edit}` supports `enqueue
{id,text,mode}`, `edit {id,revision,text}`, `cancel {id,revision}`, `reorder
{revision,ids}`, `resume {revision}` and uncertain-delivery `acknowledge
{id,revision}`. Revisions are optimistic; failed writes preserve saved data and
drafts. Identical enqueue IDs are idempotent within their conversation.

`startTaskInput {session,id}` selects the oldest pending steer, otherwise queued
input, then uses the normal primary-run admission path. `id` is the client run
ID; the returned `instructionId/input` identify the actual saved submission. It
preserves unrelated draft text/attachments. Normal polling and cancellation still
use the client run ID. The frontend starts eligible follow-ups only after the
previous done event has released ownership. No closed-app/background dispatcher
is added. Existing same-root serialization and host run limits remain.

The core approval adapter collects steering at provider/tool boundaries. A new
instruction cannot edit an in-flight request or grant tools. Pending steering
supersedes undispatched operations; a review crossed by steering requires a fresh
operation. Durable Applied receipts acknowledge context insertion, not model
success. Stop suspends inputs before future dispatch; failed suspension never
prevents cancellation. Reopen converts Delivering to Uncertain and requires
inspection/resume. Uncertain input is never replayed. See the
[full contract](../design/everyday-harness-contracts.md).

## Reviewed Windows native updates (20.5)

Working-chat `build_harness_repair {repairId,revision,evaluationId}` requires a
complete qualified 20.4 receipt matching current source, candidate and Cargo.
Its independent one-use review includes the exact implementation diff and fixed
offline/locked release command, 300 seconds/256 KiB, plus a host-owned compiler
target. No provider-chosen path/argv or installation action exists. At most four
builds per repair are retained locally; ready means only the Rust bridge DLL
differs in two complete hashed bundles. Build scripts, UI, policy, credentials,
storage, evaluator and launcher changes are refused by the initial allowlist.

`nativeRepairs {session?}` reads build summaries and the last durable handoff.
`reviewNativeUpdate {session,buildId,restore}` creates a five-minute single-use
direct user review for an idle normal app, exact profile/process/bundles and
compatible schema. `applyNativeUpdate {token}` consumes/rechecks it and returns
`restart:true` only after durable intent and protected helper dispatch;
`discardNativeUpdate {token}` releases that review. These are local UI commands,
not model tools. Full access does not cover any native build or restart review.

`bootstrap.nativeStartup` identifies a launcher-owned blocked startup.
After normal idle chat initialization, `nativeStartupReady` publishes the pinned
source/schema/process and logical history receipt; `nativeUpdateReady` releases
the write barrier only after the launcher records healthy acceptance. No model
request or task replay occurs. Helper/child creation identities and complete
bundle hashes survive interruption. Failed startup can recover the consistent
pre-startup SQLite snapshot; separately reviewed later Restore changes code while
preserving current history. Schema stays 33. Compiler caches use compact private
`nt/<intent-prefix>/<phase>` directories to avoid the reproduced Windows linker
path limit; their full owner IDs and exact commands are retained without reuse.

## Native repair proposals (20.2)

20.4 adds working-chat `test_harness_repair {repairId,revision,package,reproduction}`.
Package is `dolores-core` or `dolores-provider-openai`; reproduction is complete
Rust integration-test source up to 4096 bytes. Its one-use review includes the
exact candidate/test diff and three host-fixed command previews. Full access
cannot approve it automatically. Baseline/candidate get identical frozen tests;
candidate regressions run only after successful reproduction. Each command has
300 seconds/256 KiB capture, offline/locked, with no retry/install action.
Schema 33 retains per-phase results and four trials per repair. Repair inspection
adds `evaluations` summaries with revision/current/status, criteria identity,
counts and actionable notes; old receipt qualification becomes stale on edits.
Typed completed results distinguish withheld improvement from a qualified test
receipt, which still has no install authority. Interrupted receipts remain explicit.

`harnessRepairs {session,repairId?,source?}` is a local chat-scoped read, available
during an active run. Omitted ID lists retained IDs; an ID returns its current
revision, matching-source status, file identities and artifact-integrity notice.
Optional source reads a bounded retained candidate page/diff. Missing or changed
files preserve database snapshots; another chat cannot inspect them.

Working-chat `harness_repair` actions are prepare/add/propose/inspect/list. Prepare
copies exact `sourceId`/`bundleId` from inspection; add/propose require current
`repairId`/`revision`, and propose's `sourceId` is the inspected `candidateId`.
One unique `oldText` → `newText` replacement is capped at 4096 bytes per operand.
Every prepared step receives exact independent review even under Full access;
dispatch consumes its one-use plan and rechecks source/revision/artifacts.
No build/test/activation action exists. SQLite schema 32 and separate immutable
artifact versions retain source snapshots. Limits: four workspaces per chat,
eight files/2 MiB snapshots per workspace, sixteen revisions, 8 KiB cumulative
file diff, 2048-line/8 KiB inspection text and 16 KiB encoded tool result.

## Selected-window observation (14.1)

`desktopState {session?}` returns local adapter availability, capture bounds,
configured image-capable models and retained chat captures. `desktopObserve
{id,session,target?}` reserves the normal run owner: omitted target lists windows;
an exact returned identity captures only that selected window. Poll returns a
done event with `observation` or an actionable error. Cancel kills/reaps the owned
helper. `desktopPreview {session,capture}` returns bounded verified JPEG bytes as
local base64; `desktopRemove {session,capture}` deletes only matching evidence.
Neither preview nor observation contacts the provider.

`start` adds optional paired `desktopCapture` / `observationModel`. This chooses
one saved screenshot and configured image-capable profile for a fresh run with
only `inspect_desktop_capture`; it cannot combine with continuation/resume. The
normal selected model is unchanged. Successful tool records add optional `parts`
image references, never inline binary data. Old text-only history omits the field
and remains readable. A completed answer without successful image evidence fails
with a retained draft. Paused/failed observation cannot use ordinary recovery;
explicit fresh sharing is required. See [observation contract](../design/computer-use.md).

`generateMod {id,session,revision,model?}` reserves a bounded active run. `poll`
returns done with source/usage/error; malformed/truncated source remains local and
cannot activate. Model is a configured choice and does not change the selected
chat model. `setModPolicy {session,revision,automatic}` controls separate ABI 1
automatic activation, default off. Both refuse active-run conflicts. Draft source
and notices persist in the project state; restart never replays generation.

`modState {session,category}` inspects project-scoped ABI 1 recovery mods and
checks the active version; health failure restores/quarantines it. `testMod`
accepts exact source, manifest and expected revision, then runs host-owned fixed
criteria. `activateMod {session,revision,identity}` refuses drift, non-improvement,
quarantine and active runs. `restoreMod {session,revision}` restores the retained
baseline and quarantines the replaced version. All four are excluded while a run
is active; these are local operations without model requests. See
[executable mods](../design/executable-mods.md).

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
| Harness inspection | `harnessInventory`, `harnessSource`, `harnessNavigation` | Read-only inspection works during an active run. Navigation accepts a typed `query` with list/search/read, bundled relative source, optional content/bundle identity, range and continuation cursor. Reads allow up to 2048 lines and a 32 KiB requested ceiling, reduced to the current 16 KiB tool-result bound including JSON. Legacy aliases remain. Search is literal text or identifier tokens, not a call graph. An explicitly selected checkout is compared, never substituted for running source. No provider request or project change. |
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
file approval IDs use `child.<id>.<generation>.<call>` and still bind one waiting decision.
`subagentReview` is an additional backward-readable pause reason; failed/blocked
children retain reports and require explicit review/continuation. Shared step
exhaustion uses `stepLimit` for explicit numeric limits or `checkpoint` for
Automatic resource/context bounds. `noProgress` saves repeated tool-failure
evidence. None automatically replay work.

`delegate_tasks` also accepts `{childId,action,message?,offset?}` for `inspect`,
`followUp` and `interrupt`; it cannot mix this form with initial `tasks`.
Initial children optionally specify a frozen enabled model on the same connection.
Follow-ups cannot change scope, access or model and share the child's cumulative
four-model/four-file cap and parent totals. The parent waits during follow-up
work. Child receipts add `model`, `followUps`, cumulative `usage` and bounded
`observedTools`, including completed receipts before interruption. `verified`
remains false. Latest reports supersede old pause state only for the same child.

`childSupervision {session,id,request:{childId,action,offset?}}` is the local
active-run UI route for `inspect` and `interrupt`. A stale session/run/child fails;
UI follow-ups go through the parent to preserve file ownership coordination.
Inspection returns frozen model/settings, scope, usage, report and a context
page with `offset`, `nextOffset`, `contextBytes`, `contextVersion` and explicit
truncation. Pages end on UTF-8 boundaries and fit the tool-result allowance.
Saved `subagentContext` pages include the parent permission snapshot;
`subagentFollowUp` identifies each explicit generation. They remain in run events
after the active UI route expires. Context is prepared instructions and bounded
report checkpoints, not an exact provider transcript or reasoning trace.

Transport/Stop/malformed ordinary responses do not publish an incomplete chat turn; explicit output/step limits can save a paused reply with completed evidence. Continue requires the latest saved source and a fresh bounded run. File/command effects are independent of turn persistence. Feedback/comparison exports remain separate from provider-visible history.

`run_command` now accepts optional `timeout_seconds` and `capture_bytes`; its approval preview includes their resolved values. `read_text_file` accepts `start_line`/`line_count`; ranged results include `snapshot`, usable as edit `expected_snapshot`. See [practical tools](../design/practical-tools.md).

`savedDraft`, `saveDraft`, `runCheckpoint` and `checkpointDraft` are local APIs. `start.resumeRun` binds explicit recovery to the latest recoverable run; snapshots expose optional parentRun and accumulated segments. Old snapshot JSON defaults to the first segment. See [task checkpoints](../design/task-checkpoints.md).

orkSession {session, through} atomically copies complete turns and returns a new session/workspace with sharedFolder. setAutoCompact {session, enabled} controls only that chat; reviewSummary includes autoCompact. compacting/compacted events disclose a single preflight summary and reported usage; the durable run log retains these events.

### Desktop appearance

`bootstrap.appearance` is `system`, `light` or `dark` (absent legacy values use System). `saveAppearance {theme}` persists the typed local choice and returns the acknowledged theme. Invalid strings are rejected. It is allowed during a run because it changes presentation only, and does not change profiles, task authority, provider requests or conversation history. Storage failure leaves the previous acknowledged UI theme. Schema 27 adds an `appearance` singleton without rewriting existing tables.

### Project knowledge (12.1)

projectKnowledge {session} returns revision, learning, shareFeedback, notice
and facts with computed fresh status. setKnowledgePolicy {session, revision,
learning, share_feedback} changes only this working folder's opt-in choices.
saveKnowledgeFact {session, revision, id?, title, text, kind, enabled, inferred?}
saves a protected manual correction. Revisions are checked atomically; conflicts
require refresh and explicit retry. Side chats cannot save project knowledge.
Context summaries retain exact selected knowledge provenance. A successful done
event may include knowledgeUpdate for the activity log; learning failure does
not discard the saved reply or automatically replay work.

### Independent tool trials (12.2)

`trialSources {session}` returns saved project skills and at most 20 full trial
receipts, plus fixed suite/model metadata. `startToolTrial {id,session,name,
revision,text}` compares the current enabled project snapshot with parsed
candidate SKILL.md. Poll receives trialProgress and done with trial, persisted
and error. Cancel uses the ordinary run owner. No skill is activated; incomplete
or unsaved evidence cannot qualify. Baseline/candidate share frozen budgets and
the same provider/settings; exact requests and append-only receipts survive
restart. Existing tool-free comparisons remain separate.

### Project skill learning (12.3)

`learningState {session}` returns policy, causes, exact snapshots and trial
receipts for this folder. `setLearningPolicy {session,revision,enabled,automatic,
paused}` uses explicit revision checks. `createCheckWorkflow {session,
command_text}` creates the optional exact host project-check workflow;
`reflectLatest {id,session}` claims one saved event and exposes learningProgress
and done events through normal polling/cancellation. `approveLearning {session,
revision,event}` requires a completed qualifying review, unchanged skill,
knowledge and policy. Foreground done may carry learningUpdate; reflection
failure preserves its saved reply. Schema 30 adds project_adaptation locally.

`restoreLearning {session,revision,event}` restores an exact active update and
quarantines its candidate. It returns the actual saved view; a manual change
returns a conflict notice with the current skill intact. Post-activation receipts
bind monitorSession/monitorMessage/monitorTrialId; automatic restore requires a
complete matching regressed trial and unchanged activation/policy. Startup marks
pending events/monitoring interrupted, never eligible/replayed. All originating
chat receipts are included in the project Learning view when still retained.

## Workflow lifecycle ownership (EH09.5)

Parent and child tool wrappers share the reviewed hook runtime, serialized
24-call/30-second allowance and run journal. Child hook intent/results add
`childId`; hook script input remains the bounded schema-1 event/tool/outcome
object. A child's frozen `read_skill_resource` uses ordinary parent authorization
and its existing cumulative tool cap. Safe-boundary steering can extend the
parent reader to at most three selected package bindings, never replace a bound
skill/scope, and never widen a preexisting child reader.

`ScheduledTask.hooks` is an optional additive configuration snapshot. Absent
legacy data executes no hooks. Creation receipts add `lifecycleHooks` (count);
current configuration must still equal the pin before occurrence admission.
Changed/disabled configurations refuse without model work. Scheduled task and
active-occurrence package pins, plus unresolved task-input pins, retain physical
bundles after scoped catalogue/activation removal. Terminal history alone is not
an execution pin.

Transient remote clients finalize one cleanup-only DELETE independently of the
cancelled operation, at most two seconds. Primary run owners join those jobs
before terminal evidence. `mcpCleanup` events retain connection identity,
`confirmed`, `status` (confirmed/unsupported/unconfirmed), and `toolReplayed:false`;
no session ID or credential is journaled. `connectorCleanup` UI events surface
unconfirmed status without completing a run or clearing a pending decision.
Returned results remain intact and cleanup never replays a tool.
