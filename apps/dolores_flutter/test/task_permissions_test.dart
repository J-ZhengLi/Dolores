import 'package:dolores_flutter/app_dropdown.dart';
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
    };
  }
}

void main() {
  testWidgets(
    'Opening preserves custom grants; explicit selection replaces them once',
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
      expect(
        bridge.calls.where((c) => c['command'] == 'setTaskPermissions'),
        isEmpty,
      );
      expect((bridge.policy['grants'] as List).map((g) => g['pathPrefix']), [
        'src',
        'docs',
      ]);
      await tester.tap(find.byKey(const Key('access-mode-fullAccess')));
      await tester.pumpAndSettle();
      expect(bridge.policy, accessPolicy('fullAccess'));
      expect(
        bridge.calls.where((c) => c['command'] == 'setTaskPermissions').length,
        1,
      );
      await tester.pumpWidget(const SizedBox());
      chat.dispose();
    },
  );
  for (final width in [390.0, 700.0]) {
    for (final dark in [false, true]) {
      testWidgets(
        'Immediate mode selection and busy Review at $width dark=$dark',
        (tester) async {
          tester.view.physicalSize = Size(width, 680);
          tester.view.devicePixelRatio = 1;
          addTearDown(tester.view.resetPhysicalSize);
          addTearDown(tester.view.resetDevicePixelRatio);
          final bridge = PermissionBridge();
          final chat = ChatController(bridge)
            ..session = 'a'
            ..loading = false
            ..draft = 'Keep my draft';
          Widget app() => MaterialApp(
            theme: dark ? ThemeData.dark() : ThemeData.light(),
            home: TaskPermissionsInspector(chat: chat),
          );
          await tester.pumpWidget(app());
          await tester.pumpAndSettle();
          await tester.tap(find.byKey(const Key('access-mode-auto')));
          await tester.pumpAndSettle();
          expect(bridge.policy, accessPolicy('auto'));
          expect(
            (bridge.policy['grants'] as List).any(
              (g) => g['tool'] == 'run_command',
            ),
            false,
          );
          await tester.tap(find.byKey(const Key('access-mode-fullAccess')));
          await tester.pumpAndSettle();
          expect(bridge.policy, accessPolicy('fullAccess'));
          expect(find.byType(CheckboxListTile), findsNothing);
          expect(find.byType(AppDropdown<String>), findsNothing);
          expect(find.text('Save access'), findsNothing);
          chat.busy = true;
          await tester.pumpWidget(app());
          await tester.pumpAndSettle();
          expect(
            tester
                .widget<ListTile>(
                  find.byKey(const Key('access-mode-fullAccess')),
                )
                .enabled,
            false,
          );
          await tester.tap(find.byKey(const Key('access-mode-review')));
          await tester.pumpAndSettle();
          expect(bridge.policy, accessPolicy('review'));
          expect(chat.draft, 'Keep my draft');
          expect(tester.takeException(), null);
          await tester.pumpWidget(const SizedBox());
          chat.dispose();
        },
      );
    }
  }
  testWidgets(
    'Stale selection retains saved access; explicit refresh and selection recover',
    (tester) async {
      final bridge = PermissionBridge()..stale = true;
      final chat = ChatController(bridge)
        ..session = 'a'
        ..loading = false;
      await tester.pumpWidget(
        MaterialApp(home: TaskPermissionsInspector(chat: chat)),
      );
      await tester.pumpAndSettle();
      await tester.tap(find.byKey(const Key('access-mode-fullAccess')));
      await tester.pumpAndSettle();
      expect(bridge.policy['mode'], 'review');
      expect(find.textContaining('Permissions changed.'), findsOneWidget);
      await tester.tap(find.text('Refresh'));
      await tester.pumpAndSettle();
      expect(bridge.policy['mode'], 'review');
      await tester.tap(find.byKey(const Key('access-mode-fullAccess')));
      await tester.pumpAndSettle();
      expect(bridge.policy, accessPolicy('fullAccess'));
      await tester.pumpWidget(const SizedBox());
      chat.dispose();
    },
  );
}
