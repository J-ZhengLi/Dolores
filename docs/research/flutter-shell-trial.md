# Flutter desktop trial — 2026-10-01

Historical trial: built a runnable Flutter candidate to compare visual refinement and resource costs with Tauri and Iced. **Subsequent decision, 2026-10-01:** the user selected Flutter as the default; brick 2.1 now adds secure remembered connections. The measurements below describe the earlier trial artifacts, not a new resource certification.

## Implementation

Flutter 3.47.5 (framework `6a19cca56475dbfba1478ee68d7bd0c2ef891da1`), Dart 3.13.4. A Dart worker isolate calls a small Rust C ABI, loading the bundled `dolores_flutter_bridge` library. The bridge reuses the existing core, OpenAI-compatible provider and SQLite store. There is no webview, Node sidecar, network server or resident model.

The UI uses the Tauri palette, sidebar width and compact typography, with a rounded composer, selectable replies, Copy, system light/dark theme, connection dialog and narrow drawer. Actual renderer captures were reviewed at 1120×780 and 620×700 logical pixels. Two widget tests exercise keyboard Send/newline/Stop and the narrow drawer/form. Six controller smoke checks cover real loopback HTTP, Unicode, cancellation, provider denial, interrupted streams, complete replies and stored history. These do not establish real OS input/IME or accessibility acceptance.

The current Windows SDK and C++ compiler were already installed. Visual Studio's optional CMake component was absent; an official portable CMake 4.4.3 built the generated Flutter project. Flutter itself was not patched and no system installer was changed. The build helper first asks Flutter to write configuration, then uses the same generated CMake/assembly build with the portable executable. SDKs live in ignored `output/toolchains` and are not shipped.

## Paired normal-release observation

Same two-turn SQLite fixture, one launch per app, visible unminimized windows, 1120×780 logical client area at 144 DPI. Eight seconds of settling, then five process-tree samples. A Windows development machine. Flutter engine reported Impeller OpenGLESSDF. No smoke runner or browser debugging was included in these measurements.

| Metric | Iced / software | Tauri / WebView2 | Flutter / Rust FFI |
| --- | --- | --- | --- |
| Working set, mean | 29.60 MiB | 427.47 MiB | 224.36 MiB |
| Private bytes, mean | 13.81 MiB | 197.71 MiB | 229.52 MiB |
| Processes | 1 | 7 | 1 |
| Runtime bundle, uncompressed | 8.64 MiB | 12.41 MiB | 31.13 MiB |
| Short idle CPU, percent of one logical core | 0.00% | 1.19% | 0.00% |

Flutter has a smaller resident working set than Tauri in this observation, while its private committed allocation is higher. Iced remains much smaller by both measures. Rounded zero CPU values mean no measured processor-time increase in these short sample intervals; they do not prove zero work. Working-set sums can count shared pages more than once. GPU allocations and whole-system memory changes were not measured. Installed system runtimes are excluded for all shells; Flutter's engine DLL, Rust DLL, AOT library, ICU data and assets are included. Its 90,624-byte launcher alone is not a valid application-size comparison.

Source evidence: private local benchmark report (excluded from Git). Reproduce with `scripts/build-flutter.ps1`, `scripts/test-flutter.ps1` and `scripts/compare-shells.ps1`; rebuild without `-Smoke` before measuring. Window checks disable the measurement thread's Windows DPI virtualization and assert visible, unminimized client dimensions.

## Decision boundary

Flutter demonstrates visual refinement with direct Rust plugin reuse. It adds a Dart layer and an engine distribution; it uses more idle memory than Iced in this trial. The user selected it after review. Neither that decision nor these short measurements establish low-end performance, cold/warm startup, sustained scrolling, screen-reader behavior or cross-platform acceptance. macOS/Linux runners and CI definitions are supplied but have not been executed locally.

## Primary references

- [Flutter Windows integration](https://docs.flutter.dev/platform-integration/windows/building): C++ host, desktop build and complete runtime distribution.
- [Dart C interop](https://dart.dev/interop/c-interop): direct dynamic-library calls and native allocation ownership.
- [Flutter native code binding](https://docs.flutter.dev/platform-integration/bind-native-code): native libraries through FFI.
- [Flutter SDK archive](https://docs.flutter.dev/install/archive): stable SDK provenance and versions.
- [CMake Visual Studio 18 2026 generator](https://cmake.org/cmake/help/v4.2/generator/Visual%20Studio%2018%202026.html) and [official CMake releases](https://github.com/Kitware/CMake/releases): portable toolchain source.
