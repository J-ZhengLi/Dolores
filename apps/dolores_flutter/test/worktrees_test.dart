import 'package:dolores_flutter/chat.dart';
import 'package:dolores_flutter/main.dart';
import 'package:dolores_flutter/theme.dart';
import 'package:dolores_flutter/worktrees.dart';
import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';

import 'frost_design_test.dart' show capture, viewport;
import 'support/render_fonts.dart';
import 'workspace_test.dart' show WorkspaceBridge;

Map<String, dynamic> record({String id = 'owned', String state = 'ready'}) => {
  'id': id,
  'session': id,
  'revision': 2,
  'sourceRoot': 'C:/projects/public-fixture',
  'root': 'C:/Dolores/managed-worktrees/$id',
  'baseRef': 'main',
  'baseCommit': 'a' * 40,
  'setup': 'skip',
  'state': state,
  'recoveryFrom': state == 'recovery' ? 'creating' : null,
  'notice': state == 'recovery'
      ? 'Saving Ready failed. Retained work is available for inspection.'
      : 'Checkout ready. Setup was skipped.',
};

class WorktreeBridge extends WorkspaceBridge {
  final retained = <Map<String, dynamic>>[];
  Map<String, dynamic>? jobValue;
  String? jobAction;
  int sequence = 0, creates = 0;
  bool failCreateWorktree = false, failDraft = false, failStatusOnce = false;
  bool waitForCancel = false, cancelled = false, failRecovery = false;
  @override
  Future<dynamic> call(Map<String, dynamic> command) async {
    if (command['command'] == 'saveDraft' && failDraft) {
      commands.add(command);
      throw StateError('Draft storage unavailable. Keep your text and retry.');
    }
    if (command['command'] == 'bootstrap') {
      return {...await super.call(command) as Map, 'worktrees': true};
    }
    if (command['command'] != 'worktrees') return super.call(command);
    commands.add(command);
    final request = command['request'] as Map;
    switch (request['action']) {
      case 'list':
        return {'items': retained, 'nextCursor': null};
      case 'inspect':
        return retained
            .where((r) => r['session'] == command['session'])
            .firstOrNull;
      case 'discardReview':
        return null;
      case 'reviewCreate':
        jobAction = 'reviewCreate';
        jobValue = {
          'token': 'one-use-review',
          'record': {
            ...record(),
            'baseRef': request['base'],
            'setup': request['setup'],
          },
        };
        return {'job': 'job-${++sequence}'};
      case 'create':
        creates++;
        jobAction = 'create';
        final saved = record(
          state: failCreateWorktree || waitForCancel ? 'recovery' : 'ready',
        );
        retained.add(saved);
        roots['owned'] = {'kind': 'project', 'root': saved['root']};
        rows.add({
          'id': 'owned',
          'title': 'Worktree chat',
          'updatedAt': 2,
          'workspace': roots['owned'],
        });
        jobValue = {'record': saved, 'session': 'owned'};
        return {'job': 'job-${++sequence}'};
      case 'reconcileCreation':
        jobAction = 'reconcileCreation';
        if (!failRecovery) {
          retained[0] = record();
          jobValue = {'record': retained[0], 'session': 'owned'};
        }
        return {'job': 'job-${++sequence}'};
      case 'cancel':
        cancelled = true;
        return null;
      case 'poll':
        if (failStatusOnce && jobAction == 'create') {
          failStatusOnce = false;
          throw StateError(
            'Status unavailable. Check status before creating another checkout.',
          );
        }
        if (waitForCancel && jobAction == 'create' && !cancelled) {
          return {'done': false};
        }
        if (failCreateWorktree && jobAction == 'create') {
          return {
            'done': true,
            'error':
                'Disk full. Inspect the retained worktree before retrying.',
          };
        }
        if (failRecovery && jobAction == 'reconcileCreation') {
          return {
            'done': true,
            'error': 'Folder identity changed. Keep it for manual recovery.',
          };
        }
        if (cancelled) {
          return {
            'done': true,
            'error': 'Stopped. Retained target requires inspection.',
          };
        }
        return {'done': true, 'value': jobValue};
    }
    throw StateError('Unexpected worktree action: $request');
  }
}

ChatController controller(WorktreeBridge bridge) {
  bridge.roots['source'] = {
    'kind': 'project',
    'root': 'C:/projects/public-fixture',
  };
  return ChatController(bridge)
    ..loading = false
    ..session = 'source'
    ..workspaceKind = 'project'
    ..workspaceRoot = 'C:/projects/public-fixture'
    ..worktreesAvailable = true
    ..draft = 'Keep my unsent source draft';
}

Future<void> panel(
  WidgetTester t,
  ChatController chat, {
  bool dark = false,
  GlobalKey? captureKey,
}) async {
  await t.pumpWidget(
    RepaintBoundary(
      key: captureKey,
      child: MaterialApp(
        theme: doloresTheme(dark),
        home: Scaffold(
          body: Builder(
            builder: (context) => TextButton(
              onPressed: () => showWorktrees(context, chat),
              child: const Text('Open worktrees'),
            ),
          ),
        ),
      ),
    ),
  );
  await t.tap(find.text('Open worktrees'));
  await t.pumpAndSettle();
}

Future<void> review(WidgetTester t) async {
  await t.enterText(find.byType(TextField), 'main');
  await t.pump();
  await t.tap(find.text('Review creation'));
  await t.pumpAndSettle();
}

Future<void> visibleTap(WidgetTester t, String text) async {
  await t.ensureVisible(find.text(text));
  await t.pump();
  await t.tap(find.text(text));
  await t.pumpAndSettle();
}

void main() {
  testWidgets(
    'reviews exact checkout, creates once and opens without losing the source draft',
    (t) async {
      final bridge = WorktreeBridge();
      final chat = controller(bridge);
      await panel(t, chat);
      await review(t);
      expect(find.text('a' * 40), findsOneWidget);
      expect(bridge.creates, 0);
      expect(chat.session, 'source');
      await visibleTap(t, 'Create worktree');
      expect(bridge.creates, 1);
      expect(chat.session, 'source');
      expect(chat.draft, 'Keep my unsent source draft');
      await visibleTap(t, 'Open chat');
      expect(chat.session, 'owned');
      expect(chat.workspaceRoot, 'C:/Dolores/managed-worktrees/owned');
      expect(find.byType(WorktreesInspector), findsNothing);
      await chat.select('source');
      expect(chat.draft, 'Keep my unsent source draft');
      expect(bridge.commands.where((c) => c['command'] == 'start'), isEmpty);
      await t.pumpWidget(const SizedBox());
      chat.dispose();
    },
  );

  testWidgets(
    'creation failure retains intent, draft and inspection recovery with no replay',
    (t) async {
      final bridge = WorktreeBridge()..failCreateWorktree = true;
      final chat = controller(bridge);
      await panel(t, chat);
      await review(t);
      await visibleTap(t, 'Create worktree');
      expect(find.textContaining('Disk full.'), findsOneWidget);
      expect(bridge.creates, 1);
      await visibleTap(t, 'main · recovery');
      await visibleTap(t, 'Inspect existing checkout');
      expect(
        bridge.commands.lastWhere(
          (c) =>
              c['command'] == 'worktrees' &&
              c['request']['action'] == 'reconcileCreation',
        )['session'],
        'owned',
      );
      expect(bridge.creates, 1);
      expect(chat.draft, 'Keep my unsent source draft');
      expect(find.text('Checkout · ready'), findsOneWidget);
      await t.pumpWidget(const SizedBox());
      chat.dispose();
    },
  );

  testWidgets(
    'uncertain status only polls the same job and refuses duplicate creation',
    (t) async {
      final bridge = WorktreeBridge()..failStatusOnce = true;
      final chat = controller(bridge);
      await panel(t, chat);
      await review(t);
      await t.tap(find.text('Create worktree'));
      for (
        var i = 0;
        i < 20 && find.textContaining('Status unavailable.').evaluate().isEmpty;
        i++
      ) {
        await t.pump(const Duration(milliseconds: 50));
      }
      expect(find.textContaining('Status unavailable.'), findsOneWidget);
      expect(
        t
            .widget<TextButton>(find.widgetWithText(TextButton, 'Close'))
            .onPressed,
        isNull,
      );
      await visibleTap(t, 'Check status');
      expect(bridge.creates, 1);
      final polls = bridge.commands.where(
        (c) =>
            c['command'] == 'worktrees' &&
            c['request']['action'] == 'poll' &&
            c['request']['job'] == 'job-2',
      );
      expect(polls.length, 2);
      expect(chat.session, 'source');
      await t.pumpWidget(const SizedBox());
      chat.dispose();
    },
  );

  testWidgets(
    'Stop waits for the owned result and keeps interrupted files inspectable',
    (t) async {
      final bridge = WorktreeBridge()..waitForCancel = true;
      final chat = controller(bridge);
      await panel(t, chat);
      await review(t);
      await t.tap(find.text('Create worktree'));
      await t.pump();
      await t.pump();
      await t.tap(find.text('Stop operation'));
      await t.pump();
      await t.pump(const Duration(milliseconds: 300));
      await t.pumpAndSettle();
      expect(bridge.cancelled, isTrue);
      expect(find.textContaining('Stopped. Retained'), findsOneWidget);
      expect(find.text('main · recovery'), findsOneWidget);
      expect(bridge.creates, 1);
      await t.pumpWidget(const SizedBox());
      chat.dispose();
    },
  );

  testWidgets(
    'failed draft save prevents switching and preserves the ready checkout',
    (t) async {
      final bridge = WorktreeBridge()
        ..retained.add(record())
        ..failDraft = true;
      final chat = controller(bridge);
      await panel(t, chat);
      await visibleTap(t, 'main · ready');
      await visibleTap(t, 'Open chat');
      expect(find.textContaining('Draft storage unavailable.'), findsOneWidget);
      expect(chat.session, 'source');
      expect(chat.draft, 'Keep my unsent source draft');
      expect(bridge.retained.single['state'], 'ready');
      expect(bridge.creates, 0);
      await t.pumpWidget(const SizedBox());
      chat.dispose();
    },
  );

  for (final dark in [false, true]) {
    for (final width in [420.0, 1100.0]) {
      testWidgets(
        'worktree review and recovery render dark=$dark width=$width',
        (t) async {
          viewport(t, Size(width, 900));
          await t.runAsync(loadReviewFonts);
          final bridge = WorktreeBridge()
            ..retained.add(record(state: 'recovery'))
            ..failRecovery = true;
          final chat = controller(bridge);
          final key = GlobalKey();
          await panel(t, chat, dark: dark, captureKey: key);
          await review(t);
          await capture(
            t,
            key,
            'worktree-review-${dark ? 'dark' : 'light'}-${width.toInt()}',
          );
          await visibleTap(t, 'Change choices');
          await visibleTap(t, 'main · recovery');
          await visibleTap(t, 'Inspect existing checkout');
          expect(
            find.textContaining('Folder identity changed.'),
            findsOneWidget,
          );
          await capture(
            t,
            key,
            'worktree-recovery-${dark ? 'dark' : 'light'}-${width.toInt()}',
          );
          expect(bridge.creates, 0);
          expect(chat.draft, 'Keep my unsent source draft');
          expect(t.takeException(), isNull);
          await t.pumpWidget(const SizedBox());
          chat.dispose();
        },
      );
    }
  }

  testWidgets('normal chat actions expose the worktree panel', (t) async {
    final bridge = WorktreeBridge();
    final chat = controller(bridge);
    await t.pumpWidget(DoloresApp(chat: chat));
    await t.tap(find.byKey(const Key('export-chat')));
    await t.pumpAndSettle();
    await t.tap(find.text('Worktrees…'));
    await t.pumpAndSettle();
    expect(find.byType(WorktreesInspector), findsOneWidget);
    expect(chat.draft, 'Keep my unsent source draft');
    await t.pumpWidget(const SizedBox());
    chat.dispose();
  });
}
