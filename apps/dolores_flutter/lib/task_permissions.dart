import 'package:flutter/material.dart';

import 'chat.dart';
import 'inspector.dart';

Future<void> showTaskPermissions(BuildContext context, ChatController chat) =>
    showDialog<void>(
      context: context,
      builder: (_) => TaskPermissionsInspector(chat: chat),
    );

class TaskPermissionsInspector extends StatefulWidget {
  final ChatController chat;
  const TaskPermissionsInspector({super.key, required this.chat});
  @override
  State<TaskPermissionsInspector> createState() => _TaskPermissionsState();
}

class _TaskPermissionsState extends State<TaskPermissionsInspector> {
  late final String session = widget.chat.session!;
  final prefix = TextEditingController(text: '.'),
      program = TextEditingController(),
      args = TextEditingController();
  Map<String, dynamic>? report;
  String mode = 'review', expiry = '24';
  final selected = <String>{};
  final scroll = ScrollController();
  bool pending = false, understood = false, command = false;
  String? error, notice;
  @override
  void initState() {
    super.initState();
    load();
  }

  @override
  void dispose() {
    scroll.dispose();
    prefix.dispose();
    program.dispose();
    args.dispose();
    super.dispose();
  }

  Future<void> load({bool keepDraft = false}) async {
    setState(() => pending = true);
    try {
      final value = Map<String, dynamic>.from(
        await widget.chat.bridge.call({
          'command': 'taskPermissions',
          'session': session,
        }) as Map,
      );
      if (!mounted) return;
      setState(() {
        report = value;
        understood = false;
        error = null;
        if (!keepDraft) {
          final policy = value['policy'] as Map;
          mode = policy['mode'] as String;
          selected.clear();
          command = false;
          for (final raw in policy['grants'] as List) {
            final grant = raw as Map;
            if (grant['tool'] == 'run_command') {
              command = true;
              program.text = grant['command']['program'] as String;
              args.text = (grant['command']['args'] as List).join('\n');
            } else {
              selected.add(grant['tool'] as String);
              if (grant['pathPrefix'] != null) {
                prefix.text = grant['pathPrefix'] as String;
              }
            }
          }
        }
      });
    } catch (failure) {
      if (mounted) setState(() => error = failure.toString());
      if (scroll.hasClients) scroll.jumpTo(0);
    } finally {
      if (mounted) setState(() => pending = false);
    }
  }

  Future<void> save({bool revoke = false}) async {
    final grants = <Map<String, dynamic>>[];
    if (!revoke && mode == 'auto') {
      for (final tool in selected) {
        final matches = (report?['mcpTools'] as List? ?? []).where(
          (m) => m['alias'] == tool,
        );
        final mcp = matches.isEmpty ? null : matches.first;
        grants.add({
          'tool': tool,
          'pathPrefix': tool == 'inspect_harness' || mcp != null
              ? null
              : prefix.text.trim(),
          'command': null,
          'mcpConnection': mcp?['connectionId'],
          'mcpRevision': mcp?['revision'],
        });
      }
      if (command) {
        grants.add({
          'tool': 'run_command',
          'pathPrefix': null,
          'command': {
            'program': program.text.trim(),
            'args': args.text.isEmpty ? <String>[] : args.text.split('\n'),
          },
          'mcpConnection': null,
          'mcpRevision': null,
        });
      }
    }
    setState(() {
      pending = true;
      error = null;
      notice = null;
    });
    try {
      final result = await widget.chat.bridge.call({
        'command': 'setTaskPermissions',
        'session': session,
        'revision': report!['revision'],
        'policy': {
          'mode': revoke ? 'review' : mode,
          'grants': grants,
          'expiresAt': revoke || mode == 'review' || expiry == 'none'
              ? null
              : DateTime.now().millisecondsSinceEpoch ~/ 1000 +
                    int.parse(expiry) * 3600,
        },
      });
      if (mounted) {
        setState(() {
          report = Map<String, dynamic>.from(result as Map);
          mode = revoke ? 'review' : mode;
          understood = false;
          notice = revoke
              ? 'Grants revoked. Active work is stopping; inspect effects already started.'
              : 'Task access saved for this chat.';
        });
      }
      widget.chat.invalidateContext();
      if (scroll.hasClients) scroll.jumpTo(0);
    } catch (failure) {
      if (mounted) setState(() => error = failure.toString());
    } finally {
      if (mounted) setState(() => pending = false);
    }
  }

  @override
  Widget build(BuildContext context) => AnimatedBuilder(
    animation: widget.chat,
    builder: (context, _) {
      final busy = widget.chat.busy, locked = pending || busy;
      return PopScope(
        canPop: !pending,
        child: InspectorFrame(
          title: 'Task permissions',
          subtitle: 'This chat · its saved working folder',
          canClose: !pending,
          child: SizedBox(
            width: 680,
            height: 600,
            child: Column(
              children: [
                Expanded(
                  child: SingleChildScrollView(
                    controller: scroll,
                    child: Padding(
                      padding: const EdgeInsets.all(16),
                      child: Column(
                        crossAxisAlignment: CrossAxisAlignment.start,
                        children: [
                          if (error != null) SelectableText(error!),
                          if (notice != null) SelectableText(notice!),
                          if (report != null) ...[
                            SelectableText(
                              'Saved access: ${report!['policy']['mode']} · revision ${report!['revision']}\nExpiry: ${report!['policy']['expiresAt'] ?? 'until revoked'}',
                            ),
                            SelectableText(
                              '${report!['containment']}\n${report!['adaptation']}',
                            ),
                            if (report!['expired'] == true)
                              const Text(
                                'The saved grant expired. Renew it or revoke it before tool use.',
                              ),
                            DropdownButtonFormField<String>(
                              key: ValueKey('mode-$mode'),
                              initialValue: mode,
                              decoration: const InputDecoration(
                                labelText: 'Tool access',
                              ),
                              items: const [
                                DropdownMenuItem(
                                  value: 'review',
                                  child: Text('Review every operation'),
                                ),
                                DropdownMenuItem(
                                  value: 'auto',
                                  child: Text('Approve within selected grants'),
                                ),
                                DropdownMenuItem(
                                  value: 'fullAccess',
                                  child: Text('Full access for this chat'),
                                ),
                              ],
                              onChanged: locked
                                  ? null
                                  : (v) => setState(() {
                                      mode = v!;
                                      understood = false;
                                    }),
                            ),
                            if (mode != 'review')
                              DropdownButtonFormField<String>(
                                initialValue: expiry,
                                decoration: const InputDecoration(
                                  labelText: 'Grant duration',
                                ),
                                items: const [
                                  DropdownMenuItem(
                                    value: '1',
                                    child: Text('One hour'),
                                  ),
                                  DropdownMenuItem(
                                    value: '24',
                                    child: Text('24 hours'),
                                  ),
                                  DropdownMenuItem(
                                    value: 'none',
                                    child: Text('Until revoked'),
                                  ),
                                ],
                                onChanged: locked
                                    ? null
                                    : (v) => setState(() => expiry = v!),
                              ),
                            if (mode == 'auto') ...[
                              TextField(
                                controller: prefix,
                                enabled: !locked,
                                decoration: const InputDecoration(
                                  labelText: 'Relative folder/file prefix',
                                  helperText: '. covers permitted file paths in this folder.',
                                ),
                              ),
                              for (final tool in {
                                'list_folder': 'List folders',
                                'search_text': 'Search text',
                                'read_text_file': 'Read text files',
                                'edit_text_file': 'Edit existing files',
                                'create_text_file': 'Create files',
                                'inspect_harness':
                                    'Inspect Dolores source and capabilities',
                              }.entries)
                                CheckboxListTile(
                                  contentPadding: EdgeInsets.zero,
                                  title: Text(tool.value),
                                  value: selected.contains(tool.key),
                                  onChanged: locked
                                      ? null
                                      : (v) => setState(
                                          () => v!
                                              ? selected.add(tool.key)
                                              : selected.remove(tool.key),
                                        ),
                                ),
                              CheckboxListTile(
                                contentPadding: EdgeInsets.zero,
                                title: const Text('Run this exact command'),
                                value: command,
                                onChanged: locked
                                    ? null
                                    : (v) => setState(() => command = v!),
                              ),
                              if (command) ...[
                                TextField(
                                  controller: program,
                                  enabled: !locked,
                                  decoration: const InputDecoration(
                                    labelText: 'Executable',
                                  ),
                                ),
                                TextField(
                                  controller: args,
                                  enabled: !locked,
                                  maxLines: 3,
                                  decoration: const InputDecoration(
                                    labelText:
                                        'Literal arguments (one per line)',
                                  ),
                                ),
                              ],
                              for (final mcp
                                  in report!['mcpTools'] as List? ?? [])
                                CheckboxListTile(
                                  contentPadding: EdgeInsets.zero,
                                  title: Text(
                                    '${mcp['label']} · ${mcp['tool']}',
                                  ),
                                  subtitle: Text(
                                    'Reviewed revision ${mcp['revision']}',
                                  ),
                                  value: selected.contains(mcp['alias']),
                                  onChanged: locked
                                      ? null
                                      : (v) => setState(
                                          () => v!
                                              ? selected.add(
                                                  mcp['alias'] as String,
                                                )
                                              : selected.remove(mcp['alias']),
                                        ),
                                ),
                            ],
                            if (mode == 'fullAccess')
                              const Text(
                                'Advertised file, command and reviewed external tools will run without per-operation prompts. External effects may persist after Stop; this does not enable new tools or self-updates.',
                              ),
                            if (mode != 'review')
                              CheckboxListTile(
                                contentPadding: EdgeInsets.zero,
                                title: const Text(
                                  'I understand this scope and the access described above',
                                ),
                                value: understood,
                                onChanged: locked
                                    ? null
                                    : (v) => setState(() => understood = v!),
                              ),
                            if (busy)
                              const Text(
                                'Revoke is available during a run. Stop first to expand access.',
                              ),
                          ],
                          if (pending) const LinearProgressIndicator(),
                        ],
                      ),
                    ),
                  ),
                ),
                Padding(
                  padding: const EdgeInsets.all(12),
                  child: Wrap(
                    spacing: 8,
                    children: [
                      TextButton(
                        onPressed: pending
                            ? null
                            : () => load(keepDraft: report != null),
                        child: const Text('Refresh'),
                      ),
                      TextButton(
                        onPressed: pending || report == null
                            ? null
                            : () => save(revoke: true),
                        child: const Text('Revoke grants'),
                      ),
                      FilledButton(
                        onPressed:
                            locked ||
                                report == null ||
                                (mode != 'review' && !understood)
                            ? null
                            : () => save(),
                        child: const Text('Save access'),
                      ),
                    ],
                  ),
                ),
              ],
            ),
          ),
        ),
      );
    },
  );
}
