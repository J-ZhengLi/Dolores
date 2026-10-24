import 'dart:async';
import 'dart:convert';
import 'dart:io';

import 'package:dolores_flutter/app_host.dart';
import 'package:dolores_flutter/bridge.dart';
import 'package:dolores_flutter/chat.dart';
import 'package:dolores_flutter/main.dart';
import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';

import 'frost_design_test.dart' show viewport;

void main() {
  testWidgets(
    'native history searches saved answers and restores archived tasks without cancelling a running owner',
    (t) async {
      final data = Directory(Platform.environment['DOLORES_DATA_DIR']!);
      final output = Directory('../../output').absolute
          .resolveSymbolicLinksSync();
      expect(
        data.parent.resolveSymbolicLinksSync(),
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
      final held = Completer<void>();
      final bodies = <Map>[];
      addTearDown(() async {
        if (!held.isCompleted) held.complete();
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
        final result = (await t.runAsync(() async {
          try {
            return {
              'value': await bridge.call({'command': command, ...fields}),
            };
          } catch (e) {
            return {'error': '$e'};
          }
        }))!;
        if (result['error'] != null) {
          throw StateError(result['error'] as String);
        }
        return result['value'];
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
          'History fixture deadline: host=${host?.error}; chat=${host?.visible.error}; panel=${t.widgetList<SelectableText>(find.byType(SelectableText)).map((w) => w.data).join(" | ")}',
        );
      }

      server = await t.runAsync(() async {
        final s = await HttpServer.bind(InternetAddress.loopbackIPv4, 0);
        s.listen((r) async {
          bodies.add(jsonDecode(await utf8.decoder.bind(r).join()) as Map);
          final n = bodies.length;
          if (n == 3) await held.future;
          r.response.headers.contentType = ContentType(
            'text',
            'event-stream',
            charset: 'utf-8',
          );
          r.response.write(
            'data: ${jsonEncode({
              'choices': [
                {
                  'index': 0,
                  'delta': {'content': n == 1 ? 'Earlier saved answer: literal 中文预算 needle for search.' : 'Active task completed once.'},
                  'finish_reason': null,
                },
              ],
            })}\n\n',
          );
          r.response.write(
            'data: ${jsonEncode({
              'choices': [
                {'index': 0, 'delta': {}, 'finish_reason': 'stop'},
              ],
            })}\n\n',
          );
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
      final saved =
          (await call('createSession', {'kind': 'side'}))['session']['id']
              as String;
      final c = ChatController(bridge, ownsBridge: false)..loading = false;
      await t.runAsync(c.refresh);
      await t.runAsync(() => c.select(saved));
      host = AppHost(c);
      viewport(t, const Size(1100, 900));
      await t.pumpWidget(DoloresApp(chat: c, host: host));
      await t.pumpAndSettle();
      c.draft = 'Original unrelated title';
      await t.runAsync(c.send);
      await wait(() => !c.busy && c.messages.length == 2);
      expect(c.error, isNull);
      c.draft = 'Keep original unsent draft';
      await t.runAsync(c.checkpointDraft);
      await call('editTaskInputs', {
        'session': saved,
        'edit': {
          'action': 'enqueue',
          'id': 'retained-followup',
          'text': 'Keep my pending instruction',
          'mode': 'queue',
        },
      });
      final inputs = await call('taskInputs', {'session': saved});
      final active =
          (await call('createSession', {'kind': 'side'}))['session']['id']
              as String;
      await t.runAsync(() => host!.select(active));
      await t.pumpAndSettle();
      final running = host.visible;
      running.draft = 'Active task with an independent owner';
      await t.runAsync(running.send);
      await wait(
        () =>
            !running.busy && !running.changing && running.messages.length == 2,
      );
      running.draft = 'Continue the active task';
      await t.runAsync(running.send);
      await wait(() => bodies.length == 3 && running.busy);
      running.draft = 'Newer draft while task is running';
      await t.runAsync(running.checkpointDraft);
      String? rebuildRefusal;
      try {
        await call('history', {
          'request': {'action': 'rebuild'},
        });
      } catch (e) {
        rebuildRefusal = '$e';
      }
      expect(rebuildRefusal, contains('Finish or stop running tasks'));
      expect(running.busy, true);
      bool idle() {
        final button = find.widgetWithText(FilledButton, 'Search');
        return button.evaluate().isEmpty ||
            t.widget<FilledButton>(button).onPressed != null;
      }

      Future<void> search(String text) async {
        await t.tap(find.byKey(const Key('search-conversations')).first);
        await t.pump();
        await wait(
          () =>
              find
                  .byKey(const Key('history-search-text'))
                  .evaluate()
                  .isNotEmpty &&
              idle(),
        );
        await t.enterText(find.byKey(const Key('history-search-text')), text);
        await t.pump();
        await t.tap(find.text('Search'));
        await wait(idle);
      }

      Future<void> tap(String label) async {
        if (find.text(label).evaluate().isEmpty) {
          await t.scrollUntilVisible(
            find.text(label),
            200,
            scrollable: find
                .descendant(
                  of: find.byType(ListView).last,
                  matching: find.byType(Scrollable),
                )
                .first,
          );
        }
        await t.ensureVisible(find.text(label).last);
        await t.pump();
        await t.tap(find.text(label).last);
        await t.pump();
        await wait(idle);
      }

      Future<void> archiveFilter([
        String label = 'Archived conversations',
      ]) async {
        await t.drag(find.byType(ListView).last, const Offset(0, 1500));
        await t.pumpAndSettle();
        await t.tap(find.byKey(const Key('history-archive-filter')));
        await t.pumpAndSettle();
        await t.tap(find.text(label).last);
        await t.pumpAndSettle();
        await t.tap(find.text('Search'));
        await t.pump();
        await wait(idle);
      }

      await search('中文预算');
      await tap('Pin');
      await tap('Archive');
      await archiveFilter();
      await tap('Unarchive');
      await t.tap(find.text('Close'));
      await t.pumpAndSettle();
      await search('中文预算');
      await tap('Open conversation');
      await wait(
        () =>
            host!.visible.session == saved &&
            find.byKey(const Key('history-search-text')).evaluate().isEmpty,
      );
      expect(host.visible, c);
      expect(c.draft, 'Keep original unsent draft');
      expect(running.busy, true);
      expect(running.draft, 'Newer draft while task is running');
      expect(bodies, hasLength(3));
      expect(await call('taskInputs', {'session': saved}), inputs);
      await search('Active task with an independent owner');
      await tap('Archive');
      expect(running.busy, true);
      await archiveFilter();
      await tap('Unarchive');
      await archiveFilter('Current conversations');
      await tap('Open conversation');
      await wait(
        () =>
            host!.visible == running &&
            find.byKey(const Key('history-search-text')).evaluate().isEmpty,
      );
      expect(running.busy, true);
      expect(running.draft, 'Newer draft while task is running');
      held.complete();
      await wait(() => !running.busy && running.messages.length == 4);
      expect(running.error, isNull);
      expect(bodies, hasLength(3));
      await call('history', {
        'request': {'action': 'rebuild'},
      });
      final rebuilt = await call('history', {
        'request': {
          'action': 'search',
          'query': {
            'filter': {
              'text': '中文预算',
              'archive': 'visible',
              'pinned': null,
              'kind': null,
              'root': null,
            },
            'cursor': null,
          },
        },
      });
      expect(rebuilt['items'][0]['session']['id'], saved);
      expect(
        await call('savedDraft', {'session': saved}),
        'Keep original unsent draft',
      );
      final org = await call('history', {
        'request': {'action': 'organization', 'session': saved},
      });
      expect(org['pinned'], true);
      expect(org['archived'], false);
      File('${data.parent.path}/history-search-receipt.json').writeAsStringSync(
        jsonEncode({
          'scope': 'Rendered UI and normal native bridge with three local fixture requests; no external model or physical desktop reliability claim',
          'providerRequests': bodies.length,
          'savedAnswerSearch': true,
          'archiveRestore': true,
          'pendingInputAndDraftsRetained': true,
          'activeOwnerNotCancelled': true,
          'rebuildRefusedWhileRunningAndRecoveredAfterCompletion': true,
          'exactTargetOpened': true,
          'passed': true,
        }),
      );
      await t.pumpWidget(const SizedBox());
      host.dispose();
      host = null;
      await t.pump(const Duration(seconds: 2));
      expect(t.takeException(), isNull);
    },
    skip: Platform.environment['DOLORES_AUDIT_LIBRARY'] == null,
  );
}
