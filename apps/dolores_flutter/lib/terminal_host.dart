import 'dart:async';
import 'dart:convert';

import 'package:flutter/widgets.dart';
import 'package:xterm/xterm.dart';

import 'bridge.dart';

class TerminalSession {
  final String id, cwd, shell;
  final Terminal terminal = Terminal(maxLines: 5000);
  final controller = TerminalController();
  final focus = FocusNode();
  final scroll = ScrollController();
  late final ByteConversionSink decoder;
  String state, title, notice = '';
  bool polling = false, suspended = false;
  TerminalSession(Map value)
    : id = value['id'],
      cwd = value['cwd'],
      shell = value['shell'],
      state = value['state'],
      title = value['shell'].toString().split(RegExp(r'[/\\]')).last {
    decoder = const Utf8Decoder(allowMalformed: true).startChunkedConversion(
      StringConversionSink.fromStringSink(_TerminalSink(terminal)),
    );
  }
  bool get live => state == 'running' || state == 'unknown';
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
  String? active, error;
  bool opening = false, disposed = false;
  Timer? _timer;
  TerminalHost(this.bridge);
  Iterable<TerminalSession> get live => sessions.values.where((s) => s.live);
  Future<dynamic> call(Map<String, dynamic> request) =>
      bridge.call({'command': 'terminal', 'request': request});
  Future<void> enter(String? session) async {
    if (sessions.isEmpty && !opening) await create(session);
  }

  Future<TerminalSession?> create(String? session, {bool home = false}) async {
    if (opening || disposed) return null;
    opening = true;
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
      sessions[s.id] = s;
      active = s.id;
      s.terminal.onOutput = (text) => unawaited(input(s, text));
      s.terminal.onResize = (width, height, _, _) =>
          unawaited(resize(s, height, width));
      s.terminal.onTitleChange = (title) {
        s.title = title.length > 120 ? title.substring(0, 120) : title;
        if (!disposed) notifyListeners();
      };
      _schedule();
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
    active = id;
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
      await call({'action': 'close', 'id': s.id});
      sessions.remove(s.id);
      if (active == s.id) active = sessions.keys.firstOrNull;
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

  Future<bool> stop(TerminalSession s) async {
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
    disposed = true;
    _timer?.cancel();
    for (final s in sessions.values) {
      s.dispose();
    }
    super.dispose();
  }
}
