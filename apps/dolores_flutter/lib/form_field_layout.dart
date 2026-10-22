import 'package:flutter/material.dart';

import 'help_tip.dart';

/// Stable labels and help align with the control's outer edge, outside its border.
class FieldLabel extends StatelessWidget {
  final String label;
  final String? help;
  final Widget child;
  const FieldLabel({
    super.key,
    required this.label,
    this.help,
    required this.child,
  });

  @override
  Widget build(BuildContext context) => Column(
    crossAxisAlignment: CrossAxisAlignment.stretch,
    mainAxisSize: MainAxisSize.min,
    children: [
      ConstrainedBox(
        constraints: const BoxConstraints(minHeight: 32),
        child: Row(
          children: [
            Flexible(
              child: ExcludeSemantics(
                child: Text(
                  label,
                  style: Theme.of(context).textTheme.bodyMedium
                      ?.copyWith(fontWeight: FontWeight.w600),
                ),
              ),
            ),
            if (help != null && help!.length > 80) HelpTip(message: help!),
          ],
        ),
      ),
      const SizedBox(height: 8),
      Semantics(label: label, child: child),
      if (help != null && help!.isNotEmpty && help!.length <= 80) ...[
        const SizedBox(height: 8),
        Text(
          help!,
          style: Theme.of(context).textTheme.bodySmall
              ?.copyWith(color: Theme.of(context).colorScheme.onSurfaceVariant),
        ),
      ],
    ],
  );
}
