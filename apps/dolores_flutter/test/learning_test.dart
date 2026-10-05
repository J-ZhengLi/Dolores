import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:dolores_flutter/bridge.dart';
import 'package:dolores_flutter/chat.dart';
import 'package:dolores_flutter/learning.dart';

class LearningBridge implements ChatBridge {
  int writes = 0;
  @override
  Future<void> open() async {}
  @override
  Future<void> close() async {}
  @override
  Future<dynamic> call(Map<String, dynamic> v) async {
    if (v['command'] == 'createCheckWorkflow') {
      writes++;
      throw 'Skill changed. Refresh before retrying.';
    }
    return {
      'state': {
        'revision': 1,
        'enabled': false,
        'automatic': false,
        'paused': false,
        'events': [],
      },
      'trials': [],
      'workflowAvailable': true,
    };
  }
}

void main() {
  testWidgets(
    'stale workflow save preserves draft and refresh does not replay',
    (tester) async {
      final bridge = LearningBridge();
      final chat = ChatController(bridge)
        ..session = 'task'
        ..workspaceRoot = 'fixture';
      addTearDown(chat.dispose);
      await tester.pumpWidget(
        MaterialApp(
          home: Scaffold(body: LearningInspector(chat: chat)),
        ),
      );
      await tester.pumpAndSettle();
      await tester.scrollUntilVisible(
        find.byKey(const Key('learning-command')),
        300,
      );
      await tester.enterText(
        find.byKey(const Key('learning-command')),
        'node custom.cjs',
      );
      await tester.ensureVisible(find.text('Create check workflow'));
      await tester.tap(find.text('Create check workflow'));
      await tester.pumpAndSettle();
      expect(bridge.writes, 1);
      expect(
        tester
            .widget<TextField>(find.byKey(const Key('learning-command')))
            .controller!
            .text,
        'node custom.cjs',
      );
      await tester.tap(find.text('Refresh'));
      await tester.pumpAndSettle();
      expect(bridge.writes, 1);
      expect(
        tester
            .widget<TextField>(find.byKey(const Key('learning-command')))
            .controller!
            .text,
        'node custom.cjs',
      );
    },
  );
  testWidgets('side chat explains prerequisites in compact themes', (
    tester,
  ) async {
    tester.view.physicalSize = const Size(520, 650);
    tester.view.devicePixelRatio = 1;
    addTearDown(tester.view.resetPhysicalSize);
    addTearDown(tester.view.resetDevicePixelRatio);
    final chat = ChatController(LearningBridge())..session = 'task';
    addTearDown(chat.dispose);
    for (final brightness in Brightness.values) {
      await tester.pumpWidget(
        MaterialApp(
          theme: ThemeData(brightness: brightness),
          home: Scaffold(body: LearningInspector(chat: chat)),
        ),
      );
      await tester.pumpAndSettle();
      expect(
        find.text('Open a project or temporary working chat first.'),
        findsOneWidget,
      );
      expect(tester.takeException(), isNull);
    }
  });
}
