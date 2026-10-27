import 'dart:async';

import 'package:flutter/material.dart';
import 'package:flutter/services.dart';

import 'chat.dart';
import 'composer_controller.dart';
import 'goal_panel.dart';
import 'planning_panel.dart';

/// Commands open at line start; skills also open within ordinary prose.
TextRange? commandQuery(TextEditingValue value) {
  if (!value.composing.isCollapsed ||
      !value.selection.isCollapsed ||
      !value.selection.isValid ||
      value.text.length > 32768) {
    return null;
  }
  final end = value.selection.extentOffset;
  if (end > value.text.length) {
    return null;
  }
  if (end < value.text.length &&
      !RegExp(r'[\s.,!?;:)\]}，。！？；：）】]').hasMatch(value.text[end])) {
    return null;
  }
  final block = parseComposer(value.text)
      .where((b) => end >= b.bodyStart && end <= b.end)
      .firstOrNull;
  if (block?.kind == ComposerBlockKind.code) {
    return null;
  }
  final start = end == 0 ? 0 : value.text.lastIndexOf('\n', end - 1) + 1;
  final line = value.text.substring(start, end);
  if (line.startsWith('    ') || line.startsWith('\t')) return null;
  final match = RegExp(
    r'(^|[\s([{，。；：！？、\u3400-\u9fff])(/[-a-zA-Z0-9_]*|@[-a-zA-Z0-9_]*)$',
  ).firstMatch(line);
  if (match == null) return null;
  final range = TextRange(start: end - match[2]!.length, end: end);
  if (value.text[range.start] == '@' && !_atLineStart(value.text, range)) {
    return null;
  }
  // Ignore closed and unfinished inline code, including multiline code spans.
  var delimiter = '';
  final prefix = value.text.substring(block?.bodyStart ?? start, range.start);
  for (final ticks in RegExp(r'(?<!\\)`+').allMatches(prefix)) {
    if (delimiter.isEmpty) {
      delimiter = ticks[0]!;
    } else if (delimiter == ticks[0]) {
      delimiter = '';
    }
  }
  return delimiter.isEmpty ? range : null;
}

bool _atLineStart(String text, TextRange range) {
  final start = range.start == 0
      ? 0
      : text.lastIndexOf('\n', range.start - 1) + 1;
  return RegExp(r'^ {0,3}$').hasMatch(text.substring(start, range.start));
}

class ComposerCommands extends StatefulWidget {
  final ChatController chat;
  final ComposerController input;
  final FocusNode focus;
  const ComposerCommands({
    super.key,
    required this.chat,
    required this.input,
    required this.focus,
  });
  @override
  State<ComposerCommands> createState() => ComposerCommandsState();
}

class ComposerCommandsState extends State<ComposerCommands> {
  final scroll = ScrollController();
  bool loading = false;
  String? failure, dismissed;
  int selected = 0;
  String? loadedSession;
  double get rowExtent =>
      ((MediaQuery.textScalerOf(context).scale(14) * 1.4).ceilToDouble() +
              (MediaQuery.textScalerOf(context).scale(12) * 1.4).ceilToDouble() +
              16)
          .clamp(56, double.infinity);
  TextRange? get query =>
      dismissed == widget.input.text ? null : commandQuery(widget.input.value);
  @override
  void initState() {
    super.initState();
    widget.input.addListener(changed);
    WidgetsBinding.instance.addPostFrameCallback((_) {
      if (mounted) {
        unawaited(refresh());
      }
    });
  }

  @override
  void didUpdateWidget(ComposerCommands oldWidget) {
    super.didUpdateWidget(oldWidget);
    if (loadedSession != widget.chat.session && !loading) {
      WidgetsBinding.instance.addPostFrameCallback((_) {
        if (mounted) {
          unawaited(refresh());
        }
      });
    }
  }

  void changed() {
    if (!mounted) return;
    setState(() {
      selected = 0;
    });
  }

  Future<void> refresh() async {
    if (loading) return;
    setState(() {
      loading = true;
      failure = null;
    });
    final target = widget.chat.session;
    loadedSession = target;
    try {
      await widget.chat.refreshCommands();
    } catch (e) {
      if (mounted) {
        failure = 'Could not load commands. Refresh to try again. ($e)';
      }
    } finally {
      if (mounted) {
        setState(() {
          loading = false;
        });
        if (loadedSession != widget.chat.session) {
          WidgetsBinding.instance.addPostFrameCallback((_) {
            if (mounted) {
              unawaited(refresh());
            }
          });
        }
      }
    }
  }

  List<Map<String, dynamic>> entries(TextRange range) {
    final filter = widget.input.text
        .substring(range.start + 1, range.end)
        .toLowerCase();
    final chat = widget.chat;
    final isReference = widget.input.text[range.start] == '@';
    if (isReference) {
      return widget.chat.planningAvailable
          ? [
              for (final kind in [
                'file',
                'folder',
                'code',
                'terminal',
                'message',
                'conversation',
              ])
                {
                  'kind': 'reference',
                  'id': kind,
                  'title': kind,
                  'description': 'Preview and select a $kind snapshot',
                  'available': !chat.busy,
                },
            ].where((e) => '${e['title']}'.contains(filter)).toList()
          : [];
    }
    final all = [
      if (_atLineStart(widget.input.text, range))
        for (final c in (chat.commandCatalogue['commands'] as List? ?? []))
          {
            ...(c as Map).cast<String, dynamic>(),
            'kind': 'command',
            if (c['id'] == 'stop') 'available': chat.busy && !chat.stopping,
            if (c['id'] == 'resume-queue')
              'available': chat.inputsPaused && !chat.busy,
          },
      for (final s in (chat.commandCatalogue['skills'] as List? ?? []))
        {
          ...(s as Map).cast<String, dynamic>(),
          'kind': 'skill',
          'id': s['selection']['name'],
          'title': s['selection']['name'],
          'available': true,
        },
    ];
    return all
        .where(
          (e) => '${e['id']} ${e['title']} ${e['description']}'
              .toLowerCase()
              .contains(filter),
        )
        .toList();
  }

  KeyEventResult handleKey(KeyEvent event) {
    final range = query;
    if (range == null ||
        !widget.input.value.composing.isCollapsed ||
        HardwareKeyboard.instance.isControlPressed ||
        HardwareKeyboard.instance.isMetaPressed ||
        HardwareKeyboard.instance.isShiftPressed ||
        HardwareKeyboard.instance.isAltPressed) {
      return KeyEventResult.ignored;
    }
    final key = event.logicalKey;
    if (key == LogicalKeyboardKey.escape) {
      setState(() {
        dismissed = widget.input.text;
      });
      return KeyEventResult.handled;
    }
    final list = entries(range);
    if (key == LogicalKeyboardKey.arrowDown ||
        key == LogicalKeyboardKey.arrowUp) {
      if (list.isNotEmpty) {
        setState(() {
          selected =
              (selected + (key == LogicalKeyboardKey.arrowDown ? 1 : -1)) %
              list.length;
        });
        if (scroll.hasClients) {
          scroll.jumpTo(
            (selected * rowExtent).clamp(0, scroll.position.maxScrollExtent),
          );
        }
      }
      return KeyEventResult.handled;
    }
    if (key == LogicalKeyboardKey.enter || key == LogicalKeyboardKey.tab) {
      if (!loading && list.isNotEmpty) {
        unawaited(choose(list[selected.clamp(0, list.length - 1)], range));
      }
      return KeyEventResult.handled;
    }
    return KeyEventResult.ignored;
  }

  Future<void> choose(Map<String, dynamic> entry, TextRange range) async {
    final chat = widget.chat;
    if (entry['available'] != true || chat.skillSaving) return;
    final value = widget.input.value;
    if (entry['kind'] == 'reference') {
      if (!await showContextPicker(context, chat, entry['id'] as String)) {
        return;
      }
    } else if (entry['kind'] == 'skill') {
      final reference = (entry['selection'] as Map).cast<String, dynamic>();
      final next = [
        ...chat.selectedSkills.where(
          (s) =>
              !(s['name'] == reference['name'] &&
                  s['scope'] == reference['scope']),
        ),
        reference,
      ];
      if (!await chat.selectSkills(next)) return;
    } else {
      switch (entry['id']) {
        case 'stop':
          await chat.stop();
        case 'resume-queue':
          await chat.resumeInputs();
        case 'goal':
          await showGoal(context, chat);
        case 'plan':
          await showPlan(context, chat);
        default:
          return; // Only fixed host handlers can execute.
      }
      if (chat.error != null) return;
    }
    if (!mounted) return;
    // Never erase typing that arrived while a selection was being saved.
    if (widget.input.value == value) {
      // Keep the completed reference in prose; line-start selection stays a chip.
      final inlineSkill =
          entry['kind'] == 'skill' && !_atLineStart(value.text, range);
      final replacement = inlineSkill
          ? '/${entry['selection']['name']}${range.end == value.text.length ? ' ' : ''}'
          : '';
      final text = value.text.replaceRange(range.start, range.end, replacement);
      chat.draft = text;
      widget.input.edit(
        TextEditingValue(
          text: text,
          selection: TextSelection.collapsed(
            offset: range.start + replacement.length,
          ),
        ),
      );
      chat.invalidateContextPreview();
    }
    setState(() {
      dismissed = widget.input.text;
    });
    widget.focus.requestFocus();
  }

  Future<void> inspect(Map<String, dynamic> reference) async {
    final skill = (widget.chat.commandCatalogue['skills'] as List? ?? [])
        .where(
          (s) => [
            'scope',
            'name',
            'revision',
            'version',
            'packageId',
          ].every((k) => s['selection'][k] == reference[k]),
        )
        .firstOrNull;
    await showDialog<void>(
      context: context,
      builder: (context) => AlertDialog(
        title: Text(
          '${reference['name']} · ${reference['scope']}${reference['file'] == true ? '' : ' v${reference['version']}'}',
        ),
        content: SizedBox(
          width: 560,
          child: SingleChildScrollView(
            child: SelectableText(
              skill == null
                  ? 'This selection has changed or is unavailable. Remove it, refresh commands and select it again. Your draft remains.'
                  : '${skill['description']}\n\n${skill['text']}',
            ),
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
  }

  @override
  Widget build(BuildContext context) {
    final range = query;
    final list = range == null ? <Map<String, dynamic>>[] : entries(range);
    final theme = Theme.of(context);
    return Column(
      mainAxisSize: MainAxisSize.min,
      crossAxisAlignment: CrossAxisAlignment.stretch,
      children: [
        if (widget.chat.selectedSkills.isNotEmpty)
          Padding(
            padding: const EdgeInsets.symmetric(vertical: 4),
            child: Wrap(
              spacing: 8,
              runSpacing: 4,
              children: [
                for (final s in widget.chat.selectedSkills)
                  InputChip(
                    key: Key('selected-skill-${s['scope']}-${s['name']}'),
                    label: ConstrainedBox(
                      constraints: BoxConstraints(
                        maxWidth:
                            MediaQuery.sizeOf(context).width.clamp(120, 500) -
                            100,
                      ),
                      child: Text(
                        '${s['name']} · ${s['scope']}${s['file'] == true ? '' : ' v${s['version']}'}',
                        overflow: TextOverflow.ellipsis,
                      ),
                    ),
                    tooltip: 'View skill',
                    onPressed: () => inspect(s),
                    onDeleted: widget.chat.skillSaving
                        ? null
                        : () => widget.chat.selectSkills(
                            widget.chat.selectedSkills
                                .where((p) => !identical(p, s))
                                .toList(),
                          ),
                    deleteButtonTooltipMessage: 'Remove ${s['name']}',
                  ),
              ],
            ),
          ),
        if (range != null)
          Padding(
            padding: const EdgeInsets.only(bottom: 10),
            child: Material(
              key: const Key('command-picker'),
              color: theme.colorScheme.surfaceContainerLow,
              shape: RoundedRectangleBorder(
                borderRadius: BorderRadius.circular(12),
                side: BorderSide(color: theme.colorScheme.outlineVariant),
              ),
              clipBehavior: Clip.antiAlias,
              child: Padding(
                padding: const EdgeInsets.all(6),
                child: Column(
                  mainAxisSize: MainAxisSize.min,
                  children: [
                    Padding(
                      padding: const EdgeInsets.only(left: 10, right: 2),
                      child: Row(
                        children: [
                          Expanded(
                            child: Text(
                              widget.input.text[range.start] == '@'
                                  ? 'Share a context snapshot'
                                  : _atLineStart(widget.input.text, range)
                                  ? 'Commands and skills'
                                  : 'Skills',
                              style: theme.textTheme.labelSmall?.copyWith(
                                color: theme.colorScheme.onSurfaceVariant,
                              ),
                            ),
                          ),
                          IconButton(
                            tooltip: 'Refresh commands',
                            constraints: const BoxConstraints.tightFor(
                              width: 32,
                              height: 32,
                            ),
                            padding: EdgeInsets.zero,
                            onPressed: loading ? null : refresh,
                            icon: const Icon(Icons.refresh, size: 18),
                          ),
                        ],
                      ),
                    ),
                    if (loading) const LinearProgressIndicator(),
                    if (failure != null)
                      Padding(
                        padding: const EdgeInsets.all(12),
                        child: Text(failure!),
                      ),
                    if (!loading && failure == null && list.isEmpty)
                      const Padding(
                        padding: EdgeInsets.all(12),
                        child: Text('No matches. Escape keeps this as text.'),
                      ),
                    if (list.isNotEmpty)
                      SizedBox(
                        height: (list.length * rowExtent).clamp(
                          0,
                          rowExtent * 3,
                        ),
                        child: ListView.builder(
                          controller: scroll,
                          padding: EdgeInsets.zero,
                          itemExtent: rowExtent,
                          itemCount: list.length,
                          itemBuilder: (context, i) {
                            final e = list[i];
                            final enabled =
                                e['available'] == true &&
                                !widget.chat.skillSaving;
                            final detail = e['kind'] == 'skill'
                                ? "Skill · ${e['selection']['scope']}${e['selection']['file'] == true ? '' : ' v${e['selection']['version']}'} · ${e['description']}"
                                : enabled
                                ? e['description']
                                : e['reason'];
                            return Semantics(
                              selected: selected == i,
                              enabled: enabled,
                              child: Padding(
                                padding: const EdgeInsets.symmetric(
                                  vertical: 2,
                                ),
                                child: Material(
                                  key: Key("command-${e['kind']}-${e['id']}"),
                                  color: selected == i
                                      ? theme
                                            .colorScheme
                                            .surfaceContainerHighest
                                      : Colors.transparent,
                                  borderRadius: BorderRadius.circular(8),
                                  child: Tooltip(
                                    message: '${e['title']}\n$detail',
                                    child: InkWell(
                                      borderRadius: BorderRadius.circular(8),
                                      onTap: enabled
                                          ? () => choose(e, range)
                                          : null,
                                      child: Padding(
                                        padding: const EdgeInsets.symmetric(
                                          horizontal: 12,
                                          vertical: 6,
                                        ),
                                        child: Row(
                                          children: [
                                            Icon(
                                              e['kind'] == 'skill'
                                                  ? Icons.auto_stories_outlined
                                                  : Icons.keyboard_command_key,
                                              size: 18,
                                              color: enabled
                                                  ? theme
                                                        .colorScheme
                                                        .onSurfaceVariant
                                                  : theme.disabledColor,
                                            ),
                                            const SizedBox(width: 12),
                                            Expanded(
                                              child: Column(
                                                mainAxisAlignment:
                                                    MainAxisAlignment.center,
                                                crossAxisAlignment:
                                                    CrossAxisAlignment.start,
                                                children: [
                                                  Text(
                                                    e['kind'] == 'command'
                                                        ? '/${e['id']}'
                                                        : '${e['title']}',
                                                    style: theme
                                                        .textTheme
                                                        .bodyMedium
                                                        ?.copyWith(
                                                          fontSize: 14,
                                                          height: 1.4,
                                                          color: enabled
                                                              ? theme
                                                                    .colorScheme
                                                                    .onSurface
                                                              : theme
                                                                    .disabledColor,
                                                        ),
                                                    maxLines: 1,
                                                    overflow:
                                                        TextOverflow.ellipsis,
                                                  ),
                                                  Text(
                                                    '$detail',
                                                    style: theme
                                                        .textTheme
                                                        .bodySmall
                                                        ?.copyWith(
                                                          fontSize: 12,
                                                          height: 1.4,
                                                          color: enabled
                                                              ? theme
                                                                    .colorScheme
                                                                    .onSurfaceVariant
                                                              : theme
                                                                    .disabledColor,
                                                        ),
                                                    maxLines: 1,
                                                    overflow:
                                                        TextOverflow.ellipsis,
                                                  ),
                                                ],
                                              ),
                                            ),
                                          ],
                                        ),
                                      ),
                                    ),
                                  ),
                                ),
                              ),
                            );
                          },
                        ),
                      ),
                  ],
                ),
              ),
            ),
          ),
      ],
    );
  }

  @override
  void dispose() {
    widget.input.removeListener(changed);
    scroll.dispose();
    super.dispose();
  }
}
