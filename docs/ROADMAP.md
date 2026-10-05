# Dolores roadmap

**Planning baseline — 2026-10-04.** Writing this plan is authorized; future runtime bricks are not started by this document. Brick **8.4, platform CI, remains skipped**. Completed work is retained in [implementation history](IMPLEMENTATION_HISTORY.md); measured results and open gaps belong in [acceptance](ACCEPTANCE.md).

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
milestone. **14.1–14.3 are implemented; 14.4 qualification is underway.** Existing milestone numbers
and order are preserved. Windows selected-application interaction and visual
verification are the first target; arbitrary-app reliability and other platforms
need separate evidence. The subsequent “next” authorized the first observation brick.

Dolores should complete useful project work, discuss reasoning, challenge consequential false premises, remember scoped answers and improve skills/approved extensions from evidence. Keep Flutter/system theme and Rust/provider-independent ports. Support threads/compaction, explicit approval/full-access choices, attachments, settings, subagents, search, browser use and scoped desktop computer use. Automatic tested activation with rollback is limited by host-owned authority and actual containment, not an agent's self-assessment.

The [architecture specification](design/evolving-harness-architecture.md) defines component/state/lifecycle/authority contracts. The [behavior policy](design/dolores-behavior.md) defines reasoning, questioning, character and adaptation. These describe the target; [current architecture](ARCHITECTURE.md) describes the running app. No claim of consciousness, weight training or guaranteed daily improvement is made.

This replaces the prior working-agent/context/learning/plugin proposal. Extension contracts move first; permission modes, advanced tools and executable adaptation are scheduled explicitly. Existing recovery, memory, skills and MCP are extended rather than rebuilt. Every requested feature maps to a named brick.

## Starting point

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

Default order is sequential. Independent documents/measurements may proceed within a milestone; runtime dependencies remain binding. Questioning/personality starts in 9.4 and is exercised throughout. Resource checks run throughout. Dates are not promised before baselines establish work size. Each brick has scope, basic acceptance, realistic failures and exclusions below.

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

**Status:** 14.1 observation and 14.2 scoped input are delivered; 14.3–14.4 are planned.
Deliver a useful Windows selected-application workflow first,
with a provider-independent observation/action contract and on-demand native
helper. Preserve the existing browser adapter and Flutter UI. Choose the native
backend and provider transport in a design specification before implementation;
do not assume access to Codex's private desktop runtime.

The tool port now supports optional typed image references, with literal text
and a following untrusted image projection in the OpenAI-compatible adapter.
Browser screenshots remain local evidence. Desktop observation is an explicit
saved snapshot analysis, not a continuous observation/action loop. See the
[selected-window contract](design/computer-use.md) and [acceptance](ACCEPTANCE.md).
Existing
permission modes need desktop-specific resource scopes and dispatch checks;
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
| Self-inspection/editable versus protected modules | 9.1, 9.3, 13.1–13.3 |
| Experience-driven skills | 12.1–12.4 |
| Questions/scoped answers/humane character | 9.4 and every milestone; continuity 12.1 |
| Lean cross-platform design | Per-brick measurements, portability 13.1, manual qualification 13.4; desktop backend qualification 14.4 |

Deferred: platform CI (8.4 skipped), signing/public updater, cloud sync, remote/persistent MCP, vector memory, bundled model hosting, automatic worktree isolation, plugin marketplace, general native self-replacement and unattended swarms. These are not silently included elsewhere. Physical IME/screen-reader, low-end and other-OS gaps remain visible alongside scheduled work.
