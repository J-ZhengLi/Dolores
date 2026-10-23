import 'package:dolores_flutter/chat.dart';
import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';

import 'frost_design_test.dart' show capture, viewport;
import 'support/render_fonts.dart';
import 'worktrees_test.dart'
    show WorktreeBridge, controller, record, panel, visibleTap;

class ArchiveBridge extends WorktreeBridge {
  String? action;
  Map? lifecycleReview;
  int archives = 0, restores = 0;
  bool failRestore = false, failArchive = false, fullStorage = false;
  @override
  Future<dynamic> call(Map<String, dynamic> command) async {
    final request = command['request'];
    if (command['command'] == 'worktrees' && request is Map) {
      final name = request['action'];
      if ([
        'reviewArchive',
        'reviewRestore',
        'archive',
        'restore',
        'inspectLifecycle',
      ].contains(name)) {
        commands.add(command);
        action = name as String;
        if (name == 'reviewArchive' || name == 'reviewRestore') {
          lifecycleReview = {
            'token': 'lifecycle-token',
            'kind': name == 'reviewArchive' ? 'archive' : 'restore',
            'record': retained.single,
            'snapshot': 'b' * 40,
            'directory': 'C:/Dolores/worktree-archives/owned/snapshot',
            'files': 5,
            'directories': 2,
            'bytes': 4096,
            'notice': 'Retain saved files, ignored files and staging. Setup will stay skipped.',
          };
        } else if (name == 'archive') {
          archives++;
        } else if (name == 'restore') {
          restores++;
        }
        return {'job': 'lifecycle-job'};
      }
      if (name == 'poll' && action != null) {
        commands.add(command);
        if (action == 'reviewArchive' || action == 'reviewRestore') {
          return {'done': true, 'value': lifecycleReview};
        }
        if (action == 'archive' && fullStorage) {
          return {
            'done': true,
            'error': 'Local storage is full. Free space on the data drive, refresh Worktrees and review the operation again. Retained files and snapshots remain available for inspection.',
          };
        }
        if (action == 'archive' && failArchive) {
          retained[0] = {
            ...retained.single,
            'state': 'recovery',
            'recoveryFrom': 'archiving',
            'revision': 3,
            'snapshot': 'b' * 40,
          };
          return {
            'done': true,
            'error': 'Archive save failed after removal. Inspect retained snapshot before recovery.',
          };
        }
        if (action == 'restore' && failRestore) {
          retained[0] = {
            ...retained.single,
            'state': 'recovery',
            'recoveryFrom': 'restoring',
            'revision': 5,
          };
          return {
            'done': true,
            'error': 'Restore destination changed. Keep both versions and inspect the retained archive.',
          };
        }
        retained[0] = {
          ...retained.single,
          'state': action == 'archive' || action == 'inspectLifecycle'
              ? 'archived'
              : 'ready',
          'recoveryFrom': null,
          'revision': (retained.single['revision'] as int) + 1,
          'snapshot': 'b' * 40,
        };
        return {
          'done': true,
          'value': {
            'record': retained.single,
            'directory': 'C:/Dolores/worktree-archives/owned/snapshot',
          },
        };
      }
    }
    return super.call(command);
  }
}

Future<ChatController> show(
  WidgetTester t,
  ArchiveBridge bridge, {
  bool dark = false,
  GlobalKey? key,
}) async {
  final chat = controller(bridge);
  await panel(t, chat, dark: dark, captureKey: key);
  await visibleTap(t, 'main · ${bridge.retained.single['state']}');
  return chat;
}

void main() {
  testWidgets(
    'archive and restore need explicit reviewed actions, save draft and use the owning chat',
    (t) async {
      final bridge = ArchiveBridge()..retained.add(record());
      final chat = await show(t, bridge);
      await visibleTap(t, 'Review archive');
      expect(bridge.archives, 0);
      expect(find.text('b' * 40), findsOneWidget);
      await visibleTap(t, 'Archive checkout');
      expect(bridge.archives, 1);
      expect(bridge.restores, 0);
      expect(chat.session, 'source');
      expect(chat.draft, 'Keep my unsent source draft');
      expect(
        bridge.commands.any(
          (c) => c['command'] == 'saveDraft' && c['session'] == 'source',
        ),
        true,
      );
      final applied = bridge.commands.singleWhere(
        (c) =>
            c['command'] == 'worktrees' && c['request']['action'] == 'archive',
      );
      expect(applied['session'], 'owned');
      await visibleTap(t, 'Review restore');
      expect(bridge.restores, 0);
      await visibleTap(t, 'Restore checkout');
      expect(bridge.restores, 1);
      expect(find.text('Checkout · ready'), findsOneWidget);
      expect(bridge.commands.where((c) => c['command'] == 'start'), isEmpty);
      await t.pumpWidget(const SizedBox());
      chat.dispose();
    },
  );
  testWidgets(
    'failed source draft save keeps lifecycle review without consuming it',
    (t) async {
      final bridge = ArchiveBridge()
        ..retained.add(record())
        ..failDraft = true;
      final chat = await show(t, bridge);
      await visibleTap(t, 'Review archive');
      await visibleTap(t, 'Archive checkout');
      expect(find.textContaining('Draft storage unavailable'), findsOneWidget);
      expect(bridge.archives, 0);
      expect(find.text('Archive checkout'), findsOneWidget);
      expect(chat.draft, 'Keep my unsent source draft');
      bridge.failDraft = false;
      await visibleTap(t, 'Archive checkout');
      expect(bridge.archives, 1);
      await t.pumpWidget(const SizedBox());
      chat.dispose();
    },
  );
  testWidgets(
    'retained archive details stay scoped when choosing another checkout',
    (t) async {
      final bridge = ArchiveBridge()..retained.add(record());
      final chat = await show(t, bridge);
      await visibleTap(t, 'Review archive');
      await visibleTap(t, 'Archive checkout');
      expect(find.text('Copy archive folder'), findsOneWidget);
      bridge.retained.add({...record(id: 'second'), 'baseRef': 'second'});
      await visibleTap(t, 'Refresh');
      await visibleTap(t, 'second · ready');
      expect(find.text('Copy archive folder'), findsNothing);
      expect(find.textContaining('C:/Dolores/worktree-archives'), findsNothing);
      await visibleTap(t, 'main · archived');
      expect(find.text('Copy archive folder'), findsOneWidget);
      await t.pumpWidget(const SizedBox());
      chat.dispose();
    },
  );
  testWidgets(
    'full storage retains the checkout and draft, and needs a fresh review after recovery',
    (t) async {
      final bridge = ArchiveBridge()
        ..retained.add(record())
        ..fullStorage = true;
      final chat = await show(t, bridge);
      await visibleTap(t, 'Review archive');
      await visibleTap(t, 'Archive checkout');
      expect(
        find.textContaining('Free space on the data drive'),
        findsOneWidget,
      );
      expect(bridge.retained.single['state'], 'ready');
      expect(bridge.archives, 1);
      expect(bridge.restores, 0);
      expect(chat.draft, 'Keep my unsent source draft');
      await visibleTap(t, 'main · ready');
      bridge.fullStorage = false;
      await visibleTap(t, 'Review archive');
      expect(bridge.archives, 1);
      await visibleTap(t, 'Archive checkout');
      expect(bridge.archives, 2);
      expect(find.text('Checkout · archived'), findsOneWidget);
      await t.pumpWidget(const SizedBox());
      chat.dispose();
    },
  );
  testWidgets(
    'failed lifecycle save exposes inspection then explicit restore with no replay',
    (t) async {
      final bridge = ArchiveBridge()
        ..retained.add(record())
        ..failArchive = true;
      final chat = await show(t, bridge);
      await visibleTap(t, 'Review archive');
      await visibleTap(t, 'Archive checkout');
      expect(find.textContaining('Archive save failed'), findsOneWidget);
      expect(bridge.archives, 1);
      await visibleTap(t, 'main · recovery');
      await visibleTap(t, 'Inspect archive recovery');
      expect(bridge.archives, 1);
      expect(bridge.restores, 0);
      expect(find.text('Checkout · archived'), findsOneWidget);
      await visibleTap(t, 'Review restore');
      expect(bridge.restores, 0);
      await visibleTap(t, 'Keep as is');
      expect(bridge.restores, 0);
      await t.pumpWidget(const SizedBox());
      chat.dispose();
    },
  );
  for (final dark in [false, true]) {
    for (final width in [420.0, 1100.0]) {
      testWidgets(
        'archive review and restore recovery render dark=$dark width=$width',
        (t) async {
          viewport(t, Size(width, 900));
          await t.runAsync(loadReviewFonts);
          final bridge = ArchiveBridge()..retained.add(record());
          final key = GlobalKey();
          final chat = await show(t, bridge, dark: dark, key: key);
          await visibleTap(t, 'Review archive');
          await capture(
            t,
            key,
            'archive-review-${dark ? 'dark' : 'light'}-${width.toInt()}',
          );
          await visibleTap(t, 'Archive checkout');
          await visibleTap(t, 'Review restore');
          bridge.failRestore = true;
          await visibleTap(t, 'Restore checkout');
          expect(
            find.textContaining('Restore destination changed'),
            findsOneWidget,
          );
          expect(bridge.restores, 1);
          await capture(
            t,
            key,
            'archive-restore-recovery-${dark ? 'dark' : 'light'}-${width.toInt()}',
          );
          await visibleTap(t, 'main · recovery');
          await visibleTap(t, 'Review restore continuation');
          expect(bridge.restores, 1);
          expect(find.text('Restore checkout'), findsOneWidget);
          expect(t.takeException(), isNull);
          await t.pumpWidget(const SizedBox());
          chat.dispose();
        },
      );
    }
  }
}
