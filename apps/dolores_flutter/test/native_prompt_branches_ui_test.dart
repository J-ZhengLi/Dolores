import 'dart:convert';
import 'dart:io';
import 'dart:ui' show PointerDeviceKind;

import 'package:dolores_flutter/app_host.dart';
import 'package:dolores_flutter/bridge.dart';
import 'package:dolores_flutter/chat.dart';
import 'package:dolores_flutter/main.dart';
import 'package:dolores_flutter/message_frame.dart';
import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';

import 'frost_design_test.dart' show viewport;

void main() => promptBranchFlow();

// Each native owner has process lifetime; run folder modes in separate files.
void promptBranchFlow({bool managed = false}) {
  testWidgets(
    'native edited prompt retains evidence in ${managed ? "a managed worktree" : "the shared folder"}',
    (t) async {
      final directory = Directory(Platform.environment['DOLORES_DATA_DIR']!);
      final output = Directory('../../output').absolute
          .resolveSymbolicLinksSync();
      expect(
        directory.parent.resolveSymbolicLinksSync(),
        startsWith('$output${Platform.pathSeparator}'),
      );
      final project = Directory(
        '${directory.parent.path}/branch-public-project-${managed ? "managed" : "shared"}',
      )..createSync();
      final marker = File('${project.path}/a.txt');
      Future<String> git(List<String> args) async {
        final result = await t.runAsync(
          () => Process.run('git', args, workingDirectory: project.path),
        );
        expect(result!.exitCode, 0, reason: '${result.stderr}');
        return '${result.stdout}';
      }

      if (managed) {
        await git(['init', '-b', 'main']);
        await git(['config', 'core.autocrlf', 'false']);
        await git(['config', 'user.email', 'fixture@example.invalid']);
        await git(['config', 'user.name', 'Public fixture']);
        marker.writeAsStringSync('Committed base bytes\n');
        await git(['add', 'a.txt']);
        await git(['commit', '-m', 'Public base']);
      }
      final bridge = (await t.runAsync(() async {
        final b = NativeBridge();
        await b.open(
          testLibrary: Platform.environment['DOLORES_AUDIT_LIBRARY'],
        );
        return b;
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
        final end = DateTime.now().add(const Duration(seconds: 30));
        do {
          await t.runAsync(
            () => Future<void>.delayed(const Duration(milliseconds: 25)),
          );
          await t.pump(const Duration(milliseconds: 100));
          if (done()) return;
        } while (DateTime.now().isBefore(end));
        fail(
          'Branch fixture deadline: host=${host?.error}; chat=${host?.visible.error}; approval=${host?.visible.toolApproval}; panel=${t.widgetList<SelectableText>(find.byType(SelectableText)).map((w) => w.data).join(" | ")}',
        );
      }

      final bodies = <Map>[];
      server = await t.runAsync(() async {
        final s = await HttpServer.bind(InternetAddress.loopbackIPv4, 0);
        s.listen((request) async {
          final body =
              jsonDecode(await utf8.decoder.bind(request).join()) as Map;
          bodies.add(body);
          final n = bodies.length;
          final tool = switch (n) {
            1 => managed ? 'edit_text_file' : 'create_text_file',
            5 => 'read_text_file',
            _ => null,
          };
          final args = n == 1
              ? managed
                    ? {
                        'path': 'a.txt',
                        'old_text': 'Committed base bytes\n',
                        'new_text': 'Original historical effect\n',
                      }
                    : {
                        'path': 'a.txt',
                        'content': 'Original historical effect\n',
                      }
              : {'path': 'a.txt'};
          final text = switch (n) {
            2 => 'First saved effect receipt.',
            3 => 'Old answer for the prompt being edited.',
            4 => 'Later original answer to keep archived.',
            _ => 'Inspected current files; no historical effect repeated.',
          };
          final delta = tool == null
              ? {'content': text}
              : {
                  'tool_calls': [
                    {
                      'index': 0,
                      'id': 'fixture-$n',
                      'type': 'function',
                      'function': {'name': tool, 'arguments': jsonEncode(args)},
                    },
                  ],
                };
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
                  'finish_reason': tool == null ? 'stop' : 'tool_calls',
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
      final parent = ChatController(bridge, ownsBridge: false)..loading = false;
      await t.runAsync(parent.refresh);
      await t.runAsync(() => parent.select(session));
      host = AppHost(parent);
      viewport(t, const Size(1100, 900));
      await t.pumpWidget(DoloresApp(chat: parent, host: host));
      await t.pumpAndSettle();
      parent.draft = 'Create the public marker a.txt once';
      await t.runAsync(parent.send);
      await wait(() => parent.toolApproval != null);
      await t.runAsync(() => parent.decideTool(true));
      await wait(
        () => !parent.busy && !parent.changing && parent.messages.length == 2,
      );
      expect(parent.error, isNull);
      expect(marker.readAsStringSync(), 'Original historical effect\n');
      parent.draft = 'Prompt to revise';
      await t.runAsync(parent.send);
      await wait(
        () => !parent.busy && !parent.changing && parent.messages.length == 4,
      );
      parent.draft = 'Later original prompt';
      await t.runAsync(parent.send);
      await wait(
        () => !parent.busy && !parent.changing && parent.messages.length == 6,
      );
      parent.draft = 'Keep newer parent draft 世界';
      await t.runAsync(parent.checkpointDraft);
      await call('editGoal', {
        'session': session,
        'edit': {
          'action': 'set',
          'revision': null,
          'objective': 'Retain original task',
          'criteria': ['Verify current file'],
          'constraints': 'Keep prior effects',
        },
      });
      final goal = await call('goal', {'session': session});
      String? stagedIndex;
      if (managed) {
        await git(['add', 'a.txt']);
        stagedIndex = await git(['ls-files', '--stage']);
      }
      final frame = find.byWidgetPredicate(
        (w) => w is MessageFrame && w.user && w.text == 'Prompt to revise',
      );
      await t.ensureVisible(frame);
      await t.pumpAndSettle();
      final mouse = await t.createGesture(kind: PointerDeviceKind.mouse);
      await mouse.addPointer(location: Offset.zero);
      final region = find
          .descendant(of: frame, matching: find.byType(MouseRegion))
          .first;
      await mouse.moveTo(t.getCenter(region));
      await t.pumpAndSettle();
      await t.tap(
        find.descendant(
          of: frame,
          matching: find.byTooltip('Edit in a new branch'),
        ),
      );
      await t.pump();
      await wait(
        () =>
            find.byKey(const Key('branch-edit')).evaluate().isNotEmpty &&
            find.byType(LinearProgressIndicator).evaluate().isEmpty,
      );
      expect(
        t
            .widget<TextField>(find.byKey(const Key('branch-edit')))
            .controller!
            .text,
        'Prompt to revise',
      );
      await t.enterText(
        find.byKey(const Key('branch-edit')),
        'Inspect a.txt as it is now; do not repeat the create',
      );
      await t.ensureVisible(find.byKey(const Key('branch-folder')));
      await t.tap(find.byKey(const Key('branch-folder')));
      await t.pumpAndSettle();
      await t.tap(
        find.text(managed ? 'Managed worktree' : 'Share current folder').last,
      );
      await t.pumpAndSettle();
      if (managed) {
        await t.ensureVisible(find.text('Choose or create worktree'));
        await t.tap(find.text('Choose or create worktree'));
        await t.pump();
        await wait(
          () =>
              find.text('Review creation').evaluate().isNotEmpty &&
              find.byType(LinearProgressIndicator).evaluate().isEmpty,
        );
        await t.enterText(
          find.ancestor(
            of: find.text('Base branch, tag or commit'),
            matching: find.byType(TextField),
          ),
          'main',
        );
        await t.pump();
        await t.ensureVisible(find.text('Review creation'));
        await t.tap(find.text('Review creation'));
        await wait(
          () =>
              find.text('Create worktree').evaluate().isNotEmpty &&
              find.byType(LinearProgressIndicator).evaluate().isEmpty,
        );
        await t.ensureVisible(find.text('Create worktree'));
        await t.pump();
        await t.tap(find.text('Create worktree'));
        await wait(
          () =>
              find.byType(LinearProgressIndicator).evaluate().isEmpty &&
              find.text('Create worktree').evaluate().isEmpty,
        );
        await t.scrollUntilVisible(
          find.text('Use for edited branch'),
          300,
          scrollable: find
              .descendant(
                of: find.byType(ListView).last,
                matching: find.byType(Scrollable),
              )
              .first,
        );
        await t.ensureVisible(find.text('Use for edited branch'));
        await t.pump();
        await t.tap(find.text('Use for edited branch'));
        await wait(
          () =>
              find.text('Choose another worktree').evaluate().isNotEmpty &&
              find.text('Use for edited branch').evaluate().isEmpty,
        );
      }
      marker.writeAsStringSync('Newer current file after preview 世界\n');
      expect(bodies, hasLength(4));
      await t.tap(find.text('Open edited branch'));
      await wait(
        () =>
            host!.visible != parent &&
            find.byKey(const Key('branch-edit')).evaluate().isEmpty,
      );
      final child = host.visible;
      final childId = child.session!;
      expect(child.messages, hasLength(2));
      expect(
        child.workspaceRoot,
        managed ? isNot(parent.workspaceRoot) : parent.workspaceRoot,
      );
      if (managed) {
        expect(
          File('${child.workspaceRoot}/a.txt').readAsStringSync(),
          'Committed base bytes\n',
        );
      }
      expect(child.goal, isNull);
      expect(child.inputEntries, isEmpty);
      expect(
        child.messages.last['metadata']['agent']['tools'][0]['status'],
        managed ? 'edited' : 'created',
      );
      expect(bodies, hasLength(4));
      expect(
        marker.readAsStringSync(),
        'Newer current file after preview 世界\n',
      );
      if (managed) {
        expect(await git(['ls-files', '--stage']), stagedIndex);
      }
      await t.tap(find.text('Original snapshot and receipts'));
      await t.pump();
      await wait(
        () => find
            .textContaining(
              'Later original answer to keep archived.',
              findRichText: true,
            )
            .evaluate()
            .isNotEmpty,
      );
      expect(
        find.textContaining('Original historical effect', findRichText: true),
        findsOneWidget,
      );
      expect(bodies, hasLength(4));
      await t.tap(find.text('Close'));
      await t.pumpAndSettle();
      await t.tap(find.byKey(const Key('send')));
      await wait(() => child.toolApproval != null);
      expect(child.toolApproval!['name'], 'read_text_file');
      await t.runAsync(() => child.decideTool(true));
      await wait(
        () => !child.busy && !child.changing && child.messages.length == 4,
      );
      expect(child.error, isNull);
      expect(bodies, hasLength(6));
      final branchContext = jsonEncode(bodies[4]['messages']);
      expect(
        branchContext,
        contains(
          managed ? 'SEPARATE managed worktree' : 'CURRENT working folder',
        ),
      );
      expect(branchContext, contains('First saved effect receipt.'));
      expect(
        branchContext,
        isNot(contains('Old answer for the prompt being edited.')),
      );
      expect(branchContext, isNot(contains('Later original answer')));
      expect(branchContext, isNot(contains('Keep newer parent draft')));
      expect(
        jsonEncode(bodies[5]['messages']),
        contains(
          managed ? 'Committed base bytes' : 'Newer current file after preview',
        ),
      );
      expect(
        marker.readAsStringSync(),
        'Newer current file after preview 世界\n',
      );
      expect(
        await call('savedDraft', {'session': session}),
        'Keep newer parent draft 世界',
      );
      expect(await call('goal', {'session': session}), goal);
      final original = await call('messagesPage', {'session': session});
      expect(original['items'], hasLength(6));
      final runs = await call('runs', {'session': childId});
      expect(runs, isNotNull);
      await call('delete', {'session': session});
      await t.runAsync(child.refreshPromptBranch);
      await t.pumpAndSettle();
      expect(child.promptBranch!['parentAvailable'], false);
      expect(
        t.widget<TextButton>(find.byKey(const Key('branch-parent'))).onPressed,
        isNull,
      );
      expect(
        (await call('promptBranch', {
          'session': childId,
          'request': {'action': 'archive', 'offset': 0},
        }))['text'],
        contains('Old answer for the prompt being edited.'),
      );
      File(
        '${directory.parent.path}/${managed ? "prompt-managed-branches" : "prompt-branches"}-receipt.json',
      ).writeAsStringSync(
        jsonEncode({
          'scope': 'Rendered controls and normal native bridge with public loopback provider; no real-model or physical desktop reliability claim',
          'providerRequests': bodies.length,
          'folder': managed ? 'managed' : 'shared',
          'originalToolAppliedOnce': true,
          'currentFilesInspected': true,
          'openingDoesNotSend': true,
          'originalTranscriptAndReceiptsRetained': true,
          'noHistoricalReplay': true,
          'parentDraftGoalAndHistoryRetained': true,
          'deletedParentArchiveReadable': true,
          'passed': true,
        }),
      );
      await mouse.removePointer();
      expect(t.takeException(), isNull);
    },
    skip: Platform.environment['DOLORES_AUDIT_LIBRARY'] == null,
  );
}
