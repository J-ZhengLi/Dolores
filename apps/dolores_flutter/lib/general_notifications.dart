import 'package:flutter/material.dart';

import 'app_dropdown.dart';
import 'attention_host.dart';
import 'chat.dart';
import 'help_tip.dart';

class GeneralNotifications extends StatefulWidget {
  final ChatController chat;
  const GeneralNotifications({super.key, required this.chat});
  @override
  State<GeneralNotifications> createState() => _GeneralNotificationsState();
}

class _GeneralNotificationsState extends State<GeneralNotifications> {
  Map<String, dynamic> close = {'revision': 0, 'behavior': 'ask'};
  bool pending = false, loadingClose = true;
  String? error;
  @override
  void initState() {
    super.initState();
    load();
  }

  Future<void> load() async {
    try {
      final value = await widget.chat.bridge.call({
        'command': 'closePreferences',
      });
      if (mounted && value is Map) {
        setState(() => close = Map<String, dynamic>.from(value));
      }
    } catch (_) {
      if (mounted) {
        setState(
          () => error =
              'Could not load the close preference. Close will ask each time.',
        );
      }
    }
    if (mounted) setState(() => loadingClose = false);
    await widget.chat.notifications?.refresh();
  }

  Future<void> saveClose(String value) async {
    if (pending) return;
    setState(() {
      pending = true;
      error = null;
    });
    try {
      final saved = await widget.chat.bridge.call({
        'command': 'saveClosePreferences',
        'preferences': {...close, 'behavior': value},
      }) as Map;
      if (mounted) setState(() => close = Map<String, dynamic>.from(saved));
    } catch (_) {
      if (mounted) {
        setState(
          () => error = 'Could not save the close preference. Your previous choice remains. Try again.',
        );
      }
    } finally {
      if (mounted) setState(() => pending = false);
    }
  }

  Future<void> background(bool value) async {
    if (pending) return;
    setState(() {
      pending = true;
      error = null;
    });
    try {
      final host = widget.chat.backgroundHost!;
      await widget.chat.bridge.call({
        'command': 'setBackgroundPolicy',
        'enabled': value,
        'revision': host.policy['revision'],
      });
      await host.refresh();
      if (mounted) setState(() {});
    } catch (_) {
      if (mounted) {
        setState(
          () => error = 'Could not save background scheduling. Your previous choice remains.',
        );
      }
    } finally {
      if (mounted) setState(() => pending = false);
    }
  }

  @override
  Widget build(BuildContext context) {
    final notices = widget.chat.notifications;
    final availability = widget.chat.backgroundHost;
    return Column(
      crossAxisAlignment: CrossAxisAlignment.stretch,
      children: [
        const Divider(height: 32),
        const Text(
          'When closing Dolores',
          style: TextStyle(fontWeight: FontWeight.w600),
        ),
        const SizedBox(height: 12),
        AppDropdown<String>(
          key: ValueKey('close-behavior-${close['behavior']}'),
          initialValue: close['behavior'] as String? ?? 'ask',
          items: const [
            DropdownMenuItem(value: 'ask', child: Text('Ask every time')),
            DropdownMenuItem(value: 'quit', child: Text('Close the app')),
            DropdownMenuItem(
              value: 'tray',
              child: Text('Minimize to system tray'),
            ),
          ],
          onChanged: pending || loadingClose ? null : (v) => saveClose(v!),
        ),
        if (availability != null)
          AnimatedBuilder(
            animation: availability,
            builder: (context, _) => SwitchListTile(
              key: const Key('general-background'),
              contentPadding: EdgeInsets.zero,
              title: const Text('Continue scheduled tasks in tray'),
              subtitle: const Text(
                'Dolores must be open and your computer awake.',
              ),
              value: availability.policy['enabled'] == true,
              onChanged: pending || !availability.supported ? null : background,
            ),
          ),
        if (notices != null) ...[
          const Divider(height: 32),
          NotificationPreferences(host: notices),
        ],
        if (error != null)
          Padding(
            padding: const EdgeInsets.only(top: 12),
            child: Text(
              error!,
              style: TextStyle(color: Theme.of(context).colorScheme.error),
            ),
          ),
      ],
    );
  }
}

class NotificationPreferences extends StatelessWidget {
  final AttentionHost host;
  const NotificationPreferences({super.key, required this.host});
  @override
  Widget build(BuildContext context) => AnimatedBuilder(
    animation: host,
    builder: (context, _) => Column(
      children: [
        SwitchListTile(
          contentPadding: EdgeInsets.zero,
          key: const Key('windows-notifications'),
          title: const HelpLabel(
            help: 'Windows receives generic task status without chat content or file names. Dolores must be open or in the tray. Windows can suppress notifications.',
            child: Text('Windows notifications'),
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
                      : (v) => host.save({...host.preferences, kind: v}),
                ),
            ],
          ),
        if (!host.supported)
          const Align(
            alignment: Alignment.centerLeft,
            child: Text(
              'Windows delivery is unavailable. Notifications still appear in Dolores.',
            ),
          ),
        if (host.error != null)
          Row(
            children: [
              const Expanded(
                child: Text('Could not load notification settings. Try again.'),
              ),
              TextButton(
                onPressed: host.pending ? null : () => host.refresh(),
                child: const Text('Retry'),
              ),
            ],
          ),
      ],
    ),
  );
}
