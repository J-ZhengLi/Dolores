import 'dart:convert';
import 'dart:io';

import 'package:dolores_flutter/bridge.dart';
import 'package:dolores_flutter/chat.dart';
import 'package:dolores_flutter/git_host.dart';
import 'package:dolores_flutter/main.dart';
import 'package:dolores_flutter/source_control.dart';
import 'package:dolores_flutter/theme.dart';
import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';

import 'frost_design_test.dart' show viewport;

void main() {
  testWidgets(
    'native diff finding queues a reviewed repair, retains draft and resolves later evidence',
    (t) async {
      final directory = Directory(Platform.environment['DOLORES_DATA_DIR']!);
      final output = Directory('../../output').absolute
          .resolveSymbolicLinksSync();
      expect(
        directory.parent.resolveSymbolicLinksSync(),
        startsWith('$output${Platform.pathSeparator}'),
      );
      final project = Directory(
        '${directory.parent.path}/review-public-project',
      )..createSync();
      Future<void> gitCommand(List<String> args) async {
        final result = (await t.runAsync(
          () => Process.run('git', args, workingDirectory: project.path),
        ))!;
        expect(result.exitCode, 0, reason: 'Public fixture Git setup failed');
      }

      for (final args in [
        ['init', '-b', 'main'],
        ['config', 'user.name', 'Fixture'],
        ['config', 'user.email', 'fixture@example.invalid'],
        ['config', 'commit.gpgSign', 'false'],
        ['config', 'core.autocrlf', 'false'],
      ]) {
        await gitCommand(args);
      }
      File('${project.path}/a.txt').writeAsStringSync('original\n');
      await gitCommand(['add', '.']);
      await gitCommand(['commit', '-m', 'Public review fixture']);
      File('${project.path}/a.txt').writeAsStringSync('needs repair\n');
      final index = File('${project.path}/.git/index').readAsBytesSync();
      final bridge = (await t.runAsync(() async {
        final b = NativeBridge();
        await b.open(
          testLibrary: Platform.environment['DOLORES_AUDIT_LIBRARY'],
        );
        return b;
      }))!;
      ChatController? owner;
      GitHost? git;
      HttpServer? server;
      addTearDown(() async {
        await t.pumpWidget(const SizedBox());
        git?.dispose();
        owner?.dispose();
        await t.runAsync(() async {
          await server?.close(force: true);
          await bridge.close();
        });
      });
      Future<dynamic> call(
        String name, [
        Map<String, dynamic> fields = const {},
      ]) async {
        final r = (await t.runAsync(() async {
          try {
            return {
              'value': await bridge.call({'command': name, ...fields}),
            };
          } catch (e) {
            return {'error': '$e'};
          }
        }))!;
        if (r['error'] != null) throw StateError(r['error'] as String);
        return r['value'];
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
        final selected = git?.selected;
        final state = selected == null ? null : git?.reviewState(selected);
        fail(
          'Native review fixture deadline exceeded: git=${selected?.error}; review=${state?.error}; pending=${state?.pending}; chat=${owner?.error}',
        );
      }

      Future<void> tap(String label) async {
        final f = find.text(label).last;
        await t.ensureVisible(f);
        await t.tap(f);
        await t.pump();
      }

      final bodies = <Map>[];
      server = await t.runAsync(() async {
        final s = await HttpServer.bind(InternetAddress.loopbackIPv4, 0);
        s.listen((request) async {
          final body =
              jsonDecode(await utf8.decoder.bind(request).join()) as Map;
          bodies.add(body);
          final first = bodies.length == 1;
          final delta = first
              ? {
                  'tool_calls': [
                    {
                      'index': 0,
                      'id': 'public-repair',
                      'type': 'function',
                      'function': {
                        'name': 'edit_text_file',
                        'arguments': jsonEncode({
                          'path': 'a.txt',
                          'old_text': 'needs repair',
                          'new_text': 'repaired empty input',
                        }),
                      },
                    },
                  ],
                }
              : {'content': 'Inspected repair complete.'};
          request.response.headers.contentType = ContentType(
            'text',
            'event-stream',
            charset: 'utf-8',
          );
          for (final event in [
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
      final chat = owner = ChatController(bridge, ownsBridge: false)
        ..loading = false;
      await t.runAsync(chat.refresh);
      await t.runAsync(() => chat.select(session));
      chat.draft = 'Retained unsent source draft 世界';
      await t.runAsync(chat.checkpointDraft);
      final host = git = GitHost(bridge, reviewAvailable: true);
      host.afterReviewSubmission = (id) async {
        expect(id, session);
        await chat.refreshInputs();
        await chat.dispatchNextInput();
      };
      await t.runAsync(() => host.bind(session, project.path));
      final w = host.selected!;
      await t.runAsync(() => host.openDiff(w, 'a.txt', 'working'));
      viewport(t, const Size(1000, 900));
      Future<void> showDiff() async {
        await t.pumpWidget(
          MaterialApp(
            theme: doloresTheme(false),
            home: Scaffold(body: SourceControlView(git: host)),
          ),
        );
        await t.pumpAndSettle();
      }

      await showDiff();
      await t.tap(find.byKey(const ValueKey('review-right-1')));
      await t.pumpAndSettle();
      await t.enterText(
        find.byKey(const Key('review-note')),
        'Repair the empty-input behavior and verify a.txt.',
      );
      await wait(() => !host.reviewState(w).loading && !w.busy);
      await tap('Save finding');
      await t.pumpAndSettle();
      if (find.byKey(const Key('review-note')).evaluate().isNotEmpty &&
          host.reviewState(w).pending == null) {
        final texts = find
            .byType(SelectableText)
            .evaluate()
            .map((e) => (e.widget as SelectableText).data)
            .whereType<String>()
            .toList();
        fail(
          'Capture did not start: reviewLoading=${host.reviewState(w).loading}; busy=${w.busy}; editor=$texts',
        );
      }
      await wait(
        () =>
            host.reviewState(w).items.length == 1 &&
            !w.busy &&
            find.byKey(const Key('review-note')).evaluate().isEmpty,
      );
      expect(bodies, isEmpty);
      expect(find.textContaining('Review: Repair'), findsOneWidget);
      final anchor = Map<String, dynamic>.from(
        host.reviewState(w).items.values.single['anchor']['comparison'],
      );
      await tap('Review findings');
      await wait(() => !host.reviewState(w).loading);
      await t.tap(find.byType(Checkbox).first);
      await t.pump();
      await tap('Queue selected');
      await wait(() => host.reviewState(w).pending == null && chat.busy);
      await tap('Close');
      await t.pumpAndSettle();
      await t.pumpWidget(DoloresApp(chat: chat));
      await t.pump();
      await wait(
        () =>
            chat.toolApproval != null &&
            find.byKey(const Key('allow-tool')).evaluate().isNotEmpty,
      );
      await t.ensureVisible(find.byKey(const Key('allow-tool')));
      await t.tap(find.byKey(const Key('allow-tool')));
      await t.pump();
      await wait(() => bodies.length == 2 && !chat.busy && !chat.changing);
      expect(chat.error, isNull);
      expect(chat.draft, 'Retained unsent source draft 世界');
      expect(
        File('${project.path}/a.txt').readAsStringSync(),
        'repaired empty input\n',
      );
      expect(File('${project.path}/.git/index').readAsBytesSync(), index);
      expect(
        jsonEncode(bodies.first['messages']),
        contains('Repair the empty-input behavior'),
      );
      expect(
        jsonEncode(bodies),
        isNot(contains('Retained unsent source draft')),
      );
      final saved = await call('reviewFindings', {
        'session': session,
        'request': {'action': 'list'},
      });
      expect(saved['items'][0]['resolution'], isNull);
      final inputs = await call('taskInputs', {'session': session});
      expect(inputs['entries'].length, 1);
      expect(inputs['entries'][0]['state'], 'applied');
      await t.runAsync(() => host.refreshDiff(w));
      await showDiff();
      await tap('Review findings');
      await wait(() => !host.reviewState(w).loading);
      expect(find.textContaining('Location changed'), findsOneWidget);
      await tap('Resolve after inspection');
      await t.enterText(
        find.byKey(const Key('resolution-note')),
        'Verified native edit receipt and exact saved file bytes',
      );
      await tap('Save resolution');
      await wait(
        () =>
            !w.busy &&
            host.reviewState(w).items.values.single['resolution'] != null,
      );
      final result = await call('reviewFindings', {
        'session': session,
        'request': {'action': 'list'},
      });
      expect(result['items'][0]['anchor']['comparison'], anchor);
      expect(
        result['items'][0]['resolution']['comparison']['digest'],
        isNot(anchor['digest']),
      );
      await t.runAsync(() async {
        await File('${directory.parent.path}/review-findings-receipt.json')
            .writeAsString(
              jsonEncode({
                'scope': 'Rendered controls and normal bridge, public loopback fixture; no real-model or physical input claim',
                'capturedHostLine': true,
                'submittedOnce': inputs['entries'].length == 1,
                'repairToolApplied': true,
                'sourceIndexRetained': true,
                'draftRetained': true,
                'resolutionRequiresLaterComparison': true,
                'passed': true,
              }),
            );
      });
    },
    skip: Platform.environment['DOLORES_AUDIT_LIBRARY'] == null,
  );
}
