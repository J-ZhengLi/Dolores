import 'package:flutter/material.dart';

import 'theme.dart';

/// Reports remain literal. Run history holds detailed child tool evidence.
class SubagentCards extends StatelessWidget {
  final List<dynamic> children;
  final Future<Map<String, dynamic>> Function(
    String child,
    String action, {
    int? offset,
  })?
  onSupervise;
  const SubagentCards({super.key, required this.children, this.onSupervise});

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
                      '${child['model'] == null ? '' : 'Model: ${child['model']} (frozen)\n'}'
                      '${child['usage'] == null ? '' : 'Cumulative attempts: ${child['usage']['modelCalls']}/4 model calls · ${child['usage']['toolCalls']}/4 file operations\n'}'
                      '${child['modelCalls'] == null ? '' : 'Child model calls: ${child['modelCalls']}\n'}'
                      '${child['truncated'] == true ? 'Report shortened.\n' : ''}'
                      '${child['answer'] ?? child['note'] ?? 'Working within the parent task limits.'}\n\n'
                      'Reported work needs parent verification. Detailed tool evidence is in Run history; applied writes remain in Changes.',
                      style: TextStyle(fontSize: 12, color: p.text),
                    ),
                  ),
                ),
              ),
              if (onSupervise != null)
                Wrap(
                  spacing: 8,
                  children: [
                    TextButton(
                      onPressed: () => showDialog<void>(
                        context: context,
                        builder: (_) => _ChildContext(
                          load: (offset) => onSupervise!(
                            child['childId'] as String,
                            'inspect',
                            offset: offset,
                          ),
                        ),
                      ),
                      child: const Text('Inspect child context'),
                    ),
                    if (child['status'] == 'running')
                      TextButton(
                        onPressed: () async {
                          try {
                            await onSupervise!(
                              child['childId'] as String,
                              'interrupt',
                            );
                          } catch (e) {
                            if (context.mounted) {
                              ScaffoldMessenger.of(context)
                                  .showSnackBar(SnackBar(content: Text('$e')));
                            }
                          }
                        },
                        child: const Text('Interrupt child'),
                      ),
                  ],
                ),
              const Padding(
                padding: EdgeInsets.all(8),
                child: Text(
                  'Ask the parent to follow up with this child by its ID. Follow-ups keep its scope, model and remaining allowance. After the parent ends, inspect saved context and evidence in Run history.',
                  style: TextStyle(fontSize: 11),
                ),
              ),
            ],
          ),
      ],
    );
  }
}

class _ChildContext extends StatefulWidget {
  final Future<Map<String, dynamic>> Function(int offset) load;
  const _ChildContext({required this.load});
  @override
  State<_ChildContext> createState() => _ChildContextState();
}

class _ChildContextState extends State<_ChildContext> {
  int offset = 0;
  final previous = <int>[];
  late Future<Map<String, dynamic>> value = widget.load(offset);
  @override
  Widget build(BuildContext context) => AlertDialog(
    title: const Text('Child context'),
    content: SizedBox(
      width: 620,
      height: 360,
      child: FutureBuilder<Map<String, dynamic>>(
        future: value,
        builder: (context, snapshot) {
          if (snapshot.hasError) {
            return SingleChildScrollView(
              child: SelectableText('${snapshot.error}'),
            );
          }
          if (!snapshot.hasData) {
            return const Center(child: CircularProgressIndicator());
          }
          final data = snapshot.data!;
          return Column(
            crossAxisAlignment: CrossAxisAlignment.start,
            children: [
              Text(
                '${data['model'] ?? 'Child model'} · ${data['scope'] ?? 'Assigned scope'} · ${data['readOnly'] == true ? 'Read only' : 'Scoped file writes'}',
              ),
              if (data['usage'] is Map)
                Text(
                  'Cumulative attempts: ${data['usage']['modelCalls']}/4 model calls · ${data['usage']['toolCalls']}/4 file operations',
                  style: Theme.of(context).textTheme.bodySmall,
                ),
              const SizedBox(height: 8),
              Expanded(
                child: SingleChildScrollView(
                  child: SelectableText('${data['context']}'),
                ),
              ),
              const SizedBox(height: 8),
              const Text(
                'Prepared instructions and report checkpoints. Detailed tool evidence is in Run history.',
                style: TextStyle(fontSize: 11),
              ),
              Wrap(
                spacing: 8,
                children: [
                  TextButton(
                    onPressed: previous.isEmpty
                        ? null
                        : () => setState(() {
                            offset = previous.removeLast();
                            value = widget.load(offset);
                          }),
                    child: const Text('Previous page'),
                  ),
                  TextButton(
                    onPressed: data['nextOffset'] == null
                        ? null
                        : () => setState(() {
                            previous.add(offset);
                            offset = data['nextOffset'] as int;
                            value = widget.load(offset);
                          }),
                    child: const Text('Next page'),
                  ),
                ],
              ),
            ],
          );
        },
      ),
    ),
    actions: [
      TextButton(
        onPressed: () => setState(() {
          offset = 0;
          previous.clear();
          value = widget.load(offset);
        }),
        child: const Text('Refresh'),
      ),
      TextButton(
        onPressed: () => Navigator.of(context).pop(),
        child: const Text('Close'),
      ),
    ],
  );
}
