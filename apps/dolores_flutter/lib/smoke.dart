// Separate diagnostic entry point. Normal releases do not include this runner.
import 'dart:async';
import 'dart:convert';
import 'dart:io';
import 'dart:ui' as ui;

import 'package:flutter/material.dart';
import 'package:flutter/rendering.dart';
import 'package:path/path.dart' as path;

import 'bridge.dart';
import 'chat.dart';
import 'code_syntax.dart';
import 'rich_composer.dart';
import 'main.dart';
import 'reply_content.dart';
import 'usage_details.dart';
import 'inspector.dart';
import 'request_settings.dart';

void main() {
  WidgetsFlutterBinding.ensureInitialized();
  final output = Platform.environment['DOLORES_SMOKE_DIR'];
  final data = Platform.environment['DOLORES_DATA_DIR'];
  if (output == null ||
      data == null ||
      !path.isAbsolute(output) ||
      !path.isAbsolute(data)) {
    throw StateError(
      'Smoke testing requires absolute isolated output and data directories.',
    );
  }
  final chat = ChatController(NativeBridge());
  final capture = GlobalKey();
  runApp(
    DoloresApp(chat: chat, captureKey: capture, themeMode: ThemeMode.light),
  );
  unawaited(_run(chat, capture, Directory(output)));
}

Future<void> waitUntil(bool Function() condition) async {
  final deadline = DateTime.now().add(const Duration(seconds: 20));
  while (!condition()) {
    if (DateTime.now().isAfter(deadline)) {
      throw StateError('Smoke test timed out.');
    }
    await Future<void>.delayed(const Duration(milliseconds: 25));
  }
}

Future<void> screenshot(GlobalKey key, Directory directory, String name) async {
  await Future<void>.delayed(const Duration(milliseconds: 400));
  await WidgetsBinding.instance.endOfFrame;
  final boundary =
      key.currentContext!.findRenderObject() as RenderRepaintBoundary;
  final image = await boundary.toImage(pixelRatio: 1.5);
  try {
    final data = await image.toByteData(format: ui.ImageByteFormat.png);
    await File(path.join(directory.path, '$name.png'))
        .writeAsBytes(data!.buffer.asUint8List());
  } finally {
    image.dispose();
  }
}

void check(bool condition, String message, List<String> checks) {
  if (!condition) throw StateError(message);
  checks.add(message);
}

Future<void> _run(
  ChatController chat,
  GlobalKey capture,
  Directory output,
) async {
  final checks = <String>[];
  try {
    await output.create(recursive: true);
    await chat.initialize();
    if (chat.error != null) throw StateError(chat.error!);
    if (chat.sessions.isNotEmpty) {
      throw StateError('Use a fresh smoke database.');
    }
    final models = await chat.listModels('http://127.0.0.1:19421/v1', '');
    check(
      models.contains('dolores-mock') &&
          models.contains('dolores-fast') &&
          !chat.configured,
      'Discovery lists models without saving or connecting',
      checks,
    );
    await chat.configure(
      'http://127.0.0.1:19421/v1',
      'dolores-mock',
      '',
      models: models,
    );
    chat.newChat(kind: 'side');
    chat.draft = 'hello';
    await chat.send();
    await waitUntil(() => !chat.busy);
    check(
      chat.messages.length == 2 &&
          chat.messages.last['content'].contains('你好！'),
      'Unicode stream saved as a complete turn',
      checks,
    );
    final savedUsage = chat.messages.last['metadata']?['usage'];
    check(
      savedUsage?['inputTokens'] == 64 &&
          savedUsage?['outputTokens'] == 32 &&
          savedUsage?['totalTokens'] == 96 &&
          savedUsage?['cachedInputTokens'] == 0 &&
          savedUsage?['reasoningTokens'] == null,
      'Final usage-only SSE chunk survives the bridge and SQLite reload with zero and unavailable details intact',
      checks,
    );
    final id = chat.session!;
    chat.draft = 'slow';
    await chat.send();
    await waitUntil(() => chat.partial.isNotEmpty);
    await chat.stop();
    await waitUntil(() => !chat.busy);
    check(
      chat.messages.length == 2 &&
          chat.draft == 'slow' &&
          chat.error!.contains('stopped'),
      'Stop restores draft without saving a partial turn',
      checks,
    );
    chat.draft = 'fail';
    await chat.send();
    await waitUntil(() => !chat.busy);
    check(
      chat.messages.length == 2 &&
          chat.draft == 'fail' &&
          !chat.error!.contains('fixture-private-error-body'),
      'Provider denial hides its raw body and restores draft',
      checks,
    );
    chat.draft = 'truncated';
    await chat.send();
    await waitUntil(() => !chat.busy);
    check(
      chat.messages.length == 2 &&
          chat.draft == 'truncated' &&
          chat.error != null,
      'Interrupted stream does not save a partial turn',
      checks,
    );
    chat.draft = 'slow';
    await chat.send();
    await waitUntil(() => !chat.busy);
    check(
      chat.messages.length == 4 &&
          chat.messages.last['content'].endsWith('check cancellation.'),
      'Slow stream completes and saves the entire answer',
      checks,
    );
    chat.newChat();
    await chat.select(id);
    check(
      chat.messages.length == 4 && chat.messages.first['content'] == 'hello',
      'SQLite history reloads complete turns',
      checks,
    );
    chat.draft = 'model-check';
    await chat.selectModel('dolores-fast');
    check(
      chat.model == 'dolores-fast' &&
          chat.draft == 'model-check' &&
          chat.messages.length == 4,
      'Model switch keeps the draft and conversation',
      checks,
    );
    await chat.send();
    await waitUntil(() => !chat.busy);
    check(
      chat.error == null && chat.messages.length == 6,
      'The selected chat model is used in the real provider request',
      checks,
    );
    chat.newChat(kind: 'side');
    chat.draft = 'markdown';
    await chat.send();
    await waitUntil(() => chat.partial.contains('```rust'));
    await screenshot(capture, output, 'partial-code');
    await waitUntil(() => !chat.busy);
    check(
      chat.error == null && chat.messages.last['content'].endsWith('RICH_END'),
      'Markdown and code stream through the real provider without losing source',
      checks,
    );
    final richId = chat.session!;
    chat.newChat();
    await chat.select(richId);
    check(
      chat.messages.length == 2 &&
          chat.messages.last['content'].contains('```rust'),
      'Rich reply reload preserves its original Markdown source',
      checks,
    );
    await screenshot(capture, output, 'conversation-light');
    var codeBlocks = 0, tables = 0;
    void inspectRich(Element element) {
      if (element.widget is ReplyCodeBlock) codeBlocks++;
      if (element.widget is Table) tables++;
      element.visitChildren(inspectRich);
    }

    inspectRich(capture.currentContext! as Element);
    check(
      codeBlocks == 1 && tables == 1,
      'Actual release renders native code and table widgets',
      checks,
    );
    runApp(
      DoloresApp(chat: chat, captureKey: capture, themeMode: ThemeMode.dark),
    );
    await screenshot(capture, output, 'conversation-dark');
    const composerSource =
        '# Example\n\n```rust\nfn main() {\n    let x = 1;\n    println!("{x} 世界!");\n}\n```';
    chat.draft = composerSource;
    await chat.selectModel('dolores-mock');
    await screenshot(capture, output, 'composer-dark');
    TextField? heading, code;
    BuildContext? composerFieldContext;
    RichComposer? composer;
    void inspectComposer(Element element) {
      if (element.widget is RichComposer) {
        composer = element.widget as RichComposer;
      }
      if (element.widget is TextField) {
        final field = element.widget as TextField;
        if (field.key == const Key('composer-field-0')) {
          heading = field;
          composerFieldContext = element;
        }
        if (field.key == const Key('composer-field-1')) code = field;
      }
      element.visitChildren(inspectComposer);
    }

    inspectComposer(capture.currentContext! as Element);
    check(
      composer?.controller.text == composerSource &&
          heading?.controller?.text == 'Example' &&
          heading?.style?.fontSize == 24 &&
          code?.controller is CodeSyntaxController &&
          !code!.controller!.text.contains('```'),
      'Native composer renders an editable heading and syntax-colored code card with hidden markers',
      checks,
    );
    final darkComposerContext = composerFieldContext!;
    if (!darkComposerContext.mounted) {
      throw StateError('Composer unmounted before selection check');
    }
    Actions.invoke(
      darkComposerContext,
      const SelectAllTextIntent(SelectionChangedCause.keyboard),
    );
    await screenshot(capture, output, 'composer-selected-dark');
    check(
      heading!.controller!.selection ==
              TextSelection(
                baseOffset: 0,
                extentOffset: heading!.controller!.text.length,
              ) &&
          code!.controller!.selection ==
              TextSelection(
                baseOffset: 0,
                extentOffset: code!.controller!.text.length,
              ) &&
          composer!.controller.text == composerSource,
      'Native Select All highlights heading and code without changing canonical Markdown',
      checks,
    );
    heading!.controller!.selection = const TextSelection.collapsed(offset: 0);
    runApp(
      DoloresApp(chat: chat, captureKey: capture, themeMode: ThemeMode.light),
    );
    await screenshot(capture, output, 'composer-light');
    inspectComposer(capture.currentContext! as Element);
    final lightComposerContext = composerFieldContext!;
    if (!lightComposerContext.mounted) {
      throw StateError('Composer unmounted before light selection check');
    }
    Actions.invoke(
      lightComposerContext,
      const SelectAllTextIntent(SelectionChangedCause.keyboard),
    );
    await screenshot(capture, output, 'composer-selected-light');
    heading!.controller!.selection = const TextSelection.collapsed(offset: 0);
    await chat.send();
    await waitUntil(() => !chat.busy);
    check(
      chat.error == null &&
          chat.messages.length == 4 &&
          chat.messages[2]['content'] == composerSource,
      'The original fenced input is sent and stored without decoration changes',
      checks,
    );
    runApp(
      DoloresApp(chat: chat, captureKey: capture, themeMode: ThemeMode.dark),
    );
    // Capture the normal settings component in the same application's overlay.
    final element = capture.currentContext! as Element;
    BuildContext? pageContext;
    void visit(Element child) {
      if (child.widget is ChatPage) {
        pageContext = child;
      } else {
        child.visitChildren(visit);
      }
    }

    element.visitChildren(visit);
    if (pageContext == null || !pageContext!.mounted) {
      throw StateError('Conversation view unavailable.');
    }
    final smokePageContext = pageContext!;
    final contextPreview = await chat.previewContext();
    check(
      contextPreview?['savedTurns'] == 2 &&
          contextPreview?['includedTurns'] == 2 &&
          contextPreview?['omittedTurns'] == 0,
      'On-demand context preview uses saved latest turns without changing draft or history',
      checks,
    );
    check(
      (contextPreview?['messages'] as List?)?.length == 6 &&
          contextPreview?['messages'][0]['role'] == 'system' &&
          contextPreview?['messages'][3]['content'] == composerSource &&
          contextPreview?['messages'][5]['content'] == chat.draft,
      'Context inspector receives exact system, recent exchanges and draft in request order',
      checks,
    );
    if (!smokePageContext.mounted) {
      throw StateError('Page unavailable before context capture');
    }
    unawaited(showContextPreview(smokePageContext, contextPreview!));
    await screenshot(capture, output, 'context-dark');
    final boxes = <String, Rect>{};
    var helpVisible = false;
    ListTile? systemDisclosure;
    void inspectContext(Element element) {
      final key = element.widget.key;
      for (final name in [
        'composer-frame',
        'chat-model-picker',
        'context-preview',
        'send',
      ]) {
        if (key == Key(name)) {
          final box = element.findRenderObject();
          if (box is RenderBox && box.hasSize) {
            boxes[name] = box.localToGlobal(Offset.zero) & box.size;
          }
        }
      }
      if (element.widget is Text &&
          ((element.widget as Text).data ?? '').contains('Enter to send')) {
        helpVisible = true;
      }
      if (element.widget is ListTile) {
        final tile = element.widget as ListTile;
        if (tile.title is Text &&
            (tile.title as Text).data == 'System instructions') {
          systemDisclosure = tile;
        }
      }
      element.visitChildren(inspectContext);
    }

    inspectContext(capture.currentContext! as Element);
    check(
      !helpVisible &&
          boxes.length == 4 &&
          boxes['composer-frame']!.contains(
            boxes['chat-model-picker']!.center,
          ) &&
          boxes['composer-frame']!.contains(boxes['context-preview']!.center) &&
          boxes['context-preview']!.left >= boxes['chat-model-picker']!.right &&
          boxes['send']!.left >= boxes['context-preview']!.right,
      'Model picker, context ring and Send appear in order inside the composer with no bottom help label',
      checks,
    );
    systemDisclosure?.onTap?.call();
    await screenshot(capture, output, 'context-expanded-dark');
    var exactSystemVisible = false;
    void inspectSystemText(Element element) {
      if (element.widget is SelectableText &&
          (element.widget as SelectableText).data ==
              contextPreview['messages'][0]['content']) {
        exactSystemVisible = true;
      }
      element.visitChildren(inspectSystemText);
    }

    inspectSystemText(capture.currentContext! as Element);
    check(
      exactSystemVisible,
      'Context disclosure renders the exact selectable system instruction',
      checks,
    );
    if (!smokePageContext.mounted) {
      throw StateError('Page unavailable after context capture');
    }
    Navigator.of(smokePageContext).pop();
    await Future<void>.delayed(const Duration(milliseconds: 250));
    if (!smokePageContext.mounted) {
      throw StateError('Page unavailable before trajectory capture');
    }
    unawaited(showTrajectory(smokePageContext, chat));
    await screenshot(capture, output, 'trajectory-dark');
    SegmentedButton<int>? trajectoryTabs;
    var savedReplyVisible = false;
    void inspectTrajectory(Element element) {
      if (element.widget is SegmentedButton<int>) {
        trajectoryTabs = element.widget as SegmentedButton<int>;
      }
      if (element.widget is Text &&
          ((element.widget as Text).data ?? '').contains(
            'dolores-fast · Saved',
          )) {
        savedReplyVisible = true;
      }
      element.visitChildren(inspectTrajectory);
    }

    inspectTrajectory(capture.currentContext! as Element);
    check(
      savedReplyVisible && chat.messages.length == 4,
      'Independent trajectory page displays saved reply provenance without changing the main transcript',
      checks,
    );
    trajectoryTabs!.onSelectionChanged!({1});
    await screenshot(capture, output, 'trajectory-log-dark');
    var savedLogVisible = false;
    void inspectLog(Element element) {
      if (element.widget is Text &&
          (element.widget as Text).data == 'Reply saved') {
        savedLogVisible = true;
      }
      element.visitChildren(inspectLog);
    }

    inspectLog(capture.currentContext! as Element);
    check(
      savedLogVisible && chat.requestLogs.length <= 200,
      'Trajectory log displays coalesced real request lifecycle with bounded retention',
      checks,
    );
    if (!smokePageContext.mounted) {
      throw StateError('Page unavailable after trajectory capture');
    }
    Navigator.of(smokePageContext).pop();
    await Future<void>.delayed(const Duration(milliseconds: 250));
    TextButton? usageAction;
    var usageActions = 0;
    void inspectUsage(Element element) {
      if (element.widget is UsageDetails) {
        usageActions++;
        void findAction(Element child) {
          if (child.widget is TextButton) {
            usageAction = child.widget as TextButton;
          }
          child.visitChildren(findAction);
        }

        element.visitChildren(findAction);
      } else {
        element.visitChildren(inspectUsage);
      }
    }

    inspectUsage(capture.currentContext! as Element);
    check(
      usageActions == 2 &&
          usageAction?.onPressed != null &&
          chat.messages[1]['metadata']['model'] == 'dolores-fast' &&
          chat.messages[3]['metadata']['model'] == 'dolores-mock',
      'Native usage actions retain each reply model after model switching',
      checks,
    );
    usageAction!.onPressed!();
    await screenshot(capture, output, 'usage-details-dark');
    if (!smokePageContext.mounted) {
      throw StateError('Page unavailable after usage capture');
    }
    Navigator.of(smokePageContext).pop();
    await Future<void>.delayed(const Duration(milliseconds: 250));
    chat.newChat(kind: 'side');
    chat.draft = 'no-usage';
    await chat.send();
    await waitUntil(() => !chat.busy);
    check(
      chat.error == null && chat.messages.last['metadata']['usage'] == null,
      'A provider omitting usage still saves the complete turn with unavailable accounting',
      checks,
    );
    await screenshot(capture, output, 'usage-unavailable-dark');
    final earlierSettings = Map<String, dynamic>.of(
      chat.messages.last['metadata']['requestSettings'],
    );
    await chat.saveRequestSettings({
      'maxOutputTokens': 4096,
      'timeoutSeconds': 1,
    });
    await chat.refresh();
    check(
      chat.requestSettings['maxOutputTokens'] == 4096 &&
          chat
                  .messages
                  .last['metadata']['requestSettings']['maxOutputTokens'] ==
              earlierSettings['maxOutputTokens'],
      'Saved request controls reload while earlier reply details keep their original limits',
      checks,
    );
    chat.draft = 'slow';
    await chat.send();
    await waitUntil(() => !chat.busy);
    check(
      chat.messages.length == 2 &&
          chat.draft == 'slow' &&
          chat.partial.isEmpty &&
          chat.activeRecovery?['kind'] == 'timeout' &&
          chat.activeRecovery?['retryable'] == true,
      'Whole-response timeout restores the draft and keeps the partial reply out of history',
      checks,
    );
    await screenshot(capture, output, 'timeout-recovery-dark');
    final attempts = chat.requestLogs
        .where((entry) => entry.label == 'Request submitted')
        .length;
    await Future<void>.delayed(const Duration(milliseconds: 200));
    check(
      attempts > 0 &&
          chat.requestLogs
                  .where((entry) => entry.label == 'Request submitted')
                  .length ==
              attempts,
      'Failure never starts an automatic retry',
      checks,
    );
    await chat.saveRequestSettings({
      'maxOutputTokens': 4096,
      'timeoutSeconds': 8,
    });
    if (!smokePageContext.mounted) {
      throw StateError('Page unavailable before request settings capture');
    }
    unawaited(
      showDialog<void>(
        context: smokePageContext,
        builder: (_) => RequestSettingsDialog(chat: chat),
      ),
    );
    await screenshot(capture, output, 'request-settings-dark');
    if (!smokePageContext.mounted) {
      throw StateError('Page unavailable after request settings capture');
    }
    Navigator.of(smokePageContext).pop();
    await Future<void>.delayed(const Duration(milliseconds: 250));
    chat.draft = 'limit-check';
    // Notify the normal view without changing the restored failure state.
    await chat.saveRequestSettings({
      'maxOutputTokens': 4096,
      'timeoutSeconds': 8,
    });
    await Future<void>.delayed(const Duration(milliseconds: 250));
    TextButton? retryAction;
    void inspectRecovery(Element element) {
      if (element.widget.key == const Key('retry-message') &&
          element.widget is TextButton) {
        retryAction = element.widget as TextButton;
      }
      element.visitChildren(inspectRecovery);
    }

    inspectRecovery(capture.currentContext! as Element);
    check(
      retryAction?.onPressed != null,
      'Native failure card exposes an explicit Retry message action',
      checks,
    );
    retryAction!.onPressed!();
    await waitUntil(() => !chat.busy);
    check(
      chat.error == null &&
          chat.messages.length == 4 &&
          chat.messages[2]['content'] == 'limit-check' &&
          chat
                  .messages
                  .last['metadata']['requestSettings']['maxOutputTokens'] ==
              4096 &&
          chat.messages.last['metadata']['requestSettings']['timeoutSeconds'] ==
              8,
      'Explicit retry sends the edited draft and actual max_tokens, saving the new per-request settings snapshot',
      checks,
    );
    if (!smokePageContext.mounted) {
      throw StateError('Page unavailable before settings capture');
    }
    final workspace = Directory(path.join(output.path, 'approved-folder'));
    await workspace.create();
    await File(path.join(workspace.path, 'readme.txt'))
        .writeAsString('Hello from an approved workspace file. 世界.');
    await chat.chooseToolFolder(() async => workspace.path);
    chat.newChat();
    chat.draft = 'tool-read';
    await chat.send();
    await waitUntil(() => chat.toolApproval != null || !chat.busy);
    check(
      chat.busy &&
          chat.toolApproval?['target'] == 'readme.txt' &&
          chat.toolRecords.isEmpty,
      'Real function-call response waits for a user decision before reading file contents',
      checks,
    );
    await screenshot(capture, output, 'tool-approval-dark');
    FilledButton? allowTool;
    void inspectApproval(Element element) {
      if (element.widget.key == const Key('allow-tool') &&
          element.widget is FilledButton) {
        allowTool = element.widget as FilledButton;
      }
      element.visitChildren(inspectApproval);
    }

    inspectApproval(capture.currentContext! as Element);
    check(
      allowTool?.onPressed != null,
      'Native Allow once control is enabled for the current request',
      checks,
    );
    allowTool!.onPressed!();
    await waitUntil(() => !chat.busy);
    check(
      chat.error == null &&
          chat.messages.length == 2 &&
          chat.messages.last['content'].contains('approved workspace file') &&
          chat.messages.last['metadata']['agent']['tools'][0]['status'] ==
              'read' &&
          chat.messages.last['metadata']['agent']['modelCalls'] == 2,
      'Approved file read reaches the real model as a tool message and persists a complete reply with tool provenance',
      checks,
    );
    final toolSession = chat.session!;
    await screenshot(capture, output, 'tool-result-dark');
    chat.newChat();
    await chat.select(toolSession);
    check(
      chat.messages.last['metadata']['agent']['tools'][0]['content'].contains(
        '世界',
      ),
      'Reload preserves the bounded UTF-8 tool record',
      checks,
    );
    chat.draft = 'tool-deny';
    await chat.send();
    await waitUntil(() => chat.toolApproval != null || !chat.busy);
    await chat.decideTool(false);
    await waitUntil(() => !chat.busy);
    check(
      chat.error == null &&
          chat.messages.length == 4 &&
          chat.messages.last['metadata']['agent']['tools'][0]['status'] ==
              'denied' &&
          !chat.messages.last['content'].contains('approved workspace file'),
      'Deny continues using a refusal tool result without reading or sharing the file',
      checks,
    );
    chat.draft = 'tool-stop';
    await chat.send();
    await waitUntil(() => chat.toolApproval != null || !chat.busy);
    await chat.stop();
    await waitUntil(() => !chat.busy);
    check(
      chat.messages.length == 4 &&
          chat.draft == 'tool-stop' &&
          chat.toolApproval == null,
      'Stop while awaiting approval restores the draft without saving a turn',
      checks,
    );
    chat.newChat();
    chat.draft = 'tool-escape';
    await chat.send();
    await waitUntil(() => !chat.busy);
    check(
      chat.error == null &&
          chat.messages.last['metadata']['agent']['tools'][0]['status'] ==
              'blocked',
      'Outside-folder request is blocked without an approval or file read',
      checks,
    );
    chat.newChat();
    chat.draft = 'tool-loop';
    await chat.send();
    while (chat.busy) {
      if (chat.toolApproval != null) await chat.decideTool(false);
      await Future<void>.delayed(const Duration(milliseconds: 25));
    }
    check(
      chat.error?.contains('limit') == true &&
          chat.draft == 'tool-loop' &&
          chat.messages.isEmpty,
      'Repeated tool requests stop at the fixed model-call budget and never save a partial turn',
      checks,
    );
    await Directory(path.join(workspace.path, 'docs')).create();
    await File(path.join(workspace.path, 'docs', 'notes.txt')).writeAsString(
      'Find 世界.* here\nFull approved note, including its second line.',
    );
    chat.newChat();
    chat.draft = 'tool-discovery';
    await chat.send();
    await waitUntil(() => chat.toolApproval != null || !chat.busy);
    check(
      chat.toolApproval?['name'] == 'list_folder' && chat.toolRecords.isEmpty,
      'Folder listing waits for approval before sharing names',
      checks,
    );
    await screenshot(capture, output, 'listing-approval-dark');
    await chat.decideTool(true);
    await waitUntil(
      () => chat.toolApproval?['name'] == 'search_text' || !chat.busy,
    );
    check(
      chat.busy &&
          chat.toolApproval?['target'] == 'docs' &&
          chat.toolApproval?['query'] == '世界.*' &&
          chat.toolRecords.length == 1,
      'Search approval names its folder and exact literal query before scanning',
      checks,
    );
    await screenshot(capture, output, 'search-approval-dark');
    await chat.decideTool(true);
    await waitUntil(
      () => chat.toolApproval?['name'] == 'read_text_file' || !chat.busy,
    );
    check(
      chat.busy &&
          chat.toolApproval?['target'] == 'docs/notes.txt' &&
          chat.toolRecords.length == 2,
      'A search hit requests separate full-file approval with a root-relative path',
      checks,
    );
    await chat.decideTool(true);
    await waitUntil(() => !chat.busy);
    check(
      chat.error == null &&
          chat.messages.length == 2 &&
          chat.messages.last['content'].contains('second line') &&
          chat.messages.last['metadata']['agent']['modelCalls'] == 4 &&
          chat.messages.last['metadata']['agent']['tools'].length == 3,
      'Approved listing search and read complete through four real model calls with saved provenance',
      checks,
    );
    final discoverySession = chat.session!;
    await screenshot(capture, output, 'discovery-result-dark');
    chat.newChat();
    await chat.select(discoverySession);
    final savedSearch = chat.messages.last['metadata']['agent']['tools'][1];
    check(
      savedSearch['query'] == '世界.*' &&
          jsonDecode(savedSearch['content'])['matches'][0]['path'] ==
              'docs/notes.txt',
      'Reload preserves search query and Unicode matching-line provenance',
      checks,
    );
    chat.draft = 'tool-search-deny';
    await chat.send();
    await waitUntil(() => chat.toolApproval != null || !chat.busy);
    await chat.decideTool(true);
    await waitUntil(
      () => chat.toolApproval?['name'] == 'search_text' || !chat.busy,
    );
    await chat.decideTool(false);
    await waitUntil(() => !chat.busy);
    check(
      chat.error == null &&
          chat.messages.length == 4 &&
          chat.messages.last['metadata']['agent']['tools'][1]['status'] ==
              'denied' &&
          !chat.messages.last['content'].contains('second line'),
      'Deny search sends a refusal without scanning text or granting a full read',
      checks,
    );
    chat.draft = 'tool-search-stop';
    await chat.send();
    await waitUntil(() => chat.toolApproval != null || !chat.busy);
    await chat.decideTool(true);
    await waitUntil(
      () => chat.toolApproval?['name'] == 'search_text' || !chat.busy,
    );
    await chat.stop();
    await waitUntil(() => !chat.busy);
    check(
      chat.messages.length == 4 &&
          chat.draft == 'tool-search-stop' &&
          chat.toolApproval == null,
      'Stop before a folder search restores the draft and preserves saved history',
      checks,
    );
    final projectSession = chat.session!;
    final projectRoot = chat.workspaceRoot!;
    chat.newChat(kind: 'temporary');
    chat.draft = 'tool-list-deny';
    await chat.send();
    await waitUntil(() => chat.toolApproval != null || !chat.busy);
    final temporarySession = chat.session!;
    final temporaryRoot = chat.workspaceRoot!;
    check(
      chat.workspaceKind == 'temporary' &&
          temporaryRoot != projectRoot &&
          Directory(temporaryRoot).existsSync() &&
          chat.toolApproval?['name'] == 'list_folder',
      'Default temporary workspace advertises all tools and waits for listing approval without a folder toggle',
      checks,
    );
    await screenshot(capture, output, 'temporary-approval-dark');
    await chat.decideTool(false);
    await waitUntil(() => !chat.busy);
    chat.newChat(kind: 'temporary');
    chat.draft = 'workspace-hello';
    await chat.send();
    await waitUntil(() => !chat.busy);
    check(
      chat.error == null &&
          chat.workspaceRoot != temporaryRoot &&
          chat.messages.last['metadata']['agent']['modelCalls'] == 1,
      'A second temporary chat owns a different folder and ordinary prompts use the tool-enabled agent',
      checks,
    );
    await chat.select(temporarySession);
    check(
      chat.workspaceRoot == temporaryRoot && chat.workspaceKind == 'temporary',
      'Reload restores the exact temporary working folder',
      checks,
    );
    await chat.select(projectSession);
    check(
      chat.workspaceRoot == projectRoot &&
          chat.workspaceKind == 'project' &&
          chat.projects.any((p) => p['root'] == projectRoot),
      'Project selection restores its persisted root and recent project entry',
      checks,
    );
    await screenshot(capture, output, 'project-workspace-dark');
    ScaffoldState? sidebarShell;
    void findShell(Element element) {
      if (element is StatefulElement && element.state is ScaffoldState) {
        sidebarShell = element.state as ScaffoldState;
      }
      element.visitChildren(findShell);
    }

    findShell(capture.currentContext! as Element);
    if (sidebarShell?.hasDrawer == true) {
      sidebarShell!.openDrawer();
      await Future<void>.delayed(const Duration(milliseconds: 350));
    }
    void toggleSection(String key) {
      void findButton(Element element) {
        if (element.widget is TextButton && element.widget.key == Key(key)) {
          (element.widget as TextButton).onPressed?.call();
        }
        element.visitChildren(findButton);
      }

      findButton(capture.currentContext! as Element);
    }

    toggleSection('section-projects');
    await Future<void>.delayed(const Duration(milliseconds: 100));
    toggleSection('section-recents');
    await screenshot(capture, output, 'sidebar-collapsed-dark');
    toggleSection('section-projects');
    toggleSection('section-recents');
    sidebarShell?.closeDrawer();
    chat.newChat(kind: 'side');
    await screenshot(capture, output, 'side-chat-dark');
    if (!smokePageContext.mounted) {
      throw StateError('Smoke page was closed.');
    }
    unawaited(
      showDialog<void>(
        context: smokePageContext,
        builder: (_) => ConnectionDialog(chat: chat),
      ),
    );
    await screenshot(capture, output, 'settings-dark');
    await File(path.join(output.path, 'report.json')).writeAsString(
      jsonEncode({
        'ok': true,
        'checks': checks,
        'messages': chat.messages.length,
        'logicalWidth':
            (capture.currentContext!.findRenderObject()
                    as RenderRepaintBoundary)
                .size
                .width,
        'logicalHeight':
            (capture.currentContext!.findRenderObject()
                    as RenderRepaintBoundary)
                .size
                .height,
        'scope': 'Production controller, FFI, HTTP fixture, SQLite and rendered widgets. Native pointer/IME interaction is not exercised.',
      }),
    );
  } catch (error) {
    await File(path.join(output.path, 'report.json')).writeAsString(
      jsonEncode({'ok': false, 'checks': checks, 'error': error.toString()}),
    );
  }
}
