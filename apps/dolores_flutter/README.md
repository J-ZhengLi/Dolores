# Dolores Flutter desktop

This is the selected desktop UI. Widgets call the Rust host through a bundled C ABI on a worker isolate, keeping synchronous native/storage operations away from the UI isolate. The normal entry point is `lib/main.dart`; diagnostic entry points are separate builds.

| Area | Implementation |
| --- | --- |
| Native loading and worker lifecycle | `lib/bridge.dart` |
| Chat, history and run state | `lib/chat.dart`, `lib/main.dart` |
| System-theme tokens | `lib/theme.dart` |
| Rich editable draft | `lib/rich_composer.dart`, `lib/composer_controller.dart` |
| Reply Markdown/code | `lib/reply_content.dart`, `lib/code_syntax.dart` |
| Inspectors | Memory, skills, changes, instructions, MCP, feedback and comparisons modules |

See [user instructions](../../docs/USER_GUIDE.md) for controls and [contributor guide](../../CONTRIBUTING.md) for builds/tests/packaging. The [UI contract](../../docs/UI.md) governs UI changes. The [bridge contract](../../docs/flutter/flutter-api.md) describes JSON ownership and coordination.

Build from the root with `scripts/build-flutter.ps1` on Windows or `pnpm desktop:build` for the host. The complete bundle contains Flutter, the Rust bridge, plugins and assets. No model, Node service or webview is bundled. macOS/Linux definitions exist; acceptance remains open in [ACCEPTANCE](../../docs/ACCEPTANCE.md).
