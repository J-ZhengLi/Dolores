import 'package:flutter/material.dart';

/// The same contextual help is reachable by pointer, keyboard and touch.
class HelpTip extends StatefulWidget {
  final String message;
  const HelpTip({super.key, required this.message});
  @override
  State<HelpTip> createState() => _HelpTipState();
}

class _HelpTipState extends State<HelpTip> {
  final tooltip = GlobalKey<TooltipState>();
  @override
  Widget build(BuildContext context) => Focus(
    canRequestFocus: false,
    skipTraversal: true,
    onFocusChange: (focused) {
      if (focused) tooltip.currentState?.ensureTooltipVisible();
      if (!focused) Tooltip.dismissAllToolTips();
    },
    child: Tooltip(
      key: tooltip,
      message: widget.message,
      waitDuration: const Duration(milliseconds: 250),
      exitDuration: const Duration(milliseconds: 100),
      showDuration: const Duration(seconds: 30),
      constraints: const BoxConstraints(maxWidth: 320),
      padding: const EdgeInsets.all(12),
      margin: const EdgeInsets.all(16),
      child: IconButton(
        tooltip: null,
        visualDensity: VisualDensity.compact,
        constraints: const BoxConstraints(minWidth: 32, minHeight: 32),
        padding: const EdgeInsets.all(6),
        onPressed: () => tooltip.currentState?.ensureTooltipVisible(),
        icon: const Icon(Icons.help_outline, size: 16),
      ),
    ),
  );
}

class HelpLabel extends StatelessWidget {
  final Widget child;
  final String help;
  const HelpLabel({super.key, required this.child, required this.help});
  @override
  Widget build(BuildContext context) => Row(
    mainAxisSize: MainAxisSize.min,
    children: [
      Flexible(child: child),
      const SizedBox(width: 4),
      HelpTip(message: help),
    ],
  );
}
