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
import 'language_host.dart';
import 'language_setup.dart';

import 'package:path/path.dart' as paths;

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
  final chunks = NonCodeChunkAnalyzer();
  final highlights = <String, CodeHighlightTheme?>{};
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

  bool languageBusy = false;
  Future<void> languageFeature(String feature) async {
    if (languageBusy || d.readonly) return;
    setState(() => languageBusy = true);
    try {
      final position = {
        'line': view.selection.extentIndex,
        'character': view.selection.extentOffset,
      };
      var name = '';
      if (feature == 'rename') {
        final entered = await filePathDialog(
          context,
          'Rename symbol',
          label: 'New name',
        );
        if (entered == null) return;
        name = entered;
      }
      final result = await widget.host.languages.feature(
        files,
        w,
        d,
        feature,
        position,
        name: name,
      );
      final version = d.version, text = d.text;
      if (!mounted) return;
      if (feature == 'formatting' || feature == 'rename') {
        final root = w.root.replaceFirst(r'\\?\', '');
        final edit = feature == 'formatting'
            ? {
                'changes': {
                  Uri.file(paths.join(root, d.path), windows: true).toString():
                      result,
                },
              }
            : result;
        await reviewLanguageEdit(edit);
        return;
      }
      if (feature == 'completion') {
        final items =
            (result is List ? result : (result as Map?)?['items'] ?? [])
                as List;
        final picked = await showDialog<Map>(
          context: context,
          builder: (context) => AlertDialog(
            title: const Text('Completions'),
            content: SizedBox(
              width: 500,
              height: 350,
              child: ListView(
                children: [
                  for (final item in items.take(100))
                    ListTile(
                      title: Text(item['label']?.toString() ?? ''),
                      subtitle: item['detail'] == null
                          ? null
                          : Text(item['detail'].toString()),
                      onTap: () => Navigator.pop(context, item),
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
        if (picked == null) return;
        if (d.version != version || d.text != text || files.selected != w) {
          throw StateError(
            'Document changed while choosing completion. Request it again.',
          );
        }
        if (picked['insertTextFormat'] == 2 ||
            picked['additionalTextEdits'] != null) {
          throw StateError(
            'This completion needs snippet or additional-edit support. Choose a plain completion.',
          );
        }
        final edit = picked['textEdit'] as Map?;
        final range =
            edit?['range'] ??
            edit?['replace'] ??
            {'start': position, 'end': position};
        final next = LanguageHost.apply(text, [
          {
            'range': range,
            'newText':
                edit?['newText'] ?? picked['insertText'] ?? picked['label'],
          },
        ]);
        final seed = CodeLineEditingController.fromText(next);
        view.value = seed.value;
        seed.dispose();
        return;
      }
      if (feature == 'definition' || feature == 'references') {
        final locations = (result is List
            ? result
            : result == null
            ? []
            : [result]);
        final picked = await showDialog<Map>(
          context: context,
          builder: (context) => AlertDialog(
            title: Text(feature == 'definition' ? 'Definitions' : 'References'),
            content: SizedBox(
              width: 600,
              height: 350,
              child: ListView(
                children: [
                  for (final item in locations.take(200))
                    ListTile(
                      title: Text(
                        (item['uri'] ?? item['targetUri'])?.toString() ??
                            'Unavailable location',
                      ),
                      onTap: () => Navigator.pop(context, item),
                    ),
                ],
              ),
            ),
            actions: [
              TextButton(
                onPressed: () => Navigator.pop(context),
                child: const Text('Close'),
              ),
            ],
          ),
        );
        if (picked == null) return;
        if (files.selected != w || d.version != version || d.text != text) {
          throw StateError(
            'Document or project changed. Request locations again.',
          );
        }
        final uri = Uri.parse(picked['uri'] ?? picked['targetUri']);
        if (uri.scheme != 'file') {
          throw StateError(
            'Only files inside the selected project can be opened.',
          );
        }
        final full = paths.normalize(uri.toFilePath(windows: true));
        final root = paths.normalize(w.root.replaceFirst(r'\\?\', ''));
        if (!paths.isWithin(root, full)) {
          throw StateError('Location is outside the selected project.');
        }
        final target = await files.open(
          w,
          paths.relative(full, from: root).replaceAll('\\', '/'),
        );
        final range =
            picked['range'] ??
            picked['targetSelectionRange'] ??
            picked['targetRange'];
        if (target != null && range is Map) {
          w.layoutOwner
              .memory(w.layoutOwner.activeGroup, target.id)
              .selection = CodeLineSelection.collapsed(
            index: range['start']['line'],
            offset: range['start']['character'],
          );
          files.changed();
        }
        return;
      }
      if (feature == 'diagnostics') {
        final items = (result is Map ? result['items'] : null) as List? ?? [];
        final picked = await showDialog<Map>(
          context: context,
          builder: (context) => AlertDialog(
            title: const Text('Diagnostics'),
            content: SizedBox(
              width: 600,
              height: 350,
              child: items.isEmpty
                  ? const Center(child: Text('No current diagnostics.'))
                  : ListView(
                      children: [
                        for (final item in items)
                          ListTile(
                            title: Text(item['message']?.toString() ?? ''),
                            subtitle: Text(
                              'Line ${(item['range']['start']['line'] as int) + 1}',
                            ),
                            onTap: () => Navigator.pop(context, item),
                          ),
                      ],
                    ),
            ),
            actions: [
              TextButton(
                onPressed: () => Navigator.pop(context),
                child: const Text('Close'),
              ),
            ],
          ),
        );
        if (picked != null &&
            d.version == version &&
            d.text == text &&
            files.selected == w) {
          view.selection = CodeLineSelection.collapsed(
            index: picked['range']['start']['line'],
            offset: picked['range']['start']['character'],
          );
          scroll.makeVisible(
            CodeLinePosition(
              index: view.selection.extentIndex,
              offset: view.selection.extentOffset,
            ),
          );
          focus.requestFocus();
        }
        return;
      }
      final content = feature == 'hover'
          ? (result is Map ? result['contents'] : result)
          : result;
      String plain(dynamic v) => v is String
          ? v
          : v is List
          ? v.map(plain).join('\n\n')
          : v is Map && v['value'] is String
          ? v['value']
          : v?.toString() ?? 'No result.';
      await showDialog<void>(
        context: context,
        builder: (context) => AlertDialog(
          title: Text(feature == 'hover' ? 'Hover' : 'Diagnostics'),
          content: SizedBox(
            width: 600,
            height: 350,
            child: SingleChildScrollView(child: SelectableText(plain(content))),
          ),
          actions: [
            TextButton(
              onPressed: () => Navigator.pop(context),
              child: const Text('Close'),
            ),
          ],
        ),
      );
    } catch (e) {
      d.error = '$e';
      d.changed();
    } finally {
      if (mounted) setState(() => languageBusy = false);
    }
  }

  Future<void> flushLanguageTargets() async {
    for (final doc
        in files.documents.values
            .where((doc) => doc.project == w.project)
            .toList()) {
      await files.flush(doc);
      if (doc.blocked || doc.text != doc.acknowledged) {
        throw StateError(
          'Retry sync for ${doc.path} before changing language edits.',
        );
      }
    }
  }

  Future<void> reviewLanguageEdit(dynamic edit) async {
    await flushLanguageTargets();
    final preview = await widget.host.languages.edits(w, {
      'action': 'preview',
      'document': d.id,
      'version': d.version,
      'edit': edit,
    });
    if (!mounted) return;
    final entries = preview['files'] as List;
    final yes = await showDialog<bool>(
      context: context,
      builder: (context) => AlertDialog(
        title: const Text('Review language changes'),
        content: SizedBox(
          width: 880,
          height: 520,
          child: DefaultTabController(
            length: entries.length,
            child: Column(
              children: [
                Text(preview['note']),
                TabBar(
                  isScrollable: true,
                  tabs: [for (final file in entries) Tab(text: file['path'])],
                ),
                Expanded(
                  child: TabBarView(
                    children: [
                      for (final file in entries)
                        Row(
                          children: [
                            for (final side in ['before', 'after'])
                              Expanded(
                                child: Column(
                                  children: [
                                    Text(side == 'before' ? 'Before' : 'After'),
                                    Expanded(
                                      child: SingleChildScrollView(
                                        padding: const EdgeInsets.all(12),
                                        child: SelectableText(
                                          file[side],
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
                    ],
                  ),
                ),
              ],
            ),
          ),
        ),
        actions: [
          TextButton(
            onPressed: () => Navigator.pop(context, false),
            child: const Text('Cancel'),
          ),
          FilledButton(
            onPressed: () => Navigator.pop(context, true),
            child: const Text('Apply to buffers'),
          ),
        ],
      ),
    );
    if (yes != true) return;
    if (files.selected != w) {
      throw StateError('Project changed. Request a new language preview.');
    }
    await flushLanguageTargets();
    final result = await widget.host.languages.edits(w, {
      'action': 'apply',
      'token': preview['token'],
    });
    files.acceptLanguage(w, result['documents']);
    widget.host.languages.undoTokens[w.project] = result['token'];
  }

  Future<void> undoLanguageEdit() async {
    try {
      final token = widget.host.languages.undoTokens[w.project];
      if (token == null) {
        throw StateError('No language edit is available to undo.');
      }
      await flushLanguageTargets();
      final result = await widget.host.languages.edits(w, {
        'action': 'undo',
        'token': token,
      });
      files.acceptLanguage(w, result['documents']);
      widget.host.languages.undoTokens.remove(w.project);
    } catch (e) {
      d.error = '$e';
      d.changed();
    }
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
    // Each view's highlighter copies a complete source into its worker. Bound
    // this optional work independently of the document's editable byte limit.
    if (d.textBytes > 64 * 1024) return null;
    final extension = d.path.split('.').last.toLowerCase();
    final key = '$extension:$dark';
    if (highlights.containsKey(key)) return highlights[key];
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
    return highlights[key] = mode == null
        ? null
        : CodeHighlightTheme(
            languages: {
              extension: CodeHighlightThemeMode(
                mode: mode,
                maxSize: 64 * 1024,
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
  ) => c.value == null
      ? const PreferredSize(preferredSize: Size.zero, child: SizedBox.shrink())
      : PreferredSize(
          preferredSize: Size.fromHeight(
            c.value?.replaceMode == true ? 104 : 56,
          ),
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
                            decoration: const InputDecoration(
                              hintText: 'Replace',
                            ),
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
          const SingleActivator(LogicalKeyboardKey.space, control: true): () =>
              unawaited(languageFeature('completion')),
          const SingleActivator(LogicalKeyboardKey.f12): () =>
              unawaited(languageFeature('definition')),
          const SingleActivator(LogicalKeyboardKey.f12, shift: true): () =>
              unawaited(languageFeature('references')),
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
                chunkAnalyzer: chunks,
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
                      '${d.dirty ? 'Unsaved · ' : ''}UTF-8${d.snapshot['bom'] == true ? ' BOM' : ''} · ${d.snapshot['newline'].toString().toUpperCase()} · Autosave Off${d.textBytes > 64 * 1024 ? ' · Plain text above 64 KiB' : ''}${d.buffer.historyLimited ? ' · Undo history limit reached' : ''}',
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
                        case 'diagnostics':
                        case 'completion':
                        case 'hover':
                        case 'definition':
                        case 'references':
                        case 'formatting':
                        case 'rename':
                          unawaited(languageFeature(value));
                        case 'undoLanguage':
                          unawaited(undoLanguageEdit());
                        case 'installLanguage':
                          unawaited(
                            showLanguageSetup(context, widget.host.languages),
                          );
                        case 'stopLanguage':
                          unawaited(widget.host.languages.stop());
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
                        value: 'formatting',
                        child: Text('Format document…'),
                      ),
                      PopupMenuItem(
                        value: 'rename',
                        child: Text('Rename symbol…'),
                      ),
                      PopupMenuItem(
                        value: 'undoLanguage',
                        child: Text('Undo language edit'),
                      ),
                      PopupMenuItem(
                        value: 'installLanguage',
                        child: Text('Install language tools…'),
                      ),
                      PopupMenuItem(
                        value: 'diagnostics',
                        child: Text('Diagnostics'),
                      ),
                      PopupMenuItem(
                        value: 'completion',
                        child: Text('Complete (Ctrl+Space)'),
                      ),
                      PopupMenuItem(
                        value: 'hover',
                        child: Text('Hover information'),
                      ),
                      PopupMenuItem(
                        value: 'definition',
                        child: Text('Go to definition (F12)'),
                      ),
                      PopupMenuItem(
                        value: 'references',
                        child: Text('Find references (Shift+F12)'),
                      ),
                      PopupMenuItem(
                        value: 'stopLanguage',
                        child: Text('Stop language services'),
                      ),
                      PopupMenuDivider(),
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
