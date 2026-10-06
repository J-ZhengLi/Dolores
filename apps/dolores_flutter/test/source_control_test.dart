import 'dart:async';

import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:dolores_flutter/bridge.dart';
import 'package:dolores_flutter/git_host.dart';
import 'package:dolores_flutter/source_control.dart';

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
