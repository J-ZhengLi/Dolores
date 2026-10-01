# Dolores

A lightweight desktop agent harness, inspired by the gradual awakening of Dolores in *Westworld*. Start with conversation; add tools, memory, reusable skills and evaluation one brick at a time.

**Initial brick:** Rust core, Tauri 2 desktop, Svelte 5 UI, system light/dark theme, OpenAI-compatible streaming chat, cancellation, SQLite conversation history, and replaceable provider/storage interfaces. Learning and external plugin loading are on the roadmap.

The functional Windows build is verified. The desktop shell is **provisional**: idle measurements were 421.80 MiB summed process-tree working set and 189.34 MiB private bytes on this machine. Low-end performance is not established. The next roadmap step compares a native Rust UI before adding features; see [acceptance](docs/ACCEPTANCE.md).

## Run

Install Node.js ≥22.12 (24 recommended), pnpm 11, Rust stable, and the [Tauri prerequisites for your OS](https://v2.tauri.app/start/prerequisites/). Then:

```sh
pnpm install --frozen-lockfile
pnpm desktop
```

Open Connection settings, enter the API base URL, exact model ID and an API key if required. Local example: `http://localhost:11434/v1` with a model already installed on your server. Hosted example: your provider's HTTPS API prefix. Dolores does not download or launch a local model.

The key stays in native process memory for this application session. Save settings again after each restart. Preferences and completed conversations are stored unencrypted in `dolores.db` under Tauri's application data directory for `dev.dolores.desktop`. Model requests include bounded conversation history and go to the endpoint you configure. No requests occur simply from saving connection settings.

For an isolated test workspace, set `DOLORES_DATA_DIR` to an absolute directory before launching. For a deterministic local fixture, run `pnpm demo:server`, connect to `http://127.0.0.1:19421/v1` with model `dolores-mock`, and leave the key blank. The fixture is development tooling, not a language model. Send `slow`, `fail`, or `truncated` to exercise stop and error handling.

On Windows, `scripts/measure-runtime.ps1 -AppProcessId <pid>` samples the native process and all its descendants. It reports summed working sets and private bytes; shared pages can be counted more than once. The `dolores-interactive` browser performance mark records completion of initial history loading. Use a release build and record hardware, cache state and whether debugging is enabled when reporting numbers.

```sh
pnpm build                 # UI type check + production assets
pnpm test:core             # core/provider/storage tests
cargo test --workspace     # includes native host checks
pnpm desktop:build         # native executable, no installer
pnpm dev                   # browser UI preview, no model connection
```

Use `rtk` as a prefix for shell commands when working under the supplied workspace instructions. Node is development tooling; it is not bundled as an application sidecar. Supported design targets are Windows, macOS and Linux; check the [acceptance record](docs/ACCEPTANCE.md) for what was actually verified.

## Design and iteration

- [Architecture](docs/ARCHITECTURE.md): boundaries, plugin ports, resource limits and tradeoffs.
- [UI contract](docs/UI.md): one shared visual style and interaction vocabulary.
- [Roadmap](docs/ROADMAP.md): small bricks with acceptance gates.
- [Research](docs/research/architecture-options.md): official sources and alternatives.
- [Desktop API](docs/API.md): IPC and outgoing provider contract.

The core lives in `crates/dolores-core`; plugins in `crates/dolores-provider-openai` and `crates/dolores-store-sqlite`; desktop assembly in `src-tauri`; UI in `src`. New provider/storage implementations should implement the core traits and be explicitly registered in the desktop host. Do not present a compiled plugin as sandboxed.
