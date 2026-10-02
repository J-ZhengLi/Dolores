# Dolores Flutter desktop

Saved-chat headers offer Skills with Project / Global selection. Review `.agents/skills/<name>/SKILL.md` for this folder or `~/.agents/skills/<name>/SKILL.md` for all chats, then Activate skill. Global skills also apply to side chats; active project skills override same-name global skills. Saved versions support Disable, reviewed rollback and Forget. Changes wait for activation, while reviewed snapshots stay active. Skills count toward context and cannot approve tools. Below 480 pixels, use Chat actions. See [design](../../docs/design/project-skills.md).

Draft from chat selects useful completed exchanges, generates editable skill instructions and runs 1–3 frozen response comparisons. Activation is explicit and requires all candidate checks to pass with more passes than baseline. Receipts retain exact tested requests/responses and usage; saved-version rollback keeps historical evidence. Tests are tool-free, check exact snippets and do not establish general task quality. Generated versions stay local without creating skill files. See [draft design](../../docs/design/skill-drafts.md).

Memory learns explicit work/response preferences after saved replies, enabled by default. Inspect its switch, evidence, latest activity and separate usage in Memory; manual edits/disables protect learned entries. Folder/All chats scopes and bounded context retrieval remain. Learning failure or Stop preserves the reply. See [automatic memory](../../docs/design/automatic-memory.md).

Working-folder headers offer Instructions: review the root `AGENTS.md`, then Enable instructions to share and save it for chats in that folder. Changes require re-review or Disable before sending. References are literal and every tool still needs its own approval. See the [design](../../docs/design/workspace-instructions.md).

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

Assistant replies render selectable Markdown with headings, lists, tables and inline code. Code blocks have a language label, Copy code and their own horizontal scrollbar. Whole-message Copy keeps the original Markdown. HTML stays literal, images show alt text without loading, and HTTP/HTTPS links offer destination inspection and Copy link. Rich layout falls back to complete plain text for large replies. The pinned native renderer is [flutter_markdown_plus](https://pub.dev/packages/flutter_markdown_plus); its behavior is constrained by `lib/reply_content.dart`, tested in `test/reply_content_test.dart`, and specified in [UI.md](../../docs/UI.md). The existing `-Smoke` runner also checks a rich reply through real HTTP streaming, storage and reload at wide/compact sizes.

The input box renders headings and fenced code as native editable blocks with hidden markers. Code cards have syntax colors and a language menu. Down Arrow on the last visual code line moves into the next block or creates a paragraph below the final card, closing incomplete fences when needed. Source and Add text controls are removed. Ordinary paragraphs retain literal inline Markdown; native drag selection remains per block. Ctrl/Command+A selects every block and highlights both focused and unfocused content. Copy/cut retains canonical Markdown, including hidden fences; Backspace/Delete, typing and paste replace the entire selected draft. Clicking or an unmodified arrow returns to local editing. Backspace removes an empty heading or code card, including its hidden markers, and works at an adjacent empty-block boundary; heading-start Backspace removes heading formatting while retaining its text. These structural edits participate in document undo. Active IME composition retains the focused native field until it commits. Enter inside code inserts a newline; Ctrl/Command+Enter sends. Native composition suppresses folding, block exits and sending. The source sent/stored remains Markdown, with block edits mapped to its body ranges. The selectively registered `highlight` 0.7.0 grammars and block layout have explicit plain-text fallbacks for large inputs. Heading/code transitions retain the native editor and input connection so typing continues without refocusing. Shift+Enter inserts visible newlines or opens prose after a heading; user-entered leading/trailing paragraph newlines stay visible. Composer tests exercise continuous text-input updates without refocusing, newline insertion/replacement, editing, pasting, language changes, undo/redo, wrapped-line navigation, keyboard exits, composition and source preservation; the native smoke runner captures both themes at wide/compact sizes. Real OS keyboard/IME interaction still needs UAT.

Contract: [Flutter bridge](../../docs/flutter/flutter-api.md). Theme follows the system. Remember connection uses the OS credential store, never SQLite, and restores after restart. Without remembering, the connection lasts for this launch. Retry handles an unlocked store; Forget removes the key/connection while keeping conversations.

Browse conversations and long transcripts with Older/Newer; Latest returns to the newest messages. Only one sidebar/transcript page stays in memory. The header's export button saves the entire selected conversation as Markdown or JSON through the native Save dialog. Choose a new filename; existing files are never replaced. Drafts and scroll positions restore for the 20 most recently visited views during this launch. Visual rules and shared Flutter tokens are recorded in [UI.md](../../docs/UI.md) and `lib/theme.dart`.

For the full-history Windows diagnostic, start the mock provider, build with `scripts/build-flutter.ps1 -HistorySmoke`, then run `scripts/test-history.ps1 -OutputDirectory output/history/check` from the root (add `-Compact` for narrow captures). Node 24's built-in SQLite is used only to seed isolated legacy fixture data: 137 chats and a 123-turn conversation. The release checks cover paging, restored view state, actual export files and sending from an earlier page. Rebuild with no diagnostic switch afterward. Native Save dialog interaction, OS keyboard/IME and other platforms still require UAT.

On Windows without symlink permission, the build helper creates directory junctions solely in generated project plugin directories using Flutter's generated dependency metadata. It does not change Windows settings, the SDK or the Pub cache. For direct Flutter commands after a symlink failure, run `scripts/flutter-plugin-junctions.ps1` from the root, then retry the command.

The build helper regenerates the Windows release asset stage so a cached, tree-shaken icon font cannot omit newly added UI icons when switching between normal and diagnostic entry points. Compiler caches and icon tree shaking remain enabled.

For separate-process restart checks with a generated test key, build with `scripts/build-flutter.ps1 -RestartSmoke`, start the fixture server, then run `scripts/test-connection-restart.ps1` using a fresh directory. It verifies restored authorization, preserved history, forgetting and key absence from data files. Rebuild without either diagnostic switch afterward. Native vault tests are opt-in: `cargo test -p dolores-credentials -- --ignored` creates and deletes its own generated entry only.

Completed replies show provider-reported token usage, with missing counters marked unavailable and details saved with the original request model/context. Inside the composer's bottom-right corner, the model picker is followed by a context ring and Send/Stop. The ring opens a local token estimate with a composition bar and expandable system/history/draft/tool definitions. Set a context window per enabled model in Model connection; blank uses 128K (131072 tokens). Its reading names the last request or inspected draft; editing invalidates the draft preview and never contacts the provider. Older omitted turns remain saved, and the 40-turn/128-KiB byte guards remain independent resource limits. Preparation also respects the estimated model input allowance after reserving response tokens and 5% headroom. The latest call’s reported usage is distinct from before-send estimates. Metadata survives restart, model switching and export; existing messages remain readable without metadata. The header's trajectory action opens independently paged saved exchanges and a live lifecycle Log, bounded to 200 in-process events without prompts, credentials or network payloads. The bottom keyboard help label is removed; keyboard controls remain available.
