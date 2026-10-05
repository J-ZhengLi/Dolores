import 'dart:async';
import 'dart:convert';

import 'package:flutter/material.dart';

import 'chat.dart';
import 'inspector.dart';
import 'settings_frame.dart';
import 'skills.dart';
import 'learning.dart';

class SkillSettings extends StatefulWidget {
  final ChatController chat;
  const SkillSettings({super.key, required this.chat});
  @override
  State<SkillSettings> createState() => _SkillSettingsState();
}

class _SkillSettingsState extends State<SkillSettings> {
  int index = 0;
  final panels = <int, Widget>{};
  final locks = [
    ValueNotifier(false),
    ValueNotifier(false),
    ValueNotifier(false),
  ];
  ValueNotifier<bool>? outer;
  bool get pending => locks.any((l) => l.value);
  void changed() {
    outer?.value = pending;
    if (mounted) setState(() {});
  }

  @override
  void initState() {
    super.initState();
    for (final l in locks) {
      l.addListener(changed);
    }
  }

  @override
  void didChangeDependencies() {
    super.didChangeDependencies();
    outer = SettingsEmbedding.of(context)?.pending;
  }

  @override
  void dispose() {
    for (final l in locks) {
      l.dispose();
    }
    super.dispose();
  }

  @override
  Widget build(BuildContext context) => Column(
    children: [
      Padding(
        padding: const EdgeInsets.all(12),
        child: Wrap(
          spacing: 8,
          children: [
            for (var i = 0; i < 3; i++)
              ChoiceChip(
                label: Text(['Library', 'Tool trials', 'Learning'][i]),
                selected: index == i,
                onSelected: pending ? null : (_) => setState(() => index = i),
              ),
          ],
        ),
      ),
      Expanded(
        child: Builder(
          builder: (context) {
            panels.putIfAbsent(
              index,
              () => SettingsEmbedding(
                pending: locks[index],
                route: ModalRoute.of(context),
                child: index == 0
                    ? SkillsInspector(
                        bridge: widget.chat.bridge,
                        session: widget.chat.session!,
                        hasProject: widget.chat.workspaceRoot != null,
                      )
                    : index == 1
                    ? ToolTrialsInspector(chat: widget.chat)
                    : LearningInspector(chat: widget.chat),
              ),
            );
            return IndexedStack(
              index: index,
              children: [
                for (var i = 0; i < 3; i++)
                  panels[i] ?? const SizedBox.shrink(),
              ],
            );
          },
        ),
      ),
    ],
  );
}

class ToolTrialsInspector extends StatefulWidget {
  final ChatController chat;
  const ToolTrialsInspector({super.key, required this.chat});
  @override
  State<ToolTrialsInspector> createState() => _ToolTrialsState();
}

class _ToolTrialsState extends State<ToolTrialsInspector> {
  final candidate = TextEditingController();
  Map? report, receipt;
  String? name, error, progress;
  bool pending = false, stopping = false;
  int? run;
  @override
  void initState() {
    super.initState();
    load();
  }

  @override
  void dispose() {
    candidate.dispose();
    super.dispose();
  }

  List<Map> get skills => ((report?['skills'] as List?) ?? [])
      .cast<Map>()
      .where((s) => s['enabled'] == true)
      .toList();
  Map? get selected => skills.where((s) => s['name'] == name).firstOrNull;
  Future<dynamic> call(
    String command, [
    Map<String, dynamic> fields = const {},
  ]) => widget.chat.bridge.call({
    'command': command,
    'session': widget.chat.session,
    ...fields,
  });
  Future<void> load() async {
    if (widget.chat.workspaceRoot == null) return;
    setState(() => pending = true);
    try {
      final value = await call('trialSources');
      if (mounted) setState(() => report = value as Map);
    } catch (e) {
      if (mounted) setState(() => error = e.toString());
    } finally {
      if (mounted) setState(() => pending = false);
    }
  }

  Future<void> start() async {
    final skill = selected;
    if (skill == null) return;
    setState(() {
      pending = true;
      error = null;
      receipt = null;
      progress = 'Preparing fixed trials…';
      run = DateTime.now().microsecondsSinceEpoch;
    });
    try {
      await call('startToolTrial', {
        'id': run,
        'name': name,
        'revision': skill['revision'],
        'text': candidate.text,
      });
      var done = false;
      while (mounted && !done) {
        final events = await widget.chat.bridge.call({
          'command': 'poll',
          'id': run,
        }) as List;
        for (final event in events) {
          if (event['type'] == 'trialProgress') {
            setState(
              () => progress = 'Case ${event['case']} · ${event['phase']}',
            );
          }
          if (event['type'] == 'done') {
            done = true;
            setState(() {
              receipt = event['trial'] as Map?;
              error = event['error'] as String?;
              progress = event['persisted'] == false
                  ? 'Evidence not saved; no qualification.'
                  : 'Trial finished. No skill was activated.';
            });
          }
        }
        if (!done) {
          await Future<void>.delayed(const Duration(milliseconds: 100));
        }
      }
    } catch (e) {
      if (mounted) {
        setState(
          () => error =
              '$e Earlier evidence and your draft remain. Refresh before starting a fresh trial.',
        );
      }
    } finally {
      if (mounted) {
        setState(() {
          pending = false;
          stopping = false;
          run = null;
        });
      }
    }
  }

  Future<void> stop() async {
    setState(() => stopping = true);
    try {
      await widget.chat.bridge.call({'command': 'cancel', 'id': run});
    } catch (e) {
      if (mounted) {
        setState(
          () => error =
              'Stop not acknowledged: $e Keep this window open and inspect the final receipt.',
        );
      }
    }
  }

  Widget evidence(Map value) => ExpansionTile(
    title: Text(
      '${value['status']} · ${value['improved'] == true ? 'Improved' : 'No qualifying improvement'}',
    ),
    subtitle: Text('${value['model']} · ${value['suite']}'),
    children: [
      for (final r in (value['results'] as List?) ?? [])
        ExpansionTile(
          title: Text(
            'Case ${(r['case'] as int) + 1} · ${r['candidate'] == true ? 'Candidate' : 'Baseline'} · ${r['passed'] == true ? 'Passed' : 'Did not pass'}',
          ),
          children: [
            SelectableText(const JsonEncoder.withIndent('  ').convert(r)),
          ],
        ),
    ],
  );
  @override
  Widget build(BuildContext context) => PopScope(
    canPop: !pending,
    child: InspectorFrame(
      title: 'Tool trials',
      subtitle: 'Fixed fixtures · isolated data · no activation',
      canClose: !pending,
      child: Column(
        children: [
          Expanded(
            child: ListView(
              padding: const EdgeInsets.all(20),
              children: [
                const Text(
                  'Compare a project skill on an original config change and an independent regression case. Only in-memory fixture files and simulated checks are available. No process, project data, network or credentials are accessible. All four cases use the same model and allowances: 5 model calls, 8 tools, 1024 output tokens per call and 30 seconds per case. A claimed success alone cannot pass.',
                ),
                if (widget.chat.workspaceRoot == null)
                  const Padding(
                    padding: EdgeInsets.all(16),
                    child: Text(
                      'Open a project or temporary working chat first.',
                    ),
                  ),
                if (report != null) ...[
                  const SizedBox(height: 12),
                  Text('Model: ${report!['model']} · ${report!['suite']}'),
                  if (skills.isEmpty)
                    const Text(
                      'Enable a project skill in Library before comparing. Global skills are excluded.',
                    ),
                  DropdownButtonFormField<String>(
                    initialValue: name,
                    decoration: const InputDecoration(
                      labelText: 'Baseline project skill',
                    ),
                    items: [
                      for (final s in skills)
                        DropdownMenuItem(
                          value: s['name'] as String,
                          child: Text(s['name'] as String),
                        ),
                    ],
                    onChanged: pending
                        ? null
                        : (value) => setState(() {
                            name = value;
                            candidate.text =
                                ((selected!['versions'] as List)
                                        .last['document']['text'])
                                    as String;
                          }),
                  ),
                  const SizedBox(height: 12),
                  TextField(
                    key: const Key('trial-candidate'),
                    controller: candidate,
                    enabled: !pending,
                    minLines: 5,
                    maxLines: 14,
                    decoration: const InputDecoration(
                      labelText: 'Candidate SKILL.md (8 KiB maximum)',
                    ),
                  ),
                ],
                if (error != null)
                  SelectableText(
                    error!,
                    style: TextStyle(
                      color: Theme.of(context).colorScheme.error,
                    ),
                  ),
                if (progress != null) Text(progress!),
                if (receipt != null) evidence(receipt!),
                for (final t in (report?['trials'] as List?) ?? [])
                  evidence(t as Map),
              ],
            ),
          ),
          Padding(
            padding: const EdgeInsets.all(12),
            child: Wrap(
              spacing: 8,
              children: [
                TextButton(
                  onPressed: pending ? null : load,
                  child: const Text('Refresh'),
                ),
                if (pending && run != null)
                  TextButton(
                    onPressed: stopping ? null : stop,
                    child: Text(stopping ? 'Stopping…' : 'Stop trials'),
                  )
                else
                  FilledButton(
                    onPressed: pending || selected == null ? null : start,
                    child: const Text('Compare with tools'),
                  ),
              ],
            ),
          ),
        ],
      ),
    ),
  );
}
