import 'package:dolores_flutter/chat.dart';
import 'package:dolores_flutter/main.dart';
import 'package:dolores_flutter/run_history.dart';
import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';

import 'stop_recovery_test.dart' show InterruptedBridge;
import 'frost_design_test.dart' show capture, viewport;
import 'support/render_fonts.dart';

class SegmentBridge extends InterruptedBridge {
  SegmentBridge() : super(true);
  int id = 0;
  bool failPage = false;
  int cancelCalls = 0;
  @override
  Future<dynamic> call(Map<String, dynamic> c) async {
    if (c['command'] == 'cancel') cancelCalls++;
    if (c['command'] == 'runs') {
      return [
        {
          'id': 'fixture-run',
          'state': 'running',
          'model': 'fixture',
          'sequence': 3,
          'settings': {'maxOutputTokens': null},
        },
      ];
    }
    if (c['command'] == 'runEventsPage') {
      if (c['after'] == 1 && failPage) {
        throw StateError('Evidence temporarily unavailable. Retry.');
      }
      return {
        'items': [
          {
            'sequence': c['after'] == 1 ? 2 : 1,
            'kind': c['after'] == 1 ? 'toolResult' : 'taskCheckpoint',
            'state': 'running',
            'data': {'content': 'Complete fixture evidence 世界'},
          },
        ],
        'next': c['after'] == 1 ? null : 1,
        'total': 2,
      };
    }
    final result = await super.call(c);
    if (c['command'] == 'bootstrap') {
      return {...result as Map, 'taskSegments': true};
    }
    if (c['command'] == 'start') {
      id = c['id'] as int;
      queue.clear();
      queue.addAll([
        {'type': 'started', 'id': id, 'session': 'audit'},
        {'type': 'modelStep', 'id': id, 'number': 1},
        {
          'type': 'modelText',
          'id': id,
          'number': 1,
          'text': 'Earlier completed work.',
        },
        {
          'type': 'toolResult',
          'id': id,
          'record': {
            'callId': 'one',
            'name': 'read_text_file',
            'target': 'public.txt',
            'status': 'read',
            'content': 'Saved full evidence',
          },
        },
      ]);
    }
    return result;
  }

  void segment() => queue.addAll([
    {'type': 'taskMaintenance', 'id': id, 'active': true},
    {
      'type': 'taskSegment',
      'id': id,
      'archive': {
        'runId': 'fixture-run',
        'throughSegment': 2,
        'toolCount': 32,
        'modelCalls': 40,
      },
    },
  ]);
}

void main() {
  testWidgets(
    'active context follows each request and provider usage without starting work',
    (t) async {
      final bridge = SegmentBridge();
      final chat = ChatController(bridge)
        ..loading = false
        ..configured = true
        ..session = 'audit'
        ..draft = 'Original task';
      await chat.send();
      await t.pump(const Duration(milliseconds: 100));
      bridge.queue.add({
        'id': bridge.id,
        'type': 'modelContext',
        'number': 1,
        'tokens': {'inputTokens': 16479, 'contextWindowTokens': 500000},
      });
      await t.pump(const Duration(milliseconds: 100));
      expect(chat.contextSummary!['tokens']['inputTokens'], 16479);
      expect((await chat.previewContext())!['activeRun'], true);
      bridge.queue.add({
        'id': bridge.id,
        'type': 'modelFinished',
        'number': 1,
        'usage': {'totalTokens': 18000},
      });
      await t.pump(const Duration(milliseconds: 100));
      expect(chat.contextSummary!['reportedTokens'], 18000);
      bridge.queue.addAll([
        {'id': bridge.id, 'type': 'modelStep', 'number': 2},
        {
          'id': bridge.id,
          'type': 'modelContext',
          'number': 2,
          'tokens': {'inputTokens': 24000, 'contextWindowTokens': 500000},
        },
      ]);
      await t.pump(const Duration(milliseconds: 100));
      expect(chat.contextSummary!['tokens']['inputTokens'], 24000);
      expect(chat.contextSummary!.containsKey('reportedTokens'), false);
      expect(bridge.starts, 1);
      expect(bridge.commands.where((c) => c['command'] == 'context'), isEmpty);
      await chat.stop();
      chat.dispose();
    },
  );
  testWidgets(
    'silent complete-response retry updates and finishes without Stop',
    (t) async {
      final bridge = SegmentBridge();
      final chat = ChatController(bridge)
        ..loading = false
        ..configured = true
        ..session = 'audit'
        ..draft = 'Original task';
      await t.pumpWidget(DoloresApp(chat: chat));
      await chat.send();
      await t.pump(const Duration(milliseconds: 100));
      bridge.queue.addAll([
        {'type': 'modelRetry', 'id': bridge.id, 'attempt': 1, 'buffered': true},
        {'type': 'modelStep', 'id': bridge.id, 'number': 2},
        {
          'type': 'modelActivity',
          'id': bridge.id,
          'number': 2,
          'phase': 'waitingComplete',
          'elapsedSeconds': 1,
          'timeoutSeconds': 180,
        },
      ]);
      await t.pump(const Duration(milliseconds: 100));
      await t.pump();
      expect(chat.busy, true);
      expect(
        find.textContaining('No incremental output · 1s elapsed'),
        findsOneWidget,
      );
      chat.draft = 'Preserve a newer draft';
      bridge.queue.add({
        'type': 'modelActivity',
        'id': bridge.id,
        'number': 2,
        'phase': 'waitingComplete',
        'elapsedSeconds': 2,
        'timeoutSeconds': 180,
      });
      await t.pump(const Duration(milliseconds: 100));
      await t.pump();
      expect(
        find.textContaining('No incremental output · 2s elapsed'),
        findsOneWidget,
      );
      bridge.queue.add({'type': 'done', 'id': bridge.id});
      await t.pump(const Duration(milliseconds: 200));
      await t.pump();
      expect(chat.busy, false);
      expect(chat.draft, 'Preserve a newer draft');
      expect(bridge.cancelCalls, 0);
      expect(t.takeException(), isNull);
      await t.pumpWidget(const SizedBox());
      chat.dispose();
    },
  );
  testWidgets(
    'maintenance does not re-enable following after the reader scrolls up',
    (t) async {
      final c = ChatController(SegmentBridge())
        ..loading = false
        ..busy = true
        ..pendingInput = 'Long task'
        ..partial = 'paragraph\n\n' * 50;
      await t.pumpWidget(DoloresApp(chat: c));
      await t.pump(const Duration(milliseconds: 500));
      await t.pump();
      final scroll = t.widget<ListView>(find.byType(ListView).last).controller!;
      scroll.jumpTo(0);
      c.taskArchive = {'runId': 'fixture-run', 'throughSegment': 1};
      c.notifyListeners();
      await t.pump();
      await t.pump(const Duration(milliseconds: 100));
      c.partial = 'New streamed evidence\n\n' * 70;
      c.notifyListeners();
      await t.pump();
      await t.pump(const Duration(milliseconds: 100));
      await t.pump();
      expect(scroll.offset, 0);
      expect(scroll.position.extentAfter, greaterThan(80));
      await t.pumpWidget(const SizedBox.shrink());
      c.dispose();
      expect(t.takeException(), isNull);
    },
  );
  testWidgets('maintenance retains uncertain tool state in the active view', (
    t,
  ) async {
    final b = SegmentBridge();
    final c = ChatController(b)
      ..loading = false
      ..configured = true
      ..session = 'audit'
      ..draft = 'Task';
    await c.send();
    await t.pump(const Duration(milliseconds: 100));
    b.segment();
    b.queue.last['retained'] = [
      {
        'callId': 'uncertain',
        'name': 'command_session',
        'target': 'fixture',
        'status': 'running',
        'content': 'Live command session',
      },
    ];
    b.queue.last['toolSteps'] = [39];
    await t.pump(const Duration(milliseconds: 100));
    expect(c.toolRecords.first['callId'], 'one');
    expect(c.toolRecords.last['callId'], 'uncertain');
    expect(c.toolRecords.last['modelStep'], 39);
    await c.stop();
    c.dispose();
  });
  for (final dark in [false, true]) {
    for (final width in [420.0, 1000.0]) {
      testWidgets(
        'automatic maintenance keeps one task, draft and Stop ($dark $width)',
        (t) async {
          viewport(t, Size(width, 800));
          await t.runAsync(loadReviewFonts);
          final b = SegmentBridge();
          final c = ChatController(b)
            ..loading = false
            ..configured = true
            ..session = 'audit'
            ..taskSegmentsAvailable = true
            ..draft = 'Original task';
          final key = GlobalKey();
          await t.pumpWidget(
            RepaintBoundary(
              key: key,
              child: DoloresApp(
                chat: c,
                themeMode: dark ? ThemeMode.dark : ThemeMode.light,
              ),
            ),
          );
          await c.send();
          await t.pump(const Duration(milliseconds: 100));
          await t.pump();
          c.draft = 'Keep this newer draft 世界';
          final elapsed = c.runElapsed;
          b.segment();
          await t.pump(const Duration(milliseconds: 100));
          await t.pump();
          expect(c.busy, true);
          expect(c.draft, 'Keep this newer draft 世界');
          expect(b.starts, 1);
          expect(c.toolRecords.single['callId'], 'one');
          expect(c.partial, 'Earlier completed work.');
          expect(c.modelThinking, isEmpty);
          expect(c.runElapsed, greaterThanOrEqualTo(elapsed));
          expect(find.text('Updating context'), findsOneWidget);
          expect(find.byKey(const Key('task-archived-progress')), findsNothing);
          expect(
            find.textContaining('Earlier completed work.'),
            findsOneWidget,
          );
          expect(find.byTooltip('Stop response'), findsOneWidget);
          expect(find.text('Continue'), findsNothing);
          await capture(
            t,
            key,
            'automatic-maintenance-${dark ? 'dark' : 'light'}-${width.toInt()}',
          );
          await c.stop();
          await t.pump(const Duration(milliseconds: 100));
          await t.pump();
          expect(c.busy, false);
          expect(c.draft, 'Keep this newer draft 世界');
          expect(b.starts, 1);
          expect(t.takeException(), isNull);
          await t.pumpWidget(const SizedBox());
          c.dispose();
        },
      );
    }
  }
  testWidgets(
    'paged history retains its readable page and retries without dispatch',
    (t) async {
      final b = SegmentBridge()..failPage = true;
      final c = ChatController(b)
        ..session = 'audit'
        ..busy = true
        ..taskSegmentsAvailable = true;
      await t.pumpWidget(
        MaterialApp(
          home: RunHistoryInspector(chat: c, initialRun: 'fixture-run'),
        ),
      );
      await t.pumpAndSettle();
      await t.ensureVisible(find.text('Next evidence'));
      await t.tap(find.text('Next evidence'));
      await t.pumpAndSettle();
      expect(find.text('1. taskCheckpoint · running'), findsOneWidget);
      expect(find.textContaining('temporarily unavailable'), findsOneWidget);
      b.failPage = false;
      await t.tap(find.text('Next evidence'));
      await t.pumpAndSettle();
      expect(find.text('2. toolResult · running'), findsOneWidget);
      await t.tap(find.text('Previous evidence'));
      await t.pumpAndSettle();
      expect(find.text('1. taskCheckpoint · running'), findsOneWidget);
      expect(b.starts, 0);
      expect(c.busy, true);
      await t.pumpWidget(const SizedBox());
      c.dispose();
    },
  );
}
