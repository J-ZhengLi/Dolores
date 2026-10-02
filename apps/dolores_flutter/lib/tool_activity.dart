import 'dart:convert';

import 'package:flutter/material.dart';
import 'package:path/path.dart' as path;

import 'chat.dart';
import 'theme.dart';
import 'edit_diff.dart';

String toolLabel(dynamic name) => switch (name) {
  'list_folder' => 'Folder listing',
  'search_text' => 'Text search',
  'edit_text_file' => 'File edit',
  _ => 'File read',
};

String toolResultText(dynamic record) {
  final content = '${record['content']}';
  if (record['status'] != 'completed' && record['status'] != 'edited') {
    return content;
  }
  try {
    final result = jsonDecode(content) as Map;
    if (record['name'] == 'edit_text_file' && result['applied'] == true) {
      return 'Applied one file change · ${result['bytesBefore']} → ${result['bytesAfter']} bytes'
          '${result['journalStatus'] == 'pending' ? '\nLocal intent saved; its receipt needs a check in Changes.' : ''}';
    }
    final partial = result['truncated'] == true ? ' · Partial results' : '';
    if (record['name'] == 'list_folder') {
      final entries = result['entries'] as List;
      return 'Listed ${entries.length} entries$partial\n'
          'Skipped ${result['skippedEntries']} entries\n\n'
          '${entries.map((entry) => '${entry['path']}${entry['kind'] == 'folder' ? '/' : ''}').join('\n')}';
    }
    if (record['name'] == 'search_text') {
      final matches = result['matches'] as List;
      return '${matches.length} matching lines · Searched ${result['scannedFiles']} files$partial\n'
          'Skipped ${result['skippedFiles']} files and ${result['skippedEntries']} entries. Only eligible text files are searched.\n\n'
          '${matches.map((match) => '${match['path']}:${match['line']}\n${match['text']}').join('\n\n')}';
    }
  } catch (_) {
    // Legacy/unknown tool payloads remain literal and selectable.
  }
  return content;
}

class ToolApprovalCard extends StatelessWidget {
  final ChatController chat;
  const ToolApprovalCard({super.key, required this.chat});
  @override
  Widget build(BuildContext context) {
    final request = chat.toolApproval;
    if (request == null) return const SizedBox.shrink();
    final p = Palette(Theme.of(context).brightness == Brightness.dark);
    final name = request['name'];
    final editing = name == 'edit_text_file';
    final title = switch (name) {
      'list_folder' => 'Allow a folder listing?',
      'search_text' => 'Allow a text search?',
      'edit_text_file' => 'Apply this file change?',
      _ => 'Allow a file read?',
    };
    final disclosure = switch (name) {
      'list_folder' =>
        'Share up to 100 file and folder names with ${chat.model}? This allows one listing. Results are kept with a completed reply.',
      'search_text' =>
        'Scan up to 64 text files and 256 KiB under this folder? Matching snippets are shared with ${chat.model} and kept with a completed reply. Reading a whole file needs another decision.',
      'edit_text_file' => 'Review the diff before applying this one change. Local before and after snapshots are saved in Changes, even if the reply stops or fails. A changed file needs a fresh preview.',
      _ =>
        'Share this file’s text with ${chat.model}? This allows one read. File contents are also kept with a completed reply.',
    };
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
          ConstrainedBox(
            constraints: BoxConstraints(maxHeight: editing ? 270 : 180),
            child: SingleChildScrollView(
              child: Column(
                crossAxisAlignment: CrossAxisAlignment.start,
                children: [
                  Text(title, style: TextStyle(fontWeight: FontWeight.w600)),
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
                    'Folder: ${chat.workspaceKind == 'temporary' ? 'Temporary workspace' : path.basename(chat.workspaceRoot ?? '')}',
                    style: TextStyle(color: p.muted, fontSize: 12),
                  ),
                  if (request['query'] is String) ...[
                    const SizedBox(height: 6),
                    const Text(
                      'Find this exact text:',
                      style: TextStyle(fontSize: 12),
                    ),
                    SelectableText(
                      request['query'] as String,
                      style: const TextStyle(fontSize: 12),
                    ),
                    const Text(
                      'Case-sensitive · Up to four folder levels',
                      style: TextStyle(fontSize: 11),
                    ),
                  ],
                  const SizedBox(height: 6),
                  Text(
                    disclosure,
                    style: TextStyle(color: p.muted, fontSize: 12),
                  ),
                  if (editing && request['diff'] is String) ...[
                    const SizedBox(height: 8),
                    EditDiff(
                      source: request['diff'] as String,
                      identity: 'approval-${request['callId']}',
                    ),
                  ],
                ],
              ),
            ),
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
                  chat.decidingTool
                      ? 'Sending decision…'
                      : editing
                      ? 'Apply once'
                      : 'Allow once',
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
                'tool-record-${record['callId']}-${record['name']}-${record['target']}',
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
                '${toolLabel(record['name'])} · ${record['status']}',
                style: TextStyle(fontSize: 11, color: p.muted),
              ),
              childrenPadding: const EdgeInsets.all(12),
              children: [
                if (record['query'] is String)
                  SelectableText(
                    key: PageStorageKey(
                      'tool-query-${record['callId']}-${record['name']}-${record['target']}',
                    ),
                    'Search: ${record['query']}',
                    style: TextStyle(color: p.muted, fontSize: 12),
                  ),
                ConstrainedBox(
                  constraints: const BoxConstraints(maxHeight: 180),
                  child: SingleChildScrollView(
                    key: PageStorageKey(
                      'tool-content-${record['callId']}-${record['name']}-${record['target']}',
                    ),
                    child: SelectableText(
                      toolResultText(record),
                      style: TextStyle(
                        fontFamily: 'monospace',
                        fontSize: 12,
                        color: p.text,
                      ),
                    ),
                  ),
                ),
                if (record['diff'] is String) ...[
                  const SizedBox(height: 8),
                  EditDiff(
                    source: record['diff'] as String,
                    identity: 'record-${record['callId']}',
                  ),
                ],
              ],
            ),
          ),
      ],
    );
  }
}
