# Contributing to Dolores

The default desktop app is Flutter with a bundled Rust bridge. Start with [architecture](docs/ARCHITECTURE.md), [UI contract](docs/UI.md) and repository `AGENTS.md`. Keep scope small, preserve unrelated edits, record failure/recovery checks in [acceptance](docs/ACCEPTANCE.md), and commit each completed brick. Generated output, credentials and private transcripts stay out of Git.

## Prerequisites and normal Windows build

- Rust stable, including rustfmt and Clippy.
- Flutter **3.47.5 / Dart 3.13.4**, matching the app constraints.
- Visual Studio C++ desktop tools, Windows SDK and CMake; see [Windows setup](https://docs.flutter.dev/platform-integration/windows/setup).
- Python **3.11+** for verification/packaging scripts.

Node is optional development tooling, absent from the Flutter app. The pnpm helper/alternative shells need Node ≥22.12 (24 recommended), pnpm 11 and `pnpm install --frozen-lockfile`.

From the repository root:

```powershell
powershell -NoProfile -File scripts/build-flutter.ps1 -FlutterSdk C:/path/to/flutter
powershell -NoProfile -File scripts/launch-flutter.ps1
```

The helper uses `-FlutterSdk`, `FLUTTER_SDK`, ignored `output/toolchains/flutter`, or PATH. It builds/copies the Rust bridge, prepares plugin junctions without symlink privileges and assembles the normal release. If Visual Studio lacks CMake, pass `-CMake C:/path/to/cmake.exe` and, when needed, the installed `-Generator`. It does not enable Developer Mode or patch Flutter.

Close your owned preview before rebuilding locked files. Keep the complete `apps/dolores_flutter/build/windows/x64/runner/Release/` directory. For an isolated launch, pass `-DataDirectory` with an absolute ignored output directory; this does not migrate another directory's vault keys/history.

Release builds remap Rust source paths and keep Flutter debug symbols under a fresh ignored `output/release-symbols/` directory. Retain matching symbols privately when investigating crashes. The helper temporarily aliases Flutter's generated registrant to a stable package URI and restores the generated package configuration afterward. See [build privacy details](docs/design/windows-portable.md).

## Checks and recovery

```powershell
python -I -B scripts/check-docs.py
python -I -B scripts/run-regressions.py --help
python -I -B scripts/run-regressions.py --flutter-sdk C:/path/to/flutter
```

The Windows runner builds/checks the normal app and runs eight isolated save/restart fixture pairs. Logs/reports stay under fresh `output/regressions/` data. It stops on failure; inspect the log, fix the cause and explicitly start a new run. `--case` selects pairs; `--native-only` skips build/static/widgets and makes no freshness claim. See [scope](docs/design/regression-runner.md).

For focused changes, run relevant Rust/Flutter tests. After the normal build prepares dependencies, run `flutter analyze --no-pub` and `flutter test --no-pub` in the app directory. A fixture pass does not establish model competence. Authorized live probes use isolated data and bounded prompts, preserving user settings. Record real failures as failures.

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

The infinity logo source is `assets/dolores.svg`, the original rounded Material Icons outline used by Flutter. To regenerate platform assets, install development-only Pillow (`python -m pip install Pillow`), run `python -I -B scripts/generate-icons.py`, then `python -I -B scripts/generate-icons.py --check`. This updates platform icons and web SVG; Flutter keeps its original `Icons.all_inclusive_rounded`. Preserve the upstream icon license. Normal builds do not need Pillow.

`.github/workflows/ci.yml` defines Windows/macOS/Linux builds; local Windows work does not verify the other targets. Linux needs GTK/toolchain and D-Bus development packages, plus a running Secret Service for credentials. macOS needs its desktop toolchain and signing after adding the Rust library.

The optional `pnpm desktop:build` helper assembles Flutter for the current host. The bridge belongs beside the Windows EXE, in Linux `lib/`, or macOS `Contents/Frameworks/`. See [Flutter module notes](apps/dolores_flutter/README.md). Iced/Tauri remain comparisons: `pnpm desktop:iced:build` and `pnpm desktop:web:build`; they do not have every Flutter feature. `pnpm build` checks Svelte and `pnpm dev` previews it without a model connection.

For diagnostics, start `pnpm demo:server`, use the matching `-Smoke`, `-RestartSmoke` or `-HistorySmoke` build switch and `scripts/test-flutter.ps1`, `scripts/test-connection-restart.ps1` or `scripts/test-history.ps1`. Data/generated keys are isolated. Rebuild with **no diagnostic switch** afterward. Controller screenshots are not native pointer/keyboard/IME UAT. Native vault tests are opt-in and create/delete their own entry.

Measure normal release builds with `scripts/measure-runtime.ps1 -AppProcessId <pid>`, recording hardware, cache state, DPI/window size and startup separately. Shared working-set pages can be counted repeatedly. Keep raw results ignored; publish aggregate observations with their boundary.
