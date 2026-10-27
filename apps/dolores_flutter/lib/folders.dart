import 'form_field_layout.dart';

import 'package:flutter/material.dart';

import 'file_host.dart';
import 'file_editor.dart';
import 'app_host.dart';
import 'workspace_sidebar.dart';

Future<String?> filePathDialog(
  BuildContext context,
  String title, {
  String initial = '',
  String label = 'Relative path in this project',
}) async {
  final result = await showDialog<String>(
    context: context,
    builder: (context) =>
        _PathDialog(title: title, initial: initial, label: label),
  );
  return result == null || result.trim().isEmpty ? null : result.trim();
}

class _PathDialog extends StatefulWidget {
  final String title, initial, label;
  const _PathDialog({
    required this.title,
    required this.initial,
    required this.label,
  });
  @override
  State<_PathDialog> createState() => _PathDialogState();
}

class _PathDialogState extends State<_PathDialog> {
  late final input = TextEditingController(text: widget.initial);
  @override
  void dispose() {
    input.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) => AlertDialog(
    title: Text(widget.title),
    content: FieldLabel(
      label: widget.label,
      child: TextField(
        controller: input,
        autofocus: true,
        decoration: InputDecoration(),
        onSubmitted: (v) => Navigator.pop(context, v),
      ),
    ),
    actions: [
      TextButton(
        onPressed: () => Navigator.pop(context),
        child: const Text('Cancel'),
      ),
      TextButton(
        onPressed: () => Navigator.pop(context, input.text),
        child: const Text('Continue'),
      ),
    ],
  );
}

Future<void> quickOpen(BuildContext context, FileHost files) async {
  final w = files.selected;
  if (w == null) return;
  final path = await filePathDialog(
    context,
    'Quick open',
    label: 'Relative file path in this project',
  );
  if (path != null) await files.open(w, path);
}

class FolderTree extends StatelessWidget {
  final FileHost files;
  final VoidCallback openFolder;
  const FolderTree({super.key, required this.files, required this.openFolder});

  List<(Map<String, dynamic>?, String, int)> rows(
    FileWorkspace w,
    String path,
    int depth,
  ) {
    final page = w.tree[path];
    if (page == null || !w.expanded.contains(path)) return [];
    // Sort only loaded siblings: keep directory paging lazy and its cursor intact.
    final entries = [...page.entries]
      ..sort((a, b) {
        final directory =
            (b['directory'] == true ? 1 : 0) - (a['directory'] == true ? 1 : 0);
        if (directory != 0) return directory;
        final aName = a['name'] as String, bName = b['name'] as String;
        final name = aName.toLowerCase().compareTo(bName.toLowerCase());
        return name == 0 ? aName.compareTo(bName) : name;
      });
    return [
      for (final entry in entries) ...[
        (entry, path, depth),
        if (entry['directory'] == true && w.expanded.contains(entry['path']))
          ...rows(w, entry['path'] as String, depth + 1),
      ],
      (null, path, depth),
    ];
  }

  @override
  Widget build(BuildContext context) => AnimatedBuilder(
    animation: files,
    builder: (context, _) {
      final w = files.selected;
      final items = w == null
          ? <(Map<String, dynamic>?, String, int)>[]
          : rows(w, '.', 0);
      final project = w?.root
          .replaceAll('\\', '/')
          .split('/')
          .where((part) => part.isNotEmpty)
          .lastOrNull;
      return Column(
        crossAxisAlignment: CrossAxisAlignment.stretch,
        children: [
          Padding(
            padding: const EdgeInsets.fromLTRB(16, 8, 8, 8),
            child: Row(
              children: [
                const Expanded(
                  child: Text(
                    'EXPLORER',
                    maxLines: 1,
                    overflow: TextOverflow.ellipsis,
                    style: TextStyle(fontSize: 11, fontWeight: FontWeight.w600),
                  ),
                ),
                if (w != null) ...[
                  WorkspaceSidebarButton(
                    tooltip: 'Quick open (Ctrl+P)',
                    icon: Icons.search,
                    onPressed: () => quickOpen(context, files),
                  ),
                  WorkspaceSidebarButton(
                    tooltip: 'New file',
                    icon: Icons.note_add_outlined,
                    onPressed: () async {
                      final path = await filePathDialog(context, 'Create file');
                      if (path == null) return;
                      await files.open(w, path, action: 'create');
                      await files.load(w, '.', refresh: true);
                    },
                  ),
                  WorkspaceSidebarButton(
                    tooltip: 'Refresh files',
                    icon: Icons.refresh,
                    onPressed: () async {
                      await files.refreshDocuments(w);
                      await files.load(w, '.', refresh: true);
                    },
                  ),
                  WorkspaceSidebarButton(
                    tooltip: 'Collapse folders',
                    icon: Icons.unfold_less,
                    onPressed: () {
                      w.expanded
                        ..clear()
                        ..add('.');
                      files.changed();
                    },
                  ),
                ],
              ],
            ),
          ),
          if (w == null)
            TextButton(onPressed: openFolder, child: const Text('Open folder')),
          if (w != null)
            Tooltip(
              message: w.root,
              child: WorkspaceSidebarRow(
                key: const Key('files-project-root'),
                onTap: () => files.expand(w, '.'),
                child: Row(
                  children: [
                    Icon(
                      w.expanded.contains('.')
                          ? Icons.expand_more
                          : Icons.chevron_right,
                      size: 16,
                    ),
                    const SizedBox(width: 4),
                    Expanded(
                      child: Text(
                        (project ?? w.root).toUpperCase(),
                        maxLines: 1,
                        overflow: TextOverflow.ellipsis,
                        style: const TextStyle(
                          fontSize: 11,
                          fontWeight: FontWeight.w600,
                        ),
                      ),
                    ),
                  ],
                ),
              ),
            ),
          if (files.loading) const LinearProgressIndicator(),
          if (w != null)
            Expanded(
              child: ListView.builder(
                key: PageStorageKey('files-tree-${w.project}'),
                padding: const EdgeInsets.only(bottom: 72),
                itemCount: w.recovery.length + items.length,
                itemBuilder: (context, index) {
                  if (index < w.recovery.length) {
                    final path = w.recovery[index];
                    return WorkspaceSidebarRow(
                      onTap: () => files.open(w, path, action: 'recover'),
                      child: Row(
                        children: [
                          const Icon(Icons.restore, size: 16),
                          const SizedBox(width: 6),
                          Expanded(
                            child: Text(
                              'Recover $path',
                              maxLines: 1,
                              overflow: TextOverflow.ellipsis,
                            ),
                          ),
                        ],
                      ),
                    );
                  }
                  final (entry, path, depth) = items[index - w.recovery.length];
                  if (entry == null) {
                    final page = w.tree[path]!;
                    return Column(
                      children: [
                        if (page.error != null)
                          Padding(
                            padding: const EdgeInsets.all(12),
                            child: Text(page.error!),
                          ),
                        if (page.pending) const LinearProgressIndicator(),
                        if (page.cursor != null)
                          TextButton(
                            onPressed: page.pending
                                ? null
                                : () => files.load(w, path),
                            child: Text(
                              page.error == null ? 'Load more' : 'Retry',
                            ),
                          ),
                      ],
                    );
                  }
                  final directory = entry['directory'] == true;
                  final entryPath = entry['path'] as String;
                  final expanded = w.expanded.contains(entryPath);
                  final selected = !directory && w.paths[w.active] == entryPath;
                  return Tooltip(
                    message: entryPath,
                    child: Semantics(
                      button: true,
                      enabled: true,
                      selected: selected,
                      child: WorkspaceSidebarRow(
                        key: ValueKey('file-tree-$entryPath'),
                        indent: depth.clamp(0, 12) * 12,
                        selected: selected,
                        onTap: () => directory
                            ? files.expand(w, entryPath)
                            : files.open(w, entryPath),
                        onDoubleTap: directory
                            ? null
                            : () => files.open(w, entryPath, preview: false),
                        child: Row(
                          children: [
                            SizedBox(
                              width: 16,
                              child: directory
                                  ? Icon(
                                      expanded
                                          ? Icons.expand_more
                                          : Icons.chevron_right,
                                      size: 16,
                                    )
                                  : null,
                            ),
                            Icon(
                              directory
                                  ? (expanded
                                        ? Icons.folder_open_outlined
                                        : Icons.folder_outlined)
                                  : Icons.description_outlined,
                              size: 16,
                            ),
                            const SizedBox(width: 6),
                            Expanded(
                              child: Text(
                                entry['name'] as String,
                                maxLines: 1,
                                overflow: TextOverflow.ellipsis,
                              ),
                            ),
                          ],
                        ),
                      ),
                    ),
                  );
                },
              ),
            ),
        ],
      );
    },
  );
}

class FolderPreview extends StatelessWidget {
  final AppHost host;
  final FileHost files;
  const FolderPreview({super.key, required this.files, required this.host});
  @override
  Widget build(BuildContext context) => AnimatedBuilder(
    animation: files,
    builder: (context, _) {
      final d = files.active;
      return Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          if (files.error != null)
            Padding(
              padding: const EdgeInsets.all(16),
              child: Text(files.error!),
            ),
          if (d != null)
            Padding(
              padding: const EdgeInsets.all(12),
              child: Row(
                children: [
                  Expanded(child: Text(d.path)),
                  PopupMenuButton<String>(
                    tooltip: 'File actions',
                    onSelected: (action) async {
                      final w = files.selected!;
                      if (action == 'rename') {
                        final path = await filePathDialog(
                          context,
                          'Rename file',
                          initial: d.path,
                        );
                        if (path != null) {
                          await files.action(w, d, 'rename', path: path);
                        }
                      } else {
                        final confirmed = await showDialog<bool>(
                          context: context,
                          builder: (context) => AlertDialog(
                            title: Text('Delete ${d.path}?'),
                            content: const Text(
                              'This deletes the saved file from this project. Dirty documents must be saved or closed first.',
                            ),
                            actions: [
                              TextButton(
                                onPressed: () => Navigator.pop(context, false),
                                child: const Text('Cancel'),
                              ),
                              TextButton(
                                onPressed: () => Navigator.pop(context, true),
                                child: const Text('Delete'),
                              ),
                            ],
                          ),
                        );
                        if (confirmed == true) {
                          await files.action(w, d, 'delete');
                        }
                      }
                    },
                    itemBuilder: (_) => const [
                      PopupMenuItem(value: 'rename', child: Text('Rename')),
                      PopupMenuItem(value: 'delete', child: Text('Delete')),
                    ],
                  ),
                  IconButton(
                    tooltip: 'Close file',
                    onPressed: () => files.close(files.selected!, d),
                    icon: const Icon(Icons.close, size: 18),
                  ),
                ],
              ),
            ),
          if (d?.snapshot['reason'] != null)
            Padding(
              padding: const EdgeInsets.all(12),
              child: Text(
                '${d!.snapshot['reason']} Read-only preview; at most 64 KiB and 2,000 characters per displayed line.',
              ),
            ),
          Expanded(
            child: d == null
                ? const Center(child: Text('Select a file in the folder tree.'))
                : FileEditor(
                    key: ValueKey(d.id),
                    host: host,
                    workspace: files.selected!,
                    document: d,
                  ),
          ),
        ],
      );
    },
  );
}
