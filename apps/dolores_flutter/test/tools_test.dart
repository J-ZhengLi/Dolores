import 'dart:async';

import 'package:dolores_flutter/bridge.dart';
import 'package:dolores_flutter/chat.dart';
import 'package:dolores_flutter/main.dart';
import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';

class ToolBridge implements ChatBridge {
  final commands = <Map<String, dynamic>>[];
  final queue = <Map<String, dynamic>>[];
  List<Map<String, dynamic>> messages = [];
  String input = '';
  Completer<void>? pending;
  @override
  Future<void> open() async {}
  @override
  Future<void> close() async {}
  @override
  Future<dynamic> call(Map<String, dynamic> command) async {
    commands.add(command);
    final id = command['id'];
    switch (command['command']) {
      case 'start':
        input = command['input'];
        queue.addAll([
          {'type': 'started', 'id': id, 'session': 'run'},
          {'type': 'modelStep', 'id': id, 'number': 1},
          {
            'type': 'toolApproval',
            'id': id,
            'request': {
              'callId': 'file-one',
              'name': 'read_text_file',
              'target': 'readme.txt',
            },
          },
        ]);
      case 'poll':
        final events = List.of(queue);
        queue.clear();
        return events;
      case 'approveTool':
        if (pending != null) await pending!.future;
        final allowed = command['allow'] == true;
        final record = {
          'callId': 'file-one',
          'name': 'read_text_file',
          'target': 'readme.txt',
          'status': allowed ? 'read' : 'denied',
          'content': allowed ? 'File text 世界' : 'User denied this read',
        };
        messages = [
          {'id': 1, 'role': 'user', 'content': input},
          {
            'id': 2,
            'role': 'assistant',
            'content': 'Final answer',
            'metadata': {
              'model': 'fixture',
              'usage': null,
              'agent': {
                'modelCalls': 2,
                'usageByCall': [null, null],
                'tools': [record],
              },
            },
          },
        ];
        queue.addAll([
          {'type': 'toolResult', 'id': id, 'record': record},
          {'type': 'modelStep', 'id': id, 'number': 2},
          {'type': 'delta', 'id': id, 'text': 'Final answer'},
          {'type': 'done', 'id': id, 'answer': 'Final answer'},
        ]);
      case 'cancel':
        queue.clear();
        queue.add({
          'type': 'done',
          'id': id,
          'error': 'Response stopped. Your message was not saved.',
        });
      case 'messagesPage':
        return {'items': messages, 'hasOlder': false, 'hasNewer': false};
      case 'bootstrap':
        return {
          'sessions': <Map<String, dynamic>>[],
          'preferences': {'baseUrl': 'http://localhost/v1', 'model': 'fixture'},
          'configured': true,
          'enabledModels': ['fixture'],
        };
    }
    return null;
  }
}

ChatController ready(ToolBridge bridge) => ChatController(bridge)
  ..loading = false
  ..configured = true
  ..model = 'fixture'
  ..draft = 'Read readme'
  ..workspaceRoot = 'C:/chosen/project';
void compact(WidgetTester tester) {
  tester.view.physicalSize = const Size(390, 740);
  tester.view.devicePixelRatio = 1;
  addTearDown(tester.view.resetPhysicalSize);
  addTearDown(tester.view.resetDevicePixelRatio);
}

void main() {
  testWidgets(
    'Allow and Deny stay explicit and saved results are inspectable in both compact themes',
    (tester) async {
      compact(tester);
      for (final dark in [false, true]) {
        for (final allow in [false, true]) {
          final bridge = ToolBridge();
          final chat = ready(bridge);
          await tester.pumpWidget(
            DoloresApp(
              chat: chat,
              themeMode: dark ? ThemeMode.dark : ThemeMode.light,
            ),
          );
          await chat.send();
          await tester.pump(const Duration(milliseconds: 100));
          await tester.pumpAndSettle();
          expect(find.text('Allow a file read?'), findsOneWidget);
          expect(find.text('Folder: project'), findsOneWidget);
          expect(chat.busy, isTrue);
          expect(
            bridge.commands.where((c) => c['command'] == 'approveTool'),
            isEmpty,
          );
          expect(bridge.commands.first['workspace'], 'C:/chosen/project');
          await tester.tap(find.byKey(Key(allow ? 'allow-tool' : 'deny-tool')));
          await tester.pump(const Duration(milliseconds: 250));
          await tester.pumpAndSettle();
          expect(chat.busy, isFalse);
          expect(find.byKey(const Key('tool-approval')), findsNothing);
          final decision = bridge.commands
              .where((c) => c['command'] == 'approveTool')
              .single;
          expect(decision['callId'], 'file-one');
          expect(decision['id'], 1);
          expect(decision['allow'], allow);
          expect(
            chat.messages.last['metadata']['agent']['tools'].single['status'],
            allow ? 'read' : 'denied',
          );
          expect(find.text('Run details · 2 model calls'), findsOneWidget);
          await tester.ensureVisible(
            find.text('File read · ${allow ? 'read' : 'denied'}'),
          );
          await tester.tap(
            find.text('File read · ${allow ? 'read' : 'denied'}'),
          );
          await tester.pumpAndSettle();
          expect(
            find.text(allow ? 'File text 世界' : 'User denied this read'),
            findsOneWidget,
          );
          expect(tester.takeException(), isNull);
          await tester.pumpWidget(const SizedBox());
          chat.dispose();
        }
      }
    },
  );
  testWidgets(
    'Pending decision prevents double approval and Stop restores the draft',
    (tester) async {
      compact(tester);
      final bridge = ToolBridge()..pending = Completer();
      final chat = ready(bridge);
      await tester.pumpWidget(DoloresApp(chat: chat));
      await chat.send();
      await tester.pump(const Duration(milliseconds: 100));
      await tester.pumpAndSettle();
      await tester.tap(find.byKey(const Key('allow-tool')));
      await tester.pump();
      expect(
        tester
            .widget<FilledButton>(find.byKey(const Key('allow-tool')))
            .onPressed,
        isNull,
      );
      expect(
        tester.widget<TextButton>(find.byKey(const Key('deny-tool'))).onPressed,
        isNull,
      );
      await chat.decideTool(true);
      expect(
        bridge.commands.where((c) => c['command'] == 'approveTool').length,
        1,
      );
      await chat.stop();
      await tester.pump(const Duration(milliseconds: 250));
      await tester.pumpAndSettle();
      expect(chat.draft, 'Read readme');
      expect(chat.toolApproval, isNull);
      expect(chat.messages, isEmpty);
      bridge.pending!.complete();
      await tester.pumpAndSettle();
      expect(chat.toolApproval, isNull);
      expect(chat.messages, isEmpty);
      expect(tester.takeException(), isNull);
      await tester.pumpWidget(const SizedBox());
      chat.dispose();
    },
  );
  test('Folder choice is launch-only, cancel preserves state and generation excludes changes', () async {
    final bridge = ToolBridge();
    final chat = ChatController(bridge)
      ..loading = false
      ..draft = 'Keep draft';
    addTearDown(chat.dispose);
    expect(chat.workspaceRoot, isNull);
    await chat.chooseToolFolder(() async => 'C:/chosen/project');
    await chat.chooseToolFolder(() async => null);
    expect(chat.workspaceRoot, 'C:/chosen/project');
    expect(chat.draft, 'Keep draft');
    expect(bridge.commands, isEmpty);
    chat.busy = true;
    chat.disableTools();
    await chat.chooseToolFolder(
      () async => throw StateError('Must not open picker'),
    );
    expect(chat.workspaceRoot, 'C:/chosen/project');
    chat.busy = false;
    chat.disableTools();
    expect(chat.workspaceRoot, isNull);
    expect(ChatController(ToolBridge()).workspaceRoot, isNull);
  });
}
