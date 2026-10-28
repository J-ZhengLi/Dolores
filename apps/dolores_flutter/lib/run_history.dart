import 'help_tip.dart';

import 'dart:convert';

import 'package:flutter/material.dart';

import 'chat.dart';
import 'inspector.dart';
import 'command_sessions.dart';
import 'agent_timeline.dart';

Future<void> showRunHistory(
  BuildContext context,
  ChatController chat, {
  String? run,
}) => showDialog<void>(
  context: context,
  builder: (_) => RunHistoryInspector(chat: chat, initialRun: run),
);

class RunHistoryInspector extends StatefulWidget {
  final ChatController chat;
  final String? initialRun;
  const RunHistoryInspector({super.key, required this.chat, this.initialRun});
  @override
  State<RunHistoryInspector> createState() => _RunHistoryInspectorState();
}

class _RunHistoryInspectorState extends State<RunHistoryInspector> {
  late final String? session = widget.chat.session;
  List<dynamic> runs = [], events = [];
  String? selected, error;
  Map<String, dynamic>? checkpoint;
  Map<String, dynamic>? progress;
  String? progressError, checkpointError;
  bool pending = false;
  int pageAfter = 0;
  int? nextPage;
  final previousPages = <int>[];
  @override
  void initState() {
    super.initState();
    refresh();
  }

  Future<void> refresh() async {
    setState(() {
      pending = true;
      checkpoint = null;
      progress = null;
      progressError = null;
      checkpointError = null;
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
          pageAfter = 0;
          nextPage = null;
          previousPages.clear();
        });
        if (widget.initialRun != null) await select(widget.initialRun!);
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

  Future<void> select(String id, {int after = 0, bool paging = false}) async {
    setState(() {
      pending = true;
      if (!paging) checkpoint = null;
      if (!paging) {
        progress = null;
        progressError = null;
        checkpointError = null;
      }
      error = null;
      selected = id;
      if (!paging) {
        events = [];
        previousPages.clear();
      }
    });
    try {
      final result = await widget.chat.bridge.call({
        'command': widget.chat.taskSegmentsAvailable
            ? 'runEventsPage'
            : 'runEvents',
        'session': session,
        'runId': id,
        if (widget.chat.taskSegmentsAvailable) 'after': after,
      });
      if (mounted) {
        setState(() {
          if (widget.chat.taskSegmentsAvailable) {
            final page = result as Map;
            events = page['items'] as List;
            nextPage = page['next'] as int?;
          } else {
            events = result as List;
            nextPage = null;
          }
          pageAfter = after;
        });
      }
      // Optional recovery metadata must never hide a readable evidence page.
      if (!paging && widget.chat.durableDrafts) {
        try {
          final saved = await widget.chat.bridge.call({
            'command': 'runCheckpoint',
            'session': session,
            'runId': id,
          }) as Map;
          if (mounted) {
            setState(() => checkpoint = saved.cast<String, dynamic>());
          }
        } catch (failure) {
          if (mounted) {
            setState(
              () => checkpointError = 'Recovery details unavailable: $failure',
            );
          }
        }
      }
      if (!paging && widget.chat.taskSegmentsAvailable) await loadProgress(1);
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

  Future<void> loadProgress(int ordinal) async {
    try {
      final saved = await widget.chat.bridge.call({
        'command': 'runProgress',
        'session': session,
        'runId': selected,
        'ordinal': ordinal,
      }) as Map;
      if (mounted) {
        setState(() {
          progress = saved.cast<String, dynamic>();
          progressError = null;
        });
      }
    } catch (failure) {
      if (mounted) {
        setState(
          () => progressError =
              'Readable progress unavailable: $failure. Raw evidence remains below.',
        );
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
                const HelpLabel(
                  help: 'After interruption, inspect Changes and external effects before retrying. Operations are never replayed automatically. Older chats have no invented run records.',
                  child: Text(
                    'Check changes and external actions before retrying.',
                  ),
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
                      '${run['settings']['maxOutputTokens'] == null ? 'Provider output limit' : 'Output limit: ${run['settings']['maxOutputTokens']} tokens'} · ${run['sequence']} events',
                    ),
                    onTap: pending ? null : () => select(run['id'] as String),
                  ),
                if (selected != null) const Divider(),
                if (progressError != null) SelectableText(progressError!),
                if (checkpointError != null) SelectableText(checkpointError!),
                if (progress?['summary'] is Map) ...[
                  Text(
                    'Progress page ${progress!['ordinal']} of ${progress!['total']}',
                  ),
                  const Text(
                    'Saved outputs and tool results. Opening history makes no model request.',
                  ),
                  AgentTimeline(
                    steps: progress!['summary']['steps'] as List? ?? [],
                    records: progress!['summary']['tools'] as List? ?? [],
                    toolSteps: progress!['summary']['toolSteps'] as List? ?? [],
                    bridge: widget.chat.bridge,
                  ),
                  for (final record
                      in progress!['summary']['tools'] as List? ?? [])
                    if (_hasFullOutput(record))
                      TextButton.icon(
                        onPressed: () => showDialog<void>(
                          context: context,
                          builder: (_) => _ArchivedReceipt(
                            chat: widget.chat,
                            session: session!,
                            run: selected!,
                            call: record['callId'] as String,
                          ),
                        ),
                        icon: const Icon(Icons.article_outlined, size: 16),
                        label: Text(
                          'Full output · ${record['target']}',
                          maxLines: 2,
                          overflow: TextOverflow.ellipsis,
                        ),
                      ),
                  if (progress!['summary']['thinking'] is List)
                    for (final thought in progress!['summary']['thinking'])
                      ExpansionTile(
                        title: Text('Thinking · call ${thought['number']}'),
                        children: [SelectableText('${thought['text']}')],
                      ),
                  Wrap(
                    spacing: 16,
                    children: [
                      TextButton(
                        onPressed: pending || progress!['ordinal'] <= 1
                            ? null
                            : () async {
                                setState(() => pending = true);
                                await loadProgress(
                                  (progress!['ordinal'] as int) - 1,
                                );
                                if (mounted) setState(() => pending = false);
                              },
                        child: const Text('Previous progress'),
                      ),
                      TextButton(
                        onPressed:
                            pending ||
                                progress!['ordinal'] >= progress!['total']
                            ? null
                            : () async {
                                setState(() => pending = true);
                                await loadProgress(
                                  (progress!['ordinal'] as int) + 1,
                                );
                                if (mounted) setState(() => pending = false);
                              },
                        child: const Text('Next progress'),
                      ),
                    ],
                  ),
                ],
                if (selected != null && widget.initialRun != null)
                  SelectableText('Selected run: $selected'),
                if (selected != null && widget.chat.commandSessionsAvailable)
                  TextButton.icon(
                    onPressed: pending
                        ? null
                        : () => showCommandSessions(
                            context,
                            widget.chat,
                            run: selected,
                          ),
                    icon: const Icon(Icons.terminal, size: 18),
                    label: const Text('Commands from this task'),
                  ),
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
                ExpansionTile(
                  title: const Text('Technical events'),
                  initiallyExpanded: progress?['summary'] is! Map,
                  children: [
                    for (final event in events)
                      Padding(
                        padding: const EdgeInsets.symmetric(vertical: 6),
                        child: ExpansionTile(
                          title: Text(
                            '${event['sequence']}. ${event['kind']} · ${event['state']}',
                          ),
                          children: [
                            if (event['kind'] == 'toolResult' &&
                                event['data']['callId'] is String)
                              AgentTimeline(
                                records: [
                                  {
                                    ...event['data'] as Map,
                                    'status': event['data']['returned'] == false
                                        ? 'failed'
                                        : 'read',
                                  },
                                ],
                                bridge: widget.chat.bridge,
                              )
                            else
                              SelectableText('${event['data']}'),
                          ],
                        ),
                      ),
                  ],
                ),
                if (widget.chat.taskSegmentsAvailable && selected != null)
                  Wrap(
                    alignment: WrapAlignment.spaceBetween,
                    spacing: 16,
                    children: [
                      TextButton(
                        onPressed: pending || previousPages.isEmpty
                            ? null
                            : () async {
                                final previous = previousPages.last;
                                await select(
                                  selected!,
                                  after: previous,
                                  paging: true,
                                );
                                if (mounted && error == null) {
                                  setState(() => previousPages.removeLast());
                                }
                              },
                        child: const Text('Previous evidence'),
                      ),
                      TextButton(
                        onPressed: pending || nextPage == null
                            ? null
                            : () async {
                                final previous = pageAfter;
                                await select(
                                  selected!,
                                  after: nextPage!,
                                  paging: true,
                                );
                                if (mounted && error == null) {
                                  setState(() => previousPages.add(previous));
                                }
                              },
                        child: const Text('Next evidence'),
                      ),
                    ],
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

bool _hasFullOutput(dynamic record) {
  try {
    return jsonDecode('${record['content']}')['previewTruncated'] == true &&
        jsonDecode('${record['content']}')['sourceCallId'] is String;
  } catch (_) {
    return false;
  }
}

class _ArchivedReceipt extends StatefulWidget {
  final ChatController chat;
  final String session, run, call;
  const _ArchivedReceipt({
    required this.chat,
    required this.session,
    required this.run,
    required this.call,
  });
  @override
  State<_ArchivedReceipt> createState() => _ArchivedReceiptState();
}

class _ArchivedReceiptState extends State<_ArchivedReceipt> {
  Map? page;
  String? error;
  bool pending = false;
  final previous = <int>[];
  @override
  void initState() {
    super.initState();
    load(0);
  }

  Future<void> load(int offset, {bool back = false}) async {
    setState(() => pending = true);
    try {
      final saved = await widget.chat.bridge.call({
        'command': 'readTaskHistory',
        'session': widget.session,
        'runId': widget.run,
        'callId': widget.call,
        'offset': offset,
      }) as Map;
      if (mounted) {
        setState(() {
          if (back) {
            previous.removeLast();
          } else if (page != null) {
            previous.add(page!['offset'] as int);
          }
          page = saved;
          error = null;
        });
      }
    } catch (failure) {
      if (mounted) setState(() => error = '$failure');
    } finally {
      if (mounted) setState(() => pending = false);
    }
  }

  @override
  Widget build(BuildContext context) => InspectorFrame(
    title: 'Saved output',
    subtitle: 'Original receipt · no action is rerun',
    child: Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        if (error != null) SelectableText(error!),
        if (pending) const LinearProgressIndicator(),
        Expanded(
          child: SingleChildScrollView(
            padding: const EdgeInsets.all(16),
            child: SelectableText('${page?['content'] ?? ''}'),
          ),
        ),
        Wrap(
          spacing: 16,
          children: [
            TextButton(
              onPressed: pending || previous.isEmpty
                  ? null
                  : () => load(previous.last, back: true),
              child: const Text('Previous output'),
            ),
            TextButton(
              onPressed: pending || page?['hasMore'] != true
                  ? null
                  : () => load(page!['nextOffset'] as int),
              child: const Text('Next output'),
            ),
            if (error != null)
              TextButton(
                onPressed: pending
                    ? null
                    : () => load(page?['offset'] as int? ?? 0),
                child: const Text('Retry output'),
              ),
          ],
        ),
      ],
    ),
  );
}
