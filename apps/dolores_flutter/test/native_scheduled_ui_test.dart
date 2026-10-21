import 'dart:convert';
import 'dart:io';

import 'package:dolores_flutter/bridge.dart';
import 'package:dolores_flutter/scheduled_host.dart';
import 'package:dolores_flutter/scheduled_page.dart';
import 'package:dolores_flutter/theme.dart';
import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';

void main() {
  testWidgets(
    'real schedule deletion retains source results and project files',
    (tester) async {
      final directory = Directory(Platform.environment['DOLORES_DATA_DIR']!);
      final output = Directory('../../output').absolute
          .resolveSymbolicLinksSync();
      expect(
        directory.parent.resolveSymbolicLinksSync(),
        startsWith('$output${Platform.pathSeparator}'),
      );
      final project = Directory('${directory.parent.path}/schedule-project')
        ..createSync(recursive: true);
      final file = File('${project.path}/public.txt')
        ..writeAsStringSync('PUBLIC-SCHEDULE-42\n中文\n');
      final bytes = file.readAsBytesSync();
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
          await tester.pump();
          if (ready()) return;
        } while (DateTime.now().isBefore(end));
        fail('Scheduled UI did not finish its bounded operation');
      }

      Future<Map> finish(int id) async {
        final end = DateTime.now().add(const Duration(seconds: 15));
        do {
          for (final event in await call('poll', {'id': id}) as List) {
            if (event['type'] == 'done') return event as Map;
            expect(
              event['type'],
              isNot('toolApproval'),
              reason: 'Public fixture does not request file operations',
            );
          }
          await tester.runAsync(
            () => Future<void>.delayed(const Duration(milliseconds: 25)),
          );
        } while (DateTime.now().isBefore(end));
        fail('Scheduled fixture run did not complete');
      }

      var requests = 0;
      var failOccurrence = true;
      final server = (await tester.runAsync(() async {
        final server = await HttpServer.bind(InternetAddress.loopbackIPv4, 0);
        server.listen((request) async {
          final body = jsonDecode(await utf8.decoder.bind(request).join());
          requests++;
          final messages = body['messages'] as List;
          final names = (body['tools'] as List? ?? []).map(
            (t) => t['function']['name'],
          );
          final creation = names.contains('schedule_task');
          if (!creation && failOccurrence) {
            request.response.statusCode = 503;
            await request.response.close();
            return;
          }
          final Map<String, dynamic> delta;
          final String reason;
          if (creation && messages.last['role'] == 'user') {
            delta = {
              'tool_calls': [
                {
                  'index': 0,
                  'id': 'public-schedule',
                  'type': 'function',
                  'function': {
                    'name': 'schedule_task',
                    'arguments': jsonEncode({
                      'rule': {
                        'kind': 'daily',
                        'time': '21:00',
                        'date': null,
                        'weekdays': [],
                        'zone': 'Asia/Shanghai',
                      },
                      'skill': null,
                      'model': null,
                    }),
                  },
                },
              ],
            };
            reason = 'tool_calls';
          } else {
            delta = {
              'content': creation
                  ? 'Task saved. Results appear in Scheduled.'
                  : 'PUBLIC-SCHEDULE-RESULT-42',
            };
            reason = 'stop';
          }
          request.response.headers.contentType = ContentType(
            'text',
            'event-stream',
          );
          for (final event in [
            {
              'choices': [
                {'index': 0, 'delta': delta, 'finish_reason': null},
              ],
            },
            {
              'choices': [
                {'index': 0, 'delta': {}, 'finish_reason': reason},
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
      final source = (await call('createSession', {
        'kind': 'project',
        'path': project.path,
      }))['session']['id'];
      var memories = await call('memories', {'session': source});
      if (memories['automaticPolicy']['enabled'] == true) {
        await call('setAutomaticMemory', {
          'enabled': false,
          'revision': memories['automaticPolicy']['revision'],
        });
      }
      await call('start', {
        'id': 32021,
        'session': source,
        'input': 'Every day at 9pm, write a short public report. Only create the scheduled task now.',
      });
      expect((await finish(32021))['error'], isNull);
      final listed = (await call('scheduledTasks'))['items'] as List;
      expect(listed, hasLength(1));
      final task = listed.single['task'];
      final claimed = (await call('scheduledManage', {
        'task': task['id'],
        'revision': task['revision'],
        'action': 'runNow',
      }))['claimed'][0];
      await call('scheduledStart', {'occurrence': claimed['id'], 'id': 32022});
      expect((await finish(32022))['error'], isNotNull);
      await call('scheduledTick');
      final failedListing = (await call('scheduledTasks'))['items'] as List;
      expect(failedListing.single['occurrences'][0]['state'], 'failed');
      failOccurrence = false;
      var retries = 0;
      final host = ScheduledHost(
        bridge,
        onClaim: (occurrence) async {
          retries++;
          await bridge.call({
            'command': 'scheduledStart',
            'occurrence': occurrence['id'],
            'id': 32023,
          });
        },
      );
      await tester.runAsync(host.refresh);
      host.suspend();
      tester.view.physicalSize = const Size(720, 820);
      tester.view.devicePixelRatio = 1;
      addTearDown(tester.view.resetPhysicalSize);
      addTearDown(tester.view.resetDevicePixelRatio);
      String? opened;
      await tester.pumpWidget(
        MaterialApp(
          theme: doloresTheme(true),
          home: Scaffold(
            body: ScheduledPage(
              host: host,
              selected: task['id'],
              owners: [],
              openResult: (s) async {
                opened = s;
              },
            ),
          ),
        ),
      );
      await tester.pumpAndSettle();
      expect(find.text('Failed'), findsOneWidget);
      final beforeRetry = requests;
      await tester.pump(const Duration(seconds: 2));
      expect(requests, beforeRetry);
      await tester.tap(find.byTooltip('Task actions'));
      await tester.pumpAndSettle();
      await tester.tap(find.text('Run now'));
      await waitUntil(() => !host.pending);
      expect(host.error, isNull);
      expect(retries, 1);
      expect((await finish(32023))['error'], isNull);
      await call('scheduledTick');
      await tester.runAsync(host.refresh);
      await tester.pumpAndSettle();
      final history = (host.items.single['occurrences'] as List).cast<Map>();
      expect(history.any((o) => o['state'] == 'failed'), isTrue);
      expect(history.any((o) => o['state'] == 'succeeded'), isTrue);
      final sourceMessages = (await call('messagesPage', {
        'session': source,
      }))['items'];
      final resultMessages = <String, dynamic>{};
      for (final o in history) {
        if (o['session'] != null) {
          resultMessages[o['session']] = (await call('messagesPage', {
            'session': o['session'],
          }))['items'];
        }
      }
      await tester.ensureVisible(find.text('Open result').first);
      await tester.tap(find.text('Open result').first);
      expect(resultMessages.containsKey(opened), isTrue);
      await tester.drag(
        find.byType(SingleChildScrollView).first,
        const Offset(0, 10000),
      );
      await tester.pumpAndSettle();
      Future<void> deleteMenu() async {
        await tester.tap(find.byTooltip('Task actions'));
        await tester.pumpAndSettle();
        await tester.tap(find.text('Delete task'));
        await tester.pumpAndSettle();
      }

      await deleteMenu();
      expect(find.textContaining('Saved results remain'), findsOneWidget);
      await tester.tap(find.text('Keep task'));
      await tester.pumpAndSettle();
      expect(host.items, hasLength(1));
      await deleteMenu();
      await tester.tap(find.widgetWithText(TextButton, 'Delete'));
      await waitUntil(() => !host.pending);
      expect(host.error, isNull);
      expect(host.items, isEmpty);
      expect(
        find.text('Ask Dolores in a chat to schedule a task.'),
        findsOneWidget,
      );
      expect(
        (await call('messagesPage', {'session': source}))['items'],
        sourceMessages,
      );
      for (final entry in resultMessages.entries) {
        expect(
          (await call('messagesPage', {'session': entry.key}))['items'],
          entry.value,
        );
      }
      expect(file.readAsBytesSync(), bytes);
      final beforeTick = requests;
      expect((await call('scheduledTick'))['claimed'], isEmpty);
      expect(requests, beforeTick);
      File('${directory.parent.path}/schedule-receipt.json').writeAsStringSync(
        const JsonEncoder.withIndent('  ').convert({
          'mode': 'Rendered controls with normal Rust bridge and loopback provider; no real-model/native-pointer claim',
          'task': task['id'],
          'failedHistoryRetained': true,
          'explicitRunNow': retries,
          'retainedSource': source,
          'retainedResults': resultMessages.keys.toList(),
          'keepCancelledDeletion': true,
          'deleteRetainedFiles': true,
          'noRunAfterDelete': true,
          'requests': requests,
        }),
      );
      expect(tester.takeException(), isNull);
      await tester.pumpWidget(const SizedBox());
      host.dispose();
      await tester.runAsync(() => server.close(force: true));
      await tester.runAsync(bridge.close);
    },
    skip:
        !Platform.isWindows ||
        Platform.environment['DOLORES_AUDIT_LIBRARY'] == null,
  );
}
