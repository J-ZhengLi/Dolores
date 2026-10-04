import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:dolores_flutter/bridge.dart';
import 'package:dolores_flutter/chat.dart';
import 'package:dolores_flutter/dolores_settings.dart';

class SettingsBridge implements ChatBridge {
  final commands = <Map<String, dynamic>>[];
  int revision = 1;
  bool stale = false;
  bool invalidWindow = false;
  Map<String, dynamic> patch = {
    'generation': {'maxOutputTokens': 256, 'timeoutSeconds': 60},
    'interaction': {'discussion': 'discuss', 'questionAssumptions': true},
  };
  Map<String, dynamic> report() => {
    'validationError': invalidWindow
        ? 'Output reservation exceeds model window.'
        : null,
    'scopes': [
      {
        'scope': 'user',
        'record': {
          'revision': 0,
          'patch': {
            'interaction': {'discussion': 'brief', 'questionAssumptions': true},
          },
        },
      },
      {
        'scope': 'project',
        'record': {'revision': 0, 'patch': {}},
      },
      {
        'scope': 'thread',
        'record': {'revision': revision, 'patch': patch},
      },
    ],
    'effective': {
      'request':
          patch['generation'] ??
          {'maxOutputTokens': 1024, 'timeoutSeconds': 180},
      'requestOrigin': patch['generation'] == null ? 'Model profile' : 'Thread',
      'interaction':
          patch['interaction'] ??
          {'discussion': 'brief', 'questionAssumptions': true},
      'interactionOrigin': patch['interaction'] == null ? 'User' : 'Thread',
      'contextWindowTokens': invalidWindow ? 8192 : 131072,
      'contextOrigin': 'Default 128K',
    },
    'taskAccess': 'Every tool operation requires review.',
    'adaptation': 'Self-updates unavailable; memory policy is separate.',
  };
  @override
  Future<void> open() async {}
  @override
  Future<void> close() async {}
  @override
  Future<dynamic> call(Map<String, dynamic> command) async {
    commands.add(command);
    if (command['command'] == 'saveScopedSettings') {
      if (stale) {
        stale = false;
        revision++;
        throw StateError(
          'Settings changed. Refresh, keep your draft and review again.',
        );
      }
      expect(command['revision'], revision);
      patch = Map<String, dynamic>.from(command['patch'] as Map);
      invalidWindow = false;
      revision++;
    }
    return report();
  }
}

Future<void> show(WidgetTester tester, ChatController chat) async {
  await tester.pumpWidget(
    MaterialApp(
      theme: ThemeData.dark(),
      home: DoloresSettingsInspector(chat: chat),
    ),
  );
  await tester.pumpAndSettle();
}

void main() {
  testWidgets(
    'Changed window exposes invalid settings and keeps reset usable',
    (tester) async {
      final bridge = SettingsBridge()..invalidWindow = true;
      final chat = ChatController(bridge)
        ..session = 'thread-a'
        ..loading = false;
      await show(tester, chat);
      expect(
        find.textContaining('Output reservation exceeds model window.'),
        findsOneWidget,
      );
      expect(find.textContaining('8192 tokens'), findsOneWidget);
      await tester.tap(find.text('Use inherited settings'));
      await tester.pumpAndSettle();
      expect(
        find.textContaining('Output reservation exceeds model window.'),
        findsNothing,
      );
      expect(
        find.text('Inherited settings restored for this scope.'),
        findsOneWidget,
      );
      await tester.pumpWidget(const SizedBox());
      chat.dispose();
    },
  );
  testWidgets(
    'Scoped output saves exact revision and reset restores inheritance without changing chat draft',
    (tester) async {
      final bridge = SettingsBridge();
      final chat = ChatController(bridge)
        ..session = 'thread-a'
        ..loading = false
        ..draft = 'Keep my work';
      await show(tester, chat);
      expect(find.textContaining('256 tokens · Thread'), findsOneWidget);
      await tester.ensureVisible(find.byKey(const Key('scoped-output')));
      await tester.enterText(find.byKey(const Key('scoped-output')), '512');
      await tester.tap(find.text('Save'));
      await tester.pumpAndSettle();
      final save = bridge.commands.last;
      expect(save['scope'], 'thread');
      expect(save['session'], 'thread-a');
      expect(save['revision'], 1);
      expect(save['patch']['generation']['maxOutputTokens'], 512);
      expect(find.text('Settings saved for future runs.'), findsOneWidget);
      await tester.tap(find.text('Use inherited settings'));
      await tester.pumpAndSettle();
      expect(bridge.commands.last['patch'], {
        'permissions': null,
        'task': null,
        'generation': null,
        'interaction': null,
      });
      expect(
        find.textContaining('1024 tokens · Model profile'),
        findsOneWidget,
      );
      expect(chat.draft, 'Keep my work');
      await tester.pumpWidget(const SizedBox());
      chat.dispose();
    },
  );

  testWidgets(
    'Stale save retains edited values and refresh requires explicit save of the new revision',
    (tester) async {
      final bridge = SettingsBridge()..stale = true;
      final chat = ChatController(bridge)
        ..session = 'thread-a'
        ..loading = false;
      await show(tester, chat);
      await tester.ensureVisible(find.byKey(const Key('scoped-output')));
      await tester.enterText(find.byKey(const Key('scoped-output')), '768');
      await tester.tap(find.text('Save'));
      await tester.pumpAndSettle();
      expect(find.textContaining('Settings changed.'), findsOneWidget);
      expect(
        tester
            .widget<TextField>(find.byKey(const Key('scoped-output')))
            .controller!
            .text,
        '768',
      );
      await tester.tap(find.text('Refresh'));
      await tester.pumpAndSettle();
      expect(
        bridge.commands
            .where((c) => c['command'] == 'saveScopedSettings')
            .length,
        1,
      );
      expect(
        tester
            .widget<TextField>(find.byKey(const Key('scoped-output')))
            .controller!
            .text,
        '768',
      );
      await tester.tap(find.text('Save'));
      await tester.pumpAndSettle();
      expect(bridge.commands.last['revision'], 2);
      expect(
        bridge.commands.last['patch']['generation']['maxOutputTokens'],
        768,
      );
      await tester.pumpWidget(const SizedBox());
      chat.dispose();
    },
  );

  testWidgets(
    'Compact busy inspection is read only and scope switch reloads interaction style',
    (tester) async {
      tester.view.physicalSize = const Size(620, 700);
      tester.view.devicePixelRatio = 1;
      addTearDown(tester.view.resetPhysicalSize);
      addTearDown(tester.view.resetDevicePixelRatio);
      final bridge = SettingsBridge();
      final chat = ChatController(bridge)
        ..session = 'thread-a'
        ..loading = false;
      await show(tester, chat);
      await tester.tap(find.byKey(const Key('settings-scope')));
      await tester.pumpAndSettle();
      await tester.tap(find.text('User defaults').last);
      await tester.pumpAndSettle();
      expect(find.text('Keep explanations brief'), findsOneWidget);
      chat.busy = true;
      await show(tester, chat);
      expect(
        tester
            .widget<FilledButton>(find.widgetWithText(FilledButton, 'Save'))
            .onPressed,
        null,
      );
      expect(
        find.textContaining('current run keeps its snapshot'),
        findsOneWidget,
      );
      expect(
        bridge.commands.where((c) => c['command'] == 'saveScopedSettings'),
        isEmpty,
      );
      expect(tester.takeException(), null);
      await tester.pumpWidget(const SizedBox());
      chat.dispose();
    },
  );
}
