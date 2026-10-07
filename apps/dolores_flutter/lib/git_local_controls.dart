import 'package:flutter/material.dart';

import 'git_host.dart';

typedef GitReview = Future<void> Function(Map<String, dynamic> operation);

class GitLocalControls extends StatelessWidget {
  final GitHost git;
  final GitWorkspace w;
  final GitReview review;
  const GitLocalControls({
    super.key,
    required this.git,
    required this.w,
    required this.review,
  });

  Future<String?> choose(
    BuildContext context,
    String title,
    List<Map<String, String>> choices,
  ) => showDialog<String>(
    context: context,
    builder: (context) => SimpleDialog(
      title: Text(title),
      children: [
        if (choices.isEmpty)
          const Padding(
            padding: EdgeInsets.all(24),
            child: Text(
              'Nothing available. Configure this using Git, then try again.',
            ),
          ),
        for (final item in choices)
          SimpleDialogOption(
            onPressed: () => Navigator.pop(context, item['id']),
            child: Text(item['label']!),
          ),
        TextButton(
          onPressed: () => Navigator.pop(context),
          child: const Text('Cancel'),
        ),
      ],
    ),
  );

  Future<void> remote(BuildContext context, String kind) async {
    await git.loadRemotes(w);
    if (!context.mounted || w.error != null) return;
    final state = w.remoteState!;
    final remotes = (state['remotes'] as List).cast<String>();
    final eligible = kind == 'fetch'
        ? remotes
        : remotes
              .where(
                (name) => name == state['remote'] && state['branch'] != null,
              )
              .toList();
    if (eligible.isEmpty) {
      w.error = kind == 'fetch'
          ? 'No remote configured. Configure a remote using Git, then try again.'
          : 'Tracking not configured. Set an upstream using Git, then try again.';
      git.changed();
      return;
    }
    final name = await choose(context, 'Choose remote to $kind', [
      for (final name in eligible) {'id': name, 'label': name},
    ]);
    if (name != null) {
      await review({
        'kind': kind,
        'remote': name,
        if (kind != 'fetch') 'branch': state['branch'],
      });
    }
  }

  Future<void> local(BuildContext context, String kind) async {
    await git.loadLocal(w);
    if (!context.mounted || w.error != null) return;
    final branches = kind == 'branchSwitch';
    final items = (w.localState![branches ? 'branches' : 'stashes'] as List)
        .cast<Map>();
    final id = await choose(
      context,
      branches ? 'Switch branch' : 'Choose stash',
      [
        for (final item in items)
          {
            'id': (branches ? item['name'] : item['id']) as String,
            'label':
                (branches
                        ? item['name']
                        : '${item['ref']} · ${item['subject']}')
                    as String,
          },
      ],
    );
    if (id != null) {
      await review(
        branches
            ? {'kind': kind, 'name': id}
            : {'kind': 'stashApply', 'stash': id, 'pop': kind == 'stashPop'},
      );
    }
  }

  Future<void> createBranch(BuildContext context) async {
    String name = '';
    final accepted = await showDialog<bool>(
      context: context,
      builder: (context) => AlertDialog(
        title: const Text('Create branch'),
        content: TextField(
          autofocus: true,
          maxLength: 256,
          onChanged: (value) => name = value,
          decoration: const InputDecoration(labelText: 'Branch name'),
        ),
        actions: [
          TextButton(
            onPressed: () => Navigator.pop(context),
            child: const Text('Cancel'),
          ),
          TextButton(
            onPressed: () => Navigator.pop(context, true),
            child: const Text('Continue'),
          ),
        ],
      ),
    );
    if (accepted == true && name.trim().isNotEmpty) {
      await review({'kind': 'branchCreate', 'name': name});
    }
  }

  @override
  Widget build(BuildContext context) {
    final enabled = !w.busy && w.reviewOpen == null;
    Widget action(String label, VoidCallback callback) => MenuItemButton(
      onPressed: enabled ? callback : null,
      child: Text(label),
    );
    final entries = (w.status?['entries'] as List? ?? []).cast<Map>();
    final working = entries
        .where((e) => e['worktree'] != ' ')
        .map((e) => e['path'] as String)
        .toList();
    final staged = entries
        .where((e) => e['index'] != ' ' && e['index'] != '?')
        .map((e) => e['path'] as String)
        .toList();
    final stashable = entries
        .where(
          (e) =>
              e['worktree'] != ' ' &&
              e['index'] != '?' &&
              e['conflict'] != true,
        )
        .map((e) => e['path'] as String)
        .toList();
    return MenuAnchor(
      menuChildren: [
        action(
          'Commit',
          () => review({'kind': 'commit', 'message': w.commitDraft}),
        ),
        const Divider(),
        action('Pull', () => remote(context, 'pull')),
        action('Push', () => remote(context, 'push')),
        action('Fetch', () => remote(context, 'fetch')),
        const Divider(),
        SubmenuButton(
          menuChildren: [
            MenuItemButton(
              onPressed: enabled && working.isNotEmpty
                  ? () => review({'kind': 'stage', 'paths': working})
                  : null,
              child: const Text('Stage all changes'),
            ),
            MenuItemButton(
              onPressed: enabled && staged.isNotEmpty
                  ? () => review({'kind': 'unstage', 'paths': staged})
                  : null,
              child: const Text('Unstage all changes'),
            ),
          ],
          child: const Text('Changes'),
        ),
        SubmenuButton(
          menuChildren: [
            action('Create branch…', () => createBranch(context)),
            action('Switch branch…', () => local(context, 'branchSwitch')),
          ],
          child: const Text('Branch'),
        ),
        SubmenuButton(
          menuChildren: [
            MenuItemButton(
              onPressed: enabled && stashable.isNotEmpty
                  ? () => review({
                      'kind': 'stashCreate',
                      'paths': stashable,
                      'message': 'Saved workspace changes',
                    })
                  : null,
              child: const Text('Stash changes'),
            ),
            action('Apply stash…', () => local(context, 'stashApply')),
            action('Apply and drop stash…', () => local(context, 'stashPop')),
          ],
          child: const Text('Stash'),
        ),
        if (w.status?['reverting'] == true)
          action('Abort revert', () => review({'kind': 'revertAbort'})),
      ],
      builder: (context, controller, child) => IconButton(
        key: const Key('git-changes-menu'),
        tooltip: 'Git actions',
        icon: const Icon(Icons.more_horiz, size: 20),
        onPressed: enabled
            ? () => controller.isOpen ? controller.close() : controller.open()
            : null,
      ),
    );
  }
}

class GitHunkControls extends StatefulWidget {
  final GitHost git;
  final GitWorkspace w;
  final Map<String, dynamic> diff;
  final GitReview review;
  const GitHunkControls({
    super.key,
    required this.git,
    required this.w,
    required this.diff,
    required this.review,
  });
  @override
  State<GitHunkControls> createState() => _GitHunkControlsState();
}

class _GitHunkControlsState extends State<GitHunkControls> {
  final selected = <String>{};
  @override
  Widget build(BuildContext context) => ExpansionTile(
    title: const Text('Choose hunks'),
    subtitle: const Text(
      'Only displayed text hunks; new, renamed and binary files use whole-file actions.',
    ),
    children: [
      SizedBox(
        height: 120,
        child: ListView(
          children: [
            for (final h in widget.diff['hunks'] as List)
              CheckboxListTile(
                dense: true,
                title: Text(
                  h['header'] as String,
                  style: const TextStyle(fontFamily: 'Consolas', fontSize: 12),
                ),
                value: selected.contains(h['id']),
                onChanged: widget.w.busy
                    ? null
                    : (v) => setState(
                        () => v == true
                            ? selected.add(h['id'] as String)
                            : selected.remove(h['id']),
                      ),
              ),
          ],
        ),
      ),
      TextButton(
        onPressed:
            widget.w.busy ||
                selected.isEmpty ||
                widget.diff['revision'] != widget.w.status?['revision']
            ? null
            : () => widget.review({
                'kind': 'hunks',
                'path': widget.diff['path'],
                'staged': widget.diff['basis'] == 'staged',
                'ids': selected.toList(),
              }),
        child: Text(
          widget.diff['basis'] == 'staged'
              ? 'Review unstage selected hunks'
              : 'Review stage selected hunks',
        ),
      ),
    ],
  );
}
