import 'dart:convert';

import 'package:flutter/material.dart';

import 'chat.dart';
import 'inspector.dart';

class LearningInspector extends StatefulWidget {
  final ChatController chat;
  const LearningInspector({super.key, required this.chat});
  @override
  State<LearningInspector> createState() => _LearningState();
}

class _LearningState extends State<LearningInspector> {
  Map? report;
  String? error, progress;
  bool pending = false, stopping = false;
  int? run;
  final command = TextEditingController(text: 'node check.cjs');
  Map get state => (report?['state'] as Map?) ?? {};
  @override
  void initState() {
    super.initState();
    load();
  }

  @override
  void dispose() {
    command.dispose();
    super.dispose();
  }

  Future<dynamic> call(String name, [Map<String, dynamic> args = const {}]) =>
      widget.chat.bridge.call({
        'command': name,
        'session': widget.chat.session,
        ...args,
      });
  Future<void> action(
    String name, [
    Map<String, dynamic> args = const {},
  ]) async {
    setState(() {
      pending = true;
      error = null;
    });
    try {
      final data = await call(name, args);
      widget.chat.invalidateContext();
      if (mounted) setState(() => report = data as Map);
    } catch (e) {
      if (mounted) setState(() => error = e.toString());
    } finally {
      if (mounted) setState(() => pending = false);
    }
  }

  Future<void> load() async {
    if (widget.chat.workspaceRoot != null) await action('learningState');
  }

  Future<void> policy({bool? enabled, bool? automatic, bool? paused}) =>
      action('setLearningPolicy', {
        'revision': state['revision'],
        'enabled': enabled ?? state['enabled'],
        'automatic': automatic ?? state['automatic'],
        'paused': paused ?? state['paused'],
      });
  Future<void> reflect() async {
    setState(() {
      pending = true;
      error = null;
      run = DateTime.now().microsecondsSinceEpoch;
      progress = 'Inspecting latest saved task…';
    });
    try {
      await call('reflectLatest', {'id': run});
      var done = false;
      while (mounted && !done) {
        final events = await widget.chat.bridge.call({
          'command': 'poll',
          'id': run,
        }) as List;
        for (final e in events) {
          if (e['type'] == 'learningProgress') {
            setState(() => progress = e['note'] as String?);
          }
          if (e['type'] == 'done') {
            done = true;
            setState(() {
              error = e['error'] as String?;
              progress = e['learning'] as String? ?? 'No new eligible event. Previously claimed events are not replayed.';
            });
          }
        }
        if (!done) {
          await Future<void>.delayed(const Duration(milliseconds: 100));
        }
      }
      final data = await call('learningState');
      if (mounted) setState(() => report = data as Map);
    } catch (e) {
      if (mounted) {
        setState(
          () => error =
              '$e Reply and earlier evidence remain. Refresh before reviewing.',
        );
      }
    } finally {
      if (mounted) {
        setState(() {
          pending = false;
          run = null;
          stopping = false;
        });
      }
    }
  }

  Future<void> stop() async {
    setState(() => stopping = true);
    try {
      await widget.chat.bridge.call({'command': 'cancel', 'id': run});
    } catch (e) {
      if (mounted) {
        setState(
          () => error =
              'Stop not acknowledged: $e Keep this window open for the final outcome.',
        );
      }
    }
  }

  String checkCommand(dynamic document) {
    final text = (document as Map?)?['text'] as String? ?? '';
    return RegExp(r'Check command: `([^`]+)`').firstMatch(text)?.group(1) ??
        'Inspect snapshot';
  }

  Future<void> restore(Map event) async {
    final revision = state['revision'];
    final versions = (event['baseline'] as Map?)?['versions'] as List? ?? [];
    final before = checkCommand(event['candidate']);
    final after = checkCommand(
      versions.isEmpty ? null : (versions.last as Map)['document'],
    );
    final confirmed = await showDialog<bool>(
      context: context,
      builder: (context) => AlertDialog(
        title: const Text('Restore skill baseline?'),
        content: Text(
          '$before → $after\n\nThis restores the retained project skill snapshot and quarantines the candidate. Files and completed commands stay as they are. A manual skill change will prevent restoration.',
        ),
        actions: [
          TextButton(
            onPressed: () => Navigator.pop(context, false),
            child: const Text('Cancel'),
          ),
          FilledButton(
            onPressed: () => Navigator.pop(context, true),
            child: const Text('Restore & quarantine'),
          ),
        ],
      ),
    );
    if (confirmed == true && mounted) {
      await action('restoreLearning', {
        'revision': revision,
        'event': event['id'],
      });
    }
  }

  @override
  Widget build(BuildContext context) => PopScope(
    canPop: !pending,
    child: InspectorFrame(
      title: 'Skill learning',
      subtitle: 'This working folder · targeted repairs and evidence',
      canClose: !pending,
      child: Column(
        children: [
          Expanded(
            child: ListView(
              padding: const EdgeInsets.all(20),
              children: [
                if (error != null)
                  SelectableText(
                    error!,
                    style: TextStyle(
                      color: Theme.of(context).colorScheme.error,
                    ),
                  ),
                if (widget.chat.workspaceRoot == null)
                  const Text('Open a project or temporary working chat first.'),
                if (report != null) ...[
                  SwitchListTile(
                    contentPadding: EdgeInsets.zero,
                    title: const Text('Reflect on saved task evidence'),
                    subtitle: const Text(
                      'Off by default. Inspect one relevant saved event once; private notes are used only locally when separately eligible.',
                    ),
                    value: state['enabled'] == true,
                    onChanged: pending ? null : (v) => policy(enabled: v),
                  ),
                  SwitchListTile(
                    contentPadding: EdgeInsets.zero,
                    title: const Text(
                      'Activate tested command repairs automatically',
                    ),
                    subtitle: const Text(
                      'Experimental; live improvement acceptance is pending. Only the exact host config-check-v1 workflow. Unknown instruction impact, global skills and new authority require review. Passing tests alone is not permission.',
                    ),
                    value: state['automatic'] == true,
                    onChanged: pending ? null : (v) => policy(automatic: v),
                  ),
                  SwitchListTile(
                    contentPadding: EdgeInsets.zero,
                    title: const Text('Pause learning'),
                    subtitle: const Text(
                      'Keeps facts, skills and history. No new reflection or activation while paused.',
                    ),
                    value: state['paused'] == true,
                    onChanged: pending ? null : (v) => policy(paused: v),
                  ),
                  const Text(
                    'One proposed command repair can use four isolated cases: at most 20 model calls, 32 tool operations and 120 seconds total, 1024 output tokens per call. Only skill snapshots and disposable fixtures go to the configured provider. Original replies and baseline snapshots remain. No model-authored JSON draft, global rewrite or process execution is enabled.',
                  ),
                  const Text(
                    'After a matching failure of an activated workflow, one additional fixed comparison can check regression. Incomplete checks retain the current skill and do not retry. A confirmed regression restores and quarantines the candidate under the same automatic policy. Quarantine prevents automatic reactivation; deliberate Library reviews remain available.',
                  ),
                  if (report!['workflowAvailable'] == true) ...[
                    const SizedBox(height: 16),
                    const Text(
                      'Optional supported workflow: enable the top-level config.json flag and run one project check. Review its command before creating the saved project skill. This does not grant command permission.',
                    ),
                    const SizedBox(height: 16),
                    TextField(
                      key: const Key('learning-command'),
                      controller: command,
                      enabled: !pending,
                      decoration: const InputDecoration(
                        labelText: 'Current check command (node and one .cjs filename)',
                      ),
                    ),
                    TextButton(
                      onPressed: pending
                          ? null
                          : () => action('createCheckWorkflow', {
                              'command_text': command.text,
                            }),
                      child: const Text('Create check workflow'),
                    ),
                  ],
                  for (final event in (state['events'] as List?) ?? [])
                    ExpansionTile(
                      title: Text('${event['status']} · ${event['cause']}'),
                      subtitle: Text(
                        '${event['confidence']} confidence · ${event['reason']}',
                      ),
                      children: [
                        if (event['candidate'] != null)
                          SelectableText(
                            'Candidate check: ${checkCommand(event['candidate'])}',
                          ),
                        if ((event['monitorStatus'] as String? ?? '')
                            .isNotEmpty)
                          Text('Follow-up check: ${event['monitorStatus']}'),
                        if (event['status'] == 'active')
                          TextButton(
                            onPressed: pending
                                ? null
                                : () => restore(event as Map),
                            child: const Text('Restore baseline…'),
                          ),
                        if (event['status'] == 'quarantined')
                          const Text(
                            'Candidate quarantined. Automatic learning will not reactivate it. Inspect Library for deliberate edits.',
                          ),
                        if (event['status'] == 'pending' ||
                            event['status'] == 'interrupted')
                          const Text(
                            'Unfinished evidence is not eligible for activation and will not replay. Review Library for a new deliberate change.',
                          ),
                        ExpansionTile(
                          title: const Text('Snapshots & source'),
                          children: [
                            SelectableText(
                              const JsonEncoder.withIndent('  ').convert(event),
                            ),
                          ],
                        ),
                        if (event['status'] == 'review')
                          TextButton(
                            onPressed: pending
                                ? null
                                : () => action('approveLearning', {
                                    'revision': state['revision'],
                                    'event': event['id'],
                                  }),
                            child: const Text('Activate tested repair'),
                          ),
                      ],
                    ),
                  for (final trial in (report!['trials'] as List?) ?? [])
                    ExpansionTile(
                      title: Text(
                        'Trial ${trial['status']} · ${trial['improved'] == true ? 'Improved' : 'Not qualifying'}',
                      ),
                      children: [
                        SelectableText(
                          const JsonEncoder.withIndent('  ').convert(trial),
                        ),
                      ],
                    ),
                ],
              ],
            ),
          ),
          Padding(
            padding: const EdgeInsets.all(12),
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                if (error != null)
                  const Text(
                    'Change was not saved. Scroll up for details; refresh before retrying.',
                  ),
                if (progress != null)
                  Text(progress!, maxLines: 3, overflow: TextOverflow.ellipsis),
                if ((state['notice'] as String? ?? '').isNotEmpty)
                  Text(
                    state['notice'] as String,
                    key: const Key('learning-notice'),
                    maxLines: 3,
                    overflow: TextOverflow.ellipsis,
                  ),
                Wrap(
                  spacing: 8,
                  children: [
                    TextButton(
                      onPressed: pending ? null : load,
                      child: const Text('Refresh'),
                    ),
                    if (run != null)
                      TextButton(
                        onPressed: stopping ? null : stop,
                        child: Text(stopping ? 'Stopping…' : 'Stop learning'),
                      )
                    else
                      FilledButton(
                        onPressed:
                            pending ||
                                report == null ||
                                state['enabled'] != true ||
                                state['paused'] == true
                            ? null
                            : reflect,
                        child: const Text('Inspect latest task'),
                      ),
                  ],
                ),
              ],
            ),
          ),
        ],
      ),
    ),
  );
}
