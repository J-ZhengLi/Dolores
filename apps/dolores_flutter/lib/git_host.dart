import 'dart:async';

import 'package:flutter/foundation.dart';

import 'bridge.dart';

class GitWorkspace {
  final String root;
  String session;
  Map<String, dynamic>? status;
  bool busy = false;
  bool mutating = false;
  String? notice;
  Map<String, dynamic>? reviewOpen;
  String? job, jobSession, error;
  String commitDraft = '';
  final tabs = <String, Map<String, dynamic>>{};
  String? activeTab;
  List<Map<String, dynamic>> history = [];
  int? historyNext;
  String? historyHead;
  final commitFileCache = <String, Map<String, dynamic>>{};
  final expandedCommits = <String>{};
  final commitFileErrors = <String, String>{};
  String? loadingCommit;
  bool changesExpanded = true, historyExpanded = true;
  Map<String, dynamic>? localState;
  Map<String, dynamic>? remoteState;
  bool sideBySide = false;
  GitWorkspace(this.root, this.session);
}

/// Lazy Git ownership follows Home; late responses update only their own root.
class GitHost extends ChangeNotifier {
  static String rootKey(String root) => root
      .replaceAll('\\', '/')
      .replaceFirst('//?/UNC/', '//')
      .replaceFirst('//?/', '')
      .toLowerCase()
      .replaceFirst(RegExp(r'/+$'), '');
  final ChatBridge bridge;
  GitHost(this.bridge);
  final workspaces = <String, GitWorkspace>{};
  GitWorkspace? selected;
  String? error;
  void Function(GitWorkspace w, List<String> paths)? beforeMutation;
  Future<void> Function(GitWorkspace w)? afterMutation;
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
    var w =
        workspaces[root] ??
        workspaces.values
            .where((w) => rootKey(w.root) == rootKey(root))
            .firstOrNull;
    if (w == null) {
      if (workspaces.length >= 8) {
        final available = workspaces.values
            .where(
              (x) => !x.busy && x.reviewOpen == null && x.commitDraft.isEmpty,
            )
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
      if (w.status?['head'] != null && !w.busy && w.error == null) {
        await loadHistory(w);
      }
    } else {
      changed();
    }
  }

  Future<dynamic> run(GitWorkspace w, Map<String, dynamic> request) async {
    if (w.busy) {
      throw StateError('Wait for this repository operation, then Retry.');
    }
    w.busy = true;
    w.mutating = request['action'] == 'apply';
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
      w.mutating = false;
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
      if (!more) {
        final ids = items.map((item) => item['id']).toSet();
        w.expandedCommits.removeWhere((id) => !ids.contains(id));
        w.commitFileCache.removeWhere((id, _) => !ids.contains(id));
        w.commitFileErrors.removeWhere((id, _) => !ids.contains(id));
      }
      w.historyHead = head;
      w.historyNext = value['next'] as int?;
    } catch (_) {}
    changed();
  }

  Future<void> commitFiles(GitWorkspace w, String commit) async {
    if (w.busy) return;
    if (!w.commitFileCache.containsKey(commit) &&
        !w.commitFileErrors.containsKey(commit) &&
        {...w.commitFileCache.keys, ...w.commitFileErrors.keys}.length >= 8) {
      final available = {
        ...w.commitFileCache.keys,
        ...w.commitFileErrors.keys,
      }.where((id) => !w.expandedCommits.contains(id)).firstOrNull;
      if (available == null) {
        w.error =
            'Eight commits are expanded. Collapse one before opening another.';
        changed();
        return;
      }
      w.commitFileCache.remove(available);
      w.commitFileErrors.remove(available);
    }
    w.expandedCommits.add(commit);
    w.commitFileErrors.remove(commit);
    w.loadingCommit = commit;
    try {
      w.commitFileCache[commit] = Map<String, dynamic>.from(
        await run(w, {
          'action': 'commitFiles',
          'repo': w.status!['repo'],
          'commit': commit,
        }),
      );
    } catch (e) {
      w.commitFileErrors[commit] = '$e';
    } finally {
      w.loadingCommit = null;
    }
    changed();
  }

  Future<void> toggleCommit(GitWorkspace w, String commit) async {
    if (w.expandedCommits.remove(commit)) {
      changed();
    } else if (w.commitFileCache.containsKey(commit)) {
      w.expandedCommits.add(commit);
      changed();
    } else {
      await commitFiles(w, commit);
    }
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

  Future<void> loadLocal(GitWorkspace w) async {
    try {
      w.localState = Map<String, dynamic>.from(
        await run(w, {'action': 'localState', 'repo': w.status!['repo']}),
      );
    } catch (_) {}
    changed();
  }

  Future<void> loadRemotes(GitWorkspace w) async {
    try {
      w.remoteState = Map<String, dynamic>.from(
        await run(w, {'action': 'remoteState', 'repo': w.status!['repo']}),
      );
    } catch (_) {}
    changed();
  }

  Future<Map<String, dynamic>?> review(
    GitWorkspace w,
    Map<String, dynamic> operation,
  ) async {
    try {
      final preview = Map<String, dynamic>.from(
        await run(w, {
          'action': 'review',
          'repo': w.status!['repo'],
          'revision': w.status!['revision'],
          'operation': operation,
        }),
      );
      beforeMutation?.call(w, (preview['paths'] as List).cast<String>());
      w.reviewOpen = preview;
      return preview;
    } catch (e) {
      w.error = '$e';
      changed();
      return null;
    }
  }

  Future<void> resolveReview(
    GitWorkspace w,
    Map<String, dynamic> preview, {
    required bool apply,
  }) async {
    try {
      if (apply) {
        beforeMutation?.call(w, (preview['paths'] as List).cast<String>());
      }
      final value = await run(w, {
        'action': apply ? 'apply' : 'discardReview',
        'repo': preview['repo'],
        'token': preview['token'],
      });
      if (apply) {
        w.notice = value['warning'] as String? ?? 'Git action completed.';
        if (value['completed'] == true &&
            (value['operation'] as Map)['kind'] == 'commit') {
          w.commitDraft = '';
        }
        if (value['status'] != null) {
          w.status = Map<String, dynamic>.from(value['status']);
        }
        w.busy = true;
        try {
          await afterMutation?.call(w);
        } finally {
          w.busy = false;
        }
        w.localState = null;
        w.remoteState = null;
      }
    } catch (e) {
      w.error = '$e';
    } finally {
      w.reviewOpen = null;
    }
    changed();
  }

  @override
  void dispose() {
    _disposed = true;
    super.dispose();
  }
}
