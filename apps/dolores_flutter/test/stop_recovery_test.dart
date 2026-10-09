import 'dart:io';
import 'dart:ui' as ui;

import 'package:dolores_flutter/bridge.dart';
import 'package:dolores_flutter/chat.dart';
import 'package:dolores_flutter/main.dart';
import 'package:flutter/material.dart';
import 'package:flutter/rendering.dart';
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
      case 'scheduledStart':
        starts++;
        return {
          'session': 'audit',
          'model': 'fixture',
          'workspace': {'kind': 'project', 'root': 'C:/public'},
        };
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
          'recovery': {
            'kind': 'stopped',
            'retryable': false,
            'guidance': 'Your draft is restored. Send it when you are ready.',
          },
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
  const renderDirectory = String.fromEnvironment('AUDIT_RENDER_DIR');
  setUpAll(() async {
    if (renderDirectory.isEmpty) return;
    final fonts = '${Platform.environment['WINDIR'] ?? 'C:/Windows'}/Fonts';
    for (final family in ['Ahem', 'Segoe UI', 'Georgia']) {
      await ui.loadFontFromList(
        await File('$fonts/${family == 'Georgia' ? 'georgia' : 'segoeui'}.ttf')
            .readAsBytes(),
        fontFamily: family,
      );
    }
    await ui.loadFontFromList(
      await File(
        '../../output/toolchains/flutter/bin/cache/artifacts/material_fonts/MaterialIcons-Regular.otf',
      ).readAsBytes(),
      fontFamily: 'MaterialIcons',
    );
  });
  for (final width in [390.0, 1127.0]) {
    for (final dark in [false, true]) {
      testWidgets(
        'Stopped scheduled run points to Scheduled without inventing a draft width=$width dark=$dark',
        (tester) async {
          tester.view.physicalSize = Size(width, 813);
          tester.view.devicePixelRatio = 1;
          addTearDown(tester.view.resetPhysicalSize);
          addTearDown(tester.view.resetDevicePixelRatio);
          final bridge = InterruptedBridge(true);
          final chat = ChatController(bridge)..loading = false;
          final renderKey = GlobalKey();
          await tester.pumpWidget(
            RepaintBoundary(
              key: renderKey,
              child: DoloresApp(
                chat: chat,
                themeMode: dark ? ThemeMode.dark : ThemeMode.light,
              ),
            ),
          );
          await chat.send(
            scheduledOccurrence: 'due-fixture',
            taskInput: 'Read README.md',
          );
          await chat.stop();
          await tester.pump(const Duration(milliseconds: 100));
          await tester.pumpAndSettle();
          expect(chat.busy, false);
          expect(chat.draft, isEmpty);
          expect(chat.error, 'Scheduled task stopped.');
          expect(
            chat.activeRecovery?['guidance'],
            'Open Scheduled to run it again.',
          );
          expect(
            find.text('Your draft is restored. Send it when you are ready.'),
            findsNothing,
          );
          expect(find.text('Open Scheduled to run it again.'), findsOneWidget);
          expect(bridge.starts, 1);
          expect(
            bridge.commands.where((c) => c['command'] == 'start'),
            isEmpty,
          );
          expect(tester.takeException(), isNull);
          if (renderDirectory.isNotEmpty) {
            final boundary =
                renderKey.currentContext!.findRenderObject()
                    as RenderRepaintBoundary;
            await tester.runAsync(() async {
              final rendered = await boundary.toImage(pixelRatio: 1);
              try {
                final bytes = await rendered.toByteData(
                  format: ui.ImageByteFormat.png,
                );
                await Directory(renderDirectory).create(recursive: true);
                await File(
                  '$renderDirectory/scheduled-stop-${width.toInt()}-${dark ? 'dark' : 'light'}.png',
                ).writeAsBytes(bytes!.buffer.asUint8List());
              } finally {
                rendered.dispose();
              }
            });
          }
          // A later ordinary send on the same owner must regain draft recovery.
          chat.configured = true;
          chat.draft = 'New ordinary request';
          bridge.starts = 0;
          await chat.send();
          await tester.pump(const Duration(milliseconds: 100));
          await chat.stop();
          await tester.pump(const Duration(milliseconds: 100));
          await tester.pumpAndSettle();
          expect(chat.error, 'Response stopped. Your message was not saved.');
          expect(chat.draft, 'New ordinary request');
          expect(
            chat.activeRecovery?['guidance'],
            contains('Your draft is restored'),
          );
          expect(
            bridge.commands
                .where((c) => c['command'] == 'start')
                .single['input'],
            'New ordinary request',
          );
          await tester.pumpWidget(const SizedBox());
          chat.dispose();
        },
      );
    }
  }
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
        expect(
          chat.activeRecovery?['guidance'],
          contains('Your draft is restored'),
        );
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
