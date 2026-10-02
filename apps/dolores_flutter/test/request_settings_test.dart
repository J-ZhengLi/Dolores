import 'dart:async';

import 'support/workspaces.dart';

import 'package:dolores_flutter/bridge.dart';
import 'package:dolores_flutter/chat.dart';
import 'package:dolores_flutter/main.dart';
import 'package:dolores_flutter/request_settings.dart';
import 'package:dolores_flutter/theme.dart';
import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';

class SettingsBridge implements ChatBridge {
  final commands = <Map<String, dynamic>>[];
  Map<String, dynamic> settings = {
    'maxOutputTokens': 2048,
    'timeoutSeconds': 180,
  };
  Completer<void>? pending;
  bool failSave = false, failRequest = true;
  String kind = 'timeout';
  String input = '';
  List<Map<String, dynamic>> messages = [];
  @override
  Future<void> open() async {}
  @override
  Future<void> close() async {}
  @override
  Future<dynamic> call(Map<String, dynamic> command) async {
    commands.add(command);
    switch (command['command']) {
      case 'createSession':
        return createdWorkspace(command);
      case 'bootstrap':
        return {
          'sessions': <Map<String, dynamic>>[],
          'preferences': {'baseUrl': 'http://localhost/v1', 'model': 'fixture'},
          'requestSettings': settings,
          'configured': true,
          'enabledModels': ['fixture'],
        };
      case 'setRequestSettings':
        if (pending != null) await pending!.future;
        if (failSave) throw StateError('Settings could not be saved.');
        settings = Map.of(command['settings'] as Map<String, dynamic>);
      case 'start':
        input = command['input'] as String;
      case 'poll':
        final id = command['id'];
        if (!failRequest) {
          messages = [
            {'id': 1, 'role': 'user', 'content': input},
            {'id': 2, 'role': 'assistant', 'content': 'Complete answer'},
          ];
        }
        return [
          {'type': 'started', 'id': id, 'session': 'run'},
          {'type': 'delta', 'id': id, 'text': 'Partial answer'},
          {
            'type': 'done',
            'id': id,
            if (failRequest) ...{
              'error': 'Request failed.',
              'recovery': {
                'kind': kind,
                'retryable': !['access', 'stopped'].contains(kind),
                'guidance': 'Your draft is restored; choose an action.',
              },
            } else
              'answer': 'Complete answer',
          },
        ];
      case 'messagesPage':
        return {'items': messages, 'hasOlder': false, 'hasNewer': false};
    }
    return null;
  }
}

void compact(WidgetTester tester) {
  tester.view.physicalSize = const Size(390, 740);
  tester.view.devicePixelRatio = 1;
  addTearDown(tester.view.resetPhysicalSize);
  addTearDown(tester.view.resetDevicePixelRatio);
}

void main() {
  testWidgets(
    'Request settings validate integers, restore defaults locally and preserve draft in both themes',
    (tester) async {
      compact(tester);
      for (final dark in [false, true]) {
        final bridge = SettingsBridge();
        final chat = ChatController(bridge)
          ..loading = false
          ..draft = 'Keep this draft';
        await tester.pumpWidget(
          MaterialApp(
            theme: doloresTheme(dark),
            home: Scaffold(body: RequestSettingsDialog(chat: chat)),
          ),
        );
        await tester.enterText(
          find.byKey(const Key('output-token-limit')),
          '1.5',
        );
        await tester.enterText(find.byKey(const Key('request-timeout')), '901');
        await tester.tap(find.byKey(const Key('save-request-settings')));
        await tester.pumpAndSettle();
        expect(
          find.text('Enter a whole number from 1 to 32768.'),
          findsOneWidget,
        );
        expect(
          find.text('Enter a whole number from 1 to 900.'),
          findsOneWidget,
        );
        expect(bridge.commands, isEmpty);
        await tester.ensureVisible(
          find.byKey(const Key('reset-request-settings')),
        );
        await tester.tap(find.byKey(const Key('reset-request-settings')));
        await tester.pumpAndSettle();
        expect(
          tester
              .widget<TextFormField>(
                find.byKey(const Key('output-token-limit')),
              )
              .controller!
              .text,
          '2048',
        );
        expect(bridge.commands, isEmpty);
        expect(chat.draft, 'Keep this draft');
        expect(tester.takeException(), isNull);
        await tester.pumpWidget(const SizedBox());
        chat.dispose();
      }
    },
  );
  test(
    'Saving settings excludes generation and leaves previous state on failure',
    () async {
      final bridge = SettingsBridge()..pending = Completer();
      final chat = ChatController(bridge)
        ..loading = false
        ..configured = true
        ..draft = 'unsent';
      addTearDown(chat.dispose);
      final save = chat.saveRequestSettings({
        'maxOutputTokens': 4096,
        'timeoutSeconds': 300,
      });
      expect(chat.changing, isTrue);
      await chat.send();
      expect(bridge.commands.single['command'], 'setRequestSettings');
      bridge.pending!.complete();
      await save;
      expect(chat.requestSettings['maxOutputTokens'], 4096);
      expect(chat.draft, 'unsent');
      bridge.failSave = true;
      expect(
        chat.saveRequestSettings({
          'maxOutputTokens': 2048,
          'timeoutSeconds': 180,
        }),
        throwsStateError,
      );
      await Future<void>.delayed(Duration.zero);
      expect(chat.requestSettings['maxOutputTokens'], 4096);
      expect(chat.changing, isFalse);
      chat.busy = true;
      await expectLater(
        chat.saveRequestSettings({
          'maxOutputTokens': 2048,
          'timeoutSeconds': 180,
        }),
        throwsStateError,
      );
    },
  );
  testWidgets(
    'Pending save keeps the dialog present and failures can be corrected',
    (tester) async {
      final bridge = SettingsBridge()
        ..pending = Completer()
        ..failSave = true;
      final chat = ChatController(bridge)..loading = false;
      await tester.pumpWidget(
        MaterialApp(
          home: Builder(
            builder: (context) => TextButton(
              onPressed: () => showDialog<void>(
                context: context,
                builder: (_) => RequestSettingsDialog(chat: chat),
              ),
              child: const Text('Open'),
            ),
          ),
        ),
      );
      await tester.tap(find.text('Open'));
      await tester.pumpAndSettle();
      await tester.tap(find.byKey(const Key('save-request-settings')));
      await tester.pump();
      expect(find.text('Saving…'), findsOneWidget);
      expect(
        tester
            .widget<TextButton>(find.widgetWithText(TextButton, 'Cancel'))
            .onPressed,
        isNull,
      );
      bridge.pending!.complete();
      await tester.pumpAndSettle();
      expect(
        find.textContaining('Settings could not be saved.'),
        findsOneWidget,
      );
      expect(chat.changing, isFalse);
      bridge.failSave = false;
      await tester.tap(find.byKey(const Key('save-request-settings')));
      await tester.pumpAndSettle();
      expect(find.byType(RequestSettingsDialog), findsNothing);
      await tester.pumpWidget(const SizedBox());
      chat.dispose();
    },
  );
  testWidgets(
    'Timeout restores source and retries only after an explicit click using the edited draft',
    (tester) async {
      compact(tester);
      final bridge = SettingsBridge();
      final chat = ChatController(bridge)
        ..loading = false
        ..configured = true
        ..model = 'fixture'
        ..draft = '# Draft\n\n```rust\nfn main() {}\n```';
      await tester.pumpWidget(DoloresApp(chat: chat));
      await chat.send();
      await tester.pump(const Duration(milliseconds: 100));
      await tester.pumpAndSettle();
      expect(chat.messages, isEmpty);
      expect(chat.partial, '');
      expect(chat.draft, contains('```rust'));
      expect(find.byKey(const Key('retry-message')), findsOneWidget);
      expect(find.byKey(const Key('failure-request-settings')), findsOneWidget);
      await tester.pump(const Duration(seconds: 1));
      expect(bridge.commands.where((c) => c['command'] == 'start').length, 1);
      await tester.enterText(
        find.byKey(const Key('composer-field-0')),
        'Edited retry',
      );
      await tester.pumpAndSettle();
      bridge.failRequest = false;
      await tester.tap(find.byKey(const Key('retry-message')));
      await tester.pump(const Duration(milliseconds: 100));
      await tester.pumpAndSettle();
      expect(
        bridge.commands.where((c) => c['command'] == 'start').last['input'],
        '# Edited retry\n\n```rust\nfn main() {}\n```',
      );
      expect(chat.messages.length, 2);
      expect(find.byKey(const Key('retry-message')), findsNothing);
      expect(tester.takeException(), isNull);
      await tester.pumpWidget(const SizedBox());
      chat.dispose();
    },
  );
  testWidgets(
    'Access denial offers connection settings and Stop does not offer a failure retry',
    (tester) async {
      compact(tester);
      for (final kind in ['access', 'stopped']) {
        final bridge = SettingsBridge()..kind = kind;
        final chat = ChatController(bridge)
          ..loading = false
          ..configured = true
          ..model = 'fixture'
          ..draft = 'unsent';
        await tester.pumpWidget(DoloresApp(chat: chat));
        await chat.send();
        await tester.pump(const Duration(milliseconds: 100));
        await tester.pumpAndSettle();
        expect(find.byKey(const Key('retry-message')), findsNothing);
        expect(
          find.byKey(const Key('failure-connection')),
          kind == 'access' ? findsOneWidget : findsNothing,
        );
        expect(chat.draft, 'unsent');
        expect(tester.takeException(), isNull);
        await tester.pumpWidget(const SizedBox());
        chat.dispose();
      }
    },
  );
  test('Recovery advice is not attached to an unrelated later error', () async {
    final bridge = SettingsBridge();
    final chat = ChatController(bridge)
      ..loading = false
      ..configured = true
      ..draft = 'unsent';
    addTearDown(chat.dispose);
    await chat.send();
    for (var i = 0; i < 100 && chat.busy; i++) {
      await Future<void>.delayed(const Duration(milliseconds: 10));
    }
    expect(chat.activeRecovery?['kind'], 'timeout');
    chat.error = 'A different operation failed';
    expect(chat.activeRecovery, isNull);
  });
}
