import 'package:flutter/material.dart';

import 'theme.dart';

String _counter(dynamic value) =>
    value is int && value >= 0 ? '$value' : 'Unavailable';

Widget contextDetails(
  Map<String, dynamic> summary, {
  bool preview = false,
}) => Column(
  crossAxisAlignment: CrossAxisAlignment.start,
  mainAxisSize: MainAxisSize.min,
  children: [
    Text('Recent turns included: ${_counter(summary['includedTurns'])}'),
    Text('Saved turns: ${_counter(summary['savedTurns'])}'),
    Text('Older turns left out: ${_counter(summary['omittedTurns'])}'),
    const SizedBox(height: 12),
    Text(
      'Text ${preview ? 'to send' : 'sent'}: ${_counter(summary['textBytes'])} bytes',
    ),
    Text(
      'App limits: ${_counter(summary['maxTurns'])} recent turns · ${_counter(summary['maxTextBytes'])} text bytes',
    ),
    const SizedBox(height: 12),
    const Text(
      'Includes your message and system instructions. Older turns remain saved. These are app limits, not the model’s token window.',
    ),
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
    final label = usage == null
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
                    Text('Input tokens: ${_counter(input)}'),
                    Text('Output tokens: ${_counter(output)}'),
                    Text('Total tokens: ${_counter(usage?['totalTokens'])}'),
                    Text(
                      'Cached input tokens: ${_counter(usage?['cachedInputTokens'])}',
                    ),
                    Text(
                      'Reasoning tokens: ${_counter(usage?['reasoningTokens'])}',
                    ),
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
