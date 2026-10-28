import 'dart:convert';

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
    Widget tools(List<dynamic> items) => ToolExecutionGroup(
      key: ValueKey('timeline-tools-${items.first['callId']}'),
      records: items,
      bridge: bridge,
    );
    final timeline = <Widget>[];
    var pendingTools = <dynamic>[];
    if (ordered) {
      for (final number in numbers) {
        for (final step in steps.where((s) => s['number'] == number)) {
          if ((step['text'] as String? ?? '').trim().isEmpty) continue;
          if (pendingTools.isNotEmpty) {
            timeline.add(tools(pendingTools));
            pendingTools = [];
          }
          timeline.add(progress(step));
        }
        for (var i = 0; i < records.length; i++) {
          if (toolSteps[i] == number) pendingTools.add(records[i]);
        }
      }
      if (pendingTools.isNotEmpty) timeline.add(tools(pendingTools));
    }
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
          ...timeline,
      ],
    );
  }
}

/// Consecutive tool calls share one disclosure until the next public output.
class ToolExecutionGroup extends StatelessWidget {
  final List<dynamic> records;
  final ChatBridge? bridge;
  const ToolExecutionGroup({super.key, required this.records, this.bridge});

  @override
  Widget build(BuildContext context) {
    final p = Palette(Theme.of(context).brightness == Brightness.dark);
    final issues = records
        .where(
          (r) => [
            'denied',
            'blocked',
            'error',
            'failed',
            'incomplete',
          ].contains(r['status']),
        )
        .length;
    final label = records.length == 1
        ? '${toolLabel(records.first['name'])} · ${scheduledTaskTitle(records.first)}'
        : '${records.length} tool executions';
    return Padding(
      padding: const EdgeInsets.symmetric(vertical: 4),
      child: ExpansionTile(
        key: PageStorageKey(
          'tool-group-${records.first['callId'] ?? jsonEncode(records.first)}',
        ),
        dense: true,
        minTileHeight: 40,
        tilePadding: const EdgeInsets.symmetric(horizontal: 12),
        childrenPadding: const EdgeInsets.only(left: 12),
        shape: const Border(),
        collapsedShape: const Border(),
        leading: Icon(
          issues > 0 ? Icons.warning_amber_rounded : Icons.terminal_outlined,
          size: 16,
          color: p.muted,
        ),
        title: Text(
          '$label${issues > 0 ? " · $issues ${issues == 1 ? "issue" : "issues"}" : ""}',
          maxLines: 2,
          overflow: TextOverflow.ellipsis,
          style: TextStyle(color: p.muted, fontSize: 12),
        ),
        children: [
          ToolRecords(
            records: records,
            maxRecords: records.length,
            compact: true,
            bridge: bridge,
          ),
        ],
      ),
    );
  }
}
