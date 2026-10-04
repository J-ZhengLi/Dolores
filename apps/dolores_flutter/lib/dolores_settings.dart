import 'package:flutter/material.dart';

import 'chat.dart';
import 'inspector.dart';

Future<void> showDoloresSettings(BuildContext context, ChatController chat) =>
    showDialog<void>(
      context: context,
      builder: (_) => DoloresSettingsInspector(chat: chat),
    );

class DoloresSettingsInspector extends StatefulWidget {
  final ChatController chat;
  const DoloresSettingsInspector({super.key, required this.chat});
  @override
  State<DoloresSettingsInspector> createState() =>
      _DoloresSettingsInspectorState();
}

class _DoloresSettingsInspectorState extends State<DoloresSettingsInspector> {
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
    scope = session == null ? 'user' : 'thread';
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
                {'modelCalls': 4, 'toolCalls': 4, 'segments': 4})
            as Map;
    calls.text = t['modelCalls'].toString();
    tools.text = t['toolCalls'].toString();
    segments.text = t['segments'].toString();
    elapsed.text = t['elapsedSeconds']?.toString() ?? '';
    generation = patch['generation'] != null;
    interaction = patch['interaction'] != null;
    final g = (patch['generation'] ?? effective['request']) as Map,
        i = (patch['interaction'] ?? effective['interaction']) as Map;
    output.text = g['maxOutputTokens'].toString();
    timeout.text = g['timeoutSeconds'].toString();
    discussion = i['discussion'] as String;
    question = i['questionAssumptions'] as bool;
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
        task &&
        (int.tryParse(calls.text) == null ||
            int.tryParse(tools.text) == null ||
            int.tryParse(segments.text) == null ||
            (elapsed.text.trim().isNotEmpty &&
                int.tryParse(elapsed.text) == null))) {
      setState(() {
        error = 'Use whole numbers for task limits.';
      });
      return;
    }
    if (!reset && generation && (maxTokens == null || seconds == null)) {
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
          'task': !reset && task
              ? {
                  'modelCalls': int.parse(calls.text),
                  'toolCalls': int.parse(tools.text),
                  'segments': int.parse(segments.text),
                  'elapsedSeconds': int.tryParse(elapsed.text),
                }
              : null,
          'generation': !reset && generation && scope != 'user'
              ? {'maxOutputTokens': maxTokens, 'timeoutSeconds': seconds}
              : null,
          'interaction': !reset && interaction
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
    'user' => 'User defaults',
    'project' => 'This project',
    _ => 'This chat',
  };
  @override
  Widget build(BuildContext context) => AnimatedBuilder(
    animation: widget.chat,
    builder: (context, _) {
      final data = report, effective = data?['effective'] as Map?;
      final locked = pending || widget.chat.busy || widget.chat.changing;
      return PopScope(
        canPop: !pending,
        child: InspectorFrame(
          title: 'Dolores settings',
          subtitle: 'Scoped overrides · applies to future runs',
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
                              : (value) {
                                  setState(() {
                                    scope = value!;
                                    error = null;
                                    notice = null;
                                    fill();
                                  });
                                },
                        ),
                        const SizedBox(height: 12),
                        SelectableText(
                          'Effective output: ${effective!['request']['maxOutputTokens']} tokens · ${effective['requestOrigin']}\nTimeout: ${effective['request']['timeoutSeconds']} seconds\nContext: ${effective['contextWindowTokens']} tokens · ${effective['contextOrigin']}\nInteraction: ${effective['interactionOrigin']}',
                        ),
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
                        if (task) ...[
                          for (final field in [
                            (calls, 'Model calls per segment (2–16)'),
                            (tools, 'Tool operations per segment (1–32)'),
                            (segments, 'Total task segments (1–8)'),
                            (
                              elapsed,
                              'Task deadline seconds (blank inherits request timeout)',
                            ),
                          ])
                            TextField(
                              controller: field.$1,
                              enabled: !locked,
                              keyboardType: TextInputType.number,
                              decoration: InputDecoration(labelText: field.$2),
                            ),
                          const Text(
                            'Continue uses another segment. Applied work remains. Limits do not grant tool access or change model output/context settings.',
                          ),
                        ],
                        if (scope == 'user')
                          const Padding(
                            padding: EdgeInsets.symmetric(vertical: 12),
                            child: Text(
                              'Edit user/model generation defaults in Request settings. Model context windows remain in Model connection.',
                            ),
                          ),
                        if (scope != 'user') ...[
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
                                labelText: 'Maximum output tokens',
                              ),
                            ),
                            TextField(
                              key: const Key('scoped-timeout'),
                              controller: timeout,
                              enabled: !locked,
                              keyboardType: TextInputType.number,
                              decoration: const InputDecoration(
                                labelText: 'Request timeout (seconds)',
                              ),
                            ),
                          ],
                        ],
                        CheckboxListTile(
                          contentPadding: EdgeInsets.zero,
                          title: const Text(
                            'Override interaction for this scope',
                          ),
                          value: interaction,
                          onChanged: locked
                              ? null
                              : (value) => setState(() => interaction = value!),
                        ),
                        if (interaction) ...[
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
                        const Text(
                          'Dolores stays calm and candid, uses available evidence, and asks only useful questions. These controls cannot guarantee model behavior.',
                        ),
                        const SizedBox(height: 12),
                        SelectableText(
                          '${data['taskAccess']}\n${data['adaptation']}',
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
