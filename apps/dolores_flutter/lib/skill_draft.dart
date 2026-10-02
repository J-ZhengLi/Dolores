import 'dart:async';

import 'package:flutter/material.dart';

import 'bridge.dart';
import 'inspector.dart';
import 'theme.dart';

class _Trial {
  final prompt = TextEditingController();
  final required = TextEditingController();
  final forbidden = TextEditingController();
  Map<String, dynamic> get value => {
    'prompt': prompt.text,
    'required': required.text
        .split('\n')
        .where((s) => s.trim().isNotEmpty)
        .toList(),
    'forbidden': forbidden.text
        .split('\n')
        .where((s) => s.trim().isNotEmpty)
        .toList(),
  };
  void dispose() {
    prompt.dispose();
    required.dispose();
    forbidden.dispose();
  }
}

/// Local review first; generation, testing and activation are separate actions.
class SkillDraftInspector extends StatefulWidget {
  final ChatBridge bridge;
  final String session, scope;
  const SkillDraftInspector({
    super.key,
    required this.bridge,
    required this.session,
    required this.scope,
  });
  @override
  State<SkillDraftInspector> createState() => _SkillDraftInspectorState();
}

class _SkillDraftInspectorState extends State<SkillDraftInspector> {
  final name = TextEditingController(),
      description = TextEditingController(),
      instructions = TextEditingController();
  final trials = [_Trial()];
  final selected = <int>{};
  final _scroll = ScrollController();
  Map<String, dynamic>? sources, draft, result;
  String? error, progress;
  bool busy = false, stopping = false;
  int? run;
  String? get token =>
      (result?['token'] ?? draft?['token'] ?? sources?['token']) as String?;
  Future<dynamic> _call(
    String command, [
    Map<String, dynamic> args = const {},
  ]) => widget.bridge.call({
    'command': command,
    'session': widget.session,
    'scope': widget.scope,
    ...args,
  });
  @override
  void initState() {
    super.initState();
    _act(() async {
      final value = await _call('reviewSkillExamples') as Map;
      if (mounted) setState(() => sources = value.cast<String, dynamic>());
    });
  }

  @override
  void dispose() {
    if (token != null) {
      unawaited(
        _call('discardSkillDraft', {'token': token}).catchError((_) => null),
      );
    }
    name.dispose();
    _scroll.dispose();
    description.dispose();
    instructions.dispose();
    for (final t in trials) {
      t.dispose();
    }
    super.dispose();
  }

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
      if (mounted) {
        setState(() {
          busy = false;
          run = null;
          stopping = false;
          progress = null;
        });
      }
    }
  }

  void _edited(String _) {
    // Retain the host token for a new evaluation, but never offer the old result.
    if (result != null) {
      setState(
        () => result = {...result!, 'promotable': false, 'edited': true},
      );
    }
  }

  Future<Map<String, dynamic>> _request(
    String command,
    Map<String, dynamic> args,
    String field,
  ) async {
    final id = DateTime.now().microsecondsSinceEpoch;
    setState(() {
      run = id;
      stopping = false;
      progress = command == 'generateSkillDraft'
          ? 'Drafting from your selected exchanges…'
          : 'Testing the baseline…';
    });
    await _call(command, {'id': id, 'token': token, ...args});
    final deadline = DateTime.now().add(
      Duration(seconds: command == 'generateSkillDraft' ? 40 : 80),
    );
    while (mounted) {
      final events = await _call('poll', {'id': id}) as List;
      for (final event in events) {
        if (event['type'] == 'skillEvaluationProgress') {
          setState(
            () => progress =
                'Test ${event['test']} · ${event['phase'] == 'baseline' ? 'baseline' : 'with draft'}',
          );
        }
        if (event['type'] != 'done') continue;
        if (event['error'] != null) throw event['error'] as String;
        final value = (event[field] as Map).cast<String, dynamic>();
        if (stopping) {
          await _call('discardSkillDraft', {'token': value['token']});
          throw 'Stopped. Nothing was saved. Close and review the exchanges again.';
        }
        return value;
      }
      if (DateTime.now().isAfter(deadline)) {
        await _call('cancel', {'id': id});
        await _call('shutdown');
        throw 'The request timed out. Nothing was saved. Try again.';
      }
      await Future<void>.delayed(const Duration(milliseconds: 80));
    }
    throw 'Review closed.';
  }

  Future<void> _generate() => _act(() async {
    final value = await _request('generateSkillDraft', {
      'messageIds': selected.toList(),
    }, 'skillDraft');
    if (!mounted) return;
    setState(() {
      draft = value;
      name.text = value['draft']['name'] as String;
      description.text = value['draft']['description'] as String;
      instructions.text = value['draft']['instructions'] as String;
    });
    if (_scroll.hasClients) _scroll.jumpTo(0);
  });
  Future<void> _evaluate() => _act(() async {
    // Invalidating happens before validation/network, including failed retries.
    if (result != null) {
      setState(
        () => result = {...result!, 'promotable': false, 'edited': true},
      );
    }
    final value = await _request('evaluateSkillDraft', {
      'draft': {
        ...(draft!['draft'] as Map),
        'name': name.text,
        'description': description.text,
        'instructions': instructions.text,
      },
      'trials': trials.map((t) => t.value).toList(),
    }, 'skillEvaluation');
    if (mounted) setState(() => result = value);
  });
  Future<void> _activate() => _act(() async {
    if (result?['promotable'] != true) return;
    await _call('promoteSkillDraft', {'token': token});
    if (mounted) Navigator.pop(context, true);
  });
  Future<void> _stop() async {
    if (run == null || stopping) return;
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

  Widget _field(
    String label,
    TextEditingController controller,
    String key, {
    int lines = 1,
  }) => TextField(
    key: Key(key),
    controller: controller,
    enabled: !busy,
    minLines: lines,
    maxLines: lines == 1 ? 1 : lines + 4,
    decoration: InputDecoration(labelText: label),
    onChanged: _edited,
  );
  @override
  Widget build(BuildContext context) {
    final p = Palette(Theme.of(context).brightness == Brightness.dark);
    final examples = sources?['examples'] as List? ?? [];
    return PopScope(
      canPop: !busy,
      child: InspectorFrame(
        title: 'Draft a skill',
        subtitle:
            '${widget.scope == 'global' ? 'All chats' : 'This working folder'} · review, test, then activate',
        canClose: !busy,
        child: Column(
          children: [
            if (busy) const LinearProgressIndicator(minHeight: 2),
            Expanded(
              child: ListView(
                key: const Key('skill-draft-scroll'),
                controller: _scroll,
                padding: const EdgeInsets.all(20),
                children: [
                  if (error != null)
                    SelectableText(
                      error!,
                      style: TextStyle(color: p.errorText),
                    ),
                  if (progress != null) Text(progress!),
                  if (draft == null) ...[
                    const Text('Select work you found useful'),
                    const SizedBox(height: 8),
                    const Text(
                      'Choose 1–3 completed exchanges. Generate sends their full requests and responses to your configured model, including any file contents in those replies. Selection stays local until then.',
                    ),
                    const SizedBox(height: 8),
                    const Text(
                      'Latest 20 exchanges · 16 KiB combined. One tool-free request, up to 1,024 output tokens. No skill is saved yet.',
                    ),
                    if (examples.isEmpty && !busy)
                      const Text(
                        'No completed exchanges yet. Finish a chat response, then return here.',
                      ),
                    for (final example in examples)
                      Card(
                        elevation: 0,
                        color: p.surface,
                        child: Column(
                          children: [
                            CheckboxListTile(
                              key: Key('skill-example-${example['messageId']}'),
                              value: selected.contains(example['messageId']),
                              onChanged: busy
                                  ? null
                                  : (value) => setState(() {
                                      if (value == true &&
                                          selected.length < 3) {
                                        selected.add(
                                          example['messageId'] as int,
                                        );
                                      } else if (value != true) {
                                        selected.remove(example['messageId']);
                                      }
                                    }),
                              title: Text(
                                example['request'] as String,
                                maxLines: 2,
                                overflow: TextOverflow.ellipsis,
                              ),
                            ),
                            ExpansionTile(
                              title: const Text('Review full exchange'),
                              childrenPadding: const EdgeInsets.all(12),
                              children: [
                                const Text('You'),
                                SelectableText(example['request'] as String),
                                const SizedBox(height: 8),
                                const Text('Dolores'),
                                SelectableText(example['response'] as String),
                              ],
                            ),
                          ],
                        ),
                      ),
                  ] else ...[
                    Text('Drafted by ${draft!['model']}'),
                    if (draft!['warning'] is String && result == null)
                      Text(
                        'Correct the draft before evaluation: ${draft!['warning']}',
                      ),
                    const Text(
                      'Edit a narrow workflow and when it applies. Avoid private details. It will be shared in future messages after activation.',
                    ),
                    const SizedBox(height: 12),
                    _field(
                      'Skill name · lowercase letters, digits, hyphens',
                      name,
                      'draft-skill-name',
                    ),
                    const SizedBox(height: 12),
                    _field(
                      'When to use this skill',
                      description,
                      'draft-skill-description',
                      lines: 2,
                    ),
                    const SizedBox(height: 12),
                    _field(
                      'Markdown instructions · 8 KiB total skill limit',
                      instructions,
                      'draft-skill-instructions',
                      lines: 5,
                    ),
                    ExpansionTile(
                      title: const Text('Source evidence'),
                      children: [
                        for (final e in draft!['draft']['evidence'] as List)
                          SelectableText(
                            'Reply ${e['messageId']}: ${e['quote']}',
                          ),
                      ],
                    ),
                    const SizedBox(height: 12),
                    const Text(
                      'Response tests',
                      style: TextStyle(fontWeight: FontWeight.w600),
                    ),
                    const Text(
                      'Each prompt runs twice: current skills, then this draft. Other active skills stay the same. No chat history, memories, folder instructions or tools are included. Exact checks are case-sensitive and measure these responses only.',
                    ),
                    const SizedBox(height: 8),
                    const Text(
                      '1–3 tests · up to 6 requests, 512 output tokens and 10 seconds each. Your model connection and context-window setting apply. Provider usage may vary.',
                    ),
                    for (var i = 0; i < trials.length; i++)
                      Card(
                        elevation: 0,
                        color: p.surface,
                        child: Padding(
                          padding: const EdgeInsets.all(12),
                          child: Column(
                            children: [
                              Row(
                                children: [
                                  Expanded(child: Text('Test ${i + 1}')),
                                  if (trials.length > 1)
                                    IconButton(
                                      key: Key('remove-skill-test-$i'),
                                      tooltip: 'Remove test',
                                      onPressed: busy
                                          ? null
                                          : () => setState(() {
                                              trials.removeAt(i).dispose();
                                              _edited('');
                                            }),
                                      icon: const Icon(Icons.close),
                                    ),
                                ],
                              ),
                              _field(
                                'Prompt · 2 KiB',
                                trials[i].prompt,
                                'skill-test-prompt-$i',
                                lines: 2,
                              ),
                              const SizedBox(height: 12),
                              _field(
                                'Must include · 1–4 snippets, one per line',
                                trials[i].required,
                                'skill-test-required-$i',
                                lines: 2,
                              ),
                              const SizedBox(height: 12),
                              _field(
                                'Must exclude · optional, one per line',
                                trials[i].forbidden,
                                'skill-test-forbidden-$i',
                                lines: 2,
                              ),
                            ],
                          ),
                        ),
                      ),
                    Align(
                      alignment: Alignment.centerLeft,
                      child: TextButton(
                        key: const Key('add-skill-test'),
                        onPressed: busy || trials.length >= 3
                            ? null
                            : () => setState(() {
                                trials.add(_Trial());
                                _edited('');
                              }),
                        child: const Text('Add test'),
                      ),
                    ),
                    if (result != null) ...[
                      if (result!['edited'] == true)
                        const Text(
                          'Draft or tests changed. Evaluate again before activation.',
                        ),
                      SkillEvaluationView(
                        evaluation: result!['evaluation'] as Map,
                      ),
                      Text(
                        result!['promotable'] == true
                            ? 'All tests passed and the score improved. Review the responses before activation.'
                            : 'Activation requires every test to pass and more passes than the baseline.',
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
                  if (run != null)
                    TextButton(
                      key: const Key('stop-skill-draft'),
                      onPressed: stopping ? null : _stop,
                      child: Text(stopping ? 'Stopping…' : 'Stop'),
                    ),
                  if (draft == null)
                    FilledButton(
                      key: const Key('generate-skill-draft'),
                      onPressed: busy || selected.isEmpty ? null : _generate,
                      child: const Text('Generate draft'),
                    ),
                  if (draft != null) ...[
                    OutlinedButton(
                      key: const Key('evaluate-skill-draft'),
                      onPressed: busy ? null : _evaluate,
                      child: const Text('Evaluate draft'),
                    ),
                    FilledButton(
                      key: const Key('promote-skill-draft'),
                      onPressed: busy || result?['promotable'] != true
                          ? null
                          : _activate,
                      child: const Text('Activate tested skill'),
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

class SkillEvaluationView extends StatelessWidget {
  final Map evaluation;
  const SkillEvaluationView({super.key, required this.evaluation});
  bool _pass(Map r, String key) {
    final text = r[key] as String;
    final t = r['trial'] as Map;
    return (t['required'] as List).every((s) => text.contains(s as String)) &&
        (t['forbidden'] as List? ?? []).every(
          (s) => !text.contains(s as String),
        );
  }

  @override
  Widget build(BuildContext context) {
    final results = evaluation['results'] as List;
    final base = results.where((r) => _pass(r as Map, 'baseline')).length;
    final candidate = results.where((r) => _pass(r as Map, 'candidate')).length;
    return ExpansionTile(
      key: const Key('skill-evaluation-receipt'),
      initiallyExpanded: true,
      title: Text(
        'Baseline $base/${results.length} · With draft $candidate/${results.length}',
      ),
      subtitle: Text(
        '${evaluation['model']} · ${DateTime.fromMillisecondsSinceEpoch(evaluation['reviewedAt'] as int).toLocal().toString().split('.').first}',
      ),
      children: [
        const Text(
          'Historical response check. Passing a few literal tests does not establish general task improvement.',
        ),
        if (evaluation['settings'] is Map)
          Text(
            'Up to ${evaluation['settings']['maxOutputTokens']} output tokens · ${evaluation['settings']['timeoutSeconds']} seconds per request · ${evaluation['contextWindowTokens'] ?? 'unset'} context tokens',
          ),
        for (var i = 0; i < results.length; i++)
          ExpansionTile(
            title: Text(
              'Test ${i + 1} · baseline ${_pass(results[i] as Map, 'baseline') ? 'pass' : 'fail'} · with draft ${_pass(results[i] as Map, 'candidate') ? 'pass' : 'fail'}',
            ),
            children: [
              SelectableText(
                'Prompt: ${results[i]['trial']['prompt']}\nMust include: ${(results[i]['trial']['required'] as List).join(', ')}\nMust exclude: ${(results[i]['trial']['forbidden'] as List? ?? []).join(', ')}',
              ),
              const SizedBox(height: 8),
              const Text('Baseline response'),
              SelectableText(results[i]['baseline'] as String),
              const SizedBox(height: 8),
              const Text('With draft'),
              SelectableText(results[i]['candidate'] as String),
              for (final key in ['baselineUsage', 'candidateUsage'])
                if (results[i][key] is Map)
                  Text(
                    '${key == 'baselineUsage' ? 'Baseline' : 'With draft'} · provider reported ${results[i][key]['inputTokens']} in / ${results[i][key]['outputTokens']} out',
                  ),
              if (results[i]['baselineMessages'] is List)
                ExpansionTile(
                  title: const Text('Exact tested requests'),
                  children: [
                    for (final key in [
                      'baselineMessages',
                      'candidateMessages',
                    ]) ...[
                      Text(
                        key == 'baselineMessages' ? 'Baseline' : 'With draft',
                      ),
                      for (final m in results[i][key] as List)
                        SelectableText('${m['role']}: ${m['content']}'),
                    ],
                  ],
                ),
            ],
          ),
      ],
    );
  }
}
