import 'package:flutter/material.dart';

import 'chat.dart';
import 'inspector.dart';

Future<void> showRunHistory(BuildContext context, ChatController chat) =>
    showDialog<void>(
      context: context,
      builder: (_) => RunHistoryInspector(chat: chat),
    );

class RunHistoryInspector extends StatefulWidget {
  final ChatController chat;
  const RunHistoryInspector({super.key, required this.chat});
  @override
  State<RunHistoryInspector> createState() => _RunHistoryInspectorState();
}

class _RunHistoryInspectorState extends State<RunHistoryInspector> {
  late final String? session = widget.chat.session;
  List<dynamic> runs = [], events = [];
  String? selected, error;
  Map<String, dynamic>? checkpoint;
  bool pending = false;
  @override
  void initState() {
    super.initState();
    refresh();
  }

  Future<void> refresh() async {
    setState(() {
      pending = true;
      checkpoint = null;
      error = null;
    });
    try {
      final result = await widget.chat.bridge.call({
        'command': 'runs',
        'session': session,
      });
      if (mounted) {
        setState(() {
          runs = result as List;
          events = [];
          selected = null;
        });
      }
    } catch (failure) {
      if (mounted) {
        setState(() {
          error = failure.toString();
        });
      }
    } finally {
      if (mounted) {
        setState(() {
          pending = false;
        });
      }
    }
  }

  Future<void> select(String id) async {
    setState(() {
      pending = true;
      checkpoint = null;
      error = null;
      selected = id;
      events = [];
    });
    try {
      final result = await widget.chat.bridge.call({
        'command': 'runEvents',
        'session': session,
        'runId': id,
      });
      final saved = widget.chat.durableDrafts
          ? await widget.chat.bridge.call({
              'command': 'runCheckpoint',
              'session': session,
              'runId': id,
            }) as Map
          : null;
      if (mounted) {
        setState(() {
          checkpoint = saved?.cast<String, dynamic>();
          events = result as List;
        });
      }
    } catch (failure) {
      if (mounted) {
        setState(() {
          error = failure.toString();
        });
      }
    } finally {
      if (mounted) {
        setState(() {
          pending = false;
        });
      }
    }
  }

  @override
  Widget build(BuildContext context) => InspectorFrame(
    title: 'Earlier tasks',
    subtitle: 'Local execution evidence · latest 20 runs',
    child: Column(
      children: [
        Expanded(
          child: SingleChildScrollView(
            padding: const EdgeInsets.all(20),
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                const Text(
                  'After interruption, inspect Changes and external effects before retrying. Operations are never replayed automatically. Older chats have no invented run records.',
                ),
                const SizedBox(height: 12),
                if (error != null) SelectableText(error!),
                if (!pending && runs.isEmpty)
                  const Text('No recorded runs yet.'),
                for (final run in runs)
                  ListTile(
                    contentPadding: EdgeInsets.zero,
                    selected: selected == run['id'],
                    title: Text('${run['state']} · ${run['model']}'),
                    subtitle: Text(
                      '${run['settings']['maxOutputTokens']} output tokens · ${run['sequence']} events',
                    ),
                    onTap: pending ? null : () => select(run['id'] as String),
                  ),
                if (selected != null) const Divider(),
                if (selected != null)
                  ExpansionTile(
                    title: const Text('Run details'),
                    children: [
                      for (final run in runs.where((r) => r['id'] == selected))
                        SelectableText('$run'),
                    ],
                  ),
                if (checkpoint != null) ...[
                  SelectableText(
                    'Goal: ${checkpoint!['goal']}\nProposed plan (unverified): ${checkpoint!['proposedPlan']}\nUncertain effects: ${checkpoint!['uncertainEffects']}\nPause/end: ${checkpoint!['pauseReason']}\n${checkpoint!['note']}',
                  ),
                  if (checkpoint!['savedDraft'] != '')
                    SelectableText('Saved draft: ${checkpoint!['savedDraft']}'),
                  TextButton(
                    onPressed:
                        pending ||
                            widget.chat.busy ||
                            checkpoint!['run']['state'] == 'completed' ||
                            ![
                              'paused',
                              'failed',
                              'cancelled',
                              'interrupted',
                            ].contains(checkpoint!['run']['state'])
                        ? null
                        : () async {
                            try {
                              await widget.chat.prepareCheckpoint(selected!);
                              if (context.mounted) Navigator.pop(context);
                            } catch (failure) {
                              if (mounted) {
                                setState(() => error = failure.toString());
                              }
                            }
                          },
                    child: const Text('Prepare resume draft'),
                  ),
                ],
                for (final event in events)
                  Padding(
                    padding: const EdgeInsets.symmetric(vertical: 6),
                    child: ExpansionTile(
                      title: Text(
                        '${event['sequence']}. ${event['kind']} · ${event['state']}',
                      ),
                      children: [SelectableText('${event['data']}')],
                    ),
                  ),
                if (pending) const LinearProgressIndicator(),
              ],
            ),
          ),
        ),
        Padding(
          padding: const EdgeInsets.all(12),
          child: TextButton(
            onPressed: pending ? null : refresh,
            child: const Text('Refresh'),
          ),
        ),
      ],
    ),
  );
}
