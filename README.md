# Dolores

A lightweight desktop agent harness, inspired by the gradual awakening of Dolores in *Westworld*. Start with conversation; add tools, memory, reusable skills and evaluation one brick at a time.

**Current bricks:** Flutter desktop, Rust core, system theme, OpenAI-compatible streaming chat, cancellation, SQLite history and replaceable provider/storage/credential interfaces. Remember connections using the OS credential store, restore them after restart, retry recovery, or forget them while keeping conversations. Learning and external plugin loading are on the roadmap.

Flutter is selected as the default after the visual/resource comparison. The earlier Windows observation was 224.36 MiB working set and 229.52 MiB private allocation; it exceeds the provisional 150 MiB working-set target. Refinement now proceeds in Flutter, with low-end performance and cross-platform release acceptance still open. Iced and Tauri remain optional comparison shells. See [acceptance](docs/ACCEPTANCE.md).

The [Flutter desktop](apps/dolores_flutter/README.md) uses the Rust core/plugins through a bundled C ABI. Keep the entire generated release directory together.

## Run

Install Rust stable, Flutter 3.47.5 and its [desktop prerequisites](https://docs.flutter.dev/platform-integration/desktop). Linux secure storage also needs the D-Bus development library and a running Secret Service. On Windows, build directly from PowerShell:

```powershell
rtk proxy powershell -NoProfile -File scripts/build-flutter.ps1 -FlutterSdk C:/path/to/flutter
./apps/dolores_flutter/build/windows/x64/runner/Release/dolores_flutter.exe
```

Open Model connection, enter the API base URL and a key if required, then **Fetch models**. Search and check the models you want available, save, and choose the active model beside the message box. Manual entry is available for servers that do not list models. Local example: `http://localhost:11434/v1` with a model already installed on your server. Hosted example: your provider's HTTPS API prefix. Dolores does not download or launch a local model.

Enable **Remember connection** to keep the key in Windows Credential Manager, macOS Keychain or Linux Secret Service. Disable it to use the connection until the app closes. A blank key field keeps the current/saved key for the same endpoint; **Use without a key** explicitly clears it on save. Switching models preserves the conversation, draft and key and is disabled during a response. Your choices and last active model survive restart, with the connection restored only when remembered. No key is returned to the UI or stored in SQLite. If recovery fails, history remains available and the app offers retry or reconnection. **Forget saved connection** removes the stored key and connection while keeping history.

Preferences and completed conversations are stored unencrypted in `dolores.db` under the OS application data directory for `dev.dolores.desktop`. Open one shell at a time against a data directory. Remembered keys are scoped to that directory; a copied database does not restore its key elsewhere. Model requests include bounded history and go to your configured endpoint. Saving settings makes no network request. Enter sends, Shift+Enter inserts a newline; replies are selectable and have Copy actions.

For an isolated test workspace, set `DOLORES_DATA_DIR` to an absolute directory before launching. For a deterministic local fixture, run `pnpm demo:server`, connect to `http://127.0.0.1:19421/v1` with model `dolores-mock`, and leave the key blank. The fixture is development tooling, not a language model. Send `slow`, `fail`, or `truncated` to exercise stop and error handling.

On Windows, `scripts/measure-runtime.ps1 -AppProcessId <pid>` samples either shell and its descendants. It reports working sets, private bytes and interval CPU as a percentage of one logical core. Shared pages can be counted more than once. Measure a release without smoke/debugging, record hardware, cache state, window size/DPI and startup separately. The alternative web UI's `dolores-interactive` mark records history readiness; it is not a native startup measurement.

The optional webview shell and fixture need Node.js ≥22.12 (24 recommended), pnpm 11 and `pnpm install --frozen-lockfile`. The webview shell also needs [Tauri's platform prerequisites](https://v2.tauri.app/start/prerequisites/).

```sh
pnpm build                 # UI type check + production assets
pnpm test:core             # core/provider/storage tests
cargo test --workspace     # includes native host checks
pnpm desktop              # build and open the default Flutter release
pnpm desktop:build        # Flutter release bundle, no installer
pnpm desktop:iced         # optional Iced development window
pnpm desktop:iced:build   # optional Iced release
pnpm desktop:web          # alternative Tauri/Svelte desktop
pnpm desktop:web:build    # alternative release executable
pnpm dev                   # browser UI preview, no model connection
```

Use `rtk` as a prefix for shell commands when working under the supplied workspace instructions. Node is development tooling; it is not bundled as an application sidecar. Supported design targets are Windows, macOS and Linux; check the [acceptance record](docs/ACCEPTANCE.md) for what was actually verified.

## Design and iteration

- [Architecture](docs/ARCHITECTURE.md): boundaries, plugin ports, resource limits and tradeoffs.
- [UI contract](docs/UI.md): one shared visual style and interaction vocabulary.
- [Roadmap](docs/ROADMAP.md): small bricks with acceptance gates.
- [Research](docs/research/architecture-options.md): official sources and alternatives.
- [Desktop API](docs/API.md): IPC and outgoing provider contract.

The core lives in `crates/dolores-core`; plugins in `crates/dolores-provider-openai`, `crates/dolores-store-sqlite` and `crates/dolores-credentials`; default host/UI in `crates/dolores-flutter-bridge` and `apps/dolores_flutter`. Alternatives live in `crates/dolores-native`, `src-tauri` and `src`. New plugins implement core traits and are explicitly registered. Compiled native plugins are not sandboxed. `FLUTTER_SDK`, the ignored local SDK or PATH supplies Flutter to the development helper; Node is never bundled.

For Flutter checks, use `flutter analyze` and `flutter test` in the app directory. With the fixture server running, build with `scripts/build-flutter.ps1 -Smoke`, then run `scripts/test-flutter.ps1` against a fresh output directory. Build with `-RestartSmoke`, then run `scripts/test-connection-restart.ps1` for separate-process recovery using a generated test key, which is deleted afterward. Rebuild without diagnostic switches before normal usage or resource measurement. See the [bridge contract](docs/flutter/flutter-api.md) for recovery and cross-store cleanup limits.

For native controller/renderer checks, start `pnpm demo:server`, build `cargo build -p dolores-native --release --features smoke --locked`, then run `powershell -NoProfile -File scripts/test-native.ps1`. The runner creates isolated fixture data and four screenshots, then closes its app. It drives controller events and real HTTP; pointer, keyboard and IME interaction remain separate UAT. Rebuild without `--features smoke` for normal usage and resource measurements.
