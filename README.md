# Dolores

**Skills:** add standard `.agents/skills/<name>/SKILL.md` files to a working folder, or `~/.agents/skills/<name>/SKILL.md` for global skills. Open Skills, choose Project or Global, review the exact text, then Activate skill. Global snapshots apply to all chats, including side chats; an active project skill overrides a global skill with the same name. Both scopes support Disable, reviewed rollback and Forget. File changes take effect only after review; tools retain separate approval. See [skill design](docs/design/project-skills.md).

History browsing keeps one conversation-list page and one message page in memory. Use Older/Newer to reach all saved chats and messages, or Latest to return to the newest messages. The conversation header exports complete Markdown or JSON to a new file through the native Save dialog. Drafts and scroll positions restore for the 20 most recently visited views during the current launch. [UI.md](docs/UI.md) is the shared visual contract, backed by Flutter's `lib/theme.dart`; system light/dark mode remains the default.

A lightweight desktop agent harness, inspired by the gradual awakening of Dolores in *Westworld*. Start with conversation; add tools, memory, reusable skills and evaluation one brick at a time.

**Current bricks:** Flutter desktop, Rust core, system theme, OpenAI-compatible streaming chat, cancellation, SQLite history and replaceable provider/storage/credential interfaces. Remember connections using the OS credential store, restore them after restart, retry recovery, or forget them while keeping conversations. Automatic preference learning is available; reusable skills and external plugin loading remain on the roadmap.

Flutter is selected as the default after the visual/resource comparison. The earlier Windows observation was 224.36 MiB working set and 229.52 MiB private allocation; it exceeds the provisional 150 MiB working-set target. Refinement now proceeds in Flutter, with low-end performance and cross-platform release acceptance still open. Iced and Tauri remain optional comparison shells. See [acceptance](docs/ACCEPTANCE.md).

The [Flutter desktop](apps/dolores_flutter/README.md) uses the Rust core/plugins through a bundled C ABI. Keep the entire generated release directory together.

## Run

Working chats stream replies with tools enabled. Public commentary before a tool call stays visible at approval and is saved in expandable **Agent progress** separately from the final answer. Stop and interrupted arguments never publish a partial turn. Progress is preserved in chat, trajectory and exports. Your provider must support streaming Chat Completions function calls; use Side chat explicitly for models without tools.

Install Rust stable, Flutter 3.47.5 and its [desktop prerequisites](https://docs.flutter.dev/platform-integration/desktop). Linux secure storage also needs the D-Bus development library and a running Secret Service. On Windows, build directly from PowerShell:

```powershell
rtk proxy powershell -NoProfile -File scripts/build-flutter.ps1 -FlutterSdk C:/path/to/flutter
./apps/dolores_flutter/build/windows/x64/runner/Release/dolores_flutter.exe
```

Open Model connection, enter the API base URL and a key if required, then **Fetch models**. Search and check the models you want available, save, and choose the active model beside the message box. Manual entry is available for servers that do not list models. Local example: `http://localhost:11434/v1` with a model already installed on your server. Hosted example: your provider's HTTPS API prefix. Dolores does not download or launch a local model.

Enable **Remember connection** to keep the key in Windows Credential Manager, macOS Keychain or Linux Secret Service. Disable it to use the connection until the app closes. A blank key field keeps the current/saved key for the same endpoint; **Use without a key** explicitly clears it on save. Switching models preserves the conversation, draft and key and is disabled during a response. Your choices and last active model survive restart, with the connection restored only when remembered. No key is returned to the UI or stored in SQLite. If recovery fails, history remains available and the app offers retry or reconnection. **Forget saved connection** removes the stored key and connection while keeping history.

**Request settings** controls the output token limit (default 2048) and whole-response timeout (default 180 seconds). Changes apply to the next message and survive restart. Failed responses restore your draft and offer manual recovery actions; **Retry message** sends your current edited draft. Completed reply details retain the settings used for that reply.

Open **Memory** in the sidebar to inspect/edit/disable/delete preferences. **Learn preferences automatically** is enabled: after a saved reply, eligible explicit work/response preferences are learned from your message, preserving exact wording and source evidence. Working chats save in **This working folder**; side chats use **All chats**. Explicit corrections can replace untouched learned entries; editing/disabling protects them. Turn the switch off for manual review, then use **New preference** or **Suggest from this chat**. Eligible learning may use one extra tool-free request (10 seconds / 512 output tokens or lower configured limits), with separate usage/activity shown in Memory. Ordinary tasks and common sensitive/quoted patterns skip it; filtering and topic selection remain limited. Enabled entries use bounded folder-first retrieval, visible through the context ring. Preferences/source quotes are local plaintext and can remain in earlier reply provenance/exports after deletion. See [automatic memory](docs/design/automatic-memory.md) and [reviewed suggestions](docs/design/memory-suggestions.md).

Use **Session summary** in a saved chat’s header to review the next conversation batch, generate a draft, correct it and explicitly **Save summary**. Future messages use that summary plus recent uncovered turns; your full conversation remains in history and exports. Extend it with later turns, edit or delete it at any time. It is local plaintext used only in that chat; generated summaries can omit or distort details. Summaries still require review alongside automatic preferences. See [summary design](docs/design/session-summaries.md).

Start with **Open project…** to choose a working folder, or send a message in a **Temporary workspace** and Dolores creates a separate folder for that chat. The sidebar groups project chats under collapsible Projects and temporary/side chats under Recents. New chat starts a temporary workspace; its menu offers Side chat, and each project has a + action for a new chat in that folder. Working folders are saved per conversation and restored after restart. A temporary workspace is app-managed, not an automatically erased conversation. Existing conversations are preserved as side chats. The header identifies the current mode and lets you show/copy its working-folder path.

Project and temporary chats include folder tools automatically. Ask the model to read a relative text-file path; each valid operation offers **Allow once** or **Deny** before information is shared. Approved text is saved with the completed reply and included in exports. Inspect it in expandable chat/trajectory cards. Tool runs allow at most four model calls and four total tool operations, with bounded text and time. **Side chat** is the explicit streaming conversation without file access. Your provider/model must support Chat Completions function calls for working chats; the configured Qwen/Qwen3.5-2B passed specific synthetic live flows, without implying universal compatibility. The file tools stay inside the working folder: open a file’s containing folder as a project to use them. Separately reviewed commands have broader user-account access as described below. Deleting a chat removes its saved conversation and folder association, never project files or temporary-folder contents.

For a small existing text file, ask for an exact replacement. Dolores shows a local diff and **Apply once**/**Deny**. It rechecks the file before applying; a changed file needs a fresh preview. The reviewed diff is retained with a successful reply and exports. Applied file changes remain if the later reply stops or fails. The header’s **Changes** view retains local before/after snapshots independently of conversation history. Select a record, **Review revert**, then **Revert once** to restore it; changed files are refused. A failed receipt is labeled **Needs check** rather than assumed applied. Records survive deleting the chat and can be opened from another chat in the same folder. Older edits have no invented snapshots. See [journal design](docs/design/change-journal.md). Files and diffs are limited to 16 KiB, with one unique match per edit. See [edit design](docs/design/approved-edits.md) for the concurrency/platform boundary.

Ask for a small new text file in an existing working-folder directory. Dolores shows the complete addition and **Create once**/**Deny**, refuses an occupied path, and saves explicit file existence in Changes, including empty files. Select its record, **Review revert**, then **Remove once** to remove that created file only if it still matches. Removal needs its own decision and cannot be reversed here. Creation fits the 4-KiB JSON argument budget; its file/diff cap is 16 KiB. Publication requires a filesystem supporting hard links and never falls back to overwrite. Directory creation and arbitrary model deletion are unavailable through the file tools. See [creation design](docs/design/approved-creation.md).

Working chats can now validate work with **run_command**. Review the working folder, resolved executable and each literal argument, then choose **Run once** or **Deny**. Initial programs are installed direct executables for git, node, python, python3, cargo, rustc and dart; shell/batch expansion is unavailable. Each command has a 30-second deadline and 8-KiB combined output cap. Results retain stdout, stderr, exit code and any shortened-output label in chat/trajectory and exports. Commands run with your account permissions and can access/change files outside the folder or use the network. Their changes are outside the file-change journal and may remain after Stop or failure. See [command design](docs/design/approved-commands.md).

The model can also list a folder or search for literal text to locate a file. Each operation asks separately: a listing shares names, a search scans bounded text files and shares matching snippets, and reading a whole file needs another decision. Searches are case-sensitive and capped by depth, entries, files and bytes; cards label partial results. For example, ask “Find the note containing `meeting agenda` in this folder and read it.” Discovery skips links and common secret/generated paths; inspect each request because names cannot identify every secret. Names, queries and snippets are also retained with successful replies and exports. No background index is created.

Preferences, completed conversations and local file-change snapshots are stored unencrypted in `dolores.db` under the OS application data directory for `dev.dolores.desktop`. Open one shell at a time against a data directory. Remembered keys are scoped to that directory; a copied database does not restore its key elsewhere. Model requests include bounded history and go to your configured endpoint. Saving settings makes no network request. Enter sends, Shift+Enter inserts a newline; replies are selectable and have Copy actions.

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

The core lives in `crates/dolores-core`; plugins in `crates/dolores-provider-openai`, `crates/dolores-store-sqlite`, `crates/dolores-credentials` and `crates/dolores-tools-fs`; default host/UI in `crates/dolores-flutter-bridge` and `apps/dolores_flutter`. Alternatives live in `crates/dolores-native`, `src-tauri` and `src`. New plugins implement core traits and are explicitly registered. Compiled native plugins are not sandboxed. `FLUTTER_SDK`, the ignored local SDK or PATH supplies Flutter to the development helper; Node is never bundled.

For Flutter checks, use `flutter analyze` and `flutter test` in the app directory. With the fixture server running, build with `scripts/build-flutter.ps1 -Smoke`, then run `scripts/test-flutter.ps1` against a fresh output directory. Build with `-RestartSmoke`, then run `scripts/test-connection-restart.ps1` for separate-process recovery using a generated test key, which is deleted afterward. Rebuild without diagnostic switches before normal usage or resource measurement. See the [bridge contract](docs/flutter/flutter-api.md) for recovery and cross-store cleanup limits.

For native controller/renderer checks, start `pnpm demo:server`, build `cargo build -p dolores-native --release --features smoke --locked`, then run `powershell -NoProfile -File scripts/test-native.ps1`. The runner creates isolated fixture data and four screenshots, then closes its app. It drives controller events and real HTTP; pointer, keyboard and IME interaction remain separate UAT. Rebuild without `--features smoke` for normal usage and resource measurements.
