# Harness self-repair

2026-10-06. Bricks 20.1–20.2 are implemented with [qualification gaps](../qualification/harness-self-repair.md).
The user prioritized the reviewed native pipeline; 20.3's broader Wasm ABI is
not adopted. 20.4 implements the bounded reviewed Rust evaluator; packaged and
bounded DeepSeek non-improvement checks pass, native visual acceptance remains
open. Native install/restore and real repair in 20.5–20.6 remain
planned. The milestone follows the immediate
streaming/progress fixes and takes priority over the paused milestones 16–19.
Platform CI 8.4 stays deferred.

## Existing foundation and missing behavior

Dolores has source inspection, recent failure evidence, versioned skills and
ABI 1 recovery-hint mods with trials, activation and restore. Those mods change
a fixed guidance classification/card. They cannot repair the HTTP decoder,
agent loop, arbitrary Flutter code or native binary. No conversation-level
prepare → reproduce → patch → test → install workflow exists for those faults.
Milestone 13 delivered its narrow envelope, not general harness self-repair.

120 lines is a per-read page, not total access. Nevertheless, serial reads spend
tool operations, lack symbol/text search and expose only selected files. The
agent should not need to switch the user's project to the Dolores checkout or
look for its implementation in an unrelated working folder.

## User flow

“Diagnose this” or “fix yourself” retrieves matching source and the failed run's
actual evidence, reproduces the problem, prepares a patch and reports tests.
Keep the original task, draft and completed effects; repairing the harness does
not replay that task. One concise repair card shows Diagnosing, Preparing fix,
Testing, Ready or Applied. Expand for evidence, diff, independent tests and Restore.
Use existing icons, theme and uniform borders, without a colored left stripe.

Qualifying supported extensions may activate under the user's separately enabled
low-risk policy after fixed tests, with rollback. Native/core patches produce
a candidate build for reviewed installation/restart. Test success does not make
a native patch a low-risk extension. Never overwrite a loaded DLL in place.

## Coordinator and source service

A repair coordinator owns source snapshots, reproduction, candidate, test
criteria, implementation version and durable receipts. Typed tools address repair
IDs and relative source references. Original project tools keep their folder scope.

Ship a version-matched compressed source bundle and manifest for supported repair
modules, configuration schemas and tests. Read it lazily; packaged diagnosis must
not require Git or a separately configured development checkout. An optional
checkout must match or be explicitly labeled as different. No implicit repository
upload, private transcript, local root or credential sharing.

Replace fixed 120-line navigation with source listing, bounded symbol/text search,
context-aware ranged reads, content identity and explicit continuation. Initial
per-call ceilings: 32 KiB returned text and 2048 lines; actual model context can
reduce the response. These bound delivery, not total access to a module. Repair
work has its own explicit task budget. Missing/stale source retains diagnosis.

Host telemetry records stage, frozen settings, elapsed time, wire bytes/events,
decoded lengths, last activity and incomplete-call status. Do not persist raw
reasoning or arbitrary remote error bodies. Missing older telemetry stays unknown.

## Protected boundary and broader extension seams

### Native-first decision

The user chose native-first for architectural simplicity. It reuses the existing
Rust/Flutter source, build and test tools, and can fix behavior outside a narrow
hook. Its cost is reviewed execution, build/restart and restore: generated native
code runs with its process account's authority unless actual containment is
qualified. Passing tests does not grant installation or automatic execution.
The proposed import-free Wasm normalization hook would have a narrower runtime
capability surface but requires a new ABI and can repair only host-exposed seams.
It is deferred rather than implemented or qualified.

20.2 currently materializes selected matching files and a visible exact patch,
with immutable baseline snapshots and bounded revisions. It has no compile/test/
install action. A complete matching build workspace, independent frozen criteria
and separately reviewed native execution must precede any native repair claim.


Keep credentials, grants, final tool validation/dispatch, hard ceilings, evaluator
identity, evidence integrity and activation/rollback decisions host-owned. A
candidate cannot edit the tests that qualify it. Task Full access is not update
authority.

ABI 1 cannot express provider string/data normalization. Qualify a broader ABI
before claiming recovery-label changes repair a parser. First candidate seams:
recovery policy, public activity classification and bounded provider-format
normalization. Define memory/fuel, typed input/output, cancellation, portability
and compatibility before choosing ABI 2. Generated normalized calls still pass
independent host validation/approval. Automatic generated execution receives no
credential, network or filesystem imports.

Native patches use matching source and separate artifacts. Dependency, manifest,
build-script, protected-policy and evaluator changes escalate to review. Plain
test processes are not sandboxes: qualify actual containment before automatically
executing generated native tests. Without it, retain the patch and require reviewed
execution rather than silently expanding authority.

## Tests, activation and restart

20.4's first native evaluator supports an explicitly reviewed Rust integration
reproduction against core/provider public APIs. Freeze the exact reproduction
for both versions, preserve existing qualifying test definitions and run candidate
library regressions. Materialize full matching source in separate owned folders;
pin candidate/source/Cargo identities and persist each completed phase. Three
fixed offline/locked Cargo commands each have 300 seconds/256 KiB capture. Baseline
passing, compilation/evidence failure or test/source drift withholds qualification.
An unfinished/stopped trial never automatically resumes. This does not qualify
OS containment or general Flutter/native evaluation. Every native execution stays
reviewed even under Full access; installation/restore remains a separate gate.

States: diagnosed → reproduced → proposed → testing → qualified → ready → applied,
plus rejected, cancelled, interrupted and restored. Retain baseline/candidate
digests, fixed criteria, trial budgets/environment and receipts. Compilation is
not behavioral improvement: require baseline failure, candidate success and
independent regression passes without weakening criteria.

In-flight work pins the current implementation. Activation waits for a safe
boundary and checks the version tested. Stop preserves evidence and patch text;
exhausted generation/trials pause with explicit continuation, not unlimited retries.

Native installation uses an owned external launcher and versioned bundle: record
intent, retain the previous bundle, stop the old normal app, start the candidate,
check startup/data compatibility, then record health. Failed startup restores the
previous bundle. Interrupted installation reconciles without replay. Preserve
provider/history; prohibit automatic destructive migrations and publication.

## Planned bricks and acceptance

| Brick | Deliverable / basic acceptance | Realistic failure and recovery |
| --- | --- | --- |
| 20.1 | Exact failure diagnosis and source navigation; find/read a large module beyond 120 lines without changing project folders. | Old build/missing telemetry and stale source remain explicit; context exhaustion retains evidence and continuation. |
| 20.2 | Managed repair workspace and visible matching-source diff from ordinary chat. | Changed source or another project cannot receive a patch; Stop/missing source retains the task. |
| 20.3 | Qualified broader ABI and repair of one actual supported behavior beyond a guidance label. | Runaway/invalid output and extra capabilities cannot bypass dispatch; previous extension remains usable. |
| 20.4 | Independent reproduction and candidate trials: frozen baseline fails, candidate and regression checks pass. | Candidate-altered tests, exhausted budgets and absent containment cannot count as success. |
| 20.5 | Qualifying extension activation; reviewed native build/install/restart and restore. | Failed startup, stale revision and interrupted installation preserve history and reconcile without replay. |
| 20.6 | Bounded DeepSeek diagnose → reproduce → patch → test → activate case, plus separate reviewed native restart/rollback qualification. | Withhold a non-improvement, restore after a deliberate regression and retain work after exhaustion. |

Each brick gets a commit and basic/edge qualification; runtime changes require
normal native verification. Qwen handles routine probes, DeepSeek harder repairs.
Before adopting ABI 2/build services, measure incremental idle/active memory,
startup, cancellation and cleanup. Keep services lazy. Other-platform/resource
acceptance remains explicit rather than inferred from a Windows development host.

Exit requires useful observed repair. Automatic native core replacement, arbitrary
capability invention, signing, unattended publication and general repair competence
are excluded. The user decides when milestone execution begins.
