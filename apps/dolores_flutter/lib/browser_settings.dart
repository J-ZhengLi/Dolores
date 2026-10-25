import 'dart:async';

import 'help_tip.dart';

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
  Map setup = {};
  Timer? timer;
  bool get installing => setup['phase'] == 'installing';
  @override
  void dispose() {
    timer?.cancel();
    super.dispose();
  }

  Future<void> setupAction(String action) async {
    if (pending) return;
    setState(() {
      pending = true;
      error = null;
    });
    try {
      final result = await widget.chat.bridge.call({
        'command': 'browserSetup',
        'action': action,
      }) as Map;
      if (!mounted) return;
      setState(() => setup = result);
      timer?.cancel();
      if (installing) {
        timer = Timer(
          const Duration(milliseconds: 600),
          () => setupAction('status'),
        );
      } else if (action == 'status') {
        pending = false;
        await load();
      }
    } catch (e) {
      if (mounted) setState(() => error = '$e');
    } finally {
      if (mounted) setState(() => pending = false);
    }
  }

  Future<void> prerequisite(String kind) async {
    try {
      await widget.chat.bridge.call({
        'command': 'browserPrerequisite',
        'kind': kind,
      });
    } catch (e) {
      if (mounted) setState(() => error = '$e');
    }
  }

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
      if (mounted) {
        setState(() {
          report = value;
          setup = value['setupState'] as Map? ?? {};
        });
        if (installing) {
          timer?.cancel();
          timer = Timer(
            const Duration(milliseconds: 600),
            () => setupAction('status'),
          );
        }
      }
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
              const Text(
                'Let Dolores open websites and interact with pages for a task.',
              ),
              const SizedBox(height: 20),
              if (report != null) ...[
                Text(
                  installing
                      ? 'Setting up browser…'
                      : report!['available'] == true
                      ? 'Ready to browse'
                      : 'Set up Browser',
                  style: Theme.of(context).textTheme.titleMedium,
                ),
                const SizedBox(height: 12),
                if (installing) ...[
                  const LinearProgressIndicator(),
                  const SizedBox(height: 8),
                  const Text(
                    'Installing the browser adapter. This can take up to 90 seconds.',
                  ),
                  TextButton(
                    onPressed: pending ? null : () => setupAction('cancel'),
                    child: const Text('Cancel setup'),
                  ),
                ] else if (report!['available'] == true)
                  const Text('Ask Dolores to open a website in a working chat.')
                else ...[
                  if ((report!['prerequisites'] as Map?)?['nodeReady'] != true)
                    ListTile(
                      contentPadding: EdgeInsets.zero,
                      title: const Text('Install Node.js'),
                      subtitle: Text(
                        '${(report!['prerequisites'] as Map?)?['nodeReason'] ?? 'Node.js with npm is required.'}',
                      ),
                      trailing: TextButton(
                        onPressed: () => prerequisite('node'),
                        child: const Text('Download'),
                      ),
                    ),
                  if ((report!['prerequisites'] as Map?)?['browserReady'] !=
                      true)
                    ListTile(
                      contentPadding: EdgeInsets.zero,
                      title: const Text('Install Microsoft Edge'),
                      subtitle: const Text(
                        'Dolores uses a separate browser profile for each task.',
                      ),
                      trailing: TextButton(
                        onPressed: () => prerequisite('browser'),
                        child: const Text('Download'),
                      ),
                    ),
                  if ((report!['prerequisites'] as Map?)?['nodeReady'] ==
                          true &&
                      (report!['prerequisites'] as Map?)?['browserReady'] ==
                          true)
                    Align(
                      alignment: Alignment.centerLeft,
                      child: FilledButton(
                        onPressed: pending ? null : () => setupAction('start'),
                        child: const Text('Set up browser'),
                      ),
                    ),
                ],
                if ('${setup['note'] ?? ''}'.isNotEmpty &&
                    !installing &&
                    report!['available'] != true)
                  Padding(
                    padding: const EdgeInsets.only(top: 12),
                    child: SelectableText('${setup['note']}'),
                  ),
                const SizedBox(height: 20),
                const HelpLabel(
                  help: 'Page text is shared with your model. Each task uses a fresh browser without your saved passwords or logins. Stop closes it; changes already submitted to a website may remain.',
                  child: Text('Browser actions stay under your control.'),
                ),
                ExpansionTile(
                  title: const Text('Details'),
                  children: [
                    for (final field in ['adapter', 'profile', 'bounds'])
                      Padding(
                        padding: const EdgeInsets.all(8),
                        child: SelectableText('${report![field]}'),
                      ),
                    SelectableText(
                      'Setup folder: ${report!['setupFolder'] ?? ''}',
                    ),
                    SelectableText(
                      'Local captures: ${report!['captureFolder'] ?? ''}',
                    ),
                    if (report!['reason'] != null)
                      SelectableText('${report!['reason']}'),
                  ],
                ),
              ],
            ],
          ),
        ),
        Padding(
          padding: const EdgeInsets.all(12),
          child: TextButton(
            onPressed: pending || installing ? null : load,
            child: const Text('Check again'),
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
          key: PageStorageKey(
            'capture-error-${widget.desktopSession ?? 'browser'}-${widget.capture}',
          ),
          '$error Retry viewing; this does not repeat the action.',
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
