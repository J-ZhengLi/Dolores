import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:dolores_flutter/access_selector.dart';
import 'package:dolores_flutter/chat.dart';

import 'task_permissions_test.dart';

void main() {
  testWidgets('Picking Full access saves immediately and survives reopening', (
    tester,
  ) async {
    final bridge = PermissionBridge();
    final chat = ChatController(bridge)
      ..session = 'a'
      ..loading = false;
    Widget app() => MaterialApp(
      home: Scaffold(body: AccessSelector(chat: chat)),
    );
    await tester.pumpWidget(app());
    await tester.pumpAndSettle();
    await tester.tap(find.byKey(const Key('chat-access')));
    await tester.pumpAndSettle();
    await tester.tap(find.text('Full access').last);
    await tester.pumpAndSettle();
    expect(bridge.policy['mode'], 'fullAccess');
    expect(bridge.policy['expiresAt'], isNull);
    expect(find.text('Save access'), findsNothing);
    expect(
      find.text('I understand this scope and the access described above'),
      findsNothing,
    );
    await tester.pumpWidget(const SizedBox());
    await tester.pumpWidget(app());
    await tester.pumpAndSettle();
    expect(find.text('Full access'), findsOneWidget);
    await tester.pumpWidget(const SizedBox());
    chat.dispose();
  });
}
