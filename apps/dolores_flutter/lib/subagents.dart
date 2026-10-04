import 'package:flutter/material.dart';

import 'theme.dart';

/// Reports remain literal. Run history holds detailed child tool evidence.
class SubagentCards extends StatelessWidget {
  final List<dynamic> children;
  const SubagentCards({super.key, required this.children});

  @override
  Widget build(BuildContext context) {
    final p = Palette(Theme.of(context).brightness == Brightness.dark);
    return Column(
      mainAxisSize: MainAxisSize.min,
      children: [
        for (final child in children.take(2))
          ExpansionTile(
            key: PageStorageKey('subagent-${child['childId']}'),
            tilePadding: const EdgeInsets.symmetric(horizontal: 8),
            title: Text(
              '${child['goal']}',
              maxLines: 2,
              overflow: TextOverflow.ellipsis,
              style: const TextStyle(fontSize: 12),
            ),
            subtitle: Text(
              'Subagent · ${child['status']} · ${child['scope']} · '
              '${child['readOnly'] == true ? 'Read only' : 'File writes'}',
              maxLines: 2,
              overflow: TextOverflow.ellipsis,
              style: TextStyle(fontSize: 11, color: p.muted),
            ),
            children: [
              Padding(
                padding: const EdgeInsets.all(8),
                child: ConstrainedBox(
                  constraints: const BoxConstraints(maxHeight: 180),
                  child: SingleChildScrollView(
                    key: PageStorageKey('subagent-report-${child['childId']}'),
                    child: SelectableText(
                      '${child['goal']}\n'
                      'Child: ${child['childId']}\nScope: ${child['scope']}\nStatus: ${child['status']}\n'
                      '${child['modelCalls'] == null ? '' : 'Child model calls: ${child['modelCalls']}\n'}'
                      '${child['truncated'] == true ? 'Report shortened.\n' : ''}'
                      '${child['answer'] ?? child['note'] ?? 'Working within the parent task limits.'}\n\n'
                      'Reported work needs parent verification. Detailed tool evidence is in Run history; applied writes remain in Changes.',
                      style: TextStyle(fontSize: 12, color: p.text),
                    ),
                  ),
                ),
              ),
            ],
          ),
      ],
    );
  }
}
