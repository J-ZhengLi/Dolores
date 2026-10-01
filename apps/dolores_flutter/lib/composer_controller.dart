import 'package:flutter/material.dart';

enum ComposerBlockKind { paragraph, heading, code }

/// Offsets refer to canonical Markdown; only the bodies appear in rich mode.
class ComposerBlock {
  final ComposerBlockKind kind;
  final int start, end, bodyStart, bodyEnd, level;
  final String language;
  const ComposerBlock(
    this.kind,
    this.start,
    this.end,
    this.bodyStart,
    this.bodyEnd, {
    this.level = 0,
    this.language = '',
  });
  String body(String source) => source.substring(bodyStart, bodyEnd);
}

final composerFence = RegExp(r'^( {0,3})(`{3,}|~{3,})([^\r\n]*)$');
final _heading = RegExp(r'^( {0,3})(#{1,6})[ \t]+');

List<ComposerBlock> parseComposer(String source) {
  if (source.length > 32768 || '\n'.allMatches(source).length > 800) {
    return [
      ComposerBlock(
        ComposerBlockKind.paragraph,
        0,
        source.length,
        0,
        source.length,
      ),
    ];
  }
  final blocks = <ComposerBlock>[];
  var pos = 0, paragraph = 0;
  int skipBreak(int start, int end) {
    if (start < end &&
        source[start] == '\r' &&
        start + 1 < end &&
        source[start + 1] == '\n') {
      return start + 2;
    }
    return start < end && source[start] == '\n' ? start + 1 : start;
  }

  void flush(int end, {bool trailing = false}) {
    if (end > paragraph) {
      var start = paragraph;
      if (blocks.isNotEmpty) start = skipBreak(start, end);
      var bodyEnd = end;
      if (!trailing && bodyEnd > start && source[bodyEnd - 1] == '\n') {
        bodyEnd--;
        if (bodyEnd > start && source[bodyEnd - 1] == '\r') bodyEnd--;
      }
      if (bodyEnd <= start && source.substring(paragraph, end).trim().isEmpty) {
        return;
      }
      blocks.add(
        ComposerBlock(
          ComposerBlockKind.paragraph,
          paragraph,
          end,
          start,
          bodyEnd,
        ),
      );
    }
  }

  while (pos < source.length) {
    final nl = source.indexOf('\n', pos);
    final lineEnd = nl < 0 ? source.length : nl + 1;
    final line = source
        .substring(pos, nl < 0 ? source.length : nl)
        .replaceFirst(RegExp(r'\r$'), '');
    final heading = _heading.firstMatch(line);
    final fence = composerFence.firstMatch(line);
    if (fence != null &&
        (!fence[2]!.startsWith('`') || !fence[3]!.contains('`')) &&
        nl >= 0) {
      flush(pos);
      final bodyStart = lineEnd;
      var next = bodyStart, bodyEnd = source.length, end = source.length;
      while (next < source.length) {
        final nextNl = source.indexOf('\n', next);
        final nextEnd = nextNl < 0 ? source.length : nextNl + 1;
        final closing = composerFence.firstMatch(
          source
              .substring(next, nextNl < 0 ? source.length : nextNl)
              .replaceFirst(RegExp(r'\r$'), ''),
        );
        if (closing != null &&
            closing[2]![0] == fence[2]![0] &&
            closing[2]!.length >= fence[2]!.length &&
            closing[3]!.trim().isEmpty) {
          bodyEnd = next;
          if (bodyEnd > bodyStart && source[bodyEnd - 1] == '\n') bodyEnd--;
          if (bodyEnd > bodyStart && source[bodyEnd - 1] == '\r') bodyEnd--;
          end = nextEnd;
          break;
        }
        next = nextEnd;
      }
      blocks.add(
        ComposerBlock(
          ComposerBlockKind.code,
          pos,
          end,
          bodyStart,
          bodyEnd,
          language: fence[3]!.trim().split(RegExp(r'\s+')).first,
        ),
      );
      pos = end;
      paragraph = pos;
      continue;
    } else if (heading != null) {
      flush(pos);
      blocks.add(
        ComposerBlock(
          ComposerBlockKind.heading,
          pos,
          lineEnd,
          pos + heading.end,
          pos + line.length,
          level: heading[2]!.length,
        ),
      );
      paragraph = lineEnd;
    }
    pos = lineEnd;
  }
  flush(source.length, trailing: true);
  if ((paragraph < source.length ||
          (blocks.isNotEmpty &&
              blocks.last.kind == ComposerBlockKind.heading &&
              source.endsWith('\n'))) &&
      blocks.isNotEmpty &&
      blocks.last.kind != ComposerBlockKind.paragraph &&
      source.substring(paragraph).trim().isEmpty) {
    blocks.add(
      ComposerBlock(
        ComposerBlockKind.paragraph,
        paragraph,
        source.length,
        skipBreak(paragraph, source.length),
        source.length,
      ),
    );
  }
  if (blocks.length > 64) {
    return [
      ComposerBlock(
        ComposerBlockKind.paragraph,
        0,
        source.length,
        0,
        source.length,
      ),
    ];
  }
  if (blocks.isEmpty) {
    blocks.add(
      ComposerBlock(
        ComposerBlockKind.paragraph,
        0,
        source.length,
        0,
        source.length,
      ),
    );
  }
  return blocks;
}

/// The stored/sent value remains Markdown, independent of visible blocks.
class ComposerController extends TextEditingController {
  ComposerController({super.text});
  final _undo = <TextEditingValue>[];
  final _redo = <TextEditingValue>[];
  void edit(TextEditingValue next, {bool record = true}) {
    if (next.text != text && record) {
      _undo.add(value.copyWith(composing: TextRange.empty));
      if (_undo.length > 40) _undo.removeAt(0);
      _redo.clear();
    }
    value = next;
  }

  void undo() {
    if (_undo.isEmpty) return;
    _redo.add(value.copyWith(composing: TextRange.empty));
    value = _undo.removeLast();
  }

  void redo() {
    if (_redo.isEmpty) return;
    _undo.add(value.copyWith(composing: TextRange.empty));
    value = _redo.removeLast();
  }

  void clearHistory() {
    _undo.clear();
    _redo.clear();
  }
}
