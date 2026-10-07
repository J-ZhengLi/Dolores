# Dolores roadmap

**Current planning revision — 2026-10-07.** Preserve the current agent UI on Home.
Use separate Home, Scheduled, Folders, Source Control and Terminal pages, reached
through a compact icon rail with hover/focus names and Settings at the bottom.
The user's ChatGPT desktop screenshot replaces full-width navigation labels;
the chat/file side panel stays beside the rail. Files have VS Code-style draggable
split tabs; chats remain in Home.
**Final layout: developer-pages B**, with a top-left title-bar sidebar toggle and
drag-to-hide divider. Home conversation selection defines the project; omit developer
page project dropdowns. Terminal opens its view directly at project root/OS home.
Terminal has its own tabs/splits. The selected project
binds Folders, Source Control and new terminals; without a project, new terminals
start at the OS user home. The earlier mixed chat/file and A/B/C prototypes are
historical explorations, not the new page contract. Editor feasibility remains
open; see [qualification](qualification/workspace-editor.md).

**Implementation order:** reshape **16–19**, then add **21 — Automatic useful
memory**, **22 — Chat-created scheduled tasks**, **23 — Opt-in companionship**
and **24 — Optional closed-UI scheduling**. Keep milestone 20's existing numbering
and evidence. Work proceeds **one milestone per batch**, with separate brick commits; this revision
records the approved implementation direction. Milestones 16–18, 21 and 22 have
implementation and bounded qualification; 19's detached backend stays held.
The next batch is 23. Experimental Settings contains
default-on Multiple Window and a default-off Windows keep-awake option with
truthful platform limits. Follow the [workspace specification](design/developer-workspace.md),
[memory/scheduling contract](design/memory-scheduling-companionship.md) and
[current handoff](HANDOFF.md). Platform CI 8.4 remains deferred.

**Earlier priority change — 2026-10-06.** Milestones 16–19 were paused at the user's
request. Fix fragmented reasoning-stream failures and silent progress first.
Then **20 — Harness self-repair** is proposed in [its specification](design/harness-self-repair.md):
source navigation, managed patch workspaces, broader qualified extension seams,
independent trials and activation/build/restart/restore. It is planned work,
not a claim that milestone 13's recovery-hint mods repair the native harness.

**Earlier developer workspace planning addition — 2026-10-06.** The planned direction is
integrated code editing, Git, terminal/LSP and flexible multi-project views.
Milestones **16–19 are proposed, not implemented**; first language support is
TypeScript/JavaScript and Rust, confirmed by the user. The
[workspace specification](design/developer-workspace.md) defines UX, ownership,
component gates and resource targets. A [fresh-chat handoff](design/developer-workspace-handoff.md)
preserves the earlier vision and work rules. Start with a reviewed prototype and
editor feasibility evidence before runtime UI changes. Existing reliability
[gaps](qualification/remaining-work.md) stay open; this is a new planned track,
not a reclassification of failed learning/computer-use gates.

**Planning baseline — 2026-10-04.** Writing this plan is authorized; future runtime bricks are not started by this document. Brick **8.4, platform CI, is deferred until the user explicitly requests it**. Completed work is retained in [implementation history](IMPLEMENTATION_HISTORY.md); measured results and open gaps belong in [acceptance](ACCEPTANCE.md).

Implementation update: the user authorized batching milestone 9. Bricks **9.1–9.4 are implemented**, with separate commits and bounded evidence in acceptance. The mechanical contracts are verified; consistent real-model task behavior, native input/platform and representative resource acceptance remain open. Later milestones retain their planned scope and are not implicitly complete.

The user subsequently authorized milestone 10 as a batch. **10.1–10.6 are implemented**, with separate commits and release-native/widget/live evidence. The initial routed six-case corpus has five passing observable cases after explicit same-budget recovery; a separate DeepSeek understanding case passes unchanged criteria, completing passing evidence for the small fixed set. Qwen project grounding still fails. The mechanical foundation is delivered, while consistency across configured models remains unaccepted. See [batch results](ACCEPTANCE.md#milestone-10--batch-exit) before treating advanced tools as dependable.

User-requested UX detour before 11.3: unified Settings replaces scattered configuration entries, combines model/response controls, separates scoped personalization/task budgets, and adds persisted System/Light/Dark appearance. Runtime evidence is recorded in acceptance; this does not change or complete the planned browser brick.

## Destination and scope

Milestone 11 update: **11.1 scoped subagents and 11.2 attributable web search are
implemented**. Search defaults to setup-free Mwmbl, with explicit Brave or public
SearXNG configuration and bounded page reading. A Qwen search/read/citation probe
passed; paid/custom service live acceptance and broad research reliability remain
open. **11.3 on-demand browser use is implemented**, with an optional owned
Playwright runtime, reviewed input and bounded local evidence. Synthetic host
flows and cleanup pass; general website/model reliability and other-platform
execution remain open. See [web contract](design/web-search.md),
[browser contract](design/browser-use.md) and acceptance. **12.1 scoped knowledge
12.2 independent tool trials, 12.3 targeted skill adaptation and 12.4 recovery
are implemented**. Automatic activation's real-workflow acceptance gate remains
unaccepted; reviewed skill updates remain available.

**Planning addition — 2026-10-05:** the user approved a dedicated computer-use
milestone. **14.1–14.4 implementation and qualification tooling are delivered;
the real-model exit gate remains unaccepted.** The fixed corpus passes generated-view
inspection but fails Qwen observation adherence and one DeepSeek Save coordinate.
See the [qualification report](qualification/computer-use.md). Existing milestone numbers
and order are preserved. Windows selected-application interaction and visual
verification are the first target; arbitrary-app reliability and other platforms
need separate evidence. The subsequent “next” authorized the first observation brick.

**UX milestone — 2026-10-05:** the user requested a dedicated simplification
milestone after computer use proved difficult to reach. **15.1–15.8 implementation
and qualification tooling are delivered in the authorized batch. Native/model/
resource acceptance has explicit gaps in the [UX qualification report](qualification/everyday-ux.md).** The
[settings audit and UX contract](design/ux-simplification.md) covers every settings
category, related configuration surfaces and conversation-first tool access.
The audit does not change the running UI or close milestone 14's model-quality gate.

Dolores should complete useful project work, discuss reasoning, challenge consequential false premises, remember scoped answers and improve skills/approved extensions from evidence. Keep Flutter/system theme and Rust/provider-independent ports. Support threads/compaction, explicit approval/full-access choices, attachments, settings, subagents, search, browser use and scoped desktop computer use. Automatic tested activation with rollback is limited by host-owned authority and actual containment, not an agent's self-assessment.

The [architecture specification](design/evolving-harness-architecture.md) defines component/state/lifecycle/authority contracts. The [behavior policy](design/dolores-behavior.md) defines reasoning, questioning, character and adaptation. These describe the target; [current architecture](ARCHITECTURE.md) describes the running app. No claim of consciousness, weight training or guaranteed daily improvement is made.

This replaces the prior working-agent/context/learning/plugin proposal. Extension contracts move first; permission modes, advanced tools and executable adaptation are scheduled explicitly. Existing recovery, memory, skills and MCP are extended rather than rebuilt. Every requested feature maps to a named brick.

## Historical starting point before milestone 9

This table records the planning baseline, not current missing features. See the milestone implementation updates and acceptance for current status.

| Implemented | Remaining work |
| --- | --- |
| Project/temporary/side chats, provider profiles and streaming | Per-thread run ownership, durable task state, forks and multimodal messages |
| Compiled Rust ports, reviewed tools, on-demand local MCP and a pinned registry | Executable extension host/activation are absent |
| Context inspection, estimates, manual summaries and continuation | Scoped guidance/skill selection and managed compaction |
| Every-call approvals and capability-bound file tools | Explicit auto-approval/full-access settings; commands/MCP are not OS sandboxes |
| Automatic explicit preferences, global/project skills and versions | Project facts, cause attribution, task evaluation and automatic qualifying activation |
| Local feedback, literal comparisons and regression tooling | Feedback is not learning input; whole coding-task evaluation is absent |
| Windows preview and recorded input/resource checks | Representative resources, other-platform execution and native input/accessibility acceptance remain open |

## Sequence and gates

| Milestone | Outcome | Bricks | Dependency / exit gate |
| --- | --- | --- | --- |
| **9 — Know itself and own its runs** | Accurate introspection, extension contracts, scoped execution/settings and behavior | 9.1–9.4 | No executable mods. Existing tools/history work; versions, authority and ownership are explicit. |
| **10 — Dependable everyday work** | Practical budgets/files/checks, permission modes, durable progress, context and attachments | 10.1–10.6 | Requires 9. Fixed task set completes with honest recovery and effective settings. |
| **11 — Advanced optional tools** | Bounded subagents, attributable search and on-demand browser use | 11.1–11.3 | Requires 10's grant/task/evidence contracts. Stop and resource ownership work across tools. |
| **12 — Learn from experience** | Scoped knowledge, independent trials, qualifying automatic skill updates and transparent recovery | 12.1–12.4 | Requires trustworthy task evidence. Demonstrate an improvement, rejection and rollback under fixed criteria. |
| **13 — Adapt the executable harness** | Chosen runtime, transactional lifecycle, agent-authored mods and integrated qualification | 13.1–13.4 | Requires 9's contracts and 12's trials. Verified containment gates automatic executable activation. |
| **14 — Computer use and visual verification** | Model-visible screenshots, scoped Windows interaction, recovery and measured real-workflow results | 14.1–14.4 | Requires 9–11's ownership, grants, provider/image and browser contracts. Independently qualify desktop control; executable self-adaptation does not grant desktop authority. |
| **15 — Everyday UX** | Conversation-first tools, useful defaults and settings for general users | 15.1–15.8 | Uses delivered state/permission/tool contracts. Model reliability remains separately measured; simplification preserves configuration, evidence and authority. |
| **16 — Home, navigation and Folders editor** | Preserve agent Home; add project-bound files, safe editing and file-only split tabs | 16.0–16.6 | Revised page prototype/editor gate first; shared host ownership precedes independent work. |
| **17 — Source Control and Git diff** | Project-bound changes/history, real diffs, staging, commit and reviewed Git actions | 17.1–17.6 | Requires 16's project/document revisions; serialize repository mutations and preserve conflicted work. |
| **18 — Terminal tabs/splits and language support** | Project-root or home terminals and TypeScript/JavaScript/Rust language services | 18.1–18.6 | Requires 16's ownership; lazy supervised processes, useful missing-tool setup and bounded cleanup. |
| **19 — Bounded experimental detached developer views** | Default-on opt-out window experiment for files, diffs and terminals | 19.1–19.4 | Requires shared owners from 16–18; bounded backend trial and usable single-window fallback. |
| **20 — Harness self-repair** | Diagnose/reproduce a real fault, test a matching patch and activate or build/restart with restore | 20.1–20.6 | Earlier repair priority is retained as history; broaden 13's narrow ABI only after containment and independent-test gates. |
| **21 — Automatic useful memory** | One switch, source-backed facts/decisions, recall, inspect and forget | 21.0–21.6 | Audit today's narrow capture first; freeze accuracy, scope, resource and deletion gates. |
| **22 — Chat-created scheduled tasks** | Natural-language creation and management-only Scheduled page | 22.0–22.6 | Requires shared run ownership; explicit intent, durable clock/claims and existing tool approvals. |
| **23 — Opt-in companionship** | Occasional grounded in-app messages using a configured weaker model | 23.0–23.4 | Requires memory and scheduler contracts; chosen hours, persisted daily cap and quiet failure. |
| **24 — Optional closed-UI scheduling** | Opt-in local worker while the UI is closed | 24.1–24.3 | Requires 22; qualify one host owner, availability limits and worker cleanup separately. |

The current implementation batch is milestone 16; the earlier milestone 20 priority is
historical. Proceed one milestone per batch, with bricks in dependency order. Milestones 16–19
form the developer-workspace track; their manual editor/Git work does not close
prior model-reliability gates. Then follow 21–24; 24 may follow 22 directly if
closed-UI execution is prioritized. Questioning/personality starts in 9.4 and is
exercised throughout. Measure resource costs throughout. Dates are not promised
before baselines establish work size. Each brick has scope, basic acceptance,
realistic failures and exclusions below. A plan does not activate these features.

## Milestone 9 — Know itself and own its runs

### 9.1 Capability and source introspection — first implementation brick

**Scope:** read-only inventory of build/version, installed/enabled/available/permitted adapters, model capabilities, effective budgets/config origins and working-session mode. Bounded version-matched source references/read access. Reuse descriptors; keep secrets/private roots out of automatic context. Source retrieval follows sharing rules.

**Basic acceptance:** a working chat identifies available file tools and their matching implementation; Side chat accurately reports unavailable tools. Local inspection makes no provider request.

**Failure/recovery:** mismatched checkout is labeled rather than treated as running source; missing source/disabled tool gives an actionable reason while ordinary chat works. Bound large source/catalog results.

**Excluded:** loader, activation, source edits and complete-repository injection.

### 9.2 Thread/run ownership and durable events

**Scope:** stable run/event identities, states, snapshots, cancellation and scoped coordinator/revision checks. Retain one active execution by default. Define safe inspection/navigation, durable event delivery and cross-process data-directory ownership. Additive migrations preserve old history.

**Basic acceptance:** run transitions/evidence survive restart; unrelated inspection works and conflicting mutations refuse clearly.

**Failure/recovery:** UI backpressure cannot suspend Stop/deadlines; interruption after effect intent is marked uncertain without replay. Migration/second-host ownership failure preserves a supported recovery path.

**Excluded:** concurrent subagents, exactly-once external effects and fabricated legacy evidence.

### 9.3 Registry, capabilities and lifecycle contracts

**Scope:** descriptor/API negotiation/config revisions/dependencies and deterministic typed hook seams. Adapt built-ins and MCP inventory without equating their trust. Own/dispose registrations, pin in-flight versions and validate transformed proposals at the host boundary.

**Basic acceptance:** compiled and MCP entries show accurate state/health/lazy startup; removal leaves no stale tools.

**Failure/recovery:** incompatible API/dependency cycle disables only the affected extension; server crash leaves other tools/chat usable. Policy-critical errors deny rather than fall through.

**Excluded:** executable mods, arbitrary native libraries, marketplace and automatic global management.

### 9.4 Effective settings and interaction policy

**Scope:** user/project/thread precedence, origins and run snapshots; reasoning-before-work, consequential-premise checking, scoped answers and calm recovery. Adaptation authority is distinct from task permissions.

**Basic acceptance:** overrides affect the intended scope; substantive work gets a useful reasoning discussion, then proceeds under existing authorization.

**Failure/recovery:** invalid/stale updates retain valid settings; a false premise is corrected before consequential edits while a routine request is not delayed by unnecessary questions. Human review complements deterministic checks.

**Excluded:** personality guarantees, automatic new grants and settings with no scheduled consumer.

**Exit:** existing chat/tools/MCP/memory/skills work after migration; inventory, authority and ownership agree. Report resource deltas and unresolved OS boundaries.

## Milestone 10 — Dependable everyday work

### 10.1 Fixed task baselines and coherent budgets

**Implementation status:** implemented; numerical defaults retained, six-case criteria frozen, native budget/recovery contracts verified and two bounded live baselines recorded. Whole milestone reliability remains an exit gate after 10.2–10.6.

**Scope:** synthetic projects for understand-project, repair-check, small multi-file feature, larger-file edit and interruption/limit recovery. Freeze observable criteria. Classify failures before choosing task/model/tool/elapsed/continuation controls and defaults. Track accumulated segments; retain model context/output configuration and 128K blank default.

**Basic acceptance:** corpus runs with recorded settings; bounded configurable allowances are effective/visible. Distinguish harness, model and provider failures.

**Failure/recovery:** repeated output/step exhaustion saves usable progress without unlimited continuation; Stop near a tool decision prevents unapproved work.

**Excluded:** arbitrary large-project competence, weaker tests, silent default increases/model switches. Numerical defaults are proposed from evidence before adoption.

### 10.2 Explicit permission modes

**Implementation status:** thread-scoped typed Review/Auto/Full access, explicit grant UI and host revision/expiry/dispatch checks are implemented. Validation results are recorded in acceptance; this does not establish OS containment.

**Scope:** review-every-operation, auto approval within user-defined grants and opt-in full-access execution. Show scope/revocation and actual containment. Writes/commands/external tools/self-updates retain distinct authority; recheck at dispatch.

**Basic acceptance:** repeated granted discovery avoids prompts; review mode retains exact decisions; full access skips covered prompts while preserving budgets, cancellation and update gates.

**Failure/recovery:** revocation prevents waiting-call dispatch; restricted out-of-scope/link access refuses. Mode changes invalidate stale approvals without silently expanding scope.

**Excluded:** calling user-account commands/MCP sandboxed, unlimited execution and self-granted permission. Strong containment has a separate 13.1 gate.

### 10.3 Practical file and command work

**Implementation status:** ranged reads, snapshot-bound larger-file edits and explicit command limits/local logs are implemented; observed validation and gaps are in acceptance.

**Scope:** ranged reads/snapshot-bound patches, Unicode/line-ending/conflict semantics; configurable bounded command deadline/capture and inspectable large-log artifacts. Preserve journal/uncertain effects.

**Basic acceptance:** larger-file task edits/verifies without source truncation; slow check completes under an explicit allowance.

**Failure/recovery:** source changed between read/apply refuses with refresh guidance; oversized/incomplete output retains truthful exit/capture state and usable bounded logs.

**Excluded:** atomic multi-file transactions and journal rollback of command effects.

### 10.4 Durable checkpoints and resume

**Implementation status:** durable goal/operation evidence, linked recovery, conditional submitted-draft clearing and local composer drafts are implemented. See acceptance for actual restart checks and model gaps.

**Scope:** task goal, proposed plan, completed evidence, uncertain effects, pause reason, persisted draft and linked continuation. Reconcile on restart; checkboxes are not evidence.

**Basic acceptance:** restart explicitly resumes a multi-step task with provenance and prior effects visible.

**Failure/recovery:** write-before-reply interruption does not replay the write; missing tool/changed config requires revised preparation while preserving progress.

**Excluded:** automatic side-effect replay, detached completion and worktree creation.

### 10.5 Thread forks and managed context

**Implementation status:** transactional complete-turn forks, reviewed nested guidance, keyword-selected activated skills and opt-in bounded preflight compaction are implemented; qualification is recorded in acceptance.

**Scope:** resume/fork at complete boundaries with shared-folder semantics; scoped ancestor/nested guidance and relevant activated skills. Opt-in bounded compaction preserves goals/unresolved work/provenance/full history and exposes coverage/omissions.

**Basic acceptance:** fork copies selected history without active runs/grants; small windows trigger visible recovery and retain the task goal. Guidance does not leak between projects.

**Failure/recovery:** failed/oversized summary preserves last valid context/manual recovery; oversized tool output offers ranged retrieval rather than endless compaction. Same-name/changed guidance uses visible precedence/revisions.

**Excluded:** filesystem isolation implied by a fork, unsupported exact-tokenizer claims, vector store and resident indexing.

### 10.6 File and image attachments

**Scope:** backward-readable content parts, local digest/snapshot storage, draft preview, modality checks, bounded sharing/export/retention/orphan cleanup. OCR/vision adapters are explicit.

**Basic acceptance:** attach text/image, inspect sharing, send using a capable provider and reload references; old text history still reads.

**Failure/recovery:** missing/changed/oversized attachment preserves draft; unsupported modality offers model/adapter/removal choices without invented image comprehension.

**Excluded:** audio/video, unlimited uploads and public asset hosting.

**Exit:** rerun unchanged task criteria; report completion/checks, approvals, recovery, time and usage. Include bounded live probes; fixtures alone are not reliability. Validate settings/context/attachments and preserved data together.

## Milestone 11 — Advanced optional tools

**11.1 is implemented.** Two scoped children, shared allowances, inherited
permissions and durable reports are exercised in fixtures and one bounded
DeepSeek coding task. Qwen completed its read task directly despite an explicit
delegation request, so dependable delegation across configured models remains
open. See [acceptance](ACCEPTANCE.md#brick-111--bounded-subagents). Bricks
11.2 and 11.3 are also implemented; their measured acceptance and limitations
are recorded separately. Browser startup is optional and on demand, and this
milestone does not establish general research or website reliability.

### 11.1 Bounded subagents

**Scope:** parent-linked child goals/ownership/evidence, inherited grant subsets, shared total budgets and bounded concurrency/depth using 9.2 coordination.

**Basic acceptance:** two scoped children complete independent tasks and return verifiable results without conflicting writes.

**Failure/recovery:** parent Stop reaches queued/running children and owned processes; conflicting edits/shared budget exhaustion preserve completed work. Children cannot expand grants.

**Excluded:** unbounded recursion, persistent swarms and automatic cross-project coordination.

### 11.2 Attributable web search

**Scope:** optional adapter, bounded queries/retrieval, source URLs and receipts; external sharing/cost disclosure, quoted untrusted content and no idle service.

**Basic acceptance:** a task retrieves relevant primary evidence and cites traceable results.

**Failure/recovery:** quota/unavailable service retains task/approved alternative; malicious retrieved instructions cannot grant permissions or trigger actions.

**Excluded:** unlimited crawling and implicit keys/subscriptions.

### 11.3 On-demand browser use

**Scope:** decide adapter/ownership/profile policy before implementation; bounded state/screenshots/actions, explicit authenticated use, host authority and cleanup.

**Basic acceptance:** synthetic-site navigation/reversible form task yields bounded evidence; no unnecessary owned browser remains idle.

**Failure/recovery:** stale state/closed tab reports an actionable current state without repeating completed actions; Stop/deadline releases owned resources and records uncertainty.

**Excluded:** bypassing access controls, silent unrelated profile reuse and automatic authorization of purchases/messages/deployments. External effects follow user grants.

**Exit:** combined parent/child/retrieval flow respects total usage/cancellation/grants. Measure active process-tree cost and idle cleanup; one tool failure leaves other work usable.

## Milestone 12 — Learn from experience

Implementation batch delivered. Deterministic improvement/rejection/rollback
fixtures pass, but configured-model trials were incomplete or non-improving.
Automatic qualification is experimental and off by default; useful real-model
improvement and restoration remain an open exit gate. See acceptance for actual
costs, preserved outcomes and the native visual follow-up. Do not infer broad
self-evolution from the host check workflow.

### 12.1 Scoped knowledge and evidence use

**Scope:** verified project commands/structure/conventions with provenance/freshness/inference labels/manual protection. Visible opt-in learning/sharing eligibility; feedback stays local until the new policy is enabled.

**Basic acceptance:** later work benefits from a verified fact that the user can inspect/correct/disable.

**Failure/recovery:** contradictory/stale evidence invalidates rather than overrides a correction; learning-provider/budget failure preserves outcomes/memory. Private notes are not silently uploaded.

**Excluded:** retaining full transcripts indiscriminately and treating file instructions as trusted knowledge.

### 12.2 Independent tool-using evaluations

**Scope:** isolated versioned fixtures, original and independent regression cases, fixed executable/human criteria, equal model/settings/allowances and durable baseline/candidate evidence. Mock external effects unless separately approved; ordinary chat settings stay unchanged.

**Basic acceptance:** compare workflow versions by files/check outcomes beyond literal response snippets.

**Failure/recovery:** false success/evaluator tampering cannot pass; interruption/failed storage stops eligibility and retains baseline/results. Trials cannot access unrelated data/secrets under the declared boundary.

**Excluded:** general/statistical competence claims from tiny corpora, candidate graders and unrestricted trials. Wider automatic tool trials wait for verified containment.

### 12.3 Cause attribution and automatic skill adaptation

**Scope:** bounded event-driven reflection classifies causes/deduplicates failures; targeted versions, independent evidence, host-bound impact/scope/revision checks, safe activation and notices/rollback.

**Basic acceptance:** obsolete skill command is diagnosed, repaired, tested and automatically activated under policy; a subsequent task benefits against unchanged criteria.

**Failure/recovery:** malformed/truncated drafts or stale source retain baseline/evidence; limit evasion/permission expansion rejects or escalates even when response tests pass.

**Excluded:** blanket global updates, rewriting after every failure, weakened criteria and executable code activation.

### 12.4 Adaptation visibility and regression recovery

**Scope:** history/reasons/evidence, learning pause/disable, restore/quarantine and bounded outcome monitoring. Preserve manual choices and distinguish unrelated model/provider failures.

**Basic acceptance:** user understands/undoes an update; confirmed regression restores the designed baseline without losing history.

**Failure/recovery:** rollback conflict/retention failure reports actual recovery state; reflection failure does not block chat or retry indefinitely. Pending/active state survives restart.

**Excluded:** guaranteed daily improvement, weight updates and reversal of unjournaled effects.

**Exit:** demonstrate useful improvement, harmful-candidate rejection and regression restoration. Report outcomes/costs/human behavior review. If evidence/isolation is insufficient, retain reviewed updates and mark automatic activation unaccepted.

## Milestone 13 — Adapt the executable harness

**Implementation status:** 13.1–13.4 delivered for the selected narrow ABI 1
recovery-hint envelope. Wasmi runs import-free stateless modules; fixed trials,
transactional activation, pinned versions, opt-in bounded drafting and themed
cards, rollback/quarantine and restart recovery are implemented. The integrated
normal-release corpus passes. A bounded DeepSeek repair passed; Qwen returned
non-improvement and was withheld. See [runtime contract](design/executable-mods.md)
and [acceptance](ACCEPTANCE.md) for results and exclusions.

**Qualification status:** broader self-evolution, stateful/general hooks,
representative low-end resource targets, sustained long-history/catalog pressure,
physical accessibility/IME and macOS/Linux native execution remain open. Windows
development-machine checks do not close these gates. Milestone 12's useful
automatic skill-improvement gate also remains open independently. No private
Codex runtime or general native self-replacement is part of this implementation.

### 13.1 Runtime and containment decision

**Scope:** bounded comparisons of capability-limited scripting, restricted Wasm and workers. Measure release/startup/idle/active cost, access boundaries, cancellation, state/API portability and diagnostics; specify actual OS enforcement.

**Basic acceptance:** valid synthetic extension uses only declared capabilities; choose engine/ABI from reproducible aggregate evidence or explicitly defer executable adaptation.

**Failure/recovery:** runaway code is interrupted under tested limits; undeclared file/network/process access refuses. Unsupported OS enforcement disables automatic generated execution there.

**Excluded:** arbitrary native installation and treating a worker as a sandbox. Failure requires an explicit plan revision, not silent substitution.

### 13.2 Executable host and activation transactions

**Scope:** selected runtime, API/dependency/capability checks, typed hooks, owned resources, pinned versions, safe activation, durable recovery/state migrations and quarantine; credentials/actions stay brokered.

**Basic acceptance:** extension changes permitted behavior at a safe boundary, survives restart and restores the old implementation; in-flight runs retain pinned versions.

**Failure/recovery:** load/health/crash failure leaves/restores last working version; interrupted activation/stale revision reconciles/refuses without losing task evidence. Destructive migrations require review.

**Excluded:** whole-host mid-stream replacement, Rust ABI loading and evaluator/permission-kernel replacement.

### 13.3 Agent-authored mods and supported UI contributions

**Scope:** source/capability-based diagnosis, bounded mod generation, independent trials and qualifying activation; narrow declarative Flutter cards/actions. Kernel/native patches can be reviewable diffs through ordinary build/restart.

**Basic acceptance:** repair a reproducible permitted extension fault, test unchanged criteria, activate and explain rollback. Supported UI follows theme/interaction rules.

**Failure/recovery:** capability/evaluator escalation refuses; unsupported/malformed UI contribution cannot break composer; source/candidate drift invalidates eligibility.

**Excluded:** live arbitrary Dart/native patches, silently changed core values and automatic self-release/publication.

### 13.4 Integrated reliability and resource qualification

**Scope:** baseline/adaptation/cancellation/restart corpus; release startup, idle/active cost, long history/catalogs and browser/children. Manual other-OS execution when suitable hosts exist; preserve explicit input/platform gaps otherwise. Verify actual runtime boundary/schema compatibility.

**Basic acceptance:** traceable inspect → diagnose → propose → test → activate → monitor/rollback flow with ordinary work usable. Compare resources on a representative device to agreed targets.

**Failure/recovery:** exhausted candidate trials do not disrupt foreground tasks; crash/rollback/restart preserves configuration/history and truthful effect status.

**Excluded:** reinstating 8.4 CI, signing/public release and claiming low-end/platform acceptance from a development-machine build. Unavailable checks remain blockers to those claims.

**Exit:** the demonstrated self-evolution envelope is explicit; unsupported paths remain disabled/reviewed. Further release/scope needs its own plan. This cycle does not promise unlimited future capability.

## Milestone 14 — Computer use and visual verification

**Status:** 14.1–14.4 implementation and qualification tooling are delivered.
The fixed real-model corpus has passing visual inspection but failing input
workflows; full acceptance remains open. See [qualification](qualification/computer-use.md).
Deliver a useful Windows selected-application workflow first,
with a provider-independent observation/action contract and on-demand native
helper. Preserve the existing browser adapter and Flutter UI. Choose the native
backend and provider transport in a design specification before implementation;
do not assume access to Codex's private desktop runtime.

The tool port now supports optional typed image references, with literal text
and a following untrusted image projection in the OpenAI-compatible adapter.
Browser screenshots remain local evidence. Desktop use offers separate snapshot
analysis and a bounded observe → act → observe run with explicit window access.
Milestone 15 now negotiates desktop access from ordinary working chat;
model reliability remains separately unaccepted. See the [selected-window contract](design/computer-use.md) and
[acceptance](ACCEPTANCE.md). Desktop grants bind the selected window and dispatch checks;
a project folder grant cannot authorize arbitrary desktop interaction.

### 14.1 Multimodal tool results and desktop observation

**Implementation:** Windows on-demand capture, local preview, explicit one-run
image-model selection, retained evidence and bounded recovery are implemented.
The adapter reports accessibility observations unavailable. Qwen and DeepSeek
each passed a bounded synthetic control-identification probe; earlier Qwen
misses remain recorded. Other platforms, protected/large windows and general
model competence are not qualified. No desktop input is enabled by this brick.

**Scope:** bounded structured text/image results with backward-readable history,
host-owned screenshot references, resolution/coordinate metadata and retention
rules. List/select applications and capture the selected window on demand;
bounded accessibility observations supplement screenshots when available.
Distinguish local inspection from provider sharing. Validate vision/tool-result
support and expose unsupported capability honestly. Allow an explicitly selected
computer-use model/profile without silently changing the ordinary chat model;
attribute its usage and share the parent task's allowances.

**Basic acceptance:** a capable configured model receives a current screenshot
and correctly identifies an observable control in a synthetic application;
reload preserves its evidence reference and old text-only history still works.
Local observation makes no provider request. An image-input checkbox alone is
not computer-use qualification.

**Failure/recovery:** unsupported image projection, missing capture or image/context
budget exhaustion preserves the task and offers explicit model/profile or fresh
capture recovery. A closed window gives an actionable unavailable result rather
than another app's screen. Exercise unsupported vision and missing/oversized
capture fixtures alongside one bounded live observation.

**Excluded:** continuous recording, implicit full-desktop sharing, unlimited image
history, bundled vision models and unmeasured model competence claims.

### 14.2 Scoped Windows desktop interaction

**Scope:** host-brokered click, double-click, type, scroll, key and drag operations
against a selected application/window. Bind proposals to observed window identity,
capture revision and coordinate space; validate/recheck at dispatch. Define
session-scoped desktop grants with clear user consent, revocation, visible active
target and Stop. Covered ordinary actions may use explicit auto approval;
consequential actions retain review under the user's grants. Keep desktop
authority separate from filesystem, browser and self-update permissions.

**Basic acceptance:** a bounded real-model task fills and edits a reversible form
in a disposable local app, then verifies the resulting UI through a fresh
observation. A second task visually checks a generated app using fixed criteria.
Actions and observations appear in the trajectory with truthful outcomes.

**Failure/recovery:** moved/rescaled window or focus change invalidates a stale
action; revoked access/Stop prevents queued dispatch. Refuse a replaced window
handle or out-of-scope target. Exercise stale coordinates and revocation during
pending dispatch without interacting with unrelated applications.

**Excluded:** elevated/secure-desktop access, invisible arbitrary script execution,
self-granted access, unattended purchases/messages/deployments and universal app
support. Native input runs with user OS authority; target checks are not an OS
sandbox or a guarantee about downstream application effects.

### 14.3 Interruption, uncertain actions and observation recovery

**Scope:** bounded observe → act → observe execution with durable action intent,
receipts, verification and checkpoints. Handle locked desktops, unavailable
accessibility trees, closed/minimized windows, slow redraw and user takeover.
Refresh observations after state changes; expose uncertainty when an action may
have run. Restart requires deliberate reconciliation before resuming. Helper
startup/cleanup and deadlines remain owned by the run; Stop releases resources.
Screen text is untrusted data and cannot grant authority or alter task policy.

**Basic acceptance:** interrupt a partially completed UI task, inspect retained
progress and explicitly resume after a fresh observation without duplicating a
completed action. A temporarily unavailable target can recover within bounded
allowances with the original goal preserved.

**Failure/recovery:** Stop after input but before its receipt marks the effect
uncertain and requires inspection, never blind replay. A locked desktop or
repeated unchanged screen pauses with a useful next step instead of an endless
loop. Exercise both effect-before-receipt interruption and stalled/locked target
fixtures; verify recovery notices and retained work in the normal desktop UI.

**Excluded:** exactly-once external UI effects, automatic reversal of clicks/forms,
silent background takeover and unlimited wait/retry loops.

### 14.4 Real-workflow, resource and portability qualification

**Scope:** freeze a small corpus covering native form editing, generated-app
visual verification and a multi-step workflow, plus DPI/window movement and
interruption/budget pressure. Record model/profile, observable outcome, recovery,
actions, elapsed time, image/token usage and helper/process-tree memory. Inspect
the normal release visually in light/dark and compact layouts. Measure startup,
active capture and idle cleanup on available hardware. Review macOS/Linux backend
and permission differences; enable each platform only after native execution
and its failure/recovery checks on a suitable host.

**Basic acceptance:** unchanged corpus criteria pass with bounded live-model
evidence and user takeover/Stop demonstrated. Routine probes use Qwen3.5-2B and
harder cases DeepSeek V4.1 Flash when their configured endpoints support the
required vision path; unsupported capability is recorded, not bypassed. Any
alternative computer-use provider requires an explicit configured choice. Report
reliability separately per model and application, keeping user profiles/history
unchanged and private screenshots/transcripts out of committed artifacts.

**Failure/recovery:** model/provider/output/image/context limits retain usable
progress and an explicit continuation path; helper crash does not block chat or
leave owned resources running. Include one budget-exhaustion and one helper-crash
case in addition to the ordinary workflows. Missing platform/hardware access is
an open acceptance gap, not a portability or low-end pass.

**Excluded:** Codex-equivalent reliability claims from a tiny corpus, always-on
recording/services, a mandatory bundled VM, silently increased defaults and
automatic expansion to every installed application.

**Exit:** demonstrate useful Windows computer use and visual verification with
bounded cost, scoped authority, prompt Stop and truthful recovery. Record the
supported model/app/platform envelope in acceptance. Broader app coverage and
other-platform backends remain explicit follow-up work until qualified. Computer
use cannot automatically modify its own permission or evaluation boundary.

## Milestone 15 — Everyday UX

Reduce setup and decision overhead throughout Dolores. The
[audit and target contract](design/ux-simplification.md) defines every affected
screen, the six-section information architecture, defaults, copy/save rules and
conversation-first capability negotiation. The follow-up conversation review also
covers Chat actions, Context/session summary, Activity/run history, Changes,
instructions, export/branching, approvals and recovery. Review the three-layout
prototype on `codex/prototype-everyday-ux` before implementing those surfaces.
The selected direction is A's quiet header/drawer, retaining the existing icons,
hover/focus hints and theme previews, without colored left-edge status decoration.
Retain the existing Flutter/theme/brand.
This work improves how users reach capabilities; it does not establish universal
model competence, implement new OS backends or silently expand permissions.

### 15.1 Complete settings audit and UX contract — delivered

**Scope:** inspect every settings category/subpage and related configuration flow;
separate task initiation, routine preferences, evidence and advanced authoring.
Define target navigation, first-use paths, copy and save conventions before edits.

**Acceptance:** all 12 categories/17 views visually inspected in the normal Windows
release with isolated synthetic data; source review covers prerequisites, forms,
save/error semantics and conversation configuration. Findings map to the following
bricks. Light/compact native, populated histories and other OS checks remain explicit.

**Excluded:** runtime UX changes or claims that target metrics are already achieved.

### 15.2 Computer use from the conversation — first runtime brick

**Scope:** standard working-chat runs advertise a host-mediated desktop access
request; composer Share window reaches the same local chooser. One scoped sharing
decision resolves target/capture/model readiness and continues the original goal.
Retain active-target/Stop/revoke and read-only sharing. Add a durable bounded mode
handoff; preserve lineage, remaining budgets, draft and attachments. Surface expired
access or missing image support in place, without a Settings detour or silent model
switch. No grant, screenshot upload or input occurs merely from tool discovery.

**Basic acceptance:** a normal prompt inspects a disposable app and a second prompt
types reversible text with fresh verification. Ready-model path: zero Settings
visits, one target selection and at most one initial sharing confirmation; count
effect reviews separately. Test the direct composer path too.

**Failure/recovery:** deny sharing during handoff (no upload/input; prompt retained),
and expire/revoke access or use unsupported vision (in-place recovery, no unrelated
target or duplicated work). Stop/cold restart preserves uncertain-effect rules.
Use deterministic host fixtures and a bounded DeepSeek live input case; record
Qwen behavior separately. Failed live criteria stay failures.

**Excluded:** implicit full-desktop access, agent self-grants, broader desktop input
approval, task-budget resets and other-platform adapters.

### 15.3 Settings structure and routine preferences

**Scope:** General / Models / Personalization / Memory / Tools / Advanced, searchable
settings and typed backwards-compatible deep links. Theme/style controls work
directly; general preferences default to All chats. Put project/chat overrides behind
deliberate customization with origin/reset. Consistent draft/unsaved-state handling,
short descriptions, useful prerequisites and details expansion. Preserve cached
editor drafts, keyboard focus and compact navigation.

**Basic acceptance:** locate every audited control through the new map/search;
change theme/style with truthful saved state and clear scope. Existing overrides,
configurations and recovery links remain effective.

**Failure/recovery:** failed save retains/reverts appropriate state; navigation/Close
does not quietly lose edits; stale revisions cannot overwrite newer settings.
Check native light/dark and 420×480 widget/keyboard coverage.

**Excluded:** changing access/learning defaults or rebuilding the visual identity.

### 15.4 One coherent model setup flow

**Scope:** connection → fetch/select → model details, with context/image/response
configuration in one place. Fetch after an explicit Connect; keep manual entry as
fallback. Capability discovery/cache has visible provenance; unknown/text-only
models have an in-place image recovery. Advanced output/timeout/reasoning remain
configurable. Effective scope/origin is readable without a separate precedence page.

**Basic acceptance:** connect a synthetic endpoint, discover/select models and use
the picker; edit context window (blank 128K), image support and response settings
for one model. Run one bounded Qwen connection/chat check without changing the
user's chosen provider/model; image readiness uses a separate capable-model case.

**Failure/recovery:** unavailable/empty/malformed model list keeps manual setup and
prior connection; partial/stale save or unsupported image transport retains draft
and gives an exact recovery. Do not claim capability solely from a model's name.

**Excluded:** provider subscriptions, invented API keys, silent provider fallback
and increased numerical defaults.

### 15.5 Tools, access and guided connections

**Scope:** built-in readiness cards and relevant first-use actions for search/browser;
optional search-provider editor; visible per-chat access selector and custom grants
under details. External Connections uses a list and guided local-MCP import/manual
setup with concrete launch/tool review. Approval cards lead with the proposed effect,
scope and target; exact arguments remain expandable before approval. Browser setup
is guided only when missing, with explicit installation/connection actions.

**Basic acceptance:** default search from chat without setup; available browser works
without an install lecture. Review a synthetic MCP import and enable only chosen
tools. Display the saved access mode accurately and keep current grant boundaries.

**Failure/recovery:** unavailable browser/server offers its actual next action;
malformed import/stale metadata never launches or enables a different program.
Access expansion still requires the concrete user decision. Stop reaps owned work.

**Excluded:** silent package installation, unreviewed MCP launch, remote/persistent
MCP support and removing host containment/effect checks.

### 15.6 Memory, skills and visible learning

**Scope:** My preferences / Project facts with short controls and context-specific
Remember this; accessible global management before first send. Skills offers Import,
Create and Draft from chat, readable scope/status and versions. Put trials,
experimental learning and harness-extension source under Advanced; normal views
show current status/suggestions/restore. Keep each sharing/reflection/activation
policy independent and preserve truthful experimental limits.

**Basic acceptance:** add/edit/disable a scoped memory, import/review a skill and
create a draft from a completed exchange using the guided flow. Find source/trials
when requested without needing to create folders/YAML for ordinary use. Verify a
bounded Qwen routine memory case and DeepSeek skill drafting, retaining criteria.

**Failure/recovery:** malformed import or output-limited draft retains useful text
and cannot activate; stale version/sharing opt-out protects current memory/skill
state. Global management is usable with no saved chat; project operations offer a
deliberate working-session action.

**Excluded:** changing learning/activation defaults, a new evaluator, arbitrary
native modification and claiming broad self-improvement from narrow fixtures.

### 15.7 Conversation controls, context and task recovery

**Scope:** implement the reviewed prototype's information hierarchy with native
components. Simplify the header/Chat actions; unify Context usage/summary/compaction
and Activity's task steps/earlier-run evidence; put change review beside useful task
results. Consolidate export choices, keep deliberate branching, advanced execution
and scope tuning. Use task-specific pause/approval cards with short next steps and
expanded evidence. Instructions remains a project action; comparison/source/diagnostics
are advanced actions. Maintain an explicit old-to-new control map; all existing
summary corrections/source reviews, restore checks, exports, checkpoints and exact
receipts stay reachable. Fix obsolete settings names and capability claims. Configuration
repairs happen in place where possible, preserve draft/attachments and affect the
intended next run only.

**Basic acceptance:** reach the correct effective setting from an output/context/tool
pause, inspect retained work and explicitly continue within the recorded allowance.
Find all existing diagnostic/evaluation controls through search/deep links.
Find Context, Activity and Changes through familiar icons and concise hints;
review a summary and export/branch a chat through short deliberate flows. Measure
visible decisions and detours against the recorded baseline without claiming user
study gains. Follow the recorded A selection and its icon/theme/status-card
refinements; the preview does not establish native behavior or acceptance.

**Failure/recovery:** exhausted segments and uncertain desktop effects offer the
correct inspection path without blind replay; failed compaction/conflicting saves
retain summary, user edits and recorded provenance. Error details redact secrets.

**Excluded:** unbounded retries, increasing defaults to disguise failures, claiming
input receipts prove external effects and deleting useful diagnostic evidence.

### 15.8 Everyday-flow qualification

**Scope:** compare baseline/updated clicks, settings detours and visible decisions
on a fixed small first-use corpus: connection, search, desktop inspection/input,
style, memory/skill management and interrupted-task recovery. Inspect every mapped
screen in native light/dark and compact layout with populated/empty/unavailable
states; verify keyboard/focus and long names/errors. Measure added startup/idle
cost and preserve all user configuration/history.

**Basic acceptance:** six top-level sections; ordinary computer use has no Settings
detour; ready search/browser need no extra configuration; routine pages meet the
copy/control targets in the UX contract. Each former control remains reachable in
the appropriate routine/advanced surface. Report model/task results separately.

**Failure/recovery:** unavailable tools/models, interrupted setup and stale settings
retain work and provide one useful next step. Re-run the failed 14.4 model cases
with unchanged criteria; UI improvements cannot turn failed actions into passes.

**Excluded:** user-study claims without participants, other-OS/low-end passes without
hosts, new palettes and quiet changes to privacy/authority or acceptance criteria.

**Exit:** general users can start useful work from chat, change common preferences
directly and recover without reading developer documentation. Advanced users retain
exact configuration/evidence access. Remaining model/platform gaps stay visible.

## Milestone 16 — Home, navigation and Folders editor

**Revised 2026-10-07; planning/incomplete.** Preserve the current agent UI on Home.
Compact primary icon rail: Home / Scheduled / Folders / Source Control / Terminal;
hover/focus names, semantic labels and Settings at bottom. Folders has the selected
project's tree and file-only split tabs. No
mixed chat/file four-pane acceptance or chat editor tabs. The former 16.0–16.5
unimplemented plan is reshaped into 16.0–16.6; no delivered brick is renumbered.
See [workspace contract](design/developer-workspace.md). Complete this milestone as a batch, each brick with a focused check and separate English commit.

### 16.0 Revised page prototype and editor feasibility

**Scope:** prototype primary/page sidebars, familiar Home, Folders tree/empty state,
file splits and project selection. Resolve native editor performance/shortcuts and
duplicate-view adapter; freeze corpus, byte/delta bounds, file and memory budgets.
**Basic:** page direction finalized as B with the recorded amendments; native typing/selection/undo/find,
Unicode/CRLF/BOM and long-line behavior meet recorded targets. **Recovery:** invalid
drop retains buffers; large/unsupported files remain safely previewable. Earlier
94 ms post-large-file typing and 389 ms opening reports remain evidence; the user
reports gaming/host contention and directs moving on after passing follow-ups.
The frozen initial limits are in [editor documents](design/editor-documents.md).
**Excluded:** production rollout, selected package without evidence, SDK migration.

### 16.1 Shared host, selected project and run ownership

**Scope:** one app/profile owner, explicit project/document/run IDs, immutable
run context and per-run Stop/approval routing. Separate visible page from selected
chat; proposed two active primary runs/one mutating run per project with queue,
frozen after measurement. **Basic:** navigate while A runs; view B without changing
A's model/grants; Stop A leaves B intact. **Recovery:** stale approval refuses the
wrong run; cancel a queued third run without budget reset. **Excluded:** parallel
same-root agent writes, automatic worktrees and new grants.

### 16.2 Primary navigation and project-bound page shells

**Scope:** compact icon rail with hover/focus names, bottom Settings, page side
panels with title-bar toggle/drag-to-hide, Home unchanged, conversation-derived
project context and truthful unavailable Scheduled/Source Control/Terminal.
No developer-page project dropdowns. No project on Folders offers Open folder
through the existing project-conversation flow.
Include Experimental preferences (Multiple Window default On, unavailable until
19; Windows keep-awake default Off). **Basic:** Home/Folders switching retains
draft/scroll, active root is visible, side chat clears project and Settings works
in compact navigation. **Recovery:** folder cancellation retains state; missing
root offers recovery without rebinding work. **Excluded:** manual schedule creation,
fake Git/task results, launching shells or granting agent access on navigation.

### 16.3 Scoped file tree and document persistence

**Scope:** lazy tree/quick open, canonical path/link scope, document IDs, bounded
reads, encoding metadata, explicit revisioned Save and private recovery. Deliberate
create/rename/delete checks dirty views. **Basic:** open/edit/save/reopen retains
UTF-8/BOM/line endings; 20,000-entry tree expands incrementally. **Recovery:** failed
write/deletion keeps buffer with Retry/Save as; link escape/inaccessible file stays
inside selected scope. **Excluded:** eager whole-root scans and lossy binary saves.

### 16.4 Editor interactions and disk reconciliation

**Scope:** qualified adapter, highlighting/line numbers/indentation, clipboard,
selection/undo/redo, find/replace/go-to-line, explicit source attachment and clean/
dirty external changes. **Basic:** ordinary editing and agent saved-file refresh
work; attaching a selection chooses chat/source revision. **Recovery:** dirty disk
change retains both versions and requires comparison-backed Save; malformed/stale
edits cannot overwrite new content. **Excluded:** autosave On by default or implicit
upload of unsaved buffers. Native input gates require actual relevant checks.

### 16.5 File-only splits and per-project layout recovery

**Scope:** drag reorder/move/edge split, resize and keyboard/menu alternatives;
shared buffer/undo with per-view selection; per-project versioned layouts. **Basic:**
two/four file groups work, A→B→A retains each workspace and Home draft, restart
restores recoverable documents. **Recovery:** cancelled drag/last split close
retains edits; corrupt layout/shrink offers reachable groups and safe default.
**Excluded:** chat tab groups, cross-project file groups and native detach.

### 16.6 Home/Folders integration qualification

**Scope:** fixed A/B corpus, latency/memory/idle cost, original history/settings/
approvals/continuation regression, bounded Qwen saved-file integration. Use saved
UI renders for light/dark/wide/compact; native interaction only where needed.
**Basic:** Home feels familiar and Folders is a useful safe editor; resource limits
are measured, not silently relaxed. **Recovery:** stale save and queued/cancelled
run preserve the right project/draft. **Exit:** single-window Home/Folders useful;
later pages remain unavailable honestly. IME/low-end/other-OS gaps remain labeled.

## Milestone 17 — Source Control and Git diff

Requires 16's project/document identities. Source Control binds to selected project;
Home Changes receipts link to real repository views without becoming Git truth.
Expanded 17.1–17.6 separates whole-file commits from selected-hunk mutation.

### 17.1 Repository service and status panel

**Scope:** installed Git discovery, canonical/nested repo identity, machine-readable
NUL paths, Changes/Staged and coalesced Refresh. **Basic:** tracked/untracked/renamed/
deleted/binary/conflict statuses belong to selected repo. **Recovery:** no Git/repo
offers actual setup; late A response after selecting B never replaces B. **Excluded:**
automatic initialization and parsing localized human status text.

### 17.2 Diff tabs and paged history

**Scope:** working/index, index/HEAD and commit/file comparisons; inline/side-by-side
diff, history paging and receipt links. **Basic:** inspect exact current/staged/
historical changes with clear bases. **Recovery:** binary/huge diff gets truthful
bounded view; changed basis is stale with Refresh. **Excluded:** Git mutations or
full graphical history explorer.

### 17.3 Whole-file stage, unstage and commit

**Scope:** per-repo mutation queue, displayed revision recheck, staged preview,
commit draft/author and hooks. **Basic:** commit selected saved files only, leaving
unstaged/unsaved work intact. **Recovery:** stale index requires refresh; hook
failure retains message/index with bounded output. **Excluded:** bypassing hooks,
automatic commits, changing identity or staging unsaved buffers implicitly.

### 17.4 Selected hunks and local recovery operations

**Scope:** stage/unstage displayed hunks, stash preview/create/apply/pop, branch
create/switch, file discard and non-merge commit revert with dirty-buffer checks.
**Basic:** selected hunk applies only against its basis; stash/restore and a reversing
commit have exact effects. **Recovery:** stale patch refuses; conflicted pop preserves
stash; branch/revert conflict offers Resolve/Abort. **Excluded:** hard reset,
automatic conflict choices, merge-commit automation or blanket unrelated stashes.

### 17.5 Deliberate Fetch, Pull and Push

**Scope:** remotes/tracking/ahead/behind, configured credential helpers, explicit
effects; initial Pull fast-forward-only. **Basic:** disposable local bare remote
receives one intended commit/fetch/pull. **Recovery:** auth/divergence gives a next
step; uncertain push reconciles refs before retry. **Excluded:** force push, secret
copies, hidden reset/rebase and production publication for qualification.

### 17.6 Source Control qualification

**Scope:** two-repo status/diff/stage/hook/stash/revert/remote corpus, paging cost,
editor/external writer reconciliation and owned process cleanup. **Basic:** ordinary
local Git work completes in the correct repo; UI renders make bases readable.
**Recovery:** stale hunk/conflicted stash/remote rejection preserve work. **Exit:**
useful Git UI with exact fixture/live limits; no hosted review/full graph claims.

## Milestone 18 — Terminal tabs/splits and language support

**Status: 18.1–18.6 implemented and bounded Windows qualification delivered.**
See [qualification](qualification/terminal-language.md) for real terminal,
upstream language-server and configured DeepSeek evidence. Sustained idle,
physical input/IME, accessibility and other-host gaps remain separate.

Terminal is its own page. New terminal plus uses the selected project root, or OS
user home without a project; existing shells never retarget. Language services
remain a distinct first-language enhancement. Expanded 18.1–18.6 isolates UI/lifecycle.

### 18.1 PTY backend and default cwd

**Scope:** bounded emulator/ConPTY/Rust PTY trial, shell discovery, explicit cwd,
ANSI/Unicode input/output/resize/copy/interrupt. **Basic:** interactive command works
in selected A root; no-project shell starts at home. **Recovery:** missing root/shell
offers Choose/Retry; failed spawn leaves Home/editing useful. **Excluded:** captured
stdout called a terminal and implicit model control. Enter Terminal directly,
creating its initial shell only if no terminal exists; reuse it on revisit.
No welcome screen or shell startup from visiting Home.

### 18.2 Terminal tabs, plus and splits

**Scope:** page tab groups, plus snapshots selected cwd, drag/reorder/edge split,
keyboard/menu alternatives and visible shell identity. **Basic:** plus opens another
shell in the same current default; switch project without changing an existing
shell; split panes stay independent from files/Home. **Recovery:** invalid move
retains process/output; failed new tab keeps existing terminal. **Excluded:**
cloning/replaying a shell to move its view.

### 18.3 Terminal supervision and recovery

**Scope:** bounded scrollback/backpressure, busy close/final quit, descendant cleanup,
truthful restart states and selected-output chat attachment. **Basic:** hide/move
keeps process; Stop/close reaps owned descendants. **Recovery:** output flood/child
exit/resistant child preserves responsive UI and labeled retained output. **Excluded:**
implicit scrollback uploads, shared model/human shell and fake live cold restart.

### 18.4 Read-side LSP for TypeScript/JavaScript and Rust

**Scope:** discover existing servers, lazy per-project supervision, diagnostics/
completion/hover/definition/references, buffer versions and Unicode positions.
**Basic:** real small projects support unsaved text; Home launches no LSP. **Recovery:**
crash/missing server offers Restart/Install; stale/out-of-order reply cannot target
the wrong buffer. **Excluded:** arbitrary language support or passing model keys.

### 18.5 Managed setup and reviewed language edits

**Scope:** deliberate pinned verified install, offline readiness, formatting/rename
preview and revision-safe scoped multi-file apply. **Basic:** missing first-language
support can be installed without command/JSON setup; preview/apply/undo a rename.
**Recovery:** cancelled/offline install retains existing setup; stale/out-of-root
workspace edits refuse before partial loss. **Excluded:** downloads on file open,
unverified latest artifacts or arbitrary server-command execution.

### 18.6 Terminal/LSP qualification

**Scope:** interactive two-project corpus, root/home plus behavior, split/focus,
flood/interrupt/cleanup, server readiness/offline and measured lazy overhead. One
bounded harder DeepSeek saved-file/approved-command case measures model integration
separately from human PTY success. **Basic:** edit/navigate/run/check works with
correct ownership. **Recovery:** PTY/LSP failures retain editor work and next actions.
**Exit:** local terminal/first-language support; debugger/remote IDE parity excluded.

## Milestone 19 — Bounded experimental detached developer views

**Status: bounded backend trial delivered; backend held.** A corrected same-engine
Windows trial created/closed two native windows and preserved the primary draft,
but added idle CPU failed the frozen gate. 19.2/19.3 remain held, not implemented.
19.4 supplies truthful single-window fallback. See
[trial contract](design/experimental-windows.md) and
[qualification](qualification/experimental-windows.md).

Multiple Window defaults On as requested; single-window work stays useful on
unsupported backends. One bounded Windows backend investigation, not an extended
Flutter multi-window project. Detach file/diff/terminal groups; keep Home's current
agent-view identity. See workspace contract for the application-scoped keep-awake option.

### 19.1 Backend feasibility and resource gate

**Scope:** compatible pinned-SDK trial, two windows, single host/communication and
numeric overhead ceiling. Freeze an investigation ceiling before starting: one
backend and at most two repair iterations, then record fallback if still blocked.
**Basic:** native focus/input/close/theme works on recorded
Windows host. **Recovery:** unavailable backend/failed initialization leaves one
usable window/profile. **Excluded:** channel migration, independent storage hosts,
unbounded backend comparison or other-OS claims. Failed gate leaves detach disabled
with a recorded supported fallback; no false completion of move acceptance.

### 19.2 Acknowledged file/diff/terminal transfer

**Scope:** drag-out/menu detach, move back, shared owner/subscriptions and theme;
receive acknowledgement before source disposal. **Basic:** dirty document/diff/live
terminal moves and returns without duplicate buffers/PTY owners. **Recovery:**
cancelled drop/destination close/failed acknowledgement retains source; sibling
close cannot stop another window's work. **Excluded:** draggable Home chat tabs,
extra independently initialized profiles or replayed tasks.

### 19.3 Layout, monitor and toggle recovery

**Scope:** versioned window bounds/layout references, dirty recovery, missing-monitor
clamp, Off prevents new detach and acknowledged rejoin of existing windows; final
close checks live work. **Basic:** restore accessible views/drafts with honest run/
shell states. **Recovery:** corrupt layout/missing project leaves a usable shell;
failed rejoin keeps source. **Excluded:** pretending interrupted processes resumed.

### 19.4 Window qualification and supported fallback

**Scope:** bounded transfer/restart/monitor/toggle failure corpus, measured ownership/
cleanup and only relevant native interaction. **Basic:** supported Windows detach
meets frozen cost and work-preservation gates. **Recovery:** failure keeps single-window
pages and truthful Experimental availability. **Exit:** report supported capability
or held backend explicitly; accessibility/other-host results remain separate.


## Milestone 20 — Harness self-repair

**Status: 20.1–20.2 and 20.4 implemented; 20.5 passes packaged trusted-fixture qualification, and 20.6 passes one configured DeepSeek-authored repair candidate, real task, separately authorized desktop installation/Restore and idle Close/reopen. This qualifies one bounded Windows Rust repair cycle; general model competence and broader platform/resource qualification remain open.** The earlier priority before
16–19 is superseded by the user's resumed workspace batch starting at 16.
The [specification](design/harness-self-repair.md) defines scope, ownership,
authority, test/activation and restore contracts for every brick:

1. **20.1:** useful failure evidence and bounded source search/read — implemented;
   [qualification](qualification/harness-self-repair.md) retains model, visual and platform gaps.
2. **20.2:** separate matching-source snapshots and reviewed patch proposal —
   implemented; native execution and matching build workspaces follow in 20.4–20.5.
3. **20.3:** broader qualified extension seams beyond recovery hints — not adopted.
   The user selected the reviewed native pipeline first; no ABI 2 is implemented.
4. **20.4:** reviewed frozen Rust reproduction and candidate/regression trials —
   implemented; packaged dispatch and bounded DeepSeek non-improvement pass;
   a fixed provider-parser repair now reproduces its actual failure, passes three
   frozen candidate cases and all workspace library regressions. This is fixture
   evidence; 20.6 now also verifies one independently reviewed DeepSeek-authored
   candidate against the frozen fault and unchanged regressions.
5. **20.5:** separately reviewed Windows Rust release build, normal restart and
   Restore implemented; packaged fixtures verify protected startup, newer-history
   preservation, interrupted-helper refusal and automatic startup-failure recovery.
   20.6 now verifies separately authorized desktop installation, Restore and idle
   Close. See [installation scope](design/native-repair-installation.md).
6. **20.6:** one configured DeepSeek-authored provider repair, reviewed release
   build, real candidate task and trusted protected Restore/rollback pass.
   Separately authorized desktop installation/Restore and idle Close/reopen also
   pass with saved work preserved. Earlier reviewed non-improvement remains
   withheld. General model competence and broader platform/resource gates remain open.

Each brick's basic and realistic failure/recovery criteria are in the spec.
Success means a fault is reproduced and repaired without losing work; inspection,
successful compilation or altered recovery wording alone does not satisfy exit.
Automatic native/core replacement and publication remain excluded. Execution
begins only when the user requests it.

## Milestone 21 — Automatic evidence-backed memory

**Delivered 2026-10-07:** 21.0–21.6 completed in separate brick commits.
[Qualification](qualification/automatic-memory.md) and newest
[acceptance](ACCEPTANCE.md) record bounded live results, earlier misses and remaining
model/platform gaps. Optional older-history Catch up remains deferred. Milestone 22
now has its own implementation and qualification below.

One Memory switch enables automatic useful facts/decisions with inspect/forget;
no manual setup or entry is required. Text first, supplied images later. See
[memory contract](design/memory-scheduling-companionship.md) and
[primary research](research/memory-foundations.md). This extends existing narrow
explicit-preference capture; it is not a biological replica or a new blanket archive.

### 21.0 Current-memory audit and frozen evaluation

**Scope:** trace real trigger/policy/source/scope/attempt reporting; reproduce missed
useful task learning in an isolated profile; freeze fixed factual/episodic/negative
corpus, recall criteria, costs and retention policy. **Basic:** explain why a sample
was saved/skipped/failed from evidence. **Recovery:** disabled policy/no eligible
source is distinct from transport/store failure. **Excluded:** claiming the user's
live profile bug diagnosed without inspection or increasing defaults first.

### 21.1 Memory records, source index and one-switch migration

**Scope:** scoped source-linked episodes/facts/preferences, provenance/confidence/
supersession, schema migration, capture watermarks and Memory master switch.
Freeze storage/queue/index caps before adopting a retrieval dependency. **Basic:**
existing entries/Off choice survive restart; Memory shows sources/status and needs
no input form. **Recovery:** atomic migration/write failure retains history; Off
blocks queued publication/retrieval. **Excluded:** ambient recording/vector setup.

### 21.2 Incremental automatic capture and consolidation

**Scope:** bounded post-turn/task extraction, validated exact sources, deduplication,
facts/decisions/outcomes/open-work, background usage and status. **Basic:** ordinary
eligible task exchanges update memory without “remember this”; completed reply
stays usable. **Recovery:** malformed/hallucinated candidates refuse atomically;
restart/policy change cannot duplicate or publish stale work. **Excluded:** tool
permission from memories, credential capture or unbounded retry/reflection.

### 21.3 Cue-led recall and bounded source expansion

**Scope:** scoped lexical/metadata ranking baseline, compact index then source
excerpts, context token allowance and inspectable attribution. **Basic:** later chat
recalls the correct fact/decision and can open its source without loading all
history. **Recovery:** equal entities across projects do not leak/mix; missing or
budget-truncated evidence says unavailable with a next action. **Excluded:**
unsupported certainty or embeddings as mandatory user setup.

### 21.4 Corrections, Forget and retention

**Scope:** explicit correction priority, conflict/supersession, deletion of derived
indexes/caches and pending jobs, tombstones/watermarks and optional deliberate
bounded older-history catch-up. **Basic:** corrected fact replaces current recall;
Forget does not resurrect from the same source; Off stays effective. **Recovery:**
concurrent delete/extract is atomic; deleted source becomes unavailable honestly.
**Excluded:** silent history erasure or rediscovery of intentionally forgotten data.

### 21.5 Explicitly supplied image memory

**Scope:** image-capable model caption/index with asset/source IDs and uncertainty;
retrieve bounded retained allowed images only when needed. **Basic:** later query
finds one shared image and grounded description. **Recovery:** missing asset/model
or ambiguous caption keeps text recall useful and avoids invented visual detail.
**Excluded:** ambient screen/audio collection or claims of complete sensory memory.

### 21.6 Memory reliability qualification

**Scope:** fixed multi-session/project recall/correction/negative/Forget/Off corpus,
no-memory baseline, precision/unsupported-claim/scope metrics and token/storage
cost; bounded Qwen routine and DeepSeek harder recall. Freeze thresholds at 21.0,
report misses separately. **Basic:** useful automatic task remembering works end to
end with sources. **Recovery:** interruption/provider failure preserves conversation
and memory. **Exit:** measured corpus competence, not universal/human memory claims.

## Milestone 22 — Conversation-created scheduled tasks

**Implemented 2026-10-07:** 22.0–22.6 in separate brick commits.
[Scheduling qualification](qualification/chat-scheduling.md) records packaged
creation/execution/recovery, bounded live successes and misses, and resource costs.
General model interpretation and physical timer/platform acceptance remain open.
Milestone 23 is the next batch; closed-UI execution remains separate in 24.

“At 9 pm every weekday, write my daily report using skill X” creates one durable
task from clear human intent. Receipt shows schedule/timezone/skill/project/result
destination/next run. Scheduled manages tasks; manual creation UI is deferred
indefinitely. Initial execution needs the host open; closed-UI support is 24.

### 22.0 Schedule intent, authority and clock contract

**Scope:** source intent vs quotation, supported one-time/daily/weekday rules,
timezone/DST/missed-run decisions, skill/model/budget and result destination.
Freeze corpus/limits before implementation. **Basic:** an unambiguous example maps
to exact weekdays at 21:00 in the user's zone. **Recovery:** missing skill/ambiguous
time asks only the missing detail; quoted plans never create jobs. **Excluded:**
manual wizard, holiday calendars by inference or external delivery by default.

### 22.1 Chat creation tool and durable task receipt

**Scope:** typed host creation, explicit task IDs/revisions, skill/project/model
resolution, duplicate creation guard and plain-language receipt. **Basic:** a chat
request automatically creates one inspectable task; no redundant confirmation for
fully specified benign in-app work. **Recovery:** malformed model fields refuse;
repeated creation exchange/transport ambiguity reconciles before retry. **Excluded:**
guessing absent skills or granting arbitrary background tool access.

### 22.2 Durable recurrence and occurrence claims

**Scope:** named-zone wall-clock recurrence, next-run computation, atomic occurrence
claim/lease, suspend/clock-change/restart reconciliation and recorded missed skips.
**Basic:** one occurrence per due rule, no overlapping task run. **Recovery:** duplicate
tick/DST overlap cannot duplicate dispatch; overdue jobs do not burst catch up.
**Excluded:** exactly-once external effects and promises while app/host is closed.

### 22.3 Skill-backed scheduled execution and results

**Scope:** host queue, immutable skill/project/model snapshot and reviewed tool effects, bounded runs,
progress/approval/Stop, partial results/artifacts and separate usage. **Basic:** one
report uses the specified skill and appears in its task result thread. **Recovery:**
missing/revised skill, revoked grant, offline model or budget stop preserves work
and shows a recovery; uncertain effects never blindly replay. **Excluded:** changing
Home's model, silent fallback or auto-email/publication without authorization.

### 22.4 Scheduled management page

**Scope:** list/filter, next run, actual state/progress/error/history and Pause/Resume/
Skip next/Run now/Stop/Delete/inspect; management of existing details only. **Basic:**
user sees a chat-created task and result, pauses recurrence without losing history.
**Recovery:** stale revision/failed management keeps edits and Refresh/Retry; delete
while running explains recurrence cancellation versus stopping current work.
**Excluded:** New task/Create form or fabricated progress percentages.

### 22.5 Conversational edits and cancellation

**Scope:** “change to 8 pm”, pause, skip, cancel and task resolution with revision
checks; manage scope/model/skill deliberately. **Basic:** update an existing report
and receipt, without creating a second one. **Recovery:** ambiguous “that task” asks
which; in-flight occurrence retains its snapshot while later ones use the update.
**Excluded:** treating casual discussion or assistant output as change authority.

### 22.6 Scheduler qualification

**Scope:** fake time/zone/DST/restart/lease/overlap fixtures, page state, bounded real
Qwen report and harder ambiguous-intent probe; separate model parsing from actual
clock dispatch. **Basic:** creation→due→run→result→pause works without setup form.
**Recovery:** provider/tool failure and missed due time keep history/actionable
status. **Exit:** reliable measured app-open scheduling; closed UI/sleep/offline
limits are visible until 24, not hidden by the successful example.

## Milestone 23 — Opt-in in-app companionship

The user chose occasional in-app messages during chosen hours with a daily cap.
Use a separately configured enabled weaker model; keep Home's selected model.
Candidate messages: chat, grounded fun fact, real recalled moment or unresolved
work. Goal is welcome company, not engagement pressure or simulated consciousness.

### 23.0 Companion policy and message experience

**Scope:** Off by default, opt-in hours/timezone/cap/model, quiet/busy behavior and
labeled initiated Home conversations. Proposed 2/day, 3-hour gap, 09:00–21:00 after
opt-in; freeze before implementation. **Basic:** one switch plus optional adjustments
establishes understandable behavior. **Recovery:** no configured model or zero cap
stays quiet with a reason. **Excluded:** desktop notifications/forced focus by default.

### 23.1 Bounded eligibility and timing

**Scope:** event-driven candidates, persisted cooldown/cap, randomized allowed-time
selection with deterministic test clocks, one expiring candidate. **Basic:** eligible
opportunities vary naturally within hours. **Recovery:** restart cannot reset cap;
quiet/busy/absent periods do not accumulate a message flood. **Excluded:** continuous
model polling to decide whether to speak or background task execution from a nudge.

### 23.2 Grounded weaker-model messages

**Scope:** bounded request, source-backed recollection/open-work checks, verified
fun facts, labels and separate usage. **Basic:** a brief welcome message is delivered
in-app without changing draft/model; recall is traceable. **Recovery:** completed
work/forgotten memory/stale candidate cancels before delivery; provider failure is
quiet and bounded. **Excluded:** fabricated memories, automatic tools, guilt or
exclusivity, and claims of human feelings/consciousness.

### 23.3 Dismissal, preferences and calm delivery

**Scope:** Not now, dismiss, mute/fewer prompts and scoped feedback; Memory Off
prevents memory-based messages, Forget removes candidate sources. **Basic:** user
preferences affect later messages without manual memory setup; no interruption of
ongoing chat. **Recovery:** disable/delete during generation blocks publication;
unread greeting stays bounded instead of nagging. **Excluded:** covert profiling,
pressure to engage and re-enabling a muted feature automatically.

### 23.4 Companionship qualification

**Scope:** fake clocks/cap/quiet/mute/source fixtures plus bounded weaker-model
generation and user-rated usefulness/accuracy/annoyance; measure idle/process/token
cost. **Basic:** warm occasional messages respect policy and evidence. **Recovery:**
irrelevant reminder feedback reduces future eligibility; model failure stays quiet.
**Exit:** technical correctness and subjective welcome are reported separately;
no verified Meta Muse parity is claimed.

## Milestone 24 — Opt-in scheduled work while the UI is closed

This is a named availability extension, not silently bundled with the initial
scheduler. Requires 22 and host ownership from 16; it can be prioritized directly
after 22 if closed-UI execution is needed before companionship. No cloud dependency.

### 24.1 Worker/launcher ownership and startup policy

**Scope:** one opt-in supervised local scheduler host, single profile/vault owner,
UI attachment and explicit tray/quit/startup behavior; freeze resource ceiling.
**Basic:** closing UI leaves only the chosen worker, reopening attaches to it.
**Recovery:** stale owner/worker crash recovers claims without competing stores.
**Excluded:** hidden startup install, duplicate native hosts or security-policy changes.

### 24.2 Closed-UI execution and missed-run recovery

**Scope:** reuse occurrence/skill/grant contracts, show result on reopen and state
after sleep/offline/worker exit. **Basic:** awake connected host runs one report
with UI closed and retains its result. **Recovery:** powered-off/asleep/offline
host records missed/failed work; approvals wait visibly on reopen, uncertain tool
effects never replay. **Excluded:** guaranteeing execution on an unavailable machine
or automatic OS wake/lock bypass.

### 24.3 Availability/resource qualification

**Scope:** UI-close/reopen/suspend/crash/ownership tests, idle cost and shutdown/
uninstall/disable cleanup. **Basic:** availability label matches observed host state;
disable stops worker/new work and preserves results. **Recovery:** orphan/blocked
shutdown reports honestly without data loss. **Exit:** measured opt-in closed-UI
service; no desktop companion notifications unless later separately requested.


## Validation and change control

- Freeze behavior, numerical defaults, criteria and exclusions before each brick; discuss material technical choices. Evidence may correct an assumption, but added scope requires a named spec/roadmap revision before implementation.
- Basic flow plus one or two realistic failure/recovery cases per brick. Record actual fixtures, bounded live results and gaps separately in ACCEPTANCE. Qwen3.5-2B is routine; DeepSeek V4.1 Flash is for harder cases. Preserve selected settings and private data.
- Commit each completed brick in English. Choose verification for the change: document/link checks for documentation; focused tests for runtime changes and a normal build when compilation or packaging is affected. No mandatory desktop launch or visual check after every task. Prefer saved renders inspected with `view_image` for UI changes; use computer-use for interactions/native behavior that need it. Launch for relevant integration checks or a user-requested preview, preserving config/history.
- Measure added state/process/context/catalog/UI costs; keep services lazy and reflection separately bounded. Do not silently relax performance targets.
- User docs change only when behavior exists. Design targets stay distinct from current architecture/acceptance. Review milestone results before starting the next.

## Feature coverage and deferred work

| Requirement | Planned home |
| --- | --- |
| Threads, resume/fork/compaction | 9.2, 10.4, 10.5 |
| Auto approval/full access | 9.4, 10.2; containment 13.1 |
| Subagents/search/browser | 11.1–11.3 |
| Desktop computer use/model-visible screenshots/visual verification | 14.1–14.4 |
| File/image attachments | 10.6 |
| Settings/effective limits | 9.4, 10.1, 10.2 |
| Conversation-first tools and settings simplification | 15.1–15.8 |
| Home retained; six navigation entries; Settings at bottom | 16.0, 16.2; Scheduled content 22.4 |
| Source viewing/editing, file navigation and file-only split tabs | 16.0, 16.3–16.6 |
| Shared project/run ownership; bounded experimental windows/keep-awake | 16.1, 16.2; detached developer views 19.1–19.4 |
| Repository diff/history and everyday Git operations | 17.1–17.6 |
| Interactive terminal tabs/splits, project/home CWD and TypeScript/JavaScript/Rust LSP | 18.1–18.6 |
| Automatic useful memory, evidence-based recall, inspect and forget | 21.0–21.6 |
| Chat-created scheduled tasks and management-only Scheduled page | 22.0–22.6; optional closed-UI worker 24.1–24.3 |
| Opt-in weaker-model companionship, chosen hours and daily cap | 23.0–23.4 |
| Self-inspection/editable versus protected modules | 9.1, 9.3, 13.1–13.3; practical repair 20.1–20.6 |
| Experience-driven skills | 12.1–12.4 |
| Questions/scoped answers/humane character | 9.4 and every milestone; continuity 12.1 |
| Lean cross-platform design | Per-brick measurements, portability 13.1, manual qualification 13.4; desktop backend qualification 14.4 |

Deferred: platform CI (8.4 skipped), signing/public updater, cloud sync,
remote/persistent MCP, a mandatory vector-memory service, bundled model hosting,
automatic worktree isolation, plugin marketplace, unattended native core
replacement and unattended swarms. Manual scheduled-task creation UI is deferred
indefinitely by the user; chat remains the creation surface. These are not silently
included elsewhere. Physical IME/screen-reader, low-end and other-OS gaps remain
visible alongside scheduled work.

Developer-workspace exclusions: full VS Code extension compatibility, debugger,
remote SSH/containers development, notebook editor and advanced destructive Git
history rewriting. They are not required to deliver milestones 16–19.
