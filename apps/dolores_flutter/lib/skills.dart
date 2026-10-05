import 'skill_create.dart';

import 'package:flutter/material.dart';
import 'package:file_selector/file_selector.dart';

import 'bridge.dart';
import 'chat.dart';
import 'inspector.dart';
import 'skill_draft.dart';
import 'theme.dart';

Future<void> showSkills(
  BuildContext context,
  ChatController chat, {
  Future<String?> Function(String name, int version)? chooseExportPath,
}) => chat.inspectLocalSettings(() async {
  if (chat.session == null) return;
  await showDialog<void>(
    context: context,
    barrierDismissible: false,
    builder: (_) => SkillsInspector(
      bridge: chat.bridge,
      session: chat.session!,
      hasProject: chat.workspaceRoot != null,
      chooseExportPath: chooseExportPath,
    ),
  );
  chat.invalidateContext();
});

class SkillsInspector extends StatefulWidget {
  final ChatBridge bridge;
  final String session;
  final bool hasProject;
  final Future<String?> Function(String name, int version)? chooseExportPath;
  const SkillsInspector({
    super.key,
    required this.bridge,
    this.session = '',
    this.hasProject = true,
    this.chooseExportPath,
  });
  @override
  State<SkillsInspector> createState() => _SkillsInspectorState();
}

class _SkillsInspectorState extends State<SkillsInspector> {
  Map<String, dynamic>? catalog, review;
  String? error, notice;
  bool busy = false;
  late String scope;
  final _scroll = ScrollController();
  Future<dynamic> _call(
    String command, [
    Map<String, dynamic> args = const {},
  ]) => widget.bridge.call({
    'command': command,
    'session': widget.session,
    'scope': scope,
    ...args,
  });
  @override
  void initState() {
    super.initState();
    scope = widget.hasProject ? 'project' : 'global';
    _list();
  }

  void _showStatus() {
    WidgetsBinding.instance.addPostFrameCallback((_) {
      if (mounted && _scroll.hasClients) _scroll.jumpTo(0);
    });
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
      if (mounted) {
        setState(() => error = failure.toString());
        _showStatus();
      }
    } finally {
      if (mounted) setState(() => busy = false);
    }
  }

  Future<void> _loadList() async {
    if (mounted) setState(() => review = null);
    final result = await _call('projectSkills');
    if (mounted) {
      setState(() => catalog = (result as Map).cast<String, dynamic>());
      _showStatus();
    }
  }

  Future<void> _list() => _act(_loadList);
  Future<void> _create(bool importing) => _act(() async {
    final saved = await showDialog<bool>(
      context: context,
      barrierDismissible: false,
      builder: (_) => SkillCreateDialog(
        bridge: widget.bridge,
        session: widget.session,
        scope: scope,
        importing: importing,
      ),
    );
    if (!mounted) return;
    if (saved == true) notice = 'Skill activated for future messages.';
    await _loadList();
  });
  Future<void> _draft() => _act(() async {
    final promoted = await showDialog<bool>(
      context: context,
      barrierDismissible: false,
      builder: (_) => SkillDraftInspector(
        bridge: widget.bridge,
        session: widget.session,
        scope: scope,
      ),
    );
    if (!mounted) return;
    if (promoted == true) {
      notice = 'Tested skill activated for future messages.';
    }
    await _loadList();
  });
  Future<void> _changeScope(String value) => _act(() async {
    setState(() {
      scope = value;
      notice = null;
      catalog = null;
    });
    await _loadList();
  });
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
  Future<void> _export() => _act(() async {
    final token = review?['token'];
    final version = review?['reviewVersion'] as int?;
    if (token == null || version == null) return;
    final name = review!['document']['name'] as String;
    final path = widget.chooseExportPath != null
        ? await widget.chooseExportPath!(name, version)
        : (await getSaveLocation(
            suggestedName: 'SKILL.md',
            confirmButtonText: 'Export version $version',
            acceptedTypeGroups: [
              const XTypeGroup(
                label: 'Markdown',
                extensions: ['md'],
                uniformTypeIdentifiers: ['public.text'],
              ),
            ],
          ))?.path;
    if (path == null) return;
    await _call('exportSkill', {'token': token, 'path': path});
    if (mounted) {
      setState(
        () => notice =
            'Exported $name version $version. Activation is unchanged.',
      );
      _showStatus();
    }
  });
  Future<void> _mutate(bool forget) => _act(() async {
    final name = review!['document']['name'];
    await _call(forget ? 'forgetSkill' : 'disableSkill', {
      'name': name,
      'revision': review!['revision'],
    });
    notice = forget
        ? 'Saved versions forgotten. The source file is unchanged.'
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
        title: 'Skills',
        subtitle: scope == 'global' ? 'All projects' : 'This project',
        canClose: !busy,
        child: Column(
          children: [
            if (busy) const LinearProgressIndicator(minHeight: 2),
            Padding(
              padding: const EdgeInsets.fromLTRB(20, 12, 20, 0),
              child: SegmentedButton<String>(
                segments: [
                  ButtonSegment(
                    value: 'project',
                    label: const Text('This project'),
                    enabled: widget.hasProject,
                  ),
                  const ButtonSegment(
                    value: 'global',
                    label: Text('All projects'),
                  ),
                ],
                selected: {scope},
                onSelectionChanged: busy
                    ? null
                    : (values) => _changeScope(values.single),
              ),
            ),
            Expanded(
              child: ListView(
                key: const Key('skills-scroll'),
                controller: _scroll,
                padding: const EdgeInsets.all(20),
                children: [
                  if (notice != null) ...[
                    Text(notice!),
                    const SizedBox(height: 12),
                  ],
                  if (error != null) ...[
                    SelectableText(
                      error!,
                      style: TextStyle(color: p.errorText),
                    ),
                    const SizedBox(height: 12),
                  ],
                  Text(
                    'Review instructions before activation. Tools keep their existing access rules.',
                  ),
                  const SizedBox(height: 8),
                  ExpansionTile(
                    tilePadding: EdgeInsets.zero,
                    title: const Text('Limits and resources'),
                    children: [
                      Text(
                        'Active skills use saved versions. File changes wait for a new review. Project skills override active global skills with the same name. Up to 3 active skills and 8 KiB per scope; 5 recent versions per skill. References and scripts require approved tools within the working folder; global resources outside it are unavailable.',
                        style: TextStyle(color: p.muted, fontSize: 12),
                      ),
                    ],
                  ),
                  const SizedBox(height: 16),
                  if (catalog?['directory'] is String) ...[
                    SelectableText(
                      catalog!['directory'] as String,
                      style: TextStyle(color: p.muted, fontSize: 12),
                    ),
                    const SizedBox(height: 12),
                  ],
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
                    if (review!['reviewVersion'] != null) ...[
                      const SizedBox(height: 8),
                      Text(
                        'Export copies this saved version. In the save dialog, choose or create a folder named ${document['name']} and save as SKILL.md. Existing files are never overwritten. References and scripts are not copied; review them separately before reuse.',
                        style: TextStyle(color: p.muted, fontSize: 12),
                      ),
                    ],
                    if (review!['generated'] == true)
                      const Text(
                        'Drafted and tested in Dolores. Stored locally; no source file was created.',
                      ),
                    if (review!['sourceMatches'] != true &&
                        review!['generated'] != true)
                      const Text(
                        'The reviewed snapshot differs from the current file, or the file is unavailable.',
                      ),
                    if (review!['problem'] != null)
                      Text(
                        review!['problem'] as String,
                        style: TextStyle(color: p.muted, fontSize: 12),
                      ),
                    const SizedBox(height: 8),
                    if (review!['evaluation'] is Map)
                      SkillEvaluationView(
                        evaluation: review!['evaluation'] as Map,
                      ),
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
                      Text('No $scope skills yet.'),
                      const SizedBox(height: 8),
                      const Text(
                        'Import a skill, create one, or draft it from a completed chat.',
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
                              if (item['overridden'] == true &&
                                  item['enabled'] == true)
                                const Text(
                                  'Overridden here by the active project skill.',
                                ),
                              if (item['description'] != null)
                                Text(
                                  item['description'] as String,
                                  maxLines: 3,
                                  overflow: TextOverflow.ellipsis,
                                ),
                              if (item['generated'] == true)
                                const Text(
                                  'Drafted and tested · saved locally',
                                ),
                              Wrap(
                                spacing: 8,
                                children: [
                                  TextButton(
                                    key: Key('review-skill-${item['name']}'),
                                    onPressed:
                                        busy || item['sourceAvailable'] == false
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
                  if (review == null) ...[
                    TextButton.icon(
                      key: const Key('import-skill'),
                      onPressed: busy ? null : () => _create(true),
                      icon: const Icon(Icons.file_download_outlined),
                      label: const Text('Import'),
                    ),
                    TextButton.icon(
                      key: const Key('create-skill'),
                      onPressed: busy ? null : () => _create(false),
                      icon: const Icon(Icons.add),
                      label: const Text('Create'),
                    ),
                  ],
                  if (review == null && widget.session.isNotEmpty)
                    TextButton(
                      key: const Key('draft-skill'),
                      onPressed: busy ? null : _draft,
                      child: const Text('Draft from chat'),
                    ),
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
                    if (review!['reviewVersion'] != null)
                      TextButton(
                        key: const Key('export-skill'),
                        onPressed: busy || review!['token'] == null
                            ? null
                            : _export,
                        child: const Text('Export SKILL.md'),
                      ),
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
