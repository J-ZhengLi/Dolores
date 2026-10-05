import 'package:dolores_flutter/chat.dart';
import 'package:dolores_flutter/chat_details.dart';
import 'package:dolores_flutter/theme.dart';
import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';

import 'session_summary_test.dart';

import 'package:dolores_flutter/session_summary.dart';

class DetailsBridge extends SummaryBridge {
  bool missingContext = false;
  @override
  Future<dynamic> call(Map<String, dynamic> command) async {
    switch (command['command']) {
      case 'context':
        if (missingContext) throw 'Model configuration unavailable';
        return {
          'tokens': {'inputTokens': 120, 'contextWindowTokens': 131072},
          'messages': [],
        };
      case 'messagesPage':
        return {'items': [], 'hasOlder': false, 'hasNewer': false};
      case 'changesPage':
        return {'items': [], 'hasOlder': false};
      default:
        return super.call(command);
    }
  }
}

void main() {
  for (final dark in [false, true]) {
    testWidgets(
      'right panel unifies context, activity and changes in compact theme $dark',
      (t) async {
        t.view.physicalSize = const Size(420, 480);
        t.view.devicePixelRatio = 1;
        addTearDown(t.view.resetPhysicalSize);
        addTearDown(t.view.resetDevicePixelRatio);
        final b = DetailsBridge();
        final chat = ChatController(b)
          ..loading = false
          ..session = 'chat'
          ..workspaceRoot = 'fixture'
          ..draft = 'keep draft';
        addTearDown(chat.dispose);
        await t.pumpWidget(
          MaterialApp(
            theme: doloresTheme(dark),
            home: Builder(
              builder: (context) => Scaffold(
                body: TextButton(
                  onPressed: () => showChatDetails(context, chat),
                  child: const Text('Open'),
                ),
              ),
            ),
          ),
        );
        await t.tap(find.text('Open'));
        await t.pumpAndSettle();
        expect(find.text('120 / 131,072 tokens'), findsOneWidget);
        expect(find.text('Generate draft'), findsNothing);
        await t.tap(find.text('Activity'));
        await t.pumpAndSettle();
        expect(find.text('Earlier tasks'), findsOneWidget);
        await t.tap(find.widgetWithText(ChoiceChip, 'Changes'));
        await t.pumpAndSettle();
        expect(find.text('No file changes recorded yet.'), findsOneWidget);
        await t.tap(find.widgetWithText(ChoiceChip, 'Context'));
        await t.pumpAndSettle();
        expect(chat.draft, 'keep draft');
        expect(t.takeException(), isNull);
      },
    );
  }
  testWidgets(
    'failed context and unsaved summary retain edits across panels and close',
    (t) async {
      final b = DetailsBridge()..missingContext = true;
      b.summary = {
        'text': 'Saved progress',
        'provenance': {
          'revision': 1,
          'coveredTurns': 1,
          'model': 'fixture',
          'updatedAt': 1,
        },
      };
      final chat = ChatController(b)
        ..loading = false
        ..session = 'chat';
      addTearDown(chat.dispose);
      await t.pumpWidget(
        MaterialApp(
          home: Builder(
            builder: (context) => Scaffold(
              body: TextButton(
                onPressed: () => showChatDetails(context, chat),
                child: const Text('Open'),
              ),
            ),
          ),
        ),
      );
      await t.tap(find.text('Open'));
      await t.pumpAndSettle();
      expect(find.text('Context usage unavailable'), findsOneWidget);
      await t.ensureVisible(find.text('Edit'));
      await t.tap(find.text('Edit'));
      await t.pumpAndSettle();
      await t.scrollUntilVisible(
        find.byKey(const Key('summary-text')),
        100,
        scrollable: find
            .descendant(
              of: find.byType(SessionSummaryInspector),
              matching: find.byType(Scrollable),
            )
            .first,
      );
      await t.enterText(
        find.byKey(const Key('summary-text')),
        'Retained correction',
      );
      await t.tap(find.text('Activity'));
      await t.pumpAndSettle();
      b.fail = true;
      await t.tap(find.byKey(const Key('close-chat-details')));
      await t.pumpAndSettle();
      await t.tap(find.text('Save'));
      await t.pumpAndSettle();
      expect(find.byType(ChatDetails), findsOneWidget);
      expect(find.textContaining('Storage failed'), findsWidgets);
      await t.scrollUntilVisible(
        find.byKey(const Key('summary-text')),
        100,
        scrollable: find
            .descendant(
              of: find.byType(SessionSummaryInspector),
              matching: find.byType(Scrollable),
            )
            .first,
      );
      expect(
        t
            .widget<TextField>(find.byKey(const Key('summary-text')))
            .controller!
            .text,
        'Retained correction',
      );
      expect(t.takeException(), isNull);
    },
  );
}
