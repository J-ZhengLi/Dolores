import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:dolores_flutter/main.dart';
import 'package:dolores_flutter/chat.dart';
import 'history_test.dart' show HistoryBridge;

void main() {
  testWidgets('Idle conversation schedules no continuous animation frames', (tester) async {
    final chat = ChatController(HistoryBridge())..loading = false;
    await tester.pumpWidget(DoloresApp(chat: chat));
    await tester.pumpAndSettle();
    await tester.pump(const Duration(seconds: 2));
    expect(tester.binding.transientCallbackCount, 0);
    expect(tester.binding.hasScheduledFrame, isFalse);
    await tester.pumpWidget(const SizedBox());
    chat.dispose();
  });
}
