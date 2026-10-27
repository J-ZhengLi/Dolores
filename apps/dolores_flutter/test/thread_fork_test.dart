import 'dart:async';
import 'dart:ui' show PointerDeviceKind;

import 'package:dolores_flutter/app_host.dart';
import 'package:dolores_flutter/chat.dart';
import 'package:dolores_flutter/main.dart';
import 'package:dolores_flutter/message_frame.dart';
import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:flutter_test/flutter_test.dart';

import 'history_test.dart' show HistoryBridge;
import 'frost_design_test.dart' show capture, viewport;
import 'support/render_fonts.dart';

class ForkBridge extends HistoryBridge {
  final forks = <Map<String, dynamic>>[];
  final drafts = <String, String>{};
  int? through;
  bool failFork = false, failOpen = false;
  Completer<void>? gate;
  @override
  Future<dynamic> call(Map<String, dynamic> c) async {
    switch (c['command']) {
      case 'saveDraft':
        drafts[c['session'] as String] = c['text'] as String;
        return null;
      case 'savedDraft':
        return drafts[c['session']] ?? '';
      case 'workspace':
        if (c['session'] == 'branched' && failOpen) {
          throw StateError('Saved branch temporarily unavailable.');
        }
        return {'kind': 'project', 'root': 'C:/public-project'};
      case 'forkSession':
        forks.add(Map.of(c));
        if (gate != null) await gate!.future;
        if (failFork) throw StateError('Branch could not be saved. Retry.');
        through = c['through'] as int;
        return {
          'session': {'id': 'branched'},
        };
      case 'bootstrap':
        return {
          ...await super.call(c) as Map,
          'durableDrafts': true,
          'sessions': [
            if (through != null)
              {'id': 'branched', 'title': 'Branch', 'updatedAt': 2000},
            ...sessionItems(0),
          ],
        };
      case 'messagesPage':
        if (c['session'] == 'branched') {
          return {
            'items': messageItems(through!)
                .where((m) => (m['id'] as int) > 0)
                .toList(),
            'hasOlder': false,
            'hasNewer': false,
          };
        }
    }
    return super.call(c);
  }
}

void main() {
  for (final dark in [false, true]) {
    for (final width in [420.0, 1000.0]) {
      testWidgets('branch the visible older reply directly ($dark $width)', (
        t,
      ) async {
        viewport(t, Size(width, 800));
        await t.runAsync(loadReviewFonts);
        final b = ForkBridge();
        final c = ChatController(b);
        await c.initialize();
        await c.browseMessages(newer: false);
        c.draft = 'Keep my unsent parent draft';
        final key = GlobalKey();
        await t.pumpWidget(
          RepaintBoundary(
            key: key,
            child: DoloresApp(
              chat: c,
              themeMode: dark ? ThemeMode.dark : ThemeMode.light,
            ),
          ),
        );
        await t.pumpAndSettle();
        final scroll = t
            .widget<ListView>(find.byType(ListView).last)
            .controller!;
        scroll.jumpTo(0);
        await t.pumpAndSettle();
        final frame = find.byKey(const ValueKey('chat-0:2'));
        await t.ensureVisible(frame);
        final mouse = await t.createGesture(kind: PointerDeviceKind.mouse);
        await mouse.addPointer(location: Offset.zero);
        await mouse.moveTo(t.getCenter(frame));
        await t.pump();
        final more = find.descendant(
          of: frame,
          matching: find.byTooltip('More message actions'),
        );
        try {
          debugDisableShadows = false;
          await t.tap(more);
          await t.pumpAndSettle();
          expect(find.text('Branch in new chat'), findsOneWidget);
          expect(find.text('Fork conversation'), findsNothing);
          await capture(
            t,
            key,
            'reply-branch-${dark ? 'dark' : 'light'}-${width.toInt()}',
          );
        } finally {
          debugDisableShadows = true;
        }
        await t.tap(find.text('Branch in new chat'));
        await t.pumpAndSettle();
        expect(b.forks.single['session'], 'chat-0');
        expect(b.forks.single['through'], 2);
        expect(c.session, 'branched');
        expect(c.messages.last['id'], 2);
        expect(c.draft, isEmpty);
        expect(b.drafts['chat-0'], 'Keep my unsent parent draft');
        expect(b.commands.where((r) => r['command'] == 'start'), isEmpty);
        await c.select('chat-0');
        expect(c.draft, 'Keep my unsent parent draft');
        await mouse.removePointer();
        await t.pumpWidget(const SizedBox.shrink());
        c.dispose();
        expect(t.takeException(), isNull);
      });
    }
  }
  test(
    'fork failures, duplicate clicks and stale sources preserve the parent',
    () async {
      final b = ForkBridge();
      final c = ChatController(b);
      await c.initialize();
      c.draft = 'Retain this draft';
      b.failFork = true;
      await c.forkAt(2, source: 'chat-0');
      expect(c.error, contains('could not be saved'));
      expect(c.session, 'chat-0');
      expect(c.draft, 'Retain this draft');
      expect(c.canFork, true);
      b.failFork = false;
      b.forks.clear();
      b.gate = Completer<void>();
      final pending = c.forkAt(2, source: 'chat-0');
      await Future<void>.delayed(Duration.zero);
      await c.forkAt(2, source: 'chat-0');
      expect(b.forks.length, 1);
      expect(c.canFork, false);
      await c.send();
      expect(b.commands.where((r) => r['command'] == 'start'), isEmpty);
      expect(c.draft, 'Retain this draft');
      b.gate!.complete();
      await pending;
      await c.forkAt(2, source: 'chat-0');
      expect(b.forks.length, 1);
      expect(c.error, contains('source chat changed'));
      c.dispose();
    },
  );
  test('AppHost opens a separate owner and retains the original controller and draft', () async {
    final b = ForkBridge();
    final c = ChatController(b);
    await c.initialize();
    c.draft = 'Parent stays here';
    final host = AppHost(c);
    await c.forkAt(2);
    expect(host.visible.session, 'branched');
    expect(host.visible, same(host.owners.last));
    expect(c.session, 'chat-0');
    expect(c.draft, 'Parent stays here');
    expect(host.visible.draft, isEmpty);
    await host.select('chat-0');
    expect(host.visible, same(c));
    host.dispose();
  });
  test(
    'a saved branch with failed navigation reports its identity without replay',
    () async {
      final b = ForkBridge()..failOpen = true;
      final c = ChatController(b);
      await c.initialize();
      c.draft = 'Keep draft';
      await c.forkAt(2);
      expect(c.session, 'chat-0');
      expect(c.draft, 'Keep draft');
      expect(c.error, contains('Branch saved (branched)'));
      expect(b.forks.length, 1);
      b.failOpen = false;
      await c.navigateConversation('branched');
      expect(c.session, 'branched');
      expect(b.forks.length, 1);
      c.dispose();
    },
  );
  testWidgets(
    'keyboard opens reply branch menu; streaming and user turns have none',
    (t) async {
      var branches = 0;
      await t.pumpWidget(
        MaterialApp(
          home: Scaffold(
            body: MessageFrame(
              user: false,
              text: 'Completed',
              onBranch: () => branches++,
              child: const Text('Completed'),
            ),
          ),
        ),
      );
      for (var i = 0; i < 3; i++) {
        await t.sendKeyEvent(LogicalKeyboardKey.tab);
        await t.pump();
      }
      await t.sendKeyEvent(LogicalKeyboardKey.enter);
      await t.pumpAndSettle();
      expect(find.text('Branch in new chat'), findsOneWidget);
      await t.sendKeyEvent(LogicalKeyboardKey.escape);
      await t.pumpAndSettle();
      expect(branches, 0);
      final b = ForkBridge();
      final c = ChatController(b)
        ..loading = false
        ..session = 'chat-0'
        ..busy = true
        ..pendingInput = 'Prompt'
        ..partial = 'Streaming';
      await t.pumpWidget(DoloresApp(chat: c));
      await t.pump(const Duration(milliseconds: 100));
      expect(find.byTooltip('More message actions'), findsNothing);
      await t.pumpWidget(const SizedBox.shrink());
      c.dispose();
    },
  );
}
