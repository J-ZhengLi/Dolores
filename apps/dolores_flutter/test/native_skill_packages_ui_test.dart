import 'dart:convert';
import 'dart:io';

import 'package:dolores_flutter/app_host.dart';
import 'package:dolores_flutter/bridge.dart';
import 'package:dolores_flutter/chat.dart';
import 'package:dolores_flutter/main.dart';
import 'package:dolores_flutter/skill_packages.dart';
import 'package:dolores_flutter/theme.dart';
import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';

void main() {
  testWidgets(
    'native Project and Global package resources survive updates, recovery and rollback',
    (t) async {
      final data = Directory(Platform.environment['DOLORES_DATA_DIR']!);
      final base = data.parent;
      final output = Directory('../../output').absolute
          .resolveSymbolicLinksSync();
      expect(
        base.resolveSymbolicLinksSync(),
        startsWith('$output${Platform.pathSeparator}'),
      );
      final project = Directory('${base.path}/package-project')..createSync();
      final bridge = (await t.runAsync(() async {
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
        final result = (await t.runAsync(() async {
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

      Future<void> waitUntil(bool Function() done) async {
        final end = DateTime.now().add(const Duration(seconds: 25));
        do {
          await t.runAsync(
            () => Future<void>.delayed(const Duration(milliseconds: 25)),
          );
          await t.pump(const Duration(milliseconds: 80));
          if (done()) return;
        } while (DateTime.now().isBefore(end));
        fail('Package native fixture exceeded its bounded wait.');
      }

      Future<void> press(String key) async {
        final target = find.byKey(Key(key));
        if (target.evaluate().isEmpty) {
          await t.scrollUntilVisible(
            target,
            160,
            scrollable: find
                .descendant(
                  of: find.byKey(const Key('packages-scroll')),
                  matching: find.byType(Scrollable),
                )
                .first,
          );
        }
        await t.ensureVisible(target);
        await t.pump();
        await t.tap(target);
        if (key == 'remove-package' ||
            key.startsWith('forget-package-version-')) {
          await t.pump();
          await t.pump(const Duration(milliseconds: 300));
          return;
        }
        await waitUntil(
          () => find.byType(LinearProgressIndicator).evaluate().isEmpty,
        );
        await t.pumpAndSettle();
      }

      var toolStep = 0;
      final bodies = <Map>[];
      final server = (await t.runAsync(() async {
        final server = await HttpServer.bind(InternetAddress.loopbackIPv4, 0);
        server.listen((request) async {
          final body =
              jsonDecode(await utf8.decoder.bind(request).join()) as Map;
          bodies.add(body);
          final step = toolStep++ % 4;
          final system = (body['messages'] as List).first['content'] as String;
          final identityText = RegExp(
            r'Host-pinned package identity: (\{[^\n]+\})',
          ).firstMatch(system)?.group(1);
          final identity = identityText == null
              ? <String, dynamic>{}
              : (jsonDecode(identityText) as Map).cast<String, dynamic>();
          final args = {
            ...identity,
            'action': step == 0 ? 'list' : 'read',
            'maxBytes': 2048,
            if (step == 1) 'path': 'references/check.md',
            if (step == 2) 'path': 'assets/image.bin',
          };
          final delta = step < 3
              ? {
                  'tool_calls': [
                    {
                      'index': 0,
                      'id': 'resource-${bodies.length}',
                      'type': 'function',
                      'function': {
                        'name': 'read_skill_resource',
                        'arguments': jsonEncode(args),
                      },
                    },
                  ],
                }
              : {
                  'content': 'Verified public package reference and binary metadata. No scripts executed.',
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
                  'finish_reason': step < 3 ? 'tool_calls' : 'stop',
                },
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
      AppHost? host;
      ChatController? ownedChat;
      addTearDown(() async {
        host?.dispose();
        if (host == null) ownedChat?.dispose();
        await server.close(force: true);
        await bridge.close();
      });
      await call('configure', {
        'preferences': {
          'baseUrl': 'http://127.0.0.1:${server.port}/v1',
          'model': 'fixture',
        },
        'apiKey': '',
        'rememberConnection': true,
      });
      final receipts = <Map>[];
      t.view.physicalSize = const Size(1000, 900);
      t.view.devicePixelRatio = 1;
      addTearDown(t.view.resetPhysicalSize);
      addTearDown(t.view.resetDevicePixelRatio);
      for (final scope in ['project', 'global']) {
        final session =
            (await call('createSession', {
                  'kind': scope == 'project' ? 'project' : 'side',
                  if (scope == 'project') 'path': project.path,
                }))['session']['id']
                as String;
        final memory = await call('memories', {'session': session});
        if (memory['automaticPolicy']['enabled'] == true) {
          await call('setAutomaticMemory', {
            'enabled': false,
            'revision': memory['automaticPolicy']['revision'],
          });
        }
        Future<dynamic> packages(
          String action, [
          Map<String, dynamic> fields = const {},
        ]) => call('skillPackages', {
          'session': session,
          'scope': scope,
          'request': {'action': action, ...fields},
        });
        var source = '${base.path}/review-v1.zip';
        var exported = '${base.path}/$scope-export-v1.zip';
        Future<void> openPackages() async {
          await t.pumpWidget(
            MaterialApp(
              theme: doloresTheme(false),
              home: Scaffold(
                body: SkillPackages(
                  key: ValueKey(scope),
                  bridge: bridge,
                  session: session,
                  initialScope: scope,
                  hasProject: scope == 'project',
                  onBack: () {},
                  chooseArchive: () async => source,
                  chooseExport: (_) async => exported,
                ),
              ),
            ),
          );
          await waitUntil(
            () => find.byType(LinearProgressIndicator).evaluate().isEmpty,
          );
        }

        await openPackages();
        Future<void> import() async {
          await press('import-package');
          await t.enterText(find.byKey(const Key('package-name')), 'review');
          await press('choose-package-zip');
          await press('review-package-zip');
          await press('install-package');
        }

        await import();
        expect(
          (await call('projectSkills', {
            'session': session,
            'scope': scope,
          }))['items'],
          isEmpty,
        );
        final p1 = (await packages('list'))['installed'][0] as Map;
        final id1 = p1['package']['id'] as String;
        expect(p1['package']['directories'], contains('assets/empty'));
        await press('activate-package');
        await press('package-list');
        source = '${base.path}/review-v2.zip';
        await import();
        await press('activate-package');
        final catalogue = (await packages('list'))['installed'] as List;
        final p2 =
            catalogue.firstWhere((p) => p['package']['id'] != id1) as Map;
        final id2 = p2['package']['id'] as String;
        await press('package-list');
        await press('inspect-package-$id1');
        await t.scrollUntilVisible(
          find.text('Saved versions'),
          160,
          scrollable: find
              .descendant(
                of: find.byKey(const Key('packages-scroll')),
                matching: find.byType(Scrollable),
              )
              .first,
        );
        await t.tap(find.text('Saved versions'));
        await t.pumpAndSettle();
        await press('rollback-package-1');
        await press('export-package');
        expect(
          File(exported).readAsBytesSync(),
          File('${base.path}/review-v1.zip').readAsBytesSync(),
        );
        await press('package-list');
        final damaged = File('${p2['package']['root']}/references/check.md');
        damaged.writeAsStringSync('CHANGED_UNTRUSTED_CACHE_BYTES');
        await press('inspect-package-$id2');
        expect(find.byKey(const Key('package-status')), findsOneWidget);
        await t.ensureVisible(find.text('Retained data and recovery'));
        await t.tap(find.text('Retained data and recovery'));
        await t.pumpAndSettle();
        await press('retained-package-$id2');
        exported = '${base.path}/$scope-export-damaged-v2.zip';
        await press('export-package');
        expect(
          File(exported).readAsBytesSync(),
          File('${base.path}/review-v2.zip').readAsBytesSync(),
        );
        damaged.writeAsStringSync('PUBLIC_PACKAGE_V2\nVerify the dock 世界.\n');
        final saved = await call('reviewSkill', {
          'session': session,
          'scope': scope,
          'name': 'review',
          'version': 3,
        });
        expect(saved['package']['id'], id1);
        await call('cancelSkillReview', {
          'session': session,
          'token': saved['token'],
        });
        await t.pumpWidget(const SizedBox());
        final chat = ownedChat = ChatController(bridge, ownsBridge: false)
          ..loading = false;
        await t.runAsync(chat.refresh);
        await t.runAsync(() => chat.select(session));
        host = AppHost(chat);
        await t.pumpWidget(DoloresApp(chat: chat, host: host));
        await t.pumpAndSettle();
        await t.enterText(find.byKey(const Key('composer-field-0')), '/review');
        await t.pump();
        await waitUntil(
          () =>
              find
                  .byKey(const Key('command-skill-review'))
                  .evaluate()
                  .isNotEmpty &&
              find
                  .descendant(
                    of: find.byKey(const Key('command-picker')),
                    matching: find.byType(LinearProgressIndicator),
                  )
                  .evaluate()
                  .isEmpty,
        );
        // "Review" also matches Plan mode's description; choose the exact skill.
        await t.tap(find.byKey(const Key('command-skill-review')));
        await waitUntil(
          () => chat.selectedSkills.length == 1 && chat.draft.isEmpty,
        );
        final preview = await t.runAsync(chat.previewContext);
        expect(
          jsonEncode(preview!['messages']),
          contains('Host-pinned package identity:'),
        );
        expect(
          (preview['tools'] as List).any(
            (v) => v['name'] == 'read_skill_resource',
          ),
          isTrue,
        );
        expect(preview['skills'][0]['package']['id'], id1);
        await t.enterText(
          find.byKey(const Key('composer-field-0')),
          'Use the selected review package. Read its reference and inspect the binary metadata.',
        );
        await t.pump();
        await t.tap(find.byKey(const Key('send')));
        var approvals = 0;
        for (var i = 0; i < 3; i++) {
          await waitUntil(
            () =>
                chat.toolApproval != null || (!chat.busy && chat.error != null),
          );
          expect(chat.error, isNull);
          expect(chat.toolApproval!['name'], 'read_skill_resource');
          final approvedCall = chat.toolApproval!['callId'];
          await t.ensureVisible(find.byKey(const Key('allow-tool')));
          await t.tap(find.byKey(const Key('allow-tool')));
          approvals++;
          await waitUntil(
            () =>
                chat.toolApproval == null ||
                chat.toolApproval!['callId'] != approvedCall,
          );
        }
        await waitUntil(() => !chat.busy && !chat.changing);
        expect(chat.error, isNull);
        final messages =
            (await call('messagesPage', {'session': session}))['items'] as List;
        final metadata =
            messages.lastWhere((m) => m['role'] == 'assistant')['metadata']
                as Map;
        final tools = metadata['agent']['tools'] as List;
        expect(tools.length, 3);
        expect(
          tools.every(
            (tool) =>
                tool['name'] == 'read_skill_resource' &&
                tool['status'] == 'read',
          ),
          isTrue,
        );
        expect(metadata['context']['skills'][0]['package']['id'], id1);
        final sent = bodies.skip(bodies.length - 4).toList();
        expect(sent.length, 4);
        expect(
          jsonEncode(sent.last['messages']),
          contains('PUBLIC_PACKAGE_V1'),
        );
        expect(jsonEncode(sent.last['messages']), contains('binary'));
        expect(
          jsonEncode(sent),
          isNot(contains('CHANGED_UNTRUSTED_CACHE_BYTES')),
        );
        expect(jsonEncode(sent), isNot(contains('PUBLIC_PACKAGE_V2')));
        for (final body in sent) {
          expect(jsonEncode((body['messages'] as List).first), contains(id1));
        }
        await t.pumpWidget(const SizedBox());
        host.dispose();
        host = null;
        ownedChat = null;
        await t.pump(const Duration(seconds: 2));
        await openPackages();
        for (var version = 3; version <= 5; version++) {
          source = '${base.path}/review-v$version.zip';
          await import();
          await press('activate-package');
          await press('package-list');
        }
        final five = (await packages('list'))['installed'] as List;
        expect(five.length, 5);
        final p5 = five.firstWhere(
          (p) => p['package']['manifest']['version'] == '5.0',
        ) as Map;
        final currentBytes = File(
          '${p5['package']['root']}/references/check.md',
        ).readAsBytesSync();
        source = '${base.path}/review-v6.zip';
        await import();
        expect(find.textContaining('five installed'), findsWidgets);
        final currentBefore = await call('reviewSkill', {
          'session': session,
          'scope': scope,
          'name': 'review',
          'version': 6,
        });
        expect(currentBefore['package']['id'], p5['package']['id']);
        await call('cancelSkillReview', {
          'session': session,
          'token': currentBefore['token'],
        });
        await press('package-list');
        await press('inspect-package-$id1');
        await t.scrollUntilVisible(
          find.text('Saved versions'),
          160,
          scrollable: find
              .descendant(
                of: find.byKey(const Key('packages-scroll')),
                matching: find.byType(Scrollable),
              )
              .first,
        );
        await t.tap(find.text('Saved versions'));
        await t.pumpAndSettle();
        await press('forget-package-version-3');
        await press('confirm-forget-package-version');
        expect(
          File('${p5['package']['root']}/references/check.md')
              .readAsBytesSync(),
          currentBytes,
        );
        await press('remove-package');
        await t.tap(find.text('Remove installation'));
        await waitUntil(
          () => find.byType(LinearProgressIndicator).evaluate().isEmpty,
        );
        final currentAfter = await call('reviewSkill', {
          'session': session,
          'scope': scope,
          'name': 'review',
          'version': 6,
        });
        expect(currentAfter['package']['id'], p5['package']['id']);
        await call('cancelSkillReview', {
          'session': session,
          'token': currentAfter['token'],
        });
        await import();
        expect(
          File('${p5['package']['root']}/references/check.md')
              .readAsBytesSync(),
          currentBytes,
        );
        await press('activate-package');
        final latest = await call('reviewSkill', {
          'session': session,
          'scope': scope,
          'name': 'review',
          'version': 7,
        });
        expect(latest['package']['manifest']['version'], '6.0');
        await call('cancelSkillReview', {
          'session': session,
          'token': latest['token'],
        });
        receipts.add({
          'scope': scope,
          'installDidNotActivate': true,
          'rollbackPackageId': id1,
          'originalZipExportExact': true,
          'damagedPayloadOriginalExportExact': true,
          'reviewedResourceCalls': approvals,
          'providerRequests': sent.length,
          'referenceV1AndBinaryMetadataReachedProvider': true,
          'sixthInstallRefusedUntilReviewedOldActivationAndInstallationRemoval':
              true,
          'currentFifthPackagePreservedDuringRecovery': true,
          'sixthPackageActivatedWithoutForgettingSkill': true,
        });
        await call('disableSkill', {
          'session': session,
          'scope': scope,
          'name': 'review',
          'revision': latest['revision'],
        });
        await t.pumpWidget(const SizedBox());
        await t.pump(const Duration(seconds: 2));
      }
      File('${base.path}/skill-packages-receipt.json').writeAsStringSync(
        const JsonEncoder.withIndent('  ').convert({
          'scope': 'Normal Windows bridge and rendered package/composer/review controls; deterministic public loopback model; chooser paths injected. No physical-dialog or real-model reliability claim.',
          'flows': receipts,
          'passed': true,
        }),
      );
      expect(t.takeException(), isNull);
    },
    skip:
        !Platform.isWindows ||
        Platform.environment['DOLORES_AUDIT_LIBRARY'] == null,
  );
}
