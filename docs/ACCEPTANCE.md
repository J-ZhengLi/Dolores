# Dolores acceptance — 2026-10-01

Flutter is the selected default. Its earlier Windows working-set measurement exceeds the provisional 150 MiB target; choosing it does not establish performance acceptance. **Release acceptance remains open** for representative hardware, sustained interaction/startup, real keyboard/IME input, screen-reader support and other platforms. This is not yet a self-learning agent.

## Brick 2.1.1 — model discovery and selection

The Flutter connection dialog fetches a searchable model list and enables a chosen subset, with manual IDs as a fallback. The composer picker changes the active model without discarding the conversation/draft or re-entering a key. Choices and the active model persist; remembered credentials continue to restore after restart.

Validation: 29 Rust workspace tests and five Flutter widget tests pass. HTTP tests cover authenticated sorted/deduplicated discovery, denial, unsupported/malformed/empty lists and redirects; connection tests cover enabled-only switching, unchanged vault identity, restart restoration and launch-only key preservation. Listing/switching are rejected during generation. Nine real HTTP/FFI/SQLite controller checks pass at both wide and compact sizes, including a fixture that rejects requests using the wrong selected model. Four fresh processes verify model/choice restoration, authenticated discovery with the saved key, forgetting and history retention. The generated test key is deleted and absent from data files. Analysis and all-target Clippy are clean.

Evidence: private local benchmark report (excluded from Git). Models returned by a provider are not guaranteed to support text chat. Real vendor endpoints, OS input/accessibility, macOS/Linux and low-end hardware remain unverified. Earlier resource measurements below are historical; this refinement adds no idle discovery timer.

## Brick 2.1 — retained credential checks

Implemented a replaceable OS credentials port/plugin, restart restoration, Remember connection, recovery warnings/Retry, and Forget without conversation loss. SQLite schema 2 stores nonsecret metadata and opaque references, with additive migration from schema 1. API keys remain in the OS vault or process memory for a launch-only connection; no plaintext fallback exists. Endpoint binding and per-directory vault names prevent restoring a key to another configured endpoint/workspace.

| Check | Result and boundary |
| --- | --- |
| Rust workspace | 26 tests passed; native-vault test separately opt-in; all-target Clippy passed with warnings denied |
| Native Windows credential store | Generated entry written, reopened/read, deleted and absence confirmed; no unrelated entries accessed |
| Four separate Flutter processes | Save → restore and authorized real fixture stream → forget while retaining history → restart confirms connection absent and history retained |
| Secret handling | Bootstrap omits keys; generated key absent from DB/data files; no real user key used in validation |
| Recovery/endpoint isolation | Locked/missing vault, keyless restoration without a vault, launch-only mode, failed secure save preserving old settings and changed-endpoint rejection passed against an injected test vault |
| SQLite migration/rollback | Legacy DB/history retained; metadata and preferences commit atomically, including a forced transaction failure |
| Flutter | Three widget tests pass; analysis clean; recovery, hidden/omitted key and Forget behavior covered |
| Streaming/rendering | Six real HTTP/FFI/SQLite checks pass at each of 1120×780 and 620×700 logical pixels; updated settings/light/dark captures inspected |
| Default build | Root Flutter build helper successfully builds the normal Windows release with bundled Rust library |
| Normal release observation | One visible window, two complete fixture turns and reconnect guidance: 216.56 MiB working set / 228.91 MiB private bytes, five settled samples averaging 0.30% of one logical core; this is not a paired comparison or low-end acceptance |

Evidence: private local benchmark report (excluded from Git). Reproduce with the Flutter app's [instructions](../apps/dolores_flutter/README.md). The OS vault and SQLite have separate transactions: interrupted/failed cleanup may leave an unreferenced secure entry; failures are reported and never trigger plaintext storage. Locked-vault tests use an injected deterministic implementation; Windows lock/unlock UI, macOS Keychain, Linux Secret Service, real hosted providers and CI are not verified locally. History remains unencrypted. Brick 2.2 is next; the rest of brick 2 is not yet implemented.

## Brick 1.2 — retained Flutter trial

A real Flutter 3.47.5 Windows release now shares the same Rust core/provider/store through a bundled C ABI. Two Rust bridge tests, two Flutter widget tests and six real HTTP/FFI/SQLite controller checks passed. Static analysis reports no issues. Light, dark, settings and narrow renderer captures were inspected at 1120×780 and 620×700 logical pixels.

| Normal-release metric, new paired observation | Iced | Tauri | Flutter |
| --- | --- | --- | --- |
| Process-tree working set, mean | 29.60 MiB | 427.47 MiB | 224.36 MiB |
| Process-tree private bytes, mean | 13.81 MiB | 197.71 MiB | 229.52 MiB |
| Runtime bundle, uncompressed | 8.64 MiB | 12.41 MiB | 31.13 MiB |

The same two-turn fixture was copied into isolated directories for each normal release. Windows were visible and unminimized, with verified 1120×780 logical client areas at 144 DPI. Five samples after eight seconds of settling, one launch each, on the Windows development machine. Flutter's working set is lower than Tauri's, but private allocation is higher. GPU/system-wide memory is not included. The Flutter bundle includes its engine and Rust DLLs, AOT library and assets; its tiny launcher is not the bundle size. All entries exclude already-installed system runtimes.

Evidence: private local benchmark report (excluded from Git), [design/resource comparison](research/flutter-shell-trial.md), [build/run instructions](../apps/dolores_flutter/README.md). Reproduce with `scripts/compare-shells.ps1` after rebuilding Flutter without diagnostic switches. Flutter was subsequently selected. Low-end hardware, real model/OS input/IME, theme switching, accessibility, startup/scrolling and macOS/Linux acceptance remain open; CI definitions have not been executed.

## Brick 1.1 — retained comparison

Both normal release builds used the same SQLite fixture data with two complete turns, a visible unminimized 1120×780 logical window at 144 DPI (150%), no browser debugging and no smoke feature. A Windows development machine. Five process-tree samples per launch after eight seconds of settling; this is not low-end hardware. The Tauri executable retains the verified brick 1 implementation; Iced reuses the same core/provider/store crates.

| Metric | Iced + tiny-skia | Tauri + WebView2 |
| --- | --- | --- |
| Process-tree working set, mean | 29.47 MiB | 430.60 MiB |
| Process-tree private bytes, mean | 13.68 MiB | 202.58 MiB |
| Processes | 1 | 7 |
| Windows executable | 9,060,352 bytes / 8.64 MiB | 13,014,528 bytes / 12.41 MiB |
| Sampled idle CPU, one logical core | 0.00% | 0.30% |

Working set is 93.16% lower in this paired observation. Shared pages can be counted more than once; private bytes represent committed allocation, not resident physical-memory savings. Two additional native launches averaged 29.57 and 29.48 MiB working set, with 13.78 and 13.69 MiB private bytes. Their short idle intervals also showed no CPU-time increase at the available timer resolution. Do not turn a rounded zero into a claim of zero work under all conditions.

Earlier launches showed roughly one core of CPU consumption, including a native 1040×760 run. That did not recur in subsequent native launches, and the renderer trace showed five startup redraws rather than a continuous redraw loop. The cause was not established; repeat startup, foreground/background and sustained interaction checks before accepting performance. No driver, framework or app fix is claimed from that observation. The CPU script also now uses floating-point arithmetic to avoid rounding sub-second deltas to integers.

Raw machine-specific evidence is retained locally and excluded from Git. Measurement helper: `scripts/measure-runtime.ps1`. Local screenshots and detailed traces: ignored `output/native-comparison`; repeatable controller/renderer runner: `scripts/test-native.ps1` (requires a smoke-feature build and the fixture server).

| Native check | Result and limit |
| --- | --- |
| Windows release | Built with software renderer and no wgpu/webview dependency |
| Shared Rust workspace | 18 tests passed; Clippy all targets including smoke feature passed with warnings denied |
| Real loopback HTTP fixture | Configuration, Unicode streaming and two complete persisted turns passed |
| Stop, HTTP 401, truncated stream | Draft restored, failed/partial turns absent, raw provider body hidden |
| Stored history reload | Four messages reload through the controller; SQLite restart/deletion behavior also covered by existing store tests |
| Light/dark/settings/narrow screenshots | Real renderer captures inspected, including CJK text and wrapping |
| Run bounds | One reserved run; bounded event queues; newest 80 messages retained by UI/store; core context/output limits unchanged |
| Input and accessibility | Ctrl/Command+Enter, Copy, Back/Escape implemented; actual pointer/keyboard/IME UAT and screen-reader acceptance remain open |
| Native startup | Cold/warm readiness timing not measured; old webview readiness timing below is not transferable |

The Iced shell deliberately uses a settings page and Ctrl/Command+Enter. Iced has no current screen-reader integration; arbitrary transcript selection is also absent (whole-message Copy is available). Tauri remains runnable with `pnpm desktop:web`; Iced with `pnpm desktop:iced`. The selected Flutter shell uses `pnpm desktop`.

## Brick 1 — retained webview evidence

### Verified

| Check | Result |
| --- | --- |
| Frozen frontend dependency installation | Passed |
| Svelte/TypeScript + accessibility diagnostics | 0 errors, 0 warnings |
| Production frontend build | Passed; JS 53.54 kB (20.70 kB gzip), CSS 10.30 kB (2.79 kB gzip) |
| Rust workspace tests | 14 passed across core, provider, storage and host |
| Rust Clippy, all targets, warnings denied | Passed |
| Rust and frontend formatting | Passed |
| Windows release executable | Built successfully; 13,014,528 bytes (12.41 MiB), no installer |
| Native desktop against real local HTTP fixture | Connection configuration → Unicode streaming → complete turn persistence passed |
| Stop during preparation and active response | Passed; draft restored and cancelled turn absent after reload |
| Process restart | Completed history survives; connection/credential must be configured again |
| HTTP 401 and truncated stream | User-facing errors; raw fixture error body not exposed; failed turns absent after reload |
| Session delete and remaining history | Passed in native UI; cascade also covered by storage test |
| System light/dark + narrow layout | Screenshots inspected; native narrow conversation selector checked |
| Console errors | None in final native checks and production browser preview |

The fixture is `scripts/mock-provider.mjs`, bound to loopback only. It is not a language model. Native validation uses `output/native-smoke-data`, separate from default user data. Screenshots and raw normal-run metrics are in ignored `output/playwright`.

### Original measured baseline

A Windows development machine; release executable with two short stored turns. This is **not** the proposed 2-core/4-GiB reference machine.

| Metric | Observation | Boundary |
| --- | --- | --- |
| No-debug idle process-tree working set | Mean 421.80 MiB; range 421.67–421.93; 7 processes, 5 samples | Above provisional 150 MiB target. Summed working sets may count shared pages multiple times. |
| No-debug process-tree private bytes | Mean 189.34 MiB; range 189.27–189.47 | A separate allocation measure, not equivalent to working set or physical system-memory delta. |
| With CDP testing attached | Mean 445.26 MiB working set; about 210 MiB private bytes | Test instrumentation increases the observation; it does not explain the whole gap. |
| Warm process start → UI ready | 1.3914 seconds in one instrumented run | Process start timestamp to `dolores-interactive` mark (initial history loaded and rendered). Cold startup target remains unverified. |

Normal-run working set was measured with `scripts/measure-runtime.ps1`, including the native host and all its descendants, after removing the browser debugging argument. It excludes the separate fixture server, browser preview and build tools. Raw evidence: `output/playwright/runtime-metrics.json`. Warm startup was measured before any page reload; absolute activity timestamps remain private.

No optimization benefit is claimed from WebView2's inactive-memory control. Its documented Low mode is best effort and can affect responsiveness; see [research](research/architecture-options.md). Do not weaken sandboxing or silently lower acceptance thresholds to label this target achieved.

## Remaining acceptance

- Complete native pointer/keyboard/copy-paste/IME/system-theme UAT, investigate any recurring CPU spike, and resolve or explicitly accept accessibility/selection gaps before release.
- Repeat release memory, startup, scrolling and cancellation checks on a 2-core/4-GiB reference device or representative equivalent.
- Run a real hosted or installed local model; no user endpoint/key was provided for live acceptance.
- Execute the supplied Windows/macOS/Linux CI matrix. macOS/Linux builds and platform UI behavior were not run in this Windows workspace.
- Packaging, signing, OS credential storage, long-history browsing, tools, memories, skills and learning remain future bricks.
