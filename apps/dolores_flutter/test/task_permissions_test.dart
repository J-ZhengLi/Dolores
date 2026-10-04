import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:dolores_flutter/bridge.dart';
import 'package:dolores_flutter/chat.dart';
import 'package:dolores_flutter/task_permissions.dart';

class PermissionBridge implements ChatBridge {
  final calls = <Map<String, dynamic>>[];
  int revision = 0;
  bool stale = false;
  Map<String, dynamic> policy = {
    'mode': 'review',
    'grants': [],
    'expiresAt': null,
  };
  @override
  Future<void> open() async {}
  @override
  Future<void> close() async {}
  @override
  Future<dynamic> call(Map<String, dynamic> command) async {
    calls.add(command);
    if (command['command'] == 'setTaskPermissions') {
      if (stale) {
        stale = false;
        revision++;
        throw StateError('Permissions changed. Refresh and review.');
      }
      expect(command['revision'], revision);
      revision++;
      policy = Map<String, dynamic>.from(command['policy'] as Map);
    }
    return {
      'revision': revision,
      'policy': policy,
      'expired': false,
      'working': true,
      'mcpTools': [],
      'containment': 'Commands use your OS account. Not an OS sandbox.',
      'adaptation': 'Does not enable self-updates.',
    };
  }
}

void main() {
  testWidgets(
    'Complex saved grant boundaries require revocation before simplified replacement',
    (tester) async {
      final bridge = PermissionBridge()
        ..policy = {
          'mode': 'auto',
          'expiresAt': null,
          'grants': [
            {'tool': 'read_text_file', 'pathPrefix': 'src', 'command': null},
            {'tool': 'list_folder', 'pathPrefix': 'docs', 'command': null},
          ],
        };
      final chat = ChatController(bridge)
        ..session = 'a'
        ..loading = false;
      await tester.pumpWidget(
        MaterialApp(home: TaskPermissionsInspector(chat: chat)),
      );
      await tester.pumpAndSettle();
      final ack = find.text(
        'I understand this scope and the access described above',
      );
      await tester.ensureVisible(ack);
      await tester.tap(ack);
      await tester.pumpAndSettle();
      expect(
        tester
            .widget<FilledButton>(
              find.widgetWithText(FilledButton, 'Save access'),
            )
            .onPressed,
        isNull,
      );
      expect(
        bridge.calls.where((c) => c['command'] == 'setTaskPermissions'),
        isEmpty,
      );
      expect((bridge.policy['grants'] as List).map((g) => g['pathPrefix']), [
        'src',
        'docs',
      ]);
      await tester.tap(find.text('Revoke grants'));
      await tester.pumpAndSettle();
      expect(bridge.policy['mode'], 'review');
      expect(bridge.policy['grants'], isEmpty);
      await tester.pumpWidget(const SizedBox());
      chat.dispose();
    },
  );
  testWidgets(
    'Full access needs acknowledgement; busy Revoke remains usable in compact themes',
    (tester) async {
      tester.view.physicalSize = const Size(700, 680);
      tester.view.devicePixelRatio = 1;
      addTearDown(tester.view.resetPhysicalSize);
      addTearDown(tester.view.resetDevicePixelRatio);
      for (final dark in [false, true]) {
        final bridge = PermissionBridge();
        final chat = ChatController(bridge)
          ..session = 'a'
          ..loading = false
          ..draft = 'Keep my draft';
        await tester.pumpWidget(
          MaterialApp(
            theme: dark ? ThemeData.dark() : ThemeData.light(),
            home: TaskPermissionsInspector(chat: chat),
          ),
        );
        await tester.pumpAndSettle();
        await tester.tap(find.byType(DropdownButtonFormField<String>).first);
        await tester.pumpAndSettle();
        await tester.tap(find.text('Full access for this chat').last);
        await tester.pumpAndSettle();
        expect(
          tester
              .widget<FilledButton>(
                find.widgetWithText(FilledButton, 'Save access'),
              )
              .onPressed,
          null,
        );
        final ack = find.text(
          'I understand this scope and the access described above',
        );
        await tester.ensureVisible(ack);
        await tester.tap(ack);
        await tester.pumpAndSettle();
        await tester.tap(find.text('Save access'));
        await tester.pumpAndSettle();
        expect(bridge.policy['mode'], 'fullAccess');
        expect(bridge.policy['grants'], isEmpty);
        expect(bridge.policy['expiresAt'], isA<int>());
        chat.busy = true;
        await tester.pumpWidget(
          MaterialApp(
            theme: dark ? ThemeData.dark() : ThemeData.light(),
            home: TaskPermissionsInspector(chat: chat),
          ),
        );
        await tester.pumpAndSettle();
        expect(
          tester
              .widget<FilledButton>(
                find.widgetWithText(FilledButton, 'Save access'),
              )
              .onPressed,
          null,
        );
        await tester.tap(find.text('Revoke grants'));
        await tester.pumpAndSettle();
        expect(bridge.policy['mode'], 'review');
        expect(chat.draft, 'Keep my draft');
        expect(tester.takeException(), null);
        await tester.pumpWidget(const SizedBox());
        chat.dispose();
      }
    },
  );
  testWidgets(
    'A stale grant save preserves the chosen prefix until explicit refresh and renewed review',
    (tester) async {
      final bridge = PermissionBridge()..stale = true;
      final chat = ChatController(bridge)
        ..session = 'a'
        ..loading = false;
      await tester.pumpWidget(
        MaterialApp(home: TaskPermissionsInspector(chat: chat)),
      );
      await tester.pumpAndSettle();
      await tester.tap(find.byType(DropdownButtonFormField<String>).first);
      await tester.pumpAndSettle();
      await tester.tap(find.text('Approve within selected grants').last);
      await tester.pumpAndSettle();
      final prefix = find.widgetWithText(
        TextField,
        'Relative folder/file prefix',
      );
      await tester.ensureVisible(prefix);
      await tester.enterText(prefix, 'src');
      await tester.ensureVisible(find.text('Read text files'));
      await tester.tap(find.text('Read text files'));
      final ack = find.text(
        'I understand this scope and the access described above',
      );
      await tester.ensureVisible(ack);
      await tester.tap(ack);
      await tester.pumpAndSettle();
      await tester.tap(find.text('Save access'));
      await tester.pumpAndSettle();
      expect(tester.widget<TextField>(prefix).controller!.text, 'src');
      await tester.tap(find.text('Refresh'));
      await tester.pumpAndSettle();
      expect(
        tester
            .widget<FilledButton>(
              find.widgetWithText(FilledButton, 'Save access'),
            )
            .onPressed,
        null,
      );
      await tester.ensureVisible(ack);
      await tester.tap(ack);
      await tester.pumpAndSettle();
      await tester.tap(find.text('Save access'));
      await tester.pumpAndSettle();
      expect(bridge.calls.last['revision'], 1);
      expect((bridge.policy['grants'] as List).single['pathPrefix'], 'src');
      await tester.pumpWidget(const SizedBox());
      chat.dispose();
    },
  );
}
