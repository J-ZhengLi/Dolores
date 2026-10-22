import 'package:flutter/material.dart';

import 'chat.dart';
import 'task_permissions.dart';
import 'settings.dart';
import 'theme.dart';

class AccessSelector extends StatefulWidget {
  final ChatController chat;
  const AccessSelector({super.key, required this.chat});
  @override
  State<AccessSelector> createState() => _AccessSelectorState();
}

class _AccessSelectorState extends State<AccessSelector> {
  String? session;
  String mode = 'review';
  int generation = 0, revision = -1;
  bool pending = false;
  bool custom = false;
  @override
  void initState() {
    super.initState();
    load();
  }

  @override
  void didUpdateWidget(AccessSelector old) {
    super.didUpdateWidget(old);
    if (session != widget.chat.session ||
        revision != widget.chat.settingsRevision) {
      load();
    }
  }

  Future<void> load() async {
    final serial = ++generation;
    revision = widget.chat.settingsRevision;
    session = widget.chat.session;
    if (session == null) {
      if (mounted) {
        setState(() {
          mode = 'review';
          custom = false;
        });
      }
      return;
    }
    try {
      final report = await widget.chat.bridge.call({
        'command': 'taskPermissions',
        'session': session,
      }) as Map;
      if (mounted && serial == generation) {
        setState(() {
          mode = report['expired'] == true
              ? 'review'
              : report['policy']['mode'] as String;
          custom = customAccess(report);
        });
      }
    } catch (_) {
      /* Keep the last confirmed mode when a refresh fails. */
    }
  }

  Future<void> select(String choice) async {
    if (pending) return;
    final chat = widget.chat;
    final selectedSession = chat.session;
    setState(() => pending = true);
    try {
      await chat.prepareWindowSharing();
      if (selectedSession != null && chat.session != selectedSession) {
        throw StateError('Chat changed. Choose access again.');
      }
      final target = chat.session;
      if (target == null) throw StateError('Choose a working chat first.');
      final report = await chat.bridge.call({
        'command': 'taskPermissions',
        'session': target,
      }) as Map;
      await saveAccessMode(chat, target, report['revision'] as int, choice);
      await load();
    } catch (failure) {
      if (mounted) {
        ScaffoldMessenger.of(context)
            .showSnackBar(SnackBar(content: Text('$failure')));
      }
    } finally {
      if (mounted) setState(() => pending = false);
    }
  }

  @override
  Widget build(BuildContext context) => PopupMenuButton<String>(
    key: const Key('chat-access'),
    popUpAnimationStyle: surfaceAnimation(context),
    enabled: !pending && !widget.chat.changing && !widget.chat.loading,
    tooltip: 'Chat access',
    constraints: const BoxConstraints(minWidth: 240, maxWidth: 320),
    onSelected: (choice) => choice == 'settings'
        ? showSettings(
            context,
            widget.chat,
            initial: SettingsCategory.permissions,
          )
        : select(choice),
    itemBuilder: (_) => [
      for (final entry in accessModes.entries)
        PopupMenuItem(
          value: entry.key,
          enabled: !widget.chat.busy || entry.key == 'review',
          child: Row(
            children: [
              Icon(!custom && mode == entry.key ? Icons.check : null, size: 16),
              const SizedBox(width: 8),
              Expanded(
                child: Column(
                  crossAxisAlignment: CrossAxisAlignment.start,
                  children: [
                    Text(entry.value),
                    Text(
                      accessHints[entry.key]!,
                      style: Theme.of(context).textTheme.bodySmall,
                    ),
                  ],
                ),
              ),
            ],
          ),
        ),
      const PopupMenuDivider(),
      const PopupMenuItem(value: 'settings', child: Text('Access settings')),
    ],
    child: Padding(
      padding: const EdgeInsets.symmetric(horizontal: 6, vertical: 10),
      child: Row(
        mainAxisSize: MainAxisSize.min,
        children: [
          const Icon(Icons.shield_outlined, size: 14),
          const SizedBox(width: 4),
          Text(
            custom ? 'Custom access' : accessModes[mode] ?? 'Access',
            style: const TextStyle(fontSize: 12),
          ),
          const Icon(Icons.expand_more, size: 14),
        ],
      ),
    ),
  );
}
