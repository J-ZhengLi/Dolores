import 'package:flutter/material.dart';

import 'git_host.dart';
import 'git_diff_view.dart';

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
          Expanded(
            child: ListView(
              children: [
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
                      trailing: Text(
                        entry['conflict'] == true
                            ? 'Conflict'
                            : (staged ? entry['index'] : entry['worktree'])
                                  as String,
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
