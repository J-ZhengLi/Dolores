import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:dolores_flutter/app_host.dart';
import 'package:dolores_flutter/chat.dart';
import 'package:dolores_flutter/workspace_shell.dart';
import 'package:dolores_flutter/settings.dart';

import 'app_host_test.dart' show HostBridge;

class ExperimentalBridge extends HostBridge {
  bool fail = false;
  Map<String, dynamic> preferences = {
    'revision': 0,
    'multipleWindow': true,
    'preventWindowsFromLocked': false,
  };
  @override
  Future<dynamic> call(Map<String, dynamic> request) async {
    if (request['command'] == 'saveExperimentalPreferences') {
      if (fail) throw StateError('Storage is unavailable.');
      preferences = {
        ...request['preferences'] as Map<String, dynamic>,
        'revision': preferences['revision'] + 1,
      };
    }
    if ('${request['command']}'.contains('ExperimentalPreferences') ||
        request['command'] == 'experimentalPreferences') {
      return {
        'preferences': preferences,
        'windows': true,
        'keepAwakeActive': preferences['preventWindowsFromLocked'],
      };
    }
    return super.call(request);
  }
}

void main() {
  testWidgets(
    'rail, title toggle and drag hide preserve Home draft and root; later pages are truthful',
    (t) async {
      t.view.physicalSize = const Size(1200, 800);
      t.view.devicePixelRatio = 1;
      addTearDown(t.view.resetPhysicalSize);
      addTearDown(t.view.resetDevicePixelRatio);
      final chat = ChatController(HostBridge())
        ..loading = false
        ..workspaceRoot = 'C:/A'
        ..workspaceKind = 'project'
        ..draft = 'Retained draft';
      final host = AppHost(chat);
      await t.pumpWidget(MaterialApp(home: WorkspaceShell(host: host)));
      await t.pumpAndSettle();
      expect(find.byKey(const Key('rail-settings')), findsOneWidget);
      await t.tap(find.byKey(const Key('page-folders')));
      await t.pumpAndSettle();
    expect(host.projectRoot, 'C:/A');
      expect(find.byType(DropdownButton<String>), findsNothing);
      await t.tap(find.byKey(const Key('title-panel-toggle')));
      await t.pumpAndSettle();
      expect(host.panelHidden, true);
      await t.tap(find.byKey(const Key('title-panel-toggle')));
      await t.pumpAndSettle();
      await t.drag(
        find.byKey(const Key('page-panel-divider')),
        const Offset(-240, 0),
      );
      await t.pumpAndSettle();
      expect(host.panelHidden, true);
      await t.tap(find.byKey(const Key('page-terminal')));
      await t.pumpAndSettle();
      expect(
        find.text('Terminal is planned for milestone 18.'),
        findsOneWidget,
      );
      await t.tap(find.byKey(const Key('page-home')));
      await t.pumpAndSettle();
      expect(chat.draft, 'Retained draft');
      expect(host.projectRoot, 'C:/A');
      t.view.physicalSize = const Size(420, 600);
      await t.pumpAndSettle();
      expect(find.byKey(const Key('rail-settings')), findsOneWidget);
      expect(t.takeException(), isNull);
      await t.pumpWidget(const SizedBox());
      host.dispose();
    },
  );
  testWidgets(
    'Experimental defaults and failed writes preserve the acknowledged preference',
    (t) async {
      t.view.physicalSize = const Size(1100, 800);
      t.view.devicePixelRatio = 1;
      addTearDown(t.view.resetPhysicalSize);
      addTearDown(t.view.resetDevicePixelRatio);
      final bridge = ExperimentalBridge();
      final chat = ChatController(bridge)..loading = false;
      await t.pumpWidget(
        MaterialApp(
          home: SettingsWindow(
            chat: chat,
            initial: SettingsCategory.experimental,
          ),
        ),
      );
      await t.pumpAndSettle();
      SwitchListTile tile(String key) =>
          t.widget<SwitchListTile>(find.byKey(Key(key)));
      expect(tile('experimental-multiple-window').value, true);
      expect(tile('experimental-keep-awake').value, false);
      bridge.fail = true;
      await t.tap(find.byKey(const Key('experimental-multiple-window')));
      await t.pumpAndSettle();
      expect(tile('experimental-multiple-window').value, true);
      expect(
        find.textContaining('previous preference is retained'),
        findsOneWidget,
      );
      bridge.fail = false;
      await t.tap(find.byKey(const Key('experimental-multiple-window')));
      await t.pumpAndSettle();
      expect(tile('experimental-multiple-window').value, false);
      expect(bridge.preferences['revision'], 1);
      await t.pumpWidget(const SizedBox());
      chat.dispose();
    },
  );
}
