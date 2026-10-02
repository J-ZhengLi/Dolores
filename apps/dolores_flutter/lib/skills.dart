import 'package:flutter/material.dart';

import 'bridge.dart';
import 'chat.dart';
import 'inspector.dart';
import 'theme.dart';

Future<void> showSkills(BuildContext context, ChatController chat) =>
    chat.inspectLocalChanges(() async {
      await showDialog<void>(
        context: context,
        barrierDismissible: false,
        builder: (_) =>
            SkillsInspector(bridge: chat.bridge, session: chat.session!),
      );
      chat.invalidateContext();
    });

class SkillsInspector extends StatefulWidget {
  final ChatBridge bridge;
  final String session;
  const SkillsInspector({
    super.key,
    required this.bridge,
    required this.session,
  });
  @override
  State<SkillsInspector> createState() => _SkillsInspectorState();
}

class _SkillsInspectorState extends State<SkillsInspector> {
  Map<String, dynamic>? catalog, review;
  String? error, notice;
  bool busy = false;
  final _scroll = ScrollController();
  Future<dynamic> _call(
    String command, [
    Map<String, dynamic> args = const {},
  ]) => widget.bridge.call({
    'command': command,
    'session': widget.session,
    ...args,
  });
  @override
  void initState() {
    super.initState();
    _list();
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
      if (mounted) setState(() => busy = false);
    }
  }

  Future<void> _loadList() async {
    if (mounted) setState(() => review = null);
    final result = await _call('projectSkills');
    if (mounted) {
      setState(() => catalog = (result as Map).cast<String, dynamic>());
      if (_scroll.hasClients) _scroll.jumpTo(0);
    }
  }

  Future<void> _list() => _act(_loadList);
  Future<void> _loadReview(String name, int? version) async {
    if (review != null && mounted) {
      setState(() => review = {...review!, 'token': null});
    }
    final result = await _call('reviewSkill', {
      'name': name,
      'version': version,
    });
    if (mounted) {
      setState(() => review = (result as Map).cast<String, dynamic>());
      if (_scroll.hasClients) _scroll.jumpTo(0);
    }
  }

  Future<void> _select(String name, [int? version]) =>
      _act(() => _loadReview(name, version));
  Future<void> _activate() => _act(() async {
    final token = review?['token'];
    if (token == null) return;
    setState(() => review = {...review!, 'token': null});
    final result = await _call('activateSkill', {'token': token}) as Map;
    notice =
        'Skill activated · version ${(result['versions'] as List).last['version']}.';
    await _loadReview(
      result['name'] as String,
      (result['versions'] as List).last['version'] as int,
    );
  });
  Future<void> _mutate(bool forget) => _act(() async {
    final name = review!['document']['name'];
    await _call(forget ? 'forgetSkill' : 'disableSkill', {
      'name': name,
      'revision': review!['revision'],
    });
    notice = forget
        ? 'Saved versions forgotten. The project file is unchanged.'
        : 'Skill disabled. Saved versions remain available.';
    await _loadList();
  });
  @override
  void dispose() {
    _scroll.dispose();
    final token = review?['token'];
    if (token != null) {
      _call('cancelSkillReview', {'token': token}).catchError((_) => null);
    }
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    final p = Palette(Theme.of(context).brightness == Brightness.dark);
    final document = review?['document'] as Map?;
    final items = catalog?['items'] as List? ?? [];
    return PopScope(
      canPop: !busy,
      child: InspectorFrame(
        title: 'Project skills',
        subtitle: '.agents/skills · reviewed instructions for this folder',
        canClose: !busy,
        child: Column(
          children: [
            if (busy) const LinearProgressIndicator(minHeight: 2),
            Expanded(
              child: ListView(
                key: const Key('skills-scroll'),
                controller: _scroll,
                padding: const EdgeInsets.all(20),
                children: [
                  const Text(
                    'Choose a skill, review its text, then activate it for chats in this folder. Activation shares the text with your model and saves it locally. Each tool still needs your approval.',
                  ),
                  const SizedBox(height: 8),
                  Text(
                    'Active skills use saved versions. File changes wait for a new review. References and scripts are not loaded or run automatically. Up to 3 active skills, 8 KiB combined; 5 recent versions per skill.',
                    style: TextStyle(color: p.muted, fontSize: 12),
                  ),
                  if (notice != null)
                    Padding(
                      padding: const EdgeInsets.only(top: 12),
                      child: Text(notice!),
                    ),
                  if (error != null)
                    Padding(
                      padding: const EdgeInsets.only(top: 12),
                      child: SelectableText(
                        error!,
                        style: TextStyle(color: p.errorText),
                      ),
                    ),
                  const SizedBox(height: 16),
                  if (document != null) ...[
                    Text(
                      document['name'] as String,
                      style: const TextStyle(
                        fontSize: 18,
                        fontWeight: FontWeight.w600,
                      ),
                    ),
                    const SizedBox(height: 8),
                    Text(document['description'] as String),
                    const SizedBox(height: 8),
                    Text(
                      '${review!['enabled'] == true ? 'Enabled' : 'Not enabled'}${review!['activeVersion'] == null ? '' : ' · saved snapshot v${review!['activeVersion']}'}',
                    ),
                    if (review!['reviewVersion'] != null)
                      Text(
                        'Reviewing saved version ${review!['reviewVersion']}. Activation records a new version; the source file stays unchanged.',
                        style: TextStyle(color: p.muted, fontSize: 12),
                      ),
                    if (review!['sourceMatches'] != true)
                      const Text(
                        'The reviewed snapshot differs from the current file, or the file is unavailable.',
                      ),
                    if (review!['problem'] != null)
                      Text(
                        review!['problem'] as String,
                        style: TextStyle(color: p.muted, fontSize: 12),
                      ),
                    const SizedBox(height: 8),
                    Card(
                      elevation: 0,
                      color: p.surface,
                      child: Padding(
                        padding: const EdgeInsets.all(16),
                        child: SelectableText(
                          document['text'] as String,
                          key: const Key('skill-source-text'),
                        ),
                      ),
                    ),
                    const SizedBox(height: 8),
                    if ((review!['versions'] as List).isNotEmpty) ...[
                      const Text(
                        'Saved versions · select to review before rollback',
                      ),
                      Wrap(
                        spacing: 8,
                        runSpacing: 4,
                        children: [
                          for (final version in review!['versions'] as List)
                            TextButton(
                              key: Key('skill-version-${version['version']}'),
                              onPressed: busy
                                  ? null
                                  : () => _select(
                                      document['name'] as String,
                                      version['version'] as int,
                                    ),
                              child: Text(
                                'Version ${version['version']}${version['rollbackFrom'] == null ? '' : ' · from v${version['rollbackFrom']}'}\n${DateTime.fromMillisecondsSinceEpoch(version['reviewedAt'] as int).toLocal().toString().split('.').first}',
                              ),
                            ),
                        ],
                      ),
                    ],
                  ] else ...[
                    if (catalog?['problem'] != null)
                      Text(
                        catalog!['problem'] as String,
                        style: TextStyle(color: p.muted),
                      ),
                    if (catalog?['partial'] == true)
                      const Text(
                        'Partial list: directory scan limit reached. Saved skills are still shown.',
                      ),
                    if (items.isEmpty && !busy) ...[
                      const Text('No project skills yet.'),
                      const SizedBox(height: 8),
                      const SelectableText(
                        'Add .agents/skills/<skill-name>/SKILL.md in this folder, then Refresh. Use YAML name and description, followed by Markdown instructions.',
                      ),
                    ],
                    for (final item in items)
                      Card(
                        elevation: 0,
                        color: p.surface,
                        child: Padding(
                          padding: const EdgeInsets.all(12),
                          child: Column(
                            crossAxisAlignment: CrossAxisAlignment.start,
                            children: [
                              Text(
                                item['name'] as String,
                                style: const TextStyle(
                                  fontWeight: FontWeight.w600,
                                ),
                              ),
                              Text(
                                item['revision'] == null
                                    ? 'Available · not reviewed'
                                    : '${item['enabled'] == true ? 'Enabled' : 'Disabled'} · version ${item['version']}',
                              ),
                              if (item['description'] != null)
                                Text(
                                  item['description'] as String,
                                  maxLines: 3,
                                  overflow: TextOverflow.ellipsis,
                                ),
                              Wrap(
                                spacing: 8,
                                children: [
                                  TextButton(
                                    key: Key('review-skill-${item['name']}'),
                                    onPressed: busy
                                        ? null
                                        : () => _select(item['name'] as String),
                                    child: const Text('Review file'),
                                  ),
                                  if (item['revision'] != null)
                                    TextButton(
                                      key: Key('saved-skill-${item['name']}'),
                                      onPressed: busy
                                          ? null
                                          : () => _select(
                                              item['name'] as String,
                                              item['version'] as int,
                                            ),
                                      child: const Text('Saved versions'),
                                    ),
                                ],
                              ),
                            ],
                          ),
                        ),
                      ),
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
                  if (review != null)
                    TextButton(
                      key: const Key('skills-back'),
                      onPressed: busy ? null : _list,
                      child: const Text('Back'),
                    ),
                  TextButton(
                    key: const Key('refresh-skills'),
                    onPressed: busy
                        ? null
                        : () => document == null
                              ? _list()
                              : _select(
                                  document['name'] as String,
                                  review!['reviewVersion'] as int?,
                                ),
                    child: const Text('Refresh'),
                  ),
                  if (review != null) ...[
                    TextButton(
                      key: const Key('disable-skill'),
                      onPressed: busy || review!['enabled'] != true
                          ? null
                          : () => _mutate(false),
                      child: const Text('Disable'),
                    ),
                    TextButton(
                      key: const Key('forget-skill'),
                      onPressed: busy || review!['revision'] == null
                          ? null
                          : () => _mutate(true),
                      child: const Text('Forget saved versions'),
                    ),
                    FilledButton(
                      key: const Key('activate-skill'),
                      onPressed:
                          busy ||
                              review!['token'] == null ||
                              review!['alreadyActive'] == true
                          ? null
                          : _activate,
                      child: Text(
                        review!['reviewVersion'] == null
                            ? 'Activate skill'
                            : 'Activate version',
                      ),
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
