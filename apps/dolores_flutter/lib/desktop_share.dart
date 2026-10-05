import 'package:flutter/material.dart';

import 'chat.dart';
import 'inspector.dart';
import 'theme.dart';

Future<void> shareWindow(
  BuildContext context,
  ChatController chat, {
  bool handoff = false,
}) async {
  try {
    await chat.prepareWindowSharing();
    String? goal, run;
    if (handoff) {
      goal = chat.messages.last['metadata']['paused']['task'] as String;
      final runs = await chat.bridge.call({
        'command': 'runs',
        'session': chat.session,
      }) as List;
      if (runs.isEmpty ||
          runs.first['input'] != goal ||
          runs.first['state'] != 'paused') {
        throw StateError(
          'The saved task changed. Return to its latest request before sharing.',
        );
      }
      run = runs.first['id'] as String;
    }
    if (!context.mounted) return;
    await showDialog<void>(
      context: context,
      barrierDismissible: false,
      builder: (_) => WindowSharing(chat: chat, goal: goal, run: run),
    );
  } catch (e) {
    chat.reportLocalError('$e');
  }
}

class WindowSharing extends StatefulWidget {
  final ChatController chat;
  final String? goal, run;
  const WindowSharing({super.key, required this.chat, this.goal, this.run});
  @override
  State<WindowSharing> createState() => _WindowSharingState();
}

class _WindowSharingState extends State<WindowSharing> {
  List<Map> windows = [];
  Map? target, state;
  String? model, error;
  String? sourceRun;
  bool pending = false, control = false;
  int? operation;
  @override
  void initState() {
    super.initState();
    model = widget.chat.model;
    sourceRun = widget.run;
    refresh();
  }

  Future<Map> perform(
    String command, {
    Map<String, dynamic> arguments = const {},
  }) async {
    final id = DateTime.now().microsecondsSinceEpoch;
    operation = id;
    try {
      await widget.chat.bridge.call({
        'command': command,
        'id': id,
        ...arguments,
      });
      final deadline = DateTime.now().add(const Duration(seconds: 18));
      while (mounted && operation == id) {
        final events = await widget.chat.bridge.call({
          'command': 'poll',
          'id': id,
        }) as List;
        for (final event in events) {
          if (event['type'] == 'done') {
            if (event['runId'] is String) sourceRun = event['runId'] as String;
            if (event['error'] != null) throw StateError('${event['error']}');
            return event as Map;
          }
        }
        if (DateTime.now().isAfter(deadline)) {
          await widget.chat.bridge.call({'command': 'cancel', 'id': id});
          throw StateError(
            'Window sharing timed out. Your task remains; try again.',
          );
        }
        await Future<void>.delayed(const Duration(milliseconds: 50));
      }
      throw StateError('Window sharing stopped. Nothing was sent.');
    } finally {
      operation = null;
    }
  }

  Future<void> refresh() async {
    if (pending) return;
    setState(() {
      pending = true;
      error = null;
    });
    try {
      state = await widget.chat.bridge.call({
        'command': 'desktopState',
        'session': widget.chat.session,
      }) as Map;
      if (state!['available'] != true) {
        throw StateError(
          '${state!['reason'] ?? 'Computer use is unavailable on this device.'}',
        );
      }
      final event = await perform(
        'desktopObserve',
        arguments: {'session': widget.chat.session},
      );
      windows = (event['observation']['windows'] as List).cast<Map>();
      target = null;
    } catch (e) {
      error = '$e';
    } finally {
      if (mounted) setState(() => pending = false);
    }
  }

  Future<void> share() async {
    if (pending || target == null || model == null) return;
    final chat = widget.chat;
    if (chat.attachments.isNotEmpty && widget.run == null) {
      setState(
        () => error = 'Your attachments are retained. Send them before sharing a window; this desktop task does not consume an unrelated draft.',
      );
      return;
    }
    setState(() {
      pending = true;
      error = null;
    });
    bool granted = false;
    try {
      if (!(state?['models'] as List? ?? []).contains(model)) {
        await perform(
          'checkImageSupport',
          arguments: {
            'model': model,
            if (widget.run != null) 'session': chat.session,
            if (sourceRun != null) 'runId': sourceRun,
          },
        );
        state = await chat.bridge.call({
          'command': 'desktopState',
          'session': chat.session,
        }) as Map;
      }
      final captured = await perform(
        'desktopObserve',
        arguments: {'session': chat.session, 'target': target},
      );
      final capture = captured['observation'] as Map;
      String? token;
      if (control) {
        final existing = state?['access'] as Map?;
        final sameTarget =
            existing?['enabled'] == true && existing?['target'] is Map &&
            existing!['target']['handle'] == target!['handle'] &&
            existing['target']['pid'] == target!['pid'] &&
            existing['target']['title'] == target!['title'];
        final access = sameTarget
            ? existing
            : await chat.bridge.call({
                'command': 'desktopGrant',
                'session': chat.session,
                'capture': capture['id'],
                'automatic': false,
                'consent': true,
              }) as Map;
        token = access['token'] as String;
        granted = !sameTarget;
      }
      if (!mounted) return;
      // Keep the original request for a handoff, and preserve unrelated edits.
      final input =
          widget.goal ??
          (chat.draft.trim().isEmpty
              ? 'Inspect this shared window and describe what is visible.'
              : chat.draft);
      await chat.send(
        taskInput: widget.goal != null || chat.draft.trim().isEmpty
            ? input
            : null,
        desktopCapture: capture['id'] as String,
        observationModel: model,
        desktopGrant: token,
        desktopResume: sourceRun,
        desktopHandoff: widget.run != null,
        desktopTarget: target,
      );
      if (chat.error != null) throw StateError(chat.error!);
      if (mounted) Navigator.pop(context);
    } catch (e) {
      if (granted) {
        await chat.bridge.call({
          'command': 'desktopRevoke',
          'session': chat.session,
        });
      }
      if (mounted) setState(() => error = '$e');
    } finally {
      if (mounted) setState(() => pending = false);
    }
  }

  @override
  Widget build(BuildContext context) {
    final p = Palette(Theme.of(context).brightness == Brightness.dark);
    return PopScope(
      canPop: !pending,
      child: InspectorFrame(
        title: 'Share a window',
        subtitle:
            'Only the window you choose is shared with your selected model.',
        canClose: !pending,
        child: Column(
          children: [
            if (pending) const LinearProgressIndicator(minHeight: 2),
            if (error != null)
              Padding(
                padding: const EdgeInsets.all(12),
                child: ConstrainedBox(
                  constraints: const BoxConstraints(maxHeight: 100),
                  child: SingleChildScrollView(child: SelectableText(error!)),
                ),
              ),
            Expanded(
              child: ListView(
                padding: const EdgeInsets.all(20),
                children: [
                  if (widget.goal != null)
                    Text(
                      widget.goal!,
                      maxLines: 3,
                      overflow: TextOverflow.ellipsis,
                    ),
                  const SizedBox(height: 12),
                  SegmentedButton<bool>(
                    segments: const [
                      ButtonSegment(value: false, label: Text('View only')),
                      ButtonSegment(
                        value: true,
                        label: Text('Control this window'),
                      ),
                    ],
                    selected: {control},
                    onSelectionChanged: pending
                        ? null
                        : (v) => setState(() => control = v.first),
                  ),
                  const SizedBox(height: 16),
                  if (windows.isEmpty && !pending)
                    const Text(
                      'No windows found. Open the app you want to share, then refresh.',
                    ),
                  for (final window in windows)
                    ListTile(
                      key: ValueKey('share-window-${window['handle']}'),
                      leading: const Icon(Icons.desktop_windows_outlined),
                      title: Text(
                        '${window['title']}',
                        maxLines: 2,
                        overflow: TextOverflow.ellipsis,
                      ),
                      selected: target == window,
                      selectedTileColor: p.soft,
                      trailing: target == window
                          ? const Icon(Icons.check)
                          : null,
                      onTap: pending
                          ? null
                          : () => setState(() => target = window),
                    ),
                  const SizedBox(height: 16),
                  DropdownButtonFormField<String>(
                    key: const Key('share-window-model'),
                    initialValue: model,
                    isExpanded: true,
                    decoration: const InputDecoration(labelText: 'Model'),
                    items: [
                      for (final value in widget.chat.enabledModels)
                        DropdownMenuItem(
                          value: value,
                          child: Text(value, overflow: TextOverflow.ellipsis),
                        ),
                    ],
                    onChanged: pending
                        ? null
                        : (v) => setState(() => model = v),
                  ),
                  const SizedBox(height: 8),
                  Text(
                    (state?['models'] as List? ?? []).contains(model)
                        ? 'Image support enabled for this model.'
                        : 'Image support will be checked with a generated test image first. Your window stays private if the check fails.',
                    style: TextStyle(color: p.muted, fontSize: 12),
                  ),
                  if (control)
                    const Padding(
                      padding: EdgeInsets.only(top: 12),
                      child: Text(
                        'Control applies to this chat and window for 15 minutes. Input is reviewed before it runs. Stop or revoke access at any time.',
                      ),
                    ),
                ],
              ),
            ),
            Padding(
              padding: const EdgeInsets.all(12),
              child: Wrap(
                spacing: 8,
                runSpacing: 8,
                children: [
                  TextButton(
                    onPressed: pending ? null : refresh,
                    child: const Text('Refresh windows'),
                  ),
                  if (pending)
                    TextButton(
                      onPressed: operation == null
                          ? null
                          : () async {
                              await widget.chat.bridge.call({
                                'command': 'cancel',
                                'id': operation,
                              });
                            },
                      child: const Text('Stop'),
                    ),
                  FilledButton(
                    key: const Key('confirm-share-window'),
                    onPressed: pending || target == null ? null : share,
                    child: Text(pending ? 'Preparing…' : 'Share and continue'),
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
