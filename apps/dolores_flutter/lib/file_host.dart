import 'dart:async';

import 'package:flutter/foundation.dart';

import 'bridge.dart';

class FileDocument extends ChangeNotifier {
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
    snapshot = Map<String, dynamic>.from(value['snapshot'] as Map);
    text = value['text'] as String;
    version = value['version'] as int;
    error = null;
    notifyListeners();
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
  Future<bool> action(FileWorkspace w,FileDocument d,String action,{String? path}) async {
    if(d.pending)return false;
    d.pending=true;changed();
    try {
      final value=await call(w,{'action':action,'document':d.id,'version':d.version,'path':?path});
      if(action=='delete'){
        documents.remove(d.id);w.active=null;d.dispose();
      }else{_accept(Map<String,dynamic>.from(value as Map));}
      error=null;await load(w,'.',refresh:true);return true;
    }catch(e){d.error='$e';error='$e';return false;}
    finally{d.pending=false;changed();}
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

  Future<bool> close(
    FileWorkspace w,
    FileDocument d, {
    bool discard = false,
  }) async {
    if (d.pending) return false;
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
    _disposed = true;
    for (final d in documents.values) {
      d.dispose();
    }
    super.dispose();
  }
}
