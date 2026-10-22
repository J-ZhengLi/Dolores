import 'package:flutter/material.dart';
import 'help_tip.dart';

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
      HelpTip(key: helpKey, message: help),
    ],
  );
}
