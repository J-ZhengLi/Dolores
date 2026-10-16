import 'dart:io';
import 'dart:convert';

import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:dolores_flutter/bridge.dart';
import 'package:dolores_flutter/instructions.dart';
import 'package:dolores_flutter/theme.dart';
import 'package:dolores_flutter/memory.dart';

Future<void> settled(WidgetTester tester) async {
  final deadline = DateTime.now().add(const Duration(seconds: 10));
  do {
    await tester.runAsync(
      () => Future<void>.delayed(const Duration(milliseconds: 25)),
    );
    await tester.pump();
    if (find.byType(LinearProgressIndicator).evaluate().isEmpty) return;
  } while (DateTime.now().isBefore(deadline));
  fail('Native instruction operation did not settle');
}

void main() {
  testWidgets(
    'native instructions and Memory controls preserve scope and usable work',
    (tester) async {
      final directory = Directory(Platform.environment['DOLORES_DATA_DIR']!);
      final output = Directory('../../output').absolute
          .resolveSymbolicLinksSync();
      expect(
        directory.parent.resolveSymbolicLinksSync(),
        startsWith('$output${Platform.pathSeparator}'),
      );
      final project = Directory('${directory.parent.path}/project')
        ..createSync(recursive: true);
      final file = File('${project.path}/AGENTS.md');
      file.writeAsStringSync('Use focused tests.\n@../public-reference.txt\n');
      File('${directory.parent.path}/public-reference.txt')
          .writeAsStringSync('NOT-AUTOMATICALLY-INCLUDED');
      final bridge = (await tester.runAsync(() async {
        final bridge = NativeBridge();
        await bridge.open(
          testLibrary: Platform.environment['DOLORES_AUDIT_LIBRARY'],
        );
        return bridge;
      }))!;
      final receipts = <String, dynamic>{};
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

      await call('bootstrap');
      await call('configure', {
        'preferences': {'baseUrl': 'http://127.0.0.1:1/v1', 'model': 'fixture'},
        'apiKey': '',
        'rememberConnection': true,
      });
      final session = (await call('createSession', {
        'kind': 'project',
        'path': project.path,
      }))['session']['id'];
      await call('saveDraft', {
        'session': session,
        'text': 'Retain public draft',
      });
      tester.view.physicalSize = const Size(620, 780);
      tester.view.devicePixelRatio = 1;
      addTearDown(tester.view.resetPhysicalSize);
      addTearDown(tester.view.resetDevicePixelRatio);
      await tester.pumpWidget(
        MaterialApp(
          theme: doloresTheme(true),
          home: InstructionsInspector(bridge: bridge, session: session),
        ),
      );
      await settled(tester);
      expect(find.text('Not enabled'), findsOneWidget);
      expect(
        tester
            .widget<SelectableText>(
              find.byKey(const Key('instruction-source-text')),
            )
            .data,
        'Use focused tests.\n@../public-reference.txt\n',
      );
      file.writeAsStringSync(
        'Changed public guidance\n@../public-reference.txt\n',
      );
      await tester.tap(find.byKey(const Key('enable-instructions')));
      await settled(tester);
      expect(find.textContaining('changed after review'), findsOneWidget);
      expect(
        tester
            .widget<FilledButton>(find.byKey(const Key('enable-instructions')))
            .onPressed,
        isNull,
      );
      await tester.tap(find.byKey(const Key('refresh-instructions')));
      await settled(tester);
      await tester.tap(find.byKey(const Key('enable-instructions')));
      await settled(tester);
      expect(find.text('Enabled'), findsOneWidget);
      final context = await call('context', {
        'session': session,
        'input': 'Public hello',
      });
      expect(
        jsonEncode(context['messages']),
        contains('Changed public guidance'),
      );
      expect(
        jsonEncode(context['messages']),
        contains('@../public-reference.txt'),
      );
      expect(
        jsonEncode(context),
        isNot(contains('NOT-AUTOMATICALLY-INCLUDED')),
      );
      receipts['enabledContext'] = context;
      file.writeAsStringSync('New public guidance');
      Object? changed;
      try {
        await call('context', {'session': session, 'input': 'Public hello'});
      } catch (e) {
        changed = e;
      }
      expect(changed.toString(), contains('need review'));
      receipts['changedBlocked'] = changed.toString();
      await tester.tap(find.byKey(const Key('refresh-instructions')));
      await settled(tester);
      expect(find.text('Needs review'), findsOneWidget);
      await tester.tap(find.byKey(const Key('enable-instructions')));
      await settled(tester);
      expect(find.text('Enabled'), findsOneWidget);
      file.deleteSync();
      Object? missing;
      try {
        await call('context', {'session': session, 'input': 'Public hello'});
      } catch (e) {
        missing = e;
      }
      expect(missing.toString(), contains('need review'));
      receipts['missingBlocked'] = missing.toString();
      await tester.tap(find.byKey(const Key('refresh-instructions')));
      await settled(tester);
      expect(find.text('Needs review'), findsOneWidget);
      expect(
        tester
            .widget<FilledButton>(find.byKey(const Key('enable-instructions')))
            .onPressed,
        isNull,
      );
      await tester.tap(find.byKey(const Key('disable-instructions')));
      await settled(tester);
      expect(find.text('Not enabled'), findsOneWidget);
      final disabled = await call('context', {
        'session': session,
        'input': 'Public hello',
      });
      expect(disabled['instructions'], isNull);
      expect(
        await call('savedDraft', {'session': session}),
        'Retain public draft',
      );
      receipts['disabledContext'] = disabled;
      final memory = await call('saveMemory', {
        'session': session,
        'scope': 'all',
        'title': 'Public marker',
        'text': 'The public marker is PUBLIC-MEMORY-42.',
        'enabled': true,
      });
      var memories = await call('memories', {'session': session});
      if (memories['automaticPolicy']['enabled'] != true) {
        await call('setAutomaticMemory', {
          'enabled': true,
          'revision': memories['automaticPolicy']['revision'],
        });
      }
      final recalled = await call('context', {
        'session': session,
        'input': 'Which public marker is saved?',
      });
      expect(jsonEncode(recalled['messages']), contains('PUBLIC-MEMORY-42'));
      await tester.pumpWidget(
        MaterialApp(
          theme: doloresTheme(false),
          home: MemoryInspector(bridge: bridge, session: session),
        ),
      );
      await settled(tester);
      final toggle = find.byKey(const Key('automatic-memory'));
      await tester.ensureVisible(toggle);
      await tester.tap(toggle);
      await settled(tester);
      expect(tester.widget<SwitchListTile>(toggle).value, isFalse);
      final notRecalled = await call('context', {
        'session': session,
        'input': 'Which public marker is saved?',
      });
      expect(
        jsonEncode(notRecalled['messages']),
        isNot(contains('PUBLIC-MEMORY-42')),
      );
      expect(
        (await call('memories', {'session': session}))['items'],
        hasLength(1),
      );
      final body = find.byKey(Key('memory-body-${memory['id']}'));
      await tester.scrollUntilVisible(
        body,
        150,
        scrollable: find.byType(Scrollable).first,
      );
      expect(
        tester.widget<SelectableText>(body).data,
        'The public marker is PUBLIC-MEMORY-42.',
      );
      var requests = 0;
      final server = (await tester.runAsync(() async {
        final server = await HttpServer.bind(InternetAddress.loopbackIPv4, 0);
        server.listen((request) async {
          await utf8.decoder.bind(request).join();
          requests++;
          request.response.headers.contentType = ContentType(
            'text',
            'event-stream',
          );
          request.response.write(
            'data: ${jsonEncode({
              'choices': [
                {
                  'index': 0,
                  'delta': {'content': 'Completed public fixture task.'},
                  'finish_reason': null,
                },
              ],
            })}\n\n',
          );
          request.response.write(
            'data: ${jsonEncode({
              'choices': [
                {'index': 0, 'delta': {}, 'finish_reason': 'stop'},
              ],
            })}\n\ndata: [DONE]\n\n',
          );
          await request.response.close();
        });
        return server;
      }))!;
      await call('configure', {
        'preferences': {
          'baseUrl': 'http://127.0.0.1:${server.port}/v1',
          'model': 'fixture',
        },
        'apiKey': '',
        'rememberConnection': true,
      });
      await call('start', {
        'id': 31017,
        'session': session,
        'input': 'Complete a public fixture task while memory is off.',
      });
      var done = false;
      final deadline = DateTime.now().add(const Duration(seconds: 10));
      while (!done && DateTime.now().isBefore(deadline)) {
        final events = await call('poll', {'id': 31017}) as List;
        for (final event in events) {
          if (event['type'] == 'done') {
            expect(event['error'], isNull);
            done = true;
          }
        }
        await tester.runAsync(
          () => Future<void>.delayed(const Duration(milliseconds: 25)),
        );
      }
      expect(done, isTrue);
      await tester.runAsync(
        () => Future<void>.delayed(const Duration(milliseconds: 100)),
      );
      expect(requests, 1, reason: 'Memory Off must not start extraction');
      await tester.runAsync(() => server.close(force: true));
      final history = (await call('messagesPage', {
        'session': session,
      }))['items'];
      expect(history, hasLength(2));
      await call('saveDraft', {
        'session': session,
        'text': 'Keep the next public question',
      });
      final forget = find.byKey(Key('delete-memory-${memory['id']}'));
      await tester.ensureVisible(forget);
      await tester.tap(forget);
      await settled(tester);
      expect(
        find.text('Memory forgotten; your conversations are unchanged.'),
        findsOneWidget,
      );
      expect((await call('memories', {'session': session}))['items'], isEmpty);
      expect(
        (await call('messagesPage', {'session': session}))['items'],
        history,
      );
      expect(
        await call('savedDraft', {'session': session}),
        'Keep the next public question',
      );
      receipts['memoryOff'] = {
        'recalledBeforeOff': true,
        'excludedAfterOff': true,
        'inspectableWhileOff': true,
        'completedTaskRequests': requests,
        'forgotten': true,
        'retainedMessages': history.length,
        'draftRetained': true,
      };
      receipts['mode'] = 'Rendered Flutter controls with actual normal Rust bridge; one loopback fixture request, zero external providers; no native pointer claim';
      File(
        '${directory.parent.path}/receipt.json',
      ).writeAsStringSync(const JsonEncoder.withIndent('  ').convert(receipts));
      expect(tester.takeException(), isNull);
      await tester.pumpWidget(const SizedBox());
      await tester.runAsync(bridge.close);
    },
    skip:
        !Platform.isWindows ||
        Platform.environment['DOLORES_AUDIT_LIBRARY'] == null,
  );
}
