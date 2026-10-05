import 'dart:convert';

import 'package:flutter/material.dart';

import 'chat.dart';
import 'inspector.dart';
import 'memory.dart';
import 'settings_frame.dart';

class MemorySettings extends StatefulWidget {
  final ChatController chat;
  const MemorySettings({super.key, required this.chat});
  @override
  State<MemorySettings> createState() => _MemorySettingsState();
}

class _MemorySettingsState extends State<MemorySettings> {
  bool project = false;
  final locks = [ValueNotifier(false), ValueNotifier(false)];
  final panels = <int, Widget>{};
  final drafts = [SettingsDraft(), SettingsDraft()];
  ValueNotifier<bool>? outer;
  bool get busy => locks.any((l) => l.value);
  @override
  void initState() {
    super.initState();
    for (final lock in locks) {
      lock.addListener(changed);
    }
  }

  void changed() {
    outer?.value = busy;
    if (mounted) setState(() {});
  }

  @override
  void dispose() {
    for (final lock in locks) {
      lock.dispose();
    }
    super.dispose();
  }

  @override
  void didChangeDependencies() {
    super.didChangeDependencies();
    outer = SettingsEmbedding.of(context)?.pending;
  }

  @override
  Widget build(BuildContext context) {
    reportSettingsDraft(
      context,
      dirty: () => drafts.any((d) => d.dirty),
      save: () async {
        for (final d in drafts) {
          if (d.dirty && !await d.save!()) return false;
        }
        return true;
      },
    );
    return Column(
      children: [
        Padding(
          padding: const EdgeInsets.all(12),
          child: Wrap(
            spacing: 8,
            children: [
              ChoiceChip(
                label: const Text('My preferences'),
                selected: !project,
                onSelected: busy
                    ? null
                    : (_) => setState(() => project = false),
              ),
              ChoiceChip(
                label: const Text('Project facts'),
                selected: project,
                onSelected: busy ? null : (_) => setState(() => project = true),
              ),
            ],
          ),
        ),
        Expanded(
          child: Builder(
            builder: (context) {
              final index = project ? 1 : 0;
              panels.putIfAbsent(
                index,
                () => SettingsEmbedding(
                  draft: drafts[index],
                  pending: locks[index],
                  route: ModalRoute.of(context),
                  child: index == 1
                      ? KnowledgeInspector(chat: widget.chat)
                      : MemoryInspector(
                          bridge: widget.chat.bridge,
                          session: widget.chat.session,
                        ),
                ),
              );
              return IndexedStack(
                index: index,
                children: [
                  for (var i = 0; i < 2; i++)
                    panels[i] ?? const SizedBox.shrink(),
                ],
              );
            },
          ),
        ),
      ],
    );
  }
}

class KnowledgeInspector extends StatefulWidget {
  final ChatController chat;
  const KnowledgeInspector({super.key, required this.chat});
  @override
  State<KnowledgeInspector> createState() => _KnowledgeState();
}

class _KnowledgeState extends State<KnowledgeInspector> {
  String? baseline;
  String get draft =>
      jsonEncode([title.text, text.text, kind, enabled, inferred]);
  Future<bool> saveFact() async {
    await action('saveKnowledgeFact', {
      'revision': report!['revision'],
      'id': editing?['id'],
      'title': title.text,
      'text': text.text,
      'kind': kind,
      'enabled': enabled,
      'inferred': inferred,
    });
    if (mounted && error == null) setState(() => form = false);
    return !form;
  }

  Map? report, editing;
  bool pending = false, form = false, enabled = true, inferred = false;
  String kind = 'convention';
  String? error;
  final title = TextEditingController(), text = TextEditingController();
  bool get available =>
      widget.chat.session != null && widget.chat.workspaceRoot != null;
  @override
  void initState() {
    super.initState();
    if (available) load();
  }

  @override
  void dispose() {
    title.dispose();
    text.dispose();
    super.dispose();
  }

  Future<void> action(
    String command, [
    Map<String, dynamic> fields = const {},
  ]) async {
    if (pending) return;
    setState(() {
      pending = true;
      error = null;
    });
    try {
      final result = await widget.chat.bridge.call({
        'command': command,
        'session': widget.chat.session,
        ...fields,
      });
      if (mounted) setState(() => report = result as Map);
    } catch (e) {
      if (mounted) {
        setState(
          () => error =
              '$e Your edits remain. Refresh the revision, then save explicitly.',
        );
      }
    } finally {
      if (mounted) setState(() => pending = false);
    }
  }

  Future<void> load() => action('projectKnowledge');
  void edit(Map? fact) {
    setState(() {
      editing = fact;
      form = true;
      title.text = fact?['title'] ?? '';
      text.text = fact?['text'] ?? '';
      kind = fact?['kind'] ?? 'convention';
      enabled = fact?['enabled'] ?? true;
      inferred = fact?['basis'] == 'inferred';
    });
    baseline = draft;
  }

  @override
  Widget build(BuildContext context) {
    reportSettingsDraft(
      context,
      dirty: () => form && baseline != draft,
      save: saveFact,
    );
    return InspectorFrame(
      title: 'Project facts',
      subtitle: 'This working folder · evidence and corrections',
      canClose: !pending,
      child: Column(
        children: [
          if (pending) const LinearProgressIndicator(minHeight: 2),
          Expanded(
            child: ListView(
              padding: const EdgeInsets.all(20),
              children: [
                if (!available)
                  const Text(
                    'Project facts belong to a working folder. Side chats use My preferences.',
                  ),
                if (!available && widget.chat.workspaceKind != 'side')
                  TextButton.icon(
                    icon: const Icon(Icons.create_new_folder_outlined),
                    label: const Text('Start working session'),
                    onPressed: pending ? null : () async {
                      try {
                        await widget.chat.prepareWindowSharing();
                        if (mounted && available) await load();
                      } catch (e) {
                        if (mounted) setState(() => error = '$e');
                      }
                    },
                  ),
                if (error != null) SelectableText(error!),
                if (report != null) ...[
                  SwitchListTile(
                    contentPadding: EdgeInsets.zero,
                    title: const Text('Learn from approved task evidence'),
                    subtitle: const Text(
                      'Reuse brief verified observations in this project.',
                    ),
                    value: report!['learning'] == true,
                    onChanged: pending
                        ? null
                        : (v) => action('setKnowledgePolicy', {
                            'revision': report!['revision'],
                            'learning': v,
                            'share_feedback': report!['shareFeedback'],
                          }),
                  ),
                  SwitchListTile(
                    contentPadding: EdgeInsets.zero,
                    title: const Text('Allow feedback notes for learning'),
                    subtitle: const Text(
                      'Send eligible feedback notes to your model during reflection. Reflection stays a separate choice.',
                    ),
                    value: report!['shareFeedback'] == true,
                    onChanged: pending
                        ? null
                        : (v) => action('setKnowledgePolicy', {
                            'revision': report!['revision'],
                            'learning': report!['learning'],
                            'share_feedback': v,
                          }),
                  ),
                  ExpansionTile(
                    title: const Text('Learning details'),
                    children: [
                      const Text(
                        'File observations become stale when their source changes. Other observations expire after seven days. A declared script is not a tested command. Corrections are protected; knowledge never grants tool access.',
                      ),
                    ],
                  ),
                  if ('${report!['notice'] ?? ''}'.isNotEmpty)
                    Padding(
                      padding: const EdgeInsets.symmetric(vertical: 12),
                      child: SelectableText('${report!['notice']}'),
                    ),
                  if (form) ...[
                    TextField(
                      controller: title,
                      enabled: !pending,
                      decoration: const InputDecoration(labelText: 'Title'),
                    ),
                    const SizedBox(height: 12),
                    TextField(
                      controller: text,
                      enabled: !pending,
                      minLines: 2,
                      maxLines: 5,
                      decoration: const InputDecoration(
                        labelText: 'Fact or correction (512 bytes maximum)',
                      ),
                    ),
                    DropdownButtonFormField<String>(
                      initialValue: kind,
                      isExpanded: true,
                      items: [
                        for (final k in ['command', 'structure', 'convention'])
                          DropdownMenuItem(value: k, child: Text(k)),
                      ],
                      onChanged: pending
                          ? null
                          : (v) => setState(() => kind = v!),
                    ),
                    CheckboxListTile(
                      title: const Text(
                        'Inference — verify before relying on it',
                      ),
                      value: inferred,
                      onChanged: pending
                          ? null
                          : (v) => setState(() => inferred = v!),
                    ),
                    CheckboxListTile(
                      title: const Text('Include in future context'),
                      value: enabled,
                      onChanged: pending
                          ? null
                          : (v) => setState(() => enabled = v!),
                    ),
                  ] else
                    for (final fact in report!['facts'] as List) ...[
                      const Divider(),
                      ListTile(
                        contentPadding: EdgeInsets.zero,
                        title: Text('${fact['title']}'),
                        subtitle: Text(
                          '${fact['basis']} · ${fact['fresh'] == true ? 'Current' : 'Stale — excluded'} · ${fact['enabled'] == true ? 'Enabled' : 'Disabled'}${fact['protected'] == true ? ' · Protected correction' : ''}',
                        ),
                        trailing: TextButton(
                          onPressed: pending ? null : () => edit(fact as Map),
                          child: const Text('Edit'),
                        ),
                      ),
                      SelectableText('${fact['text']}'),
                      if (fact['source'] != null)
                        ExpansionTile(
                          title: const Text('Source evidence'),
                          children: [SelectableText('${fact['source']}')],
                        ),
                    ],
                ],
              ],
            ),
          ),
          Padding(
            padding: const EdgeInsets.all(12),
            child: Wrap(
              spacing: 8,
              children: [
                TextButton(
                  onPressed: pending || !available ? null : load,
                  child: const Text('Refresh'),
                ),
                if (report != null && !form)
                  TextButton(
                    onPressed: pending ? null : () => edit(null),
                    child: const Text('New fact'),
                  ),
                if (form) ...[
                  TextButton(
                    onPressed: pending
                        ? null
                        : () => setState(() => form = false),
                    child: const Text('Cancel'),
                  ),
                  FilledButton(
                    onPressed: pending ? null : saveFact,
                    child: const Text('Save correction'),
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
