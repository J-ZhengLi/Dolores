import 'dart:convert';

import 'package:flutter/material.dart';

import 'agent_timeline.dart';
import 'bridge.dart';

/// Stored execution windows are an implementation detail of one conversation.
/// Reading their original output is local and never changes the model context.
class TaskTranscript extends StatefulWidget {
  final ChatBridge bridge;
  final String? session;
  final Map? archive;
  final Map<String, dynamic> current;
  final int? streamingStep;
  final VoidCallback? onRendered;
  final Widget Function(List<dynamic>) thinking;

  const TaskTranscript({
    super.key,
    required this.bridge,
    required this.session,
    required this.current,
    required this.thinking,
    this.archive,
    this.streamingStep,
    this.onRendered,
  });

  @override
  State<TaskTranscript> createState() => _TaskTranscriptState();
}

class _TaskTranscriptState extends State<TaskTranscript> {
  final pages = <Map>[];
  String? error;
  bool loading = false;
  int generation = 0;

  @override
  void initState() {
    super.initState();
    load();
  }

  @override
  void didUpdateWidget(TaskTranscript oldWidget) {
    super.didUpdateWidget(oldWidget);
    if (oldWidget.session != widget.session ||
        oldWidget.archive?['runId'] != widget.archive?['runId']) {
      generation++;
      pages.clear();
      error = null;
      loading = false;
      load();
    }
  }

  Future<void> load() async {
    final run = widget.archive?['runId'];
    if (loading || run == null || widget.session == null) return;
    final revision = generation;
    setState(() {
      loading = true;
      error = null;
    });
    try {
      while (mounted && revision == generation) {
        final result = await widget.bridge.call({
          'command': 'runProgress',
          'session': widget.session,
          'runId': run,
          'ordinal': pages.length + 1,
        }) as Map;
        if (!mounted || revision != generation) return;
        if (result['summary'] is! Map) break;
        setState(() => pages.add(result['summary'] as Map));
        if ((result['ordinal'] as int) >= (result['total'] as int)) break;
      }
    } catch (failure) {
      if (mounted && revision == generation) {
        setState(
          () =>
              error = 'Some conversation output could not be loaded: $failure',
        );
      }
    } finally {
      if (mounted && revision == generation) setState(() => loading = false);
    }
  }

  @override
  Widget build(BuildContext context) {
    final progress = pages.isEmpty
        ? widget.current
        : mergeTaskProgress([...pages, widget.current]);
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        if (loading) const LinearProgressIndicator(),
        if (error != null) ...[
          SelectableText(error!),
          TextButton(
            onPressed: load,
            child: const Text('Retry loading conversation'),
          ),
        ],
        widget.thinking(progress['thinking'] as List? ?? []),
        AgentTimeline(
          steps: progress['steps'] as List? ?? [],
          records: progress['tools'] as List? ?? [],
          toolSteps: progress['toolSteps'] as List? ?? [],
          bridge: widget.bridge,
          streamingStep: widget.streamingStep,
          onRendered: widget.onRendered,
        ),
      ],
    );
  }
}

Map<String, dynamic> mergeTaskProgress(List<Map> windows) {
  final steps = <dynamic, dynamic>{}, thinking = <dynamic, dynamic>{};
  final receipts = <String>{};
  final tools = <dynamic>[], toolSteps = <dynamic>[];
  bool ordered = true;
  for (final window in windows) {
    for (final step in window['steps'] as List? ?? []) {
      steps[step['number']] = step;
    }
    for (final thought in window['thinking'] as List? ?? []) {
      thinking[thought['number']] = thought;
    }
    final records = window['tools'] as List? ?? [];
    final numbers = window['toolSteps'] as List? ?? [];
    for (var i = 0; i < records.length; i++) {
      final record = records[i];
      // Unresolved receipts can be carried across multiple stored windows.
      if (!receipts.add(record['callId'] as String? ?? jsonEncode(record))) {
        continue;
      }
      tools.add(record);
      if (i < numbers.length && numbers[i] is int && numbers[i] >= 0) {
        toolSteps.add(numbers[i]);
      } else {
        ordered = false;
      }
    }
  }
  return {
    'steps': steps.values.toList(),
    'thinking': thinking.values.toList(),
    'tools': tools,
    'toolSteps': ordered ? toolSteps : <dynamic>[],
  };
}
