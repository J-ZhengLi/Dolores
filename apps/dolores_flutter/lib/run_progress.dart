import 'dart:async';
import 'dart:math';

import 'package:flutter/material.dart';

import 'theme.dart';

String formatRunElapsed(Duration elapsed) {
  final seconds = elapsed.inSeconds;
  if (seconds < 60) return '${seconds}s';
  if (seconds < 3600) return '${seconds ~/ 60}m ${seconds % 60}s';
  return '${seconds ~/ 3600}h ${(seconds ~/ 60) % 60}m';
}

/// Updates only this label; a clock tick must not rebuild or scroll the chat.
class RunElapsed extends StatefulWidget {
  final Duration Function() elapsed;
  final bool queued;
  const RunElapsed({super.key, required this.elapsed, this.queued = false});
  @override
  State<RunElapsed> createState() => _RunElapsedState();
}

class _RunElapsedState extends State<RunElapsed> {
  Timer? timer;
  @override
  void initState() {
    super.initState();
    timer = Timer.periodic(const Duration(seconds: 1), (_) => setState(() {}));
  }

  @override
  void dispose() {
    timer?.cancel();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) => Padding(
    key: const Key('run-elapsed'),
    padding: const EdgeInsets.only(bottom: 12),
    child: Text(
      '${widget.queued ? 'Queued' : 'Working'} for ${formatRunElapsed(widget.elapsed())}',
      style: Theme.of(context).textTheme.bodySmall?.copyWith(
        color: Palette(Theme.of(context).brightness == Brightness.dark).muted,
      ),
    ),
  );
}

class RunStatus extends StatefulWidget {
  final String phase;
  final String? detail;
  const RunStatus({super.key, required this.phase, this.detail});
  @override
  State<RunStatus> createState() => _RunStatusState();
}

class _RunStatusState extends State<RunStatus>
    with SingleTickerProviderStateMixin {
  static const fixed = {
    'queued': 'Waiting for a task slot',
    'review': 'Waiting for your review',
    'question': 'Waiting for your answer',
    'stopping': 'Stopping…',
    'compacting': 'Updating context',
    'retrying': 'Reconnecting to the model',
    'waitingComplete': 'Waiting for a complete response',
  };
  static const phrases = {
    'reasoning': [
      'Thinking it through',
      'Connecting the dots',
      'Working through the details',
      'Weighing the next step',
    ],
    'responding': [
      'Writing the response',
      'Putting the answer together',
      'Shaping the response',
    ],
    'toolArguments': [
      'Preparing the next action',
      'Putting a tool call together',
      'Setting up the next step',
    ],
    'waiting': ['Waiting for the model', 'Awaiting the next response'],
    'working': [
      'Working on it',
      'Following through',
      'Working through the next step',
    ],
  };
  final random = Random();
  late final pulse = AnimationController(
    vsync: this,
    lowerBound: .55,
    upperBound: 1,
    value: 1,
    duration: const Duration(milliseconds: 1400),
  );
  Timer? rotation;
  String label = '';
  String choose() {
    if (fixed.containsKey(widget.phase)) return fixed[widget.phase]!;
    final choices = (phrases[widget.phase] ?? phrases['working']!)
        .where((s) => s != label)
        .toList();
    return choices[random.nextInt(choices.length)];
  }

  @override
  void initState() {
    super.initState();
    label = choose();
  }

  @override
  void didChangeDependencies() {
    super.didChangeDependencies();
    syncMotion();
  }

  @override
  void didUpdateWidget(RunStatus oldWidget) {
    super.didUpdateWidget(oldWidget);
    if (widget.phase != oldWidget.phase) {
      label = choose();
      rotation?.cancel();
      rotation = null;
    }
    syncMotion();
  }

  void syncMotion() {
    final moving =
        !MediaQuery.disableAnimationsOf(context) &&
        !MediaQuery.accessibleNavigationOf(context) &&
        TickerMode.valuesOf(context).enabled &&
        !['queued', 'review', 'question', 'stopping'].contains(widget.phase);
    if (moving) {
      if (!pulse.isAnimating) pulse.repeat(reverse: true);
      if (!fixed.containsKey(widget.phase)) {
        rotation ??= Timer.periodic(
          const Duration(seconds: 8),
          (_) => setState(() => label = choose()),
        );
      }
    } else {
      rotation?.cancel();
      rotation = null;
      pulse.stop();
      pulse.value = 1;
    }
  }

  @override
  void dispose() {
    rotation?.cancel();
    pulse.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) => Padding(
    key: const Key('run-status'),
    padding: const EdgeInsets.only(top: 12, bottom: 4),
    child: FadeTransition(
      opacity: pulse,
      child: Text(
        widget.detail == null ? label : '$label\n${widget.detail}',
        maxLines: widget.detail == null ? 1 : 3,
        overflow: TextOverflow.ellipsis,
        style: Theme.of(context).textTheme.bodySmall?.copyWith(
          color: Palette(Theme.of(context).brightness == Brightness.dark).muted,
        ),
      ),
    ),
  );
}
