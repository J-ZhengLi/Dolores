import 'package:flutter/material.dart';
import 'package:flutter/services.dart';

import 'git_host.dart';
import 'git_diff_view.dart';
import 'git_local_controls.dart';
import 'review_findings.dart';
import 'workspace_sidebar.dart';

Future<void> executeGitAction(
  BuildContext context,
  GitHost git,
  GitWorkspace w,
  Map<String, dynamic> operation,
) async {
  if (w.busy || w.reviewOpen != null) return;
  if (operation['kind'] == 'stage' ||
      operation['kind'] == 'unstage' ||
      operation['kind'] == 'stageAll' ||
      operation['kind'] == 'unstageAll') {
    await git.changeIndex(w, operation);
    return;
  }
  if (operation['kind'] == 'commit') {
    operation = {...operation, 'kind': 'commitAll'};
  }
  // The selected action is the user's decision. Prepare and consume the
  // host's revision token internally, preserving stale and editor guards.
  final prepared = await git.review(w, operation);
  if (prepared == null) return;
  await git.resolveReview(w, prepared, apply: context.mounted);
}

class GitCommitBox extends StatefulWidget {
  final GitHost git;
  final GitWorkspace workspace;
  const GitCommitBox({super.key, required this.git, required this.workspace});
  @override
  State<GitCommitBox> createState() => _GitCommitBoxState();
}

class _GitCommitBoxState extends State<GitCommitBox> {
  late final controller = TextEditingController(
    text: widget.workspace.commitDraft,
  );
  @override
  void didUpdateWidget(covariant GitCommitBox oldWidget) {
    super.didUpdateWidget(oldWidget);
    if (controller.text != widget.workspace.commitDraft) {
      controller.text = widget.workspace.commitDraft;
    }
  }

  @override
  void dispose() {
    controller.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    final w = widget.workspace;
    final entries = (w.status?['entries'] as List? ?? []).cast<Map>();
    final staged = entries
        .where((e) => e['index'] != ' ' && e['index'] != '?')
        .length;
    final count = staged > 0
        ? staged
        : entries.where((e) => e['worktree'] != ' ').length;
    final enabled =
        !w.busy &&
        w.reviewOpen == null &&
        controller.text.trim().isNotEmpty &&
        count > 0;
    void commit() {
      if (enabled) {
        executeGitAction(context, widget.git, w, {
          'kind': 'commit',
          'message': controller.text,
        });
      }
    }

    return Padding(
      padding: const EdgeInsets.symmetric(horizontal: 12, vertical: 8),
      child: CallbackShortcuts(
        bindings: {
          const SingleActivator(LogicalKeyboardKey.enter, control: true): () {
            commit();
          },
        },
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.stretch,
          children: [
            TextField(
              key: const Key('git-commit-message'),
              controller: controller,
              minLines: 1,
              maxLines: 4,
              maxLength: 8192,
              decoration: const InputDecoration(
                hintText: 'Message (Ctrl+Enter to commit)',
                counterText: '',
                isDense: true,
                contentPadding: EdgeInsets.symmetric(
                  horizontal: 10,
                  vertical: 8,
                ),
                border: OutlineInputBorder(),
              ),
              style: const TextStyle(fontSize: 12),
              onChanged: (v) =>
                  setState(() => widget.workspace.commitDraft = v),
            ),
            const SizedBox(height: 6),
            FilledButton.icon(
              key: const Key('git-commit-button'),
              onPressed: enabled ? commit : null,
              style: FilledButton.styleFrom(
                minimumSize: const Size(0, 28),
                padding: const EdgeInsets.symmetric(
                  horizontal: 10,
                  vertical: 4,
                ),
                textStyle: Theme.of(context).textTheme.labelLarge
                    ?.copyWith(fontSize: 12),
                tapTargetSize: MaterialTapTargetSize.shrinkWrap,
              ),
              icon: const Icon(Icons.check, size: 16),
              label: Text(
                staged > 0 ? 'Commit · $count' : 'Commit all · $count',
              ),
            ),
          ],
        ),
      ),
    );
  }
}

class SourceControlPanel extends StatelessWidget {
  final GitHost git;
  final VoidCallback openFolder;
  const SourceControlPanel({
    super.key,
    required this.git,
    required this.openFolder,
  });

  Widget pathLabel(String path, {String? oldPath}) {
    final parts = path.split('/');
    final name = parts.removeLast();
    return Expanded(
      child: Tooltip(
        message: oldPath == null ? path : '$path\nFrom $oldPath',
        child: LayoutBuilder(
          builder: (context, constraints) => Row(
            children: [
              Flexible(
                flex: 3,
                child: Text(name, maxLines: 1, overflow: TextOverflow.ellipsis),
              ),
              if (parts.isNotEmpty && constraints.maxWidth >= 160) ...[
                const SizedBox(width: 6),
                Flexible(
                  flex: 2,
                  child: Text(
                    parts.join('/'),
                    maxLines: 1,
                    overflow: TextOverflow.ellipsis,
                    style: TextStyle(
                      fontSize: 11,
                      color: Theme.of(context).colorScheme.onSurfaceVariant,
                    ),
                  ),
                ),
              ],
            ],
          ),
        ),
      ),
    );
  }

  Widget fileRow(BuildContext context, GitWorkspace w, Map entry, bool staged) {
    final path = entry['path'] as String;
    return WorkspaceSidebarRow(
      key: ValueKey('git-${staged ? 'staged' : 'working'}-$path'),
      selected:
          w.tabs[w.activeTab]?['path'] == path &&
          w.tabs[w.activeTab]?['basis'] == (staged ? 'staged' : 'working'),
      onTap: w.busy
          ? null
          : () => git.openDiff(w, path, staged ? 'staged' : 'working'),
      child: Row(
        children: [
          const Icon(Icons.insert_drive_file_outlined, size: 16),
          const SizedBox(width: 6),
          pathLabel(path, oldPath: entry['oldPath'] as String?),
          const SizedBox(width: 4),
          Text(
            (staged ? entry['index'] : entry['worktree']) as String,
            style: const TextStyle(fontSize: 11),
          ),
          WorkspaceSidebarButton(
            key: ValueKey('git-${staged ? 'unstage' : 'stage'}-$path'),
            tooltip: staged ? 'Unstage changes' : 'Stage changes',
            icon: staged ? Icons.remove : Icons.add,
            onPressed: w.busy || w.reviewOpen != null
                ? null
                : () => git.changeIndex(w, {
                    'kind': staged ? 'unstage' : 'stage',
                    'paths': [path],
                  }),
          ),
          PopupMenuButton<String>(
            tooltip: 'File Git actions',
            iconSize: 16,
            style: IconButton.styleFrom(
              minimumSize: const Size(28, 26),
              maximumSize: const Size(28, 26),
              tapTargetSize: MaterialTapTargetSize.shrinkWrap,
            ),
            constraints: const BoxConstraints(minWidth: 32, minHeight: 32),
            padding: EdgeInsets.zero,
            enabled: !w.busy && w.reviewOpen == null,
            onSelected: (kind) => executeGitAction(context, git, w, {
              'kind': kind,
              if (kind == 'discard')
                'path': entry['path']
              else
                'paths': [entry['path']],
              if (kind == 'stashCreate') 'message': 'Saved ${entry['path']}',
            }),
            itemBuilder: (_) => [
              PopupMenuItem(
                value: staged ? 'unstage' : 'stage',
                child: Text(staged ? 'Unstage changes' : 'Stage changes'),
              ),
              if (!staged &&
                  entry['index'] != '?' &&
                  entry['conflict'] != true) ...[
                const PopupMenuItem(
                  value: 'stashCreate',
                  child: Text('Stash file'),
                ),
                const PopupMenuItem(
                  value: 'discard',
                  child: Text('Discard changes'),
                ),
              ],
            ],
          ),
        ],
      ),
    );
  }

  Widget commitRow(
    BuildContext context,
    GitWorkspace w,
    Map<String, dynamic> item,
  ) {
    final id = item['id'] as String;
    final expanded = w.expandedCommits.contains(id);
    final files = w.commitFileCache[id]?['files'] as List?;
    return Column(
      key: ValueKey('git-history-$id'),
      children: [
        WorkspaceSidebarRow(
          key: ValueKey('git-commit-$id'),
          minHeight: 44,
          onTap: w.busy ? null : () => git.toggleCommit(w, id),
          child: Row(
            children: [
              Icon(
                expanded ? Icons.expand_more : Icons.chevron_right,
                size: 16,
              ),
              const SizedBox(width: 6),
              Expanded(
                child: Padding(
                  padding: const EdgeInsets.symmetric(vertical: 5),
                  child: Column(
                    crossAxisAlignment: CrossAxisAlignment.start,
                    children: [
                      Tooltip(
                        message: item['subject'] as String,
                        child: Text(
                          item['subject'] as String,
                          maxLines: 1,
                          overflow: TextOverflow.ellipsis,
                        ),
                      ),
                      Text(
                        '${id.substring(0, id.length.clamp(0, 8))} · ${item['author']}',
                        maxLines: 1,
                        overflow: TextOverflow.ellipsis,
                        style: TextStyle(
                          fontSize: 11,
                          color: Theme.of(context).colorScheme.onSurfaceVariant,
                        ),
                      ),
                    ],
                  ),
                ),
              ),
              PopupMenuButton<String>(
                tooltip: 'Commit actions',
                icon: const Icon(Icons.more_vert, size: 16),
                padding: EdgeInsets.zero,
                style: IconButton.styleFrom(
                  minimumSize: const Size(28, 26),
                  maximumSize: const Size(28, 26),
                  tapTargetSize: MaterialTapTargetSize.shrinkWrap,
                ),
                enabled: !w.busy && w.reviewOpen == null,
                onSelected: (_) => executeGitAction(context, git, w, {
                  'kind': 'revert',
                  'commit': id,
                }),
                itemBuilder: (_) => const [
                  PopupMenuItem(value: 'revert', child: Text('Revert commit…')),
                ],
              ),
            ],
          ),
        ),
        if (expanded) ...[
          if (w.loadingCommit == id)
            const Padding(
              padding: EdgeInsets.all(8),
              child: LinearProgressIndicator(),
            ),
          if (w.commitFileErrors[id] != null)
            Padding(
              padding: const EdgeInsets.only(left: 32, right: 12),
              child: Column(
                crossAxisAlignment: CrossAxisAlignment.start,
                children: [
                  Text(
                    w.commitFileErrors[id]!,
                    style: const TextStyle(fontSize: 12),
                  ),
                  TextButton(
                    key: ValueKey('git-retry-$id'),
                    onPressed: w.busy ? null : () => git.commitFiles(w, id),
                    child: const Text('Retry files'),
                  ),
                ],
              ),
            ),
          if (files != null && files.isEmpty)
            const Padding(
              padding: EdgeInsets.all(8),
              child: Text('No file changes'),
            ),
          for (final path in files ?? [])
            WorkspaceSidebarRow(
              key: ValueKey('git-file-$id:$path'),
              indent: 20,
              onTap: w.busy
                  ? null
                  : () => git.openDiff(w, path, 'commit', commit: id),
              child: Row(
                children: [
                  const Icon(Icons.insert_drive_file_outlined, size: 16),
                  const SizedBox(width: 6),
                  pathLabel(path as String),
                ],
              ),
            ),
        ],
      ],
    );
  }

  Widget section(
    String title,
    bool expanded,
    VoidCallback toggle, {
    Widget? trailing,
    int? count,
  }) => WorkspaceSidebarRow(
    minHeight: 28,
    onTap: toggle,
    child: Row(
      children: [
        Icon(expanded ? Icons.expand_more : Icons.chevron_right, size: 16),
        const SizedBox(width: 4),
        Expanded(
          child: Text(
            title,
            maxLines: 1,
            overflow: TextOverflow.ellipsis,
            style: const TextStyle(fontSize: 12, fontWeight: FontWeight.w600),
          ),
        ),
        if (count != null)
          Padding(
            padding: const EdgeInsets.symmetric(horizontal: 6),
            child: Text('$count', style: const TextStyle(fontSize: 11)),
          ),
        ?trailing,
      ],
    ),
  );

  @override
  Widget build(BuildContext context) => ListenableBuilder(
    listenable: git,
    builder: (context, _) {
      final w = git.selected;
      final status = w?.status;
      final repositoryRoot = status?['root'] as String? ?? w?.root;
      final entries = (status?['entries'] as List? ?? []).cast<Map>();
      return Column(
        crossAxisAlignment: CrossAxisAlignment.stretch,
        children: [
          Padding(
            padding: const EdgeInsets.fromLTRB(16, 8, 8, 8),
            child: Row(
              children: [
                const Expanded(
                  child: Text(
                    'SOURCE CONTROL',
                    maxLines: 1,
                    overflow: TextOverflow.ellipsis,
                    style: TextStyle(fontSize: 11, fontWeight: FontWeight.w600),
                  ),
                ),
                if (w != null) ...[
                  WorkspaceSidebarButton(
                    tooltip: 'Refresh',
                    icon: Icons.refresh,
                    onPressed: w.busy ? null : () => git.refresh(w),
                  ),
                  if (git.reviewAvailable)
                    WorkspaceSidebarButton(
                      tooltip: 'Review findings',
                      icon: Icons.rate_review_outlined,
                      onPressed: () => showReviewFindings(context, git, w),
                    ),
                  if (w.busy)
                    WorkspaceSidebarButton(
                      tooltip: 'Stop Git operation',
                      icon: Icons.stop_circle_outlined,
                      onPressed: () => git.stop(w),
                    ),
                  if (status != null)
                    GitLocalControls(
                      git: git,
                      w: w,
                      review: (op) => executeGitAction(context, git, w, op),
                    ),
                ],
              ],
            ),
          ),
          if (w == null)
            TextButton(onPressed: openFolder, child: const Text('Open folder')),
          if (status != null)
            Tooltip(
              message: 'Repository: $repositoryRoot',
              child: WorkspaceSidebarRow(
                key: const Key('git-branch-summary'),
                child: Row(
                  children: [
                    const Icon(Icons.account_tree_outlined, size: 16),
                    const SizedBox(width: 6),
                    Expanded(
                      child: Text(
                        status['branch'] as String? ??
                            (status['head'] == null
                                ? 'No commits yet'
                                : 'Detached HEAD'),
                        maxLines: 1,
                        overflow: TextOverflow.ellipsis,
                        style: const TextStyle(fontSize: 12),
                      ),
                    ),
                    if (GitHost.rootKey(repositoryRoot!) !=
                        GitHost.rootKey(w!.root))
                      const Tooltip(
                        message: 'Enclosing repository',
                        child: Icon(Icons.folder_copy_outlined, size: 14),
                      ),
                    const SizedBox(width: 6),
                    Text(
                      '${entries.length} changes',
                      style: const TextStyle(fontSize: 11),
                    ),
                  ],
                ),
              ),
            ),
          if (w?.busy == true)
            const LinearProgressIndicator(semanticsLabel: 'Reading repository'),
          if (w?.error != null || git.error != null)
            Padding(
              padding: const EdgeInsets.all(12),
              child: Text(
                w?.error ?? git.error!,
                style: const TextStyle(fontSize: 12),
              ),
            ),
          if (w?.notice != null)
            Padding(
              padding: const EdgeInsets.all(12),
              child: Text(w!.notice!, style: const TextStyle(fontSize: 12)),
            ),
          if (w != null)
            Expanded(
              child: ListView(
                key: PageStorageKey('git-sidebar-${w.root}'),
                padding: const EdgeInsets.only(bottom: 72),
                children: [
                  GitCommitBox(key: ValueKey(w.root), git: git, workspace: w),
                  for (final staged in [true, false]) ...[
                    Builder(
                      builder: (context) {
                        final group = entries
                            .where(
                              (e) => staged
                                  ? e['index'] != ' ' && e['index'] != '?'
                                  : e['worktree'] != ' ',
                            )
                            .toList();
                        return section(
                          staged ? 'Staged Changes' : 'Changes',
                          staged ? w.stagedExpanded : w.changesExpanded,
                          () {
                            if (staged) {
                              w.stagedExpanded = !w.stagedExpanded;
                            } else {
                              w.changesExpanded = !w.changesExpanded;
                            }
                            git.changed();
                          },
                          count: group.length,
                          trailing: WorkspaceSidebarButton(
                            key: Key(
                              staged ? 'git-unstage-all' : 'git-stage-all',
                            ),
                            tooltip: staged
                                ? 'Unstage all changes'
                                : 'Stage all changes',
                            icon: staged ? Icons.remove : Icons.add,
                            onPressed:
                                w.busy || w.reviewOpen != null || group.isEmpty
                                ? null
                                : () => git.changeIndex(w, {
                                    'kind': staged ? 'unstageAll' : 'stageAll',
                                  }),
                          ),
                        );
                      },
                    ),
                    if (staged ? w.stagedExpanded : w.changesExpanded)
                      for (final entry in entries.where(
                        (e) => staged
                            ? e['index'] != ' ' && e['index'] != '?'
                            : e['worktree'] != ' ',
                      ))
                        fileRow(context, w, entry, staged),
                  ],
                  if (entries.isEmpty && status != null)
                    const Padding(
                      padding: EdgeInsets.all(12),
                      child: Text('No changes', style: TextStyle(fontSize: 12)),
                    ),
                  const Divider(height: 12),
                  section(
                    'History',
                    w.historyExpanded,
                    () {
                      w.historyExpanded = !w.historyExpanded;
                      git.changed();
                      if (w.historyExpanded && w.history.isEmpty && !w.busy) {
                        git.loadHistory(w);
                      }
                    },
                    trailing: WorkspaceSidebarButton(
                      tooltip: 'Refresh history',
                      icon: Icons.refresh,
                      onPressed: w.busy ? null : () => git.loadHistory(w),
                    ),
                  ),
                  if (w.historyExpanded) ...[
                    if (w.history.isEmpty)
                      TextButton(
                        onPressed: w.busy ? null : () => git.loadHistory(w),
                        child: const Text('Load history'),
                      ),
                    for (final item in w.history) commitRow(context, w, item),
                    if (w.historyNext != null)
                      TextButton(
                        onPressed: w.busy
                            ? null
                            : () => git.loadHistory(w, more: true),
                        child: const Text('Load more'),
                      ),
                  ],
                ],
              ),
            ),
          if (w == null) const Spacer(),
        ],
      );
    },
  );
}

class SourceControlView extends StatelessWidget {
  final GitHost git;
  const SourceControlView({super.key, required this.git});
  @override
  Widget build(BuildContext context) => ListenableBuilder(
    listenable: git,
    builder: (context, _) {
      final w = git.selected;
      final diff = w?.tabs[w.activeTab];
      if (w == null || diff == null) {
        return Center(
          child: Padding(
            padding: const EdgeInsets.all(24),
            child: Text(
              w == null
                  ? 'Select a project conversation on Home, or open a folder.'
                  : w.error ?? 'Select a change to inspect its saved Git diff.',
              textAlign: TextAlign.center,
            ),
          ),
        );
      }
      final sideBySide = diff['sideBySide'] as bool? ?? w.sideBySide;
      return Column(
        children: [
          SizedBox(
            height: 40,
            child: ListView(
              scrollDirection: Axis.horizontal,
              children: [
                for (final entry in w.tabs.entries)
                  Padding(
                    padding: const EdgeInsets.symmetric(horizontal: 4),
                    child: InputChip(
                      selected: w.activeTab == entry.key,
                      label: Text(
                        '${entry.value['path']} · ${entry.value['basis'] == 'commit' && entry.value['commit'] is String ? (entry.value['commit'] as String).characters.take(8) : entry.value['basis']}',
                      ),
                      onPressed: () {
                        w.activeTab = entry.key;
                        git.changed();
                      },
                      onDeleted: () {
                        w.tabs.remove(entry.key);
                        w.activeTab = w.tabs.keys.lastOrNull;
                        git.changed();
                      },
                    ),
                  ),
              ],
            ),
          ),
          Padding(
            padding: const EdgeInsets.all(8),
            child: Wrap(
              spacing: 12,
              crossAxisAlignment: WrapCrossAlignment.center,
              children: [
                Text(
                  '${diff['path']} · ${diff['leftLabel']} → ${diff['rightLabel']}',
                ),
                SegmentedButton<bool>(
                  segments: const [
                    ButtonSegment(value: false, label: Text('Inline')),
                    ButtonSegment(value: true, label: Text('Side by side')),
                  ],
                  selected: {sideBySide},
                  onSelectionChanged: w.busy
                      ? null
                      : (v) => git.setDiffLayout(w, v.first),
                ),
                TextButton(
                  onPressed: w.busy ? null : () => git.refreshDiff(w),
                  child: const Text('Refresh diff'),
                ),
                if (git.reviewAvailable)
                  TextButton.icon(
                    onPressed: () => showReviewFindings(context, git, w),
                    icon: const Icon(Icons.rate_review_outlined, size: 18),
                    label: const Text('Review findings'),
                  ),
              ],
            ),
          ),
          if (diff['revision'] != w.status?['revision'])
            const Padding(
              padding: EdgeInsets.all(8),
              child: Text(
                'Saved Git state changed. Refresh this comparison before acting.',
              ),
            ),
          if (w.error != null)
            ConstrainedBox(
              constraints: const BoxConstraints(maxHeight: 72),
              child: SingleChildScrollView(
                child: Padding(
                  padding: const EdgeInsets.all(8),
                  child: Text(w.error!),
                ),
              ),
            ),
          if (diff['encodingWarning'] == true)
            const Padding(
              padding: EdgeInsets.all(8),
              child: Text(
                'Some bytes are not UTF-8; replacement characters are shown.',
              ),
            ),
          if (diff['next'] != null || (diff['cursor'] as int? ?? 0) > 0)
            Padding(
              padding: const EdgeInsets.symmetric(horizontal: 8),
              child: Wrap(
                spacing: 12,
                crossAxisAlignment: WrapCrossAlignment.center,
                children: [
                  Text(
                    'Change rows ${(diff['cursor'] as int? ?? 0) + 1}–${(diff['cursor'] as int? ?? 0) + ((sideBySide ? diff['sideRows'] : null) as List? ?? diff['rows'] as List).length} of ${diff['totalRows']}',
                  ),
                  TextButton(
                    key: const Key('git-previous-page'),
                    onPressed: w.busy || (diff['pageTrail'] as List).length < 2
                        ? null
                        : () {
                            final trail = (diff['pageTrail'] as List)
                                .cast<int>();
                            git.loadDiffPage(w, trail[trail.length - 2]);
                          },
                    child: const Text('Previous changes'),
                  ),
                  TextButton(
                    key: const Key('git-next-page'),
                    onPressed: w.busy || diff['next'] == null
                        ? null
                        : () => git.loadDiffPage(w, diff['next'] as int),
                    child: const Text('Next changes'),
                  ),
                ],
              ),
            ),
          if (diff['conflict'] == true)
            const Padding(
              padding: EdgeInsets.all(8),
              child: Text(
                'Unresolved conflict. Edit and save the file, inspect it, then stage deliberately.',
              ),
            ),
          if ((diff['hunks'] as List? ?? []).isNotEmpty &&
              diff['conflict'] != true)
            GitHunkControls(
              key: ValueKey('${w.root}:${w.activeTab}:${diff['revision']}'),
              git: git,
              w: w,
              diff: diff,
              review: (op) => executeGitAction(context, git, w, op),
            ),
          Expanded(
            child: GitDiffView(
              key: ValueKey('${w.root}:${w.activeTab}:${diff['revision']}'),
              diff: diff,
              sideBySide: sideBySide,
              onReview:
                  !git.reviewAvailable ||
                      w.busy ||
                      diff['digest'] == null ||
                      diff['encodingWarning'] == true
                  ? null
                  : (side, line) =>
                        showFindingEditor(context, git, w, diff, side, line),
              findings: git
                  .reviewState(w)
                  .items
                  .values
                  .where((f) => git.findingCurrent(w, f, diff))
                  .toList(),
            ),
          ),
        ],
      );
    },
  );
}
