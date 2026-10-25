import 'dart:convert';

import 'package:dolores_flutter/tool_activity.dart';
import 'package:flutter_test/flutter_test.dart';

import 'tools_test.dart' show ToolBridge, ready;

void main() {
  testWidgets(
    'cleanup warning cannot finish a run or clear a pending decision',
    (t) async {
      final bridge = ToolBridge();
      final chat = ready(bridge);
      await chat.send();
      await t.pump(const Duration(milliseconds: 250));
      final approval = chat.toolApproval;
      expect(chat.busy, true);
      bridge.queue.add({
        'type': 'connectorCleanup',
        'id': 1,
        'receipt': {'confirmed': false, 'status': 'unsupported'},
      });
      await t.pump(const Duration(milliseconds: 500));
      expect(chat.busy, true);
      expect(chat.toolApproval, approval);
      await chat.stop();
      await t.pump(const Duration(milliseconds: 250));
      chat.dispose();
    },
  );
  test('scheduled confirmation discloses the reviewed hook snapshot', () {
    final text = toolResultText({
      'name': 'schedule_task',
      'status': 'completed',
      'content': jsonEncode({
        'task': 'public',
        'schedule': 'Daily at 09:00',
        'lifecycleHooks': 2,
      }),
    });
    expect(text, contains('Lifecycle hooks: 2 reviewed'));
  });
}
