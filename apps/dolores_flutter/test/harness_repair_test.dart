import 'dart:convert';

import 'package:dolores_flutter/chat.dart';
import 'package:dolores_flutter/tool_activity.dart';
import 'package:dolores_flutter/edit_diff.dart';
import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';

import 'history_test.dart' show HistoryBridge;

void main() {
  testWidgets(
    'native execution has separate review and bounded account-access disclosure',
    (tester) async {
      tester.view.physicalSize = const Size(420, 720);
      tester.view.devicePixelRatio = 1;
      addTearDown(tester.view.resetPhysicalSize);
      addTearDown(tester.view.resetDevicePixelRatio);
      final chat = ChatController(HistoryBridge())
        ..model = 'fixture'
        ..toolApproval = {
          'name': 'test_harness_repair',
          'target': 'Dolores native evaluation',
          'callId': 'trial',
          'query': '{"repairId":"retained","revision":2,"commands":{"baseline":"cargo test --offline --locked --workspace --lib","candidate":"same frozen suite"}}',
          'diff': '--- before\n+++ after\n@@ -1 +1 @@\n-old\n+candidate\n',
        };
      await tester.pumpWidget(
        MaterialApp(
          home: Scaffold(body: ToolApprovalCard(chat: chat)),
        ),
      );
      await tester.pumpAndSettle();
      expect(find.text('Build and test this repair?'), findsOneWidget);
      expect(find.text('Run once'), findsOneWidget);
      await tester.ensureVisible(find.text('Operation details'));
      await tester.pumpAndSettle();
      await tester.tap(find.text('Operation details'));
      await tester.pumpAndSettle();
      expect(find.textContaining('not an OS sandbox'), findsOneWidget);
      expect(find.textContaining('300 seconds and 256 KiB'), findsOneWidget);
      expect(
        find.textContaining('cannot install, restart or replay'),
        findsOneWidget,
      );
      expect(find.byType(EditDiff), findsOneWidget);
      expect(tester.takeException(), isNull);
      await tester.pumpWidget(const SizedBox());
      chat.dispose();
    },
  );
  test('withheld native evidence explains baseline passing and preserves next step', () {
    final text = toolResultText({
      'name': 'test_harness_repair',
      'status': 'completed',
      'content': jsonEncode({
        'evaluation': {
          'status': 'withheld',
          'baseline': {
            'passed': ['check'],
            'failed': [],
          },
          'candidate': null,
          'note': 'Baseline already passes. Candidate not executed; reproduce the actual fault with frozen criteria.',
        },
      }),
    });
    expect(text, contains('withheld'));
    expect(text, contains('Candidate: not run'));
    expect(text, contains('frozen criteria'));
    expect(
      toolStatus({
        'name': 'test_harness_repair',
        'status': 'completed',
        'content': '{"evaluation":{"status":"withheld"}}',
      }),
      'Improvement withheld',
    );
  });
  testWidgets(
    'repair approval shows separate storage and exact diff in compact themes',
    (tester) async {
      tester.view.physicalSize = const Size(420, 720);
      tester.view.devicePixelRatio = 1;
      addTearDown(tester.view.resetPhysicalSize);
      addTearDown(tester.view.resetDevicePixelRatio);
      for (final dark in [false, true]) {
        final chat = ChatController(HistoryBridge())
          ..model = 'fixture'
          ..toolApproval = {
            'name': 'harness_repair',
            'target': 'Dolores managed repair',
            'callId': 'patch',
            'query': '{"action":"propose","repairId":"retained"}',
            'diff': '--- before\n+++ after\n@@ -1 +1 @@\n-old behavior\n+proposed behavior\n',
          };
        await tester.pumpWidget(
          MaterialApp(
            theme: ThemeData(
              brightness: dark ? Brightness.dark : Brightness.light,
            ),
            home: Scaffold(body: ToolApprovalCard(chat: chat)),
          ),
        );
        await tester.pumpAndSettle();
        expect(find.text('Review this repair step?'), findsOneWidget);
        await tester.tap(find.text('Operation details'));
        await tester.pumpAndSettle();
        expect(
          find.textContaining('Your project stays unchanged'),
          findsOneWidget,
        );
        expect(
          find.textContaining('cannot compile, execute tests or install'),
          findsOneWidget,
        );
        expect(find.text('Exact repair step:'), findsOneWidget);
        expect(find.byType(EditDiff), findsOneWidget);
        expect(find.text('Run once'), findsNothing);
        expect(tester.takeException(), isNull);
        await tester.pumpWidget(const SizedBox());
        chat.dispose();
      }
    },
  );
  testWidgets(
    'retained repair receipt explains changed artifact without implying improvement',
    (tester) async {
      final content = jsonEncode({
        'repairId': 'retained',
        'revision': 2,
        'status': 'proposed',
        'sourceMatch': 'matches',
        'artifactIntegrity':
            'Repair artifact changed. Inspect retained snapshots.',
        'files': [
          {'path': 'agent.rs', 'changed': true, 'protectedReview': true},
        ],
        'note': 'Nothing compiled, executed or installed.',
      });
      final record = {
        'callId': 'patch',
        'name': 'harness_repair',
        'target': 'Dolores managed repair',
        'status': 'completed',
        'content': content,
      };
      await tester.pumpWidget(
        MaterialApp(
          home: Scaffold(body: ToolRecords(records: [record])),
        ),
      );
      expect(find.text('Repair proposal · completed'), findsOneWidget);
      await tester.tap(find.text('Dolores managed repair'));
      await tester.pumpAndSettle();
      expect(find.textContaining('Repair artifact changed'), findsOneWidget);
      expect(
        find.textContaining('Nothing compiled, executed or installed'),
        findsOneWidget,
      );
      expect(find.textContaining('protected review'), findsOneWidget);
      expect(tester.takeException(), isNull);
    },
  );
}
