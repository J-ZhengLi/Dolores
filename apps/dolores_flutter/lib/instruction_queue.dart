import 'package:flutter/material.dart';

import 'chat.dart';

class InstructionQueue extends StatelessWidget {
  const InstructionQueue({super.key, required this.chat});
  final ChatController chat;

  @override
  Widget build(BuildContext context) {
    final entries = chat.inputEntries;
    if (entries.isEmpty) return const SizedBox.shrink();
    final pending = entries.where((v) => v['state'] == 'pending').toList();
    final visible = [
      ...entries.where(
        (v) =>
            v['state'] == 'pending' ||
            v['state'] == 'delivering' ||
            (v['state'] == 'uncertain' && v['inspected'] != true),
      ),
      ...entries
          .where(
            (v) =>
                v['state'] != 'pending' &&
                v['state'] != 'delivering' &&
                (v['state'] != 'uncertain' || v['inspected'] == true),
          )
          .toList()
          .reversed
          .take(5),
    ];
    final panel = Padding(
      padding: const EdgeInsets.only(bottom: 8),
      child: Column(
        mainAxisSize: MainAxisSize.min,
        crossAxisAlignment: CrossAxisAlignment.stretch,
        children: [
          Row(
            children: [
              Expanded(
                child: Text(
                  'Instructions · ${pending.length} pending',
                  style: Theme.of(context).textTheme.labelLarge,
                ),
              ),
              if (chat.inputsPaused)
                TextButton(
                  onPressed: chat.busy || chat.inputSaving
                      ? null
                      : chat.resumeInputs,
                  child: const Text('Resume queue'),
                ),
              IconButton(
                tooltip: 'Refresh instruction receipts',
                onPressed: () async {
                  try {
                    await chat.refreshInputs();
                  } catch (error) {
                    chat.reportLocalError(
                      'Could not refresh instruction receipts: $error. Try Refresh again.',
                    );
                  }
                },
                icon: const Icon(Icons.refresh, size: 18),
              ),
            ],
          ),
          if (chat.inputsPaused)
            const Text(
              'Paused. Inspect retained instructions before resuming.',
            ),
          ConstrainedBox(
            constraints: const BoxConstraints(maxHeight: 160),
            child: ListView(
              shrinkWrap: true,
              children: [
                for (final entry in visible)
                  ListTile(
                    dense: true,
                    contentPadding: EdgeInsets.zero,
                    title: Text(
                      entry['text'] as String,
                      maxLines: 2,
                      overflow: TextOverflow.ellipsis,
                    ),
                    subtitle: Text(
                      '${entry['mode'] == 'steer' ? 'Steer' : 'Queue'} · ${entry['state']}${entry['state'] == 'applied' ? ' to request; inspect run for outcome' : ''}',
                    ),
                    trailing: PopupMenuButton<String>(
                      tooltip: 'Instruction actions',
                      enabled: !chat.inputSaving,
                      onSelected: (action) async {
                        if (action == 'inspect') {
                          await showDialog<void>(
                            context: context,
                            builder: (_) => AlertDialog(
                              title: const Text('Instruction receipt'),
                              content: SingleChildScrollView(
                                child: SelectableText(
                                  '${entry['text']}\n\nState: ${entry['state']}\nRun: ${entry['run'] ?? 'Not delivered'}\nSkills: ${(entry['selectedSkills'] as List? ?? []).map((s) => "${s['name']} · ${s['scope']} v${s['version']} (revision ${s['revision']})").join(', ')}',
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
                        } else if (action == 'edit') {
                          await showDialog<void>(
                            context: context,
                            builder: (_) =>
                                _EditInstruction(chat: chat, entry: entry),
                          );
                        } else if (action == 'cancel') {
                          await chat.changeInput({
                            'action': 'cancel',
                            'id': entry['id'],
                            'revision': entry['revision'],
                          });
                        } else if (action == 'acknowledge') {
                          await chat.changeInput({
                            'action': 'acknowledge',
                            'id': entry['id'],
                            'revision': entry['revision'],
                          });
                        } else {
                          final ids = pending
                              .map((v) => v['id'] as String)
                              .toList();
                          final index = ids.indexOf(entry['id'] as String);
                          final target = action == 'up' ? index - 1 : index + 1;
                          if (index >= 0 &&
                              target >= 0 &&
                              target < ids.length) {
                            final swap = ids[target];
                            ids[target] = ids[index];
                            ids[index] = swap;
                            await chat.changeInput({
                              'action': 'reorder',
                              'revision': chat.taskInputs['revision'],
                              'ids': ids,
                            });
                          }
                        }
                      },
                      itemBuilder: (_) => [
                        const PopupMenuItem(
                          value: 'inspect',
                          child: Text('Inspect'),
                        ),
                        if (entry['state'] == 'uncertain' &&
                            entry['inspected'] != true)
                          const PopupMenuItem(
                            value: 'acknowledge',
                            child: Text(
                              'Acknowledge uncertainty; do not retry',
                            ),
                          ),
                        if (entry['state'] == 'pending') ...[
                          const PopupMenuItem(
                            value: 'edit',
                            child: Text('Edit'),
                          ),
                          const PopupMenuItem(
                            value: 'up',
                            child: Text('Move up'),
                          ),
                          const PopupMenuItem(
                            value: 'down',
                            child: Text('Move down'),
                          ),
                          const PopupMenuItem(
                            value: 'cancel',
                            child: Text('Cancel instruction'),
                          ),
                        ],
                      ],
                    ),
                  ),
              ],
            ),
          ),
        ],
      ),
    );
    final unresolved = entries.any(
      (v) =>
          v['state'] == 'pending' ||
          v['state'] == 'delivering' ||
          (v['state'] == 'uncertain' && v['inspected'] != true),
    );
    if (unresolved || chat.inputsPaused) return panel;
    return Material(
      type: MaterialType.transparency,
      child: ExpansionTile(
        title: const Text('Instruction receipts'),
        children: [panel],
      ),
    );
  }
}

class _EditInstruction extends StatefulWidget {
  const _EditInstruction({required this.chat, required this.entry});
  final ChatController chat;
  final Map<String, dynamic> entry;
  @override
  State<_EditInstruction> createState() => _EditInstructionState();
}

class _EditInstructionState extends State<_EditInstruction> {
  late final text = TextEditingController(text: widget.entry['text'] as String);
  bool saving = false;
  String? error;
  late int revision = widget.entry['revision'] as int;
  String? latest;
  @override
  void dispose() {
    text.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) => PopScope(
    canPop: !saving,
    child: AlertDialog(
      title: const Text('Edit pending instruction'),
      content: SizedBox(
        width: 480,
        child: Column(
          mainAxisSize: MainAxisSize.min,
          children: [
            TextField(
              controller: text,
              autofocus: true,
              minLines: 2,
              maxLines: 8,
              readOnly: saving,
            ),
            if (error != null) Text(error!),
            if (latest != null)
              SelectableText('Latest saved instruction:\n$latest'),
          ],
        ),
      ),
      actions: [
        if (widget.chat.busy)
          TextButton(
            onPressed: widget.chat.stop,
            child: const Text('Stop task'),
          ),
        if (error != null)
          TextButton(
            onPressed: saving
                ? null
                : () async {
                    try {
                      await widget.chat.refreshInputs();
                      if (!context.mounted) return;
                      final current = widget.chat.inputEntries
                          .where((e) => e['id'] == widget.entry['id'])
                          .firstOrNull;
                      if (current == null || current['state'] != 'pending') {
                        setState(
                          () => error = 'This instruction is no longer pending. Your edited text is kept here to copy.',
                        );
                      } else {
                        setState(() {
                          revision = current['revision'] as int;
                          latest = current['text'] as String;
                          error = 'Review the latest saved text below, then Save your edits.';
                        });
                      }
                    } catch (failure) {
                      if (context.mounted) {
                        setState(() => error = failure.toString());
                      }
                    }
                  },
            child: const Text('Compare latest'),
          ),
        TextButton(
          onPressed: saving ? null : () => Navigator.pop(context),
          child: const Text('Keep original'),
        ),
        FilledButton(
          onPressed: saving
              ? null
              : () async {
                  setState(() => saving = true);
                  await widget.chat.changeInput({
                    'action': 'edit',
                    'id': widget.entry['id'],
                    'revision': revision,
                    'text': text.text,
                  });
                  if (!context.mounted) return;
                  if (widget.chat.error == null) {
                    Navigator.pop(context);
                  } else {
                    setState(() {
                      saving = false;
                      error = widget.chat.error;
                    });
                  }
                },
          child: Text(saving ? 'Saving…' : 'Save'),
        ),
      ],
    ),
  );
}
