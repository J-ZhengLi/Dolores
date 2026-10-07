import 'package:flutter/material.dart';
import 'package:flutter/services.dart';

import 'git_host.dart';
import 'git_diff_view.dart';
import 'git_local_controls.dart';

String gitActionName(Map<String, dynamic> op) => switch (op['kind']) {
  'stashCreate' => 'stash selected files',
  'stashApply' => op['pop'] == true ? 'apply and drop stash' : 'apply stash',
  'branchCreate' => 'create branch',
  'branchSwitch' => 'switch branch',
  'revertAbort' => 'abort revert',
  'hunks' =>
    op['staged'] == true ? 'unstage selected hunks' : 'stage selected hunks',
  'pull' => 'fast-forward pull',
  _ => op['kind'] as String,
};

Future<void> reviewGitAction(
  BuildContext context,
  GitHost git,
  GitWorkspace w,
  Map<String, dynamic> operation,
) async {
  final preview = await git.review(w, operation);
  if (preview == null) return;
  if (!context.mounted) {
    await git.resolveReview(w, preview, apply: false);
    return;
  }
  final approved = await showDialog<bool>(
    context: context,
    builder: (context) => Dialog(
      child: ConstrainedBox(
        constraints: const BoxConstraints(maxWidth: 860, maxHeight: 620),
        child: Padding(
          padding: const EdgeInsets.all(20),
          child: Column(
            children: [
              Text(
                'Review ${gitActionName(operation)}',
                style: Theme.of(context).textTheme.titleLarge,
              ),
              SelectableText('${preview['root']}'),
              if (preview['author'] != null)
                SelectableText('Author: ${preview['author']}'),
              Text(preview['notice'] as String),
              Expanded(
                child: ListView(
                  children: [
                    for (final path in preview['paths'] as List)
                      Text(path as String),
                    if (operation['message'] != null)
                      SelectableText(operation['message'] as String),
                    SelectableText(
                      preview['patch'] as String,
                      style: const TextStyle(
                        fontFamily: 'Consolas',
                        fontSize: 12,
                      ),
                    ),
                  ],
                ),
              ),
              Wrap(
                spacing: 12,
                children: [
                  TextButton(
                    onPressed: () => Navigator.pop(context, false),
                    child: const Text('Cancel'),
                  ),
                  FilledButton(
                    key: const Key('git-apply-review'),
                    onPressed: () => Navigator.pop(context, true),
                    child: Text('Apply ${gitActionName(operation)}'),
                  ),
                ],
              ),
            ],
          ),
        ),
      ),
    ),
  );
  await git.resolveReview(w, preview, apply: approved == true);
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
  Widget build(BuildContext context) => Padding(
    padding: const EdgeInsets.symmetric(horizontal: 12, vertical: 8),
    child: CallbackShortcuts(
      bindings: {
        const SingleActivator(LogicalKeyboardKey.enter, control: true): () {
          if (!widget.workspace.busy && widget.workspace.reviewOpen == null) {
            reviewGitAction(context, widget.git, widget.workspace, {
              'kind': 'commit',
              'message': controller.text,
            });
          }
        },
      },
      child: TextField(
        key: const Key('git-commit-message'),
        controller: controller,
        minLines: 1,
        maxLines: 4,
        maxLength: 8192,
        decoration: const InputDecoration(
          hintText: 'Message (Ctrl+Enter to commit)',
          counterText: '',
          isDense: true,
          border: OutlineInputBorder(),
        ),
        onChanged: (v) => widget.workspace.commitDraft = v,
      ),
    ),
  );
}

class SourceControlPanel extends StatelessWidget {
  final GitHost git;
  final VoidCallback openFolder;
  const SourceControlPanel({
    super.key,
    required this.git,
    required this.openFolder,
  });

  Widget fileRow(
    BuildContext context,
    GitWorkspace w,
    Map entry,
    bool staged,
  ) => ListTile(
    dense: true,
    visualDensity: VisualDensity.compact,
    contentPadding: const EdgeInsets.only(left: 20, right: 4),
    leading: const Icon(Icons.insert_drive_file_outlined, size: 16),
    title: Tooltip(
      message: entry['path'] as String,
      child: Text(
        entry['path'] as String,
        maxLines: 1,
        overflow: TextOverflow.ellipsis,
      ),
    ),
    subtitle: entry['oldPath'] == null
        ? null
        : Text(
            'From ${entry['oldPath']}',
            maxLines: 1,
            overflow: TextOverflow.ellipsis,
          ),
    onTap: w.busy
        ? null
        : () => git.openDiff(
            w,
            entry['path'] as String,
            staged ? 'staged' : 'working',
          ),
    trailing: Row(
      mainAxisSize: MainAxisSize.min,
      children: [
        Text((staged ? entry['index'] : entry['worktree']) as String),
        PopupMenuButton<String>(
          tooltip: 'File Git actions',
          enabled: !w.busy && w.reviewOpen == null,
          onSelected: (kind) => reviewGitAction(context, git, w, {
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
        ListTile(
          key: ValueKey('git-commit-$id'),
          dense: true,
          visualDensity: VisualDensity.compact,
          contentPadding: const EdgeInsets.only(left: 4, right: 4),
          leading: Icon(
            expanded ? Icons.expand_more : Icons.chevron_right,
            size: 20,
          ),
          title: Tooltip(
            message: item['subject'] as String,
            child: Text(
              item['subject'] as String,
              maxLines: 1,
              overflow: TextOverflow.ellipsis,
            ),
          ),
          subtitle: Text(
            '${id.substring(0, 8)} · ${item['author']}',
            maxLines: 1,
            overflow: TextOverflow.ellipsis,
          ),
          onTap: w.busy ? null : () => git.toggleCommit(w, id),
          trailing: PopupMenuButton<String>(
            tooltip: 'Commit actions',
            enabled: !w.busy && w.reviewOpen == null,
            onSelected: (_) => reviewGitAction(context, git, w, {
              'kind': 'revert',
              'commit': id,
            }),
            itemBuilder: (_) => const [
              PopupMenuItem(value: 'revert', child: Text('Revert commit…')),
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
            ListTile(
              key: ValueKey('git-file-$id:$path'),
              dense: true,
              visualDensity: VisualDensity.compact,
              contentPadding: const EdgeInsets.only(left: 36, right: 12),
              leading: const Icon(Icons.insert_drive_file_outlined, size: 16),
              title: Tooltip(
                message: path as String,
                child: Text(path, maxLines: 1, overflow: TextOverflow.ellipsis),
              ),
              onTap: w.busy
                  ? null
                  : () => git.openDiff(w, path, 'commit', commit: id),
            ),
        ],
      ],
    );
  }

  @override
  Widget build(BuildContext context) => ListenableBuilder(
    listenable: git,
    builder: (context, _) {
      final w = git.selected;
      final status = w?.status;
      final entries = (status?['entries'] as List? ?? []).cast<Map>();
      return Column(
        crossAxisAlignment: CrossAxisAlignment.stretch,
        children: [
          const Padding(
            padding: EdgeInsets.all(16),
            child: Text(
              'SOURCE CONTROL',
              style: TextStyle(fontSize: 12, fontWeight: FontWeight.w600),
            ),
          ),
          if (w == null)
            TextButton(onPressed: openFolder, child: const Text('Open folder')),
          if (w != null)
            Padding(
              padding: const EdgeInsets.symmetric(horizontal: 12),
              child: Tooltip(
                message: status?['root'] as String? ?? w.root,
                child: Text(
                  status?['branch'] as String? ?? 'No commit / detached HEAD',
                  maxLines: 1,
                  overflow: TextOverflow.ellipsis,
                  style: const TextStyle(fontSize: 12),
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
                children: [
                  Row(
                    children: [
                      Expanded(
                        child: InkWell(
                          onTap: () {
                            w.changesExpanded = !w.changesExpanded;
                            git.changed();
                          },
                          child: Padding(
                            padding: const EdgeInsets.symmetric(vertical: 8),
                            child: Row(
                              children: [
                                Icon(
                                  w.changesExpanded
                                      ? Icons.expand_more
                                      : Icons.chevron_right,
                                  size: 20,
                                ),
                                const Text(
                                  'Changes',
                                  style: TextStyle(fontWeight: FontWeight.w600),
                                ),
                                const SizedBox(width: 8),
                                Text(
                                  entries.length.toString(),
                                  style: const TextStyle(fontSize: 12),
                                ),
                              ],
                            ),
                          ),
                        ),
                      ),
                      IconButton(
                        tooltip: 'Refresh',
                        icon: const Icon(Icons.refresh, size: 18),
                        onPressed: w.busy ? null : () => git.refresh(w),
                      ),
                      if (w.busy)
                        IconButton(
                          tooltip: 'Stop Git operation',
                          icon: const Icon(
                            Icons.stop_circle_outlined,
                            size: 18,
                          ),
                          onPressed: () => git.stop(w),
                        ),
                      if (status != null)
                        GitLocalControls(
                          git: git,
                          w: w,
                          review: (op) => reviewGitAction(context, git, w, op),
                        ),
                    ],
                  ),
                  if (w.changesExpanded) ...[
                    GitCommitBox(key: ValueKey(w.root), git: git, workspace: w),
                    for (final staged in [true, false]) ...[
                      Padding(
                        padding: const EdgeInsets.fromLTRB(20, 12, 12, 4),
                        child: Text(
                          staged ? 'Staged Changes' : 'Working Changes',
                          style: const TextStyle(
                            fontSize: 12,
                            fontWeight: FontWeight.w600,
                          ),
                        ),
                      ),
                      for (final entry in entries.where(
                        (e) => staged
                            ? e['index'] != ' ' && e['index'] != '?'
                            : e['worktree'] != ' ',
                      ))
                        fileRow(context, w, entry, staged),
                    ],
                    if (entries.isEmpty && status != null)
                      const Padding(
                        padding: EdgeInsets.all(16),
                        child: Text(
                          'No changes',
                          style: TextStyle(fontSize: 12),
                        ),
                      ),
                  ],
                  const Divider(),
                  Row(
                    children: [
                      Expanded(
                        child: InkWell(
                          onTap: () {
                            w.historyExpanded = !w.historyExpanded;
                            git.changed();
                            if (w.historyExpanded &&
                                w.history.isEmpty &&
                                !w.busy) {
                              git.loadHistory(w);
                            }
                          },
                          child: Padding(
                            padding: const EdgeInsets.symmetric(vertical: 8),
                            child: Row(
                              children: [
                                Icon(
                                  w.historyExpanded
                                      ? Icons.expand_more
                                      : Icons.chevron_right,
                                  size: 20,
                                ),
                                const Text(
                                  'History',
                                  style: TextStyle(fontWeight: FontWeight.w600),
                                ),
                              ],
                            ),
                          ),
                        ),
                      ),
                      IconButton(
                        tooltip: 'Refresh history',
                        icon: const Icon(Icons.refresh, size: 18),
                        onPressed: w.busy ? null : () => git.loadHistory(w),
                      ),
                    ],
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
                        '${entry.value['path']} · ${entry.value['basis']}',
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
                  selected: {w.sideBySide},
                  onSelectionChanged: (v) {
                    w.sideBySide = v.first;
                    git.changed();
                  },
                ),
                TextButton(
                  onPressed: w.busy
                      ? null
                      : () => git.openDiff(
                          w,
                          diff['path'] as String,
                          diff['basis'] as String,
                          commit: diff['commit'] as String?,
                        ),
                  child: const Text('Refresh diff'),
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
              review: (op) => reviewGitAction(context, git, w, op),
            ),
          Expanded(
            child: GitDiffView(
              key: ValueKey('${w.root}:${w.activeTab}:${diff['revision']}'),
              diff: diff,
              sideBySide: w.sideBySide,
            ),
          ),
        ],
      );
    },
  );
}
