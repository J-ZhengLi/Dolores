import 'dart:async';

import 'package:flutter/foundation.dart';

import 'bridge.dart';

class RequestLog {
  final int run;
  String? session;
  final String model, label;
  final DateTime time;
  final int elapsedMs;
  RequestLog(
    this.run,
    this.session,
    this.model,
    this.label,
    this.time,
    this.elapsedMs,
  );
}

class _ViewState {
  final String draft;
  final double scroll;
  final int? cursor;
  _ViewState(this.draft, this.scroll, this.cursor);
}

class ChatController extends ChangeNotifier {
  final ChatBridge bridge;
  ChatController(this.bridge);
  List<Map<String, dynamic>> sessions = [];
  List<Map<String, dynamic>> messages = [];
  List<String> enabledModels = [];
  String baseUrl = 'http://localhost:11434/v1',
      model = '',
      draft = '',
      pendingInput = '',
      partial = '';
  String? session, error, connectionWarning;
  bool rememberConnection = false, hasSavedKey = false;
  bool loading = true,
      configured = false,
      busy = false,
      changing = false,
      stopping = false;
  bool _disposed = false;
  int _run = 0;
  Timer? _timer;
  bool sessionsOlder = false, sessionsNewer = false;
  bool messagesOlder = false, messagesNewer = false;
  int viewRevision = 0;
  double scrollOffset = 0;
  final _views = <String, _ViewState>{};
  // Lifecycle only: no duplicate prompts, token chunks, credentials or HTTP payloads.
  final requestLogs = <RequestLog>[];
  final _clock = Stopwatch();
  String _requestModel = '';
  bool _firstDelta = false;
  bool _terminal = false;
  Map<String, dynamic>? contextSummary;
  String? contextBasis;

  void invalidateContextPreview() {
    if (contextBasis == 'Next message preview') {
      contextSummary = null;
      contextBasis = null;
    }
  }

  void _record(String label) {
    requestLogs.add(
      RequestLog(
        _run,
        session,
        _requestModel,
        label,
        DateTime.now(),
        _clock.elapsedMilliseconds,
      ),
    );
    if (requestLogs.length > 200) requestLogs.removeAt(0);
  }

  Future<Map<String, dynamic>?> inspectHistory({
    int? cursor,
    bool newer = false,
  }) async {
    if (busy || changing || loading || session == null) return null;
    changing = true;
    _notify();
    try {
      return (await bridge.call({
        'command': 'messagesPage',
        'session': session,
        'cursor': cursor,
        'newer': newer,
      }) as Map).cast<String, dynamic>();
    } finally {
      changing = false;
      _notify();
    }
  }

  void rememberScroll(double offset) => scrollOffset = offset;
  void _rememberView() {
    final key = session ?? '';
    _views.remove(key);
    _views[key] = _ViewState(
      draft,
      scrollOffset,
      messagesNewer && messages.isNotEmpty
          ? (messages.last['id'] as int) + 1
          : null,
    );
    while (_views.length > 20) {
      _views.remove(_views.keys.first);
    }
  }

  void _setMessages(dynamic page) {
    messages = (page['items'] as List).cast<Map<String, dynamic>>();
    messagesOlder = page['hasOlder'] == true;
    messagesNewer = page['hasNewer'] == true;
  }

  Future<Map<String, dynamic>?> previewContext() async {
    if (busy || changing || loading) return null;
    changing = true;
    _notify();
    try {
      final result = await bridge.call({
        'command': 'context',
        'session': session,
        'input': draft,
      });
      error = null;
      final report = (result as Map).cast<String, dynamic>();
      contextSummary = Map.of(report)..remove('messages');
      contextBasis = 'Next message preview';
      return report;
    } catch (failure) {
      error = failure.toString();
      return null;
    } finally {
      changing = false;
      _notify();
    }
  }

  Future<void> initialize() async {
    try {
      await bridge.open();
      await refresh();
      if (sessions.isNotEmpty) await select(sessions.first['id'] as String);
    } catch (failure) {
      error = failure.toString();
    } finally {
      loading = false;
      _notify();
    }
  }

  Future<void> refresh() async {
    final state = await bridge.call({'command': 'bootstrap'});
    sessions = (state['sessions'] as List).cast<Map<String, dynamic>>();
    sessionsOlder = state['sessionPage']?['hasOlder'] == true;
    sessionsNewer = state['sessionPage']?['hasNewer'] == true;
    baseUrl = state['preferences']['baseUrl'] as String;
    model = state['preferences']['model'] as String;
    enabledModels =
        ((state['enabledModels'] as List?) ?? (model.isEmpty ? [] : [model]))
            .cast<String>();
    configured = state['configured'] as bool;
    rememberConnection = state['rememberConnection'] == true;
    hasSavedKey = state['hasSavedKey'] == true;
    connectionWarning = state['connectionWarning'] as String?;
  }

  Future<void> configure(
    String url,
    String name,
    String? key, {
    bool remember = false,
    List<String>? models,
  }) async {
    if (busy || changing) throw StateError('Finish the current action first.');
    changing = true;
    _notify();
    try {
      await bridge.call({
        'command': 'configure',
        'preferences': {'baseUrl': url.trim(), 'model': name.trim()},
        'apiKey': key,
        'rememberConnection': remember,
        'enabledModels': ?models,
      });
      await refresh();
      error = null;
    } finally {
      changing = false;
      _notify();
    }
  }

  Future<List<String>> listModels(String url, String? key) async {
    if (busy || changing) throw StateError('Finish the current action first.');
    changing = true;
    _notify();
    try {
      final result = await bridge.call({
        'command': 'listModels',
        'baseUrl': url.trim(),
        'apiKey': key,
      });
      return (result as List).cast<String>();
    } finally {
      changing = false;
      _notify();
    }
  }

  Future<void> selectModel(String value) async {
    if (busy || changing || !configured || value == model) return;
    changing = true;
    _notify();
    try {
      await bridge.call({'command': 'selectModel', 'model': value});
      await refresh();
      error = null;
    } catch (failure) {
      error = failure.toString();
    } finally {
      changing = false;
      _notify();
    }
  }

  Future<void> recoverConnection() async {
    if (busy || changing) return;
    changing = true;
    _notify();
    try {
      await bridge.call({'command': 'recoverConnection'});
      error = null;
    } catch (failure) {
      error = failure.toString();
    } finally {
      try {
        await refresh();
      } catch (failure) {
        error = failure.toString();
      }
      changing = false;
      _notify();
    }
  }

  Future<void> forgetConnection() async {
    if (busy || changing) throw StateError('Finish the current action first.');
    changing = true;
    _notify();
    try {
      await bridge.call({'command': 'forgetConnection'});
      error = null;
    } finally {
      try {
        await refresh();
      } finally {
        changing = false;
        _notify();
      }
    }
  }

  void newChat() {
    if (busy || changing) return;
    _rememberView();
    session = null;
    contextSummary = null;
    contextBasis = null;
    messages = [];
    messagesOlder = messagesNewer = false;
    draft = _views['']?.draft ?? '';
    scrollOffset = 0;
    viewRevision++;
    error = null;
    _notify();
  }

  Future<void> select(String id) async {
    if (busy || changing) return;
    _rememberView();
    changing = true;
    _notify();
    try {
      final state = _views[id];
      final history = await bridge.call({
        'command': 'messagesPage',
        'session': id,
        'cursor': state?.cursor,
      });
      session = id;
      _setMessages(history);
      contextSummary = null;
      contextBasis = null;
      if (!messagesNewer &&
          messages.isNotEmpty &&
          messages.last['metadata']?['context'] is Map) {
        contextSummary = (messages.last['metadata']['context'] as Map)
            .cast<String, dynamic>();
        contextBasis = 'Last saved request';
      }
      draft = state?.draft ?? '';
      scrollOffset = state?.scroll ?? double.infinity;
      viewRevision++;
      error = null;
    } catch (failure) {
      error = failure.toString();
    } finally {
      changing = false;
      _notify();
    }
  }

  Future<void> browseSessions({required bool newer}) async {
    if (busy || changing || loading) return;
    changing = true;
    _notify();
    try {
      final edge = sessions.isEmpty
          ? null
          : (newer ? sessions.first : sessions.last);
      final page = await bridge.call({
        'command': 'sessionsPage',
        'newer': newer && edge != null,
        'cursor': edge == null
            ? null
            : {'id': edge['id'], 'updatedAt': edge['updatedAt']},
      });
      sessions = (page['items'] as List).cast<Map<String, dynamic>>();
      sessionsOlder = page['hasOlder'] == true;
      sessionsNewer = page['hasNewer'] == true;
      error = null;
    } catch (failure) {
      error = failure.toString();
    } finally {
      changing = false;
      _notify();
    }
  }

  Future<void> browseMessages({
    required bool newer,
    bool latest = false,
  }) async {
    if (busy || changing || session == null) return;
    changing = true;
    _notify();
    try {
      final edge = messages.isEmpty
          ? null
          : (newer ? messages.last : messages.first);
      final page = await bridge.call({
        'command': 'messagesPage',
        'session': session,
        'cursor': latest ? null : edge?['id'],
        'newer': !latest && newer && edge != null,
      });
      _setMessages(page);
      scrollOffset = latest ? double.infinity : 0;
      viewRevision++;
      error = null;
    } catch (failure) {
      error = failure.toString();
    } finally {
      changing = false;
      _notify();
    }
  }

  Future<int?> exportConversation(
    String format,
    Future<String?> Function() choosePath,
  ) async {
    if (busy || changing || loading || session == null) return null;
    changing = true;
    _notify();
    try {
      final path = await choosePath();
      if (path == null || _disposed) return null;
      final result = await bridge.call({
        'command': 'export',
        'session': session,
        'path': path,
        'format': format,
      });
      error = null;
      return result['messageCount'] as int;
    } catch (failure) {
      error = failure.toString();
      return null;
    } finally {
      changing = false;
      _notify();
    }
  }

  Future<void> delete(String id) async {
    if (busy || changing) return;
    changing = true;
    _notify();
    try {
      await bridge.call({'command': 'delete', 'session': id});
      if (session == id) {
        session = null;
        messages = [];
        messagesOlder = messagesNewer = false;
        draft = '';
        scrollOffset = 0;
        viewRevision++;
      }
      _views.remove(id);
      requestLogs.removeWhere((event) => event.session == id);
      if (session == null) {
        contextSummary = null;
        contextBasis = null;
      }
      await refresh();
      error = null;
    } catch (failure) {
      error = failure.toString();
    } finally {
      changing = false;
      _notify();
    }
  }

  Future<void> send() async {
    if (busy || changing || loading || draft.trim().isEmpty) return;
    if (!configured) {
      error = 'Set up a model connection first.';
      _notify();
      return;
    }
    if (messagesNewer) {
      await browseMessages(newer: false, latest: true);
      if (messagesNewer || error != null) return;
    }
    busy = true;
    stopping = false;
    pendingInput = draft;
    draft = '';
    partial = '';
    error = null;
    final id = ++_run;
    _requestModel = model;
    _firstDelta = false;
    _terminal = false;
    _clock
      ..reset()
      ..start();
    _record('Request submitted');
    _notify();
    try {
      await bridge.call({
        'command': 'start',
        'id': id,
        'session': session,
        'input': pendingInput,
      });
      // Remember a Stop pressed before the native reservation was acknowledged.
      if (stopping) await bridge.call({'command': 'cancel', 'id': id});
      _schedule(id);
    } catch (failure) {
      if (!_disposed && id == _run) {
        _failed(failure.toString());
        _notify();
      }
    }
  }

  Future<void> stop() async {
    if (!busy) return;
    stopping = true;
    _record('Stop requested');
    _notify();
    try {
      await bridge.call({'command': 'cancel', 'id': _run});
    } catch (failure) {
      if (!_disposed) {
        error = failure.toString();
        _notify();
      }
    }
  }

  void _schedule(int id) {
    if (!_disposed && busy && id == _run) {
      // Bounded active polling only; zero idle timers.
      _timer = Timer(const Duration(milliseconds: 25), () => _poll(id));
    }
  }

  Future<void> _poll(int id) async {
    try {
      final events = await bridge.call({'command': 'poll', 'id': id});
      if (_disposed || id != _run) return;
      for (final event in events as List) {
        if (event['id'] != id) continue;
        switch (event['type']) {
          case 'started':
            session = event['session'] as String;
            for (final record in requestLogs.where(
              (record) => record.run == id,
            )) {
              record.session = session;
            }
            contextSummary = (event['context'] as Map?)
                ?.cast<String, dynamic>();
            contextBasis = 'Current request';
            _record('Context prepared');
          case 'delta':
            if (!_firstDelta) {
              _record('First response text');
              _firstDelta = true;
            }
            partial += event['text'] as String;
          case 'done':
            changing = true;
            if (event['error'] != null) {
              _failed(event['error'] as String);
            } else {
              _record('Reply saved');
              _terminal = true;
              _clock.stop();
              contextBasis = 'Last saved request';
              busy = false;
              stopping = false;
              pendingInput = '';
              partial = '';
              _setMessages(
                await bridge.call({
                  'command': 'messagesPage',
                  'session': session,
                }),
              );
            }
            await refresh();
            changing = false;
        }
      }
      if (events.isNotEmpty) _notify();
      _schedule(id);
    } catch (failure) {
      changing = false;
      if (!_disposed && id == _run) {
        _failed(failure.toString());
        _notify();
      }
    }
  }

  void _failed(String failure) {
    _record(
      _terminal
          ? 'History refresh failed'
          : stopping
          ? 'Request stopped'
          : 'Request failed',
    );
    _terminal = true;
    if (contextBasis == 'Current request') {
      contextBasis = 'Last attempted request';
    }
    _clock.stop();
    error = failure;
    draft = pendingInput;
    pendingInput = '';
    partial = '';
    busy = false;
    stopping = false;
  }

  void _notify() {
    if (!_disposed) notifyListeners();
  }

  @override
  void dispose() {
    _disposed = true;
    _timer?.cancel();
    unawaited(bridge.close());
    super.dispose();
  }
}
