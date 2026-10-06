import 'package:flutter/material.dart';

import 'git_host.dart';

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
    builder: (context, _) => Center(
      child: Padding(
        padding: const EdgeInsets.all(24),
        child: Text(
          git.selected == null
              ? 'Select a project conversation on Home, or open a folder.'
              : git.selected!.error ??
                    (git.selected!.busy
                        ? 'Reading saved Git state…'
                        : 'Select a change to inspect its saved Git diff.'),
          textAlign: TextAlign.center,
        ),
      ),
    ),
  );
}
