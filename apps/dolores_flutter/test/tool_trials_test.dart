import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:dolores_flutter/bridge.dart';
import 'package:dolores_flutter/chat.dart';
import 'package:dolores_flutter/tool_trials.dart';

class TrialBridge implements ChatBridge {
  @override
  Future<void> open() async {}
  @override
  Future<void> close() async {}
  @override
  Future<dynamic> call(Map<String, dynamic> v) async {
    if (v['command'] == 'trialSources') {
      return {
        'model': 'fixture',
        'suite': 'config-command-v1',
        'skills': [
          {
            'name': 'check',
            'enabled': true,
            'revision': 1,
            'versions': [
              {
                'document': {'text': 'Original skill'},
              },
            ],
          },
        ],
        'trials': [],
      };
    }
    if (v['command'] == 'startToolTrial') {
      throw 'Skill changed. Refresh before testing.';
    }
    return null;
  }
}

void main() {
  testWidgets(
    'failed preparation retains candidate and refresh does not replay',
    (tester) async {
      final chat = ChatController(TrialBridge())
        ..session = 'task'
        ..workspaceRoot = 'fixture';
      addTearDown(chat.dispose);
      await tester.pumpWidget(
        MaterialApp(
          home: Scaffold(body: ToolTrialsInspector(chat: chat)),
        ),
      );
      await tester.pumpAndSettle();
      await tester.tap(find.byType(DropdownButtonFormField<String>));
      await tester.pumpAndSettle();
      await tester.tap(find.text('check').last);
      await tester.pumpAndSettle();
      await tester.enterText(
        find.byKey(const Key('trial-candidate')),
        'Edited candidate',
      );
      await tester.tap(find.text('Compare with tools'));
      await tester.pumpAndSettle();
      await tester.drag(find.byType(ListView), const Offset(0, -250));
      await tester.pumpAndSettle();
      expect(find.textContaining('Skill changed'), findsOneWidget);
      expect(
        tester
            .widget<TextField>(find.byKey(const Key('trial-candidate')))
            .controller!
            .text,
        'Edited candidate',
      );
      await tester.tap(find.text('Refresh'));
      await tester.pumpAndSettle();
      expect(
        tester
            .widget<TextField>(find.byKey(const Key('trial-candidate')))
            .controller!
            .text,
        'Edited candidate',
      );
    },
  );
  testWidgets('side chat prerequisite in compact light and dark', (
    tester,
  ) async {
    tester.view.physicalSize = const Size(520, 650);
    tester.view.devicePixelRatio = 1;
    addTearDown(tester.view.resetPhysicalSize);
    addTearDown(tester.view.resetDevicePixelRatio);
    final chat = ChatController(TrialBridge())..session = 'task';
    addTearDown(chat.dispose);
    for (final brightness in Brightness.values) {
      await tester.pumpWidget(
        MaterialApp(
          theme: ThemeData(brightness: brightness),
          home: Scaffold(body: ToolTrialsInspector(chat: chat)),
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
