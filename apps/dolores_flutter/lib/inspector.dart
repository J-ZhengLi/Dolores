import 'dart:convert';

import 'package:flutter/material.dart';

import 'chat.dart';
import 'theme.dart';
import 'usage_details.dart';

String _bytes(int n) =>
    n < 1024 ? '$n B' : '${(n / 1024).toStringAsFixed(1)} KiB';

Future<void> showContextPreview(
  BuildContext context,
  Map<String, dynamic> report,
) => showDialog<void>(
  context: context,
  builder: (_) => ContextInspector(report: report),
);

class InspectorFrame extends StatelessWidget {
  final String title, subtitle;
  final Widget child;
  const InspectorFrame({
    super.key,
    required this.title,
    required this.subtitle,
    required this.child,
  });

  @override
  Widget build(BuildContext context) {
    final p = Palette(Theme.of(context).brightness == Brightness.dark);
    return Dialog(
      insetPadding: const EdgeInsets.all(16),
      backgroundColor: p.bg,
      child: ConstrainedBox(
        constraints: const BoxConstraints(maxWidth: 720, maxHeight: 640),
        child: Column(
          children: [
            Padding(
              padding: const EdgeInsets.fromLTRB(20, 16, 8, 12),
              child: Row(
                children: [
                  Expanded(
                    child: Column(
                      crossAxisAlignment: CrossAxisAlignment.start,
                      children: [
                        Text(
                          title,
                          style: const TextStyle(
                            fontSize: 18,
                            fontWeight: FontWeight.w600,
                          ),
                        ),
                        Text(
                          subtitle,
                          style: TextStyle(fontSize: 12, color: p.muted),
                        ),
                      ],
                    ),
                  ),
                  TextButton(
                    onPressed: () => Navigator.pop(context),
                    child: const Text('Close'),
                  ),
                ],
              ),
            ),
            Divider(height: 1, color: p.border),
            Expanded(child: child),
          ],
        ),
      ),
    );
  }
}

class ContextIndicator extends StatelessWidget {
  final Map<String, dynamic>? summary;
  final String? basis;
  final VoidCallback? onPressed;
  const ContextIndicator({super.key, this.summary, this.basis, this.onPressed});

  @override
  Widget build(BuildContext context) {
    final p = Palette(Theme.of(context).brightness == Brightness.dark);
    final used = summary?['textBytes'], limit = summary?['maxTextBytes'];
    final value = used is int && limit is int && limit > 0
        ? (used / limit).clamp(0.0, 1.0)
        : null;
    final label = value == null
        ? 'Inspect next message context · usage not inspected'
        : 'Inspect next message context · $basis: ${(value * 100).toStringAsFixed(1)}% of app text budget';
    return IconButton(
      key: const Key('context-preview'),
      tooltip: label,
      onPressed: onPressed,
      icon: SizedBox(
        width: 18,
        height: 18,
        child: Stack(
          alignment: Alignment.center,
          children: [
            CircularProgressIndicator(
              key: const Key('context-ring'),
              value: value ?? 0,
              strokeWidth: 2.5,
              backgroundColor: p.border,
              color: p.accent,
              semanticsLabel: label,
            ),
            if (value == null)
              Text('?', style: TextStyle(fontSize: 9, color: p.muted)),
          ],
        ),
      ),
    );
  }
}

class ContextInspector extends StatelessWidget {
  final Map<String, dynamic> report;
  const ContextInspector({super.key, required this.report});

  @override
  Widget build(BuildContext context) {
    final p = Palette(Theme.of(context).brightness == Brightness.dark);
    final messages = ((report['messages'] as List?) ?? []).cast<Map>();
    final system = messages.where((m) => m['role'] == 'system').toList();
    final draft = messages.isNotEmpty ? [messages.last] : <Map>[];
    final history = messages.length >= 2
        ? messages.sublist(1, messages.length - 1)
        : <Map>[];
    final groups = [system, history, draft];
    final labels = [
      'System instructions',
      'Conversation history',
      'Draft message',
    ];
    final colors = [p.syntaxName, p.accent, p.syntaxString];
    final counts = groups
        .map(
          (group) => group.fold<int>(
            0,
            (sum, m) => sum + utf8.encode(m['content'] as String).length,
          ),
        )
        .toList();
    final used = report['textBytes'] as int? ?? 0;
    final limit = report['maxTextBytes'] as int? ?? 131072;
    return InspectorFrame(
      title: 'Context for your next message',
      subtitle: 'Exact prepared text · local preview',
      child: ListView(
        padding: const EdgeInsets.all(20),
        children: [
          Row(
            crossAxisAlignment: CrossAxisAlignment.end,
            children: [
              Text(
                '${(used / limit * 100).toStringAsFixed(1)}%',
                style: const TextStyle(
                  fontSize: 32,
                  fontWeight: FontWeight.w600,
                ),
              ),
              const SizedBox(width: 12),
              Expanded(
                child: Text(
                  '${_bytes(used)} / ${_bytes(limit)}\nApp text budget',
                  style: TextStyle(color: p.muted, fontSize: 12),
                ),
              ),
            ],
          ),
          const SizedBox(height: 12),
          ClipRRect(
            borderRadius: BorderRadius.circular(4),
            child: SizedBox(
              height: 8,
              child: Row(
                children: [
                  for (var i = 0; i < counts.length; i++)
                    if (counts[i] > 0)
                      Expanded(
                        flex: counts[i],
                        child: ColoredBox(
                          color: colors[i],
                          child: const SizedBox.expand(),
                        ),
                      ),
                  if (limit > used)
                    Expanded(
                      flex: limit - used,
                      child: ColoredBox(
                        color: p.border,
                        child: const SizedBox.expand(),
                      ),
                    ),
                ],
              ),
            ),
          ),
          const SizedBox(height: 16),
          for (var i = 0; i < groups.length; i++)
            Padding(
              padding: const EdgeInsets.only(bottom: 6),
              child: Row(
                children: [
                  Container(
                    width: 8,
                    height: 8,
                    decoration: BoxDecoration(
                      color: colors[i],
                      shape: BoxShape.circle,
                    ),
                  ),
                  const SizedBox(width: 8),
                  Expanded(
                    child: Text(
                      labels[i],
                      style: const TextStyle(fontSize: 12),
                    ),
                  ),
                  Text(
                    messages.isEmpty ? 'Unavailable' : _bytes(counts[i]),
                    style: TextStyle(fontSize: 12, color: p.muted),
                  ),
                ],
              ),
            ),
          const SizedBox(height: 14),
          Text(
            'Recent turns included: ${report['includedTurns'] ?? 'Unavailable'}',
          ),
          Text('Saved turns: ${report['savedTurns'] ?? 'Unavailable'}'),
          Text(
            'Older turns left out: ${report['omittedTurns'] ?? 'Unavailable'}',
          ),
          const SizedBox(height: 8),
          Text(
            'Latest ${report['maxTurns'] ?? 40} complete turns at most. Older turns stay saved. The model’s token window is unavailable.',
            style: TextStyle(color: p.muted, fontSize: 12),
          ),
          const SizedBox(height: 20),
          for (var i = 0; i < groups.length; i++)
            _ContextGroup(
              label: labels[i],
              messages: groups[i],
              color: colors[i],
            ),
        ],
      ),
    );
  }
}

class _ContextGroup extends StatelessWidget {
  final String label;
  final List<Map> messages;
  final Color color;
  const _ContextGroup({
    required this.label,
    required this.messages,
    required this.color,
  });
  @override
  Widget build(BuildContext context) {
    final p = Palette(Theme.of(context).brightness == Brightness.dark);
    return Card(
      elevation: 0,
      color: p.surface,
      margin: const EdgeInsets.only(bottom: 8),
      child: ExpansionTile(
        key: PageStorageKey('context-group-$label'),
        leading: Icon(Icons.subject, size: 18, color: color),
        title: Text(label, style: const TextStyle(fontSize: 13)),
        subtitle: Text(
          '${messages.length} ${messages.length == 1 ? 'message' : 'messages'}',
          style: TextStyle(fontSize: 11, color: p.muted),
        ),
        children: [
          for (var i = 0; i < messages.length; i++)
            Padding(
              padding: const EdgeInsets.fromLTRB(16, 0, 16, 16),
              child: Align(
                alignment: Alignment.centerLeft,
                child: Column(
                  crossAxisAlignment: CrossAxisAlignment.start,
                  children: [
                    Text(
                      '${i + 1} · ${messages[i]['role']}',
                      style: TextStyle(color: p.muted, fontSize: 11),
                    ),
                    SelectableText(
                      key: PageStorageKey('context-text-$i'),
                      messages[i]['content'] as String,
                      style: const TextStyle(fontSize: 12),
                    ),
                  ],
                ),
              ),
            ),
        ],
      ),
    );
  }
}

Future<void> showTrajectory(BuildContext context, ChatController chat) =>
    showDialog<void>(
      context: context,
      builder: (_) => TrajectoryInspector(chat: chat),
    );

class TrajectoryInspector extends StatefulWidget {
  final ChatController chat;
  const TrajectoryInspector({super.key, required this.chat});
  @override
  State<TrajectoryInspector> createState() => _TrajectoryInspectorState();
}

class _TrajectoryInspectorState extends State<TrajectoryInspector> {
  List<Map<String, dynamic>> items = [];
  bool older = false, newer = false, loading = false;
  String? failure;
  int tab = 0;
  bool wasBusy = false;
  @override
  void initState() {
    super.initState();
    final chat = widget.chat;
    wasBusy = chat.busy;
    items = List.of(chat.messages);
    older = chat.messagesOlder;
    newer = chat.messagesNewer;
    chat.addListener(_changed);
    if (!chat.busy) _load();
  }

  void _changed() {
    if (!mounted) return;
    // changing remains true while the successful reply is loaded.
    if (!widget.chat.busy && !widget.chat.changing && wasBusy) {
      wasBusy = false;
      if (!newer) _load();
    } else if (widget.chat.busy) {
      wasBusy = true;
    }
    setState(() {});
  }

  Future<void> _load({int? cursor, bool forward = false}) async {
    if (loading ||
        widget.chat.busy ||
        widget.chat.changing ||
        widget.chat.session == null) {
      return;
    }
    setState(() {
      loading = true;
      failure = null;
    });
    try {
      final page = await widget.chat.inspectHistory(
        cursor: cursor,
        newer: forward,
      );
      if (!mounted || page == null) return;
      setState(() {
        items = (page['items'] as List).cast<Map<String, dynamic>>();
        older = page['hasOlder'] == true;
        newer = page['hasNewer'] == true;
      });
    } catch (_) {
      if (mounted) {
        setState(
          () => failure = 'History could not be loaded. Try Latest again.',
        );
      }
    } finally {
      if (mounted) setState(() => loading = false);
    }
  }

  @override
  void dispose() {
    widget.chat.removeListener(_changed);
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    final p = Palette(Theme.of(context).brightness == Brightness.dark);
    final chat = widget.chat;
    final logs = chat.requestLogs
        .where((e) => e.session == chat.session)
        .toList();
    final enabled = !loading && !chat.busy && !chat.changing;
    return InspectorFrame(
      title: 'Chat trajectory',
      subtitle: 'Saved exchanges and request lifecycle',
      child: Column(
        children: [
          Padding(
            padding: const EdgeInsets.all(12),
            child: SegmentedButton<int>(
              segments: const [
                ButtonSegment(
                  value: 0,
                  icon: Icon(Icons.timeline, size: 16),
                  label: Text('Trajectory'),
                ),
                ButtonSegment(
                  value: 1,
                  icon: Icon(Icons.list_alt, size: 16),
                  label: Text('Log'),
                ),
              ],
              selected: {tab},
              onSelectionChanged: (value) => setState(() => tab = value.single),
            ),
          ),
          if (tab == 0) ...[
            Padding(
              padding: const EdgeInsets.symmetric(horizontal: 12),
              child: Row(
                children: [
                  TextButton(
                    onPressed: enabled && older && items.isNotEmpty
                        ? () => _load(cursor: items.first['id'] as int)
                        : null,
                    child: const Text('Older'),
                  ),
                  TextButton(
                    onPressed: enabled && newer && items.isNotEmpty
                        ? () => _load(
                            cursor: items.last['id'] as int,
                            forward: true,
                          )
                        : null,
                    child: const Text('Newer'),
                  ),
                  const Spacer(),
                  TextButton(
                    onPressed: enabled && chat.session != null
                        ? () => _load()
                        : null,
                    child: const Text('Latest'),
                  ),
                ],
              ),
            ),
            if (loading) const LinearProgressIndicator(minHeight: 2),
            if (failure != null)
              Padding(
                padding: const EdgeInsets.all(12),
                child: Text(failure!, style: TextStyle(color: p.errorText)),
              ),
            Expanded(
              child: items.isEmpty
                  ? const Center(child: Text('No saved exchanges yet.'))
                  : ListView.builder(
                      padding: const EdgeInsets.fromLTRB(16, 8, 16, 16),
                      itemCount: items.length,
                      itemBuilder: (_, index) {
                        final message = items[index];
                        final assistant = message['role'] == 'assistant';
                        final metadata = (message['metadata'] as Map?)
                            ?.cast<String, dynamic>();
                        return Card(
                          elevation: 0,
                          color: p.surface,
                          margin: const EdgeInsets.only(bottom: 6),
                          child: ExpansionTile(
                            key: ValueKey(message['id']),
                            leading: Icon(
                              assistant
                                  ? Icons.auto_awesome_outlined
                                  : Icons.person_outline,
                              size: 18,
                              color: assistant ? p.accent : p.muted,
                            ),
                            title: Text(
                              '${assistant ? 'Reply' : 'Message'} · #${message['id']}',
                              style: const TextStyle(fontSize: 13),
                            ),
                            subtitle: Text(
                              assistant
                                  ? '${metadata?['model'] ?? 'Model unavailable'} · Saved'
                                  : message['content'] as String,
                              maxLines: 1,
                              overflow: TextOverflow.ellipsis,
                              style: TextStyle(fontSize: 11, color: p.muted),
                            ),
                            childrenPadding: const EdgeInsets.fromLTRB(
                              16,
                              0,
                              16,
                              12,
                            ),
                            children: [
                              Align(
                                alignment: Alignment.centerLeft,
                                child: SelectableText(
                                  message['content'] as String,
                                ),
                              ),
                              if (assistant) UsageDetails(metadata: metadata),
                            ],
                          ),
                        );
                      },
                    ),
            ),
            if (chat.busy)
              Padding(
                padding: const EdgeInsets.all(12),
                child: Text(
                  chat.stopping
                      ? 'Stopping current request…'
                      : 'Current request is streaming…',
                  style: TextStyle(color: p.accent),
                ),
              ),
          ] else ...[
            Padding(
              padding: const EdgeInsets.symmetric(horizontal: 20, vertical: 8),
              child: Text(
                'Recent events from this app session · up to 200 events across conversations. Saved exchanges remain in Trajectory.',
                style: TextStyle(fontSize: 12, color: p.muted),
              ),
            ),
            Expanded(
              child: logs.isEmpty
                  ? const Center(
                      child: Text('No request events in this app session.'),
                    )
                  : ListView.builder(
                      padding: const EdgeInsets.symmetric(horizontal: 20),
                      itemCount: logs.length,
                      itemBuilder: (_, i) {
                        final event = logs[i];
                        final time =
                            '${event.time.hour.toString().padLeft(2, '0')}:${event.time.minute.toString().padLeft(2, '0')}:${event.time.second.toString().padLeft(2, '0')}';
                        return Padding(
                          padding: const EdgeInsets.only(bottom: 14),
                          child: Row(
                            crossAxisAlignment: CrossAxisAlignment.start,
                            children: [
                              Container(
                                margin: const EdgeInsets.only(top: 6),
                                width: 7,
                                height: 7,
                                decoration: BoxDecoration(
                                  color: p.accent,
                                  shape: BoxShape.circle,
                                ),
                              ),
                              const SizedBox(width: 12),
                              Expanded(
                                child: Column(
                                  crossAxisAlignment: CrossAxisAlignment.start,
                                  children: [
                                    Text(
                                      event.label,
                                      style: const TextStyle(fontSize: 13),
                                    ),
                                    Text(
                                      '$time · ${(event.elapsedMs / 1000).toStringAsFixed(3)} s · Request ${event.run}\n${event.model}',
                                      style: TextStyle(
                                        color: p.muted,
                                        fontSize: 11,
                                      ),
                                    ),
                                  ],
                                ),
                              ),
                            ],
                          ),
                        );
                      },
                    ),
            ),
          ],
        ],
      ),
    );
  }
}
