import 'dart:convert';

import 'package:flutter/material.dart';
import 'package:path/path.dart' as path;

import 'chat.dart';
import 'theme.dart';
import 'edit_diff.dart';
import 'subagents.dart';
import 'bridge.dart';
import 'browser_settings.dart';

String toolLabel(dynamic name) => switch (name) {
  String value when value.startsWith('mcp_tool_') => 'External tool',
  'list_folder' => 'Folder listing',
  'search_text' => 'Text search',
  'edit_text_file' => 'File edit',
  'create_text_file' => 'File creation',
  'run_command' => 'Command',
  'inspect_harness' => 'Harness inspection',
  'harness_repair' => 'Repair proposal',
  'test_harness_repair' => 'Native evaluation',
  'delegate_tasks' => 'Subagents',
  'web_search' => 'Web search',
  'read_web_page' => 'Web page',
  'browser' => 'Browser',
  'inspect_desktop_capture' => 'Desktop screenshot',
  'desktop_control' => 'Computer use',
  'request_desktop_access' => 'Window access',
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
    if (record['name'] == 'test_harness_repair') {
      final trial = result['evaluation'] as Map? ?? result;
      final baseline = trial['baseline'] as Map?;
      final candidate = trial['candidate'] as Map?;
      return '${trial['status']} · Native evaluation\n'
          'Baseline: ${baseline == null ? trial['baselinePassed'] ?? 'not run' : (baseline['passed'] as List? ?? []).length} passed, ${baseline == null ? trial['baselineFailed'] ?? 'unknown' : (baseline['failed'] as List? ?? []).length} failed\n'
          'Candidate: ${candidate == null ? trial['candidatePassed'] ?? 'not run' : (candidate['passed'] as List? ?? []).length} passed, ${candidate == null ? trial['candidateFailed'] ?? 'unknown' : (candidate['failed'] as List? ?? []).length} failed\n'
          '${trial['note'] ?? result['note']}';
    }
    if (record['name'] == 'harness_repair') {
      if (result['repairIds'] is List) {
        return 'Retained repairs: ${(result['repairIds'] as List).join(', ')}\n${result['note'] ?? ''}';
      }
      final files = (result['files'] as List? ?? []).whereType<Map>();
      final source = result['source'] as Map?;
      return '${result['status']} · Revision ${result['revision']}\n'
          'Repair: ${result['repairId']}\nSource: ${result['sourceMatch']}\n'
          '${files.map((f) => '${f['path']} · ${f['changed'] == true ? 'changed' : 'baseline'}${f['protectedReview'] == true ? ' · protected review' : ''}').join('\n')}\n'
          '${result['artifactIntegrity'] == null ? '' : 'Saved files: ${result['artifactIntegrity']}\n'}'
          '${source == null ? '' : '\n${source['text']}\nCandidate: ${source['candidateId']}\n${source['hasMore'] == true ? 'Continue at line ${source['nextLine']}' : ''}\n'}'
          '${result['diffNotice'] ?? ''}\n${result['note'] ?? ''}';
    }
    if (record['name'] == 'web_search') {
      final sources = (result['results'] as List).whereType<Map>();
      return '${result['provider']} · ${sources.length} sources\nQuery: ${result['query']}\n\n${sources.map((s) => '${s['title']}\n${s['url']}\n${s['snippet']}').join('\n\n')}\n\n${result['note']}';
    }
    if (record['name'] == 'read_web_page') {
      return '${result['title']}\n${result['url']}\n${result['truncated'] == true ? 'Partial excerpt · next startCharacter: ${result['nextCharacter']}\n' : ''}\n${result['text']}\n\n${result['note']}';
    }
    if (record['name'] == 'browser') {
      return '${result['outcome']} · ${result['title'] ?? ''}\n${result['url'] ?? ''}\n${result['partial'] == true ? 'Partial page state\n' : ''}${result['recovery'] ?? ''}\n${result['text'] ?? ''}\n\n${result['controls'] ?? ''}\n${result['note'] ?? ''}';
    }
    if (record['name'] == 'inspect_desktop_capture') {
      final capture = result['capture'] as Map;
      final observation = capture['observation'] as Map;
      return 'Shared screenshot · ${observation['target']['title']}\n${observation['width']} × ${observation['height']} image pixels · ${observation['dpi']} DPI\nCapture: ${capture['id']}\n${result['freshness']}\nUntrusted observation; no desktop input authority.';
    }
    if (record['name'] == 'delegate_tasks') {
      final usage = result['sharedUsage'] as Map?;
      return 'Shared task used ${usage?['modelCalls'] ?? '?'} model calls and ${usage?['toolCalls'] ?? '?'} tool operations when children returned.\n${result['note'] ?? ''}';
    }
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
          '${result['previewTruncated'] == true ? '\nPreview shortened; capture status is shown above.' : ''}'
          '${result['localLog'] is String ? '\nLocal log in working folder: ${result['localLog']}\nOpen it locally or request a ranged read.' : ''}'
          '${result['logError'] == true ? '\nLocal log could not be saved. Inspect files and rerun with a smaller capture.' : ''}'
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
  if (record['name'] == 'browser') {
    try {
      final outcome = jsonDecode('${record['content']}')['outcome'];
      if (outcome == 'stale') return 'Needs fresh review · no action ran';
      if (outcome == 'uncertain') return 'Action outcome uncertain';
    } catch (_) {
      /* Failure/legacy content stays literal. */
    }
  }
  if (record['name'] == 'test_harness_repair' &&
      record['status'] == 'completed') {
    try {
      final result = jsonDecode('${record['content']}') as Map;
      final trial = result['evaluation'] as Map? ?? result;
      return switch (trial['status']) {
        'qualified' => 'Tests qualified · installation needs review',
        'withheld' => 'Improvement withheld',
        'stopped' => 'Stopped · proposal retained',
        _ => 'Verification incomplete',
      };
    } catch (_) {
      return 'Verification incomplete';
    }
  }
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

String? browserCaptureId(dynamic record) {
  try {
    final id = jsonDecode('${record['content']}')['capture'];
    return id is String &&
            RegExp(
              r'^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$',
            ).hasMatch(id)
        ? id
        : null;
  } catch (_) {
    return null;
  }
}

Map? desktopCaptureReference(dynamic record) {
  try {
    final capture = jsonDecode('${record['content']}')['capture'];
    return capture is Map &&
            capture['session'] is String &&
            capture['id'] is String &&
            RegExp(
              r'^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$',
            ).hasMatch(capture['id'])
        ? capture
        : null;
  } catch (_) {
    return null;
  }
}

List<dynamic> subagentReports(dynamic record) {
  try {
    final value = jsonDecode('${record['content']}');
    return value is Map && value['children'] is List
        ? (value['children'] as List).whereType<Map>().take(2).toList()
        : [];
  } catch (_) {
    return [];
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
    final repairing = name == 'harness_repair';
    final evaluating = name == 'test_harness_repair';
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
      'inspect_harness' => 'Inspect the running harness?',
      'harness_repair' => 'Review this repair step?',
      'test_harness_repair' => 'Build and test this repair?',
      'delegate_tasks' => 'Delegate these scoped tasks?',
      'web_search' => 'Share this web search query?',
      'read_web_page' => 'Read this public web page?',
      'browser' => 'Allow this browser operation?',
      'desktop_control' => 'Allow this desktop input?',
      _ => 'Allow a file read?',
    };
    final disclosure = switch (name) {
      'desktop_control' => 'Review the exact action and selected window below. Coordinates use the latest screenshot. This may submit data or change files through that application. Input receipts prove dispatch only; Dolores must observe again. Stop or failure can leave effects; inspect before repeating. Desktop access is separate from folder access.',
      'browser' =>
        'Uses one fresh visible browser for this run. Review the literal operation below. Click/input can submit data or cause external effects and always need fresh approval. Only this origin loads; no saved profile, passwords, uploads/downloads or popups. Page text/state goes to ${chat.model} and local evidence. Screenshots stay local. Stop closes owned resources; submitted effects may remain.',
      'web_search' =>
        'Send the exact query to the displayed search endpoint. A configured paid API may consume quota. Up to five source URLs/snippets are shared with ${chat.model} and retained in run evidence. Retrieved text cannot grant permissions. No automatic retries or provider switching.',
      'read_web_page' =>
        'Send this URL to its public HTTPS host without login, cookies or scripts. Download at most 256 KiB within 20 seconds; share an 8 KiB excerpt with ${chat.model} and keep it in run evidence. Redirects and private networks are refused. Retrieved text is untrusted.',
      'delegate_tasks' => 'Start up to two children with this model and the parent’s shared task limits. Children get scoped file tools; each operation still follows current permissions. No commands, external tools or further delegation. Stop reaches both; applied changes remain. Reports require parent verification.',
      'inspect_harness' =>
        'Share the selected running capabilities or bounded bundled source with ${chat.model}? This is read-only and cannot update Dolores or grant permissions. Results are kept with a completed reply.',
      'harness_repair' =>
        'Review matching source and the proposed diff in Dolores’s separate repair storage. Your project stays unchanged. This step cannot compile, execute tests or install a replacement app. Results go to ${chat.model} and retained local evidence. Native execution and installation need separate review.',
      'test_harness_repair' =>
        'Build and run the same reviewed reproduction test against matching baseline and candidate, then run candidate library regressions in separate repair storage. Build scripts and native code run with your account permissions and can access files, network or processes; this is not an OS sandbox. At most three commands, each with 300 seconds and 256 KiB output. Candidate runs only after a complete baseline test failure. Tests stay fixed; Stop preserves the proposal and available evidence. Results go to ${chat.model}. This cannot install, restart or replay your task.',
      String value when value.startsWith('mcp_tool_') =>
        'Starts the reviewed server with your permissions. It can change files outside this folder and use the network. Effects may remain after Stop and are not recorded in Changes. Results are shared with ${chat.model} and saved with a completed reply. Limit: 30 seconds · 8 KiB text.${credentialNames.isEmpty ? '' : '\nServer receives saved credentials: ${credentialNames.join(', ')}.'}',
      'list_folder' =>
        'Share up to 100 file and folder names with ${chat.model}? This allows one listing. Results are kept with a completed reply.',
      'search_text' =>
        'Scan up to 64 text files and 256 KiB under this folder? Matching snippets are shared with ${chat.model} and kept with a completed reply. Reading a whole file needs another decision.',
      'edit_text_file' => 'Review the diff before applying this one change. Local before and after snapshots are saved in Changes, even if the reply stops or fails. A changed file needs a fresh preview.',
      'create_text_file' => 'Review the complete addition. Create one small text file in an existing folder; an occupied path is never replaced. A local snapshot is saved in Changes, even if the reply stops or fails. Removing it later needs another review.',
      'run_command' =>
        'Runs with your permissions. It can access or change files outside this folder and use the network. Command changes are not recorded in Changes and may remain after Stop. Output is shared with ${chat.model} and saved with a completed reply. Limit: ${request['command']?['timeout_seconds'] ?? 30} seconds · ${request['command']?['capture_bytes'] ?? 8192} bytes capture. Larger output is kept in a local log with a shortened preview.',
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
              maxHeight:
                  editing ||
                      creating ||
                      repairing ||
                      evaluating ||
                      running ||
                      external
                  ? 270
                  : 180,
            ),
            child: SingleChildScrollView(
              child: Column(
                crossAxisAlignment: CrossAxisAlignment.start,
                children: [
                  Text(title, style: TextStyle(fontWeight: FontWeight.w600)),
                  if ('${request['callId']}'.startsWith('child.'))
                    Text(
                      'Requested by subagent ${'${request['callId']}'.split('.').elementAtOrNull(1) ?? ''}',
                      style: TextStyle(color: p.muted, fontSize: 11),
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
                    repairing || evaluating
                        ? 'Storage: Dolores repair workspace'
                        : 'Folder: ${chat.workspaceKind == 'temporary' ? 'Temporary workspace' : path.basename(chat.workspaceRoot ?? '')}',
                    style: TextStyle(color: p.muted, fontSize: 12),
                  ),
                  if (running || external) ...[
                    const SizedBox(height: 6),
                    Text(
                      external || running
                          ? 'Runs with your OS permissions; may change files or use the network. Effects may remain after Stop.'
                          : disclosure,
                      style: TextStyle(color: p.muted, fontSize: 12),
                    ),
                  ],
                  if (external) ...[
                    if (credentialNames.isNotEmpty)
                      Text(
                        'Server receives: ${credentialNames.join(', ')}',
                        style: TextStyle(color: p.muted, fontSize: 12),
                      ),
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
                    Text(
                      name == 'delegate_tasks'
                          ? 'Exact child goals and file scopes:'
                          : name == 'inspect_harness'
                          ? 'Inspect this source/range:'
                          : repairing || evaluating
                          ? 'Exact repair step:'
                          : name == 'read_text_file'
                          ? 'Read these lines (start:count):'
                          : name == 'web_search'
                          ? 'Exact query shared with the service:'
                          : name == 'read_web_page'
                          ? 'Start character in extracted text:'
                          : 'Find this exact text:',
                      style: TextStyle(fontSize: 12),
                    ),
                    SelectableText(
                      request['query'] as String,
                      style: const TextStyle(fontSize: 12),
                    ),
                    if (name == 'search_text')
                      const Text(
                        'Case-sensitive · Up to four folder levels',
                        style: TextStyle(fontSize: 11),
                      ),
                  ],
                  const SizedBox(height: 6),
                  if (!running && !external)
                    Text(switch (name) {
                      'desktop_control' => 'Input affects the selected window. Submission or deletion may be irreversible.',
                      'browser' => 'Click or input may submit data or change this website.',
                      'web_search' => 'The query and returned sources are shared with the search service and your model.',
                      'read_web_page' => 'The URL is sent to its host; extracted text is shared with your model.',
                      _ =>
                        editing || creating
                            ? 'Applies this change to the displayed file.'
                            : 'Shares this result with your model.',
                    }, style: TextStyle(color: p.muted, fontSize: 12)),
                  Material(
                    color: Colors.transparent,
                    child: ExpansionTile(
                      tilePadding: EdgeInsets.zero,
                      title: const Text('Operation details'),
                      children: [
                        SelectableText(
                          disclosure,
                          style: TextStyle(color: p.muted, fontSize: 12),
                        ),
                      ],
                    ),
                  ),
                  if ((editing || creating || repairing || evaluating) &&
                      request['diff'] is String) ...[
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
                      : running || external || evaluating
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
  final ChatBridge? bridge;
  final List<dynamic> records;
  final int maxRecords;
  const ToolRecords({
    super.key,
    required this.records,
    this.maxRecords = 4,
    this.bridge,
  });
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
                if (record['name'] == 'inspect_desktop_capture' &&
                    bridge != null &&
                    desktopCaptureReference(record) != null)
                  BrowserCapturePreview(
                    bridge: bridge!,
                    capture: desktopCaptureReference(record)!['id'],
                    desktopSession: desktopCaptureReference(record)!['session'],
                  ),
                if (record['name'] == 'browser' &&
                    bridge != null &&
                    browserCaptureId(record) != null)
                  BrowserCapturePreview(
                    bridge: bridge!,
                    capture: browserCaptureId(record)!,
                  ),
                if (record['name'] == 'delegate_tasks' &&
                    subagentReports(record).isNotEmpty)
                  SubagentCards(children: subagentReports(record)),
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
                    '${record['name'] == 'delegate_tasks'
                        ? 'Plan'
                        : record['name'] == 'inspect_harness'
                        ? 'Inspection'
                        : record['name'] == 'harness_repair'
                        ? 'Repair'
                        : 'Search'}: ${record['query']}',
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
