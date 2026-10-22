import 'package:flutter/material.dart';

import 'help_tip.dart';
import 'theme.dart';

String formatTokens(dynamic value) => value is int && value >= 0
    ? value.toString().replaceAllMapped(
        RegExp(r'(\d)(?=(\d{3})+(?!\d))'),
        (m) => '${m[1]},',
      )
    : 'Unknown';
String _counter(dynamic value) =>
    value is int && value >= 0 ? formatTokens(value) : 'Unavailable';

class DetailRow extends StatelessWidget {
  final String label, value;
  const DetailRow(this.label, this.value, {super.key});
  @override
  Widget build(BuildContext context) => Padding(
    padding: const EdgeInsets.symmetric(vertical: 6),
    child: Row(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        Expanded(
          flex: 2,
          child: Text(
            label,
            style: TextStyle(
              color: Theme.of(context).colorScheme.onSurfaceVariant,
            ),
          ),
        ),
        const SizedBox(width: 16),
        Expanded(
          flex: 3,
          child: SelectableText(value, textAlign: TextAlign.right),
        ),
      ],
    ),
  );
}

Widget contextDetails(Map<String, dynamic> summary, {bool preview = false}) {
  final tokens = summary['tokens'] as Map?;
  final memory = summary['memory'] as Map?;
  final used = memory?['used'] as List? ?? [];
  final skills = summary['skills'] as List? ?? [];
  return Column(
    crossAxisAlignment: CrossAxisAlignment.stretch,
    mainAxisSize: MainAxisSize.min,
    children: [
      const SizedBox(height: 16),
      HelpLabel(
        help: 'Initial request before tool results. Local estimates include instructions, messages, tools and framing, separate from provider usage. Older turns remain saved.',
        child: Text(
          preview ? 'Next message context' : 'Initial context',
          style: const TextStyle(fontWeight: FontWeight.w600),
        ),
      ),
      DetailRow('Input estimate', '${_counter(tokens?['inputTokens'])} tokens'),
      DetailRow(
        'Context window',
        '${_counter(tokens?['contextWindowTokens'])} tokens',
      ),
      DetailRow(
        'Conversation',
        '${_counter(summary['includedTurns'])} turns included',
      ),
      if (summary['omittedTurns'] != 0)
        DetailRow(
          'Left out',
          '${_counter(summary['omittedTurns'])} older turns',
        ),
      if (memory != null)
        DetailRow(
          'Memory',
          '${used.length} used · ${_counter(memory['omitted'])} left out',
        ),
      ExpansionTile(
        key: const Key('context-breakdown'),
        title: const Text('Context details'),
        children: [
          DetailRow('Saved turns', _counter(summary['savedTurns'])),
          DetailRow(
            'Response reserved',
            _counter(tokens?['reservedOutputTokens']),
          ),
          DetailRow('Input allowance', _counter(tokens?['maxInputTokens'])),
          if (summary['summary'] is Map)
            SelectableText(
              'Session summary: ${summary['summary']['coveredTurns']} turns covered · revision ${summary['summary']['revision']}\nDrafted by ${summary['summary']['model']}',
            ),
        ],
      ),
      if (summary['instructions'] is Map ||
          skills.isNotEmpty ||
          used.isNotEmpty)
        ExpansionTile(
          key: const Key('context-sources'),
          title: const Text('Sources'),
          children: [
            if (summary['instructions'] is Map)
              SelectableText(
                'Workspace instructions: ${summary['instructions']['source']}\nReviewed revision: ${summary['instructions']['revision']}',
              ),
            for (final skill in skills)
              SelectableText(
                '${skill['scope'] == 'global' ? 'Global' : 'Project'} skill: ${skill['name']} · version ${skill['version']}\n${skill['source']}${skill['rollbackFrom'] == null ? '' : '\nRolled back from version ${skill['rollbackFrom']}'}',
              ),
            for (final item in used) ...[
              SelectableText(
                '${item['title']} · ${item['scope'] == 'folder' ? 'This working folder' : 'All chats'} · ${item['origin'] is Map ? '${item['source'] == 'automatic' ? 'Learned automatically' : 'Reviewed chat'} · message ${item['origin']['messageId']}' : 'Added by you'} · revision ${item['revision']}',
              ),
              if (item['origin'] is Map)
                SelectableText(
                  'Source quote: ${item['origin']['quote']}\nSuggested by ${item['origin']['model']}',
                ),
            ],
          ],
        ),
    ],
  );
}

class UsagePanel extends StatelessWidget {
  final Map<String, dynamic>? metadata;
  const UsagePanel({super.key, this.metadata});
  @override
  Widget build(BuildContext context) {
    final agent = metadata?['agent'] as Map?;
    final calls = agent == null
        ? [metadata?['usage'] as Map?]
        : (agent['usageByCall'] as List? ?? []).map((v) => v as Map?).toList();
    return Column(
      crossAxisAlignment: CrossAxisAlignment.stretch,
      mainAxisSize: MainAxisSize.min,
      children: [
        SelectableText(
          '${metadata?['model'] ?? 'Model unavailable'}',
          style: Theme.of(context).textTheme.titleMedium,
        ),
        if (metadata?['paused'] is Map)
          Padding(
            padding: const EdgeInsets.only(top: 12),
            child: Text(
              'Paused: ${metadata!['paused']['reason']}. Progress remains saved.',
            ),
          ),
        const SizedBox(height: 16),
        HelpLabel(
          help: 'Reported by the provider, per model call. No run total is inferred. Cached input and reasoning are breakdowns, not additional totals. Missing values are unavailable; no estimates or prices are added.',
          child: Text(
            agent == null
                ? 'Tokens'
                : '${_counter(agent['modelCalls'])} model ${agent['modelCalls'] == 1 ? 'call' : 'calls'} · tokens per call',
            style: const TextStyle(fontWeight: FontWeight.w600),
          ),
        ),
        if (calls.isEmpty) const Text('Usage unavailable'),
        for (final (index, usage) in calls.indexed)
          Container(
            margin: const EdgeInsets.only(top: 8),
            padding: const EdgeInsets.all(12),
            decoration: BoxDecoration(
              color: Theme.of(context).colorScheme.surfaceContainerHighest
                  .withValues(alpha: .4),
              borderRadius: BorderRadius.circular(12),
            ),
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.stretch,
              children: [
                if (agent != null) ...[
                  Text(
                    'Call ${index + 1}',
                    style: Theme.of(context).textTheme.labelMedium,
                  ),
                  const SizedBox(height: 8),
                ],
                Row(
                  children: [
                    for (final entry in const [
                      ('Input', 'inputTokens'),
                      ('Output', 'outputTokens'),
                      ('Total', 'totalTokens'),
                    ])
                      Expanded(
                        child: Padding(
                          padding: const EdgeInsets.only(right: 8),
                          child: Column(
                            crossAxisAlignment: CrossAxisAlignment.start,
                            children: [
                              Text(
                                entry.$1,
                                style: Theme.of(context).textTheme.bodySmall,
                              ),
                              const SizedBox(height: 4),
                              FittedBox(
                                fit: BoxFit.scaleDown,
                                alignment: Alignment.centerLeft,
                                child: Text(
                                  _counter(usage?[entry.$2]),
                                  style: const TextStyle(
                                    fontSize: 20,
                                    fontWeight: FontWeight.w600,
                                  ),
                                ),
                              ),
                            ],
                          ),
                        ),
                      ),
                  ],
                ),
              ],
            ),
          ),
        ExpansionTile(
          key: const Key('token-breakdown'),
          title: const Text('Token breakdown'),
          children: [
            for (final (index, usage) in calls.indexed) ...[
              if (agent != null)
                Align(
                  alignment: Alignment.centerLeft,
                  child: Text('Call ${index + 1}'),
                ),
              DetailRow('Cached input', _counter(usage?['cachedInputTokens'])),
              DetailRow('Reasoning', _counter(usage?['reasoningTokens'])),
            ],
          ],
        ),
        if (metadata?['context'] is Map)
          contextDetails((metadata!['context'] as Map).cast<String, dynamic>())
        else
          const Padding(
            padding: EdgeInsets.symmetric(vertical: 12),
            child: Text('Saved request context: Unavailable'),
          ),
        if (metadata?['requestSettings'] is Map)
          ExpansionTile(
            key: const Key('usage-request-settings'),
            title: const Text('Request settings'),
            children: [
              DetailRow(
                'Output limit',
                metadata!['requestSettings']['maxOutputTokens'] == null
                    ? 'Provider default'
                    : _counter(metadata!['requestSettings']['maxOutputTokens']),
              ),
              DetailRow(
                'Stall timeout',
                '${metadata!['requestSettings']['timeoutSeconds']} seconds',
              ),
            ],
          ),
      ],
    );
  }
}

class UsageDetails extends StatelessWidget {
  final Map<String, dynamic>? metadata;
  const UsageDetails({super.key, this.metadata});
  @override
  Widget build(BuildContext context) {
    final p = Palette(Theme.of(context).brightness == Brightness.dark);
    final usage = metadata?['usage'] as Map?;
    final label = metadata?['agent'] is Map
        ? 'Run details · ${metadata!['agent']['modelCalls']} model calls'
        : usage == null
        ? 'Usage unavailable'
        : 'Tokens: ${_counter(usage['inputTokens'])} in · ${_counter(usage['outputTokens'])} out';
    return Align(
      alignment: Alignment.centerLeft,
      child: TextButton(
        style: TextButton.styleFrom(
          foregroundColor: p.muted,
          padding: const EdgeInsets.symmetric(vertical: 4),
          textStyle: Theme.of(context).textTheme.bodySmall,
        ),
        onPressed: () => showDialog<void>(
          context: context,
          animationStyle: surfaceAnimation(context),
          builder: (context) => AlertDialog(
            title: const Text('Run details'),
            content: SizedBox(
              width: 480,
              child: SingleChildScrollView(
                child: UsagePanel(metadata: metadata),
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
        child: Text(label),
      ),
    );
  }
}
