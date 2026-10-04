import 'package:flutter/material.dart';
import 'package:path/path.dart' as path;

import 'chat.dart';
import 'theme.dart';

class ChatSidebar extends StatefulWidget {
  final ChatController chat;
  final VoidCallback onNewTemporary, onNewSide, onSettings;
  final Future<void> Function() onOpenProject;
  final Future<void> Function(String) onProject, onSelect;
  const ChatSidebar({
    super.key,
    required this.chat,
    required this.onNewTemporary,
    required this.onNewSide,
    required this.onSettings,
    required this.onOpenProject,
    required this.onProject,
    required this.onSelect,
  });
  @override
  State<ChatSidebar> createState() => _ChatSidebarState();
}

class _ChatSidebarState extends State<ChatSidebar> {
  bool projectsOpen = true, recentsOpen = true;
  final expanded = <String>{}, collapsed = <String>{};
  ChatController get chat => widget.chat;
  bool get locked => chat.busy || chat.changing || chat.loading;
  Widget section(
    String title,
    String key,
    bool open,
    VoidCallback toggle, {
    Widget? action,
  }) => Row(
    children: [
      Expanded(
        child: TextButton(
          key: Key(key),
          onPressed: toggle,
          style: TextButton.styleFrom(
            foregroundColor: Palette(
              Theme.of(context).brightness == Brightness.dark,
            ).muted,
            alignment: Alignment.centerLeft,
            padding: const EdgeInsets.symmetric(horizontal: 10, vertical: 12),
          ),
          child: Row(
            mainAxisSize: MainAxisSize.min,
            children: [
              Text(
                title,
                style: const TextStyle(
                  fontSize: 14,
                  fontWeight: FontWeight.w600,
                ),
              ),
              const SizedBox(width: 6),
              Icon(open ? Icons.expand_more : Icons.chevron_right, size: 17),
            ],
          ),
        ),
      ),
      ?action,
    ],
  );
  Future<void> delete(String id) async {
    final confirmed = await showDialog<bool>(
      context: context,
      builder: (context) => AlertDialog(
        title: const Text('Delete conversation?'),
        content: const Text(
          'This removes its saved messages. Working files are kept.',
        ),
        actions: [
          TextButton(
            onPressed: () => Navigator.pop(context, false),
            child: const Text('Keep'),
          ),
          TextButton(
            onPressed: () => Navigator.pop(context, true),
            child: const Text('Delete'),
          ),
        ],
      ),
    );
    if (confirmed == true && mounted) await chat.delete(id);
  }

  Widget chatRow(Map<String, dynamic> item, Palette p, {bool project = false}) {
    final id = item['id'] as String;
    final temporary = item['workspace']?['kind'] == 'temporary';
    return Padding(
      padding: EdgeInsets.only(left: project ? 24 : 0, bottom: 2),
      child: Material(
        color: chat.session == id ? p.soft : Colors.transparent,
        borderRadius: BorderRadius.circular(8),
        child: InkWell(
          key: ValueKey('chat-$id'),
          borderRadius: BorderRadius.circular(8),
          onTap: locked ? null : () => widget.onSelect(id),
          child: Padding(
            padding: const EdgeInsets.only(left: 10),
            child: Row(
              children: [
                if (!project) ...[
                  Icon(
                    temporary
                        ? Icons.workspaces_outline
                        : Icons.chat_bubble_outline,
                    size: 16,
                    color: p.muted,
                  ),
                  const SizedBox(width: 9),
                ],
                Expanded(
                  child: Column(
                    crossAxisAlignment: CrossAxisAlignment.start,
                    children: [
                      Text(
                        item['title'] as String,
                        maxLines: 1,
                        overflow: TextOverflow.ellipsis,
                        style: TextStyle(color: p.text, fontSize: 13),
                      ),
                      if (!project)
                        Text(
                          temporary ? 'Temporary' : 'Side chat',
                          style: TextStyle(color: p.muted, fontSize: 10),
                        ),
                    ],
                  ),
                ),
                IconButton(
                  tooltip: 'Delete conversation',
                  iconSize: 15,
                  onPressed: locked ? null : () => delete(id),
                  icon: Icon(Icons.delete_outline, color: p.muted),
                ),
              ],
            ),
          ),
        ),
      ),
    );
  }

  @override
  Widget build(BuildContext context) {
    final p = Palette(Theme.of(context).brightness == Brightness.dark);
    final projects = {
      for (final project in chat.projects) project['root'] as String: project,
    };
    // Include projects represented on the current history page, even beyond recent-project cap.
    for (final item in chat.sessions) {
      if (item['workspace']?['kind'] == 'project') {
        final root = item['workspace']['root'] as String;
        projects.putIfAbsent(
          root,
          () => {'root': root, 'name': path.basename(root)},
        );
      }
    }
    final recents = chat.sessions
        .where((item) => item['workspace']?['kind'] != 'project')
        .toList();
    return Container(
      width: UiTokens.sidebarWidth,
      color: p.sidebar,
      child: SafeArea(
        child: Padding(
          padding: const EdgeInsets.fromLTRB(12, 28, 12, 16),
          child: Column(
            crossAxisAlignment: CrossAxisAlignment.stretch,
            children: [
              Padding(
                padding: const EdgeInsets.fromLTRB(10, 0, 0, 24),
                child: Row(
                  children: [
                    Icon(
                      Icons.all_inclusive_rounded,
                      size: 27,
                      color: p.accent,
                    ),
                    const SizedBox(width: 12),
                    Expanded(
                      child: Text(
                        'Dolores',
                        maxLines: 1,
                        overflow: TextOverflow.ellipsis,
                        style: TextStyle(
                          fontFamily: 'Georgia',
                          fontSize: 26,
                          letterSpacing: -.8,
                          color: p.text,
                        ),
                      ),
                    ),
                  ],
                ),
              ),
              Row(
                children: [
                  Expanded(
                    child: TextButton.icon(
                      key: const Key('new-chat'),
                      onPressed: locked ? null : widget.onNewTemporary,
                      icon: const Icon(Icons.edit_square, size: 19),
                      label: const Text('New chat'),
                      style: TextButton.styleFrom(
                        foregroundColor: p.text,
                        alignment: Alignment.centerLeft,
                        padding: const EdgeInsets.all(12),
                      ),
                    ),
                  ),
                  PopupMenuButton<String>(
                    key: const Key('new-chat-options'),
                    enabled: !locked,
                    tooltip: 'New chat options',
                    icon: Icon(Icons.expand_more, size: 18, color: p.muted),
                    onSelected: (kind) => kind == 'side'
                        ? widget.onNewSide()
                        : widget.onNewTemporary(),
                    itemBuilder: (_) => const [
                      PopupMenuItem(
                        value: 'temporary',
                        child: Text('Temporary chat'),
                      ),
                      PopupMenuItem(value: 'side', child: Text('Side chat')),
                    ],
                  ),
                ],
              ),
              const SizedBox(height: 16),
              Expanded(
                child: ListView(
                  key: const Key('sidebar-history'),
                  children: [
                    section(
                      'Projects',
                      'section-projects',
                      projectsOpen,
                      () => setState(() => projectsOpen = !projectsOpen),
                      action: IconButton(
                        key: const Key('open-project'),
                        tooltip: 'Open project…',
                        iconSize: 18,
                        onPressed: locked ? null : widget.onOpenProject,
                        icon: Icon(Icons.add, color: p.muted),
                      ),
                    ),
                    if (projectsOpen) ...[
                      if (projects.isEmpty)
                        TextButton.icon(
                          key: const Key('empty-open-project'),
                          onPressed: locked ? null : widget.onOpenProject,
                          icon: const Icon(
                            Icons.folder_open_outlined,
                            size: 16,
                          ),
                          label: const Text('Open project…'),
                          style: TextButton.styleFrom(
                            alignment: Alignment.centerLeft,
                          ),
                        ),
                      for (final project in projects.values) ...[
                        Builder(
                          builder: (_) {
                            final root = project['root'] as String;
                            final open =
                                expanded.contains(root) ||
                                (chat.workspaceRoot == root &&
                                    !collapsed.contains(root));
                            return Column(
                              children: [
                                Row(
                                  children: [
                                    Expanded(
                                      child: TextButton(
                                        key: ValueKey('project-$root'),
                                        onPressed: () => setState(() {
                                          if (open) {
                                            expanded.remove(root);
                                            collapsed.add(root);
                                          } else {
                                            expanded.add(root);
                                            collapsed.remove(root);
                                          }
                                        }),
                                        style: TextButton.styleFrom(
                                          foregroundColor: p.text,
                                          alignment: Alignment.centerLeft,
                                          padding: const EdgeInsets.symmetric(
                                            horizontal: 10,
                                          ),
                                        ),
                                        child: Row(
                                          children: [
                                            Icon(
                                              open
                                                  ? Icons.expand_more
                                                  : Icons.chevron_right,
                                              size: 16,
                                              color: p.muted,
                                            ),
                                            const SizedBox(width: 4),
                                            Icon(
                                              Icons.folder_outlined,
                                              size: 16,
                                              color: p.muted,
                                            ),
                                            const SizedBox(width: 8),
                                            Expanded(
                                              child: Text(
                                                project['name'] as String,
                                                maxLines: 1,
                                                overflow: TextOverflow.ellipsis,
                                                style: const TextStyle(
                                                  fontSize: 13,
                                                ),
                                              ),
                                            ),
                                          ],
                                        ),
                                      ),
                                    ),
                                    IconButton(
                                      key: ValueKey('new-project-chat-$root'),
                                      tooltip: 'New chat in ${project['name']}',
                                      iconSize: 16,
                                      onPressed: locked
                                          ? null
                                          : () => widget.onProject(root),
                                      icon: Icon(Icons.add, color: p.muted),
                                    ),
                                  ],
                                ),
                                if (open) ...[
                                  for (final item in chat.sessions.where(
                                    (item) =>
                                        item['workspace']?['kind'] ==
                                            'project' &&
                                        item['workspace']?['root'] == root,
                                  ))
                                    chatRow(item, p, project: true),
                                  if (!chat.sessions.any(
                                    (item) =>
                                        item['workspace']?['root'] == root,
                                  ))
                                    Padding(
                                      padding: const EdgeInsets.fromLTRB(
                                        32,
                                        4,
                                        8,
                                        10,
                                      ),
                                      child: Text(
                                        'No chats on this page',
                                        style: TextStyle(
                                          color: p.muted,
                                          fontSize: 11,
                                        ),
                                      ),
                                    ),
                                ],
                              ],
                            );
                          },
                        ),
                      ],
                    ],
                    const SizedBox(height: 12),
                    section(
                      'Recents',
                      'section-recents',
                      recentsOpen,
                      () => setState(() => recentsOpen = !recentsOpen),
                    ),
                    if (recentsOpen) ...[
                      for (final item in recents) chatRow(item, p),
                      if (recents.isEmpty)
                        Padding(
                          padding: const EdgeInsets.fromLTRB(10, 4, 10, 12),
                          child: Text(
                            'Temporary and side chats appear here',
                            style: TextStyle(color: p.muted, fontSize: 11),
                          ),
                        ),
                    ],
                  ],
                ),
              ),
              if (chat.sessionsOlder || chat.sessionsNewer)
                Row(
                  children: [
                    Expanded(
                      child: TextButton(
                        key: const Key('newer-chats'),
                        onPressed: locked || !chat.sessionsNewer
                            ? null
                            : () => chat.browseSessions(newer: true),
                        child: const Text('Newer'),
                      ),
                    ),
                    Expanded(
                      child: TextButton(
                        key: const Key('older-chats'),
                        onPressed: locked || !chat.sessionsOlder
                            ? null
                            : () => chat.browseSessions(newer: false),
                        child: const Text('Older'),
                      ),
                    ),
                  ],
                ),
              Divider(color: p.border),
              TextButton.icon(
                key: const Key('settings'),
                onPressed: chat.loading || chat.changing
                    ? null
                    : widget.onSettings,
                icon: const Icon(Icons.settings_outlined, size: 18),
                label: const Text('Settings'),
                style: TextButton.styleFrom(
                  alignment: Alignment.centerLeft,
                  padding: const EdgeInsets.all(12),
                ),
              ),
            ],
          ),
        ),
      ),
    );
  }
}
