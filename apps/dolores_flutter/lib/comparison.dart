import 'dart:async';
import 'dart:convert';

import 'package:flutter/material.dart';
import 'package:flutter/services.dart';

import 'chat.dart';
import 'inspector.dart';
import 'theme.dart';
import 'settings_frame.dart';

Future<void> showComparisons(BuildContext context, ChatController chat) =>
    chat.inspectLocalSettings(() async {
      if (chat.session == null) return;
      await showDialog<void>(
        context: context,
        barrierDismissible: false,
        builder: (_) => ComparisonInspector(chat: chat),
      );
    });

class ComparisonInspector extends StatefulWidget {
  final ChatController chat;
  const ComparisonInspector({super.key, required this.chat});
  @override
  State<ComparisonInspector> createState() => _ComparisonInspectorState();
}

class _Trial {
  final prompt = TextEditingController(),
      requiredText = TextEditingController(),
      forbidden = TextEditingController();
  Map<String, dynamic> value() => {
    'prompt': prompt.text,
    'required': requiredText.text
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
    requiredText.dispose();
    forbidden.dispose();
  }
}

class _ComparisonInspectorState extends State<ComparisonInspector> {
  final title = TextEditingController(text: 'Instruction comparison');
  final baselineLabel = TextEditingController(text: 'Baseline'),
      candidateLabel = TextEditingController(text: 'Candidate');
  final baseline = TextEditingController(), candidate = TextEditingController();
  final tokens = TextEditingController(text: '1024'),
      timeout = TextEditingController(text: '30');
  final scroll = ScrollController();
  final trials = <_Trial>[_Trial()];
  List<Map<String, dynamic>> sources = [], records = [];
  Map<String, dynamic>? selected;
  String? baselineSource, candidateSource, error;
  Map<String, dynamic> settings = {};
  String model = '', progress = '';
  bool loading = true,
      busy = false,
      stopping = false,
      older = false,
      persisted = true;
  int? runId;
  @override
  void initState() {
    super.initState();
    load();
  }

  @override
  void dispose() {
    for (final c in [
      title,
      baselineLabel,
      candidateLabel,
      baseline,
      candidate,
      tokens,
      timeout,
    ]) {
      c.dispose();
    }
    for (final t in trials) {
      t.dispose();
    }
    scroll.dispose();
    super.dispose();
  }

  Future<dynamic> call(
    String command, [
    Map<String, dynamic> fields = const {},
  ]) => widget.chat.bridge.call({
    'command': command,
    'session': widget.chat.session,
    ...fields,
  });
  void showError(Object e) {
    if (!mounted) return;
    setState(() => error = e.toString());
    WidgetsBinding.instance.addPostFrameCallback((_) {
      if (mounted && scroll.hasClients) scroll.jumpTo(0);
    });
  }

  Future<void> load() async {
    try {
      final result = await call('comparisonSources');
      if (!mounted) return;
      setState(() {
        sources = (result['items'] as List)
            .map((v) => (v as Map).cast<String, dynamic>())
            .toList();
        model = result['model'] as String;
        final initialize = settings.isEmpty;
        settings = (result['settings'] as Map).cast<String, dynamic>();
        if (initialize) {
          tokens.text = '${settings['maxOutputTokens']}';
          timeout.text = '${settings['timeoutSeconds']}';
        }
      });
      await page();
    } catch (e) {
      showError(e);
    } finally {
      if (mounted) setState(() => loading = false);
    }
  }

  Future<void> page({int? cursor}) async {
    final result = await call('comparisonsPage', {'cursor': cursor});
    if (!mounted) return;
    setState(() {
      records = (result['items'] as List)
          .map((v) => (v as Map).cast<String, dynamic>())
          .toList();
      older = result['hasOlder'] == true;
    });
  }

  Map<String, dynamic> variant(bool base) => {
    'label': base ? baselineLabel.text : candidateLabel.text,
    'text': base ? baseline.text : candidate.text,
    'source': base ? baselineSource : candidateSource,
  };
  Future<void> evaluate() async {
    if (busy || loading) return;
    final output = int.tryParse(tokens.text.trim()),
        seconds = int.tryParse(timeout.text.trim());
    if (output == null ||
        output < 1 ||
        output > 2048 ||
        seconds == null ||
        seconds < 1 ||
        seconds > 60) {
      showError('Use 1–2048 output tokens and 1–60 seconds per response.');
      return;
    }
    final draft = {
      'title': title.text,
      'baseline': variant(true),
      'candidate': variant(false),
      'trials': trials.map((t) => t.value()).toList(),
    };
    setState(() {
      busy = true;
      stopping = false;
      error = null;
      progress = 'Preparing frozen tests…';
      selected = null;
    });
    final id = DateTime.now().microsecondsSinceEpoch ~/ 1000;
    runId = id;
    try {
      await call('startComparison', {
        'id': id,
        'draft': draft,
        'settings': {
          ...settings,
          'maxOutputTokens': output,
          'timeoutSeconds': seconds,
        },
      });
      final deadline = DateTime.now().add(
        Duration(seconds: seconds * trials.length * 2 + 15),
      );
      while (mounted) {
        final events = await widget.chat.bridge.call({
          'command': 'poll',
          'id': id,
        }) as List;
        for (final event in events) {
          if (event['type'] == 'comparisonProgress') {
            setState(
              () => progress = 'Test ${event['test']} · ${event['phase']}',
            );
          }
          if (event['type'] == 'done') {
            setState(() {
              selected = (event['comparison'] as Map?)?.cast<String, dynamic>();
              persisted = event['persisted'] == true;
              progress = '';
            });
            if (event['error'] != null) showError(event['error']);
            try {
              await page();
            } catch (e) {
              showError(e);
            }
            return;
          }
        }
        if (DateTime.now().isAfter(deadline)) {
          await widget.chat.bridge.call({'command': 'cancel', 'id': id});
          throw 'Comparison stopped waiting. Earlier saved evidence remains in Saved comparisons; refresh before starting a new run.';
        }
        await Future<void>.delayed(const Duration(milliseconds: 50));
      }
    } catch (e) {
      try {
        await widget.chat.bridge.call({'command': 'cancel', 'id': id});
      } catch (_) {
        /* Worker may be unavailable. */
      }
      showError(e);
    } finally {
      runId = null;
      if (mounted) {
        setState(() {
          busy = false;
          stopping = false;
        });
      }
    }
  }

  Future<void> stop() async {
    if (runId == null || stopping) return;
    setState(() => stopping = true);
    try {
      await widget.chat.bridge.call({'command': 'cancel', 'id': runId});
    } catch (e) {
      showError(e);
      if (mounted) setState(() => stopping = false);
    }
  }

  Future<void> select(Map<String, dynamic> record) async {
    setState(() {
      loading = true;
      error = null;
    });
    try {
      final result = await call('comparison', {'comparisonId': record['id']});
      if (mounted) {
        setState(() {
          selected = (result as Map).cast<String, dynamic>();
          persisted = true;
        });
      }
    } catch (e) {
      showError(e);
    } finally {
      if (mounted) setState(() => loading = false);
    }
  }

  void useDraft() {
    final draft = selected!['draft'];
    title.text = draft['title'];
    baselineLabel.text = draft['baseline']['label'];
    baseline.text = draft['baseline']['text'];
    baselineSource = draft['baseline']['source'];
    candidateLabel.text = draft['candidate']['label'];
    candidate.text = draft['candidate']['text'];
    candidateSource = draft['candidate']['source'];
    if (!sources.any(
      (v) => v['source'] == baselineSource && v['text'] == baseline.text,
    )) {
      baselineSource = null;
    }
    if (!sources.any(
      (v) => v['source'] == candidateSource && v['text'] == candidate.text,
    )) {
      candidateSource = null;
    }
    for (final t in trials) {
      t.dispose();
    }
    trials.clear();
    for (final test in draft['trials']) {
      trials.add(
        _Trial()
          ..prompt.text = test['prompt']
          ..requiredText.text = (test['required'] as List).join('\n')
          ..forbidden.text = (test['forbidden'] as List).join('\n'),
      );
    }
    setState(() {
      selected = null;
      error = null;
    });
  }

  Future<void> deleteSelected() async {
    if (loading || busy || selected == null || !persisted) return;
    setState(() => loading = true);
    try {
      await call('deleteComparison', {
        'comparisonId': selected!['id'],
        'revision': selected!['revision'],
      });
      if (mounted) setState(() => selected = null);
      await page();
    } catch (e) {
      showError(e);
    } finally {
      if (mounted) setState(() => loading = false);
    }
  }

  Widget instructions(bool base) {
    final source = base ? baselineSource : candidateSource;
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        Text(
          base ? 'Baseline instructions' : 'Candidate instructions',
          style: const TextStyle(fontWeight: FontWeight.w600),
        ),
        if (sources.isNotEmpty)
          DropdownButtonFormField<String>(
            key: ValueKey('${base ? 'baseline' : 'candidate'}-$source'),
            initialValue: sources.any((v) => v['source'] == source)
                ? source
                : null,
            isExpanded: true,
            decoration: const InputDecoration(
              labelText: 'Copy a saved memory / skill version',
            ),
            items: sources
                .map(
                  (v) => DropdownMenuItem(
                    value: v['source'] as String,
                    child: Text(
                      v['label'] as String,
                      maxLines: 1,
                      overflow: TextOverflow.ellipsis,
                    ),
                  ),
                )
                .toList(),
            onChanged: busy
                ? null
                : (id) {
                    final value = sources.firstWhere((v) => v['source'] == id);
                    setState(() {
                      if (base) {
                        baselineSource = id;
                        baselineLabel.text = value['label'];
                        baseline.text = value['text'];
                      } else {
                        candidateSource = id;
                        candidateLabel.text = value['label'];
                        candidate.text = value['text'];
                      }
                    });
                  },
          ),
        TextField(
          controller: base ? baselineLabel : candidateLabel,
          enabled: !busy,
          decoration: const InputDecoration(labelText: 'Snapshot label'),
          onChanged: (_) => setState(() {
            if (base) {
              baselineSource = null;
            } else {
              candidateSource = null;
            }
          }),
        ),
        TextField(
          key: Key(base ? 'comparison-baseline' : 'comparison-candidate'),
          controller: base ? baseline : candidate,
          enabled: !busy,
          minLines: 2,
          maxLines: 6,
          decoration: const InputDecoration(
            labelText: 'Exact instruction text',
            helperText: 'Blank is allowed. At most 8 KiB; text only.',
          ),
          onChanged: (_) => setState(() {
            if (base) {
              baselineSource = null;
            } else {
              candidateSource = null;
            }
          }),
        ),
        Text(
          source == null
              ? 'Manual snapshot · use labels to identify copied revisions'
              : 'Saved snapshot · rechecked before the first request',
        ),
        const SizedBox(height: 16),
      ],
    );
  }

  Widget receipt(Palette p) {
    final run = selected!, summary = run['summary'] as Map;
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        Text(
          '${run['draft']['title']} · ${run['status']}',
          style: const TextStyle(fontWeight: FontWeight.w600),
        ),
        Text(
          'Model: ${run['model']} · ${run['settings']['maxOutputTokens']} output tokens · ${run['settings']['timeoutSeconds']} seconds per response',
        ),
        Text(
          'Baseline ${summary['baselinePassed']}/${summary['tests']} · Candidate ${summary['candidatePassed']}/${summary['tests']}',
        ),
        Text(
          !persisted
              ? 'This visible receipt was not fully saved. Copy it before closing.'
              : summary['improved'] == true
              ? 'Candidate passes all frozen checks with a strict gain. Review before using it; no memory or skill was activated.'
              : 'No complete strict improvement. Incomplete responses never pass.',
        ),
        if (run['status'] == 'running')
          const Text(
            'Unfinished receipt. It may have been interrupted; there is no automatic resumption.',
          ),
        for (final (index, result) in (run['results'] as List).indexed)
          ExpansionTile(
            title: Text(
              'Test ${index ~/ 2 + 1} · ${index.isEven ? 'Baseline' : 'Candidate'} · ${result['outcome']}',
            ),
            children: [
              if (result['detail'] != null)
                SelectableText(
                  result['detail'],
                  style: TextStyle(color: p.errorText),
                ),
              SelectableText(result['output'] as String),
              SelectableText(
                'Elapsed: ${result['elapsedMs']} ms · Reported usage: ${jsonEncode(result['usage'])}',
              ),
            ],
          ),
        ExpansionTile(
          title: const Text('Exact frozen requests and checks'),
          children: [
            SelectableText(
              const JsonEncoder.withIndent('  ').convert({
                'draft': run['draft'],
                'baselineMessages': run['baselineMessages'],
                'candidateMessages': run['candidateMessages'],
              }),
            ),
          ],
        ),
      ],
    );
  }

  @override
  Widget build(BuildContext context) {
    final p = Palette(Theme.of(context).brightness == Brightness.dark);
    return PopScope(
      canPop: !busy && !loading,
      child: InspectorFrame(
        title: 'Compare instructions',
        subtitle:
            'Frozen response tests · no tool access or automatic activation',
        canClose: !busy && !loading,
        child: Column(
          children: [
            Expanded(
              child: ListView(
                controller: scroll,
                padding: const EdgeInsets.all(20),
                children: [
                  if (error != null)
                    Text(
                      error!,
                      key: const Key('comparison-error'),
                      style: TextStyle(color: p.errorText),
                    ),
                  if (loading) const LinearProgressIndicator(),
                  const Text(
                    'Compare two instruction versions with the same model.',
                  ),
                  const Text(
                    'Selected text goes to your provider. Up to six paid responses.',
                  ),
                  const SettingsDetails(
                    title: 'Comparison details',
                    children: [
                      Text(
                        'Compare two memory / skill instruction snapshots with the same model and 1–3 tests. Up to six paid responses. Chat history, other active instructions and tools are excluded. Selected text and responses are saved locally and included in chat exports.',
                      ),
                      Text(
                        'Exact snippet checks measure these responses only; they do not establish general task quality, statistical improvement or permission to activate a change.',
                      ),
                    ],
                  ),
                  if (selected != null)
                    receipt(p)
                  else ...[
                    Text(
                      'Model: ${model.isEmpty ? 'Configure a model connection first' : model}',
                    ),
                    TextField(
                      controller: title,
                      enabled: !busy,
                      decoration: const InputDecoration(
                        labelText: 'Comparison title',
                      ),
                    ),
                    const SizedBox(height: 12),
                    instructions(true),
                    instructions(false),
                    for (final (index, test) in trials.indexed) ...[
                      Text(
                        'Test ${index + 1}',
                        style: const TextStyle(fontWeight: FontWeight.w600),
                      ),
                      TextField(
                        key: Key('comparison-prompt-$index'),
                        controller: test.prompt,
                        enabled: !busy,
                        minLines: 2,
                        maxLines: 4,
                        decoration: const InputDecoration(
                          labelText: 'Prompt · up to 2 KiB',
                        ),
                      ),
                      TextField(
                        key: Key('comparison-required-$index'),
                        controller: test.requiredText,
                        enabled: !busy,
                        minLines: 1,
                        maxLines: 4,
                        decoration: const InputDecoration(
                          labelText: 'Required exact snippets · one per line',
                          helperText: '1–4 snippets, each within 128 bytes.',
                        ),
                      ),
                      TextField(
                        controller: test.forbidden,
                        enabled: !busy,
                        minLines: 1,
                        maxLines: 4,
                        decoration: const InputDecoration(
                          labelText: 'Forbidden exact snippets · optional',
                        ),
                      ),
                      if (trials.length > 1)
                        TextButton(
                          onPressed: busy
                              ? null
                              : () => setState(() {
                                  trials.removeAt(index).dispose();
                                }),
                          child: const Text('Remove test'),
                        ),
                      const SizedBox(height: 12),
                    ],
                    if (trials.length < 3)
                      TextButton(
                        onPressed: busy
                            ? null
                            : () => setState(() => trials.add(_Trial())),
                        child: const Text('Add test'),
                      ),
                    TextField(
                      key: const Key('comparison-tokens'),
                      controller: tokens,
                      enabled: !busy,
                      keyboardType: TextInputType.number,
                      decoration: const InputDecoration(
                        labelText: 'Output tokens per response · up to 2048',
                      ),
                    ),
                    TextField(
                      controller: timeout,
                      enabled: !busy,
                      keyboardType: TextInputType.number,
                      decoration: const InputDecoration(
                        labelText: 'Timeout per response · up to 60 seconds',
                      ),
                    ),
                    const Text(
                      'These allowances apply only to this comparison. Reasoning uses the selected model profile; chat settings are unchanged.',
                    ),
                  ],
                  if (progress.isNotEmpty)
                    Text(
                      stopping
                          ? 'Stopping… Earlier evidence remains.'
                          : progress,
                    ),
                  const Divider(),
                  const Text('Saved comparisons · up to 30 per chat'),
                  if (records.isEmpty && !loading)
                    const Text('No saved comparisons yet.'),
                  for (final record in records)
                    ListTile(
                      contentPadding: EdgeInsets.zero,
                      title: Text(record['title']),
                      subtitle: Text(
                        '${record['model']} · ${record['status']} · #${record['id']}',
                      ),
                      onTap: busy || loading ? null : () => select(record),
                    ),
                  Wrap(
                    spacing: 8,
                    children: [
                      TextButton(
                        onPressed: busy || loading
                            ? null
                            : () async {
                                try {
                                  await page();
                                } catch (e) {
                                  showError(e);
                                }
                              },
                        child: const Text('Newest'),
                      ),
                      TextButton(
                        onPressed: busy || loading || !older
                            ? null
                            : () async {
                                try {
                                  await page(cursor: records.last['id'] as int);
                                } catch (e) {
                                  showError(e);
                                }
                              },
                        child: const Text('Older'),
                      ),
                    ],
                  ),
                ],
              ),
            ),
            Padding(
              padding: const EdgeInsets.all(12),
              child: Wrap(
                spacing: 8,
                runSpacing: 6,
                children: [
                  if (busy)
                    FilledButton(
                      onPressed: stopping ? null : stop,
                      child: const Text('Stop'),
                    )
                  else if (selected == null) ...[
                    TextButton(
                      onPressed: loading ? null : load,
                      child: const Text('Refresh sources'),
                    ),
                    FilledButton(
                      key: const Key('comparison-evaluate'),
                      onPressed: loading || !widget.chat.configured
                          ? null
                          : evaluate,
                      child: const Text('Run comparison'),
                    ),
                  ] else ...[
                    TextButton(
                      onPressed: loading
                          ? null
                          : () => setState(() => selected = null),
                      child: const Text('New comparison'),
                    ),
                    TextButton(
                      onPressed: loading ? null : useDraft,
                      child: const Text('Use as draft'),
                    ),
                    TextButton(
                      onPressed: () => Clipboard.setData(
                        ClipboardData(
                          text: const JsonEncoder.withIndent('  ')
                              .convert(selected),
                        ),
                      ),
                      child: const Text('Copy receipt'),
                    ),
                    TextButton(
                      onPressed: !persisted || loading ? null : deleteSelected,
                      child: const Text('Delete comparison'),
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
