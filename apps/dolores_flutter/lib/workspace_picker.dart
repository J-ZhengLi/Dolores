import 'package:flutter/material.dart';
import 'package:flutter/services.dart';

import 'chat.dart';
import 'theme.dart';

class WorkspacePicker extends StatelessWidget {
  final ChatController chat;
  final Future<void> Function() onOpenProject;
  const WorkspacePicker({
    super.key,
    required this.chat,
    required this.onOpenProject,
  });
  @override
  Widget build(BuildContext context) {
    final p = Palette(Theme.of(context).brightness == Brightness.dark);
    return PopupMenuButton<String>(
      key: const Key('workspace-picker'),
      enabled: !chat.busy && !chat.changing && !chat.loading,
      tooltip: 'Working folder and chat mode',
      onSelected: (value) async {
        if (value == 'project') {
          await onOpenProject();
        } else if (value == 'folder') {
          final root = chat.workspaceRoot;
          if (root == null) return;
          await showDialog<void>(
            context: context,
            builder: (context) => AlertDialog(
              title: const Text('Working folder'),
              content: SelectableText(root),
              actions: [
                TextButton(
                  onPressed: () => Clipboard.setData(ClipboardData(text: root)),
                  child: const Text('Copy path'),
                ),
                TextButton(
                  onPressed: () => Navigator.pop(context),
                  child: const Text('Done'),
                ),
              ],
            ),
          );
        } else {
          chat.newChat(kind: value);
        }
      },
      itemBuilder: (_) => [
        const PopupMenuItem(value: 'project', child: Text('Open project…')),
        const PopupMenuItem(value: 'temporary', child: Text('Temporary chat')),
        const PopupMenuItem(value: 'side', child: Text('Side chat')),
        if (chat.workspaceRoot != null) ...[
          const PopupMenuDivider(),
          const PopupMenuItem(
            value: 'folder',
            child: Text('Show working folder'),
          ),
        ],
      ],
      child: Padding(
        padding: const EdgeInsets.symmetric(vertical: 12),
        child: Row(
          mainAxisSize: MainAxisSize.min,
          children: [
            Icon(
              chat.workspaceKind == 'side'
                  ? Icons.chat_bubble_outline
                  : Icons.folder_open_outlined,
              size: 18,
              color: p.muted,
            ),
            const SizedBox(width: 8),
            Flexible(
              child: Text(
                chat.workspaceLabel,
                maxLines: 1,
                overflow: TextOverflow.ellipsis,
                style: TextStyle(
                  color: p.text,
                  fontWeight: FontWeight.w600,
                  fontSize: 14,
                ),
              ),
            ),
            const SizedBox(width: 4),
            Icon(Icons.expand_more, size: 16, color: p.muted),
          ],
        ),
      ),
    );
  }
}
