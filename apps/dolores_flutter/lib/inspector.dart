import 'dart:convert';

import 'package:flutter/material.dart';

import 'chat.dart';
import 'settings_frame.dart';
import 'theme.dart';
import 'usage_details.dart';
import 'tool_activity.dart';
import 'model_steps.dart';
import 'task_feedback.dart';

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
  final bool canClose;
  final VoidCallback? onClose;
  const InspectorFrame({
    super.key,
    required this.title,
    required this.subtitle,
    required this.child,
    this.canClose = true,
    this.onClose,
  });

  @override
  Widget build(BuildContext context) {
    final embedding = SettingsEmbedding.of(context);
    if (embedding != null) {
      return EmbeddedSettingsFrame(
        title: title,
        subtitle: subtitle,
        canClose: canClose,
        pending: embedding.pending,
        child: child,
      );
    }
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
                    onPressed: canClose
                        ? onClose ?? () => Navigator.pop(context)
                        : null,
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
    final tokens = summary?['tokens'] as Map?;
    final reported = summary?['reportedTokens'];
    final used = reported ?? tokens?['inputTokens'],
        limit = tokens?['contextWindowTokens'];
    final value = used is int && limit is int && limit > 0
        ? (used / limit).clamp(0.0, 1.0)
        : null;
    final label = value == null
        ? 'Inspect next message context · ${used is int ? '${formatTokens(used)} tokens · context window not set' : 'usage not inspected'}'
        : 'Inspect next message context · $basis: ${(value * 100).toStringAsFixed(1)}% of model context window · ${reported is int ? 'Provider reported' : 'Estimated'} ${formatTokens(used)} tokens';
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
    final colors = [
      p.syntaxName,
      p.accent,
      p.syntaxString,
      p.syntaxValue,
      p.muted,
      p.accent.withValues(alpha: .6),
    ];
    final tokens = report['tokens'] as Map?;
    final counts = [
      for (final key in [
        'systemTokens',
        'historyTokens',
        'draftTokens',
        'toolTokens',
        'framingTokens',
        'imageTokens',
      ])
        tokens?[key] as int? ?? 0,
    ];
    final used = tokens?['inputTokens'] as int?;
    final limit = tokens?['contextWindowTokens'] as int?;
    final value = used != null && limit != null && limit > 0
        ? used / limit
        : null;
    return InspectorFrame(
      title: 'Context for your next message',
      subtitle: 'Estimated tokens · local preview',
      child: ListView(
        padding: const EdgeInsets.all(20),
        children: [
          Row(
            crossAxisAlignment: CrossAxisAlignment.end,
            children: [
              Text(
                value == null ? '—' : '${(value * 100).toStringAsFixed(1)}%',
                style: const TextStyle(
                  fontSize: 32,
                  fontWeight: FontWeight.w600,
                ),
              ),
              const SizedBox(width: 12),
              Expanded(
                child: Text(
                  '${formatTokens(used)} / ${formatTokens(limit)} tokens\n${limit == null ? 'Context window not set for this model' : 'Model context window · ${report['model'] ?? ''}'}',
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
                  if (limit != null && used != null && limit > used)
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
          for (var i = 0; i < counts.length; i++)
            if (i != 5 || counts[i] > 0)
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
                        [
                          ...labels,
                          'Tool definitions',
                          'Message framing',
                          'Image allowance (estimated)',
                        ][i],
                        style: const TextStyle(fontSize: 12),
                      ),
                    ),
                    Text(
                      tokens == null
                          ? 'Unavailable'
                          : '${formatTokens(counts[i])} tokens',
                      style: TextStyle(fontSize: 12, color: p.muted),
                    ),
                  ],
                ),
              ),
          const SizedBox(height: 14),
          if ((tokens?['imageTokens'] as int? ?? 0) > 0)
            const Text(
              'Images reserve approximately 4096 tokens each for low-detail input. Provider usage may differ.',
            ),
          Text(
            'Response reserved: ${formatTokens(tokens?['reservedOutputTokens'])} tokens',
          ),
          if (limit != null)
            Text(
              'Input allowance: ${formatTokens(tokens?['maxInputTokens'])} tokens · 5% headroom',
            ),
          const SizedBox(height: 8),
          Text(
            'Recent turns included: ${report['includedTurns'] ?? 'Unavailable'}',
          ),
          Text('Saved turns: ${report['savedTurns'] ?? 'Unavailable'}'),
          if (report['summary'] is Map)
            Text(
              'Turns covered by summary: ${report['summary']['coveredTurns']} · revision ${report['summary']['revision']}',
            ),
          Text(
            'Older turns left out: ${report['omittedTurns'] ?? 'Unavailable'}',
          ),
          const SizedBox(height: 8),
          Text(
            'Latest uncovered complete turns that fit, at most ${report['maxTurns'] ?? 40}. Older turns stay saved; summary coverage is counted separately from turns left out. ${limit == null ? 'Set this model’s context window in Model connection. ' : ''}Token counts are approximate; your provider may count differently.',
            style: TextStyle(color: p.muted, fontSize: 12),
          ),
          const SizedBox(height: 20),
          for (var i = 0; i < groups.length; i++)
            _ContextGroup(
              label: labels[i],
              messages: groups[i],
              color: colors[i],
            ),
          if (report['tools'] is List && (report['tools'] as List).isNotEmpty)
            _ContextGroup(
              label: 'Tool definitions',
              messages: [
                for (final tool in report['tools'] as List)
                  {
                    'role': tool['name'],
                    'content': const JsonEncoder.withIndent('  ').convert(tool),
                  },
              ],
              color: colors[3],
            ),
          if (report['instructions'] is Map)
            Card(
              elevation: 0,
              color: p.surface,
              child: Padding(
                padding: const EdgeInsets.all(16),
                child: SelectableText(
                  'Workspace instructions · ${report['instructions']['source']}\nReviewed revision: ${report['instructions']['revision']}\nIncluded in System instructions above; tool approval policy is unchanged.',
                  style: const TextStyle(fontSize: 12),
                ),
              ),
            ),
          if (report['skills'] is List && (report['skills'] as List).isNotEmpty)
            _ContextGroup(
              label: 'Skills · reviewed snapshots',
              messages: [
                for (final skill in report['skillEntries'] as List? ?? [])
                  {
                    'role':
                        '${skill['scope'] == 'global' ? 'Global' : 'Project'} · ${skill['document']['name']} · version ${skill['version']}',
                    'content': skill['document']['text'],
                  },
              ],
              color: colors[0],
            ),
          if (report['sessionSummary'] is Map)
            _ContextGroup(
              label: 'Session summary',
              messages: [
                {
                  'role':
                      'Reviewed background · revision ${report['summary']['revision']}',
                  'content': report['sessionSummary']['text'],
                },
              ],
              color: p.syntaxName,
            ),
          if (report['memory'] is Map) ...[
            Text(
              'Memory · ${(report['memory']['used'] as List).length} preferences used · ${report['memory']['omitted']} enabled preferences left out',
              style: TextStyle(color: p.muted, fontSize: 12),
            ),
            _ContextGroup(
              label: 'Saved preferences',
              messages: [
                for (final item in (report['memoryEntries'] as List? ?? []))
                  {
                    'role':
                        '${item['title']} · ${item['scope'] == 'folder' ? 'This working folder' : 'All chats'} · ${item['origin'] is Map ? '${item['source'] == 'automatic' ? 'Learned automatically' : 'Reviewed chat'} · message ${item['origin']['messageId']}' : 'Added by you'} · revision ${item['revision']}',
                    'content': item['text'],
                  },
              ],
              color: p.syntaxName,
            ),
          ],
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
    if (!chat.busy) {
      WidgetsBinding.instance.addPostFrameCallback((_) {
        if (mounted) _load();
      });
    }
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
      title: 'Activity',
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
                  label: Text('Conversation'),
                ),
                ButtonSegment(
                  value: 1,
                  icon: Icon(Icons.list_alt, size: 16),
                  label: Text('Requests'),
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
                              if (assistant) ...[
                                if (metadata?['agent']?['tools'] is List)
                                  ToolRecords(
                                    bridge: chat.bridge,
                                    records:
                                        metadata!['agent']['tools'] as List,
                                  ),
                                if (metadata?['agent']?['steps'] is List)
                                  ModelSteps(
                                    steps: metadata!['agent']['steps'] as List,
                                  ),
                                UsageDetails(metadata: metadata),
                                TaskFeedbackButton(
                                  chat: chat,
                                  message: message,
                                ),
                              ],
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
