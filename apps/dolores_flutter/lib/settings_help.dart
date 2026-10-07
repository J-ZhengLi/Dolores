import 'package:flutter/material.dart';

/// Keeps setting explanations out of the form until the pointer requests them.
class SettingsHelpLabel extends StatelessWidget {
  final String label, help;
  final Key? helpKey;
  const SettingsHelpLabel({
    super.key,
    required this.label,
    required this.help,
    this.helpKey,
  });

  @override
  Widget build(BuildContext context) => Row(
    mainAxisSize: MainAxisSize.min,
    children: [
      Flexible(child: Text(label)),
      const SizedBox(width: 6),
      Tooltip(
        message: help,
        triggerMode: TooltipTriggerMode.manual,
        waitDuration: const Duration(milliseconds: 400),
        exitDuration: const Duration(milliseconds: 100),
        constraints: const BoxConstraints(maxWidth: 320),
        padding: const EdgeInsets.all(12),
        child: GestureDetector(
          key: helpKey,
          // A click on help must not toggle its parent SwitchListTile.
          onTap: () {},
          onLongPress: () {},
          excludeFromSemantics: true,
          behavior: HitTestBehavior.opaque,
          child: MouseRegion(
            cursor: SystemMouseCursors.help,
            child: Padding(
              padding: const EdgeInsets.all(4),
              child: Icon(
                Icons.help_outline,
                size: 16,
                color: Theme.of(context).colorScheme.onSurfaceVariant,
              ),
            ),
          ),
        ),
      ),
    ],
  );
}
