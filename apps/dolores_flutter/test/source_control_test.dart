import 'dart:async';

import 'package:flutter/material.dart';
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
      final host = GitHost(DiffBridge());
      final w = GitWorkspace('C:/A', 'A')
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
      await t.tap(find.byTooltip('Reviewed remote actions').first);
      await t.pumpAndSettle();
      expect(find.text('Review fast-forward pull'), findsOneWidget);
      await t.tap(find.text('Review push'));
      await t.pumpAndSettle();
      expect(op, {
        'kind': 'push',
        'remote': 'origin',
        'branch': 'refs/heads/main',
      });
      await t.tap(find.byTooltip('Reviewed remote actions').last);
      await t.pumpAndSettle();
      expect(find.text('Review push'), findsNothing);
      await t.tap(find.text('Review fetch'));
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
