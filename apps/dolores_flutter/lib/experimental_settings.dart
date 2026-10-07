import 'package:flutter/material.dart';

import 'chat.dart';
import 'settings_frame.dart';
import 'settings_help.dart';

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
    } catch (_) {
      if (mounted) {
        setState(() => error = "Couldn't load settings. Refresh to retry.");
      }
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
    } catch (_) {
      if (mounted) {
        setState(
          () => error = "Couldn't save. Your setting is unchanged. Try again.",
        );
      }
    } finally {
      if (mounted) setState(() => pending = false);
    }
  }

  @override
  Widget build(BuildContext context) => EmbeddedSettingsFrame(
    title: 'Experimental',
    subtitle: 'Optional features',
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
            title: SettingsHelpLabel(
              label: 'Multiple windows',
              helpKey: const Key('multiple-window-help'),
              help: state!['multipleWindowCapability']?['available'] == true
                  ? 'Open workspace views in separate windows.'
                  : 'Use split views for now. Your choice is saved for when multiple windows become available.',
            ),
            subtitle: Text(
              state!['multipleWindowCapability']?['available'] == true
                  ? 'Open views in separate windows'
                  : 'Not available yet',
            ),
            value: state!['preferences']['multipleWindow'] as bool,
            onChanged: pending ? null : (v) => save('multipleWindow', v),
          ),
          SwitchListTile(
            key: const Key('experimental-keep-awake'),
            title: const SettingsHelpLabel(
              label: 'Keep Windows awake',
              helpKey: Key('keep-awake-help'),
              help: 'Keeps the display and computer awake while Dolores is open. You can still lock Windows manually; device security settings may still lock it.',
            ),
            subtitle: Text(
              state!['windows'] == true
                  ? 'While Dolores is open'
                  : 'Windows only',
            ),
            value: state!['preferences']['preventWindowsFromLocked'] as bool,
            onChanged: pending || state!['windows'] != true
                ? null
                : (v) => save('preventWindowsFromLocked', v),
          ),
          if (state!['preferences']['preventWindowsFromLocked'] == true &&
              state!['keepAwakeActive'] != true)
            Text("Couldn't keep Windows awake. Turn this off and on to retry."),
        ],
        TextButton(
          onPressed: pending ? null : load,
          child: const Text('Refresh'),
        ),
      ],
    ),
  );
}
