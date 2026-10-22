import 'dart:async';
import 'dart:convert';
import 'dart:io';
import 'dart:ui' as ui;

import 'package:dolores_flutter/bridge.dart';
import 'package:dolores_flutter/chat.dart';
import 'package:dolores_flutter/command_sessions.dart';
import 'package:dolores_flutter/main.dart';
import 'package:flutter/material.dart';
import 'package:flutter/rendering.dart';
import 'package:flutter_test/flutter_test.dart';

import 'support/render_fonts.dart';

void main() => runCommandJourney();

void runCommandJourney({bool isolated = false}) {
  setUpAll(() async {
    if (Platform.environment['DOLORES_AUDIT_LIBRARY'] != null) {
      await loadReviewFonts();
    }
  });
  testWidgets(
    '${isolated ? 'Isolated' : 'Native'} command input, interrupt and saved output use rendered controls',
    (tester) async {
      final data = Directory(Platform.environment['DOLORES_DATA_DIR']!);
      final outputRoot = Directory('../../output').absolute
          .resolveSymbolicLinksSync();
      expect(
        data.parent.resolveSymbolicLinksSync(),
        startsWith('$outputRoot${Platform.pathSeparator}'),
      );
      final project = Directory(
        '${data.parent.path}/${isolated ? 'isolated' : 'command'}-project',
      )..createSync();
      final prefix = isolated ? 'commands-isolated' : 'commands';
      final outside = File('${data.parent.path}/public-outside.txt');
      if (isolated) outside.writeAsStringSync('PUBLIC_KEEP');
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
            return {'error': '$e'};
          }
        }))!;
        if (result.containsKey('error')) throw result['error']!;
        return result['value'];
      }

      final releaseReply = Completer<void>();
      final requests = <Map>[];
      final server = (await tester.runAsync(() async {
        final server = await HttpServer.bind(InternetAddress.loopbackIPv4, 0);
        server.listen((request) async {
          requests.add(
            jsonDecode(await utf8.decoder.bind(request).join()) as Map,
          );
          final first = requests.length == 1;
          if (!first) await releaseReply.future;
          final delta = first
              ? {
                  'tool_calls': [
                    {
                      'index': 0,
                      'id': 'public-command-start',
                      'type': 'function',
                      'function': {
                        'name': 'command_session',
                        'arguments': jsonEncode({
                          'operation': 'start',
                          'program': 'node',
                          if (isolated) 'isolation': 'isolated',
                          'args': [
                            '-e',
                            "${isolated ? "try{require('fs').writeFileSync('../public-outside.txt','BAD')}catch(e){console.log('PUBLIC_FILE_DENIED '+e.code)}const s=require('net').createConnection({host:'127.0.0.1',port:PLACEHOLDER});s.on('error',e=>{console.log('PUBLIC_NETWORK_DENIED '+e.code);s.destroy()});s.on('connect',()=>{console.log('UNEXPECTED_CONNECTION');s.destroy()});".replaceAll('PLACEHOLDER', '${request.connectionInfo!.localPort}') : ''}process.stdin.on('data', b => {process.stdout.write(b);require('fs').appendFileSync('public-input.txt',b)});console.log('PUBLIC_COMMAND_READY')",
                          ],
                        }),
                      },
                    },
                  ],
                }
              : {
                  'content':
                      'Command interrupted; retained output is available.',
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
                {'delta': {}, 'finish_reason': first ? 'tool_calls' : 'stop'},
              ],
            })}\n\ndata: [DONE]\n\n',
          );
          await request.response.close();
        });
        return server;
      }))!;
      ChatController? owned;
      addTearDown(() async {
        if (!releaseReply.isCompleted) releaseReply.complete();
        owned?.dispose();
        await tester.runAsync(() async {
          await bridge.close();
          await server.close(force: true);
        });
      });
      Future<void> waitUntil(
        bool Function() ready, {
        bool approve = false,
      }) async {
        final end = DateTime.now().add(const Duration(seconds: 20));
        do {
          await tester.runAsync(
            () => Future<void>.delayed(const Duration(milliseconds: 30)),
          );
          await tester.pump(const Duration(milliseconds: 100));
          if (approve &&
              owned?.toolApproval != null &&
              find.byKey(const Key('allow-tool')).evaluate().isNotEmpty) {
            await tester.ensureVisible(find.byKey(const Key('allow-tool')));
            await tester.tap(find.byKey(const Key('allow-tool')));
          }
          if (ready()) return;
          if (owned?.error != null && owned?.busy == false) {
            fail('Command fixture stopped before completion: ${owned?.error}');
          }
        } while (DateTime.now().isBefore(end));
        fail(
          'Command fixture deadline: requests=${requests.length}, error=${owned?.error}',
        );
      }

      await call('bootstrap');
      final policy = (await call('memories'))['automaticPolicy'];
      await call('setAutomaticMemory', {
        'enabled': false,
        'revision': policy['revision'],
      });
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
      final chat = owned = ChatController(bridge, ownsBridge: false)
        ..loading = false;
      await tester.runAsync(chat.refresh);
      await tester.runAsync(() => chat.select(session));
      expect(chat.commandSessionsAvailable, true);
      tester.view.physicalSize = const Size(1080, 850);
      tester.view.devicePixelRatio = 1;
      addTearDown(tester.view.resetPhysicalSize);
      addTearDown(tester.view.resetDevicePixelRatio);
      final renderKey = GlobalKey();
      await tester.pumpWidget(
        RepaintBoundary(
          key: renderKey,
          child: DoloresApp(chat: chat),
        ),
      );
      await tester.pumpAndSettle();
      await tester.enterText(
        find.byKey(const Key('composer-field-0')),
        'Start the public command fixture.',
      );
      await tester.pump();
      expect(
        tester.widget<IconButton>(find.byKey(const Key('send'))).onPressed,
        isNotNull,
      );
      await tester.tap(find.byKey(const Key('send')));
      if (isolated) {
        await waitUntil(() => chat.toolApproval != null);
        expect(
          find.byKey(const Key('command-isolation-grants')),
          findsOneWidget,
        );
        expect(find.textContaining('network denied.'), findsOneWidget);
        expect(
          find.textContaining('Runs with your OS permissions'),
          findsNothing,
        );
      }
      await waitUntil(() => requests.length == 2, approve: true);
      chat.draft = 'Keep my next question';
      await call('saveDraft', {'session': session, 'text': chat.draft});
      final dialog = showCommandSessions(
        tester.element(find.byType(ChatPage)),
        chat,
      );
      await waitUntil(
        () => find.textContaining('PUBLIC_COMMAND_READY').evaluate().isNotEmpty,
      );
      await tester.enterText(
        find.byKey(const Key('command-input')),
        'PUBLIC_INPUT 世界\n',
      );
      await tester.tap(find.byKey(const Key('command-send')));
      await waitUntil(
        () => File('${project.path}/public-input.txt').existsSync(),
      );
      await waitUntil(
        () => find.textContaining('PUBLIC_INPUT 世界').evaluate().isNotEmpty,
      );
      expect(
        File('${project.path}/public-input.txt').readAsStringSync(),
        'PUBLIC_INPUT 世界\n',
      );
      await tester.tap(find.text('Interrupt'));
      await waitUntil(
        () => tester
            .widget<TextField>(find.byKey(const Key('command-input')))
            .readOnly,
      );
      await tester.pumpAndSettle();
      final boundary =
          renderKey.currentContext!.findRenderObject() as RenderRepaintBoundary;
      await tester.runAsync(() async {
        final image = await boundary.toImage();
        final bytes = await image.toByteData(format: ui.ImageByteFormat.png);
        await File('${data.parent.path}/$prefix-native.png')
            .writeAsBytes(bytes!.buffer.asUint8List());
        image.dispose();
      });
      await tester.tap(find.text('Close').first);
      await tester.pumpAndSettle();
      await dialog;
      releaseReply.complete();
      await waitUntil(() => !chat.busy);
      expect(chat.draft, 'Keep my next question');
      final page = await call('commandSession', {
        'session': session,
        'request': {'operation': 'list'},
      });
      final receipt = (page['items'] as List).single as Map;
      expect(receipt['state'], 'interrupted');
      expect(receipt['cleanupComplete'], true);
      if (isolated) {
        expect(receipt['execution'], startsWith('isolated;'));
        expect(page['isolation']['pendingCleanup'], 0);
        expect(outside.readAsStringSync(), 'PUBLIC_KEEP');
      }
      expect(page['liveItems'], isEmpty);
      final saved = await call('commandSession', {
        'session': session,
        'request': {
          'operation': 'read',
          'run': receipt['run'],
          'id': receipt['id'],
          'offset': 0,
        },
      });
      expect(saved['output'], contains('PUBLIC_INPUT 世界'));
      if (isolated) {
        expect(saved['output'], contains('PUBLIC_FILE_DENIED EPERM'));
        expect(saved['output'], contains('PUBLIC_NETWORK_DENIED EACCES'));
      }
      final reopened = showCommandSessions(
        tester.element(find.byType(ChatPage)),
        chat,
        run: receipt['run'] as String,
      );
      await waitUntil(
        () => find.textContaining('PUBLIC_INPUT 世界').evaluate().isNotEmpty,
      );
      expect(
        tester
            .widget<TextField>(find.byKey(const Key('command-input')))
            .readOnly,
        true,
      );
      expect(
        tester
            .widget<FilledButton>(find.byKey(const Key('command-send')))
            .onPressed,
        isNull,
      );
      await tester.tap(find.text('Close').first);
      await tester.pumpAndSettle();
      await reopened;
      final events = await call('runEvents', {
        'session': session,
        'runId': receipt['run'],
      });
      expect(jsonEncode(events), contains('explicit human stdin Send'));
      expect(requests.length, 2);
      File('${data.parent.path}/$prefix-receipt.json').writeAsStringSync(
        jsonEncode({
          'scope': 'public loopback provider; rendered Flutter with normal native bridge; no real model or physical input qualification',
          'receipt': receipt,
          'savedOutput': saved,
          'requests': requests.length,
          'isolated': isolated,
          'draftRetained': chat.draft == 'Keep my next question',
          'passed': true,
        }),
      );
      expect(tester.takeException(), isNull);
      await tester.pumpWidget(const SizedBox());
    },
    skip: Platform.environment['DOLORES_AUDIT_LIBRARY'] == null,
  );
}
