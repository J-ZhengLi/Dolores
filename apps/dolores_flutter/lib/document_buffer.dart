import 'dart:math' as math;

import 'package:flutter/services.dart';
import 'package:re_editor/re_editor.dart';

/// The document owns text/history; mounted views own independent selection/scroll.
class DocumentBuffer {
  late CodeLineEditingValue value;
  final bool Function(String) onEdit;
  final views = <DocumentView>[];
  final previous = <DocumentView, CodeLineEditingValue>{};
  final listeners = <DocumentView, void Function()>{};
  final undoEntries = <_Edit>[];
  final redoEntries = <_Edit>[];
  bool syncing = false, historyLimited = false;
  int revision = 0;
  static const maxEntries = 64, maxRetainedUnits = 4 * 1024 * 1024;
  DocumentBuffer(String text, {required this.onEdit}) {
    final seed = CodeLineEditingController.fromText(text);
    value = seed.value;
    seed.dispose();
  }
  DocumentView createView() {
    final view = DocumentView._(
      this,
      CodeLineEditingController(codeLines: value.codeLines),
    );
    previous[view] = view.value;
    listeners[view] = () => changed(view);
    view.addListener(listeners[view]!);
    views.add(view);
    return view;
  }

  void changed(DocumentView source) {
    if (syncing) return;
    final before = previous[source]!;
    previous[source] = source.value;
    if (identical(value.codeLines, source.codeLines)) return;
    if (!onEdit(source.text)) {
      publish(value, undoView: source);
      return;
    }
    undoEntries.add(
      _Edit(value.copyWith(selection: before.selection), source.value),
    );
    redoEntries.clear();
    while (undoEntries.length > maxEntries ||
        retainedUnits > maxRetainedUnits) {
      undoEntries.removeAt(0);
      historyLimited = true;
    }
    publish(source.value, source: source);
  }

  int get retainedUnits =>
      [...undoEntries, ...redoEntries].fold(0, (n, e) => n + e.units);
  CodeLineSelection clamp(CodeLineSelection s, CodeLines lines) {
    final a = math.max(0, math.min(s.baseIndex, lines.length - 1)),
        b = math.max(0, math.min(s.extentIndex, lines.length - 1));
    return s.copyWith(
      baseIndex: a,
      baseOffset: math.max(0, math.min(s.baseOffset, lines[a].length)),
      extentIndex: b,
      extentOffset: math.max(0, math.min(s.extentOffset, lines[b].length)),
    );
  }

  void publish(
    CodeLineEditingValue next, {
    DocumentView? source,
    DocumentView? undoView,
  }) {
    value = next;
    revision++;
    syncing = true;
    try {
      for (final v in views) {
        if (v != source) {
          v.value = v.value.copyWith(
            codeLines: next.codeLines,
            selection: clamp(
              v == undoView ? next.selection : v.selection,
              next.codeLines,
            ),
            composing: TextRange.empty,
          );
        }
        v.clearHistory();
        previous[v] = v.value;
      }
    } finally {
      syncing = false;
    }
  }

  void replace(String text) {
    final seed = CodeLineEditingController.fromText(text);
    undoEntries.clear();
    redoEntries.clear();
    publish(seed.value);
    seed.dispose();
  }

  void undo(DocumentView from) {
    if (undoEntries.isEmpty) return;
    final entry = undoEntries.last;
    if (!onEdit(entry.before.codeLines.asString(TextLineBreak.lf))) return;
    undoEntries.removeLast();
    redoEntries.add(entry);
    publish(entry.before, undoView: from);
  }

  void redo(DocumentView from) {
    if (redoEntries.isEmpty) return;
    final entry = redoEntries.last;
    if (!onEdit(entry.after.codeLines.asString(TextLineBreak.lf))) return;
    redoEntries.removeLast();
    undoEntries.add(entry);
    publish(entry.after, undoView: from);
  }

  void remove(DocumentView v) {
    v.removeListener(listeners.remove(v)!);
    views.remove(v);
    previous.remove(v);
  }

  void dispose() {
    for (final v in List<DocumentView>.of(views)) {
      v.dispose();
    }
  }
}

class DocumentView extends CodeLineEditingControllerDelegate {
  bool _disposed=false;
  final DocumentBuffer document;
  DocumentView._(this.document, CodeLineEditingController backend)
    : super(delegate: backend);
  @override
  bool get canUndo => document.undoEntries.isNotEmpty;
  @override
  bool get canRedo => document.redoEntries.isNotEmpty;
  @override
  void undo() => document.undo(this);
  @override
  void redo() => document.redo(this);
  @override
  void dispose() {
    if(_disposed)return;
    _disposed=true;
    document.remove(this);
    super.dispose();
  }
}

class _Edit {
  final CodeLineEditingValue before, after;
  late final int units;
  _Edit(this.before, this.after) {
    var n = 0;
    for (final v in [before, after]) {
      n += v.codeLines.length - 1;
      for (var i = 0; i < v.codeLines.length; i++) {
        n += v.codeLines[i].charCount;
      }
    }
    units = n;
  }
}
