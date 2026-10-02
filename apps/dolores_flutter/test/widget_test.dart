import 'package:flutter/material.dart';

import 'support/workspaces.dart';

import 'package:flutter/services.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:dolores_flutter/bridge.dart';
import 'package:dolores_flutter/chat.dart';
import 'package:dolores_flutter/main.dart';

class FakeBridge implements ChatBridge {
  final commands = <Map<String, dynamic>>[];
  @override
  Future<dynamic> call(Map<String, dynamic> command) async {
    commands.add(command);
    if (command['command'] == 'createSession') return createdWorkspace(command);
    return null;
  }

  @override
  Future<void> open() async {}
  @override
  Future<void> close() async {}
}

class ConnectionBridge extends FakeBridge {
  bool configured = false, remembered = true, savedKey = false;
  String? warning = 'Unlock secure storage and retry.';
  String activeModel = 'fixture';
  List<String> choices = ['fixture'];
  bool listingFails = false;
  @override
  Future<dynamic> call(Map<String, dynamic> command) async {
    commands.add(command);
    switch (command['command']) {
      case 'bootstrap':
        return {
          'sessions': <Map<String, dynamic>>[],
          'preferences': {
            'baseUrl': 'http://localhost:11434/v1',
            'model': activeModel,
          },
          'configured': configured,
          'rememberConnection': remembered,
          'hasSavedKey': savedKey,
          'connectionWarning': warning,
          'enabledModels': choices,
        };
      case 'recoverConnection':
        configured = true;
        savedKey = true;
        warning = null;
      case 'forgetConnection':
        configured = false;
        remembered = false;
        savedKey = false;
        warning = null;
      case 'configure':
        activeModel = command['preferences']['model'] as String;
        choices =
            (command['enabledModels'] as List?)?.cast<String>() ??
            [activeModel];
        configured = true;
        remembered = command['rememberConnection'] == true;
        warning = null;
      case 'listModels':
        if (listingFails) {
          throw StateError('Model listing unavailable. Add a model manually.');
        }
        return ['faster', 'fixture', 'larger'];
      case 'selectModel':
        activeModel = command['model'] as String;
    }
    return null;
  }
}

void main() {
  testWidgets(
    'Fetch, enable a subset, and switch beside the draft without losing it',
    (tester) async {
      tester.view.physicalSize = const Size(1120, 780);
      tester.view.devicePixelRatio = 1;
      addTearDown(tester.view.resetPhysicalSize);
      addTearDown(tester.view.resetDevicePixelRatio);
      final bridge = ConnectionBridge()
        ..configured = true
        ..warning = null;
      final chat = ChatController(bridge);
      await chat.initialize();
      await tester.pumpWidget(DoloresApp(chat: chat));
      await tester.tap(find.byKey(const Key('connection')));
      await tester.pumpAndSettle();
      expect(find.text('Model name'), findsNothing);
      await tester.tap(find.byKey(const Key('fetch-models')));
      await tester.pumpAndSettle();
      await tester.tap(find.byKey(const ValueKey('enable-model-faster')));
      await tester.tap(find.byKey(const Key('save-connection')));
      await tester.pumpAndSettle();
      expect(chat.enabledModels, ['fixture', 'faster']);
      await tester.enterText(
        find.byKey(const Key('composer')),
        'Keep my draft',
      );
      await tester.tap(find.byKey(const Key('chat-model-picker')));
      await tester.pumpAndSettle();
      await tester.tap(find.text('faster').last);
      await tester.pumpAndSettle();
      expect(chat.model, 'faster');
      expect(chat.draft, 'Keep my draft');
      chat.busy = true;
      await tester.pumpWidget(DoloresApp(chat: chat));
      expect(
        tester
            .widget<PopupMenuButton<String>>(
              find.byKey(const Key('chat-model-picker')),
            )
            .enabled,
        isFalse,
      );
      await tester.pumpWidget(const SizedBox());
      chat.dispose();
    },
  );
  testWidgets(
    'Unsupported listing offers manual fallback and blocks an empty selection',
    (tester) async {
      final bridge = ConnectionBridge()
        ..activeModel = ''
        ..choices = []
        ..listingFails = true;
      final chat = ChatController(bridge);
      await chat.initialize();
      await tester.pumpWidget(
        MaterialApp(
          home: Scaffold(body: ConnectionDialog(chat: chat)),
        ),
      );
      expect(
        tester
            .widget<FilledButton>(find.byKey(const Key('save-connection')))
            .onPressed,
        isNull,
      );
      await tester.tap(find.byKey(const Key('fetch-models')));
      await tester.pumpAndSettle();
      await tester.ensureVisible(find.byKey(const Key('manual-model')));
      await tester.enterText(
        find.byKey(const Key('manual-model')),
        'custom-model',
      );
      await tester.tap(find.byKey(const Key('add-manual-model')));
      await tester.pumpAndSettle();
      expect(
        tester
            .widget<FilledButton>(find.byKey(const Key('save-connection')))
            .onPressed,
        isNotNull,
      );
      await tester.pumpWidget(const SizedBox());
      chat.dispose();
    },
  );
  testWidgets(
    'Retry restores connection; blank saved key stays hidden; Forget keeps history',
    (tester) async {
      tester.view.physicalSize = const Size(1120, 780);
      tester.view.devicePixelRatio = 1;
      addTearDown(tester.view.resetPhysicalSize);
      addTearDown(tester.view.resetDevicePixelRatio);
      final bridge = ConnectionBridge();
      final chat = ChatController(bridge);
      await chat.initialize();
      chat.messages = [
        {'role': 'user', 'content': 'Keep this conversation'},
      ];
      await tester.pumpWidget(DoloresApp(chat: chat));
      expect(find.byKey(const Key('connection-warning')), findsOneWidget);
      await tester.tap(find.byKey(const Key('retry-connection')));
      await tester.pumpAndSettle();
      expect(chat.configured, isTrue);
      expect(find.byKey(const Key('connection-warning')), findsNothing);
      await tester.tap(find.byKey(const Key('connection')));
      await tester.pumpAndSettle();
      expect(
        tester
            .widget<TextField>(find.byKey(const Key('api-key')))
            .controller!
            .text,
        isEmpty,
      );
      await tester.tap(find.byKey(const Key('save-connection')));
      await tester.pumpAndSettle();
      final save = bridge.commands.firstWhere(
        (command) => command['command'] == 'configure',
      );
      expect(save['apiKey'], isNull);
      expect(save['rememberConnection'], isTrue);
      await tester.tap(find.byKey(const Key('connection')));
      await tester.pumpAndSettle();
      await tester.ensureVisible(find.byKey(const Key('forget-connection')));
      await tester.tap(find.byKey(const Key('forget-connection')));
      await tester.pumpAndSettle();
      expect(chat.configured, isFalse);
      expect(chat.messages.single['content'], 'Keep this conversation');
      await tester.pumpWidget(const SizedBox());
      chat.dispose();
    },
  );
  testWidgets(
    'Enter sends, Shift+Enter preserves draft, Stop cancels the active run',
    (tester) async {
      tester.view.physicalSize = const Size(1120, 780);
      tester.view.devicePixelRatio = 1;
      addTearDown(tester.view.resetPhysicalSize);
      addTearDown(tester.view.resetDevicePixelRatio);
      final bridge = FakeBridge();
      final chat = ChatController(bridge)
        ..loading = false
        ..configured = true
        ..model = 'fixture';
      await tester.pumpWidget(DoloresApp(chat: chat));
      await tester.enterText(find.byKey(const Key('composer')), 'hello');
      await tester.sendKeyDownEvent(LogicalKeyboardKey.shiftLeft);
      await tester.sendKeyEvent(LogicalKeyboardKey.enter);
      await tester.sendKeyUpEvent(LogicalKeyboardKey.shiftLeft);
      await tester.pump();
      expect(bridge.commands, isEmpty);
      await tester.sendKeyEvent(LogicalKeyboardKey.enter);
      await tester.pump();
      expect(
        bridge.commands.singleWhere((c) => c['command'] == 'start')['input'],
        startsWith('hello'),
      );
      await tester.tap(find.byKey(const Key('send')));
      await tester.pump();
      expect(bridge.commands.last['command'], 'cancel');
      await tester.pumpWidget(const SizedBox());
      chat.dispose();
      await tester.pump();
    },
  );
  testWidgets('Narrow window exposes a drawer and connection form', (
    tester,
  ) async {
    tester.view.physicalSize = const Size(620, 700);
    tester.view.devicePixelRatio = 1;
    addTearDown(tester.view.resetPhysicalSize);
    addTearDown(tester.view.resetDevicePixelRatio);
    final chat = ChatController(FakeBridge())..loading = false;
    await tester.pumpWidget(DoloresApp(chat: chat, themeMode: ThemeMode.dark));
    await tester.tap(find.byTooltip('Conversations'));
    await tester.pumpAndSettle();
    await tester.tap(find.byKey(const Key('connection')));
    await tester.pumpAndSettle();
    expect(find.byKey(const Key('base-url')), findsOneWidget);
    expect(find.byKey(const Key('api-key')), findsOneWidget);
    expect(tester.takeException(), isNull);
    await tester.pumpWidget(const SizedBox());
    chat.dispose();
    await tester.pump();
  });
}
