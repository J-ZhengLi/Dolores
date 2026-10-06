# Harness self-repair qualification

## 20.1 — source navigation and failure evidence

2026-10-06. Implemented source inspection, not patching or self-update authority.
The normal Windows bundle includes 348 source/configuration/test files (3,702,356
source bytes, 963,164 compressed bytes in the qualified build). Each file and the
manifest have content identities. Decompression happens on request, bounded to
the declared size; there is no checkout dependency or background source service.
The manifest includes Rust/Dart/Python source, Cargo manifests/lockfile and the
Flutter package manifest, rather than arbitrary local configuration files.

Listing, literal/identifier-token search and numbered reads have explicit cursor
or line continuation. Identifier matching is not an AST/call graph. A request can
choose 1–2048 lines and a 1–32 KiB allowance; actual encoded delivery respects the
existing 16 KiB tool-result ceiling. A smaller requested allowance narrows pages.
The agent's independent context checkpoint retains receipts and progress; this
service does not estimate the remaining context after every tool result. No hard
task or dispatch ceiling was relaxed. Both prepared-query validators now admit
bounded 4 KiB inspection JSON so two 71-byte source identities fit together.

### Exercised

- Rust workspace library suites: 332 passed, one ignored; Clippy all targets with
  warnings denied passed. Flutter's unchanged UI: 248 tests and analysis passed.
- Five source/privacy tests cover list/search continuation, a read beyond 120,
  stale identities, mismatched/unavailable checkout, corrupt compressed source
  and a page too small to fit. Refusals keep evidence and give a fresh-navigation
  or larger-page next step rather than substituting project files.
- Two provider HTTP tests verify wire/event/decoded-field counters, failed partial
  calls and independent fresh slots. Only typed numeric/enum values are exposed;
  arbitrary reasoning/error text does not enter the diagnostic receipt.
- `scripts/test-harness-navigation.py` passed against the normal release DLL in
  an unrelated temporary project: three fixture HTTP requests, one reviewed
  180-line read with both identities, followed by a disconnected incomplete call.
  The latter requested no approval and caused no dispatch. Failed-stream counters
  and matching bundle identity survived in the run journal; a private reasoning
  sentinel was absent from the inspection. The project sentinel was unchanged.
- Configured DeepSeek V4.1 Flash passed a bounded live inventory → identifier
  search → 180-line source read, with three reviewed inspections and three model
  calls in 13.34 seconds. The final reply copied the returned source identity and
  named ModelProvider. Output was capped at 4096 tokens and model inactivity at
  60 seconds; an isolated profile preserved the original selected settings.

### Failures and limits

Two routine Qwen3.5-2B probes remain unqualified. The first completed search but
failed its next argument preparation; its old generic recovery message obscured
the actual cause. The second supplied stale identity values and encountered a
driver-denied recovery listing. Stale values were correctly refused. These do
not establish general Qwen navigation reliability. The direct fixture then
reproduced an additional 256-byte host-registry query rejection, fixed in both
validation layers and covered through actual approval/dispatch.

Old runs without bundle identity or counters explicitly remain unknown. External
interruption may leave no final counters; absence is not zero. Current telemetry
covers completed/failed instrumented tool-model requests, not every adapter or
ordinary text-only response. Cross-platform, representative memory/startup and
low-end resource qualification remain gaps; compressed size is not an idle-memory
measurement. No repair, behavioral improvement, installation or restore is
claimed by this brick.

The normal desktop was built/launched with the original profile; its window was
reported present by the maintained launcher and all 38 original tables were
unchanged. Foreground visual inspection is pending while Windows is locked.

## 20.2 — reviewed native proposals

2026-10-06. Implemented separate, chat-scoped snapshots and exact diff review.
The user chose to prioritize the reviewed native pipeline. No broader Wasm ABI,
native test execution, installation or actual behavioral repair is delivered here.

`harness_repair` prepares matching bundled source, adds captured files, proposes
one unique exact replacement, and lists/inspects retained repairs. Every step
requires its own review even under task Full access. Preparation checks copied
bundle/source identities; proposal and dispatch recheck revisions, source and
artifact integrity. SQLite schema 32 stores immutable baselines and current
candidates; separately retained version directories contain baseline/candidate
files and an identity receipt. This is selected-file materialization, not a full
buildable checkout. It never opens the user's project for repair output.

Limits are four workspaces per chat, eight files and 2 MiB combined snapshots per
workspace, sixteen retained revisions, 4096 bytes per replacement operand, an
8 KiB cumulative file diff and the existing 16 KiB encoded tool-result ceiling.
Inspection pages at most 2048 lines/8 KiB text with continuation. Exhaustion keeps
work and gives a smaller-proposal or explicitly separate-repair next step;
there is no silent cleanup, retry or background service.

### Exercised

- Rust workspace library suites: 336 passed, one ignored; Clippy all targets with
  warnings denied passed. Flutter: 250 tests and analysis passed. Compact
  light/dark cards show separate storage, exact diff and review buttons.
- Storage/host tests cover restart, immutable baseline, stale revisions, changed
  source, cross-chat access, cancelled dispatch and tampered saved files. A missing
  or changed artifact still permits database-backed inspection with an actionable
  notice; further mutation refuses and retained source remains.
- `scripts/test-harness-repair.py` passed against the normal release DLL: seven
  fixture HTTP requests, two reviewed prepare/propose operations, a retained
  visible diff, stale/cross-chat refusal and Stop before a fresh preparation.
  The original temporary project and prior candidate bytes stayed unchanged.
- Configured DeepSeek V4.1 Flash passed inventory → source read → prepare →
  propose → inspect in five reviewed operations/model calls, 21.1 seconds.
  The candidate added a harmless comment, not a behavioral fix. Output was capped
  at 4096 tokens, inactivity at 60 seconds and the driver at 180 seconds. An
  isolated profile preserved selected settings; no candidate code ran.

### Failures and limits

The routine Qwen3.5-2B probe remains unqualified: the test driver attempted to
decode a non-harness tool query as JSON and cancelled after two steps. The driver
was corrected; this attempt establishes neither product failure nor Qwen repair
reliability. Test qualification above does not establish containment, independent
baseline failure/candidate improvement, frozen regression criteria, installation
or rollback. A complete matching build workspace and reviewed native execution
remain later work. Normal-desktop foreground inspection remains pending while
Windows is locked.

The final normal build at `13c1534` passed, and the packaged native fixture passed
again. The maintained normal launcher reports a present visible window with the
original profile. Read-only comparison verified all 38 original tables unchanged;
the only added table is empty `harness_repairs`, with schema version 32. Foreground
visual acceptance remains pending desktop unlock; window presence alone does not
establish it.

## Later bricks

20.3's broader extension ABI is not adopted. The Windows 20.5 implementation and
its current evidence are recorded below. Real-model repair qualification in 20.6
remains open.

## 20.4 — reviewed Rust reproduction and regression trials

2026-10-06. `test_harness_repair` adds a separate execution review for a retained
repair ID/revision, supported public-API test target (`dolores-core` or
`dolores-provider-openai`) and complete Rust reproduction source up to 4096 bytes.
The review shows the candidate diff, exact reproduction and three fixed Cargo
commands. Host-owned storage freezes that same reproduction in baseline/candidate
workspaces. Existing inline/test-file definitions must remain unchanged; test,
manifest, build-script, evaluator and other-language candidate edits refuse this
evaluator. This is a bounded Rust path, not general Flutter/native evaluation.

After explicit review, the host materializes the entire matching source bundle,
including the browser worker asset needed by Rust compilation. It runs baseline
reproduction, candidate reproduction only after complete baseline test failure,
then candidate workspace library regressions only after successful reproduction.
Each command has 300 seconds and 256 KiB combined output; at most three commands
run, offline/locked and without retries. Cargo identity, source/revision and frozen
test bytes are checked. Output limits, incomplete/failed compilation, zero tests,
missing regression evidence and baseline already passing cannot qualify.

SQLite schema 33 retains bounded chat/repair-scoped receipts with source/candidate,
criteria and Cargo identities, the frozen reproduction and separate typed results.
Completed phases are saved before starting the next. Four trials per repair are
allowed. Stop/crash leaves source, logs and completed evidence; an unfinished
receipt requires a fresh review and never resumes automatically. A newer proposal
makes old qualification stale. These receipts grant no installation authority.

### Exercised so far

- Rust library suites: 342 passed, one ignored; Clippy all targets with warnings
  denied passed. Flutter: 252 tests and analysis passed; subsequent focused card
  checks pass, including explicit Run once and withheld-improvement labeling.
- A real installed Cargo fixture runs an unchanged integration reproduction
  against a deliberately faulty baseline and fixed candidate, then library
  regressions. It demonstrates the evaluator mechanics, not a real-model repair.
- Fixed inline assertions/ignore/configuration changes refuse; duplicate test
  names retain their multiplicity. Truncated evidence and changed source cannot
  qualify. Cross-chat access, immutable/finished receipts, revision drift and
  reopening storage preserve evidence without replay.
- A native child fixture writes a completed marker, then Stop interrupts it.
  Its marker and staged source remain. No candidate installation occurs.
- Compiler qualification initially failed because filtered Windows child
  environment omitted MSVC discovery hints. Standard installation, architecture
  and command-interpreter hints now pass; credentials and arbitrary compiler
  overrides remain excluded. Compile failure was never counted as improvement.

The normal release build at `53c4178` passed. The packaged native fixture passed:
10 fixture requests, two proposal reviews and two native reviews. Decline/stale
revision executed nothing; the approved ordinary-budget reproduction passed one
baseline test and withheld improvement, with zero candidate execution/install.
The initial driver used camelCase command-preview keys; correcting them to the
actual snake_case fields fixed that driver failure before any execution approval.

Configured DeepSeek V4.1 Flash passed the same live non-improvement path in two
model calls, 23.53 seconds: one separately reviewed trial, one passing baseline
test, zero candidate tests and no installation. Output was bounded to 4096 tokens,
inactivity to 60 seconds and the driver to 180 seconds. The routine Qwen3.5-2B
probe remains unqualified: four calls repeatedly chose source inspection rather
than the requested trial; those operations were denied by the fixed test review,
and no native tests ran. Neither probe demonstrates an actual behavioral repair.
All live/fixture projects remained unchanged; isolated profiles retained results.

The maintained normal launcher reported a visible window with the original
profile. Read-only comparison verified all 38 original tables unchanged, with
only empty `harness_repairs`/`repair_evaluations` additions and schema version 33.
The earlier normal 20.2 app was inspected after unlock with its paused chat/model
intact. Initial 20.4 foreground inspection failed: fresh-window activation
retried once still returned `failed to activate captured window`, and read-only
capture displayed the Windows background. The final normal build at `576187b` includes the corrected
fixture; its packaged rerun passed with the same 10 requests and no installation.
The original-profile preservation check passed again after normal launch.

Follow-up on 2026-10-06: the updated normal window was activated and inspected
successfully at 1127×813, showing the retained paused Mario chat, review access and
selected DeepSeek V4.1 Flash. No Continue, model change or approval was submitted.
This closes the ordinary normal-app foreground gate; the native trial-card
light/dark/compact matrix, actual behavioral repair, resources and other platforms
remain separate acceptance gaps.

Native code/build scripts use the
account's OS permissions: supervision, source checks and offline Cargo are not OS
containment against malicious code. Arbitrary generated native execution remains
separately reviewed; frozen-input/receipt checks do not establish such containment.

## 20.5 — reviewed Windows Rust build and retained normal startup

2026-10-06. Implemented a separately reviewed offline/locked Rust release build
for qualified provider and command-outcome/task-budget repairs. Only the bridge
DLL may change. Policy, credentials, SQLite schema, tests, build scripts,
evaluator, native launcher and Flutter shell remain protected. Full access
cannot authorize build or installation. Installation and Restore use fresh
one-use five-minute UI reviews; neither is exposed as a model tool.

The lazy protected launcher validates every file in both complete retained
bundles, owns one intent, waits at most 30 seconds for the old app to leave,
takes a consistent SQLite backup and blocks candidate startup writes until
source, schema, process identity and logical history agree. The normal Flutter
shell must initialize and acknowledge health within 30 seconds. A startup
failure stops only the verified candidate, recovers pre-startup history and
starts the previous normal bundle. A later explicit Restore keeps current
history, including work acknowledged after installation. No task is replayed.
Unknown ownership or changed state retains evidence and requires fresh review.

### Actual packaged evidence

The final private disposable run is `0899abfd-55d8-4803-8d03-28dba7ac41b7`;
evaluation `07dfa6bd-d4b6-45a0-8e35-273d7f76cbe6`, build
`94c7d014-8cbd-417f-946c-31904731e880`. Evidence stays under ignored output.
The fixed public provider repair unwraps one JSON-string layer only if it contains
an object; host validation and approval remain mandatory. It is trusted fixture
code, not a model-authored patch or a change to the maintained provider source.

- Complete baseline: two cases pass and the double-encoded argument case fails.
  Candidate: all three frozen cases pass, including malformed/non-object/recursive
  arguments and exhausted-output refusal. Unchanged workspace regressions:
  354 pass. The three bounded phases take 320.25 seconds total; each retains its
  existing 300-second and 256-KiB combined-output limits without retries.
- Stale and declined builds execute nothing. The approved cold release build
  completes in 182.89 seconds and retains a ready bundle. A Python bridge host
  refuses installation because it is not the genuine normal app executable.
- The protected helper starts the actual candidate normal shell successfully.
  A supported acknowledged draft is then written through that candidate bridge.
  Restore starts the retained previous normal shell and preserves this newer
  work. All table contents compare equal across each handoff.
- Interrupting the owned waiting helper leaves the old app and history intact;
  reopening the same intent refuses replay. A fresh trusted test intent with a
  mismatched candidate source cannot become healthy and recovers the previous
  normal version after the 30-second startup bound. History remains identical.
  A completed intent also refuses replay. The helper exits after its handoff.
- These fixed test intents are created outside the product UI. Only precisely
  recorded idle fixture processes are stopped. They do not establish direct
  user approval or the product's graceful Close path. Those are manual gates.

### Failures found and recovery verified

The first Windows baseline link failed because an owned compiler directory made
MSVC's build-script path exceed its effective path limit. Fresh compact compiler
roots now carry the full owner identity and refuse collisions; limits remain
unchanged. The next regression run found that the matching source omitted the
existing `scripts/mock-mcp.mjs` helper; it is now included unchanged. Both trials
withheld qualification rather than treating compilation or incomplete regressions
as improvement. A driver field-name error subsequently stopped before build.

The first completed candidate compile then failed DLL staging because Windows
cannot flush a read-only file handle. Its failed receipt offered inspection and
fresh review without installation. Staging now writes and flushes through the
same writable handle, with a Windows regression check. During the first handoff
test, a transient Windows receipt-read sharing conflict stopped driver polling;
the helper still completed rollback. Polling now tolerates only that transient
error within its original deadline. A complete rerun passes all helper cases.

### UI, model, resource and remaining acceptance boundaries

354 Rust library tests pass, one ignored; Clippy all targets with warnings denied
passes. All 256 Flutter tests and analysis pass. Compact light/dark widgets cover
stale review recovery, unsaved Settings preventing restart, ready-build wording,
failed-build inspection and truthful condensed qualified trial evidence.
Native dark checks inspect the empty page, retained failed/ready build cards and
the recovered normal shell with its post-install draft. This is a sample, not the
full theme/window/input/accessibility matrix. The original profile's 38 tables,
selected model and paused work are preserved; only the previously accepted empty
schema-33 repair tables exist there.

Routine configured Qwen3.5-2B performs two bounded calls in 14.58 seconds and
completes inspection, but does not identify the build tool correctly. It runs no
native build or installation, and its unrelated project remains unchanged.
20.6 real-model repair and broader reliability remain open; prior DeepSeek's
20.4 non-improvement check does not close that gap.

Each retained normal bundle has 139 files and is 62.33 MiB; the protected launcher
is about 2.14 MiB. This cold fixture profile retains 5360.19 MiB, dominated by
compiler outputs. This is a desktop observation, not a low-end resource target
pass. There is no idle helper service. Four build receipts per repair, per-file
128-MiB and per-bundle 512-MiB/2048-file bounds are explicit; retained compiler
outputs are not silently pruned. Native code still runs with account permissions,
not qualified OS containment.

The repaired normal app runs from its retained versioned directory. Maintained
launchers/shortcuts are not retargeted, so reopening through the maintained launcher
opens that maintained version. Persistent routing, signing/public updates, direct
user review/graceful Close, real-model authored repair, low-end/other-OS hosts and
the complete native visual matrix remain unaccepted. 20.3 Wasm is not adopted;
milestones 16–19 and CI 8.4 remain deferred.

## 20.6 — bounded DeepSeek-authored candidate and a real task

2026-10-06. The configured `deepseek-v4.1-flash` connection is used from an isolated
profile, with its key held in memory and the original profile read-only. No
original chat is copied into prompts, exports or test reports. Automatic memory
is disabled only in the test profile. Reports retain public source diffs,
identities, numeric outcomes and booleans rather than model transcripts or keys.

The task gives the model a public example of the existing wrapped-JSON argument
fault and the provider source path. It does not give replacement code. DeepSeek
uses nine successful source inspections, prepares matching managed source and
authors its own small replacement at final call assembly. It unwraps exactly one
JSON string layer, then runs the existing call validation and object check.
Malformed/non-object/recursively encoded input still fails, and output exhaustion
still produces no executable partial calls. Two attempted reads outside the
test's file scope are denied; the model recovers without changing that scope.

This is a file-scoped assisted diagnosis, not discovery of an unspecified defect.
The fault is independently reconstructed with fixed public provider responses;
it was not spontaneously observed on the configured live DeepSeek connection.
The reproduction is host-owned, frozen and supplied separately after the model
proposal. The model cannot rewrite tests or alter approval/policy/limits. Every
proposal, native test and build review pauses for inspection; none is blanket
approval, and native code still has account permissions rather than OS containment.

### Results and identity

Private disposable run: `4e7bf33c-795c-4f8b-a3bf-36db299a52a1`; repair
`090ced32-d372-4854-b382-eb2cbb6c57e6`; evaluation
`0e3cdd14-f40c-487c-9cb1-33e3c3a023fc`; release build
`6a2a5203-45ed-41ee-b4ae-2763b501c18d`. All local artifacts stay under ignored output.
The maintained provider implementation is unchanged; the reviewed model patch
exists only in its managed candidate.

- Baseline reproduction: two pass and the actual wrapped-object fault fails.
  Candidate: three pass, including five malformed/recursive/non-object variants
  in one case and exhausted-output refusal. All 354 unchanged Rust workspace
  library regressions pass. No criterion is weakened or invented after results.
- Separate fixed offline/locked release build returns ready. Native command
  bounds remain 300 seconds and 256 KiB each. Model requests never install code.
- The actual normal candidate shell passes protected startup using trusted
  fixed test intents. After stopping only that recorded idle fixture process,
  an actual two-call DeepSeek read task runs through the candidate bridge.
  Its saved implementation identity equals the candidate source identity.
- The task reads `notes/report.json` once with explicit exact-path approval,
  correctly reports its status and true completion field, and returns quoted JSON
  and Chinese content intact. Project digests compare equal before/after. The
  model correctly distinguishes the literal file marker from proof of a repair.
- Subsequent trusted Restore preserves the completed live model turn and a newer
  acknowledged draft. Interrupted waiting-helper refusal leaves the old app and
  history intact. Deliberate candidate startup source mismatch cannot become
  healthy; bounded rollback starts the previous normal bundle and preserves all
  current history. Used/interrupted intents refuse replay. All table contents
  compare equal across each handoff.
- Native dark foreground inspection after rollback shows the live read receipt,
  correct public answer and newer draft. This is a sample, not a complete theme,
  compact window, IME or accessibility matrix. Original-profile comparison again
  confirms all 38 original tables unchanged and only the accepted empty repair
  tables at schema 33.

The first live-task driver result falsely failed because it expected the generic
tool status `completed`; this file tool's successful status is `read`. The original
false report is retained. Independent audit verifies the saved completed turn,
no pause reason, exact candidate implementation, actual answer, file digests and
history preservation before recording success. The driver is corrected for later
runs; no model retry or production-default adjustment is used to conceal failure.

### Bounds, costs and acceptance boundary

There are 14 actual model calls: 8 proposal, 2 evaluation, 2 build, 2 candidate task.
Observed phase wall times are 120.13, 357.97, 215.55 and 5.20 seconds respectively;
these include review waits and native work, not just model latency. Output limits
are 4096 tokens per repair-phase call and 1024 for the read task; inactivity is
60 seconds. Driver caps are 10/4/4/4 model calls per phase, with separate bounded
phase/review waits and no automatic continuation. The production ceilings remain.

Provider-reported usage totals: 229,991 input, 13,513 output, 243,504 total tokens;
16,768 cached input tokens are a subset of input, not an additional total. Source
inspection dominates repeated context. The cold private profile retains
5359.04 MiB including compiler caches and versioned bundles. These are observations
for one development laptop; no cost, low-end or general-reliability target is
claimed to pass. No new idle service or schema is added by this qualification.

One bounded model-authored repair candidate and a real candidate task now pass.
The earlier reviewed non-improvement remains withheld, and trusted helper
restart/Restore/rollback qualification passes for this actual model patch.
Direct human installation approval and graceful product Close remain unexercised:
fixed intents and exact owned idle cleanup do not replace those gates. Nothing is
installed in the original profile and no model-authored code is promoted to the
maintained checkout. General model repair competence, persistent shortcut routing,
qualified OS containment and other-platform/complete visual acceptance remain
open. Milestones 16–19, Wasm 20.3 and CI 8.4 retain their deferred status.

### Follow-up — authorized desktop installation and idle Close

2026-10-06. The human explicitly approves the prepared isolated installation and
restart. The actual Native repairs review identifies the existing ready revision
2, retained current/target bundles and private test profile. Its installation
button dispatches the production review path rather than a constructed test
intent. Intent `023679c1-6816-46ae-96b6-b6caebe493a9` reaches `applied` with candidate
source `sha256:564dbd3e4850fd581896c6d71a8f38dc9deece5049e4decb3801201dadba6303`.
The old process and lazy helper exit normally; the candidate process identity and
foreground normal window are verified. No fixture stop or model request is used.

Every private-profile table compares equal before/after installation. The saved
real DeepSeek answer, read receipt and prior draft are visible in the restarted
candidate. A native `x` keystroke appends to the public draft and is acknowledged
in `session_drafts`; no other table changes. Normal title-bar Close exits the idle
candidate. Reopening the retained installed bundle through `desktop.py`'s launch
function preserves all current table contents and visibly renders the appended
draft and saved answer. The maintained launcher is not retargeted.

Ignored evidence stays in the same disposable run: `native-ui-review-before.json`,
`native-ui-install-audit.json`, `native-ui-close-audit.json`,
`native-ui-after-draft-tables.json` and `native-ui-reopen-audit.json`. The complete
previous/candidate bundles are verified against their saved manifests before
launch. The original app remains the same owned process and every original table
compares equal. No credentials or private original transcripts are exported.

The automation's literal-text entry did not change the composer despite reported
focus; a physical keystroke succeeds and storage independently confirms the edit.
This does not establish a product typing regression or a paste/IME pass. Idle Close
with an acknowledged draft is qualified; busy Close and a last unacknowledged
keystroke are not. Desktop installation/restart now passes with direct human
authorization, but desktop Restore remains separately unexercised and requires
its own review. Earlier trusted rollback, interruption and replay-refusal cases
remain separate evidence. General competence, containment, shortcut routing and
other-platform/full native visual acceptance remain open.
