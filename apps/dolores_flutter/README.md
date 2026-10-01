# Dolores Flutter desktop

Flutter 3.47.5 / Dart 3.13.4 is the selected desktop UI, sharing Dolores's Rust core, provider, SQLite and OS credential plugins through a small bundled C ABI. No model, Node server or webview is shipped. From the root, `pnpm desktop` builds and opens the app; `pnpm desktop:build` creates the release bundle. The helper uses `FLUTTER_SDK`, the ignored local SDK or PATH. Windows users can also build without Node:

From the repository root on Windows:

```powershell
rtk proxy powershell -NoProfile -File scripts/build-flutter.ps1 -FlutterSdk C:/path/to/flutter
```

If the existing MSVC/Windows SDK installation lacks Visual Studio's CMake component, use an official portable CMake supporting your Visual Studio generator, without modifying Flutter or the system install:

```powershell
rtk proxy powershell -NoProfile -File scripts/build-flutter.ps1 -CMake D:/path/to/cmake/bin/cmake.exe
```

Run `build/windows/x64/runner/Release/dolores_flutter.exe`. Copy **the whole Release folder** to distribute it, including the Rust DLL, Flutter DLL, `data/` and assets; the tiny launcher executable is not the application size. The Microsoft VC runtime is also required on destination machines. Use `--compact` to preview a 620×700 client area. The normal window starts at 1120×780 logical pixels.

Tests and static checks (from this directory, with Flutter available):

```text
rtk proxy flutter analyze
rtk proxy flutter test
```

For the real HTTP/FFI/SQLite controller smoke test, start the root `scripts/mock-provider.mjs`, build with `scripts/build-flutter.ps1 -Smoke`, then run `scripts/test-flutter.ps1` with a fresh output directory. Add `-Compact` for the narrow-layout screenshot. Rebuild **without** `-Smoke` before measuring or using the app normally. Only the diagnostic entry point reads `DOLORES_SMOKE_DIR`. The smoke runner does not exercise real OS keyboard/IME/pointer interactions.

macOS/Linux runners are scaffolded, not locally verified. Build the Rust bridge for that host, then `flutter build macos --release` or `flutter build linux --release`. Copy `libdolores_flutter_bridge.dylib` into the macOS app's `Contents/Frameworks/`, or `libdolores_flutter_bridge.so` into the Linux bundle's `lib/` directory. macOS distribution must sign the complete bundle after adding the library. CI build definitions provide these copy steps but are not evidence of executed builds.

Model connection now fetches a searchable model list and saves an enabled subset. The picker beside the composer changes the active model without losing the draft or conversation. Manual IDs remain available when listing is unsupported. Choices persist without persisting launch-only keys.

Contract: [Flutter bridge](../../docs/flutter/flutter-api.md). Theme follows the system. Remember connection uses the OS credential store, never SQLite, and restores after restart. Without remembering, the connection lasts for this launch. Retry handles an unlocked store; Forget removes the key/connection while keeping conversations.

For separate-process restart checks with a generated test key, build with `scripts/build-flutter.ps1 -RestartSmoke`, start the fixture server, then run `scripts/test-connection-restart.ps1` using a fresh directory. It verifies restored authorization, preserved history, forgetting and key absence from data files. Rebuild without either diagnostic switch afterward. Native vault tests are opt-in: `cargo test -p dolores-credentials -- --ignored` creates and deletes its own generated entry only.
