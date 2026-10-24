import 'dart:convert';
import 'dart:io';

import 'package:dolores_flutter/app_host.dart';
import 'package:dolores_flutter/bridge.dart';
import 'package:dolores_flutter/chat.dart';
import 'package:dolores_flutter/main.dart';
import 'package:dolores_flutter/mcp.dart';
import 'package:dolores_flutter/theme.dart';
import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';

void main() {
  testWidgets(
    'native remote MCP setup, auth recovery and approved model invocation',
    (t) async {
      final base = Directory(Platform.environment['DOLORES_DATA_DIR']!).parent;
      final output = Directory('../../output').absolute
          .resolveSymbolicLinksSync();
      expect(
        base.resolveSymbolicLinksSync(),
        startsWith('$output${Platform.pathSeparator}'),
      );
      final project = Directory('${base.path}/remote-project')..createSync();
      final marker = File('${project.path}/remote-result.txt');
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

      Future<void> waitUntil(bool Function() ready) async {
        final end = DateTime.now().add(const Duration(seconds: 25));
        do {
          await t.runAsync(
            () => Future<void>.delayed(const Duration(milliseconds: 25)),
          );
          await t.pump(const Duration(milliseconds: 80));
          if (ready()) return;
        } while (DateTime.now().isBefore(end));
        fail('Remote MCP native fixture exceeded its bounded wait.');
      }

      Future<void> reveal(String key) async {
        final target = find.byKey(Key(key));
        if (target.evaluate().isEmpty) {
          final scroll = find
              .descendant(
                of: find.byKey(const Key('mcp-scroll')),
                matching: find.byType(Scrollable),
              )
              .first;
          t.state<ScrollableState>(scroll).position.jumpTo(0);
          await t.pump();
          await t.scrollUntilVisible(target, 160, scrollable: scroll);
        }
        await t.ensureVisible(target);
        await t.pump();
      }

      Future<void> press(String key, {bool wait = true}) async {
        await reveal(key);
        await t.tap(find.byKey(Key(key)));
        await t.pump();
        if (wait) {
          await waitUntil(
            () => find.byType(LinearProgressIndicator).evaluate().isEmpty,
          );
          await t.pumpAndSettle();
        }
      }

      Future<void> fill(String key, String value) async {
        await reveal(key);
        await t.enterText(find.byKey(Key(key)), value);
        await t.pump();
      }

      var expectedKey = 'public-fixture-key';
      var hold = false;
      var calls = 0;
      final mcpRequests = <Map>[];
      final mcp = (await t.runAsync(() async {
        final server = await HttpServer.bind(InternetAddress.loopbackIPv4, 0);
        server.listen((request) async {
          final body =
              jsonDecode(await utf8.decoder.bind(request).join()) as Map;
          mcpRequests.add(body);
          if (hold) return;
          request.response.headers.contentType = ContentType.json;
          if (request.headers.value(HttpHeaders.authorizationHeader) !=
              'Bearer $expectedKey') {
            request.response.statusCode = 401;
            request.response.write('{"error":"Public auth refusal"}');
            await request.response.close();
            return;
          }
          final result = switch (body['method']) {
            'server/discover' => {
              'resultType': 'complete',
              'supportedVersions': ['2026-07-28'],
              'capabilities': {'tools': {}},
              '_meta': {
                'io.modelcontextprotocol/serverInfo': {
                  'name': 'Public fixture',
                  'version': '1',
                },
              },
            },
            'tools/list' => {
              'resultType': 'complete',
              'tools': [
                {
                  'name': 'echo',
                  'description': 'Write one public fixture marker.',
                  'inputSchema': {
                    'type': 'object',
                    'properties': {
                      'text': {'type': 'string'},
                    },
                    'required': ['text'],
                  },
                },
              ],
            },
            'tools/call' => {
              'resultType': 'complete',
              'content': [
                {'type': 'text', 'text': body['params']['arguments']['text']},
              ],
            },
            _ => throw StateError('Unexpected remote fixture request'),
          };
          if (body['method'] == 'tools/call') {
            calls++;
            marker.writeAsStringSync(
              body['params']['arguments']['text'] as String,
            );
          }
          request.response.write(
            jsonEncode({'jsonrpc': '2.0', 'id': body['id'], 'result': result}),
          );
          await request.response.close();
        });
        return server;
      }))!;
      final bodies = <Map>[];
      final provider = (await t.runAsync(() async {
        final server = await HttpServer.bind(InternetAddress.loopbackIPv4, 0);
        server.listen((request) async {
          final body =
              jsonDecode(await utf8.decoder.bind(request).join()) as Map;
          bodies.add(body);
          final alias = (body['tools'] as List)
              .map((v) => v['function']['name'] as String)
              .firstWhere((n) => n.startsWith('mcp_tool_'));
          final first = bodies.length == 1;
          final delta = first
              ? {
                  'tool_calls': [
                    {
                      'index': 0,
                      'id': 'public-remote-call',
                      'type': 'function',
                      'function': {
                        'name': alias,
                        'arguments': jsonEncode({
                          'text': 'MCP_PUBLIC_EFFECT 世界',
                        }),
                      },
                    },
                  ],
                }
              : {'content': 'Verified the public remote fixture marker.'};
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
        return server;
      }))!;
      AppHost? host;
      ChatController? chat;
      addTearDown(() async {
        host?.dispose();
        if (host == null) chat?.dispose();
        await mcp.close(force: true);
        await provider.close(force: true);
        await bridge.close();
      });
      await call('configure', {
        'preferences': {
          'baseUrl': 'http://127.0.0.1:${provider.port}/v1',
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
      final memories = await call('memories', {'session': session});
      if (memories['automaticPolicy']['enabled'] == true) {
        await call('setAutomaticMemory', {
          'enabled': false,
          'revision': memories['automaticPolicy']['revision'],
        });
      }
      t.view.physicalSize = const Size(1000, 900);
      t.view.devicePixelRatio = 1;
      addTearDown(t.view.resetPhysicalSize);
      addTearDown(t.view.resetDevicePixelRatio);
      final endpoint = 'http://127.0.0.1:${mcp.port}/mcp';
      Future<void> openConnections(bool dark) async {
        await t.pumpWidget(
          MaterialApp(
            theme: doloresTheme(dark),
            home: Scaffold(
              body: Builder(
                builder: (context) => TextButton(
                  key: const Key('open-connections'),
                  onPressed: () => showDialog<void>(
                    context: context,
                    builder: (_) =>
                        McpInspector(bridge: bridge, session: session),
                  ),
                  child: const Text('Connections'),
                ),
              ),
            ),
          ),
        );
        await t.tap(find.byKey(const Key('open-connections')));
        await waitUntil(
          () => find.byType(LinearProgressIndicator).evaluate().isEmpty,
        );
        await t.pumpAndSettle();
      }

      await openConnections(false);
      await press('mcp-add-server');
      await press('mcp-transport');
      await t.tap(find.text('Remote endpoint').last);
      await t.pumpAndSettle();
      await fill('mcp-name', 'Public MCP');
      await fill('mcp-endpoint', endpoint);
      await press('mcp-auth');
      await t.tap(find.text('Bearer key').last);
      await t.pumpAndSettle();
      await fill('mcp-bearer', expectedKey);
      expect(
        t.widget<TextField>(find.byKey(const Key('mcp-bearer'))).obscureText,
        true,
      );
      await press('mcp-inspect');
      expect(calls, 0);
      expect(marker.existsSync(), false);
      final inspectedRequests = mcpRequests.length;
      await press('mcp-tool-echo');
      await fill('mcp-tool-search', 'not present');
      expect(find.text('No tools match this search.'), findsOneWidget);
      await press('mcp-enable');
      final initial =
          (await call('mcpSettings', {'session': session}))['connections'][0]
              as Map;
      final id = initial['id'] as String;
      expect(mcpRequests.length, inspectedRequests);
      expect(initial['remote']['endpoint'], endpoint);
      expect(initial.containsKey('launch'), false);
      expect(initial['credentials'][0]['name'], 'MCP_BEARER_TOKEN');
      expect(jsonEncode(initial), isNot(contains(expectedKey)));
      await t.tap(find.text('Close'));
      await t.pumpAndSettle();
      await openConnections(true);
      expect(mcpRequests.length, inspectedRequests);
      await reveal('mcp-health');
      expect(find.text('Connection status: Not checked'), findsOneWidget);
      await press('mcp-inspect');
      expect(find.textContaining('Endpoint checked.'), findsOneWidget);
      expectedKey = 'public-rotated-key';
      await press('mcp-inspect');
      expect(find.textContaining('authentication expired'), findsOneWidget);
      final preserved = (await call('mcpSettings', {
        'session': session,
      }))['connections'][0];
      expect(preserved, initial);
      final count = mcpRequests.length;
      await fill('mcp-endpoint', '$endpoint/changed');
      await press('mcp-inspect');
      expect(mcpRequests.length, count);
      expect(find.textContaining('nonempty MCP bearer'), findsOneWidget);
      await fill('mcp-endpoint', endpoint);
      await fill('mcp-bearer', expectedKey);
      await press('mcp-inspect');
      await press('mcp-tool-echo');
      await press('mcp-enable');
      final rotated = (await call('mcpSettings', {
        'session': session,
      }))['connections'][0];
      expect(rotated['revision'], 2);
      expect(
        rotated['credentials'][0]['credentialId'],
        isNot(initial['credentials'][0]['credentialId']),
      );
      hold = true;
      final before = mcpRequests.length;
      await press('mcp-inspect', wait: false);
      await waitUntil(() => mcpRequests.length > before);
      await press('mcp-stop');
      expect(find.textContaining('stopped'), findsWidgets);
      hold = false;
      expect(
        (await call('mcpSettings', {'session': session}))['connections'][0],
        rotated,
      );
      await t.tap(find.text('Close'));
      await t.pumpAndSettle();
      final owned = chat = ChatController(bridge, ownsBridge: false)
        ..loading = false;
      await t.runAsync(owned.refresh);
      await t.runAsync(() => owned.select(session));
      host = AppHost(owned);
      await t.pumpWidget(DoloresApp(chat: owned, host: host));
      await t.pumpAndSettle();
      await t.enterText(
        find.byKey(const Key('composer-field-0')),
        'Use the public MCP fixture once and verify its marker.',
      );
      await t.pump();
      await t.tap(find.byKey(const Key('send')));
      await waitUntil(
        () =>
            owned.toolApproval != null || (!owned.busy && owned.error != null),
      );
      expect(owned.error, isNull);
      expect(owned.toolApproval!['mcp']['remoteEndpoint'], endpoint);
      expect(find.byKey(const Key('mcp-approval-endpoint')), findsOneWidget);
      await t.ensureVisible(find.byKey(const Key('allow-tool')));
      await t.tap(find.byKey(const Key('allow-tool')));
      await waitUntil(() => !owned.busy && !owned.changing);
      expect(owned.error, isNull);
      expect(calls, 1);
      expect(marker.readAsStringSync(), 'MCP_PUBLIC_EFFECT 世界');
      expect(bodies.length, 2);
      final sent = jsonEncode(bodies);
      expect(sent, isNot(contains('public-fixture-key')));
      expect(sent, isNot(contains('public-rotated-key')));
      expect(
        jsonEncode(bodies.last['messages']),
        contains('MCP_PUBLIC_EFFECT'),
      );
      final messages =
          (await call('messagesPage', {'session': session}))['items'] as List;
      final tools =
          messages.lastWhere(
                (m) => m['role'] == 'assistant',
              )['metadata']['agent']['tools']
              as List;
      expect(tools.length, 1);
      expect(tools.single['status'], 'completed');
      await call('disableMcp', {
        'session': session,
        'connectionId': id,
        'revision': 2,
      });
      await t.pumpWidget(const SizedBox());
      host.dispose();
      host = null;
      chat = null;
      await t.pump(const Duration(seconds: 2));
      File('${base.path}/mcp-remote-receipt.json').writeAsStringSync(
        const JsonEncoder.withIndent('  ').convert({
          'scope': 'Rendered controls and normal Windows bridge with public loopback MCP/model fixtures. No physical-input or real-model reliability claim.',
          'connection': id,
          'toolCalls': calls,
          'modelRequests': bodies.length,
          'authExpiryPreservedConnection': true,
          'changedTargetDidNotReceiveSavedKey': true,
          'keyRotated': true,
          'inspectionStoppedWithoutReplacingConnection': true,
          'filteredSelectionPreserved': true,
          'exactEndpointApproved': true,
          'toolReceiptStatus': tools.single['status'],
          'publicMarkerVerified': true,
          'credentialsExcludedFromProvider': true,
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
