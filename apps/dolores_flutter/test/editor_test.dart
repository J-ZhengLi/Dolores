import 'dart:async';

import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:re_editor/re_editor.dart';
import 'package:dolores_flutter/app_host.dart';
import 'package:dolores_flutter/chat.dart';
import 'package:dolores_flutter/document_buffer.dart';
import 'package:dolores_flutter/file_editor.dart';

import 'file_host_test.dart' show FileBridge;

class EditingBridge extends FileBridge {
  Completer<void>? delayed;
  bool fail = false, failSave = false;
  @override
  Future<dynamic> call(Map<String, dynamic> cmd) async {
    final r = cmd['request'];
    if (cmd['command'] == 'editor' && r['action'] == 'edit') {
      editorCalls.add(Map<String, dynamic>.from(r as Map));
      if (delayed != null) await delayed!.future;
      if (fail) throw StateError('Document version changed.');
      final d = docs[r['document']]!;
      if (d['version'] != r['version']) throw StateError('Stale version.');
      for (final e in r['edits'] as List) {
        d['text'] = (d['text'] as String).replaceRange(
          e['start'] as int,
          e['end'] as int,
          e['text'] as String,
        );
      }
      d['version']++;
      d['dirty'] = d['text'] != d['snapshot']['text'];
      return {'version': d['version'], 'dirty': d['dirty']};
    }
    if (cmd['command'] == 'editor' && r['action'] == 'save') {
      if (failSave) {
        throw StateError('File changed on disk. Compare before saving.');
      }
      final d = docs[r['document']]!;
      d['snapshot'] = {...d['snapshot'] as Map, 'text': d['text']};
      d['version']++;
      d['dirty'] = false;
      return d;
    }
    return super.call(cmd);
  }
}

void main() {
  for (final control in [
    LogicalKeyboardKey.controlLeft,
    LogicalKeyboardKey.controlRight,
  ]) {
    testWidgets('editor keyboard undo/redo uses shared history with $control', (
      t,
    ) async {
      final host = AppHost(ChatController(EditingBridge())..loading = false);
      await host.files.bind('A', 'C:/A');
      final workspace = host.files.selected!;
      final document = (await host.files.open(workspace, 'a.txt'))!;
      await t.pumpWidget(
        MaterialApp(
          home: Scaffold(
            body: FileEditor(
              host: host,
              workspace: workspace,
              document: document,
            ),
          ),
        ),
      );
      await t.pump();
      final editor = t.widget<CodeEditor>(find.byType(CodeEditor));
      editor.focusNode!.requestFocus();
      editor.controller!.replaceSelection('mine ');
      await host.files.flush(document);
      await t.pump();
      expect(document.text, 'mine saved');
      expect(editor.focusNode!.hasFocus, isTrue);
      expect(document.pending, isFalse);
      expect(t.widget<CodeEditor>(find.byType(CodeEditor)).readOnly, isFalse);
      expect(editor.controller!.canUndo, isTrue);
      await t.sendKeyDownEvent(control);
      await t.sendKeyEvent(LogicalKeyboardKey.keyZ);
      await t.sendKeyUpEvent(control);
      await t.pump();
      expect(document.text, 'saved');
      await host.files.flush(document);
      await t.pump();
      await t.sendKeyDownEvent(control);
      await t.sendKeyDownEvent(LogicalKeyboardKey.shiftLeft);
      await t.sendKeyEvent(LogicalKeyboardKey.keyZ);
      await t.sendKeyUpEvent(LogicalKeyboardKey.shiftLeft);
      await t.sendKeyUpEvent(control);
      await t.pump();
      expect(document.text, 'mine saved');
      await host.files.flush(document);
      await t.pump();
      await t.sendKeyDownEvent(control);
      await t.sendKeyEvent(LogicalKeyboardKey.keyZ);
      await t.sendKeyUpEvent(control);
      await host.files.flush(document);
      await t.pump();
      await t.sendKeyDownEvent(control);
      await t.sendKeyEvent(LogicalKeyboardKey.keyY);
      await t.sendKeyUpEvent(control);
      await t.pump();
      expect(document.text, 'mine saved');
      await host.files.flush(document);
      await t.pump();
      await t.sendKeyDownEvent(control);
      await t.sendKeyEvent(LogicalKeyboardKey.keyS);
      await t.sendKeyUpEvent(control);
      await t.pumpAndSettle();
      expect(document.dirty, isFalse);
      expect(document.snapshot['text'], 'mine saved');
      await t.pumpWidget(const SizedBox());
      host.dispose();
    }, variant: const TargetPlatformVariant({TargetPlatform.windows}));
  }
  test('UTF-8 byte accounting refuses a multibyte long line and refreshes cached bases', () async {
    final host = AppHost(ChatController(EditingBridge())..loading = false);
    await host.files.bind('A', 'C:/A');
    final d = (await host.files.open(host.files.selected!, 'a.txt'))!;
    final view = d.buffer.createView();
    view.selection = const CodeLineSelection(
      baseIndex: 0,
      baseOffset: 0,
      extentIndex: 0,
      extentOffset: 5,
    );
    view.replaceSelection('😀' * 2048);
    await host.files.flush(d);
    expect(d.textBytes, 8192);
    expect(d.savedBytes, 5);
    view.replaceSelection('x');
    expect(d.error, contains('8 KiB'));
    expect(d.textBytes, 8192);
    expect(d.text, '😀' * 2048);
    d.accept({
      'snapshot': {...d.snapshot, 'text': 'new'},
      'text': 'new',
      'version': d.version + 1,
    });
    expect(d.textBytes, 3);
    expect(d.savedBytes, 3);
    final separate = '${'x' * 4097}\n${'y' * 4097}';
    d.accept({
      'snapshot': {...d.snapshot, 'text': separate},
      'text': separate,
      'version': d.version + 1,
    });
    view.selection = const CodeLineSelection(
      baseIndex: 0,
      baseOffset: 4097,
      extentIndex: 1,
      extentOffset: 0,
    );
    view.replaceSelection('');
    expect(d.error, contains('8 KiB'));
    expect(d.text, separate);
    view.dispose();
    host.dispose();
  });
  test('duplicate views share text and undo, with independent selection and bounded history', () {
    final doc = DocumentBuffer('a😀b\n', onEdit: (_) => true);
    final a = doc.createView(), b = doc.createView();
    b.selection = const CodeLineSelection.collapsed(index: 1, offset: 0);
    a.selection = const CodeLineSelection(
      baseIndex: 0,
      baseOffset: 1,
      extentIndex: 0,
      extentOffset: 3,
    );
    a.replaceSelection('世界');
    expect(a.text, 'a世界b\n');
    expect(b.text, a.text);
    expect(b.selection.baseIndex, 1);
    b.undo();
    expect(a.text, 'a😀b\n');
    a.redo();
    expect(b.text, 'a世界b\n');
    for (var i = 0; i < 80; i++) {
      a.replaceSelection('x');
    }
    expect(doc.undoEntries.length, 64);
    expect(doc.historyLimited, true);
    expect(
      doc.retainedUnits,
      lessThanOrEqualTo(DocumentBuffer.maxRetainedUnits),
    );
    a.dispose();
    b.dispose();
    expect(doc.views, isEmpty);
    doc.dispose();
  });
  test('one acknowledged transaction coalesces later edits and preserves local work on failed sync', () async {
    final bridge = EditingBridge();
    final host = AppHost(ChatController(bridge)..loading = false);
    await host.files.bind('A', 'C:/A');
    final w = host.files.selected!;
    final d = (await host.files.open(w, 'a.txt'))!;
    final view = d.buffer.createView();
    bridge.delayed = Completer<void>();
    view.replaceSelection('x');
    view.replaceSelection('y');
    expect(bridge.editorCalls.where((r) => r['action'] == 'edit').length, 1);
    bridge.delayed!.complete();
    await host.files.flush(d);
    bridge.delayed = null;
    expect(d.text, 'xysaved');
    expect(d.acknowledged, d.text);
    expect(d.version, 2);
    expect(
      bridge.editorCalls
          .where((r) => r['action'] == 'edit')
          .every((r) => !r.containsKey('text')),
      true,
    );
    bridge.fail = true;
    view.replaceSelection('z');
    await host.files.flush(d);
    expect(d.blocked, true);
    expect(d.text, 'xyzsaved');
    final count = bridge.editorCalls.where((r) => r['action'] == 'edit').length;
    view.replaceSelection('!');
    await host.files.flush(d);
    expect(
      bridge.editorCalls.where((r) => r['action'] == 'edit').length,
      count,
    );
    bridge.fail = false;
    await host.files.retrySync(d);
    expect(d.acknowledged, d.text);
    expect(d.blocked, false);
    bridge.failSave = true;
    expect(await host.files.save(w, d), false);
    expect(d.dirty, true);
    expect(d.text, 'xyz!saved');
    view.dispose();
    host.dispose();
  });
  test(
    'oversized paste is refused before mutation and keeps undo history',
    () async {
      final host = AppHost(ChatController(EditingBridge())..loading = false);
      await host.files.bind('A', 'C:/A');
      final d = (await host.files.open(host.files.selected!, 'a.txt'))!;
      final view = d.buffer.createView();
      view.replaceSelection('\n' * 65537);
      expect(d.text, 'saved');
      expect(view.text, 'saved');
      expect(d.buffer.undoEntries, isEmpty);
      expect(d.error, contains('64 KiB'));
      view.dispose();
      host.dispose();
    },
  );
  testWidgets(
    'editor Find and Save failure retain the active controller and visible draft',
    (t) async {
      t.view.physicalSize = const Size(900, 650);
      t.view.devicePixelRatio = 1;
      addTearDown(t.view.resetPhysicalSize);
      addTearDown(t.view.resetDevicePixelRatio);
      final bridge = EditingBridge();
      final host = AppHost(ChatController(bridge)..loading = false);
      await host.files.bind('A', 'C:/A');
      final w = host.files.selected!;
      final d = (await host.files.open(w, 'a.txt'))!;
      await t.pumpWidget(
        MaterialApp(
          home: Scaffold(
            body: FileEditor(host: host, workspace: w, document: d),
          ),
        ),
      );
      await t.pump();
      final editor = t.widget<CodeEditor>(find.byType(CodeEditor));
      expect(find.byKey(const Key('editor-find')), findsNothing);
      editor.controller!.replaceSelection('mine ');
      await t.pump();
      final refreshed = t.widget<CodeEditor>(find.byType(CodeEditor));
      expect(identical(editor.chunkAnalyzer, refreshed.chunkAnalyzer), true);
      expect(
        identical(editor.style!.codeTheme, refreshed.style!.codeTheme),
        true,
      );
      await host.files.flush(d);
      bridge.failSave = true;
      editor.focusNode!.requestFocus();
      await t.pump();
      await t.sendKeyDownEvent(LogicalKeyboardKey.controlLeft);
      await t.sendKeyEvent(LogicalKeyboardKey.keyS);
      await t.sendKeyUpEvent(LogicalKeyboardKey.controlLeft);
      await t.pump();
      await t.pump();
      expect(d.text, 'mine saved');
      expect(d.dirty, true);
      expect(find.textContaining('File changed on disk'), findsOneWidget);
      bridge.failSave = false;
      await t.sendKeyDownEvent(LogicalKeyboardKey.controlRight);
      await t.sendKeyEvent(LogicalKeyboardKey.keyS);
      await t.sendKeyUpEvent(LogicalKeyboardKey.controlRight);
      await t.pumpAndSettle();
      expect(d.text, 'mine saved');
      expect(d.dirty, false);
      expect(d.error, isNull);
      await t.tap(find.byTooltip('Editor actions'));
      await t.pumpAndSettle();
      await t.tap(find.text('Find (Ctrl+F)'));
      await t.pumpAndSettle();
      expect(find.byKey(const Key('editor-find')), findsOneWidget);
      d.accept({
        'snapshot': {...d.snapshot, 'path': 'large.rs', 'text': '\n' * 65537},
        'text': '\n' * 65537,
        'version': d.version + 1,
      });
      await t.pump();
      expect(
        t.widget<CodeEditor>(find.byType(CodeEditor)).style!.codeTheme,
        isNull,
      );
      expect(find.textContaining('Plain text above 64 KiB'), findsOneWidget);
      expect(t.takeException(), isNull);
      await t.pumpWidget(const SizedBox());
      host.dispose();
    },
    variant: const TargetPlatformVariant({TargetPlatform.windows}),
  );
}
