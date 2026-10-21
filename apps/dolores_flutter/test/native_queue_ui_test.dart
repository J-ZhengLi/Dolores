import 'dart:async';
import 'dart:convert';
import 'dart:io';

import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:dolores_flutter/bridge.dart';
import 'package:dolores_flutter/chat.dart';
import 'package:dolores_flutter/main.dart';

void main() {
  testWidgets(
    'native queue reorder and steering preserve completed effects and a newer draft',
    (tester) async {
      final directory = Directory(Platform.environment['DOLORES_DATA_DIR']!);
      final output = Directory('../../output').absolute
          .resolveSymbolicLinksSync();
      expect(
        directory.parent.resolveSymbolicLinksSync(),
        startsWith('$output${Platform.pathSeparator}'),
      );
      final project = Directory('${directory.parent.path}/queue-project')
        ..createSync();
      final bridge = (await tester.runAsync(() async {
        final bridge = NativeBridge();
        await bridge.open(
          testLibrary: Platform.environment['DOLORES_AUDIT_LIBRARY'],
        );
        return bridge;
      }))!;
      Future<dynamic> call(
        String command, [
        Map<String, dynamic> fields = const {},
      ]) async {
        final result = (await tester.runAsync(() async {
          try {
            return {
              'value': await bridge.call({'command': command, ...fields}),
            };
          } catch (e) {
            return {'error': e.toString()};
          }
        }))!;
        if (result.containsKey('error')) throw result['error']!;
        return result['value'];
      }

      Future<void> waitUntil(bool Function() ready) async {
        final end = DateTime.now().add(const Duration(seconds: 20));
        do {
          await tester.runAsync(
            () => Future<void>.delayed(const Duration(milliseconds: 25)),
          );
          await tester.pump(const Duration(milliseconds: 100));
          if (ready()) return;
        } while (DateTime.now().isBefore(end));
        fail('Queue fixture deadline exceeded');
      }

      final release = Completer<void>();
      var requests = 0;
      final bodies = <Map<String, dynamic>>[];
      final server = (await tester.runAsync(() async {
        final server = await HttpServer.bind(InternetAddress.loopbackIPv4, 0);
        server.listen((request) async {
          final body = (jsonDecode(
            await utf8.decoder.bind(request).join(),
          ) as Map).cast<String, dynamic>();
          bodies.add(body);
          final number = ++requests;
          if (number == 2) await release.future;
          final tool = number == 1 || number == 2;
          final delta = tool
              ? {
                  'tool_calls': [
                    {
                      'index': 0,
                      'id': 'effect-$number',
                      'type': 'function',
                      'function': {
                        'name': 'create_text_file',
                        'arguments': jsonEncode({
                          'path': number == 1 ? 'kept.txt' : 'superseded.txt',
                          'content': 'PUBLIC-EFFECT\n世界\n',
                        }),
                      },
                    },
                  ],
                }
              : {'content': 'Verified response $number'};
          request.response.headers.contentType = ContentType(
            'text',
            'event-stream',
            charset: 'utf-8',
          );
          for (final event in [
            {
              'choices': [
                {'index': 0, 'delta': delta, 'finish_reason': null},
              ],
            },
            {
              'choices': [
                {
                  'index': 0,
                  'delta': {},
                  'finish_reason': tool ? 'tool_calls' : 'stop',
                },
              ],
            },
          ]) {
            request.response.write('data: ${jsonEncode(event)}\n\n');
          }
          request.response.write('data: [DONE]\n\n');
          await request.response.close();
        });
        return server;
      }))!;
      ChatController? ownedChat;
      addTearDown(() async {
        if (!release.isCompleted) release.complete();
        ownedChat?.dispose();
        await server.close(force: true);
        await bridge.close();
      });
      await call('configure', {
        'preferences': {
          'baseUrl': 'http://127.0.0.1:${server.port}/v1',
          'model': 'fixture',
        },
        'apiKey': '',
        'rememberConnection': true,
      });
      var session = (await call('createSession', {
        'kind': 'project',
        'path': project.path,
      }))['session']['id'];
      final memories = await call('memories', {'session': session});
      if (memories['automaticPolicy']['enabled'] == true) {
        await call('setAutomaticMemory', {
          'enabled': false,
          'revision': memories['automaticPolicy']['revision'],
        });
      }
      final reviewed = await call('reviewSkillText', {
        'session': session,
        'scope': 'project',
        'name': 'precise-format',
        'text': '---\nname: precise-format\ndescription: Structure exports\n---\nEXACT_PUBLIC_SELECTED_SKILL_V1\n',
      });
      await call('activateSkill', {
        'session': session,
        'token': reviewed['token'],
      });
      final chat = ownedChat = ChatController(bridge, ownsBridge: false)
        ..loading = false;
      await tester.runAsync(chat.refresh);
      await tester.runAsync(() => chat.select(session));
      tester.view.physicalSize = const Size(900, 1000);
      tester.view.devicePixelRatio = 1;
      addTearDown(tester.view.resetPhysicalSize);
      addTearDown(tester.view.resetDevicePixelRatio);
      await tester.pumpWidget(DoloresApp(chat: chat));
      await tester.pumpAndSettle();
      final setupSession = session;
      chat.newChat(kind: 'project');
      await tester.pump();
      await waitUntil(
        () =>
            chat.session != null &&
            chat.session != setupSession &&
            !chat.changing &&
            chat.commandCatalogue['skills'] is List,
      );
      session = chat.session!;
      await tester.enterText(
        find.byKey(const Key('composer-field-0')),
        '/precise',
      );
      await tester.pump();
      await waitUntil(
        () =>
            find
                .byKey(const Key('command-skill-precise-format'))
                .evaluate()
                .isNotEmpty &&
            find
                .descendant(
                  of: find.byKey(const Key('command-picker')),
                  matching: find.byType(LinearProgressIndicator),
                )
                .evaluate()
                .isEmpty,
      );
      await tester.sendKeyEvent(LogicalKeyboardKey.enter);
      await waitUntil(
        () => chat.selectedSkills.length == 1 && chat.draft.isEmpty,
      );
      final preview = await tester.runAsync(chat.previewContext);
      expect(
        jsonEncode(preview!['messages']),
        contains('EXACT_PUBLIC_SELECTED_SKILL_V1'),
      );
      expect(preview['skills'][0]['version'], 1);
      await tester.enterText(
        find.byKey(const Key('composer-field-0')),
        'Original task: create kept.txt, then summarize.',
      );
      await tester.pump();
      await tester.tap(find.byKey(const Key('send')));
      await waitUntil(
        () =>
            chat.toolApproval != null &&
            find.byKey(const Key('allow-tool')).evaluate().isNotEmpty,
      );
      await tester.ensureVisible(find.byKey(const Key('allow-tool')));
      await tester.tap(find.byKey(const Key('allow-tool')));
      await waitUntil(() => requests == 2 && chat.toolRecords.isNotEmpty);
      expect(
        File('${project.path}/kept.txt').readAsStringSync(),
        'PUBLIC-EFFECT\n世界\n',
      );
      Future<void> submit(String text, String key) async {
        await tester.enterText(find.byKey(const Key('composer-field-0')), text);
        await tester.pump();
        await tester.ensureVisible(find.byKey(Key(key)));
        await tester.tap(find.byKey(Key(key)));
        await waitUntil(() => !chat.inputSaving && chat.draft.isEmpty);
        expect(chat.error, isNull);
      }

      await submit('Follow-up first', 'queue-instruction');
      await submit('Follow-up second', 'queue-instruction');
      final pending = chat.inputEntries
          .where((e) => e['state'] == 'pending')
          .toList();
      expect(pending, hasLength(2));
      await tester.runAsync(
        () => chat.changeInput({
          'action': 'reorder',
          'revision': chat.taskInputs['revision'],
          'ids': [pending[1]['id'], pending[0]['id']],
        }),
      );
      final first = chat.inputEntries.firstWhere(
        (e) => e['id'] == pending[0]['id'],
      );
      await tester.runAsync(
        () => chat.changeInput({
          'action': 'edit',
          'id': first['id'],
          'revision': first['revision'],
          'text': 'Follow-up first, edited',
        }),
      );
      await submit(
        'Keep original task; do not create superseded.txt',
        'steer-instruction',
      );
      await tester.enterText(
        find.byKey(const Key('composer-field-0')),
        'Newer private draft stays unsent',
      );
      await tester.pump();
      release.complete();
      await waitUntil(() => requests == 5 && !chat.busy && !chat.changing);
      expect(chat.error, isNull);
      expect(chat.draft, 'Newer private draft stays unsent');
      expect(File('${project.path}/superseded.txt').existsSync(), isFalse);
      expect(
        File('${project.path}/kept.txt').readAsStringSync(),
        'PUBLIC-EFFECT\n世界\n',
      );
      expect(
        jsonEncode(bodies[2]['messages']),
        contains('do not create superseded.txt'),
      );
      expect(
        jsonEncode(bodies[2]['messages']),
        contains('Original task: create kept.txt'),
      );
      expect(
        (bodies[3]['messages'] as List).last['content'],
        'Follow-up second',
      );
      expect(
        (bodies[4]['messages'] as List).last['content'],
        'Follow-up first, edited',
      );
      expect(
        jsonEncode(bodies),
        isNot(contains('Newer private draft stays unsent')),
      );
      for (final body in bodies) {
        expect(
          jsonEncode((body['messages'] as List).first),
          contains('EXACT_PUBLIC_SELECTED_SKILL_V1'),
        );
      }
      final saved = await call('taskInputs', {'session': session});
      expect(
        (saved['entries'] as List).every((e) => e['state'] == 'applied'),
        isTrue,
      );
      expect(
        await call('savedDraft', {'session': session}),
        'Newer private draft stays unsent',
      );
      File('${directory.parent.path}/queue-receipt.json').writeAsStringSync(
        const JsonEncoder.withIndent('  ').convert({
          'mode': 'Rendered controls and normal bridge, public loopback fixture; no native pointer or real model claim',
          'providerRequests': requests,
          'followupsInEditedOrder': true,
          'originalGoalRetained': true,
          'completedEffectRetained': true,
          'supersededEffectPrevented': true,
          'newerDraftUnsentAndRetained': true,
          'inputs': saved,
          'selectedSkillInPreviewNormalQueueAndSteer': true,
          'selectedSkill': chat.selectedSkills.single,
        }),
      );
      expect(tester.takeException(), isNull);
      await tester.pumpWidget(const SizedBox());
    },
    skip:
        !Platform.isWindows ||
        Platform.environment['DOLORES_AUDIT_LIBRARY'] == null,
  );
}
