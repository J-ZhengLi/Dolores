# xterm 4.0.0 compatibility patch

This directory vendors the `lib`, `pubspec.yaml` and `LICENSE` files from the
published xterm 4.0.0 package. Upstream: https://github.com/TerminalStudio/xterm.dart.
Original pub.dev archive SHA-256:
`168dfedca77cba33fdb6f52e2cd001e9fde216e398e89335c19b524bb22da3a2`.
The original MIT license is retained. The application uses this local, pinned
dependency so builds never depend on a modified machine-wide package cache.

The only upstream source change is in `lib/src/ui/custom_text_edit.dart`:

```dart
final config = TextInputConfiguration(
  viewId: View.of(context).viewId,
```

Flutter 3.47.5's Windows engine rejects `TextInput.setClient` when `viewId` is
null. xterm 4.0.0 omits it. Ordinary text and IME commits therefore disappear,
although Enter and other keys handled directly by xterm still work. Binding to
the containing view fixes native text entry without disabling IME or duplicating
characters through a hardware-key fallback.

The local `.gitattributes` preserves four upstream trailing-space lines inside
the default key table's multiline string instead of rewriting the snapshot.

Qualification lives in the application's `test/terminal_test.dart` and its
non-executing `terminal_input_smoke.dart` native fixture. When upstream provides
this fix in a release, replace the path dependency and rerun those checks before
removing this directory. Do not format or otherwise edit the upstream snapshot
as part of unrelated application changes.
