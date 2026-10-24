import 'dart:convert';
import 'dart:async';
import 'dart:io';

import 'package:dolores_flutter/app_host.dart';
import 'package:dolores_flutter/bridge.dart';
import 'package:dolores_flutter/chat.dart';
import 'package:dolores_flutter/main.dart';
import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';

import 'attention_test.dart' show NoticeWindow;
import 'frost_design_test.dart' show viewport;

void main() {
  testWidgets(
    'native attention opens a live question and stale evidence without answering or replaying work',
    (t) async {
      final directory = Directory(Platform.environment['DOLORES_DATA_DIR']!);
      void stage(String value) =>
          File('${directory.parent.path}/attention-stage.txt')
              .writeAsStringSync(value);
      stage('starting');
      final output = Directory('../../output').absolute
          .resolveSymbolicLinksSync();
      expect(
        directory.parent.resolveSymbolicLinksSync(),
        startsWith('$output${Platform.pathSeparator}'),
      );
      final bridge = (await t.runAsync(() async {
        final b = NativeBridge();
        await b.open(
          testLibrary: Platform.environment['DOLORES_AUDIT_LIBRARY'],
        );
        return b;
      }))!;
      AppHost? host;
      HttpServer? server;
      final bodies = <Map>[];
      final window = NoticeWindow();
      addTearDown(() async {
        await t.pumpWidget(const SizedBox());
        host?.dispose();
        await t.runAsync(() async {
          await server?.close(force: true);
          await bridge.close();
        });
      });
      Future<dynamic> call(
        String command, [
        Map<String, dynamic> fields = const {},
      ]) async {
        final r = (await t.runAsync(() async {
          try {
            return {
              'value': await bridge.call({'command': command, ...fields}),
            };
          } catch (e) {
            return {'error': '$e'};
          }
        }))!;
        if (r['error'] != null) throw StateError(r['error'] as String);
        return r['value'];
      }

      Future<void> wait(bool Function() done) async {
        final end = DateTime.now().add(const Duration(seconds: 30));
        do {
          await t.runAsync(
            () => Future<void>.delayed(const Duration(milliseconds: 25)),
          );
          await t.pump(const Duration(milliseconds: 100));
          if (done()) return;
        } while (DateTime.now().isBefore(end));
        fail(
          'Attention deadline: ${host?.error}; ${host?.attention.error}; ${host?.visible.error}',
        );
      }

      server = await t.runAsync(() async {
        final s = await HttpServer.bind(InternetAddress.loopbackIPv4, 0);
        s.listen((r) async {
          bodies.add(jsonDecode(await utf8.decoder.bind(r).join()) as Map);
          final first = bodies.length == 1;
          final delta = first
              ? {
                  'tool_calls': [
                    {
                      'index': 0,
                      'id': 'fixture-question',
                      'type': 'function',
                      'function': {
                        'name': 'request_user_input',
                        'arguments': jsonEncode({
                          'questions': [
                            {
                              'id': 'style',
                              'question': 'Public fixture: which style?',
                              'options': [
                                {'label': 'Brief'},
                                {'label': 'Detailed'},
                              ],
                            },
                          ],
                        }),
                      },
                    },
                  ],
                }
              : {
                  'content':
                      'Saved completed result after the explicit user answer.',
                };
          r.response.headers.contentType = ContentType(
            'text',
            'event-stream',
            charset: 'utf-8',
          );
          for (final e in [
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
                  'finish_reason': first ? 'tool_calls' : 'stop',
                },
              ],
            },
          ]) {
            r.response.write('data: ${jsonEncode(e)}\n\n');
          }
          r.response.write('data: [DONE]\n\n');
          await r.response.close();
        });
        return s;
      });
      await call('configure', {
        'preferences': {
          'baseUrl': 'http://127.0.0.1:${server!.port}/v1',
          'model': 'fixture',
        },
        'apiKey': '',
        'rememberConnection': true,
      });
      final task =
          (await call('createSession', {'kind': 'side'}))['session']['id']
              as String;
      final prefs = await call('attention', {
        'request': {'action': 'preferences'},
      }) as Map;
      await call('attention', {
        'request': {
          'action': 'savePreferences',
          'preferences': {...prefs, 'enabled': true},
        },
      });
      final c = ChatController(bridge, ownsBridge: false)..loading = false;
      await t.runAsync(c.refresh);
      await t.runAsync(() => c.select(task));
      host = AppHost(c, attentionWindow: window);
      viewport(t, const Size(1100, 900));
      await t.pumpWidget(DoloresApp(chat: c, host: host));
      await t.pumpAndSettle();
      c.draft = 'Ask one public question, then finish';
      await t.runAsync(c.send);
      await wait(
        () =>
            c.userQuestion != null &&
            window.shown.any((n) => n.endsWith(':question')),
      );
      final notice = window.shown
          .firstWhere((n) => n.endsWith(':question'))
          .split(':')
          .first;
      final slot = c.userQuestion!['callId'];
      stage('question-notified');
      await t.runAsync(() => host!.newConversation(kind: 'side'));
      await t.pumpAndSettle();
      final other = host.visible;
      other.draft = 'Keep other unsent draft 世界';
      await t.runAsync(() => window.opened!(notice));
      await t.pumpAndSettle();
      expect(host.visible, c);
      expect(c.userQuestion!['callId'], slot);
      expect(c.busy, true);
      expect(other.draft, 'Keep other unsent draft 世界');
      expect(bodies, hasLength(1));
      stage('live-click-preserved-question');
      c.setQuestionAnswer(0, 'Brief');
      await t.runAsync(c.answerQuestion);
      await wait(
        () =>
            !c.busy &&
            c.messages.length == 2 &&
            window.shown.any((n) => n.endsWith(':completed')),
      );
      expect(bodies, hasLength(2));
      expect(c.error, isNull);
      stage('answered-and-completed');
      // Native callbacks are represented by the injected window seam here. The
      // Windows runner callback/privacy boundary has a separate native fixture.
      var openedDone = false;
      unawaited(
        window.opened!(notice).then((_) {
          openedDone = true;
        }),
      );
      await t.pump();
      await wait(() => find.text('Saved task evidence').evaluate().isNotEmpty);
      stage('stale-evidence-visible');
      expect(
        find.textContaining('Current run state: completed'),
        findsOneWidget,
      );
      expect(c.userQuestion, isNull);
      expect(bodies, hasLength(2));
      await t.tap(find.text('Open conversation').last);
      await t.pumpAndSettle();
      stage('stale-evidence-closed');
      await wait(() => openedDone);
      stage('open-future-finished');
      final page = await call('attention', {
        'request': {'action': 'poll', 'cursor': null},
      });
      expect(
        (page['items'] as List).where((n) => n['kind'] == 'question'),
        hasLength(1),
      );
      expect(window.shown.where((n) => n.endsWith(':question')), hasLength(1));
      File('${directory.parent.path}/attention-receipt.json').writeAsStringSync(
        jsonEncode({
          'scope': 'Normal Rust bridge and rendered UI; injected native notification delivery seam. Windows callback code qualified separately; no real model or physical click claim',
          'providerRequests': bodies.length,
          'liveQuestionOpenedWithoutAnswer': true,
          'staleNoticeOpensEvidence': true,
          'otherDraftRetained': true,
          'deduplicated': true,
          'passed': true,
        }),
      );
      await t.pumpWidget(const SizedBox());
      host.dispose();
      host = null;
      await t.pump(const Duration(seconds: 2));
      expect(t.takeException(), isNull);
      stage('passed');
    },
    skip: Platform.environment['DOLORES_AUDIT_LIBRARY'] == null,
  );
}
