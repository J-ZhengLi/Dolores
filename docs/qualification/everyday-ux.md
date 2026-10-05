# Everyday UX qualification

Milestone 15 implements the approved A layout using native Flutter controls.
Implementation is delivered; acceptance has the explicit gaps below. The browser
prototype is design evidence only. No privacy, approval or numerical budget
defaults were changed to make a test pass.

## Evidence and reproduction

Use the normal Windows release and a fresh ignored output directory. The preview
contains a synthetic project, one model, one memory and one global skill, without
provider credentials. Its local response fixture prepares history only; it does
not establish model reliability. Never point these scripts at a personal profile.

```text
python scripts/desktop.py build
python scripts/prepare-ux-preview.py --directory <absolute-output-directory>
python scripts/measure-ux-preview.py --directory <same-preview-directory>
```

The measurement leaves the normal app visible and records its owned PID in the
ignored directory. Inspect that window, then close only that owned instance.
The user's normal profile is restored for the final visible launch. The
[control map](../design/ux-control-map.md) records all former destinations.

## Fixed journeys and decisions

These are navigation/structure observations, not timed user-study results.
The baseline is the audit of `c1e4f7e`; no matched before/after participant study
was performed. Counts exclude the contents of lists and consequential reviews.

| Journey | Recorded baseline | Implemented route |
| --- | --- | --- |
| Find a preference | 12 settings peers | Six sections, search with old/new names |
| Set up a model | Separate connection and response editors/selectors | Models → Connect or choose models; select a model for context/images/responses |
| Search | Setup-free search existed but its setting was difficult to locate | Ask in a working chat; optional connection under Tools → Web search |
| Inspect/control a window | Settings was the task entry | Ask in a working chat or composer plus → Share window; zero required Settings visits |
| Change style | This chat and override machinery came first | Personalization starts with effective All chats controls; deliberate overrides remain |
| Remember a preference | Separate memory editor | Remember this beside the user's message, or Memory → My preferences |
| Manage a skill | Folder/YAML prerequisite in empty state | Tools → Skills → Import / Create / Draft from chat |
| Inspect context/task/files | Five independent header affordances | Three header icons; ring opens Context / Activity / Changes |
| Export/branch | Eight chat-menu entries | Three entries: Branch chat / Export / Chat details |
| Recover interrupted work | Separate limits/evidence destinations | Pause offers actual limit, View progress and relevant Adjust limits; explicit bounded continuation |

Window sharing is a meaningful consent decision, not eliminated by the zero
Settings target. With a ready model, target selection and initial sharing are
combined; uncertain input is never automatically repeated. Configuration/image
failures and real-model failures remain failures even when their recovery is
easier to reach. Ready means locally available/configured, not network-tested or
qualified for every task.

## Native and deterministic coverage

Native review uses the normal release on Windows with public synthetic data.
General's System preview is one continuous light/dark scene; existing infinity
icons, hints and theme palette are retained. Wide dark review covered all six
sections, the model detail dialog and populated memory. The final rebuilt release
was inspected in dark General/Advanced/Harness extensions and light General/
Advanced, then light Context/Activity/Changes. Context showed its token budget,
Activity showed the synthetic exchanges, and Changes showed its empty state.
Native Escape closed Settings and Chat details. This is a sampled native review,
not an exhaustive nested-screen matrix.

The Flutter suite exercises compact 420×480 light/dark settings and details,
unavailable tools, long/error content, stale model/MCP/summary saves, malformed
skill imports, output-limited drafts, interrupted operations and cleanup failure.
Recovery assertions check retained work, truthful save state and explicit actions.
Rust tests independently cover exact skill review, activation tokens and concurrent
revision refusal; they do not establish real-model skill quality.

Native qualification found Escape failing on protected modal routes. A test using
the real non-dismissible route reproduced it before the fix. Escape now invokes
the editor's existing Close path, retaining pending-operation and Save/Discard/
Keep editing checks. Nested cancellation keeps the original text and saves nothing.
Skills is visible on Tools' overview. Advanced opens a short task chooser;
testing/learning and diagnostics/storage are collapsed groups. Nested learning,
extension, comparison and response pages disclose technical text on demand;
sharing and experimental status remain visible at the decision. Failure tests
verify an open workflow editor retains its state when an error appears.

## Frozen real-model checks

The original [computer-use corpus](../../scripts/desktop-corpus-v1.json) was not
edited. Each started rerun kept 1024 output tokens, 30 seconds per request and a
90-second task allowance. Two focus-timeout setup attempts made no provider
request; after activating the fresh owned window, one model attempt ran per case.

| Case | Result | Observed outcome |
| --- | --- | --- |
| Qwen3.5-2B native draft | FAIL | Screenshot acknowledgement refused before input; entry unchanged and zero saves. No automatic retry/model switch. |
| DeepSeek V4.1 Flash save once | FAIL | Correct text and fresh post-input image, but proposed Save target refused by the unchanged test review; zero saves. Paused for review; no repeated submission. |

The DeepSeek run used four model calls/four tools, two reviewed inputs and two
fresh observations. The Qwen run stopped at initial image receipt validation.
These outcomes do not close milestone 14's reliability gate. Previous generated
view inspection remains separate passing evidence, not a substitute for input.

Learning checks also remain separate: a bounded DeepSeek skill draft parsed and
was not activated; Qwen replied but automatic extraction saved no verified
preference. Useful ordinary chat was preserved. No public artifacts contain
credentials, private screenshots or personal transcripts.

## Resource observations and remaining gaps

The final normal Flutter release with the synthetic profile was alive after a
3.11-second warmup. This is **not first-frame latency**. Its initial 15.06-second
sample overlapped the test suite: 233 MiB sampled peak working set, 261 MiB private
bytes and 4.391 CPU seconds (29.16% of one core). It is not an idle measurement.
Two subsequent settled samples observed 6.81% and 6.92% of one core. After native
navigation, the latter 15.12-second sample used 1.047 CPU seconds and ended at
322 MiB working set / 326 MiB private bytes. These endpoint values are not peaks.
No desktop helper remained running. The earlier preview's 0.31% sample is not
substituted for these final-release results; sustained idle cost needs investigation.
Short-lived processes can escape sampling. These numbers describe this Windows
host/profile, not an added-cost or low-end pass.

No matched pre-milestone startup/idle baseline was collected. Physical IME,
screen-reader behavior, other OS backends and lower-end hardware remain
unqualified. Exhaustive native light/dark/compact coverage of every mapped nested
screen remains open; widget coverage and reviewed native screens are reported
separately. Optional browser runtime setup still supplies instructions rather than
a one-click installer. The model-quality failures above require further harness/
model work with unchanged observable criteria.

## 2026-10-06 maintenance follow-up

Current build, launch, resource and qualification commands use Python. Retired
Iced/Tauri/Svelte source and root build/dependency caches were removed. Platform
CI remains deferred. The normal release was rechecked in wide dark/light General
and light Advanced. A narrow light window verified wrapped conversation text,
composer controls, the Conversations drawer, compact General, section selection
and Escape. This extends the sampled review; it does not close the exhaustive
nested-screen or physical-input/accessibility gaps.

Two Qwen preference/correction runs now save exact wording through bounded local
extraction. General-language extraction remains model-dependent. Real DeepSeek
testing reproduced and fixed missing approved project evidence after a failed
check; four independent skill trials then tied, retaining the baseline. Useful
automatic improvement and live regression restoration remain open. The frozen
computer-use corpus still has failures. [Learning](learning-followup.md),
[computer use](computer-use.md) and [idle resources](idle-resources.md) retain the
actual results; [remaining work](remaining-work.md) lists open gates. Original
provider, selected model and all 37 profile tables remain unchanged.
