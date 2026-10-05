import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:dolores_flutter/bridge.dart';
import 'package:dolores_flutter/chat.dart';
import 'package:dolores_flutter/knowledge.dart';

class KnowledgeBridge implements ChatBridge {
  bool fail = true;
  int saves = 0;
  final state = <String, dynamic>{
    'revision': 1,
    'learning': false,
    'shareFeedback': false,
    'facts': <Map>[],
    'notice': '',
  };
  @override
  Future<void> open() async {}
  @override
  Future<void> close() async {}
  @override
  Future<dynamic> call(Map<String, dynamic> value) async {
    if (value['command'] == 'saveKnowledgeFact') {
      saves++;
      if (fail) throw 'Knowledge revision changed.';
      state['facts'] = [
        {
          'id': 'fact',
          'title': value['title'],
          'text': value['text'],
          'basis': 'manual',
          'kind': value['kind'],
          'enabled': true,
          'protected': true,
          'fresh': true,
        },
      ];
      state['revision'] = 2;
    }
    return Map<String, dynamic>.from(state);
  }
}

void main() {
  testWidgets(
    'stale save preserves correction and explicit refresh does not replay it',
    (tester) async {
      final bridge = KnowledgeBridge();
      final chat = ChatController(bridge)
        ..session = 'fixture'
        ..workspaceRoot = 'fixture';
      addTearDown(chat.dispose);
      await tester.pumpWidget(
        MaterialApp(
          home: Scaffold(body: KnowledgeInspector(chat: chat)),
        ),
      );
      await tester.pumpAndSettle();
      expect(find.text('Learn from approved task evidence'), findsOneWidget);
      await tester.tap(find.text('New fact'));
      await tester.pumpAndSettle();
      await tester.enterText(
        find.byType(TextField).first,
        'Project test command',
      );
      await tester.enterText(
        find.byType(TextField).last,
        'Use the corrected test command.',
      );
      await tester.tap(find.text('Save correction'));
      await tester.pumpAndSettle();
      expect(bridge.saves, 1);
      expect(find.text('Use the corrected test command.'), findsOneWidget);
      await tester.tap(find.text('Refresh'));
      await tester.pumpAndSettle();
      expect(bridge.saves, 1);
      bridge.fail = false;
      await tester.tap(find.text('Save correction'));
      await tester.pumpAndSettle();
      expect(bridge.saves, 2);
      expect(find.textContaining('Protected correction'), findsOneWidget);
      expect(tester.takeException(), isNull);
    },
  );
  testWidgets(
    'compact themes show prerequisites and keep recovery controls reachable',
    (tester) async {
      tester.view.physicalSize = const Size(420, 480);
      tester.view.devicePixelRatio = 1;
      addTearDown(tester.view.resetPhysicalSize);
      addTearDown(tester.view.resetDevicePixelRatio);
      final chat = ChatController(KnowledgeBridge());
      addTearDown(chat.dispose);
      for (final brightness in Brightness.values) {
        await tester.pumpWidget(
          MaterialApp(
            theme: ThemeData(brightness: brightness),
            home: Scaffold(body: KnowledgeInspector(chat: chat)),
          ),
        );
        await tester.pumpAndSettle();
        expect(find.textContaining('Project facts belong'), findsOneWidget);
        expect(find.text('Start working session'), findsOneWidget);
        expect(tester.takeException(), isNull);
      }
    },
  );
}
