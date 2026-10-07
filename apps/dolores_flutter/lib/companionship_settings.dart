import 'dart:convert';

import 'package:flutter/material.dart';

import 'chat.dart';
import 'settings_frame.dart';
import 'settings_help.dart';

class CompanionshipSettings extends StatefulWidget {
  final ChatController chat;
  const CompanionshipSettings({super.key, required this.chat});
  @override
  State<CompanionshipSettings> createState() => _CompanionshipSettingsState();
}

class _CompanionshipSettingsState extends State<CompanionshipSettings> {
  Map? report;
  Map<String, dynamic>? policy;
  String? saved, error;
  bool pending = false;
  final zone = TextEditingController();
  String get draft => jsonEncode({...?policy, 'zone': zone.text.trim()});
  @override
  void initState() {
    super.initState();
    load();
  }

  @override
  void dispose() {
    zone.dispose();
    super.dispose();
  }

  void accept(Map value) {
    report = value;
    policy = Map<String, dynamic>.from(
      (value['state'] as Map)['policy'] as Map,
    );
    zone.text = policy!['zone'] as String;
    saved = draft;
    error = null;
  }

  Future<void> load() async {
    setState(() => pending = true);
    try {
      final v =
          await widget.chat.bridge.call({'command': 'companionState'}) as Map;
      if (mounted) setState(() => accept(v));
    } catch (e) {
      if (mounted) setState(() => error = '$e');
    } finally {
      if (mounted) setState(() => pending = false);
    }
  }

  Future<bool> save() async {
    setState(() => pending = true);
    try {
      final v = await widget.chat.bridge.call({
        'command': 'companionPolicy',
        'revision': report!['state']['revision'],
        'policy': {...policy!, 'zone': zone.text.trim()},
      }) as Map;
      if (mounted) setState(() => accept(v));
      return true;
    } catch (e) {
      if (mounted) setState(() => error = '$e');
      return false;
    } finally {
      if (mounted) setState(() => pending = false);
    }
  }

  String hour(int v) =>
      '${(v ~/ 60).toString().padLeft(2, '0')}:${(v % 60).toString().padLeft(2, '0')}';
  Future<void> hours(String key) async {
    final v = policy![key] as int;
    final time = await showTimePicker(
      context: context,
      initialTime: TimeOfDay(hour: v ~/ 60, minute: v % 60),
      builder: (c, w) => MediaQuery(
        data: MediaQuery.of(c).copyWith(alwaysUse24HourFormat: true),
        child: w!,
      ),
    );
    if (time != null && mounted) {
      setState(() => policy![key] = time.hour * 60 + time.minute);
    }
  }

  @override
  Widget build(BuildContext context) {
    reportSettingsDraft(
      context,
      dirty: () => saved != null && saved != draft,
      save: () => pending ? Future.value(false) : save(),
    );
    final models = (report?['models'] as List? ?? []).cast<String>();
    return EmbeddedSettingsFrame(
      title: 'Companionship',
      pending: SettingsEmbedding.of(context)!.pending,
      canClose: !pending,
      child: Column(
        children: [
          if (error != null)
            Padding(
              padding: const EdgeInsets.symmetric(horizontal: 20, vertical: 8),
              child: Text(error!),
            ),
          Expanded(
            child: ListView(
              key: const Key('companion-form'),
              padding: const EdgeInsets.all(20),
              children: [
                if (pending) const LinearProgressIndicator(),
                if (policy != null) ...[
                  SwitchListTile(
                    contentPadding: EdgeInsets.zero,
                    key: const Key('companion-enabled'),
                    title: const SettingsHelpLabel(
                      label: 'Companionship',
                      help: 'Occasional in-app notes during your chosen hours. Uses the selected model and may quote current memories when Memory is on. Turning off stops new notes.',
                    ),
                    subtitle: Text(
                      report!['available'] == true
                          ? 'Occasional notes from Dolores'
                          : 'Windows only for now',
                    ),
                    value: policy!['enabled'] as bool,
                    onChanged: pending || report!['available'] != true
                        ? null
                        : (v) => setState(() => policy!['enabled'] = v),
                  ),
                  const SizedBox(height: 16),
                  DropdownButtonFormField<String>(
                    key: const Key('companion-model'),
                    initialValue: models.contains(policy!['model'])
                        ? policy!['model'] as String
                        : null,
                    isExpanded: true,
                    decoration: const InputDecoration(labelText: 'Model'),
                    hint: const Text('Choose a model'),
                    items: [
                      for (final m in models)
                        DropdownMenuItem(
                          value: m,
                          child: Text(m, overflow: TextOverflow.ellipsis),
                        ),
                    ],
                    onChanged: pending
                        ? null
                        : (v) => setState(() => policy!['model'] = v ?? ''),
                  ),
                  const SizedBox(height: 16),
                  Wrap(
                    spacing: 12,
                    runSpacing: 8,
                    children: [
                      OutlinedButton(
                        onPressed: pending ? null : () => hours('start'),
                        child: Text('From ${hour(policy!['start'] as int)}'),
                      ),
                      OutlinedButton(
                        onPressed: pending ? null : () => hours('end'),
                        child: Text('Until ${hour(policy!['end'] as int)}'),
                      ),
                    ],
                  ),
                  const SizedBox(height: 16),
                  DropdownButtonFormField<int>(
                    initialValue: policy!['dailyCap'] as int,
                    decoration: const InputDecoration(labelText: 'Daily limit'),
                    items: [
                      for (final n in [0, 1, 2])
                        DropdownMenuItem(
                          value: n,
                          child: Text(n == 0 ? 'Quiet' : 'Up to $n'),
                        ),
                    ],
                    onChanged: pending
                        ? null
                        : (v) => setState(() => policy!['dailyCap'] = v),
                  ),
                  SettingsDetails(
                    title: 'Details',
                    children: [
                      TextField(
                        controller: zone,
                        enabled: !pending,
                        decoration: const InputDecoration(
                          labelText: 'Timezone',
                        ),
                      ),
                      const SizedBox(height: 12),
                      const SettingsHelpLabel(
                        label: 'Timing',
                        help: 'At least three hours between attempts. Dolores waits while you work or are away. Only one unread note is kept; failures are not retried.',
                      ),
                      const SizedBox(height: 12),
                      for (final a
                          in (report!['state']['activity'] as List).reversed)
                        ListTile(
                          contentPadding: EdgeInsets.zero,
                          title: Text(
                            a['kind'] == 'openWork'
                                ? 'Work reminder'
                                : a['kind'] == 'memory'
                                ? 'Recollection'
                                : a['kind'] == 'fact'
                                ? 'Fun fact'
                                : 'Chat',
                          ),
                          subtitle: Text(
                            '${a['status']}${a['note'] == null ? '' : ' · ${a['note']}'}',
                          ),
                          onTap: () => showDialog<void>(
                            context: context,
                            builder: (c) => AlertDialog(
                              title: const Text('Note details'),
                              content: SingleChildScrollView(
                                child: SelectableText(
                                  [
                                    if (a['source'] != null)
                                      '${a['source']['quote']}\n\nSource: ${a['source']['session']} · message ${a['source']['messageId']}',
                                    if (a['citation'] != null) a['citation'],
                                    a['usage'] == null
                                        ? 'Usage unavailable'
                                        : 'Usage: ${a['usage']}',
                                  ].join('\n\n'),
                                ),
                              ),
                              actions: [
                                TextButton(
                                  onPressed: () => Navigator.pop(c),
                                  child: const Text('Close'),
                                ),
                              ],
                            ),
                          ),
                        ),
                    ],
                  ),
                  Wrap(
                    spacing: 8,
                    children: [
                      FilledButton(
                        key: const Key('save-companion'),
                        onPressed: pending ? null : save,
                        child: const Text('Save'),
                      ),
                      TextButton(
                        onPressed: pending ? null : load,
                        child: const Text('Refresh'),
                      ),
                    ],
                  ),
                ],
              ],
            ),
          ),
        ],
      ),
    );
  }
}
