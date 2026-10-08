import 'package:dolores_flutter/bridge.dart';
import 'package:dolores_flutter/chat.dart';
import 'package:dolores_flutter/main.dart';
import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';

class InterruptedBridge implements ChatBridge {
  final bool agent;
  InterruptedBridge(this.agent);
  final commands = <Map<String, dynamic>>[];
  final queue = <Map<String, dynamic>>[];
  List<Map<String, dynamic>> messages = [];
  int starts = 0;
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
        starts++;
        if (starts == 1) {
          queue.add({'type': 'started', 'id': id, 'session': 'audit'});
          if (agent) {
            for (var step = 1; step <= 4; step++) {
              queue.addAll([
                {'type': 'modelStep', 'id': id, 'number': step},
                {
                  'type': 'modelText',
                  'id': id,
                  'number': step,
                  'text': step == 4
                      ? 'Useful unfinished reply 世界'
                      : 'Earlier step $step',
                },
              ]);
            }
          } else {
            queue.add({
              'type': 'delta',
              'id': id,
              'text': 'Useful unfinished reply 世界',
            });
          }
        } else {
          messages = [
            {'id': 1, 'role': 'user', 'content': command['input']},
            {'id': 2, 'role': 'assistant', 'content': 'Ready.'},
          ];
          queue.add({'type': 'done', 'id': id});
        }
      case 'poll':
        final result = List.of(queue);
        queue.clear();
        return result;
      case 'cancel':
        queue.add({
          'type': 'done',
          'id': id,
          'error': 'Response stopped. Your message was not saved.',
        });
      case 'bootstrap':
        return {
          'sessions': <Map<String, dynamic>>[],
          'preferences': {'baseUrl': 'http://localhost/v1', 'model': 'fixture'},
          'configured': true,
        };
      case 'messagesPage':
        return {'items': messages, 'hasOlder': false, 'hasNewer': false};
    }
    return null;
  }
}

void main() {
  for (final agent in [false, true]) {
    testWidgets(
      'Stop retains unsaved ${agent ? 'agent' : 'side-chat'} output and a new request stays independent',
      (tester) async {
        tester.view.physicalSize = const Size(390, 740);
        tester.view.devicePixelRatio = 1;
        addTearDown(tester.view.resetPhysicalSize);
        addTearDown(tester.view.resetDevicePixelRatio);
        final bridge = InterruptedBridge(agent);
        final chat = ChatController(bridge)
          ..loading = false
          ..configured = true
          ..session = 'audit'
          ..workspaceKind = agent ? 'project' : 'side'
          ..workspaceRoot = agent ? 'C:/public' : null
          ..draft = 'Original request';
        await tester.pumpWidget(
          DoloresApp(
            chat: chat,
            themeMode: agent ? ThemeMode.dark : ThemeMode.light,
          ),
        );
        await chat.send();
        await tester.pump(const Duration(milliseconds: 100));
        expect(chat.partial, 'Useful unfinished reply 世界');
        await chat.stop();
        await tester.pump(const Duration(milliseconds: 100));
        await tester.pumpAndSettle();
        expect(chat.busy, false);
        expect(chat.draft, 'Original request');
        expect(chat.messages, isEmpty);
        expect(chat.modelTexts.last['text'], 'Useful unfinished reply 世界');
        expect(chat.modelTexts.length, lessThanOrEqualTo(3));
        await tester.ensureVisible(find.text('Agent progress · Not saved'));
        await tester.tap(find.text('Agent progress · Not saved'));
        await tester.pumpAndSettle();
        expect(
          find.text(agent ? 'Step 4 · Partial' : 'Partial response'),
          findsOneWidget,
        );
        expect(find.text('Useful unfinished reply 世界'), findsOneWidget);
        expect(tester.takeException(), isNull);
        chat.draft = 'New request';
        await chat.send();
        expect(chat.modelTexts, isEmpty);
        await tester.pump(const Duration(milliseconds: 100));
        await tester.pumpAndSettle();
        expect(chat.messages.last['content'], 'Ready.');
        expect(
          bridge.commands.where((c) => c['command'] == 'start').last['input'],
          'New request',
        );
        expect(bridge.commands.where((c) => c['command'] == 'start').length, 2);
        await tester.pumpWidget(const SizedBox());
        chat.dispose();
      },
    );
  }
}
