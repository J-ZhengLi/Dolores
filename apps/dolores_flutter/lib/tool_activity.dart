import 'dart:convert';

import 'package:flutter/material.dart';
import 'package:path/path.dart' as path;

import 'chat.dart';
import 'theme.dart';
import 'edit_diff.dart';

String toolLabel(dynamic name) => switch (name) {
  String value when value.startsWith('mcp_tool_') => 'External tool',
  'list_folder' => 'Folder listing',
  'search_text' => 'Text search',
  'edit_text_file' => 'File edit',
  'create_text_file' => 'File creation',
  'run_command' => 'Command',
  _ => 'File read',
};

String toolResultText(dynamic record) {
  final content = '${record['content']}';
  if (![
    'completed',
    'failed',
    'incomplete',
    'edited',
    'created',
  ].contains(record['status'])) {
    return content;
  }
  try {
    final result = jsonDecode(content) as Map;
    if ('${record['name']}'.startsWith('mcp_tool_')) {
      return '${result['isError'] == true ? 'Tool reported an error\n\n' : ''}${result['text'] ?? ''}';
    }
    if (record['name'] == 'run_command') {
      final outcome = switch (result['reason']) {
        'timedOut' => 'Stopped at the time limit',
        'outputLimit' => 'Stopped at the output limit',
        'processError' => 'Process status unavailable',
        _ => 'Exited ${result['exitCode'] ?? 'with unavailable status'}',
      };
      return '$outcome${result['truncated'] == true ? ' · Output shortened' : ''}'
          '${result['lossyUtf8'] == true ? '\nSome bytes could not be displayed as UTF-8.' : ''}'
          '${result['outputError'] == true ? '\nSome output could not be read.' : ''}'
          '\n\nStandard output:\n${result['stdout'] ?? ''}\n\nStandard error:\n${result['stderr'] ?? ''}';
    }
    if (record['name'] == 'create_text_file' && result['applied'] == true) {
      return 'Created one file · ${result['bytesAfter']} bytes'
          '${result['journalStatus'] == 'pending' ? '\nLocal intent saved; its receipt needs a check in Changes.' : ''}';
    }
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

String toolStatus(dynamic record) {
  if (record['name'] != 'run_command' ||
      ['denied', 'blocked', 'error'].contains(record['status'])) {
    return '${record['status']}';
  }
  try {
    final result = jsonDecode('${record['content']}') as Map;
    if (result['reason'] != 'completed' || result['exitCode'] is! int) {
      return 'Verification incomplete';
    }
    if (result['exitCode'] != 0) return 'Failed · exit ${result['exitCode']}';
    if (result['truncated'] != false ||
        result['lossyUtf8'] == true ||
        result['outputError'] == true) {
      return 'Verification incomplete';
    }
    return 'Exited 0';
  } catch (_) {
    return 'Verification incomplete';
  }
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
    final creating = name == 'create_text_file';
    final running = name == 'run_command';
    final external = request['mcp'] is Map;
    final credentialNames =
        (request['mcp'] as Map?)?['credentialNames'] as List? ?? [];
    final title = switch (name) {
      String value when value.startsWith('mcp_tool_') =>
        'Allow this external tool?',
      'list_folder' => 'Allow a folder listing?',
      'search_text' => 'Allow a text search?',
      'edit_text_file' => 'Apply this file change?',
      'create_text_file' => 'Create this file?',
      'run_command' => 'Run this command?',
      _ => 'Allow a file read?',
    };
    final disclosure = switch (name) {
      String value when value.startsWith('mcp_tool_') =>
        'Starts the reviewed server with your permissions. It can change files outside this folder and use the network. Effects may remain after Stop and are not recorded in Changes. Results are shared with ${chat.model} and saved with a completed reply. Limit: 30 seconds · 8 KiB text.${credentialNames.isEmpty ? '' : '\nServer receives saved credentials: ${credentialNames.join(', ')}.'}',
      'list_folder' =>
        'Share up to 100 file and folder names with ${chat.model}? This allows one listing. Results are kept with a completed reply.',
      'search_text' =>
        'Scan up to 64 text files and 256 KiB under this folder? Matching snippets are shared with ${chat.model} and kept with a completed reply. Reading a whole file needs another decision.',
      'edit_text_file' => 'Review the diff before applying this one change. Local before and after snapshots are saved in Changes, even if the reply stops or fails. A changed file needs a fresh preview.',
      'create_text_file' => 'Review the complete addition. Create one small text file in an existing folder; an occupied path is never replaced. A local snapshot is saved in Changes, even if the reply stops or fails. Removing it later needs another review.',
      'run_command' =>
        'Runs with your permissions. It can access or change files outside this folder and use the network. Command changes are not recorded in Changes and may remain after Stop. Output is shared with ${chat.model} and saved with a completed reply. Limit: 30 seconds · 8 KiB output.',
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
            constraints: BoxConstraints(
              maxHeight: editing || creating || running || external ? 270 : 180,
            ),
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
                  if (running || external) ...[
                    const SizedBox(height: 6),
                    Text(
                      disclosure,
                      style: TextStyle(color: p.muted, fontSize: 12),
                    ),
                  ],
                  if (external) ...[
                    const SizedBox(height: 8),
                    SelectableText(
                      'Server: ${request['mcp']['server']}\nTool: ${request['mcp']['tool']}\nReviewed revision: ${request['mcp']['revision']}',
                      style: const TextStyle(fontSize: 12),
                    ),
                    const SizedBox(height: 6),
                    const Text(
                      'Exact arguments (JSON)',
                      style: TextStyle(fontSize: 11),
                    ),
                    SelectableText(
                      '${request['mcp']['arguments']}',
                      style: const TextStyle(
                        fontFamily: 'monospace',
                        fontSize: 12,
                      ),
                    ),
                  ],
                  if (running && request['command'] is Map) ...[
                    const SizedBox(height: 8),
                    const Text(
                      'Working folder',
                      style: TextStyle(fontSize: 11),
                    ),
                    SelectableText(
                      chat.workspaceRoot ?? '',
                      style: const TextStyle(
                        fontFamily: 'monospace',
                        fontSize: 12,
                      ),
                    ),
                    const SizedBox(height: 6),
                    const Text('Executable', style: TextStyle(fontSize: 11)),
                    SelectableText(
                      '${request['command']['executable']}',
                      style: const TextStyle(
                        fontFamily: 'monospace',
                        fontSize: 12,
                      ),
                    ),
                    const SizedBox(height: 6),
                    const Text(
                      'Arguments (each line is one literal value)',
                      style: TextStyle(fontSize: 11),
                    ),
                    SelectableText(
                      ((request['command']['invocation']['args'] as List)
                              .isEmpty
                          ? 'No arguments'
                          : (request['command']['invocation']['args'] as List)
                                .asMap()
                                .entries
                                .map(
                                  (e) => '${e.key + 1}. ${jsonEncode(e.value)}',
                                )
                                .join('\n')),
                      style: const TextStyle(
                        fontFamily: 'monospace',
                        fontSize: 12,
                      ),
                    ),
                  ],
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
                  if (!running && !external)
                    Text(
                      disclosure,
                      style: TextStyle(color: p.muted, fontSize: 12),
                    ),
                  if ((editing || creating) && request['diff'] is String) ...[
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
                      : running || external
                      ? 'Run once'
                      : creating
                      ? 'Create once'
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
  final int maxRecords;
  const ToolRecords({super.key, required this.records, this.maxRecords = 4});
  @override
  Widget build(BuildContext context) {
    final p = Palette(Theme.of(context).brightness == Brightness.dark);
    return Column(
      mainAxisSize: MainAxisSize.min,
      children: [
        for (final record in records.take(maxRecords))
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
                '${toolLabel(record['name'])} · ${toolStatus(record)}',
                style: TextStyle(fontSize: 11, color: p.muted),
              ),
              childrenPadding: const EdgeInsets.all(12),
              children: [
                if (record['mcp'] is Map)
                  SelectableText(
                    'Server: ${record['mcp']['server']}\nTool: ${record['mcp']['tool']}\nReviewed revision: ${record['mcp']['revision']}\nArguments: ${record['mcp']['arguments']}',
                    style: TextStyle(
                      color: p.muted,
                      fontFamily: 'monospace',
                      fontSize: 12,
                    ),
                  ),
                if (record['command'] is Map)
                  SelectableText(
                    key: PageStorageKey(
                      'tool-command-${record['callId']}-${record['name']}-${record['target']}',
                    ),
                    '${record['command']['program']}\n${(record['command']['args'] as List).asMap().entries.map((e) => '${e.key + 1}. ${jsonEncode(e.value)}').join('\n')}',
                    style: TextStyle(
                      color: p.muted,
                      fontFamily: 'monospace',
                      fontSize: 12,
                    ),
                  ),
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
