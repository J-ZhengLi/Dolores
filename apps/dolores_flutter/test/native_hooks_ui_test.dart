import 'dart:convert';
import 'dart:io';

import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:dolores_flutter/bridge.dart';
import 'package:dolores_flutter/lifecycle_hooks.dart';
import 'package:dolores_flutter/theme.dart';

void main() {
  testWidgets(
    'native reviewed hooks gate tools and preserve observation failures',
    (t) async {
      final base = Directory(Platform.environment['DOLORES_DATA_DIR']!).parent;
      expect(
        base.resolveSymbolicLinksSync(),
        startsWith(
          Directory('../../output').absolute.resolveSymbolicLinksSync(),
        ),
      );
      final project = Directory('${base.path}/hooks-project')..createSync();
      final script = File('${project.path}/check.cjs')
        ..writeAsStringSync('''
const fs=require('fs');const event=JSON.parse(process.argv.at(-1));
fs.appendFileSync('hook-events.jsonl',JSON.stringify(event)+'\\n');
if(event.event==='afterTool'){console.log('PUBLIC_MALFORMED_OBSERVER');}
else {console.log(JSON.stringify({allow:event.event!=='beforeTool'||fs.existsSync('allow.txt'),message:'PUBLIC_HOOK_STATUS'}));}
''');
      final bridge = (await t.runAsync(() async {
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
        final envelope = (await t.runAsync(() async {
          try {
            return {
              'result': await bridge.call({'command': command, ...fields}),
            };
          } catch (e) {
            return {'error': '$e'};
          }
        }))!;
        if (envelope.containsKey('error')) throw envelope['error']!;
        return envelope['result'];
      }

      var requests = 0, runNumber = 0;
      final bodies = <Map>[];
      final server = (await t.runAsync(() async {
        final s = await HttpServer.bind(InternetAddress.loopbackIPv4, 0);
        s.listen((request) async {
          final body =
              jsonDecode(await utf8.decoder.bind(request).join()) as Map;
          bodies.add(body);
          final first = (requests++).isEven;
          final delta = first
              ? {
                  'tool_calls': [
                    {
                      'index': 0,
                      'id': 'public-create-$runNumber',
                      'type': 'function',
                      'function': {
                        'name': 'create_text_file',
                        'arguments': jsonEncode({
                          'path': 'result-$runNumber.txt',
                          'content': 'PUBLIC_RESULT',
                        }),
                      },
                    },
                  ],
                }
              : {'content': 'Public fixture turn finished.'};
          request.response.headers.contentType = ContentType(
            'text',
            'event-stream',
          );
          request.response.write(
            'data: ${jsonEncode({
              'choices': [
                {'index': 0, 'delta': delta, 'finish_reason': null},
              ],
            })}\n\n',
          );
          request.response.write(
            'data: ${jsonEncode({
              'choices': [
                {'index': 0, 'delta': {}, 'finish_reason': first ? 'tool_calls' : 'stop'},
              ],
            })}\n\ndata: [DONE]\n\n',
          );
          await request.response.close();
        });
        return s;
      }))!;
      addTearDown(() async {
        await server.close(force: true);
        await bridge.close();
      });
      await call('configure', {
        'preferences': {
          'baseUrl': 'http://127.0.0.1:${server.port}/v1',
          'model': 'fixture',
        },
        'apiKey': '',
        'rememberConnection': false,
      });
      final session =
          (await call('createSession', {
                'kind': 'project',
                'path': project.path,
              }))['session']['id']
              as String;
      final memories = await call('memories', {'session': session});
      if (memories['automaticPolicy']['enabled'] == true) {
        await call('setAutomaticMemory', {
          'enabled': false,
          'revision': memories['automaticPolicy']['revision'],
        });
      }
      Future<dynamic> hookCall(Map<String, dynamic> r) =>
          call('lifecycleHooks', {'session': session, 'request': r});
      Future<void> wait() async {
        final end = DateTime.now().add(const Duration(seconds: 15));
        while (find.byType(LinearProgressIndicator).evaluate().isNotEmpty) {
          await t.runAsync(
            () => Future<void>.delayed(const Duration(milliseconds: 20)),
          );
          await t.pump(const Duration(milliseconds: 40));
          if (DateTime.now().isAfter(end)) {
            fail('Hook settings exceeded fixture deadline.');
          }
        }
        await t.pumpAndSettle();
      }

      Future<void> reveal(String key) async {
        final target = find.byKey(Key(key));
        if (target.evaluate().isEmpty) {
          t
              .state<ScrollableState>(find.byType(Scrollable).first)
              .position
              .jumpTo(0);
          await t.pump();
          await t.scrollUntilVisible(
            target,
            160,
            scrollable: find.byType(Scrollable).first,
          );
        }
        await t.ensureVisible(target);
        await t.pump();
      }

      Future<void> press(String key) async {
        await reveal(key);
        await t.tap(find.byKey(Key(key)));
        await t.pump();
        await wait();
      }

      Future<void> fill(String key, String text) async {
        await reveal(key);
        await t.enterText(find.byKey(Key(key)), text);
        await t.pump();
      }

      t.view.physicalSize = const Size(900, 900);
      t.view.devicePixelRatio = 1;
      addTearDown(t.view.resetPhysicalSize);
      addTearDown(t.view.resetDevicePixelRatio);
      await t.pumpWidget(
        MaterialApp(
          theme: doloresTheme(false),
          home: Scaffold(
            body: LifecycleHooksInspector(bridge: bridge, session: session),
          ),
        ),
      );
      await wait();
      await fill('hook-name', 'gate');
      await fill('hook-script', 'check.cjs');
      await press('hook-stage');
      await press('hooks-review');
      expect(File('${project.path}/hook-events.jsonl').existsSync(), false);
      await press('hooks-enable');
      expect(File('${project.path}/hook-events.jsonl').existsSync(), false);
      final initial = await hookCall({'action': 'view'});
      expect(initial['configuration']['hooks'].length, 1);
      // Enable the rest of the same reviewed script's host events via the normal API.
      final hooks = <Map<String, dynamic>>[
        for (final event in [
          'taskStart',
          'beforeTool',
          'afterTool',
          'taskComplete',
        ])
          {
            'name': event,
            'event': event,
            'mode': event == 'beforeTool' ? 'blocking' : 'observe',
            'program': 'node',
            'script': 'check.cjs',
            'args': <String>[],
            'timeoutSeconds': 2,
          },
      ];
      final reviewed = await hookCall({
        'action': 'review',
        'revision': 1,
        'hooks': hooks,
      });
      await hookCall({'action': 'enable', 'token': reviewed['token']});
      Future<List> run() async {
        runNumber++;
        final id = 100 + runNumber;
        await call('start', {
          'id': id,
          'session': session,
          'input': 'Create one public fixture result.',
        });
        final end = DateTime.now().add(const Duration(seconds: 20));
        while (DateTime.now().isBefore(end)) {
          final events = await call('poll', {'id': id}) as List;
          for (final event in events) {
            if (event['type'] == 'toolApproval') {
              await call('approveTool', {
                'id': id,
                'callId': event['request']['callId'],
                'allow': true,
              });
            }
            if (event['type'] == 'done') {
              expect(event['error'], isNull);
              return (await call('messagesPage', {'session': session}))['items']
                  as List;
            }
          }
          await t.runAsync(
            () => Future<void>.delayed(const Duration(milliseconds: 20)),
          );
        }
        fail('Hook task exceeded fixture deadline.');
      }

      await run();
      expect(File('${project.path}/result-1.txt').existsSync(), false);
      File('${project.path}/allow.txt').writeAsStringSync('PUBLIC_ALLOW');
      final messages = await run();
      expect(
        File('${project.path}/result-2.txt').readAsStringSync(),
        'PUBLIC_RESULT',
      );
      final tool =
          (messages.lastWhere(
                    (m) => m['role'] == 'assistant',
                  )['metadata']['agent']['tools']
                  as List)
              .single;
      expect(tool['status'], 'created');
      final runs = await call('runs', {'session': session}) as List;
      final events = await call('runEvents', {
        'session': session,
        'runId': runs.first['id'],
      }) as List;
      expect(
        events.where(
          (e) =>
              e['kind'] == 'hookResult' &&
              e['data']['event'] == 'afterTool' &&
              e['data']['status'] == 'error',
        ),
        hasLength(1),
      );
      expect(
        events.where(
          (e) =>
              e['kind'] == 'hookResult' && e['data']['event'] == 'taskComplete',
        ),
        hasLength(1),
      );
      final eventData = File('${project.path}/hook-events.jsonl')
          .readAsLinesSync()
          .map((s) => jsonDecode(s) as Map)
          .toList();
      expect(
        eventData.every(
          (e) => e.keys.every(['schema', 'event', 'tool', 'outcome'].contains),
        ),
        true,
      );
      expect(jsonEncode(bodies), isNot(contains('PUBLIC_HOOK_STATUS')));
      expect(jsonEncode(bodies), isNot(contains('PUBLIC_MALFORMED_OBSERVER')));
      script.writeAsStringSync(
        'throw Error("changed script must not execute");',
      );
      await run();
      expect(File('${project.path}/result-3.txt').existsSync(), false);
      final state = await hookCall({'action': 'view'});
      await hookCall({
        'action': 'disable',
        'revision': state['configuration']['revision'],
      });
      await run();
      expect(
        File('${project.path}/result-4.txt').readAsStringSync(),
        'PUBLIC_RESULT',
      );
      File('${base.path}/hooks-receipt.json').writeAsStringSync(
        jsonEncode({
          'passed': true,
          'reviewDidNotRun': true,
          'blockingPreventedEffect': true,
          'observationFailurePreservedToolAndTurn': true,
          'changedScriptRefused': true,
          'disableRecovered': true,
          'hookContextBytes': 0,
          'modelRequests': requests,
        }),
      );
      await t.pumpWidget(const SizedBox());
      expect(t.takeException(), isNull);
    },
    skip:
        !Platform.isWindows ||
        Platform.environment['DOLORES_AUDIT_LIBRARY'] == null,
  );
}
