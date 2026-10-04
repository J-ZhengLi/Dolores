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
  void invalidateContext() {
    contextSummary = null;
    contextBasis = null;
    _notify();
  }

  Future<void> inspectLocalChanges(Future<void> Function() inspect) async {
    if (session == null || workspaceRoot == null) return;
    await inspectLocalSettings(inspect);
  }

  Future<void> inspectLocalSettings(Future<void> Function() inspect) async {
    if (busy || changing || loading) return;
    changing = true;
    _notify();
    try {
      await inspect();
    } finally {
      changing = false;
      _notify();
    }
  }

  String? workspaceRoot;
  String workspaceKind = 'temporary';
  List<Map<String, dynamic>> projects = [];
  String get workspaceLabel => workspaceKind == 'project'
      ? projects
                .where((p) => p['root'] == workspaceRoot)
                .map((p) => p['name'] as String)
                .firstOrNull ??
            'Project'
      : workspaceKind == 'side'
      ? 'Side chat'
      : 'Temporary workspace';
  void _setWorkspace(dynamic value) {
    final data = (value as Map?)?.cast<String, dynamic>();
    workspaceKind = data?['kind'] as String? ?? 'side';
    workspaceRoot = data?['root'] as String?;
  }

  Future<void> _ensureWorkingSession() async {
    if (session != null) return;
    final result = await bridge.call({
      'command': 'createSession',
      'kind': workspaceKind,
      if (workspaceKind == 'project') 'path': workspaceRoot,
    });
    if (_disposed) return;
    session = result['session']['id'] as String;
    _setWorkspace(result['workspace']);
    sessions.insert(0, {
      ...(result['session'] as Map).cast<String, dynamic>(),
      'workspace': result['workspace'],
    });
    if (sessions.length > 50) {
      sessions.removeLast();
      sessionsOlder = true;
    }
  }

  Future<void> openProject(String root) async {
    if (busy || changing || loading) return;
    changing = true;
    _notify();
    try {
      final result = await bridge.call({
        'command': 'createSession',
        'kind': 'project',
        'path': root,
      });
      if (_disposed) return;
      _rememberView();
      session = result['session']['id'] as String;
      _setWorkspace(result['workspace']);
      _clearConversation();
      await refresh();
      error = null;
    } catch (failure) {
      error = failure.toString();
    } finally {
      changing = false;
      _notify();
    }
  }

  Map<String, dynamic>? toolApproval;
  final toolRecords = <Map<String, dynamic>>[];
  final modelTexts = <Map<String, dynamic>>[];
  bool decidingTool = false;
  int modelStep = 0;
  Future<void> chooseToolFolder(Future<String?> Function() choose) async {
    if (busy || changing || loading) return;
    changing = true;
    _notify();
    try {
      final folder = await choose();
      changing = false;
      if (!_disposed && folder != null) await openProject(folder);
    } catch (_) {
      if (!_disposed) error = 'Could not open the folder picker.';
    } finally {
      changing = false;
      _notify();
    }
  }

  void disableTools() {
    if (busy || changing || loading) return;
    newChat(kind: 'side');
  }

  Future<void> decideTool(bool allow) async {
    final request = toolApproval;
    if (!busy || stopping || decidingTool || request == null) return;
    decidingTool = true;
    _notify();
    try {
      await bridge.call({
        'command': 'approveTool',
        'id': _run,
        'callId': request['callId'],
        'allow': allow,
      });
      if (!_disposed && identical(toolApproval, request)) {
        toolApproval = null;
        _record(allow ? 'Tool allowed' : 'Tool denied');
      }
    } catch (_) {
      // A terminal/Stop event can invalidate this one-use decision. Keep
      // polling the run instead of replacing its failure/restoration state.
      if (!_disposed && identical(toolApproval, request)) toolApproval = null;
    } finally {
      decidingTool = false;
      _notify();
    }
  }

  List<Map<String, dynamic>> sessions = [];
  List<Map<String, dynamic>> messages = [];
  List<String> enabledModels = [];
  String baseUrl = 'http://localhost:11434/v1',
      model = '',
      _draft = '',
      pendingInput = '',
      partial = '';
  bool durableDrafts = false;
  String? resumeRun;
  Timer? _draftTimer;
  String get draft => _draft;
  set draft(String value) {
    if (_draft == value) return;
    _draft = value;
    if (value.trim().isEmpty && !busy) resumeRun = null;
    _draftTimer?.cancel();
    if (!durableDrafts || session == null || busy || changing || loading) {
      return;
    }
    final target = session!, text = value;
    _draftTimer = Timer(const Duration(milliseconds: 250), () async {
      try {
        await bridge.call({
          'command': 'saveDraft',
          'session': target,
          'text': text,
        });
      } catch (failure) {
        if (!_disposed && session == target) {
          error =
              'Draft could not be saved: $failure. Your text is still here; copy it or edit again to retry.';
          _notify();
        }
      }
    });
  }

  Future<void> prepareCheckpoint(String source) async {
    if (busy || changing || loading || session == null) return;
    if (draft.trim().isNotEmpty) {
      throw StateError(
        'Send or clear the current draft before preparing recovery.',
      );
    }
    final value = await bridge.call({
      'command': 'checkpointDraft',
      'session': session,
      'runId': source,
    });
    if (_disposed) return;
    draft = value as String;
    resumeRun = source;
    viewRevision++;
    _notify();
  }

  String? session, error, connectionWarning;
  Map<String, dynamic>? recovery;
  String? _recoveryError;
  Map<String, dynamic>? get activeRecovery =>
      error == _recoveryError ? recovery : null;
  Map<String, dynamic> requestSettings = {
    'maxOutputTokens': 2048,
    'timeoutSeconds': 180,
  };
  Map<String, dynamic> defaultRequestSettings = {
    'maxOutputTokens': 2048,
    'timeoutSeconds': 180,
  };
  Map<String, dynamic> modelRequestSettings = {};
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
  Map<String, int?> modelContexts = {};
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
    if (durableDrafts && session != null && !busy) {
      final target = session!, text = draft;
      _draftTimer?.cancel();
      unawaited(
        bridge
            .call({'command': 'saveDraft', 'session': target, 'text': text})
            .catchError((Object failure) {
              if (!_disposed) {
                error =
                    'Draft could not be saved: $failure. The text remains in this chat until you close the app.';
                _notify();
              }
              return null;
            }),
      );
    }
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

  void _restoreContext() {
    if (messagesNewer || messages.isEmpty) return;
    final metadata = messages.last['metadata'] as Map?;
    if (metadata?['context'] is! Map) return;
    if (metadata?['model'] is String && metadata!['model'] != model) {
      contextSummary = null;
      contextBasis = null;
      return;
    }
    final summary = Map<String, dynamic>.from(metadata!['context'] as Map);
    if (summary['tokens'] is Map &&
        summary['tokens']['contextWindowTokens'] !=
            (modelContexts[model] ?? 131072)) {
      contextSummary = null;
      contextBasis = null;
      return;
    }
    final calls = metadata['agent']?['usageByCall'] as List?;
    final usage = calls != null && calls.isNotEmpty
        ? calls.last
        : metadata['usage'];
    final total = usage?['totalTokens'];
    if (total is int && total >= 0) summary['reportedTokens'] = total;
    contextSummary = summary;
    contextBasis = total is int
        ? 'Last model call'
        : metadata['agent'] == null
        ? 'Last saved request'
        : 'Saved agent input';
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
        if (workspaceKind != 'side') 'tools': true,
      });
      error = null;
      final report = (result as Map).cast<String, dynamic>();
      contextSummary = Map.of(report)
        ..remove('messages')
        ..remove('tools')
        ..remove('memoryEntries');
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
    durableDrafts = state['durableDrafts'] == true;
    sessions = (state['sessions'] as List).cast<Map<String, dynamic>>();
    projects = ((state['projects'] as List?) ?? [])
        .cast<Map<String, dynamic>>();
    sessionsOlder = state['sessionPage']?['hasOlder'] == true;
    sessionsNewer = state['sessionPage']?['hasNewer'] == true;
    baseUrl = state['preferences']['baseUrl'] as String;
    model = state['preferences']['model'] as String;
    enabledModels =
        ((state['enabledModels'] as List?) ?? (model.isEmpty ? [] : [model]))
            .cast<String>();
    modelContexts = ((state['modelContexts'] as Map?) ?? {})
        .cast<String, int?>();
    configured = state['configured'] as bool;
    rememberConnection = state['rememberConnection'] == true;
    hasSavedKey = state['hasSavedKey'] == true;
    connectionWarning = state['connectionWarning'] as String?;
    requestSettings =
        (state['requestSettings'] as Map?)?.cast<String, dynamic>() ??
        {'maxOutputTokens': 2048, 'timeoutSeconds': 180};
    defaultRequestSettings =
        (state['defaultRequestSettings'] as Map?)?.cast<String, dynamic>() ??
        Map.of(requestSettings);
    modelRequestSettings =
        (state['modelRequestSettings'] as Map?)?.cast<String, dynamic>() ?? {};
  }

  Future<void> saveModelRequestSettings(
    String endpoint,
    String name,
    Map<String, dynamic>? settings,
  ) async {
    if (busy || changing || loading) {
      throw StateError('Finish the current action first.');
    }
    changing = true;
    _notify();
    try {
      final result = await bridge.call({
        'command': 'setModelRequestSettings',
        'preferences': {'baseUrl': endpoint, 'model': name},
        'settings': settings,
      });
      requestSettings = (result['requestSettings'] as Map)
          .cast<String, dynamic>();
      modelRequestSettings = (result['modelRequestSettings'] as Map)
          .cast<String, dynamic>();
      contextSummary = null;
      contextBasis = null;
    } finally {
      changing = false;
      _notify();
    }
  }

  Future<void> saveRequestSettings(Map<String, dynamic> settings) async {
    if (busy || changing || loading) {
      throw StateError('Finish the current action first.');
    }
    changing = true;
    _notify();
    try {
      await bridge.call({
        'command': 'setRequestSettings',
        'settings': settings,
      });
      // The acknowledged command changed only these settings. A second
      // bootstrap read could fail after a successful save and misreport it.
      requestSettings = Map.of(settings);
      contextSummary = null;
      contextBasis = null;
    } finally {
      changing = false;
      _notify();
    }
  }

  Future<void> configure(
    String url,
    String name,
    String? key, {
    bool remember = false,
    List<String>? models,
    Map<String, int?>? contexts,
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
        'modelContexts': ?contexts,
      });
      await refresh();
      contextSummary = null;
      contextBasis = null;
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
      contextSummary = null;
      contextBasis = null;
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

  void _clearConversation() {
    toolRecords.clear();
    modelTexts.clear();
    toolApproval = null;
    contextSummary = null;
    contextBasis = null;
    messages = [];
    messagesOlder = messagesNewer = false;
    draft = '';
    scrollOffset = 0;
    viewRevision++;
  }

  void newChat({String? kind}) {
    if (!busy && !changing) {
      toolRecords.clear();
      modelTexts.clear();
      toolApproval = null;
    }
    if (busy || changing) return;
    _rememberView();
    final nextKind =
        kind ?? (workspaceKind == 'project' ? 'project' : 'temporary');
    if (nextKind != 'project') workspaceRoot = null;
    workspaceKind = nextKind;
    session = null;
    resumeRun = null;
    contextSummary = null;
    contextBasis = null;
    messages = [];
    messagesOlder = messagesNewer = false;
    draft = '';
    scrollOffset = 0;
    viewRevision++;
    error = null;
    _notify();
  }

  Future<void> forkAt(int through) async {
    if (busy || changing || session == null) return;
    try {
      final result = await bridge.call({
        'command': 'forkSession',
        'session': session,
        'through': through,
      }) as Map;
      await refresh();
      await select(result['session']['id'] as String);
    } catch (failure) {
      error = failure.toString();
      _notify();
    }
  }

  Future<void> select(String id) async {
    if (busy || changing) return;
    _rememberView();
    changing = true;
    _notify();
    try {
      final state = _views[id];
      final workspace = await bridge.call({
        'command': 'workspace',
        'session': id,
      });
      final history = await bridge.call({
        'command': 'messagesPage',
        'session': id,
        'cursor': state?.cursor,
      });
      session = id;
      _setWorkspace(workspace);
      toolRecords.clear();
      modelTexts.clear();
      toolApproval = null;
      _setMessages(history);
      contextSummary = null;
      contextBasis = null;
      _restoreContext();
      _draftTimer?.cancel();
      _draft =
          state?.draft ??
          (durableDrafts
              ? await bridge.call({'command': 'savedDraft', 'session': id})
                    as String
              : '');
      resumeRun = null;
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
        workspaceKind = 'temporary';
        workspaceRoot = null;
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

  bool _continuing = false;
  int? get latestPausedId =>
      !messagesNewer &&
          messages.isNotEmpty &&
          messages.last['role'] == 'assistant' &&
          messages.last['metadata']?['paused'] != null
      ? messages.last['id'] as int?
      : null;

  Future<void> continueTask(int sourceId) async {
    if (busy ||
        changing ||
        loading ||
        sourceId != latestPausedId ||
        draft.isNotEmpty) {
      return;
    }
    if (!configured) {
      error = 'Set up a model connection first.';
      _notify();
      return;
    }
    draft = 'Continue working on the previous task.';
    await send(continuation: sourceId);
  }

  Future<void> send({int? continuation}) async {
    if (busy || changing || loading || draft.trim().isEmpty) return;
    if (!configured) {
      error = 'Set up a model connection first.';
      _notify();
      return;
    }
    if (session == null) {
      changing = true;
      _notify();
      try {
        await _ensureWorkingSession();
      } catch (failure) {
        error = failure.toString();
        changing = false;
        _notify();
        return;
      }
      changing = false;
    }
    if (_disposed) return;
    if (messagesNewer) {
      await browseMessages(newer: false, latest: true);
      if (messagesNewer || error != null) return;
    }
    busy = true;
    _continuing = continuation != null;
    stopping = false;
    pendingInput = draft;
    draft = '';
    partial = '';
    error = null;
    final id = ++_run;
    toolRecords.clear();
    modelTexts.clear();
    toolApproval = null;
    modelStep = 0;
    recovery = null;
    _recoveryError = null;
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
        'continuation': ?continuation,
        'resumeRun': ?resumeRun,
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
      _timer = Timer(
        Duration(milliseconds: toolApproval == null ? 25 : 200),
        () => _poll(id),
      );
    }
  }

  Future<void> _poll(int id) async {
    try {
      final events = await bridge.call({'command': 'poll', 'id': id});
      if (_disposed || id != _run) return;
      for (final event in events as List) {
        if (event['id'] != id) continue;
        switch (event['type']) {
          case 'compacting':
            _record('Compacting context · one bounded attempt');
          case 'compacted':
            _record(
              'Context compacted · ${event['summary']['coveredTurns']} turns · usage ${event['usage']}',
            );
          case 'started':
            session = event['session'] as String;
            for (final record in requestLogs.where(
              (record) => record.run == id,
            )) {
              record.session = session;
            }
            contextSummary = (event['context'] as Map?)
                ?.cast<String, dynamic>();
            contextBasis = workspaceRoot == null
                ? 'Current request'
                : 'Initial agent input';
            _record('Context prepared');
          case 'delta':
            if (!_firstDelta) {
              _record('First response text');
              _firstDelta = true;
            }
            partial += event['text'] as String;
          case 'modelStep':
            if (partial.isNotEmpty && modelTexts.length < 3) {
              modelTexts.add({'number': modelStep, 'text': partial});
            }
            partial = '';
            modelStep = event['number'] as int;
            _record('Model call $modelStep');
          case 'modelText':
            if (event['number'] == modelStep) {
              if (!_firstDelta) {
                _record('First response text');
                _firstDelta = true;
              }
              partial += event['text'] as String;
            }
          case 'toolApproval':
            if (!stopping) {
              toolApproval = (event['request'] as Map).cast<String, dynamic>();
              _record('Tool requested');
            }
          case 'toolResult':
            toolApproval = null;
            if (toolRecords.length < 4) {
              toolRecords.add((event['record'] as Map).cast<String, dynamic>());
            }
            _record('Tool result');
          case 'memoryUpdating':
            _record('Reply saved · learning preferences');
          case 'done':
            toolApproval = null;
            changing = true;
            if (event['error'] != null) {
              _failed(
                event['error'] as String,
                advice: (event['recovery'] as Map?)?.cast<String, dynamic>(),
              );
            } else {
              _record('Reply saved');
              if (event['memoryUpdate'] is Map) {
                final update = event['memoryUpdate'] as Map;
                _record('Memory: ${update['status']} · ${update['note']}');
              }
              _terminal = true;
              _clock.stop();
              contextBasis = workspaceRoot == null
                  ? 'Last saved request'
                  : 'Saved agent input';
              busy = false;
              resumeRun = null;
              stopping = false;
              pendingInput = '';
              partial = '';
              toolRecords.clear();
              modelTexts.clear();
              _setMessages(
                await bridge.call({
                  'command': 'messagesPage',
                  'session': session,
                }),
              );
              _restoreContext();
            }
            await refresh();
            if (event['evidenceWarning'] is String) {
              error = event['evidenceWarning'] as String;
              _record('Run evidence needs inspection');
            }
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

  void _failed(String failure, {Map<String, dynamic>? advice}) {
    toolApproval = null;
    recovery = _terminal ? null : advice;
    _recoveryError = failure;
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
    } else if (contextBasis == 'Initial agent input') {
      contextBasis = 'Last attempted agent input';
    }
    _clock.stop();
    error = failure;
    draft = _continuing ? '' : pendingInput;
    _continuing = false;
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
    _draftTimer?.cancel();
    unawaited(bridge.close());
    super.dispose();
  }
}
