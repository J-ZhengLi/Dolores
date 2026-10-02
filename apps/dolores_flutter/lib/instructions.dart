import 'package:flutter/material.dart';

import 'bridge.dart';
import 'chat.dart';
import 'inspector.dart';
import 'theme.dart';

Future<void> showInstructions(BuildContext context, ChatController chat) =>
    chat.inspectLocalChanges(() async {
      await showDialog<void>(
        context: context,
        barrierDismissible: false,
        builder: (_) =>
            InstructionsInspector(bridge: chat.bridge, session: chat.session!),
      );
      chat.invalidateContext();
    });

class InstructionsInspector extends StatefulWidget {
  final ChatBridge bridge;
  final String session;
  const InstructionsInspector({
    super.key,
    required this.bridge,
    required this.session,
  });
  @override
  State<InstructionsInspector> createState() => _InstructionsInspectorState();
}

class _InstructionsInspectorState extends State<InstructionsInspector> {
  Map<String, dynamic>? review;
  bool busy = false;
  String? error, notice;

  @override
  void initState() {
    super.initState();
    _refresh();
  }

  Future<dynamic> _call(
    String command, [
    Map<String, dynamic> args = const {},
  ]) => widget.bridge.call({
    'command': command,
    'session': widget.session,
    ...args,
  });

  Future<void> _act(Future<void> Function() action) async {
    if (busy) return;
    setState(() {
      busy = true;
      error = null;
    });
    try {
      await action();
    } catch (failure) {
      if (mounted) setState(() => error = failure.toString());
    } finally {
      if (mounted) setState(() => busy = false);
    }
  }

  Future<void> _load() async {
    if (review != null) {
      setState(() => review = {...review!, 'token': null, 'current': false});
    }
    final result = await _call('reviewInstructions');
    if (mounted) {
      setState(() => review = (result as Map).cast<String, dynamic>());
    }
  }

  Future<void> _refresh() => _act(() async {
    notice = null;
    await _load();
  });
  Future<void> _enable() => _act(() async {
    final token = review?['token'];
    if (token == null) return;
    setState(() => review = {...review!, 'token': null});
    await _call('enableInstructions', {'token': token});
    if (mounted) {
      setState(() => notice = 'Instructions enabled for this working folder.');
    }
    await _load();
  });
  Future<void> _disable() => _act(() async {
    await _call('disableInstructions');
    if (mounted) {
      setState(() {
        review = null;
        notice = 'Instructions disabled. Your saved chats are unchanged.';
      });
    }
    await _load();
  });

  @override
  void dispose() {
    final token = review?['token'];
    if (token != null) {
      _call('cancelInstructionReview', {
        'token': token,
      }).catchError((_) => null);
    }
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    final p = Palette(Theme.of(context).brightness == Brightness.dark);
    final enabled = review?['enabled'] == true;
    final current = review?['current'] == true;
    final provenance = review?['provenance'] as Map?;
    return PopScope(
      canPop: !busy,
      child: InspectorFrame(
        title: 'Workspace instructions',
        subtitle: 'AGENTS.md · applies to chats in this working folder',
        canClose: !busy,
        child: Column(
          children: [
            if (busy) const LinearProgressIndicator(minHeight: 2),
            Expanded(
              child: ListView(
                padding: const EdgeInsets.all(20),
                children: [
                  Text(
                    review == null
                        ? busy
                              ? 'Loading…'
                              : 'Unavailable'
                        : enabled
                        ? current
                              ? 'Enabled'
                              : 'Needs review'
                        : 'Not enabled',
                    style: const TextStyle(fontWeight: FontWeight.w600),
                  ),
                  const SizedBox(height: 8),
                  const Text(
                    'Review this file before enabling it. Its text will be shared with the model and saved locally. Each tool still needs your approval.',
                  ),
                  const SizedBox(height: 8),
                  Text(
                    'Only this folder’s AGENTS.md is loaded. Referenced files are not loaded automatically. Changes require another review.',
                    style: TextStyle(color: p.muted, fontSize: 12),
                  ),
                  if (provenance != null) ...[
                    const SizedBox(height: 12),
                    SelectableText(
                      'Reviewed revision: ${provenance['revision']}\n${provenance['textBytes']} bytes · ${DateTime.fromMillisecondsSinceEpoch(provenance['approvedAt'] as int).toLocal()}',
                      style: TextStyle(color: p.muted, fontSize: 11),
                    ),
                  ],
                  for (final problem in [error, review?['problem'] as String?])
                    if (problem != null)
                      Padding(
                        padding: const EdgeInsets.only(top: 12),
                        child: Text(
                          problem,
                          style: TextStyle(color: p.errorText),
                        ),
                      ),
                  if (notice != null)
                    Padding(
                      padding: const EdgeInsets.only(top: 12),
                      child: Text(notice!, style: TextStyle(color: p.accent)),
                    ),
                  if (review?['text'] is String) ...[
                    const SizedBox(height: 16),
                    Text(
                      'AGENTS.md · current file',
                      style: TextStyle(color: p.muted, fontSize: 12),
                    ),
                    const SizedBox(height: 8),
                    Card(
                      elevation: 0,
                      color: p.surface,
                      child: Padding(
                        padding: const EdgeInsets.all(16),
                        child: SelectableText(
                          review!['text'] as String,
                          key: const Key('instruction-source-text'),
                        ),
                      ),
                    ),
                  ],
                ],
              ),
            ),
            Divider(height: 1, color: p.border),
            Padding(
              padding: const EdgeInsets.all(12),
              child: Wrap(
                spacing: 8,
                runSpacing: 8,
                children: [
                  TextButton(
                    key: const Key('refresh-instructions'),
                    onPressed: busy ? null : _refresh,
                    child: const Text('Refresh'),
                  ),
                  TextButton(
                    key: const Key('disable-instructions'),
                    onPressed: busy || (!enabled && error == null)
                        ? null
                        : _disable,
                    child: const Text('Disable'),
                  ),
                  FilledButton(
                    key: const Key('enable-instructions'),
                    onPressed:
                        busy || review?['token'] == null || (enabled && current)
                        ? null
                        : _enable,
                    child: const Text('Enable instructions'),
                  ),
                ],
              ),
            ),
          ],
        ),
      ),
    );
  }
}
