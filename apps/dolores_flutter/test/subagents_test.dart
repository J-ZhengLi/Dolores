import 'dart:convert';
import 'dart:async';

import 'package:dolores_flutter/main.dart';
import 'package:dolores_flutter/tool_activity.dart';
import 'package:dolores_flutter/subagents.dart';
import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';

import 'tools_test.dart' show ToolBridge, ready, compact;

Map<String, dynamic> child(String id, String state) => {
  'childId': id,
  'goal': 'Read $id.txt',
  'scope': '$id.txt',
  'readOnly': true,
  'status': state,
  'verified': false,
};

void main() {
  testWidgets('child inspector retries pages and closes with a pending read', (
    t,
  ) async {
    compact(t);
    var failRead = true;
    final pending = Completer<Map<String, dynamic>>();
    var stall = false;
    final calls = <String>[];
    await t.pumpWidget(
      MaterialApp(
        home: Scaffold(
          body: SingleChildScrollView(
            child: SubagentCards(
              children: [child('owned', 'running')],
              onSupervise: (id, action, {offset}) async {
                calls.add('$id:$action:${offset ?? 0}');
                if (action == 'interrupt') {
                  throw StateError(
                    'Child already settled; inspect its report.',
                  );
                }
                if (stall) return pending.future;
                if (failRead) {
                  failRead = false;
                  throw StateError('Read failed; saved child work remains.');
                }
                return {
                  'context': offset == 4 ? 'SECOND_PAGE' : 'FIRST_PAGE',
                  'nextOffset': offset == 4 ? null : 4,
                };
              },
            ),
          ),
        ),
      ),
    );
    await t.tap(find.text('Read owned.txt'));
    await t.pumpAndSettle();
    await t.ensureVisible(find.text('Inspect child context'));
    await t.tap(find.text('Inspect child context'));
    await t.pumpAndSettle();
    expect(find.textContaining('saved child work remains'), findsOneWidget);
    await t.tap(find.text('Refresh'));
    await t.pumpAndSettle();
    expect(find.textContaining('FIRST_PAGE'), findsOneWidget);
    await t.tap(find.text('Next page'));
    await t.pumpAndSettle();
    expect(find.textContaining('SECOND_PAGE'), findsOneWidget);
    await t.tap(find.text('Previous page'));
    await t.pumpAndSettle();
    expect(find.textContaining('FIRST_PAGE'), findsOneWidget);
    stall = true;
    await t.tap(find.text('Refresh'));
    await t.pump();
    await t.tap(find.text('Close'));
    await t.pumpAndSettle();
    expect(find.text('Child context'), findsNothing);
    pending.complete({'context': 'late result'});
    await t.pumpAndSettle();
    await t.ensureVisible(find.text('Interrupt child'));
    await t.tap(find.text('Interrupt child'));
    await t.pumpAndSettle();
    expect(find.textContaining('already settled'), findsOneWidget);
    expect(calls, contains('owned:inspect:4'));
    expect(calls.last, 'owned:interrupt:0');
    expect(t.takeException(), isNull);
  });
  testWidgets(
    'child events replace status, ignore stale runs and survive Stop until chat switch',
    (tester) async {
      final bridge = ToolBridge();
      final chat = ready(bridge);
      await chat.send();
      await tester.pump(const Duration(milliseconds: 100));
      bridge.queue.addAll([
        {'type': 'subagent', 'id': 1, 'child': child('left', 'queued')},
        {'type': 'subagent', 'id': 1, 'child': child('left', 'running')},
        {'type': 'subagent', 'id': 99, 'child': child('stale', 'running')},
        {'type': 'subagent', 'id': 1, 'child': child('right', 'reported')},
      ]);
      await tester.pump(const Duration(milliseconds: 250));
      expect(chat.subagents.length, 2);
      expect(chat.subagents.first['status'], 'running');
      await chat.stop();
      await tester.pump(const Duration(milliseconds: 250));
      expect(chat.subagents.first['status'], 'interrupted');
      expect(chat.subagents.last['status'], 'reported');
      expect(chat.draft, 'Read readme');
      chat.newChat(kind: 'side');
      expect(chat.subagents, isEmpty);
      chat.dispose();
    },
  );
  for (final dark in [false, true]) {
    testWidgets(
      'compact child review and literal reports remain reachable in ${dark ? 'dark' : 'light'}',
      (tester) async {
        compact(tester);
        final chat = ready(ToolBridge());
        chat.subagents.addAll([
          child('left', 'running'),
          child('right', 'queued'),
        ]);
        chat.toolApproval = {
          'callId': 'batch',
          'name': 'delegate_tasks',
          'target': '2 scoped subagent(s)',
          'query': jsonEncode({
            'tasks': [
              {'goal': 'Read left.txt', 'scope': 'left.txt', 'readOnly': true},
            ],
          }),
        };
        await tester.pumpWidget(
          DoloresApp(
            chat: chat,
            themeMode: dark ? ThemeMode.dark : ThemeMode.light,
          ),
        );
        await tester.pumpAndSettle();
        expect(find.text('Delegate these scoped tasks?'), findsOneWidget);
        expect(find.byKey(const Key('allow-tool')), findsOneWidget);
        expect(tester.takeException(), isNull);
        final receipt = {
          'callId': 'batch',
          'name': 'delegate_tasks',
          'target': '2 scoped subagent(s)',
          'status': 'completed',
          'content': jsonEncode({
            'children': [
              {
                ...child('left', 'paused'),
                'answer': '<script>literal</script>',
                'truncated': true,
              },
            ],
            'sharedUsage': {'modelCalls': 3, 'toolCalls': 2},
            'note': 'Parent verification required.',
          }),
        };
        await tester.pumpWidget(
          MaterialApp(
            home: Scaffold(
              body: SingleChildScrollView(
                child: ToolRecords(records: [receipt]),
              ),
            ),
          ),
        );
        await tester.pumpAndSettle();
        await tester.tap(find.text('2 scoped subagent(s)'));
        await tester.pumpAndSettle();
        await tester.tap(find.text('Read left.txt'));
        await tester.pumpAndSettle();
        expect(find.textContaining('<script>literal</script>'), findsOneWidget);
        expect(find.textContaining('Report shortened.'), findsOneWidget);
        expect(
          find.textContaining(
            'Shared task used 3 model calls and 2 tool operations',
          ),
          findsOneWidget,
        );
        expect(tester.takeException(), isNull);
        await tester.pumpWidget(const SizedBox());
        chat.dispose();
      },
    );
  }
  test('malformed child reports retain literal fallback', () {
    expect(subagentReports({'content': 'broken'}), isEmpty);
    expect(
      toolResultText({
        'name': 'delegate_tasks',
        'status': 'completed',
        'content': 'broken',
      }),
      'broken',
    );
  });
}
