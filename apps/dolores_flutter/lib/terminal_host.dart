import 'dart:async';
import 'dart:convert';

import 'package:flutter/widgets.dart';
import 'package:xterm/xterm.dart';

import 'bridge.dart';
import 'file_layout.dart';

class TerminalSession {
  final String id, cwd, shell;
  final Terminal terminal = Terminal(maxLines: 5000);
  final controller = TerminalController();
  final focus = FocusNode();
  final scroll = ScrollController();
  late final ByteConversionSink decoder;
  String state, title, notice = '';
  bool polling = false, suspended = false;
  bool restored = false;
  TerminalSession(Map value)
    : id = value['id'],
      cwd = value['cwd'],
      shell = value['shell'],
      state = value['state'],
      title = displayTitle(value['shell'].toString()) {
    decoder = const Utf8Decoder(allowMalformed: true).startChunkedConversion(
      StringConversionSink.fromStringSink(_TerminalSink(terminal)),
    );
  }
  bool get live => state == 'running' || state == 'unknown';
  String get displayCwd => cwd.replaceFirst(RegExp(r'^\\\\\?\\'), '');
  static String displayTitle(String value) {
    final name = value
        .split(RegExp(r'[/\\]'))
        .last
        .replaceAll(RegExp(r'[\x00-\x1f\x7f]'), '');
    if (name.toLowerCase() == 'pwsh.exe' ||
        name.toLowerCase() == 'powershell.exe') {
      return 'PowerShell';
    }
    if (name.toLowerCase() == 'cmd.exe') return 'Command Prompt';
    return name.length > 120 ? name.substring(0, 120) : name;
  }

  String selectedText() => controller.selection == null
      ? ''
      : terminal.buffer.getText(controller.selection);
  void dispose() {
    decoder.close();
    controller.dispose();
    focus.dispose();
    scroll.dispose();
  }
}

class _TerminalSink implements StringSink {
  final Terminal terminal;
  _TerminalSink(this.terminal);
  @override
  void write(Object? value) => terminal.write('$value');
  @override
  void writeAll(Iterable objects, [String separator = '']) =>
      write(objects.join(separator));
  @override
  void writeCharCode(int charCode) => write(String.fromCharCode(charCode));
  @override
  void writeln([Object? value = '']) => write('$value\n');
}

/// App-owned emulators drain live PTYs even while their page is hidden.
class TerminalHost extends ChangeNotifier {
  final ChatBridge bridge;
  final sessions = <String, TerminalSession>{};
  final layout = FileLayout();
  String? error;
  String? get active => layout.active.active;
  bool opening = false, disposed = false;
  Timer? _timer, _checkpointTimer;
  bool entered = false, checkpointFailed = false;
  Future<void>? _saving;
  TerminalHost(this.bridge) {
    layout.onChanged = () {
      changed();
      notifyListeners();
    };
  }
  Iterable<TerminalSession> get live => sessions.values.where((s) => s.live);
  Future<dynamic> call(Map<String, dynamic> request) =>
      bridge.call({'command': 'terminal', 'request': request});
  Future<void> enter(String? session) async {
    if (!entered) {
      entered = true;
      try {
        final v = await call({'action': 'restore'});
        if (v is Map) restore(v);
      } catch (e) {
        error =
            '$e Open a new terminal to continue; the previous checkpoint remains.';
        checkpointFailed = true;
        notifyListeners();
        return;
      }
    }
    if (sessions.isEmpty && !opening) await create(session);
  }

  void restore(Map v) {
    if (v['version'] != 1 ||
        v['sessions'] is! List ||
        (v['sessions'] as List).length > 8) {
      throw StateError('Terminal recovery is invalid.');
    }
    final retained = <String, TerminalSession>{};
    try {
      for (final item in v['sessions']) {
        final s = TerminalSession({...item, 'state': 'stopped'});
        if (retained.containsKey(s.id)) {
          s.dispose();
          throw StateError('Duplicate terminal.');
        }
        retained[s.id] = s;
        s.restored = true;
        s.title = item['title'];
        // Plain display text: never feed stored escape sequences to the parser.
        final text = (item['output'] as String).replaceAll(
          RegExp(r'[\x00-\x08\x0b-\x1f\x7f]'),
          '',
        );
        s.terminal.write(text.replaceAll('\n', '\r\n'));
        s.notice = 'Previous session stopped. Retained output is limited to the last 8 KiB. Open a new terminal to run commands.';
      }
      final groups = <String, FileGroup>{};
      final placed = <String>{};
      for (final item in v['groups']) {
        final g = FileGroup(item['id']);
        g.tabs.addAll((item['tabs'] as List).cast<String>());
        g.active = item['active'];
        if (groups.containsKey(g.id) ||
            g.tabs.any((id) => !retained.containsKey(id) || !placed.add(id)) ||
            (g.active != null && !g.tabs.contains(g.active))) {
          throw StateError('Terminal groups are invalid.');
        }
        groups[g.id] = g;
      }
      final leaves = <String>{};
      FileLayoutNode node(Map value, int depth) {
        if (depth > 3) throw StateError('Terminal layout is too deep.');
        if (value['group'] is String) {
          if (!groups.containsKey(value['group']) ||
              !leaves.add(value['group'])) {
            throw StateError('Terminal layout is invalid.');
          }
          return FileLayoutNode.leaf(value['group']);
        }
        final ratio = (value['ratio'] as num).toDouble();
        if (ratio < .15 ||
            ratio > .85 ||
            !['horizontal', 'vertical'].contains(value['axis'])) {
          throw StateError('Terminal split is invalid.');
        }
        return FileLayoutNode.split(
          value['axis'] == 'vertical' ? Axis.vertical : Axis.horizontal,
          node(value['first'], depth + 1),
          node(value['second'], depth + 1),
          ratio: ratio,
        );
      }

      final tree = node(v['tree'], 0);
      if (groups.isEmpty ||
          groups.length > 4 ||
          leaves.length != groups.length ||
          placed.length != retained.length ||
          !groups.containsKey(v['activeGroup'])) {
        throw StateError('Terminal recovery is incomplete.');
      }
      sessions.addAll(retained);
      layout.groups
        ..clear()
        ..addAll(groups);
      layout.tree = tree;
      layout.activeGroup = v['activeGroup'];
      notifyListeners();
    } catch (_) {
      for (final s in retained.values) {
        s.dispose();
      }
      rethrow;
    }
  }

  static String tail(String text) {
    final bytes = utf8.encode(text);
    var start = (bytes.length - 8192).clamp(0, bytes.length);
    while (start < bytes.length && (bytes[start] & 0xc0) == 0x80) {
      start++;
    }
    return utf8.decode(bytes.sublist(start));
  }

  Map<String, dynamic> snapshot() => {
    'version': 1,
    'sessions': [
      for (final s in sessions.values)
        {
          'id': s.id,
          'cwd': s.cwd,
          'shell': s.shell,
          'title': s.title,
          'output': tail(s.terminal.buffer.getText()),
        },
    ],
    'groups': [
      for (final g in layout.groups.values)
        {'id': g.id, 'tabs': g.tabs.toList(), 'active': g.active},
    ],
    'activeGroup': layout.activeGroup,
    'tree': layout.tree.json(),
  };
  void changed() {
    if (!entered || disposed || checkpointFailed || _checkpointTimer != null) {
      return;
    }
    _checkpointTimer = Timer(const Duration(seconds: 2), () {
      _checkpointTimer = null;
      unawaited(
        checkpoint().catchError((Object e) {
          if (!disposed) {
            error = '$e Retry saving recovery before closing.';
            notifyListeners();
          }
        }),
      );
    });
  }

  Future<void> checkpoint() async {
    if (!entered) return;
    _checkpointTimer?.cancel();
    _checkpointTimer = null;
    if (_saving != null) await _saving;
    final value = snapshot();
    if (utf8.encode(jsonEncode(value)).length > 96 * 1024) {
      throw StateError(
        'Terminal recovery exceeds 96 KiB. Close an unused tab before closing Dolores.',
      );
    }
    final saving = call({'action': 'checkpoint', 'value': value})
        .then<void>((_) {});
    _saving = saving;
    try {
      await saving;
      checkpointFailed = false;
    } catch (_) {
      checkpointFailed = true;
      rethrow;
    } finally {
      _saving = null;
    }
  }

  Future<TerminalSession?> create(
    String? session, {
    bool home = false,
    String? group,
  }) async {
    if (opening || disposed) return null;
    opening = true;
    final target = group ?? layout.activeGroup;
    error = null;
    notifyListeners();
    try {
      final value = await call({
        'action': 'create',
        'session': session,
        'home': home,
      });
      if (disposed) {
        await call({'action': 'close', 'id': value['id']});
        return null;
      }
      final s = TerminalSession(value);
      entered = true;
      checkpointFailed = false;
      sessions[s.id] = s;
      if (layout.groups.containsKey(target)) layout.focus(target);
      layout.open(s.id, preview: false);
      s.terminal.onOutput = (text) => unawaited(input(s, text));
      s.terminal.onResize = (width, height, _, _) =>
          unawaited(resize(s, height, width));
      s.terminal.onTitleChange = (title) {
        s.title = TerminalSession.displayTitle(title);
        if (!disposed) notifyListeners();
      };
      _schedule();
      changed();
      return s;
    } catch (e) {
      error = '$e';
      return null;
    } finally {
      opening = false;
      if (!disposed) notifyListeners();
    }
  }

  void select(String id) {
    if (!sessions.containsKey(id)) return;
    final group = layout.groups.values
        .where((g) => g.tabs.contains(id))
        .firstOrNull;
    if (group != null) layout.select(group.id, id);
    error = null;
    notifyListeners();
  }

  Future<void> input(TerminalSession s, String text) async {
    if (!s.live || disposed) return;
    try {
      await call({'action': 'input', 'id': s.id, 'text': text});
    } catch (e) {
      s.notice = '$e';
      if (!disposed) notifyListeners();
    }
  }

  Future<void> resize(TerminalSession s, int rows, int cols) async {
    if (!s.live || disposed) return;
    try {
      await call({
        'action': 'resize',
        'id': s.id,
        'rows': rows.clamp(2, 500),
        'cols': cols.clamp(2, 500),
      });
    } catch (e) {
      s.notice = '$e';
      if (!disposed) notifyListeners();
    }
  }

  void _schedule() {
    _timer?.cancel();
    if (disposed || !live.any((s) => !s.suspended)) return;
    _timer = Timer(const Duration(milliseconds: 100), () async {
      await Future.wait(live.where((s) => !s.suspended).toList().map(poll));
      _schedule();
    });
  }

  Future<void> poll(TerminalSession s) async {
    if (s.polling || disposed) return;
    s.polling = true;
    try {
      final value = await call({'action': 'poll', 'id': s.id});
      if (disposed || sessions[s.id] != s) return;
      final before = s.state;
      s.decoder.add(base64Decode(value['bytes']));
      if (value['bytes'] != '') changed();
      s.state = value['session']['state'];
      if (value['outputError'] == true) s.notice = 'Terminal output failed. Retained output remains; Stop and open another shell.';
      if (before != s.state || value['outputError'] == true) notifyListeners();
    } catch (e) {
      s.notice = '$e';
      s.state = 'unknown';
      s.suspended = true;
      if (!disposed) notifyListeners();
    } finally {
      s.polling = false;
    }
  }

  void retry(TerminalSession s) {
    s.suspended = false;
    s.notice = '';
    _schedule();
    notifyListeners();
  }

  Future<bool> close(TerminalSession s) async {
    try {
      if (!s.restored) await call({'action': 'close', 'id': s.id});
      sessions.remove(s.id);
      layout.removeDocument(s.id);
      for (final group
          in layout.groups.values.where((g) => g.tabs.isEmpty).toList()) {
        layout.closeGroup(group.id);
      }
      s.dispose();
      notifyListeners();
      return true;
    } catch (e) {
      s.notice = '$e';
      error = '$e';
      notifyListeners();
      return false;
    }
  }

  Future<void> splitNew(String? session, String group, Axis axis) async {
    if (layout.groups.length >= 4) {
      error = 'Four terminal groups are open. Move a tab to an existing group or close a group.';
      notifyListeners();
      return;
    }
    final s = await create(session, group: group);
    if (s != null) layout.split(group, s.id, axis, source: group);
  }

  Future<bool> stop(TerminalSession s) async {
    if (s.restored) return true;
    try {
      final v = await call({'action': 'stop', 'id': s.id});
      s.state = v['state'];
      await poll(s);
      notifyListeners();
      return true;
    } catch (e) {
      s.state = 'unknown';
      s.notice = '$e';
      error = '$e';
      notifyListeners();
      return false;
    }
  }

  @override
  void dispose() {
    layout.onChanged = null;
    disposed = true;
    _timer?.cancel();
    _checkpointTimer?.cancel();
    for (final s in sessions.values) {
      s.dispose();
    }
    super.dispose();
  }
}
