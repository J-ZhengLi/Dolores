import 'dart:async';

import 'package:dolores_flutter/bridge.dart';
import 'package:dolores_flutter/chat.dart';
import 'package:dolores_flutter/instructions.dart';
import 'package:dolores_flutter/inspector.dart';
import 'package:dolores_flutter/theme.dart';
import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';

class InstructionBridge implements ChatBridge {
  final commands = <Map<String, dynamic>>[];
  bool enabled = false, current = false, missing = false, conflict = false;
  Completer<void>? save;
  @override
  Future<void> open() async {}
  @override
  Future<void> close() async {}
  @override
  Future<dynamic> call(Map<String, dynamic> command) async {
    commands.add(command);
    switch (command['command']) {
      case 'reviewInstructions':
        return {
          'enabled': enabled,
          'current': current && !missing,
          'token': missing ? null : 'review-token',
          'text': missing
              ? null
              : '# Rules\n@../private.env\nUse tests.\n${'Long guidance line.\n' * 80}',
          'problem': missing ? 'AGENTS.md is missing or unreadable.' : null,
          'provenance': enabled
              ? {
                  'source': 'AGENTS.md',
                  'revision': 'reviewed-revision',
                  'textBytes': 80,
                  'approvedAt': 1,
                }
              : null,
        };
      case 'enableInstructions':
        if (save != null) await save!.future;
        if (conflict) {
          throw Exception(
            'AGENTS.md changed after review. Refresh and review it again.',
          );
        }
        enabled = true;
        current = true;
        return {'enabled': true};
      case 'disableInstructions':
        enabled = false;
        current = false;
        return null;
      default:
        return null;
    }
  }
}

Future<void> open(
  WidgetTester tester,
  InstructionBridge bridge, {
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
              builder: (_) =>
                  InstructionsInspector(bridge: bridge, session: 'chat'),
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

void main() {
  testWidgets(
    'review stays literal and Close never activates; compact controls remain reachable',
    (tester) async {
      final bridge = InstructionBridge();
      await open(tester, bridge, dark: true);
      expect(find.text('Not enabled'), findsOneWidget);
      await tester.scrollUntilVisible(
        find.byKey(const Key('instruction-source-text')),
        150,
        scrollable: find.byType(Scrollable).first,
      );
      expect(find.byKey(const Key('instruction-source-text')), findsOneWidget);
      final text = tester.widget<SelectableText>(
        find.byKey(const Key('instruction-source-text')),
      );
      expect(text.data, contains('@../private.env'));
      expect(tester.takeException(), isNull);
      await tester.tap(find.text('Close'));
      await tester.pumpAndSettle();
      expect(
        bridge.commands.any((c) => c['command'] == 'enableInstructions'),
        isFalse,
      );
      expect(bridge.commands.last['command'], 'cancelInstructionReview');
    },
  );

  testWidgets(
    'pending enable locks Close and duplicate decisions; stale review requires refresh',
    (tester) async {
      final bridge = InstructionBridge()
        ..save = Completer<void>()
        ..conflict = true;
      await open(tester, bridge);
      await tester.tap(find.byKey(const Key('enable-instructions')));
      await tester.pump();
      expect(
        tester
            .widget<TextButton>(find.widgetWithText(TextButton, 'Close'))
            .onPressed,
        isNull,
      );
      expect(
        tester
            .widget<FilledButton>(find.byKey(const Key('enable-instructions')))
            .onPressed,
        isNull,
      );
      bridge.save!.complete();
      await tester.pumpAndSettle();
      expect(bridge.enabled, isFalse);
      expect(
        tester
            .widget<FilledButton>(find.byKey(const Key('enable-instructions')))
            .onPressed,
        isNull,
      );
      bridge.conflict = false;
      bridge.save = null;
      await tester.tap(find.byKey(const Key('refresh-instructions')));
      await tester.pumpAndSettle();
      await tester.tap(find.byKey(const Key('enable-instructions')));
      await tester.pumpAndSettle();
      expect(find.text('Enabled'), findsOneWidget);
      expect(
        bridge.commands
            .where((c) => c['command'] == 'enableInstructions')
            .length,
        2,
      );
      bridge.missing = true;
      await tester.tap(find.byKey(const Key('refresh-instructions')));
      await tester.pumpAndSettle();
      expect(find.text('Needs review'), findsOneWidget);
      await tester.tap(find.byKey(const Key('disable-instructions')));
      await tester.pumpAndSettle();
      expect(bridge.enabled, isFalse);
      expect(find.text('Not enabled'), findsOneWidget);
      expect(tester.takeException(), isNull);
    },
  );

  testWidgets(
    'folder inspection locks chat switching and clears stale context after closure',
    (tester) async {
      final bridge = InstructionBridge();
      final chat = ChatController(bridge)
        ..loading = false
        ..session = 'chat'
        ..workspaceRoot = 'fixture'
        ..contextSummary = {
          'instructions': {'revision': 'old'},
        };
      final pending = Completer<void>();
      final inspection = chat.inspectLocalChanges(() async {
        await pending.future;
        chat.invalidateContext();
      });
      expect(chat.changing, isTrue);
      chat.newChat(kind: 'side');
      expect(chat.session, 'chat');
      pending.complete();
      await inspection;
      expect(chat.contextSummary, isNull);
      expect(chat.changing, isFalse);
      chat.dispose();
    },
  );

  testWidgets('context inspector exposes reviewed source and revision', (
    tester,
  ) async {
    await tester.pumpWidget(
      MaterialApp(
        theme: doloresTheme(false),
        home: ContextInspector(
          report: {
            'messages': [
              {'role': 'system', 'content': 'Reviewed rules'},
              {'role': 'user', 'content': ''},
            ],
            'instructions': {
              'source': 'AGENTS.md',
              'revision': 'revision-proof',
            },
          },
        ),
      ),
    );
    await tester.scrollUntilVisible(
      find.textContaining('revision-proof'),
      200,
      scrollable: find.byType(Scrollable).first,
    );
    expect(
      find.textContaining('Workspace instructions · AGENTS.md'),
      findsOneWidget,
    );
    expect(tester.takeException(), isNull);
  });
}
