import 'package:flutter/material.dart';

import 'theme.dart';

String _counter(dynamic value) =>
    value is int && value >= 0 ? '$value' : 'Unavailable';

String formatTokens(dynamic value) => value is int && value >= 0
    ? value.toString().replaceAllMapped(
        RegExp(r'(\d)(?=(\d{3})+(?!\d))'),
        (m) => '${m[1]},',
      )
    : 'Unknown';

Widget contextDetails(
  Map<String, dynamic> summary, {
  bool preview = false,
}) => Column(
  crossAxisAlignment: CrossAxisAlignment.start,
  mainAxisSize: MainAxisSize.min,
  children: [
    Text('Recent turns included: ${_counter(summary['includedTurns'])}'),
    Text('Saved turns: ${_counter(summary['savedTurns'])}'),
    if (summary['summary'] is Map)
      SelectableText(
        'Session summary: ${summary['summary']['coveredTurns']} turns covered · revision ${summary['summary']['revision']}\nDrafted by ${summary['summary']['model']}',
      ),
    Text('Older turns left out: ${_counter(summary['omittedTurns'])}'),
    const SizedBox(height: 12),
    Text(
      'Estimated input tokens: ${_counter(summary['tokens']?['inputTokens'])}',
    ),
    Text(
      'Model context window: ${_counter(summary['tokens']?['contextWindowTokens'])} tokens',
    ),
    if (summary['tokens'] is Map) ...[
      Text(
        'Response reserved: ${_counter(summary['tokens']['reservedOutputTokens'])} tokens',
      ),
      Text(
        'Input allowance: ${_counter(summary['tokens']['maxInputTokens'])} tokens',
      ),
    ],
    const SizedBox(height: 12),
    const Text(
      'Includes instructions, messages, advertised tools and approximate framing. Older turns remain saved. Estimates are separate from provider-reported usage; legacy context estimates are unavailable.',
    ),
    if (summary['instructions'] is Map) ...[
      const SizedBox(height: 12),
      SelectableText(
        'Workspace instructions: ${summary['instructions']['source']}\nReviewed revision: ${summary['instructions']['revision']}',
      ),
    ],
    if (summary['skills'] is List)
      for (final skill in summary['skills'] as List)
        Text(
          'Project skill: ${skill['name']} · version ${skill['version']}\n${skill['source']}${skill['rollbackFrom'] == null ? '' : '\nRolled back from version ${skill['rollbackFrom']}'}',
        ),
    if (summary['memory'] is Map) ...[
      const SizedBox(height: 12),
      Text(
        'Memory: ${(summary['memory']['used'] as List).length} preferences used · ${summary['memory']['omitted']} enabled preferences left out',
      ),
      for (final item in summary['memory']['used'] as List)
        SelectableText(
          '${item['title']} · ${item['scope'] == 'folder' ? 'This working folder' : 'All chats'} · ${item['origin'] is Map ? '${item['source'] == 'automatic' ? 'Learned automatically' : 'Reviewed chat'} · message ${item['origin']['messageId']}' : 'Added by you'} · revision ${item['revision']}',
        ),
      for (final item in summary['memory']['used'] as List)
        if (item['origin'] is Map)
          SelectableText(
            'Source quote: ${item['origin']['quote']}\nSuggested by ${item['origin']['model']}',
          ),
    ],
  ],
);

class UsageDetails extends StatelessWidget {
  final Map<String, dynamic>? metadata;
  const UsageDetails({super.key, this.metadata});

  @override
  Widget build(BuildContext context) {
    final p = Palette(Theme.of(context).brightness == Brightness.dark);
    final usage = metadata?['usage'] as Map?;
    final input = usage?['inputTokens'], output = usage?['outputTokens'];
    final label = metadata?['agent'] is Map
        ? 'Run details · ${metadata!['agent']['modelCalls']} model calls'
        : usage == null
        ? 'Usage unavailable'
        : 'Tokens: ${_counter(input)} in · ${_counter(output)} out';
    return Align(
      alignment: Alignment.centerLeft,
      child: TextButton(
        style: TextButton.styleFrom(
          foregroundColor: p.muted,
          padding: const EdgeInsets.symmetric(horizontal: 0, vertical: 4),
          textStyle: const TextStyle(fontSize: 11),
        ),
        onPressed: () => showDialog<void>(
          context: context,
          builder: (context) => AlertDialog(
            title: const Text('Reply usage and context'),
            content: SizedBox(
              width: 440,
              child: SingleChildScrollView(
                child: Column(
                  crossAxisAlignment: CrossAxisAlignment.start,
                  mainAxisSize: MainAxisSize.min,
                  children: [
                    Text('Model: ${metadata?['model'] ?? 'Unavailable'}'),
                    if (metadata?['requestSettings'] is Map) ...[
                      Text(
                        'Output token limit: ${metadata!['requestSettings']['maxOutputTokens']}',
                      ),
                      Text(
                        'Request timeout: ${metadata!['requestSettings']['timeoutSeconds']} seconds',
                      ),
                    ],
                    const SizedBox(height: 12),
                    if (metadata?['agent'] is Map) ...[
                      const Text(
                        'Usage is shown per model call; no run total is inferred.',
                      ),
                      for (final (i, usage)
                          in (metadata!['agent']['usageByCall'] as List)
                              .indexed)
                        Text(
                          'Call ${i + 1}: ${_counter(usage?['inputTokens'])} in · ${_counter(usage?['outputTokens'])} out · ${_counter(usage?['totalTokens'])} total · ${_counter(usage?['cachedInputTokens'])} cached · ${_counter(usage?['reasoningTokens'])} reasoning',
                        ),
                      const Text(
                        'Context below describes the initial request before tool results.',
                      ),
                      const SizedBox(height: 12),
                    ],
                    if (metadata?['agent'] is! Map) ...[
                      Text('Input tokens: ${_counter(input)}'),
                      Text('Output tokens: ${_counter(output)}'),
                      Text('Total tokens: ${_counter(usage?['totalTokens'])}'),
                      Text(
                        'Cached input tokens: ${_counter(usage?['cachedInputTokens'])}',
                      ),
                      Text(
                        'Reasoning tokens: ${_counter(usage?['reasoningTokens'])}',
                      ),
                    ],
                    const SizedBox(height: 12),
                    const Text(
                      'Reported by the provider. Cached and reasoning tokens are breakdowns, not additional totals. Missing values are unavailable; no token estimates or prices are added.',
                    ),
                    const SizedBox(height: 20),
                    if (metadata?['context'] is Map)
                      contextDetails(
                        (metadata!['context'] as Map).cast<String, dynamic>(),
                      )
                    else
                      const Text('Saved request context: Unavailable'),
                  ],
                ),
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
