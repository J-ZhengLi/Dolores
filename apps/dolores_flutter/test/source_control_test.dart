import 'dart:async';

import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:dolores_flutter/bridge.dart';
import 'package:dolores_flutter/git_host.dart';
import 'package:dolores_flutter/source_control.dart';
import 'package:dolores_flutter/git_diff_view.dart';

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
