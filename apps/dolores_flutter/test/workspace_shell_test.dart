import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:dolores_flutter/app_host.dart';
import 'package:dolores_flutter/chat.dart';
import 'package:dolores_flutter/workspace_shell.dart';
import 'package:dolores_flutter/settings.dart';

import 'app_host_test.dart' show HostBridge;

class FreshScheduleBridge extends HostBridge {
  bool saved = false;
  bool unavailable = false;
  int reads = 0;
  @override
  Future<dynamic> call(Map<String, dynamic> request) async {
    if (request['command'] == 'scheduledTasks') {
      reads++;
      if (unavailable) {
        throw StateError('Task storage unavailable. Try Refresh.');
      }
      return {
        'items': [
          if (saved)
            {
              'task': {
                'id': 'saved',
                'revision': 1,
                'title': 'Weather report',
                'paused': true,
                'nextDue': null,
              },
              'receipt': {
                'schedule': 'Weekdays at 21:00',
                'model': 'test-model',
                'timezone': 'Asia/Shanghai',
              },
              'occurrences': <Map>[],
            },
        ],
      };
    }
    return super.call(request);
  }
}

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
        'multipleWindowCapability': {
          'available': false,
          'status': 'held',
          'reason': 'Additional windows are unavailable in this build because their performance checks did not pass. Use split views in Folders, Source Control or Terminal.',
        },
      };
    }
    return super.call(request);
  }
}

void main() {
  testWidgets(
    'entering Scheduled sees tasks saved during a busy chat and recovers failed refresh',
    (t) async {
      t.view.physicalSize = const Size(1200, 800);
      t.view.devicePixelRatio = 1;
      addTearDown(t.view.resetPhysicalSize);
      addTearDown(t.view.resetDevicePixelRatio);
      final bridge = FreshScheduleBridge();
      final chat = ChatController(bridge)
        ..loading = false
        ..draft = 'Keep this draft';
      final host = AppHost(chat);
      await t.pumpWidget(MaterialApp(home: WorkspaceShell(host: host)));
      await t.pumpAndSettle();
      expect(host.scheduled.items, isEmpty);
      final initialReads = bridge.reads;
      bridge.saved = true;
      chat.busy = true;
      await t.tap(find.byKey(const Key('page-scheduled')));
      await t.pumpAndSettle();
      expect(find.text('Weather report'), findsWidgets);
      expect(bridge.reads, greaterThan(initialReads));
      expect(chat.busy, isTrue);
      expect(chat.draft, 'Keep this draft');
      await t.tap(find.byKey(const Key('page-home')));
      await t.pumpAndSettle();
      bridge.unavailable = true;
      await t.tap(find.byKey(const Key('page-scheduled')));
      await t.pumpAndSettle();
      expect(find.text('Weather report'), findsWidgets);
      expect(host.scheduled.error, contains('Task storage unavailable'));
      bridge.unavailable = false;
      await t.tap(find.byKey(const Key('page-home')));
      await t.pumpAndSettle();
      await t.tap(find.byKey(const Key('page-scheduled')));
      await t.pumpAndSettle();
      expect(host.scheduled.error, isNull);
      expect(chat.draft, 'Keep this draft');
      chat.busy = false;
      await t.pumpWidget(const SizedBox());
      host.dispose();
    },
  );
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
      expect(find.text('Retry'), findsOneWidget);
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
      expect(
        find.textContaining('performance checks did not pass'),
        findsNothing,
      );
      expect(find.text('Not available yet'), findsOneWidget);
      bridge.fail = true;
      await t.tap(find.byKey(const Key('experimental-multiple-window')));
      await t.pumpAndSettle();
      expect(tile('experimental-multiple-window').value, true);
      expect(find.textContaining('Your setting is unchanged'), findsOneWidget);
      bridge.fail = false;
      await t.tap(find.byKey(const Key('experimental-multiple-window')));
      await t.pumpAndSettle();
      expect(tile('experimental-multiple-window').value, false);
      expect(bridge.preferences['revision'], 1);
      expect(
        find.textContaining('performance checks did not pass'),
        findsNothing,
      );
      await t.pumpWidget(const SizedBox());
      chat.dispose();
    },
  );
  testWidgets(
    'held window Off preference remains Off after reopening settings',
    (t) async {
      final bridge = ExperimentalBridge();
      bridge.preferences['multipleWindow'] = false;
      final chat = ChatController(bridge)..loading = false;
      for (var visit = 0; visit < 2; visit++) {
        await t.pumpWidget(
          MaterialApp(
            home: SettingsWindow(
              chat: chat,
              initial: SettingsCategory.experimental,
            ),
          ),
        );
        await t.pumpAndSettle();
        expect(
          t
              .widget<SwitchListTile>(
                find.byKey(const Key('experimental-multiple-window')),
              )
              .value,
          false,
        );
        expect(find.text('Not available yet'), findsOneWidget);
        expect(t.takeException(), isNull);
        await t.pumpWidget(const SizedBox());
        await t.pumpAndSettle();
      }
      chat.dispose();
    },
  );
}
