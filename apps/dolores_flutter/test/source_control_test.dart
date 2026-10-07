import 'dart:async';

import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:dolores_flutter/bridge.dart';
import 'package:dolores_flutter/git_host.dart';
import 'package:dolores_flutter/source_control.dart';
import 'package:dolores_flutter/git_diff_view.dart';
import 'package:dolores_flutter/git_local_controls.dart';

class DiffBridge implements ChatBridge {
  bool fail = false;
  Map<String, dynamic>? pending;
  @override
  Future<void> open() async {}
  @override
  Future<void> close() async {}
  @override
  Future<dynamic> call(Map<String, dynamic> command) async {
    final request = Map<String, dynamic>.from(command['request'] as Map);
    if (request['action'] == 'poll') {
      if (fail) throw StateError('Git basis changed. Refresh.');
      return {
        'done': true,
        'value': {
          'path': pending!['path'],
          'basis': pending!['basis'],
          'revision': 'v1',
          'leftLabel': 'Index',
          'rightLabel': 'Saved working tree',
          'left': 'old\n',
          'right': 'new\n',
          'patch': '@@ -1 +1 @@\n-old\n+new\n',
          'reason': null,
          'historical': false,
          'conflict': false,
        },
      };
    }
    pending = request;
    return {'job': 'diff'};
  }
}

class HistoryBridge extends DiffBridge {
  String? failCommit;
  final requests = <Map<String, dynamic>>[];
  Completer<void>? hold;
  @override
  Future<dynamic> call(Map<String, dynamic> command) async {
    final r = Map<String, dynamic>.from(command['request'] as Map);
    if (r['action'] != 'poll') requests.add(r);
    if (r['action'] == 'poll' && pending!['action'] == 'commitFiles') {
      await hold?.future;
      if (pending!['commit'] == failCommit) {
        throw StateError('Commit temporarily unavailable. Retry files.');
      }
      return {
        'done': true,
        'value': {
          'commit': pending!['commit'],
          'files': ['${pending!['commit']}.txt'],
        },
      };
    }
    if (r['action'] == 'poll' && pending!['action'] == 'remoteState') {
      return {
        'done': true,
        'value': {
          'remotes': ['origin', 'archive'],
          'remote': 'origin',
          'branch': 'refs/heads/main',
        },
      };
    }
    return super.call(command);
  }
}

class PagedDiffBridge extends DiffBridge {
  @override
  Future<dynamic> call(Map<String, dynamic> command) async {
    final request = command['request'] as Map;
    if (request['action'] != 'poll') {
      pending = Map<String, dynamic>.from(request);
      return {'job': 'paged'};
    }
    if (this.fail) {
      throw StateError('Git basis changed. Refresh this comparison.');
    }
    final cursor = pending!['cursor'] as int? ?? 0;
    return {
      'done': true,
      'value': {
        'path': 'large.txt',
        'basis': 'working',
        'revision': 'v1',
        'leftLabel': 'Index',
        'rightLabel': 'Saved working tree',
        'cursor': cursor,
        'next': cursor == 0 ? 4 : null,
        'totalRows': 6,
        'digest': 'pinned-pages',
        'patch': '',
        'reason': null,
        'hunks': [],
        'rows': cursor == 0
            ? [
                {'kind': 'meta', 'text': '@@ -10,2 +10,2 @@'},
                {'kind': 'remove', 'text': 'old value', 'oldLine': 10},
                {'kind': 'add', 'text': 'new value', 'newLine': 10},
                {
                  'kind': 'context',
                  'text': 'unchanged',
                  'oldLine': 11,
                  'newLine': 11,
                },
              ]
            : [
                {'kind': 'add', 'text': 'later change', 'newLine': 12},
                {'kind': 'add', 'text': 'last change', 'newLine': 13},
              ],
      },
    };
  }
}

class MutationBridge implements ChatBridge {
  Map<String, dynamic>? pending;
  bool reject = false;
  @override
  Future<void> open() async {}
  @override
  Future<void> close() async {}
  @override
  Future<dynamic> call(Map<String, dynamic> command) async {
    final r = Map<String, dynamic>.from(command['request'] as Map);
    if (r['action'] != 'poll') {
      pending = r;
      return {'job': 'mutation'};
    }
    if (pending!['action'] == 'review') {
      return {
        'done': true,
        'value': {
          'token': 'once',
          'repo': 'A',
          'paths': ['same.txt'],
          'root': 'C:/A',
          'revision': 'v1',
          'operation': pending!['operation'],
          'patch': 'fixture saved diff',
          'notice': 'Configured hooks remain enabled.',
        },
      };
    }
    if (reject) throw StateError('Commit hook refused. Draft remains.');
    return {
      'done': true,
      'value': {
        'completed': true,
        'operation': {'kind': 'commit'},
        'warning': null,
        'status': {'repo': 'A', 'revision': 'v2'},
      },
    };
  }
}

class GitBridge implements ChatBridge {
  final jobs = <String, Completer<dynamic>>{};
  bool fail = false;
  @override
  Future<void> open() async {}
  @override
  Future<void> close() async {}
  @override
  Future<dynamic> call(Map<String, dynamic> command) async {
    final r = command['request'] as Map;
    if (r['action'] == 'status') {
      final id = command['session'] as String;
      jobs[id] = Completer<dynamic>();
      return {'job': id};
    }
    if (r['action'] == 'poll') {
      if (fail) throw StateError('Repository unavailable. Refresh.');
      return {'done': true, 'value': await jobs[r['job']]!.future};
    }
    return null;
  }

  void finish(String id) => jobs[id]!.complete({
    'repo': id,
    'root': 'C:/$id',
    'revision': 'v1',
    'branch': 'main',
    'entries': [
      {'path': 'same.txt', 'index': ' ', 'worktree': 'M', 'conflict': false},
    ],
  });
}

void main() {
  testWidgets(
    'side diff aligns and colors changes; stale paging retains the readable page and explicit retry works',
    (t) async {
      final bridge = PagedDiffBridge();
      final host = GitHost(bridge);
      final w = GitWorkspace('C:/A', 'A')
        ..status = {'repo': 'A', 'revision': 'v1'}
        ..sideBySide = true;
      host.selected = w;
      await host.openDiff(w, 'large.txt', 'working');
      await t.pumpWidget(
        MaterialApp(
          home: Scaffold(body: SourceControlView(git: host)),
        ),
      );
      await t.pumpAndSettle();
      final old = find.widgetWithText(SelectableText, 'old value');
      final next = find.widgetWithText(SelectableText, 'new value');
      expect(t.getTopLeft(old).dy, t.getTopLeft(next).dy);
      expect(t.getTopLeft(old).dx, lessThan(t.getTopLeft(next).dx));
      expect(
        t
            .widget<Container>(find.byKey(const ValueKey('git-left-cell-1')))
            .color,
        Colors.red.withValues(alpha: .18),
      );
      expect(
        t
            .widget<Container>(find.byKey(const ValueKey('git-right-cell-1')))
            .color,
        Colors.green.withValues(alpha: .18),
      );
      final displayed = w.tabs[w.activeTab];
      bridge.fail = true;
      await t.tap(find.byKey(const Key('git-next-page')));
      await t.pumpAndSettle();
      expect(w.tabs[w.activeTab], same(displayed));
      expect(old, findsOneWidget);
      expect(find.textContaining('basis changed'), findsOneWidget);
      bridge.fail = false;
      await t.tap(find.byKey(const Key('git-next-page')));
      await t.pumpAndSettle();
      expect(find.text('later change'), findsOneWidget);
      expect(old, findsNothing);
      expect(bridge.pending!['digest'], 'pinned-pages');
      expect(bridge.pending!['revision'], 'v1');
      expect(w.tabs[w.activeTab]!['pageTrail'], [0, 4]);
      await t.tap(find.byKey(const Key('git-previous-page')));
      await t.pumpAndSettle();
      expect(old, findsOneWidget);
      expect(w.tabs[w.activeTab]!['pageTrail'], [0]);
      expect(t.takeException(), isNull);
      await t.pumpWidget(const SizedBox());
      host.dispose();
    },
  );
  testWidgets(
    'Ctrl+Enter reviews the retained message and stash menu excludes untracked files',
    (t) async {
      final host = GitHost(MutationBridge());
      final w = GitWorkspace('C:/A', 'A')
        ..status = {
          'repo': 'A',
          'revision': 'v1',
          'entries': [
            {'path': 'tracked.txt', 'index': ' ', 'worktree': 'M'},
            {'path': 'new.txt', 'index': '?', 'worktree': '?'},
          ],
        }
        ..commitDraft = 'Keyboard message';
      host.selected = w;
      await t.pumpWidget(
        MaterialApp(
          home: Scaffold(
            body: SizedBox(
              width: 300,
              child: SourceControlPanel(git: host, openFolder: () {}),
            ),
          ),
        ),
      );
      await t.tap(find.byKey(const Key('git-commit-message')));
      await t.sendKeyDownEvent(LogicalKeyboardKey.controlLeft);
      await t.sendKeyEvent(LogicalKeyboardKey.enter);
      await t.sendKeyUpEvent(LogicalKeyboardKey.controlLeft);
      await t.pumpAndSettle();
      expect(find.byKey(const Key('git-apply-review')), findsOneWidget);
      await t.tap(find.text('Cancel'));
      await t.pumpAndSettle();
      expect(w.commitDraft, 'Keyboard message');
      Map<String, dynamic>? op;
      await t.pumpWidget(
        MaterialApp(
          home: Scaffold(
            body: GitLocalControls(
              git: host,
              w: w,
              review: (value) async {
                op = value;
              },
            ),
          ),
        ),
      );
      await t.tap(find.byTooltip('Git actions'));
      await t.pumpAndSettle();
      await t.tap(find.text('Stash'));
      await t.pumpAndSettle();
      await t.tap(find.text('Stash changes'));
      await t.pumpAndSettle();
      expect(op!['paths'], ['tracked.txt']);
      expect(t.takeException(), isNull);
      await t.pumpWidget(const SizedBox());
      host.dispose();
    },
  );
  testWidgets(
    'commit expansion nests exact files, opens its diff, and retries only the failed commit',
    (t) async {
      final bridge = HistoryBridge();
      final host = GitHost(bridge);
      final w = GitWorkspace('C:/A', 'A')
        ..status = {'repo': 'A', 'revision': 'v1'}
        ..commitDraft = 'Keep message'
        ..history = [
          for (final id in ['aaaaaaaa', 'bbbbbbbb'])
            {'id': id, 'subject': 'Commit $id', 'author': 'Fixture'},
        ];
      host.selected = w;
      await t.pumpWidget(
        MaterialApp(
          home: Scaffold(
            body: SizedBox(
              width: 300,
              child: SourceControlPanel(git: host, openFolder: () {}),
            ),
          ),
        ),
      );
      await t.tap(find.byKey(const ValueKey('git-commit-aaaaaaaa')));
      await t.pumpAndSettle();
      final firstFile = find.byKey(
        const ValueKey('git-file-aaaaaaaa:aaaaaaaa.txt'),
      );
      expect(
        find.descendant(
          of: find.byKey(const ValueKey('git-history-aaaaaaaa')),
          matching: firstFile,
        ),
        findsOneWidget,
      );
      bridge.failCommit = 'bbbbbbbb';
      await t.tap(find.byKey(const ValueKey('git-commit-bbbbbbbb')));
      await t.pumpAndSettle();
      expect(find.byKey(const ValueKey('git-retry-bbbbbbbb')), findsOneWidget);
      expect(
        find.byKey(const ValueKey('git-file-bbbbbbbb:aaaaaaaa.txt')),
        findsNothing,
      );
      expect(firstFile, findsOneWidget);
      bridge.failCommit = null;
      await t.ensureVisible(find.byKey(const ValueKey('git-retry-bbbbbbbb')));
      await t.pumpAndSettle();
      await t.tap(find.byKey(const ValueKey('git-retry-bbbbbbbb')));
      await t.pumpAndSettle();
      expect(
        find.byKey(const ValueKey('git-file-bbbbbbbb:bbbbbbbb.txt')),
        findsOneWidget,
      );
      expect(w.commitDraft, 'Keep message');
      await t.ensureVisible(firstFile);
      await t.pumpAndSettle();
      await t.tap(firstFile);
      await t.pumpAndSettle();
      expect(bridge.requests.last, containsPair('commit', 'aaaaaaaa'));
      expect(bridge.requests.last, containsPair('path', 'aaaaaaaa.txt'));
      await t.tap(find.byKey(const ValueKey('git-commit-aaaaaaaa')));
      await t.pumpAndSettle();
      expect(firstFile, findsNothing);
      expect(w.commitFileCache['aaaaaaaa'], isNotNull);
      expect(t.takeException(), isNull);
      await t.pumpWidget(const SizedBox());
      host.dispose();
    },
  );

  test('late commit files stay with their owner and eight expanded commits give explicit recovery', () async {
    final bridge = HistoryBridge()..hold = Completer<void>();
    final host = GitHost(bridge);
    final a = GitWorkspace('C:/A', 'A')
      ..status = {'repo': 'A', 'revision': 'v1'};
    final b = GitWorkspace('C:/B', 'B')
      ..status = {'repo': 'B', 'revision': 'v1'};
    host.selected = a;
    final request = host.commitFiles(a, 'aaaaaaaa');
    await Future<void>.delayed(Duration.zero);
    host.selected = b;
    bridge.hold!.complete();
    await request;
    expect(a.commitFileCache.keys, ['aaaaaaaa']);
    expect(b.commitFileCache, isEmpty);
    expect(host.selected, same(b));
    bridge.hold = null;
    for (var i = 1; i < 8; i++) {
      await host.commitFiles(a, 'commit0$i');
    }
    await host.commitFiles(a, 'ninth000');
    expect(a.error, contains('Collapse one'));
    expect(a.commitFileCache.length, 8);
    await host.toggleCommit(a, 'aaaaaaaa');
    await host.toggleCommit(a, 'ninth000');
    expect(a.error, isNull);
    expect(a.commitFileCache.length, 8);
    expect(a.expandedCommits, contains('ninth000'));
    host.dispose();
  });

  testWidgets(
    'Changes menu commits through confirmation and Cancel preserves message',
    (t) async {
      final host = GitHost(MutationBridge());
      final w = GitWorkspace('C:/A', 'A')
        ..status = {'repo': 'A', 'revision': 'v1'}
        ..commitDraft = 'Retained message';
      host.selected = w;
      await t.pumpWidget(
        MaterialApp(
          home: Scaffold(
            body: SizedBox(
              width: 252,
              child: SourceControlPanel(git: host, openFolder: () {}),
            ),
          ),
        ),
      );
      expect(find.text('Review commit'), findsNothing);
      expect(find.text('New branch'), findsNothing);
      await t.tap(find.byTooltip('Git actions'));
      await t.pumpAndSettle();
      expect(find.text('Branch'), findsOneWidget);
      expect(find.text('Stash'), findsOneWidget);
      await t.tap(find.text('Commit'));
      await t.pumpAndSettle();
      expect(find.byKey(const Key('git-apply-review')), findsOneWidget);
      await t.tap(find.text('Cancel'));
      await t.pumpAndSettle();
      expect(w.commitDraft, 'Retained message');
      expect(t.takeException(), isNull);
      await t.pumpWidget(const SizedBox());
      host.dispose();
    },
  );
  test('equivalent Windows project paths retain diff owners', () async {
    final host = GitHost(DiffBridge());
    final w = GitWorkspace('C:/A', 'A')
      ..status = {'repo': 'A', 'revision': 'v1'}
      ..tabs['saved'] = {'path': 'same.txt'}
      ..activeTab = 'saved';
    host.workspaces[w.root] = w;
    await host.bind('A', r'\\?\C:\A');
    expect(host.selected, same(w));
    expect(host.selected!.activeTab, 'saved');
    expect(host.workspaces.length, 1);
    host.dispose();
  });
  testWidgets(
    'remote actions require a named configured target and expose only tracked pull and push',
    (t) async {
      final host = GitHost(HistoryBridge());
      final w = GitWorkspace('C:/A', 'A')
        ..status = {'repo': 'A', 'revision': 'v1'}
        ..remoteState = {
          'remotes': ['origin', 'archive'],
          'remote': 'origin',
          'branch': 'refs/heads/main',
          'ahead': 1,
          'behind': 0,
        };
      Map<String, dynamic>? op;
      await t.pumpWidget(
        MaterialApp(
          home: Scaffold(
            body: SingleChildScrollView(
              child: GitLocalControls(
                git: host,
                w: w,
                review: (v) async {
                  op = v;
                },
              ),
            ),
          ),
        ),
      );
      await t.tap(find.byTooltip('Git actions'));
      await t.pumpAndSettle();
      expect(find.text('Review commit'), findsNothing);
      await t.tap(find.text('Push'));
      await t.pumpAndSettle();
      expect(find.text('origin'), findsOneWidget);
      expect(find.text('archive'), findsNothing);
      await t.tap(find.text('origin'));
      await t.pumpAndSettle();
      expect(op, {
        'kind': 'push',
        'remote': 'origin',
        'branch': 'refs/heads/main',
      });
      await t.tap(find.byTooltip('Git actions'));
      await t.pumpAndSettle();
      await t.tap(find.text('Fetch'));
      await t.pumpAndSettle();
      await t.tap(find.text('archive'));
      await t.pumpAndSettle();
      expect(op, {'kind': 'fetch', 'remote': 'archive'});
      expect(t.takeException(), isNull);
      await t.pumpWidget(const SizedBox());
      host.dispose();
    },
  );
  testWidgets(
    'hunk choices review selected host IDs and stale comparisons disable apply',
    (t) async {
      final host = GitHost(DiffBridge());
      final w = GitWorkspace('C:/A', 'A')..status = {'revision': 'v1'};
      final diff = <String, dynamic>{
        'revision': 'v1',
        'path': 'same.txt',
        'basis': 'working',
        'hunks': [
          {'id': 'host-one', 'header': '@@ -1,2 +1,2 @@'},
          {'id': 'host-two', 'header': '@@ -20,2 +20,2 @@'},
        ],
      };
      Map<String, dynamic>? operation;
      await t.pumpWidget(
        MaterialApp(
          home: Scaffold(
            body: GitHunkControls(
              git: host,
              w: w,
              diff: diff,
              review: (op) async {
                operation = op;
              },
            ),
          ),
        ),
      );
      await t.tap(find.text('Choose hunks'));
      await t.pumpAndSettle();
      await t.tap(find.byType(CheckboxListTile).first);
      await t.pump();
      await t.tap(find.text('Review stage selected hunks'));
      await t.pump();
      expect(operation!['ids'], ['host-one']);
      expect(operation!.containsKey('patch'), false);
      w.status = {'revision': 'v2'};
      await t.pumpWidget(
        MaterialApp(
          home: Scaffold(
            body: GitHunkControls(
              git: host,
              w: w,
              diff: diff,
              review: (op) async {
                operation = op;
              },
            ),
          ),
        ),
      );
      expect(
        t
            .widget<TextButton>(
              find.widgetWithText(TextButton, 'Review stage selected hunks'),
            )
            .onPressed,
        isNull,
      );
      expect(t.takeException(), isNull);
      await t.pumpWidget(const SizedBox());
      host.dispose();
    },
  );
  test('failed reviewed commit retains its draft; a fresh successful review clears it', () async {
    final bridge = MutationBridge();
    final host = GitHost(bridge);
    final w = GitWorkspace('C:/A', 'A')
      ..status = {'repo': 'A', 'revision': 'v1'}
      ..commitDraft = 'Keep this message';
    final review = await host.review(w, {
      'kind': 'commit',
      'message': w.commitDraft,
    });
    expect(review, isNotNull);
    bridge.reject = true;
    await host.resolveReview(w, review!, apply: true);
    expect(w.commitDraft, 'Keep this message');
    expect(w.error, contains('hook refused'));
    expect(w.mutating, false);
    bridge.reject = false;
    final retry = await host.review(w, {
      'kind': 'commit',
      'message': w.commitDraft,
    });
    await host.resolveReview(w, retry!, apply: true);
    expect(w.commitDraft, '');
    expect(w.status!['revision'], 'v2');
    host.dispose();
  });
  testWidgets(
    'Git action review requires a separate decision and Cancel retains draft',
    (t) async {
      final host = GitHost(MutationBridge());
      final w = GitWorkspace('C:/A', 'A')
        ..status = {'repo': 'A', 'revision': 'v1'}
        ..commitDraft = 'draft';
      host.selected = w;
      await t.pumpWidget(
        MaterialApp(
          home: Builder(
            builder: (context) => TextButton(
              onPressed: () => reviewGitAction(context, host, w, {
                'kind': 'commit',
                'message': w.commitDraft,
              }),
              child: const Text('Review'),
            ),
          ),
        ),
      );
      await t.tap(find.text('Review'));
      await t.pumpAndSettle();
      expect(find.byKey(const Key('git-apply-review')), findsOneWidget);
      await t.tap(find.text('Cancel'));
      await t.pumpAndSettle();
      expect(w.commitDraft, 'draft');
      expect(t.takeException(), isNull);
      await t.pumpWidget(const SizedBox());
      host.dispose();
    },
  );
  test(
    'stale diff failure preserves the displayed tab; explicit refresh recovers',
    () async {
      final bridge = DiffBridge();
      final host = GitHost(bridge);
      final w = GitWorkspace('C:/A', 'A')
        ..status = {'repo': 'A', 'revision': 'v1'};
      host.selected = w;
      await host.openDiff(w, 'same.txt', 'working');
      final tab = w.tabs.values.single;
      bridge.fail = true;
      await host.openDiff(w, 'same.txt', 'working');
      expect(w.tabs.values.single, same(tab));
      expect(w.error, contains('basis changed'));
      bridge.fail = false;
      await host.openDiff(w, 'same.txt', 'working');
      expect(w.error, isNull);
      expect(w.tabs.values.single['right'], 'new\n');
      host.dispose();
    },
  );
  testWidgets(
    'inline and side comparisons name bases and render compact read-only diffs',
    (t) async {
      t.view.physicalSize = const Size(420, 600);
      t.view.devicePixelRatio = 1;
      addTearDown(t.view.resetPhysicalSize);
      addTearDown(t.view.resetDevicePixelRatio);
      final host = GitHost(DiffBridge());
      final w = GitWorkspace('C:/A', 'A')
        ..status = {'repo': 'A', 'revision': 'v1'};
      host.selected = w;
      await t.runAsync(() => host.openDiff(w, 'same.txt', 'working'));
      await t.pumpWidget(
        MaterialApp(
          home: Scaffold(body: SourceControlView(git: host)),
        ),
      );
      await t.pumpAndSettle();
      expect(find.byType(GitDiffView), findsOneWidget);
      expect(find.textContaining('Index → Saved working tree'), findsOneWidget);
      await t.tap(find.text('Side by side'));
      await t.pumpAndSettle();
      expect(find.text('old'), findsOneWidget);
      expect(find.text('new'), findsOneWidget);
      expect(t.takeException(), isNull);
      await t.pumpWidget(const SizedBox());
      host.dispose();
    },
  );
  test('late A status cannot replace B; failure retains last state and explicit refresh recovers', () async {
    final bridge = GitBridge();
    final host = GitHost(bridge);
    final a = host.bind('A', 'C:/A');
    await Future<void>.delayed(Duration.zero);
    final b = host.bind('B', 'C:/B');
    await Future<void>.delayed(Duration.zero);
    bridge.finish('B');
    await b;
    bridge.finish('A');
    await a;
    expect(host.selected!.status!['repo'], 'B');
    expect(host.workspaces['C:/A']!.status!['repo'], 'A');
    bridge.fail = true;
    await host.refresh(host.selected!);
    expect(host.selected!.status!['repo'], 'B');
    expect(host.selected!.error, contains('unavailable'));
    bridge.fail = false;
    final retry = host.refresh(host.selected!);
    await Future<void>.delayed(Duration.zero);
    bridge.finish('B');
    await retry;
    expect(host.selected!.error, isNull);
    await host.bind(null, null);
    expect(host.selected, isNull);
    host.dispose();
  });
  testWidgets(
    'repository panel has no project picker and preserves exact paths',
    (t) async {
      final bridge = GitBridge();
      final host = GitHost(bridge);
      await t.runAsync(() async {
        final load = host.bind('A', 'C:/A');
        await Future<void>.delayed(Duration.zero);
        bridge.finish('A');
        await load;
      });
      await t.pumpWidget(
        MaterialApp(
          home: Scaffold(
            body: Row(
              children: [
                SizedBox(
                  width: 252,
                  child: SourceControlPanel(git: host, openFolder: () {}),
                ),
                Expanded(child: SourceControlView(git: host)),
              ],
            ),
          ),
        ),
      );
      expect(find.text('Changes'), findsOneWidget);
      expect(find.text('same.txt'), findsOneWidget);
      expect(find.byType(DropdownButton), findsNothing);
      expect(t.takeException(), isNull);
      await t.pumpWidget(const SizedBox());
      host.dispose();
    },
  );
}
