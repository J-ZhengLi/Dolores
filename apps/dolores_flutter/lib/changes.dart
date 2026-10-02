import 'package:flutter/material.dart';

import 'bridge.dart';
import 'chat.dart';
import 'edit_diff.dart';
import 'inspector.dart';
import 'theme.dart';

Future<void> showChanges(BuildContext context, ChatController chat) =>
    chat.inspectLocalChanges(
      () => showDialog<void>(
        context: context,
        barrierDismissible: false,
        builder: (_) =>
            ChangesInspector(bridge: chat.bridge, session: chat.session!),
      ),
    );

class ChangesInspector extends StatefulWidget {
  final ChatBridge bridge;
  final String session;
  const ChangesInspector({
    super.key,
    required this.bridge,
    required this.session,
  });
  @override
  State<ChangesInspector> createState() => _ChangesInspectorState();
}

class _ChangesInspectorState extends State<ChangesInspector> {
  List<Map<String, dynamic>> items = [];
  Map<String, dynamic>? detail, preview;
  int? cursor;
  bool older = false, busy = false;
  String? error, notice;

  @override
  void initState() {
    super.initState();
    _load();
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

  Future<void> _cancel() async {
    final token = preview?['token'];
    preview = null;
    if (token != null) await _call('cancelRevert', {'token': token});
  }

  Future<void> _page() async {
    final page = await _call('changesPage', {'cursor': cursor});
    if (!mounted) return;
    setState(() {
      items = (page['items'] as List)
          .map((v) => (v as Map).cast<String, dynamic>())
          .toList();
      older = page['hasOlder'] == true;
      detail = null;
    });
  }

  Future<void> _load([int? next]) => _act(() async {
    await _cancel();
    cursor = next;
    await _page();
  });
  Future<void> _select(int id) => _act(() async {
    await _cancel();
    final value = await _call('changeDetails', {'changeId': id});
    if (mounted) {
      setState(() {
        detail = (value as Map).cast<String, dynamic>();
        notice = null;
      });
    }
  });
  Future<void> _review() => _act(() async {
    final value = await _call('previewRevert', {
      'changeId': detail!['change']['id'],
    });
    if (mounted) {
      setState(() => preview = (value as Map).cast<String, dynamic>());
    }
  });
  Future<void> _apply() => _act(() async {
    final token = preview!['token'];
    setState(() => preview = null); // Approval is consumed even on failure.
    final result = await _call('applyRevert', {'token': token});
    if (!mounted) return;
    final action = result['removed'] == true
        ? 'Created file removed'
        : 'File reverted';
    setState(
      () => notice = result['journalStatus'] == 'pending'
          ? '$action. Its receipt needs a check; the local intent was saved.'
          : '$action. A separate change record was saved.',
    );
    cursor = null;
    await _page();
  });
  @override
  void dispose() {
    final token = preview?['token'];
    if (token != null) {
      _call('cancelRevert', {'token': token}).catchError((_) => null);
    }
    super.dispose();
  }

  String _status(dynamic value) => switch (value) {
    'applied' => 'Applied',
    'notApplied' => 'Not applied',
    'reverted' => 'Reverted',
    _ => 'Needs check',
  };
  String _kind(Map value) => value['beforeExists'] == false
      ? 'Created'
      : value['afterExists'] == false
      ? 'Removed'
      : 'Edited';
  @override
  Widget build(BuildContext context) {
    final p = Palette(Theme.of(context).brightness == Brightness.dark);
    final change = detail?['change'] as Map?;
    final removing = preview?['operation'] == 'remove';
    final canRevert =
        change != null &&
        change['reverts'] == null &&
        (change['status'] == 'applied' || change['status'] == 'pending');
    return PopScope(
      canPop: !busy,
      child: InspectorFrame(
        title: preview == null
            ? 'Changes'
            : removing
            ? 'Remove this created file?'
            : 'Revert this change?',
        subtitle: 'This working folder · Local records survive chat deletion',
        canClose: !busy,
        child: Padding(
          padding: const EdgeInsets.fromLTRB(20, 0, 20, 16),
          child: Column(
            children: [
              if (busy) const LinearProgressIndicator(),
              Expanded(
                child: ListView(
                  children: [
                    if (error != null)
                      Padding(
                        padding: const EdgeInsets.symmetric(vertical: 8),
                        child: Text(
                          error!,
                          style: TextStyle(color: p.errorText),
                        ),
                      ),
                    if (notice != null)
                      Padding(
                        padding: const EdgeInsets.symmetric(vertical: 8),
                        child: Text(notice!),
                      ),
                    if (preview != null) ...[
                      SelectableText('${preview!['target']}'),
                      const SizedBox(height: 8),
                      Text(
                        removing
                            ? 'Remove this file only if it still matches the saved creation. This creates another local record. Removal cannot be reversed here.'
                            : 'Restore the saved text only if the file still matches this preview. This creates another local record.',
                      ),
                      const SizedBox(height: 12),
                      EditDiff(
                        source: preview!['diff'] as String,
                        identity: 'revert',
                      ),
                      const SizedBox(height: 12),
                      Wrap(
                        spacing: 8,
                        runSpacing: 8,
                        children: [
                          TextButton(
                            onPressed: busy ? null : () => _act(_cancel),
                            child: const Text('Cancel'),
                          ),
                          FilledButton(
                            key: const Key('apply-revert'),
                            onPressed: busy ? null : _apply,
                            child: Text(
                              removing ? 'Remove once' : 'Revert once',
                            ),
                          ),
                        ],
                      ),
                    ] else ...[
                      if (items.isEmpty && !busy)
                        const Padding(
                          padding: EdgeInsets.symmetric(vertical: 24),
                          child: Text('No file changes recorded yet.'),
                        ),
                      for (final item in items)
                        ListTile(
                          key: Key('change-${item['id']}'),
                          contentPadding: EdgeInsets.zero,
                          selected: change?['id'] == item['id'],
                          title: Text(
                            '${item['target']}',
                            maxLines: 2,
                            overflow: TextOverflow.ellipsis,
                          ),
                          subtitle: Text(
                            '${_kind(item)} · ${_status(item['status'])}${item['reverts'] == null ? '' : ' · Revert'} · ${item['bytesBefore']} → ${item['bytesAfter']} bytes\n${DateTime.fromMillisecondsSinceEpoch(item['createdAt'] as int).toLocal()}',
                          ),
                          onTap: busy ? null : () => _select(item['id'] as int),
                        ),
                      if (detail != null) ...[
                        const Divider(),
                        SelectableText('${change!['target']}'),
                        Text(
                          '${_kind(change)} · ${change['bytesAfter']} bytes after change',
                        ),
                        if (change['status'] == 'pending')
                          const Padding(
                            padding: EdgeInsets.symmetric(vertical: 8),
                            child: Text(
                              'An intent was saved, but completion was not recorded. Review checks whether the file matches the saved result.',
                            ),
                          ),
                        const SizedBox(height: 8),
                        EditDiff(
                          source: detail!['diff'] as String,
                          identity: 'journal-${change['id']}',
                        ),
                        if (canRevert)
                          Align(
                            alignment: Alignment.centerLeft,
                            child: TextButton(
                              key: const Key('review-revert'),
                              onPressed: busy ? null : _review,
                              child: const Text('Review revert'),
                            ),
                          ),
                      ],
                    ],
                  ],
                ),
              ),
              if (preview == null)
                Wrap(
                  spacing: 8,
                  children: [
                    TextButton(
                      onPressed: busy ? null : () => _load(),
                      child: const Text('Newest changes'),
                    ),
                    if (older)
                      TextButton(
                        onPressed: busy
                            ? null
                            : () => _load(items.last['id'] as int),
                        child: const Text('Older changes'),
                      ),
                  ],
                ),
            ],
          ),
        ),
      ),
    );
  }
}
