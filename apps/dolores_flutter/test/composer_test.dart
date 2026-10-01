import 'package:dolores_flutter/bridge.dart';
import 'package:dolores_flutter/chat.dart';
import 'package:dolores_flutter/code_syntax.dart';
import 'package:dolores_flutter/composer_controller.dart';
import 'package:dolores_flutter/main.dart';
import 'package:dolores_flutter/rich_composer.dart';
import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:flutter_test/flutter_test.dart';

const source =
    '# Example\n\n```rust\nfn main() {\n    let x = 1;\n    println!("{x} 世界!");\n}\n```';
Iterable<TextSpan> spans(TextSpan root) sync* {
  yield root;
  for (final child in root.children ?? <InlineSpan>[]) {
    if (child is TextSpan) yield* spans(child);
  }
}

class _Bridge implements ChatBridge {
  final commands = <Map<String, dynamic>>[];
  @override
  Future<dynamic> call(Map<String, dynamic> command) async {
    commands.add(command);
    return command['command'] == 'poll' ? [] : null;
  }

  @override
  Future<void> open() async {}
  @override
  Future<void> close() async {}
}

class _Chat extends ChatController {
  _Chat(super.bridge);
  bool closed = false;
  @override
  void dispose() {
    if (closed) return;
    closed = true;
    super.dispose();
  }
}

Future<ChatController> mount(
  WidgetTester tester, {
  String draft = '',
  bool dark = false,
  ChatBridge? bridge,
}) async {
  final chat = _Chat(bridge ?? _Bridge())
    ..loading = false
    ..configured = true
    ..draft = draft;
  await tester.pumpWidget(
    DoloresApp(chat: chat, themeMode: dark ? ThemeMode.dark : ThemeMode.light),
  );
  await tester.pumpAndSettle();
  addTearDown(() async {
    await tester.pumpWidget(const SizedBox());
    chat.dispose();
  });
  return chat;
}

Finder field(int i) => find.byKey(Key('composer-field-$i'));

Future<void> shortcut(
  WidgetTester tester,
  LogicalKeyboardKey key, {
  bool command = false,
}) async {
  final modifier = command
      ? LogicalKeyboardKey.metaLeft
      : LogicalKeyboardKey.controlLeft;
  await tester.sendKeyDownEvent(modifier);
  await tester.sendKeyEvent(key);
  await tester.sendKeyUpEvent(modifier);
  await tester.pumpAndSettle();
}

void main() {
  testWidgets(
    'Shift+Enter inserts visible paragraph newlines at end/start and replaces a selection',
    (tester) async {
      final chat = await mount(tester, draft: 'first');
      await tester.showKeyboard(field(0));
      var editor = tester.widget<TextField>(field(0)).controller!;
      editor.selection = TextSelection.collapsed(offset: editor.text.length);
      await tester.sendKeyDownEvent(LogicalKeyboardKey.shiftLeft);
      await tester.sendKeyEvent(LogicalKeyboardKey.enter);
      await tester.sendKeyUpEvent(LogicalKeyboardKey.shiftLeft);
      await tester.pumpAndSettle();
      expect(chat.draft, 'first\n');
      expect(tester.widget<TextField>(field(0)).controller!.text, 'first\n');
      tester.testTextInput.updateEditingValue(
        const TextEditingValue(
          text: 'first\nsecond',
          selection: TextSelection.collapsed(offset: 12),
        ),
      );
      await tester.pumpAndSettle();
      editor = tester.widget<TextField>(field(0)).controller!;
      editor.selection = const TextSelection.collapsed(offset: 0);
      await tester.sendKeyDownEvent(LogicalKeyboardKey.shiftLeft);
      await tester.sendKeyEvent(LogicalKeyboardKey.enter);
      await tester.sendKeyUpEvent(LogicalKeyboardKey.shiftLeft);
      await tester.pumpAndSettle();
      expect(chat.draft, '\nfirst\nsecond');
      expect(tester.widget<TextField>(field(0)).controller!.text, chat.draft);
      tester.widget<TextField>(field(0)).controller!.selection =
          const TextSelection(baseOffset: 6, extentOffset: 13);
      await tester.sendKeyDownEvent(LogicalKeyboardKey.shiftLeft);
      await tester.sendKeyEvent(LogicalKeyboardKey.enter);
      await tester.sendKeyUpEvent(LogicalKeyboardKey.shiftLeft);
      await tester.pumpAndSettle();
      expect(chat.draft, '\nfirst\n');
      expect(tester.widget<TextField>(field(0)).controller!.text, chat.draft);
      expect(chat.busy, isFalse);
    },
  );

  testWidgets(
    'Fence Enter, code Shift+Enter and Down exit all keep native typing active',
    (tester) async {
      final chat = await mount(tester);
      await tester.showKeyboard(field(0));
      tester.testTextInput.updateEditingValue(
        const TextEditingValue(
          text: '```rust',
          selection: TextSelection.collapsed(offset: 7),
        ),
      );
      await tester.pumpAndSettle();
      await tester.sendKeyEvent(LogicalKeyboardKey.enter);
      await tester.pumpAndSettle();
      expect(find.byKey(const Key('composer-code-0')), findsOneWidget);
      tester.testTextInput.updateEditingValue(
        const TextEditingValue(
          text: 'one',
          selection: TextSelection.collapsed(offset: 3),
        ),
      );
      await tester.pumpAndSettle();
      await tester.sendKeyDownEvent(LogicalKeyboardKey.shiftLeft);
      await tester.sendKeyEvent(LogicalKeyboardKey.enter);
      await tester.sendKeyUpEvent(LogicalKeyboardKey.shiftLeft);
      await tester.pumpAndSettle();
      expect(tester.widget<TextField>(field(0)).controller!.text, 'one\n');
      tester.testTextInput.updateEditingValue(
        const TextEditingValue(
          text: 'one\nsecond',
          selection: TextSelection.collapsed(offset: 10),
        ),
      );
      await tester.pumpAndSettle();
      await tester.sendKeyEvent(LogicalKeyboardKey.arrowDown);
      await tester.pumpAndSettle();
      tester.testTextInput.updateEditingValue(
        const TextEditingValue(
          text: 'after code',
          selection: TextSelection.collapsed(offset: 10),
        ),
      );
      await tester.pumpAndSettle();
      expect(chat.draft, '```rust\none\nsecond\n```\n\nafter code');
      expect(chat.busy, isFalse);
    },
  );

  testWidgets(
    'Heading Shift+Enter opens prose and repeated newlines remain visible without refocusing',
    (tester) async {
      final chat = await mount(tester, draft: '# Title');
      await tester.showKeyboard(field(0));
      tester.widget<TextField>(field(0)).controller!.selection =
          const TextSelection.collapsed(offset: 5);
      await tester.sendKeyDownEvent(LogicalKeyboardKey.shiftLeft);
      await tester.sendKeyEvent(LogicalKeyboardKey.enter);
      await tester.sendKeyUpEvent(LogicalKeyboardKey.shiftLeft);
      await tester.pumpAndSettle();
      expect(tester.widget<TextField>(field(1)).focusNode!.hasFocus, isTrue);
      await tester.sendKeyDownEvent(LogicalKeyboardKey.shiftLeft);
      await tester.sendKeyEvent(LogicalKeyboardKey.enter);
      await tester.sendKeyUpEvent(LogicalKeyboardKey.shiftLeft);
      await tester.pumpAndSettle();
      expect(tester.widget<TextField>(field(1)).controller!.text, '\n');
      tester.testTextInput.updateEditingValue(
        const TextEditingValue(
          text: '\nprose',
          selection: TextSelection.collapsed(offset: 6),
        ),
      );
      await tester.pumpAndSettle();
      expect(chat.draft, '# Title\n\n\nprose');
      expect(chat.busy, isFalse);
    },
  );

  testWidgets(
    'Removing a later empty code card keeps the active native editor connected',
    (tester) async {
      final chat = await mount(tester, draft: '# Title\n\n```rust\n\n```');
      await tester.showKeyboard(field(1));
      tester.widget<TextField>(field(1)).controller!.selection =
          const TextSelection.collapsed(offset: 0);
      await tester.sendKeyEvent(LogicalKeyboardKey.backspace);
      await tester.pumpAndSettle();
      tester.testTextInput.updateEditingValue(
        const TextEditingValue(
          text: 'after removal',
          selection: TextSelection.collapsed(offset: 13),
        ),
      );
      await tester.pumpAndSettle();
      expect(chat.draft, '# Title\n\nafter removal');
      expect(find.byKey(const Key('composer-code-1')), findsNothing);
    },
  );
  testWidgets(
    'Native typing stays connected through heading creation and removal without refocusing',
    (tester) async {
      final chat = await mount(tester);
      await tester.showKeyboard(field(0));
      tester.testTextInput.updateEditingValue(
        const TextEditingValue(
          text: '# ',
          selection: TextSelection.collapsed(offset: 2),
        ),
      );
      await tester.pumpAndSettle();
      expect(
        tester.testTextInput.isRegistered,
        isTrue,
        reason: 'Folding a heading must retain its native input connection',
      );
      tester.testTextInput.updateEditingValue(
        const TextEditingValue(
          text: 'Title',
          selection: TextSelection.collapsed(offset: 5),
        ),
      );
      await tester.pumpAndSettle();
      expect(chat.draft, '# Title');
      tester.widget<TextField>(field(0)).controller!.selection =
          const TextSelection.collapsed(offset: 0);
      await tester.sendKeyEvent(LogicalKeyboardKey.backspace);
      await tester.pumpAndSettle();
      expect(tester.testTextInput.isRegistered, isTrue);
      tester.testTextInput.updateEditingValue(
        const TextEditingValue(
          text: 'Title continued',
          selection: TextSelection.collapsed(offset: 15),
        ),
      );
      await tester.pumpAndSettle();
      expect(chat.draft, 'Title continued');
    },
  );

  testWidgets(
    'Native typing stays connected through code creation and empty-card removal',
    (tester) async {
      final chat = await mount(tester);
      await tester.showKeyboard(field(0));
      tester.testTextInput.updateEditingValue(
        const TextEditingValue(
          text: '```rust\n',
          selection: TextSelection.collapsed(offset: 8),
        ),
      );
      await tester.pumpAndSettle();
      expect(tester.testTextInput.isRegistered, isTrue);
      tester.testTextInput.updateEditingValue(
        const TextEditingValue(
          text: 'let x = 1;',
          selection: TextSelection.collapsed(offset: 10),
        ),
      );
      await tester.pumpAndSettle();
      expect(chat.draft, '```rust\nlet x = 1;');
      tester.testTextInput.updateEditingValue(
        const TextEditingValue(
          text: '',
          selection: TextSelection.collapsed(offset: 0),
        ),
      );
      await tester.pumpAndSettle();
      await tester.sendKeyEvent(LogicalKeyboardKey.backspace);
      await tester.pumpAndSettle();
      expect(tester.testTextInput.isRegistered, isTrue);
      tester.testTextInput.updateEditingValue(
        const TextEditingValue(
          text: 'Normal typing',
          selection: TextSelection.collapsed(offset: 13),
        ),
      );
      await tester.pumpAndSettle();
      expect(chat.draft, 'Normal typing');
    },
  );

  testWidgets(
    'A native paragraph newline stays visible and subsequent typing stays on the new line',
    (tester) async {
      final chat = await mount(tester);
      await tester.showKeyboard(field(0));
      tester.testTextInput.updateEditingValue(
        const TextEditingValue(
          text: 'first\n',
          selection: TextSelection.collapsed(offset: 6),
        ),
      );
      await tester.pumpAndSettle();
      expect(tester.widget<TextField>(field(0)).controller!.text, 'first\n');
      tester.testTextInput.updateEditingValue(
        const TextEditingValue(
          text: 'first\nsecond',
          selection: TextSelection.collapsed(offset: 12),
        ),
      );
      await tester.pumpAndSettle();
      expect(chat.draft, 'first\nsecond');
    },
  );
  String? clipboard;
  setUp(() {
    clipboard = null;
    TestDefaultBinaryMessengerBinding.instance.defaultBinaryMessenger
        .setMockMethodCallHandler(SystemChannels.platform, (call) async {
          switch (call.method) {
            case 'Clipboard.setData':
              clipboard = (call.arguments as Map)['text'] as String;
              return null;
            case 'Clipboard.getData':
              return {'text': clipboard};
            case 'Clipboard.hasStrings':
              return {'value': clipboard?.isNotEmpty ?? false};
            default:
              return null;
          }
        });
  });
  tearDown(
    () => TestDefaultBinaryMessengerBinding.instance.defaultBinaryMessenger
        .setMockMethodCallHandler(SystemChannels.platform, null),
  );
  testWidgets(
    'Local copy delegates natively and same-character typing replaces Select All',
    (tester) async {
      final raw = '# X\n\n```rust\nlet x = 1;\n```';
      final chat = await mount(tester, draft: raw);
      await tester.showKeyboard(field(1));
      tester.widget<TextField>(field(1)).controller!.selection =
          const TextSelection(baseOffset: 0, extentOffset: 3);
      await shortcut(tester, LogicalKeyboardKey.keyC);
      expect(clipboard, 'let');
      await tester.showKeyboard(field(0));
      await shortcut(tester, LogicalKeyboardKey.keyA);
      await tester.sendKeyEvent(LogicalKeyboardKey.keyX, character: 'X');
      tester.testTextInput.updateEditingValue(
        const TextEditingValue(
          text: 'X',
          selection: TextSelection.collapsed(offset: 1),
        ),
      );
      await tester.pumpAndSettle();
      expect(chat.draft, 'X');
      expect(find.byType(TextField), findsOneWidget);
      expect(tester.widget<TextField>(field(0)).style!.fontSize, 14);
    },
  );

  testWidgets(
    'Clicking after Select All clears document selection before local editing',
    (tester) async {
      final chat = await mount(tester, draft: source);
      await tester.tap(field(0));
      await tester.pump();
      await shortcut(tester, LogicalKeyboardKey.keyA);
      await tester.tap(field(1));
      await tester.pumpAndSettle();
      expect(
        tester.widget<TextField>(field(0)).controller!.selection.isCollapsed,
        isTrue,
      );
      await tester.enterText(field(1), 'Local edit');
      await tester.pumpAndSettle();
      expect(chat.draft, '# Example\n\n```rust\nLocal edit\n```');
      expect(tester.takeException(), isNull);
    },
  );
  testWidgets(
    'Select All from code supports whole Markdown copy, cut and undo',
    (tester) async {
      final chat = await mount(tester, draft: source);
      await tester.tap(field(1));
      await tester.pump();
      await shortcut(tester, LogicalKeyboardKey.keyA, command: true);
      await shortcut(tester, LogicalKeyboardKey.keyC, command: true);
      expect(clipboard, source);
      await shortcut(tester, LogicalKeyboardKey.keyX);
      expect(chat.draft, '');
      expect(find.byType(TextField), findsOneWidget);
      await shortcut(tester, LogicalKeyboardKey.keyZ);
      expect(chat.draft, source);
      expect(find.byKey(const Key('composer-code-1')), findsOneWidget);
      expect(tester.takeException(), isNull);
    },
  );

  testWidgets(
    'Typing and identical-body clipboard paste replace the whole selected document',
    (tester) async {
      final chat = await mount(tester, draft: source);
      await tester.tap(field(1));
      await tester.pump();
      await shortcut(tester, LogicalKeyboardKey.keyA);
      await tester.enterText(field(1), 'Replacement');
      await tester.pumpAndSettle();
      expect(chat.draft, 'Replacement');
      expect(find.byType(TextField), findsOneWidget);
      await shortcut(tester, LogicalKeyboardKey.keyZ);
      expect(chat.draft, source);
      await tester.tap(field(0));
      await tester.pump();
      await shortcut(tester, LogicalKeyboardKey.keyA);
      clipboard = 'Example';
      await shortcut(tester, LogicalKeyboardKey.keyV);
      expect(chat.draft, 'Example');
      expect(find.byType(TextField), findsOneWidget);
      expect(tester.widget<TextField>(field(0)).style!.fontSize, 14);
    },
  );

  testWidgets('IME replacement after Select All commits one normal paragraph', (
    tester,
  ) async {
    final chat = await mount(tester, draft: source);
    await tester.showKeyboard(field(1));
    await shortcut(tester, LogicalKeyboardKey.keyA);
    tester.testTextInput.updateEditingValue(
      const TextEditingValue(
        text: '你好',
        selection: TextSelection.collapsed(offset: 2),
        composing: TextRange(start: 0, end: 2),
      ),
    );
    await tester.pump();
    expect(chat.draft, '你好');
    expect(
      tester.widget<TextField>(field(1)).controller!.value.composing,
      const TextRange(start: 0, end: 2),
    );
    tester.testTextInput.updateEditingValue(
      const TextEditingValue(
        text: '你好',
        selection: TextSelection.collapsed(offset: 2),
      ),
    );
    await tester.pumpAndSettle();
    expect(chat.draft, '你好');
    expect(find.byType(TextField), findsOneWidget);
    expect(tester.widget<TextField>(field(0)).focusNode!.hasFocus, isTrue);
    await shortcut(tester, LogicalKeyboardKey.keyZ);
    expect(chat.draft, source);
    expect(tester.takeException(), isNull);
  });

  testWidgets(
    'Empty tilde/open/CRLF cards are removable and removal can be undone',
    (tester) async {
      for (final raw in ['~~~rust\n\n~~~', '```rust\n', '```rust\r\n\r\n```']) {
        final chat = await mount(tester, draft: raw);
        await tester.tap(field(0));
        await tester.pump();
        tester.widget<TextField>(field(0)).controller!.selection =
            const TextSelection.collapsed(offset: 0);
        await tester.sendKeyEvent(LogicalKeyboardKey.backspace);
        await tester.pumpAndSettle();
        expect(chat.draft, '');
        expect(find.byKey(const Key('composer-code-0')), findsNothing);
        await shortcut(tester, LogicalKeyboardKey.keyZ);
        expect(chat.draft, raw);
        expect(find.byKey(const Key('composer-code-0')), findsOneWidget);
        await tester.pumpWidget(const SizedBox());
        chat.dispose();
      }
    },
  );
  testWidgets(
    'Backspace removes an empty code card without removing the following heading',
    (tester) async {
      final chat = await mount(tester, draft: '```rust\n\n```\n\n# Example');
      await tester.tap(field(0));
      await tester.pump();
      tester.widget<TextField>(field(0)).controller!.selection =
          const TextSelection.collapsed(offset: 0);
      await tester.sendKeyEvent(LogicalKeyboardKey.backspace);
      await tester.pumpAndSettle();
      expect(find.byKey(const Key('composer-code-0')), findsNothing);
      expect(tester.widget<TextField>(field(0)).controller!.text, 'Example');
      expect(chat.draft.trim(), '# Example');
    },
  );

  testWidgets(
    'Backspace removes empty heading formatting and leaves a normal input',
    (tester) async {
      final chat = await mount(tester, draft: '# Example');
      await tester.enterText(field(0), '');
      await tester.pumpAndSettle();
      await tester.sendKeyEvent(LogicalKeyboardKey.backspace);
      await tester.pumpAndSettle();
      expect(chat.draft, '');
      expect(tester.widget<TextField>(field(0)).style!.fontSize, 14);
      await tester.enterText(field(0), 'Ordinary text');
      await tester.pumpAndSettle();
      expect(chat.draft, 'Ordinary text');
    },
  );

  testWidgets(
    'Ctrl+A selects all composer blocks and Backspace clears the whole draft',
    (tester) async {
      final chat = await mount(tester, draft: source);
      await tester.tap(field(0));
      await tester.pump();
      await tester.sendKeyDownEvent(LogicalKeyboardKey.controlLeft);
      await tester.sendKeyEvent(LogicalKeyboardKey.keyA);
      await tester.sendKeyUpEvent(LogicalKeyboardKey.controlLeft);
      await tester.pump();
      for (final i in [0, 1]) {
        final controller = tester.widget<TextField>(field(i)).controller!;
        expect(
          controller.selection,
          TextSelection(baseOffset: 0, extentOffset: controller.text.length),
        );
      }
      await tester.sendKeyEvent(LogicalKeyboardKey.backspace);
      await tester.pumpAndSettle();
      expect(chat.draft, '');
      expect(find.byType(TextField), findsOneWidget);
      expect(find.byKey(const Key('composer-code-1')), findsNothing);
    },
  );
  testWidgets('Down enters existing prose and undo restores a keyboard exit', (
    tester,
  ) async {
    final raw = '$source\n\nExisting prose';
    final chat = await mount(tester, draft: raw);
    await tester.tap(field(1));
    await tester.pump();
    var code = tester.widget<TextField>(field(1)).controller!;
    code.selection = TextSelection.collapsed(offset: code.text.length);
    await tester.sendKeyEvent(LogicalKeyboardKey.arrowDown);
    await tester.pumpAndSettle();
    expect(tester.widget<TextField>(field(2)).focusNode!.hasFocus, isTrue);
    expect(chat.draft, raw);
    // Reload a draft ending in code, then leave it through the keyboard.
    final rich = tester.widget<RichComposer>(find.byKey(const Key('composer')));
    rich.controller.text = source;
    await tester.pumpAndSettle();
    await tester.tap(field(1));
    await tester.pump();
    code = tester.widget<TextField>(field(1)).controller!;
    code.selection = TextSelection.collapsed(offset: code.text.length);
    await tester.sendKeyEvent(LogicalKeyboardKey.arrowDown);
    await tester.pumpAndSettle();
    await tester.sendKeyDownEvent(LogicalKeyboardKey.controlLeft);
    await tester.sendKeyEvent(LogicalKeyboardKey.keyZ);
    await tester.sendKeyUpEvent(LogicalKeyboardKey.controlLeft);
    await tester.pumpAndSettle();
    expect(chat.draft, source);
    expect(field(2), findsNothing);
    expect(tester.widget<TextField>(field(1)).focusNode!.hasFocus, isTrue);
    expect(tester.takeException(), isNull);
  });

  testWidgets('Down stays within wrapped code; Shift+Down retains selection', (
    tester,
  ) async {
    tester.view.physicalSize = const Size(620, 700);
    tester.view.devicePixelRatio = 1;
    addTearDown(tester.view.resetPhysicalSize);
    addTearDown(tester.view.resetDevicePixelRatio);
    final raw = '```rust\n${'value ' * 80}\n```';
    final chat = await mount(tester, draft: raw);
    await tester.tap(field(0));
    await tester.pump();
    final code = tester.widget<TextField>(field(0)).controller!;
    code.selection = const TextSelection.collapsed(offset: 0);
    final rendered = tester
        .state<EditableTextState>(
          find.descendant(of: field(0), matching: find.byType(EditableText)),
        )
        .renderEditable;
    expect(
      rendered.getLineAtOffset(const TextPosition(offset: 0)).end,
      lessThan(code.text.length),
    );
    await tester.sendKeyEvent(LogicalKeyboardKey.arrowDown);
    await tester.pump();
    expect(field(1), findsNothing);
    expect(chat.draft, raw);
    code.selection = TextSelection.collapsed(offset: code.text.length);
    await tester.sendKeyDownEvent(LogicalKeyboardKey.shiftLeft);
    await tester.sendKeyEvent(LogicalKeyboardKey.arrowDown);
    await tester.sendKeyUpEvent(LogicalKeyboardKey.shiftLeft);
    await tester.pump();
    expect(field(1), findsNothing);
    expect(chat.draft, raw);
    expect(tester.takeException(), isNull);
  });
  testWidgets(
    'Down on the last code line creates and focuses prose without buttons',
    (tester) async {
      for (final raw in [
        source,
        '```rust\nlet x = 1;',
        '~~~rust\nlet x = 1;',
        source.replaceAll('\n', '\r\n'),
      ]) {
        final chat = await mount(tester, draft: raw);
        final codeIndex = raw.startsWith('#') ? 1 : 0;
        await tester.tap(field(codeIndex));
        await tester.pump();
        final code = tester.widget<TextField>(field(codeIndex)).controller!;
        code.selection = TextSelection.collapsed(offset: code.text.length);
        await tester.sendKeyEvent(LogicalKeyboardKey.arrowDown);
        await tester.pumpAndSettle();
        final next = codeIndex + 1;
        expect(find.text('Source'), findsNothing);
        expect(find.text('Add text'), findsNothing);
        expect(
          tester.widget<TextField>(field(next)).focusNode!.hasFocus,
          isTrue,
        );
        await tester.sendKeyEvent(LogicalKeyboardKey.arrowDown);
        await tester.pump();
        expect(find.byType(TextField), findsNWidgets(next + 1));
        await tester.enterText(field(next), 'Explain this code.');
        await tester.pumpAndSettle();
        expect(
          chat.draft,
          '${raw.startsWith('#') ? raw : '$raw\n${raw.startsWith('~~~') ? '~~~' : '```'}'}${raw.contains('\r\n') ? '\r\n\r\n' : '\n\n'}Explain this code.',
        );
        expect(tester.takeException(), isNull);
        await tester.pumpWidget(const SizedBox());
        chat.dispose();
      }
    },
  );
  test('Folded body ranges preserve raw fences, CRLF, unknown languages and incomplete code', () {
    for (final raw in [
      source,
      source.replaceAll('\n', '\r\n'),
      '````markdown\n```\nliteral\n```\n````',
      '~~~unknown\nabc\n~~~',
      '```rust\nlet x = 1;',
    ]) {
      final blocks = parseComposer(raw);
      expect(blocks.last.kind, ComposerBlockKind.code);
      final c = ComposerController(text: raw);
      final b = blocks.last;
      c.edit(
        TextEditingValue(
          text: raw.replaceRange(b.bodyStart, b.bodyEnd, 'changed'),
        ),
      );
      expect(c.text.substring(0, b.bodyStart), raw.substring(0, b.bodyStart));
      expect(c.text.substring(b.bodyStart + 7), raw.substring(b.bodyEnd));
      c.undo();
      expect(c.text, raw);
      c.redo();
      expect(c.text, contains('changed'));
      c.dispose();
    }
    expect(parseComposer('x\n' * 801), hasLength(1));
    expect(parseComposer('x' * 32769), hasLength(1));
  });

  testWidgets(
    'Heading and syntax-colored code card are editable in both compact themes',
    (tester) async {
      tester.view.physicalSize = const Size(620, 700);
      tester.view.devicePixelRatio = 1;
      addTearDown(tester.view.resetPhysicalSize);
      addTearDown(tester.view.resetDevicePixelRatio);
      final chat = await mount(tester, draft: source);
      for (final dark in [false, true]) {
        await tester.pumpWidget(
          DoloresApp(
            chat: chat,
            themeMode: dark ? ThemeMode.dark : ThemeMode.light,
          ),
        );
        await tester.pumpAndSettle();
        final heading = tester.widget<TextField>(field(0));
        expect(heading.controller!.text, 'Example');
        expect(heading.style!.fontSize, 24);
        final code = tester.widget<TextField>(field(1));
        expect(code.controller, isA<CodeSyntaxController>());
        expect(code.controller!.text, startsWith('fn main()'));
        expect(code.controller!.text, isNot(contains('```')));
        expect(find.byKey(const Key('composer-code-1')), findsOneWidget);
        expect(find.text('Rust'), findsOneWidget);
        final rich = code.controller!.buildTextSpan(
          context: tester.element(field(1)),
          style: code.style,
          withComposing: true,
        );
        expect(rich.toPlainText(), code.controller!.text);
        expect(
          spans(rich)
              .where((s) => s.style?.color != null)
              .map((s) => s.style!.color)
              .toSet()
              .length,
          greaterThanOrEqualTo(4),
        );
        expect(chat.draft, source);
        expect(tester.takeException(), isNull);
      }
    },
  );

  testWidgets(
    'Pasting a Markdown document folds it immediately without changing sent source',
    (tester) async {
      final bridge = _Bridge();
      final chat = await mount(tester, bridge: bridge);
      await tester.enterText(field(0), source);
      await tester.pumpAndSettle();
      expect(tester.widget<TextField>(field(0)).controller!.text, 'Example');
      expect(find.byKey(const Key('composer-code-1')), findsOneWidget);
      expect(chat.draft, source);
      await tester.tap(find.byKey(const Key('send')));
      await tester.pump();
      expect(bridge.commands.first['input'], source);
      chat.dispose();
    },
  );

  testWidgets(
    'Editing heading and code changes only their canonical body ranges',
    (tester) async {
      final chat = await mount(tester, draft: source);
      await tester.enterText(field(0), 'Renamed');
      await tester.pumpAndSettle();
      expect(chat.draft, source.replaceFirst('Example', 'Renamed'));
      await tester.enterText(field(1), 'let answer = 42;\n// 你好');
      await tester.pumpAndSettle();
      expect(chat.draft, '# Renamed\n\n```rust\nlet answer = 42;\n// 你好\n```');
      expect(tester.takeException(), isNull);
    },
  );

  testWidgets(
    'Language menu updates canonical fences and visible highlighting',
    (tester) async {
      final chat = await mount(tester, draft: source);
      await tester.tap(find.byKey(const Key('composer-language-1')));
      await tester.pumpAndSettle();
      await tester.tap(find.text('Python').last);
      await tester.pumpAndSettle();
      expect(chat.draft, source.replaceFirst('```rust', '```python'));
      await tester.tap(find.byKey(const Key('composer-language-1')));
      await tester.pumpAndSettle();
      await tester.tap(find.text('Plain text').last);
      await tester.pumpAndSettle();
      final rendered =
          tester
                  .state<EditableTextState>(
                    find.descendant(
                      of: field(1),
                      matching: find.byType(EditableText),
                    ),
                  )
                  .renderEditable
                  .text
              as TextSpan;
      expect(
        rendered.toPlainText(),
        tester.widget<TextField>(field(1)).controller!.text,
      );
      expect(
        spans(rendered).where((span) => span.style?.color != null).length,
        lessThanOrEqualTo(1),
      );
      expect(chat.draft, source.replaceFirst('```rust', '```'));
      await tester.tap(find.byKey(const Key('composer-language-1')));
      await tester.pumpAndSettle();
      await tester.tap(find.text('Rust').last);
      await tester.pumpAndSettle();
      expect(chat.draft, source);
      expect(find.text('Rust'), findsOneWidget);
    },
  );

  testWidgets('Undo/redo traverses canonical block edits', (tester) async {
    final chat = await mount(tester, draft: source);
    await tester.enterText(field(0), 'Renamed');
    await tester.pumpAndSettle();
    await tester.sendKeyDownEvent(LogicalKeyboardKey.controlLeft);
    await tester.sendKeyEvent(LogicalKeyboardKey.keyZ);
    await tester.sendKeyUpEvent(LogicalKeyboardKey.controlLeft);
    await tester.pumpAndSettle();
    expect(chat.draft, source);
    await tester.sendKeyDownEvent(LogicalKeyboardKey.controlLeft);
    await tester.sendKeyEvent(LogicalKeyboardKey.keyY);
    await tester.sendKeyUpEvent(LogicalKeyboardKey.controlLeft);
    await tester.pumpAndSettle();
    expect(chat.draft, source.replaceFirst('Example', 'Renamed'));
  });

  testWidgets(
    'Code Enter keeps editing, Ctrl+Enter sends, and arrows cross blocks',
    (tester) async {
      final bridge = _Bridge();
      final chat = await mount(tester, draft: source, bridge: bridge);
      await tester.tap(field(1));
      await tester.pump();
      final code = tester.widget<TextField>(field(1)).controller!;
      code.selection = const TextSelection.collapsed(offset: 0);
      await tester.sendKeyEvent(LogicalKeyboardKey.arrowUp);
      await tester.pump();
      expect(tester.widget<TextField>(field(0)).focusNode!.hasFocus, isTrue);
      await tester.sendKeyEvent(LogicalKeyboardKey.arrowDown);
      await tester.pump();
      expect(tester.widget<TextField>(field(1)).focusNode!.hasFocus, isTrue);
      await tester.sendKeyEvent(LogicalKeyboardKey.enter);
      await tester.pump();
      expect(bridge.commands, isEmpty);
      await tester.sendKeyDownEvent(LogicalKeyboardKey.controlLeft);
      await tester.sendKeyEvent(LogicalKeyboardKey.enter);
      await tester.sendKeyUpEvent(LogicalKeyboardKey.controlLeft);
      await tester.pump();
      expect(bridge.commands.first['command'], 'start');
      chat.dispose();
    },
  );

  testWidgets(
    'IME keeps native composition and cannot send or fold until committed',
    (tester) async {
      final bridge = _Bridge();
      final chat = await mount(tester, draft: source, bridge: bridge);
      await tester.showKeyboard(field(1));
      tester.testTextInput.updateEditingValue(
        const TextEditingValue(
          text: '你好',
          selection: TextSelection.collapsed(offset: 2),
          composing: TextRange(start: 0, end: 2),
        ),
      );
      await tester.pump();
      final code = tester.widget<TextField>(field(1));
      final rich = code.controller!.buildTextSpan(
        context: tester.element(field(1)),
        style: code.style,
        withComposing: true,
      );
      expect(
        spans(rich).any(
          (s) =>
              s.text == '你好' && s.style?.decoration == TextDecoration.underline,
        ),
        isTrue,
      );
      await tester.sendKeyEvent(LogicalKeyboardKey.enter);
      expect(bridge.commands, isEmpty);
      await tester.sendKeyEvent(LogicalKeyboardKey.arrowDown);
      await tester.pump();
      expect(field(2), findsNothing);
      expect(tester.widget<TextField>(field(1)).focusNode!.hasFocus, isTrue);
      tester.testTextInput.updateEditingValue(
        const TextEditingValue(
          text: '你好',
          selection: TextSelection.collapsed(offset: 2),
        ),
      );
      await tester.pump();
      expect(chat.draft, '# Example\n\n```rust\n你好\n```');
    },
  );

  testWidgets(
    'Code syntax handles unsupported/large input without hiding text or doing I/O',
    (tester) async {
      final focus = FocusNode();
      final input = ComposerController(
        text: '```unknown\n<img src="https://example.invalid/pixel">\n```',
      );
      await tester.pumpWidget(
        MaterialApp(
          home: Scaffold(
            body: RichComposer(
              controller: input,
              focusNode: focus,
              readOnly: false,
              hint: '',
              onChanged: (_) {},
              onSend: () {},
              trailing: const SizedBox(),
            ),
          ),
        ),
      );
      await tester.pumpAndSettle();
      expect(find.byType(Image), findsNothing);
      final code = tester.widget<TextField>(field(0));
      for (final text in ['let x = 1;', 'x' * 8193, 'x\n' * 201]) {
        final c = CodeSyntaxController(text: text, language: 'rust');
        expect(
          c
              .buildTextSpan(
                context: tester.element(field(0)),
                style: code.style,
                withComposing: true,
              )
              .toPlainText(),
          text,
        );
        c.dispose();
      }
      await tester.pumpWidget(const SizedBox());
      input.dispose();
      focus.dispose();
    },
  );
}
