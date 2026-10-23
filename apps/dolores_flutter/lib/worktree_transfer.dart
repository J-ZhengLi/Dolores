import 'dart:async';

import 'package:flutter/material.dart';
import 'package:flutter/services.dart';

import 'chat.dart';
import 'inspector.dart';

Future<void> showWorktreeTransfer(
  BuildContext context,
  ChatController chat,
  Map record,
) => showDialog<void>(
  context: context,
  barrierDismissible: false,
  builder: (_) => WorktreeTransferInspector(chat: chat, record: record),
);

class WorktreeTransferInspector extends StatefulWidget {
  final ChatController chat;
  final Map record;
  const WorktreeTransferInspector({
    super.key,
    required this.chat,
    required this.record,
  });
  @override
  State<WorktreeTransferInspector> createState() =>
      _WorktreeTransferInspectorState();
}

class _WorktreeTransferInspectorState extends State<WorktreeTransferInspector> {
  final scroll = ScrollController();
  final selected = <String>{};
  final destinations = <String, Map>{};
  String? destination, job, operation, error;
  List<String> paths = [];
  List<Map> retained = [];
  Map? review, result;
  bool busy = false, polling = false, older = false;
  Map? cursor;
  Timer? timer;
  bool get locked => busy || job != null;
  String get owner => widget.record['session'] as String;
  @override
  void initState() {
    super.initState();
    for (final row in widget.chat.sessions) {
      addDestination(row);
    }
    destination = destinations.entries
        .where(
          (e) => e.value['workspace']?['root'] == widget.record['sourceRoot'],
        )
        .firstOrNull
        ?.key;
    refresh();
  }

  void addDestination(Map row) {
    final root = row['workspace']?['root'];
    if (root is String && root != widget.record['root']) {
      destinations[row['id'] as String] = row;
    }
  }

  Future<dynamic> call(
    String action, [
    Map<String, dynamic> fields = const {},
  ]) => widget.chat.bridge.call({
    'command': 'worktrees',
    'session': owner,
    'request': {'action': action, ...fields},
  });

  void fail(Object failure) {
    if (!mounted) return;
    setState(() => error = '$failure');
    if (scroll.hasClients) scroll.jumpTo(0);
  }

  Future<void> act(Future<void> Function() action) async {
    if (locked) return;
    setState(() {
      busy = true;
      error = null;
    });
    try {
      await action();
    } catch (failure) {
      fail(failure);
    } finally {
      if (mounted) setState(() => busy = false);
    }
  }

  Future<void> discard() async {
    if (review != null) {
      await call('discardReview', {'token': review!['token']});
    }
    review = null;
  }

  Future<void> listing() async {
    final page = await call('listTransfers') as Map;
    if (mounted) setState(() => retained = (page['items'] as List).cast<Map>());
  }

  Future<void> refresh() => act(() async {
    await discard();
    await listing();
  });

  Future<void> loadDestinations() => act(() async {
    final page = await widget.chat.bridge.call({
      'command': 'sessionsPage',
      'cursor': cursor,
      'newer': false,
    }) as Map;
    if (!mounted) return;
    setState(() {
      final items = (page['items'] as List).cast<Map>();
      for (final row in items) {
        addDestination(row);
      }
      older = page['hasOlder'] == true;
      if (items.isNotEmpty) {
        cursor = {'id': items.last['id'], 'updatedAt': items.last['updatedAt']};
      }
    });
  });

  Future<void> start(
    String action, [
    Map<String, dynamic> fields = const {},
  ]) async {
    final value = await call(action, fields) as Map;
    if (!mounted) return;
    setState(() {
      job = value['job'] as String;
      operation = action;
      result = null;
    });
    timer = Timer(Duration.zero, poll);
  }

  Future<void> poll() async {
    if (job == null || polling || !mounted) return;
    timer?.cancel();
    polling = true;
    try {
      final value = await call('poll', {'job': job}) as Map;
      if (!mounted) return;
      if (value['done'] != true) {
        timer = Timer(const Duration(milliseconds: 250), poll);
        return;
      }
      final completed = operation;
      setState(() {
        job = null;
        busy = true;
      });
      if (value['error'] != null) fail(value['error']);
      if (value['value'] is Map) {
        final outcome = value['value'] as Map;
        setState(() {
          if (completed == 'compareTransfer') {
            paths = (outcome['paths'] as List).cast<String>();
            selected.removeWhere((path) => !paths.contains(path));
          } else if (completed == 'reviewTransfer') {
            review = outcome;
          } else {
            result = outcome;
          }
        });
        if (completed == 'applyTransfer' && outcome['applied'] != true) {
          fail(
            outcome['notice'] ??
                'Inspect retained versions before another transfer.',
          );
        }
      }
      await listing();
    } catch (failure) {
      fail(failure);
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

  @override
  void dispose() {
    timer?.cancel();
    scroll.dispose();
    super.dispose();
  }

  Widget detail(String label, Object? value) => Padding(
    padding: const EdgeInsets.only(bottom: 10),
    child: Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        Text(label, style: Theme.of(context).textTheme.labelMedium),
        SelectableText('${value ?? ''}'),
      ],
    ),
  );

  Widget evidence(Map item) {
    final receipt = item['receipt'] as Map;
    final outcome = item['result'] as Map;
    return ExpansionTile(
      title: Text('Transfer · ${outcome['state']}'),
      subtitle: Text(
        (receipt['paths'] as List).join(', '),
        maxLines: 2,
        overflow: TextOverflow.ellipsis,
      ),
      childrenPadding: const EdgeInsets.all(12),
      expandedCrossAxisAlignment: CrossAxisAlignment.start,
      children: [
        Text('${outcome['notice']}'),
        detail('Source folder', receipt['source']?['root']),
        detail('Destination folder', receipt['destinationRoot']),
        detail('Retained versions and patch', item['directory']),
        const Text(
          'The numbered .bin files map to paths in each manifest.json. Index copies and Git snapshot refs are retained. Inspect these versions before resolving conflicts or reviewing a new transfer.',
        ),
        TextButton.icon(
          onPressed: () =>
              Clipboard.setData(ClipboardData(text: '${item['directory']}')),
          icon: const Icon(Icons.copy, size: 16),
          label: const Text('Copy recovery folder'),
        ),
      ],
    );
  }

  @override
  Widget build(BuildContext context) {
    final receipt = review?['receipt'] as Map?;
    return PopScope(
      canPop: false,
      onPopInvokedWithResult: (didPop, _) {
        if (!didPop) close();
      },
      child: InspectorFrame(
        title: 'Transfer changes',
        subtitle: 'Review selected saved files between checkouts',
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
              const Text(
                'Working… Files and retained versions stay available.',
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
                          await call('cancel', {'job': job});
                        } catch (failure) {
                          fail(failure);
                        }
                      },
                      child: const Text('Stop operation'),
                    ),
                  ],
                ),
              const SizedBox(height: 12),
            ],
            if (result != null) ...[
              Text('${result!['notice']}'),
              const SizedBox(height: 12),
            ],
            detail('Source folder', widget.record['root']),
            if (receipt == null) ...[
              DropdownButtonFormField<String>(
                key: ValueKey(destination),
                initialValue: destination,
                isExpanded: true,
                decoration: const InputDecoration(
                  labelText: 'Destination chat',
                ),
                items: [
                  for (final entry in destinations.entries)
                    DropdownMenuItem(
                      value: entry.key,
                      child: Text(
                        '${entry.value['title']} · ${entry.value['workspace']?['root']}',
                        maxLines: 1,
                        overflow: TextOverflow.ellipsis,
                      ),
                    ),
                ],
                onChanged: locked
                    ? null
                    : (value) => setState(() => destination = value),
              ),
              const SizedBox(height: 8),
              if (destination != null)
                detail(
                  'Destination folder',
                  destinations[destination]?['workspace']?['root'],
                ),
              TextButton(
                onPressed: locked ? null : loadDestinations,
                child: Text(
                  older
                      ? 'Load older destination chats'
                      : 'Find destination chats',
                ),
              ),
              const Text(
                'Only saved changes from the source base are transferred. Ignored files and unsaved editors stay where they are. Conflicts keep both versions and require a new review after resolution.',
              ),
              Align(
                alignment: Alignment.centerLeft,
                child: OutlinedButton(
                  onPressed: locked
                      ? null
                      : () => act(() => start('compareTransfer')),
                  child: const Text('Compare changes'),
                ),
              ),
              for (final path in paths)
                CheckboxListTile(
                  contentPadding: EdgeInsets.zero,
                  title: Text(path),
                  value: selected.contains(path),
                  controlAffinity: ListTileControlAffinity.leading,
                  onChanged: locked
                      ? null
                      : (chosen) => setState(() {
                          if (chosen == true) {
                            selected.add(path);
                          } else {
                            selected.remove(path);
                          }
                        }),
                ),
              Align(
                alignment: Alignment.centerLeft,
                child: FilledButton(
                  onPressed: locked || selected.isEmpty || destination == null
                      ? null
                      : () => act(
                          () => start('reviewTransfer', {
                            'destination': destination,
                            'paths': selected.toList(),
                          }),
                        ),
                  child: Text(
                    'Review transfer${selected.isEmpty ? '' : ' (${selected.length})'}',
                  ),
                ),
              ),
            ] else ...[
              detail('Destination folder', receipt['destinationRoot']),
              Text(receipt['notice'] as String),
              const SizedBox(height: 12),
              for (final path in receipt['paths'] as List) Text('• $path'),
              const SizedBox(height: 12),
              ExpansionTile(
                title: const Text('Selected patch'),
                children: [
                  ConstrainedBox(
                    constraints: const BoxConstraints(maxHeight: 260),
                    child: SingleChildScrollView(
                      child: SelectableText(
                        review!['patch'] as String,
                        style: const TextStyle(
                          fontFamily: 'Consolas',
                          fontSize: 12,
                        ),
                      ),
                    ),
                  ),
                ],
              ),
              detail('Retained versions and patch', review!['directory']),
              Wrap(
                spacing: 8,
                children: [
                  FilledButton(
                    onPressed: locked || receipt['applicable'] != true
                        ? null
                        : () => act(() async {
                            final token = review!['token'];
                            await start('applyTransfer', {'token': token});
                            review = null;
                          }),
                    child: const Text('Apply transfer'),
                  ),
                  TextButton(
                    onPressed: locked ? null : () => act(discard),
                    child: const Text('Change selection'),
                  ),
                ],
              ),
            ],
            const Divider(height: 32),
            Row(
              children: [
                const Expanded(child: Text('Retained transfers')),
                TextButton(
                  onPressed: locked ? null : refresh,
                  child: const Text('Refresh'),
                ),
              ],
            ),
            if (retained.isEmpty) const Text('No transfer receipts yet.'),
            for (final item in retained) evidence(item),
          ],
        ),
      ),
    );
  }
}
