import 'package:flutter/material.dart';

import 'bridge.dart';
import 'reply_content.dart';
import 'theme.dart';
import 'tool_activity.dart';

/// Public progress and receipts only. Provider thinking remains separate.
class AgentTimeline extends StatelessWidget {
  final List<dynamic> steps;
  final List<dynamic> records;
  final List<dynamic> toolSteps;
  final ChatBridge? bridge;
  final bool unsaved;
  final int? streamingStep;
  final VoidCallback? onRendered;

  const AgentTimeline({
    super.key,
    this.steps = const [],
    this.records = const [],
    this.toolSteps = const [],
    this.bridge,
    this.unsaved = false,
    this.streamingStep,
    this.onRendered,
  });

  @override
  Widget build(BuildContext context) {
    final p = Palette(Theme.of(context).brightness == Brightness.dark);
    final ordered =
        toolSteps.length == records.length &&
        toolSteps.every((n) => n is int && n >= 0) &&
        List.generate(
          toolSteps.length,
          (i) => i == 0 || (toolSteps[i] as int) >= (toolSteps[i - 1] as int),
        ).every((v) => v);
    final numbers = <int>{
      for (final step in steps)
        if (step['number'] is int) step['number'] as int,
      if (ordered) ...toolSteps.cast<int>(),
    }.toList()..sort();
    Widget progress(dynamic step) => Padding(
      key: ValueKey('timeline-progress-${step['number']}'),
      padding: const EdgeInsets.symmetric(vertical: 8),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          if (step['partial'] == true)
            Text(
              'Partial response',
              style: TextStyle(color: p.muted, fontSize: 12),
            ),
          ReplyContent(
            key: ValueKey('progress-${step['number']}'),
            text: step['text'] as String? ?? '',
            streaming: streamingStep == step['number'],
            onRendered: onRendered,
          ),
        ],
      ),
    );
    Widget tools(List<dynamic> items) => ToolRecords(
      key: ValueKey('timeline-tools-${items.first['callId']}'),
      records: items,
      maxRecords: items.length,
      compact: true,
      bridge: bridge,
    );
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      mainAxisSize: MainAxisSize.min,
      children: [
        if (unsaved && (steps.isNotEmpty || records.isNotEmpty))
          Text(
            'Progress · Not saved',
            style: TextStyle(color: p.muted, fontSize: 12),
          ),
        if (!ordered) ...[
          // Old history has no ordering evidence. Never guess the interleaving.
          if (records.isNotEmpty && steps.isNotEmpty)
            Text(
              'Earlier activity',
              style: TextStyle(color: p.muted, fontSize: 12),
            ),
          for (final step in steps) progress(step),
          if (records.isNotEmpty) tools(records),
        ] else
          for (final number in numbers) ...[
            for (final step in steps.where((s) => s['number'] == number))
              progress(step),
            if (toolSteps.contains(number))
              tools([
                for (var i = 0; i < records.length; i++)
                  if (toolSteps[i] == number) records[i],
              ]),
          ],
      ],
    );
  }
}
