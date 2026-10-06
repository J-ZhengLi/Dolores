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
  Future<void> createBranch(BuildContext context) async {
    final controller = TextEditingController();
    final name = await showDialog<String>(
      context: context,
      builder: (context) => AlertDialog(
        title: const Text('Create branch'),
        content: TextField(
          controller: controller,
          autofocus: true,
          maxLength: 256,
          decoration: const InputDecoration(labelText: 'Branch name'),
        ),
        actions: [
          TextButton(
            onPressed: () => Navigator.pop(context),
            child: const Text('Cancel'),
          ),
          TextButton(
            onPressed: () => Navigator.pop(context, controller.text),
            child: const Text('Review'),
          ),
        ],
      ),
    );
    // The route owns its field until its closing animation finishes.
    await Future<void>.delayed(const Duration(milliseconds: 300));
    controller.dispose();
    if (name != null && name.isNotEmpty) {
      await review({'kind': 'branchCreate', 'name': name});
    }
  }

  @override
  Widget build(BuildContext context) => Column(
    children: [
      Wrap(
        children: [
          TextButton(
            onPressed: w.busy ? null : () => git.loadLocal(w),
            child: const Text('Branches & stashes'),
          ),
          TextButton(
            onPressed: w.busy ? null : () => createBranch(context),
            child: const Text('New branch'),
          ),
        ],
      ),
      TextButton(
        onPressed: w.busy ? null : () => git.loadRemotes(w),
        child: const Text('Remotes & tracking'),
      ),
      if (w.remoteState != null) ...[
        Text(
          w.remoteState!['ahead'] == null
              ? 'Tracking not configured. Set it using external Git, then Refresh.'
              : '${w.remoteState!['ahead']} ahead · ${w.remoteState!['behind']} behind',
        ),
        for (final remote in w.remoteState!['remotes'] as List)
          ListTile(
            dense: true,
            title: Text(remote as String),
            trailing: PopupMenuButton<String>(
              tooltip: 'Reviewed remote actions',
              enabled: !w.busy,
              onSelected: (kind) => review({
                'kind': kind,
                'remote': remote,
                if (kind != 'fetch') 'branch': w.remoteState!['branch'],
              }),
              itemBuilder: (_) => [
                const PopupMenuItem(
                  value: 'fetch',
                  child: Text('Review fetch'),
                ),
                if (w.remoteState!['remote'] == remote &&
                    w.remoteState!['branch'] != null) ...[
                  const PopupMenuItem(
                    value: 'pull',
                    child: Text('Review fast-forward pull'),
                  ),
                  const PopupMenuItem(
                    value: 'push',
                    child: Text('Review push'),
                  ),
                ],
              ],
            ),
          ),
      ],
      if (w.status?['reverting'] == true)
        TextButton(
          onPressed: w.busy ? null : () => review({'kind': 'revertAbort'}),
          child: const Text('Review abort revert'),
        ),
      for (final branch in w.localState?['branches'] as List? ?? [])
        ListTile(
          dense: true,
          title: Text(branch['name'] as String),
          trailing: IconButton(
            tooltip: 'Review switch branch',
            onPressed: w.busy
                ? null
                : () =>
                      review({'kind': 'branchSwitch', 'name': branch['name']}),
            icon: const Icon(Icons.call_split, size: 18),
          ),
        ),
      for (final stash in w.localState?['stashes'] as List? ?? [])
        ListTile(
          dense: true,
          title: Text(stash['subject'] as String),
          subtitle: Text(stash['ref'] as String),
          trailing: PopupMenuButton<bool>(
            tooltip: 'Stash actions',
            enabled: !w.busy,
            onSelected: (pop) => review({
              'kind': 'stashApply',
              'stash': stash['id'],
              'pop': pop,
            }),
            itemBuilder: (_) => const [
              PopupMenuItem(value: false, child: Text('Review apply')),
              PopupMenuItem(value: true, child: Text('Review apply and drop')),
            ],
          ),
        ),
    ],
  );
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
