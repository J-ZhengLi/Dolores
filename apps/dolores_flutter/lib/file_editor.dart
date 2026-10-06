import 'dart:async';
import 'dart:math' as math;

import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:re_editor/re_editor.dart';
import 'package:re_highlight/languages/javascript.dart';
import 'package:re_highlight/languages/typescript.dart';
import 'package:re_highlight/languages/rust.dart';
import 'package:re_highlight/languages/dart.dart';
import 'package:re_highlight/languages/json.dart';
import 'package:re_highlight/languages/python.dart';
import 'package:re_highlight/languages/markdown.dart';
import 'package:re_highlight/styles/vs.dart';
import 'package:re_highlight/styles/vs2015.dart';

import 'app_host.dart';
import 'document_buffer.dart';
import 'file_host.dart';
import 'folders.dart' show filePathDialog;
import 'theme.dart';
import 'file_layout.dart';

class FileEditor extends StatefulWidget {
  final AppHost host;
  final FileWorkspace workspace;
  final FileDocument document;
  final FileViewMemory? memory;
  final VoidCallback? onFocus;
  const FileEditor({
    super.key,
    required this.host,
    required this.workspace,
    required this.document,
    this.memory,
    this.onFocus,
  });
  @override
  State<FileEditor> createState() => _FileEditorState();
}

class _FileEditorState extends State<FileEditor> {
  late final memory = widget.memory ?? FileViewMemory();
  late final DocumentView view = createView();
  DocumentView createView() {
    final v = widget.document.buffer.createView();
    v.selection = widget.document.buffer.clamp(memory.selection, v.codeLines);
    return v;
  }

  late final find = CodeFindController(view);
  final focus = FocusNode();
  late final scroll = CodeScrollController(
    verticalScroller: ScrollController(initialScrollOffset: memory.vertical),
    horizontalScroller: ScrollController(
      initialScrollOffset: memory.horizontal,
    ),
  );
  FileDocument get d => widget.document;
  FileHost get files => widget.host.files;
  FileWorkspace get w => widget.workspace;
  @override
  void initState() {
    super.initState();
    focus.addListener(focused);
    view.addListener(remember);
    scroll.verticalScroller.addListener(remember);
    scroll.horizontalScroller.addListener(remember);
  }

  void focused() {
    if (focus.hasFocus) widget.onFocus?.call();
  }

  void remember() {
    memory.selection = view.selection;
    if (scroll.verticalScroller.hasClients) {
      memory.vertical = scroll.verticalScroller.offset;
    }
    if (scroll.horizontalScroller.hasClients) {
      memory.horizontal = scroll.horizontalScroller.offset;
    }
    files.scheduleLayout(w);
  }

  @override
  void dispose() {
    remember();
    view.removeListener(remember);
    scroll.verticalScroller.removeListener(remember);
    scroll.horizontalScroller.removeListener(remember);
    find.dispose();
    view.dispose();
    focus.dispose();
    scroll.verticalScroller.dispose();
    scroll.horizontalScroller.dispose();
    scroll.dispose();
    super.dispose();
  }

  Future<void> save() async {
    await files.save(w, d);
    if (mounted) focus.requestFocus();
  }

  Future<void> saveAs() async {
    final path = await filePathDialog(context, 'Save as', initial: d.path);
    if (path != null) await files.action(w, d, 'saveAs', path: path);
  }

  Future<void> compare() async {
    final comparison = await files.compare(w, d);
    if (comparison == null || !mounted) return;
    final disk = comparison['disk'] as Map;
    final version = d.version;
    final editorText = d.text;
    final result = await showDialog<String>(
      context: context,
      builder: (context) => AlertDialog(
        title: Text('Compare ${d.path}'),
        content: SizedBox(
          width: 880,
          height: 480,
          child: Column(
            crossAxisAlignment: CrossAxisAlignment.start,
            children: [
              const Text(
                'Inspect disk and editor. Keep my edits adopts this reviewed disk revision without saving.',
              ),
              const SizedBox(height: 12),
              Expanded(
                child: Row(
                  children: [
                    for (final entry in [
                      ('Disk', disk['text'] as String),
                      ('Editor', editorText),
                    ])
                      Expanded(
                        child: Column(
                          children: [
                            Text(entry.$1),
                            Expanded(
                              child: SingleChildScrollView(
                                padding: const EdgeInsets.all(8),
                                child: SelectableText(
                                  entry.$2,
                                  style: const TextStyle(
                                    fontFamily: UiTokens.codeFont,
                                  ),
                                ),
                              ),
                            ),
                          ],
                        ),
                      ),
                  ],
                ),
              ),
            ],
          ),
        ),
        actions: [
          TextButton(
            onPressed: () => Navigator.pop(context),
            child: const Text('Keep editing'),
          ),
          if (disk['readonly'] != true)
            TextButton(
              onPressed: () => Navigator.pop(context, 'reload'),
              child: const Text('Reload disk'),
            ),
          if (disk['readonly'] != true)
            TextButton(
              onPressed: () => Navigator.pop(context, 'rebase'),
              child: const Text('Keep my edits'),
            ),
        ],
      ),
    );
    if (result != null && mounted) {
      if (d.version != version || d.text != editorText) {
        d.error = 'Editor changed during comparison. Compare again.';
        d.changed();
        return;
      }
      await files.reconcile(w, d, result, disk['revision'] as String);
    }
  }

  Future<void> line() async {
    final value = await filePathDialog(
      context,
      'Go to line',
      initial: '1',
      label: 'Line number',
    );
    if (value == null || !mounted) return;
    final target = int.tryParse(value);
    if (target == null || target < 1 || target > view.lineCount) {
      d.error = 'Choose a line between 1 and ${view.lineCount}.';
      d.changed();
      return;
    }
    view.selection = CodeLineSelection.collapsed(index: target - 1, offset: 0);
    scroll.makeVisible(CodeLinePosition(index: target - 1, offset: 0));
    focus.requestFocus();
  }

  int offset(int index, int column) {
    var n = column;
    for (var i = 0; i < index; i++) {
      n += view.codeLines[i].length + 1;
    }
    return n;
  }

  Future<void> attach() async {
    await files.flush(d);
    if (d.blocked || !mounted) return;
    final s = view.selection;
    final a = offset(s.baseIndex, s.baseOffset),
        b = offset(s.extentIndex, s.extentOffset);
    if (a == b) {
      d.error = 'Select source text before attaching it to a conversation.';
      d.changed();
      return;
    }
    final targets = widget.host.owners
        .where(
          (c) =>
              c.session != null &&
              c.workspaceRoot == w.root &&
              !c.busy &&
              !c.changing,
        )
        .toList();
    if (targets.isEmpty) {
      d.error = 'Open an idle project conversation on Home before attaching.';
      d.changed();
      return;
    }
    final version = d.version, text = d.text;
    final chosen = await showDialog<String>(
      context: context,
      builder: (context) => AlertDialog(
        title: const Text('Attach selection to conversation'),
        content: SizedBox(
          width: 500,
          child: Column(
            mainAxisSize: MainAxisSize.min,
            crossAxisAlignment: CrossAxisAlignment.start,
            children: [
              Text(
                '${d.path} · editor v$version · ${d.dirty ? 'unsaved' : 'saved'}\nUTF-16 ${math.min(a, b)}..${math.max(a, b)}\nSending shares this immutable snapshot with the configured provider.',
              ),
              for (final target in targets)
                TextButton(
                  onPressed: () => Navigator.pop(context, target.session),
                  child: Text(
                    target.sessions
                                .where((row) => row['id'] == target.session)
                                .firstOrNull?['title']
                            as String? ??
                        'Project conversation',
                  ),
                ),
            ],
          ),
        ),
        actions: [
          TextButton(
            onPressed: () => Navigator.pop(context),
            child: const Text('Cancel'),
          ),
        ],
      ),
    );
    if (chosen == null || !mounted) return;
    if (d.version != version || d.text != text) {
      d.error = 'Selection changed during review. Attach again.';
      d.changed();
      return;
    }
    try {
      final parts = await files.call(w, {
        'action': 'attach',
        'document': d.id,
        'version': version,
        'start': math.min(a, b),
        'end': math.max(a, b),
        'target': chosen,
      });
      final target = targets.firstWhere((c) => c.session == chosen);
      target.acceptAttachmentParts(
        (parts as List).cast<Map<String, dynamic>>(),
      );
      d.error = null;
      d.changed();
      if (mounted) {
        ScaffoldMessenger.of(context).showSnackBar(
          const SnackBar(
            content: Text('Selection attached to the chosen conversation.'),
          ),
        );
      }
    } catch (e) {
      d.error = '$e';
      d.changed();
    }
  }

  CodeHighlightTheme? highlight(bool dark) {
    final extension = d.path.split('.').last.toLowerCase();
    final mode = switch (extension) {
      'js' || 'jsx' => langJavascript,
      'ts' || 'tsx' => langTypescript,
      'rs' => langRust,
      'dart' => langDart,
      'json' => langJson,
      'py' => langPython,
      'md' => langMarkdown,
      _ => null,
    };
    return mode == null
        ? null
        : CodeHighlightTheme(
            languages: {
              extension: CodeHighlightThemeMode(
                mode: mode,
                maxSize: 1024 * 1024,
                maxLineLength: 8192,
              ),
            },
            theme: dark ? vs2015Theme : vsTheme,
          );
  }

  PreferredSizeWidget findBar(
    BuildContext context,
    CodeFindController c,
    bool readonly,
  ) => PreferredSize(
    preferredSize: Size.fromHeight(c.value?.replaceMode == true ? 104 : 56),
    child: Material(
      child: Padding(
        padding: const EdgeInsets.all(8),
        child: Column(
          children: [
            Row(
              children: [
                Expanded(
                  child: TextField(
                    key: const Key('editor-find'),
                    controller: c.findInputController,
                    focusNode: c.findInputFocusNode,
                    decoration: const InputDecoration(hintText: 'Find'),
                    onSubmitted: (_) => c.nextMatch(),
                  ),
                ),
                IconButton(
                  tooltip: 'Previous match',
                  onPressed: c.previousMatch,
                  icon: const Icon(Icons.keyboard_arrow_up),
                ),
                IconButton(
                  tooltip: 'Next match',
                  onPressed: c.nextMatch,
                  icon: const Icon(Icons.keyboard_arrow_down),
                ),
                IconButton(
                  tooltip: 'Close Find',
                  onPressed: c.close,
                  icon: const Icon(Icons.close),
                ),
              ],
            ),
            if (c.value?.replaceMode == true)
              Row(
                children: [
                  Expanded(
                    child: TextField(
                      key: const Key('editor-replace'),
                      controller: c.replaceInputController,
                      focusNode: c.replaceInputFocusNode,
                      decoration: const InputDecoration(hintText: 'Replace'),
                    ),
                  ),
                  TextButton(
                    onPressed: readonly ? null : c.replaceMatch,
                    child: const Text('Replace'),
                  ),
                  TextButton(
                    onPressed: readonly ? null : c.replaceAllMatches,
                    child: const Text('All'),
                  ),
                ],
              ),
          ],
        ),
      ),
    ),
  );
  @override
  Widget build(BuildContext context) => AnimatedBuilder(
    animation: d,
    builder: (context, _) {
      final dark = Theme.of(context).brightness == Brightness.dark;
      return CallbackShortcuts(
        bindings: {
          const SingleActivator(LogicalKeyboardKey.keyS, control: true): () =>
              unawaited(save()),
          const SingleActivator(LogicalKeyboardKey.keyG, control: true): () =>
              unawaited(line()),
          const SingleActivator(LogicalKeyboardKey.keyH, control: true):
              find.replaceMode,
        },
        child: Column(
          children: [
            if (d.error != null)
              Padding(
                padding: const EdgeInsets.all(8),
                child: Wrap(
                  crossAxisAlignment: WrapCrossAlignment.center,
                  children: [
                    Text(d.error!),
                    TextButton(
                      onPressed: () => files.retrySync(d),
                      child: const Text('Retry sync'),
                    ),
                    TextButton(
                      onPressed: compare,
                      child: const Text('Compare'),
                    ),
                    TextButton(onPressed: saveAs, child: const Text('Save as')),
                  ],
                ),
              ),
            if (d.readonly)
              Padding(
                padding: const EdgeInsets.all(8),
                child: Text(
                  '${d.snapshot['reason']} Read-only preview; up to 64 KiB and 2,000 characters per displayed line.',
                ),
              ),
            Expanded(
              child: CodeEditor(
                controller: view,
                scrollController: scroll,
                findController: find,
                findBuilder: findBar,
                focusNode: focus,
                readOnly: d.readonly || d.pending,
                wordWrap: false,
                chunkAnalyzer: NonCodeChunkAnalyzer(),
                indicatorBuilder: (context, controller, chunks, notifier) =>
                    DefaultCodeLineNumber(
                      controller: controller,
                      notifier: notifier,
                    ),
                style: CodeEditorStyle(
                  fontFamily: UiTokens.codeFont,
                  fontSize: 14,
                  textColor: Theme.of(context).colorScheme.onSurface,
                  backgroundColor: Theme.of(context).scaffoldBackgroundColor,
                  codeTheme: highlight(dark),
                ),
              ),
            ),
            SizedBox(
              height: 32,
              child: Row(
                children: [
                  const SizedBox(width: 8),
                  Expanded(
                    child: Text(
                      '${d.dirty ? 'Unsaved · ' : ''}UTF-8${d.snapshot['bom'] == true ? ' BOM' : ''} · ${d.snapshot['newline'].toString().toUpperCase()} · Autosave Off${d.buffer.historyLimited ? ' · Undo history limit reached' : ''}',
                      maxLines: 1,
                      overflow: TextOverflow.ellipsis,
                    ),
                  ),
                  TextButton(
                    onPressed: d.readonly || d.pending ? null : save,
                    child: const Text('Save'),
                  ),
                  PopupMenuButton<String>(
                    tooltip: 'Editor actions',
                    onSelected: (value) {
                      switch (value) {
                        case 'find':
                          find.findMode();
                        case 'replace':
                          find.replaceMode();
                        case 'line':
                          unawaited(line());
                        case 'compare':
                          unawaited(compare());
                        case 'saveAs':
                          unawaited(saveAs());
                        case 'attach':
                          unawaited(attach());
                      }
                    },
                    itemBuilder: (_) => const [
                      PopupMenuItem(
                        value: 'find',
                        child: Text('Find (Ctrl+F)'),
                      ),
                      PopupMenuItem(
                        value: 'replace',
                        child: Text('Replace (Ctrl+H)'),
                      ),
                      PopupMenuItem(
                        value: 'line',
                        child: Text('Go to line (Ctrl+G)'),
                      ),
                      PopupMenuItem(
                        value: 'compare',
                        child: Text('Compare with disk'),
                      ),
                      PopupMenuItem(value: 'saveAs', child: Text('Save as')),
                      PopupMenuItem(
                        value: 'attach',
                        child: Text('Attach selection to conversation'),
                      ),
                    ],
                  ),
                ],
              ),
            ),
          ],
        ),
      );
    },
  );
}
