import 'dart:async';

import 'package:flutter/foundation.dart';

import 'bridge.dart';

class GitWorkspace {
  final String root;
  String session;
  Map<String, dynamic>? status;
  bool busy = false;
  String? job, jobSession, error;
  String commitDraft = '';
  final tabs = <String, Map<String, dynamic>>{};
  String? activeTab;
  List<Map<String, dynamic>> history = [];
  int? historyNext;
  String? historyHead;
  Map<String, dynamic>? historyFiles;
  bool sideBySide = false;
  GitWorkspace(this.root, this.session);
}

/// Lazy Git ownership follows Home; late responses update only their own root.
class GitHost extends ChangeNotifier {
  final ChatBridge bridge;
  GitHost(this.bridge);
  final workspaces = <String, GitWorkspace>{};
  GitWorkspace? selected;
  String? error;
  bool _disposed = false;
  void changed() {
    if (!_disposed) notifyListeners();
  }

  Future<void> bind(String? session, String? root) async {
    if (session == null || root == null) {
      selected = null;
      changed();
      return;
    }
    var w = workspaces[root];
    if (w == null) {
      if (workspaces.length >= 8) {
        final available = workspaces.values
            .where((x) => !x.busy && x.commitDraft.isEmpty)
            .firstOrNull;
        if (available == null) {
          error = 'Eight repositories are retained. Finish work before opening another.';
          changed();
          return;
        }
        workspaces.remove(available.root);
      }
      w = GitWorkspace(root, session);
      workspaces[root] = w;
    }
    w.session = session;
    selected = w;
    error = null;
    if (w.status == null && !w.busy) {
      await refresh(w);
    } else {
      changed();
    }
  }

  Future<dynamic> run(GitWorkspace w, Map<String, dynamic> request) async {
    if (w.busy) {
      throw StateError('Wait for this repository operation, then Retry.');
    }
    w.busy = true;
    w.error = null;
    changed();
    final session = w.session;
    w.jobSession = session;
    try {
      final start = await bridge.call({
        'command': 'git',
        'session': session,
        'request': request,
      });
      w.job = start['job'] as String;
      while (!_disposed) {
        final poll = await bridge.call({
          'command': 'git',
          'session': session,
          'request': {'action': 'poll', 'job': w.job},
        });
        if (poll['done'] == true) return poll['value'];
        await Future<void>.delayed(const Duration(milliseconds: 40));
      }
      throw StateError('Source Control closed. Refresh before retrying.');
    } catch (e) {
      w.error = '$e';
      rethrow;
    } finally {
      w.busy = false;
      w.job = null;
      w.jobSession = null;
      changed();
    }
  }

  Future<void> refresh(GitWorkspace w) async {
    try {
      w.status = Map<String, dynamic>.from(await run(w, {'action': 'status'}));
    } catch (_) {
      /* Retain the last displayed state and its explicit error. */
    }
    changed();
  }

  Future<void> openDiff(
    GitWorkspace w,
    String path,
    String basis, {
    String? commit,
  }) async {
    final status = w.status;
    if (status == null) return;
    final key = '$basis:$commit:$path';
    if (!w.tabs.containsKey(key) && w.tabs.length >= 8) {
      w.error = 'Eight diff tabs are open. Close one before opening another.';
      changed();
      return;
    }
    try {
      final value = await run(w, {
        'action': 'diff',
        'repo': status['repo'],
        'revision': status['revision'],
        'path': path,
        'basis': basis,
        'commit': ?commit,
      });
      w.tabs[key] = Map<String, dynamic>.from(value);
      w.activeTab = key;
    } catch (_) {
      /* Preserve the prior tab on stale/bounded read failures. */
    }
    changed();
  }

  Future<void> loadHistory(GitWorkspace w, {bool more = false}) async {
    final status = w.status;
    if (status?['head'] == null) {
      w.error = 'This repository has no commits yet.';
      changed();
      return;
    }
    final head = more ? w.historyHead : status!['head'] as String;
    try {
      final value = await run(w, {
        'action': 'history',
        'repo': status!['repo'],
        'head': head,
        'cursor': more ? w.historyNext ?? 0 : 0,
      });
      final items = (value['items'] as List)
          .map((v) => Map<String, dynamic>.from(v))
          .toList();
      w.history = more ? [...w.history, ...items] : items;
      w.historyHead = head;
      w.historyNext = value['next'] as int?;
    } catch (_) {}
    changed();
  }

  Future<void> commitFiles(GitWorkspace w, String commit) async {
    try {
      w.historyFiles = Map<String, dynamic>.from(
        await run(w, {
          'action': 'commitFiles',
          'repo': w.status!['repo'],
          'commit': commit,
        }),
      );
    } catch (_) {}
    changed();
  }

  Future<void> stop(GitWorkspace w) async {
    if (w.job != null) {
      await bridge.call({
        'command': 'git',
        'session': w.jobSession ?? w.session,
        'request': {'action': 'cancel', 'job': w.job},
      });
    }
  }

  @override
  void dispose() {
    _disposed = true;
    super.dispose();
  }
}
