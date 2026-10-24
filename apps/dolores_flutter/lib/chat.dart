import 'dart:async';
import 'dart:convert';
import 'dart:io';

import 'package:flutter/foundation.dart';

import 'bridge.dart';
import 'image_clipboard.dart';

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
  final appearanceChanges = ValueNotifier<String>('system');
  final reducedMotion = ValueNotifier<bool>(false);
  Future<void> saveReducedMotion(bool reduce) async {
    final current =
        await bridge.call({'command': 'experimentalPreferences'}) as Map;
    final saved = await bridge.call({
      'command': 'saveExperimentalPreferences',
      'preferences': {
        ...Map<String, dynamic>.from(current['preferences'] as Map),
        'reduceMotion': reduce,
      },
    }) as Map;
    if (!_disposed) {
      reducedMotion.value = saved['preferences']['reduceMotion'] == true;
    }
  }

  String get appearance => appearanceChanges.value;
  set appearance(String theme) {
    if (!_disposed) appearanceChanges.value = theme;
  }

  Future<void> saveAppearance(String theme) async {
    if (!['system', 'light', 'dark'].contains(theme)) {
      throw ArgumentError('Choose System, Light or Dark.');
    }
    await bridge.call({'command': 'saveAppearance', 'theme': theme});
    if (_disposed) return;
    appearance = theme;
    _notify();
  }

  Map? activeDesktopTarget;
  Future<void> revokeDesktop() async {
    if (session == null) return;
    try {
      await bridge.call({'command': 'desktopRevoke', 'session': session});
      await stop();
    } catch (e) {
      error = '$e';
      _notify();
    }
  }

  final ChatBridge bridge;
  final bool ownsBridge;
  ChatController(this.bridge, {this.ownsBridge = true});
  bool nativeStartup = false;
  Future<void> Function()? beforeNativeRestart;
  Future<void> Function(String path)? onOpenSourceControl;
  Future<void> Function(String session)? onNavigateConversation;
  Future<void> Function(String parent)? onSideConclusion;
  Map<String, dynamic>? sideConversation;
  String? sideConversationError;
  bool sideConversationsAvailable = false;
  bool promptBranchesAvailable = false;
  bool historySearchAvailable = false;
  bool attentionAvailable = false;
  Map<String, dynamic>? promptBranch;
  String? promptBranchError;

  Future<dynamic> branchCall(String owner, Map<String, dynamic> request) =>
      bridge.call({
        'command': 'promptBranch',
        'session': owner,
        'request': request,
      });
  Future<void> refreshPromptBranch() async {
    if (!promptBranchesAvailable || session == null) {
      promptBranch = null;
      promptBranchError = null;
      return;
    }
    final owner = session;
    try {
      final value = await branchCall(owner!, {'action': 'info'});
      if (session == owner) {
        promptBranch = value is Map ? Map<String, dynamic>.from(value) : null;
        promptBranchError = null;
      }
    } catch (e) {
      if (session == owner) {
        promptBranchError = '$e';
      }
    }
    _notify();
  }

  Future<void> createPromptBranch(
    String parent,
    Map<String, dynamic> request,
  ) async {
    if (session != parent) {
      throw StateError(
        'Source chat changed. Return to the original and preview again; your edit remains.',
      );
    }
    await checkpointDraft();
    final target = request['id'] as String;
    try {
      await branchCall(parent, {'action': 'create', 'request': request});
    } catch (e) {
      try {
        final saved = await branchCall(target, {'action': 'info'});
        if (saved is! Map || saved['origin']['parent'] != parent) {
          rethrow;
        }
        final initial = saved['request'] as Map;
        if (initial['id'] != target ||
            initial['edited'] != request['edited'] ||
            initial['includeAttachments'] != request['includeAttachments'] ||
            initial['includeContext'] != request['includeContext'] ||
            initial['folder'] != request['folder'] ||
            initial['targetRevision'] != request['targetRevision'] ||
            initial['basis']['message'] != request['basis']['message'] ||
            initial['basis']['revision'] != request['basis']['revision']) {
          rethrow;
        }
      } catch (_) {
        throw StateError(
          '$e Your edit remains. Retry this identity, refresh the source preview, or inspect the saved branch: $target',
        );
      }
    }
    await navigateConversation(target);
  }

  Future<void> refreshSideConversation() async {
    if (!sideConversationsAvailable || session == null) {
      sideConversation = null;
      sideConversationError = null;
      return;
    }
    final owner = session;
    try {
      final value = await bridge.call({
        'command': 'sideConversation',
        'session': owner,
        'request': {'action': 'info'},
      });
      if (session == owner) {
        sideConversation = value is Map
            ? Map<String, dynamic>.from(value)
            : null;
        sideConversationError = null;
      }
    } catch (e) {
      if (session == owner) sideConversationError = '$e';
    }
    _notify();
  }

  Future<void> navigateConversation(String target) async {
    if (onNavigateConversation != null) {
      await onNavigateConversation!(target);
      return;
    }
    if (busy || changing) {
      throw StateError(
        'Finish this response before switching in this view; the parent and excerpt remain.',
      );
    }
    await select(target);
    if (session != target) {
      throw StateError(
        error ?? 'Conversation is unavailable. The frozen excerpt remains.',
      );
    }
  }

  Future<dynamic> sideCall(String owner, Map<String, dynamic> request) =>
      bridge.call({
        'command': 'sideConversation',
        'session': owner,
        'request': request,
      });
  Future<void> createSide(String parent, Map<String, dynamic> request) async {
    if (session != parent) {
      throw StateError(
        'Source chat changed. Return to it and preview the selection again.',
      );
    }
    await checkpointDraft();
    String target = request['id'] as String;
    try {
      final result = await sideCall(parent, {
        'action': 'create',
        'request': request,
      });
      target = result['session']['id'] as String;
    } catch (e) {
      try {
        final saved = await sideCall(target, {'action': 'info'});
        if (saved is! Map ||
            saved['origin']['parent'] != parent ||
            saved['origin']['quote']['revision'] !=
                request['quote']['revision'] ||
            saved['origin']['quote']['message'] !=
                request['quote']['message'] ||
            saved['origin']['quote']['excerpt'] !=
                request['quote']['excerpt'] ||
            saved['origin']['question'] != request['question']) {
          rethrow;
        }
      } catch (_) {
        throw StateError(
          '$e The parent and question remain. Retry the same discussion or inspect its saved identity: $target',
        );
      }
    }
    await navigateConversation(target);
  }

  Future<void> returnSide(String child, Map<String, dynamic> request) async {
    if (session != child) {
      throw StateError(
        'Side chat changed. Return to it before sharing this conclusion.',
      );
    }
    await checkpointDraft();
    dynamic result;
    try {
      result = await sideCall(child, {
        'action': 'bringBack',
        'request': request,
      });
    } catch (e) {
      try {
        result = await sideCall(child, {
          'action': 'inspectReturn',
          'request': request,
        });
        if (result is! Map) {
          rethrow;
        }
      } catch (_) {
        throw StateError(
          '$e Selection and drafts remain. Inspect the return receipts before starting a different return.',
        );
      }
    }
    if (session == child) {
      sideConversation = Map<String, dynamic>.from(result);
      sideConversationError = null;
      _notify();
    }
    if (result['parentAvailable'] != true) {
      throw StateError(
        'The conclusion was already accepted, but its parent is now unavailable. This discussion and receipt remain.',
      );
    }
    final parent = result['origin']['parent'] as String;
    if (onSideConclusion != null) {
      await onSideConclusion!(parent);
    } else {
      await navigateConversation(parent);
      await refreshInputs();
      await dispatchNextInput();
    }
  }

  Future<void> saveNativeDrafts() async {
    await beforeNativeRestart?.call();
    if (busy || loading) {
      throw StateError('Finish or stop the current task first.');
    }
    _draftTimer?.cancel();
    for (final entry in _views.entries) {
      if (entry.key.isNotEmpty && entry.key != session) {
        await bridge.call({
          'command': 'saveDraft',
          'session': entry.key,
          'text': entry.value.draft,
        });
      }
    }
    if (session != null) {
      await bridge.call({
        'command': 'saveDraft',
        'session': session,
        'text': draft,
      });
    }
  }

  int settingsRevision = 0;
  Future<void> checkpointDraft() async {
    if (session == null && draft.isNotEmpty) await _ensureWorkingSession();
    _draftTimer?.cancel();
    if (session != null) {
      await bridge.call({
        'command': 'saveDraft',
        'session': session,
        'text': draft,
      });
    }
  }

  void invalidateContext() {
    settingsRevision++;
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

  Future<void> prepareWindowSharing() async {
    if (busy || changing || loading) return;
    if (workspaceKind == 'side') {
      throw StateError(
        'Side chats have no tools. Choose a project or temporary workspace first.',
      );
    }
    await _ensureWorkingSession();
    _notify();
  }

  Future<void> openProject(String root) async {
    if (busy || changing || loading || skillSaving) return;
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
  Map<String, dynamic>? userQuestion;
  List<String> questionAnswers = [];
  int questionIndex = 0;
  String? questionError;
  bool answeringQuestion = false;
  void setQuestionAnswer(int index, String value) {
    if (userQuestion == null || answeringQuestion || stopping) return;
    questionAnswers[index] = value;
    questionError = null;
    _notify();
  }

  void showQuestion(int index) {
    questionIndex = index;
    _notify();
  }

  Future<void> answerQuestion() async {
    final request = userQuestion;
    if (!busy ||
        stopping ||
        answeringQuestion ||
        request == null ||
        questionAnswers.any((a) => a.trim().isEmpty)) {
      return;
    }
    answeringQuestion = true;
    questionError = null;
    _notify();
    try {
      await bridge.call({
        'command': 'answerQuestion',
        'id': _run,
        'callId': request['callId'],
        'answers': List.of(questionAnswers),
      });
      if (!_disposed && identical(userQuestion, request)) {
        userQuestion = null;
        _record('Answer sent');
      }
    } catch (e) {
      if (!_disposed && identical(userQuestion, request)) {
        questionError = 'Could not send your answer. Try again. ($e)';
      }
    } finally {
      answeringQuestion = false;
      _notify();
    }
  }

  final toolRecords = <Map<String, dynamic>>[];
  final subagents = <Map<String, dynamic>>[];
  final modelTexts = <Map<String, dynamic>>[];
  final modelThinking = <Map<String, dynamic>>[];
  bool modelActive = false;
  bool decidingTool = false;
  int modelStep = 0;
  String modelPhase = 'waiting';
  int modelElapsedSeconds = 0;
  int modelTimeoutSeconds = 0;
  String get modelActivityLabel => userQuestion != null
      ? 'Waiting for your answer'
      : toolApproval != null
      ? 'Waiting for your review'
      : !modelActive && workspaceRoot != null
      ? 'Working…'
      : '${switch (modelPhase) {
          'reasoning' => 'Thinking',
          'toolArguments' => 'Preparing tool call',
          'responding' => 'Responding',
          _ => 'Waiting for model',
        }} · ${modelElapsedSeconds}s';
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
        userQuestion = null;
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
  bool instructionQueueAvailable = false;
  bool commandPickerAvailable = false;
  bool commandSessionsAvailable = false;
  bool worktreesAvailable = false;
  // These unsent process-input drafts survive inspector navigation in this app
  // session. They are separate from the saved conversation draft.
  final commandInputDrafts = <String, String>{};
  bool planningAvailable = false;
  Map<String, dynamic> planning = {};
  Future<void> refreshPlanning() async {
    if (!planningAvailable) return;
    if (session == null) {
      if (loading || changing || busy) return;
      changing = true;
      try {
        await _ensureWorkingSession();
      } finally {
        changing = false;
        _notify();
      }
    }
    final target = session;
    final state = await bridge.call({'command': 'planning', 'session': target});
    if (session == target) {
      planning = (state as Map).cast<String, dynamic>();
      _notify();
    }
  }

  Future<bool> changePlanning(Map<String, dynamic> edit) async {
    if (busy || changing || loading) return false;
    changing = true;
    _notify();
    try {
      await _ensureWorkingSession();
      planning = (await bridge.call({
        'command': 'editPlanning',
        'session': session,
        'revision': planning['revision'] ?? 0,
        'edit': edit,
      }) as Map).cast<String, dynamic>();
      error = null;
      invalidateContextPreview();
      return true;
    } catch (e) {
      error = '$e';
      return false;
    } finally {
      changing = false;
      _notify();
    }
  }

  Future<void> executePlan() async {
    if (busy || changing || planning['accepted'] == null) return;
    _goalStarted = planning['accepted']['goal'] != null;
    await send(taskInput: 'Execute the accepted plan.', planRun: true);
  }

  bool goalsAvailable = false;
  Map<String, dynamic>? goal;
  bool _goalStarted = false;
  Future<void> refreshGoal() async {
    if (!goalsAvailable || session == null) return;
    final target = session;
    final saved = await bridge.call({'command': 'goal', 'session': target});
    if (session == target) {
      goal = (saved as Map?)?.cast<String, dynamic>();
      _notify();
    }
  }

  Future<bool> changeGoal(Map<String, dynamic> edit) async {
    if (busy || changing || loading || skillSaving) return false;
    changing = true;
    _notify();
    try {
      await _ensureWorkingSession();
      goal = (await bridge.call({
        'command': 'editGoal',
        'session': session,
        'edit': edit,
      }) as Map).cast<String, dynamic>();
      _goalStarted = false;
      error = null;
      await refreshInputs();
      return true;
    } catch (e) {
      error = '$e';
      return false;
    } finally {
      changing = false;
      _notify();
    }
  }

  Future<void> startGoal() async {
    if (!goalsAvailable ||
        busy ||
        changing ||
        loading ||
        skillSaving ||
        session == null ||
        goal?['state'] != 'active') {
      return;
    }
    _goalStarted = true;
    await send(taskInput: 'Continue the saved goal.', goalRun: true);
  }

  Future<void> dispatchGoal() async {
    if (!_goalStarted ||
        goal?['state'] != 'active' ||
        goal?['currentRun'] != null ||
        busy ||
        changing ||
        loading ||
        inputsPaused ||
        draft.trim().isNotEmpty ||
        attachments.isNotEmpty ||
        planning['planning'] == true ||
        error != null ||
        inputEntries.any((e) => e['state'] == 'pending')) {
      return;
    }
    await startGoal();
  }

  bool skillSaving = false;
  Map<String, dynamic> commandCatalogue = {};
  List<Map<String, dynamic>> selectedSkills = [];
  Future<void> refreshCommands() async {
    if (!commandPickerAvailable) return;
    // A new project chat needs its existing workspace identity before reading
    // project-scoped skills. Serialize creation with Send/navigation.
    if (session == null && workspaceKind == 'project') {
      if (changing || loading) return;
      changing = true;
      _notify();
      try {
        await _ensureWorkingSession();
      } finally {
        changing = false;
        _notify();
      }
    }
    final target = session;
    final result = (await bridge.call({
      'command': 'composerCommands',
      'session': target,
    }) as Map).cast<String, dynamic>();
    if (session == target) {
      commandCatalogue = result;
      selectedSkills = ((result['selected'] as List?) ?? [])
          .cast<Map<String, dynamic>>();
      _notify();
    }
  }

  Future<bool> selectSkills(List<Map<String, dynamic>> selections) async {
    if (skillSaving || changing || loading) return false;
    skillSaving = true;
    _notify();
    try {
      await _ensureWorkingSession();
      final target = session;
      await bridge.call({
        'command': 'saveComposerSkills',
        'session': target,
        'selections': selections,
      });
      if (session != target) return false;
      selectedSkills = List.of(selections);
      error = null;
      invalidateContext();
      return true;
    } catch (failure) {
      error =
          'Could not save selected skills. Your draft remains. Refresh commands and try again. ($failure)';
      return false;
    } finally {
      skillSaving = false;
      _notify();
    }
  }

  bool get acceptsLiveInputs =>
      instructionQueueAvailable && !_scheduledRun && !_observationRun;
  bool inputSaving = false;
  Map<String, dynamic> taskInputs = {};
  final Set<String> _locallyPausedInputs = {};
  List<Map<String, dynamic>> get inputEntries =>
      ((taskInputs['entries'] as List?) ?? []).cast<Map<String, dynamic>>();
  bool get inputsPaused =>
      taskInputs['suspended'] == true ||
      taskInputs['recoveryRequired'] == true ||
      _locallyPausedInputs.contains(session);

  Future<void> refreshInputs() async {
    if (!instructionQueueAvailable || session == null) return;
    final target = session;
    final state = (await bridge.call({
      'command': 'taskInputs',
      'session': target,
    }) as Map).cast<String, dynamic>();
    if (session == target) taskInputs = state;
    _notify();
  }

  Future<void> changeInput(Map<String, dynamic> edit) async {
    if (inputSaving || session == null) return;
    inputSaving = true;
    _notify();
    try {
      taskInputs = (await bridge.call({
        'command': 'editTaskInputs',
        'session': session,
        'edit': edit,
      }) as Map).cast<String, dynamic>();
      error = null;
    } catch (failure) {
      error = failure.toString();
      // Refresh stale revisions while retaining unsaved text in the editor.
      try {
        await refreshInputs();
      } catch (_) {}
    } finally {
      inputSaving = false;
      _notify();
    }
  }

  String? _inputSubmissionId;
  String? _inputSubmissionText;
  Future<void> queueDraft({bool steer = false}) async {
    if (!instructionQueueAvailable ||
        skillSaving ||
        inputSaving ||
        changing ||
        loading ||
        session == null ||
        draft.trim().isEmpty) {
      return;
    }
    final text = draft;
    final submission = jsonEncode([
      session,
      steer,
      text,
      selectedSkills
          .map((s) => [s['scope'], s['name'], s['revision'], s['version']])
          .toList(),
    ]);
    if (_inputSubmissionId == null || _inputSubmissionText != submission) {
      _inputSubmissionId =
          'input-${DateTime.now().microsecondsSinceEpoch}-${++_nextRun}';
      _inputSubmissionText = submission;
    }
    // Do not let an older debounce re-save a draft after atomic acceptance.
    _draftTimer?.cancel();
    await changeInput({
      'action': 'enqueue',
      'id': _inputSubmissionId,
      'text': text,
      'mode': steer ? 'steer' : 'queue',
    });
    if (error == null) {
      if (draft == text) {
        _draftTimer?.cancel();
        _draft = '';
        viewRevision++;
      }
      _inputSubmissionId = null;
      _inputSubmissionText = null;
      _notify();
      await dispatchNextInput();
    }
  }

  Future<void> resumeInputs() async {
    await changeInput({'action': 'resume', 'revision': taskInputs['revision']});
    if (error == null) {
      _locallyPausedInputs.remove(session);
      await dispatchNextInput();
    }
  }

  Future<void> dispatchNextInput() async {
    if (!instructionQueueAvailable ||
        busy ||
        changing ||
        loading ||
        inputSaving ||
        inputsPaused ||
        !configured ||
        session == null) {
      return;
    }
    final pending = inputEntries.where((v) => v['state'] == 'pending').toList();
    pending.sort((a, b) {
      final mode = (a['mode'] == 'steer' ? 0 : 1).compareTo(
        b['mode'] == 'steer' ? 0 : 1,
      );
      return mode != 0
          ? mode
          : (a['position'] as int).compareTo(b['position'] as int);
    });
    if (pending.isNotEmpty) {
      await send(
        taskInput: pending.first['text'] as String,
        queuedInstruction: true,
      );
    }
  }

  bool attachmentsAvailable = false;
  List<Map<String, dynamic>> attachments = [];
  List<Map<String, dynamic>> pendingParts = [];
  List<String> imageModels = [];
  void reportLocalError(String message) {
    if (_disposed) return;
    error = message;
    _notify();
  }

  Future<void> addAttachment(String path) async {
    if (busy || changing || loading) return;
    changing = true;
    _notify();
    try {
      await _ensureWorkingSession();
      attachments = (await bridge.call({
        'command': 'attachFile',
        'session': session,
        'path': path,
      }) as List).cast<Map<String, dynamic>>();
      invalidateContextPreview();
      error = null;
    } catch (failure) {
      error = failure.toString();
    } finally {
      changing = false;
      _notify();
    }
  }

  void acceptAttachmentParts(List<Map<String, dynamic>> parts) {
    attachments = parts;
    invalidateContext();
    _notify();
  }

  Future<bool> pasteImage({Future<Uint8List?> Function()? readImage}) async {
    if (busy || changing || loading || _disposed) return true;
    // Reserve this chat before awaiting the clipboard: switching/sending cannot
    // attach delayed clipboard data to another session.
    changing = true;
    Directory? temporary;
    try {
      final bytes = await (readImage ?? readClipboardImage)();
      if (bytes == null) return false;
      if (_disposed) return true;
      if (bytes.isEmpty || bytes.length > maxClipboardImageBytes) {
        throw StateError(
          'Image exceeds 2 MiB or is empty. Resize it and paste again.',
        );
      }
      await _ensureWorkingSession();
      if (_disposed) return true;
      temporary = await Directory.systemTemp.createTemp('dolores-clipboard-');
      final file = File('${temporary.path}/Pasted image.png');
      await file.writeAsBytes(bytes, flush: true);
      if (_disposed) return true;
      final parts = (await bridge.call({
        'command': 'attachFile',
        'session': session,
        'path': file.path,
      }) as List).cast<Map<String, dynamic>>();
      if (!_disposed) {
        attachments = parts;
        invalidateContextPreview();
        error = null;
      }
      return true;
    } catch (failure) {
      if (!_disposed) {
        error =
            'Could not paste the image: $failure. Use Attach file or copy a smaller PNG/JPEG and paste again. Your draft remains.';
      }
      return true;
    } finally {
      if (temporary != null) {
        try {
          await temporary.delete(recursive: true);
        } catch (_) {
          if (!_disposed) error = 'Image retained, but its temporary clipboard copy could not be removed. Your draft remains.';
        }
      }
      changing = false;
      _notify();
    }
  }

  Future<void> removeAttachment(String digest) async {
    if (busy || changing || loading || session == null) return;
    changing = true;
    _notify();
    try {
      attachments = (await bridge.call({
        'command': 'removeAttachment',
        'session': session,
        'digest': digest,
      }) as List).cast<Map<String, dynamic>>();
      invalidateContextPreview();
      error = null;
    } catch (failure) {
      error = failure.toString();
    } finally {
      changing = false;
      _notify();
    }
  }

  String? resumeRun;
  Timer? _draftTimer;
  String get draft => _draft;
  set draft(String value) {
    if (_draft == value) return;
    _draft = value;
    if (value.trim().isEmpty && !busy) resumeRun = null;
    _draftTimer?.cancel();
    if (!durableDrafts ||
        session == null ||
        (busy && !instructionQueueAvailable) ||
        changing ||
        loading) {
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
    if (draft.trim().isNotEmpty || attachments.isNotEmpty) {
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
    'maxOutputTokens': null,
    'timeoutSeconds': 180,
  };
  Map<String, dynamic> defaultRequestSettings = {
    'maxOutputTokens': null,
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
  static int _nextRun = 0;
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
      if (nativeStartup) {
        if (error != null) {
          throw StateError('Chat initialization failed: $error');
        }
        await bridge.call({'command': 'nativeStartupReady'});
        final deadline = DateTime.now().add(const Duration(seconds: 35));
        while ((await bridge.call({'command': 'nativeUpdateReady'}))['ready'] !=
            true) {
          if (DateTime.now().isAfter(deadline)) {
            throw StateError(
              'Native startup verification did not finish. Keep retained versions and history; inspect Native repairs before recovery.',
            );
          }
          await Future<void>.delayed(const Duration(milliseconds: 150));
        }
      }
    } catch (failure) {
      error = failure.toString();
    } finally {
      loading = false;
      _notify();
    }
  }

  Future<void> refresh() async {
    final state = await bridge.call({'command': 'bootstrap'});
    nativeStartup = state['nativeStartup'] == true;
    appearance = state['appearance'] as String? ?? 'system';
    reducedMotion.value = state['reduceMotion'] == true;
    durableDrafts = state['durableDrafts'] == true;
    instructionQueueAvailable = state['instructionQueue'] == true;
    commandPickerAvailable = state['commandPicker'] == true;
    commandSessionsAvailable = state['commandSessions'] == true;
    worktreesAvailable = state['worktrees'] == true;
    sideConversationsAvailable = state['sideConversations'] == true;
    promptBranchesAvailable = state['promptBranches'] == true;
    historySearchAvailable = state['historySearch'] == true;
    attentionAvailable = state['attention'] == true;
    goalsAvailable = state['goals'] == true;
    planningAvailable = state['planning'] == true;
    attachmentsAvailable = state['attachments'] == true;
    imageModels = ((state['imageModels'] as List?) ?? []).cast<String>();
    _setConversationList(state);
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
        {'maxOutputTokens': null, 'timeoutSeconds': 180};
    defaultRequestSettings =
        (state['defaultRequestSettings'] as Map?)?.cast<String, dynamic>() ??
        Map.of(requestSettings);
    modelRequestSettings =
        (state['modelRequestSettings'] as Map?)?.cast<String, dynamic>() ?? {};
  }

  Future<void> refreshConversationList() async {
    final state = await bridge.call({'command': 'bootstrap'});
    // A retained chat owns its draft, model and active run. Navigation refreshes
    // only the shared catalog, without reloading those conversation settings.
    _setConversationList(state);
    _notify();
  }

  void _setConversationList(dynamic state) {
    final nextSessions = (state['sessions'] as List)
        .cast<Map<String, dynamic>>()
        .toList();
    final nextProjects = ((state['projects'] as List?) ?? [])
        .cast<Map<String, dynamic>>()
        .toList();
    final hasOlder = state['sessionPage']?['hasOlder'] == true;
    final hasNewer = state['sessionPage']?['hasNewer'] == true;
    sessions = nextSessions;
    projects = nextProjects;
    sessionsOlder = hasOlder;
    sessionsNewer = hasNewer;
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
      defaultRequestSettings = Map.of(settings);
      requestSettings =
          (modelRequestSettings[model] as Map?)?.cast<String, dynamic>() ??
          Map.of(settings);
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

  Future<void> removeModel(String value) async {
    if (busy || changing || loading) {
      throw StateError('Finish the current action first.');
    }
    changing = true;
    _notify();
    try {
      await bridge.call({
        'command': 'removeModel',
        'preferences': {'baseUrl': baseUrl, 'model': value},
      });
      // Apply the acknowledged mutation without a fallible bootstrap reread.
      enabledModels = enabledModels.where((m) => m != value).toList();
      if (model == value) {
        model = enabledModels.firstOrNull ?? '';
        requestSettings =
            (modelRequestSettings[model] as Map?)?.cast<String, dynamic>() ??
            Map.of(defaultRequestSettings);
      }
      configured = configured && model.isNotEmpty;
      modelContexts.remove(value);
      modelRequestSettings.remove(value);
      imageModels = imageModels.where((m) => m != value).toList();
      contextSummary = null;
      contextBasis = null;
      error = null;
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
    promptBranch = null;
    promptBranchError = null;
    sideConversation = null;
    sideConversationError = null;
    goal = null;
    planning = {};
    _goalStarted = false;
    selectedSkills = [];
    commandCatalogue = {};
    taskInputs = {};
    toolRecords.clear();
    subagents.clear();
    modelTexts.clear();
    toolApproval = null;
    userQuestion = null;
    contextSummary = null;
    contextBasis = null;
    messages = [];
    attachments = [];
    pendingParts = [];
    messagesOlder = messagesNewer = false;
    draft = '';
    scrollOffset = 0;
    viewRevision++;
  }

  void newChat({String? kind}) {
    if (skillSaving) return;
    if (!busy && !changing) {
      toolRecords.clear();
      subagents.clear();
      modelTexts.clear();
      toolApproval = null;
      userQuestion = null;
    }
    if (busy || changing) return;
    _rememberView();
    final nextKind =
        kind ?? (workspaceKind == 'project' ? 'project' : 'temporary');
    if (nextKind != 'project') workspaceRoot = null;
    workspaceKind = nextKind;
    session = null;
    sideConversation = null;
    sideConversationError = null;
    promptBranch = null;
    promptBranchError = null;
    goal = null;
    planning = {};
    _goalStarted = false;
    taskInputs = {};
    selectedSkills = [];
    commandCatalogue = {};
    resumeRun = null;
    contextSummary = null;
    contextBasis = null;
    messages = [];
    attachments = [];
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
    if (busy || changing || skillSaving) return;
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
      final restoredDraft =
          state?.draft ??
          (durableDrafts
              ? await bridge.call({'command': 'savedDraft', 'session': id})
                    as String
              : '');
      final restoredAttachments = attachmentsAvailable
          ? (await bridge.call({
              'command': 'draftAttachments',
              'session': id,
            }) as List).cast<Map<String, dynamic>>()
          : <Map<String, dynamic>>[];
      session = id;
      sideConversation = null;
      await refreshSideConversation();
      promptBranch = null;
      await refreshPromptBranch();
      goal = null;
      planning = {};
      _goalStarted = false;
      await refreshGoal();
      await refreshPlanning();
      selectedSkills = [];
      commandCatalogue = {};
      await refreshCommands();
      taskInputs = {};
      await refreshInputs();
      _setWorkspace(workspace);
      toolRecords.clear();
      subagents.clear();
      modelTexts.clear();
      toolApproval = null;
      userQuestion = null;
      _setMessages(history);
      attachments = restoredAttachments;
      contextSummary = null;
      contextBasis = null;
      _restoreContext();
      _draftTimer?.cancel();
      _draft = restoredDraft;
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
    if (busy || changing || skillSaving) return;
    changing = true;
    _notify();
    try {
      await bridge.call({'command': 'delete', 'session': id});
      if (session == id) {
        session = null;
        sideConversation = null;
        sideConversationError = null;
        promptBranch = null;
        promptBranchError = null;
        workspaceKind = 'temporary';
        workspaceRoot = null;
        messages = [];
        attachments = [];
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
  bool _scheduledRun = false;
  bool _queuedRun = false;
  bool _observationRun = false;
  String? _desktopDraft;
  bool _desktopKeepsAttachments = false;
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
        draft.isNotEmpty ||
        attachments.isNotEmpty) {
      return;
    }
    if (!configured) {
      error = 'Set up a model connection first.';
      _notify();
      return;
    }
    draft = messages.last['metadata']?['paused']?['reason'] == 'commandReview'
        ? 'Repair the failed check and verify it by rerunning the same command. Propose any needed changes for fresh review.'
        : 'Continue working on the previous task.';
    await send(continuation: sourceId);
  }

  Future<void> send({
    bool goalRun = false,
    bool planRun = false,
    String? scheduledOccurrence,
    String? taskInput,
    bool queuedInstruction = false,
    bool desktopHandoff = false,
    int? continuation,
    String? desktopCapture,
    String? desktopGrant,
    String? desktopResume,
    bool desktopReconciled = false,
    Map? desktopTarget,
    String? observationModel,
  }) async {
    if (busy ||
        skillSaving ||
        changing ||
        loading ||
        (taskInput == null && draft.trim().isEmpty && attachments.isEmpty)) {
      return;
    }
    if (!configured && scheduledOccurrence == null) {
      error = 'Set up a model connection first.';
      _notify();
      return;
    }
    if (session == null && scheduledOccurrence == null) {
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
    activeDesktopTarget = desktopGrant == null ? null : desktopTarget;
    _continuing = continuation != null;
    _scheduledRun = scheduledOccurrence != null;
    _queuedRun = queuedInstruction;
    _observationRun = observationModel != null;
    stopping = false;
    if (taskInput != null && !queuedInstruction) _desktopDraft = draft;
    _desktopKeepsAttachments =
        desktopHandoff || queuedInstruction || goalRun || planRun;
    pendingInput =
        taskInput ??
        (draft.trim().isEmpty ? 'Please review the attached files.' : draft);
    pendingParts = desktopHandoff || queuedInstruction || goalRun || planRun
        ? []
        : [for (final part in attachments) Map.of(part)];
    if (!queuedInstruction && !goalRun && !planRun) draft = '';
    partial = '';
    error = null;
    final id = _run = ++_nextRun;
    toolRecords.clear();
    subagents.clear();
    modelTexts.clear();
    toolApproval = null;
    userQuestion = null;
    modelStep = 0;
    modelThinking.clear();
    modelActive = false;
    modelPhase = 'waiting';
    modelElapsedSeconds = 0;
    modelTimeoutSeconds = 0;
    recovery = null;
    _recoveryError = null;
    _requestModel = observationModel ?? model;
    _firstDelta = false;
    _terminal = false;
    _clock
      ..reset()
      ..start();
    _record('Request submitted');
    _notify();
    try {
      final started = await bridge.call({
        'command': 'start',
        'id': id,
        'session': session,
        'input': pendingInput,
        if (queuedInstruction) 'command': 'startTaskInput',
        if (goalRun) ...{'command': 'startGoal', 'revision': goal?['revision']},
        if (planRun) ...{
          'command': 'startPlan',
          'revision': planning['revision'],
        },
        'continuation': ?continuation,
        'resumeRun': ?(desktopResume ?? resumeRun),
        'desktopHandoff': desktopHandoff,
        'desktopReconciled': desktopReconciled,
        'desktopCapture': ?desktopCapture,
        'desktopGrant': ?desktopGrant,
        'observationModel': ?observationModel,
        if (scheduledOccurrence != null) ...{
          'command': 'scheduledStart',
          'occurrence': scheduledOccurrence,
        },
      });
      if (queuedInstruction && started is Map && started['input'] is String) {
        pendingInput = started['input'] as String;
      }
      if (scheduledOccurrence != null && started is Map) {
        session = started['session'] as String;
        _requestModel = started['model'] as String;
        model = started['model'] as String;
        _setWorkspace(started['workspace']);
      }
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
    _goalStarted = false;
    if (!busy) return;
    if (session != null) _locallyPausedInputs.add(session!);
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
        Duration(
          milliseconds: toolApproval == null && userQuestion == null ? 25 : 200,
        ),
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
          case 'queued':
            modelPhase = 'queued';
            _record('Queued · waiting for a run slot or project');
          case 'admitted':
            modelPhase = 'waiting';
            _clock
              ..reset()
              ..start();
            _record('Run admitted');
          case 'subagent':
            if (event['child'] is Map) {
              final child = (event['child'] as Map).cast<String, dynamic>();
              final index = subagents.indexWhere(
                (v) => v['childId'] == child['childId'],
              );
              if (index >= 0) {
                subagents[index] = child;
              } else if (subagents.length < 2) {
                subagents.add(child);
              }
              _record('Subagent ${child['scope']} · ${child['status']}');
            }
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
            if (partial.isNotEmpty) {
              modelTexts.add({'number': modelStep, 'text': partial});
            }
            partial = '';
            modelStep = event['number'] as int;
            modelActive = true;
            modelPhase = 'waiting';
            modelElapsedSeconds = 0;
            modelTimeoutSeconds = 0;
            _record('Model call $modelStep');
          case 'modelActivity':
            if (event['number'] == modelStep && !stopping) {
              modelPhase = event['phase'] as String;
              modelElapsedSeconds = event['elapsedSeconds'] as int;
              modelTimeoutSeconds = event['timeoutSeconds'] as int;
            }
          case 'modelText':
            if (event['number'] == modelStep) {
              if (!_firstDelta) {
                _record('First response text');
                _firstDelta = true;
              }
              partial += event['text'] as String;
            }
          case 'modelThinking':
            if (event['number'] == modelStep && !stopping) {
              final index = modelThinking.indexWhere(
                (s) => s['number'] == modelStep,
              );
              final step = {
                'number': modelStep,
                'text': event['text'] as String,
              };
              if (index < 0) {
                modelThinking.add(step);
              } else {
                modelThinking[index] = step;
              }
            }
          case 'modelFinished':
            if (event['number'] == modelStep) modelActive = false;
          case 'toolApproval':
            if (!stopping) {
              toolApproval = (event['request'] as Map).cast<String, dynamic>();
              _record('Tool requested');
            }
          case 'userQuestion':
            if (!stopping) {
              final request = (event['request'] as Map).cast<String, dynamic>();
              if (userQuestion?['callId'] != request['callId']) {
                userQuestion = request;
                questionAnswers = List.filled(
                  (request['questions'] as List).length,
                  '',
                );
                questionIndex = 0;
                questionError = null;
              }
              _record('Waiting for your answer');
            }
          case 'toolResult':
            toolApproval = null;
            userQuestion = null;
            toolRecords.add({
              ...(event['record'] as Map).cast<String, dynamic>(),
              'modelStep': modelStep,
            });
            _record('Tool result');
          case 'memoryUpdating':
            _record('Reply saved · learning preferences');
          case 'learningProgress':
            _record(
              event['note'] as String? ?? 'Reply saved · checking skill repair',
            );
          case 'modHint':
            _record('Recovery guidance · ${event['card']['text']}');
          case 'done':
            for (final child in subagents) {
              if (['queued', 'running'].contains(child['status'])) {
                child['status'] = 'interrupted';
                child['note'] = 'Parent run ended. Inspect Run history and Changes before continuing.';
              }
            }
            toolApproval = null;
            userQuestion = null;
            changing = true;
            if (event['error'] != null) {
              _failed(
                event['error'] as String,
                advice: (event['recovery'] as Map?)?.cast<String, dynamic>(),
              );
            } else {
              _record('Reply saved');
              if (!_desktopKeepsAttachments) attachments = [];
              if (event['knowledgeUpdate'] is String) {
                _record('Project knowledge: ${event['knowledgeUpdate']}');
              }
              if (event['learningUpdate'] is String) {
                _record('Skill learning: ${event['learningUpdate']}');
              }
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
              if (_desktopDraft != null) {
                draft = _desktopDraft!;
                _desktopDraft = null;
              }
              partial = '';
              toolRecords.clear();
              subagents.clear();
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
      if (events.isNotEmpty) {
        _notify();
        await refreshInputs();
        await refreshGoal();
        await refreshPlanning();
      }
      _schedule(id);
      if (!busy && error == null) await dispatchNextInput();
      if (!busy && error == null && events.any((e) => e['type'] == 'done')) {
        await dispatchGoal();
      }
    } catch (failure) {
      changing = false;
      if (!_disposed && id == _run) {
        _failed(failure.toString());
        _notify();
      }
    }
  }

  void _failed(String failure, {Map<String, dynamic>? advice}) {
    if (session != null) _locallyPausedInputs.add(session!);
    if (_scheduledRun &&
        (advice?['kind'] == 'stopped' ||
            failure == 'Response stopped.' ||
            failure == 'Response stopped. Your message was not saved.')) {
      failure = 'Scheduled task stopped.';
      advice = {
        'kind': 'stopped',
        'retryable': false,
        'guidance': 'Open Scheduled to run it again.',
      };
    }
    toolApproval = null;
    userQuestion = null;
    // Keep already delivered public text available to inspect/copy. It is not
    // a saved exchange and must never become the next request's history.
    if (!_terminal && partial.isNotEmpty) {
      modelTexts.add({'number': modelStep, 'text': partial, 'partial': true});
    }
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
    // Editing during a running response must survive Stop or provider failure.
    draft = draft.isNotEmpty
        ? draft
        : (_desktopDraft ?? (_continuing || _queuedRun ? '' : pendingInput));
    _desktopDraft = null;
    _continuing = false;
    pendingInput = '';
    pendingParts = [];
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
    appearanceChanges.dispose();
    reducedMotion.dispose();
    _timer?.cancel();
    _draftTimer?.cancel();
    if (ownsBridge) unawaited(bridge.close());
    super.dispose();
  }
}
