import 'dart:async';
import 'dart:convert';

import 'package:flutter/material.dart';

import 'bridge.dart';
import 'chat.dart';
import 'inspector.dart';
import 'theme.dart';

Future<void> showSessionSummary(BuildContext context, ChatController chat) =>
    chat.inspectLocalSettings(() async {
      if (chat.session == null) return;
      await showDialog<void>(
        context: context,
        barrierDismissible: false,
        builder: (_) => SessionSummaryInspector(
          bridge: chat.bridge,
          session: chat.session!,
        ),
      );
      chat.invalidateContext();
    });

class SessionSummaryInspector extends StatefulWidget {
  final ChatBridge bridge;
  final String session;
  const SessionSummaryInspector({
    super.key,
    required this.bridge,
    required this.session,
  });
  @override
  State<SessionSummaryInspector> createState() =>
      _SessionSummaryInspectorState();
}

class _SessionSummaryInspectorState extends State<SessionSummaryInspector> {
  final text = TextEditingController();
  Map<String, dynamic>? review, draft;
  bool busy = false, generating = false, stopping = false;
  bool sources = false, editing = false;
  int? run;
  String? error, notice;
  Map? get saved => review?['summary'] as Map?;
  List get messages => review?['messages'] as List? ?? [];
  String? get token => (draft?['token'] ?? review?['token']) as String?;

  @override
  void initState() {
    super.initState();
    _act(_load);
  }

  @override
  void dispose() {
    if (token != null) {
      unawaited(
        _call('discardSummaryReview', {'token': token}).catchError((_) => null),
      );
    }
    text.dispose();
    super.dispose();
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
    final result = await _call('reviewSummary') as Map;
    if (!mounted) return;
    setState(() {
      review = result.cast<String, dynamic>();
      draft = null;
      editing = false;
      sources = saved == null;
    });
  }

  Future<void> _generate() => _act(() async {
    final number = DateTime.now().microsecondsSinceEpoch;
    setState(() {
      generating = true;
      stopping = false;
      run = number;
      notice = null;
    });
    try {
      await _call('generateSummary', {'id': number, 'token': token});
      final deadline = DateTime.now().add(const Duration(seconds: 40));
      while (mounted) {
        final events = await _call('poll', {'id': number}) as List;
        for (final event in events) {
          if (event['type'] != 'done') continue;
          if (event['error'] != null) throw event['error'] as String;
          if (stopping) {
            await _call('discardSummaryReview', {
              'token': event['summaryDraft']['token'],
            });
            throw 'Summary stopped. Nothing was saved.';
          }
          setState(() {
            draft = (event['summaryDraft'] as Map).cast<String, dynamic>();
            text.text = draft!['text'] as String;
          });
          return;
        }
        if (DateTime.now().isAfter(deadline)) {
          await _call('cancel', {'id': number});
          await _call('shutdown');
          throw 'Summary did not finish. Nothing was saved. Try again.';
        }
        await Future<void>.delayed(const Duration(milliseconds: 80));
      }
    } catch (_) {
      await _load(); // Local review only; never automatically retry the model.
      if (mounted) setState(() => sources = true);
      rethrow;
    } finally {
      if (mounted) {
        setState(() {
          generating = false;
          stopping = false;
          run = null;
        });
      }
    }
  });

  Future<void> _stop() async {
    if (!generating || stopping || run == null) return;
    setState(() => stopping = true);
    try {
      await _call('cancel', {'id': run});
    } catch (failure) {
      if (mounted) {
        setState(() {
          error = failure.toString();
          stopping = false;
        });
      }
    }
  }

  Future<void> _save() => _act(() async {
    if (text.text.trim().isEmpty ||
        utf8.encode(text.text).length > 8192 ||
        text.text.contains('\x00')) {
      throw const FormatException('Write a summary within 8 KiB of text.');
    }
    await _call(editing ? 'correctSummary' : 'saveSummary', {
      'text': text.text,
      if (editing) 'revision': saved!['provenance']['revision'],
      if (!editing) 'token': draft!['token'],
    });
    if (mounted) {
      setState(
        () => notice = 'Summary saved for future messages in this chat.',
      );
    }
    await _load();
  });

  Future<void> _discard() => _act(() async {
    if (token != null) await _call('discardSummaryReview', {'token': token});
    await _load();
  });

  Future<void> _delete() => _act(() async {
    await _call('deleteSummary', {
      'revision': saved!['provenance']['revision'],
    });
    if (mounted) {
      setState(
        () => notice =
            'Summary deleted. Future messages use recent conversation history.',
      );
    }
    await _load();
  });

  @override
  Widget build(BuildContext context) {
    final p = Palette(Theme.of(context).brightness == Brightness.dark);
    final form = draft != null || editing;
    return PopScope(
      canPop: !busy,
      child: InspectorFrame(
        title: 'Session summary',
        subtitle: 'Keep this conversation’s progress in context',
        canClose: !busy,
        child: Column(
          children: [
            if (busy) const LinearProgressIndicator(minHeight: 2),
            Expanded(
              child: ListView(
                padding: const EdgeInsets.all(20),
                children: [
                  const Text(
                    'A saved summary replaces the older turns it covers in future model context. Your full chat stays in history. It is used only in this chat.',
                  ),
                  if (error != null)
                    Padding(
                      padding: const EdgeInsets.only(top: 12),
                      child: Text(error!, style: TextStyle(color: p.errorText)),
                    ),
                  if (notice != null)
                    Padding(
                      padding: const EdgeInsets.only(top: 12),
                      child: Text(notice!, style: TextStyle(color: p.accent)),
                    ),
                  const SizedBox(height: 16),
                  if (form) ...[
                    Text(
                      editing ? 'Correct saved summary' : 'Review draft',
                      style: const TextStyle(fontWeight: FontWeight.w600),
                    ),
                    const SizedBox(height: 8),
                    const Text(
                      'Check facts, decisions and remaining work before saving. Summaries can miss details; correct the text below.',
                    ),
                    const SizedBox(height: 12),
                    TextField(
                      key: const Key('summary-text'),
                      controller: text,
                      enabled: !busy,
                      minLines: 6,
                      maxLines: 14,
                      decoration: const InputDecoration(labelText: 'Summary'),
                    ),
                  ] else ...[
                    if (saved != null) ...[
                      Text(
                        'Saved summary · ${saved!['provenance']['coveredTurns']} ${saved!['provenance']['coveredTurns'] == 1 ? 'turn' : 'turns'} covered · revision ${saved!['provenance']['revision']}',
                        style: const TextStyle(fontWeight: FontWeight.w600),
                      ),
                      Text(
                        'Drafted by ${saved!['provenance']['model']} · reviewed ${DateTime.fromMillisecondsSinceEpoch(saved!['provenance']['updatedAt'] as int).toLocal()}',
                        style: TextStyle(color: p.muted, fontSize: 12),
                      ),
                      const SizedBox(height: 8),
                      SelectableText(
                        saved!['text'] as String,
                        key: const Key('saved-summary-text'),
                      ),
                      const SizedBox(height: 16),
                    ],
                    if (sources) ...[
                      Text(
                        saved == null
                            ? 'Review conversation to summarize'
                            : 'Extend with the next turns',
                        style: const TextStyle(fontWeight: FontWeight.w600),
                      ),
                      const SizedBox(height: 8),
                      Text(
                        'Generate sends the turns shown below${saved == null ? '' : ' and your saved summary above'} to ${review?['model'] ?? 'the selected model'}. Replies may quote private file contents. Review before sharing.',
                        style: TextStyle(color: p.muted, fontSize: 12),
                      ),
                      Text(
                        '${messages.length ~/ 2} complete ${messages.length == 2 ? 'turn' : 'turns'}${review?['hasMore'] == true ? ' · More turns remain for a later update' : ''}',
                      ),
                      if (messages.isEmpty)
                        Text(
                          saved == null
                              ? 'Finish a conversation first to create a summary.'
                              : 'The saved summary covers all completed turns.',
                        ),
                      for (final message in messages)
                        Card(
                          elevation: 0,
                          color: p.surface,
                          child: ExpansionTile(
                            key: ValueKey('summary-source-${message['id']}'),
                            title: Text(
                              '${message['role'] == 'user' ? 'You' : 'Dolores'} · message ${message['id']}',
                              style: const TextStyle(fontSize: 13),
                            ),
                            children: [
                              Padding(
                                padding: const EdgeInsets.fromLTRB(
                                  16,
                                  0,
                                  16,
                                  16,
                                ),
                                child: Align(
                                  alignment: Alignment.centerLeft,
                                  child: SelectableText(
                                    message['content'] as String,
                                  ),
                                ),
                              ),
                            ],
                          ),
                        ),
                    ],
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
                  if (generating)
                    TextButton(
                      key: const Key('stop-summary'),
                      onPressed: stopping ? null : _stop,
                      child: Text(stopping ? 'Stopping…' : 'Stop'),
                    )
                  else if (form) ...[
                    FilledButton(
                      key: const Key('save-summary'),
                      onPressed: busy ? null : _save,
                      child: const Text('Save summary'),
                    ),
                    TextButton(
                      key: const Key('discard-summary'),
                      onPressed: busy ? null : _discard,
                      child: Text(editing ? 'Cancel' : 'Discard'),
                    ),
                  ] else ...[
                    if (sources)
                      FilledButton(
                        key: const Key('generate-summary'),
                        onPressed: busy || messages.isEmpty ? null : _generate,
                        child: const Text('Generate draft'),
                      )
                    else
                      TextButton(
                        key: const Key('review-summary'),
                        onPressed: busy
                            ? null
                            : () => setState(() => sources = true),
                        child: const Text('Review next turns'),
                      ),
                    if (saved != null) ...[
                      TextButton(
                        key: const Key('edit-summary'),
                        onPressed: busy
                            ? null
                            : () => setState(() {
                                editing = true;
                                text.text = saved!['text'] as String;
                                error = null;
                              }),
                        child: const Text('Edit'),
                      ),
                      TextButton(
                        key: const Key('delete-summary'),
                        onPressed: busy ? null : _delete,
                        child: const Text('Delete'),
                      ),
                    ],
                    if (review == null && !busy)
                      TextButton(
                        onPressed: () => _act(_load),
                        child: const Text('Refresh'),
                      ),
                  ],
                ],
              ),
            ),
          ],
        ),
      ),
    );
  }
}
