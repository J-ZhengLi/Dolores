import 'package:dolores_flutter/app_host.dart';
import 'package:dolores_flutter/chat.dart';
import 'package:dolores_flutter/prompt_branches.dart';
import 'package:dolores_flutter/message_frame.dart';

import 'dart:ui' show PointerDeviceKind;

import 'package:dolores_flutter/theme.dart';
import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';

import 'app_host_test.dart' show HostBridge;
import 'frost_design_test.dart' show capture, viewport;
import 'support/render_fonts.dart';
import 'worktrees_test.dart' show WorktreeBridge;

class BranchBridge extends HostBridge {
  final records = <String, Map<String, dynamic>>{};
  final requests = <Map<String, dynamic>>[];
  final drafts = <String, String>{'parent': 'Keep parent draft'};
  bool failPreview = false,
      failSave = false,
      loseSave = false,
      failPage = false;
  String revision = 'a' * 64;
  @override
  Future<dynamic> call(Map<String, dynamic> c) async {
    if (c['command'] == 'bootstrap') {
      return {
        ...await super.call(c) as Map,
        'promptBranches': true,
        'durableDrafts': true,
      };
    }
    if (c['command'] == 'saveDraft') {
      drafts[c['session'] as String] = c['text'] as String;
      return null;
    }
    if (c['command'] == 'savedDraft') return drafts[c['session']] ?? '';
    if (c['command'] != 'promptBranch') return super.call(c);
    requests.add(c);
    final r = c['request'] as Map;
    final owner = c['session'] as String;
    switch (r['action']) {
      case 'preview':
        if (failPreview) {
          throw StateError(
            'Original source is unavailable. Your edit remains.',
          );
        }
        return {
          'basis': {'message': r['message'], 'revision': revision},
          'originalPrompt': 'Original user prompt',
          'attachments': [
            {'name': 'note.txt', 'digest': 'b' * 64},
          ],
          'context': [
            {
              'kind': 'sideExcerpt',
              'source': 'parent message',
              'text': 'A frozen excerpt',
              'revision': 'c' * 64,
              'omissions': 'Other messages',
            },
          ],
          'workspace': {'kind': 'project', 'root': 'C:/public-project'},
          'priorTurns': 1,
          'laterTurns': 2,
          'archiveBytes': 1800,
          'effectTurns': 2,
        };
      case 'info':
        return records[owner];
      case 'create':
        if (failSave) {
          throw StateError(
            'Original conversation changed after preview. Refresh the preview while keeping your edited text.',
          );
        }
        final request = Map<String, dynamic>.from(r['request']);
        final id = request['id'] as String;
        if (records[id] != null &&
            records[id]!['request'].toString() != request.toString()) {
          throw StateError('Branch identity already used.');
        }
        if (records[id] == null) {
          records[id] = {
            'origin': {
              'parent': owner,
              'parentTitle': 'Original task',
              'folder': request['folder'],
            },
            'request': request,
            'parentAvailable': true,
            'sourceAvailable': true,
          };
          drafts[id] = request['edited'] as String;
          roots[id] = request['folder'] == 'shared'
              ? {'kind': 'project', 'root': 'C:/public-project'}
              : {'kind': 'side', 'root': null};
        }
        if (loseSave) throw StateError('Reply lost after saving.');
        return {
          'session': {'id': id},
          'info': records[id],
        };
      case 'archive':
        if (failPage) {
          throw StateError('Snapshot read unavailable. Retry this page.');
        }
        return {
          'text': r['offset'] == 0
              ? 'Historical effect receipt 世界'
              : 'Later retained original answer',
          'offset': r['offset'],
          'nextOffset': r['offset'] == 0 ? 100 : null,
          'totalBytes': 140,
        };
    }
    throw StateError('Unexpected branch request');
  }
}

class ManagedBranchBridge extends BranchBridge {
  final trees = WorktreeBridge();
  bool failTarget = false;
  @override
  Future<dynamic> call(Map<String, dynamic> c) async {
    if (c['command'] == 'worktrees') return trees.call(c);
    if (c['command'] == 'workspace' && trees.roots.containsKey(c['session'])) {
      return trees.call(c);
    }
    if (c['command'] == 'bootstrap') {
      return {...await super.call(c) as Map, 'worktrees': true};
    }
    if (c['command'] == 'promptBranch' && c['request']['action'] == 'target') {
      if (failTarget) {
        throw StateError(
          'This target already has work. Choose a new empty worktree.',
        );
      }
      return trees.retained.singleWhere(
        (r) => r['session'] == c['request']['target'],
      );
    }
    return super.call(c);
  }
}

ChatController parent(BranchBridge bridge) {
  bridge.roots['parent'] = {'kind': 'project', 'root': 'C:/public-project'};
  return ChatController(bridge)
    ..loading = false
    ..session = 'parent'
    ..draft = 'Keep parent draft'
    ..promptBranchesAvailable = true
    ..durableDrafts = true;
}

Future<void> show(
  WidgetTester t,
  ChatController chat, {
  bool dark = false,
  GlobalKey? key,
}) async {
  await t.pumpWidget(
    MaterialApp(
      theme: doloresTheme(dark),
      builder: (_, child) => RepaintBoundary(key: key, child: child!),
      home: Scaffold(
        body: Builder(
          builder: (context) => TextButton(
            onPressed: () => showPromptBranch(context, chat, 7),
            child: const Text('Edit saved prompt'),
          ),
        ),
      ),
    ),
  );
  await t.tap(find.text('Edit saved prompt'));
  await t.pumpAndSettle();
}

Future<void> choose(WidgetTester t, String label) async {
  FocusManager.instance.primaryFocus?.unfocus();
  await t.pumpAndSettle();
  await t.ensureVisible(find.byKey(const Key('branch-folder')));
  await t.pumpAndSettle();
  await t.tap(
    find.descendant(
      of: find.byKey(const Key('branch-folder')),
      matching: find.byType(InkWell),
    ).first,
  );
  await t.pumpAndSettle();
  await t.tap(find.widgetWithText(MenuItemButton, label));
  await t.pumpAndSettle();
}

void main() {
  setUpAll(loadReviewFonts);
  testWidgets(
    'managed picker retains edited text and checkout through target and attachment failure without replaying creation',
    (t) async {
      final bridge = ManagedBranchBridge();
      final chat = parent(bridge)
        ..workspaceKind = 'project'
        ..workspaceRoot = 'C:/public-project'
        ..worktreesAvailable = true;
      String? opened;
      chat.onNavigateConversation = (id) async {
        opened = id;
      };
      await show(t, chat);
      await t.enterText(
        find.byKey(const Key('branch-edit')),
        'Keep my managed prompt',
      );
      await choose(t, 'Managed worktree');
      expect(
        t
            .widget<FilledButton>(
              find.widgetWithText(FilledButton, 'Open edited branch'),
            )
            .onPressed,
        isNull,
      );
      await t.ensureVisible(find.text('Choose or create worktree'));
      await t.tap(find.text('Choose or create worktree'));
      await t.pumpAndSettle();
      expect(find.text('Choose a managed worktree'), findsOneWidget);
      final base = find.ancestor(
        of: find.text('Base branch, tag or commit'),
        matching: find.byType(TextField),
      );
      await t.enterText(base, 'main');
      await t.pump();
      Future<void> tap(String label) async {
        await t.ensureVisible(find.text(label).last);
        await t.pump();
        await t.tap(find.text(label).last);
        await t.pumpAndSettle();
      }

      await tap('Review creation');
      await tap('Create worktree');
      expect(bridge.trees.creates, 1);
      bridge.failTarget = true;
      await tap('Use for edited branch');
      expect(find.textContaining('target already has work'), findsOneWidget);
      expect(opened, isNull);
      bridge.failTarget = false;
      await tap('Use for edited branch');
      expect(find.text('Choose a managed worktree'), findsNothing);
      expect(
        t
            .widget<TextField>(find.byKey(const Key('branch-edit')))
            .controller!
            .text,
        'Keep my managed prompt',
      );
      expect(opened, isNull);
      bridge.failSave = true;
      await tap('Open edited branch');
      expect(bridge.records, isEmpty);
      expect(bridge.trees.retained, hasLength(1));
      expect(bridge.trees.creates, 1);
      bridge.failSave = false;
      await tap('Retry opening saved branch');
      expect(opened, 'owned');
      expect(bridge.records['owned']!['request']['targetRevision'], 2);
      expect(bridge.records['owned']!['request']['folder'], 'managed');
      expect(bridge.trees.creates, 1);
      expect(chat.session, 'parent');
      expect(chat.draft, 'Keep parent draft');
      expect(bridge.commands.where((c) => c['command'] == 'start'), isEmpty);
      await t.pumpWidget(const SizedBox());
      chat.dispose();
    },
  );
  testWidgets(
    'saved user message reveals its named edit action on actual message hover',
    (t) async {
      int opened = 0;
      await t.pumpWidget(
        MaterialApp(
          home: Scaffold(
            body: MessageFrame(
              user: true,
              text: 'Prompt to revise',
              onEditBranch: () => opened++,
              child: const Text('Prompt to revise'),
            ),
          ),
        ),
      );
      final mouse = await t.createGesture(kind: PointerDeviceKind.mouse);
      await mouse.addPointer(location: Offset.zero);
      final region = find
          .descendant(
            of: find.byType(MessageFrame),
            matching: find.byType(MouseRegion),
          )
          .first;
      await mouse.moveTo(t.getCenter(region));
      await t.pumpAndSettle();
      await t.tap(find.byTooltip('Edit in a new branch'));
      await t.pumpAndSettle();
      expect(opened, 1);
      expect(t.takeException(), isNull);
      await mouse.removePointer();
    },
  );
  testWidgets(
    'edited prompt opens another owner without sending and lost-reply recovery preserves both drafts',
    (t) async {
      final bridge = BranchBridge()..loseSave = true;
      final chat = parent(bridge)..busy = true;
      final host = AppHost(chat);
      await show(t, chat);
      expect(
        t
            .widget<FilledButton>(
              find.widgetWithText(FilledButton, 'Open edited branch'),
            )
            .onPressed,
        isNull,
      );
      await t.enterText(
        find.byKey(const Key('branch-edit')),
        'New edited prompt 世界',
      );
      await choose(t, 'No working folder');
      expect(bridge.records, isEmpty);
      await t.tap(find.text('Open edited branch'));
      await t.pumpAndSettle();
      expect(host.visible.session, startsWith('edit-'));
      expect(host.visible.draft, 'New edited prompt 世界');
      expect(host.visible.workspaceRoot, isNull);
      expect(host.visible.promptBranch!['origin']['parent'], 'parent');
      expect(chat.busy, true);
      expect(chat.draft, 'Keep parent draft');
      expect(bridge.commands.where((c) => c['command'] == 'start'), isEmpty);
      final id = host.visible.session!;
      host.visible.draft = 'Newer branch draft';
      await host.visible.checkpointDraft();
      await t.runAsync(
        () => chat.createPromptBranch(
          'parent',
          Map<String, dynamic>.from(bridge.records[id]!['request']),
        ),
      );
      expect(bridge.records, hasLength(1));
      expect(bridge.drafts[id], 'Newer branch draft');
      expect(host.visible.draft, 'Newer branch draft');
      await t.pumpWidget(const SizedBox());
      host.dispose();
    },
  );
  testWidgets(
    'stale preview refresh keeps edited text and deliberate context and folder choices',
    (t) async {
      final bridge = BranchBridge()..failSave = true;
      final chat = parent(bridge);
      String? opened;
      chat.onNavigateConversation = (id) async => opened = id;
      await show(t, chat);
      await t.enterText(
        find.byKey(const Key('branch-edit')),
        'Keep my revised prompt',
      );
      await t.tap(find.text('Include original prompt attachments'));
      await t.tap(find.text('Include current context snapshots'));
      await choose(t, 'Share current folder');
      await t.tap(find.text('Open edited branch'));
      await t.pumpAndSettle();
      expect(find.textContaining('changed after preview'), findsOneWidget);
      expect(opened, isNull);
      final old =
          bridge.requests.firstWhere(
                (c) => c['request']['action'] == 'create',
              )['request']['request']
              as Map;
      bridge
        ..failSave = false
        ..revision = 'd' * 64;
      await t.tap(find.text('Refresh source preview'));
      await t.pumpAndSettle();
      expect(
        t
            .widget<TextField>(find.byKey(const Key('branch-edit')))
            .controller!
            .text,
        'Keep my revised prompt',
      );
      await t.tap(find.text('Open edited branch'));
      await t.pumpAndSettle();
      final saved = bridge.records.values.single['request'] as Map;
      expect(saved['id'], old['id']);
      expect(saved['basis']['revision'], 'd' * 64);
      expect(saved['includeContext'], true);
      expect(saved['includeAttachments'], true);
      expect(saved['folder'], 'shared');
      expect(chat.draft, 'Keep parent draft');
      await t.pumpWidget(const SizedBox());
      chat.dispose();
    },
  );
  testWidgets(
    'failed preview and oversize edit recover without replacing typed text',
    (t) async {
      final bridge = BranchBridge()..failPreview = true;
      final chat = parent(bridge)..onNavigateConversation = (_) async {};
      await show(t, chat);
      expect(
        find.textContaining('Original source is unavailable'),
        findsOneWidget,
      );
      bridge.failPreview = false;
      await t.tap(find.text('Refresh source preview'));
      await t.pumpAndSettle();
      await t.enterText(find.byKey(const Key('branch-edit')), '世' * 6000);
      await choose(t, 'No working folder');
      await t.tap(find.text('Open edited branch'));
      await t.pumpAndSettle();
      expect(find.textContaining('within 16 KiB'), findsOneWidget);
      expect(
        t.widget<TextField>(find.byKey(const Key('branch-edit'))).readOnly,
        false,
      );
      expect(bridge.records, isEmpty);
      await t.enterText(find.byKey(const Key('branch-edit')), 'Short edit');
      await t.tap(find.text('Open edited branch'));
      await t.pumpAndSettle();
      expect(bridge.records.values.single['request']['edited'], 'Short edit');
      await t.pumpWidget(const SizedBox());
      chat.dispose();
    },
  );
  testWidgets(
    'retained snapshot read failure keeps its page and recovery never sends a model request',
    (t) async {
      final bridge = BranchBridge();
      final chat = parent(bridge);
      await t.pumpWidget(
        MaterialApp(
          home: Scaffold(
            body: BranchArchive(chat: chat, owner: 'branch'),
          ),
        ),
      );
      await t.pumpAndSettle();
      expect(find.text('Historical effect receipt 世界'), findsOneWidget);
      bridge.failPage = true;
      await t.tap(find.text('Next page'));
      await t.pumpAndSettle();
      expect(find.textContaining('Snapshot read unavailable'), findsOneWidget);
      expect(find.text('Historical effect receipt 世界'), findsOneWidget);
      bridge.failPage = false;
      await t.tap(find.text('Next page'));
      await t.pumpAndSettle();
      expect(find.text('Later retained original answer'), findsOneWidget);
      await t.tap(find.text('Previous page'));
      await t.pumpAndSettle();
      expect(find.text('Historical effect receipt 世界'), findsOneWidget);
      expect(
        t
            .widget<TextButton>(
              find.widgetWithText(TextButton, 'Previous page'),
            )
            .onPressed,
        isNull,
      );
      expect(bridge.commands.where((c) => c['command'] == 'start'), isEmpty);
      await t.pumpWidget(const SizedBox());
      chat.dispose();
    },
  );
  testWidgets(
    'managed branch choices and picker fit compact Light and Dark layouts',
    (t) async {
      viewport(t, const Size(390, 740));
      for (final dark in [false, true]) {
        final chat = parent(ManagedBranchBridge())
          ..workspaceKind = 'project'
          ..workspaceRoot = 'C:/public-project'
          ..worktreesAvailable = true;
        final key = GlobalKey();
        await show(t, chat, dark: dark, key: key);
        await choose(t, 'Managed worktree');
        await t.ensureVisible(find.text('Choose or create worktree'));
        await t.pumpAndSettle();
        await capture(t, key, 'managed-branch-${dark ? 'dark' : 'light'}');
        await t.tap(find.text('Choose or create worktree'));
        await t.pumpAndSettle();
        await capture(t, key, 'managed-picker-${dark ? 'dark' : 'light'}');
        expect(find.text('Choose a managed worktree'), findsOneWidget);
        expect(t.takeException(), isNull);
        await t.tap(find.text('Close').last);
        await t.pumpAndSettle();
        await t.tap(find.text('Close'));
        await t.pumpAndSettle();
        await t.pumpWidget(const SizedBox());
        chat.dispose();
      }
    },
  );
  testWidgets(
    'branch editor and unavailable-parent banner fit compact Light and Dark layouts',
    (t) async {
      viewport(t, const Size(390, 740));
      for (final dark in [false, true]) {
        final chat = parent(BranchBridge());
        final key = GlobalKey();
        await show(t, chat, dark: dark, key: key);
        await t.enterText(
          find.byKey(const Key('branch-edit')),
          'Revised prompt with current files',
        );
        await choose(t, 'Share current folder');
        await capture(t, key, 'branch-editor-${dark ? 'dark' : 'light'}');
        expect(find.text('Edited prompt'), findsOneWidget);
        expect(t.takeException(), isNull);
        await t.drag(
          find.byType(SingleChildScrollView).first,
          const Offset(0, 450),
        );
        await t.pumpAndSettle();
        await capture(t, key, 'branch-editor-top-${dark ? 'dark' : 'light'}');
        await t.ensureVisible(find.byKey(const Key('branch-folder')));
        await t.pumpAndSettle();
        await capture(
          t,
          key,
          'branch-editor-folder-${dark ? 'dark' : 'light'}',
        );
        await t.tap(find.text('Close'));
        await t.pumpAndSettle();
        chat.promptBranch = {
          'origin': {'parentTitle': 'Original task', 'folder': 'shared'},
          'parentAvailable': false,
          'sourceAvailable': false,
        };
        await t.pumpWidget(
          MaterialApp(
            theme: doloresTheme(dark),
            home: Scaffold(body: PromptBranchBanner(chat: chat)),
          ),
        );
        await t.pumpAndSettle();
        expect(
          t
              .widget<TextButton>(find.byKey(const Key('branch-parent')))
              .onPressed,
          isNull,
        );
        expect(
          find.textContaining('retained snapshot remains readable'),
          findsOneWidget,
        );
        expect(t.takeException(), isNull);
        await t.pumpWidget(const SizedBox());
        chat.dispose();
      }
    },
  );
}
