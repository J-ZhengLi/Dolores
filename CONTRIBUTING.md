# Contributing to Dolores

The default desktop app is Flutter with a bundled Rust bridge. Start with [architecture](docs/ARCHITECTURE.md), [UI contract](docs/UI.md) and repository `AGENTS.md`. Keep scope small, preserve unrelated edits, record failure/recovery checks in [acceptance](docs/ACCEPTANCE.md), and commit each completed brick. Generated output, credentials and private transcripts stay out of Git.

## Prerequisites and normal Windows build

- Rust stable, including rustfmt and Clippy.
- Flutter **3.47.5 / Dart 3.13.4**, matching the app constraints.
- Visual Studio C++ desktop tools, Windows SDK and CMake; see [Windows setup](https://docs.flutter.dev/platform-integration/windows/setup).
- Python **3.11+** for verification/packaging scripts.

Node is optional development tooling, absent from the Flutter app. Browser fixtures and the optional browser adapter need Node ≥22.12 (24 recommended). Build and launch use Python directly; no root npm/pnpm install is needed. Browser dependencies and their npm lockfile belong under `adapters/browser/`.

From the repository root:

```text
python scripts/desktop.py build --flutter-sdk C:/path/to/flutter
python scripts/desktop.py launch
```

The helper uses `--flutter-sdk`, `FLUTTER_SDK`, ignored `output/toolchains/flutter`, or PATH. It builds/copies the Rust bridge, prepares plugin junctions without symlink privileges and assembles the normal release. If Visual Studio lacks CMake, pass `--cmake C:/path/to/cmake.exe` and, when needed, the installed `--generator`. It does not enable Developer Mode or patch Flutter.

Close your owned preview before rebuilding locked files. Keep the complete `apps/dolores_flutter/build/windows/x64/runner/Release/` directory. For an isolated launch, pass `--data-directory` with an absolute ignored output directory; this does not migrate another directory's vault keys/history. `launch --pid-file <absolute-output-record> --replace-owned` replaces only a preview whose PID, executable and creation time match. Other processes are retained. Window presence is checked separately from native visual verification.

Release builds remap Rust source paths and keep Flutter debug symbols under a fresh ignored `output/release-symbols/` directory. Retain matching symbols privately when investigating crashes. The helper temporarily aliases Flutter's generated registrant to a stable package URI and restores the generated package configuration afterward. See [build privacy details](docs/design/windows-portable.md).

## Checks and recovery

```powershell
python -I -B scripts/check-docs.py
python -I -B scripts/run-regressions.py --help
python -I -B scripts/run-regressions.py --flutter-sdk C:/path/to/flutter
```

The Windows runner builds/checks the normal app and runs eight isolated save/restart fixture pairs. Logs/reports stay under fresh `output/regressions/` data. It stops on failure; inspect the log, fix the cause and explicitly start a new run. `--case` selects pairs; `--native-only` skips build/static/widgets and makes no freshness claim. See [scope](docs/design/regression-runner.md).

For focused changes, run relevant Rust/Flutter tests. After the normal build prepares dependencies, run `flutter analyze --no-pub` and `flutter test --no-pub` in the app directory. A fixture pass does not establish model competence. Authorized live probes use isolated data and bounded prompts, preserving user settings. Record real failures as failures.

For rendered controls connected to the real Rust bridge, run
`python scripts/qualify-native-ui.py` after a normal Windows build. It uses a
fresh `output/native-ui/` profile, a public loopback response and no credentials.
The initial case covers workspace instruction review/stale-source recovery and
Memory Off/Forget and conversation deletion with retained files/other chats. Logs and a receipt remain in
that profile. Ordinary widget runs skip this opt-in case. This does not qualify
native pointer/IME/dialog behavior or real-model reliability.

Async deadline tests should use a paused Tokio clock and establish the blocked
queue/provider phase before advancing time. Keep provider inactivity, legacy
fallback, delivery expiry and Stop assertions separate; do not race equal real
timers or infer completion from a short sleep. Both CI entry points run
`python scripts/check-rust-timing.py` after workspace tests: 20 repetitions of
the three bridge deadline cases with two concurrent processes, failing on any
failure or missing cases, without retry. For stronger local contention, use
`--repeat 100 --jobs 8`. This is a focused regression gate, not proof that all
other timing tests are deterministic.

Flutter controller tests should advance a controlled widget clock or await an
explicit completion signal before asserting restored state. A short real sleep
does not establish that Stop, polling or a save has completed. Verify pending
action refusal as well as recovery after the terminal event.

MCP inspection shares its real 30-second deadline across executable hashing,
server startup and protocol discovery. Bridge review/credential tests begin at
the reviewed-state boundary with small real files: changed-byte refusal, locked
storage, failed saving and explicit recovery do not depend on Node starting
within that deadline. The MCP crate retains real Node stdio coverage. Test
profiles optimize both SHA-256 and the MCP caller that instantiates its generic
digest code. Both CI workflows repeat the three bridge MCP cases eight times
with four concurrent processes, plus four real Node protocol cases with two
concurrent processes, using software SHA-256 so a developer CPU's acceleration
cannot hide that path.
Workspace tests use four threads on both hosted and local runs. No failed run is
retried automatically and production deadlines are unchanged.

Language-server tests observe the initialization allowance plus the request
allowance before their outer watchdog fires; a first request can validly exceed
ten seconds. Both remain 15 seconds in production. The repeat gate also runs
the four language cases twice in two concurrent processes, including delayed
initialization/response, a real request timeout, malformed output and explicit
restart recovery with unsaved Unicode text. Run
`python scripts/check-rust-timing.py --suite language --repeat 6 --jobs 2`
for focused local qualification. Missing cases and any failed repetition fail
the gate without retry.

## Run Windows CI locally

Use Python 3.13, Node 24, Rust 1.95.0 with rustfmt/Clippy and the pinned Flutter
checkout. These commands require no GitHub token, push or tag:

```powershell
python scripts/local-ci.py checks
python scripts/local-ci.py release --generator "Visual Studio 18 2026"
```

`checks` runs the shared Checks entry point. `release` runs full checks, the
normal build, native fixtures, verified payload, installer compilation and the
isolated uninstall fixture. Select your installed generator (GitHub uses Visual
Studio 17 2022); pass `--cmake` or `--flutter-sdk` when needed. Each run creates a
fresh directory under `output/local-ci/` with step logs, `RESULT.json` and, for a
successful release run, `pipeline/installer/` assets. It uses an isolated Python
environment for the pinned history-audit dependency and stops on any failure.
The tool restricts itself and children to four CPUs; use `--cpus 2` for stronger
contention. This runs CI on your Windows host with existing compiler caches; it
does not recreate the hosted Windows Server 2025 image or upload/publish assets.
Changing CPU affinity does not affect other applications.

Installer verification waits until the entire owned installation directory is
removed before reinstalling. Inno's first process can exit while its temporary
process still removes the old uninstaller; app-file removal alone is not a
completion signal. The fixture uses its unique current-user uninstall
registration and validates that executable stays inside its owned directory.
It requires the exact refusal exit code. Release verification also runs a
diagnostic small-payload `--rapid-reinstall` case to expose cleanup overlap on
fast hosts. That fixture installer is never a release asset. Native fixture
logs/results are retained as `dolores-installer-verification` Actions artifacts
even when verification fails; product installer upload/publication still requires
success.

For a focused MCP stress check, run
`python scripts/check-rust-timing.py --suite mcp --software-hash --repeat 16 --jobs 16`.
Use the shared pipeline for overall qualification; this filter checks only MCP.

`scripts/check-docs.py` checks local Markdown file/heading links, including design/research docs; it does not verify external URLs or facts. Keep product controls in the [user guide](docs/USER_GUIDE.md), build details here, invariants in `docs/design/`, evidence in ACCEPTANCE and future scope in ROADMAP. README should help a first-time reader.

## Windows portable preview

After the normal build:

```powershell
python -I -B scripts/test-package-windows.py
python -I -B scripts/test-windows-launcher.py
python -I -B scripts/collect-windows-notices.py --flutter-sdk C:/path/to/flutter --directory C:/path/to/repository/output/releases/notices
python -I -B scripts/package-windows.py --notices C:/path/to/repository/output/releases/notices --directory C:/path/to/repository/output/releases/preview
```

Choose fresh output. Notice collection uses locally prepared locked Rust/Dart dependencies and the matching Flutter SDK, without downloads. Python 3.11+ is required. It retains full notices, versioned source references, corresponding MPL source and a conservative dependency inventory. Packaging refuses missing/stale notice collections when the runtime or locked inputs differ. Recollect after rebuilding or changing dependencies. See [dependency scope](docs/DEPENDENCIES.md).

The packager accepts reviewed runtime paths, checks required files/x64 binaries, and refuses unexpected files/links, embedded local build paths or existing destinations. It adds user guidance, licenses, a startup helper and per-file hashes, verifies the ZIP, then writes a ZIP SHA-256 sidecar. It packages an existing build and does not claim source/build freshness. Rebuild the normal entry point beforehand.

`--verify C:/path/to/package.zip` checks integrity and notice/bundle binding, not publisher identity or runtime behavior. Destination machines need Microsoft's C++ x64 runtime; system DLLs are not copied. `Start-Dolores.cmd --check` checks file presence without launching. Signing, clean-machine and resource/input testing remain release gates. See [contract](docs/design/windows-portable.md).

## Cross-platform and diagnostics

The infinity logo source is `assets/dolores.svg`, the original rounded Material Icons outline used by Flutter. To regenerate platform assets, install development-only Pillow (`python -m pip install Pillow`), run `python -I -B scripts/generate-icons.py`, then `python -I -B scripts/generate-icons.py --check`. This updates maintained platform icons; Flutter keeps its original `Icons.all_inclusive_rounded`. Preserve the upstream icon license. Normal builds do not need Pillow.

GitHub Actions runs source/fixture checks on branch pushes and pull requests,
without compiling or packaging the native release. Tag/manual release runs build
and package **Windows x64** installers, retaining downloadable Actions artifacts;
tags create a draft GitHub release with reviewed notes/checksums. See
[releasing](docs/RELEASING.md), [installers](docs/design/installers.md) and
[changelog](CHANGELOG.md). macOS PKG/Linux DEB commands are experimental and require
native qualification before release CI. Linux needs GTK/toolchain, D-Bus and a
running Secret Service; macOS needs its desktop toolchain and signing. Retired
Iced and Tauri/Svelte shells are not maintained.

CI pins Flutter 3.47.5, Rust 1.95.0 and its actions. Its deterministic checks use no
model keys or private profiles. The SDK checkout includes release tags: a shallow
commit-only checkout can report `0.0.0-unknown` despite matching the pinned commit.
CI checks the SDK revision and version before compiling. After fetching missing
tags, use a fresh SDK checkout/cache and rerun rather than retaining unknown
version metadata. To run the same entry points locally, set
`FLUTTER_SDK`, install `.github/requirements-ci.txt`, then use
`python scripts/ci-windows.py check` for ordinary checks. For release qualification,
use `python scripts/ci-windows.py check --full --generator "<installed CMake generator>"`,
then `python scripts/ci-windows.py package`, `python scripts/setup-inno.py` and
`python scripts/installer.py windows --archive <verified-payload.zip> --directory
<fresh-absolute-output-directory>`. The ZIP is retained as an internal audited
installer payload. Fresh output is required. For a local retry,
pass the same `--directory <absolute-path-under-output>` to both commands, choosing
a new directory so the failed run's evidence is retained. Node 24 is needed
for fixtures; the optional browser adapter is installed separately and excluded
from the installer. The complete payload includes the native update launcher,
changelog and dependency notices. Signing and clean-machine acceptance remain open.

Optional live helpers require an explicitly chosen enabled `--model`; the tray
helper uses `--live-model` with `--live-source`. Memory/scheduling helpers accept
an optional `--secondary-model`; reasoning overrides are explicit. Existing
named-model qualification reports are historical observations, not contributor
requirements. RTK, CodeGraph and session handoffs are optional local tooling.

`python scripts/desktop.py build` assembles Flutter for the current host. The bridge belongs beside the Windows EXE, in Linux `lib/`, or macOS `Contents/Frameworks/`. See [Flutter module notes](apps/dolores_flutter/README.md). No alternative desktop shell is maintained.

Diagnostic entry points use `build --diagnostic smoke`, `restart` or `history`. Rebuild without that option before handing the app to a user or packaging it. Data/generated keys stay isolated. Diagnostics do not establish physical keyboard/IME acceptance. Native vault tests are opt-in and create/delete only their own entry.

After a normal build, `python scripts/test-history-evidence.py --directory
<fresh-absolute-path-under-output>` executes two reviewed public print commands
with a loopback provider, then checks their saved success/failure receipts in a
separate process. The follow-up must use history without repeating the command.
For an optional live follow-up, add `--source <original-profile-dolores.db>
--model <enabled-model-id>`. Only connection configuration is copied; credentials
stay in memory and original-profile table digests must remain unchanged. This is
background integration evidence, not native interaction qualification.

After a normal Windows build, `python scripts/test-scoped-settings.py` verifies actual HTTP allowances, scope inheritance, frozen runs and changed-window recovery against an isolated local fixture. It needs no model key and releases the child host before removing its temporary data. This does not replace bounded real-model task checks.

For executable recovery mods, run the following in separate processes after a
normal Windows build, replacing the example with a new absolute directory under
the repository's `output/` folder:

```powershell
python scripts/test-mods.py save --directory D:/path/to/project_dolores/output/mod-check
python scripts/test-mods.py reopen --directory D:/path/to/project_dolores/output/mod-check
```

The synthetic local provider needs no key and exercises output/Stop recovery,
activation, health rollback and durable restart. Never point this fixture at your
normal data directory. Live model competence is a separate acceptance check.

Measure normal release builds with `python scripts/measure-desktop.py --help`, recording hardware, cache state, DPI/window size and startup separately. Shared working-set pages can be counted repeatedly. Keep raw results ignored; publish aggregate observations with their boundary.
