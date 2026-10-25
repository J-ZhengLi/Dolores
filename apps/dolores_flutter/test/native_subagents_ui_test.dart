import 'dart:async';
import 'dart:convert';
import 'dart:io';
import 'dart:ui' as ui;

import 'package:dolores_flutter/bridge.dart';
import 'package:dolores_flutter/chat.dart';
import 'package:dolores_flutter/main.dart';
import 'package:flutter/material.dart';
import 'package:flutter/rendering.dart';
import 'package:flutter_test/flutter_test.dart';

import 'support/render_fonts.dart';

void main() {
  testWidgets(
    'native child supervision retains effects and parent ownership',
    (t) async {
      final base = Directory(Platform.environment['DOLORES_DATA_DIR']!).parent;
      expect(
        base.resolveSymbolicLinksSync(),
        startsWith(
          Directory('../../output').absolute.resolveSymbolicLinksSync(),
        ),
      );
      final project = Directory('${base.path}/child-project')..createSync();
      File('${project.path}/sibling.txt').writeAsStringSync('PUBLIC_SIBLING');
      final retained = File('${project.path}/retained.txt');
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
              'value': await bridge.call({'command': command, ...fields}),
            };
          } catch (e) {
            return {'error': '$e'};
          }
        }))!;
        if (envelope.containsKey('error')) throw envelope['error']!;
        return envelope['value'];
      }

      var stage = 0, mode = 1, requests = 0;
      var writingChildWaiting = false;
      final release = Completer<void>();
      final bodies = <Map>[];
      final server = (await t.runAsync(() async {
        final s = await HttpServer.bind(InternetAddress.loopbackIPv4, 0);
        s.listen((request) async {
          final body =
              jsonDecode(await utf8.decoder.bind(request).join()) as Map;
          bodies.add(body);
          requests++;
          final messages = body['messages'] as List;
          final child = '${messages.first['content']}'.contains(
            'You are a scoped child',
          );
          Map delta;
          Map tool(String name, Map arguments) => {
            'tool_calls': [
              {
                'index': 0,
                'id': 'fixture-$requests',
                'type': 'function',
                'function': {'name': name, 'arguments': jsonEncode(arguments)},
              },
            ],
          };
          if (child) {
            final writing = '${messages[1]['content']}'.contains(
              'Write retained.txt',
            );
            if (writing && messages.length > 2) {
              writingChildWaiting = true;
              await release.future;
              await request.response.close();
              return;
            }
            if (messages.length == 2) {
              delta = writing
                  ? tool('create_text_file', {
                      'path': 'retained.txt',
                      'content': 'PUBLIC_RETAINED',
                    })
                  : tool('read_text_file', {
                      'path':
                          '${messages[1]['content']}'.contains(
                            'Read retained.txt',
                          )
                          ? 'retained.txt'
                          : 'sibling.txt',
                    });
            } else {
              delta = {
                'content': 'PUBLIC_CHILD_REPORT: observed file evidence. No tests ran.',
              };
            }
          } else {
            switch (stage++) {
              case 0:
                delta = tool('delegate_tasks', {
                  'tasks': [
                    {
                      'goal': mode == 1
                          ? 'Write retained.txt'
                          : 'Read retained.txt',
                      'scope': 'retained.txt',
                      'readOnly': mode != 1,
                      'model': 'fixture-child',
                    },
                    {
                      'goal': 'Read sibling.txt',
                      'scope': 'sibling.txt',
                      'readOnly': true,
                    },
                  ],
                });
              case 1:
                final report = jsonDecode(messages.last['content'] as String);
                delta = tool('delegate_tasks', {
                  'childId': report['children'][1]['childId'],
                  'action': 'followUp',
                  'message': 'Clarify the exact observed evidence; keep the same ownership.',
                });
              case 2:
                final report = jsonDecode(messages.last['content'] as String);
                delta = tool('delegate_tasks', {
                  'childId': report['children'][0]['childId'],
                  'action': 'inspect',
                });
              case 3:
                delta = tool('read_text_file', {'path': 'retained.txt'});
              case 4:
                expect(
                  '${messages.last['content']}',
                  contains('PUBLIC_RETAINED'),
                );
                delta = tool('read_text_file', {'path': 'sibling.txt'});
              default:
                expect(
                  '${messages.last['content']}',
                  contains('PUBLIC_SIBLING'),
                );
                delta = {
                  'content': 'PUBLIC_PARENT_VERIFIED retained file bytes; interrupted child requires review.',
                };
            }
          }
          request.response.headers.contentType = ContentType(
            'text',
            'event-stream',
            charset: 'utf-8',
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
                {'index': 0, 'delta': {}, 'finish_reason': delta.containsKey('tool_calls') ? 'tool_calls' : 'stop'},
              ],
            })}\n\ndata: [DONE]\n\n',
          );
          await request.response.close();
        });
        return s;
      }))!;
      addTearDown(() async {
        if (!release.isCompleted) release.complete();
        await server.close(force: true);
        await bridge.close();
      });
      await call('bootstrap');
      await call('configure', {
        'preferences': {
          'baseUrl': 'http://127.0.0.1:${server.port}/v1',
          'model': 'fixture-parent',
        },
        'enabledModels': ['fixture-parent', 'fixture-child'],
        'apiKey': '',
        'rememberConnection': false,
      });
      final session = (await call('createSession', {
        'kind': 'project',
        'path': project.path,
      }))['session']['id'];
      final memories = await call('memories', {'session': session});
      await call('setAutomaticMemory', {
        'enabled': false,
        'revision': memories['automaticPolicy']['revision'],
      });
      final chat = ChatController(bridge, ownsBridge: false)..loading = false;
      addTearDown(chat.dispose);
      await t.runAsync(chat.refresh);
      await t.runAsync(() => chat.select(session));
      t.view.physicalSize = const Size(760, 800);
      t.view.devicePixelRatio = 1;
      addTearDown(t.view.resetPhysicalSize);
      addTearDown(t.view.resetDevicePixelRatio);
      await t.runAsync(loadReviewFonts);
      final renderKey = GlobalKey();
      Future<void> show(bool dark) async {
        await t.pumpWidget(
          RepaintBoundary(
            key: renderKey,
            child: DoloresApp(
              chat: chat,
              themeMode: dark ? ThemeMode.dark : ThemeMode.light,
            ),
          ),
        );
        await t.pump(const Duration(milliseconds: 300));
      }

      Future<void> waitUntil(
        bool Function() ready, {
        bool approve = false,
      }) async {
        final deadline = DateTime.now().add(const Duration(seconds: 18));
        do {
          await t.runAsync(
            () => Future<void>.delayed(const Duration(milliseconds: 20)),
          );
          await t.pump(const Duration(milliseconds: 100));
          if (ready()) return;
          if (approve && chat.toolApproval != null) {
            await t.runAsync(() => chat.decideTool(true));
          }
        } while (DateTime.now().isBefore(deadline));
        fail(
          'Child fixture deadline; phase=$stage error=${chat.error} approval=${chat.toolApproval}',
        );
      }

      Future<void> capture(String name) async {
        expect(t.takeException(), isNull);
        final boundary =
            renderKey.currentContext!.findRenderObject()
                as RenderRepaintBoundary;
        await t.runAsync(() async {
          final image = await boundary.toImage(pixelRatio: 1);
          final bytes = await image.toByteData(format: ui.ImageByteFormat.png);
          File('${base.path}/$name.png')
              .writeAsBytesSync(bytes!.buffer.asUint8List());
          image.dispose();
        });
      }

      await show(false);
      await t.enterText(
        find.byKey(const Key('composer-field-0')),
        'Delegate the public owned file tasks, then verify actual saved bytes.',
      );
      await t.pump();
      await t.tap(find.byKey(const Key('send')));
      await waitUntil(
        () => writingChildWaiting && retained.existsSync(),
        approve: true,
      );
      final left = chat.subagents.firstWhere(
        (c) => c['scope'] == 'retained.txt',
      );
      expect(left['model'], 'fixture-child');
      await t.ensureVisible(find.text('Write retained.txt').first);
      await t.tap(find.text('Write retained.txt').first);
      await t.pump(const Duration(milliseconds: 300));
      for (final dark in [false, true]) {
        await show(dark);
        await t.ensureVisible(find.text('Inspect child context').first);
        await t.tap(find.text('Inspect child context').first);
        await waitUntil(
          () => find
              .textContaining('System:\nYou are Dolores')
              .evaluate()
              .isNotEmpty,
        );
        await capture('children-context-${dark ? 'dark' : 'light'}');
        await t.tap(find.text('Close').last);
        await t.pump(const Duration(milliseconds: 300));
      }
      await t.ensureVisible(find.text('Interrupt child'));
      await t.tap(find.text('Interrupt child'));
      await waitUntil(() => !chat.busy && !chat.changing, approve: true);
      expect(retained.readAsStringSync(), 'PUBLIC_RETAINED');
      expect(chat.messages.last['content'], contains('PUBLIC_PARENT_VERIFIED'));
      final first = (await call('runs', {'session': session})).first;
      final saved = await call('runEvents', {
        'session': session,
        'runId': first['id'],
      });
      final children = <String, Map>{};
      for (final event in saved as List) {
        if (event['kind'] == 'subagent') {
          final child = event['data'] as Map;
          children[child['scope'] as String] = child;
        }
      }
      expect(children['retained.txt']!['status'], 'interrupted');
      final sibling = children['sibling.txt']!;
      expect(sibling['status'], 'reported');
      expect(sibling['followUps'], 1);
      expect(sibling['usage']['modelCalls'], 3);
      expect(sibling['verified'], false);
      expect(jsonEncode(saved), contains('subagentContext'));
      expect(jsonEncode(saved), contains('subagentFollowUp'));
      final childBodies = bodies
          .where(
            (b) => '${b['messages'][0]['content']}'.contains(
              'You are a scoped child',
            ),
          )
          .toList();
      expect(childBodies.any((b) => b['model'] == 'fixture-child'), true);
      expect(
        childBodies.every(
          (b) => (b['tools'] as List).every(
            (tool) => [
              'read_text_file',
              'list_folder',
              'search_text',
              'edit_text_file',
              'create_text_file',
            ].contains(tool['function']['name']),
          ),
        ),
        true,
      );
      // A second run stops while a child file approval is pending. Neither a
      // fresh batch nor this follow-up lifecycle grants that file operation.
      mode = 2;
      stage = 0;
      await t.enterText(
        find.byKey(const Key('composer-field-0')),
        'Read both retained fixture files in scoped children.',
      );
      await t.pump();
      await t.tap(find.byKey(const Key('send')));
      await waitUntil(() => chat.toolApproval?['name'] == 'delegate_tasks');
      await t.runAsync(() => chat.decideTool(true));
      await waitUntil(
        () =>
            chat.toolApproval?['name'] == 'read_text_file' &&
            chat.subagents.any((c) => c['status'] == 'running'),
      );
      final pending = chat.toolApproval!['callId'];
      final liveRun = (await call('runs', {'session': session})).first;
      final liveEvents = await call('runEvents', {
        'session': session,
        'runId': liveRun['id'],
      }) as List;
      final client = liveEvents.firstWhere(
        (e) => e['kind'] == 'started',
      )['data']['clientId'];
      for (final owner in [
        {'session': 'different-chat', 'id': client, 'childId': left['childId']},
        {'session': session, 'id': 0, 'childId': left['childId']},
        {'session': session, 'id': client, 'childId': left['childId']},
      ]) {
        var refused = false;
        try {
          await call('childSupervision', {
            'session': owner['session'],
            'id': owner['id'],
            'request': {'childId': owner['childId'], 'action': 'interrupt'},
          });
        } catch (_) {
          refused = true;
        }
        expect(refused, true);
        expect(chat.toolApproval!['callId'], pending);
        expect(chat.busy, true);
      }
      await t.tap(find.byTooltip('Stop response'));
      await waitUntil(() => !chat.busy && !chat.changing);
      expect(retained.readAsStringSync(), 'PUBLIC_RETAINED');
      expect(chat.subagents.every((c) => c['status'] == 'interrupted'), true);
      final stopped = (await call('runs', {'session': session})).first;
      expect(stopped['state'], 'cancelled');
      File('${base.path}/subagents-receipt.json').writeAsStringSync(
        jsonEncode({
          'passed': true,
          'fixtureRequests': requests,
          'liveRequests': 0,
          'frozenOptionalModel': true,
          'childInterruptKeptSiblingAndWrites': true,
          'sameIdentityFollowUp': true,
          'parentReadVerifiedActualBytes': true,
          'contextSavedAndRendered': true,
          'parentStopDuringChildApproval': pending,
          'automaticReplays': 0,
          'staleSessionParentAndChildRefused': true,
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
