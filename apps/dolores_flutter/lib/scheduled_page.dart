import 'package:flutter/material.dart';

import 'chat.dart';
import 'scheduled_host.dart';
import 'tool_activity.dart';
import 'theme.dart';

String scheduleState(String? state) => switch (state) {
  'claimed' || 'queued' => 'Queued',
  'running' => 'Running',
  'waitingForApproval' => 'Waiting for approval',
  'succeeded' => 'Succeeded',
  'failed' => 'Failed',
  'interrupted' => 'Interrupted',
  'cancelled' => 'Stopped',
  'missed' => 'Missed',
  'skipped' => 'Skipped',
  'paused' => 'Paused',
  _ => 'Scheduled',
};
String scheduleTime(dynamic seconds) => seconds == null
    ? 'No upcoming run'
    : DateTime.fromMillisecondsSinceEpoch((seconds as num).toInt() * 1000)
          .toLocal()
          .toString()
          .substring(0, 16);

class ScheduledPanel extends StatelessWidget {
  final ScheduledHost host;
  final String? selected;
  final ValueChanged<String> onSelect;
  const ScheduledPanel({
    super.key,
    required this.host,
    required this.selected,
    required this.onSelect,
  });
  @override
  Widget build(BuildContext context) => AnimatedBuilder(
    animation: host,
    builder: (context, _) => Material(
      color: Palette(Theme.of(context).brightness == Brightness.dark).sidebar,
      child: Column(
        children: [
          Padding(
            padding: const EdgeInsets.all(16),
            child: Row(
              children: [
                const Expanded(
                  child: Text('Scheduled', style: TextStyle(fontSize: 18)),
                ),
                PopupMenuButton<String>(
                  tooltip: 'Filter tasks',
                  icon: const Icon(Icons.filter_list, size: 18),
                  onSelected: host.setFilter,
                  itemBuilder: (_) => [
                    for (final entry in {
                      'all': 'All',
                      'active': 'Active',
                      'paused': 'Paused',
                      'finished': 'Finished',
                    }.entries)
                      CheckedPopupMenuItem(
                        value: entry.key,
                        checked: host.filter == entry.key,
                        child: Text(entry.value),
                      ),
                  ],
                ),
                IconButton(
                  tooltip: 'Refresh',
                  onPressed: host.pending ? null : host.refresh,
                  icon: const Icon(Icons.refresh, size: 18),
                ),
              ],
            ),
          ),
          Expanded(
            child: host.visibleItems.isEmpty
                ? const Center(child: Text('No tasks'))
                : ListView(
                    children: [
                      for (final item in host.visibleItems)
                        ListTile(
                          selected: (item['task'] as Map)['id'] == selected,
                          title: Text(
                            (item['task'] as Map)['title'] as String,
                            maxLines: 2,
                            overflow: TextOverflow.ellipsis,
                          ),
                          subtitle: Text(
                            (item['task'] as Map)['paused'] == true
                                ? 'Paused'
                                : scheduleTime(
                                    (item['task'] as Map)['nextDue'],
                                  ),
                            maxLines: 1,
                          ),
                          onTap: () =>
                              onSelect((item['task'] as Map)['id'] as String),
                        ),
                    ],
                  ),
          ),
        ],
      ),
    ),
  );
}

class ScheduledPage extends StatelessWidget {
  final ScheduledHost host;
  final String? selected;
  final List<ChatController> owners;
  final Future<void> Function(String session) openResult;
  const ScheduledPage({
    super.key,
    required this.host,
    required this.selected,
    required this.owners,
    required this.openResult,
  });
  @override
  Widget build(BuildContext context) => AnimatedBuilder(
    animation: host,
    builder: (context, _) {
      final item =
          host.visibleItems
              .where((i) => (i['task'] as Map)['id'] == selected)
              .firstOrNull ??
          host.visibleItems.firstOrNull;
      return SizedBox.expand(
        child: SingleChildScrollView(
          padding: const EdgeInsets.all(24),
          child: Align(
            alignment: Alignment.topCenter,
            child: ConstrainedBox(
              constraints: const BoxConstraints(
                maxWidth: UiTokens.contentWidth,
              ),
              child: Column(
                crossAxisAlignment: CrossAxisAlignment.start,
                children: [
                  const Text('Scheduled', style: TextStyle(fontSize: 26)),
                  const SizedBox(height: 8),
                  const Text('Runs while Dolores is open.'),
                  if (host.error != null)
                    Padding(
                      padding: const EdgeInsets.only(top: 16),
                      child: Column(
                        crossAxisAlignment: CrossAxisAlignment.start,
                        children: [
                          SelectableText(host.error!),
                          TextButton(
                            onPressed: host.pending ? null : host.refresh,
                            child: const Text('Refresh'),
                          ),
                        ],
                      ),
                    ),
                  if (item == null) ...[
                    const SizedBox(height: 48),
                    Text(
                      host.items.isEmpty
                          ? 'Ask Dolores in a chat to schedule a task.'
                          : 'No tasks match this filter.',
                    ),
                    if (host.items.isEmpty) ...[
                      const SizedBox(height: 12),
                      const SelectableText(
                        '“Every weekday at 9 pm, write my daily report.”',
                      ),
                    ],
                  ] else
                    taskDetails(context, item),
                ],
              ),
            ),
          ),
        ),
      );
    },
  );
  Widget taskDetails(BuildContext context, Map item) {
    final task = item['task'] as Map, receipt = item['receipt'] as Map;
    final history = (item['occurrences'] as List).cast<Map>();
    final active = history
        .where(
          (o) => [
            'claimed',
            'queued',
            'running',
            'waitingForApproval',
          ].contains(o['state']),
        )
        .firstOrNull;
    final owner = owners
        .where((c) => c.session != null && c.session == active?['session'])
        .firstOrNull;
    final paused = task['paused'] == true;
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        const SizedBox(height: 28),
        Row(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Expanded(
              child: Text(
                task['title'] as String,
                style: const TextStyle(fontSize: 20),
              ),
            ),
            PopupMenuButton<String>(
              tooltip: 'Task actions',
              enabled: !host.pending,
              onSelected: (action) async {
                if (action == 'delete') {
                  final confirm = await showDialog<bool>(
                    context: context,
                    builder: (context) => AlertDialog(
                      title: const Text('Delete task?'),
                      content: const Text(
                        'Future runs stop. Saved results remain. Current work keeps running until you stop it.',
                      ),
                      actions: [
                        TextButton(
                          onPressed: () => Navigator.pop(context, false),
                          child: const Text('Keep task'),
                        ),
                        TextButton(
                          onPressed: () => Navigator.pop(context, true),
                          child: const Text('Delete'),
                        ),
                      ],
                    ),
                  );
                  if (confirm != true) return;
                }
                await host.manage(task, action);
              },
              itemBuilder: (_) => [
                PopupMenuItem(
                  value: paused ? 'resume' : 'pause',
                  child: Text(paused ? 'Resume' : 'Pause'),
                ),
                PopupMenuItem(
                  value: 'runNow',
                  enabled: active == null,
                  child: const Text('Run now'),
                ),
                PopupMenuItem(
                  value: 'skip',
                  enabled: task['nextDue'] != null,
                  child: const Text('Skip next'),
                ),
                PopupMenuItem(
                  value: 'stop',
                  enabled: active != null,
                  child: const Text('Stop current run'),
                ),
                const PopupMenuDivider(),
                const PopupMenuItem(
                  value: 'delete',
                  child: Text('Delete task'),
                ),
              ],
            ),
          ],
        ),
        const SizedBox(height: 12),
        SelectableText(receipt['schedule'] as String),
        const SizedBox(height: 8),
        Text(paused ? 'Paused' : 'Next: ${scheduleTime(task['nextDue'])}'),
        const SizedBox(height: 16),
        ExpansionTile(
          tilePadding: EdgeInsets.zero,
          title: const Text('Details'),
          children: [
            for (final entry in <String, String>{
              'Model': '${receipt['model']}',
              'Skill': '${receipt['skill'] ?? 'None'}',
              'Task ID': '${task['id']}',
              'Project': '${receipt['project'] ?? 'No project'}',
              'Timezone': '${receipt['timezone']}',
              'Results': 'Separate task conversations',
            }.entries)
              ListTile(
                dense: true,
                title: Text(entry.key),
                subtitle: SelectableText(entry.value),
              ),
            TextButton(
              onPressed: () => openResult(task['sourceSession'] as String),
              child: const Text('Open source chat'),
            ),
          ],
        ),
        if (active != null)
          Padding(
            padding: const EdgeInsets.symmetric(vertical: 16),
            child: owner == null
                ? Text(scheduleState(active['state'] as String?))
                : AnimatedBuilder(
                    animation: owner,
                    builder: (_, _) => Column(
                      crossAxisAlignment: CrossAxisAlignment.start,
                      children: [
                        Text(
                          owner.toolApproval != null
                              ? 'Waiting for approval'
                              : owner.busy
                              ? owner.modelPhase == 'queued'
                                    ? 'Queued'
                                    : 'Running'
                              : scheduleState(active['state'] as String?),
                        ),
                        if (owner.partial.isNotEmpty)
                          SelectableText(owner.partial),
                        if (owner.toolApproval != null)
                          ToolApprovalCard(chat: owner),
                        if (owner.busy)
                          TextButton(
                            onPressed: owner.stopping ? null : owner.stop,
                            child: const Text('Stop'),
                          ),
                      ],
                    ),
                  ),
          ),
        const SizedBox(height: 24),
        const Text('History', style: TextStyle(fontSize: 18)),
        if (history.isEmpty)
          const Padding(
            padding: EdgeInsets.only(top: 12),
            child: Text('No runs yet.'),
          ),
        for (final occurrence in history)
          Padding(
            padding: const EdgeInsets.only(top: 12),
            child: Card(
              child: Padding(
                padding: const EdgeInsets.all(16),
                child: Column(
                  crossAxisAlignment: CrossAxisAlignment.start,
                  children: [
                    Wrap(
                      spacing: 16,
                      runSpacing: 8,
                      children: [
                        Text(scheduleState(occurrence['state'] as String?)),
                        Text(scheduleTime(occurrence['due'])),
                      ],
                    ),
                    if (occurrence['error'] != null)
                      Padding(
                        padding: const EdgeInsets.only(top: 8),
                        child: SelectableText(occurrence['error'] as String),
                      ),
                    if (occurrence['session'] != null)
                      TextButton(
                        onPressed: () =>
                            openResult(occurrence['session'] as String),
                        child: const Text('Open result'),
                      ),
                  ],
                ),
              ),
            ),
          ),
      ],
    );
  }
}
