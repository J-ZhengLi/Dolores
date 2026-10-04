import 'dart:convert';
import 'dart:typed_data';

import 'package:flutter/material.dart';

import 'bridge.dart';
import 'chat.dart';
import 'inspector.dart';

class BrowserSettingsInspector extends StatefulWidget {
  final ChatController chat;
  const BrowserSettingsInspector({super.key, required this.chat});
  @override
  State<BrowserSettingsInspector> createState() => _BrowserSettingsState();
}

class _BrowserSettingsState extends State<BrowserSettingsInspector> {
  bool pending = false;
  Map? report;
  String? error;
  @override
  void initState() {
    super.initState();
    load();
  }

  Future<void> load() async {
    if (pending) return;
    setState(() {
      pending = true;
      error = null;
    });
    try {
      final value =
          await widget.chat.bridge.call({'command': 'browserSettings'}) as Map;
      if (mounted) setState(() => report = value);
    } catch (e) {
      if (mounted) setState(() => error = '$e');
    } finally {
      if (mounted) setState(() => pending = false);
    }
  }

  @override
  Widget build(BuildContext context) => InspectorFrame(
    title: 'Browser',
    subtitle: 'Optional · starts only when used',
    child: Column(
      children: [
        if (pending) const LinearProgressIndicator(minHeight: 2),
        Expanded(
          child: ListView(
            padding: const EdgeInsets.all(20),
            children: [
              if (error != null) SelectableText(error!),
              if (report != null) ...[
                Text(
                  report!['available'] == true
                      ? 'Browser runtime installed'
                      : 'Browser adapter unavailable',
                  style: Theme.of(context).textTheme.titleMedium,
                ),
                if (report!['reason'] != null)
                  Padding(
                    padding: const EdgeInsets.only(top: 12),
                    child: SelectableText('${report!['reason']}'),
                  ),
                for (final field in ['adapter', 'profile', 'bounds', 'setup'])
                  Padding(
                    padding: const EdgeInsets.only(top: 12),
                    child: SelectableText('${report![field]}'),
                  ),
                if (report!['captureFolder'] != null)
                  Padding(
                    padding: const EdgeInsets.only(top: 12),
                    child: SelectableText(
                      'Local captures: ${report!['captureFolder']}\nAt 128 images, remove older captures here before taking another.',
                    ),
                  ),
              ],
              const SizedBox(height: 16),
              const Text(
                'Project and temporary chats can use one fresh visible browser. Side chats and subagents cannot. Clicks and input need fresh review even under Full access. Approve external actions only when they match your intent.',
              ),
              const SizedBox(height: 12),
              const Text(
                'Only the selected origin loads; cross-origin resources, popups, uploads, downloads and password input are unavailable. Manual login is your explicit action in this new browser; it is not saved between runs. Stop or run end closes the owned process tree and does not undo submitted actions.',
              ),
              const SizedBox(height: 12),
              const Text(
                'Viewport screenshots stay in the local Dolores capture folder and can be viewed from tool evidence. They are not automatically sent as image input to the model. No browser process runs while idle.',
              ),
            ],
          ),
        ),
        Padding(
          padding: const EdgeInsets.all(12),
          child: TextButton(
            onPressed: pending ? null : load,
            child: const Text('Refresh'),
          ),
        ),
      ],
    ),
  );
}

class BrowserCapturePreview extends StatefulWidget {
  final ChatBridge bridge;
  final String capture;
  const BrowserCapturePreview({
    super.key,
    required this.bridge,
    required this.capture,
  });
  @override
  State<BrowserCapturePreview> createState() => _BrowserCaptureState();
}

class _BrowserCaptureState extends State<BrowserCapturePreview> {
  Uint8List? bytes;
  String? error;
  bool pending = false;
  Future<void> load() async {
    setState(() {
      pending = true;
      error = null;
    });
    try {
      final data = await widget.bridge.call({
        'command': 'browserCapture',
        'capture': widget.capture,
      });
      final decoded = base64Decode(data['data'] as String);
      if (decoded.length > 512 * 1024) {
        throw 'Capture exceeds its preview limit.';
      }
      if (mounted) setState(() => bytes = decoded);
    } catch (e) {
      if (mounted) setState(() => error = '$e');
    } finally {
      if (mounted) setState(() => pending = false);
    }
  }

  @override
  Widget build(BuildContext context) => Column(
    crossAxisAlignment: CrossAxisAlignment.start,
    children: [
      if (bytes == null)
        TextButton(
          onPressed: pending ? null : load,
          child: Text(pending ? 'Loading capture…' : 'View local screenshot'),
        ),
      if (error != null)
        SelectableText(
          '$error Retry viewing; this does not repeat the browser action.',
        ),
      if (bytes != null)
        Image.memory(
          bytes!,
          height: 220,
          fit: BoxFit.contain,
          semanticLabel: 'Local browser viewport',
          errorBuilder: (_, _, _) =>
              const Text('Capture could not be displayed.'),
        ),
    ],
  );
}
