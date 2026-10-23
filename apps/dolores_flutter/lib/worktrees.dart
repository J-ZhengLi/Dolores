import 'dart:async';

import 'package:flutter/material.dart';
import 'package:flutter/services.dart';

import 'chat.dart';
import 'inspector.dart';
import 'worktree_transfer.dart';

Future<void> showWorktrees(
  BuildContext context,
  ChatController chat, {
  Future<void> Function(String)? onSelect,
  bool forPromptBranch = false,
}) => showDialog<void>(
  context: context,
  barrierDismissible: false,
  builder: (_) => WorktreesInspector(
    chat: chat,
    onSelect: onSelect,
    forPromptBranch: forPromptBranch,
  ),
);

/// Creation is a reviewed host operation. Polling and reopening never repeat it.
class WorktreesInspector extends StatefulWidget {
  final ChatController chat;
  final Future<void> Function(String)? onSelect;
  final bool forPromptBranch;
  const WorktreesInspector({
    super.key,
    required this.chat,
    this.onSelect,
    this.forPromptBranch = false,
  });
  @override
  State<WorktreesInspector> createState() => _WorktreesInspectorState();
}

class _WorktreesInspectorState extends State<WorktreesInspector> {
  late final String source = widget.chat.session ?? '';
  final base = TextEditingController();
  final scroll = ScrollController();
  final records = <String, Map>{};
  String setup = 'skip';
  Map? review, selected;
  String? reviewOwner, lifecycleNotice, archiveDirectory, lifecycleOwner;
  String? error, job, jobOwner, operation;
  int? cursor;
  bool busy = false, polling = false;
  Timer? timer;
  void showError(String failure) {
    if (!mounted) return;
    setState(() => error = failure);
    if (scroll.hasClients) scroll.jumpTo(0);
  }

  bool get locked => busy || job != null;
  bool get sourceIdle =>
      !widget.chat.busy && !widget.chat.changing && !widget.chat.loading;
  @override
  void initState() {
    super.initState();
    refresh();
  }

  Future<dynamic> call(
    String action, {
    String? owner,
    Map<String, dynamic> fields = const {},
  }) => widget.chat.bridge.call({
    'command': 'worktrees',
    'session': owner ?? source,
    'request': {'action': action, ...fields},
  });

  Future<void> act(Future<void> Function() action) async {
    if (locked) return;
    setState(() {
      busy = true;
      error = null;
    });
    try {
      await action();
    } catch (failure) {
      showError('$failure');
    } finally {
      if (mounted) setState(() => busy = false);
    }
  }

  Future<void> discard() async {
    final token = review?['token'];
    if (token != null) {
      await call('discardReview', owner: reviewOwner, fields: {'token': token});
      review = null;
      reviewOwner = null;
    }
  }

  Future<void> listing({bool more = false}) async {
    final page =
        await call('list', fields: {'cursor': more ? cursor : null}) as Map;
    if (!mounted) return;
    setState(() {
      if (!more) records.clear();
      for (final record in page['items'] as List) {
        records[record['session'] as String] = record as Map;
      }
      cursor = page['nextCursor'] as int?;
      if (selected != null) {
        selected = records[selected!['session']] ?? selected;
      }
    });
  }

  Future<void> refresh() => act(() async {
    await discard();
    await listing();
    if (source.isNotEmpty) {
      final record = await call('inspect');
      if (mounted && record is Map) {
        setState(() {
          selected = record;
          records[record['session'] as String] = record;
        });
      }
    }
  });

  Future<void> start(
    String action, {
    String? owner,
    Map<String, dynamic> fields = const {},
  }) async {
    final result = await call(action, owner: owner, fields: fields) as Map;
    if (!mounted) return;
    setState(() {
      job = result['job'] as String;
      jobOwner = owner ?? source;
      operation = action;
    });
    timer = Timer(Duration.zero, poll);
  }

  Future<void> poll() async {
    if (!mounted || job == null || polling) return;
    timer?.cancel();
    polling = true;
    try {
      final result =
          await call('poll', owner: jobOwner, fields: {'job': job}) as Map;
      if (!mounted) return;
      if (result['done'] != true) {
        timer = Timer(const Duration(milliseconds: 250), poll);
        return;
      }
      final completed = operation;
      setState(() {
        job = null;
        error = result['error'] as String?;
        busy = true;
      });
      if (error != null) showError(error!);
      final value = result['value'];
      if (value is Map) {
        setState(() {
          if (completed == 'reviewCreate' ||
              completed == 'reviewArchive' ||
              completed == 'reviewRestore') {
            review = value;
            reviewOwner = jobOwner;
          } else if (value['record'] is Map) {
            selected = value['record'] as Map;
            records[selected!['session'] as String] = selected!;
            lifecycleOwner = selected!['session'] as String;
            lifecycleNotice = value['notice'] as String?;
            archiveDirectory = value['directory'] as String?;
          }
        });
      }
      // A creation failure may have saved a new chat and recoverable intent.
      await listing();
      await widget.chat.refreshConversationList();
    } catch (failure) {
      showError('$failure');
      // A status transport failure is uncertain. Check status explicitly;
      // never resubmit creation just because its response was unavailable.
    } finally {
      polling = false;
      if (mounted) setState(() => busy = false);
    }
  }

  Future<void> close() async {
    if (locked) return;
    await act(() async {
      await discard();
      if (mounted) Navigator.pop(context);
    });
  }

  Future<void> open() => act(() async {
    if (!sourceIdle) {
      throw StateError('Finish or stop this task before switching chats.');
    }
    // A failed draft save must leave this panel and the source text available.
    await widget.chat.checkpointDraft();
    await widget.chat.refreshConversationList();
    final target = selected!['session'] as String;
    if (!widget.chat.sessions.any((s) => s['id'] == target)) {
      // Recent history is paged: query the target rather than treating absence
      // from the first page as deletion. This is read-only and never recreates it.
      await widget.chat.bridge.call({
        'command': 'workspace',
        'session': target,
      });
    }
    await discard();
    if (widget.onSelect != null) {
      await widget.onSelect!(target);
    } else {
      await widget.chat.select(target);
      if (widget.chat.session != target) {
        throw StateError(
          widget.chat.error ?? 'Could not open this chat. Its retained checkout is still available.',
        );
      }
    }
    if (mounted) Navigator.pop(context);
  });

  @override
  void dispose() {
    timer?.cancel();
    base.dispose();
    scroll.dispose();
    super.dispose();
  }

  Widget path(String label, Object? value) => Padding(
    padding: const EdgeInsets.only(bottom: 10),
    child: Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        Text(label, style: Theme.of(context).textTheme.labelMedium),
        SelectableText('${value ?? ''}'),
      ],
    ),
  );

  Widget details(Map record) => Column(
    crossAxisAlignment: CrossAxisAlignment.start,
    children: [
      Text(
        'Checkout · ${record['state']}',
        style: Theme.of(context).textTheme.titleMedium,
      ),
      const SizedBox(height: 8),
      path('Working folder', record['root']),
      path('Original folder', record['sourceRoot']),
      path('Base', record['baseRef']),
      path('Commit', record['baseCommit']),
      if (record['snapshot'] != null)
        path('Retained snapshot', record['snapshot']),
      Text(record['notice'] as String? ?? ''),
      if (lifecycleOwner == record['session'] && lifecycleNotice != null)
        Text(lifecycleNotice!),
      if (lifecycleOwner == record['session'] && archiveDirectory != null) ...[
        path('Retained archive folder', archiveDirectory),
        TextButton.icon(
          onPressed: () =>
              Clipboard.setData(ClipboardData(text: archiveDirectory!)),
          icon: const Icon(Icons.copy, size: 16),
          label: const Text('Copy archive folder'),
        ),
      ],
      const SizedBox(height: 8),
      Wrap(
        spacing: 8,
        children: [
          if (record['state'] == 'ready')
            OutlinedButton(
              onPressed: locked || !sourceIdle
                  ? null
                  : () => act(() async {
                      await discard();
                      await start(
                        'reviewArchive',
                        owner: record['session'] as String,
                      );
                    }),
              child: const Text('Review archive'),
            ),
          if (record['state'] == 'archived' ||
              record['state'] == 'recovery' &&
                  record['recoveryFrom'] == 'restoring')
            OutlinedButton(
              onPressed: locked || !sourceIdle
                  ? null
                  : () => act(() async {
                      await discard();
                      await start(
                        'reviewRestore',
                        owner: record['session'] as String,
                      );
                    }),
              child: Text(
                record['state'] == 'archived'
                    ? 'Review restore'
                    : 'Review restore continuation',
              ),
            ),
          if (record['state'] == 'recovery' &&
              ['archiving', 'restoring'].contains(record['recoveryFrom']))
            OutlinedButton(
              onPressed: locked || !sourceIdle
                  ? null
                  : () => act(
                      () => start(
                        'inspectLifecycle',
                        owner: record['session'] as String,
                        fields: {'revision': record['revision']},
                      ),
                    ),
              child: const Text('Inspect archive recovery'),
            ),
          if (record['state'] == 'ready')
            TextButton.icon(
              onPressed: locked || !sourceIdle
                  ? null
                  : () => showWorktreeTransfer(context, widget.chat, record),
              icon: const Icon(Icons.compare_arrows, size: 18),
              label: const Text('Transfer changes'),
            ),
          TextButton.icon(
            onPressed: locked || !sourceIdle ? null : open,
            icon: const Icon(Icons.chat_bubble_outline, size: 18),
            label: Text(
              widget.forPromptBranch ? 'Use for edited branch' : 'Open chat',
            ),
          ),
          if (record['state'] == 'recovery' &&
              record['recoveryFrom'] == 'creating')
            OutlinedButton(
              onPressed: locked || !sourceIdle
                  ? null
                  : () => act(
                      () => start(
                        'reconcileCreation',
                        owner: record['session'] as String,
                        fields: {'revision': record['revision']},
                      ),
                    ),
              child: const Text('Inspect existing checkout'),
            ),
        ],
      ),
      if (record['state'] == 'recovery')
        const Text(
          'Keep the folder and its Git registration. Inspection will not repeat checkout or setup. If inspection fails, use the paths above for manual recovery.',
        ),
    ],
  );

  @override
  Widget build(BuildContext context) {
    final lifecycle =
        review?['kind'] == 'archive' || review?['kind'] == 'restore';
    final reviewed = !lifecycle ? (review?['record'] as Map?) : null;
    return PopScope(
      canPop: false,
      onPopInvokedWithResult: (didPop, _) {
        if (!didPop) close();
      },
      child: InspectorFrame(
        title: widget.forPromptBranch
            ? 'Choose a managed worktree'
            : 'Worktrees',
        subtitle: widget.forPromptBranch
            ? 'Create or choose an empty Ready chat from this source folder'
            : 'Separate working folders for independent tasks',
        canClose: !locked,
        onClose: close,
        child: ListView(
          controller: scroll,
          padding: const EdgeInsets.all(20),
          children: [
            if (error != null) ...[
              SelectableText(
                error!,
                style: TextStyle(color: Theme.of(context).colorScheme.error),
              ),
              const SizedBox(height: 12),
            ],
            if (locked) ...[
              const LinearProgressIndicator(),
              const SizedBox(height: 8),
              const Text(
                'Working… Keep this panel open until the result is saved.',
              ),
              if (job != null)
                Wrap(
                  spacing: 8,
                  children: [
                    TextButton(
                      onPressed: polling ? null : poll,
                      child: const Text('Check status'),
                    ),
                    TextButton(
                      onPressed: () async {
                        try {
                          await call(
                            'cancel',
                            owner: jobOwner,
                            fields: {'job': job},
                          );
                        } catch (failure) {
                          if (mounted) setState(() => error = '$failure');
                        }
                      },
                      child: const Text('Stop operation'),
                    ),
                  ],
                ),
              const SizedBox(height: 16),
            ],
            if (lifecycle) ...[
              Text(
                review!['kind'] == 'archive'
                    ? 'Review checkout archive'
                    : 'Review checkout restore',
                style: Theme.of(context).textTheme.titleMedium,
              ),
              const SizedBox(height: 12),
              path('Managed working folder', review!['record']?['root']),
              path('Original folder kept', review!['record']?['sourceRoot']),
              path('Retained snapshot', review!['snapshot']),
              path('Retained archive folder', review!['directory']),
              Text(
                '${review!['files']} files · ${review!['directories']} folders · ${review!['bytes']} bytes',
              ),
              const SizedBox(height: 8),
              Text(review!['notice'] as String),
              const SizedBox(height: 12),
              Wrap(
                spacing: 8,
                children: [
                  FilledButton(
                    onPressed: locked || !sourceIdle
                        ? null
                        : () => act(() async {
                            await widget.chat.checkpointDraft();
                            final token = review!['token'];
                            final action = review!['kind'] == 'archive'
                                ? 'archive'
                                : 'restore';
                            final owner = reviewOwner;
                            await start(
                              action,
                              owner: owner,
                              fields: {'token': token},
                            );
                            review = null;
                            reviewOwner = null;
                          }),
                    child: Text(
                      review!['kind'] == 'archive'
                          ? 'Archive checkout'
                          : 'Restore checkout',
                    ),
                  ),
                  TextButton(
                    onPressed: locked ? null : () => act(discard),
                    child: const Text('Keep as is'),
                  ),
                ],
              ),
            ] else if (reviewed != null) ...[
              Text(
                'Review new worktree',
                style: Theme.of(context).textTheme.titleMedium,
              ),
              const SizedBox(height: 8),
              path('Original folder', reviewed['sourceRoot']),
              path('New working folder', reviewed['root']),
              path('Base', reviewed['baseRef']),
              path('Exact commit', reviewed['baseCommit']),
              Text(
                reviewed['setup'] == 'manual'
                    ? 'Run setup commands separately after opening the new chat.'
                    : 'Skip setup.',
              ),
              const Text(
                'Starts at this committed revision. Existing uncommitted files stay in the original folder. Hooks, filters and setup scripts are skipped.',
              ),
              Wrap(
                spacing: 8,
                children: [
                  FilledButton(
                    onPressed: locked || !sourceIdle
                        ? null
                        : () => act(() async {
                            final token = review!['token'];
                            await start('create', fields: {'token': token});
                            review = null;
                          }),
                    child: const Text('Create worktree'),
                  ),
                  TextButton(
                    onPressed: locked ? null : () => act(discard),
                    child: const Text('Change choices'),
                  ),
                ],
              ),
            ] else if (widget.chat.workspaceKind == 'project' &&
                source.isNotEmpty) ...[
              Text(
                'New worktree',
                style: Theme.of(context).textTheme.titleMedium,
              ),
              const SizedBox(height: 12),
              TextField(
                controller: base,
                enabled: !locked,
                decoration: const InputDecoration(
                  labelText: 'Base branch, tag or commit',
                  hintText: 'For example: main',
                ),
                onChanged: (_) => setState(() {}),
              ),
              const SizedBox(height: 12),
              DropdownButtonFormField<String>(
                initialValue: setup,
                decoration: const InputDecoration(labelText: 'Setup'),
                items: const [
                  DropdownMenuItem(value: 'skip', child: Text('Skip setup')),
                  DropdownMenuItem(
                    value: 'manual',
                    child: Text('Set up manually'),
                  ),
                ],
                onChanged: locked
                    ? null
                    : (value) => setState(() => setup = value!),
              ),
              const SizedBox(height: 12),
              Align(
                alignment: Alignment.centerLeft,
                child: OutlinedButton(
                  onPressed: locked || !sourceIdle || base.text.trim().isEmpty
                      ? null
                      : () => act(
                          () => start(
                            'reviewCreate',
                            fields: {'base': base.text.trim(), 'setup': setup},
                          ),
                        ),
                  child: const Text('Review creation'),
                ),
              ),
            ] else
              const Text('Open a project chat to create a worktree.'),
            const Divider(height: 32),
            Row(
              children: [
                const Expanded(child: Text('Retained worktrees')),
                TextButton(
                  onPressed: locked ? null : refresh,
                  child: const Text('Refresh'),
                ),
              ],
            ),
            if (records.isEmpty) const Text('No managed worktrees yet.'),
            for (final record in records.values)
              ListTile(
                contentPadding: EdgeInsets.zero,
                selected: selected?['session'] == record['session'],
                leading: const Icon(Icons.account_tree_outlined),
                title: Text('${record['baseRef']} · ${record['state']}'),
                subtitle: Text(
                  record['root'] as String,
                  maxLines: 2,
                  overflow: TextOverflow.ellipsis,
                ),
                onTap: locked ? null : () => setState(() => selected = record),
              ),
            if (cursor != null)
              TextButton(
                onPressed: locked ? null : () => act(() => listing(more: true)),
                child: const Text('Load more'),
              ),
            if (selected != null && !lifecycle) ...[
              const Divider(height: 24),
              details(selected!),
            ],
          ],
        ),
      ),
    );
  }
}
