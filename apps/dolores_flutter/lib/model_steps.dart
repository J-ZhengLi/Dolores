import 'package:flutter/material.dart';

import 'theme.dart';

/// Public assistant text and unsaved partial responses. Never model reasoning.
class ModelSteps extends StatelessWidget {
  final List steps;
  final bool saved;
  const ModelSteps({super.key, required this.steps, this.saved = true});

  @override
  Widget build(BuildContext context) {
    if (steps.isEmpty) return const SizedBox.shrink();
    final p = Palette(Theme.of(context).brightness == Brightness.dark);
    return ExpansionTile(
      key: const PageStorageKey('model-progress-expansion'),
      tilePadding: EdgeInsets.zero,
      childrenPadding: const EdgeInsets.only(bottom: 12),
      title: Text(
        saved ? 'Agent progress' : 'Agent progress · Not saved',
        style: TextStyle(color: p.muted, fontSize: 12),
      ),
      children: [
        for (final step in steps.take(3))
          Padding(
            padding: const EdgeInsets.only(bottom: 12),
            child: Align(
              alignment: Alignment.centerLeft,
              child: Column(
                crossAxisAlignment: CrossAxisAlignment.start,
                children: [
                  Text(
                    step['partial'] == true
                        ? (step['number'] == 0
                              ? 'Partial response'
                              : 'Step ${step['number']} · Partial')
                        : 'Step ${step['number']}',
                    style: TextStyle(color: p.muted, fontSize: 11),
                  ),
                  ConstrainedBox(
                    constraints: const BoxConstraints(maxHeight: 180),
                    child: SingleChildScrollView(
                      key: PageStorageKey(
                        'model-progress-scroll-${step['number']}',
                      ),
                      child: SelectableText(
                        step['text'] as String,
                        style: TextStyle(
                          color: p.text,
                          fontSize: 13,
                          height: 1.6,
                        ),
                      ),
                    ),
                  ),
                ],
              ),
            ),
          ),
      ],
    );
  }
}
