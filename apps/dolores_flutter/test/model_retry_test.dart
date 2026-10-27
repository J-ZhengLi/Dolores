import 'package:dolores_flutter/chat.dart';
import 'package:dolores_flutter/main.dart';
import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';

import 'stop_recovery_test.dart' show InterruptedBridge;
import 'frost_design_test.dart' show capture, viewport;
import 'support/render_fonts.dart';

void main() {
  for (final dark in [false, true]) {
    testWidgets(
      'retry keeps one task, earlier evidence, newer draft and Stop ($dark)',
      (t) async {
        viewport(t, const Size(420, 800));
        await t.runAsync(loadReviewFonts);
        final b = InterruptedBridge(true);
        final c = ChatController(b)
          ..loading = false
          ..configured = true
          ..session = 'audit'
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
        c.draft = 'Keep this newer draft';
        final start = b.commands.singleWhere((v) => v['command'] == 'start');
        final id = start['id'];
        final clock = c.runElapsed;
        b.queue.addAll([
          {
            'type': 'toolResult',
            'id': id,
            'record': {
              'callId': 'completed',
              'name': 'read_text_file',
              'target': 'public.txt',
              'status': 'read',
              'content': 'Full prior receipt',
            },
          },
          {'type': 'modelFinished', 'id': id, 'number': 4, 'usage': null},
          {
            'type': 'modelRetry',
            'id': id,
            'number': 4,
            'attempt': 1,
            'delayMs': 1000,
            'buffered': true,
          },
        ]);
        await t.pump(const Duration(milliseconds: 100));
        await t.pump();
        expect(c.busy, true);
        expect(c.draft, 'Keep this newer draft');
        expect(c.runElapsed, greaterThanOrEqualTo(clock));
        expect(c.partial, contains('interrupted before tool dispatch'));
        expect(c.toolRecords.single['callId'], 'completed');
        expect(
          c.requestLogs.any(
            (log) => log.label.contains('requesting a complete response'),
          ),
          true,
        );
        expect(find.text('Reconnecting to the model'), findsOneWidget);
        expect(find.byTooltip('Stop response'), findsOneWidget);
        expect(b.starts, 1);
        expect(find.text('Continue'), findsNothing);
        await capture(t, key, 'model-retry-${dark ? 'dark' : 'light'}-420');
        b.queue.addAll([
          {'type': 'modelStep', 'id': id, 'number': 5},
          {
            'type': 'modelText',
            'id': id,
            'number': 5,
            'text': 'Fresh response',
          },
        ]);
        await t.pump(const Duration(milliseconds: 100));
        expect(c.partial, 'Fresh response');
        expect(
          c.modelTexts.last['text'],
          contains('interrupted before tool dispatch'),
        );
        await c.stop();
        await t.pump(const Duration(milliseconds: 100));
        expect(c.draft, 'Keep this newer draft');
        expect(b.starts, 1);
        await t.pumpWidget(const SizedBox.shrink());
        c.dispose();
        expect(t.takeException(), isNull);
      },
    );
  }
}
