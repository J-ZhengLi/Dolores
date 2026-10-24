import 'package:flutter/material.dart';

import 'attention_host.dart';
import 'inspector.dart';

Future<void> showAttention(BuildContext context, AttentionHost host) =>
    showDialog<void>(
      context: context,
      barrierDismissible: false,
      builder: (_) => AttentionPanel(host: host),
    );

class AttentionPanel extends StatefulWidget {
  final AttentionHost host;
  const AttentionPanel({super.key, required this.host});
  @override
  State<AttentionPanel> createState() => _AttentionPanelState();
}

class _AttentionPanelState extends State<AttentionPanel> {
  AttentionHost get host => widget.host;
  @override
  void initState() {
    super.initState();
    host.addListener(changed);
    WidgetsBinding.instance.addPostFrameCallback((_) {
      if (mounted) host.refresh();
    });
  }

  void changed() {
    if (mounted) setState(() {});
  }

  @override
  void dispose() {
    host.removeListener(changed);
    super.dispose();
  }

  @override
  Widget build(BuildContext context) => InspectorFrame(
    title: 'Attention',
    subtitle: 'Saved task events · opening never approves or answers',
    canClose: !host.pending,
    child: Padding(
      padding: const EdgeInsets.all(16),
      child: Column(
        children: [
          Expanded(
            child: ListView(
              children: [
                SwitchListTile(
                  contentPadding: EdgeInsets.zero,
                  title: const Text('Windows task notifications'),
                  subtitle: const Text(
                    'Off until enabled. Generic status only; no prompts, answers, titles or file paths.',
                  ),
                  value: host.preferences['enabled'] == true,
                  onChanged: host.pending
                      ? null
                      : (v) => host.save({...host.preferences, 'enabled': v}),
                ),
                if (host.preferences['enabled'] == true)
                  ExpansionTile(
                    title: const Text('Notify about'),
                    children: [
                      for (final kind in [
                        'completed',
                        'failed',
                        'approval',
                        'question',
                      ])
                        CheckboxListTile(
                          title: Text(attentionLabel(kind)),
                          value: host.preferences[kind] == true,
                          onChanged: host.pending
                              ? null
                              : (v) =>
                                    host.save({...host.preferences, kind: v}),
                        ),
                    ],
                  ),
                const Text(
                  'Runs while Dolores is open or in the tray. Windows may suppress display. Delivery accepted does not confirm that a notification was shown or read.',
                ),
                if (!host.supported)
                  const Text(
                    'Native notifications are unavailable here. Every decision remains reachable inside Dolores.',
                  ),
                if (host.omitted > 0)
                  Text(
                    '${host.omitted} older attention entries left out. Their saved run history remains.',
                  ),
                if (host.items.isEmpty && !host.pending)
                  const Text(
                    'No retained task events. Completed, failed, review and question events appear here.',
                  ),
                for (final n in host.items)
                  Card(
                    child: Padding(
                      padding: const EdgeInsets.all(12),
                      child: Column(
                        crossAxisAlignment: CrossAxisAlignment.start,
                        children: [
                          Text(
                            n['title'] as String? ??
                                attentionLabel(n['kind'] as String),
                            style: Theme.of(context).textTheme.titleSmall,
                          ),
                          if (n['title'] != null)
                            Text(attentionLabel(n['kind'] as String)),
                          Text(
                            '${n['seen'] == true ? 'Seen' : 'Unread'} · delivery ${n['delivery']}',
                          ),
                          ExpansionTile(
                            tilePadding: EdgeInsets.zero,
                            title: const Text('Saved event'),
                            children: [
                              SelectableText(
                                'Run ${n['run']} · event ${n['sequence']}',
                              ),
                            ],
                          ),
                          if ([
                            'failed',
                            'uncertain',
                            'suppressed',
                          ].contains(n['delivery']))
                            const Text(
                              'Open this saved entry to inspect the task or its current decision.',
                            ),
                          Wrap(
                            spacing: 8,
                            children: [
                              TextButton(
                                onPressed: host.pending
                                    ? null
                                    : () async {
                                        Navigator.pop(context);
                                        await host.open(n['id'] as String);
                                      },
                                child: const Text('Open task or evidence'),
                              ),
                              if (n['seen'] != true)
                                TextButton(
                                  onPressed: host.pending
                                      ? null
                                      : () => host.seen(n['id'] as String),
                                  child: const Text('Mark seen'),
                                ),
                            ],
                          ),
                        ],
                      ),
                    ),
                  ),
              ],
            ),
          ),
          if (host.error != null)
            ConstrainedBox(
              constraints: const BoxConstraints(maxHeight: 96),
              child: SingleChildScrollView(
                child: SelectableText(
                  host.error!,
                  style: TextStyle(color: Theme.of(context).colorScheme.error),
                ),
              ),
            ),
          if (host.pending) const LinearProgressIndicator(),
          Wrap(
            spacing: 8,
            children: [
              TextButton(
                onPressed: host.pending ? null : () => host.refresh(),
                child: const Text('Refresh attention'),
              ),
              if (host.cursor != null)
                TextButton(
                  onPressed: host.pending
                      ? null
                      : () => host.refresh(next: true),
                  child: const Text('Older attention'),
                ),
              if (host.deferred != null)
                TextButton(
                  onPressed: host.pending
                      ? null
                      : () async {
                          Navigator.pop(context);
                          await host.open(host.deferred!);
                        },
                  child: const Text('Open deferred target'),
                ),
            ],
          ),
        ],
      ),
    ),
  );
}
