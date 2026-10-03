import 'dart:convert';

import 'package:dolores_flutter/bridge.dart';
import 'package:dolores_flutter/mcp.dart';
import 'package:dolores_flutter/theme.dart';
import 'package:dolores_flutter/tool_activity.dart';
import 'package:dolores_flutter/chat.dart';
import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';

class McpBridge implements ChatBridge {
  final commands = <Map<String, dynamic>>[];
  Map<String, dynamic>? saved;
  final connections = <Map<String, dynamic>>[];
  String inspectedId = 'legacy';
  String? failEnable;
  int serial = 0;
  bool pending = false, cancelled = false, failSave = false, failForget = false;
  final tools = [
    for (final name in ['echo', 'second', 'third'])
      {
        'name': name,
        'description': 'Synthetic $name',
        'inputSchema': {'type': 'object'},
      },
  ];
  @override
  Future<void> open() async {}
  @override
  Future<void> close() async {}
  @override
  Future<dynamic> call(Map<String, dynamic> c) async {
    commands.add(c);
    switch (c['command']) {
      case 'mcpSettings':
        return {
          'directory': 'C:/synthetic/project',
          'connection': saved,
          'connections': connections.isEmpty
              ? [if (saved != null) saved]
              : connections,
        };
      case 'inspectMcp':
        inspectedId = c['connectionId'] as String? ?? 'legacy';
        if (inspectedId.isEmpty) {
          inspectedId =
              '00000000-0000-4000-8000-${(++serial).toString().padLeft(12, '0')}';
        }
        return null;
      case 'poll':
        if (pending && !cancelled) return [];
        return [
          {
            'type': 'done',
            if (cancelled)
              'error': 'MCP inspection stopped. Nothing was saved.'
            else
              'mcpInspection': {
                'token': 'review',
                'connectionId': inspectedId,
                'tools': tools,
                'serverName': 'Fixture',
                'serverVersion': '1',
                'protocolVersion': '2025-11-25',
              },
          },
        ];
      case 'cancel':
        cancelled = true;
        return null;
      case 'enableMcp':
        if (failSave) throw 'Synthetic save failure';
        if (failEnable != null) throw failEnable!;
        saved = {
          'id': inspectedId,
          'enabled': true,
          'revision': 1,
          'launch': {
            'label': 'Fixture',
            'executable': 'C:/Runtime/node.exe',
            'args': ['server.mjs'],
          },
          'tools': tools
              .where((t) => (c['names'] as List).contains(t['name']))
              .toList(),
          'credentials': [
            for (final row
                in commands.lastWhere(
                          (c) => c['command'] == 'inspectMcp',
                        )['credentials']
                        as List? ??
                    [])
              {'name': row['name'], 'credentialId': 'opaque-fixture-reference'},
          ],
        };
        connections.removeWhere((s) => s['id'] == inspectedId);
        connections.add(saved!);
        return saved;
      case 'disableMcp':
        saved = connections.firstWhere((s) => s['id'] == c['connectionId']);
        saved!['enabled'] = false;
        saved!['revision'] = 2;
        return null;
      case 'forgetMcp':
        saved = connections.firstWhere((s) => s['id'] == c['connectionId']);
        if (failForget) {
          saved!['enabled'] = false;
          saved!['revision'] = 3;
          throw 'MCP tools are disabled, but credential removal failed. Unlock secure storage and press Forget again.';
        }
        connections.removeWhere((s) => s['id'] == c['connectionId']);
        saved = connections.lastOrNull;
        return null;
      case 'discardMcpReview':
        return null;
      default:
        throw 'Unexpected ${c['command']}';
    }
  }
}

Future<void> open(WidgetTester t, McpBridge b, bool dark) async {
  t.view.physicalSize = const Size(390, 700);
  t.view.devicePixelRatio = 1;
  addTearDown(t.view.resetPhysicalSize);
  addTearDown(t.view.resetDevicePixelRatio);
  await t.pumpWidget(
    MaterialApp(
      theme: doloresTheme(dark),
      home: Scaffold(
        body: Builder(
          builder: (context) => TextButton(
            onPressed: () => showDialog<void>(
              context: context,
              barrierDismissible: false,
              builder: (_) => McpInspector(
                bridge: b,
                session: 'work',
                chooseExecutable: () async => 'C:/Runtime/node.exe',
              ),
            ),
            child: const Text('Open'),
          ),
        ),
      ),
    ),
  );
  await t.tap(find.text('Open'));
  await t.pumpAndSettle();
  await t.scrollUntilVisible(
    find.byKey(const Key('mcp-name')),
    120,
    scrollable: find.byType(Scrollable).first,
  );
  await t.enterText(find.byKey(const Key('mcp-name')), 'Fixture');
  await press(t, 'mcp-choose-program');
}

Future<void> press(WidgetTester t, String key, {bool settle = true}) async {
  final target = find.byKey(Key(key));
  if (target.evaluate().isEmpty) {
    await t.scrollUntilVisible(
      target,
      120,
      scrollable: find.byType(Scrollable).first,
    );
  }
  await Scrollable.ensureVisible(t.element(target), alignment: .5);
  await t.pump();
  await t.tap(target);
  if (settle) {
    await t.pumpAndSettle();
  } else {
    await t.pump();
  }
}

void main() {
  testWidgets(
    'shared-slot refusal preserves review and fields while disabling another server enables retry',
    (t) async {
      final b = McpBridge();
      await open(t, b, true);
      await press(t, 'mcp-inspect');
      await press(t, 'mcp-tool-echo');
      await press(t, 'mcp-enable');
      final first = b.saved!['id'] as String;
      await press(t, 'mcp-add-server');
      await t.scrollUntilVisible(
        find.byKey(const Key('mcp-name')),
        120,
        scrollable: find.byType(Scrollable).first,
      );
      await t.enterText(find.byKey(const Key('mcp-name')), 'Second');
      await press(t, 'mcp-choose-program');
      await press(t, 'mcp-inspect');
      await press(t, 'mcp-tool-echo');
      await press(t, 'mcp-tool-second');
      b.failEnable = 'Only two external tools can be enabled per folder. Disable another server or select fewer tools, then retry Enable. Your review is preserved.';
      await press(t, 'mcp-enable');
      expect(find.textContaining('Your review is preserved.'), findsOneWidget);
      expect(find.byKey(const Key('mcp-enable')), findsOneWidget);
      final inspections = b.commands
          .where((c) => c['command'] == 'inspectMcp')
          .length;
      final discards = b.commands
          .where((c) => c['command'] == 'discardMcpReview')
          .length;
      await press(t, 'mcp-disable-other-$first');
      expect(
        b.commands.where((c) => c['command'] == 'discardMcpReview').length,
        discards,
      );
      expect(find.byKey(const Key('mcp-enable')), findsOneWidget);
      b.failEnable = null;
      await press(t, 'mcp-enable');
      expect(
        b.commands.where((c) => c['command'] == 'inspectMcp').length,
        inspections,
      );
      expect(b.connections.length, 2);
      expect(b.connections.first['enabled'], false);
      expect(b.connections.last['enabled'], true);
      expect(t.takeException(), isNull);
    },
  );
  testWidgets(
    'credential values are masked, reused by name and never sent to Enable',
    (t) async {
      final b = McpBridge();
      await open(t, b, true);
      await press(t, 'mcp-add-credential');
      final name = find.byKey(const Key('mcp-credential-name-0'));
      final value = find.byKey(const Key('mcp-credential-value-0'));
      await t.scrollUntilVisible(
        name,
        100,
        scrollable: find.byType(Scrollable).first,
      );
      await t.enterText(name, 'SERVICE_API_KEY');
      await Scrollable.ensureVisible(t.element(value));
      await t.pump();
      await t.enterText(value, 'synthetic-secret-value');
      expect(t.widget<TextField>(value).obscureText, true);
      expect(t.widget<TextField>(value).enableSuggestions, false);
      await press(t, 'mcp-inspect');
      expect(
        b.commands.lastWhere(
          (c) => c['command'] == 'inspectMcp',
        )['credentials'],
        [
          {'name': 'SERVICE_API_KEY', 'value': 'synthetic-secret-value'},
        ],
      );
      await press(t, 'mcp-tool-echo');
      await press(t, 'mcp-enable');
      expect(
        b.commands
            .lastWhere((c) => c['command'] == 'enableMcp')
            .containsKey('credentials'),
        false,
      );
      await press(t, 'mcp-inspect');
      expect(
        b.commands.lastWhere(
          (c) => c['command'] == 'inspectMcp',
        )['credentials'],
        [
          {'name': 'SERVICE_API_KEY', 'value': null},
        ],
      );
      await press(t, 'mcp-tool-echo');
      b.failForget = true;
      await press(t, 'mcp-forget');
      expect(find.byKey(const Key('mcp-disable')), findsNothing);
      expect(find.textContaining('press Forget again'), findsOneWidget);
      b.failForget = false;
      await press(t, 'mcp-forget');
      expect(
        b.commands.lastWhere((c) => c['command'] == 'forgetMcp')['revision'],
        3,
      );
      expect(b.saved, isNull);
      expect(t.takeException(), isNull);
    },
  );
  for (final dark in [false, true]) {
    testWidgets(
      'MCP review and explicit enable/disable/forget in compact ${dark ? 'dark' : 'light'}',
      (t) async {
        final b = McpBridge();
        await open(t, b, dark);
        expect(b.commands.where((c) => c['command'] == 'inspectMcp'), isEmpty);
        await press(t, 'mcp-add-argument');
        await t.enterText(
          find.byKey(const Key('mcp-arg-0')),
          'server with spaces.mjs',
        );
        await press(t, 'mcp-inspect');
        final launch = b.commands.lastWhere(
          (c) => c['command'] == 'inspectMcp',
        )['launch'];
        expect(launch['args'], ['server with spaces.mjs']);
        expect(b.saved, isNull);
        expect(
          t.widget<FilledButton>(find.byKey(const Key('mcp-enable'))).onPressed,
          isNull,
        );
        await press(t, 'mcp-tool-echo');
        await press(t, 'mcp-tool-second');
        await press(t, 'mcp-tool-third');
        expect(
          find.text(
            'Choose at most two MCP tools. This folder shares two external tool slots across servers.',
          ),
          findsOneWidget,
        );
        await press(t, 'mcp-enable');
        expect(b.saved!['enabled'], true);
        expect(
          b.commands.lastWhere((c) => c['command'] == 'enableMcp')['names'],
          ['echo', 'second'],
        );
        await press(t, 'mcp-disable');
        expect(b.saved!['enabled'], false);
        await press(t, 'mcp-forget');
        expect(b.saved, isNull);
        expect(t.takeException(), isNull);
      },
    );
  }
  testWidgets(
    'editing invalidates review and failed save allows explicit retry',
    (t) async {
      final b = McpBridge();
      await open(t, b, false);
      await press(t, 'mcp-inspect');
      await press(t, 'mcp-tool-echo');
      b.failSave = true;
      await press(t, 'mcp-enable');
      expect(find.text('Synthetic save failure'), findsOneWidget);
      expect(b.saved, isNull);
      expect(find.byKey(const Key('mcp-enable')), findsOneWidget);
      b.failSave = false;
      await press(t, 'mcp-enable');
      expect(b.saved, isNotNull);
      await press(t, 'mcp-inspect');
      t
          .widget<ListView>(find.byKey(const Key('mcp-scroll')))
          .controller!
          .jumpTo(300);
      await t.pumpAndSettle();
      await t.scrollUntilVisible(
        find.byKey(const Key('mcp-name')),
        120,
        scrollable: find.byType(Scrollable).first,
      );
      await Scrollable.ensureVisible(
        t.element(find.byKey(const Key('mcp-name'))),
      );
      await t.pump();
      await t.enterText(find.byKey(const Key('mcp-name')), 'Changed');
      await t.pumpAndSettle();
      expect(find.byKey(const Key('mcp-enable')), findsNothing);
      expect(b.commands.any((c) => c['command'] == 'discardMcpReview'), true);
      expect(b.saved!['launch']['label'], 'Fixture');
      expect(t.takeException(), isNull);
    },
  );
  testWidgets('Stop prevents an inspection from being enabled', (t) async {
    final b = McpBridge()..pending = true;
    await open(t, b, true);
    await press(t, 'mcp-inspect', settle: false);
    await t.pump(const Duration(milliseconds: 80));
    expect(
      t.widget<FilledButton>(find.byKey(const Key('mcp-inspect'))).onPressed,
      isNull,
    );
    await press(t, 'mcp-stop');
    await t.pump(const Duration(milliseconds: 80));
    await t.pumpAndSettle();
    expect(b.cancelled, true);
    expect(b.saved, isNull);
    expect(find.byKey(const Key('mcp-enable')), findsNothing);
    expect(t.takeException(), isNull);
  });
  test('External tool labels and error results are honest', () {
    expect(toolLabel('mcp_tool_1'), 'External tool');
    expect(
      toolResultText({
        'name': 'mcp_tool_1',
        'status': 'completed',
        'content': jsonEncode({'text': 'bad arguments', 'isError': true}),
      }),
      'Tool reported an error\n\nbad arguments',
    );
  });
  testWidgets(
    'External approval shows exact selectable arguments with visible one-use buttons',
    (t) async {
      t.view.physicalSize = const Size(390, 700);
      t.view.devicePixelRatio = 1;
      addTearDown(t.view.resetPhysicalSize);
      addTearDown(t.view.resetDevicePixelRatio);
      final chat = ChatController(McpBridge())
        ..loading = false
        ..model = 'fixture'
        ..workspaceRoot = 'C:/synthetic/project'
        ..workspaceKind = 'project';
      addTearDown(chat.dispose);
      chat.toolApproval = {
        'callId': 'one',
        'name': 'mcp_tool_1',
        'target': 'Fixture / echo',
        'mcp': {
          'server': 'Fixture',
          'tool': 'echo',
          'revision': 3,
          'credentialNames': ['SERVICE_API_KEY'],
          'arguments': jsonEncode({'text': 'x' * 3000}),
        },
      };
      for (final dark in [false, true]) {
        await t.pumpWidget(
          MaterialApp(
            theme: doloresTheme(dark),
            home: Scaffold(
              body: Column(children: [ToolApprovalCard(chat: chat)]),
            ),
          ),
        );
        await t.pumpAndSettle();
        expect(find.text('Allow this external tool?'), findsOneWidget);
        expect(find.text('Run once').hitTestable(), findsOneWidget);
        expect(find.text('Deny').hitTestable(), findsOneWidget);
        expect(find.textContaining('does not sandbox'), findsNothing);
        expect(find.textContaining('Effects may remain'), findsOneWidget);
        expect(
          find.textContaining(
            'Server receives saved credentials: SERVICE_API_KEY',
          ),
          findsOneWidget,
        );
        expect(t.takeException(), isNull);
      }
    },
  );
}
