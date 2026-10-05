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
                      ? 'Ready to browse'
                      : 'Needs setup',
                  style: Theme.of(context).textTheme.titleMedium,
                ),
                if (report!['reason'] != null)
                  Padding(
                    padding: const EdgeInsets.only(top: 12),
                    child: SelectableText('${report!['reason']}'),
                  ),
                const SizedBox(height: 12),
                Text(
                  report!['available'] == true
                      ? 'Ask Dolores to open a website in a working chat.'
                      : 'Open Setup below, then check again.',
                ),
                ExpansionTile(
                  title: Text(
                    report!['available'] == true ? 'Browser details' : 'Setup',
                  ),
                  children: [
                    for (final field in [
                      'adapter',
                      'profile',
                      'bounds',
                      'setup',
                    ])
                      Padding(
                        padding: const EdgeInsets.all(8),
                        child: SelectableText('${report![field]}'),
                      ),
                    if (report!['captureFolder'] != null)
                      SelectableText(
                        'Local captures: ${report!['captureFolder']}',
                      ),
                  ],
                ),
              ],
              const Text(
                'Clicks and input need your review. Stop closes the browser; submitted changes may remain.',
              ),
              ExpansionTile(
                title: const Text('Privacy and limits'),
                children: [
                  const Text(
                    'A fresh profile is used for each run. Only the selected origin loads; no saved passwords, file uploads, downloads or popups. Screenshots stay local. Page text is shared with your model. Side chats have no tools.',
                  ),
                ],
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
  final String? desktopSession;
  const BrowserCapturePreview({
    super.key,
    required this.bridge,
    required this.capture,
    this.desktopSession,
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
        'command': widget.desktopSession == null
            ? 'browserCapture'
            : 'desktopPreview',
        'session': ?widget.desktopSession,
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
          semanticLabel: widget.desktopSession == null
              ? 'Local browser viewport'
              : 'Local selected-window screenshot',
          errorBuilder: (_, _, _) =>
              const Text('Capture could not be displayed.'),
        ),
    ],
  );
}
