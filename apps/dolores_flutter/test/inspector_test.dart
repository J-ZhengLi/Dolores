import 'dart:convert';

import 'package:dolores_flutter/chat.dart';
import 'package:dolores_flutter/inspector.dart';
import 'package:dolores_flutter/main.dart';
import 'package:dolores_flutter/theme.dart';
import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';

import 'history_test.dart' show HistoryBridge;

class RunBridge extends HistoryBridge {
  bool failStart = false, hold = false, stopped = false;
  bool failHistory = false;
  @override
  Future<dynamic> call(Map<String, dynamic> command) async {
    if (command['command'] == 'messagesPage' && failHistory) {
      throw StateError('Storage unavailable');
    }
    if (command['command'] == 'start' && failStart) {
      throw StateError('Connection unavailable');
    }
    if (command['command'] == 'cancel') stopped = true;
    if (command['command'] == 'poll') {
      if (hold && !stopped) return [];
      final id = command['id'];
      return [
        {
          'type': 'started',
          'id': id,
          'session': 'assigned',
          'context': {'textBytes': 10, 'maxTextBytes': 131072},
        },
        if (!stopped) ...[
          {'type': 'delta', 'id': id, 'text': 'first'},
          {'type': 'delta', 'id': id, 'text': 'second'},
        ],
        {
          'type': 'done',
          'id': id,
          if (stopped) 'error': 'Stopped' else 'answer': 'firstsecond',
        },
      ];
    }
    return super.call(command);
  }
}

Future<void> finish(ChatController chat) async {
  for (var i = 0; i < 100 && (chat.busy || chat.changing); i++) {
    await Future<void>.delayed(const Duration(milliseconds: 10));
  }
  expect(chat.busy, isFalse);
  expect(chat.changing, isFalse);
}

void main() {
  test(
    'Log distinguishes a saved reply from a later history refresh failure',
    () async {
      final bridge = RunBridge()..failHistory = true;
      final chat = ChatController(bridge)
        ..loading = false
        ..configured = true
        ..model = 'original'
        ..draft = 'private';
      addTearDown(chat.dispose);
      await chat.send();
      await finish(chat);
      expect(chat.requestLogs.map((e) => e.label), [
        'Request submitted',
        'Context prepared',
        'First response text',
        'Reply saved',
        'History refresh failed',
      ]);
      expect(chat.contextBasis, 'Saved agent input');
    },
  );
  testWidgets(
    'Circle is static when unknown and labels the app budget when known',
    (tester) async {
      for (final summary in [
        null,
        {'textBytes': 65536, 'maxTextBytes': 131072},
      ]) {
        await tester.pumpWidget(
          MaterialApp(
            home: Scaffold(
              body: ContextIndicator(
                summary: summary,
                basis: 'Last saved request',
                onPressed: () {},
              ),
            ),
          ),
        );
        final ring = tester.widget<CircularProgressIndicator>(
          find.byKey(const Key('context-ring')),
        );
        expect(ring.value, summary == null ? 0 : 0.5);
        expect(
          ring.semanticsLabel,
          contains(
            summary == null
                ? 'not inspected'
                : 'Last saved request: 50.0% of app text budget',
          ),
        );
      }
    },
  );

  testWidgets(
    'Exact context groups are readable in compact light and dark dialogs',
    (tester) async {
      tester.view.physicalSize = const Size(360, 620);
      tester.view.devicePixelRatio = 1;
      addTearDown(tester.view.resetPhysicalSize);
      addTearDown(tester.view.resetDevicePixelRatio);
      const messages = [
        {'role': 'system', 'content': 'Real system instructions'},
        {'role': 'user', 'content': 'Past question'},
        {'role': 'assistant', 'content': 'Past reply'},
        {'role': 'user', 'content': '你好 draft'},
      ];
      for (final dark in [false, true]) {
        final report = {
          'includedTurns': 1,
          'savedTurns': 8,
          'omittedTurns': 7,
          'maxTurns': 40,
          'maxTextBytes': 131072,
          'textBytes': messages.fold<int>(
            0,
            (n, m) => n + utf8.encode(m['content']!).length,
          ),
          'messages': messages,
        };
        await tester.pumpWidget(
          MaterialApp(
            theme: doloresTheme(dark),
            home: ContextInspector(report: report),
          ),
        );
        expect(find.text('Older turns left out: 7'), findsOneWidget);
        final systemGroup = find.byKey(
          const PageStorageKey('context-group-System instructions'),
        );
        final draftGroup = find.byKey(
          const PageStorageKey('context-group-Draft message'),
        );
        await tester.scrollUntilVisible(
          systemGroup,
          160,
          scrollable: find.byType(Scrollable).first,
        );
        await tester.pumpAndSettle();
        await tester.tap(
          find.descendant(
            of: systemGroup,
            matching: find.text('System instructions'),
          ),
        );
        await tester.pumpAndSettle();
        expect(find.text('Real system instructions'), findsOneWidget);
        await tester.scrollUntilVisible(
          draftGroup,
          160,
          scrollable: find.byType(Scrollable).first,
        );
        await tester.pumpAndSettle();
        await tester.tap(
          find.descendant(of: draftGroup, matching: find.text('Draft message')),
        );
        await tester.pumpAndSettle();
        expect(find.text('你好 draft'), findsOneWidget);
        expect(tester.takeException(), isNull);
        await tester.pumpWidget(const SizedBox());
      }
    },
  );

  testWidgets(
    'Trajectory pages independently and preserves main transcript draft and scroll',
    (tester) async {
      final bridge = HistoryBridge();
      final chat = ChatController(bridge);
      await chat.initialize();
      await chat.browseMessages(newer: false);
      chat.draft = 'Keep my unsent text';
      chat.scrollOffset = 120;
      final oldMessages = chat.messages;
      await tester.pumpWidget(
        MaterialApp(home: TrajectoryInspector(chat: chat)),
      );
      await tester.pumpAndSettle();
      expect(find.text('Message · #81'), findsOneWidget);
      await tester.tap(find.text('Older'));
      await tester.pumpAndSettle();
      expect(find.text('Message · #1'), findsOneWidget);
      expect(bridge.commands.last, {
        'command': 'messagesPage',
        'session': 'chat-0',
        'cursor': 81,
        'newer': false,
      });
      expect(identical(chat.messages, oldMessages), isTrue);
      expect(chat.draft, 'Keep my unsent text');
      expect(chat.scrollOffset, 120);
      expect(chat.messagesNewer, isTrue);
      await tester.tap(find.text('Log'));
      await tester.pumpAndSettle();
      expect(
        find.text('No request events in this app session.'),
        findsOneWidget,
      );
      await tester.pumpWidget(const SizedBox());
      chat.dispose();
    },
  );

  test('Lifecycle coalesces deltas and associates original model with assigned session', () async {
    final chat = ChatController(RunBridge())
      ..loading = false
      ..configured = true
      ..model = 'original'
      ..draft = 'private prompt';
    addTearDown(chat.dispose);
    await chat.send();
    await finish(chat);
    expect(chat.requestLogs.map((e) => e.label), [
      'Request submitted',
      'Context prepared',
      'First response text',
      'Reply saved',
    ]);
    expect(
      chat.requestLogs.every(
        (e) => e.session == 'assigned' && e.model == 'original',
      ),
      isTrue,
    );
    expect(
      chat.requestLogs.last.elapsedMs,
      greaterThanOrEqualTo(chat.requestLogs.first.elapsedMs),
    );
    expect(
      chat.requestLogs.map((e) => e.label).join(),
      isNot(contains('private prompt')),
    );
  });

  test('Stop records lifecycle and restores draft; failed runs remain bounded and delete removes logs', () async {
    final bridge = RunBridge()..hold = true;
    final chat = ChatController(bridge)
      ..loading = false
      ..configured = true
      ..model = 'original'
      ..draft = 'private';
    addTearDown(chat.dispose);
    await chat.send();
    await chat.stop();
    await finish(chat);
    expect(chat.draft, 'private');
    expect(chat.requestLogs.map((e) => e.label), [
      'Request submitted',
      'Stop requested',
      'Context prepared',
      'Request stopped',
    ]);
    bridge.failStart = true;
    for (var i = 0; i < 120; i++) {
      chat.draft = 'private';
      await chat.send();
    }
    expect(chat.requestLogs.length, 200);
    expect(chat.requestLogs.last.label, 'Request failed');
    await chat.delete('assigned');
    expect(chat.requestLogs, isEmpty);
  });

  testWidgets(
    'Footer removes help and typing invalidates previous preview without fetching',
    (tester) async {
      final bridge = HistoryBridge();
      final chat = ChatController(bridge)
        ..loading = false
        ..configured = true
        ..model = 'fixture'
        ..enabledModels = ['fixture']
        ..contextBasis = 'Next message preview'
        ..contextSummary = {'textBytes': 20, 'maxTextBytes': 131072};
      await tester.pumpWidget(DoloresApp(chat: chat));
      expect(find.textContaining('Enter to send'), findsNothing);
      expect(find.byKey(const Key('context-ring')), findsOneWidget);
      final actions = find.byKey(const Key('composer-actions'));
      final picker = find.byKey(const Key('chat-model-picker'));
      final ring = find.byKey(const Key('context-preview'));
      expect(find.descendant(of: actions, matching: picker), findsOneWidget);
      expect(find.descendant(of: actions, matching: ring), findsOneWidget);
      expect(
        tester.getTopLeft(ring).dx,
        greaterThanOrEqualTo(tester.getBottomRight(picker).dx),
      );
      expect(
        tester.getRect(actions).top,
        greaterThan(
          tester.getRect(find.byKey(const Key('composer'))).bottom - 1,
        ),
      );
      await tester.enterText(find.byKey(const Key('composer')), 'new draft');
      await tester.pump();
      expect(chat.contextSummary, isNull);
      expect(bridge.commands, isEmpty);
      await tester.pumpWidget(const SizedBox());
    },
  );

  testWidgets(
    'Live log can be inspected without history reads during generation',
    (tester) async {
      final bridge = RunBridge()..hold = true;
      final chat = ChatController(bridge)
        ..loading = false
        ..configured = true
        ..model = 'fixture'
        ..draft = 'private';
      await chat.send();
      await tester.pumpWidget(
        MaterialApp(home: TrajectoryInspector(chat: chat)),
      );
      await tester.tap(find.text('Log'));
      await tester.pump();
      expect(find.text('Request submitted'), findsOneWidget);
      expect(
        bridge.commands.where((c) => c['command'] == 'messagesPage'),
        isEmpty,
      );
      await chat.stop();
      await tester.pump(const Duration(milliseconds: 100));
      await tester.pump(const Duration(milliseconds: 100));
      expect(find.text('Request stopped'), findsOneWidget);
      expect(tester.takeException(), isNull);
      await tester.pumpWidget(const SizedBox());
      chat.dispose();
    },
  );
}
