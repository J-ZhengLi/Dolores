import 'form_field_layout.dart';
import 'dart:convert';

import 'package:flutter/material.dart';

import 'chat.dart';

Future<void> showGoal(BuildContext context, ChatController chat) async {
  await chat.refreshGoal();
  if (!context.mounted) return;
  await showDialog<void>(
    context: context,
    builder: (_) => _GoalEditor(chat: chat),
  );
}

class GoalPanel extends StatelessWidget {
  final ChatController chat;
  const GoalPanel({super.key, required this.chat});
  @override
  Widget build(BuildContext context) {
    final goal = chat.goal;
    if (goal == null || goal['state'] == 'cleared') {
      return const SizedBox.shrink();
    }
    final steps = (goal['steps'] as List? ?? []);
    return ListTile(
      contentPadding: EdgeInsets.zero,
      dense: true,
      leading: const Icon(Icons.flag_outlined, size: 18),
      title: Text(
        '${goal['objective']}',
        maxLines: 1,
        overflow: TextOverflow.ellipsis,
      ),
      subtitle: Text(
        "${goal['state']} · ${steps.where((s) => s['done'] == true).length}/${steps.length} criteria",
        maxLines: 1,
      ),
      trailing: IconButton(
        tooltip: 'Inspect goal and progress',
        onPressed: () => showGoal(context, chat),
        icon: const Icon(Icons.more_horiz),
      ),
    );
  }
}

class _GoalEditor extends StatefulWidget {
  final ChatController chat;
  const _GoalEditor({required this.chat});
  @override
  State<_GoalEditor> createState() => _GoalEditorState();
}

class _GoalEditorState extends State<_GoalEditor> {
  late final objective = TextEditingController(
    text: widget.chat.goal?['objective'] as String? ?? '',
  );
  late final criteria = TextEditingController(
    text: ((widget.chat.goal?['steps'] as List?) ?? [])
        .map((s) => s['criterion'])
        .join('\n'),
  );
  late final constraints = TextEditingController(
    text: widget.chat.goal?['constraints'] as String? ?? '',
  );
  bool saving = false;
  String? failure;
  ChatController get chat => widget.chat;
  @override
  void initState() {
    super.initState();
    chat.addListener(changed);
  }

  void changed() {
    if (mounted) setState(() {});
  }

  Future<void> action(Map<String, dynamic> edit) async {
    setState(() {
      saving = true;
      failure = null;
    });
    final ok = await chat.changeGoal(edit);
    if (mounted) {
      setState(() {
        saving = false;
        failure = ok ? null : chat.error;
      });
    }
  }

  @override
  Widget build(BuildContext context) {
    final labelStyle = TextStyle(
      fontFamily: Theme.of(context).textTheme.bodyMedium?.fontFamily,
    );
    final g = chat.goal;
    final existing =
        g != null && !['completed', 'cleared'].contains(g['state']);
    final used = g?['used'] as Map?;
    final limits = g?['limits'] as Map?;
    return PopScope(
      canPop: !saving,
      child: AlertDialog(
        title: const Text('Goal and progress'),
        content: SizedBox(
          width: 560,
          child: SingleChildScrollView(
            child: Column(
              mainAxisSize: MainAxisSize.min,
              crossAxisAlignment: CrossAxisAlignment.stretch,
              children: [
                if (!existing) ...[
                  FieldLabel(
                    label: 'Objective',
                    child: TextField(
                      key: const Key('goal-objective'),
                      controller: objective,
                      readOnly: saving,
                      minLines: 1,
                      maxLines: 4,
                      decoration: const InputDecoration(),
                    ),
                  ),
                  const SizedBox(height: 12),
                  FieldLabel(
                    label: 'Success criteria',
                    child: TextField(
                      key: const Key('goal-criteria'),
                      controller: criteria,
                      readOnly: saving,
                      minLines: 2,
                      maxLines: 8,
                      decoration: const InputDecoration(
                        hintText: 'One verifiable criterion per line',
                      ),
                    ),
                  ),
                  const SizedBox(height: 12),
                  FieldLabel(
                    label: 'Constraints',
                    child: TextField(
                      key: const Key('goal-constraints'),
                      controller: constraints,
                      readOnly: saving,
                      minLines: 1,
                      maxLines: 4,
                      decoration: const InputDecoration(),
                    ),
                  ),
                  const SizedBox(height: 12),
                  const Text(
                    'Saving creates a goal. Start explicitly to work toward it. Task limits apply cumulatively across continuations and compaction.',
                  ),
                ],
                if (g != null) ...[
                  const SizedBox(height: 12),
                  SelectableText(
                    '${g['objective']}\n${g['state']} · ${g['note']}',
                  ),
                  if ('${g['constraints']}'.isNotEmpty)
                    SelectableText('Constraints: ${g['constraints']}'),
                  for (final s in (g['steps'] as List? ?? []))
                    ListTile(
                      contentPadding: EdgeInsets.zero,
                      dense: true,
                      leading: Icon(
                        s['done'] == true
                            ? Icons.check_circle_outline
                            : Icons.radio_button_unchecked,
                      ),
                      title: Text('${s['criterion']}'),
                      subtitle: SelectableText(
                        'Evidence: ${jsonEncode(s['evidence'])}',
                      ),
                    ),
                  SelectableText(
                    'Cumulative attempts: ${used?['modelCalls']}/${limits?['modelCalls']} model calls · ${used?['toolCalls']}/${limits?['toolCalls']} tools · ${used?['segments']}/${limits?['segments']} segments · ${used?['activeSeconds']} active seconds',
                  ),
                  ExpansionTile(
                    title: const Text('Saved goal details'),
                    children: [
                      SelectableText(
                        const JsonEncoder.withIndent('  ').convert(g),
                      ),
                    ],
                  ),
                ],
                if (failure != null) SelectableText(failure!),
              ],
            ),
          ),
        ),
        actions: [
          if (chat.busy)
            TextButton(
              onPressed: chat.stopping ? null : chat.stop,
              child: Text('Stop task', style: labelStyle),
            ),
          TextButton(
            onPressed: saving ? null : () => Navigator.pop(context),
            child: Text('Close', style: labelStyle),
          ),
          if (!existing)
            FilledButton(
              key: const Key('save-goal'),
              onPressed: saving || chat.busy
                  ? null
                  : () => action({
                      'action': 'set',
                      'revision': g?['revision'],
                      'objective': objective.text,
                      'criteria': criteria.text
                          .split('\n')
                          .map((s) => s.trim())
                          .where((s) => s.isNotEmpty)
                          .toList(),
                      'constraints': constraints.text,
                    }),
              child: Text(saving ? 'Saving…' : 'Save goal', style: labelStyle),
            ),
          if (existing) ...[
            TextButton(
              onPressed: saving || chat.busy
                  ? null
                  : () =>
                        action({'action': 'clear', 'revision': g['revision']}),
              child: Text('Clear goal', style: labelStyle),
            ),
            if (g['state'] == 'active') ...[
              TextButton(
                onPressed: saving || chat.busy
                    ? null
                    : () => action({
                        'action': 'pause',
                        'revision': g['revision'],
                      }),
                child: Text('Pause goal', style: labelStyle),
              ),
              FilledButton(
                key: const Key('start-goal'),
                onPressed: saving || chat.busy
                    ? null
                    : () async {
                        Navigator.pop(context);
                        await chat.startGoal();
                      },
                child: Text('Start goal', style: labelStyle),
              ),
            ] else
              TextButton(
                key: const Key('resume-goal'),
                onPressed: saving || chat.busy
                    ? null
                    : () => action({
                        'action': 'resume',
                        'revision': g['revision'],
                      }),
                child: Text('Resume goal', style: labelStyle),
              ),
          ],
        ],
      ),
    );
  }

  @override
  void dispose() {
    chat.removeListener(changed);
    objective.dispose();
    criteria.dispose();
    constraints.dispose();
    super.dispose();
  }
}
