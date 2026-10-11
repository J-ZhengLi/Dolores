import 'dart:async';
import 'dart:io';
import 'dart:ui' as ui;

import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:flutter/rendering.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:dolores_flutter/bridge.dart';
import 'package:dolores_flutter/git_host.dart';
import 'package:dolores_flutter/source_control.dart';
import 'package:dolores_flutter/git_diff_view.dart';
import 'package:dolores_flutter/git_local_controls.dart';
import 'package:dolores_flutter/theme.dart';

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
  final requests = <Map<String, dynamic>>[];
  @override
  Future<dynamic> call(Map<String, dynamic> command) async {
    final request = command['request'] as Map;
    if (request['action'] != 'poll') {
      pending = Map<String, dynamic>.from(request);
      requests.add(pending!);
      return {'job': 'paged'};
    }
    if (this.fail) {
      throw StateError('Git basis changed. Refresh this comparison.');
    }
    final cursor = pending!['cursor'] as int? ?? 0;
    return {
      'done': true,
      'value': {
        'path': pending!['path'],
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

class RefreshDiffBridge extends PagedDiffBridge {
  String revision = 'v1';
  bool failStatus = false;
  Completer<void>? holdStatus;
  @override
  Future<dynamic> call(Map<String, dynamic> command) async {
    final request = command['request'] as Map;
    if (request['action'] == 'poll') {
      if (pending!['action'] == 'status') {
        await holdStatus?.future;
        if (failStatus) {
          throw StateError('Cannot read current repository. Retry.');
        }
        return {
          'done': true,
          'value': {'repo': 'A', 'revision': revision},
        };
      }
      if (pending!['revision'] != revision) {
        throw StateError(
          'Git basis changed. Refresh and review the new state.',
        );
      }
      final result = await super.call(command);
      result['value']['revision'] = revision;
      return result;
    }
    return super.call(command);
  }
}

class MutationBridge implements ChatBridge {
  Map<String, dynamic>? pending;
  final requests = <Map<String, dynamic>>[];
  bool reject = false;
  String refusal = 'Commit hook refused. Draft remains.';
  Completer<void>? holdReview;
  @override
  Future<void> open() async {}
  @override
  Future<void> close() async {}
  @override
  Future<dynamic> call(Map<String, dynamic> command) async {
    final r = Map<String, dynamic>.from(command['request'] as Map);
    if (r['action'] != 'poll') {
      requests.add(r);
      pending = r;
      return {'job': 'mutation'};
    }
    if (pending!['action'] == 'review') {
      await holdReview?.future;
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
    if (reject) throw StateError(refusal);
    return {
      'done': true,
      'value': {
        'completed': true,
        'operation': requests.lastWhere(
          (r) => r['action'] == 'review',
        )['operation'],
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
  testWidgets('missing repository status never invents a HEAD state', (
    t,
  ) async {
    final host = GitHost(MutationBridge());
    final w = GitWorkspace('C:/public-nonrepository', 'A');
    host.selected = w;
    await t.pumpWidget(
      MaterialApp(
        home: Scaffold(
          body: SourceControlPanel(git: host, openFolder: () {}),
        ),
      ),
    );
    expect(find.text('No commit / detached HEAD'), findsNothing);
    w.error = 'This folder is not a Git worktree. Open a repository from Home.';
    host.changed();
    await t.pump();
    expect(find.text(w.error!), findsOneWidget);
    expect(find.text('No commit / detached HEAD'), findsNothing);
    expect(find.text('Detached HEAD'), findsNothing);
    expect(t.takeException(), isNull);
    await t.pumpWidget(const SizedBox());
    host.dispose();
  });
  for (final hasHead in [false, true]) {
    testWidgets('repository without a branch distinguishes HEAD ($hasHead)', (
      t,
    ) async {
      final host = GitHost(MutationBridge());
      host.selected = GitWorkspace('C:/public-repository', 'A')
        ..status = {
          'repo': 'A',
          'revision': 'v1',
          'branch': null,
          'head': hasHead ? 'commit' : null,
          'entries': [],
        };
      await t.pumpWidget(
        MaterialApp(
          home: Scaffold(
            body: SourceControlPanel(git: host, openFolder: () {}),
          ),
        ),
      );
      expect(
        find.text(hasHead ? 'Detached HEAD' : 'No commits yet'),
        findsOneWidget,
      );
      expect(find.text('No commit / detached HEAD'), findsNothing);
      expect(t.takeException(), isNull);
      await t.pumpWidget(const SizedBox());
      host.dispose();
    });
  }
  test('a late diff refresh does not replace a newly selected tab', () async {
    final bridge = RefreshDiffBridge();
    final host = GitHost(bridge);
    final w = GitWorkspace('C:/A', 'A')
      ..status = {'repo': 'A', 'revision': 'v1'};
    host.selected = w;
    await host.openDiff(w, 'large.txt', 'working');
    final firstKey = w.activeTab;
    await host.openDiff(w, 'other.txt', 'working');
    final otherKey = w.activeTab;
    final other = w.tabs[otherKey];
    w.activeTab = firstKey;
    bridge.holdStatus = Completer<void>();
    final refresh = host.refreshDiff(w);
    w.activeTab = otherKey;
    bridge.holdStatus!.complete();
    await refresh;
    expect(w.activeTab, otherKey);
    expect(w.tabs[otherKey], same(other));
    expect(bridge.requests.last['action'], 'status');
    expect(w.busy, false);
    host.dispose();
  });
  for (final failedRead in [false, true]) {
    testWidgets(
      'Refresh diff recovers an external change; failed state read=$failedRead',
      (t) async {
        final bridge = RefreshDiffBridge();
        final host = GitHost(bridge);
        final w = GitWorkspace('C:/A', 'A')
          ..status = {'repo': 'A', 'revision': 'v1'}
          ..sideBySide = true;
        host.selected = w;
        await host.openDiff(w, 'large.txt', 'working');
        await host.loadDiffPage(w, 4);
        final displayed = w.tabs[w.activeTab];
        final priorStatus = w.status;
        bridge.revision = 'v2';
        await host.loadDiffPage(w, 0);
        expect(w.error, contains('basis changed'));
        expect(w.tabs[w.activeTab], same(displayed));
        await t.pumpWidget(
          MaterialApp(
            home: Scaffold(body: SourceControlView(git: host)),
          ),
        );
        bridge.failStatus = failedRead;
        final diffs = bridge.requests
            .where((r) => r['action'] == 'diff')
            .length;
        await t.tap(find.text('Refresh diff'));
        await t.pumpAndSettle();
        if (failedRead) {
          expect(w.error, contains('Cannot read current repository'));
          expect(w.status, same(priorStatus));
          expect(w.tabs[w.activeTab], same(displayed));
          expect(w.tabs[w.activeTab]!['pageTrail'], [0, 4]);
          expect(
            bridge.requests.where((r) => r['action'] == 'diff').length,
            diffs,
          );
          bridge.failStatus = false;
          await t.tap(find.text('Refresh diff'));
          await t.pumpAndSettle();
        }
        expect(w.error, isNull);
        expect(w.tabs[w.activeTab]!['revision'], 'v2');
        expect(w.tabs[w.activeTab]!['sideBySide'], true);
        expect(w.tabs[w.activeTab]!['pageTrail'], [0]);
        expect(find.text('old value'), findsOneWidget);
        expect(find.text('new value'), findsOneWidget);
        expect(bridge.requests.last['cursor'], isNull);
        expect(bridge.requests.last['digest'], isNull);
        expect(bridge.requests.last['revision'], 'v2');
        expect(t.takeException(), isNull);
        await t.pumpWidget(const SizedBox());
        host.dispose();
      },
    );
  }
  testWidgets('a streamed side page renders both revisions without raw rows', (
    t,
  ) async {
    await t.pumpWidget(
      MaterialApp(
        home: Scaffold(
          body: GitDiffView(
            sideBySide: true,
            diff: {
              'leftLabel': 'Index',
              'rightLabel': 'Working tree',
              'rows': [],
              'sideRows': [
                {
                  'left': {
                    'kind': 'remove',
                    'text': 'old across page',
                    'oldLine': 251,
                  },
                  'right': {
                    'kind': 'add',
                    'text': 'new across page',
                    'newLine': 251,
                  },
                },
                {
                  'left': null,
                  'right': {
                    'kind': 'add',
                    'text': 'extra addition',
                    'newLine': 252,
                  },
                },
              ],
            },
          ),
        ),
      ),
    );
    await t.pumpAndSettle();
    final old = find.widgetWithText(SelectableText, 'old across page');
    final next = find.widgetWithText(SelectableText, 'new across page');
    expect(old, findsOneWidget);
    expect(next, findsOneWidget);
    expect(t.getTopLeft(old).dy, t.getTopLeft(next).dy);
    expect(t.getTopLeft(old).dx, lessThan(t.getTopLeft(next).dx));
    expect(find.text('extra addition'), findsOneWidget);
    expect(t.takeException(), isNull);
  });
  test(
    'layout changes reset paging only on success and tabs retain their layout',
    () async {
      final bridge = PagedDiffBridge();
      final host = GitHost(bridge);
      final w = GitWorkspace('C:/A', 'A')
        ..status = {'repo': 'A', 'revision': 'v1'};
      host.selected = w;
      await host.openDiff(w, 'large.txt', 'working');
      final firstKey = w.activeTab;
      await host.loadDiffPage(w, 4);
      final displayed = w.tabs[firstKey];
      bridge.fail = true;
      await host.setDiffLayout(w, true);
      expect(w.tabs[firstKey], same(displayed));
      expect(w.tabs[firstKey]!['sideBySide'], false);
      expect(w.tabs[firstKey]!['pageTrail'], [0, 4]);
      expect(w.error, contains('basis changed'));
      bridge.fail = false;
      await host.setDiffLayout(w, true);
      expect(w.tabs[firstKey]!['sideBySide'], true);
      expect(w.tabs[firstKey]!['pageTrail'], [0]);
      expect(bridge.requests.last['cursor'], isNull);
      expect(bridge.requests.last['digest'], isNull);
      await host.loadDiffPage(w, 4);
      expect(bridge.requests.last['sideBySide'], true);
      expect(bridge.requests.last['digest'], 'pinned-pages');
      await host.openDiff(w, 'other.txt', 'working', sideBySide: false);
      expect(w.tabs[w.activeTab]!['sideBySide'], false);
      w.activeTab = firstKey;
      await host.loadDiffPage(w, 0);
      expect(bridge.requests.last['sideBySide'], true);
      expect(w.tabs[firstKey]!['sideBySide'], true);
      expect(w.tabs[firstKey]!['pageTrail'], [0]);
      host.dispose();
    },
  );
  final renderDirectory = Platform.environment['DOLORES_GIT_RENDER_DIRECTORY'];
  setUpAll(() async {
    if (renderDirectory == null) return;
    final font = ByteData.sublistView(
      await File('C:/Windows/Fonts/segoeui.ttf').readAsBytes(),
    );
    for (final family in ['Segoe UI', 'Roboto']) {
      await (FontLoader(family)..addFont(Future.value(font))).load();
    }
    final icons = await File(
      '../../output/toolchains/flutter/bin/cache/artifacts/material_fonts/MaterialIcons-Regular.otf',
    ).readAsBytes();
    await (FontLoader(
      'MaterialIcons',
    )..addFont(Future.value(ByteData.sublistView(icons)))).load();
  });
  for (final dark in [false, true]) {
    for (final width in [252.0, 320.0]) {
      testWidgets('staging panel compact layout ($dark, $width)', (t) async {
        t.view.physicalSize = Size(width, 680);
        t.view.devicePixelRatio = 1;
        addTearDown(t.view.resetPhysicalSize);
        addTearDown(t.view.resetDevicePixelRatio);
        final host = GitHost(MutationBridge());
        host.selected = GitWorkspace('C:/public-fixture', 'A')
          ..status = {
            'repo': 'A',
            'revision': 'v1',
            'branch': 'main',
            'entries': [
              {
                'path': 'src/very-long-file-name.dart',
                'index': 'M',
                'worktree': 'M',
              },
              {'path': 'docs/guide.md', 'index': '?', 'worktree': '?'},
            ],
          }
          ..commitDraft = 'Improve workspace';
        final boundaryKey = GlobalKey();
        await t.pumpWidget(
          RepaintBoundary(
            key: boundaryKey,
            child: MaterialApp(
              debugShowCheckedModeBanner: false,
              theme: doloresTheme(dark).copyWith(
                textButtonTheme: TextButtonThemeData(
                  style: TextButton.styleFrom(
                    textStyle: const TextStyle(fontFamily: 'Segoe UI'),
                  ),
                ),
              ),
              home: Scaffold(
                body: SourceControlPanel(git: host, openFolder: () {}),
              ),
            ),
          ),
        );
        await t.pumpAndSettle();
        expect(find.byTooltip('Stage all changes'), findsOneWidget);
        expect(find.byTooltip('Unstage all changes'), findsOneWidget);
        expect(find.byTooltip('Stage changes'), findsNWidgets(2));
        expect(find.byTooltip('Unstage changes'), findsOneWidget);
        expect(t.takeException(), isNull);
        if (renderDirectory != null) {
          await t.runAsync(() async {
            final image =
                await (boundaryKey.currentContext!.findRenderObject()
                        as RenderRepaintBoundary)
                    .toImage();
            final bytes = await image.toByteData(
              format: ui.ImageByteFormat.png,
            );
            await Directory(renderDirectory).create(recursive: true);
            await File(
              '$renderDirectory/staging-${dark ? 'dark' : 'light'}-${width.toInt()}.png',
            ).writeAsBytes(bytes!.buffer.asUint8List());
            image.dispose();
          });
        }
        await t.pumpWidget(const SizedBox());
        host.dispose();
      });
    }
  }
  testWidgets(
    'section and file buttons stage or unstage without another dialog',
    (t) async {
      final bridge = MutationBridge();
      final host = GitHost(bridge);
      final w = GitWorkspace('C:/A', 'A')
        ..status = {
          'repo': 'A',
          'revision': 'v1',
          'entries': [
            {'path': 'working.txt', 'index': ' ', 'worktree': 'M'},
            {'path': 'staged.txt', 'index': 'M', 'worktree': ' '},
          ],
        };
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
      for (final key in [
        'git-stage-all',
        'git-unstage-all',
        'git-stage-working.txt',
        'git-unstage-staged.txt',
      ]) {
        expect(find.byKey(ValueKey(key)), findsOneWidget);
      }
      final status = w.status;
      w.commitDraft = 'Retained draft';
      for (final item in [
        ('git-stage-working.txt', 'stage', ['working.txt']),
        ('git-unstage-staged.txt', 'unstage', ['staged.txt']),
        ('git-stage-all', 'stageAll', null),
        ('git-unstage-all', 'unstageAll', null),
      ]) {
        w.status = status;
        host.changed();
        await t.pumpAndSettle();
        await t.tap(find.byKey(ValueKey(item.$1)));
        await t.pumpAndSettle();
        final op =
            bridge.requests[bridge.requests.length - 2]['operation'] as Map;
        expect(op['kind'], item.$2);
        expect(op['paths'], item.$3);
        expect(bridge.pending!['action'], 'apply');
        expect(find.byKey(const Key('git-apply-review')), findsNothing);
        expect(w.commitDraft, 'Retained draft');
        expect(w.status!['revision'], 'v2');
      }
      expect(t.takeException(), isNull);
      await t.pumpWidget(const SizedBox());
      host.dispose();
    },
  );
  testWidgets(
    'failed index action retains state and draft; buttons disable while busy and retry',
    (t) async {
      final bridge = MutationBridge()..reject = true;
      final host = GitHost(bridge);
      final w = GitWorkspace('C:/A', 'A')
        ..status = {
          'repo': 'A',
          'revision': 'v1',
          'entries': [
            {'path': 'working.txt', 'index': ' ', 'worktree': 'M'},
          ],
        }
        ..commitDraft = 'Keep draft'
        ..notice = 'Git action completed.';
      final status = w.status;
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
      w.busy = true;
      host.changed();
      await t.pump();
      expect(
        t.widget<IconButton>(find.byKey(const Key('git-stage-all'))).onPressed,
        isNull,
      );
      expect(
        t
            .widget<IconButton>(find.byKey(const Key('git-stage-working.txt')))
            .onPressed,
        isNull,
      );
      w.busy = false;
      host.changed();
      await t.pump();
      await t.tap(find.byKey(const Key('git-stage-working.txt')));
      await t.pumpAndSettle();
      expect(w.status, same(status));
      expect(w.commitDraft, 'Keep draft');
      expect(w.error, contains('refused'));
      expect(w.notice, isNull);
      expect(w.reviewOpen, isNull);
      bridge.reject = false;
      await t.tap(find.byKey(const Key('git-stage-working.txt')));
      await t.pumpAndSettle();
      expect(w.error, isNull);
      expect(w.notice, 'Changes staged.');
      expect(w.commitDraft, 'Keep draft');
      await t.pumpWidget(const SizedBox());
      host.dispose();
    },
  );
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
    'Ctrl+Enter commits directly and stash menu excludes untracked files',
    (t) async {
      final bridge = MutationBridge();
      final host = GitHost(bridge);
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
      final originalStatus = w.status;
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
      expect(find.byType(Dialog), findsNothing);
      expect(w.commitDraft, '');
      expect(bridge.pending!['action'], 'apply');
      final committed =
          bridge.requests.firstWhere(
                (r) => r['action'] == 'review',
              )['operation']
              as Map;
      expect(committed, {'kind': 'commitAll', 'message': 'Keyboard message'});
      w.status = originalStatus;
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
      await t.ensureVisible(find.byKey(const ValueKey('git-commit-bbbbbbbb')));
      await t.pumpAndSettle();
      await t.tap(find.byKey(const ValueKey('git-commit-bbbbbbbb')));
      await t.pumpAndSettle();
      await t.drag(find.byType(ListView).first, const Offset(0, -160));
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

  testWidgets('Changes menu commits directly without a review dialog', (
    t,
  ) async {
    final bridge = MutationBridge();
    final host = GitHost(bridge);
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
    expect(find.byType(Dialog), findsNothing);
    expect(w.commitDraft, '');
    expect(bridge.pending!['action'], 'apply');
    expect(w.status!['revision'], 'v2');
    expect(t.takeException(), isNull);
    await t.pumpWidget(const SizedBox());
    host.dispose();
  });
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
    'hunk choices submit selected host IDs and stale comparisons disable actions',
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
      await t.tap(find.text('Stage selected hunks'));
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
              find.widgetWithText(TextButton, 'Stage selected hunks'),
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
  for (final operation in <Map<String, dynamic>>[
    {'kind': 'commit', 'message': 'direct commit'},
    {'kind': 'discard', 'path': 'same.txt'},
    {
      'kind': 'hunks',
      'path': 'same.txt',
      'staged': false,
      'ids': ['h1'],
    },
    {
      'kind': 'stashCreate',
      'paths': ['same.txt'],
      'message': 'saved',
    },
    {'kind': 'stashApply', 'stash': 'exact-object', 'pop': true},
    {'kind': 'branchCreate', 'name': 'feature'},
    {'kind': 'branchSwitch', 'name': 'feature'},
    {'kind': 'revert', 'commit': 'exact-commit'},
    {'kind': 'revertAbort'},
    {'kind': 'fetch', 'remote': 'origin'},
    {'kind': 'pull', 'remote': 'origin', 'branch': 'refs/heads/main'},
    {'kind': 'push', 'remote': 'origin', 'branch': 'refs/heads/main'},
  ]) {
    testWidgets('selected ${operation['kind']} runs without a review dialog', (
      t,
    ) async {
      final bridge = MutationBridge();
      final host = GitHost(bridge);
      final w = GitWorkspace('C:/A', 'A')
        ..status = {'repo': 'A', 'revision': 'v1'};
      await t.pumpWidget(
        MaterialApp(
          home: Builder(
            builder: (context) => TextButton(
              onPressed: () => executeGitAction(context, host, w, operation),
              child: const Text('Run action'),
            ),
          ),
        ),
      );
      await t.tap(find.text('Run action'));
      await t.pumpAndSettle();
      expect(find.byType(Dialog), findsNothing);
      expect(bridge.requests.map((r) => r['action']), ['review', 'apply']);
      expect(bridge.requests.last['token'], 'once');
      expect(bridge.requests.first['operation'], {
        ...operation,
        if (operation['kind'] == 'commit') 'kind': 'commitAll',
      });
      expect(w.reviewOpen, isNull);
      expect(w.status!['revision'], 'v2');
      expect(t.takeException(), isNull);
      await t.pumpWidget(const SizedBox());
      host.dispose();
    });
  }
  testWidgets(
    'direct commit retains draft on dirty editor and stale failure; retry recovers',
    (t) async {
      final bridge = MutationBridge();
      final host = GitHost(bridge);
      final w = GitWorkspace('C:/A', 'A')
        ..status = {'repo': 'A', 'revision': 'v1'}
        ..commitDraft = 'Retain this message';
      final status = w.status;
      host.beforeMutation = (_, _) =>
          throw StateError('Save or close unsaved editors. Drafts remain.');
      await t.pumpWidget(
        MaterialApp(
          home: Builder(
            builder: (context) => TextButton(
              onPressed: () => executeGitAction(context, host, w, {
                'kind': 'commit',
                'message': w.commitDraft,
              }),
              child: const Text('Commit'),
            ),
          ),
        ),
      );
      await t.tap(find.text('Commit'));
      await t.pumpAndSettle();
      expect(bridge.requests.map((r) => r['action']), ['review']);
      expect(w.error, 'Save or close unsaved editors. Drafts remain.');
      expect(w.status, same(status));
      expect(w.commitDraft, 'Retain this message');
      host.beforeMutation = null;
      bridge.reject = true;
      bridge.refusal = 'Git basis changed. Refresh before retrying.';
      await t.tap(find.text('Commit'));
      await t.pumpAndSettle();
      expect(w.error, bridge.refusal);
      expect(w.status, same(status));
      expect(w.commitDraft, 'Retain this message');
      expect(w.reviewOpen, isNull);
      bridge.reject = false;
      await t.tap(find.text('Commit'));
      await t.pumpAndSettle();
      expect(w.error, isNull);
      expect(w.commitDraft, '');
      expect(w.status!['revision'], 'v2');
      expect(find.byType(Dialog), findsNothing);
      await t.pumpWidget(const SizedBox());
      host.dispose();
    },
  );
  testWidgets(
    'pending direct action refuses duplicate and cancels when its page closes',
    (t) async {
      final bridge = MutationBridge()..holdReview = Completer<void>();
      final host = GitHost(bridge);
      final w = GitWorkspace('C:/A', 'A')
        ..status = {'repo': 'A', 'revision': 'v1'};
      await t.pumpWidget(
        MaterialApp(
          home: Builder(
            builder: (context) => TextButton(
              onPressed: () => executeGitAction(context, host, w, {
                'kind': 'discard',
                'path': 'same.txt',
              }),
              child: const Text('Discard'),
            ),
          ),
        ),
      );
      await t.tap(find.text('Discard'));
      await t.pump();
      expect(w.busy, true);
      await t.tap(find.text('Discard'));
      await t.pump();
      expect(bridge.requests, hasLength(1));
      await t.pumpWidget(const SizedBox());
      bridge.holdReview!.complete();
      await t.pumpAndSettle();
      expect(bridge.requests.map((r) => r['action']), [
        'review',
        'discardReview',
      ]);
      expect(w.status!['revision'], 'v1');
      expect(w.reviewOpen, isNull);
      expect(w.busy, false);
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
    'same file at different commits has distinct tabs that retain exact diffs',
    (t) async {
      final host = GitHost(DiffBridge());
      final w = GitWorkspace('C:/A', 'A');
      host.selected = w;
      for (final letter in ['a', 'b']) {
        w.tabs[letter] = {
          'path': 'same.txt',
          'basis': 'commit',
          'commit': letter * 40,
          'leftLabel': 'parent',
          'rightLabel': letter * 8,
          'patch': '@@ -1 +1 @@\n-old\n+revision $letter\n',
        };
      }
      w.activeTab = 'a';
      await t.pumpWidget(
        MaterialApp(
          home: Scaffold(body: SourceControlView(git: host)),
        ),
      );
      expect(find.text('same.txt · aaaaaaaa'), findsOneWidget);
      expect(find.text('same.txt · bbbbbbbb'), findsOneWidget);
      expect(find.text('revision a'), findsOneWidget);
      await t.tap(find.text('same.txt · bbbbbbbb'));
      await t.pumpAndSettle();
      expect(w.activeTab, 'b');
      expect(find.text('revision b'), findsOneWidget);
      expect(find.text('revision a'), findsNothing);
      await t.tap(find.text('same.txt · aaaaaaaa'));
      await t.pumpAndSettle();
      expect(find.text('revision a'), findsOneWidget);
      expect(w.tabs.length, 2);
      expect(t.takeException(), isNull);
      await t.pumpWidget(const SizedBox());
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
