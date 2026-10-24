import 'package:dolores_flutter/chat.dart';
import 'package:dolores_flutter/history_search.dart';
import 'package:dolores_flutter/main.dart';
import 'package:dolores_flutter/theme.dart';
import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:flutter_test/flutter_test.dart';

import 'app_host_test.dart' show HostBridge;
import 'frost_design_test.dart' show capture, viewport;
import 'support/render_fonts.dart';

class SearchBridge extends HostBridge {
  bool failSearch = false, failEdit = false, failRebuild = false;
  bool paged = false;
  int revision = 0, rebuilds = 0;
  bool archived = false, pinned = false;
  final calls = <Map<String, dynamic>>[];
  Map<String, dynamic> get hit => {
    'session': {'id': 'saved', 'title': 'Retained task 世界', 'updatedAt': 4},
    'workspace': {'kind': 'project', 'root': 'C:/public-project'},
    'organization': {
      'revision': revision,
      'archived': archived,
      'pinned': pinned,
    },
    'excerpt': {
      'message': 9,
      'text': 'Earlier saved solution with 中文预算 and exact evidence',
    },
  };
  @override
  Future<dynamic> call(Map<String, dynamic> command) async {
    calls.add(command);
    if (command['command'] == 'history') {
      final r = command['request'] as Map;
      switch (r['action']) {
        case 'search':
          if (failSearch) {
            throw StateError(
              'Local index unavailable. Rebuild search; history remains.',
            );
          }
          final f = r['query']['filter'];
          if (paged) {
            final second = r['query']['cursor'] != null;
            final item = hit;
            item['session'] = {
              'id': second ? 'second' : 'saved',
              'title': second ? 'Second retained task' : 'First retained task',
              'updatedAt': 4,
            };
            return {
              'items': [item],
              'nextCursor': second
                  ? null
                  : {
                      'filter': f,
                      'pinned': false,
                      'updatedAt': 4,
                      'id': 'saved',
                    },
            };
          }
          final found =
              (f['archive'] == 'all' ||
                  (f['archive'] == 'archived') == archived) &&
              (!(f['pinned'] == true) || pinned);
          return {
            'items': found ? [hit] : [],
            'nextCursor': null,
          };
        case 'organize':
          if (failEdit) {
            throw StateError(
              'Organization changed. Refresh and retry; drafts remain.',
            );
          }
          final o = r['organization'];
          if (o['revision'] != revision) throw StateError('Stale organization');
          revision++;
          archived = o['archived'];
          pinned = o['pinned'];
          return hit['organization'];
        case 'rebuild':
          rebuilds++;
          if (failRebuild) {
            throw StateError(
              'Search rebuild unavailable. Previous index and history remain.',
            );
          }
          failSearch = false;
          return null;
      }
    }
    return super.call(command);
  }
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
            onPressed: () => showHistorySearch(context, chat),
            child: const Text('History'),
          ),
        ),
      ),
    ),
  );
  await t.tap(find.text('History'));
  await t.pumpAndSettle();
}

Future<void> tap(WidgetTester t, String label) async {
  if (find.text(label).evaluate().isEmpty) {
    await t.scrollUntilVisible(
      find.text(label),
      200,
      scrollable: find
          .descendant(
            of: find.byType(ListView).last,
            matching: find.byType(Scrollable),
          )
          .first,
    );
  }
  await t.ensureVisible(find.text(label).last);
  await t.pump();
  await t.tap(find.text(label).last);
  await t.pumpAndSettle();
}

Future<void> archiveFilter(WidgetTester t, String label) async {
  await t.drag(find.byType(ListView).last, const Offset(0, 1500));
  await t.pumpAndSettle();
  await t.ensureVisible(find.byKey(const Key('history-archive-filter')));
  await t.tap(find.byKey(const Key('history-archive-filter')));
  await t.pumpAndSettle();
  await t.tap(find.text(label).last);
  await t.pumpAndSettle();
}

void main() {
  setUpAll(loadReviewFonts);
  testWidgets(
    'Ctrl K opens local history once and leaves an existing modal in place',
    (t) async {
      final b = SearchBridge();
      final c = ChatController(b)
        ..loading = false
        ..historySearchAvailable = true;
      viewport(t, const Size(1100, 900));
      await t.pumpWidget(DoloresApp(chat: c));
      await t.pumpAndSettle();
      await t.tap(find.byKey(const Key('composer-field-0')));
      Future<void> shortcut() async {
        await t.sendKeyDownEvent(LogicalKeyboardKey.controlLeft);
        await t.sendKeyEvent(LogicalKeyboardKey.keyK);
        await t.sendKeyUpEvent(LogicalKeyboardKey.controlLeft);
        await t.pumpAndSettle();
      }

      await shortcut();
      expect(find.byType(HistorySearch), findsOneWidget);
      await shortcut();
      expect(find.byType(HistorySearch), findsOneWidget);
      expect(b.calls.where((v) => v['command'] == 'start'), isEmpty);
      await t.pumpWidget(const SizedBox());
      c.dispose();
    },
  );
  testWidgets(
    'history result paging replaces the page and Search returns to the first page',
    (t) async {
      final b = SearchBridge()..paged = true;
      final c = ChatController(b)
        ..loading = false
        ..session = 'parent';
      await show(t, c);
      await tap(t, 'Next results');
      expect(find.text('Second retained task'), findsOneWidget);
      expect(find.text('First retained task'), findsNothing);
      expect(
        find.text('1 conversation on this page · pinned first'),
        findsOneWidget,
      );
      await tap(t, 'Search');
      expect(find.text('First retained task'), findsOneWidget);
      await t.pumpWidget(const SizedBox());
      c.dispose();
    },
  );
  testWidgets(
    'search pin archive and restore preserve draft and open exact conversation without running a model',
    (t) async {
      final b = SearchBridge();
      final c = ChatController(b)
        ..loading = false
        ..session = 'parent'
        ..draft = 'Keep parent draft';
      String? opened;
      c.onNavigateConversation = (id) async {
        opened = id;
      };
      await show(t, c);
      await t.enterText(find.byKey(const Key('history-search-text')), '中文预算');
      await tap(t, 'Search');
      await tap(t, 'Pin');
      expect(b.pinned, true);
      await tap(t, 'Archive');
      expect(find.text('Retained task 世界'), findsNothing);
      expect(c.draft, 'Keep parent draft');
      await archiveFilter(t, 'Archived conversations');
      await tap(t, 'Search');
      expect(find.text('Retained task 世界'), findsOneWidget);
      await tap(t, 'Unarchive');
      expect(b.archived, false);
      await archiveFilter(t, 'All conversations');
      await tap(t, 'Search');
      await tap(t, 'Open conversation');
      expect(opened, 'saved');
      expect(c.session, 'parent');
      expect(c.draft, 'Keep parent draft');
      expect(b.calls.where((v) => v['command'] == 'start'), isEmpty);
      expect(
        b.calls.where((v) => v['command'] == 'saveDraft').last['text'],
        'Keep parent draft',
      );
      await t.pumpWidget(const SizedBox());
      c.dispose();
    },
  );
  testWidgets(
    'stale organization and failed rebuild retain search text and previous results for explicit recovery',
    (t) async {
      final b = SearchBridge();
      final c = ChatController(b)
        ..loading = false
        ..session = 'parent'
        ..draft = 'Keep draft';
      await show(t, c);
      b.failEdit = true;
      await tap(t, 'Archive');
      expect(find.textContaining('Organization changed.'), findsOneWidget);
      expect(find.text('Retained task 世界'), findsOneWidget);
      expect(b.archived, false);
      await t.drag(find.byType(ListView).last, const Offset(0, 1500));
      await t.pumpAndSettle();
      await t.enterText(find.byKey(const Key('history-search-text')), '中文预算');
      b.failSearch = true;
      await tap(t, 'Search');
      expect(find.textContaining('Local index unavailable.'), findsOneWidget);
      await t.scrollUntilVisible(
        find.text('Retained task 世界'),
        200,
        scrollable: find
            .descendant(
              of: find.byType(ListView).last,
              matching: find.byType(Scrollable),
            )
            .first,
      );
      expect(find.text('Retained task 世界'), findsOneWidget);
      b.failRebuild = true;
      await tap(t, 'Rebuild search');
      expect(find.textContaining('rebuild unavailable.'), findsOneWidget);
      await t.drag(find.byType(ListView).last, const Offset(0, 1500));
      await t.pumpAndSettle();
      expect(
        t
            .widget<TextField>(find.byKey(const Key('history-search-text')))
            .controller!
            .text,
        '中文预算',
      );
      b.failRebuild = false;
      await tap(t, 'Rebuild search');
      expect(b.rebuilds, 2);
      expect(find.textContaining('unavailable.'), findsNothing);
      b.failEdit = false;
      await tap(t, 'Archive');
      expect(b.archived, true);
      expect(c.draft, 'Keep draft');
      await t.pumpWidget(const SizedBox());
      c.dispose();
    },
  );
  testWidgets(
    'compact history filters results and recovery fit Light and Dark',
    (t) async {
      viewport(t, const Size(390, 740));
      for (final dark in [false, true]) {
        final b = SearchBridge();
        final c = ChatController(b)
          ..loading = false
          ..session = 'parent'
          ..workspaceRoot = 'C:/public-project';
        final key = GlobalKey();
        await show(t, c, dark: dark, key: key);
        await capture(t, key, 'history-search-${dark ? 'dark' : 'light'}');
        await t.scrollUntilVisible(
          find.text('Open conversation'),
          200,
          scrollable: find
              .descendant(
                of: find.byType(ListView).last,
                matching: find.byType(Scrollable),
              )
              .first,
        );
        await t.pumpAndSettle();
        await capture(t, key, 'history-results-${dark ? 'dark' : 'light'}');
        expect(t.takeException(), isNull);
        b.failEdit = true;
        await tap(t, 'Archive');
        await t.ensureVisible(find.textContaining('Organization changed.'));
        await t.pumpAndSettle();
        await capture(t, key, 'history-recovery-${dark ? 'dark' : 'light'}');
        expect(t.takeException(), isNull);
        await t.pumpWidget(const SizedBox());
        c.dispose();
      }
    },
  );
}
