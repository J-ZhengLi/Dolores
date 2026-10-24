import 'dart:convert';
import 'dart:io';

import 'package:dolores_flutter/app_host.dart';
import 'package:dolores_flutter/bridge.dart';
import 'package:dolores_flutter/chat.dart';
import 'package:dolores_flutter/main.dart';
import 'package:flutter/material.dart';
import 'package:flutter/rendering.dart';
import 'package:flutter_test/flutter_test.dart';

import 'frost_design_test.dart' show viewport;

void main() {
  testWidgets(
    'native selected text opens an independent side chat and shares one selected conclusion through the parent owner',
    (t) async {
      final directory = Directory(Platform.environment['DOLORES_DATA_DIR']!);
      final output = Directory('../../output').absolute
          .resolveSymbolicLinksSync();
      expect(
        directory.parent.resolveSymbolicLinksSync(),
        startsWith('$output${Platform.pathSeparator}'),
      );
      final bridge = (await t.runAsync(() async {
        final value = NativeBridge();
        await value.open(
          testLibrary: Platform.environment['DOLORES_AUDIT_LIBRARY'],
        );
        return value;
      }))!;
      AppHost? host;
      HttpServer? server;
      addTearDown(() async {
        await t.pumpWidget(const SizedBox());
        host?.dispose();
        await t.runAsync(() async {
          await server?.close(force: true);
          await bridge.close();
        });
      });
      Future<dynamic> call(
        String name, [
        Map<String, dynamic> fields = const {},
      ]) async {
        final result = (await t.runAsync(() async {
          try {
            return {
              'value': await bridge.call({'command': name, ...fields}),
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
        final end = DateTime.now().add(const Duration(seconds: 25));
        do {
          await t.runAsync(
            () => Future<void>.delayed(const Duration(milliseconds: 25)),
          );
          await t.pump(const Duration(milliseconds: 100));
          if (done()) return;
        } while (DateTime.now().isBefore(end));
        fail(
          'Native side fixture deadline: host=${host?.error}; chat=${host?.visible.error}',
        );
      }

      final bodies = <Map>[];
      server = await t.runAsync(() async {
        final s = await HttpServer.bind(InternetAddress.loopbackIPv4, 0);
        s.listen((request) async {
          final body =
              jsonDecode(await utf8.decoder.bind(request).join()) as Map;
          bodies.add(body);
          final text = switch (bodies.length) {
            1 => 'Original shared excerpt. Unshared parent tail.',
            2 => 'Conclusion handles empty input. Unshared child tail.',
            _ => 'Parent inspected the shared conclusion.',
          };
          request.response.headers.contentType = ContentType(
            'text',
            'event-stream',
            charset: 'utf-8',
          );
          for (final event in [
            {
              'choices': [
                {
                  'index': 0,
                  'delta': {'content': text},
                  'finish_reason': null,
                },
              ],
            },
            {
              'choices': [
                {'index': 0, 'delta': {}, 'finish_reason': 'stop'},
              ],
            },
          ]) {
            request.response.write('data: ${jsonEncode(event)}\n\n');
          }
          request.response.write('data: [DONE]\n\n');
          await request.response.close();
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
      final project = Directory('${directory.parent.path}/side-public-project')
        ..createSync();
      final session =
          (await call('createSession', {
                'kind': 'project',
                'path': project.path,
              }))['session']['id']
              as String;
      final memory = await call('memories', {'session': session});
      if (memory['automaticPolicy']['enabled'] == true) {
        await call('setAutomaticMemory', {
          'enabled': false,
          'revision': memory['automaticPolicy']['revision'],
        });
      }
      final parent = ChatController(bridge, ownsBridge: false)..loading = false;
      await t.runAsync(parent.refresh);
      await t.runAsync(() => parent.select(session));
      host = AppHost(parent);
      viewport(t, const Size(1100, 900));
      await t.pumpWidget(DoloresApp(chat: parent, host: host));
      await t.pumpAndSettle();
      parent.draft = 'Original public task';
      await t.runAsync(parent.send);
      await wait(() => !parent.busy && parent.messages.length == 2);
      expect(parent.error, isNull);
      parent.draft = 'Keep private unsent parent draft 世界';
      await t.runAsync(parent.checkpointDraft);
      await call('editGoal', {
        'session': session,
        'edit': {
          'action': 'set',
          'revision': null,
          'objective': 'Retain the original task',
          'criteria': ['Inspect the conclusion'],
          'constraints': 'Preserve prior work',
        },
      });
      final originalGoal = await call('goal', {'session': session});
      Future<void> selectWord(
        String full,
        int start,
        int end,
        String action,
      ) async {
        final finder = find.text(full, findRichText: true);
        await t.ensureVisible(finder);
        final paragraph = t.renderObject<RenderParagraph>(finder);
        final box = paragraph
            .getBoxesForSelection(
              TextSelection(baseOffset: start, extentOffset: end),
            )
            .first
            .toRect();
        await t.longPressAt(paragraph.localToGlobal(box.center));
        await t.pumpAndSettle();
        await t.tap(find.text(action));
        await t.pump();
        await wait(
          () =>
              find.byKey(const Key('side-excerpt')).evaluate().isNotEmpty &&
              find.byType(LinearProgressIndicator).evaluate().isEmpty,
        );
      }

      await selectWord(
        'Original shared excerpt. Unshared parent tail.',
        9,
        15,
        'Ask about this',
      );
      expect(
        t
            .widget<TextField>(find.byKey(const Key('side-excerpt')))
            .controller!
            .text,
        'shared',
      );
      await t.enterText(
        find.byKey(const Key('side-question')),
        'Explain only this selection',
      );
      await t.tap(find.text('Open side chat'));
      await wait(
        () =>
            host!.visible != parent &&
            find.byKey(const Key('side-excerpt')).evaluate().isEmpty,
      );
      final child = host.visible;
      final childId = child.session!;
      expect(child.workspaceRoot, isNull);
      expect(child.messages, isEmpty);
      expect(child.draft, 'Explain only this selection');
      expect(bodies, hasLength(1));
      expect(parent.draft, 'Keep private unsent parent draft 世界');
      await t.tap(find.byKey(const Key('send')));
      await wait(() => !child.busy && child.messages.length == 2);
      expect(child.error, isNull);
      expect(bodies, hasLength(2));
      final sideContext = jsonEncode(bodies[1]['messages']);
      expect(sideContext, contains('shared'));
      expect(sideContext, isNot(contains('Unshared parent tail')));
      expect(sideContext, isNot(contains('Keep private unsent')));
      expect(sideContext, isNot(contains('Original public task')));
      await t.tap(find.byKey(const Key('side-return-parent')));
      await wait(() => host!.visible == parent);
      expect(parent.draft, 'Keep private unsent parent draft 世界');
      await t.runAsync(() => host!.select(childId));
      await t.pumpAndSettle();
      await selectWord(
        'Conclusion handles empty input. Unshared child tail.',
        0,
        10,
        'Bring to parent task',
      );
      expect(
        t
            .widget<TextField>(find.byKey(const Key('side-excerpt')))
            .controller!
            .text,
        'Conclusion',
      );
      expect(bodies, hasLength(2));
      await t.tap(find.text('Queue in parent'));
      await wait(
        () => host!.visible == parent && !parent.busy && bodies.length == 3,
      );
      expect(parent.error, isNull);
      final inputs = await call('taskInputs', {'session': session});
      expect(inputs['entries'], hasLength(1));
      expect(inputs['entries'][0]['state'], 'applied');
      expect(inputs['entries'][0]['text'], contains('Conclusion'));
      expect(
        inputs['entries'][0]['text'],
        isNot(contains('Unshared child tail')),
      );
      expect(
        await call('savedDraft', {'session': session}),
        'Keep private unsent parent draft 世界',
      );
      expect(await call('goal', {'session': session}), originalGoal);
      final link = await call('sideConversation', {
        'session': childId,
        'request': {'action': 'info'},
      });
      expect(link['origin']['returns'], hasLength(1));
      await call('delete', {'session': session});
      await t.runAsync(() => host!.select(childId));
      await t.pumpAndSettle();
      expect(child.sideConversation!['parentAvailable'], false);
      expect(
        t
            .widget<TextButton>(find.byKey(const Key('side-return-parent')))
            .onPressed,
        isNull,
      );
      expect(child.planning['references'][0]['text'], 'shared');
      File('${directory.parent.path}/side-conversations-receipt.json')
          .writeAsStringSync(
            jsonEncode({
              'scope': 'Rendered selection and normal native bridge with a public loopback provider; no physical desktop or real-model reliability claim',
              'providerRequests': bodies.length,
              'selectedContextOnly': true,
              'openingDoesNotRun': true,
              'separateOwners': true,
              'parentDraftAndGoalRetained': true,
              'conclusionAppliedOnce': true,
              'deletedParentRetainsExcerpt': true,
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
