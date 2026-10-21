import 'dart:convert';

import 'package:flutter/material.dart';

import 'chat.dart';

class PlanningPanel extends StatelessWidget {
  final ChatController chat;
  const PlanningPanel({super.key, required this.chat});
  @override
  Widget build(BuildContext context) {
    final state = chat.planning;
    final refs = state['references'] as List? ?? [];
    return Column(
      mainAxisSize: MainAxisSize.min,
      children: [
        if (state['planning'] == true ||
            (state['draft'] as String? ?? '').isNotEmpty)
          ListTile(
            dense: true,
            contentPadding: EdgeInsets.zero,
            leading: const Icon(Icons.list_alt_outlined, size: 18),
            title: Text(
              state['planning'] == true
                  ? 'Planning · inspection only'
                  : state['accepted'] != null
                  ? 'Accepted plan'
                  : 'Draft plan',
              maxLines: 1,
            ),
            subtitle: Text(
              'Revision ${state['revision']} · ${state['accepted'] == null ? 'Review before execution' : 'Accepted; Start remains explicit'}',
            ),
            trailing: IconButton(
              tooltip: 'Review plan',
              onPressed: () => showPlan(context, chat),
              icon: const Icon(Icons.more_horiz),
            ),
          ),
        if (refs.isNotEmpty)
          Align(
            alignment: Alignment.centerLeft,
            child: Wrap(
              spacing: 8,
              runSpacing: 4,
              children: [
                for (final r in refs)
                  InputChip(
                    label: ConstrainedBox(
                      constraints: const BoxConstraints(maxWidth: 250),
                      child: Text(
                        '@${r['kind']}: ${r['source']}',
                        overflow: TextOverflow.ellipsis,
                      ),
                    ),
                    onPressed: () => showDialog<void>(
                      context: context,
                      builder: (context) => AlertDialog(
                        title: Text('Selected ${r['kind']} snapshot'),
                        content: SizedBox(
                          width: 560,
                          child: SingleChildScrollView(
                            child: SelectableText(
                              '${r['source']}\nRevision ${r['revision']}\n${r['omissions']}\n\n${r['text']}',
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
                    ),
                    onDeleted: chat.busy || chat.changing
                        ? null
                        : () => chat.changePlanning({
                            'action': 'remove',
                            'digest': r['revision'],
                          }),
                    deleteButtonTooltipMessage: 'Remove shared context',
                  ),
              ],
            ),
          ),
      ],
    );
  }
}

Future<void> showPlan(BuildContext context, ChatController chat) async {
  await chat.refreshPlanning();
  if (!context.mounted) return;
  await showDialog<void>(
    context: context,
    builder: (_) => _PlanEditor(chat: chat),
  );
}

class _PlanEditor extends StatefulWidget {
  final ChatController chat;
  const _PlanEditor({required this.chat});
  @override
  State<_PlanEditor> createState() => _PlanEditorState();
}

class _PlanEditorState extends State<_PlanEditor> {
  late final text = TextEditingController(
    text: widget.chat.planning['draft'] as String? ?? '',
  );
  late final decisions = TextEditingController(
    text: widget.chat.planning['decisions'] as String? ?? '',
  );
  bool get dirty =>
      text.text != (widget.chat.planning['draft'] ?? '') ||
      decisions.text != (widget.chat.planning['decisions'] ?? '');
  Future<bool> save() => widget.chat.changePlanning({
    'action': 'draft',
    'text': text.text,
    'decisions': decisions.text,
  });
  Future<void> close() async {
    if (widget.chat.busy || widget.chat.changing) return;
    if (!dirty) {
      Navigator.pop(context);
      return;
    }
    final action = await showDialog<String>(
      context: context,
      builder: (context) => AlertDialog(
        title: const Text('Keep plan edits?'),
        actions: [
          for (final a in ['Keep editing', 'Discard', 'Save'])
            TextButton(
              onPressed: () => Navigator.pop(context, a),
              child: Text(a),
            ),
        ],
      ),
    );
    if (action == 'Discard' || (action == 'Save' && await save())) {
      if (mounted) Navigator.pop(context);
    }
  }

  @override
  Widget build(BuildContext context) => ListenableBuilder(
    listenable: widget.chat,
    builder: (context, _) {
      final chat = widget.chat;
      final state = chat.planning;
      final enabled = !chat.busy && !chat.changing;
      final style = Theme.of(context).textTheme.bodyMedium;
      return PopScope(
        canPop: !dirty && enabled,
        onPopInvokedWithResult: (didPop, _) {
          if (!didPop) close();
        },
        child: AlertDialog(
          title: const Text('Plan and decisions'),
          content: SizedBox(
            width: 620,
            child: SingleChildScrollView(
              child: Column(
                mainAxisSize: MainAxisSize.min,
                crossAxisAlignment: CrossAxisAlignment.stretch,
                children: [
                  SwitchListTile(
                    contentPadding: EdgeInsets.zero,
                    title: const Text('Planning mode'),
                    subtitle: Text(
                      state['planning'] == true
                          ? 'Inspect local files and discuss; execution tools stay unavailable.'
                          : 'Next message uses normal tool permissions. Accept a saved plan to execute its reviewed revision.',
                    ),
                    value: state['planning'] == true,
                    onChanged: enabled
                        ? (v) => chat.changePlanning({
                            'action': 'mode',
                            'enabled': v,
                          })
                        : null,
                  ),
                  TextField(
                    key: const Key('plan-text'),
                    controller: text,
                    maxLines: 8,
                    onChanged: (_) => setState(() {}),
                    decoration: const InputDecoration(
                      labelText: 'Concrete plan (up to 8 KiB)',
                    ),
                  ),
                  const SizedBox(height: 12),
                  TextField(
                    key: const Key('plan-decisions'),
                    controller: decisions,
                    maxLines: 3,
                    onChanged: (_) => setState(() {}),
                    decoration: const InputDecoration(
                      labelText: 'Your decisions and constraints',
                    ),
                  ),
                  const SizedBox(height: 12),
                  const Text(
                    'Accept binds this saved revision to the current goal and checklist when one exists. Direct instructions and tool permissions remain in force.',
                  ),
                  if (state['accepted'] != null)
                    Text('Accepted revision ${state['accepted']['revision']}'),
                  if (chat.error != null) SelectableText(chat.error!),
                ],
              ),
            ),
          ),
          actions: [
            TextButton(
              onPressed: enabled ? close : null,
              child: Text('Close', style: style),
            ),
            TextButton(
              onPressed: enabled && !dirty
                  ? () async {
                      await chat.refreshPlanning();
                      if (mounted) {
                        text.text = chat.planning['draft'] as String? ?? '';
                        decisions.text =
                            chat.planning['decisions'] as String? ?? '';
                        setState(() {});
                      }
                    }
                  : null,
              child: Text('Refresh saved plan', style: style),
            ),
            TextButton(
              key: const Key('save-plan'),
              onPressed: enabled ? save : null,
              child: Text('Save plan', style: style),
            ),
            TextButton(
              key: const Key('accept-plan'),
              onPressed:
                  enabled &&
                      !dirty &&
                      (state['draft'] as String? ?? '').trim().isNotEmpty
                  ? () => chat.changePlanning({'action': 'accept'})
                  : null,
              child: Text('Accept saved plan', style: style),
            ),
            FilledButton(
              key: const Key('execute-plan'),
              onPressed: enabled && !dirty && state['accepted'] != null
                  ? () async {
                      await chat.executePlan();
                      if (context.mounted && chat.busy) Navigator.pop(context);
                    }
                  : null,
              child: Text('Execute accepted plan', style: style),
            ),
            if (chat.busy)
              TextButton(
                onPressed: chat.stop,
                child: Text('Stop task', style: style),
              ),
          ],
        ),
      );
    },
  );
  @override
  void dispose() {
    text.dispose();
    decisions.dispose();
    super.dispose();
  }
}

Future<bool> showContextPicker(
  BuildContext context,
  ChatController chat,
  String kind,
) async {
  await chat.refreshPlanning();
  if (!context.mounted) return false;
  return await showDialog<bool>(
        context: context,
        builder: (_) => _ContextPicker(chat: chat, kind: kind),
      ) ??
      false;
}

class _ContextPicker extends StatefulWidget {
  final ChatController chat;
  final String kind;
  const _ContextPicker({required this.chat, required this.kind});
  @override
  State<_ContextPicker> createState() => _ContextPickerState();
}

class _ContextPickerState extends State<_ContextPicker> {
  String path = '.';
  String? owner, error;
  int cursor = 0;
  Map data = {}, preview = {}, source = {};
  bool loading = false;
  @override
  void initState() {
    super.initState();
    load();
  }

  Future<void> load() async {
    setState(() {
      loading = true;
      error = null;
      preview = {};
    });
    try {
      data = await widget.chat.bridge.call({
        'command': 'referenceChoices',
        'session': widget.chat.session,
        'kind': widget.kind,
        'path': path,
        'cursor': cursor,
        'owner': owner,
      }) as Map;
    } catch (e) {
      error = '$e';
    } finally {
      if (mounted) setState(() => loading = false);
    }
  }

  Future<void> choose(Map e) async {
    if (e['folder'] != null) {
      path = e['folder'] as String;
      cursor = 0;
      await load();
      return;
    }
    if (e['owner'] != null) {
      owner = e['owner'] as String;
      await load();
      return;
    }
    setState(() {
      loading = true;
      error = null;
    });
    try {
      source = e['source'] as Map;
      preview = await widget.chat.bridge.call({
        'command': 'referencePreview',
        'session': widget.chat.session,
        'source': source,
      }) as Map;
    } catch (e) {
      error = '$e';
    } finally {
      if (mounted) setState(() => loading = false);
    }
  }

  @override
  Widget build(BuildContext context) => AlertDialog(
    title: Text('Share ${widget.kind} context'),
    content: SizedBox(
      width: 600,
      child: SingleChildScrollView(
        child: Column(
          mainAxisSize: MainAxisSize.min,
          crossAxisAlignment: CrossAxisAlignment.stretch,
          children: [
            const Text(
              'Selected text goes to this chat’s model. Snapshots keep exactly these bytes even if the source later changes or is deleted. No access permission is granted.',
            ),
            if (loading) const LinearProgressIndicator(),
            if (error != null) SelectableText(error!),
            if (data['notice'] != null) Text('${data['notice']}'),
            if (preview.isEmpty) ...[
              if (path != '.' || owner != null || cursor != 0)
                TextButton(
                  onPressed: loading
                      ? null
                      : () {
                          path = '.';
                          owner = null;
                          cursor = 0;
                          load();
                        },
                  child: const Text('Back to sources'),
                ),
              for (final e in data['choices'] as List? ?? [])
                ListTile(
                  dense: true,
                  title: Text(
                    '${e['title']}',
                    maxLines: 2,
                    overflow: TextOverflow.ellipsis,
                  ),
                  trailing: e['folder'] != null || e['owner'] != null
                      ? const Icon(Icons.chevron_right)
                      : const Icon(Icons.visibility_outlined),
                  onTap: loading ? null : () => choose(e as Map),
                ),
              if (data['cursor'] != null)
                TextButton(
                  onPressed: loading
                      ? null
                      : () {
                          cursor = data['cursor'] as int;
                          load();
                        },
                  child: const Text('Next directory page'),
                ),
            ] else ...[
              Text(
                '${preview['source']} · ${utf8.encode(preview['text'].toString()).length} bytes · selected content shown below',
              ),
              Text('${preview['omissions']}'),
              SelectableText('${preview['text']}'),
            ],
          ],
        ),
      ),
    ),
    actions: [
      TextButton(
        onPressed: loading ? null : () => Navigator.pop(context, false),
        child: const Text('Cancel'),
      ),
      TextButton(
        onPressed: loading ? null : load,
        child: const Text('Refresh sources'),
      ),
      FilledButton(
        key: const Key('share-context'),
        onPressed: loading || preview.isEmpty
            ? null
            : () async {
                setState(() => loading = true);
                final saved = await widget.chat.changePlanning({
                  'action': 'add',
                  'source': source,
                  'expected': preview['revision'],
                });
                if (!context.mounted) return;
                if (saved) {
                  Navigator.pop(context, true);
                } else {
                  setState(() {
                    loading = false;
                    error = widget.chat.error;
                  });
                }
              },
        child: const Text('Share snapshot'),
      ),
    ],
  );
}
