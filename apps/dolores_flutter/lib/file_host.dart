import 'dart:async';
import 'dart:convert';

import 'package:flutter/foundation.dart';

import 'bridge.dart';
import 'document_buffer.dart';
import 'file_layout.dart';

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
  int? _savedBytes, _textBytes, _textBreaks;
  int get savedBytes =>
      _savedBytes ??= utf8.encode(snapshot['text'] as String).length;
  int get textBytes => _textBytes ??= utf8.encode(text).length;
  int get textBreaks => _textBreaks ??= '\n'.allMatches(text).length;
  void accept(Map<String, dynamic> value) {
    if (closed) return;
    final previousText = text;
    snapshot = Map<String, dynamic>.from(value['snapshot'] as Map);
    text = value['text'] as String;
    _savedBytes = _textBytes = _textBreaks = null;
    acknowledged = text;
    blocked = false;
    if (previousText != text) _buffer?.replace(text);
    version = value['version'] as int;
    error = null;
    notifyListeners();
  }

  void changed() {
    if (!closed) notifyListeners();
  }

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
  final layoutOwner = FileLayout();
  final paths = <String, String>{};
  bool restoring = false;
  String? layoutError;
  String? recoveryError;
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
  Future<void> _opening = Future.value();
  final _layouts = <String, Timer>{};
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
  final bool Function(String root)? mutationBusy;
  FileHost(this.bridge, {this.mutationBusy});
  Future<dynamic> call(FileWorkspace w, Map<String, dynamic> request) {
    if (mutationBusy?.call(w.root) == true &&
        [
          'edit',
          'save',
          'saveAs',
          'rename',
          'delete',
          'create',
          'reload',
          'rebase',
        ].contains(request['action'])) {
      return Future.error(
        StateError(
          'Finish the Git action before changing files. Drafts remain.',
        ),
      );
    }
    return bridge.call({
      'command': 'editor',
      'session': w.session,
      'request': {'project': w.project, ...request},
    });
  }

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
      await restoreActive(cached);
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
      wireLayout(w);
      await restoreLayout(w);
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
        d.error =
            '$warning The file action completed; private recovery needs Retry.';
        workspaces[d.project]?.recoveryError = '$warning';
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
    bool preview = true,
  }) {
    final result = _opening.then(
      (_) => _open(w, path, action: action, preview: preview),
    );
    _opening = result.then<void>((_) {});
    return result;
  }

  Future<FileDocument?> _open(
    FileWorkspace w,
    String path, {
    required String action,
    required bool preview,
  }) async {
    if (_disposed) return null;
    try {
      error = null;
      final oldId = w.paths.entries
          .where((e) => e.value == path)
          .firstOrNull
          ?.key;
      if (oldId == null && w.paths.values.toSet().length >= 4) {
        throw StateError(
          'Four documents are retained in this project. Close a file before opening another.',
        );
      }
      if (documents.length >= 4 &&
          !documents.values.any(
            (d) => d.project == w.project && d.path == path,
          )) {
        final candidate = documents.values
            .where(
              (d) =>
                  !d.dirty &&
                  !d.pending &&
                  !d.blocked &&
                  d.syncing == null &&
                  !(selected?.project == d.project &&
                      selected!.layoutOwner.groups.values.any(
                        (g) => g.active == d.id,
                      )),
            )
            .firstOrNull;
        if (candidate == null) {
          throw StateError(
            'Four documents are retained. Save and close an inactive document before opening another.',
          );
        }
        final owner = workspaces[candidate.project]!;
        await call(owner, {
          'action': 'close',
          'document': candidate.id,
          'version': candidate.version,
          'discard': false,
        });
        if (_disposed) return null;
        documents.remove(candidate.id);
        candidate.dispose();
      }
      final value = await call(w, {'action': action, 'path': path});
      if (_disposed) return null;
      final d = _accept(Map<String, dynamic>.from(value as Map));
      if (oldId != null && oldId != d.id) {
        w.layoutOwner.replaceIdentity(oldId, d.id);
        w.paths.remove(oldId);
      }
      w.paths[d.id] = d.path;
      if (!w.restoring) {
        final group = w.layoutOwner.active;
        final alreadyOpen = group.tabs.contains(d.id);
        final previous = group.preview;
        final previousDocument = documents[previous];
        if (!alreadyOpen &&
            previous != null &&
            previous != d.id &&
            (previousDocument == null || !previousDocument.dirty) &&
            previousDocument?.pending != true &&
            previousDocument?.blocked != true) {
          w.layoutOwner.remove(group.id, previous);
          if (!w.layoutOwner.groups.values.any(
            (g) => g.tabs.contains(previous),
          )) {
            w.paths.remove(previous);
          }
        }
        w.layoutOwner.open(d.id, preview: preview && !alreadyOpen);
        if (!preview) w.layoutOwner.pin(group.id, d.id);
      }
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
  void acceptLanguage(FileWorkspace w, List values) {
    for (final value in values) {
      final d = _accept(Map<String, dynamic>.from(value));
      w.paths[d.id] = d.path;
    }
    changed();
  }

  void wireLayout(FileWorkspace w) {
    w.layoutOwner.onChanged = () {
      w.active = w.layoutOwner.active.active;
      changed();
      if (!w.restoring) scheduleLayout(w);
    };
  }

  void scheduleLayout(FileWorkspace w) {
    if (_disposed) return;
    _layouts[w.project]?.cancel();
    _layouts[w.project] = Timer(
      const Duration(milliseconds: 250),
      () => persistLayout(w),
    );
  }

  Future<void> persistLayout(FileWorkspace w) async {
    if (_disposed) return;
    try {
      final data = w.layoutOwner.serialize(w.paths);
      FileLayout.validate(data);
      await call(w, {'action': 'layout', 'layout': data});
      w.layoutError = null;
    } catch (e) {
      w.layoutError =
          '$e Current layout and documents are retained. Retry layout save.';
    }
    changed();
  }

  Future<void> restoreLayout(FileWorkspace w) async {
    if (w.layout == null) return;
    w.restoring = true;
    try {
      final paths = FileLayout.paths(w.layout);
      final ids = <String, String>{};
      for (final path in paths) {
        final d = await open(
          w,
          path,
          action: w.recovery.contains(path) ? 'recover' : 'open',
        );
        if (d != null) ids[path] = d.id;
      }
      w.layoutOwner.restore(w.layout, ids);
      w.active = w.layoutOwner.active.active;
    } catch (e) {
      error =
          '$e A safe single group is available; private recovery is retained.';
    } finally {
      w.restoring = false;
      changed();
    }
  }

  Future<void> restoreActive(FileWorkspace w) async {
    w.restoring = true;
    try {
      for (final group in w.layoutOwner.groups.values) {
        final id = group.active;
        if (id != null && !documents.containsKey(id)) {
          final path = w.paths[id];
          if (path != null) await open(w, path);
        }
      }
      w.active = w.layoutOwner.active.active;
    } finally {
      w.restoring = false;
      changed();
    }
  }

  Future<void> activate(FileWorkspace w, String group, String id) async {
    if (!documents.containsKey(id)) {
      final path = w.paths[id];
      if (path == null) return;
      w.restoring = true;
      try {
        final d = await open(w, path);
        if (d == null) return;
        id = d.id;
      } finally {
        w.restoring = false;
      }
    }
    w.layoutOwner.select(group, id);
  }

  bool edit(FileDocument d, String text) {
    try {
      if (mutationBusy?.call(workspaces[d.project]?.root ?? '') == true) {
        throw StateError('Finish the Git action before editing.');
      }
      if (d.readonly || d.pending || d.closed) {
        throw StateError(
          'This document is read-only or completing a file action.',
        );
      }
      final local = textDelta(d.text, text);
      final inserted = local['text'] as String;
      final removed = d.text.substring(
        local['start'] as int,
        local['end'] as int,
      );
      final units = inserted.codeUnits;
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
        if (u == 13 || u == 0) {
          throw StateError(
            'Edit exceeds the 8 KiB line limit or contains unsupported text. Split the paste.',
          );
        }
      }
      final start = local['start'] as int;
      final left = start == 0 ? 0 : text.lastIndexOf('\n', start - 1) + 1;
      final right = text.indexOf('\n', start + inserted.length);
      if (text
          .substring(left, right < 0 ? text.length : right)
          .split('\n')
          .any((line) => utf8.encode(line).length > 8192)) {
        throw StateError('Edit exceeds the 8 KiB line limit. Split the paste.');
      }
      final insertBytes = utf8.encode(inserted).length;
      final normalizedBytes =
          d.textBytes + insertBytes - utf8.encode(removed).length;
      final lines =
          d.textBreaks +
          '\n'.allMatches(inserted).length -
          '\n'.allMatches(removed).length;
      final bytes =
          normalizedBytes +
          (d.snapshot['bom'] == true ? 3 : 0) +
          (d.snapshot['newline'] == 'crlf' ? lines : 0);
      if (bytes > 1024 * 1024) {
        throw StateError(
          'Edit exceeds the 1 MiB file limit. Split the change.',
        );
      }
      if (documents.values.fold<int>(
            0,
            (n, x) =>
                n + x.savedBytes + (x == d ? normalizedBytes : x.textBytes),
          ) >
          4 * 1024 * 1024) {
        throw StateError(
          'The resident text limit is reached. Save and close an inactive document.',
        );
      }
      final base = d.sending ?? d.acknowledged;
      final transaction = base == d.text ? local : textDelta(base, text);
      if (utf8.encode(transaction['text'] as String).length > 64 * 1024) {
        throw StateError(
          'Pending edit exceeds 64 KiB. Wait for synchronization, or split the paste.',
        );
      }
      d.text = text;
      d._textBytes = normalizedBytes;
      d._textBreaks = lines;
      final w = workspaces[d.project];
      if (w != null) {
        for (final group in w.layoutOwner.groups.values) {
          if (group.preview == d.id) w.layoutOwner.pin(group.id, d.id);
        }
      }
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
    var edited = false;
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
        edited = true;
        d.sending = null;
        d.changed();
      }
      if (!edited || d.closed || _disposed) return;
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
      if (_disposed || d.closed) return;
      if (value['document'] != d.id) {
        throw StateError('Document identity changed. Keep edits or Save as.');
      }
      d.acknowledged = value['text'] as String;
      d.version = value['version'] as int;
      d.blocked = false;
      d.error = null;
      d.snapshot = Map<String, dynamic>.from(value['snapshot'] as Map);
      d._savedBytes = null;
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
      if (_disposed || d.closed) return false;
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
      if (_disposed || d.closed) return false;
      if (action == 'delete') {
        if (value is Map && value['recoveryWarning'] != null) {
          w.recoveryError = '${value['recoveryWarning']}';
        }
        documents.remove(d.id);
        w.layoutOwner.removeDocument(d.id);
        w.paths.remove(d.id);
        w.active = null;
        d.dispose();
      } else {
        _accept(Map<String, dynamic>.from(value as Map));
        w.paths[d.id] = d.path;
        w.layoutOwner.changed();
      }
      error =
          action == 'delete' && value is Map && value['recoveryWarning'] != null
          ? '${value['recoveryWarning']} The file was deleted; Retry private recovery.'
          : null;
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

  Future<bool> checkpoint(FileWorkspace w) async {
    if (_disposed) return false;
    try {
      for (final d
          in documents.values.where((d) => d.project == w.project).toList()) {
        await flush(d);
        if (d.pending || d.blocked || d.text != d.acknowledged) {
          throw StateError(
            'Finish the file action or Retry sync before private recovery.',
          );
        }
      }
      await call(w, {'action': 'checkpoint'});
      w.recoveryError = null;
      changed();
      return true;
    } catch (e) {
      w.recoveryError =
          '$e Your edits remain open. Retry private recovery before closing.';
    }
    changed();
    return false;
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
        if (_disposed || d.closed) return;
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
      w.layoutOwner.removeDocument(d.id);
      w.paths.remove(d.id);
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
    for (final timer in _layouts.values) {
      timer.cancel();
    }
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
