import 'dart:async';

import 'package:flutter/material.dart';
import 'package:flutter/services.dart';

import 'code_syntax.dart';
import 'composer_controller.dart';
import 'theme.dart';

class RichComposer extends StatefulWidget {
  final ComposerController controller;
  final FocusNode focusNode;
  final bool readOnly;
  final String hint;
  final ValueChanged<String> onChanged;
  final VoidCallback onSend;
  final Widget trailing;
  const RichComposer({
    super.key,
    required this.controller,
    required this.focusNode,
    required this.readOnly,
    required this.hint,
    required this.onChanged,
    required this.onSend,
    required this.trailing,
  });
  @override
  State<RichComposer> createState() => _RichComposerState();
}

/// Material TextField only highlights the focused field. Paint native selection
/// boxes behind the other fields when the whole document is selected.
class _ComposerSelectionPainter extends CustomPainter {
  final bool selected;
  final FocusNode focus;
  final GlobalKey owner;
  final Color color;
  _ComposerSelectionPainter({
    required this.selected,
    required this.focus,
    required this.owner,
    required this.color,
  });
  @override
  void paint(Canvas canvas, Size size) {
    if (!selected || focus.hasFocus) return;
    final editable = focus.context
        ?.findAncestorStateOfType<EditableTextState>()
        ?.renderEditable;
    final parent = owner.currentContext?.findRenderObject();
    if (editable == null ||
        !editable.attached ||
        parent is! RenderBox ||
        !parent.hasSize) {
      return;
    }
    canvas.save();
    canvas.clipRect(Offset.zero & size);
    final paint = Paint()..color = color;
    for (final box in editable.getBoxesForSelection(editable.selection!)) {
      final rect = box.toRect();
      canvas.drawRect(
        Rect.fromPoints(
          parent.globalToLocal(editable.localToGlobal(rect.topLeft)),
          parent.globalToLocal(editable.localToGlobal(rect.bottomRight)),
        ),
        paint,
      );
    }
    canvas.restore();
  }

  @override
  bool shouldRepaint(_ComposerSelectionPainter old) =>
      selected != old.selected || focus != old.focus || color != old.color;
}

class _RichComposerState extends State<RichComposer> {
  List<ComposerBlock> blocks = [];
  final fields = <TextEditingController>[];
  final focuses = <FocusNode>[];
  final selectionKeys = <GlobalKey>[];
  String seen = '';
  bool syncing = false, editing = false;
  bool allSelected = false;
  bool selectionTyping = false;
  int active = 0;
  ComposerController get input => widget.controller;

  @override
  void initState() {
    super.initState();
    input.addListener(_sync);
    _sync(rebuild: false);
  }

  void _sync({bool rebuild = true}) {
    if (syncing) return;
    // Do not fold a just-typed marker in the middle of an IME composition.
    if (input.value.isComposingRangeValid &&
        !input.value.composing.isCollapsed) {
      return;
    }
    if (seen == input.text && blocks.isNotEmpty) return;
    allSelected = false;
    final next = parseComposer(input.text);
    final shapeChanged =
        blocks.length != next.length ||
        List.generate(
          next.length,
          (i) => i,
        ).any((i) => i >= blocks.length || blocks[i].kind != next[i].kind);
    final hadFocus = focuses.any((f) => f.hasFocus);
    syncing = true;
    if (shapeChanged) {
      // Keep native editor state, controller and focus when a block changes
      // appearance. Replacing its keyed subtree closes the input connection.
      final removedFields = fields.skip(next.length).toList();
      final removedFocuses = focuses.skip(next.length).toList();
      if (fields.length > next.length) {
        fields.removeRange(next.length, fields.length);
        focuses.removeRange(next.length, focuses.length);
        selectionKeys.removeRange(next.length, selectionKeys.length);
      }
      for (var i = fields.length; i < next.length; i++) {
        final block = next[i];
        final field = CodeSyntaxController(
          text: block.body(input.text),
          language: block.kind == ComposerBlockKind.code ? block.language : '',
        );
        fields.add(field);
        focuses.add(i == 0 ? widget.focusNode : FocusNode());
        selectionKeys.add(GlobalKey());
        final index = i;
        field.addListener(() => _fieldChanged(index));
      }
      WidgetsBinding.instance.addPostFrameCallback((_) {
        for (final c in removedFields) {
          c.dispose();
        }
        for (final f in removedFocuses) {
          if (f != widget.focusNode) f.dispose();
        }
      });
    }
    blocks = next;
    for (var i = 0; i < fields.length; i++) {
      final field = fields[i], block = blocks[i];
      if (field is CodeSyntaxController) {
        field.language = block.kind == ComposerBlockKind.code
            ? block.language
            : '';
      }
      final body = block.body(input.text);
      if (field.text != body) {
        field.value = TextEditingValue(
          text: body,
          selection: TextSelection.collapsed(
            offset: (input.selection.extentOffset - block.bodyStart).clamp(
              0,
              body.length,
            ),
          ),
        );
      }
    }
    seen = input.text;
    syncing = false;
    if (rebuild && mounted) setState(() {});
    if (shapeChanged && hadFocus && !widget.readOnly) {
      final offset = input.selection.extentOffset;
      active = blocks.indexWhere((b) => offset < b.end);
      if (active < 0) active = blocks.length - 1;
      final index = active;
      WidgetsBinding.instance.addPostFrameCallback((_) {
        if (!mounted || index >= fields.length) return;
        fields[index].selection = TextSelection.collapsed(
          offset: (input.selection.extentOffset - blocks[index].bodyStart)
              .clamp(0, fields[index].text.length),
        );
        focuses[index].requestFocus();
        // A reparented editor can already have focus but lack a keyboard token.
        // Reopen its native input connection without requiring another click.
        focuses[index].context
            ?.findAncestorStateOfType<EditableTextState>()
            ?.requestKeyboard();
      });
    }
  }

  void _fieldChanged(int index) {
    if (syncing || editing || index >= blocks.length || widget.readOnly) {
      return;
    }
    final block = blocks[index], field = fields[index];
    if (allSelected) {
      if (field.text != block.body(input.text) ||
          selectionTyping ||
          (field.value.isComposingRangeValid &&
              !field.value.composing.isCollapsed)) {
        _replaceSelection(field.value, index);
        return;
      }
      if (field.selection ==
          TextSelection(baseOffset: 0, extentOffset: field.text.length)) {
        return;
      }
      _clearSelection(index);
    }
    if (!focuses[index].hasFocus && field.text == block.body(input.text)) {
      return;
    }
    active = index;
    final newText = input.text.replaceRange(
      block.bodyStart,
      block.bodyEnd,
      field.text,
    );
    final selection = field.selection;
    final composing = field.value.composing;
    editing = true;
    input.edit(
      TextEditingValue(
        text: newText,
        selection: TextSelection(
          baseOffset:
              block.bodyStart +
              selection.baseOffset.clamp(0, field.text.length),
          extentOffset:
              block.bodyStart +
              selection.extentOffset.clamp(0, field.text.length),
        ),
        composing: composing.isValid && !composing.isCollapsed
            ? TextRange(
                start: block.bodyStart + composing.start,
                end: block.bodyStart + composing.end,
              )
            : TextRange.empty,
      ),
    );
    editing = false;
    if (newText != seen &&
        input.value.isComposingRangeValid &&
        !input.value.composing.isCollapsed) {
      // Keep ranges current while postponing any structural folding.
      final shift = field.text.length - (block.bodyEnd - block.bodyStart);
      blocks = [
        for (var i = 0; i < blocks.length; i++)
          if (i < index)
            blocks[i]
          else if (i == index)
            ComposerBlock(
              block.kind,
              block.start,
              block.end + shift,
              block.bodyStart,
              block.bodyEnd + shift,
              level: block.level,
              language: block.language,
            )
          else
            ComposerBlock(
              blocks[i].kind,
              blocks[i].start + shift,
              blocks[i].end + shift,
              blocks[i].bodyStart + shift,
              blocks[i].bodyEnd + shift,
              level: blocks[i].level,
              language: blocks[i].language,
            ),
      ];
    }
    widget.onChanged(input.text);
  }

  void _selectAll() {
    if (widget.readOnly ||
        !input.value.composing.isCollapsed ||
        input.text.isEmpty) {
      return;
    }
    syncing = true;
    allSelected = true;
    selectionTyping = false;
    for (final field in fields) {
      field.selection = TextSelection(
        baseOffset: 0,
        extentOffset: field.text.length,
      );
    }
    input.value = input.value.copyWith(
      selection: TextSelection(baseOffset: 0, extentOffset: input.text.length),
    );
    syncing = false;
    setState(() {});
  }

  void _clearSelection(int keep) {
    syncing = true;
    allSelected = false;
    selectionTyping = false;
    for (var i = 0; i < fields.length; i++) {
      if (i != keep) {
        fields[i].selection = TextSelection.collapsed(
          offset: fields[i].text.length,
        );
      }
    }
    syncing = false;
    setState(() {});
  }

  void _replaceSelection(TextEditingValue replacement, int index) {
    allSelected = false;
    selectionTyping = false;
    if (replacement.isComposingRangeValid &&
        !replacement.composing.isCollapsed) {
      // Keep the focused native field and its IME connection until composition commits.
      syncing = true;
      blocks = [
        for (var i = 0; i < blocks.length; i++)
          ComposerBlock(
            blocks[i].kind,
            0,
            i == index ? replacement.text.length : 0,
            0,
            i == index ? replacement.text.length : 0,
            level: blocks[i].level,
            language: blocks[i].language,
          ),
      ];
      for (var i = 0; i < fields.length; i++) {
        if (i != index) fields[i].clear();
      }
      input.edit(replacement);
      seen = '\u0000';
      syncing = false;
      setState(() {});
    } else {
      input.edit(replacement);
    }
    widget.onChanged(input.text);
  }

  Future<void> _copySelection({bool cut = false}) async {
    final text = input.text;
    try {
      await Clipboard.setData(ClipboardData(text: text));
      if (cut &&
          mounted &&
          allSelected &&
          !widget.readOnly &&
          input.text == text) {
        _replaceSelection(TextEditingValue.empty, active);
      }
    } catch (_) {
      _clipboardFailure();
    }
  }

  Future<void> _pasteSelection() async {
    final text = input.text;
    try {
      final data = await Clipboard.getData(Clipboard.kTextPlain);
      if (!mounted ||
          !allSelected ||
          widget.readOnly ||
          input.text != text ||
          !input.value.composing.isCollapsed ||
          data?.text == null) {
        return;
      }
      _replaceSelection(
        TextEditingValue(
          text: data!.text!,
          selection: TextSelection.collapsed(offset: data.text!.length),
        ),
        active,
      );
    } catch (_) {
      _clipboardFailure();
    }
  }

  void _clipboardFailure() {
    if (mounted) {
      ScaffoldMessenger.maybeOf(context)?.showSnackBar(
        const SnackBar(content: Text('Could not access the clipboard.')),
      );
    }
  }

  void _removeEmptyBlock(int index) {
    final block = blocks[index];
    var text = input.text.replaceRange(block.start, block.end, '');
    if (text.trim().isEmpty) text = '';
    input.edit(
      TextEditingValue(
        text: text,
        selection: TextSelection.collapsed(
          offset: block.start.clamp(0, text.length),
        ),
      ),
    );
    widget.onChanged(input.text);
  }

  void _language(int index, String language) {
    final b = blocks[index];
    final prefix = input.text.substring(b.start, b.bodyStart);
    final match = composerFence.firstMatch(prefix.trimRight())!;
    final ending = prefix.endsWith('\r\n') ? '\r\n' : '\n';
    final replacement = '${match[1]}${match[2]}$language$ending';
    input.edit(
      TextEditingValue(
        text: input.text.replaceRange(b.start, b.bodyStart, replacement),
        selection: TextSelection.collapsed(
          offset: b.start + replacement.length,
        ),
        composing: TextRange.empty,
      ),
    );
    widget.onChanged(input.text);
    focuses[index].requestFocus();
  }

  void _exitLastBlock() {
    if (widget.readOnly || !input.value.composing.isCollapsed) return;
    final last = blocks.last;
    var text = input.text;
    final newline = text.contains('\r\n') ? '\r\n' : '\n';
    if (last.kind == ComposerBlockKind.code && last.bodyEnd == text.length) {
      final fence = composerFence.firstMatch(
        text.substring(last.start, last.bodyStart).trimRight(),
      )!;
      text += '$newline${fence[2]}';
    }
    text += '$newline$newline';
    input.edit(
      TextEditingValue(
        text: text,
        selection: TextSelection.collapsed(offset: text.length),
      ),
    );
    widget.onChanged(input.text);
    WidgetsBinding.instance.addPostFrameCallback((_) {
      if (mounted) focuses.last.requestFocus();
    });
  }

  void _insertNewline() {
    if (allSelected) {
      _replaceSelection(
        const TextEditingValue(
          text: '\n',
          selection: TextSelection.collapsed(offset: 1),
        ),
        active,
      );
      return;
    }
    final field = fields[active];
    final selection = field.selection;
    final start = selection.start.clamp(0, field.text.length);
    final end = selection.end.clamp(start, field.text.length);
    final newline = input.text.contains('\r\n') ? '\r\n' : '\n';
    final insertion = blocks[active].kind == ComposerBlockKind.heading
        ? '$newline$newline'
        : newline;
    field.value = TextEditingValue(
      text: field.text.replaceRange(start, end, insertion),
      selection: TextSelection.collapsed(offset: start + insertion.length),
    );
  }

  KeyEventResult _key(FocusNode node, KeyEvent event) {
    if ((event is! KeyDownEvent && event is! KeyRepeatEvent) ||
        widget.readOnly) {
      return KeyEventResult.ignored;
    }
    final keyboard = HardwareKeyboard.instance;
    final modified = keyboard.isControlPressed || keyboard.isMetaPressed;
    if (!input.value.composing.isCollapsed) return KeyEventResult.ignored;
    if (event.logicalKey == LogicalKeyboardKey.enter &&
        keyboard.isShiftPressed) {
      _insertNewline();
      return KeyEventResult.handled;
    }
    if (modified && event.logicalKey == LogicalKeyboardKey.keyA) {
      _selectAll();
      return KeyEventResult.handled;
    }
    if (allSelected) {
      if (modified &&
          (event.logicalKey == LogicalKeyboardKey.keyC ||
              event.logicalKey == LogicalKeyboardKey.keyX)) {
        unawaited(
          _copySelection(cut: event.logicalKey == LogicalKeyboardKey.keyX),
        );
        return KeyEventResult.handled;
      }
      if ((modified && event.logicalKey == LogicalKeyboardKey.keyV) ||
          (keyboard.isShiftPressed &&
              event.logicalKey == LogicalKeyboardKey.insert)) {
        unawaited(_pasteSelection());
        return KeyEventResult.handled;
      }
      if (event.logicalKey == LogicalKeyboardKey.backspace ||
          event.logicalKey == LogicalKeyboardKey.delete) {
        _replaceSelection(TextEditingValue.empty, active);
        return KeyEventResult.handled;
      }
      final start =
          event.logicalKey == LogicalKeyboardKey.arrowLeft ||
          event.logicalKey == LogicalKeyboardKey.arrowUp ||
          event.logicalKey == LogicalKeyboardKey.home;
      final end =
          event.logicalKey == LogicalKeyboardKey.arrowRight ||
          event.logicalKey == LogicalKeyboardKey.arrowDown ||
          event.logicalKey == LogicalKeyboardKey.end;
      if (!modified &&
          !keyboard.isShiftPressed &&
          (start || end || event.logicalKey == LogicalKeyboardKey.escape)) {
        final index = start
            ? 0
            : end
            ? fields.length - 1
            : active;
        _clearSelection(index);
        active = index;
        fields[index].selection = TextSelection.collapsed(
          offset: start ? 0 : fields[index].text.length,
        );
        focuses[index].requestFocus();
        return KeyEventResult.handled;
      }
      // An insertion may match the focused field's existing text. Remember the
      // typing intent so its native value update still replaces the document.
      if (!modified &&
          !keyboard.isAltPressed &&
          event.character != null &&
          event.character!.runes.any((rune) => rune >= 32)) {
        selectionTyping = true;
      }
    }
    if (modified && event.logicalKey == LogicalKeyboardKey.keyZ) {
      if (keyboard.isShiftPressed) {
        input.redo();
      } else {
        input.undo();
      }
      widget.onChanged(input.text);
      return KeyEventResult.handled;
    }
    if (modified && event.logicalKey == LogicalKeyboardKey.keyY) {
      input.redo();
      widget.onChanged(input.text);
      return KeyEventResult.handled;
    }
    if (event.logicalKey == LogicalKeyboardKey.enter &&
        !keyboard.isShiftPressed) {
      final inCode =
          blocks[active.clamp(0, blocks.length - 1)].kind ==
          ComposerBlockKind.code;
      final opening = composerFence.hasMatch(
        fields[active].text.split('\n').last,
      );
      if (modified || (!inCode && !opening)) {
        widget.onSend();
        return KeyEventResult.handled;
      }
      _insertNewline();
      return KeyEventResult.handled;
    }
    // Native TextFields retain selection, clipboard, IME and line navigation.
    if (!modified && !keyboard.isShiftPressed && fields.isNotEmpty) {
      final field = fields[active.clamp(0, fields.length - 1)];
      if (field.selection.isCollapsed) {
        final offset = field.selection.extentOffset;
        if (event.logicalKey == LogicalKeyboardKey.backspace && offset == 0) {
          if (field.text.isEmpty &&
              (blocks[active].kind != ComposerBlockKind.paragraph ||
                  active > 0)) {
            _removeEmptyBlock(active);
            return KeyEventResult.handled;
          }
          if (active > 0 && fields[active - 1].text.isEmpty) {
            _removeEmptyBlock(active - 1);
            return KeyEventResult.handled;
          }
          if (blocks[active].kind == ComposerBlockKind.heading) {
            final block = blocks[active];
            input.edit(
              TextEditingValue(
                text: input.text.replaceRange(block.start, block.bodyStart, ''),
                selection: TextSelection.collapsed(offset: block.start),
              ),
            );
            widget.onChanged(input.text);
            return KeyEventResult.handled;
          }
        }
        final line = focuses[active].context
            ?.findAncestorStateOfType<EditableTextState>()
            ?.renderEditable
            .getLineAtOffset(
              TextPosition(
                offset: offset.clamp(0, field.text.length),
                affinity: field.selection.affinity,
              ),
            );
        final up =
            event.logicalKey == LogicalKeyboardKey.arrowUp &&
            (line != null
                ? line.start == 0
                : !field.text
                      .substring(0, offset.clamp(0, field.text.length))
                      .contains('\n'));
        final down =
            event.logicalKey == LogicalKeyboardKey.arrowDown &&
            (line != null
                ? line.end >= field.text.length
                : !field.text
                      .substring(offset.clamp(0, field.text.length))
                      .contains('\n'));
        if ((up ||
                (event.logicalKey == LogicalKeyboardKey.arrowLeft &&
                    offset == 0)) &&
            active > 0) {
          active--;
          fields[active].selection = TextSelection.collapsed(
            offset: fields[active].text.length,
          );
          focuses[active].requestFocus();
          return KeyEventResult.handled;
        }
        if (down &&
            active == fields.length - 1 &&
            blocks[active].kind != ComposerBlockKind.paragraph) {
          _exitLastBlock();
          return KeyEventResult.handled;
        }
        if ((down ||
                (event.logicalKey == LogicalKeyboardKey.arrowRight &&
                    offset == field.text.length)) &&
            active + 1 < fields.length) {
          active++;
          fields[active].selection = const TextSelection.collapsed(offset: 0);
          focuses[active].requestFocus();
          return KeyEventResult.handled;
        }
      }
    }
    return KeyEventResult.ignored;
  }

  @override
  void dispose() {
    input.removeListener(_sync);
    for (final c in fields) {
      c.dispose();
    }
    for (final f in focuses) {
      if (f != widget.focusNode) f.dispose();
    }
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    final p = Palette(Theme.of(context).brightness == Brightness.dark);
    final rich = blocks.any((b) => b.kind != ComposerBlockKind.paragraph);
    InputDecoration decoration({String? hint}) => InputDecoration(
      hintText: hint,
      hintStyle: TextStyle(color: p.muted),
      filled: false,
      border: InputBorder.none,
      enabledBorder: InputBorder.none,
      focusedBorder: InputBorder.none,
      isDense: true,
      contentPadding: const EdgeInsets.symmetric(vertical: 8),
    );
    Widget field(int i) {
      final b = blocks[i];
      final code = b.kind == ComposerBlockKind.code;
      final editor = CustomPaint(
        key: selectionKeys[i],
        painter: _ComposerSelectionPainter(
          selected: allSelected,
          focus: focuses[i],
          owner: selectionKeys[i],
          color:
              TextSelectionTheme.of(context).selectionColor ??
              p.accent.withValues(alpha: .3),
        ),
        child: Focus(
          onFocusChange: (yes) {
            if (yes) active = i;
          },
          child: TextField(
            key: Key('composer-field-$i'),
            controller: fields[i],
            focusNode: focuses[i],
            readOnly: widget.readOnly,
            contextMenuBuilder: (context, editable) =>
                AdaptiveTextSelectionToolbar.buttonItems(
                  anchors: editable.contextMenuAnchors,
                  buttonItems: [
                    for (final item in editable.contextMenuButtonItems)
                      ContextMenuButtonItem(
                        type: item.type,
                        label: item.label,
                        onPressed: () {
                          switch (item.type) {
                            case ContextMenuButtonType.selectAll:
                              _selectAll();
                            case ContextMenuButtonType.copy when allSelected:
                              unawaited(_copySelection());
                            case ContextMenuButtonType.cut when allSelected:
                              unawaited(_copySelection(cut: true));
                            case ContextMenuButtonType.paste when allSelected:
                              unawaited(_pasteSelection());
                            default:
                              item.onPressed?.call();
                          }
                          editable.hideToolbar();
                        },
                      ),
                  ],
                ),
            minLines: 1,
            maxLines: null,
            keyboardType: TextInputType.multiline,
            style: TextStyle(
              fontSize: code
                  ? 14
                  : b.kind == ComposerBlockKind.heading
                  ? (b.level == 1 ? 24 : 20)
                  : 14,
              fontWeight: b.kind == ComposerBlockKind.heading
                  ? FontWeight.w600
                  : FontWeight.normal,
              fontFamily: code ? UiTokens.codeFont : null,
              fontFamilyFallback: code ? UiTokens.codeFontFallback : null,
              height: code ? 1.5 : 1.55,
              color: p.text,
            ),
            decoration: decoration(hint: !rich && i == 0 ? widget.hint : null),
          ),
        ),
      );
      if (!code) return editor;
      final lang = codeLanguage(b.language);
      return Container(
        key: Key('composer-code-$i'),
        width: double.infinity,
        padding: const EdgeInsets.fromLTRB(12, 4, 12, 8),
        decoration: BoxDecoration(
          color: p.sidebar,
          borderRadius: BorderRadius.circular(12),
        ),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            PopupMenuButton<String>(
              key: Key('composer-language-$i'),
              tooltip: 'Code language',
              enabled: !widget.readOnly,
              onSelected: (v) => _language(i, v),
              itemBuilder: (_) => [
                for (final e in codeLanguages.entries)
                  PopupMenuItem(value: e.key, child: Text(e.value)),
              ],
              child: Padding(
                padding: const EdgeInsets.symmetric(vertical: 8),
                child: Row(
                  mainAxisSize: MainAxisSize.min,
                  children: [
                    Text(
                      codeLanguages[lang] ??
                          (b.language.isEmpty ? 'Plain text' : b.language),
                      style: TextStyle(fontSize: 12, color: p.muted),
                    ),
                    const SizedBox(width: 4),
                    Icon(Icons.expand_more, size: 14, color: p.muted),
                  ],
                ),
              ),
            ),
            editor,
          ],
        ),
      );
    }

    final editor = ConstrainedBox(
      constraints: const BoxConstraints(maxHeight: 320),
      child: SingleChildScrollView(
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.stretch,
          children: [
            for (var i = 0; i < blocks.length; i++)
              Padding(
                padding: EdgeInsets.only(bottom: i < blocks.length - 1 ? 8 : 0),
                child: field(i),
              ),
          ],
        ),
      ),
    );
    return Actions(
      actions: {
        SelectAllTextIntent: CallbackAction<SelectAllTextIntent>(
          onInvoke: (_) {
            _selectAll();
            return null;
          },
        ),
        CopySelectionTextIntent: _ComposerCopyAction(this),
        PasteTextIntent: _ComposerPasteAction(this),
      },
      child: Focus(
        onKeyEvent: _key,
        child: rich
            ? Column(
                crossAxisAlignment: CrossAxisAlignment.stretch,
                mainAxisSize: MainAxisSize.min,
                children: [
                  editor,
                  Row(children: [const Spacer(), widget.trailing]),
                ],
              )
            : Row(
                crossAxisAlignment: CrossAxisAlignment.end,
                children: [
                  Expanded(child: editor),
                  const SizedBox(width: 8),
                  widget.trailing,
                ],
              ),
      ),
    );
  }
}

class _ComposerCopyAction extends Action<CopySelectionTextIntent> {
  final _RichComposerState state;
  _ComposerCopyAction(this.state);
  @override
  bool isEnabled(CopySelectionTextIntent intent) =>
      state.allSelected || (callingAction?.isEnabled(intent) ?? false);
  @override
  Object? invoke(CopySelectionTextIntent intent) {
    if (!state.allSelected) return callingAction?.invoke(intent);
    unawaited(state._copySelection(cut: intent.collapseSelection));
    return null;
  }
}

class _ComposerPasteAction extends Action<PasteTextIntent> {
  final _RichComposerState state;
  _ComposerPasteAction(this.state);
  @override
  bool isEnabled(PasteTextIntent intent) =>
      state.allSelected || (callingAction?.isEnabled(intent) ?? false);
  @override
  Object? invoke(PasteTextIntent intent) {
    if (!state.allSelected) return callingAction?.invoke(intent);
    unawaited(state._pasteSelection());
    return null;
  }
}
