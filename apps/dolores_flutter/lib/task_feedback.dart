import 'package:flutter/material.dart';

import 'chat.dart';
import 'inspector.dart';
import 'theme.dart';
import 'usage_details.dart';

class TaskFeedbackButton extends StatefulWidget {
  final ChatController chat;
  final Map<String, dynamic> message;
  const TaskFeedbackButton({
    super.key,
    required this.chat,
    required this.message,
  });
  @override
  State<TaskFeedbackButton> createState() => _TaskFeedbackButtonState();
}

class _TaskFeedbackButtonState extends State<TaskFeedbackButton> {
  Map? saved;
  @override
  void initState() {
    super.initState();
    saved = widget.message['feedback'] as Map?;
  }

  @override
  Widget build(BuildContext context) {
    final chat = widget.chat;
    final label = saved?['outcome'] == 'worked'
        ? 'Worked'
        : saved?['outcome'] == 'needsWork'
        ? 'Needs work'
        : 'Task feedback';
    return TextButton.icon(
      key: ValueKey('task-feedback-${widget.message['id']}'),
      icon: const Icon(Icons.rate_review_outlined, size: 15),
      label: Text(label),
      onPressed:
          chat.busy || chat.changing || chat.loading || chat.session == null
          ? null
          : () async {
              await chat.inspectLocalSettings(() async {
                final result = await showDialog<Map<String, dynamic>>(
                  context: context,
                  barrierDismissible: false,
                  builder: (_) => TaskFeedbackDialog(
                    chat: chat,
                    message: {...widget.message, 'feedback': saved},
                  ),
                );
                if (result != null && mounted) {
                  setState(() => saved = result);
                  for (final message in chat.messages) {
                    if (message['id'] == widget.message['id']) {
                      message['feedback'] = result;
                    }
                  }
                }
              });
            },
    );
  }
}

class TaskFeedbackDialog extends StatefulWidget {
  final ChatController chat;
  final Map<String, dynamic> message;
  const TaskFeedbackDialog({
    super.key,
    required this.chat,
    required this.message,
  });
  @override
  State<TaskFeedbackDialog> createState() => _TaskFeedbackDialogState();
}

class _TaskFeedbackDialogState extends State<TaskFeedbackDialog> {
  final note = TextEditingController();
  final scroll = ScrollController();
  String? outcome, error;
  bool saving = false;
  @override
  void initState() {
    super.initState();
    outcome = widget.message['feedback']?['outcome'] as String?;
    note.text = widget.message['feedback']?['note'] as String? ?? '';
  }

  @override
  void dispose() {
    note.dispose();
    scroll.dispose();
    super.dispose();
  }

  Future<void> save({bool clear = false}) async {
    if (saving || (!clear && outcome == null)) return;
    setState(() {
      saving = true;
      error = null;
    });
    try {
      final result = await widget.chat.bridge.call({
        'command': 'saveTaskFeedback',
        'session': widget.chat.session,
        'draft': {
          'messageId': widget.message['id'],
          'expectedContent': widget.message['content'],
          'expectedMetadata': widget.message['metadata'],
          'revision': widget.message['feedback']?['revision'] ?? 0,
          'outcome': clear ? null : outcome,
          'note': clear ? '' : note.text,
        },
      });
      if (mounted) {
        Navigator.pop(context, (result as Map).cast<String, dynamic>());
      }
    } catch (e) {
      if (mounted) {
        setState(() {
          error = e.toString();
          saving = false;
        });
        WidgetsBinding.instance.addPostFrameCallback((_) {
          if (mounted && scroll.hasClients) scroll.jumpTo(0);
        });
      }
    }
  }

  @override
  Widget build(BuildContext context) {
    final p = Palette(Theme.of(context).brightness == Brightness.dark);
    final metadata = (widget.message['metadata'] as Map?)
        ?.cast<String, dynamic>();
    return PopScope(
      canPop: !saving,
      child: InspectorFrame(
        title: 'Task feedback',
        subtitle: 'Local assessment · reply #${widget.message['id']}',
        canClose: !saving,
        child: Column(
          children: [
            Expanded(
              child: ListView(
                controller: scroll,
                padding: const EdgeInsets.all(20),
                children: [
                  if (error != null)
                    Text(
                      error!,
                      key: const Key('feedback-error'),
                      style: TextStyle(color: p.errorText),
                    ),
                  const Text(
                    'Did this task work for you? Feedback stays on this reply and is not sent to the model or used to change memory or skills.',
                  ),
                  const SizedBox(height: 12),
                  const Text(
                    'Your assessment is separate from command results. A reply or an exited-zero command alone does not prove the whole task worked.',
                  ),
                  UsageDetails(metadata: metadata),
                  if (metadata?['paused'] is Map)
                    Text(
                      'This run paused: ${metadata!['paused']['reason']}. Progress remains saved.',
                    ),
                  const SizedBox(height: 12),
                  Wrap(
                    spacing: 8,
                    children: [
                      for (final value in ['worked', 'needsWork'])
                        ChoiceChip(
                          label: Text(
                            value == 'worked' ? 'Worked' : 'Needs work',
                          ),
                          selected: outcome == value,
                          onSelected: saving
                              ? null
                              : (_) => setState(() => outcome = value),
                        ),
                    ],
                  ),
                  const SizedBox(height: 12),
                  TextField(
                    key: const Key('feedback-note'),
                    controller: note,
                    enabled: !saving,
                    minLines: 3,
                    maxLines: 8,
                    maxLength: 1024,
                    decoration: const InputDecoration(
                      labelText: 'Optional note',
                      helperText: 'Keep credentials out of notes. Included in chat exports.',
                    ),
                  ),
                ],
              ),
            ),
            Padding(
              padding: const EdgeInsets.all(12),
              child: Wrap(
                spacing: 8,
                children: [
                  TextButton(
                    onPressed:
                        saving || widget.message['feedback']?['outcome'] == null
                        ? null
                        : () => save(clear: true),
                    child: const Text('Clear feedback'),
                  ),
                  FilledButton(
                    onPressed: saving || outcome == null ? null : save,
                    child: Text(saving ? 'Saving…' : 'Save feedback'),
                  ),
                ],
              ),
            ),
          ],
        ),
      ),
    );
  }
}
