import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:dolores_flutter/chat.dart';
import 'package:dolores_flutter/comparison.dart';
import 'package:dolores_flutter/theme.dart';

import 'history_test.dart';

class ComparisonBridge extends HistoryBridge {
  bool reject = false, hold = false, cancelled = false;
  Map<String, dynamic>? started;
  @override
  Future<dynamic> call(Map<String, dynamic> command) async {
    commands.add(command);
    switch (command['command']) {
      case 'comparisonSources':
        return {
          'items': [],
          'model': 'original-model',
          'settings': {'maxOutputTokens': 512, 'timeoutSeconds': 10},
        };
      case 'comparisonsPage':
        return {'items': [], 'hasOlder': false};
      case 'startComparison':
        if (reject) throw 'Instruction snapshot changed. Refresh sources or use a labeled manual copy before comparing.';
        started = command;
        cancelled = false;
        return {'comparisonId': 1};
      case 'cancel':
        cancelled = true;
        return null;
      case 'poll':
        if (hold && !cancelled) return [];
        final draft = started!['draft'];
        return [
          {
            'type': 'done',
            'persisted': true,
            'comparison': {
              'id': 1,
              'revision': 3,
              'model': 'original-model',
              'settings': started!['settings'],
              'draft': draft,
              'status': cancelled ? 'stopped' : 'failed',
              'baselineMessages': [],
              'candidateMessages': [],
              'summary': {
                'baselinePassed': 0,
                'candidatePassed': 0,
                'tests': 1,
                'improved': false,
              },
              'results': [
                {
                  'outcome': 'completed',
                  'output': 'retained baseline',
                  'elapsedMs': 1,
                  'usage': null,
                },
                {
                  'outcome': cancelled ? 'stopped' : 'outputLimit',
                  'output': 'PASS',
                  'elapsedMs': 1,
                  'usage': null,
                  'detail': cancelled
                      ? 'Stopped. Earlier completed responses remain.'
                      : 'Output limit: 512 tokens. Partial response is not a passing test.',
                },
              ],
            },
          },
        ];
    }
    return null;
  }
}

Future<void> fill(WidgetTester tester) async {
  for (final (key, text) in [
    ('comparison-candidate', 'Include PASS'),
    ('comparison-prompt-0', 'Respond briefly'),
    ('comparison-required-0', 'PASS'),
  ]) {
    await tester.scrollUntilVisible(
      find.byKey(Key(key)),
      150,
      scrollable: find.byType(Scrollable).first,
    );
    await tester.enterText(find.byKey(Key(key)), text);
    await tester.pump();
  }
}

void main() {
  for (final dark in [false, true]) {
    testWidgets(
      'compact comparison refusal keeps draft and limit receipt never improves ${dark ? 'dark' : 'light'}',
      (tester) async {
        tester.view.physicalSize = const Size(620, 700);
        tester.view.devicePixelRatio = 1;
        addTearDown(tester.view.resetPhysicalSize);
        addTearDown(tester.view.resetDevicePixelRatio);
        final bridge = ComparisonBridge()..reject = true;
        final chat = ChatController(bridge)
          ..session = 'task'
          ..loading = false
          ..configured = true;
        addTearDown(chat.dispose);
        await tester.pumpWidget(
          MaterialApp(
            theme: doloresTheme(dark),
            home: ComparisonInspector(chat: chat),
          ),
        );
        await tester.pumpAndSettle();
        await fill(tester);
        await tester.tap(find.byKey(const Key('comparison-evaluate')));
        await tester.pumpAndSettle();
        expect(
          find.textContaining('Instruction snapshot changed'),
          findsOneWidget,
        );
        await tester.scrollUntilVisible(
          find.byKey(const Key('comparison-candidate')),
          150,
          scrollable: find.byType(Scrollable).first,
        );
        expect(
          tester
              .widget<TextField>(find.byKey(const Key('comparison-candidate')))
              .controller!
              .text,
          'Include PASS',
        );
        bridge.reject = false;
        await tester.tap(find.byKey(const Key('comparison-evaluate')));
        await tester.pumpAndSettle();
        expect(find.text('Baseline 0/1 · Candidate 0/1'), findsOneWidget);
        expect(
          find.textContaining('No complete strict improvement'),
          findsOneWidget,
        );
        await tester.scrollUntilVisible(
          find.text('Test 1 · Candidate · outputLimit'),
          150,
          scrollable: find.byType(Scrollable).first,
        );
        await tester.tap(find.text('Test 1 · Candidate · outputLimit'));
        await tester.pumpAndSettle();
        expect(
          find.textContaining('Partial response is not a passing test'),
          findsOneWidget,
        );
        expect(find.text('Use as draft'), findsOneWidget);
        await tester.tap(find.text('Use as draft'));
        await tester.pumpAndSettle();
        await tester.drag(find.byType(ListView).first, const Offset(0, 10000));
        await tester.pumpAndSettle();
        await tester.scrollUntilVisible(
          find.byKey(const Key('comparison-candidate')),
          150,
          scrollable: find.byType(Scrollable).first,
        );
        expect(
          tester
              .widget<TextField>(find.byKey(const Key('comparison-candidate')))
              .controller!
              .text,
          'Include PASS',
        );
        expect(tester.takeException(), isNull);
      },
    );
  }
  testWidgets(
    'pending comparison locks Close, excludes duplicate runs, Stop retains baseline',
    (tester) async {
      final bridge = ComparisonBridge()..hold = true;
      final chat = ChatController(bridge)
        ..session = 'task'
        ..loading = false
        ..configured = true;
      addTearDown(chat.dispose);
      await tester.pumpWidget(
        MaterialApp(home: ComparisonInspector(chat: chat)),
      );
      await tester.pumpAndSettle();
      await fill(tester);
      await tester.tap(find.byKey(const Key('comparison-evaluate')));
      await tester.pump();
      expect(find.text('Stop'), findsOneWidget);
      expect(
        tester
            .widget<TextButton>(find.widgetWithText(TextButton, 'Close'))
            .onPressed,
        isNull,
      );
      expect(
        bridge.commands.where((c) => c['command'] == 'startComparison').length,
        1,
      );
      await tester.tap(find.text('Stop'));
      await tester.pump(const Duration(milliseconds: 100));
      await tester.pumpAndSettle();
      expect(find.text('Test 1 · Baseline · completed'), findsOneWidget);
      expect(
        find.textContaining('No complete strict improvement'),
        findsOneWidget,
      );
      expect(tester.takeException(), isNull);
    },
  );
}
