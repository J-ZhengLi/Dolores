import 'package:dolores_flutter/bridge.dart';
import 'package:dolores_flutter/chat.dart';
import 'package:dolores_flutter/main.dart';
import 'package:dolores_flutter/message_frame.dart';
import 'package:dolores_flutter/reply_content.dart';
import 'package:dolores_flutter/theme.dart';
import 'package:flutter/material.dart';
import 'package:flutter/gestures.dart';
import 'package:flutter/services.dart';
import 'package:flutter_markdown_plus/flutter_markdown_plus.dart';
import 'package:flutter_test/flutter_test.dart';

String selectedText(WidgetTester tester) => tester
    .widgetList<SelectableText>(find.byType(SelectableText))
    .map((w) => w.data ?? w.textSpan!.toPlainText())
    .join('\n');

Widget host(
  String text, {
  bool dark = false,
  bool streaming = false,
  VoidCallback? onRendered,
}) => MaterialApp(
  theme: doloresTheme(dark),
  home: Scaffold(
    body: SingleChildScrollView(
      child: Padding(
        padding: const EdgeInsets.all(16),
        child: ReplyContent(
          text: text,
          streaming: streaming,
          onRendered: onRendered,
        ),
      ),
    ),
  ),
);

void main() {
  testWidgets('Rich content stays readable in both themes at compact width', (
    tester,
  ) async {
    tester.view.physicalSize = const Size(390, 900);
    tester.view.devicePixelRatio = 1;
    addTearDown(tester.view.resetPhysicalSize);
    addTearDown(tester.view.resetDevicePixelRatio);
    const text = '''# A useful answer

**Bold**, *italic* and `inline_code` — 你好.

- First
- Second

1. Ordered
2. Items

> A quoted passage.

| Name | Value |
| --- | --- |
| Key | A long table cell with several words |

```dart
print('hello'); // a deliberately very long line that should scroll horizontally rather than overflow the transcript
```
''';
    for (final dark in [false, true]) {
      await tester.pumpWidget(host(text, dark: dark));
      await tester.pumpAndSettle();
      expect(selectedText(tester), contains('A useful answer'));
      expect(selectedText(tester), contains('inline_code'));
      expect(selectedText(tester), contains('Ordered'));
      expect(find.byType(Table), findsOneWidget);
      expect(find.byType(ReplyCodeBlock), findsOneWidget);
      expect(find.text('dart'), findsOneWidget);
      final horizontal = tester
          .widgetList<SingleChildScrollView>(find.byType(SingleChildScrollView))
          .where((w) => w.scrollDirection == Axis.horizontal);
      expect(horizontal, hasLength(2)); // table and independent code scroll
      expect(
        horizontal.last.controller!.position.maxScrollExtent,
        greaterThan(0),
      );
      expect(tester.takeException(), isNull);
    }
  });

  testWidgets(
    'Code copy preserves indentation, blank lines, Unicode and HTML',
    (tester) async {
      String? copied;
      tester.binding.defaultBinaryMessenger.setMockMethodCallHandler(
        SystemChannels.platform,
        (call) async {
          if (call.method == 'Clipboard.setData') {
            copied = (call.arguments as Map)['text'] as String;
          }
          return null;
        },
      );
      addTearDown(
        () => tester.binding.defaultBinaryMessenger.setMockMethodCallHandler(
          SystemChannels.platform,
          null,
        ),
      );
      const code = "  print('你好');\n\n<script>literal()</script>\n";
      await tester.pumpWidget(host('```unknown_language\n$code```'));
      await tester.pumpAndSettle();
      await tester.tap(find.text('Copy code'));
      await tester.pumpAndSettle();
      expect(copied, code);
      expect(find.text('Code copied.'), findsOneWidget);
      expect(tester.takeException(), isNull);
      await tester.pumpWidget(host('    unlabelled code\n'));
      await tester.pumpAndSettle();
      expect(find.text('Code'), findsOneWidget);
    },
  );

  testWidgets(
    'HTML stays inert and all image source types avoid image widgets',
    (tester) async {
      await tester.pumpWidget(
        host('''<script>alert('no')</script>

<img src="https://example.invalid/pixel">

![Remote](https://example.invalid/pixel)

![Local](file:///private/image.png)

![Inline](data:image/png;base64,AAAA)

![Asset](resource:assets/private.png)
'''),
      );
      await tester.pumpAndSettle();
      expect(selectedText(tester), contains('<script>'));
      expect(selectedText(tester), contains('<img src='));
      expect(find.byType(Image), findsNothing);
      for (final label in ['Remote', 'Local', 'Inline', 'Asset']) {
        expect(find.text('Image: $label'), findsOneWidget);
      }
      expect(tester.takeException(), isNull);
    },
  );

  testWidgets(
    'Links reveal the destination and copy only after an explicit tap',
    (tester) async {
      String? copied;
      tester.binding.defaultBinaryMessenger.setMockMethodCallHandler(
        SystemChannels.platform,
        (call) async {
          if (call.method == 'Clipboard.setData') {
            copied = (call.arguments as Map)['text'] as String;
          }
          return null;
        },
      );
      addTearDown(
        () => tester.binding.defaultBinaryMessenger.setMockMethodCallHandler(
          SystemChannels.platform,
          null,
        ),
      );
      await tester.pumpWidget(host('[Docs](https://example.invalid/docs?q=1)'));
      await tester.pumpAndSettle();
      expect(copied, isNull);
      await tester.tap(find.text('Docs'));
      await tester.pumpAndSettle();
      expect(find.text('Link destination'), findsOneWidget);
      expect(find.text('https://example.invalid/docs?q=1'), findsOneWidget);
      expect(copied, isNull);
      await tester.tap(find.text('Copy link'));
      await tester.pumpAndSettle();
      expect(copied, 'https://example.invalid/docs?q=1');
      expect(find.text('Link destination'), findsNothing);
      for (final href in [
        'javascript:alert(1)',
        'file:///private/file',
        'data:text/html,abc',
        'https://user@example.invalid',
        '/relative',
        'https://example.invalid/\u202ehidden',
      ]) {
        final context = tester.element(find.byType(ReplyContent));
        await showReplyLink(context, href);
        await tester.pump();
        expect(find.text('Link destination'), findsNothing);
      }
      expect(copied, 'https://example.invalid/docs?q=1');
      expect(tester.takeException(), isNull);
    },
  );

  testWidgets(
    'Partial fences coalesce updates, flush on completion and cancel on disposal',
    (tester) async {
      var rendered = 0;
      await tester.pumpWidget(
        host('```rust\nlet x', streaming: true, onRendered: () => rendered++),
      );
      await tester.pumpWidget(
        host(
          '```rust\nlet x = 1;',
          streaming: true,
          onRendered: () => rendered++,
        ),
      );
      await tester.pump(const Duration(milliseconds: 40));
      expect(selectedText(tester), contains('let x'));
      expect(selectedText(tester), isNot(contains('= 1')));
      await tester.pumpWidget(
        host(
          '```rust\nlet x = 2;',
          streaming: true,
          onRendered: () => rendered++,
        ),
      );
      await tester.pump(const Duration(milliseconds: 40));
      expect(rendered, 1);
      expect(selectedText(tester), contains('let x = 2;'));
      await tester.pumpWidget(
        host('```rust\nlet x = 3;\n```', streaming: true),
      );
      await tester.pumpWidget(
        host('```rust\nlet x = 4;\n```'),
      ); // immediate final flush
      expect(selectedText(tester), contains('let x = 4;'));
      await tester.pumpWidget(host('partial', streaming: true));
      await tester.pumpWidget(host('partial update', streaming: true));
      await tester.pumpWidget(const SizedBox());
      await tester.pump(const Duration(milliseconds: 100));
      expect(tester.takeException(), isNull);
    },
  );

  testWidgets(
    'Completed replies reuse the parsed widget; long replies keep their full source',
    (tester) async {
      await tester.pumpWidget(host('**Completed** reply'));
      final first = tester.widget<MarkdownBody>(find.byType(MarkdownBody));
      await tester.pumpWidget(host('**Completed** reply'));
      expect(
        identical(
          first,
          tester.widget<MarkdownBody>(find.byType(MarkdownBody)),
        ),
        isTrue,
      );
      final long = '${'long ' * 7000}END';
      await tester.pumpWidget(host(long));
      expect(find.byType(MarkdownBody), findsNothing);
      expect(selectedText(tester), long);
      expect(find.text('Long reply shown as plain text.'), findsOneWidget);
      expect(tester.takeException(), isNull);
    },
  );

  testWidgets(
    'Chat renders only assistant replies and whole-message copy keeps Markdown source',
    (tester) async {
      String? copied;
      tester.binding.defaultBinaryMessenger.setMockMethodCallHandler(
        SystemChannels.platform,
        (call) async {
          if (call.method == 'Clipboard.setData') {
            copied = (call.arguments as Map)['text'] as String;
          }
          return null;
        },
      );
      addTearDown(
        () => tester.binding.defaultBinaryMessenger.setMockMethodCallHandler(
          SystemChannels.platform,
          null,
        ),
      );
      final chat = ChatController(_NoBridge())..loading = false;
      chat.messages = [
        {'id': 1, 'role': 'user', 'content': '**literal prompt**'},
        {'id': 2, 'role': 'assistant', 'content': '**Formatted reply**'},
      ];
      await tester.pumpWidget(DoloresApp(chat: chat));
      await tester.pumpAndSettle();
      expect(find.text('**literal prompt**'), findsOneWidget);
      expect(find.byType(ReplyContent), findsOneWidget);
      final mouse = await tester.createGesture(kind: PointerDeviceKind.mouse);
      await mouse.addPointer(location: Offset.zero);
      await mouse.moveTo(tester.getCenter(find.byType(MessageFrame).last));
      await tester.pump();
      await tester.tap(find.byTooltip('Copy message').hitTestable().last);
      await tester.pump();
      expect(copied, '**Formatted reply**');
      await mouse.removePointer();
      await tester.pumpWidget(const SizedBox());
      chat.dispose();
    },
  );

  testWidgets(
    'Coalesced layout follows the reply unless the reader scrolls away',
    (tester) async {
      final chat = ChatController(_NoBridge())
        ..loading = false
        ..busy = true
        ..pendingInput = 'A long answer'
        ..partial = 'start\n\n${'paragraph\n\n' * 40}';
      await tester.pumpWidget(DoloresApp(chat: chat));
      await tester.pumpAndSettle();
      final scroll = tester
          .widget<ListView>(find.byType(ListView).last)
          .controller!;
      scroll.jumpTo(scroll.position.maxScrollExtent);
      chat.partial += '\n\n```rust\n${'let x = 1;\n' * 15}';
      chat.notifyListeners();
      await tester.pump();
      await tester.pump(const Duration(milliseconds: 80));
      await tester.pump();
      expect(scroll.position.extentAfter, lessThan(1));
      scroll.jumpTo(0);
      chat.partial += '\nmore code\n```';
      chat.notifyListeners();
      await tester.pump();
      await tester.pump(const Duration(milliseconds: 80));
      await tester.pump();
      expect(scroll.offset, 0);
      await tester.pumpWidget(const SizedBox());
      chat.dispose();
      expect(tester.takeException(), isNull);
    },
  );
}

class _NoBridge implements ChatBridge {
  @override
  Future<dynamic> call(Map<String, dynamic> command) async => null;
  @override
  Future<void> open() async {}
  @override
  Future<void> close() async {}
}
