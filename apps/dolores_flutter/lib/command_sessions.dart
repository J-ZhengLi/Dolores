import 'form_field_layout.dart';

import 'dart:async';
import 'dart:convert';

import 'package:flutter/material.dart';

import 'chat.dart';
import 'inspector.dart';
import 'settings_frame.dart';
import 'theme.dart';

Future<void> showCommandSessions(
  BuildContext context,
  ChatController chat, {
  String? run,
}) => showDialog<void>(
  context: context,
  barrierDismissible: false,
  builder: (_) => CommandSessionsInspector(chat: chat, run: run),
);

/// Inspection never starts or reattaches a process. Effectful buttons are single
/// explicit requests; polling only reads receipts and output.
class CommandSessionsInspector extends StatefulWidget {
  final ChatController chat;
  final String? run;
  const CommandSessionsInspector({super.key, required this.chat, this.run});
  @override
  State<CommandSessionsInspector> createState() =>
      _CommandSessionsInspectorState();
}

class _CommandSessionsInspectorState extends State<CommandSessionsInspector> {
  late final String? session = widget.chat.session;
  late String? viewedRun = widget.run;
  final input = TextEditingController();
  final rows = <String, Map<String, dynamic>>{};
  final live = <String>{};
  List<dynamic> warnings = [], retained = [];
  String? selected, error, historyError;
  Map<String, dynamic>? receipt;
  Map<String, dynamic> isolation = {};
  String output = '';
  final outputPages = <int, String>{};
  int readGeneration = 0;
  int nextOffset = 0;
  dynamic cursor;
  bool loading = false, pending = false, hasMore = false;
  Timer? timer;

  String key(Map row) => '${row['run']}/${row['id']}';
  String get draftKey => '$session/$selected';
  bool get running =>
      ['starting', 'running', 'stopping'].contains(receipt?['state']);
  bool get attached => selected != null && live.contains(selected);
  bool get canInput =>
      attached &&
      receipt?['state'] == 'running' &&
      receipt?['stdinClosed'] != true &&
      receipt?['stdinClosing'] != true &&
      receipt?['lastInput']?['state'] != 'pending';

  @override
  void initState() {
    super.initState();
    input.addListener(() {
      if (selected != null) {
        widget.chat.commandInputDrafts[draftKey] = input.text;
      }
    });
    unawaited(refresh());
    timer = Timer.periodic(const Duration(seconds: 1), (_) {
      if (!pending && !loading && (live.isNotEmpty || retained.isNotEmpty)) {
        unawaited(refresh(automatic: true));
      }
    });
  }

  @override
  void dispose() {
    timer?.cancel();
    input.dispose();
    super.dispose();
  }

  Future<Map<String, dynamic>> call(Map<String, dynamic> request) async =>
      (await widget.chat.bridge.call({
        'command': 'commandSession',
        'session': session,
        'request': request,
      }) as Map).cast<String, dynamic>();

  Future<void> refresh({bool older = false, bool automatic = false}) async {
    if (loading || pending || session == null) return;
    setState(() {
      loading = true;
      if (!automatic) error = null;
    });
    try {
      final page = await call({
        'operation': 'list',
        'run': viewedRun,
        'cursor': older ? cursor : null,
      });
      if (!mounted) return;
      if (!older) {
        live.clear();
        warnings = page['storageWarnings'] as List? ?? [];
        retained = page['retainedRuns'] as List? ?? [];
        historyError = page['historyError'] as String?;
        isolation = (page['isolation'] as Map?)?.cast<String, dynamic>() ?? {};
      }
      for (final row in [
        ...(page['items'] as List? ?? []),
        ...(page['liveItems'] as List? ?? []),
      ]) {
        final item = (row as Map).cast<String, dynamic>();
        rows[key(item)] = item;
      }
      for (final row in page['liveItems'] as List? ?? []) {
        live.add(key(row as Map));
      }
      // Refreshing the head must not destroy the saved history pagination cursor.
      if (older || cursor == null) cursor = page['nextCursor'];
      if (selected == null && rows.isNotEmpty) {
        select(rows.keys.first);
      } else if (selected != null) {
        await read();
      }
    } catch (failure) {
      if (mounted) error = '$failure';
    } finally {
      if (mounted) setState(() => loading = false);
    }
  }

  void select(String value) {
    selected = value;
    receipt = rows[value];
    input.text = widget.chat.commandInputDrafts[draftKey] ?? '';
    output = '';
    outputPages.clear();
    nextOffset = 0;
    hasMore = false;
    unawaited(read());
  }

  Future<void> read({bool more = false}) async {
    final target = selected, owner = receipt;
    if (target == null || owner == null) return;
    final offset = more ? nextOffset : outputPages.keys.lastOrNull ?? 0;
    final generation = ++readGeneration;
    try {
      final page = await call({
        'operation': 'read',
        'run': owner['run'],
        'id': owner['id'],
        'offset': offset,
      });
      if (!mounted || selected != target || readGeneration != generation) {
        return;
      }
      setState(() {
        receipt = (page['receipt'] as Map).cast<String, dynamic>();
        rows[target] = receipt!;
        outputPages[offset] = '${page['output']}';
        output = outputPages.values.join();
        nextOffset = page['nextOffset'] as int;
        hasMore = page['hasMore'] == true;
      });
    } catch (failure) {
      if (mounted && selected == target) setState(() => error = '$failure');
    }
  }

  Future<void> effect(String operation, {bool eof = false, String? run}) async {
    if (pending || loading || (receipt == null && run == null)) return;
    final owner = receipt, text = eof ? '' : input.text, target = draftKey;
    if (operation == 'input' &&
        (utf8.encode(text).length > 4096 || text.isEmpty && !eof)) {
      setState(
        () => error = 'Send 1–4096 bytes at a time. Your input remains here.',
      );
      return;
    }
    setState(() {
      pending = true;
      error = null;
    });
    try {
      await call({
        'operation': operation,
        'run': run ?? owner!['run'],
        if (operation != 'save') 'id': owner!['id'],
        if (operation == 'input') ...{
          'revision': owner!['inputRevision'],
          'text': text,
          'eof': eof,
        },
      });
      if (!mounted) return;
      if (operation == 'input' &&
          !eof &&
          target == draftKey &&
          input.text == text) {
        input.clear();
      }
    } catch (failure) {
      if (mounted) {
        error =
            '$failure\nInspect the latest receipt before sending again. Your input is retained.';
      }
    } finally {
      if (mounted) {
        setState(() => pending = false);
        await refresh(automatic: true);
      }
    }
  }

  void close() {
    if (!pending) Navigator.pop(context);
  }

  Future<void> recoverIsolation() async {
    if (pending || loading) return;
    setState(() {
      pending = true;
      error = null;
    });
    try {
      final result = await call({'operation': 'recoverIsolation'});
      if (mounted) isolation = result;
    } catch (failure) {
      if (mounted) error = '$failure';
    } finally {
      if (mounted) {
        setState(() => pending = false);
        await refresh(automatic: true);
      }
    }
  }

  @override
  Widget build(BuildContext context) {
    final p = Palette(Theme.of(context).brightness == Brightness.dark);
    return CloseOnEscape(
      onClose: close,
      child: PopScope(
        canPop: !pending,
        child: InspectorFrame(
          title: 'Commands',
          subtitle: 'Task-owned processes · output is untrusted',
          canClose: !pending,
          onClose: close,
          child: Column(
            children: [
              Expanded(
                child: SingleChildScrollView(
                  padding: const EdgeInsets.all(16),
                  child: Column(
                    crossAxisAlignment: CrossAxisAlignment.start,
                    children: [
                      if (error != null)
                        SelectableText(
                          error!,
                          style: TextStyle(color: p.errorText),
                        ),
                      if (historyError != null)
                        SelectableText(
                          'Saved history unavailable: $historyError. Refresh to retry.',
                        ),
                      if (isolation['error'] != null ||
                          (isolation['pendingCleanup'] as int? ?? 0) > 0) ...[
                        SelectableText(
                          isolation['error'] as String? ?? 'Isolation cleanup is pending. Inspect retained work before another isolated command.',
                          style: TextStyle(color: p.errorText),
                        ),
                        TextButton(
                          key: const Key('retry-isolation-cleanup'),
                          onPressed:
                              pending ||
                                  loading ||
                                  isolation['available'] != true
                              ? null
                              : recoverIsolation,
                          child: const Text('Retry cleanup'),
                        ),
                      ],
                      for (final warning in warnings)
                        SelectableText(
                          'Output is not fully saved: ${warning['message']}',
                          style: TextStyle(color: p.errorText),
                        ),
                      for (final owner in retained.where(
                        (r) => r['unsaved'] == true,
                      ))
                        Wrap(
                          spacing: 8,
                          children: [
                            TextButton(
                              onPressed: pending || loading
                                  ? null
                                  : () {
                                      setState(() {
                                        viewedRun = owner['run'] as String;
                                        rows.clear();
                                        selected = null;
                                        receipt = null;
                                        cursor = null;
                                      });
                                      unawaited(refresh());
                                    },
                              child: Text(
                                'Inspect unsaved task ${owner['run']}',
                              ),
                            ),
                            TextButton(
                              onPressed: pending || loading
                                  ? null
                                  : () => effect(
                                      'save',
                                      run: owner['run'] as String,
                                    ),
                              child: const Text('Retry saving'),
                            ),
                          ],
                        ),
                      if (rows.isEmpty)
                        Text(
                          loading ? 'Loading commands…' : 'No recorded commands. Commands started by the agent appear here.',
                        ),
                      for (final entry in rows.entries)
                        ListTile(
                          key: ValueKey('command-${entry.key}'),
                          selected: selected == entry.key,
                          contentPadding: EdgeInsets.zero,
                          title: Text(
                            '${entry.value['program']} · ${entry.value['state']}',
                          ),
                          subtitle: Text(
                            '${entry.value['execution']}\n${entry.value['run']} · ${entry.value['id']}',
                          ),
                          onTap: pending || loading
                              ? null
                              : () {
                                  setState(() => select(entry.key));
                                },
                        ),
                      if (cursor != null)
                        TextButton(
                          onPressed: loading || pending
                              ? null
                              : () => refresh(older: true),
                          child: const Text('Earlier commands'),
                        ),
                      if (receipt != null) ...[
                        const Divider(),
                        SelectableText(
                          '${receipt!['state']} · exit ${receipt!['exitCode'] ?? 'unknown'} · ${receipt!['execution']}',
                        ),
                        Text(
                          'Deadline ${receipt!['timeoutSeconds']} s · output ${receipt!['outputBytes']} / ${receipt!['captureBytes']} bytes · input ${receipt!['inputBytes']} / 65536 bytes',
                        ),
                        if (receipt!['truncated'] == true)
                          Text(
                            receipt!['localLog'] is String
                                ? 'Inline output shortened. Complete output file: ${receipt!['localLog']}'
                                : 'Output capture is partial. Inspect the exit status and retained output before a new command.',
                          ),
                        if (receipt!['logError'] == true)
                          const Text(
                            'Some output could not be saved. Inspect retained files and process status.',
                          ),
                        if (receipt!['outputError'] == true ||
                            receipt!['lossyUtf8'] == true)
                          const Text(
                            'Some output could not be read as text. Inspect the result before relying on it.',
                          ),
                        if (!running && receipt!['cleanupComplete'] != true)
                          const Text(
                            'Process cleanup could not be confirmed. Restart Dolores before starting replacement work.',
                          ),
                        if (receipt!['message'] != null)
                          SelectableText('${receipt!['message']}'),
                        if (receipt!['lastInput'] != null)
                          SelectableText(
                            'Last input: ${receipt!['lastInput']['state']} · ${receipt!['lastInput']['writtenBytes']} / ${receipt!['lastInput']['bytes']} bytes delivered to stdin. Delivery does not confirm the command acted on it.',
                          ),
                        const SizedBox(height: 12),
                        SelectableText(
                          output.isEmpty ? 'No captured output yet.' : output,
                          key: const Key('command-output'),
                          style: const TextStyle(
                            fontFamily: UiTokens.codeFont,
                            fontFamilyFallback: UiTokens.codeFontFallback,
                            fontSize: 12,
                          ),
                        ),
                        if (hasMore)
                          TextButton(
                            onPressed: pending || loading
                                ? null
                                : () => read(more: true),
                            child: const Text('More output'),
                          ),
                      ],
                    ],
                  ),
                ),
              ),
              if (receipt != null)
                Padding(
                  padding: const EdgeInsets.fromLTRB(16, 8, 16, 4),
                  child: FieldLabel(
                    label: 'Input to command',
                    help: canInput
                        ? 'Send exact text · include a newline when needed'
                        : attached
                        ? 'Stdin unavailable. Inspect the receipt above.'
                        : 'Saved output; no live process attached.',
                    child: TextField(
                      key: const Key('command-input'),
                      controller: input,
                      minLines: 1,
                      maxLines: 3,
                      readOnly: !canInput || pending,
                      decoration: InputDecoration(),
                    ),
                  ),
                ),
              Padding(
                padding: const EdgeInsets.fromLTRB(16, 4, 16, 12),
                child: Wrap(
                  spacing: 8,
                  runSpacing: 4,
                  children: [
                    TextButton(
                      onPressed: loading || pending ? null : () => refresh(),
                      child: const Text('Refresh'),
                    ),
                    if (receipt != null) ...[
                      FilledButton(
                        key: const Key('command-send'),
                        onPressed: pending || loading || !canInput
                            ? null
                            : () => effect('input'),
                        child: const Text('Send input'),
                      ),
                      TextButton(
                        onPressed: pending || loading || !canInput
                            ? null
                            : () => effect('input', eof: true),
                        child: const Text('Close stdin'),
                      ),
                      TextButton(
                        onPressed: pending || loading || !attached || !running
                            ? null
                            : () => effect('interrupt'),
                        child: const Text('Interrupt'),
                      ),
                      TextButton(
                        onPressed: pending || loading || !attached
                            ? null
                            : () => effect('close'),
                        child: const Text('Close command'),
                      ),
                    ],
                  ],
                ),
              ),
            ],
          ),
        ),
      ),
    );
  }
}
