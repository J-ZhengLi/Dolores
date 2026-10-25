import 'package:flutter/material.dart';
import 'package:window_manager/window_manager.dart';

import 'app_host.dart';

class CloseChoice {
  final String behavior;
  final bool remember;
  const CloseChoice(this.behavior, this.remember);
}

Future<CloseChoice?> chooseCloseBehavior(
  BuildContext context, {
  bool trayAvailable = true,
}) => showDialog<CloseChoice>(
  context: context,
  builder: (context) {
    bool remember = false;
    return StatefulBuilder(
      builder: (context, setState) => AlertDialog(
        title: const Text('Close Dolores?'),
        content: SizedBox(
          width: 360,
          child: Column(
            mainAxisSize: MainAxisSize.min,
            crossAxisAlignment: CrossAxisAlignment.start,
            children: [
              const Text(
                'Close stops running tasks. Minimize keeps your work running in the tray.',
              ),
              const SizedBox(height: 12),
              CheckboxListTile(
                key: const Key('remember-close-choice'),
                contentPadding: EdgeInsets.zero,
                title: const Text('Remember my choice'),
                value: remember,
                onChanged: (v) => setState(() => remember = v!),
              ),
            ],
          ),
        ),
        actions: [
          TextButton(
            onPressed: () => Navigator.pop(context),
            child: const Text('Cancel'),
          ),
          TextButton(
            key: const Key('close-choice-quit'),
            onPressed: () =>
                Navigator.pop(context, CloseChoice('quit', remember)),
            child: const Text('Close the app'),
          ),
          FilledButton(
            key: const Key('close-choice-tray'),
            onPressed: trayAvailable
                ? () => Navigator.pop(context, CloseChoice('tray', remember))
                : null,
            child: const Text('Minimize to system tray'),
          ),
        ],
      ),
    );
  },
);

Future<void> closeDolores(BuildContext context, AppHost host) async {
  Map<String, dynamic> preferences = {'revision': 0, 'behavior': 'ask'};
  try {
    preferences = Map<String, dynamic>.from(
      await host.initial.bridge
              .call({'command': 'closePreferences'})
              .timeout(const Duration(milliseconds: 400))
          as Map,
    );
  } catch (_) {
    /* A lost setting falls back to an explicit choice. */
  }
  if (!context.mounted) return;
  CloseChoice? choice;
  if (preferences['behavior'] == 'ask') {
    choice = await chooseCloseBehavior(
      context,
      trayAvailable: host.background.supported,
    );
  } else {
    choice = CloseChoice(preferences['behavior'] as String, false);
  }
  if (choice == null) return;
  if (choice.remember) {
    try {
      await host.initial.bridge
          .call({
            'command': 'saveClosePreferences',
            'preferences': {...preferences, 'behavior': choice.behavior},
          })
          .timeout(const Duration(milliseconds: 400));
    } catch (_) {
      host.initial.notice(
        'close-preference',
        'Close choice was not saved',
        'You can change the choice in Settings > General.',
        severity: 'warning',
      );
    }
  }
  if (choice.behavior == 'tray') {
    if (!await host.minimizeToTray()) {
      host.initial.notice(
        'tray',
        'Could not minimize to tray',
        'Dolores is still open. Try again or choose Close the app.',
        severity: 'error',
      );
      if (!context.mounted) return;
      final quit = await showDialog<bool>(
        context: context,
        builder: (c) => AlertDialog(
          title: const Text('Could not minimize to tray'),
          content: const Text(
            'Dolores is still open and your work is available. You can try again or close the app.',
          ),
          actions: [
            TextButton(
              onPressed: () => Navigator.pop(c, false),
              child: const Text('Keep open'),
            ),
            TextButton(
              onPressed: () => Navigator.pop(c, true),
              child: const Text('Close the app'),
            ),
          ],
        ),
      );
      if (quit != true) return;
    } else {
      return;
    }
  }
  try {
    await windowManager.hide().timeout(const Duration(milliseconds: 250));
  } catch (_) {
    /* Quit remains available. */
  }
  try {
    await host
        .prepareQuit(saveFiles: false)
        .timeout(const Duration(seconds: 2));
  } catch (_) {
    /* Recovery cannot trap Quit. */
  }
  await windowManager.destroy();
}
