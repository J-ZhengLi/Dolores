# Brick 1 acceptance — 2026-10-01

Functional foundation verified locally. **Resource acceptance is open**: the Windows webview build misses the provisional 150 MiB process-tree working-set target. The shell choice remains provisional; do not describe this as verified on low-end hardware or as a self-learning agent.

## Verified

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

## Measured baseline

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

- Compare a minimal native Rust UI with this Tauri build using the same core; measure both process-tree and private memory under comparable conditions. Set a user-relevant memory budget from evidence before finalizing the shell.
- Repeat release memory, startup, scrolling and cancellation checks on a 2-core/4-GiB reference device or representative equivalent.
- Run a real hosted or installed local model; no user endpoint/key was provided for live acceptance.
- Execute the supplied Windows/macOS/Linux CI matrix. macOS/Linux builds and platform UI behavior were not run in this Windows workspace.
- Packaging, signing, OS credential storage, long-history browsing, tools, memories, skills and learning remain future bricks.
