import 'dart:async';

import 'package:dolores_flutter/chat.dart';
import 'package:dolores_flutter/main.dart';
import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';

import 'tools_test.dart' show ToolBridge;

class WorkspaceBridge extends ToolBridge {
  final roots = <String, Map<String, dynamic>>{};
  final rows = <Map<String, dynamic>>[];
  final recentProjects = <Map<String, dynamic>>[];
  int next = 0;
  String? active;
  bool failCreate = false;
  Completer<void>? creating;
  @override
  Future<dynamic> call(Map<String, dynamic> command) async {
    if (command['command'] == 'createSession') {
      commands.add(command);
      if (creating != null) await creating!.future;
      if (failCreate) throw StateError('Working folder is unavailable.');
      final id = 'session-${++next}';
      final kind = command['kind'];
      final root = kind == 'side' ? null : command['path'] ?? 'C:/managed/$id';
      final workspace = {'kind': kind, 'root': root};
      roots[id] = workspace;
      final row = {
        'id': id,
        'title': 'New chat',
        'updatedAt': next,
        'workspace': workspace,
      };
      rows.insert(0, row);
      if (kind == 'project' && !recentProjects.any((p) => p['root'] == root)) {
        recentProjects.add({'name': 'project', 'root': root});
      }
      return {'session': row, 'workspace': workspace};
    }
    if (command['command'] == 'workspace') {
      commands.add(command);
      return roots[command['session']];
    }
    if (command['command'] == 'bootstrap') {
      final state = await super.call(command);
      return {
        ...(state as Map),
        'sessions': List.of(rows),
        'projects': List.of(recentProjects),
      };
    }
    if (command['command'] == 'start') active = command['session'] as String;
    final result = await super.call(command);
    if (command['command'] == 'poll') {
      for (final event in result as List) {
        if (event['type'] == 'started') event['session'] = active;
      }
    }
    return result;
  }
}

void main() {
  testWidgets(
    'Sidebar groups project chats and Recents, collapses sections and offers temporary or side chats',
    (tester) async {
      tester.view.physicalSize = const Size(1120, 780);
      tester.view.devicePixelRatio = 1;
      addTearDown(tester.view.resetPhysicalSize);
      addTearDown(tester.view.resetDevicePixelRatio);
      for (final dark in [false, true]) {
        final bridge = WorkspaceBridge();
        final chat = ChatController(bridge)
          ..loading = false
          ..configured = true
          ..workspaceKind = 'project'
          ..workspaceRoot = 'C:/chosen/alpha';
        chat.projects = [
          {'root': 'C:/chosen/alpha', 'name': 'Alpha'},
        ];
        chat.sessions = [
          {
            'id': 'p',
            'title': 'Project task',
            'updatedAt': 3,
            'workspace': {'kind': 'project', 'root': 'C:/chosen/alpha'},
          },
          {
            'id': 't',
            'title': 'Temporary task',
            'updatedAt': 2,
            'workspace': {'kind': 'temporary', 'root': 'C:/managed/t'},
          },
          {
            'id': 's',
            'title': 'Side discussion',
            'updatedAt': 1,
            'workspace': {'kind': 'side', 'root': null},
          },
        ];
        await tester.pumpWidget(
          DoloresApp(
            chat: chat,
            themeMode: dark ? ThemeMode.dark : ThemeMode.light,
          ),
        );
        expect(find.text('Projects'), findsOneWidget);
        expect(find.text('Recents'), findsOneWidget);
        for (final id in ['p', 't', 's']) {
          expect(find.byKey(ValueKey('chat-$id')), findsOneWidget);
        }
        expect(find.text('Folder tools'), findsNothing);
        await tester.tap(find.byKey(const Key('section-projects')));
        await tester.pumpAndSettle();
        expect(find.byKey(const ValueKey('chat-p')), findsNothing);
        expect(find.byKey(const ValueKey('chat-t')), findsOneWidget);
        await tester.tap(find.byKey(const Key('section-recents')));
        await tester.pumpAndSettle();
        expect(find.byKey(const ValueKey('chat-t')), findsNothing);
        expect(find.byKey(const ValueKey('chat-s')), findsNothing);
        await tester.tap(find.byKey(const Key('new-chat')));
        await tester.pumpAndSettle();
        expect(chat.workspaceKind, 'temporary');
        expect(chat.workspaceRoot, isNull);
        await tester.tap(find.byKey(const Key('new-chat-options')));
        await tester.pumpAndSettle();
        await tester.tap(find.text('Side chat').last);
        await tester.pumpAndSettle();
        expect(chat.workspaceKind, 'side');
        expect(tester.takeException(), isNull);
        await tester.pumpWidget(const SizedBox());
        chat.dispose();
      }
    },
  );
  testWidgets(
    'Default temporary chat gets a folder and explicit approval; modes fit both compact themes',
    (tester) async {
      tester.view.physicalSize = const Size(620, 700);
      tester.view.devicePixelRatio = 1;
      addTearDown(tester.view.resetPhysicalSize);
      addTearDown(tester.view.resetDevicePixelRatio);
      for (final dark in [false, true]) {
        final bridge = WorkspaceBridge();
        final chat = ChatController(bridge);
        await chat.initialize();
        await tester.pumpWidget(
          DoloresApp(
            chat: chat,
            themeMode: dark ? ThemeMode.dark : ThemeMode.light,
          ),
        );
        expect(chat.workspaceKind, 'temporary');
        expect(
          bridge.commands.where((c) => c['command'] == 'createSession'),
          isEmpty,
        );
        await tester.enterText(
          find.byKey(const Key('composer')),
          'Read readme',
        );
        await tester.pump();
        expect(chat.draft, 'Read readme');
        await tester.tap(find.byKey(const Key('send')));
        await tester.pump(const Duration(milliseconds: 250));
        expect(chat.error, isNull);
        expect(chat.workspaceRoot, 'C:/managed/session-1');
        expect(find.text('Folder: Temporary workspace'), findsOneWidget);
        expect(
          bridge.commands.where((c) => c['command'] == 'approveTool'),
          isEmpty,
        );
        expect(
          bridge.commands
              .singleWhere((c) => c['command'] == 'start')
              .containsKey('workspace'),
          isFalse,
        );
        await tester.tap(find.byKey(const Key('deny-tool')));
        await tester.pump(const Duration(milliseconds: 250));
        await tester.pumpAndSettle();
        expect(chat.error, isNull);
        await tester.tap(find.byKey(const Key('workspace-picker')));
        await tester.pumpAndSettle();
        expect(find.text('Temporary chat'), findsOneWidget);
        expect(find.text('Side chat'), findsOneWidget);
        await tester.tap(find.text('Side chat').last);
        await tester.pumpAndSettle();
        expect(chat.workspaceKind, 'side');
        expect(chat.workspaceRoot, isNull);
        expect(
          find.text('A conversation without file access.'),
          findsOneWidget,
        );
        expect(tester.takeException(), isNull);
        await tester.pumpWidget(const SizedBox());
        chat.dispose();
      }
    },
  );
  test('Project and temporary roots restore per chat and across controller restart', () async {
    final bridge = WorkspaceBridge();
    final chat = ChatController(bridge);
    await chat.initialize();
    await chat.openProject('C:/chosen/project');
    final project = chat.session!;
    chat.draft = 'Keep project draft';
    chat.newChat(kind: 'temporary');
    chat.draft = 'Read readme';
    await chat.send();
    final temporary = chat.session!;
    final temporaryRoot = chat.workspaceRoot;
    await Future<void>.delayed(const Duration(milliseconds: 120));
    await chat.stop();
    await Future<void>.delayed(const Duration(milliseconds: 120));
    await chat.select(project);
    expect(chat.workspaceRoot, 'C:/chosen/project');
    expect(chat.draft, 'Keep project draft');
    chat.newChat();
    expect(chat.workspaceKind, 'project');
    expect(chat.workspaceRoot, 'C:/chosen/project');
    await chat.select(temporary);
    expect(chat.workspaceRoot, temporaryRoot);
    expect(chat.workspaceKind, 'temporary');
    chat.dispose();
    final restarted = ChatController(bridge);
    await restarted.initialize();
    expect(restarted.session, temporary);
    expect(restarted.workspaceRoot, temporaryRoot);
    expect(restarted.projects.single['root'], 'C:/chosen/project');
    restarted.dispose();
  });
  test('Missing project and pending workspace creation preserve state and prevent a tool-free send', () async {
    final bridge = WorkspaceBridge();
    final chat = ChatController(bridge);
    await chat.initialize();
    chat.draft = 'Keep this';
    bridge.failCreate = true;
    await chat.openProject('C:/missing/project');
    expect(chat.draft, 'Keep this');
    expect(chat.session, isNull);
    await chat.send();
    expect(chat.draft, 'Keep this');
    expect(bridge.commands.where((c) => c['command'] == 'start'), isEmpty);
    bridge.failCreate = false;
    bridge.creating = Completer<void>();
    final sending = chat.send();
    expect(chat.changing, isTrue);
    chat.newChat(kind: 'side');
    await chat.openProject('C:/other/project');
    expect(chat.workspaceKind, 'temporary');
    bridge.creating!.complete();
    await sending;
    expect(chat.workspaceKind, 'temporary');
    expect(bridge.commands.where((c) => c['command'] == 'start'), hasLength(1));
    chat.dispose();
  });
}
