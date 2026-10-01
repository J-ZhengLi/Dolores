# Dolores architecture options

Research checked on 2026-10-01. This document records evidence and recommendations; the project architecture document owns final decisions.

## Desktop framework comparison

| Option | Verified architecture | Fit for Dolores | Caveats to test |
| --- | --- | --- | --- |
| Tauri 2 + Rust core + small web UI | Uses a Rust core and operating-system WebView: WebView2 on Windows, WKWebView on macOS, WebKitGTK on Linux. WebView libraries are dynamically linked rather than included in the executable. [Tauri process model](https://tauri.app/concept/process-model/) | Recommended starting point: rich chat UI while avoiding a bundled Node/Chromium runtime. Core can also support a future CLI. This recommendation is an inference from architecture, not a benchmark. | WebView processes still consume memory; platform rendering and deployment dependencies differ. Measure the entire process tree. |
| Electron | Embeds Chromium and Node.js; uses Chromium's multiple-process architecture. [Electron introduction](https://www.electronjs.org/docs/latest/), [process model](https://www.electronjs.org/docs/latest/tutorial/process-model) | Strong web ecosystem and a consistent bundled browser. Could fit a team optimizing for JavaScript delivery speed. | A bundled browser/runtime works against the small-distribution goal. No measured RAM comparison exists for this project. |
| Iced + Rust | Rust GUI library with Windows/macOS/Linux support, reactive state/messages/views/updates, wgpu rendering and a tiny-skia software fallback. Its README labels it experimental. [Iced official repository](https://github.com/iced-rs/iced) | Worth a future spike if eliminating WebViews becomes a measured requirement. Shares Rust core types directly. | Rich Markdown, selectable text, accessible controls, and extension UI need an explicit prototype. Software fallback does not guarantee good low-end performance. |

No framework alone proves that Dolores will run well on low-end machines. Installer size, resident memory, CPU while idle/streaming, startup latency, long-history behavior, and GPU compatibility require repeatable measurements on a declared baseline machine.

## Verified inspirations

The user's DeepSeek Harness reference is identifiable: the official project is [deepseek-ai/deepseek-harness](https://github.com/deepseek-ai/deepseek-harness), which describes an everything-is-a-plugin architecture powered by Cordis. Its README explicitly warns that developer-preview APIs will have compatibility-breaking changes. The [official product page](https://www.deepseek.com/en/harness/) describes plugins extending tools, skills, and the interface. Dolores can borrow composability and explicit extension lifecycles without depending on the Node/Cordis runtime or claiming compatibility with its plugins.

Hermes provides a concrete interpretation of learning: bounded factual/user memory, searchable past sessions, and reusable procedural skills. Its built-in memory is injected as a frozen snapshot at session start, keeping a stable prompt prefix; memory writes become available in subsequent sessions. It also supports approving memory writes. [Hermes memory documentation](https://hermes-agent.nousresearch.com/docs/user-guide/features/memory/)

Hermes loads skill metadata first, then a skill's instructions and supporting files on demand. Its guide distinguishes facts in memory from procedures in skills and permits the agent to create/update skills. [Hermes skills documentation](https://hermes-agent.nousresearch.com/docs/guides/work-with-skills/)

Recommendation: describe Dolores learning as improved durable memory and workflows, then measure whether accepted lessons improve repeat tasks. Do not imply that saving text changes model weights or guarantees increasing intelligence. Begin with transparent, editable, versioned memories and skills; delay automated optimization until evaluations exist.

## Proposed lightweight boundaries

These are design proposals, not capabilities already delivered:

1. A small Rust kernel owns session identity, cancellation, resource limits, the plugin registry, and policy checks. Keep it independent of Tauri.
2. Provider, conversation storage, tools, memory, skills, and later agent-loop implementations use narrow Rust interfaces. Built-in implementations are linked at build time initially.
3. A desktop adapter exposes only explicit commands and typed results to the UI. Credentials and provider HTTP requests live in Rust. Browser development mode uses a clearly identified mock transport.
4. Use one desktop window, bounded visible history, bounded queues, one active chat generation initially, and lazy optional features. Avoid a background server, bundled Python/Node runtime, mandatory vector store, local inference engine, or resident plugin workers for the first release.
5. Use a durable local store before introducing search embeddings. Keep transcript storage separate from the bounded prompt context; large history must not automatically create a large request.

For token streaming, Tauri documents channels as ordered and optimized for streaming. Its event mechanism is unsuitable for high-throughput/low-latency data and does not provide fine-grained capability control over event data. Recommendation: use a per-request channel with session/request IDs and explicit terminal states, avoiding global token broadcasts. [Calling the frontend from Rust](https://tauri.app/develop/calling-frontend/)

## Plugin lifecycle and trust proposal

Start with trusted, compiled first-party plugins. A minimal descriptor contains a unique ID, version, interface version, kind, declared dependencies, requested capabilities, and enabled state. Registration must reject duplicate IDs and missing required dependencies; later versions should reject dependency cycles and incompatible interface versions. Activation should be lazy. Disabling/disposal should cancel owned work and remove listeners/resources; shutdown should dispose in reverse dependency order. Do not implement dynamic native-library loading or hot reload merely to call the scaffold modular.

Separate capability declarations from enforcement. Trusted in-process Rust plugins can access the process's privileges and are not sandboxed. Tauri capabilities constrain WebView access to commands; they explicitly do not protect against malicious/insecure Rust code. Custom application commands are exposed to all windows by default unless configured through the application manifest. [Tauri capabilities](https://tauri.app/security/capabilities/)

Before accepting third-party code, add a versioned process protocol, explicit installation provenance, timeouts/output limits, and declared filesystem/network scope enforced by an appropriate OS boundary. A subprocess alone provides crash isolation, not a security sandbox. Keep third-party code unavailable until these guarantees are tested; a future WASM backend is optional and must follow a real extension requirement.

## Small-first roadmap implications

| Brick | Observable acceptance | Research limitation |
| --- | --- | --- |
| Scaffold | Desktop shell and browser preview share a transport contract; extension registry has one demonstrable implementation | Cross-platform claim requires builds/tests on each OS |
| Basic chatbot | One real provider; streamed responses; cancellation; useful errors; session history; private credentials | Test network errors and long transcripts, not only a demo |
| Tools | Bounded tool loop; user-visible calls/results; permissions and cancellation | Policy checks are not automatically an OS sandbox |
| Memory | Inspect/edit/delete records with provenance and limits; repeat-task retrieval | Storage is not evidence of quality improvement |
| Skills | Lazy skill loading; proposed lessons; review/version/rollback; repeatable evaluation tasks | Automatic self-improvement remains a hypothesis until measured |
| Ecosystem | External plugin protocol and trustworthy installation lifecycle | ABI, isolation, permissions, resource limits, and compatibility need separate acceptance |

Performance budgets should be recorded as targets and reported as measured only after release-build testing. Report startup, idle, streaming, and long-history results separately and include child WebView processes. Local model memory belongs to the model host's budget if introduced later.

## Windows memory follow-up

The initial release-build measurement reported by the implementation task was approximately 445 MiB summed working set and 210 MiB private bytes across seven processes, with approximately 1.39 s startup using warm caches on a Windows development machine. The WebView2 CDP debugging flag was attached. This exceeds the provisional 150 MiB working-set target; a separate run without debugging instrumentation is still required. These numbers are a project observation, not a framework benchmark or low-end acceptance result.

Checked dependency versions in the workspace lockfile: Tauri 2.12.1, Wry 0.57.0, webview2-com 0.39.1. Safe documented controls and their limits:

| Surface | Exact API and availability | Meaning / tradeoff |
| --- | --- | --- |
| Wry 0.57.0, Windows only | `WebViewExtWindows::set_memory_usage_level(&self, MemoryUsageLevel) -> Result<()>`; enum values `Low` and `Normal`, default `Normal`; WebView2 Runtime >= 114.0.1823.32. Older runtimes do nothing. [Wry Windows extension](https://docs.rs/wry/latest/x86_64-pc-windows-msvc/wry/trait.WebViewExtWindows.html), [enum](https://docs.rs/wry/latest/x86_64-pc-windows-msvc/wry/enum.MemoryUsageLevel.html) | Intended for visibility/inactivity changes. Set `Low` for an inactive view and explicitly restore `Normal` when it becomes active. This is a target level, not a numeric memory cap. |
| WebView2 native | `ICoreWebView2_19::put_MemoryUsageTargetLevel(COREWEBVIEW2_MEMORY_USAGE_TARGET_LEVEL)`; stable Win32 SDK introduced in 1.0.1823.32. [Microsoft interface reference](https://learn.microsoft.com/en-us/microsoft-edge/webview2/reference/win32/icorewebview2_19?view=webview2-1.0.4129.50) | Best effort, returns before reduction finishes. Scripts and network connections can continue; some browser-process memory may be paged to disk. Later access pages it back in and can reduce responsiveness. Restoration to `Normal` is not automatic. Choose this mode or `TrySuspend`/`Resume`; Microsoft advises against mixing them. |
| Tauri 2.12.1 | `WebviewWindow::with_webview` supplies the platform handle on the main thread, including the Windows controller. Pin the minor version when using native handles because underlying bindings can change. [Tauri API](https://docs.rs/tauri/latest/x86_64-pc-windows-msvc/tauri/webview/struct.WebviewWindow.html#method.with_webview) | The checked public window/config documentation has no `memoryUsageLevel` option. A Windows-specific native adapter would be needed; a Wry method is not automatically a Tauri window method. No implementation was added in this research pass. |
| Wry theme / WebView2 color scheme | Wry `WebViewExtWindows::set_theme(Theme)`, Runtime >= 101.0.1210.39 (older runtimes return an error); WebView2 `CoreWebView2Profile.PreferredColorScheme`. [Wry API](https://docs.rs/wry/latest/x86_64-pc-windows-msvc/wry/trait.WebViewExtWindows.html), [Microsoft color scheme](https://learn.microsoft.com/en-us/dotnet/api/microsoft.web.webview2.core.corewebview2profile.preferredcolorscheme?view=webview2-dotnet-1.0.4129.50) | Controls browser UI and `prefers-color-scheme`, with `Auto` following the OS. Useful for consistent appearance; no documented RAM-saving guarantee. |

Microsoft additionally recommends sharing a WebView2 environment if multiple views become necessary, releasing unused views, inspecting JavaScript heap/retained DOM/listeners, and keeping GPU acceleration enabled for rendering performance. Its troubleshooting guidance suggests comparing minimal HTML with the real application to separate host/runtime overhead from page complexity. [WebView2 performance guidance](https://learn.microsoft.com/en-us/microsoft-edge/webview2/concepts/performance)

Recommendation: first measure the uninstrumented release build in active, idle, and minimized states. If adding `Low`, measure both inactive memory and reactivation latency, keeping `Normal` during active chat. Record working set and private bytes separately: paging can lower resident memory without proving a smaller allocation footprint. None of these APIs establishes that the 150 MiB active working-set target will be met.
