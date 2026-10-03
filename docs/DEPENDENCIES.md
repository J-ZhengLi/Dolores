# Windows dependency notices

The portable preview includes **THIRD-PARTY-NOTICES.txt**, **DEPENDENCIES.json**, the Dolores MIT license and Flutter's existing compressed `data/flutter_assets/NOTICES.Z`. These describe the selected Windows Flutter/Rust app. Iced/Tauri/Node development tooling, installed model servers and external MCP servers are outside this bundle and need their own distribution review.

## Reproduce the inventory

Build the normal app, then follow the [contributor packaging commands](../CONTRIBUTING.md#windows-portable-preview). Collection reads the locally resolved locked Cargo/pub packages and the matching Flutter SDK. It performs no network request or dependency upgrade. Keep the generated output out of Git; it is included in the ZIP. The public inventory contains package names, versions, license declarations, upstream source links and notice hashes, never package-cache paths or builder identity.

The Rust selection starts at `dolores-flutter-bridge`, uses Cargo's Windows x64 graph, excludes dev edges and conservatively retains normal/build/proc-macro dependencies. The Dart selection follows direct production dependencies and their closure; federated file-selector packages for unused platforms may remain. This is a notice superset, not an exact binary-link map. Engine/standard-library consolidated notices also contain unused components. No package count is a memory or shipped-code metric.

## Reviewed cases

| Component | Notice handling |
| --- | --- |
| Rust crates | Full local LICENSE/COPYING/COPYRIGHT/NOTICE texts, including nested/vendor notices; declared SPDX expression and versioned crate/source-archive links. Missing declarations/text refuse. |
| `ring` and Rust TLS dependencies | BoringSSL, ISC/MIT/Apache and fiat notices are retained separately rather than assuming all native code has the wrapper's license. |
| `libsqlite3-sys` / SQLite | Wrapper/vendor licenses plus the actual bundled SQLite 3.46.0 copyright-disclaimer header. Unused SQLCipher notice is retained conservatively. |
| `option-ext` 0.2.0 (MPL-2.0) | Full MPL license, versioned upstream source URL and corresponding source files included in the text collection. Cached crate hash must match Cargo.lock; extracted source must match that archive. Altered sources refuse pending a new review. This can be an unused target dependency in the conservative graph. See [Mozilla's distribution guidance](https://www.mozilla.org/en-US/MPL/2.0/FAQ/#q8-i-want-to-distribute-outside-my-organization-executable-programs-or-libraries-that-i-have-compiled-from-someone-elses-unchanged-mpl-licensed-source-code-either-standalone-or-part-of-a-larger-work-what-do-i-have-to-do). |
| Flutter/Dart and engine | Package licenses, repository SDK license and the full engine `sky_engine/LICENSE` tied to the Windows engine's cached revision; existing generated NOTICES.Z stays bundled. |
| Rust standard library | Installed compiler version, its complete library copyright inventory and accompanying license texts. Compiler and documentation tools are not bundled. |
| Material Icons and infinity artwork | SDK font's CC BY 4.0 text and Google attribution, plus upstream SVG Apache 2.0 license. The original rounded font outline was converted/resized/rasterized for platform icons. |

## Integrity and release boundary

The inventory is bound to every reviewed runtime file, Cargo.lock, pubspec.lock, pubspec.yaml and the artwork attribution. The packager refuses changed/truncated notices or mismatched inputs before creating the destination. ZIP verification checks file hashes and the notice/runtime binding again. This detects stale/corrupt artifacts; it is not a signature, exact linkage proof or a blanket legal-compliance certification. Changing dependencies, native build flags or artwork requires reviewing scope and obligations again.

The launcher checks required app files and the three Visual C++ runtime files used by the current EXE/plugin imports. It never copies system DLLs or installs prerequisites. Runtime version/loadability, clean-machine execution, signing and macOS/Linux distributions remain separate gates. Follow [Flutter's ZIP deployment guidance](https://docs.flutter.dev/platform-integration/windows/building#building-your-own-zip-file-for-windows) and [Microsoft's runtime guidance](https://learn.microsoft.com/en-us/cpp/windows/latest-supported-vc-redist?view=msvc-170).
