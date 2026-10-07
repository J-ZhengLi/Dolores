import 'dart:async';

import 'package:dolores_flutter/bridge.dart';
import 'package:dolores_flutter/chat.dart';
import 'package:dolores_flutter/memory.dart';
import 'package:dolores_flutter/inspector.dart';
import 'package:dolores_flutter/theme.dart';
import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';

class MemoryBridge implements ChatBridge {
  final commands = <Map<String, dynamic>>[];
  final items = <Map<String, dynamic>>[];
  bool folder = true, fail = false;
  bool supportsAutomatic = false, automatic = true;
  int policyRevision = 1;
  Completer<void>? pending;
  @override
  Future<void> open() async {}
  @override
  Future<void> close() async {}
  @override
  Future<dynamic> call(Map<String, dynamic> command) async {
    commands.add(command);
    switch (command['command']) {
      case 'memories':
        return {
          'items': items.map((v) => {...v}).toList(),
          'folderAvailable': folder,
          if (supportsAutomatic)
            'automaticPolicy': {
              'enabled': automatic,
              'revision': policyRevision,
            },
          if (supportsAutomatic)
            'automaticAttempt': {
              'messageId': 1,
              'status': 'completed',
              'note': '1 saved',
              'updatedAt': 1,
              'saved': 1,
              'skipped': 0,
              'usage': {'inputTokens': 100, 'outputTokens': 25},
            },
        };
      case 'setAutomaticMemory':
        if (fail) throw Exception('Policy changed. Refresh Memory.');
        automatic = command['enabled'] as bool;
        policyRevision++;
        return {'enabled': automatic, 'revision': policyRevision};
      case 'saveMemory':
        if (pending != null) await pending!.future;
        if (fail) {
          throw Exception(
            'Preference changed. Cancel, refresh and review it again.',
          );
        }
        final id = command['id'] ?? 'manual';
        final item =
            {
                ...command,
                'id': id,
                'revision': (command['revision'] as int? ?? 0) + 1,
                'source': 'user',
                'updatedAt': 1,
                'createdAt': 1,
              }
              ..remove('command')
              ..remove('session');
        items.removeWhere((v) => v['id'] == id);
        items.add(item);
        return item;
      case 'deleteMemory':
        items.removeWhere((v) => v['id'] == command['id']);
        return null;
    }
    return null;
  }
}

Future<void> open(
  WidgetTester tester,
  MemoryBridge bridge, {
  bool dark = false,
}) async {
  tester.view.physicalSize = const Size(390, 700);
  tester.view.devicePixelRatio = 1;
  addTearDown(tester.view.resetPhysicalSize);
  addTearDown(tester.view.resetDevicePixelRatio);
  await tester.pumpWidget(
    MaterialApp(
      theme: doloresTheme(dark),
      home: Scaffold(
        body: Builder(
          builder: (context) => TextButton(
            onPressed: () => showDialog<void>(
              context: context,
              builder: (_) => MemoryInspector(bridge: bridge, session: 'chat'),
            ),
            child: const Text('Open'),
          ),
        ),
      ),
    ),
  );
  await tester.tap(find.text('Open'));
  await tester.pumpAndSettle();
}

Future<void> enter(WidgetTester tester, String key, String value) async {
  final finder = find.byKey(Key(key));
  await tester.scrollUntilVisible(
    finder,
    100,
    scrollable: find.byType(Scrollable).first,
  );
  await tester.ensureVisible(finder);
  await tester.pumpAndSettle();
  await tester.enterText(finder, value);
}

void main() {
  for (final dark in [false, true]) {
    testWidgets(
      'automatic policy and learning activity are inspectable and reversible ($dark)',
      (tester) async {
        final bridge = MemoryBridge()..supportsAutomatic = true;
        await open(tester, bridge, dark: dark);
        final toggle = find.byKey(const Key('automatic-memory'));
        await tester.ensureVisible(toggle);
        await tester.pumpAndSettle();
        expect(tester.widget<SwitchListTile>(toggle).value, true);
        await tester.tap(toggle);
        await tester.pumpAndSettle();
        expect(bridge.automatic, false);
        expect(bridge.policyRevision, 2);
        expect(
          bridge.commands.where((c) => c['command'] == 'saveMemory'),
          isEmpty,
        );
        await tester.scrollUntilVisible(
          find.byKey(const Key('automatic-memory-attempt')),
          100,
          scrollable: find.byType(Scrollable).first,
        );
        await tester.ensureVisible(find.text('Latest learning activity in this chat'));
        await tester.pumpAndSettle();
        await tester.tap(find.text('Latest learning activity in this chat'));
        await tester.pumpAndSettle();
        expect(find.textContaining('Learning tokens: 100 in'), findsOneWidget);
        bridge.fail = true;
        await tester.scrollUntilVisible(
          toggle,
          -100,
          scrollable: find.byType(Scrollable).first,
        );
        await Scrollable.ensureVisible(tester.element(toggle), alignment: 0.5);
        await tester.pumpAndSettle();
        await tester.tap(toggle);
        await tester.pumpAndSettle();
        expect(tester.widget<SwitchListTile>(toggle).value, false);
        await tester.scrollUntilVisible(
          find.textContaining('Policy changed.'),
          100,
          scrollable: find.byType(Scrollable).first,
        );
        expect(find.textContaining('Policy changed.'), findsOneWidget);
        expect(tester.takeException(), isNull);
      },
    );
  }
  testWidgets(
    'Close and Cancel do not create preferences; Side has only All chats',
    (tester) async {
      final bridge = MemoryBridge()..folder = false;
      await open(tester, bridge);
      await tester.tap(find.byKey(const Key('new-memory')));
      await tester.pumpAndSettle();
      final picker = tester.widget<DropdownButtonFormField<String>>(
        find.byKey(const Key('memory-scope-all')),
      );
      expect(picker.initialValue, 'all');
      await enter(tester, 'memory-title', 'Unsaved');
      await tester.tap(find.byKey(const Key('cancel-memory-edit')));
      await tester.pumpAndSettle();
      await tester.tap(find.text('Close'));
      await tester.pumpAndSettle();
      expect(bridge.commands.where((c) => c['command'] != 'memories'), isEmpty);
      expect(bridge.items, isEmpty);
      expect(tester.takeException(), isNull);
    },
  );

  testWidgets(
    'explicit Save, revisioned edit, disable and delete remain reachable in compact dark view',
    (tester) async {
      final bridge = MemoryBridge();
      await open(tester, bridge, dark: true);
      await tester.tap(find.byKey(const Key('new-memory')));
      await tester.pumpAndSettle();
      expect(find.byKey(const Key('memory-scope-folder')), findsOneWidget);
      await enter(tester, 'memory-title', 'Response style');
      await enter(
        tester,
        'memory-text',
        '# Literal preference\n@../private.env',
      );
      await tester.tap(find.byKey(const Key('save-memory')));
      await tester.pumpAndSettle();
      expect(bridge.items.single['scope'], 'folder');
      expect(bridge.items.single['enabled'], true);
      await tester.scrollUntilVisible(
        find.byKey(const Key('edit-memory-manual')),
        100,
        scrollable: find.byType(Scrollable).first,
      );
      expect(find.textContaining('Added by you'), findsWidgets);
      expect(
        tester
            .widget<SelectableText>(find.byKey(const Key('memory-body-manual')))
            .data,
        '# Literal preference\n@../private.env',
      );
      await tester.tap(find.byKey(const Key('edit-memory-manual')));
      await tester.pumpAndSettle();
      final picker = tester.widget<DropdownButtonFormField<String>>(
        find.byKey(const Key('memory-scope-folder')),
      );
      expect(picker.onChanged, isNull);
      await enter(tester, 'memory-text', 'Corrected preference');
      await tester.tap(find.byKey(const Key('save-memory')));
      await tester.pumpAndSettle();
      expect(bridge.items.single['revision'], 2);
      await tester.scrollUntilVisible(
        find.byKey(const Key('toggle-memory-manual')),
        100,
        scrollable: find.byType(Scrollable).first,
      );
      await tester.tap(find.byKey(const Key('toggle-memory-manual')));
      await tester.pumpAndSettle();
      expect(bridge.items.single['enabled'], false);
      expect(bridge.items.single['revision'], 3);
      await tester.scrollUntilVisible(
        find.byKey(const Key('delete-memory-manual')),
        100,
        scrollable: find.byType(Scrollable).first,
      );
      await tester.tap(find.byKey(const Key('delete-memory-manual')));
      await tester.pumpAndSettle();
      expect(bridge.items, isEmpty);
      await tester.ensureVisible(find.text('Memory details'));
      await tester.tap(find.text('Memory details'));
      await tester.pumpAndSettle();
      expect(find.textContaining('past replies'), findsWidgets);
      expect(tester.takeException(), isNull);
    },
  );

  testWidgets(
    'pending and failed save preserve the form without overwriting a newer revision',
    (tester) async {
      final bridge = MemoryBridge()
        ..pending = Completer<void>()
        ..fail = true;
      await open(tester, bridge);
      await tester.tap(find.byKey(const Key('new-memory')));
      await tester.pumpAndSettle();
      await enter(tester, 'memory-title', 'Draft');
      await enter(tester, 'memory-text', 'Retained on failure');
      await tester.tap(find.byKey(const Key('save-memory')));
      await tester.pump();
      expect(
        tester
            .widget<FilledButton>(find.byKey(const Key('save-memory')))
            .onPressed,
        isNull,
      );
      expect(
        tester
            .widget<TextButton>(
              find.ancestor(
                of: find.text('Close'),
                matching: find.byType(TextButton),
              ),
            )
            .onPressed,
        isNull,
      );
      bridge.pending!.complete();
      await tester.pumpAndSettle();
      expect(
        tester
            .widget<TextField>(find.byKey(const Key('memory-title')))
            .controller!
            .text,
        'Draft',
      );
      expect(
        tester
            .widget<TextField>(find.byKey(const Key('memory-text')))
            .controller!
            .text,
        'Retained on failure',
      );
      expect(bridge.items, isEmpty);
      await tester.scrollUntilVisible(
        find.textContaining('Cancel, refresh'),
        100,
        scrollable: find.byType(Scrollable).first,
      );
      expect(find.textContaining('Cancel, refresh'), findsOneWidget);
      expect(tester.takeException(), isNull);
    },
  );

  test('Memory inspection works without a folder, locks chat changes and clears context', () async {
    final chat = ChatController(MemoryBridge())
      ..loading = false
      ..contextSummary = {'memory': 'old'};
    final pending = Completer<void>();
    final inspected = chat.inspectLocalSettings(() async {
      await pending.future;
      chat.invalidateContext();
    });
    expect(chat.changing, true);
    chat.newChat(kind: 'side');
    expect(chat.workspaceKind, 'temporary');
    pending.complete();
    await inspected;
    expect(chat.contextSummary, isNull);
    expect(chat.changing, false);
    chat.dispose();
  });

  testWidgets('context exposes exact selected preferences and omitted count', (
    tester,
  ) async {
    await tester.pumpWidget(
      MaterialApp(
        theme: doloresTheme(false),
        home: ContextInspector(
          report: {
            'messages': [
              {'role': 'system', 'content': 'System'},
              {'role': 'user', 'content': ''},
            ],
            'memory': {
              'used': [
                {'id': 'one'},
              ],
              'omitted': 2,
            },
            'memoryEntries': [
              {
                'id': 'one',
                'title': 'Style',
                'text': 'EXACT_LITERAL',
                'scope': 'folder',
                'revision': 3,
              },
            ],
          },
        ),
      ),
    );
    await tester.scrollUntilVisible(
      find.text('Saved preferences'),
      150,
      scrollable: find.byType(Scrollable).first,
    );
    await tester.tap(find.text('Saved preferences'));
    await tester.pumpAndSettle();
    await tester.scrollUntilVisible(
      find.text('EXACT_LITERAL'),
      150,
      scrollable: find.byType(Scrollable).first,
    );
    expect(find.text('EXACT_LITERAL'), findsOneWidget);
    expect(
      find.textContaining('2 enabled preferences left out'),
      findsOneWidget,
    );
    expect(tester.takeException(), isNull);
  });
}
