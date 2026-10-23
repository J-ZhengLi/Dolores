import 'package:dolores_flutter/app_host.dart';
import 'package:dolores_flutter/chat.dart';
import 'package:dolores_flutter/reply_content.dart';
import 'package:dolores_flutter/side_conversations.dart';
import 'package:dolores_flutter/theme.dart';
import 'package:flutter/material.dart';
import 'package:flutter/rendering.dart';
import 'package:flutter_test/flutter_test.dart';

import 'app_host_test.dart' show HostBridge;
import 'frost_design_test.dart' show capture, viewport;
import 'support/render_fonts.dart';

class SideBridge extends HostBridge {
  final links = <String, Map<String, dynamic>>{};
  final sideRequests = <Map<String, dynamic>>[];
  bool failReturn = false, loseCreate = false, loseReturn = false;
  Map<String, dynamic>? returned;
  @override
  Future<dynamic> call(Map<String, dynamic> c) async {
    if (c['command'] == 'bootstrap') {
      return {...await super.call(c) as Map, 'sideConversations': true};
    }
    if (c['command'] != 'sideConversation') return super.call(c);
    sideRequests.add(c);
    final r = c['request'] as Map;
    final owner = c['session'] as String;
    switch (r['action']) {
      case 'preview':
      case 'previewReturn':
        if ((r['excerpt'] as String).length > 8192) {
          throw StateError(
            'Narrow the selection to 8 KiB; original text remains.',
          );
        }
        return {
          'message': r['message'],
          'revision': 'a' * 64,
          'excerpt': r['excerpt'],
        };
      case 'info':
        return links[owner];
      case 'create':
        if (failCreate) throw StateError('Storage is unavailable.');
        final request = r['request'] as Map;
        final id = request['id'] as String;
        roots[id] = {'kind': 'side', 'root': null};
        links.putIfAbsent(
          id,
          () => {
            'origin': {
              'parent': owner,
              'parentTitle': 'Original task',
              'quote': request['quote'],
              'question': request['question'],
              'returns': <Map>[],
            },
            'parentAvailable': true,
            'sourceAvailable': true,
          },
        );
        if (loseCreate) throw StateError('Reply connection lost.');
        return {
          'session': {'id': id},
          'info': links[id],
        };
      case 'bringBack':
        if (failReturn) {
          throw StateError(
            'Parent queue is full. Inspect or cancel a pending input.',
          );
        }
        final request = Map<String, dynamic>.from(r['request']);
        returned ??= request;
        final receipts = links[owner]!['origin']['returns'] as List;
        if (receipts.isEmpty) {
          receipts.add({'id': request['id'], 'mode': request['mode']});
        }
        if (loseReturn) throw StateError('Reply connection lost.');
        return links[owner];
      case 'inspectReturn':
        final request = r['request'] as Map;
        return returned != null && returned.toString() == request.toString()
            ? links[owner]
            : null;
    }
    throw StateError('Unexpected side request');
  }
}

ChatController parent(SideBridge bridge) {
  bridge.roots['parent'] = {'kind': 'project', 'root': 'C:/public-project'};
  return ChatController(bridge)
    ..loading = false
    ..session = 'parent'
    ..sideConversationsAvailable = true
    ..workspaceKind = 'project'
    ..workspaceRoot = 'C:/public-project'
    ..draft = 'Keep my parent draft';
}

Future<void> show(
  WidgetTester t,
  ChatController chat, {
  bool bringBack = false,
  bool dark = false,
  GlobalKey? key,
  String excerpt = 'Rendered selected text',
}) async {
  await t.pumpWidget(
    MaterialApp(
      theme: doloresTheme(dark),
      builder: (_, child) => RepaintBoundary(key: key, child: child!),
      home: Scaffold(
        body: Builder(
          builder: (context) => TextButton(
            onPressed: () => showSideConversation(
              context,
              chat,
              7,
              excerpt,
              bringBack: bringBack,
            ),
            child: const Text('Open selection'),
          ),
        ),
      ),
    ),
  );
  await t.tap(find.text('Open selection'));
  await t.pumpAndSettle();
}

void main() {
  setUpAll(loadReviewFonts);
  test('opening a fresh project from a side chat clears its old link and recovery error', () async {
    final bridge = SideBridge();
    final chat = parent(bridge)
      ..sideConversation = {
        'origin': {'parent': 'old-parent'},
      }
      ..sideConversationError = 'Previous parent is unavailable';
    await chat.openProject('C:/another-public-project');
    expect(chat.workspaceRoot, 'C:/another-public-project');
    expect(chat.sideConversation, isNull);
    expect(chat.sideConversationError, isNull);
    chat.dispose();
  });
  testWidgets(
    'code selection shares the selected word and a long plain reply without a handler has no discussion action',
    (t) async {
      String? selected;
      await t.pumpWidget(
        MaterialApp(
          home: Scaffold(
            body: ReplyContent(
              text: '```text\nalpha target omega\n```',
              onAskAbout: (text) => selected = text,
            ),
          ),
        ),
      );
      await t.pumpAndSettle();
      final finder = find.byType(SelectableText);
      final widget = t.widget<SelectableText>(finder);
      final editable = t
          .state<EditableTextState>(
            find.descendant(of: finder, matching: find.byType(EditableText)),
          )
          .renderEditable;
      expect(widget.data ?? widget.textSpan!.toPlainText(), contains('target'));
      final box = editable.getLocalRectForCaret(const TextPosition(offset: 9));
      await t.longPressAt(editable.localToGlobal(box.center));
      await t.pumpAndSettle();
      await t.tap(find.text('Ask about this'));
      await t.pumpAndSettle();
      expect(selected, 'target');
      await t.pumpWidget(
        MaterialApp(
          home: Scaffold(
            body: SingleChildScrollView(
              child: ReplyContent(text: 'plain ' * 6000),
            ),
          ),
        ),
      );
      await t.pumpAndSettle();
      final plain = find.byType(SelectableText).last;
      await t.longPressAt(t.getTopLeft(plain) + const Offset(15, 10));
      await t.pumpAndSettle();
      expect(find.text('Ask about this'), findsNothing);
      expect(t.takeException(), isNull);
    },
  );
  testWidgets(
    'oversized question remains editable and can be shortened before creating a discussion',
    (t) async {
      final bridge = SideBridge();
      final chat = parent(bridge);
      chat.onNavigateConversation = (_) async {};
      await show(t, chat);
      await t.enterText(find.byKey(const Key('side-question')), '世' * 1400);
      await t.tap(find.text('Open side chat'));
      await t.pumpAndSettle();
      expect(
        find.textContaining('Shorten the question to 4 KiB'),
        findsOneWidget,
      );
      expect(bridge.links, isEmpty);
      expect(
        t.widget<TextField>(find.byKey(const Key('side-question'))).readOnly,
        false,
      );
      await t.enterText(
        find.byKey(const Key('side-question')),
        'Narrow question',
      );
      await t.tap(find.text('Open side chat'));
      await t.pumpAndSettle();
      expect(
        bridge.links.values.single['origin']['question'],
        'Narrow question',
      );
      await t.pumpWidget(const SizedBox());
      chat.dispose();
    },
  );
  testWidgets(
    'rendered Markdown selection offers only the selected words for discussion',
    (t) async {
      String? selected;
      await t.pumpWidget(
        MaterialApp(
          home: Scaffold(
            body: ReplyContent(
              text: 'Alpha target omega',
              onAskAbout: (text) => selected = text,
            ),
          ),
        ),
      );
      await t.pumpAndSettle();
      final paragraph = t.renderObject<RenderParagraph>(
        find.text('Alpha target omega', findRichText: true),
      );
      final box = paragraph
          .getBoxesForSelection(
            const TextSelection(baseOffset: 6, extentOffset: 12),
          )
          .single
          .toRect();
      await t.longPressAt(paragraph.localToGlobal(box.center));
      await t.pumpAndSettle();
      await t.tap(find.text('Ask about this'));
      await t.pumpAndSettle();
      expect(selected, 'target');
      expect(t.takeException(), isNull);
    },
  );
  testWidgets(
    'side selection opens a separate owner while the parent keeps running and its draft',
    (t) async {
      final bridge = SideBridge()..loseCreate = true;
      final chat = parent(bridge)..busy = true;
      final host = AppHost(chat);
      await show(t, chat);
      await t.enterText(
        find.byKey(const Key('side-question')),
        'Explain this phrase',
      );
      expect(bridge.links, isEmpty);
      await t.tap(find.text('Open side chat'));
      await t.pumpAndSettle();
      expect(host.visible.session, startsWith('side-'));
      expect(host.visible.workspaceRoot, isNull);
      expect(chat.busy, true);
      expect(chat.draft, 'Keep my parent draft');
      expect(
        bridge.links.values.single['origin']['question'],
        'Explain this phrase',
      );
      expect(bridge.commands.where((c) => c['command'] == 'start'), isEmpty);
      await host.visible.navigateConversation('parent');
      expect(identical(host.visible, chat), true);
      await t.pumpWidget(const SizedBox());
      host.dispose();
    },
  );
  testWidgets(
    'failed creation keeps excerpt and question and retries the same identity',
    (t) async {
      final bridge = SideBridge()..failCreate = true;
      final chat = parent(bridge);
      String? destination;
      chat.onNavigateConversation = (s) async => destination = s;
      await show(t, chat);
      await t.enterText(
        find.byKey(const Key('side-question')),
        'Keep my question 世界',
      );
      await t.tap(find.text('Open side chat'));
      await t.pumpAndSettle();
      expect(find.textContaining('Storage is unavailable'), findsOneWidget);
      expect(
        t
            .widget<TextField>(find.byKey(const Key('side-question')))
            .controller!
            .text,
        'Keep my question 世界',
      );
      final attempt = bridge.sideRequests.singleWhere(
        (c) => c['request']['action'] == 'create',
      )['request']['request'];
      bridge.failCreate = false;
      await t.tap(find.text('Retry opening saved discussion'));
      await t.pumpAndSettle();
      expect(destination, attempt['id']);
      expect(
        bridge.sideRequests
            .where((c) => c['request']['action'] == 'create')
            .last['request']['request'],
        attempt,
      );
      expect(chat.draft, 'Keep my parent draft');
      await t.pumpWidget(const SizedBox());
      chat.dispose();
    },
  );
  testWidgets(
    'full queue preserves return selection, lost reply inspects exactly one receipt',
    (t) async {
      final bridge = SideBridge()..failReturn = true;
      final chat = parent(bridge)..session = 'child';
      bridge.links['child'] = {
        'origin': {
          'parent': 'parent',
          'parentTitle': 'Original task',
          'quote': {'excerpt': 'frozen'},
          'returns': <Map>[],
        },
        'parentAvailable': true,
        'sourceAvailable': true,
      };
      int delivered = 0;
      chat.onSideConclusion = (s) async {
        expect(s, 'parent');
        delivered++;
      };
      await show(t, chat, bringBack: true);
      expect(bridge.returned, isNull);
      await t.tap(find.text('Steer parent'));
      await t.pumpAndSettle();
      expect(find.textContaining('Parent queue is full'), findsOneWidget);
      expect(
        t
            .widget<TextField>(find.byKey(const Key('side-excerpt')))
            .controller!
            .text,
        'Rendered selected text',
      );
      expect(delivered, 0);
      bridge
        ..failReturn = false
        ..loseReturn = true;
      await t.tap(find.text('Retry steering conclusion'));
      await t.pumpAndSettle();
      expect(delivered, 1);
      expect(bridge.links['child']!['origin']['returns'], hasLength(1));
      expect(bridge.sideRequests.last['request']['action'], 'inspectReturn');
      final conflicting = {
        ...bridge.returned!,
        'quote': {
          ...bridge.returned!['quote'] as Map,
          'excerpt': 'Different text',
        },
      };
      await expectLater(
        chat.returnSide('child', conflicting),
        throwsStateError,
      );
      expect(delivered, 1);
      await t.pumpWidget(const SizedBox());
      chat.dispose();
    },
  );
  testWidgets(
    'compact selection and missing-parent recovery render in both themes',
    (t) async {
      viewport(t, const Size(390, 740));
      for (final dark in [false, true]) {
        final chat = parent(SideBridge());
        final key = GlobalKey();
        await show(t, chat, dark: dark, key: key);
        await t.enterText(
          find.byKey(const Key('side-question')),
          'What does this mean?',
        );
        expect(find.text('Selected excerpt'), findsOneWidget);
        expect(find.text('Question draft (optional)'), findsOneWidget);
        await t.pumpAndSettle();
        await capture(t, key, 'side-selection-${dark ? 'dark' : 'light'}');
        expect(t.takeException(), isNull);
        await t.tap(find.text('Close'));
        await t.pumpAndSettle();
        chat.sideConversation = {
          'origin': {
            'parent': 'missing',
            'parentTitle': 'Original task',
            'quote': {'excerpt': 'Frozen excerpt remains'},
            'returns': <Map>[],
          },
          'parentAvailable': false,
          'sourceAvailable': false,
        };
        await t.pumpWidget(
          MaterialApp(
            theme: doloresTheme(dark),
            home: Scaffold(body: SideConversationBanner(chat: chat)),
          ),
        );
        await t.pumpAndSettle();
        expect(
          t
              .widget<TextButton>(find.byKey(const Key('side-return-parent')))
              .onPressed,
          isNull,
        );
        expect(t.takeException(), isNull);
        await t.pumpWidget(const SizedBox());
        chat.dispose();
      }
    },
  );
}
