import 'dart:convert';

import 'package:flutter/material.dart';

import 'bridge.dart';
import 'chat.dart';
import 'inspector.dart';
import 'theme.dart';

Future<void> showMemory(BuildContext context, ChatController chat) =>
    chat.inspectLocalSettings(() async {
      await showDialog<void>(
        context: context,
        barrierDismissible: false,
        builder: (_) =>
            MemoryInspector(bridge: chat.bridge, session: chat.session),
      );
      chat.invalidateContext();
    });

class MemoryInspector extends StatefulWidget {
  final ChatBridge bridge;
  final String? session;
  const MemoryInspector({super.key, required this.bridge, this.session});
  @override
  State<MemoryInspector> createState() => _MemoryInspectorState();
}

class _MemoryInspectorState extends State<MemoryInspector> {
  final title = TextEditingController(), text = TextEditingController();
  List<Map<String, dynamic>> items = [];
  Map<String, dynamic>? editing;
  bool form = false,
      enabled = true,
      folderAvailable = false,
      busy = false,
      loaded = false;
  String scope = 'all';
  String? error, notice;

  @override
  void initState() {
    super.initState();
    _refresh();
  }

  @override
  void dispose() {
    title.dispose();
    text.dispose();
    super.dispose();
  }

  Future<dynamic> _call(
    String command, [
    Map<String, dynamic> args = const {},
  ]) => widget.bridge.call({
    'command': command,
    'session': widget.session,
    ...args,
  });
  Future<void> _act(Future<void> Function() action) async {
    if (busy) return;
    setState(() {
      busy = true;
      error = null;
    });
    try {
      await action();
    } catch (failure) {
      if (mounted) setState(() => error = failure.toString());
    } finally {
      if (mounted) setState(() => busy = false);
    }
  }

  Future<void> _load() async {
    final result = await _call('memories');
    if (!mounted) return;
    setState(() {
      items = (result['items'] as List)
          .map((v) => (v as Map).cast<String, dynamic>())
          .toList();
      folderAvailable = result['folderAvailable'] == true;
      loaded = true;
    });
  }

  Future<void> _refresh() => _act(_load);
  void _edit([Map<String, dynamic>? item]) {
    if (busy) return;
    setState(() {
      editing = item;
      form = true;
      title.text = item?['title'] as String? ?? '';
      text.text = item?['text'] as String? ?? '';
      enabled = item?['enabled'] as bool? ?? true;
      scope = item?['scope'] as String? ?? (folderAvailable ? 'folder' : 'all');
      error = null;
      notice = null;
    });
  }

  void _cancel() {
    if (busy) return;
    setState(() {
      form = false;
      editing = null;
      error = null;
    });
  }

  Map<String, dynamic> _fields(Map<String, dynamic> item) => {
    for (final field in ['id', 'revision', 'scope', 'title', 'text', 'enabled'])
      field: item[field],
  };
  Future<void> _save() => _act(() async {
    if (title.text.trim().isEmpty ||
        title.text.runes.length > 80 ||
        title.text.contains(RegExp(r'[\x00-\x1f\x7f]'))) {
      throw const FormatException(
        'Give this preference a short title (up to 80 characters).',
      );
    }
    if (text.text.trim().isEmpty ||
        utf8.encode(text.text).length > 1024 ||
        text.text.contains('\x00')) {
      throw const FormatException(
        'Write a brief preference within 1 KiB of text.',
      );
    }
    await _call('saveMemory', {
      'scope': scope,
      'title': title.text,
      'text': text.text,
      'enabled': enabled,
      if (editing != null) 'id': editing!['id'],
      if (editing != null) 'revision': editing!['revision'],
    });
    if (mounted) {
      setState(() {
        form = false;
        editing = null;
        notice =
            'Preference saved${enabled ? ' and enabled for new messages' : ''}.';
      });
    }
    await _load();
  });
  Future<void> _toggle(Map<String, dynamic> item) => _act(() async {
    await _call('saveMemory', {
      ..._fields(item),
      'enabled': item['enabled'] != true,
    });
    if (mounted) {
      setState(
        () => notice = item['enabled'] == true
            ? 'Preference disabled for new messages.'
            : 'Preference enabled for new messages.',
      );
    }
    await _load();
  });
  Future<void> _delete(Map<String, dynamic> item) => _act(() async {
    await _call('deleteMemory', {
      'id': item['id'],
      'revision': item['revision'],
      'scope': item['scope'],
    });
    if (mounted) {
      setState(
        () => notice =
            'Preference deleted. Past replies remain in your saved chats.',
      );
    }
    await _load();
  });
  @override
  Widget build(BuildContext context) {
    final p = Palette(Theme.of(context).brightness == Brightness.dark);
    return PopScope(
      canPop: !busy,
      child: InspectorFrame(
        title: 'Memory',
        subtitle: 'Preferences you choose to keep',
        canClose: !busy,
        child: Column(
          children: [
            if (busy) const LinearProgressIndicator(minHeight: 2),
            Expanded(
              child: ListView(
                padding: const EdgeInsets.all(20),
                children: [
                  const Text(
                    'Saved preferences are shared with the model when enabled. Keep credentials and other secrets out of Memory.',
                  ),
                  const SizedBox(height: 8),
                  Text(
                    'Preferences are added by you. Delete or disable them to stop using them in new messages; past replies stay in your chats. Only a bounded selection is used—inspect context to see which ones.',
                    style: TextStyle(color: p.muted, fontSize: 12),
                  ),
                  if (error != null)
                    Padding(
                      padding: const EdgeInsets.only(top: 12),
                      child: Text(error!, style: TextStyle(color: p.errorText)),
                    ),
                  if (notice != null)
                    Padding(
                      padding: const EdgeInsets.only(top: 12),
                      child: Text(notice!, style: TextStyle(color: p.accent)),
                    ),
                  const SizedBox(height: 16),
                  if (form) ...[
                    Text(
                      editing == null ? 'New preference' : 'Edit preference',
                      style: const TextStyle(fontWeight: FontWeight.w600),
                    ),
                    const SizedBox(height: 12),
                    DropdownButtonFormField<String>(
                      key: ValueKey('memory-scope-$scope'),
                      initialValue: scope,
                      isExpanded: true,
                      decoration: const InputDecoration(labelText: 'Use in'),
                      items: [
                        const DropdownMenuItem(
                          value: 'all',
                          child: Text('All chats'),
                        ),
                        if (folderAvailable)
                          const DropdownMenuItem(
                            value: 'folder',
                            child: Text('This working folder'),
                          ),
                      ],
                      onChanged: busy || editing != null
                          ? null
                          : (value) => setState(() => scope = value!),
                    ),
                    const SizedBox(height: 12),
                    TextField(
                      key: const Key('memory-title'),
                      controller: title,
                      enabled: !busy,
                      maxLength: 80,
                      decoration: const InputDecoration(
                        labelText: 'Title',
                        hintText: 'Response style',
                      ),
                    ),
                    const SizedBox(height: 12),
                    TextField(
                      key: const Key('memory-text'),
                      controller: text,
                      enabled: !busy,
                      minLines: 4,
                      maxLines: 8,
                      decoration: const InputDecoration(
                        labelText: 'Preference',
                        hintText:
                            'Prefer concise explanations with a short example.',
                      ),
                    ),
                    CheckboxListTile(
                      key: const Key('memory-enabled'),
                      value: enabled,
                      onChanged: busy
                          ? null
                          : (value) => setState(() => enabled = value!),
                      contentPadding: EdgeInsets.zero,
                      title: const Text('Use in new messages'),
                    ),
                  ] else ...[
                    if (loaded && items.isEmpty)
                      const Text(
                        'No saved preferences. Add one when you want Dolores to remember how you like to work.',
                      ),
                    for (final item in items)
                      Card(
                        elevation: 0,
                        color: p.surface,
                        margin: const EdgeInsets.only(bottom: 8),
                        child: Padding(
                          padding: const EdgeInsets.all(16),
                          child: Column(
                            crossAxisAlignment: CrossAxisAlignment.start,
                            children: [
                              Text(
                                item['title'] as String,
                                style: const TextStyle(
                                  fontWeight: FontWeight.w600,
                                ),
                              ),
                              Text(
                                '${item['scope'] == 'folder' ? 'This working folder' : 'All chats'} · ${item['enabled'] == true ? 'Enabled' : 'Disabled'} · Added by you',
                                style: TextStyle(fontSize: 11, color: p.muted),
                              ),
                              const SizedBox(height: 8),
                              SelectableText(
                                item['text'] as String,
                                key: ValueKey('memory-body-${item['id']}'),
                              ),
                              const SizedBox(height: 8),
                              Text(
                                'Updated ${DateTime.fromMillisecondsSinceEpoch(item['updatedAt'] as int).toLocal()} · revision ${item['revision']}',
                                style: TextStyle(fontSize: 11, color: p.muted),
                              ),
                              Wrap(
                                spacing: 8,
                                children: [
                                  TextButton(
                                    key: ValueKey('edit-memory-${item['id']}'),
                                    onPressed: busy ? null : () => _edit(item),
                                    child: const Text('Edit'),
                                  ),
                                  TextButton(
                                    key: ValueKey(
                                      'toggle-memory-${item['id']}',
                                    ),
                                    onPressed: busy
                                        ? null
                                        : () => _toggle(item),
                                    child: Text(
                                      item['enabled'] == true
                                          ? 'Disable'
                                          : 'Enable',
                                    ),
                                  ),
                                  TextButton(
                                    key: ValueKey(
                                      'delete-memory-${item['id']}',
                                    ),
                                    onPressed: busy
                                        ? null
                                        : () => _delete(item),
                                    child: const Text('Delete'),
                                  ),
                                ],
                              ),
                            ],
                          ),
                        ),
                      ),
                  ],
                ],
              ),
            ),
            Divider(height: 1, color: p.border),
            Padding(
              padding: const EdgeInsets.all(12),
              child: Wrap(
                spacing: 8,
                runSpacing: 8,
                children: [
                  TextButton(
                    key: const Key('refresh-memory'),
                    onPressed: busy ? null : _refresh,
                    child: const Text('Refresh'),
                  ),
                  if (form) ...[
                    TextButton(
                      key: const Key('cancel-memory-edit'),
                      onPressed: busy ? null : _cancel,
                      child: const Text('Cancel'),
                    ),
                    FilledButton(
                      key: const Key('save-memory'),
                      onPressed: busy ? null : _save,
                      child: const Text('Save preference'),
                    ),
                  ] else
                    FilledButton(
                      key: const Key('new-memory'),
                      onPressed: busy || !loaded ? null : () => _edit(),
                      child: const Text('New preference'),
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
