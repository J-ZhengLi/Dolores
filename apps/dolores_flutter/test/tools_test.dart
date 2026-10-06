import 'dart:async';
import 'dart:convert';

import 'support/workspaces.dart';

import 'package:dolores_flutter/bridge.dart';
import 'package:dolores_flutter/chat.dart';
import 'package:dolores_flutter/main.dart';
import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';

class ToolBridge implements ChatBridge {
  bool streamText = false;
  bool editConflict = false;
  static const editDiff =
      '--- before\n+++ after\n@@ -1,1 +1,1 @@\n-File text 世界\n+# New text <script>literal</script>\n';
  String tool = 'read_text_file';
  String query = '世界.*';
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
      case 'createSession':
        return createdWorkspace(command);
      case 'workspace':
        return {'kind': 'project', 'root': 'C:/chosen/project'};
      case 'start':
        input = command['input'];
        queue.addAll([
          {'type': 'started', 'id': id, 'session': 'run'},
          {'type': 'modelStep', 'id': id, 'number': 1},
          if (streamText)
            {
              'type': 'modelText',
              'id': id,
              'number': 1,
              'text': 'Let me read 世界.',
            },
          {
            'type': 'toolApproval',
            'id': id,
            'request': {
              'callId': 'file-one',
              'name': tool,
              if (tool == 'run_command')
                'command': {
                  'invocation': {
                    'program': 'node',
                    'args': ['--version', 'a b', ''],
                  },
                  'executable': 'C:/Runtime/node.exe',
                },
              'target':
                  [
                    'read_text_file',
                    'edit_text_file',
                    'create_text_file',
                  ].contains(tool)
                  ? 'readme.txt'
                  : '.',
              if (tool == 'search_text') 'query': query,
              if (['edit_text_file', 'create_text_file'].contains(tool))
                'diff': editDiff,
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
          'name': tool,
          if (tool == 'run_command')
            'command': {
              'program': 'node',
              'args': ['--version', 'a b', ''],
            },
          'target':
              [
                'read_text_file',
                'edit_text_file',
                'create_text_file',
              ].contains(tool)
              ? 'readme.txt'
              : '.',
          if (tool == 'search_text') 'query': query,
          if (['edit_text_file', 'create_text_file'].contains(tool))
            'diff': editDiff,
          'status': allowed
              ? (tool == 'read_text_file'
                    ? 'read'
                    : tool == 'edit_text_file'
                    ? (editConflict ? 'error' : 'edited')
                    : tool == 'create_text_file'
                    ? (editConflict ? 'error' : 'created')
                    : 'completed')
              : 'denied',
          'content': !allowed
              ? 'User denied this read'
              : tool == 'run_command'
              ? jsonEncode({
                  'exitCode': 7,
                  'reason': 'completed',
                  'stdout': '# Output <script>literal</script> 世界',
                  'stderr': 'diagnostic',
                  'truncated': true,
                  'lossyUtf8': false,
                  'outputError': false,
                })
              : tool == 'create_text_file'
              ? (editConflict
                    ? 'Target already exists. No file was created.'
                    : jsonEncode({
                        'applied': true,
                        'created': true,
                        'bytesAfter': 0,
                      }))
              : tool == 'edit_text_file'
              ? (editConflict
                    ? 'File changed since preview. No edit was applied.'
                    : jsonEncode({
                        'applied': true,
                        'bytesBefore': 15,
                        'bytesAfter': 36,
                      }))
              : tool == 'read_text_file'
              ? 'File text 世界'
              : jsonEncode(
                  tool == 'list_folder'
                      ? {
                          'entries': [
                            {'path': 'docs', 'kind': 'folder'},
                          ],
                          'skippedEntries': 2,
                          'truncated': true,
                        }
                      : {
                          'matches': [
                            {
                              'path': 'docs/notes.txt',
                              'line': 2,
                              'text': 'Find 世界.* here',
                            },
                          ],
                          'scannedFiles': 3,
                          'skippedFiles': 1,
                          'skippedEntries': 2,
                          'truncated': true,
                        },
                ),
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
                if (streamText)
                  'steps': [
                    {'number': 1, 'text': 'Let me read 世界.'},
                  ],
              },
            },
          },
        ];
        queue.addAll([
          {'type': 'toolResult', 'id': id, 'record': record},
          {'type': 'modelStep', 'id': id, 'number': 2},
          {
            'type': streamText ? 'modelText' : 'delta',
            'id': id,
            'number': 2,
            'text': 'Final answer',
          },
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
  ..workspaceKind = 'project'
  ..workspaceRoot = 'C:/chosen/project';
void compact(WidgetTester tester) {
  tester.view.physicalSize = const Size(390, 740);
  tester.view.devicePixelRatio = 1;
  addTearDown(tester.view.resetPhysicalSize);
  addTearDown(tester.view.resetDevicePixelRatio);
}

void _editTests() {
  testWidgets(
    'Edit approval shows a literal colored diff in both compact themes and applies only on explicit decision',
    (tester) async {
      compact(tester);
      for (final dark in [false, true]) {
        final bridge = ToolBridge()..tool = 'edit_text_file';
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
        expect(find.text('Apply this file change?'), findsOneWidget);
        expect(
          find.byKey(const Key('edit-diff-approval-file-one')),
          findsOneWidget,
        );
        expect(find.text('Apply once'), findsOneWidget);
        expect(
          bridge.commands.where((c) => c['command'] == 'approveTool'),
          isEmpty,
        );
        final rich = tester
            .widgetList<SelectableText>(find.byType(SelectableText))
            .where((w) => w.textSpan?.toPlainText() == ToolBridge.editDiff)
            .single;
        expect(
          rich.textSpan!.children!.whereType<TextSpan>().any(
            (s) => s.text!.contains('+# New text <script>literal</script>'),
          ),
          isTrue,
        );
        await tester.tap(find.byKey(const Key('allow-tool')));
        await tester.pump(const Duration(milliseconds: 250));
        await tester.pumpAndSettle();
        final record = chat.messages.last['metadata']['agent']['tools'].single;
        expect(record['status'], 'edited');
        expect(record['diff'], ToolBridge.editDiff);
        await tester.ensureVisible(find.text('readme.txt'));
        await tester.tap(find.text('readme.txt'));
        await tester.pumpAndSettle();
        expect(
          find.text('Applied one file change · 15 → 36 bytes'),
          findsOneWidget,
        );
        expect(
          find.byKey(const Key('edit-diff-record-file-one')),
          findsOneWidget,
        );
        expect(tester.takeException(), isNull);
        await tester.pumpWidget(const SizedBox());
        chat.dispose();
      }
    },
  );
  testWidgets(
    'Changed-file results retain the reviewed diff and Stop at an edit approval never grants permission',
    (tester) async {
      final bridge = ToolBridge()
        ..tool = 'edit_text_file'
        ..editConflict = true;
      final chat = ready(bridge);
      await chat.send();
      await tester.pump(const Duration(milliseconds: 100));
      await chat.decideTool(true);
      await tester.pump(const Duration(milliseconds: 250));
      expect(
        chat.messages.last['metadata']['agent']['tools'].single['content'],
        'File changed since preview. No edit was applied.',
      );
      chat.draft = 'Another change';
      await chat.send();
      await tester.pump(const Duration(milliseconds: 100));
      final before = bridge.commands
          .where((c) => c['command'] == 'approveTool')
          .length;
      await chat.stop();
      await tester.pump(const Duration(milliseconds: 250));
      expect(
        bridge.commands.where((c) => c['command'] == 'approveTool').length,
        before,
      );
      expect(chat.messages.length, 2);
      expect(chat.draft, 'Another change');
      expect(chat.toolApproval, isNull);
      chat.dispose();
    },
  );
}

void main() {
  testWidgets(
    'Reasoning-only activity is visible, stale steps are ignored and Stop restores the draft',
    (tester) async {
      final bridge = ToolBridge();
      final chat = ready(bridge);
      await tester.pumpWidget(DoloresApp(chat: chat));
      await chat.send();
      final id = bridge.commands.lastWhere(
        (c) => c['command'] == 'start',
      )['id'];
      bridge.queue.removeWhere((e) => e['type'] == 'toolApproval');
      bridge.queue.add({
        'type': 'modelActivity',
        'id': id,
        'number': 1,
        'phase': 'reasoning',
        'elapsedSeconds': 7,
        'timeoutSeconds': 180,
      });
      await tester.pump(const Duration(milliseconds: 100));
      expect(find.text('Thinking · 7s'), findsOneWidget);
      expect(chat.partial, isEmpty);
      expect(chat.toolApproval, isNull);
      bridge.queue.addAll([
        {
          'type': 'modelThinking',
          'id': id,
          'number': 1,
          'text': 'Check collision and keyboard input.',
        },
        {
          'type': 'modelText',
          'id': id,
          'number': 1,
          'text': 'Preparing an HTML file.',
        },
        {
          'type': 'modelActivity',
          'id': id,
          'number': 1,
          'phase': 'toolArguments',
          'elapsedSeconds': 181,
          'timeoutSeconds': 180,
        },
      ]);
      await tester.pump(const Duration(milliseconds: 100));
      expect(find.text('Preparing tool call · 181s'), findsOneWidget);
      expect(find.text('Preparing an HTML file.'), findsOneWidget);
      await tester.tap(find.text('Thinking'));
      await tester.pump(const Duration(milliseconds: 250));
      expect(find.text('Check collision and keyboard input.'), findsOneWidget);
      bridge.queue.add({
        'type': 'modelActivity',
        'id': id,
        'number': 0,
        'phase': 'toolArguments',
        'elapsedSeconds': 99,
        'timeoutSeconds': 180,
      });
      await tester.pump(const Duration(milliseconds: 100));
      expect(find.text('Preparing tool call · 181s'), findsOneWidget);
      await chat.stop();
      await tester.pump(const Duration(milliseconds: 250));
      expect(chat.busy, isFalse);
      expect(chat.draft, 'Read readme');
      expect(find.text('Thinking · 7s'), findsNothing);
      expect(
        bridge.commands.where((c) => c['command'] == 'approveTool'),
        isEmpty,
      );
      await tester.pumpWidget(const SizedBox());
      chat.dispose();
    },
  );
  _editTests();
  for (final dark in [false, true]) {
    testWidgets(
      'Command review in compact ${dark ? 'dark' : 'light'} shows exact arguments, permissions and explicit Run once',
      (tester) async {
        compact(tester);
        final bridge = ToolBridge()..tool = 'run_command';
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
        expect(find.text('Run this command?'), findsOneWidget);
        expect(find.text('Run once'), findsOneWidget);
        expect(find.text('C:/Runtime/node.exe'), findsOneWidget);
        expect(find.text('C:/chosen/project'), findsOneWidget);
        expect(find.text('1. "--version"\n2. "a b"\n3. ""'), findsOneWidget);
        await tester.ensureVisible(find.text('Operation details'));
        await tester.tap(find.text('Operation details'));
        await tester.pumpAndSettle();
        expect(find.textContaining('outside this folder'), findsOneWidget);
        expect(
          bridge.commands.where((c) => c['command'] == 'approveTool'),
          isEmpty,
        );
        await tester.tap(find.text('Deny'));
        await tester.pump(const Duration(milliseconds: 250));
        await tester.pumpAndSettle();
        expect(
          chat.messages.last['metadata']['agent']['tools'].single['status'],
          'denied',
        );
        chat.draft = 'Run again';
        await chat.send();
        await tester.pump(const Duration(milliseconds: 100));
        await tester.pumpAndSettle();
        await tester.tap(find.text('Run once'));
        await tester.pump(const Duration(milliseconds: 250));
        await tester.pumpAndSettle();
        await tester.ensureVisible(find.text('.').last);
        await tester.tap(find.text('.').last);
        await tester.pumpAndSettle();
        expect(
          find.textContaining('Exited 7 · Output shortened'),
          findsOneWidget,
        );
        expect(
          find.textContaining('# Output <script>literal</script> 世界'),
          findsOneWidget,
        );
        expect(tester.takeException(), isNull);
        await tester.pumpWidget(const SizedBox());
        chat.dispose();
      },
    );
  }
  for (final dark in [false, true]) {
    testWidgets(
      'Creation in compact ${dark ? 'dark' : 'light'} theme requires an explicit decision and shows its result',
      (tester) async {
        compact(tester);
        final bridge = ToolBridge()..tool = 'create_text_file';
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
        expect(find.text('Create this file?'), findsOneWidget);
        expect(find.text('Create once'), findsOneWidget);
        expect(
          find.byKey(const Key('edit-diff-approval-file-one')),
          findsOneWidget,
        );
        expect(
          bridge.commands.where((c) => c['command'] == 'approveTool'),
          isEmpty,
        );
        await tester.tap(find.text('Deny'));
        await tester.pump(const Duration(milliseconds: 250));
        await tester.pumpAndSettle();
        expect(
          chat.messages.last['metadata']['agent']['tools'].single['status'],
          'denied',
        );
        chat.draft = 'Create again';
        await chat.send();
        await tester.pump(const Duration(milliseconds: 100));
        await tester.pumpAndSettle();
        await tester.tap(find.text('Create once'));
        await tester.pump(const Duration(milliseconds: 250));
        await tester.pumpAndSettle();
        expect(
          chat.messages.last['metadata']['agent']['tools'].single['status'],
          'created',
        );
        final cards = find.text('readme.txt');
        await tester.ensureVisible(cards.last);
        await tester.tap(cards.last);
        await tester.pumpAndSettle();
        expect(find.text('Created one file · 0 bytes'), findsOneWidget);
        expect(tester.takeException(), isNull);
        await tester.pumpWidget(const SizedBox());
        chat.dispose();
      },
    );
  }
  testWidgets(
    'Streamed public text stays visible at approval and saved progress stays separate from the final answer',
    (tester) async {
      compact(tester);
      for (final dark in [false, true]) {
        final bridge = ToolBridge()..streamText = true;
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
        expect(chat.partial, 'Let me read 世界.');
        expect(chat.modelStep, 1);
        expect(find.text('Allow a file read?'), findsOneWidget);
        expect(
          bridge.commands.where((c) => c['command'] == 'approveTool'),
          isEmpty,
        );
        await tester.tap(find.byKey(const Key('allow-tool')));
        await tester.pump(const Duration(milliseconds: 250));
        await tester.pumpAndSettle();
        expect(chat.busy, isFalse);
        expect(chat.messages.last['content'], 'Final answer');
        expect(chat.modelTexts, isEmpty);
        expect(
          chat.messages.last['metadata']['agent']['steps'].single['text'],
          'Let me read 世界.',
        );
        await tester.ensureVisible(find.text('Agent progress'));
        await tester.tap(find.text('Agent progress'));
        await tester.pumpAndSettle();
        expect(find.text('Let me read 世界.'), findsOneWidget);
        expect(tester.takeException(), isNull);
        await tester.pumpWidget(const SizedBox());
        chat.dispose();
      }
    },
  );
  testWidgets(
    'New steps replace streamed text and stale step deltas cannot alter the current answer',
    (tester) async {
      final bridge = ToolBridge()..streamText = true;
      final chat = ready(bridge);
      await chat.send();
      await tester.pump(const Duration(milliseconds: 100));
      bridge.queue.addAll([
        {'type': 'modelStep', 'id': 1, 'number': 2},
        {'type': 'modelText', 'id': 1, 'number': 1, 'text': 'STALE'},
        {'type': 'modelText', 'id': 1, 'number': 2, 'text': 'Final '},
        {'type': 'modelText', 'id': 1, 'number': 2, 'text': 'answer'},
      ]);
      await tester.pump(const Duration(milliseconds: 250));
      expect(chat.partial, 'Final answer');
      expect(chat.modelTexts.single['text'], 'Let me read 世界.');
      await chat.stop();
      await tester.pump(const Duration(milliseconds: 250));
      expect(chat.messages, isEmpty);
      expect(chat.partial, isEmpty);
      expect(chat.draft, 'Read readme');
      chat.newChat(kind: 'side');
      expect(chat.modelTexts, isEmpty);
      chat.dispose();
    },
  );
  testWidgets(
    'Discovery approvals disclose scan/query and saved cards show partial coverage in both compact themes',
    (tester) async {
      compact(tester);
      for (final dark in [false, true]) {
        for (final tool in ['list_folder', 'search_text']) {
          for (final allow in [false, true]) {
            final bridge = ToolBridge()..tool = tool;
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
            expect(
              find.text(
                tool == 'list_folder'
                    ? 'Allow a folder listing?'
                    : 'Allow a text search?',
              ),
              findsOneWidget,
            );
            expect(
              bridge.commands.where((c) => c['command'] == 'approveTool'),
              isEmpty,
            );
            if (tool == 'search_text') {
              await tester.ensureVisible(find.text('Operation details'));
              await tester.tap(find.text('Operation details'));
              await tester.pumpAndSettle();
              expect(find.text('世界.*'), findsOneWidget);
              expect(
                find.textContaining('Scan up to 64 text files'),
                findsOneWidget,
              );
            }
            await tester.tap(
              find.byKey(Key(allow ? 'allow-tool' : 'deny-tool')),
            );
            await tester.pump(const Duration(milliseconds: 250));
            await tester.pumpAndSettle();
            expect(chat.busy, isFalse);
            expect(
              chat.messages.last['metadata']['agent']['tools'].single['name'],
              tool,
            );
            if (tool == 'search_text') {
              expect(
                chat
                    .messages
                    .last['metadata']['agent']['tools']
                    .single['query'],
                '世界.*',
              );
            }
            final label =
                '${tool == 'list_folder' ? 'Folder listing' : 'Text search'} · ${allow ? 'completed' : 'denied'}';
            await tester.ensureVisible(find.text(label));
            await tester.tap(find.text(label));
            await tester.pumpAndSettle();
            if (allow) {
              expect(find.textContaining('Partial results'), findsOneWidget);
              expect(
                find.textContaining(
                  tool == 'list_folder' ? 'docs/' : 'docs/notes.txt:2',
                ),
                findsOneWidget,
              );
            } else {
              expect(find.text('User denied this read'), findsOneWidget);
            }
            expect(tester.takeException(), isNull);
            await tester.pumpWidget(const SizedBox());
            chat.dispose();
          }
        }
      }
    },
  );
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
          expect(bridge.commands.first['path'], 'C:/chosen/project');
          expect(bridge.commands.first['command'], 'createSession');
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
  test('Opening a project creates a bound chat, cancellation and generation exclude changes', () async {
    final bridge = ToolBridge();
    final chat = ChatController(bridge)
      ..loading = false
      ..draft = 'Keep draft';
    addTearDown(chat.dispose);
    expect(chat.workspaceRoot, isNull);
    await chat.chooseToolFolder(() async => 'C:/chosen/project');
    await chat.chooseToolFolder(() async => null);
    expect(chat.workspaceRoot, 'C:/chosen/project');
    expect(chat.draft, '');
    expect(chat.workspaceKind, 'project');
    expect(
      bridge.commands.where((c) => c['command'] == 'createSession'),
      hasLength(1),
    );
    chat.busy = true;
    chat.disableTools();
    await chat.chooseToolFolder(
      () async => throw StateError('Must not open picker'),
    );
    expect(chat.workspaceRoot, 'C:/chosen/project');
    chat.busy = false;
    chat.disableTools();
    expect(chat.workspaceRoot, isNull);
    expect(chat.workspaceKind, 'side');
    expect(ChatController(ToolBridge()).workspaceRoot, isNull);
  });
}
