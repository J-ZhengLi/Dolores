import 'package:flutter/material.dart';

import 'settings_frame.dart';

class AdvancedHome extends StatelessWidget {
  final void Function(String, String) open;
  const AdvancedHome({super.key, required this.open});

  Widget entry(
    String title,
    IconData icon,
    String category, [
    String page = 'connection',
  ]) => ListTile(
    title: Text(title),
    leading: Icon(icon),
    trailing: const Icon(Icons.chevron_right),
    onTap: () => open(category, page),
  );

  @override
  Widget build(BuildContext context) => EmbeddedSettingsFrame(
    title: 'Advanced',
    subtitle: 'Tuning, testing and troubleshooting',
    canClose: true,
    pending: SettingsEmbedding.of(context)!.pending,
    child: ListView(
      padding: const EdgeInsets.all(16),
      children: [
        entry('Task limits', Icons.timelapse, 'limits'),
        entry('Response defaults', Icons.tune, 'models', 'defaults'),
        entry('Scope overrides', Icons.layers_outlined, 'models', 'overrides'),
        ExpansionTile(
          title: const Text('Skill testing & learning'),
          leading: const Icon(Icons.school_outlined),
          children: [
            entry('Skill testing', Icons.science_outlined, 'skillTesting'),
            entry(
              'Learning experiments',
              Icons.psychology_outlined,
              'skillLearning',
            ),
          ],
        ),
        entry('Harness extensions', Icons.extension_outlined, 'mods'),
        ExpansionTile(
          title: const Text('Diagnostics & storage'),
          leading: const Icon(Icons.build_outlined),
          children: [
            entry('Compare instructions', Icons.compare_arrows, 'comparisons'),
            entry('Capabilities & source', Icons.code, 'capabilities'),
            entry('Native repairs', Icons.restore, 'nativeRepairs'),
            entry('Attachment storage', Icons.inventory_2_outlined, 'storage'),
          ],
        ),
      ],
    ),
  );
}
