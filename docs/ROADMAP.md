# Dolores roadmap

**Fresh-session continuation — 2026-10-06.** The
[current handoff](HANDOFF.md) gathers earlier handoffs, design references and the
latest acceptance evidence. Response/approval recovery and Automatic task loops
are implemented; exact DeepSeek task-validation qualification remains open. Start
there, then implement milestone 20. The user requested continuation in a fresh
chat; 16–19 retain their paused status and 8.4 remains deferred.

**Priority change — 2026-10-06.** Milestones 16–19 are paused at the user's
request. Fix fragmented reasoning-stream failures and silent progress first.
Then **20 — Harness self-repair** is proposed in [its specification](design/harness-self-repair.md):
source navigation, managed patch workspaces, broader qualified extension seams,
independent trials and activation/build/restart/restore. It is planned work,
not a claim that milestone 13's recovery-hint mods repair the native harness.

**Developer workspace planning addition — 2026-10-06.** The planned direction is
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
| **16 — Workspace and editor** | Project-bound views, safe file editing, feature rail and four-pane layout | 16.0–16.5 | Prototype/editor gate first; replace single-active-run assumptions before concurrent chats. |
| **17 — Git workflow** | Real diffs/history, staging, commit, stash/revert and remote actions | 17.1–17.5 | Requires 16's project/document revisions; serialize repository mutations and preserve conflicted work. |
| **18 — Terminal and language support** | Interactive project terminals and TypeScript/JavaScript/Rust language services | 18.1–18.5 | Requires 16's ownership; lazy supervised processes, useful missing-tool setup and bounded cleanup. |
| **19 — Detached windows** | Move live views between windows/monitors without duplicating resources | 19.1–19.4 | Requires shared owners from 16–18; qualify pinned-SDK backend and per-window overhead before adoption. |
| **20 — Harness self-repair** | Diagnose/reproduce a real fault, test a matching patch and activate or build/restart with restore | 20.1–20.6 | Prioritized before 16–19 after immediate AI fixes; broaden 13's narrow ABI only after containment and independent-test gates. |

Milestone 20 is prioritized before the paused milestones 16–19; other dependencies remain sequential. Independent documents/measurements may proceed within a milestone; runtime dependencies remain binding. Milestones 16–19 form the newly requested developer-workspace track using delivered contracts; their manual editor/Git work does not require falsely closing prior model-reliability gates. Questioning/personality starts in 9.4 and is exercised throughout. Resource checks run throughout. Dates are not promised before baselines establish work size. Each brick has scope, basic acceptance, realistic failures and exclusions below.

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

## Milestone 16 — Workspace and editor

Deliver the first useful integrated workspace on the maintained Flutter/Rust
stack. Follow the [workspace specification](design/developer-workspace.md),
retaining theme/brand/short controls. A view is always bound to its project/resource;
switching focus changes navigation context, never a live task's authority.

### 16.0 Workspace prototype and editor feasibility

**Scope:** prototype Chats/Files/Git rail, project panel, chat/file tabs and the
requested A/B four-pane layout. Exercise tab move/split, compact focus and dirty
file/conflict flows with synthetic data. Compare a Flutter-native editor candidate
against actual input requirements; identify the detached-window backend seam
without adopting experimental windowing. Freeze document/buffer ownership,
message bounds, corpus and a numeric incremental memory budget from matched
baseline/editor measurements before selecting/pinning dependencies.

**Basic acceptance:** user reviews the workspace interaction; a native editor spike
demonstrates typing, selection, undo, syntax/find, CRLF/BOM, Unicode and proposed
large-file behavior. Record measured cost and the dependency decision, not just
a package README's performance claim.

**Failure/recovery:** a 100 KiB single line and unsupported encoding stay usable
without lossy saves; prototype cancellation/invalid drop retains tabs and drafts.
Real IME/accessibility claims need actual checks; missing checks remain gates.

**Excluded:** production UI rollout, an SDK channel switch, complete IDE parity
or promising a cheap second native window without measurements.

### 16.1 App, project and concurrent-run ownership

**Scope:** separate shared application services from conversation controllers;
replace the Rust single-active-run state with a bounded run registry and per-run
approval/cancellation routing. Project/document IDs and revisions become explicit
in the view contract. Proposed starting scheduler: two global active project runs,
one mutating agent run per project, visible queue/cancel. Freeze new scheduler
limits after measurement; retain all existing per-run/model/continuation budgets.
View closing must not shut down the app bridge or duplicate profile recovery.

**Basic acceptance:** A and B run independently while either is viewed; Stop A
does not stop B and approvals/model snapshots stay with their original run.
Closing/reopening a chat tab preserves drafts, progress and history. Migrate the
existing single-chat flow without changing provider/configuration/history.

**Failure/recovery:** stale run/approval IDs cannot act on another project; a
queued third run can be cancelled without resetting limits or silently starting.
Same-project competing writers remain queued with an understandable explanation.

**Excluded:** unattended swarms, automatic worktrees, new access grants and
parallel writes in the same working root.

### 16.2 Feature rail, files and workspace tabs

**Scope:** add Chats / Files / Git rail plus the single bottom Settings entry;
retain project/session icons and hints. Files uses lazy scoped traversal and quick
open. Introduce typed chat/file/diff/history/terminal ViewRefs and per-pane tab
strips; Git/terminal surfaces show availability until their later bricks exist.
Implement preview/pin/close/reorder and project selection with explicit identities.

**Basic acceptance:** switch feature panels, open an A file and B chat as tabs,
disambiguate equal names and return with view state intact. Side chats offer
Choose project; temporary chats use their existing folder. Settings remains easy
to reach in wide/compact layouts.

**Failure/recovery:** inaccessible folder/symlink escape provides a scoped recovery
without scanning outside the root; missing project/file does not destroy tabs.
Large/generated trees load incrementally without flooding the UI.

**Excluded:** recursive eager indexing, automatic Git initialization, enabling
tools for a side chat and a populated Git feature before 17.

### 16.3 Safe code editing and disk reconciliation

**Scope:** selected editor adapter with line numbers/highlighting, find/replace,
go to line, indentation, clipboard, undo/redo and explicit Save. One document owner
per canonical project/path, revisioned writes, existing encoding/line endings and
private dirty-buffer recovery. Reconcile outside/agent changes; link selection/file
attachment to an explicitly chosen chat without automatic prompt sharing.
Add deliberate create/rename/delete with buffer-preservation checks.

**Basic acceptance:** edit/save/reopen ordinary source, preserve bytes/line endings,
undo across view changes, and restore a dirty draft after an interrupted app.
Agent saved-file changes refresh a clean view; unsaved data remains distinct.

**Failure/recovery:** agent/external disk update while dirty retains both versions
and opens comparison; write failure or deletion retains the buffer and useful
Save as/Retry recovery. A stale revision cannot silently overwrite newer content.

**Excluded:** autosave by default, lossy binary/encoding edits, hidden whole-file
uploads and accepting the editor package's text widget as sufficient persistence.

### 16.4 Split panes and persistent layouts

**Scope:** binary split-tree layout, edge drop/move/reorder, resize/reset and
keyboard/menu equivalents. Persist tab references and layout separately from
documents/runs. Duplicate file views share one buffer/undo; duplicate chat views
share one draft/run. Narrow mode selects one pane without deleting the wide layout.

**Basic acceptance:** A chat top left/A file top right/B chat bottom left/B file
bottom right; edit and monitor both projects, move tabs and restart with layout
and drafts intact. Focus-specific shortcuts act only on the intended view.

**Failure/recovery:** cancelled drag or closing the last split retains live work;
malformed/stale layout restores a usable default while recovering dirty documents.
Shrinking below pane minimums cannot hide Close/Stop/Settings or lose content.

**Excluded:** detached native windows (19), copying live owners into layout JSON
and tab-close as an implicit task Stop or conversation deletion.

### 16.5 Workspace/editor qualification

**Scope:** frozen two-project corpus, one/two/four panes, editor latency/resources,
normal native light/dark/wide/compact and keyboard checks. Regress original chat,
attachments, settings, approvals, continuation and history. One bounded Qwen
two-project saved-file case verifies model integration; fixtures test concurrency
and failures deterministically.

**Basic acceptance:** user can monitor A/B and edit code without another editor,
with original profile/settings preserved, resource targets measured and explicit
unsupported-file/platform boundaries. Review the normal desktop visually.

**Failure/recovery:** stale/failed save and queued/interrupted run retain work and
correct project ownership. Report physical IME/low-end/other-platform gaps honestly.

**Exit/excluded:** single-window workspace is usable; Git, terminal/LSP and detached
windows are still scheduled work, not advertised as already available.

## Milestone 17 — Git workflow

Use real Git working-tree/index/history state through a typed project service.
Keep run change receipts reachable as separate evidence; do not confuse them with
the current repository diff. Direct user operations do not alter model authority.

### 17.1 Repository status, diffs and history

**Scope:** installed Git discovery, nested repository identity, Changes/Staged/
History panel, paged commit/file history, working/index/HEAD/commit comparisons,
inline and side-by-side diff tabs. Parse machine-readable NUL-delimited paths;
refresh from coalesced events and explicit Refresh.

**Basic acceptance:** tracked/untracked/renamed/deleted/binary files appear correctly;
open the appropriate diff/history and link from a chat's Changes action.

**Failure/recovery:** absent Git/non-repository folder has a short actual setup
action; unusual/Unicode paths or a refresh during external writes cannot show
another repository's result. Failed reads retain a labeled stale view and Retry.

**Excluded:** mutations, a full graphical history explorer and parsing localized
human-readable Git output.

### 17.2 Stage, unstage and commit

**Scope:** whole-file then selected-hunk stage/unstage, commit message and staged
preview. Serialize app-owned repository mutations and bind commands to displayed
HEAD/index/file basis. Preserve Git hooks and configured author identity.

**Basic acceptance:** stage selected changes, inspect staged diff, commit only
those changes and see the resulting commit; unstaged/dirty editor work is retained.

**Failure/recovery:** changed index/patch basis requires refresh, not blind apply;
hook failure retains message/index and gives bounded output with Retry. Another
project's commit cannot be targeted by focus switching during the operation.

**Excluded:** automatic commits, bypassing hooks, changing Git identity and staging
unsaved editor buffers without explicitly saving them first.

### 17.3 Stash, branches and reversing changes

**Scope:** stash preview/create/apply/pop, branch list/create/switch, selected file
discard and non-merge commit revert. Preview exact effect and check dirty buffers/
repository revisions. Conflict view offers per-file inspection and ordinary editor
resolution; preserve underlying Git state and next actions.

**Basic acceptance:** stash and restore selected work, switch clean branches and
create a reverting commit; distinguish Discard file changes from Revert commit.

**Failure/recovery:** conflicted stash pop preserves the stash; branch/revert conflict
or dirty buffers retains edits and offers Resolve/Abort without destructive cleanup.

**Excluded:** hard reset, merge-commit revert automation, automatic conflict choices
and blanket stashes of unrelated/ignored work.

### 17.4 Fetch, pull and push

**Scope:** configured remotes/tracking, ahead/behind, deliberate Fetch/Pull/Push and
credential-helper integration. Show branch/remote/outgoing effect; first Pull is
fast-forward-only. Model push authority remains unchanged.

**Basic acceptance:** disposable local bare-remote fixture receives one intended
commit and fetch/pull updates only the intended repository. No production remote
push or credential change is required for qualification.

**Failure/recovery:** rejected authentication/non-fast-forward offers a concrete
next step; interrupted push is marked uncertain and reconciles remote refs before
retry. Never force/reset/rebase as a hidden recovery.

**Excluded:** force push, stored credential copies, automatic publication and
silently resolving divergence.

### 17.5 Git workflow qualification

**Scope:** frozen two-repository edit/diff/stage/commit/stash/revert/remote corpus,
native readability and operation recovery, with direct UI and agent/editor outside
changes. Measure large-history paging and owned Git/hook cleanup.

**Basic acceptance:** complete ordinary local Git work in Dolores with correct
repository effects and original project history preserved. Record exact fixture
coverage, hook/credential limits and normal native verification.

**Failure/recovery:** stale hunk/index and conflicted stash preserve usable work;
remote rejection/uncertainty never causes destructive retries.

**Exit/excluded:** everyday Git is useful; hosted review integrations, full graph
exploration and advanced history rewriting remain outside the series.

## Milestone 18 — Terminal and language support

Provide the remaining everyday editor workflows without making shells/language
servers always-on. First LSP families are TypeScript/JavaScript and Rust.

### 18.1 Interactive project terminals

**Scope:** qualify terminal emulator/Rust PTY adapter, use Windows ConPTY and portable
backend seams, shell discovery, explicit project/cwd, ANSI, resize, scrollback,
selection/copy/paste and interrupt. Terminal is a movable workspace view.

**Basic acceptance:** run an interactive synthetic project command, resize during
output, copy text and interrupt correctly; A/B terminals stay in their own cwd.

**Failure/recovery:** missing shell/failed spawn gives Retry/Choose shell; rapid
output and split UTF-8/escape sequences are bounded without blocking Stop or UI.

**Excluded:** plain captured stdout presented as a terminal, automatic model control
of the user's shell, SSH/container development and always-running terminals.

### 18.2 Terminal lifecycle and local context sharing

**Scope:** supervisor/backpressure, owned process-tree cleanup, busy-close/final-quit
UX, tab move/restore and explicit selected-output attachment to a chosen chat.
Terminal restart state describes terminated processes accurately.

**Basic acceptance:** moving/hiding a terminal preserves its process; explicit
Stop/close reaps descendants; user-selected output can be attached with provenance.

**Failure/recovery:** output flood/child exit keeps the editor responsive; interrupted
shutdown or resistant child yields truthful status and retained bounded output.

**Excluded:** implicit scrollback uploads, shared model/human shell sessions and
claiming cold restart preserves a process that actually stopped.

### 18.3 Read-side LSP for the first languages

**Scope:** standard protocol client and lazy supervised servers for TypeScript/
JavaScript and Rust. Discover installed toolchains; diagnostics, completion, hover,
definition/references use buffer versions and negotiated position encoding. Servers
are project-scoped, requests cancellable and events bounded.

**Basic acceptance:** meaningful diagnostics/completion/navigation work on small
real A/B projects, including unsaved text. Ordinary chat starts no language server.

**Failure/recovery:** crashed/missing server retains editing with Restart/Install;
stale/out-of-order replies and Unicode positions cannot apply to the wrong buffer.

**Excluded:** syntax highlighting marketed as LSP, arbitrary-language support and
model credentials passed into server environments.

### 18.4 Managed language setup and reviewed language edits

**Scope:** concise first-language readiness/install flow, pinned verified artifacts
and toolchain prerequisites with explicit user download action. Add formatting and
rename previews; workspace edits use document/repository revisions and scoped paths.
Keep installation details expandable and basic offline editing functional.

**Basic acceptance:** a missing first-language server can be installed through a
guided UI without hand-written command/JSON; preview/apply a format or multi-file
rename and undo/recover appropriately. Existing installations are reused.

**Failure/recovery:** offline/cancelled installation retains the prior usable setup;
out-of-root or stale multi-file edits are rejected before partial data loss.

**Excluded:** downloads merely from opening a file, unverified latest artifacts,
arbitrary server-driven command execution and unsafe auto-applied refactors.

### 18.5 Terminal/LSP qualification

**Scope:** interactive process corpus, two projects/two first language families,
typing/diagnostic latency, idle memory/process counts, cleanup and native shortcut/
focus behavior. Validate both ready and missing/offline tool paths.

**Basic acceptance:** routine edit/complete/navigate/run/check workflow fits in
Dolores with lazy startup and measured overhead. A bounded harder DeepSeek repair
case uses saved files and approved command tools; direct PTY success is separately
verified and does not establish model reliability.

**Failure/recovery:** LSP crash/stale reply and PTY flood/interrupt preserve editor
work and an actionable recovery. Missing other-OS/hardware checks remain open.

**Exit/excluded:** local terminal and first-language productivity are useful;
debugger, full VS Code extension support and remote development are not claimed.

## Milestone 19 — Detached windows

Complete the user's drag-out/multiple-monitor request without copying the host,
database, credentials or live resources into independently initialized apps.

### 19.1 Window backend and resource gate

**Scope:** verify maintained SDK support; measure an isolated two-window candidate
with native focus/input/close/theme behavior. Compare official experimental API
availability against a pinned compatible community/native backend. Record engines,
processes, incremental memory/CPU and a numeric resource ceiling before adoption.

**Basic acceptance:** usable normal-build-compatible backend and single-host
communication plan are evidenced on Windows without silently changing SDK channel.

**Failure/recovery:** unavailable backend/second-window creation returns to the
existing workspace; engine/plugin initialization failure cannot corrupt the profile.

**Excluded:** assuming another window is free, one full Rust/storage host per view,
experimental channel migration without discussion and other-OS claims from Windows.

### 19.2 Move views across native windows

**Scope:** drag out/Move to new window, drag/menu move back, window tabs and focus.
Transfer protocol acknowledges the receiving view before disposing its source;
document/run/terminal owners and IDs remain in the host. Keep themed chrome/one
brand and Settings available; sibling windows observe saved appearance changes.

**Basic acceptance:** move a dirty file, active A chat and terminal onto a second
monitor, then back, without duplicate buffers, approvals, sends or processes.

**Failure/recovery:** cancelled drop/destination-close/failed acknowledgement keeps
the source view and usable work; closing one window cannot shut down another's run.

**Excluded:** independent profile opens/recovery, credential copies and moving a
view by serializing/replaying its entire task.

### 19.3 Window/layout restart and monitor recovery

**Scope:** persist versioned window/pane/tab state and safe bounds; resolve host
resources after restart, restore private dirty documents, clamp missing-monitor
windows onscreen and handle final-window close with pending work.

**Basic acceptance:** relaunch restores the two-project workspace and recoverable
drafts, with task/terminal interruption states truthful. Monitor removal leaves
all windows/actions accessible.

**Failure/recovery:** corrupt layout or unavailable project restores a usable shell
and recoverable documents; a lost window/client invalidates stale subscriptions
without replaying work or starting duplicate recovery.

**Excluded:** resurrecting stopped processes, offscreen-only recovery and keeping
a hidden permanent service alive after the user explicitly quits.

### 19.4 Integrated developer-workspace qualification

**Scope:** fixed A/B chat/edit/diff/commit/check corpus in four panes and two native
windows; light/dark, focus/keyboard, detach/rejoin, monitor loss and bounded resource
measurement. Inspect the normal desktop visually and preserve the original profile.

**Basic acceptance:** view/edit source, perform ordinary Git work and monitor two
projects across panes/windows without a separate editor for this qualified corpus.
Measure added cost, cleanup and actual model outcomes independently.

**Failure/recovery:** stale disk save plus interrupted window transfer retains all
work; LSP/terminal/helper failure stays within its service and preserves chat/editor.

**Exit/excluded:** requested developer workspace works within the demonstrated
platform/language/resource envelope. Prior model reliability and broad platform/
accessibility gaps remain explicit; no claim of complete Cursor/VS Code parity.

## Milestone 20 — Harness self-repair

**Status: 20.1–20.2 and 20.4 implemented; 20.5 passes packaged trusted-fixture qualification, and 20.6 passes one configured DeepSeek-authored repair candidate and real task. Direct user installation/Close, general model competence and broader platform qualification remain open.** This milestone takes priority before
the paused 16–19 developer-workspace track after immediate AI reliability fixes.
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
   Direct user review/graceful close remains unexercised. See [installation scope](design/native-repair-installation.md).
6. **20.6:** one configured DeepSeek-authored provider repair, reviewed release
   build, real candidate task and trusted protected Restore/rollback pass.
   Earlier reviewed non-improvement remains withheld. Direct user installation/
   graceful Close, general model competence and broader platforms remain open.

Each brick's basic and realistic failure/recovery criteria are in the spec.
Success means a fault is reproduced and repaired without losing work; inspection,
successful compilation or altered recovery wording alone does not satisfy exit.
Automatic native/core replacement and publication remain excluded. Execution
begins only when the user requests it.

## Validation and change control

- Freeze behavior, numerical defaults, criteria and exclusions before each brick; discuss material technical choices. Evidence may correct an assumption, but added scope requires a named spec/roadmap revision before implementation.
- Basic flow plus one or two realistic failure/recovery cases per brick. Record actual fixtures, bounded live results and gaps separately in ACCEPTANCE. Qwen3.5-2B is routine; DeepSeek V4.1 Flash is for harder cases. Preserve selected settings and private data.
- Commit each completed brick in English. App/runtime changes require a normal build and visible launch with config/history preserved. Documentation-only tasks use document/link checks and a commit, without rebuilding/relaunching.
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
| Source viewing/editing and file navigation | 16.0, 16.2, 16.3, 16.5 |
| Multiple project chats and flexible pane/tab layout | 16.1, 16.4; detached views 19.1–19.4 |
| Repository diff/history and everyday Git operations | 17.1–17.5 |
| Interactive terminal and TypeScript/JavaScript/Rust LSP | 18.1–18.5 |
| Self-inspection/editable versus protected modules | 9.1, 9.3, 13.1–13.3; practical repair 20.1–20.6 |
| Experience-driven skills | 12.1–12.4 |
| Questions/scoped answers/humane character | 9.4 and every milestone; continuity 12.1 |
| Lean cross-platform design | Per-brick measurements, portability 13.1, manual qualification 13.4; desktop backend qualification 14.4 |

Deferred: platform CI (8.4 skipped), signing/public updater, cloud sync, remote/persistent MCP, vector memory, bundled model hosting, automatic worktree isolation, plugin marketplace, unattended native core replacement and unattended swarms. These are not silently included elsewhere. Physical IME/screen-reader, low-end and other-OS gaps remain visible alongside scheduled work.

Developer-workspace exclusions: full VS Code extension compatibility, debugger,
remote SSH/containers development, notebook editor and advanced destructive Git
history rewriting. They are not required to deliver milestones 16–19.
