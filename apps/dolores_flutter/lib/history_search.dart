import 'dart:convert';

import 'package:flutter/material.dart';

import 'chat.dart';
import 'form_field_layout.dart';
import 'inspector.dart';
import 'help_tip.dart';

Future<void> showHistorySearch(BuildContext context, ChatController chat) =>
    showDialog<void>(
      context: context,
      barrierDismissible: false,
      builder: (_) => HistorySearch(chat: chat),
    );

class HistorySearch extends StatefulWidget {
  final ChatController chat;
  const HistorySearch({super.key, required this.chat});
  @override
  State<HistorySearch> createState() => _HistorySearchState();
}

class _HistorySearchState extends State<HistorySearch> {
  final text = TextEditingController();
  String archive = 'visible', kind = 'all';
  bool pinned = false, thisFolder = false, busy = false;
  List<Map<String, dynamic>> items = [];
  Map<String, dynamic>? cursor, appliedFilter;
  String? error;
  Map<String, dynamic> get filter => {
    'text': text.text.trim(),
    'archive': archive,
    'pinned': pinned ? true : null,
    'kind': kind == 'all' ? null : kind,
    'root': thisFolder ? widget.chat.workspaceRoot : null,
  };
  bool get changed =>
      appliedFilter != null && jsonEncode(filter) != jsonEncode(appliedFilter);
  Future<dynamic> call(Map<String, dynamic> request) =>
      widget.chat.bridge.call({'command': 'history', 'request': request});
  @override
  void initState() {
    super.initState();
    WidgetsBinding.instance.addPostFrameCallback((_) {
      if (mounted) search();
    });
  }

  @override
  void dispose() {
    text.dispose();
    super.dispose();
  }

  Future<void> act(Future<void> Function() action) async {
    if (busy) return;
    setState(() {
      busy = true;
      error = null;
    });
    try {
      await action();
    } catch (e) {
      if (mounted) setState(() => error = '$e');
    } finally {
      if (mounted) setState(() => busy = false);
    }
  }

  Future<void> load({bool more = false}) async {
    final selected = Map<String, dynamic>.from(more ? appliedFilter! : filter);
    final result = await call({
      'action': 'search',
      'query': {'filter': selected, 'cursor': more ? cursor : null},
    }) as Map;
    if (!mounted) return;
    setState(() {
      final page = (result['items'] as List).map(
        (v) => Map<String, dynamic>.from(v as Map),
      );
      // Replace one bounded page; opening history never retains the entire DB.
      items = page.toList();
      cursor = result['nextCursor'] is Map
          ? Map<String, dynamic>.from(result['nextCursor'] as Map)
          : null;
      appliedFilter = selected;
    });
  }

  Future<void> search() => act(load);
  Future<void> organize(
    Map<String, dynamic> item, {
    bool? pin,
    bool? archived,
  }) => act(() async {
    final flags = item['organization'] as Map;
    await call({
      'action': 'organize',
      'session': item['session']['id'],
      'organization': {
        ...flags,
        'pinned': pin ?? flags['pinned'],
        'archived': archived ?? flags['archived'],
      },
    });
    await widget.chat.refreshConversationList();
    await load();
  });
  Future<void> open(Map<String, dynamic> item) => act(() async {
    await widget.chat.checkpointDraft();
    await widget.chat.navigateConversation(item['session']['id'] as String);
    if (mounted) Navigator.pop(context);
  });
  @override
  Widget build(BuildContext context) => PopScope(
    canPop: !busy,
    child: InspectorFrame(
      title: 'Search conversations',
      subtitle: 'Saved titles and messages · local search',
      canClose: !busy,
      child: Padding(
        padding: const EdgeInsets.all(16),
        child: Column(
          children: [
            Expanded(
              child: ListView(
                children: [
                  FieldLabel(
                    label: 'Search text',
                    child: TextField(
                      key: const Key('history-search-text'),
                      controller: text,
                      enabled: !busy,
                      onChanged: (_) => setState(() {}),
                      onSubmitted: (_) => search(),
                      decoration: const InputDecoration(
                        hintText: 'Find exact text in saved chats',
                      ),
                    ),
                  ),
                  const SizedBox(height: 12),
                  FieldLabel(
                    label: 'Conversations',
                    child: DropdownButtonFormField<String>(
                      key: const Key('history-archive-filter'),
                      initialValue: archive,
                      isExpanded: true,
                      items: const [
                        DropdownMenuItem(
                          value: 'visible',
                          child: Text('Current conversations'),
                        ),
                        DropdownMenuItem(
                          value: 'archived',
                          child: Text('Archived conversations'),
                        ),
                        DropdownMenuItem(
                          value: 'all',
                          child: Text('All conversations'),
                        ),
                      ],
                      onChanged: busy
                          ? null
                          : (v) => setState(() => archive = v!),
                    ),
                  ),
                  const SizedBox(height: 12),
                  ExpansionTile(
                    title: const Text('More filters'),
                    tilePadding: EdgeInsets.zero,
                    children: [
                      FieldLabel(
                        label: 'Workspace',
                        child: DropdownButtonFormField<String>(
                          key: const Key('history-kind-filter'),
                          initialValue: kind,
                          isExpanded: true,
                          items: const [
                            DropdownMenuItem(
                              value: 'all',
                              child: Text('All workspaces'),
                            ),
                            DropdownMenuItem(
                              value: 'project',
                              child: Text('Project chats'),
                            ),
                            DropdownMenuItem(
                              value: 'temporary',
                              child: Text('Temporary chats'),
                            ),
                            DropdownMenuItem(
                              value: 'side',
                              child: Text('Side chats'),
                            ),
                          ],
                          onChanged: busy
                              ? null
                              : (v) => setState(() => kind = v!),
                        ),
                      ),
                      CheckboxListTile(
                        contentPadding: EdgeInsets.zero,
                        title: const Text('Pinned only'),
                        value: pinned,
                        onChanged: busy
                            ? null
                            : (v) => setState(() => pinned = v!),
                      ),
                      if (widget.chat.workspaceRoot != null)
                        CheckboxListTile(
                          contentPadding: EdgeInsets.zero,
                          title: const Text('This working folder only'),
                          value: thisFolder,
                          onChanged: busy
                              ? null
                              : (v) => setState(() => thisFolder = v!),
                        ),
                    ],
                  ),
                  const HelpLabel(
                    help: 'Archiving hides a chat from Recents. It keeps history, drafts, goals and queued input, and does not stop a running task.',
                    child: Text('What archiving keeps'),
                  ),
                  const SizedBox(height: 12),
                  if (changed)
                    const Text(
                      'Filters changed. Search again to update these results.',
                    ),
                  if (busy) const LinearProgressIndicator(),
                  if (appliedFilter != null)
                    Text(
                      '${items.length} ${items.length == 1 ? 'conversation' : 'conversations'} on this page · pinned first',
                    ),
                  if (!busy && appliedFilter != null && items.isEmpty)
                    const Text(
                      'No matching conversations. Try shorter text or include archived chats.',
                    ),
                  for (final item in items)
                    Card(
                      child: Padding(
                        padding: const EdgeInsets.all(12),
                        child: Column(
                          crossAxisAlignment: CrossAxisAlignment.start,
                          children: [
                            Text(
                              item['session']['title'] as String,
                              style: Theme.of(context).textTheme.titleSmall,
                            ),
                            Text(
                              '${item['workspace']['kind']} · ${item['organization']['archived'] == true ? 'Archived' : 'Current'}',
                            ),
                            if (item['workspace']['root'] != null)
                              Text(
                                item['workspace']['root'] as String,
                                maxLines: 2,
                                overflow: TextOverflow.ellipsis,
                              ),
                            if (item['excerpt'] != null)
                              SelectableText(item['excerpt']['text'] as String),
                            Wrap(
                              spacing: 8,
                              children: [
                                TextButton(
                                  onPressed: busy ? null : () => open(item),
                                  child: const Text('Open conversation'),
                                ),
                                TextButton(
                                  onPressed: busy
                                      ? null
                                      : () => organize(
                                          item,
                                          pin:
                                              item['organization']['pinned'] !=
                                              true,
                                        ),
                                  child: Text(
                                    item['organization']['pinned'] == true
                                        ? 'Unpin'
                                        : 'Pin',
                                  ),
                                ),
                                TextButton(
                                  onPressed: busy
                                      ? null
                                      : () => organize(
                                          item,
                                          archived:
                                              item['organization']['archived'] !=
                                              true,
                                        ),
                                  child: Text(
                                    item['organization']['archived'] == true
                                        ? 'Unarchive'
                                        : 'Archive',
                                  ),
                                ),
                              ],
                            ),
                          ],
                        ),
                      ),
                    ),
                  if (cursor != null)
                    TextButton(
                      onPressed: busy || changed
                          ? null
                          : () => act(() => load(more: true)),
                      child: const Text('Next results'),
                    ),
                ],
              ),
            ),
            const SizedBox(height: 8),
            if (error != null)
              ConstrainedBox(
                constraints: const BoxConstraints(maxHeight: 96),
                child: SingleChildScrollView(
                  child: SelectableText(
                    error!,
                    style: TextStyle(
                      color: Theme.of(context).colorScheme.error,
                    ),
                  ),
                ),
              ),
            Wrap(
              spacing: 8,
              runSpacing: 8,
              children: [
                FilledButton(
                  onPressed: busy ? null : search,
                  child: const Text('Search'),
                ),
                TextButton(
                  onPressed: busy
                      ? null
                      : () => act(() async {
                          await call({'action': 'rebuild'});
                          await load();
                        }),
                  child: const Text('Rebuild search'),
                ),
              ],
            ),
          ],
        ),
      ),
    ),
  );
}
