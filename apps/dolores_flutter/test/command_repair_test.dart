import 'dart:convert';

import 'package:dolores_flutter/chat.dart';
import 'package:dolores_flutter/main.dart';
import 'package:dolores_flutter/tool_activity.dart';
import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';

import 'continuation_test.dart' show PausedBridge;

Map<String, dynamic> receipt({
  int? exit = 1,
  String reason = 'completed',
  bool shortened = false,
}) => {
  'callId': 'check', 'name': 'run_command', 'target': 'node',
  'status':
      'completed', // Legacy persisted receipts still show the actual result.
  'command': {
    'program': 'node',
    'args': ['check.cjs'],
  },
  'content': jsonEncode({
    'reason': reason,
    'exitCode': exit,
    'truncated': shortened,
    'stdout': '',
    'stderr': 'Assertion failed 世界',
  }),
};

void main() {
  test(
    'Command status uses exit and capture evidence, including old records',
    () {
      expect(toolStatus(receipt(exit: 0)), 'Exited 0');
      expect(toolStatus(receipt()), 'Failed · exit 1');
      expect(
        toolStatus(receipt(exit: 0, shortened: true)),
        'Verification incomplete',
      );
      expect(
        toolStatus(receipt(reason: 'timedOut', exit: null)),
        'Verification incomplete',
      );
      expect(
        toolStatus({...receipt(), 'content': 'All checks passed'}),
        'Verification incomplete',
      );
      expect(toolStatus({...receipt(), 'status': 'denied'}), 'denied');
      expect(
        toolResultText({...receipt(), 'status': 'failed'}),
        contains('Assertion failed 世界'),
      );
    },
  );

  testWidgets(
    'Compact repair shows inherited failure despite a model success claim and protects the draft',
    (tester) async {
      tester.view.physicalSize = const Size(620, 700);
      tester.view.devicePixelRatio = 1;
      addTearDown(tester.view.resetPhysicalSize);
      addTearDown(tester.view.resetDevicePixelRatio);
      for (final brightness in Brightness.values) {
        final bridge = PausedBridge();
        final chat = ChatController(bridge)
          ..loading = false
          ..configured = true
          ..session = 'fixture'
          ..messages = [
            {
              'id': 1,
              'role': 'assistant',
              'content': 'Everything passed.',
              'metadata': {
                'paused': {
                  'reason': 'commandReview',
                  'task': 'Repair',
                  'receipts': [
                    {
                      'callId': 'created',
                      'name': 'create_text_file',
                      'target': 'sum.cjs',
                      'status': 'created',
                      'content': jsonEncode({
                        'applied': true,
                        'bytesAfter': 32,
                      }),
                    },
                    receipt(),
                  ],
                },
                'agent': {'tools': [], 'steps': []},
              },
            },
          ];
        await tester.pumpWidget(
          DoloresApp(
            chat: chat,
            themeMode: brightness == Brightness.dark
                ? ThemeMode.dark
                : ThemeMode.light,
          ),
        );
        await tester.pumpAndSettle();
        expect(find.text('Repair and verify'), findsOneWidget);
        expect(find.textContaining('A command failed'), findsOneWidget);
        expect(find.text('Command · Failed · exit 1'), findsOneWidget);
        expect(find.text('sum.cjs'), findsOneWidget);
        await tester.tap(find.text('node'));
        await tester.pumpAndSettle();
        expect(find.textContaining('Assertion failed 世界'), findsOneWidget);
        await tester.enterText(
          find.byKey(const Key('composer')),
          'Keep my draft',
        );
        await tester.pump();
        expect(
          tester
              .widget<TextButton>(find.byKey(const Key('continue-task')))
              .onPressed,
          isNull,
        );
        await chat.continueTask(1);
        expect(chat.draft, 'Keep my draft');
        expect(bridge.commands.where((c) => c['command'] == 'start'), isEmpty);
        await tester.enterText(find.byKey(const Key('composer')), '');
        await tester.pump();
        await tester.ensureVisible(find.byKey(const Key('continue-task')));
        await tester.tap(find.byKey(const Key('continue-task')));
        await tester.pump();
        final start = bridge.commands.lastWhere((c) => c['command'] == 'start');
        expect(start['continuation'], 1);
        expect(start['input'], 'Continue working on the previous task.');
        expect(tester.takeException(), isNull);
        await tester.pumpWidget(const SizedBox());
        chat.dispose();
      }
    },
  );
}
