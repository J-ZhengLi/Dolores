import 'dart:async';

import 'package:flutter_test/flutter_test.dart';
import 'package:dolores_flutter/app_host.dart';
import 'package:dolores_flutter/chat.dart';
import 'package:dolores_flutter/scheduled_host.dart';

import 'workspace_test.dart' show WorkspaceBridge;

class ScheduleBridge extends WorkspaceBridge {
  bool paused = false, fail = false, claimed = false;
  Map workspace = {'kind': 'project', 'root': 'C:/scheduled-project'};
  Map get listing => {
    'items': [
      {
        'task': {'id': 'task', 'paused': paused, 'nextDue': 1791385200},
        'occurrences': [],
      },
    ],
  };
  @override
  Future<dynamic> call(Map<String, dynamic> command) async {
    final name = command['command'];
    if (name.toString().startsWith('scheduled')) commands.add(command);
    if (name == 'scheduledTasks') {
      if (fail) throw StateError('Storage unavailable');
      return listing;
    }
    if (name == 'scheduledTick') {
      final fresh = !claimed;
      claimed = true;
      return {
        'tasks': listing,
        'claimed': fresh
            ? [
                {
                  'id': 'occurrence',
                  'snapshot': {'prompt': 'Write a report every day at 9pm'},
                },
              ]
            : [],
      };
    }
    if (name == 'scheduledStart') {
      return {
        'session': 'result',
        'model': 'pinned-model',
        'workspace': workspace,
      };
    }
    if (name == 'scheduledAbandon') return null;
    if (name == 'poll') return <Map>[];
    return super.call(command);
  }
}

class DelayedScheduleBridge extends ScheduleBridge {
  final ready = Completer<void>();
  @override
  Future<dynamic> call(Map<String, dynamic> command) async {
    if (command['command'] == 'scheduledTasks') await ready.future;
    return super.call(command);
  }
}

void main() {
  test(
    'Close waits for an in-flight refresh and leaves its clock suspended',
    () async {
      final bridge = DelayedScheduleBridge();
      var launched = false, settled = false;
      final host = ScheduledHost(
        bridge,
        onClaim: (_) async {
          launched = true;
        },
      );
      final refresh = host.refresh();
      final closing = host.quiesce().then((_) {
        settled = true;
      });
      await Future<void>.value();
      expect(settled, false);
      bridge.ready.complete();
      await refresh;
      await closing;
      expect(host.pending, false);
      await host.tick();
      expect(launched, false);
      expect(
        bridge.commands.where((c) => c['command'] == 'scheduledTick'),
        isEmpty,
      );
      host.dispose();
    },
  );
  test('background task keeps Home project model and draft', () async {
    final bridge = ScheduleBridge();
    final chat = ChatController(bridge)
      ..loading = false
      ..session = 'home'
      ..workspaceRoot = 'C:/home-project'
      ..model = 'home-model'
      ..draft = 'my draft';
    final host = AppHost(chat);
    await Future<void>.delayed(Duration.zero);
    await host.scheduled.tick();
    final owner = host.owners.last;
    expect(owner.session, 'result');
    expect(owner.busy, true);
    expect(owner.workspaceKind, 'project');
    expect(owner.workspaceRoot, 'C:/scheduled-project');
    expect(owner.workspaceLabel, 'Project');
    expect(owner.model, 'pinned-model');
    expect(host.visible, same(chat));
    expect(chat.draft, 'my draft');
    expect(chat.model, 'home-model');
    expect(host.projectRoot, 'C:/home-project');
    expect(bridge.commands.where((c) => c['command'] == 'start'), isEmpty);
    await owner.stop();
    expect(
      bridge.commands.lastWhere((c) => c['command'] == 'cancel')['id'],
      bridge.commands.firstWhere((c) => c['command'] == 'scheduledStart')['id'],
    );
    host.dispose();
  });
  test(
    'temporary scheduled results retain their actual working folder',
    () async {
      final bridge = ScheduleBridge()
        ..workspace = {'kind': 'temporary', 'root': 'C:/temporary-task'};
      final chat = ChatController(bridge)
        ..loading = false
        ..session = 'home'
        ..workspaceKind = 'project'
        ..workspaceRoot = 'C:/home-project'
        ..model = 'home-model'
        ..draft = 'Retain Home draft';
      final host = AppHost(chat);
      await Future<void>.delayed(Duration.zero);
      await host.scheduled.tick();
      final owner = host.owners.last;
      expect(owner.workspaceKind, 'temporary');
      expect(owner.workspaceRoot, 'C:/temporary-task');
      expect(owner.model, 'pinned-model');
      expect(host.visible, same(chat));
      expect(chat.workspaceRoot, 'C:/home-project');
      expect(chat.model, 'home-model');
      expect(chat.draft, 'Retain Home draft');
      await owner.stop();
      host.dispose();
    },
  );
  test(
    'paused schedules have no idle clock; failed refresh keeps prior list',
    () async {
      final bridge = ScheduleBridge()..paused = true;
      final host = ScheduledHost(bridge, onClaim: (_) async {});
      await host.refresh();
      expect(host.needsClock, false);
      expect(host.items.length, 1);
      bridge.fail = true;
      await host.refresh();
      expect(host.items.length, 1);
      expect(host.error, contains('unavailable'));
      host.dispose();
    },
  );
  test(
    'launch failure records recovery rather than retrying the same claim',
    () async {
      final bridge = ScheduleBridge();
      var attempts = 0;
      final host = ScheduledHost(
        bridge,
        onClaim: (_) async {
          attempts++;
          throw StateError('Owner limit');
        },
      );
      await host.tick();
      await host.tick();
      expect(attempts, 1);
      expect(
        bridge.commands.where((c) => c['command'] == 'scheduledAbandon').length,
        1,
      );
      host.dispose();
    },
  );
}
