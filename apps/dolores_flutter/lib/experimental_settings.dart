import 'package:flutter/material.dart';

import 'chat.dart';
import 'settings_frame.dart';

class ExperimentalSettings extends StatefulWidget {
  final ChatController chat;
  const ExperimentalSettings({super.key, required this.chat});
  @override
  State<ExperimentalSettings> createState() => _ExperimentalSettingsState();
}

class _ExperimentalSettingsState extends State<ExperimentalSettings> {
  Map<String, dynamic>? state;
  bool pending = false;
  String? error;
  @override
  void initState() {
    super.initState();
    load();
  }

  Future<void> load() async {
    setState(() => pending = true);
    try {
      final next = await widget.chat.bridge.call({
        'command': 'experimentalPreferences',
      });
      if (mounted) {
        setState(() {
          state = Map<String, dynamic>.from(next as Map);
          error = null;
        });
      }
    } catch (e) {
      if (mounted) setState(() => error = '$e');
    } finally {
      if (mounted) setState(() => pending = false);
    }
  }

  Future<void> save(String key, bool value) async {
    setState(() => pending = true);
    try {
      final next = await widget.chat.bridge.call({
        'command': 'saveExperimentalPreferences',
        'preferences': {
          ...Map<String, dynamic>.from(state!['preferences'] as Map),
          key: value,
        },
      });
      if (mounted) {
        setState(() {
          state = Map<String, dynamic>.from(next as Map);
          error = null;
        });
      }
    } catch (e) {
      if (mounted) {
        setState(() => error = '$e Your previous preference is retained.');
      }
    } finally {
      if (mounted) setState(() => pending = false);
    }
  }

  @override
  Widget build(BuildContext context) => EmbeddedSettingsFrame(
    title: 'Experimental',
    subtitle: 'Optional features that are still being qualified',
    pending: SettingsEmbedding.of(context)!.pending,
    canClose: !pending,
    child: ListView(
      padding: const EdgeInsets.all(20),
      children: [
        if (error != null) Text(error!),
        if (pending) const LinearProgressIndicator(),
        if (state != null) ...[
          SwitchListTile(
            key: const Key('experimental-multiple-window'),
            title: const Text('Multiple Window'),
            subtitle: const Text(
              'On by default. Detached developer views become available in milestone 19.',
            ),
            value: state!['preferences']['multipleWindow'] as bool,
            onChanged: pending ? null : (v) => save('multipleWindow', v),
          ),
          SwitchListTile(
            key: const Key('experimental-keep-awake'),
            title: const Text('Prevent Windows From Locked'),
            subtitle: Text(
              state!['windows'] == true
                  ? 'Keep the display and system awake while Dolores is open. Manual locks, security policy and Modern Standby limits still apply.'
                  : 'Available on Windows only.',
            ),
            value: state!['preferences']['preventWindowsFromLocked'] as bool,
            onChanged: pending || state!['windows'] != true
                ? null
                : (v) => save('preventWindowsFromLocked', v),
          ),
          if (state!['preferences']['preventWindowsFromLocked'] == true)
            Text(
              state!['keepAwakeActive'] == true
                  ? 'Windows keep-awake request is active.'
                  : 'The saved preference could not be activated. Turn it off and on to retry.',
            ),
        ],
        TextButton(
          onPressed: pending ? null : load,
          child: const Text('Refresh'),
        ),
      ],
    ),
  );
}
