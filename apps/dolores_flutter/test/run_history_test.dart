import 'package:dolores_flutter/run_history.dart';
import 'package:dolores_flutter/chat.dart';
import 'package:dolores_flutter/theme.dart';
import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';

import 'history_test.dart' show HistoryBridge;
import 'frost_design_test.dart' show capture, viewport;
import 'support/render_fonts.dart';

class ReadableBridge extends RunBridge {
  bool failProgress = false;
  int receiptReads = 0;
  @override
  Future<dynamic> call(Map<String, dynamic> command) async {
    if (command['command'] == 'runCheckpoint') {
      throw StateError('Older recovery details unavailable');
    }
    if (command['command'] == 'runEventsPage') {
      return {'items': [], 'next': null};
    }
    if (command['command'] == 'readTaskHistory') {
      receiptReads++;
      return {
        'content': 'Complete original output',
        'offset': 0,
        'nextOffset': 24,
        'hasMore': false,
      };
    }
    if (command['command'] == 'runProgress') {
      if (failProgress) throw StateError('Temporary history read failure');
      return {
        'ordinal': command['ordinal'],
        'total': 2,
        'summary': {
          'steps': [
            {'number': 1, 'text': 'Earlier model output'},
          ],
          'tools': [
            {
              'callId': 'saved-call',
              'name': 'read_text_file',
              'target': 'original-public.txt',
              'status': 'completed',
              'content': '{"sourceCallId":"saved-call","previewTruncated":true,"preview":"Original saved tool output"}',
            },
          ],
          'toolSteps': [1],
          'thinking': [],
        },
      };
    }
    return super.call(command);
  }
}

class RunBridge extends HistoryBridge {
  bool inspectionFailed = false;
  String? requested;
  @override
  Future<dynamic> call(Map<String, dynamic> command) async {
    if (command['command'] == 'runs') {
      if (inspectionFailed) throw 'Evidence unavailable. Refresh.';
      return [
        {
          'id': 'durable',
          'model': 'fixture',
          'state': 'interrupted',
          'sequence': 2,
          'settings': {'maxOutputTokens': 512},
        },
      ];
    }
    if (command['command'] == 'runEvents') {
      requested = command['runId'] as String;
      expect(command['session'], 'chat');
      return [
        {
          'sequence': 2,
          'kind': 'restart',
          'state': 'interrupted',
          'data': {'message': 'Effect outcome is unknown'},
        },
      ];
    }
    return super.call(command);
  }
}

void main() {
  for (final dark in [false, true]) {
    for (final width in [420.0, 1000.0]) {
      testWidgets(
        'readable archived history survives recovery and page failures $dark $width',
        (t) async {
          viewport(t, Size(width, 820));
          await t.runAsync(loadReviewFonts);
          final bridge = ReadableBridge();
          final chat = ChatController(bridge)
            ..session = 'chat'
            ..busy = true
            ..durableDrafts = true
            ..taskSegmentsAvailable = true;
          final key = GlobalKey();
          await t.pumpWidget(
            MaterialApp(
              theme: doloresTheme(dark),
              home: RepaintBoundary(
                key: key,
                child: RunHistoryInspector(
                  chat: chat,
                  initialRun: 'older-retained-run',
                ),
              ),
            ),
          );
          await t.pumpAndSettle();
          expect(
            find.textContaining('Recovery details unavailable'),
            findsOneWidget,
          );
          expect(
            find.text('Earlier model output', findRichText: true),
            findsOneWidget,
          );
          expect(
            find.textContaining('original-public.txt', findRichText: true),
            findsNWidgets(2),
          );
          expect(find.text('Progress page 1 of 2'), findsOneWidget);
          await capture(
            t,
            key,
            'long-task-history-${dark ? 'dark' : 'light'}-${width.toInt()}',
          );
          expect(bridge.receiptReads, 0);
          await t.ensureVisible(find.textContaining('Full output ·'));
          await t.tap(find.textContaining('Full output ·'));
          await t.pumpAndSettle();
          expect(find.text('Complete original output'), findsOneWidget);
          expect(bridge.receiptReads, 1);
          await t.tap(find.text('Close').last);
          await t.pumpAndSettle();
          bridge.failProgress = true;
          await t.ensureVisible(find.text('Next progress'));
          await t.tap(find.text('Next progress'));
          await t.pumpAndSettle();
          expect(find.text('Progress page 1 of 2'), findsOneWidget);
          expect(
            find.text('Earlier model output', findRichText: true),
            findsOneWidget,
          );
          bridge.failProgress = false;
          await t.tap(find.text('Next progress'));
          await t.pumpAndSettle();
          expect(find.text('Progress page 2 of 2'), findsOneWidget);
          expect(chat.busy, true);
          expect(t.takeException(), isNull);
          await t.pumpWidget(const SizedBox());
          chat.dispose();
        },
      );
    }
  }
  testWidgets(
    'attention target opens exact retained evidence outside the recent twenty runs',
    (t) async {
      final b = RunBridge();
      final c = ChatController(b)
        ..session = 'chat'
        ..busy = true;
      await t.pumpWidget(
        MaterialApp(
          home: RunHistoryInspector(chat: c, initialRun: 'older-retained-run'),
        ),
      );
      await t.pumpAndSettle();
      expect(b.requested, 'older-retained-run');
      expect(find.text('Selected run: older-retained-run'), findsOneWidget);
      expect(find.text('2. restart · interrupted'), findsOneWidget);
      expect(c.busy, true);
      await t.pumpWidget(const SizedBox());
      c.dispose();
    },
  );
  testWidgets(
    'compact run inspection works during execution and explains uncertainty',
    (tester) async {
      tester.view.physicalSize = const Size(480, 600);
      tester.view.devicePixelRatio = 1;
      addTearDown(tester.view.resetPhysicalSize);
      addTearDown(tester.view.resetDevicePixelRatio);
      final chat = ChatController(RunBridge())
        ..session = 'chat'
        ..busy = true;
      addTearDown(chat.dispose);
      await tester.pumpWidget(
        MaterialApp(
          theme: doloresTheme(true),
          home: RunHistoryInspector(chat: chat),
        ),
      );
      await tester.pumpAndSettle();
      expect(find.textContaining('interrupted · fixture'), findsOneWidget);
      await tester.tap(find.textContaining('interrupted · fixture'));
      await tester.pumpAndSettle();
      await tester.tap(find.text('2. restart · interrupted'));
      await tester.pumpAndSettle();
      expect(find.textContaining('Effect outcome is unknown'), findsOneWidget);
      expect(chat.busy, isTrue);
      expect(tester.takeException(), isNull);
    },
  );
  testWidgets('failed refresh can recover without executing a run', (
    tester,
  ) async {
    final bridge = RunBridge()..inspectionFailed = true;
    final chat = ChatController(bridge)..session = 'chat';
    addTearDown(chat.dispose);
    await tester.pumpWidget(MaterialApp(home: RunHistoryInspector(chat: chat)));
    await tester.pumpAndSettle();
    expect(find.textContaining('Evidence unavailable'), findsOneWidget);
    bridge.inspectionFailed = false;
    await tester.tap(find.text('Refresh'));
    await tester.pumpAndSettle();
    expect(find.textContaining('interrupted · fixture'), findsOneWidget);
    expect(chat.busy, isFalse);
  });
}
