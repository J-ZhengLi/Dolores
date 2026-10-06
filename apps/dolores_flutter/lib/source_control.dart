import 'package:flutter/material.dart';

import 'git_host.dart';
import 'git_diff_view.dart';
import 'git_local_controls.dart';

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
                'Review ${operation['kind']}',
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
                    child: Text('${operation['kind']} once'),
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
    padding: const EdgeInsets.all(12),
    child: Column(
      children: [
        TextField(
          key: const Key('git-commit-message'),
          controller: controller,
          minLines: 2,
          maxLines: 4,
          maxLength: 8192,
          decoration: const InputDecoration(hintText: 'Commit message'),
          onChanged: (v) => widget.workspace.commitDraft = v,
        ),
        TextButton(
          onPressed: widget.workspace.busy
              ? null
              : () => reviewGitAction(context, widget.git, widget.workspace, {
                  'kind': 'commit',
                  'message': controller.text,
                }),
          child: const Text('Review commit'),
        ),
      ],
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
  @override
  Widget build(BuildContext context) => ListenableBuilder(
    listenable: git,
    builder: (context, _) {
      final w = git.selected;
      final status = w?.status;
      final entries = (status?['entries'] as List? ?? []).cast<Map>();
      return Column(
        children: [
          Padding(
            padding: const EdgeInsets.all(12),
            child: Row(
              children: [
                const Expanded(
                  child: Text('Source Control', style: TextStyle(fontSize: 18)),
                ),
                if (w != null)
                  IconButton(
                    tooltip: 'Refresh',
                    onPressed: w.busy ? null : () => git.refresh(w),
                    icon: const Icon(Icons.refresh),
                  ),
                if (w?.busy == true)
                  IconButton(
                    tooltip: 'Stop Git operation',
                    onPressed: () => git.stop(w!),
                    icon: const Icon(Icons.stop_circle_outlined),
                  ),
              ],
            ),
          ),
          if (w == null)
            TextButton(onPressed: openFolder, child: const Text('Open folder')),
          if (w != null)
            Padding(
              padding: const EdgeInsets.symmetric(horizontal: 12),
              child: SelectableText(
                '${status?['root'] ?? w.root}\n${status?['branch'] ?? 'No commit / detached HEAD'}',
                style: const TextStyle(fontSize: 12),
              ),
            ),
          if (w?.busy == true)
            LinearProgressIndicator(semanticsLabel: 'Reading repository'),
          if (w?.error != null || git.error != null)
            Padding(
              padding: const EdgeInsets.all(12),
              child: Text(w?.error ?? git.error!),
            ),
          if (w?.notice != null)
            Padding(padding: const EdgeInsets.all(12), child: Text(w!.notice!)),
          Expanded(
            child: ListView(
              children: [
                if (w != null)
                  GitCommitBox(key: ValueKey(w.root), git: git, workspace: w),
                if (w?.status != null)
                  GitLocalControls(
                    git: git,
                    w: w!,
                    review: (op) => reviewGitAction(context, git, w, op),
                  ),
                for (final staged in [true, false]) ...[
                  Padding(
                    padding: const EdgeInsets.all(12),
                    child: Text(
                      staged ? 'Staged' : 'Changes',
                      style: const TextStyle(fontWeight: FontWeight.bold),
                    ),
                  ),
                  for (final entry in entries.where(
                    (e) => staged
                        ? e['index'] != ' ' && e['index'] != '?'
                        : e['worktree'] != ' ',
                  ))
                    ListTile(
                      onTap: w == null || w.busy
                          ? null
                          : () => git.openDiff(
                              w,
                              entry['path'] as String,
                              staged ? 'staged' : 'working',
                            ),
                      dense: true,
                      title: Text(entry['path'] as String),
                      subtitle: entry['oldPath'] == null
                          ? null
                          : Text('From ${entry['oldPath']}'),
                      trailing: Row(
                        mainAxisSize: MainAxisSize.min,
                        children: [
                          Text(
                            (staged ? entry['index'] : entry['worktree'])
                                as String,
                          ),
                          PopupMenuButton<String>(
                            tooltip: 'File Git actions',
                            enabled: w != null && !w.busy,
                            onSelected: (kind) =>
                                reviewGitAction(context, git, w!, {
                                  'kind': kind,
                                  if (kind == 'discard')
                                    'path': entry['path']
                                  else
                                    'paths': [entry['path']],
                                  if (kind == 'stashCreate')
                                    'message': 'Saved ${entry['path']}',
                                }),
                            itemBuilder: (_) => [
                              PopupMenuItem(
                                value: staged ? 'unstage' : 'stage',
                                child: Text(
                                  staged ? 'Review unstage' : 'Review stage',
                                ),
                              ),
                              if (!staged &&
                                  entry['index'] != '?' &&
                                  entry['conflict'] != true) ...[
                                const PopupMenuItem(
                                  value: 'stashCreate',
                                  child: Text('Review stash this file'),
                                ),
                                const PopupMenuItem(
                                  value: 'discard',
                                  child: Text('Review discard saved changes'),
                                ),
                              ],
                            ],
                          ),
                        ],
                      ),
                    ),
                ],
                TextButton(
                  onPressed: w == null || w.busy
                      ? null
                      : () => git.loadHistory(w),
                  child: const Text('History'),
                ),
                if (w != null)
                  for (final item in w.history)
                    ListTile(
                      dense: true,
                      title: Text(item['subject'] as String),
                      subtitle: Text(
                        '${(item['id'] as String).substring(0, 8)} · ${item['author']}',
                      ),
                      onTap: w.busy
                          ? null
                          : () => git.commitFiles(w, item['id'] as String),
                      trailing: PopupMenuButton<String>(
                        tooltip: 'Commit actions',
                        enabled: !w.busy,
                        onSelected: (_) => reviewGitAction(context, git, w, {
                          'kind': 'revert',
                          'commit': item['id'],
                        }),
                        itemBuilder: (_) => const [
                          PopupMenuItem(
                            value: 'revert',
                            child: Text('Review revert with a new commit'),
                          ),
                        ],
                      ),
                    ),
                if (w?.historyNext != null)
                  TextButton(
                    onPressed: w!.busy
                        ? null
                        : () => git.loadHistory(w, more: true),
                    child: const Text('Load more'),
                  ),
                if (w?.historyFiles != null)
                  for (final path in w!.historyFiles!['files'] as List)
                    ListTile(
                      dense: true,
                      title: Text(path as String),
                      subtitle: Text(
                        'Commit ${(w.historyFiles!['commit'] as String).substring(0, 8)}',
                      ),
                      onTap: w.busy
                          ? null
                          : () => git.openDiff(
                              w,
                              path,
                              'commit',
                              commit: w.historyFiles!['commit'] as String,
                            ),
                    ),
              ],
            ),
          ),
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
