import 'settings_frame.dart';

import 'dart:convert';
import 'dart:async';

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
  final String? session, initialText;
  const MemoryInspector({
    super.key,
    required this.bridge,
    this.session,
    this.initialText,
  });
  @override
  State<MemoryInspector> createState() => _MemoryInspectorState();
}

class _MemoryInspectorState extends State<MemoryInspector> {
  String? baseline;
  String get draft => jsonEncode([title.text, text.text, scope, enabled]);
  final title = TextEditingController(), text = TextEditingController();
  List<Map<String, dynamic>> items = [];
  Map<String, dynamic>? editing;
  Map<String, dynamic>? sourceReview, suggestions;
  final selected = <int>{}, savedSuggestions = <int>{};
  int? candidate, run;
  bool sourceMode = false, generating = false, stopping = false;
  bool form = false,
      enabled = true,
      folderAvailable = false,
      busy = false,
      loaded = false;
  String scope = 'all';
  String? error, notice;
  Map<String, dynamic>? automaticPolicy, automaticAttempt;

  @override
  void initState() {
    super.initState();
    _refresh().then((_) {
      if (mounted && widget.initialText != null) {
        _edit();
        text.text = widget.initialText!;
        title.text = 'Remembered preference';
        baseline = '';
      }
    });
  }

  @override
  void dispose() {
    final token = suggestions?['token'] ?? sourceReview?['token'];
    if (token != null) {
      unawaited(
        _call('discardMemoryReview', {'token': token}).catchError((_) => null),
      );
    }
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
      automaticPolicy = (result['automaticPolicy'] as Map?)
          ?.cast<String, dynamic>();
      automaticAttempt = (result['automaticAttempt'] as Map?)
          ?.cast<String, dynamic>();
      loaded = true;
    });
  }

  Future<void> _refresh() => _act(() async {
    await _load();
    if (editing != null && mounted) {
      final current = items.where((i) => i['id'] == editing!['id']);
      if (current.isNotEmpty) {
        setState(() => editing = current.first);
      } else {
        throw const FormatException(
          'This preference was removed. Copy your edits, then create a new preference.',
        );
      }
    }
  });
  Future<void> _close() async {
    if (busy) return;
    if (form &&
        baseline != draft &&
        !await resolveSettingsDraft(
          context,
          save: () async {
            await _save();
            return !form;
          },
        )) {
      return;
    }
    if (mounted) Navigator.pop(context);
  }

  Future<void> _setAutomatic(bool value) => _act(() async {
    await _call('setAutomaticMemory', {
      'enabled': value,
      'revision': automaticPolicy!['revision'],
    });
    await _load();
  });
  void _edit([Map<String, dynamic>? item]) {
    if (busy) return;
    setState(() {
      editing = item;
      candidate = null;
      form = true;
      title.text = item?['title'] as String? ?? '';
      text.text = item?['text'] as String? ?? '';
      enabled = item?['enabled'] as bool? ?? true;
      scope = item?['scope'] as String? ?? (folderAvailable ? 'folder' : 'all');
      error = null;
      notice = null;
    });
    baseline = draft;
  }

  void _cancel() {
    if (busy) return;
    setState(() {
      form = false;
      editing = null;
      candidate = null;
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
    final fromSuggestion = candidate != null;
    await _call(fromSuggestion ? 'saveMemorySuggestion' : 'saveMemory', {
      'scope': scope,
      'title': title.text,
      'text': text.text,
      'enabled': enabled,
      if (editing != null) 'id': editing!['id'],
      if (editing != null) 'revision': editing!['revision'],
      if (fromSuggestion) 'token': suggestions!['token'],
      if (fromSuggestion) 'index': candidate,
    });
    if (mounted) {
      setState(() {
        form = false;
        editing = null;
        if (fromSuggestion) savedSuggestions.add(candidate!);
        candidate = null;
        notice =
            'Preference saved${enabled ? ' and enabled for new messages' : ''}.';
      });
    }
    try {
      await _load();
    } catch (_) {
      if (mounted) {
        setState(() => error = 'Preference saved. Refresh to update the list.');
      }
    }
  });

  Future<void> _loadSources({bool keepSelection = false}) async {
    final previous = {
      for (final item in (sourceReview?['items'] as List? ?? []))
        item['messageId']: item['text'],
    };
    final result = (await _call('reviewMemorySources')) as Map;
    if (!mounted) return;
    setState(() {
      sourceReview = result.cast<String, dynamic>();
      sourceMode = true;
      suggestions = null;
      savedSuggestions.clear();
      if (keepSelection) {
        final current = {
          for (final item in result['items'] as List)
            item['messageId']: item['text'],
        };
        selected.removeWhere(
          (id) => current[id] == null || current[id] != previous[id],
        );
      } else {
        selected.clear();
      }
    });
  }

  Future<void> _sources() => _act(_loadSources);
  Future<void> _discard() => _act(() async {
    final token = suggestions?['token'] ?? sourceReview?['token'];
    if (token != null) await _call('discardMemoryReview', {'token': token});
    if (!mounted) return;
    setState(() {
      sourceMode = false;
      sourceReview = null;
      suggestions = null;
      selected.clear();
      savedSuggestions.clear();
      candidate = null;
      form = false;
    });
  });

  Future<void> _generate() => _act(() async {
    final entries = (sourceReview?['items'] as List? ?? []).where(
      (v) => selected.contains(v['messageId']),
    );
    final bytes = entries.fold<int>(
      0,
      (n, v) => n + utf8.encode(v['text'] as String).length,
    );
    if (selected.isEmpty || selected.length > 6 || bytes > 8192) {
      throw const FormatException('Choose 1–6 messages within 8 KiB.');
    }
    final number = DateTime.now().microsecondsSinceEpoch;
    setState(() {
      generating = true;
      stopping = false;
      run = number;
    });
    try {
      await _call('suggestMemories', {
        'id': number,
        'token': sourceReview!['token'],
        'messageIds': selected.toList(),
      });
      final deadline = DateTime.now().add(const Duration(seconds: 40));
      while (mounted) {
        final events = await _call('poll', {'id': number}) as List;
        for (final event in events) {
          if (event['type'] != 'done') continue;
          if (event['error'] != null) throw event['error'] as String;
          if (stopping) {
            if (event['memorySuggestions']?['token'] != null) {
              await _call('discardMemoryReview', {
                'token': event['memorySuggestions']['token'],
              });
            }
            throw 'Suggestions stopped. Nothing was saved.';
          }
          setState(
            () => suggestions = (event['memorySuggestions'] as Map)
                .cast<String, dynamic>(),
          );
          return;
        }
        if (DateTime.now().isAfter(deadline)) {
          await _call('cancel', {'id': number});
          // Release the single native run before allowing other actions.
          await _call('shutdown');
          throw 'Suggestions did not finish. Nothing was saved. Try again.';
        }
        await Future<void>.delayed(const Duration(milliseconds: 80));
      }
    } catch (_) {
      // Local-only refresh restores an expired/consumed review; no model retry.
      await _loadSources(keepSelection: true);
      rethrow;
    } finally {
      if (mounted) {
        setState(() {
          generating = false;
          stopping = false;
          run = null;
        });
      }
    }
  });

  Future<void> _stop() async {
    if (!generating || stopping || run == null) return;
    setState(() => stopping = true);
    try {
      await _call('cancel', {'id': run});
    } catch (failure) {
      if (mounted) {
        setState(() {
          error = failure.toString();
          stopping = false;
        });
      }
    }
  }

  void _reviewCandidate(int index) {
    if (busy) return;
    final item = suggestions!['items'][index] as Map;
    setState(() {
      candidate = index;
      editing = null;
      form = true;
      title.text = item['title'] as String;
      text.text = item['text'] as String;
      scope = folderAvailable ? 'folder' : 'all';
      enabled = true;
      error = null;
      notice = null;
    });
  }

  Widget _origin(
    Map item,
    Palette p, {
    bool? available,
    bool automatic = false,
    bool showQuote = true,
  }) => Padding(
    padding: const EdgeInsets.only(top: 8),
    child: Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        Text(
          '${automatic ? 'From your message (learned automatically)' : 'From a reviewed chat'} · message ${item['messageId']} · ${item['model'] ?? suggestions?['model'] ?? ''}',
          style: TextStyle(color: p.muted, fontSize: 11),
        ),
        if (available == false)
          Text(
            'Source message is no longer available; the original quote is retained.',
            style: TextStyle(color: p.muted, fontSize: 11),
          ),
        if (showQuote) SelectableText(item['quote'] as String),
      ],
    ),
  );
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
    reportSettingsDraft(
      context,
      dirty: () => form && baseline != draft,
      save: () async {
        await _save();
        return !form;
      },
    );
    return PopScope(
      canPop: !busy && !(form && baseline != draft),
      onPopInvokedWithResult: (didPop, _) {
        if (!didPop && SettingsEmbedding.of(context) == null) _close();
      },
      child: InspectorFrame(
        title: 'Memory',
        subtitle: 'How Dolores remembers your preferences',
        canClose: !busy,
        onClose: _close,
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
                  ExpansionTile(
                    title: const Text('Memory details'),
                    children: [
                      Text(
                        'Delete or disable preferences to stop using them in new messages. Editing or disabling a learned preference protects it from automatic replacement; past replies keep their original context.',
                        style: TextStyle(color: p.muted, fontSize: 12),
                      ),
                    ],
                  ),
                  if (automaticPolicy != null && !form && !sourceMode) ...[
                    SwitchListTile(
                      key: const Key('automatic-memory'),
                      contentPadding: EdgeInsets.zero,
                      title: const Text('Learn preferences automatically'),
                      value: automaticPolicy!['enabled'] == true,
                      onChanged: busy ? null : _setAutomatic,
                      subtitle: const Text(
                        'May send your explicit preferences in one small extra model request.',
                      ),
                    ),
                    ExpansionTile(
                      title: const Text('Learning details'),
                      children: [
                        Text(
                          'Eligible messages may use one extra model request, up to 10 seconds and 512 output tokens. No files, tools or assistant replies are used. Quoted text and common sensitive patterns are skipped; this is a conservative filter, not a complete classifier.',
                          style: TextStyle(color: p.muted, fontSize: 12),
                        ),
                      ],
                    ),
                    if (automaticAttempt != null)
                      ExpansionTile(
                        key: const Key('automatic-memory-attempt'),
                        tilePadding: EdgeInsets.zero,
                        title: const Text(
                          'Latest learning activity in this chat',
                        ),
                        subtitle: Text(
                          '${automaticAttempt!['status']} · ${automaticAttempt!['note']}',
                        ),
                        children: [
                          Text(
                            'Your message ${automaticAttempt!['messageId']} · ${DateTime.fromMillisecondsSinceEpoch(automaticAttempt!['updatedAt'] as int).toLocal()}',
                          ),
                          if (automaticAttempt!['usage'] is Map)
                            Text(
                              'Learning tokens: ${automaticAttempt!['usage']['inputTokens'] ?? '—'} in · ${automaticAttempt!['usage']['outputTokens'] ?? '—'} out',
                            ),
                        ],
                      ),
                  ],
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
                      candidate != null
                          ? 'Review suggestion'
                          : editing == null
                          ? 'New preference'
                          : 'Edit preference',
                      style: const TextStyle(fontWeight: FontWeight.w600),
                    ),
                    const SizedBox(height: 12),
                    if (candidate != null)
                      _origin(suggestions!['items'][candidate!], p),
                    if (editing?['origin'] is Map)
                      _origin(
                        editing!['origin'],
                        p,
                        available: editing!['originAvailable'] as bool?,
                        automatic: editing!['source'] == 'automatic',
                      ),
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
                  ] else if (sourceMode) ...[
                    if (suggestions == null) ...[
                      const Text(
                        'Choose messages to share',
                        style: TextStyle(fontWeight: FontWeight.w600),
                      ),
                      const SizedBox(height: 8),
                      Text(
                        'Only these completed messages you wrote will be sent to ${sourceReview?['model'] ?? 'the selected model'}. Replies, tools, files, saved preferences and your draft are excluded. Choose at most 6 messages within 8 KiB; check that they contain no secrets.',
                        style: TextStyle(color: p.muted, fontSize: 12),
                      ),
                      Text(
                        '${selected.length} selected · latest 20 completed messages${sourceReview?['hasOlder'] == true ? ' · Earlier history excluded' : ''}',
                      ),
                      if ((sourceReview?['items'] as List? ?? []).isEmpty)
                        const Text(
                          'No completed messages in this chat yet. Add a preference manually or finish a conversation first.',
                        ),
                      for (final item
                          in (sourceReview?['items'] as List? ?? []))
                        Card(
                          elevation: 0,
                          color: p.surface,
                          child: Padding(
                            padding: const EdgeInsets.all(12),
                            child: Column(
                              crossAxisAlignment: CrossAxisAlignment.start,
                              children: [
                                CheckboxListTile(
                                  key: ValueKey(
                                    'memory-source-${item['messageId']}',
                                  ),
                                  contentPadding: EdgeInsets.zero,
                                  value: selected.contains(item['messageId']),
                                  title: Text(
                                    'Your message ${item['messageId']}',
                                  ),
                                  onChanged: busy
                                      ? null
                                      : (value) => setState(() {
                                          if (value == true) {
                                            selected.add(
                                              item['messageId'] as int,
                                            );
                                          } else {
                                            selected.remove(item['messageId']);
                                          }
                                        }),
                                ),
                                SelectableText(item['text'] as String),
                              ],
                            ),
                          ),
                        ),
                    ] else ...[
                      Text(
                        'Suggested by ${suggestions!['model']}',
                        style: const TextStyle(fontWeight: FontWeight.w600),
                      ),
                      const Text(
                        'Review and edit each draft before saving. Suggestions can be wrong; the source quote does not prove the wording is accurate. Saving retains the quote locally and in reply details and exports, even after deleting the source chat.',
                      ),
                      if ((suggestions!['items'] as List).isEmpty)
                        const Text(
                          'No stable preferences found in the selected messages. Nothing was saved.',
                        ),
                      for (
                        var i = 0;
                        i < (suggestions!['items'] as List).length;
                        i++
                      )
                        Card(
                          elevation: 0,
                          color: p.surface,
                          child: Padding(
                            padding: const EdgeInsets.all(16),
                            child: Column(
                              crossAxisAlignment: CrossAxisAlignment.start,
                              children: [
                                Text(
                                  suggestions!['items'][i]['title'] as String,
                                  style: const TextStyle(
                                    fontWeight: FontWeight.w600,
                                  ),
                                ),
                                SelectableText(
                                  suggestions!['items'][i]['text'] as String,
                                ),
                                _origin(suggestions!['items'][i], p),
                                TextButton(
                                  key: ValueKey('review-suggestion-$i'),
                                  onPressed:
                                      busy || savedSuggestions.contains(i)
                                      ? null
                                      : () => _reviewCandidate(i),
                                  child: Text(
                                    savedSuggestions.contains(i)
                                        ? 'Saved'
                                        : 'Review suggestion',
                                  ),
                                ),
                              ],
                            ),
                          ),
                        ),
                    ],
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
                                '${item['scope'] == 'folder' ? 'This working folder' : 'All chats'} · ${item['enabled'] == true ? 'Enabled' : 'Disabled'} · ${item['source'] == 'automatic'
                                    ? 'Learned automatically'
                                    : item['origin'] is Map
                                    ? 'From a reviewed chat'
                                    : 'Added by you'}',
                                style: TextStyle(fontSize: 11, color: p.muted),
                              ),
                              const SizedBox(height: 8),
                              if (item['origin'] is Map)
                                _origin(
                                  item['origin'],
                                  p,
                                  available: item['originAvailable'] as bool?,
                                  automatic: item['source'] == 'automatic',
                                  showQuote:
                                      item['source'] != 'automatic' ||
                                      item['text'] != item['origin']['quote'],
                                ),
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
                  if (!sourceMode)
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
                  ] else if (sourceMode) ...[
                    if (generating)
                      const Padding(
                        padding: EdgeInsets.symmetric(
                          horizontal: 8,
                          vertical: 12,
                        ),
                        child: Text('Generating suggestions…'),
                      ),
                    if (generating)
                      TextButton(
                        key: const Key('stop-memory-suggestions'),
                        onPressed: stopping ? null : _stop,
                        child: Text(stopping ? 'Stopping…' : 'Stop'),
                      )
                    else ...[
                      TextButton(
                        key: const Key('discard-memory-suggestions'),
                        onPressed: busy ? null : _discard,
                        child: const Text('Discard'),
                      ),
                      TextButton(
                        key: const Key('refresh-memory-sources'),
                        onPressed: busy ? null : _sources,
                        child: const Text('Review messages'),
                      ),
                      if (suggestions == null)
                        FilledButton(
                          key: const Key('generate-memory-suggestions'),
                          onPressed: busy || selected.isEmpty
                              ? null
                              : _generate,
                          child: const Text('Generate suggestions'),
                        ),
                    ],
                  ] else ...[
                    if (widget.session != null)
                      TextButton(
                        key: const Key('suggest-from-chat'),
                        onPressed: busy || !loaded ? null : _sources,
                        child: const Text('Suggest from this chat'),
                      ),
                    FilledButton(
                      key: const Key('new-memory'),
                      onPressed: busy || !loaded ? null : () => _edit(),
                      child: const Text('New preference'),
                    ),
                  ],
                ],
              ),
            ),
          ],
        ),
      ),
    );
  }
}
