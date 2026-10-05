import 'package:flutter/material.dart';

import 'chat.dart';
import 'task_permissions.dart';

class AccessSelector extends StatefulWidget {
  final ChatController chat;
  const AccessSelector({super.key, required this.chat});
  @override
  State<AccessSelector> createState() => _AccessSelectorState();
}

class _AccessSelectorState extends State<AccessSelector> {
  String? session;
  String label = 'Access';
  int generation = 0;
  @override
  void initState() {
    super.initState();
    load();
  }

  @override
  void didUpdateWidget(AccessSelector old) {
    super.didUpdateWidget(old);
    if (session != widget.chat.session) load();
  }

  Future<void> load() async {
    final serial = ++generation;
    session = widget.chat.session;
    if (session == null) {
      if (mounted) setState(() => label = 'Review');
      return;
    }
    try {
      final report = await widget.chat.bridge.call({
        'command': 'taskPermissions',
        'session': session,
      }) as Map;
      if (!mounted || serial != generation) return;
      final mode = report['expired'] == true
          ? 'review'
          : report['policy']['mode'];
      setState(
        () => label = switch (mode) {
          'review' => 'Review',
          'auto' => 'Custom',
          'fullAccess' => 'Full access',
          _ => 'Access',
        },
      );
    } catch (_) {
      if (mounted && serial == generation) setState(() => label = 'Access');
    }
  }

  @override
  Widget build(BuildContext context) => TextButton.icon(
    key: const Key('chat-access'),
    onPressed: widget.chat.busy || widget.chat.changing || widget.chat.loading
        ? null
        : () async {
            try {
              await widget.chat.prepareWindowSharing();
              if (!context.mounted) return;
              await showTaskPermissions(context, widget.chat);
              await load();
            } catch (e) {
              if (context.mounted) {
                ScaffoldMessenger.of(context)
                    .showSnackBar(SnackBar(content: Text('$e')));
              }
            }
          },
    style: TextButton.styleFrom(
      padding: const EdgeInsets.symmetric(horizontal: 6),
      minimumSize: const Size(0, 36),
    ),
    icon: const Icon(Icons.shield_outlined, size: 14),
    label: Text(label, style: const TextStyle(fontSize: 12)),
  );
}
