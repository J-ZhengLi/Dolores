import 'dart:async';

import 'package:flutter/material.dart';
import 'package:window_manager/window_manager.dart';

import 'chat.dart';
import 'inspector.dart';
import 'theme.dart';

class NativeRepairs extends StatefulWidget {
  final ChatController chat;
  final bool Function()? hasUnsavedSettings;
  const NativeRepairs({super.key, required this.chat, this.hasUnsavedSettings});
  @override
  State<NativeRepairs> createState() => _NativeRepairsState();
}

class _NativeRepairsState extends State<NativeRepairs> {
  final scroll = ScrollController();
  Map<String, dynamic>? state, review;
  bool busy = false;
  String? error;
  @override
  void initState() {
    super.initState();
    refresh();
  }

  Future<dynamic> call(
    String command, [
    Map<String, dynamic> args = const {},
  ]) => widget.chat.bridge.call({'command': command, ...args});
  Future<void> act(Future<void> Function() action) async {
    if (busy) return;
    setState(() {
      busy = true;
      error = null;
    });
    try {
      await action();
    } catch (failure) {
      if (mounted) {
        setState(() => error = '$failure');
        if (scroll.hasClients) scroll.jumpTo(0);
      }
    } finally {
      if (mounted) setState(() => busy = false);
    }
  }

  Future<void> discard() async {
    final token = review?['token'];
    review = null;
    if (token != null) await call('discardNativeUpdate', {'token': token});
  }

  Future<void> refresh() => act(() async {
    await discard();
    final result = await call('nativeRepairs');
    if (mounted) {
      setState(() => state = (result as Map).cast<String, dynamic>());
    }
  });
  Future<void> prepare(Map build, bool restore) => act(() async {
    await discard();
    if (widget.chat.busy || widget.chat.loading) {
      throw StateError('Finish or stop the current task first.');
    }
    final result = await call('reviewNativeUpdate', {
      'session': build['session'],
      'buildId': build['id'],
      'restore': restore,
    });
    if (mounted) {
      setState(() => review = (result as Map).cast<String, dynamic>());
    }
  });
  Future<void> apply() => act(() async {
    final token = review?['token'];
    if (token == null) return;
    if (widget.hasUnsavedSettings?.call() == true) {
      throw StateError(
        'Other Settings pages have unsaved edits. Save or discard them before reviewing restart; this review and your drafts remain.',
      );
    }
    // Save every retained chat draft before consuming the restart review.
    await widget.chat.saveNativeDrafts();
    setState(() => review = null);
    await call('applyNativeUpdate', {'token': token});
    await call('shutdown');
    try {
      await windowManager.destroy();
    } catch (failure) {
      throw StateError(
        'Restart is waiting for this app to close. Drafts were saved. Close the app within 30 seconds; if it stays open, the launcher retains the current version. $failure',
      );
    }
  });
  @override
  void dispose() {
    unawaited(discard().catchError((_) {}));
    scroll.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    const stages = {
      'waiting': 'Waiting for app to close',
      'previousStopped': 'App closed',
      'snapshotSaved': 'History saved',
      'starting': 'Starting app',
      'rollingBack': 'Recovering previous version',
      'applied': 'Installed',
      'restored': 'Previous version restored',
      'rolledBack': 'Recovered previous version',
      'failed': 'Restart did not complete',
      'cancelled': 'Cancelled',
      'recoveryRequired': 'Recovery needs attention',
    };
    const buildStates = {
      'started': 'Building',
      'ready': 'Ready to review',
      'failed': 'Build failed',
      'stopped': 'Build stopped',
    };
    final p = Palette(Theme.of(context).brightness == Brightness.dark);
    final builds = (state?['builds'] as List? ?? []).whereType<Map>();
    final intent = state?['intent'] as Map?;
    return PopScope(
      canPop: !busy,
      child: InspectorFrame(
        title: 'Native repairs',
        subtitle: 'Reviewed Rust builds, installation and Restore',
        canClose: !busy,
        child: Column(
          children: [
            Expanded(
              child: ListView(
                controller: scroll,
                padding: const EdgeInsets.all(16),
                children: [
                  if (busy) const LinearProgressIndicator(),
                  if (error != null)
                    Padding(
                      padding: const EdgeInsets.only(bottom: 12),
                      child: SelectableText(
                        error!,
                        style: TextStyle(color: p.text),
                      ),
                    ),
                  if (intent != null) ...[
                    Text(
                      stages[intent['stage']] ?? 'Inspect restart evidence',
                      style: const TextStyle(fontWeight: FontWeight.w600),
                    ),
                    SelectableText('${intent['note']}'),
                    if (intent['interrupted'] == true)
                      SelectableText('${intent['recovery']}'),
                    const SizedBox(height: 16),
                  ],
                  if (state == null && !busy)
                    const Text(
                      'Native repairs could not be loaded. Refresh inspects saved state without repeating an operation.',
                    ),
                  if (state != null && builds.isEmpty)
                    const Text(
                      'No native builds retained. Ask Dolores to inspect, propose and test a repair first. A qualified trial can then receive a separately reviewed release build.',
                    ),
                  for (final b in builds)
                    Card(
                      child: Padding(
                        padding: const EdgeInsets.all(12),
                        child: Column(
                          crossAxisAlignment: CrossAxisAlignment.start,
                          children: [
                            Text(
                              '${buildStates[b['status']] ?? 'Inspect build'} · Revision ${b['revision']}',
                              style: const TextStyle(
                                fontWeight: FontWeight.w600,
                              ),
                            ),
                            SelectableText('${b['note']}'),
                            ExpansionTile(
                              title: const Text('Details'),
                              children: [
                                SelectableText(
                                  'Build: ${b['id']}\nRepair: ${b['repairId']}\nTrial: ${b['evaluationId']}\nSource: ${b['sourceId']}\nCandidate: ${b['candidateSourceId']}\nCargo: ${b['cargoId']}\nPrevious bundle: ${b['previousId']}\nCandidate bundle: ${b['candidateId']}',
                                ),
                              ],
                            ),
                            if (b['status'] == 'ready')
                              Wrap(
                                spacing: 8,
                                runSpacing: 6,
                                children: [
                                  TextButton(
                                    onPressed: busy
                                        ? null
                                        : () => prepare(b, false),
                                    child: const Text('Review installation'),
                                  ),
                                  TextButton(
                                    onPressed: busy
                                        ? null
                                        : () => prepare(b, true),
                                    child: const Text('Review Restore'),
                                  ),
                                ],
                              ),
                          ],
                        ),
                      ),
                    ),
                  if (review != null) ...[
                    const SizedBox(height: 12),
                    Text(
                      review!['operation'] == 'restore'
                          ? 'Review Restore'
                          : 'Review installation',
                      style: const TextStyle(fontWeight: FontWeight.w600),
                    ),
                    SelectableText('${review!['note']}'),
                    SelectableText(
                      'Current bundle: ${review!['currentBundle']}\nTarget bundle: ${review!['targetBundle']}\nProfile: ${review!['profile']}\nCurrent app: ${review!['currentExecutable']}\nClose: ${review!['shutdownSeconds']} seconds · Startup: ${review!['startupSeconds']} seconds',
                    ),
                  ],
                  const SizedBox(height: 12),
                  Text(
                    '${state?['note'] ?? 'Native code runs with this account’s permissions. Every build and restart needs separate review.'}',
                    style: TextStyle(color: p.muted),
                  ),
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
                    onPressed: busy ? null : refresh,
                    child: const Text('Refresh'),
                  ),
                  if (review != null)
                    TextButton(
                      onPressed: busy
                          ? null
                          : () => act(() async {
                              await discard();
                            }),
                      child: const Text('Cancel review'),
                    ),
                  if (review != null)
                    FilledButton(
                      onPressed: busy ? null : apply,
                      child: Text(
                        review!['operation'] == 'restore'
                            ? 'Restore & restart'
                            : 'Install & restart',
                      ),
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
