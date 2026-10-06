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

20.3's broader extension ABI is not adopted. Native installation/restore in
20.5 and actual repair/rollback qualification in 20.6 remain unimplemented.

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
intact. Updated 20.4 foreground acceptance remains open: fresh-window activation
retried once still returned `failed to activate captured window`, and read-only
capture displayed the Windows background. A user-assisted foreground check and
final build including the corrected fixture remain required.

Native code/build scripts use the
account's OS permissions: supervision, source checks and offline Cargo are not OS
containment against malicious code. Arbitrary generated native execution remains
separately reviewed; frozen-input/receipt checks do not establish such containment.
