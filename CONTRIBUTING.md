# Contributing to Dolores

Start with `AGENTS.md` and the affected code. Read only the matching documentation
from [the index](docs/README.md). Product behavior belongs in the user guide,
invariants/rationale in the owning feature specification, shared appearance in
UI.md, and active scope in the selected plan. Report routine verification in the
task response/CI; update qualification documentation when its boundary changes.
The documentation check enforces compact entry-point budgets and keeps dated
test diaries out of the current acceptance index.

## Prerequisites and normal Windows build

Use Rust stable MSVC, Python 3.11+, Visual Studio C++ desktop
tools/Windows SDK, CMake and a Flutter SDK compatible with the app's pubspec.
Flutter 3.47.5 / Dart 3.13.4 is the currently recorded build baseline.
Node is optional tooling for the browser adapter; its dependencies and lockfile
stay under `adapters/browser/`, not the Flutter application.

From the repository root:

```powershell
python scripts/desktop.py build --flutter-sdk C:/path/to/flutter
python scripts/desktop.py launch
```

The helper finds the SDK through `--flutter-sdk`, `FLUTTER_SDK`, ignored
`output/toolchains/flutter`, or PATH. It builds/copies the Rust bridge and plugins
and assembles the normal release. Use `--cmake` / `--generator` for the installed
Windows toolchain. It does not patch Flutter or require Developer Mode.

Keep the complete `apps/dolores_flutter/build/windows/x64/runner/Release/` bundle.
Serialize builds sharing these artifacts. Close only an owned preview before
rebuilding locked files. For isolated integration, `launch --data-directory`
uses a fresh absolute directory under ignored `output/`. `--pid-file` with
`--replace-owned` validates PID, executable and creation time before replacement.
Preserve real profiles and matching private symbols in `output/release-symbols/`.

## Checks and recovery

Choose checks for the changed surface; stop after the relevant checks pass unless
another change or unresolved failure justifies repetition. Keep successful output
concise and inspect failure detail from logs.

| Change | Relevant verification |
| --- | --- |
| Documentation/workflow | `python scripts/check-docs.py`; `python scripts/test-docs.py` when changing doc checks |
| Rust logic | Focused crate/tests, formatting and relevant Clippy targets |
| Flutter logic | Focused `flutter test --no-pub`; `flutter analyze --no-pub` |
| Appearance/motion | Saved affected Light/Dark/compact renders; relevant interaction/recovery tests |
| Native integration/packaging | Normal helper build and the matching isolated native journey |
| Release | [Release procedure](docs/RELEASING.md), normal build and installer verification |

Rust commands run from the root; Flutter commands run in `apps/dolores_flutter/`.
After a normal build prepares dependencies, no provider credentials are needed for
deterministic tests. Add realistic failure/recovery coverage when applicable.
Distinguish fixture, physical-input, performance and live-model qualification.

For credential-free rendered controls connected to the Rust bridge:

```powershell
python scripts/qualify-native-ui.py --help
python scripts/qualify-native-ui.py --case local
python scripts/run-regressions.py --help
```

Select the relevant case from the helper rather than loading every journey's
history. `--native-only` checks reuse a build and make no source-freshness claim.
Use isolated public data; private transcripts, keys and raw captures stay ignored.
Optional live checks explicitly select a suitable available model and bound usage.

## Windows portable preview

After a normal build, collect matching dependency notices and package into fresh
directories with `scripts/collect-windows-notices.py` and
`scripts/package-windows.py` (see `--help`). Packaging refuses missing/stale
notices, unexpected runtime files and existing output. It verifies integrity and
notice binding, not publisher identity. Destination hosts need the Microsoft C++
x64 runtime. See [dependency notices](docs/DEPENDENCIES.md) and the
[portable contract](docs/design/windows-portable.md).

## Cross-platform and diagnostics

Windows x64 is the supported release target; macOS/Linux definitions await native
qualification. Diagnostic entry points and test fixtures are not the normal app.
Measure release startup/resources with `python scripts/measure-desktop.py --help`,
recording host/cache/window conditions. Preserve failed results and original gates.

The infinity logo source is `assets/dolores.svg`. Icon regeneration uses the
development-only Pillow dependency and `scripts/generate-icons.py --check`; normal
builds do not need Pillow. Keep upstream icon licensing.

Before committing, run `git diff --check` and `python scripts/check-publication.py`.
Use explicit file staging and English commit messages. Update the changelog only
for user-visible behavior changes. Push/publish only when requested.
