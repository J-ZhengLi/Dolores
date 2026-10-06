import 'package:flutter/foundation.dart';

import 'chat.dart';
import 'file_host.dart';

/// One bridge/profile owner; visible conversation and running owners are separate.
class AppHost extends ChangeNotifier {
  late final FileHost files = FileHost(initial.bridge);
  final ChatController initial;
  final owners = <ChatController>[];
  late ChatController visible;
  bool selecting = false;
  String? error;
  bool panelHidden = false;
  double panelWidth = 252;
  final appearanceChanges = ValueNotifier(0);
  String? _appearance;
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
    owners.add(owner);
    owner.addListener(_changed);
  }

  void _changed() {
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
    files.dispose();
    for (final owner in owners) {
      owner.removeListener(_changed);
    }
    for (final owner in owners.where((c) => c != initial)) {
      owner.dispose();
    }
    initial.dispose();
    appearanceChanges.dispose();
    super.dispose();
  }
}
