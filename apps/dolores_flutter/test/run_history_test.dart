import 'package:dolores_flutter/run_history.dart';
import 'package:dolores_flutter/chat.dart';
import 'package:dolores_flutter/theme.dart';
import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';

import 'history_test.dart' show HistoryBridge;

class RunBridge extends HistoryBridge {
  bool inspectionFailed = false;
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
