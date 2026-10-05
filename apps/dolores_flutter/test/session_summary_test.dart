import 'package:dolores_flutter/session_summary.dart';
import 'package:dolores_flutter/theme.dart';
import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';

import 'memory_test.dart' as manual;

class SummaryBridge extends manual.MemoryBridge {
  Map<String, dynamic>? summary;
  bool slow = false, stopped = false, empty = false;
  int generations = 0;
  @override
  Future<dynamic> call(Map<String, dynamic> command) async {
    commands.add(command);
    switch (command['command']) {
      case 'reviewSummary':
        return {
          'token': 'source',
          'summary': summary,
          'model': 'fixture',
          'hasMore': true,
          'messages': empty
              ? []
              : [
                  {'id': 1, 'role': 'user', 'content': 'Goal: Unicode parser.'},
                  {
                    'id': 2,
                    'role': 'assistant',
                    'content': 'Next: test Unicode errors.',
                  },
                ],
        };
      case 'generateSummary':
        generations++;
        return null;
      case 'poll':
        if (slow && !stopped) return [];
        return [
          {
            'type': 'done',
            'id': command['id'],
            if (stopped)
              'error': 'Summary stopped. Nothing was saved.'
            else
              'summaryDraft': {
                'token': 'draft',
                'text': 'Goal: parser. Next: tests.',
                'model': 'fixture',
              },
          },
        ];
      case 'cancel':
        stopped = true;
        return null;
      case 'saveSummary':
      case 'correctSummary':
        if (this.fail) throw 'Storage failed. Nothing was saved.';
        summary = {
          'text': command['text'],
          'provenance': {
            'revision': (summary?['provenance']['revision'] as int? ?? 0) + 1,
            'coveredTurns': 1,
            'coveredThrough': 2,
            'model': 'fixture',
            'updatedAt': DateTime(2026, 10, 3).millisecondsSinceEpoch,
          },
        };
        return summary;
      case 'deleteSummary':
        summary = null;
        return null;
    }
    return null;
  }
}

Future<void> open(
  WidgetTester tester,
  SummaryBridge bridge, {
  bool dark = true,
}) async {
  tester.view.physicalSize = const Size(390, 700);
  tester.view.devicePixelRatio = 1;
  addTearDown(tester.view.resetPhysicalSize);
  addTearDown(tester.view.resetDevicePixelRatio);
  await tester.pumpWidget(
    MaterialApp(
      theme: doloresTheme(dark),
      home: Scaffold(
        body: Builder(
          builder: (context) => TextButton(
            onPressed: () => showDialog<void>(
              context: context,
              builder: (_) =>
                  SessionSummaryInspector(bridge: bridge, session: 'chat'),
            ),
            child: const Text('Open'),
          ),
        ),
      ),
    ),
  );
  await tester.tap(find.text('Open'));
  await tester.pumpAndSettle();
  await tester.tap(find.byKey(const Key('review-summary')));
  await tester.pumpAndSettle();
}

void main() {
  testWidgets(
    'review is local; corrected draft is explicitly saved, revised and deleted in compact dark UI',
    (tester) async {
      final bridge = SummaryBridge();
      await open(tester, bridge);
      expect(bridge.generations, 0);
      expect(bridge.summary, isNull);
      expect(find.textContaining('More turns remain'), findsOneWidget);
      await tester.ensureVisible(find.byKey(const Key('summary-source-1')));
      await tester.pumpAndSettle();
      await tester.tap(find.byKey(const Key('summary-source-1')));
      await tester.pumpAndSettle();
      expect(find.text('Goal: Unicode parser.'), findsOneWidget);
      await tester.tap(find.byKey(const Key('generate-summary')));
      await tester.pumpAndSettle();
      expect(bridge.summary, isNull);
      await manual.enter(
        tester,
        'summary-text',
        'Goal: Unicode parser. Next: error tests.',
      );
      await tester.tap(find.byKey(const Key('save-summary')));
      await tester.pumpAndSettle();
      expect(
        bridge.summary!['text'],
        'Goal: Unicode parser. Next: error tests.',
      );
      expect(find.textContaining('Drafted by fixture'), findsOneWidget);
      await tester.tap(find.byKey(const Key('edit-summary')));
      await tester.pumpAndSettle();
      await manual.enter(tester, 'summary-text', 'Corrected task');
      await tester.tap(find.byKey(const Key('save-summary')));
      await tester.pumpAndSettle();
      expect(bridge.summary!['provenance']['revision'], 2);
      await tester.tap(find.byKey(const Key('delete-summary')));
      await tester.pumpAndSettle();
      expect(bridge.summary, isNull);
      expect(bridge.generations, 1);
      expect(tester.takeException(), isNull);
    },
  );
  testWidgets('failed save keeps correction and Discard or Close never saves', (
    tester,
  ) async {
    final bridge = SummaryBridge()..fail = true;
    await open(tester, bridge, dark: false);
    await tester.tap(find.byKey(const Key('generate-summary')));
    await tester.pumpAndSettle();
    await manual.enter(tester, 'summary-text', 'Keep my correction');
    await tester.tap(find.byKey(const Key('save-summary')));
    await tester.pumpAndSettle();
    expect(
      tester
          .widget<TextField>(find.byKey(const Key('summary-text')))
          .controller!
          .text,
      'Keep my correction',
    );
    await tester.tap(find.byKey(const Key('discard-summary')));
    await tester.pumpAndSettle();
    await tester.tap(find.text('Close'));
    await tester.pumpAndSettle();
    expect(bridge.summary, isNull);
    expect(bridge.generations, 1);
    expect(
      bridge.commands.where((c) => c['command'] == 'discardSummaryReview'),
      isNotEmpty,
    );
    expect(tester.takeException(), isNull);
  });
  testWidgets(
    'Stop stays available, blocks Close until done and never retries',
    (tester) async {
      final bridge = SummaryBridge()..slow = true;
      await open(tester, bridge);
      await tester.tap(find.byKey(const Key('generate-summary')));
      await tester.pump(const Duration(milliseconds: 100));
      expect(
        tester
            .widget<TextButton>(find.widgetWithText(TextButton, 'Close'))
            .onPressed,
        isNull,
      );
      await tester.tap(find.byKey(const Key('stop-summary')));
      await tester.pumpAndSettle();
      expect(bridge.generations, 1);
      expect(bridge.summary, isNull);
      expect(find.textContaining('Nothing was saved'), findsOneWidget);
      expect(tester.takeException(), isNull);
    },
  );
  testWidgets('empty history cannot generate', (tester) async {
    final bridge = SummaryBridge()..empty = true;
    await open(tester, bridge);
    expect(
      tester
          .widget<FilledButton>(find.byKey(const Key('generate-summary')))
          .onPressed,
      isNull,
    );
    expect(bridge.generations, 0);
    expect(tester.takeException(), isNull);
  });
}
