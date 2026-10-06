import 'dart:async';

import 'package:flutter/material.dart';
import 'package:flutter/services.dart';

import 'app_host.dart';
import 'file_host.dart';
import 'file_layout.dart';
import 'file_editor.dart';
import 'folders.dart' show filePathDialog;
import 'theme.dart';

class FileTabDrag {
  final String project, document, group;
  FileTabDrag(this.project, this.document, this.group);
}

class FileWorkspaceView extends StatefulWidget {
  final AppHost host;
  const FileWorkspaceView({super.key, required this.host});
  @override
  State<FileWorkspaceView> createState() => _FileWorkspaceViewState();
}

class _FileWorkspaceViewState extends State<FileWorkspaceView> {
  FileHost get files => widget.host.files;
  FileTabDrag? dragging;
  void drop(VoidCallback action) {
    setState(() => dragging = null);
    action();
  }

  FileWorkspace? get workspace => files.selected;
  Future<void> closeTab(FileWorkspace w, FileGroup g, String id) async {
    final d = files.documents[id];
    if (w.layoutOwner.visibleElsewhere(g.id, id)) {
      w.layoutOwner.remove(g.id, id);
      return;
    }
    if (d == null) {
      w.layoutOwner.remove(g.id, id);
      w.paths.remove(id);
      return;
    }
    String? result;
    if (d.dirty) {
      result = await showDialog<String>(
        context: context,
        builder: (context) => AlertDialog(
          title: Text('Unsaved ${d.path}'),
          content: const Text(
            'Keep this document open, save it, or discard its unsaved edits.',
          ),
          actions: [
            TextButton(
              onPressed: () => Navigator.pop(context),
              child: const Text('Keep editing'),
            ),
            TextButton(
              onPressed: () => Navigator.pop(context, 'discard'),
              child: const Text('Discard'),
            ),
            TextButton(
              onPressed: () => Navigator.pop(context, 'save'),
              child: const Text('Save'),
            ),
          ],
        ),
      );
      if (result == null) return;
      if (result == 'save' && !await files.save(w, d)) return;
    }
    await files.close(w, d, discard: result == 'discard');
  }

  void split(
    FileWorkspace w,
    FileGroup g,
    Axis axis, {
    String? doc,
    bool before = false,
    String? source,
  }) {
    final id = doc ?? g.active;
    if (id == null) return;
    if (!w.layoutOwner.split(g.id, id, axis, before: before, source: source)) {
      files.error = 'Four file groups are already open. Close a split before adding another; tabs and edits are retained.';
      files.changed();
    }
  }

  Widget tab(FileWorkspace w, FileGroup g, String id, int index) {
    final d = files.documents[id];
    Widget label() => Padding(
      padding: const EdgeInsets.only(left: 12, right: 4),
      child: Row(
        mainAxisSize: MainAxisSize.min,
        children: [
          ConstrainedBox(
            constraints: const BoxConstraints(maxWidth: 150),
            child: Text(
              (d?.path ?? w.paths[id] ?? 'File').split('/').last,
              overflow: TextOverflow.ellipsis,
              style: TextStyle(
                fontSize: 13,
                fontStyle: g.preview == id
                    ? FontStyle.italic
                    : FontStyle.normal,
              ),
            ),
          ),
          if (d?.dirty == true)
            const Padding(
              padding: EdgeInsets.symmetric(horizontal: 4),
              child: Icon(Icons.circle, size: 7),
            ),
        ],
      ),
    );
    Widget target() => DragTarget<FileTabDrag>(
      onWillAcceptWithDetails: (detail) => detail.data.project == w.project,
      onAcceptWithDetails: (detail) => drop(
        () => w.layoutOwner.move(
          detail.data.document,
          detail.data.group,
          g.id,
          index: index,
        ),
      ),
      builder: (context, candidates, rejected) => Container(
        key: Key('file-tab-${g.id}-$id'),
        height: 38,
        decoration: BoxDecoration(
          color: g.active == id
              ? Theme.of(context).colorScheme.surfaceContainerHigh
              : null,
          border: Border(
            bottom: BorderSide(
              color: g.active == id
                  ? Theme.of(context).colorScheme.primary
                  : Colors.transparent,
              width: 2,
            ),
          ),
        ),
        child: Row(
          mainAxisSize: MainAxisSize.min,
          children: [
            GestureDetector(
              onTap: () => files.activate(w, g.id, id),
              onDoubleTap: () => w.layoutOwner.pin(g.id, id),
              child: Draggable<FileTabDrag>(
                data: FileTabDrag(w.project, id, g.id),
                onDragStarted: () =>
                    setState(() => dragging = FileTabDrag(w.project, id, g.id)),
                onDragEnd: (_) {
                  if (mounted) setState(() => dragging = null);
                },
                feedback: Material(
                  elevation: 5,
                  child: SizedBox(
                    width: 180,
                    height: 38,
                    child: Center(
                      child: Text(
                        (d?.path ?? w.paths[id] ?? 'File').split('/').last,
                      ),
                    ),
                  ),
                ),
                childWhenDragging: Opacity(opacity: .35, child: label()),
                child: SizedBox(height: 38, child: label()),
              ),
            ),
            IconButton(
              tooltip: 'Close file',
              onPressed: () => unawaited(closeTab(w, g, id)),
              constraints: const BoxConstraints.tightFor(width: 28, height: 28),
              padding: EdgeInsets.zero,
              icon: const Icon(Icons.close, size: 15),
            ),
          ],
        ),
      ),
    );
    return d == null
        ? target()
        : AnimatedBuilder(animation: d, builder: (context, _) => target());
  }

  Widget edge(FileWorkspace w, FileGroup g, Axis axis, bool before) =>
      DragTarget<FileTabDrag>(
        onWillAcceptWithDetails: (detail) =>
            detail.data.project == w.project && w.layoutOwner.groups.length < 4,
        onAcceptWithDetails: (detail) => drop(
          () => split(
            w,
            g,
            axis,
            doc: detail.data.document,
            before: before,
            source: detail.data.group,
          ),
        ),
        builder: (context, candidates, rejected) => IgnorePointer(
          ignoring: dragging == null,
          child: Container(
            decoration: BoxDecoration(
              color: candidates.isNotEmpty
                  ? Theme.of(context).colorScheme.primary.withValues(alpha: .15)
                  : Colors.transparent,
              border: candidates.isNotEmpty
                  ? Border.all(color: Theme.of(context).colorScheme.primary)
                  : null,
            ),
            child: candidates.isEmpty
                ? null
                : Center(
                    child: Text(
                      axis == Axis.horizontal
                          ? 'Split beside'
                          : 'Split above/below',
                    ),
                  ),
          ),
        ),
      );
  Future<void> fileAction(FileWorkspace w, FileGroup g, String action) async {
    final d = files.documents[g.active];
    if (d == null) return;
    if (action == 'rename') {
      final path = await filePathDialog(
        context,
        'Rename file',
        initial: d.path,
      );
      if (path != null) await files.action(w, d, 'rename', path: path);
    } else if (action == 'delete') {
      final confirmed = await showDialog<bool>(
        context: context,
        builder: (context) => AlertDialog(
          title: Text('Delete ${d.path}?'),
          content: const Text(
            'This deletes the saved file. Dirty views must be saved or closed first.',
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
      if (confirmed == true) await files.action(w, d, 'delete');
    }
  }

  Widget group(FileWorkspace w, FileGroup g) {
    final d = files.documents[g.active];
    final active = w.layoutOwner.activeGroup == g.id;
    final p = Palette(Theme.of(context).brightness == Brightness.dark);
    return Container(
      key: Key('file-group-${g.id}'),
      decoration: BoxDecoration(
        border: Border.all(color: active ? p.border : Colors.transparent),
      ),
      child: LayoutBuilder(
        builder: (context, c) => Stack(
          children: [
            DragTarget<FileTabDrag>(
              onWillAcceptWithDetails: (detail) =>
                  detail.data.project == w.project,
              onAcceptWithDetails: (detail) => drop(
                () => w.layoutOwner.move(
                  detail.data.document,
                  detail.data.group,
                  g.id,
                ),
              ),
              builder: (context, candidates, rejected) => Column(
                children: [
                  SizedBox(
                    height: 38,
                    child: Row(
                      children: [
                        Expanded(
                          child: SingleChildScrollView(
                            scrollDirection: Axis.horizontal,
                            child: Row(
                              children: [
                                for (var i = 0; i < g.tabs.length; i++)
                                  tab(w, g, g.tabs[i], i),
                              ],
                            ),
                          ),
                        ),
                        PopupMenuButton<String>(
                          tooltip: 'File group actions',
                          onSelected: (action) {
                            switch (action) {
                              case 'right':
                                split(w, g, Axis.horizontal);
                              case 'down':
                                split(w, g, Axis.vertical);
                              case 'close':
                                w.layoutOwner.closeGroup(g.id);
                              case 'pin':
                                if (g.active != null) {
                                  w.layoutOwner.pin(g.id, g.active!);
                                }
                              case 'move':
                                final next = w.layoutOwner.groups.values
                                    .where((x) => x.id != g.id)
                                    .firstOrNull;
                                if (next != null && g.active != null) {
                                  w.layoutOwner.move(g.active!, g.id, next.id);
                                }
                              case 'rename' || 'delete':
                                unawaited(fileAction(w, g, action));
                              case 'retry':
                                unawaited(files.persistLayout(w));
                            }
                          },
                          itemBuilder: (_) => [
                            PopupMenuItem(
                              value: 'right',
                              enabled: d != null,
                              child: const Text('Split right (Ctrl+\\)'),
                            ),
                            PopupMenuItem(
                              value: 'down',
                              enabled: d != null,
                              child: const Text('Split down'),
                            ),
                            PopupMenuItem(
                              value: 'move',
                              enabled:
                                  g.active != null &&
                                  w.layoutOwner.groups.length > 1,
                              child: const Text('Move tab to next group'),
                            ),
                            PopupMenuItem(
                              value: 'pin',
                              enabled: g.active != null,
                              child: const Text('Keep tab'),
                            ),
                            PopupMenuItem(
                              value: 'close',
                              enabled: w.layoutOwner.groups.length > 1,
                              child: const Text('Close split'),
                            ),
                            PopupMenuItem(
                              value: 'rename',
                              enabled: d != null,
                              child: const Text('Rename file'),
                            ),
                            PopupMenuItem(
                              value: 'delete',
                              enabled: d != null,
                              child: const Text('Delete file'),
                            ),
                            const PopupMenuItem(
                              value: 'retry',
                              child: Text('Retry layout save'),
                            ),
                          ],
                        ),
                      ],
                    ),
                  ),
                  Expanded(
                    child: d == null
                        ? Center(
                            child: TextButton(
                              onPressed: g.active == null
                                  ? null
                                  : () => files.activate(w, g.id, g.active!),
                              child: Text(
                                g.active == null
                                    ? 'Open a file in this group.'
                                    : 'Reload this retained tab',
                              ),
                            ),
                          )
                        : FileEditor(
                            key: ValueKey('${g.id}:${d.id}'),
                            host: widget.host,
                            workspace: w,
                            document: d,
                            memory: w.layoutOwner.memory(g.id, d.id),
                            onFocus: () => w.layoutOwner.focus(g.id),
                          ),
                  ),
                ],
              ),
            ),
            if (dragging != null &&
                dragging!.project == w.project &&
                c.maxHeight > 80) ...[
              Positioned(
                left: 0,
                top: 38,
                bottom: 0,
                width: c.maxWidth * .2,
                child: edge(w, g, Axis.horizontal, true),
              ),
              Positioned(
                right: 0,
                top: 38,
                bottom: 0,
                width: c.maxWidth * .2,
                child: edge(w, g, Axis.horizontal, false),
              ),
              Positioned(
                left: c.maxWidth * .2,
                right: c.maxWidth * .2,
                top: 38,
                height: (c.maxHeight - 38) * .2,
                child: edge(w, g, Axis.vertical, true),
              ),
              Positioned(
                left: c.maxWidth * .2,
                right: c.maxWidth * .2,
                bottom: 0,
                height: (c.maxHeight - 38) * .2,
                child: edge(w, g, Axis.vertical, false),
              ),
            ],
          ],
        ),
      ),
    );
  }

  Widget node(FileWorkspace w, FileLayoutNode n) {
    if (n.group != null) return group(w, w.layoutOwner.groups[n.group]!);
    return LayoutBuilder(
      builder: (context, c) {
        final vertical = n.axis == Axis.vertical;
        final span = vertical ? c.maxHeight : c.maxWidth;
        final minimum = vertical ? 150.0 : 240.0;
        if (span < minimum * 2 + 6) {
          List<String> leaves(FileLayoutNode node) => node.group != null
              ? [node.group!]
              : [...leaves(node.first!), ...leaves(node.second!)];
          final ids = leaves(n);
          final chosen = ids.contains(w.layoutOwner.activeGroup)
              ? w.layoutOwner.activeGroup
              : ids.first;
          return Column(
            children: [
              SizedBox(
                height: 36,
                child: ListView(
                  scrollDirection: Axis.horizontal,
                  children: [
                    for (final id in ids)
                      TextButton(
                        onPressed: () => w.layoutOwner.focus(id),
                        child: Text(
                          'Group ${w.layoutOwner.groups.keys.toList().indexOf(id) + 1}',
                        ),
                      ),
                  ],
                ),
              ),
              Expanded(child: group(w, w.layoutOwner.groups[chosen]!)),
            ],
          );
        }
        final first = ((span - 6) * n.ratio).clamp(minimum, span - 6 - minimum),
            second = (span - 6) - first;
        void resize(double delta) {
          n.ratio = (n.ratio + delta / (span - 6)).clamp(.15, .85);
          w.layoutOwner.changed();
        }

        final divider = Semantics(
          label: 'Resize file split',
          onIncrease: () => resize(span * .05),
          onDecrease: () => resize(-span * .05),
          child: Focus(
            onKeyEvent: (_, event) {
              if (event is! KeyDownEvent) return KeyEventResult.ignored;
              if (event.logicalKey == LogicalKeyboardKey.home) {
                n.ratio = .5;
                w.layoutOwner.changed();
                return KeyEventResult.handled;
              }
              if ([
                LogicalKeyboardKey.arrowRight,
                LogicalKeyboardKey.arrowDown,
              ].contains(event.logicalKey)) {
                resize(span * .05);
                return KeyEventResult.handled;
              }
              if ([
                LogicalKeyboardKey.arrowLeft,
                LogicalKeyboardKey.arrowUp,
              ].contains(event.logicalKey)) {
                resize(-span * .05);
                return KeyEventResult.handled;
              }
              return KeyEventResult.ignored;
            },
            child: MouseRegion(
              cursor: vertical
                  ? SystemMouseCursors.resizeUpDown
                  : SystemMouseCursors.resizeLeftRight,
              child: GestureDetector(
                behavior: HitTestBehavior.opaque,
                onPanUpdate: (e) => resize(vertical ? e.delta.dy : e.delta.dx),
                child: ColoredBox(
                  color: Theme.of(context).dividerColor,
                  child: SizedBox(
                    width: vertical ? double.infinity : 6,
                    height: vertical ? 6 : double.infinity,
                  ),
                ),
              ),
            ),
          ),
        );
        return Flex(
          direction: n.axis,
          children: [
            SizedBox(
              width: vertical ? null : first,
              height: vertical ? first : null,
              child: node(w, n.first!),
            ),
            divider,
            SizedBox(
              width: vertical ? null : second,
              height: vertical ? second : null,
              child: node(w, n.second!),
            ),
          ],
        );
      },
    );
  }

  @override
  Widget build(BuildContext context) => AnimatedBuilder(
    animation: files,
    builder: (context, _) {
      final w = workspace;
      if (w == null) {
        return Center(
          child: Text(
            files.loading
                ? 'Opening project files…'
                : files.error ??
                      'Open a folder or select a project conversation on Home.',
          ),
        );
      }
      return CallbackShortcuts(
        bindings: {
          const SingleActivator(
            LogicalKeyboardKey.backslash,
            control: true,
          ): () =>
              split(w, w.layoutOwner.active, Axis.horizontal),
          const SingleActivator(LogicalKeyboardKey.keyW, control: true): () {
            final g = w.layoutOwner.active;
            if (g.active != null) unawaited(closeTab(w, g, g.active!));
          },
          const SingleActivator(LogicalKeyboardKey.tab, control: true): () {
            final g = w.layoutOwner.active;
            if (g.tabs.isNotEmpty) {
              final at = g.tabs.indexOf(g.active ?? '');
              unawaited(
                files.activate(w, g.id, g.tabs[(at + 1) % g.tabs.length]),
              );
            }
          },
        },
        child: Column(
          children: [
            if (files.error != null)
              Padding(
                padding: const EdgeInsets.all(8),
                child: Text(files.error!),
              ),
            if (w.layoutError != null)
              Padding(
                padding: const EdgeInsets.all(8),
                child: Text(w.layoutError!),
              ),
            Expanded(
              child: LayoutBuilder(
                builder: (context, c) {
                  final compact = c.maxWidth < 600 || c.maxHeight < 360;
                  if (!compact) return node(w, w.layoutOwner.tree);
                  return Column(
                    children: [
                      if (w.layoutOwner.groups.length > 1)
                        SizedBox(
                          height: 36,
                          child: ListView(
                            scrollDirection: Axis.horizontal,
                            children: [
                              for (final g in w.layoutOwner.groups.values)
                                TextButton(
                                  onPressed: () => w.layoutOwner.focus(g.id),
                                  child: Text(
                                    'Group ${w.layoutOwner.groups.keys.toList().indexOf(g.id) + 1}${g.id == w.layoutOwner.activeGroup ? ' · active' : ''}',
                                  ),
                                ),
                            ],
                          ),
                        ),
                      Expanded(child: group(w, w.layoutOwner.active)),
                    ],
                  );
                },
              ),
            ),
          ],
        ),
      );
    },
  );
}
