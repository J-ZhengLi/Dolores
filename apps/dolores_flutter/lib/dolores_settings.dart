import 'package:flutter/material.dart';

import 'chat.dart';
import 'inspector.dart';
import 'settings_frame.dart';

import 'dart:convert';

Future<void> showDoloresSettings(BuildContext context, ChatController chat) =>
    showDialog<void>(
      context: context,
      builder: (_) => DoloresSettingsInspector(chat: chat),
    );

enum SettingsGroup { all, personalization, generation, task }

class DoloresSettingsInspector extends StatefulWidget {
  final ChatController chat;
  final SettingsGroup group;
  const DoloresSettingsInspector({
    super.key,
    required this.chat,
    this.group = SettingsGroup.all,
  });
  @override
  State<DoloresSettingsInspector> createState() =>
      _DoloresSettingsInspectorState();
}

class _DoloresSettingsInspectorState extends State<DoloresSettingsInspector> {
  String? savedDraft;
  String get draftValue => jsonEncode([
    scope,
    task,
    generation,
    interaction,
    discussion,
    question,
    calls.text,
    tools.text,
    segments.text,
    elapsed.text,
    output.text,
    timeout.text,
  ]);
  Future<bool> saveDraft() async {
    if (pending || widget.chat.busy || report == null) return false;
    await save();
    return draftValue == savedDraft;
  }

  bool shows(SettingsGroup group) =>
      widget.group == SettingsGroup.all || widget.group == group;
  late final String? session = widget.chat.session;
  final output = TextEditingController(), timeout = TextEditingController();
  final calls = TextEditingController(),
      tools = TextEditingController(),
      segments = TextEditingController(),
      elapsed = TextEditingController();
  bool task = false;
  final scroll = ScrollController();
  Map<String, dynamic>? report;
  String scope = 'user', discussion = 'discuss';
  String? error, notice;
  bool pending = false,
      generation = false,
      interaction = false,
      question = true;
  @override
  void initState() {
    super.initState();
    scope = widget.group == SettingsGroup.personalization || session == null
        ? 'user'
        : 'thread';
    load();
  }

  @override
  void dispose() {
    calls.dispose();
    tools.dispose();
    segments.dispose();
    elapsed.dispose();
    output.dispose();
    timeout.dispose();
    scroll.dispose();
    super.dispose();
  }

  Map<String, dynamic> get record => Map<String, dynamic>.from(
    (report!['scopes'] as List).firstWhere((r) => r['scope'] == scope)['record']
        as Map,
  );
  void fill() {
    final effective = report!['effective'] as Map,
        patch = record['patch'] as Map;
    task = patch['task'] != null;
    final t =
        (patch['task'] ??
                effective['task'] ??
                {'modelCalls': null, 'toolCalls': null, 'segments': 4})
            as Map;
    calls.text = t['modelCalls']?.toString() ?? '';
    tools.text = t['toolCalls']?.toString() ?? '';
    segments.text = t['segments'].toString();
    elapsed.text = t['elapsedSeconds']?.toString() ?? '';
    generation = patch['generation'] != null;
    interaction =
        patch['interaction'] != null ||
        (widget.group == SettingsGroup.personalization && scope == 'user');
    Map style = {'discussion': 'discuss', 'questionAssumptions': true};
    for (final item in report!['scopes'] as List) {
      final value = item['record']['patch']['interaction'];
      if (value is Map) style = value;
      if (item['scope'] == scope) break;
    }
    final g = (patch['generation'] ?? effective['request']) as Map, i = style;
    output.text = g['maxOutputTokens']?.toString() ?? '';
    timeout.text = g['timeoutSeconds'].toString();
    discussion = i['discussion'] as String;
    question = i['questionAssumptions'] as bool;
    savedDraft = draftValue;
  }

  Future<void> load({bool keepDraft = false}) async {
    setState(() {
      pending = true;
      error = null;
      notice = null;
    });
    try {
      final result = await widget.chat.bridge.call({
        'command': 'scopedSettings',
        'session': session,
      });
      if (mounted) {
        setState(() {
          report = Map<String, dynamic>.from(result as Map);
          if (!keepDraft) fill();
        });
      }
    } catch (failure) {
      if (mounted) {
        setState(() {
          error = failure.toString();
        });
      }
    } finally {
      if (mounted) {
        setState(() {
          pending = false;
        });
      }
    }
  }

  Future<void> save({bool reset = false}) async {
    final maxTokens = int.tryParse(output.text),
        seconds = int.tryParse(timeout.text);
    if (!reset &&
        shows(SettingsGroup.task) &&
        task &&
        ((calls.text.trim().isNotEmpty && int.tryParse(calls.text) == null) ||
            (tools.text.trim().isNotEmpty &&
                int.tryParse(tools.text) == null) ||
            int.tryParse(segments.text) == null ||
            (elapsed.text.trim().isNotEmpty &&
                int.tryParse(elapsed.text) == null))) {
      setState(() {
        error = 'Use whole numbers for task limits.';
      });
      return;
    }
    if (!reset &&
        shows(SettingsGroup.generation) &&
        generation &&
        ((output.text.trim().isNotEmpty && maxTokens == null) ||
            seconds == null)) {
      setState(() {
        error = 'Use whole numbers for output tokens and timeout.';
      });
      return;
    }
    setState(() {
      pending = true;
      error = null;
      notice = null;
    });
    try {
      final result = await widget.chat.bridge.call({
        'command': 'saveScopedSettings',
        'session': session,
        'scope': scope,
        'revision': record['revision'],
        'patch': {
          'permissions': record['patch']['permissions'],
          'task': !shows(SettingsGroup.task)
              ? record['patch']['task']
              : !reset && task
              ? {
                  'modelCalls': int.tryParse(calls.text.trim()),
                  'toolCalls': int.tryParse(tools.text.trim()),
                  'segments': int.parse(segments.text),
                  'elapsedSeconds': int.tryParse(elapsed.text),
                }
              : null,
          'generation': !shows(SettingsGroup.generation)
              ? record['patch']['generation']
              : !reset && generation && scope != 'user'
              ? {'maxOutputTokens': maxTokens, 'timeoutSeconds': seconds}
              : null,
          'interaction': !shows(SettingsGroup.personalization)
              ? record['patch']['interaction']
              : !reset && interaction
              ? {'discussion': discussion, 'questionAssumptions': question}
              : null,
        },
      });
      if (mounted) {
        setState(() {
          report = Map<String, dynamic>.from(result as Map);
          fill();
          notice = reset
              ? 'Inherited settings restored for this scope.'
              : 'Settings saved for future runs.';
        });
        widget.chat.invalidateContext();
        if (scroll.hasClients) scroll.jumpTo(0);
      }
    } catch (failure) {
      if (mounted) {
        setState(() {
          error = failure.toString();
        });
        if (scroll.hasClients) scroll.jumpTo(0);
      }
    } finally {
      if (mounted) {
        setState(() {
          pending = false;
        });
      }
    }
  }

  String label(String value) => switch (value) {
    'user' => 'All chats',
    'project' => 'This project',
    _ => 'This chat',
  };
  @override
  Widget build(BuildContext context) => AnimatedBuilder(
    animation: widget.chat,
    builder: (context, _) {
      final data = report, effective = data?['effective'] as Map?;
      final locked = pending || widget.chat.busy || widget.chat.changing;
      reportSettingsDraft(
        context,
        dirty: () => savedDraft != null && draftValue != savedDraft,
        save: saveDraft,
      );
      return PopScope(
        canPop: !pending,
        child: InspectorFrame(
          title: switch (widget.group) {
            SettingsGroup.generation => 'Scope overrides',
            SettingsGroup.task => 'Task limits',
            _ => 'Personalization',
          },
          subtitle: widget.group == SettingsGroup.personalization
              ? 'How Dolores works with you'
              : 'Applies to the next task',
          canClose: !pending,
          child: Column(
            children: [
              Expanded(
                child: SingleChildScrollView(
                  controller: scroll,
                  padding: const EdgeInsets.all(20),
                  child: Column(
                    crossAxisAlignment: CrossAxisAlignment.start,
                    children: [
                      if (error != null) SelectableText(error!),
                      if (notice != null) Text(notice!),
                      if (data != null) ...[
                        if (data['validationError'] != null)
                          SelectableText(
                            '${data['validationError']} Reduce the override, use inherited settings, or correct the model context window before sending.',
                          ),
                        DropdownButtonFormField<String>(
                          key: const Key('settings-scope'),
                          initialValue: scope,
                          decoration: const InputDecoration(
                            labelText: 'Apply to',
                          ),
                          items: [
                            for (final item in data['scopes'] as List)
                              DropdownMenuItem(
                                value: item['scope'] as String,
                                child: Text(label(item['scope'] as String)),
                              ),
                          ],
                          onChanged: locked
                              ? null
                              : (value) async {
                                  if (savedDraft != null &&
                                      draftValue != savedDraft &&
                                      !await resolveSettingsDraft(
                                        context,
                                        save: saveDraft,
                                      )) {
                                    return;
                                  }
                                  if (!mounted) return;
                                  setState(() {
                                    scope = value!;
                                    error = null;
                                    notice = null;
                                    fill();
                                  });
                                },
                        ),
                        const SizedBox(height: 12),
                        if (widget.group == SettingsGroup.personalization)
                          SelectableText(
                            scope == 'user'
                                ? 'Default for all chats'
                                : 'Custom style for ${label(scope).toLowerCase()}',
                          )
                        else if (widget.group == SettingsGroup.task)
                          SelectableText(
                            '${effective!['task']?['modelCalls'] == null ? 'Automatic model calls' : '${effective['task']['modelCalls']} model calls'} · ${effective['task']?['toolCalls'] == null ? 'Automatic tools' : '${effective['task']['toolCalls']} tools'}',
                          )
                        else
                          SettingsDetails(
                            title: 'Effective settings',
                            children: [
                              SelectableText(
                                'Effective output: ${effective!['request']['maxOutputTokens'] ?? 'Provider default'} · ${effective['requestOrigin']}\nTimeout: ${effective['request']['timeoutSeconds']} seconds\nContext: ${effective['contextWindowTokens']} tokens · ${effective['contextOrigin']}\nInteraction: ${effective['interactionOrigin']}',
                              ),
                            ],
                          ),
                        if (shows(SettingsGroup.task))
                          CheckboxListTile(
                            contentPadding: EdgeInsets.zero,
                            title: const Text(
                              'Override task limits for this scope',
                            ),
                            value: task,
                            onChanged: locked
                                ? null
                                : (value) => setState(() => task = value!),
                          ),
                        if (shows(SettingsGroup.task) && task) ...[
                          for (final field in [
                            (calls, 'Model calls (optional)'),
                            (tools, 'Tool operations (optional)'),
                            (segments, 'Total task segments (1–8)'),
                            (
                              elapsed,
                              'Task time limit seconds (blank = no limit)',
                            ),
                          ])
                            TextField(
                              controller: field.$1,
                              enabled: !locked,
                              keyboardType: TextInputType.number,
                              decoration: InputDecoration(
                                labelText: field.$2,
                                hintText: field.$1 == calls || field.$1 == tools
                                    ? 'Automatic'
                                    : null,
                              ),
                            ),
                          const Text(
                            'Leave call limits blank for Automatic. Continue resumes saved progress.',
                          ),
                        ],
                        if (shows(SettingsGroup.generation) && scope == 'user')
                          const Padding(
                            padding: EdgeInsets.symmetric(vertical: 12),
                            child: Text(
                              'Change response defaults in Advanced. Change context size in Models.',
                            ),
                          ),
                        if (shows(SettingsGroup.generation) &&
                            scope != 'user') ...[
                          CheckboxListTile(
                            contentPadding: EdgeInsets.zero,
                            title: const Text(
                              'Override generation for this scope',
                            ),
                            value: generation,
                            onChanged: locked
                                ? null
                                : (value) =>
                                      setState(() => generation = value!),
                          ),
                          if (generation) ...[
                            TextField(
                              key: const Key('scoped-output'),
                              controller: output,
                              enabled: !locked,
                              keyboardType: TextInputType.number,
                              decoration: const InputDecoration(
                                labelText: 'Output tokens (optional)',
                                hintText: 'Provider default',
                              ),
                            ),
                            TextField(
                              key: const Key('scoped-timeout'),
                              controller: timeout,
                              enabled: !locked,
                              keyboardType: TextInputType.number,
                              decoration: const InputDecoration(
                                labelText: 'Stall timeout (seconds)',
                              ),
                            ),
                          ],
                        ],
                        if (shows(SettingsGroup.personalization) &&
                            scope != 'user')
                          CheckboxListTile(
                            contentPadding: EdgeInsets.zero,
                            title: const Text(
                              'Override interaction for this scope',
                            ),
                            value: interaction,
                            onChanged: locked
                                ? null
                                : (value) =>
                                      setState(() => interaction = value!),
                          ),
                        if (shows(SettingsGroup.personalization) &&
                            interaction) ...[
                          DropdownButtonFormField<String>(
                            key: ValueKey('discussion-$scope-$discussion'),
                            initialValue: discussion,
                            decoration: const InputDecoration(
                              labelText: 'Explanation style',
                            ),
                            items: const [
                              DropdownMenuItem(
                                value: 'discuss',
                                child: Text('Discuss before substantial work'),
                              ),
                              DropdownMenuItem(
                                value: 'brief',
                                child: Text('Keep explanations brief'),
                              ),
                            ],
                            onChanged: locked
                                ? null
                                : (value) =>
                                      setState(() => discussion = value!),
                          ),
                          CheckboxListTile(
                            contentPadding: EdgeInsets.zero,
                            title: const Text('Check important assumptions'),
                            value: question,
                            onChanged: locked
                                ? null
                                : (value) => setState(() => question = value!),
                          ),
                        ],
                        if (widget.group != SettingsGroup.personalization)
                          ExpansionTile(
                            title: const Text('Details'),
                            children: [
                              SelectableText(
                                '${data['taskAccess']}\n${data['adaptation']}',
                              ),
                            ],
                          ),
                        if (widget.chat.busy)
                          const Text(
                            'The current run keeps its snapshot. Save after it finishes or is stopped.',
                          ),
                      ],
                      if (pending) const LinearProgressIndicator(),
                    ],
                  ),
                ),
              ),
              Padding(
                padding: const EdgeInsets.all(12),
                child: Wrap(
                  spacing: 8,
                  runSpacing: 8,
                  children: [
                    TextButton(
                      onPressed: pending
                          ? null
                          : () => load(keepDraft: report != null),
                      child: const Text('Refresh'),
                    ),
                    TextButton(
                      onPressed: locked || data == null
                          ? null
                          : () => save(reset: true),
                      child: const Text('Use inherited settings'),
                    ),
                    FilledButton(
                      onPressed: locked || data == null ? null : save,
                      child: const Text('Save'),
                    ),
                  ],
                ),
              ),
            ],
          ),
        ),
      );
    },
  );
}
