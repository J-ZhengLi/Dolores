import 'package:flutter/material.dart';

import 'companion_host.dart';

class CompanionNote extends StatelessWidget {
  final CompanionHost host;
  final Future<void> Function(String) open;
  const CompanionNote({super.key, required this.host, required this.open});
  @override
  Widget build(BuildContext context) => AnimatedBuilder(
    animation: host,
    builder: (context, _) {
      final a = host.unread;
      if (a == null) return const SizedBox.shrink();
      return Material(
        color: Theme.of(context).colorScheme.surfaceContainerLow,
        child: Padding(
          padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 8),
          child: Row(
            children: [
              const Icon(Icons.chat_bubble_outline, size: 18),
              const SizedBox(width: 10),
              const Expanded(
                child: Text(
                  'A note from Dolores',
                  maxLines: 1,
                  overflow: TextOverflow.ellipsis,
                ),
              ),
              TextButton(
                onPressed: host.pending
                    ? null
                    : () async {
                        await open(a['session'] as String);
                      },
                child: const Text('Open'),
              ),
              PopupMenuButton<String>(
                tooltip: 'Note options',
                enabled: !host.pending,
                onSelected: (action) =>
                    host.feedback(a['id'] as String, action),
                itemBuilder: (_) => [
                  for (final p in [
                    ('notNow', 'Not now'),
                    ('dismiss', 'Dismiss'),
                    ('fewer', 'Fewer messages'),
                    ('mute', 'Turn off'),
                  ])
                    PopupMenuItem(value: p.$1, child: Text(p.$2)),
                ],
              ),
              if (host.error != null)
                Tooltip(
                  message: host.error!,
                  child: const Icon(Icons.error_outline),
                ),
            ],
          ),
        ),
      );
    },
  );
}
