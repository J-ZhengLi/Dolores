import 'dart:async';

import 'package:flutter/foundation.dart';

import 'chat.dart';
import 'file_host.dart';
import 'git_host.dart';

/// One bridge/profile owner; visible conversation and running owners are separate.
class AppHost extends ChangeNotifier {
  late final FileHost files = FileHost(initial.bridge);
  late final GitHost git = GitHost(initial.bridge);
  final ChatController initial;
  final owners = <ChatController>[];
  late ChatController visible;
  bool selecting = false;
  String? error;
  bool panelHidden = false;
  double panelWidth = 252;
  final appearanceChanges = ValueNotifier(0);
  String? _appearance;
  final _running = <ChatController>{};
  final _themes = <ChatController, VoidCallback>{};
  Future<bool> Function()? closeReview;
  Future<void> Function(String path)? repositoryNavigation;
  Future<bool> requestClose() async => await closeReview?.call() ?? false;
  Future<void> prepareNativeRestart(ChatController caller) async {
    if (git.workspaces.values.any((w) => w.busy) ||
        owners.any((c) => c.busy || c.loading || c.changing) ||
        files.loading ||
        files.documents.values.any(
          (d) => d.dirty || d.pending || d.blocked || d.text != d.acknowledged,
        )) {
      throw StateError(
        'Finish current work and save or close unsaved file editors before restarting. All drafts and this review remain.',
      );
    }
    for (final owner in owners.where((c) => c != caller)) {
      await owner.checkpointDraft();
    }
  }

  Future<bool> prepareQuit({required bool saveFiles}) async {
    try {
      if (git.workspaces.values.any((w) => w.busy)) {
        throw StateError(
          'Stop or finish the Source Control operation before closing.',
        );
      }
      if (owners.any((c) => c.changing || c.loading)) {
        throw StateError('Finish the current operation before closing.');
      }
      if (saveFiles) {
        for (final d in files.documents.values.where((d) => d.dirty).toList()) {
          if (!await files.save(files.workspaces[d.project]!, d)) {
            throw StateError(
              'A file could not be saved. Its edits remain open.',
            );
          }
        }
      }
      for (final w in files.workspaces.values) {
        if (!await files.checkpoint(w)) {
          throw StateError(w.recoveryError ?? 'Private recovery failed.');
        }
        await files.persistLayout(w);
        if (w.layoutError != null) throw StateError(w.layoutError!);
      }
      for (final c in owners) {
        await c.checkpointDraft();
      }
      for (final c in tasks.toList()) {
        await c.stop();
      }
      await initial.bridge.close();
      return true;
    } catch (e) {
      error = '$e Keep Dolores open and retry.';
      notifyListeners();
      return false;
    }
  }

  void togglePanel() {
    panelHidden = !panelHidden;
    notifyListeners();
  }

  void resizePanel(double width) {
    panelHidden = width < 80;
    if (!panelHidden) panelWidth = width.clamp(180, 420);
    notifyListeners();
  }

  static const maxOwners = 8;
  AppHost(this.initial) {
    visible = initial;
    _appearance = initial.appearance;
    _add(initial);
  }
  void _add(ChatController owner) {
    owner.onOpenSourceControl = (path) async {
      if (owner.session != null) await select(owner.session!);
      if (visible == owner) await repositoryNavigation?.call(path);
    };
    owner.beforeNativeRestart = () => prepareNativeRestart(owner);
    owner.appearance = _appearance ?? owner.appearance;
    owners.add(owner);
    owner.addListener(_changed);
    void theme() {
      if (_appearance == owner.appearance) return;
      _appearance = owner.appearance;
      for (final other in owners.where((c) => c != owner)) {
        other.appearance = _appearance!;
      }
      appearanceChanges.value++;
      notifyListeners();
    }

    _themes[owner] = theme;
    owner.appearanceChanges.addListener(theme);
  }

  void _changed() {
    final ended = _running.where((c) => !c.busy).toList();
    _running
      ..clear()
      ..addAll(tasks);
    for (final owner in ended) {
      for (final w in files.workspaces.values.where(
        (w) => w.root == owner.workspaceRoot,
      )) {
        unawaited(files.refreshDocuments(w));
        if (files.selected == w) unawaited(files.load(w, '.', refresh: true));
      }
    }
    if (_appearance != visible.appearance) {
      _appearance = visible.appearance;
      appearanceChanges.value++;
    }
    notifyListeners();
  }

  Iterable<ChatController> get tasks => owners.where((c) => c.busy);
  String? get projectRoot => visible.workspaceRoot;
  Future<ChatController> _create() async {
    if (owners.length >= maxOwners) {
      final removable = owners
          .where(
            (c) =>
                c != initial &&
                c != visible &&
                !c.busy &&
                !c.changing &&
                c.draft.isEmpty,
          )
          .firstOrNull;
      if (removable == null) {
        throw StateError(
          'Eight conversations are retained. Finish a task or save a draft before opening another.',
        );
      }
      removable.removeListener(_changed);
      removable.appearanceChanges.removeListener(_themes.remove(removable)!);
      owners.remove(removable);
      removable.dispose();
    }
    final next = ChatController(initial.bridge, ownsBridge: false);
    try {
      await next.refresh();
      next.loading = false;
      _add(next);
      return next;
    } catch (_) {
      next.dispose();
      rethrow;
    }
  }

  Future<void> select(String id) async {
    if (selecting) return;
    selecting = true;
    error = null;
    notifyListeners();
    try {
      var next = owners.where((c) => c.session == id).firstOrNull;
      if (next == null) {
        next = await _create();
        await next.select(id);
        if (next.session != id) {
          throw StateError(next.error ?? 'Conversation could not be opened.');
        }
      }
      visible = next;
    } catch (e) {
      error = '$e';
    } finally {
      selecting = false;
      notifyListeners();
    }
  }

  Future<void> newConversation({String? kind, String? projectRoot}) async {
    if (selecting) return;
    selecting = true;
    error = null;
    notifyListeners();
    try {
      final next = await _create();
      next.workspaceRoot =
          projectRoot ?? (kind == 'project' ? visible.workspaceRoot : null);
      next.workspaceKind = kind ?? 'temporary';
      next.newChat(kind: kind);
      visible = next;
    } catch (e) {
      error = '$e';
    } finally {
      selecting = false;
      notifyListeners();
    }
  }

  Future<void> openProject(String root) async {
    if (selecting) return;
    selecting = true;
    error = null;
    notifyListeners();
    try {
      final next = await _create();
      await next.openProject(root);
      if (next.error != null) throw StateError(next.error!);
      visible = next;
    } catch (e) {
      error = '$e';
    } finally {
      selecting = false;
      notifyListeners();
    }
  }

  @override
  void dispose() {
    git.dispose();
    files.dispose();
    for (final owner in owners) {
      owner.beforeNativeRestart = null;
      owner.onOpenSourceControl = null;
      owner.removeListener(_changed);
      owner.appearanceChanges.removeListener(_themes[owner]!);
    }
    for (final owner in owners.where((c) => c != initial)) {
      owner.dispose();
    }
    initial.dispose();
    appearanceChanges.dispose();
    super.dispose();
  }
}
