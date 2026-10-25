import 'help_tip.dart';

import 'package:file_selector/file_selector.dart';
import 'package:flutter/material.dart';

import 'bridge.dart';
import 'inspector.dart';
import 'form_field_layout.dart';

/// Package management uses its own review tokens, separate from SKILL.md edits.
class SkillPackages extends StatefulWidget {
  final ChatBridge bridge;
  final String session, initialScope;
  final bool hasProject;
  final VoidCallback onBack;
  final Future<String?> Function()? chooseArchive;
  final Future<String?> Function(String name)? chooseExport;
  const SkillPackages({
    super.key,
    required this.bridge,
    required this.session,
    required this.initialScope,
    required this.hasProject,
    required this.onBack,
    this.chooseArchive,
    this.chooseExport,
  });
  @override
  State<SkillPackages> createState() => _SkillPackagesState();
}

class _SkillPackagesState extends State<SkillPackages> {
  final name = TextEditingController();
  final scroll = ScrollController();
  late String scope = widget.initialScope;
  Map? catalog, review;
  String? archive, error, notice;
  bool busy = false, importing = false, retained = false;
  Map? get package => review?['package'] as Map?;
  Map? get snapshot => package?['package'] as Map?;
  List get savedVersions => review?['active']?['versions'] as List? ?? [];
  List get packageVersions => savedVersions
      .where((v) => v['package']?['id'] == snapshot?['id'])
      .toList();
  int? get currentVersion => savedVersions.lastOrNull?['version'] as int?;
  bool get referencedHere =>
      !(review?['entry'] as String? ?? '').startsWith('stage-') &&
      savedVersions.any((v) => v['package']?['id'] == snapshot?['id']);
  bool get alreadyActive =>
      review?['active']?['enabled'] == true &&
      savedVersions.lastOrNull?['package']?['id'] == snapshot?['id'];
  Future<dynamic> call(
    String action, [
    Map<String, dynamic> fields = const {},
  ]) => widget.bridge.call({
    'command': 'skillPackages',
    'session': widget.session,
    'scope': scope,
    'request': {'action': action, ...fields},
  });
  @override
  void initState() {
    super.initState();
    act(load);
  }

  Future<void> act(Future<void> Function() action) async {
    if (busy) return;
    setState(() {
      busy = true;
      error = null;
    });
    try {
      await action();
    } catch (e) {
      if (mounted) setState(() => error = e.toString());
    } finally {
      if (mounted) setState(() => busy = false);
    }
  }

  Future<void> clearReview() async {
    final token = review?['token'];
    if (token != null) await call('cancel', {'token': token});
    if (mounted) setState(() => review = null);
  }

  Future<void> load() async {
    final value = await call('list');
    if (mounted) setState(() => catalog = value as Map);
  }

  void top() => WidgetsBinding.instance.addPostFrameCallback((_) {
    if (mounted && scroll.hasClients) scroll.jumpTo(0);
  });

  Future<void> inspect(String action, Map<String, dynamic> fields) async {
    await clearReview();
    final value = await call(action, fields);
    if (mounted) {
      setState(() {
        review = value as Map;
        retained = action == 'inspectRetained';
        importing = false;
        notice = null;
      });
      top();
    }
  }

  Future<void> choose() async {
    final path = widget.chooseArchive != null
        ? await widget.chooseArchive!()
        : (await openFile(
            acceptedTypeGroups: [
              const XTypeGroup(label: 'Skill package', extensions: ['zip']),
            ],
          ))?.path;
    if (mounted && path != null) setState(() => archive = path);
  }

  Future<void> install() async {
    final result = await call('install', {'token': review!['token']}) as Map;
    if (!mounted) return;
    setState(() {
      review = {...review!, 'registered': true, 'package': result['installed']};
      notice = 'Installed. Choose Activate when you are ready.';
    });
    await load();
  }

  Future<void> activatePackage([int? rollback]) async {
    final id = snapshot!['id'];
    await call('activate', {'token': review!['token'], 'rollback': rollback});
    // Activation consumes the review. Reinspect before offering more actions.
    if (mounted) setState(() => review = null);
    await inspect('inspectInstalled', {'id': id});
    if (mounted) setState(() => notice = 'Activated for future messages.');
    await load();
  }

  Future<void> export() async {
    final suggested = '${package!['document']['name']}-package.zip';
    final path = widget.chooseExport != null
        ? await widget.chooseExport!(suggested)
        : (await getSaveLocation(
            suggestedName: suggested,
            acceptedTypeGroups: [
              const XTypeGroup(label: 'Skill package', extensions: ['zip']),
            ],
          ))?.path;
    if (path == null) return;
    await call('export', {'token': review!['token'], 'path': path});
    if (mounted) setState(() => notice = 'Exported the original ZIP.');
  }

  Future<void> remove() async {
    final confirm = await showDialog<bool>(
      context: context,
      builder: (context) => AlertDialog(
        title: const Text('Remove this installation?'),
        content: const Text(
          'Other scopes and saved skill versions keep their resources. Changed or unknown files are preserved.',
        ),
        actions: [
          TextButton(
            onPressed: () => Navigator.pop(context, false),
            child: const Text('Keep'),
          ),
          FilledButton(
            onPressed: () => Navigator.pop(context, true),
            child: const Text('Remove installation'),
          ),
        ],
      ),
    );
    if (confirm != true) return;
    final result = await call('remove', {'token': review!['token']}) as Map;
    if (mounted) {
      setState(() {
        review = null;
        notice = result['filesRetained'] == true
            ? 'Installation removed here. Shared resources were retained.'
            : 'Removed verified package data.';
      });
    }
    await load();
  }

  Future<void> forgetVersion(int version) async {
    final confirmed = await showDialog<bool>(
      context: context,
      builder: (context) => AlertDialog(
        title: Text('Forget saved v$version?'),
        content: Text(
          'Remove only this older activation. Current v$currentVersion, the installation and all package files are kept. You can still review and activate the installed package again.',
        ),
        actions: [
          TextButton(
            onPressed: () => Navigator.pop(context, false),
            child: const Text('Keep version'),
          ),
          FilledButton(
            key: const Key('confirm-forget-package-version'),
            onPressed: () => Navigator.pop(context, true),
            child: Text('Forget saved v$version'),
          ),
        ],
      ),
    );
    if (confirmed != true) return;
    final result = await call('forgetVersion', {
      'token': review!['token'],
      'version': version,
    }) as Map;
    if (mounted) {
      setState(() {
        review = {...review!, 'active': result['active']};
        notice =
            'Forgot saved v$version. Current version and package files kept.';
      });
    }
  }

  @override
  void dispose() {
    final token = review?['token'];
    if (token != null) call('cancel', {'token': token}).catchError((_) => null);
    name.dispose();
    scroll.dispose();
    super.dispose();
  }

  Widget details(String title, List<Widget> children) => ExpansionTile(
    tilePadding: EdgeInsets.zero,
    title: Text(title),
    childrenPadding: const EdgeInsets.only(bottom: 12),
    expandedCrossAxisAlignment: CrossAxisAlignment.stretch,
    children: children,
  );

  List<Widget> reviewBody() {
    final doc = package!['document'] as Map;
    final manifest = snapshot!['manifest'] as Map;
    final active = review!['active'] as Map?;
    final old = (active?['versions'] as List? ?? []).lastOrNull;
    final previous = <String, dynamic>{
      for (final r in old?['package']?['resources'] as List? ?? [])
        r['path'] as String: r,
    };
    final resources = snapshot!['resources'] as List;
    final currentPaths = resources.map((r) => r['path']).toSet();
    return [
      Text(
        '${doc['name']} · ${manifest['version']}',
        style: Theme.of(context).textTheme.titleMedium,
      ),
      Text(doc['description'] as String),
      const SizedBox(height: 8),
      Text(
        review!['registered'] == true
            ? 'Installed in this scope'
            : 'Not installed in this scope',
      ),
      if (retained)
        const Text(
          'Recovery review: payload files may be damaged. Export checks the original ZIP; activation checks all resources.',
        ),
      if (referencedHere)
        const Text(
          'Saved versions use this package. Forget older activations below to release it; keep the package used by the current version.',
        ),
      details('Instructions', [SelectableText(doc['text'] as String)]),
      if (old != null && old['document']['text'] != doc['text'])
        details('Previous instructions', [
          SelectableText(old['document']['text'] as String),
        ]),
      details('Files and changes', [
        for (final resource in resources)
          ListTile(
            dense: true,
            contentPadding: EdgeInsets.zero,
            title: Text(resource['path'] as String),
            subtitle: Text(
              '${resource['bytes']} bytes · ${previous[resource['path']] == null
                  ? 'Added'
                  : previous[resource['path']]['digest'] == resource['digest']
                  ? 'Unchanged'
                  : 'Changed'}',
            ),
          ),
        for (final path in previous.keys.where(
          (p) => !currentPaths.contains(p),
        ))
          Text('$path · Removed'),
        for (final directory in snapshot!['directories'] as List? ?? [])
          Text('$directory/'),
      ]),
      details('Origin and dependencies', [
        Text('Declared origin: ${manifest['origin'] ?? 'Not provided'}'),
        const Text(
          'Publisher claims are not independently verified. Dependencies are listed only.',
        ),
        if ((manifest['dependencies'] as List? ?? []).isEmpty)
          const Text('No declared dependencies.'),
        for (final dependency in manifest['dependencies'] as List? ?? [])
          Text('${dependency['name']} · ${dependency['requirement']}'),
      ]),
      details('Package identity', [
        SelectableText(
          'Archive: ${snapshot!['id']}\nContent: ${snapshot!['contentDigest']}\nResources: ${snapshot!['root']}',
        ),
      ]),
      if (review!['registered'] == true || savedVersions.isNotEmpty)
        details('Saved versions', [
          if (packageVersions.isEmpty)
            const Text('No saved activation for this package.'),
          for (final version in packageVersions)
            if (version['version'] == currentVersion)
              Text('Current saved v$currentVersion')
            else
              Wrap(
                spacing: 8,
                children: [
                  if (review!['registered'] == true)
                    TextButton(
                      key: Key('rollback-package-${version['version']}'),
                      onPressed: busy
                          ? null
                          : () => act(
                              () => activatePackage(version['version'] as int),
                            ),
                      child: Text('Restore saved v${version['version']}'),
                    ),
                  TextButton(
                    key: Key('forget-package-version-${version['version']}'),
                    onPressed: busy
                        ? null
                        : () => act(
                            () => forgetVersion(version['version'] as int),
                          ),
                    child: Text('Forget saved v${version['version']}'),
                  ),
                ],
              ),
        ]),
    ];
  }

  @override
  Widget build(BuildContext context) => PopScope(
    canPop: !busy,
    child: InspectorFrame(
      title: 'Skill packages',
      subtitle: 'Review, install and manage complete skills',
      canClose: !busy,
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.stretch,
        children: [
          if (busy) const LinearProgressIndicator(minHeight: 2),
          Padding(
            padding: const EdgeInsets.fromLTRB(20, 8, 20, 0),
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
                  : (values) => act(() async {
                      await clearReview();
                      if (mounted) {
                        setState(() {
                          scope = values.single;
                          catalog = null;
                          notice = null;
                        });
                      }
                      await load();
                    }),
            ),
          ),
          Expanded(
            child: ListView(
              key: const Key('packages-scroll'),
              controller: scroll,
              padding: const EdgeInsets.all(20),
              children: [
                const Text(
                  'Installation does not activate instructions, run scripts or grant tool access.',
                ),
                if (review != null)
                  ...reviewBody()
                else if (importing) ...[
                  const SizedBox(height: 16),
                  FieldLabel(
                    label: 'Skill name (from SKILL.md)',
                    child: TextField(
                      key: const Key('package-name'),
                      controller: name,
                      enabled: !busy,
                      onChanged: (_) => setState(() {}),
                    ),
                  ),
                  const SizedBox(height: 8),
                  if (archive != null) SelectableText(archive!),
                  TextButton.icon(
                    key: const Key('choose-package-zip'),
                    onPressed: busy ? null : () => act(choose),
                    icon: const Icon(Icons.folder_open_outlined),
                    label: const Text('Choose ZIP'),
                  ),
                ] else ...[
                  const SizedBox(height: 12),
                  if ((catalog?['installed'] as List? ?? []).isEmpty && !busy)
                    const Text('No packages installed here.'),
                  for (final item in catalog?['installed'] as List? ?? [])
                    ListTile(
                      contentPadding: EdgeInsets.zero,
                      title: Text(
                        '${item['document']['name']} · ${item['package']['manifest']['version']}',
                      ),
                      subtitle: Text(
                        'Archive ${(item['package']['id'] as String).substring(0, 12)}',
                      ),
                      trailing: TextButton(
                        key: Key('inspect-package-${item['package']['id']}'),
                        onPressed: busy
                            ? null
                            : () => act(
                                () => inspect('inspectInstalled', {
                                  'id': item['package']['id'],
                                }),
                              ),
                        child: const Text('Review'),
                      ),
                    ),
                  details('Retained data and recovery', [
                    const Text(
                      'Inspect the original package when resources are unavailable. Changed files stay preserved; restore their original bytes or activate another healthy version.',
                    ),
                    for (final item in catalog?['retained'] as List? ?? [])
                      ListTile(
                        contentPadding: EdgeInsets.zero,
                        title: Text(
                          '${item['name'] ?? 'Unrecognized data'} · ${item['version'] ?? item['entry']}',
                        ),
                        subtitle: Text(
                          item['problem'] as String? ??
                              (item['referenced'] == true
                                  ? 'In use by saved skills'
                                  : 'Retained package data'),
                        ),
                        trailing: item['problem'] != null
                            ? null
                            : TextButton(
                                key: Key('retained-package-${item['entry']}'),
                                onPressed: busy
                                    ? null
                                    : () => act(
                                        () => inspect('inspectRetained', {
                                          'entry': item['entry'],
                                        }),
                                      ),
                                child: const Text('Inspect original'),
                              ),
                      ),
                  ]),
                ],
                details('Package format and limits', const [
                  HelpLabel(
                    help: 'ZIP with SKILL.md at its root, plus scripts, references and assets. Optional dolores-package.json declares schema, version, origin and dependencies. ZIP: 4 MiB; expanded: 2 MiB; resource: 512 KiB; SKILL.md: 8 KiB; 64 files. Portable relative paths only. Five installed versions per skill, twelve names per scope and sixty retained bundles per profile. Remove unused installations when full.',
                    child: Text(
                      'Choose a skill ZIP with SKILL.md at its root. Maximum ZIP size: 4 MiB.',
                    ),
                  ),
                ]),
                if (error != null)
                  details('Error details', [SelectableText(error!)]),
              ],
            ),
          ),
          const Divider(height: 1),
          Padding(
            padding: const EdgeInsets.all(12),
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.stretch,
              children: [
                if (error != null || notice != null)
                  Padding(
                    padding: const EdgeInsets.only(bottom: 8),
                    child: Text(
                      error ?? notice!,
                      key: const Key('package-status'),
                      maxLines: 3,
                      overflow: TextOverflow.ellipsis,
                      style: TextStyle(
                        color: error != null
                            ? Theme.of(context).colorScheme.error
                            : null,
                      ),
                    ),
                  ),
                Wrap(
                  spacing: 8,
                  runSpacing: 4,
                  children: [
                    TextButton(
                      key: const Key('packages-back'),
                      onPressed: busy
                          ? null
                          : () => act(() async {
                              await clearReview();
                              if (mounted) widget.onBack();
                            }),
                      child: const Text('Back to skills'),
                    ),
                    if (review == null && !importing)
                      TextButton(
                        key: const Key('import-package'),
                        onPressed: busy
                            ? null
                            : () => setState(() {
                                importing = true;
                                notice = null;
                              }),
                        child: const Text('Import ZIP'),
                      ),
                    if (importing && review == null)
                      FilledButton(
                        key: const Key('review-package-zip'),
                        onPressed:
                            busy || archive == null || name.text.trim().isEmpty
                            ? null
                            : () => act(
                                () => inspect('inspect', {
                                  'name': name.text.trim(),
                                  'path': archive,
                                }),
                              ),
                        child: const Text('Review ZIP'),
                      ),
                    if (review != null) ...[
                      if (review!['registered'] != true && !retained)
                        FilledButton(
                          key: const Key('install-package'),
                          onPressed: busy ? null : () => act(install),
                          child: const Text('Install'),
                        ),
                      if (review!['registered'] == true)
                        FilledButton(
                          key: const Key('activate-package'),
                          onPressed: busy || alreadyActive
                              ? null
                              : () => act(activatePackage),
                          child: const Text('Activate'),
                        ),
                      if (review!['registered'] == true || retained)
                        TextButton(
                          key: const Key('export-package'),
                          onPressed: busy ? null : () => act(export),
                          child: const Text('Export ZIP'),
                        ),
                      if (review!['registered'] == true || retained)
                        TextButton(
                          key: const Key('remove-package'),
                          onPressed: busy || referencedHere
                              ? null
                              : () => act(remove),
                          child: const Text('Remove'),
                        ),
                      TextButton(
                        key: const Key('package-list'),
                        onPressed: busy
                            ? null
                            : () => act(() async {
                                await clearReview();
                                await load();
                              }),
                        child: const Text('All packages'),
                      ),
                    ],
                    TextButton(
                      key: const Key('refresh-packages'),
                      onPressed: busy
                          ? null
                          : () => act(() async {
                              await clearReview();
                              await load();
                            }),
                      child: const Text('Refresh'),
                    ),
                  ],
                ),
              ],
            ),
          ),
        ],
      ),
    ),
  );
}
