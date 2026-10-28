import 'package:dolores_flutter/chat.dart';
import 'package:dolores_flutter/main.dart';
import 'package:dolores_flutter/task_transcript.dart';
import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';

import 'stop_recovery_test.dart' show InterruptedBridge;
import 'frost_design_test.dart' show capture, viewport;
import 'support/render_fonts.dart';

Map<String, dynamic> receipt(String id) => {
  'callId': id,
  'name': 'read_text_file',
  'target': '$id.dart',
  'status': 'read',
  'content': 'Original $id result',
};

class TranscriptBridge extends InterruptedBridge {
  TranscriptBridge() : super(true);
  bool failSecond = false;
  final reads = <int>[];
  @override
  Future<dynamic> call(Map<String, dynamic> command) async {
    if (command['command'] == 'runProgress') {
      reads.add(command['ordinal'] as int);
      final n = command['ordinal'] as int;
      if (n == 2 && failSecond) {
        throw StateError('Temporary local read failure');
      }
      return {
        'ordinal': n,
        'total': 2,
        'summary': {
          'steps': [
            {
              'number': n,
              'text': n == 1
                  ? 'First implementation step.'
                  : 'Second implementation step.',
            },
          ],
          'tools': [receipt('first'), if (n == 2) receipt('second')],
          'toolSteps': [1, if (n == 2) 2],
          'thinking': [],
        },
      };
    }
    return super.call(command);
  }
}

Map<String, dynamic> tail() => {
  'modelCalls': 3,
  'usageByCall': [null, null, null],
  'archive': {'runId': 'fixture-run', 'throughSegment': 2},
  'steps': [
    {'number': 3, 'text': 'Validation passed.'},
  ],
  'tools': [receipt('second'), receipt('third')],
  'toolSteps': [2, 3],
  'thinking': [],
};

void main() {
  for (final dark in [false, true]) {
    for (final width in [420.0, 1000.0]) {
      testWidgets(
        'reopened conversation shows every step inline ($dark $width)',
        (t) async {
          viewport(t, Size(width, 900));
          await t.runAsync(loadReviewFonts);
          final bridge = TranscriptBridge();
          final chat = ChatController(bridge)
            ..loading = false
            ..session = 'audit'
            ..messages = [
              {'id': 1, 'role': 'user', 'content': 'Implement the feature.'},
              {
                'id': 2,
                'role': 'assistant',
                'content': 'Feature complete.',
                'metadata': {'agent': tail()},
              },
            ];
          final key = GlobalKey();
          await t.pumpWidget(
            RepaintBoundary(
              key: key,
              child: DoloresApp(
                chat: chat,
                themeMode: dark ? ThemeMode.dark : ThemeMode.light,
              ),
            ),
          );
          await t.pumpAndSettle();
          expect(bridge.reads, [1, 2]);
          expect(bridge.starts, 0);
          expect(find.text('Earlier progress'), findsNothing);
          for (final text in [
            'First implementation step.',
            'Second implementation step.',
            'Validation passed.',
            'Feature complete.',
          ]) {
            expect(find.text(text), findsOneWidget);
          }
          expect(
            t.getTopLeft(find.text('First implementation step.')).dy,
            lessThan(t.getTopLeft(find.text('Second implementation step.')).dy),
          );
          expect(
            t.getTopLeft(find.text('Second implementation step.')).dy,
            lessThan(t.getTopLeft(find.text('Validation passed.')).dy),
          );
          await capture(
            t,
            key,
            'normal-transcript-${dark ? "dark" : "light"}-${width.toInt()}',
          );
          expect(t.takeException(), isNull);
          await t.pumpWidget(const SizedBox());
          chat.dispose();
        },
      );
    }
  }
  testWidgets(
    'failed transcript page preserves output and retries only missing local data',
    (t) async {
      final bridge = TranscriptBridge()..failSecond = true;
      await t.pumpWidget(
        MaterialApp(
          home: Scaffold(
            body: SingleChildScrollView(
              child: TaskTranscript(
                bridge: bridge,
                session: 'audit',
                archive: tail()['archive'],
                current: tail(),
                thinking: (_) => const SizedBox(),
              ),
            ),
          ),
        ),
      );
      await t.pumpAndSettle();
      expect(find.text('First implementation step.'), findsOneWidget);
      expect(find.text('Validation passed.'), findsOneWidget);
      expect(
        find.textContaining('Temporary local read failure'),
        findsOneWidget,
      );
      bridge.failSecond = false;
      await t.tap(find.text('Retry loading conversation'));
      await t.pumpAndSettle();
      expect(bridge.reads, [1, 2, 2]);
      expect(bridge.starts, 0);
      expect(find.text('Second implementation step.'), findsOneWidget);
      expect(find.text('First implementation step.'), findsOneWidget);
      expect(find.textContaining('could not be loaded'), findsNothing);
    },
  );
  test(
    'carried receipts appear once and unknown legacy ordering remains unknown',
    () {
      final merged = mergeTaskProgress([
        {
          'tools': [receipt('first')],
          'toolSteps': [1],
        },
        {
          'tools': [receipt('first'), receipt('second')],
          'toolSteps': [1, 2],
        },
        tail(),
      ]);
      expect((merged['tools'] as List).map((r) => r['callId']), [
        'first',
        'second',
        'third',
      ]);
      expect(merged['toolSteps'], [1, 2, 3]);
      expect(
        mergeTaskProgress([
          {
            'tools': [receipt('legacy')],
          },
          tail(),
        ])['toolSteps'],
        isEmpty,
      );
    },
  );
}
