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
    chat.draft = 'hello';
    await chat.send();
    await waitUntil(() => !chat.busy);
    check(
      chat.messages.length == 2 &&
          chat.messages.last['content'].contains('你好！'),
      'Unicode stream saved as a complete turn',
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
    chat.newChat();
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
    unawaited(
      showDialog<void>(
        context: pageContext!,
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
