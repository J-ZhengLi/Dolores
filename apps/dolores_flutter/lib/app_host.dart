import 'dart:async';

import 'package:flutter/foundation.dart';

import 'chat.dart';
import 'file_host.dart';
import 'git_host.dart';
import 'terminal_host.dart';
import 'language_host.dart';
import 'scheduled_host.dart';
import 'companion_host.dart';
import 'background_host.dart';

import 'package:path/path.dart' as paths;

/// One bridge/profile owner; visible conversation and running owners are separate.
class AppHost extends ChangeNotifier {
  final _scheduledOwners = <ChatController>{};
  bool quitting = false;
  Future<void> Function()? destroyWindow;
  Future<void> requestQuit() async {
    quitting = true;
    try {
      if (await requestClose()) {
        await destroyWindow?.call();
      }
    } finally {
      quitting = false;
    }
  }

  late final background = BackgroundHost(
    initial.bridge,
    restored: () async {
      await companion.refresh();
      await scheduled.refresh();
    },
    quit: requestQuit,
  );
  late final companion = CompanionHost(
    initial.bridge,
    session: () => visible.session,
    busy: () =>
        owners.any((c) => c.busy || c.loading || c.changing) ||
        selecting ||
        files.loading ||
        git.workspaces.values.any((w) => w.busy),
  );
  final _settings = <ChatController, int>{};
  late final scheduled = ScheduledHost(
    initial.bridge,
    onClaim: (occurrence) async {
      final owner = await _create();
      _scheduledOwners.add(owner);
      await owner.send(
        scheduledOccurrence: occurrence['id'] as String,
        taskInput: (occurrence['snapshot'] as Map)['prompt'] as String,
      );
      if (!owner.busy && owner.error != null) throw StateError(owner.error!);
    },
  );
  late final languages = LanguageHost(initial.bridge);
  static String gitPath(String value) => value
      .replaceFirst(r'\\?\', '')
      .replaceAll('\\', '/')
      .toLowerCase()
      .replaceAll(RegExp(r'/+$'), '');
  late final FileHost files = FileHost(
    initial.bridge,
    mutationBusy: (root) => git.workspaces.values.any(
      (w) =>
          w.mutating &&
          (gitPath(root) == gitPath(w.status?['root'] as String? ?? w.root) ||
              gitPath(root).startsWith(
                '${gitPath(w.status?['root'] as String? ?? w.root)}/',
              )),
    ),
  );
  late final GitHost git = GitHost(initial.bridge);
  late final TerminalHost terminals = TerminalHost(initial.bridge);
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
    if (terminals.live.isNotEmpty ||
        git.workspaces.values.any((w) => w.busy) ||
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
    await languages.stop();
  }

  Future<bool> prepareQuit({
    required bool saveFiles,
    bool keepScheduled = false,
  }) async {
    scheduled.suspend();
    try {
      if (scheduled.pending) {
        throw StateError(
          'A scheduled task is starting. Try closing again in a moment.',
        );
      }
      if (git.workspaces.values.any(
        (w) => w.reviewOpen != null || w.commitDraft.isNotEmpty,
      )) {
        throw StateError(
          'Close the Git review and commit or clear its message before closing. The commit draft remains.',
        );
      }
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
      await terminals.checkpoint();
      await languages.stop();
      for (final terminal in terminals.live.toList()) {
        if (!await terminals.stop(terminal)) {
          throw StateError(
            'A terminal could not stop. Retained output remains; retry closing.',
          );
        }
      }
      for (final c
          in tasks
              .where((c) => !keepScheduled || !_scheduledOwners.contains(c))
              .toList()) {
        await c.stop();
      }
      scheduled.suspend();
      companion.suspend();
      if (keepScheduled) {
        await scheduled.resume();
        return true;
      }
      await initial.bridge.close();
      return true;
    } catch (e) {
      unawaited(scheduled.resume());
      unawaited(companion.refresh());
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
    git.beforeMutation = (w, selectedPaths) {
      final root = w.status?['root'] as String? ?? w.root;
      if (tasks.any(
        (c) =>
            gitPath(c.workspaceRoot ?? '').startsWith('${gitPath(root)}/') ||
            gitPath(c.workspaceRoot ?? '') == gitPath(root),
      )) {
        throw StateError(
          'Finish or stop the task in this repository before changing Git state.',
        );
      }
      for (final d in files.documents.values) {
        final dw = files.workspaces[d.project];
        if (dw == null) continue;
        final full = gitPath(paths.join(dw.root, d.path));
        if ((selectedPaths.isEmpty
                ? (full == gitPath(root) ||
                      full.startsWith('${gitPath(root)}/'))
                : selectedPaths.any(
                    (p) => gitPath(paths.join(root, p)) == full,
                  )) &&
            (d.dirty || d.pending || d.blocked || d.text != d.acknowledged)) {
          throw StateError(
            'Save or close unsaved editors for these Git paths, then review again. Drafts remain.',
          );
        }
      }
    };
    git.afterMutation = (w) async {
      for (final fw in files.workspaces.values.where(
        (fw) =>
            gitPath(fw.root) ==
                gitPath(w.status?['root'] as String? ?? w.root) ||
            gitPath(
              fw.root,
            ).startsWith('${gitPath(w.status?['root'] as String? ?? w.root)}/'),
      )) {
        await files.refreshDocuments(fw);
        await files.load(fw, '.', refresh: true);
      }
    };
    visible = initial;
    _appearance = initial.appearance;
    _add(initial);
    unawaited(scheduled.refresh());
    unawaited(companion.refresh());
    unawaited(background.refresh());
  }
  void _add(ChatController owner) {
    _settings[owner] = owner.settingsRevision;
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
    if (owners.any((c) => _settings[c] != c.settingsRevision)) {
      for (final c in owners) {
        _settings[c] = c.settingsRevision;
      }
      unawaited(companion.refresh());
      unawaited(background.refresh());
    }
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
    if (ended.isNotEmpty) unawaited(scheduled.refresh());
    if (_appearance != visible.appearance) {
      _appearance = visible.appearance;
      appearanceChanges.value++;
    }
    notifyListeners();
  }

  Iterable<ChatController> get tasks => owners.where((c) => c.busy);
  Future<void> shareTerminal(String session, String text) async {
    var owner = owners.where((c) => c.session == session).firstOrNull;
    if (owner == null) {
      owner = await _create();
      await owner.select(session);
    }
    if (owner.session != session ||
        owner.busy ||
        owner.changing ||
        owner.loading) {
      throw StateError(
        'Finish opening or running the chosen conversation before attaching output.',
      );
    }
    final parts = await terminals.call({
      'action': 'share',
      'session': session,
      'text': text,
    });
    owner.acceptAttachmentParts(
      (parts as List).map((p) => Map<String, dynamic>.from(p)).toList(),
    );
  }

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
      final note = companion.unread;
      if (note?['session'] == id) {
        await companion.feedback(note!['id'] as String, 'seen');
      }
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
    scheduled.dispose();
    background.dispose();
    companion.dispose();
    terminals.dispose();
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
