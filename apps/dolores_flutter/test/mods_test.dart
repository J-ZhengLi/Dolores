import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:dolores_flutter/bridge.dart';
import 'package:dolores_flutter/chat.dart';
import 'package:dolores_flutter/mods.dart';

class ModsBridge implements ChatBridge {
  int tests = 0, drafts = 0;
  @override
  Future<void> open() async {}
  @override
  Future<void> close() async {}
  @override
  Future<dynamic> call(Map<String, dynamic> request) async {
    switch (request['command']) {
      case 'testMod':
        tests++;
        throw 'Mods changed. Refresh and test again.';
      case 'generateMod':
        drafts++;
        return null;
      case 'poll':
        return [
          {
            'type': 'done',
            'id': request['id'],
            'source': '(module',
            'error': 'Mod draft exhausted its output tokens. Partial source retained.',
          },
        ];
      default:
        return {
          'state': {
            'revision': 1,
            'automatic': false,
            'active': null,
            'versions': [],
            'events': [],
          },
          'card': {
            'title': 'Recovery guidance',
            'text': 'Inspect before retrying.',
          },
          'boundary': 'fixture',
        };
    }
  }
}

void main() {
  testWidgets(
    'stale test preserves editable source and refresh never replays',
    (tester) async {
      final bridge = ModsBridge();
      final chat = ChatController(bridge)
        ..session = 'fixture'
        ..workspaceRoot = 'fixture';
      addTearDown(chat.dispose);
      await tester.pumpWidget(
        MaterialApp(
          home: Scaffold(body: ModsInspector(chat: chat)),
        ),
      );
      await tester.pumpAndSettle();
      await tester.scrollUntilVisible(
        find.byKey(const Key('mod-source')),
        150,
        scrollable: find.byType(Scrollable).first,
      );
      await tester.enterText(
        find.byKey(const Key('mod-source')),
        'my retained source',
      );
      await tester.tap(find.text('Test source'));
      await tester.pumpAndSettle();
      expect(bridge.tests, 1);
      expect(find.textContaining('Mods changed.'), findsOneWidget);
      await tester.tap(find.text('Refresh'));
      await tester.pumpAndSettle();
      expect(bridge.tests, 1);
      expect(
        tester
            .widget<TextField>(find.byKey(const Key('mod-source')))
            .controller!
            .text,
        'my retained source',
      );
    },
  );
  testWidgets(
    'truncated generation preserves partial draft and actionable footer',
    (tester) async {
      final bridge = ModsBridge();
      final chat = ChatController(bridge)
        ..session = 'fixture'
        ..workspaceRoot = 'fixture';
      addTearDown(chat.dispose);
      await tester.pumpWidget(
        MaterialApp(
          theme: ThemeData.light(),
          home: Scaffold(body: ModsInspector(chat: chat)),
        ),
      );
      await tester.pumpAndSettle();
      await tester.tap(find.text('Draft a repair'));
      await tester.pumpAndSettle();
      expect(bridge.drafts, 1);
      expect(bridge.tests, 0);
      expect(find.textContaining('output tokens'), findsOneWidget);
      await tester.scrollUntilVisible(
        find.byKey(const Key('mod-source')),
        150,
        scrollable: find.byType(Scrollable).first,
      );
      expect(
        tester
            .widget<TextField>(find.byKey(const Key('mod-source')))
            .controller!
            .text,
        '(module',
      );
      expect(tester.takeException(), isNull);
    },
  );
}
