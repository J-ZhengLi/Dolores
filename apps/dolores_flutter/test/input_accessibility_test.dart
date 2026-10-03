import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:flutter_test/flutter_test.dart';

import 'package:dolores_flutter/bridge.dart';

import 'composer_test.dart' show field, mount, shortcut;
import 'support/workspaces.dart';

class InputBridge implements ChatBridge {
  final commands = <Map<String, dynamic>>[];
  @override
  Future<void> open() async {}
  @override
  Future<void> close() async {}
  @override
  Future<dynamic> call(Map<String, dynamic> command) async {
    commands.add(command);
    if (command['command'] == 'createSession') return createdWorkspace(command);
    return command['command'] == 'poll' ? [] : null;
  }
}

void main() {
  testWidgets('Rich draft fields expose names, editable values, and focus', (
    tester,
  ) async {
    final semantics = tester.ensureSemantics();
    try {
      await mount(tester, draft: '# Title\n\n```rust\nx\n```\n\nProse');
      expect(
        find.bySemanticsLabel('Message, heading level 1, block 1'),
        findsOneWidget,
      );
      expect(
        find.bySemanticsLabel('Message, Rust code, block 2'),
        findsOneWidget,
      );
      expect(find.bySemanticsLabel('Message, block 3'), findsOneWidget);
      expect(
        find.bySemanticsLabel(RegExp('Code language: Rust')),
        findsOneWidget,
      );
      await tester.showKeyboard(field(1));
      await tester.pump();
      final node = tester.getSemantics(field(1));
      expect(node.getSemanticsData().value, 'x');
      expect(tester.widget<TextField>(field(1)).focusNode!.hasFocus, isTrue);
      expect(tester.takeException(), isNull);
    } finally {
      semantics.dispose();
    }
  });

  testWidgets(
    'Send preserves active IME composition, then accepts the commit',
    (tester) async {
      final bridge = InputBridge();
      final chat = await mount(tester, bridge: bridge);
      await tester.showKeyboard(field(0));
      tester.testTextInput.updateEditingValue(
        const TextEditingValue(
          text: '你好',
          selection: TextSelection.collapsed(offset: 2),
          composing: TextRange(start: 0, end: 2),
        ),
      );
      await tester.pump();
      await tester.tap(find.byKey(const Key('send')));
      await tester.pump();
      expect(bridge.commands, isEmpty);
      expect(chat.busy, isFalse);
      expect(chat.draft, '你好');
      expect(
        tester.widget<TextField>(field(0)).controller!.value.composing,
        const TextRange(start: 0, end: 2),
      );
      tester.testTextInput.updateEditingValue(
        const TextEditingValue(
          text: '你好',
          selection: TextSelection.collapsed(offset: 2),
        ),
      );
      await tester.pumpAndSettle();
      await tester.tap(find.byKey(const Key('send')));
      await tester.pump();
      expect(
        bridge.commands.where((c) => c['command'] == 'start'),
        hasLength(1),
      );
      expect(
        bridge.commands.singleWhere((c) => c['command'] == 'start')['input'],
        '你好',
      );
      chat.dispose();
    },
  );

  testWidgets('Windows shortcuts select and undo the complete rich draft', (
    tester,
  ) async {
    const source = '# Title\n\n```rust\nx\n```\n\nProse';
    final chat = await mount(tester, draft: source);
    await tester.showKeyboard(field(2));
    await shortcut(tester, LogicalKeyboardKey.keyA);
    await tester.sendKeyEvent(LogicalKeyboardKey.backspace);
    await tester.pumpAndSettle();
    expect(chat.draft, '');
    await shortcut(tester, LogicalKeyboardKey.keyZ);
    await tester.pumpAndSettle();
    expect(chat.draft, source);
    expect(find.byType(TextField), findsNWidgets(3));
  }, variant: TargetPlatformVariant.only(TargetPlatform.windows));

  testWidgets('Code language waits for composition and keeps its text', (
    tester,
  ) async {
    final chat = await mount(tester, draft: '```rust\nx\n```');
    await tester.showKeyboard(field(0));
    tester.testTextInput.updateEditingValue(
      const TextEditingValue(
        text: '你好',
        selection: TextSelection.collapsed(offset: 2),
        composing: TextRange(start: 0, end: 2),
      ),
    );
    await tester.pump();
    final menu = find.byKey(const Key('composer-language-0'));
    expect(tester.widget<PopupMenuButton<String>>(menu).enabled, isFalse);
    await tester.tap(menu);
    await tester.pump();
    expect(find.text('Python'), findsNothing);
    expect(chat.draft, '```rust\n你好\n```');
    tester.testTextInput.updateEditingValue(
      const TextEditingValue(
        text: '你好',
        selection: TextSelection.collapsed(offset: 2),
      ),
    );
    await tester.pumpAndSettle();
    expect(tester.widget<PopupMenuButton<String>>(menu).enabled, isTrue);
    await tester.tap(menu);
    await tester.pumpAndSettle();
    await tester.tap(find.text('Python').last);
    await tester.pumpAndSettle();
    expect(chat.draft, '```python\n你好\n```');
    expect(tester.widget<TextField>(field(0)).focusNode!.hasFocus, isTrue);
  });
}
