# Dolores acceptance — 2026-10-02

Flutter is the selected default. Its earlier Windows working-set measurement exceeds the provisional 150 MiB target; choosing it does not establish performance acceptance. **Release acceptance remains open** for representative hardware, sustained interaction/startup, real keyboard/IME input, screen-reader support and other platforms. This is not yet a self-learning agent.

## Brick 2.4 — Request controls and recovery

Flutter's Request settings dialog saves an output-token limit (default 2048, integer range 1–32768) and whole-response timeout (default 180 seconds, range 1–900). Defaults restore locally until saved. Schema 5 persists nonsecret settings independently of connection/credentials. A successful save activates a validated replacement provider; failed validation/storage preserves the previous state. Model switching, recovery and Forget preserve the settings. Completed replies retain the actual settings used, so later changes cannot alter their history/export details; older replies remain unavailable.

The response deadline includes headers, body, the bounded usage-option compatibility attempt, and queued delivery to Flutter. History reads and final atomic persistence remain outside it. A regression reproduced a stalled window suspending the adapter deadline because event forwarding stopped polling the provider; the bridge now bounds streaming plus delivery, and the paused-consumer test passes. Cancellation remains prompt and incomplete replies are discarded. Failure cards provide fixed local guidance and explicit Retry message/Request settings/Model connection actions as appropriate. Retry sends the current edited draft; no automatic failure retry was added.

Validation: 50 Rust tests pass (one native-vault test remains opt-in), all-target Clippy is clean, and the alternative Tauri host compiles. All 63 Flutter tests pass, including six settings/recovery checks; final analysis is clean. The final Windows release diagnostic passes 29 checks at both 1120×780 and 620×700 logical pixels. These include actual outgoing max_tokens, a mid-stream timeout with no partial saved pair, preserved draft, explicit retry of edited text, saved per-request settings, and unchanged composer footer geometry. Settings/recovery captures in both sizes were inspected; both themes and 390-pixel forms/error cards are covered by widget tests. Two separate processes using the bundled production C ABI verify saved settings/provider restoration, exported metadata, timeout atomicity, original snapshots after edits, and Forget retaining settings/history. Evidence stays under ignored `output/request-controls`; diagnostic checks exercise rendered widgets/controller/FFI/HTTP/SQLite, not OS pointer/IME input. All 163 publishable paths pass the local secret scan. No new resource-performance or live-provider/platform acceptance claim is made.

## Brick 2.3.2 — Usage and context visibility

UI refinement: the keyboard-help label is removed. Inside the composer's bottom-right corner, the model picker is followed immediately by a context ring and Send/Stop. The ring names its reading's source in its tooltip and measures the app's UTF-8 byte budget; unknown readings stay static and draft edits invalidate a preview. Its on-demand inspector shows a composition bar, exact byte breakdown, turn counts and expandable selectable system/history/draft text from the actual prepared message sequence. The header trajectory action opens independently paged saved exchanges and a live Log of submitted/prepared/first-text/saved/Stop/failure observations, with original model, local timestamps and monotonic elapsed time. The Log holds at most 200 in-process events across conversations, coalesces deltas, and contains no prompts, credentials, raw errors or network payloads. It is not a durable audit log or a provider timing trace.

Refinement validation: all 57 Flutter tests pass; 10 focused Rust bridge tests pass; Flutter analysis and bridge all-target Clippy are clean. New tests exercise compact light/dark context disclosures, exact Unicode source, known/unknown ring semantics, inline footer order, independent history paging without draft/scroll changes, live inspection without generation-time history reads, original-model/session association, Stop/failure restoration, saved-reply versus history-refresh failure labels, retention bounds, deletion and no fetch on typing. The Windows release diagnostic has 24 checks at wide and compact sizes, including the actual footer geometry, exact prepared system/history/draft sequence, system-text expansion and trajectory/log rendering. Evidence stays under ignored `output/context-inspector/final-wide` and `final-compact`; these checks exercise rendered widgets/controller/HTTP/FFI/SQLite rather than OS pointer/IME input. Final grammar and history-refresh labels are additionally covered by the final Flutter tests and normal release rebuild. The 159 publishable paths pass the local secret scan.

The release captures exposed a stale tree-shaken icon subset: new trajectory/context icons were absent even though Dart code had updated. The build helper now invalidates only generated Windows release-asset stamps before building. The icon subset is rebuilt with tree shaking retained; an actual bundled-font check fails before the fix and passes afterward for timeline, subject and list-alt glyphs (3916-byte subset). No SDK or user data is modified. Resource/platform/input acceptance remains open; no new memory-performance claim is made for this refinement.

The initial usage implementation: completed Flutter replies show provider-reported input/output tokens and a details dialog for nullable totals, cached input, reasoning, the original request model and context. Missing counters remain unavailable; genuine zeros remain zero. Its original on-demand Context control previewed the current draft with latest saved history, regardless of the viewed page, without provider I/O or writes. It displays included/saved/omitted turns and exact UTF-8 text bytes, including system/current-message content. The existing 40-complete-turn/128-KiB app bounds remain unchanged and are distinguished from unknown model token windows. Older turns remain saved; no token estimates, pricing or session aggregate is claimed.

Optional provider/storage capabilities preserve existing plugin implementations. The adapter requests usage, accepts final empty-choices chunks and retains the last valid snapshot without summing duplicates. Invalid or missing counters remain null. Only an explicit, bounded HTTP 400/422 unsupported-usage-option rejection permits one pre-stream compatibility attempt; generic validation, denial, transport and mid-stream errors are not retried. Schema version 4 adds cascading per-reply metadata, saved atomically with a complete turn. Older history is preserved, legacy replies omit metadata, and JSON/Markdown exports retain the new records. Failed saves/Stop/interruption never publish a partial turn or its metadata.

Validation: 41 Rust tests pass (one native-vault test remains ignored), all 49 Flutter tests pass, analysis and all-target Clippy are clean, and the alternative Tauri host compiles. New checks cover UTF-8 byte/turn bounds, known/unknown history totals, final usage-only chunks, null/invalid/partial/zero counters, duplicate snapshots, option rejection versus generic errors, migration from version 3 without history loss, restart/export/cascade/transaction rollback, saved request model, local-only previews and action exclusion while viewing an older page. Existing streaming/history and uninterrupted rich-composer input checks remain green.

Nineteen actual Windows release/controller/HTTP/FFI/SQLite checks pass at each of 1120×780 and 620×700 logical sizes. The native usage actions retain each reply's original model after switching; omitted usage still saves a complete reply. The final context/usage dialogs, unavailable footer, composer and light/dark conversation captures were inspected. Dialog content is capped at 440 logical pixels and scrolls as needed. Raw fixtures/captures stay in ignored `output/usage-context`; the normal release is rebuilt after diagnostics and the original preview data is preserved. These checks use a local fixture, not a real vendor or OS pointer/IME interaction. Cross-platform, accessibility, low-end performance and model-specific limits remain open.

Normal release observation: one visible, unminimized window using the isolated usage fixture database, two short messages displayed with an empty draft and launch-only connection guidance, eight seconds settling and five samples: **220.84 MiB working set / 232.70 MiB private bytes**, with no CPU-time increase at the available timer resolution across four idle intervals. This is a single development-machine observation, not paired with earlier runs or a low-end/peak/active-interaction acceptance result. The 150 MiB working-set target remains unmet. Raw machine details stay in ignored `output/usage-context/runtime.json`; the original preview was reopened with its existing data.

## Brick 2.3.1 — Markdown and code presentation

Flutter assistant replies now render native GitHub-flavored Markdown: selectable headings, lists, emphasis, quotes, tables and inline code. Code blocks have a language label, exact code Copy and their own horizontal scrollbar. Whole-message Copy keeps the original Markdown; user prompts stay literal. The shared theme supplies all colors and locally installed monospace fonts, without an added font asset or syntax highlighter. The renderer is pinned to `flutter_markdown_plus` 1.0.12 and `markdown` 7.3.1.

The input box now renders headings and fenced code as actual native editable blocks. Heading markers and fences are hidden in rich mode. Code cards have syntax colors and a language selector using selectively registered grammars from pinned `highlight` 0.7.0. The Source and Add text controls are removed. Down Arrow on the last visual line moves into the following block or creates a paragraph below the final card, closing incomplete fences as needed and preserving CRLF separators. Wrapped-line navigation, Shift+arrow selection and active composition stay native. Ordinary paragraphs retain literal inline Markdown. Enter in code inserts a newline; Ctrl/Command+Enter sends. Active IME composition remains native and suppresses folding/sending. Canonical Markdown retains its markers, whitespace and line endings when sending; edits replace the relevant body ranges. Bounded document undo includes keyboard-created exits. This is a scoped heading/code editor, with local native drag selection and composer-wide Select All, rather than a complete inline rich-text editor. No links/images load or idle timer is added. Layout falls back to full source above 32,768 UTF-16 code units, 800 newlines or 64 blocks; code coloring falls back to plain code above 8,192 code units, 200 newlines or 2,048 token nodes.

Forty-four Flutter tests pass and analysis is clean. Eight reply checks cover light/dark compact layout, code whitespace/Unicode copying, inert visible HTML, all image source types, explicit destination inspection/copy and rejection of other link schemes, partial fences, coalesced updates, final flush/disposal, completed-widget reuse, complete plain-text fallback, original whole-message Copy and following streamed layout while preserving a reader's scroll position. Twenty-eight composer checks cover native editable headings/code cards in both compact themes, syntax colors, exact pasted/sent Markdown, body edits, language changes, document undo/redo, boundary arrows and Down exits into existing/new prose, wrapped-line and Shift+arrow preservation, Enter/send distinction, native composition, CRLF/nested/tilde/partial fences, unknown/large-code fallback and continuing prose after complete/incomplete code. Three regressions reproduced empty-block deletion and focused-field-only Select All before the fix. Nine added tests now cover structural removal/undo, Ctrl/Command+A from heading and code, full Markdown copy/cut, whole-draft deletion and replacement, identical-body paste and typing, IME replacement, and returning to local copying/editing. Seven further regressions cover uninterrupted text input through heading/code creation and removal, removal of a later code card, visible native paragraph newlines, Shift+Enter at paragraph boundaries and over a selection, heading-to-prose transitions and repeated blank lines, and code Enter/Shift+Enter followed by Down and continued typing. Three of these first reproduced the lost input connection and stripped newline before the fix. The earlier tests refocused fields between edits and missed this regression; these updates connect once and continue typing without refocusing. The existing history and connection checks also pass.

Fifteen real Windows release/controller/HTTP/FFI/SQLite checks pass at each of 1120×780 and 620×700 logical sizes. They include discovery/model/stop/denial/interruption checks, rich streaming and source-preserving reload, actual native code/table widgets, and native editable headings/code cards sending/storing their exact canonical source. Native dark compact and light wide composer captures show the large heading, hidden fences, language selector and colored code; the final captures without Source/Add text controls and with headings/code selected together were inspected. The native Select All action also verifies both visible selection ranges without changing canonical Markdown; it does not simulate OS keyboard input. The welcome view now scrolls when a taller composer reduces available space. Raw captures and diagnostic data stay in ignored `output/rich-composer/focus`; no machine-identifying report is committed. The normal release is rebuilt after diagnostics.

Normal Windows release observation before the input-box refinement: one visible 1120×780 window (visibility checked through the Windows API), two fixture conversations with the single rich turn displayed, eight seconds settling and five samples: **220.74 MiB working set / 233.11 MiB private bytes**, with no CPU-time increase across the four idle intervals at the available timer resolution. This is not paired with earlier measurements, a peak/streaming frame-time measurement or representative low-end acceptance. The provisional 150 MiB target remains unmet. The raw machine/timestamp details are retained only in ignored local output; the original preview was reopened with its existing data.

After the rich composer change, a normal visible release using the isolated rich smoke database and an empty draft was observed after eight seconds settling across five samples: **232.48 MiB working set / 242.69 MiB private bytes / mean 0.82% of one core**. This checks the normal release at rest, not active rich typing, syntax-coloring peaks or low-end performance, and is not a paired before/after comparison. The 150 MiB target remains unmet. Raw details stay in ignored `output/rich-composer`; the normal preview is reopened with its original data.

Active reply rendering coalesces at 80 ms, with no idle timer. Completed visible replies reuse their parsed widget. Above 32,768 UTF-16 code units or 800 newline characters, rich layout yields to labeled selectable plain text without truncating the source. This bounds rich layout work, not total plain-text shaping/memory cost. Native clipboard interaction, OS input/theme/accessibility, macOS/Linux fonts and rendering, and representative low-end performance remain unverified. Next is provider-reported usage and visible context limits; no token/cost accuracy is claimed by this rendering brick.

## Brick 2.2 — history browsing, export and shared UI contract

Implemented bounded keyset browsing beyond the old 100-session/40-turn view: one 50-session sidebar page and one 80-message transcript page, explicit Older/Newer/Latest controls, and complete Markdown/JSON export through the native Save dialog integration. Draft/page/scroll state restores for the 20 most recently visited views during the current launch. Export streams a consistent full-conversation snapshot into a new file and never replaces an existing file. Provider context remains bounded independently of browsing. `UI.md` now states the visual contract, with the shared palette, dimensions and Material theme in Flutter's `theme.dart`.

Validation: 32 Rust workspace tests and eight Flutter tests pass; analysis and workspace all-target Clippy are clean. New storage tests cover 137 tied-timestamp sessions, both pagination directions, deleted boundaries, invalid page sizes, 123-turn history after reopening, complete Unicode/Markdown/JSON export, and writer failures. Bridge export checks cover existing destinations and errors after writing begins without partial publication. Controller/widget checks cover bounded page replacement, draft/view restoration, failures, canceled export, generation exclusion, export menu, compact layout and theme reuse.

Thirteen actual release renderer/controller/FFI/SQLite checks pass at each of the wide/compact sizes. An isolated legacy database reaches all 137 conversations and all 246 saved messages, exports both endpoints of the full history, preserves view state, rejects replacement, and sends from an earlier page using the latest bounded provider context. Nine existing streaming/model-selection regression checks also pass. Both themes and the compact history controls were visually inspected. The normal release was rebuilt and the native file-selector library is bundled. The Windows build helper's generated-only junction fallback works without system/SDK changes.

Normal release observation: one visible window on the Windows development machine, 50 sidebar entries/80 short messages loaded from the isolated 137-chat database, eight seconds settling and five samples: **231.48 MiB working set / 240.61 MiB private bytes, 0% of one core across the four idle intervals**. This single observation is not a paired comparison, peak/export measurement or low-end acceptance; the provisional 150 MiB target remains unmet. No idle paging/export timer was added.

Evidence: private local benchmark report (excluded from Git). Native Save dialog pointer/keyboard interaction, OS theme/IME/accessibility, macOS/Linux sandbox export and low-end hardware remain unverified. macOS user-selected file access and outbound-network entitlements are configured but not validated here. Brick 2.3 starts next with Markdown/code rendering.

## Brick 2.1.1 — retained model discovery and selection

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

Evidence: private local benchmark report (excluded from Git). Reproduce with the Flutter app's [instructions](../apps/dolores_flutter/README.md). The OS vault and SQLite have separate transactions: interrupted/failed cleanup may leave an unreferenced secure entry; failures are reported and never trigger plaintext storage. Locked-vault tests use an injected deterministic implementation; Windows lock/unlock UI, macOS Keychain, Linux Secret Service, real hosted providers and CI are not verified locally. History remains unencrypted. This retained record predates brick 2.2; see the current history acceptance above.

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
- Packaging, signing, tools, memories, skills and learning remain future bricks. OS credential storage and long-history browsing are implemented above.
