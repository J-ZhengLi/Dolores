import 'dart:async';

import 'package:dolores_flutter/bridge.dart';
import 'package:dolores_flutter/git_host.dart';
import 'package:dolores_flutter/source_control.dart';
import 'package:dolores_flutter/theme.dart';
import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';

import 'frost_design_test.dart' show capture, viewport;
import 'support/render_fonts.dart';

class ReviewBridge implements ChatBridge {
  final records = <String, Map<String, dynamic>>{};
  final requests = <Map<String, dynamic>>[];
  final submissions = <String, Map>{};
  Map<String, dynamic>? operation;
  bool failCapture = false,
      failSubmit = false,
      losePoll = false,
      holdResult = false;
  int captures = 0, submitted = 0;
  Completer<void>? listing;
  @override
  Future<void> open() async {}
  @override
  Future<void> close() async {}
  @override
  Future<dynamic> call(Map<String, dynamic> c) async {
    requests.add(c);
    final r = c['request'] as Map;
    if (c['command'] == 'reviewFindings') {
      switch (r['action']) {
        case 'list':
          await listing?.future;
          return {'items': records.values.toList(), 'nextCursor': null};
        case 'get':
          return holdResult ? null : records[r['id']];
        case 'submission':
          return holdResult ? null : submissions[r['id']];
        case 'remove':
          records.remove(r['id']);
          return null;
        case 'reopen':
          records[r['id']] = {...records[r['id']]!, 'resolution': null};
          return null;
      }
    }
    if (r['action'] == 'finding') {
      operation = Map<String, dynamic>.from(r['operation']);
      if (operation!['action'] == 'capture') captures++;
      if (operation!['action'] == 'submit') submitted++;
      return {'job': 'finding-job'};
    }
    if (r['action'] == 'cancel') return null;
    if (r['action'] == 'releaseFindingJob') return {'released': true};
    if (r['action'] == 'poll') {
      final op = operation!;
      dynamic value;
      if (op['action'] == 'capture') {
        if (failCapture) {
          return {
            'done': true,
            'error': 'Git basis changed. Refresh this diff; note remains.',
          };
        }
        records[op['id']] = {
          'id': op['id'],
          'revision': 0,
          'text': op['text'],
          'anchor': {
            'comparison': op['comparison'],
            'side': op['side'],
            'line': op['line'],
            'excerpt': 'new',
          },
          'submissions': [],
          'resolution': null,
        };
        value = {'finding': records[op['id']]};
      } else if (op['action'] == 'submit') {
        if (failSubmit) {
          return {
            'done': true,
            'error': 'Instruction queue is full. Cancel or inspect pending work; findings and draft remain.',
          };
        }
        final selected = op['submission']['selected'] as List;
        for (final f in selected) {
          records[f['id']] = {
            ...records[f['id']]!,
            'revision': 1,
            'submissions': [op['submission']['id']],
          };
        }
        submissions[op['submission']['id']] = op['submission'];
        value = {
          'submission': op['submission'],
          'inputs': {'entries': []},
        };
      } else {
        final f = records[op['id']]!;
        records[op['id']] = {
          ...f,
          'revision': (f['revision'] as int) + 1,
          'resolution': {'comparison': op['comparison'], 'note': op['note']},
        };
        value = {'finding': records[op['id']]};
      }
      if (losePoll) {
        losePoll = false;
        throw StateError('Temporary status transport failure');
      }
      return {'done': true, 'value': value};
    }
    throw StateError('Unexpected fixture request: $c');
  }
}

Map<String, dynamic> diff({String revision = 'v1', String digest = ''}) => {
  'path': 'a.txt',
  'basis': 'working',
  'commit': null,
  'revision': revision,
  'digest': digest.isEmpty ? 'a' * 64 : digest,
  'leftLabel': 'Index',
  'rightLabel': 'Saved working tree',
  'cursor': 0,
  'next': null,
  'reason': null,
  'sideBySide': false,
  'rows': [
    {
      'kind': 'remove',
      'text': 'old',
      'oldLine': 1,
      'newLine': null,
      'endLine': true,
    },
    {
      'kind': 'add',
      'text': 'new',
      'oldLine': null,
      'newLine': 1,
      'endLine': true,
    },
  ],
};
GitWorkspace workspace(GitHost git) {
  final w = GitWorkspace('C:/public-fixture', 'chat')
    ..status = {'repo': 'repo', 'root': 'C:/public-fixture', 'revision': 'v1'}
    ..activeTab = 'a'
    ..tabs['a'] = diff();
  git.workspaces[w.root] = w;
  git.selected = w;
  return w;
}

Future<void> tap(WidgetTester t, String label) async {
  final f = find.text(label).last;
  await t.ensureVisible(f);
  await t.tap(f);
  await t.pumpAndSettle();
}

Future<void> setup(
  WidgetTester t,
  GitHost git, {
  bool dark = false,
  GlobalKey? key,
}) async {
  await t.pumpWidget(
    MaterialApp(
      theme: doloresTheme(dark),
      builder: (context, child) => RepaintBoundary(key: key, child: child!),
      home: Scaffold(body: SourceControlView(git: git)),
    ),
  );
  await t.pumpAndSettle();
}

void main() {
  testWidgets(
    'capture waits for the initial findings read without consuming the note',
    (t) async {
      final bridge = ReviewBridge();
      final git = GitHost(bridge, reviewAvailable: true);
      final w = workspace(git);
      await setup(t, git);
      await t.tap(find.byKey(const ValueKey('review-right-1')));
      await t.pumpAndSettle();
      await t.enterText(
        find.byKey(const Key('review-note')),
        'Keep my note during the initial read',
      );
      bridge.listing = Completer<void>();
      final read = git.loadFindings(w);
      await t.pump();
      expect(
        t
            .widget<FilledButton>(
              find.widgetWithText(FilledButton, 'Save finding'),
            )
            .onPressed,
        isNull,
      );
      expect(bridge.captures, 0);
      expect(
        t
            .widget<TextField>(find.byKey(const Key('review-note')))
            .controller!
            .text,
        'Keep my note during the initial read',
      );
      bridge.listing!.complete();
      await read;
      await t.pumpAndSettle();
      await tap(t, 'Save finding');
      expect(bridge.captures, 1);
      await t.pumpWidget(const SizedBox());
      git.dispose();
    },
  );
  testWidgets(
    'line capture is inline, queue is deliberate, and later resolution preserves anchor',
    (t) async {
      final bridge = ReviewBridge();
      final git = GitHost(bridge, reviewAvailable: true);
      final w = workspace(git);
      int delivered = 0;
      git.afterReviewSubmission = (session) async {
        expect(session, 'chat');
        delivered++;
      };
      await setup(t, git);
      await t.tap(find.byKey(const ValueKey('review-right-1')));
      await t.pumpAndSettle();
      await t.enterText(
        find.byKey(const Key('review-note')),
        'Handle empty input',
      );
      expect(bridge.captures, 0);
      await tap(t, 'Save finding');
      expect(bridge.captures, 1);
      expect(delivered, 0);
      expect(find.text('Review: Handle empty input'), findsOneWidget);
      await tap(t, 'Review findings');
      await t.tap(find.byType(Checkbox).first);
      await t.pumpAndSettle();
      await tap(t, 'Queue selected');
      expect(bridge.submitted, 1);
      expect(delivered, 1);
      expect(bridge.records.values.single['resolution'], isNull);
      expect(find.textContaining('Input receipts:'), findsOneWidget);
      w.status!['revision'] = 'v2';
      w.tabs['a'] = diff(revision: 'v2', digest: 'b' * 64);
      git.changed();
      await t.pumpAndSettle();
      expect(find.textContaining('Location changed'), findsOneWidget);
      await tap(t, 'Resolve after inspection');
      await t.enterText(
        find.byKey(const Key('resolution-note')),
        'Inspected the empty-input test',
      );
      await tap(t, 'Save resolution');
      expect(
        bridge.records.values.single['resolution']['note'],
        'Inspected the empty-input test',
      );
      expect(
        bridge.records.values.single['anchor']['comparison']['revision'],
        'v1',
      );
      await t.pumpWidget(const SizedBox());
      git.dispose();
    },
  );
  testWidgets(
    'failed capture keeps the note and full queue preserves selected feedback',
    (t) async {
      final bridge = ReviewBridge()..failCapture = true;
      final git = GitHost(bridge, reviewAvailable: true);
      final w = workspace(git);
      await setup(t, git);
      await t.tap(find.byKey(const ValueKey('review-left-1')));
      await t.pumpAndSettle();
      await t.enterText(
        find.byKey(const Key('review-note')),
        'Retain my unsaved note',
      );
      await tap(t, 'Save finding');
      expect(find.textContaining('Git basis changed'), findsOneWidget);
      expect(
        t
            .widget<TextField>(find.byKey(const Key('review-note')))
            .controller!
            .text,
        'Retain my unsaved note',
      );
      bridge.failCapture = false;
      await tap(t, 'Save finding');
      await tap(t, 'Review findings');
      await t.tap(find.byType(Checkbox).first);
      await t.pumpAndSettle();
      bridge.failSubmit = true;
      await tap(t, 'Steer with selected');
      expect(find.textContaining('queue is full'), findsOneWidget);
      expect(git.reviewState(w).selected.length, 1);
      bridge.failSubmit = false;
      await tap(t, 'Steer with selected');
      expect(bridge.submissions.length, 1);
      expect(bridge.submissions.values.single['mode'], 'steer');
      await t.pumpWidget(const SizedBox());
      git.dispose();
    },
  );
  test(
    'lost committed result is inspected without a second submission',
    () async {
      final bridge = ReviewBridge();
      final git = GitHost(bridge, reviewAvailable: true);
      final w = workspace(git);
      await git.runFinding(w, {
        'action': 'capture',
        'id': 'f',
        'comparison': git.comparisonFor(w, w.tabs['a']!),
        'cursor': 0,
        'side': 'right',
        'line': 1,
        'text': 'Review it',
      });
      bridge.losePoll = true;
      expect(
        await git.runFinding(w, {
          'action': 'submit',
          'submission': {
            'id': 'submission',
            'mode': 'queue',
            'selected': [
              {'id': 'f', 'revision': 0},
            ],
          },
        }),
        true,
      );
      expect(bridge.submitted, 1);
      expect(w.busy, false);
      expect(git.reviewState(w).pending, isNull);
      expect(git.reviewState(w).notice, contains('not repeated'));
      git.dispose();
    },
  );
  test('failed status retains job, stops it and rechecks without restarting capture', () async {
    final bridge = ReviewBridge()
      ..losePoll = true
      ..holdResult = true;
    final git = GitHost(bridge, reviewAvailable: true);
    final w = workspace(git);
    expect(
      await git.runFinding(w, {
        'action': 'capture',
        'id': 'f',
        'comparison': git.comparisonFor(w, w.tabs['a']!),
        'cursor': 0,
        'side': 'right',
        'line': 1,
        'text': 'Keep note',
      }),
      false,
    );
    expect(w.busy, true);
    expect(w.job, 'finding-job');
    expect(bridge.captures, 1);
    await git.stop(w);
    bridge.holdResult = false;
    expect(await git.checkFinding(w), true);
    expect(bridge.captures, 1);
    expect(w.busy, false);
    w.session = 'other';
    expect(git.reviewState(w).items, isEmpty);
    w.session = 'chat';
    expect(git.reviewState(w).items.length, 1);
    git.dispose();
  });
  for (final dark in [false, true]) {
    for (final width in [420.0, 1100.0]) {
      testWidgets('review findings render dark=$dark width=$width', (t) async {
        viewport(t, Size(width, 900));
        await t.runAsync(loadReviewFonts);
        final bridge = ReviewBridge();
        final git = GitHost(bridge, reviewAvailable: true);
        final w = workspace(git);
        await git.runFinding(w, {
          'action': 'capture',
          'id': 'f',
          'comparison': git.comparisonFor(w, w.tabs['a']!),
          'cursor': 0,
          'side': 'right',
          'line': 1,
          'text': 'Handle empty input without replacing the retained file.',
        });
        final key = GlobalKey();
        await setup(t, git, dark: dark, key: key);
        await capture(
          t,
          key,
          'review-inline-${dark ? 'dark' : 'light'}-${width.toInt()}',
        );
        await tap(t, 'Review findings');
        w.status!['revision'] = 'v2';
        w.tabs['a'] = diff(revision: 'v2', digest: 'b' * 64);
        git.changed();
        await t.pumpAndSettle();
        expect(find.textContaining('Location changed'), findsOneWidget);
        expect(t.takeException(), isNull);
        await capture(
          t,
          key,
          'review-stale-${dark ? 'dark' : 'light'}-${width.toInt()}',
        );
        await t.pumpWidget(const SizedBox());
        git.dispose();
      });
    }
  }
}
