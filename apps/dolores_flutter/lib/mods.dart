import 'dart:async';

import 'package:flutter/material.dart';

import 'chat.dart';
import 'inspector.dart';

const modManifest = {
  'id': 'recovery-hints',
  'api': 1,
  'stateSchema': 1,
  'hook': 'recovery_hint',
  'capabilities': <String>[],
  'dependencies': <String>[],
  'title': 'Recovery guidance',
  'description': 'A scoped mod suggests a host-owned recovery view. It never executes an action or changes a limit.',
};

class ModsInspector extends StatefulWidget {
  final ChatController chat;
  const ModsInspector({super.key, required this.chat});
  @override
  State<ModsInspector> createState() => _ModsState();
}

class _ModsState extends State<ModsInspector> {
  final source = TextEditingController();
  Map? report;
  String? error;
  bool pending = false, stopping = false;
  int? run;
  int category = 1;
  Map get state => (report?['state'] as Map?) ?? {};
  bool get available =>
      widget.chat.session != null && widget.chat.workspaceRoot != null;
  @override
  void initState() {
    super.initState();
    if (available) action('modState');
  }

  @override
  void dispose() {
    source.dispose();
    super.dispose();
  }

  Future<dynamic> call(
    String command, [
    Map<String, dynamic> fields = const {},
  ]) => widget.chat.bridge.call({
    'command': command,
    'session': widget.chat.session,
    ...fields,
  });
  Future<void> action(
    String command, [
    Map<String, dynamic> fields = const {},
  ]) async {
    if (pending) return;
    setState(() {
      pending = true;
      error = null;
    });
    try {
      final value = await call(command, {'category': category, ...fields});
      if (mounted) {
        setState(() {
          report = value as Map;
          if (source.text.isEmpty &&
              (state['draft'] as String? ?? '').isNotEmpty) {
            source.text = state['draft'] as String;
          }
        });
      }
    } catch (e) {
      if (mounted) {
        setState(
          () => error = '$e Your source remains. Refresh before retrying.',
        );
      }
    } finally {
      if (mounted) setState(() => pending = false);
    }
  }

  Future<void> generate() async {
    setState(() {
      pending = true;
      error = null;
      run = DateTime.now().microsecondsSinceEpoch;
    });
    try {
      await call('generateMod', {'id': run, 'revision': state['revision']});
      var done = false;
      while (mounted && !done) {
        final events = await widget.chat.bridge.call({
          'command': 'poll',
          'id': run,
        }) as List;
        for (final e in events) {
          if (e['type'] == 'done') {
            done = true;
            setState(() {
              error = e['error'] as String?;
              if (e['source'] != null) source.text = e['source'] as String;
            });
          }
        }
        if (!done) {
          await Future<void>.delayed(const Duration(milliseconds: 100));
        }
      }
      final value = await call('modState', {'category': category});
      if (mounted) setState(() => report = value as Map);
    } catch (e) {
      if (mounted) {
        setState(
          () =>
              error = '$e Baseline and source remain. Refresh before retrying.',
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
        setState(() {
          stopping = false;
          error = '$e';
        });
      }
    }
  }

  Future<void> restore() async {
    final confirmed = await showDialog<bool>(
      context: context,
      builder: (context) => AlertDialog(
        title: const Text('Restore the previous recovery behavior?'),
        content: const Text(
          'Quarantines the active mod and restores its retained baseline. Files, chats and model settings stay intact.',
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
      await action('restoreMod', {'revision': state['revision']});
    }
  }

  @override
  Widget build(BuildContext context) => PopScope(
    canPop: !pending,
    child: InspectorFrame(
      title: 'Harness mods',
      subtitle: 'This working folder · executable recovery guidance',
      canClose: !pending,
      child: Column(
        children: [
          Expanded(
            child: ListView(
              padding: const EdgeInsets.all(20),
              children: [
                if (!available)
                  const Text(
                    'Open a project or temporary working chat to manage scoped mods.',
                  ),
                if (report != null) ...[
                  const Text(
                    'Mods can refine recovery hints. The host keeps permissions, credentials, budgets and tests. No arbitrary native or UI code runs.',
                  ),
                  const SizedBox(height: 12),
                  if ((state['draftNotice'] as String? ?? '').isNotEmpty)
                    SelectableText('${state['draftNotice']}'),
                  SwitchListTile(
                    contentPadding: EdgeInsets.zero,
                    title: const Text('Activate qualifying mods automatically'),
                    subtitle: const Text(
                      'Off by default. Only strict improvement on six fixed ABI 1 cases; restore is available. This does not enable general self-modification.',
                    ),
                    value: state['automatic'] == true,
                    onChanged: pending
                        ? null
                        : (v) => action('setModPolicy', {
                            'revision': state['revision'],
                            'automatic': v,
                          }),
                  ),
                  const SizedBox(height: 12),
                  DropdownButtonFormField<int>(
                    initialValue: category,
                    decoration: const InputDecoration(
                      labelText: 'Recovery case',
                    ),
                    isExpanded: true,
                    items: [
                      for (final entry in {
                        0: 'Unknown',
                        1: 'Output limit',
                        2: 'Context limit',
                        3: 'Tool allowance',
                        4: 'Access denied',
                        5: 'Interrupted task',
                      }.entries)
                        DropdownMenuItem(
                          value: entry.key,
                          child: Text(entry.value),
                        ),
                    ],
                    onChanged: pending
                        ? null
                        : (v) {
                            setState(() => category = v!);
                            action('modState');
                          },
                  ),
                  const SizedBox(height: 16),
                  Card(
                    child: Padding(
                      padding: const EdgeInsets.all(16),
                      child: Column(
                        crossAxisAlignment: CrossAxisAlignment.start,
                        children: [
                          Text(
                            '${report!['card']['title']}',
                            style: Theme.of(context).textTheme.titleMedium,
                          ),
                          const SizedBox(height: 8),
                          SelectableText('${report!['card']['text']}'),
                          if (report!['card']['notice'] != null)
                            SelectableText('${report!['card']['notice']}'),
                          if (state['active'] != null)
                            TextButton(
                              onPressed: pending ? null : restore,
                              child: const Text('Restore baseline…'),
                            ),
                        ],
                      ),
                    ),
                  ),
                  const SizedBox(height: 16),
                  const Text(
                    'Draft a repair shares only the current mod source with your selected model. One request, up to 1024 output tokens and 30 seconds. Partial or malformed drafts remain editable and cannot activate.',
                  ),
                  const SizedBox(height: 16),
                  TextField(
                    key: const Key('mod-source'),
                    controller: source,
                    enabled: !pending,
                    minLines: 4,
                    maxLines: 9,
                    maxLength: 8192,
                    style: const TextStyle(
                      fontFamily: 'monospace',
                      fontSize: 13,
                    ),
                    decoration: const InputDecoration(
                      labelText: 'WebAssembly text source',
                      hintText: '(module (func (export "recovery_hint") …))',
                    ),
                  ),
                  for (final v in (state['versions'] as List?) ?? [])
                    ExpansionTile(
                      title: Text(
                        '${v['status']} · ${(v['identity'] as String).substring(0, 12)}',
                      ),
                      subtitle: Text('${v['reason']}'),
                      children: [
                        SelectableText('${v['source']}'),
                        Text(
                          'Candidate: ${v['results']} · baseline: ${v['baselineResults']}',
                        ),
                        if (v['status'] == 'review')
                          TextButton(
                            onPressed: pending
                                ? null
                                : () => action('activateMod', {
                                    'revision': state['revision'],
                                    'identity': v['identity'],
                                  }),
                            child: const Text('Activate tested mod'),
                          ),
                      ],
                    ),
                  ExpansionTile(
                    title: const Text('Change history & runtime boundary'),
                    children: [
                      SelectableText('${report!['boundary']}'),
                      for (final event in (state['events'] as List?) ?? [])
                        SelectableText('$event'),
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
                  ConstrainedBox(
                    constraints: const BoxConstraints(maxHeight: 100),
                    child: SingleChildScrollView(
                      child: SelectableText(
                        error!,
                        key: const Key('mod-error'),
                        style: TextStyle(
                          color: Theme.of(context).colorScheme.error,
                        ),
                      ),
                    ),
                  ),
                if (pending) const LinearProgressIndicator(minHeight: 2),
                Wrap(
                  spacing: 8,
                  runSpacing: 8,
                  children: [
                    TextButton(
                      onPressed: pending || !available
                          ? null
                          : () => action('modState'),
                      child: const Text('Refresh'),
                    ),
                    if (run != null)
                      TextButton(
                        onPressed: stopping ? null : stop,
                        child: Text(stopping ? 'Stopping…' : 'Stop draft'),
                      )
                    else ...[
                      TextButton(
                        onPressed: pending || report == null ? null : generate,
                        child: const Text('Draft a repair'),
                      ),
                      FilledButton(
                        onPressed: pending || report == null
                            ? null
                            : () => action('testMod', {
                                'revision': state['revision'],
                                'manifest': modManifest,
                                'source': source.text,
                              }),
                        child: const Text('Test source'),
                      ),
                    ],
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
