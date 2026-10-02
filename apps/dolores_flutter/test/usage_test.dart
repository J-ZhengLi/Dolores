import 'dart:async';

import 'package:dolores_flutter/bridge.dart';
import 'package:dolores_flutter/chat.dart';
import 'package:dolores_flutter/main.dart';
import 'package:dolores_flutter/usage_details.dart';
import 'package:dolores_flutter/theme.dart';
import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';

const summary = {
  'includedTurns': 40,
  'savedTurns': 61,
  'omittedTurns': 21,
  'textBytes': 8192,
  'maxTextBytes': 131072,
  'maxTurns': 40,
};
const metadata = {
  'model': 'original-model',
  'usage': {
    'inputTokens': 0,
    'outputTokens': 12,
    'totalTokens': null,
    'cachedInputTokens': 0,
    'reasoningTokens': null,
  },
  'context': summary,
};

class UsageBridge implements ChatBridge {
  final commands = <Map<String, dynamic>>[];
  Completer<dynamic>? pending;
  bool fail = false;
  @override
  Future<void> open() async {}
  @override
  Future<void> close() async {}
  @override
  Future<dynamic> call(Map<String, dynamic> command) async {
    commands.add(command);
    if (fail) throw StateError('Context unavailable.');
    if (command['command'] == 'context') {
      return pending == null ? summary : pending!.future;
    }
    return null;
  }
}

void main() {
  testWidgets(
    'Usage shows reported zero and missing fields in both compact themes',
    (tester) async {
      tester.view.physicalSize = const Size(390, 800);
      tester.view.devicePixelRatio = 1;
      addTearDown(tester.view.resetPhysicalSize);
      addTearDown(tester.view.resetDevicePixelRatio);
      for (final dark in [false, true]) {
        await tester.pumpWidget(
          MaterialApp(
            theme: doloresTheme(dark),
            home: const Scaffold(body: UsageDetails(metadata: metadata)),
          ),
        );
        await tester.tap(find.text('Tokens: 0 in · 12 out'));
        await tester.pumpAndSettle();
        expect(find.text('Input tokens: 0'), findsOneWidget);
        expect(find.text('Total tokens: Unavailable'), findsOneWidget);
        expect(find.text('Cached input tokens: 0'), findsOneWidget);
        expect(find.text('Model: original-model'), findsOneWidget);
        expect(find.text('Older turns left out: 21'), findsOneWidget);
        expect(tester.takeException(), isNull);
        await tester.tap(find.text('Close'));
        await tester.pumpAndSettle();
      }
    },
  );
  testWidgets(
    'Legacy replies show unavailable usage and context rather than zero',
    (tester) async {
      await tester.pumpWidget(
        const MaterialApp(home: Scaffold(body: UsageDetails())),
      );
      await tester.tap(find.text('Usage unavailable'));
      await tester.pumpAndSettle();
      expect(find.text('Input tokens: Unavailable'), findsOneWidget);
      expect(find.text('Saved request context: Unavailable'), findsOneWidget);
    },
  );
  test('Context inspection preserves paged view and draft and excludes other actions', () async {
    final bridge = UsageBridge()..pending = Completer();
    final chat = ChatController(bridge)
      ..loading = false
      ..configured = true
      ..session = 'long'
      ..workspaceKind = 'side'
      ..draft = '你好'
      ..messagesNewer = true
      ..messages = [
        {'id': 1, 'role': 'user', 'content': 'older page'},
      ];
    addTearDown(chat.dispose);
    final request = chat.previewContext();
    expect(chat.changing, isTrue);
    await chat.send();
    chat.newChat();
    expect(bridge.commands, [
      {'command': 'context', 'session': 'long', 'input': '你好'},
    ]);
    bridge.pending!.complete(summary);
    expect(await request, summary);
    expect(chat.changing, isFalse);
    expect(chat.draft, '你好');
    expect(chat.messages.single['content'], 'older page');
    expect(chat.messagesNewer, isTrue);
    chat.busy = true;
    expect(await chat.previewContext(), isNull);
    expect(bridge.commands.length, 1);
  });
  test(
    'Failed context inspection preserves draft and clears the action guard',
    () async {
      final bridge = UsageBridge()..fail = true;
      final chat = ChatController(bridge)
        ..loading = false
        ..draft = 'unsent';
      addTearDown(chat.dispose);
      expect(await chat.previewContext(), isNull);
      expect(chat.error, contains('Context unavailable'));
      expect(chat.changing, isFalse);
      expect(chat.draft, 'unsent');
    },
  );
  testWidgets(
    'Context button fetches only on demand and reply retains its original model',
    (tester) async {
      final bridge = UsageBridge();
      final chat = ChatController(bridge)
        ..loading = false
        ..configured = true
        ..model = 'current-model'
        ..messages = [
          {
            'id': 2,
            'role': 'assistant',
            'content': 'answer',
            'metadata': metadata,
          },
        ];
      await tester.pumpWidget(DoloresApp(chat: chat));
      await tester.pumpAndSettle();
      expect(bridge.commands, isEmpty);
      await tester.tap(find.byKey(const Key('context-preview')));
      await tester.pumpAndSettle();
      expect(find.text('Context for your next message'), findsOneWidget);
      expect(find.text('Recent turns included: 40'), findsOneWidget);
      expect(bridge.commands.single['command'], 'context');
      await tester.tap(find.text('Close'));
      await tester.pumpAndSettle();
      await tester.tap(find.text('Tokens: 0 in · 12 out'));
      await tester.pumpAndSettle();
      expect(find.text('Model: original-model'), findsOneWidget);
      expect(tester.takeException(), isNull);
      await tester.pumpWidget(const SizedBox());
    },
  );
}
