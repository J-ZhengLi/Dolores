import 'dart:async';
import 'dart:convert';
import 'dart:io';

import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:dolores_flutter/bridge.dart';
import 'package:dolores_flutter/chat.dart';
import 'package:dolores_flutter/main.dart';
import 'package:dolores_flutter/run_history.dart';

void main() {
  testWidgets(
    'Stop preserves completed file effects and prepares explicit recovery',
    (tester) async {
      final directory = Directory(Platform.environment['DOLORES_DATA_DIR']!);
      final output = Directory('../../output').absolute
          .resolveSymbolicLinksSync();
      expect(
        directory.parent.resolveSymbolicLinksSync(),
        startsWith('$output${Platform.pathSeparator}'),
      );
      final project = Directory('${directory.parent.path}/stop-project')
        ..createSync();
      final file = File('${project.path}/completed.txt');
      const content = 'PUBLIC-COMPLETED-EFFECT\n中文\n';
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
        final end = DateTime.now().add(const Duration(seconds: 15));
        do {
          await tester.runAsync(
            () => Future<void>.delayed(const Duration(milliseconds: 25)),
          );
          await tester.pump(const Duration(milliseconds: 100));
          if (ready()) return;
        } while (DateTime.now().isBefore(end));
        fail('Stop/recovery operation did not finish');
      }

      var requests = 0;
      final release = Completer<void>();
      final bodies = <dynamic>[];
      final server = (await tester.runAsync(() async {
        final server = await HttpServer.bind(InternetAddress.loopbackIPv4, 0);
        server.listen((request) async {
          bodies.add(jsonDecode(await utf8.decoder.bind(request).join()));
          requests++;
          if (requests > 1) {
            // Stall only after the returned create receipt has reached the model.
            await release.future;
            await request.response.close();
            return;
          }
          request.response.headers.contentType = ContentType(
            'text',
            'event-stream',
          charset: 'utf-8',
          );
          final delta = {
            'tool_calls': [
              {
                'index': 0,
                'id': 'completed-effect',
                'type': 'function',
                'function': {
                  'name': 'create_text_file',
                  'arguments': jsonEncode({
                    'path': 'completed.txt',
                    'content': content,
                  }),
                },
              },
            ],
          };
          for (final event in [
            {
              'choices': [
                {'index': 0, 'delta': delta, 'finish_reason': null},
              ],
            },
            {
              'choices': [
                {'index': 0, 'delta': {}, 'finish_reason': 'tool_calls'},
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
      await call('bootstrap');
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
      final chat = ChatController(bridge, ownsBridge: false)..loading = false;
      await tester.runAsync(chat.refresh);
      await tester.runAsync(() => chat.select(session));
      tester.view.physicalSize = const Size(760, 900);
      tester.view.devicePixelRatio = 1;
      addTearDown(tester.view.resetPhysicalSize);
      addTearDown(tester.view.resetDevicePixelRatio);
      await tester.pumpWidget(DoloresApp(chat: chat));
      await tester.pumpAndSettle();
      const prompt =
          'Create completed.txt containing the public fixture, then summarize it.';
      await tester.enterText(find.byKey(const Key('composer-field-0')), prompt);
      await tester.pump();
      await tester.tap(find.byKey(const Key('send')));
      await waitUntil(() => chat.toolApproval != null);
      expect(file.existsSync(), isFalse);
      expect(chat.toolApproval?['name'], 'create_text_file');
      await tester.ensureVisible(find.byKey(const Key('allow-tool')));
      await tester.tap(find.byKey(const Key('allow-tool')));
      await waitUntil(() => requests == 2 && chat.toolRecords.isNotEmpty);
      expect(file.readAsStringSync(), content);
      expect(chat.toolRecords.single['status'], 'created');
      expect(jsonEncode(bodies.last), contains('completed-effect'));
      final bytes = file.readAsBytesSync();
      await tester.tap(find.byTooltip('Stop response'));
      await waitUntil(() => !chat.busy && !chat.changing);
      expect(chat.error, contains('stopped'));
      expect(chat.draft, prompt);
      expect(file.readAsBytesSync(), bytes);
      expect(chat.toolRecords.single['status'], 'created');
      final runs = await call('runs', {'session': session});
      final run = runs.single;
      expect(run['state'], 'cancelled');
      final checkpoint = await call('runCheckpoint', {
        'session': session,
        'runId': run['id'],
      });
      expect(checkpoint['evidence'], hasLength(1));
      expect(checkpoint['evidence'][0]['returned'], true);
      expect(checkpoint['uncertainEffects'], isEmpty);
      const newer = 'Retain my newer draft';
      await tester.enterText(find.byKey(const Key('composer-field-0')), newer);
      await tester.runAsync(chat.checkpointDraft);
      Future<void> history() async {
        await tester.pumpWidget(
          MaterialApp(
            key: UniqueKey(),
            home: Builder(
              builder: (context) => Scaffold(
                body: TextButton(
                  onPressed: () => showRunHistory(context, chat),
                  child: const Text('History'),
                ),
              ),
            ),
          ),
        );
        await tester.tap(find.text('History'));
        await waitUntil(() => find.byType(ListTile).evaluate().isNotEmpty);
        await tester.tap(find.byType(ListTile).first);
        await waitUntil(
          () => find.text('Prepare resume draft').evaluate().isNotEmpty,
        );
        await tester.ensureVisible(find.text('Prepare resume draft'));
        await tester.tap(find.text('Prepare resume draft'));
        await tester.pump();
      }

      await history();
      await waitUntil(
        () => find
            .textContaining('Send or clear the current draft')
            .evaluate()
            .isNotEmpty,
      );
      expect(chat.draft, newer);
      expect(await call('savedDraft', {'session': session}), newer);
      await tester.pumpWidget(DoloresApp(chat: chat));
      await tester.pumpAndSettle();
      await tester.enterText(find.byKey(const Key('composer-field-0')), '');
      await tester.runAsync(chat.checkpointDraft);
      await history();
      await waitUntil(() => chat.resumeRun == run['id']);
      expect(chat.draft, contains('Uncertain effects: []'));
      expect(chat.draft, contains('Never repeat an already-applied operation'));
      expect(chat.draft, contains('completed-effect'));
      expect(chat.draft, contains('completed.txt'));
      expect(chat.busy, isFalse);
      expect(requests, 2);
      expect(file.readAsBytesSync(), bytes);
      File('${directory.parent.path}/stop-receipt.json').writeAsStringSync(
        const JsonEncoder.withIndent('  ').convert({
          'mode': 'Rendered controls with normal Rust bridge and loopback fixture; no native-pointer/real-model claim',
          'run': run['id'],
          'state': run['state'],
          'completedReceipt': checkpoint['evidence'],
          'fileBytesRetained': true,
          'newerDraftProtected': true,
          'recoveryDraftPrepared': true,
          'automaticReplays': 0,
          'providerRequests': requests,
        }),
      );
      expect(tester.takeException(), isNull);
      await tester.pumpWidget(const SizedBox());
      chat.dispose();
      await tester.runAsync(() async {
        release.complete();
        await server.close(force: true);
        await bridge.close();
      });
    },
    skip:
        !Platform.isWindows ||
        Platform.environment['DOLORES_AUDIT_LIBRARY'] == null,
  );
}
