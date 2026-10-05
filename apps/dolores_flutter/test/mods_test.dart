import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:dolores_flutter/bridge.dart';
import 'package:dolores_flutter/chat.dart';
import 'package:dolores_flutter/mods.dart';

class ModsBridge implements ChatBridge {
  int tests = 0, drafts = 0;
  bool emptyStopped = false;
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
            'source': emptyStopped ? '' : '(module',
            'error': emptyStopped ? 'Mod draft stopped. Baseline unchanged.' : 'Mod draft exhausted its output tokens. Partial source retained.',
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
            'recoveryReceipt': 'Restored baseline; quarantined fixture.',
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
  testWidgets(
    'compact recovery keeps its receipt, footer and prior source after an empty Stop',
    (tester) async {
      tester.view.physicalSize = const Size(800, 600);
      tester.view.devicePixelRatio = 1;
      addTearDown(tester.view.resetPhysicalSize);
      addTearDown(tester.view.resetDevicePixelRatio);
      final bridge = ModsBridge()..emptyStopped = true;
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
      expect(find.textContaining('quarantined fixture'), findsOneWidget);
      await tester.scrollUntilVisible(
        find.byKey(const Key('mod-source')),
        150,
        scrollable: find.byType(Scrollable).first,
      );
      await tester.enterText(
        find.byKey(const Key('mod-source')),
        'my previous editable source',
      );
      await tester.tap(find.text('Draft a repair'));
      await tester.pumpAndSettle();
      expect(find.textContaining('Mod draft stopped.'), findsOneWidget);
      expect(
        tester
            .widget<TextField>(find.byKey(const Key('mod-source')))
            .controller!
            .text,
        'my previous editable source',
      );
      expect(tester.getRect(find.text('Test source')).bottom, lessThan(600));
      expect(bridge.drafts, 1);
      expect(tester.takeException(), isNull);
    },
  );
}
