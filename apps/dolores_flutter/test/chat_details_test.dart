import 'package:dolores_flutter/chat.dart';
import 'package:dolores_flutter/chat_details.dart';
import 'package:dolores_flutter/theme.dart';
import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';

import 'session_summary_test.dart';

import 'package:dolores_flutter/session_summary.dart';

import 'frost_design_test.dart' show capture, viewport;
import 'support/render_fonts.dart';

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
  testWidgets('closing active details keeps the composer context snapshot', (
    t,
  ) async {
    final chat = ChatController(DetailsBridge())
      ..loading = false
      ..busy = true
      ..session = 'chat'
      ..contextSummary = {
        'tokens': {'inputTokens': 16479, 'contextWindowTokens': 500000},
      }
      ..contextBasis = 'Current model request · estimated';
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
    await t.tap(find.byKey(const Key('close-chat-details')));
    await t.pumpAndSettle();
    expect(chat.contextSummary!['tokens']['inputTokens'], 16479);
    expect(chat.contextBasis, 'Current model request · estimated');
    expect(chat.busy, true);
    await t.pumpWidget(const SizedBox());
    chat.dispose();
  });
  for (final dark in [false, true]) {
    for (final width in [420.0, 1000.0]) {
      testWidgets('active context usage fits $dark $width', (t) async {
        viewport(t, Size(width, 800));
        await t.runAsync(loadReviewFonts);
        final chat = ChatController(DetailsBridge())
          ..loading = false
          ..busy = true
          ..session = 'chat'
          ..contextSummary = {
            'tokens': {'inputTokens': 16479, 'contextWindowTokens': 500000},
          }
          ..contextBasis = 'Current model request · estimated';
        final key = GlobalKey();
        await t.pumpWidget(
          MaterialApp(
            theme: doloresTheme(dark),
            home: Scaffold(
              body: RepaintBoundary(
                key: key,
                child: ChatDetails(chat: chat),
              ),
            ),
          ),
        );
        await t.pumpAndSettle();
        expect(find.text('16,479 / 500,000 tokens'), findsOneWidget);
        await capture(
          t,
          key,
          'active-context-${dark ? "dark" : "light"}-${width.toInt()}',
        );
        expect(t.takeException(), isNull);
        await t.pumpWidget(const SizedBox());
        chat.dispose();
      });
    }
  }
  testWidgets(
    'active context remains inspectable without preparing a competing request',
    (t) async {
      final bridge = DetailsBridge();
      final chat = ChatController(bridge)
        ..loading = false
        ..busy = true
        ..session = 'chat'
        ..draft = 'preserve draft'
        ..contextSummary = {
          'tokens': {'inputTokens': 16479, 'contextWindowTokens': 500000},
        }
        ..contextBasis = 'Initial agent input';
      await t.pumpWidget(
        MaterialApp(
          home: Scaffold(body: ContextHome(chat: chat)),
        ),
      );
      await t.pumpAndSettle();
      expect(find.text('16,479 / 500,000 tokens'), findsOneWidget);
      expect(find.text('Context usage unavailable'), findsNothing);
      expect(chat.busy, true);
      expect(chat.draft, 'preserve draft');
      chat.contextSummary = {
        'tokens': {'inputTokens': 22479, 'contextWindowTokens': 500000},
      };
      chat.contextBasis = 'Current model request · estimated';
      chat.notifyListeners();
      await t.pumpAndSettle();
      expect(find.text('22,479 / 500,000 tokens'), findsOneWidget);
      await t.tap(find.byTooltip('Refresh context'));
      await t.pumpAndSettle();
      expect(find.text('22,479 / 500,000 tokens'), findsOneWidget);
      await t.pumpWidget(const SizedBox());
      chat.dispose();
    },
  );
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
