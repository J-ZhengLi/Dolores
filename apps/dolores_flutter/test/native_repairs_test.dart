import 'dart:convert';

import 'package:dolores_flutter/chat.dart';
import 'package:dolores_flutter/native_repairs.dart';
import 'package:dolores_flutter/tool_activity.dart';
import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';

import 'history_test.dart' show HistoryBridge;

class NativeBridge extends HistoryBridge {
  bool stale = false;
  @override
  Future<dynamic> call(Map<String, dynamic> command) async {
    if (command['command'] == 'nativeRepairs') {
      commands.add(command);
      return {
        'builds': [
          {
            'id': 'build',
            'session': 'chat',
            'repairId': 'repair',
            'revision': 2,
            'status': 'ready',
            'note': 'Installation needs separate review.',
          },
        ],
        'intent': {
          'stage': 'recoveryRequired',
          'operation': 'install',
          'note': 'Interrupted before healthy startup.',
          'interrupted': true,
          'recovery': 'Both versions and history remain. Request fresh review; no task replay.',
        },
      };
    }
    if (command['command'] == 'reviewNativeUpdate') {
      commands.add(command);
      if (stale) {
        throw StateError(
          'Bundle changed. Retained work remains; request fresh review.',
        );
      }
      return {
        'token': 'review',
        'operation': command['restore'] == true ? 'restore' : 'install',
        'note': 'Native code has account permissions. No task replay.',
        'shutdownSeconds': 30,
        'startupSeconds': 30,
      };
    }
    return super.call(command);
  }
}

void main() {
  testWidgets(
    'compact themes retain recovery and fresh review after stale bundle',
    (tester) async {
      tester.view.physicalSize = const Size(420, 480);
      tester.view.devicePixelRatio = 1;
      addTearDown(tester.view.resetPhysicalSize);
      addTearDown(tester.view.resetDevicePixelRatio);
      for (final dark in [false, true]) {
        final bridge = NativeBridge();
        final chat = ChatController(bridge)..loading = false;
        await tester.pumpWidget(
          MaterialApp(
            theme: ThemeData(
              brightness: dark ? Brightness.dark : Brightness.light,
            ),
            home: Scaffold(body: NativeRepairs(chat: chat)),
          ),
        );
        await tester.pumpAndSettle();
        expect(find.textContaining('Interrupted before'), findsOneWidget);
        final button = find.text('Review installation');
        await tester.ensureVisible(button);
        await tester.pumpAndSettle();
        await tester.tap(button);
        await tester.pumpAndSettle();
        expect(find.text('Install & restart'), findsOneWidget);
        await tester.tap(find.text('Cancel review'));
        await tester.pumpAndSettle();
        bridge.stale = true;
        await tester.ensureVisible(button);
        await tester.pumpAndSettle();
        await tester.tap(button);
        await tester.pumpAndSettle();
        expect(find.textContaining('Bundle changed'), findsOneWidget);
        expect(find.text('Install & restart'), findsNothing);
        expect(
          bridge.commands.where((c) => c['command'] == 'applyNativeUpdate'),
          isEmpty,
        );
        expect(tester.takeException(), isNull);
        await tester.pumpWidget(const SizedBox());
        chat.dispose();
      }
    },
  );
  testWidgets('unsaved settings prevent restart and retain the exact review', (
    tester,
  ) async {
    final bridge = NativeBridge();
    final chat = ChatController(bridge)..loading = false;
    await tester.pumpWidget(
      MaterialApp(
        home: Scaffold(
          body: NativeRepairs(chat: chat, hasUnsavedSettings: () => true),
        ),
      ),
    );
    await tester.pumpAndSettle();
    await tester.tap(find.text('Review installation'));
    await tester.pumpAndSettle();
    await tester.tap(find.text('Install & restart'));
    await tester.pumpAndSettle();
    expect(find.textContaining('unsaved edits'), findsOneWidget);
    expect(find.text('Install & restart'), findsOneWidget);
    expect(
      bridge.commands.where((c) => c['command'] == 'applyNativeUpdate'),
      isEmpty,
    );
    await tester.pumpWidget(const SizedBox());
    chat.dispose();
  });
  test('ready build does not claim installation', () {
    final record = {
      'name': 'build_harness_repair',
      'status': 'completed',
      'content': jsonEncode({
        'build': {
          'status': 'ready',
          'id': 'build',
          'candidateSourceId': 'source',
          'note': 'Separately review installation.',
        },
      }),
    };
    expect(toolStatus(record), 'Build ready · installation needs review');
    expect(
      toolResultText(record),
      contains('Settings → Advanced → Native repairs'),
    );
  });
  test('failed build directs inspection and qualified condensed trials stay truthful', () {
    final failed = {
      'name': 'build_harness_repair',
      'status': 'completed',
      'content': jsonEncode({
        'build': {
          'status': 'failed',
          'note': 'Candidate DLL could not be synced.',
        },
      }),
    };
    expect(
      toolResultText(failed),
      contains('Candidate DLL could not be synced.'),
    );
    expect(toolResultText(failed), contains('fresh build review'));
    expect(
      toolResultText(failed),
      isNot(contains('Review installation or Restore')),
    );
    final qualified = {
      'name': 'test_harness_repair',
      'status': 'completed',
      'content': jsonEncode({
        'status': 'qualified',
        'baselinePassed': 2,
        'baselineFailed': 1,
        'candidatePassed': 3,
        'candidateFailed': 0,
      }),
    };
    expect(
      toolResultText(qualified),
      contains('Regressions: passed; detailed counts retained locally'),
    );
  });
}
