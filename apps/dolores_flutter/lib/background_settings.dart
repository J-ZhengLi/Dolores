import 'package:flutter/material.dart';

import 'background_host.dart';
import 'chat.dart';
import 'settings_frame.dart';
import 'settings_help.dart';

class BackgroundSettings extends StatefulWidget {
  final ChatController chat;
  const BackgroundSettings({super.key, required this.chat});
  @override
  State<BackgroundSettings> createState() => _BackgroundSettingsState();
}

class _BackgroundSettingsState extends State<BackgroundSettings> {
  Map? policy;
  bool pending = false, supported = false;
  String? error;
  @override
  void initState() {
    super.initState();
    load();
  }

  Future<void> load() async {
    setState(() => pending = true);
    try {
      final p =
          await widget.chat.bridge.call({'command': 'backgroundPolicy'}) as Map;
      final capable = await NativeAvailabilityWindow().available();
      if (mounted) {
        setState(() {
          policy = p;
          supported = capable;
          error = null;
        });
      }
    } catch (e) {
      if (mounted) setState(() => error = '$e');
    } finally {
      if (mounted) setState(() => pending = false);
    }
  }

  Future<void> save(bool value) async {
    setState(() => pending = true);
    try {
      final p = await widget.chat.bridge.call({
        'command': 'setBackgroundPolicy',
        'revision': policy!['revision'],
        'enabled': value,
      }) as Map;
      if (mounted) {
        setState(() {
          policy = p;
          error = null;
        });
      }
      widget.chat.invalidateContext();
    } catch (e) {
      if (mounted) setState(() => error = '$e');
    } finally {
      if (mounted) setState(() => pending = false);
    }
  }

  @override
  Widget build(BuildContext context) => EmbeddedSettingsFrame(
    title: 'Background tasks',
    pending: SettingsEmbedding.of(context)!.pending,
    canClose: !pending,
    child: ListView(
      padding: const EdgeInsets.all(20),
      children: [
        if (pending) const LinearProgressIndicator(),
        if (error != null) Text(error!),
        if (policy != null)
          SwitchListTile(
            key: const Key('background-enabled'),
            contentPadding: EdgeInsets.zero,
            title: const SettingsHelpLabel(
              label: 'Run tasks in background',
              help: 'Close Dolores to the tray to keep scheduled tasks running. Open it from the tray or launch it again to see results. Quit stops tasks. Your computer must be awake and connected; Dolores does not start with Windows or wake it.',
            ),
            subtitle: Text(
              supported
                  ? 'Close to tray; Quit to stop'
                  : 'Windows only for now',
            ),
            value: policy!['enabled'] as bool,
            onChanged: pending || !supported ? null : save,
          ),
        TextButton(
          onPressed: pending ? null : load,
          child: const Text('Refresh'),
        ),
      ],
    ),
  );
}
