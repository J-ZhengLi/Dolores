import 'dart:convert';
import 'dart:io';

import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:dolores_flutter/bridge.dart';
import 'package:dolores_flutter/chat.dart';
import 'package:dolores_flutter/main.dart';
import 'package:dolores_flutter/goal_panel.dart';

void main() {
  testWidgets(
    'native goal continues through two segments and completes only with artifact evidence',
    (tester) async {
      final directory = Directory(Platform.environment['DOLORES_DATA_DIR']!);
      final output = Directory('../../output').absolute
          .resolveSymbolicLinksSync();
      expect(
        directory.parent.resolveSymbolicLinksSync(),
        startsWith('$output${Platform.pathSeparator}'),
      );
      final project = Directory('${directory.parent.path}/goal-project')
        ..createSync();
      final bridge = (await tester.runAsync(() async {
        final b = NativeBridge();
        await b.open(
          testLibrary: Platform.environment['DOLORES_AUDIT_LIBRARY'],
        );
        return b;
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
            return {'error': '$e'};
          }
        }))!;
        if (result.containsKey('error')) throw result['error']!;
        return result['value'];
      }

      var requests = 0, creates = 0;
      final requestsBody = <Map>[];
      final server = (await tester.runAsync(() async {
        final server = await HttpServer.bind(InternetAddress.loopbackIPv4, 0);
        server.listen((request) async {
          final body =
              jsonDecode(await utf8.decoder.bind(request).join()) as Map;
          requestsBody.add(body);
          final n = ++requests;
          String? tool;
          Map<String, dynamic> args = {};
          String text = '';
          switch (n) {
            case 1:
              tool = 'create_text_file';
              args = {
                'path': 'goal-report.txt',
                'content': 'PUBLIC_GOAL_RESULT\n世界\n',
              };
              creates++;
            case 2:
              text = 'Artifact created. Verification remains for the next segment.';
            case 3:
              tool = 'read_text_file';
              args = {'path': 'goal-report.txt'};
            case 4:
              tool = 'inspect_goal';
            case 5:
              tool = 'goal_progress';
              final receipt = jsonDecode(
                (body['messages'] as List).last['content'] as String,
              ) as Map;
              final goal = receipt['goal'] as Map;
              final evidence = (receipt['receipts'] as List).lastWhere(
                (e) => e['name'] == 'read_text_file',
              ) as Map;
              args = {
                'revision': goal['progressRevision'],
                'steps': [
                  (goal['steps'] as List).first as Map<String, dynamic>
                    ..addAll({
                      'done': true,
                      'evidence': [
                        {
                          'run': evidence['run'],
                          'sequence': evidence['sequence'],
                        },
                      ],
                    }),
                ],
                'complete': true,
                'blocked': false,
                'note': 'Read-back verified the public artifact.',
              };
            default:
              text = 'Verified the report content. Goal finished with retained evidence.';
          }
          final delta = tool == null
              ? {'content': text}
              : {
                  'tool_calls': [
                    {
                      'index': 0,
                      'id': 'goal-$n',
                      'type': 'function',
                      'function': {'name': tool, 'arguments': jsonEncode(args)},
                    },
                  ],
                };
          request.response.headers.contentType = ContentType(
            'text',
            'event-stream',
            charset: 'utf-8',
          );
          request.response.write(
            'data: ${jsonEncode({
              'choices': [
                {'delta': delta, 'finish_reason': null},
              ],
            })}\n\n',
          );
          request.response.write(
            'data: ${jsonEncode({
              'choices': [
                {'delta': {}, 'finish_reason': tool == null ? 'stop' : 'tool_calls'},
              ],
            })}\n\ndata: [DONE]\n\n',
          );
          await request.response.close();
        });
        return server;
      }))!;
      ChatController? owned;
      addTearDown(() async {
        owned?.dispose();
        await server.close(force: true);
        await bridge.close();
      });
      Future<void> waitUntil(bool Function() ready) async {
        final end = DateTime.now().add(const Duration(seconds: 25));
        do {
          await tester.runAsync(
            () => Future<void>.delayed(const Duration(milliseconds: 25)),
          );
          await tester.pump(const Duration(milliseconds: 100));
          if (owned?.toolApproval != null &&
              find.byKey(const Key('allow-tool')).evaluate().isNotEmpty) {
            await tester.ensureVisible(find.byKey(const Key('allow-tool')));
            await tester.tap(find.byKey(const Key('allow-tool')));
          }
          if (ready()) return;
        } while (DateTime.now().isBefore(end));
        fail(
          'Goal fixture deadline: requests=$requests, error=${owned?.error}, goal=${owned?.goal}',
        );
      }

      await call('configure', {
        'preferences': {
          'baseUrl': 'http://127.0.0.1:${server.port}/v1',
          'model': 'fixture',
        },
        'apiKey': '',
        'rememberConnection': true,
      });
      final session = (await call('createSession', {
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
      final chat = owned = ChatController(bridge, ownsBridge: false)
        ..loading = false;
      await tester.runAsync(chat.refresh);
      await tester.runAsync(() => chat.select(session));
      tester.view.physicalSize = const Size(1000, 1000);
      tester.view.devicePixelRatio = 1;
      addTearDown(tester.view.resetPhysicalSize);
      addTearDown(tester.view.resetDevicePixelRatio);
      await tester.pumpWidget(DoloresApp(chat: chat));
      await tester.pumpAndSettle();
      final view = find.byType(ChatPage);
      final context = tester.element(view);
      final dialog = showGoal(context, chat);
      await waitUntil(
        () => find.byKey(const Key('goal-objective')).evaluate().isNotEmpty,
      );
      await tester.enterText(
        find.byKey(const Key('goal-objective')),
        'Create goal-report.txt and verify its exact public content.',
      );
      await tester.enterText(
        find.byKey(const Key('goal-criteria')),
        'Report exists with PUBLIC_GOAL_RESULT and 世界 verified by read-back',
      );
      await tester.enterText(
        find.byKey(const Key('goal-constraints')),
        'Preserve unrelated files.',
      );
      await tester.tap(find.byKey(const Key('save-goal')));
      await waitUntil(() => chat.goal != null && !chat.changing);
      expect(requests, 0);
      await tester.tap(find.byKey(const Key('start-goal')));
      await tester.pump();
      await waitUntil(() => chat.goal?['state'] == 'completed' && !chat.busy);
      await dialog;
      expect(requests, 6);
      expect(creates, 1);
      expect(chat.goal!['used']['segments'], 2);
      expect(chat.goal!['used']['modelCalls'], 6);
      expect(chat.goal!['steps'][0]['evidence'], isNotEmpty);
      expect(
        File('${project.path}/goal-report.txt').readAsStringSync(),
        'PUBLIC_GOAL_RESULT\n世界\n',
      );
      final saved = await call('goal', {'session': session});
      expect(saved['state'], 'completed');
      expect(jsonEncode(requestsBody), contains('Preserve unrelated files.'));
      File('${directory.parent.path}/goal-receipt.json').writeAsStringSync(
        const JsonEncoder.withIndent('  ').convert({
          'mode': 'Rendered goal controls, normal Rust bridge, public deterministic fixture; no native pointer or real-model reliability claim',
          'providerRequests': requests,
          'fileEffects': creates,
          'goal': saved,
          'artifactVerified': true,
          'twoSegmentsSameBudget': true,
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
