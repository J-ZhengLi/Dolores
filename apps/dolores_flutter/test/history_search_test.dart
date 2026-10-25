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

void main() {
  setUpAll(loadReviewFonts);
  testWidgets(
    'Ctrl K opens a single compact search without replacing another modal',
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
      expect(find.byType(TextField), findsNWidgets(2)); // Composer plus search.
      expect(b.calls.where((v) => v['command'] == 'start'), isEmpty);
      await t.sendKeyEvent(LogicalKeyboardKey.escape);
      await t.pumpAndSettle();
      expect(find.byType(HistorySearch), findsNothing);
      await t.pumpWidget(const SizedBox());
      c.dispose();
    },
  );

  for (final dark in [false, true]) {
    for (final width in [390.0, 1100.0]) {
      testWidgets(
        'search starts as one input and opens a saved result ($dark, $width)',
        (t) async {
          debugDisableShadows = false;
          viewport(t, Size(width, 740));
          final b = SearchBridge();
          final c = ChatController(b)
            ..loading = false
            ..session = 'parent'
            ..draft = 'Keep parent draft';
          String? opened;
          c.onNavigateConversation = (id) async => opened = id;
          final key = GlobalKey();
          await show(t, c, dark: dark, key: key);
          expect(find.byType(TextField), findsOneWidget);
          expect(find.text('More filters'), findsNothing);
          expect(find.text('Rebuild search'), findsNothing);
          expect(b.calls.where((v) => v['command'] == 'history'), isEmpty);
          expect(t.getSize(find.byType(HistorySearch)).height, lessThan(100));
          await capture(
            t,
            key,
            'search-empty-${dark ? 'dark' : 'light'}-$width',
          );
          await t.enterText(
            find.byKey(const Key('history-search-text')),
            'saved',
          );
          await t.pump(const Duration(milliseconds: 300));
          await t.pumpAndSettle();
          expect(find.text('Retained task 世界'), findsOneWidget);
          expect(
            b.calls.lastWhere(
              (v) => v['command'] == 'history',
            )['request']['query']['filter']['archive'],
            'all',
          );
          await capture(
            t,
            key,
            'search-results-${dark ? 'dark' : 'light'}-$width',
          );
          await t.tap(find.byKey(const Key('history-result-saved')));
          await t.pumpAndSettle();
          expect(opened, 'saved');
          expect(c.draft, 'Keep parent draft');
          expect(b.calls.where((v) => v['command'] == 'start'), isEmpty);
          expect(find.byType(HistorySearch), findsNothing);
          debugDisableShadows = true;
          expect(t.takeException(), isNull);
          await t.pumpWidget(const SizedBox());
          c.dispose();
        },
      );
    }
  }

  testWidgets('failed search preserves the query and retries with Enter', (
    t,
  ) async {
    final b = SearchBridge()..failSearch = true;
    final c = ChatController(b)..loading = false;
    await show(t, c);
    await t.enterText(find.byKey(const Key('history-search-text')), 'budget');
    await t.pump(const Duration(milliseconds: 300));
    await t.pumpAndSettle();
    expect(
      find.text('Search unavailable. Press Enter to retry.'),
      findsOneWidget,
    );
    expect(
      t.widget<TextField>(find.byType(TextField)).controller!.text,
      'budget',
    );
    b.failSearch = false;
    await t.testTextInput.receiveAction(TextInputAction.done);
    await t.pumpAndSettle();
    expect(find.text('Retained task 世界'), findsOneWidget);
    await t.pumpWidget(const SizedBox());
    c.dispose();
  });

  testWidgets('search paging is bounded and typing returns to the first page', (
    t,
  ) async {
    final b = SearchBridge()..paged = true;
    final c = ChatController(b)..loading = false;
    await show(t, c);
    await t.enterText(find.byType(TextField), 'saved');
    await t.pump(const Duration(milliseconds: 300));
    await t.pumpAndSettle();
    await t.tap(find.text('More results'));
    await t.pumpAndSettle();
    expect(find.text('Second retained task'), findsOneWidget);
    expect(find.text('First retained task'), findsNothing);
    await t.enterText(find.byType(TextField), 'new');
    await t.pump(const Duration(milliseconds: 300));
    await t.pumpAndSettle();
    expect(find.text('First retained task'), findsOneWidget);
    await t.pumpWidget(const SizedBox());
    c.dispose();
  });
}
