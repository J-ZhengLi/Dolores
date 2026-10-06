import 'dart:async';
import 'dart:convert';

import 'package:flutter/foundation.dart';

import 'bridge.dart';
import 'document_buffer.dart';

class FileDocument extends ChangeNotifier {
  DocumentBuffer? _buffer;
  DocumentBuffer get buffer => _buffer ??= DocumentBuffer(text, onEdit: edit);
  bool Function(String) edit = (_) => false;
  late String acknowledged = text;
  String? sending;
  bool blocked = false;
  Future<void>? syncing;
  bool closed = false;
  final String id, project;
  Map<String, dynamic> snapshot;
  String text;
  int version;
  bool pending = false;
  String? error;
  FileDocument(Map<String, dynamic> value)
    : id = value['document'] as String,
      project = value['project'] as String,
      snapshot = Map<String, dynamic>.from(value['snapshot'] as Map),
      text = value['text'] as String,
      version = value['version'] as int;
  String get path => snapshot['path'] as String;
  bool get readonly => snapshot['readonly'] == true;
  bool get dirty => text != snapshot['text'];
  void accept(Map<String, dynamic> value) {
    final previousText = text;
    snapshot = Map<String, dynamic>.from(value['snapshot'] as Map);
    text = value['text'] as String;
    acknowledged = text;
    blocked = false;
    if (previousText != text) _buffer?.replace(text);
    version = value['version'] as int;
    error = null;
    notifyListeners();
  }

  void changed() => notifyListeners();
  @override
  void dispose() {
    closed = true;
    _buffer?.dispose();
    super.dispose();
  }
}

class TreePage {
  final entries = <Map<String, dynamic>>[];
  int? cursor = 0;
  bool pending = false;
  String? error;
}

class FileWorkspace {
  final String project, root;
  String session;
  final tree = <String, TreePage>{};
  final expanded = <String>{'.'};
  final recovery = <String>[];
  dynamic layout;
  String? active;
  FileWorkspace(this.project, this.root, this.session);
}

/// Lazy page state and document owners live above widgets and conversation selection.
class FileHost extends ChangeNotifier {
  final _checkpoints = <String, Timer>{};
  final ChatBridge bridge;
  final documents = <String, FileDocument>{};
  final workspaces = <String, FileWorkspace>{};
  FileWorkspace? selected;
  String? error;
  bool loading = false;
  int _binding = 0;
  String? _pendingBinding;
  bool _disposed = false;
  FileHost(this.bridge);
  Future<dynamic> call(FileWorkspace w, Map<String, dynamic> request) =>
      bridge.call({
        'command': 'editor',
        'session': w.session,
        'request': {'project': w.project, ...request},
      });
  void changed() {
    if (!_disposed) notifyListeners();
  }

  Future<void> bind(String? session, String? root) async {
    final binding = '$session|$root';
    if (loading && _pendingBinding == binding) return;
    _pendingBinding = binding;
    final token = ++_binding;
    if (session == null || root == null) {
      selected = null;
      loading = false;
      error = null;
      changed();
      return;
    }
    final cached = workspaces.values.where((w) => w.root == root).firstOrNull;
    if (cached != null) {
      cached.session = session;
      selected = cached;
      loading = false;
      error = null;
      changed();
      unawaited(refreshDocuments(cached));
      return;
    }
    selected = null;
    loading = true;
    error = null;
    changed();
    try {
      final value = Map<String, dynamic>.from(
        await bridge.call({
          'command': 'editor',
          'session': session,
          'request': {'action': 'workspace'},
        }) as Map,
      );
      if (token != _binding || _disposed) return;
      if (value['root'] != root) {
        throw StateError(
          'Project changed while opening files. Select the conversation again.',
        );
      }
      final w = FileWorkspace(value['project'] as String, root, session)
        ..layout = value['layout'];
      w.recovery.addAll(
        (value['recovery'] as List? ?? []).map((e) => e['path'] as String),
      );
      for (final doc in value['documents'] as List? ?? []) {
        _accept(Map<String, dynamic>.from(doc as Map));
      }
      selected = w;
      workspaces[w.project] = w;
      await load(w, '.');
    } catch (e) {
      if (token == _binding && !_disposed) {
        selected = null;
        error = '$e';
      }
    } finally {
      if (token == _binding && !_disposed) {
        loading = false;
        changed();
      }
    }
  }

  Future<void> load(
    FileWorkspace w,
    String path, {
    bool refresh = false,
  }) async {
    final page = w.tree.putIfAbsent(path, () => TreePage());
    if (page.pending) return;
    if (refresh) {
      page.entries.clear();
      page.cursor = 0;
    }
    if (page.cursor == null) return;
    page.pending = true;
    page.error = null;
    changed();
    try {
      final value = await call(w, {
        'action': 'tree',
        'path': path,
        'cursor': page.cursor,
      });
      if (_disposed) return;
      page.entries.addAll(
        (value['entries'] as List).map(
          (e) => Map<String, dynamic>.from(e as Map),
        ),
      );
      page.cursor = value['cursor'] as int?;
    } catch (e) {
      page.error = '$e';
    } finally {
      page.pending = false;
      changed();
    }
  }

  void expand(FileWorkspace w, String path) {
    if (!w.expanded.add(path)) {
      w.expanded.remove(path);
    } else if (!w.tree.containsKey(path)) {
      unawaited(load(w, path));
    }
    changed();
  }

  FileDocument _accept(Map<String, dynamic> value) {
    final saved = value['saved'];
    final warning = value['recoveryWarning'];
    if (saved is Map) value = Map<String, dynamic>.from(saved);
    final id = value['document'] as String;
    final d = documents[id];
    if (d != null) {
      d.accept(value);
      if (warning != null) {
        d.error = '$warning The file was saved; private recovery needs Retry.';
      }
      return d;
    }
    final next = FileDocument(value);
    next.edit = (text) => edit(next, text);
    documents[id] = next;
    return next;
  }

  Future<FileDocument?> open(
    FileWorkspace w,
    String path, {
    String action = 'open',
  }) async {
    try {
      error = null;
      final value = await call(w, {'action': action, 'path': path});
      if (_disposed) return null;
      final d = _accept(Map<String, dynamic>.from(value as Map));
      w.active = d.id;
      w.recovery.remove(path);
      changed();
      return d;
    } catch (e) {
      error = '$e';
      changed();
      return null;
    }
  }

  FileDocument? get active => documents[selected?.active];
  bool edit(FileDocument d, String text) {
    try {
      if (d.readonly || d.pending || d.closed) {
        throw StateError(
          'This document is read-only or completing a file action.',
        );
      }
      final units = text.codeUnits;
      for (var i = 0; i < units.length; i++) {
        final u = units[i];
        if (u >= 0xd800 && u <= 0xdbff) {
          if (i + 1 >= units.length ||
              units[i + 1] < 0xdc00 ||
              units[++i] > 0xdfff) {
            throw StateError('Edit contains an incomplete Unicode character.');
          }
        } else if (u >= 0xdc00 && u <= 0xdfff) {
          throw StateError('Edit contains an incomplete Unicode character.');
        }
      }
      if (text.contains('\r') ||
          text.contains('\u0000') ||
          text.split('\n').any((l) => utf8.encode(l).length > 8192)) {
        throw StateError(
          'Edit exceeds the 8 KiB line limit or contains unsupported text. Split the paste.',
        );
      }
      final bytes =
          utf8.encode(text).length +
          (d.snapshot['bom'] == true ? 3 : 0) +
          (d.snapshot['newline'] == 'crlf'
              ? text.codeUnits.where((u) => u == 10).length
              : 0);
      if (bytes > 1024 * 1024) {
        throw StateError(
          'Edit exceeds the 1 MiB file limit. Split the change.',
        );
      }
      if (documents.values.fold<int>(
            0,
            (n, x) =>
                n +
                utf8.encode(x.snapshot['text'] as String).length +
                utf8.encode(x == d ? text : x.text).length,
          ) >
          4 * 1024 * 1024) {
        throw StateError(
          'The resident text limit is reached. Save and close an inactive document.',
        );
      }
      final transaction = textDelta(d.sending ?? d.acknowledged, text);
      if (utf8.encode(transaction['text'] as String).length > 64 * 1024) {
        throw StateError(
          'Pending edit exceeds 64 KiB. Wait for synchronization, or split the paste.',
        );
      }
      d.text = text;
      d.error = null;
      d.changed();
      if (!d.blocked) unawaited(flush(d));
      return true;
    } catch (e) {
      d.error = '$e Previous buffer is retained.';
      d.changed();
      return false;
    }
  }

  Future<void> flush(FileDocument d) {
    if (d.syncing != null) return d.syncing!;
    if (d.blocked || d.closed) return Future.value();
    final future = _drain(d);
    d.syncing = future;
    return future.whenComplete(() => d.syncing = null);
  }

  Future<void> _drain(FileDocument d) async {
    final w = workspaces[d.project];
    if (w == null) return;
    try {
      while (!d.closed && !_disposed && d.text != d.acknowledged) {
        final target = d.text;
        d.sending = target;
        final result = await call(w, {
          'action': 'edit',
          'document': d.id,
          'version': d.version,
          'edits': [textDelta(d.acknowledged, target)],
        });
        if (d.closed || _disposed) return;
        d.version = result['version'] as int;
        d.acknowledged = target;
        d.sending = null;
        d.changed();
      }
      _checkpoints[w.project]?.cancel();
      _checkpoints[w.project] = Timer(
        const Duration(milliseconds: 500),
        () => checkpoint(w),
      );
    } catch (e) {
      if (!d.closed && !_disposed) {
        d.blocked = true;
        d.error =
            '$e Local edits are retained. Compare or Retry sync; nothing was replayed.';
        d.changed();
      }
    } finally {
      d.sending = null;
    }
  }

  Future<void> retrySync(FileDocument d) async {
    final w = workspaces[d.project]!;
    try {
      final value = await call(w, {'action': 'open', 'path': d.path});
      if (value['document'] != d.id) {
        throw StateError('Document identity changed. Keep edits or Save as.');
      }
      d.acknowledged = value['text'] as String;
      d.version = value['version'] as int;
      d.blocked = false;
      d.error = null;
      d.snapshot = Map<String, dynamic>.from(value['snapshot'] as Map);
      await flush(d);
    } catch (e) {
      d.error = '$e';
    }
    d.changed();
  }

  Future<bool> save(FileWorkspace w, FileDocument d) async {
    await flush(d);
    if (d.blocked || d.text != d.acknowledged) return false;
    return action(w, d, 'save');
  }

  Future<Map<String, dynamic>?> compare(FileWorkspace w, FileDocument d) async {
    await flush(d);
    if (d.blocked) {
      error = 'Retry sync before comparing. Local edits are retained.';
      changed();
      return null;
    }
    try {
      return Map<String, dynamic>.from(
        await call(w, {
          'action': 'compare',
          'document': d.id,
          'version': d.version,
        }) as Map,
      );
    } catch (e) {
      error = '$e';
      changed();
      return null;
    }
  }

  Future<bool> reconcile(
    FileWorkspace w,
    FileDocument d,
    String action,
    String revision,
  ) async {
    await flush(d);
    if (d.blocked) return false;
    d.pending = true;
    d.changed();
    try {
      final value = await call(w, {
        'action': action,
        'document': d.id,
        'version': d.version,
        'revision': revision,
      });
      _accept(Map<String, dynamic>.from(value as Map));
      error = null;
      return true;
    } catch (e) {
      d.error = '$e';
      error = '$e';
      return false;
    } finally {
      d.pending = false;
      d.changed();
      changed();
    }
  }

  Future<bool> action(
    FileWorkspace w,
    FileDocument d,
    String action, {
    String? path,
  }) async {
    if (d.pending) return false;
    await flush(d);
    if (d.blocked || d.text != d.acknowledged) return false;
    d.pending = true;
    changed();
    try {
      final value = await call(w, {
        'action': action,
        'document': d.id,
        'version': d.version,
        'path': ?path,
      });
      if (action == 'delete') {
        documents.remove(d.id);
        w.active = null;
        d.dispose();
      } else {
        _accept(Map<String, dynamic>.from(value as Map));
      }
      error = null;
      await load(w, '.', refresh: true);
      return true;
    } catch (e) {
      d.error = '$e';
      error = '$e';
      return false;
    } finally {
      d.pending = false;
      if (!d.closed) d.changed();
      changed();
    }
  }

  Future<void> checkpoint(FileWorkspace w) async {
    try {
      await call(w, {'action': 'checkpoint'});
      error = null;
    } catch (e) {
      error =
          '$e Your edits remain open. Retry private recovery before closing.';
    }
    changed();
  }

  Future<void> refreshDocuments(FileWorkspace w) async {
    for (final d
        in documents.values.where((d) => d.project == w.project).toList()) {
      if (d.pending || d.blocked || d.closed) continue;
      await flush(d);
      if (d.blocked) continue;
      try {
        final value = await call(w, {
          'action': 'refresh',
          'document': d.id,
          'version': d.version,
        });
        if (value['changed'] == true) {
          d.error = 'File changed on disk. Your edits are retained. Compare before saving.';
          d.changed();
        } else if (value['document'] != null) {
          _accept(Map<String, dynamic>.from(value as Map));
        }
      } catch (e) {
        if (!d.closed) {
          d.error = '$e Local edits are retained; Retry or Save as.';
          d.changed();
        }
      }
    }
    changed();
  }

  Future<bool> close(
    FileWorkspace w,
    FileDocument d, {
    bool discard = false,
  }) async {
    if (d.pending) return false;
    await flush(d);
    if (d.blocked && !discard) {
      error = 'Synchronization failed. Retry sync or explicitly discard before closing.';
      changed();
      return false;
    }
    try {
      await call(w, {
        'action': 'close',
        'document': d.id,
        'version': d.version,
        'discard': discard,
      });
      documents.remove(d.id);
      if (w.active == d.id) {
        w.active = documents.values
            .where((x) => x.project == w.project)
            .firstOrNull
            ?.id;
      }
      d.dispose();
      error = null;
      changed();
      return true;
    } catch (e) {
      error = '$e';
      changed();
      return false;
    }
  }

  @override
  void dispose() {
    for (final timer in _checkpoints.values) {
      timer.cancel();
    }
    _disposed = true;
    for (final d in documents.values) {
      d.dispose();
    }
    super.dispose();
  }
}

Map<String, dynamic> textDelta(String before, String after) {
  var start = 0;
  final a = before.codeUnits, b = after.codeUnits;
  while (start < a.length && start < b.length && a[start] == b[start]) {
    start++;
  }
  if (start > 0 &&
      start < a.length &&
      a[start] >= 0xdc00 &&
      a[start] <= 0xdfff) {
    start--;
  }
  var end = a.length, nextEnd = b.length;
  while (end > start && nextEnd > start && a[end - 1] == b[nextEnd - 1]) {
    end--;
    nextEnd--;
  }
  if (end < a.length && end > start && a[end] >= 0xdc00 && a[end] <= 0xdfff) {
    end++;
    nextEnd++;
  }
  return {'start': start, 'end': end, 'text': after.substring(start, nextEnd)};
}
