import 'package:flutter/material.dart';
import 'package:path/path.dart' as path;

import 'chat.dart';
import 'theme.dart';

class ToolApprovalCard extends StatelessWidget {
  final ChatController chat;
  const ToolApprovalCard({super.key, required this.chat});
  @override
  Widget build(BuildContext context) {
    final request = chat.toolApproval;
    if (request == null) return const SizedBox.shrink();
    final p = Palette(Theme.of(context).brightness == Brightness.dark);
    return Container(
      key: const Key('tool-approval'),
      width: double.infinity,
      margin: const EdgeInsets.only(bottom: 12),
      padding: const EdgeInsets.all(14),
      decoration: BoxDecoration(
        color: p.soft,
        border: Border.all(color: p.border),
        borderRadius: BorderRadius.circular(10),
      ),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        mainAxisSize: MainAxisSize.min,
        children: [
          const Text(
            'Allow a file read?',
            style: TextStyle(fontWeight: FontWeight.w600),
          ),
          const SizedBox(height: 6),
          ConstrainedBox(
            constraints: const BoxConstraints(maxHeight: 60),
            child: SingleChildScrollView(
              child: SelectableText(
                request['target'] as String,
                style: TextStyle(color: p.text),
              ),
            ),
          ),
          Text(
            'Folder: ${path.basename(chat.workspaceRoot ?? '')}',
            style: TextStyle(color: p.muted, fontSize: 12),
          ),
          const SizedBox(height: 6),
          Text(
            'Share this file’s text with ${chat.model}? This allows one read. File contents are also kept with a completed reply.',
            style: TextStyle(color: p.muted, fontSize: 12),
          ),
          Wrap(
            spacing: 8,
            children: [
              TextButton(
                key: const Key('deny-tool'),
                onPressed: chat.stopping || chat.decidingTool
                    ? null
                    : () => chat.decideTool(false),
                child: const Text('Deny'),
              ),
              FilledButton(
                key: const Key('allow-tool'),
                onPressed: chat.stopping || chat.decidingTool
                    ? null
                    : () => chat.decideTool(true),
                child: Text(
                  chat.decidingTool ? 'Sending decision…' : 'Allow once',
                ),
              ),
            ],
          ),
        ],
      ),
    );
  }
}

class ToolRecords extends StatelessWidget {
  final List<dynamic> records;
  const ToolRecords({super.key, required this.records});
  @override
  Widget build(BuildContext context) {
    final p = Palette(Theme.of(context).brightness == Brightness.dark);
    return Column(
      mainAxisSize: MainAxisSize.min,
      children: [
        for (final record in records.take(4))
          Card(
            elevation: 0,
            color: p.surface,
            margin: const EdgeInsets.only(top: 6),
            child: ExpansionTile(
              key: PageStorageKey(
                'tool-record-${record['callId']}-${record['target']}',
              ),
              leading: Icon(
                Icons.description_outlined,
                size: 18,
                color: p.muted,
              ),
              title: Text(
                '${record['target']}',
                maxLines: 2,
                overflow: TextOverflow.ellipsis,
                style: const TextStyle(fontSize: 12),
              ),
              subtitle: Text(
                'File read · ${record['status']}',
                style: TextStyle(fontSize: 11, color: p.muted),
              ),
              childrenPadding: const EdgeInsets.all(12),
              children: [
                ConstrainedBox(
                  constraints: const BoxConstraints(maxHeight: 180),
                  child: SingleChildScrollView(
                    key: PageStorageKey(
                      'tool-content-${record['callId']}-${record['target']}',
                    ),
                    child: SelectableText(
                      '${record['content']}',
                      style: TextStyle(
                        fontFamily: 'monospace',
                        fontSize: 12,
                        color: p.text,
                      ),
                    ),
                  ),
                ),
              ],
            ),
          ),
      ],
    );
  }
}
