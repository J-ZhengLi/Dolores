import 'package:flutter/material.dart';

import 'chat.dart';
import 'inspector.dart';

const accessModes = {
  'review': 'Review',
  'auto': 'Auto edit',
  'fullAccess': 'Full access',
};
const accessHints = {
  'review': 'Ask before each operation.',
  'auto': 'Edit project files; ask before commands and external tools.',
  'fullAccess': 'Run available tools without routine approval prompts. Commands can access other folders. Browser input, desktop control and self-updates keep their own reviews.',
};
Map<String, dynamic> accessPolicy(String mode) => {
  'mode': mode,
  'expiresAt': null,
  'grants': mode == 'auto'
      ? [
          for (final tool in [
            'list_folder',
            'search_text',
            'read_text_file',
            'edit_text_file',
            'create_text_file',
          ])
            {
              'tool': tool,
              'pathPrefix': '.',
              'command': null,
              'mcpConnection': null,
              'mcpRevision': null,
            },
        ]
      : <Map<String, dynamic>>[],
};
bool customAccess(Map report) {
  if (report['expired'] == true || report['policy']['mode'] != 'auto') {
    return false;
  }
  final policy = report['policy'] as Map;
  final grants = (policy['grants'] as List).cast<Map>();
  final expected = (accessPolicy('auto')['grants'] as List).cast<Map>();
  return policy['expiresAt'] != null ||
      grants.length != expected.length ||
      expected.any(
        (e) =>
            grants
                .where(
                  (g) =>
                      g['tool'] == e['tool'] &&
                      g['pathPrefix'] == '.' &&
                      g['command'] == null &&
                      g['mcpConnection'] == null &&
                      g['mcpRevision'] == null,
                )
                .length !=
            1,
      );
}

Future<Map<String, dynamic>> saveAccessMode(
  ChatController chat,
  String session,
  int revision,
  String mode,
) async {
  if (!accessModes.containsKey(mode)) {
    throw StateError('Choose an access mode.');
  }
  if (chat.busy && mode != 'review') {
    throw StateError('Stop the task before changing access.');
  }
  final result = Map<String, dynamic>.from(
    await chat.bridge.call({
      'command': 'setTaskPermissions',
      'session': session,
      'revision': revision,
      'policy': accessPolicy(mode),
    }) as Map,
  );
  chat.invalidateContext();
  return result;
}

Future<void> showTaskPermissions(BuildContext context, ChatController chat) =>
    showDialog<void>(
      context: context,
      builder: (_) => TaskPermissionsInspector(chat: chat),
    );

class TaskPermissionsInspector extends StatefulWidget {
  final ChatController chat;
  const TaskPermissionsInspector({super.key, required this.chat});
  @override
  State<TaskPermissionsInspector> createState() => _TaskPermissionsState();
}

class _TaskPermissionsState extends State<TaskPermissionsInspector> {
  late final String session = widget.chat.session!;
  Map<String, dynamic>? report;
  bool pending = false;
  String? error;
  @override
  void initState() {
    super.initState();
    load();
  }

  Future<void> load() async {
    setState(() => pending = true);
    try {
      final value = Map<String, dynamic>.from(
        await widget.chat.bridge.call({
          'command': 'taskPermissions',
          'session': session,
        }) as Map,
      );
      if (mounted) {
        setState(() {
          report = value;
          error = null;
        });
      }
    } catch (failure) {
      if (mounted) setState(() => error = '$failure');
    } finally {
      if (mounted) setState(() => pending = false);
    }
  }

  Future<void> select(String mode) async {
    if (pending || report == null) return;
    setState(() {
      pending = true;
      error = null;
    });
    try {
      final value = await saveAccessMode(
        widget.chat,
        session,
        report!['revision'] as int,
        mode,
      );
      if (mounted) setState(() => report = value);
    } catch (failure) {
      if (mounted) setState(() => error = '$failure');
    } finally {
      if (mounted) setState(() => pending = false);
    }
  }

  @override
  Widget build(BuildContext context) => AnimatedBuilder(
    animation: widget.chat,
    builder: (context, _) {
      final mode = report?['expired'] == true
          ? 'review'
          : report?['policy']['mode'];
      final custom = report != null && customAccess(report!);
      return PopScope(
        canPop: !pending,
        child: InspectorFrame(
          title: 'Chat access',
          subtitle: 'Remembered for this chat',
          canClose: !pending,
          child: SizedBox(
            width: 500,
            child: SingleChildScrollView(
              child: Padding(
                padding: const EdgeInsets.all(16),
                child: Column(
                  mainAxisSize: MainAxisSize.min,
                  crossAxisAlignment: CrossAxisAlignment.start,
                  children: [
                    if (error != null) ...[
                      SelectableText(error!),
                      TextButton(
                        onPressed: pending ? null : load,
                        child: const Text('Refresh'),
                      ),
                    ],
                    if (report?['expired'] == true)
                      const Text('Previous access expired. Choose a mode.'),
                    if (custom)
                      const Text(
                        'Custom access is saved. Choose a mode to replace it.',
                      ),
                    for (final entry in accessModes.entries)
                      ListTile(
                        key: Key('access-mode-${entry.key}'),
                        contentPadding: EdgeInsets.zero,
                        leading: Icon(
                          !custom && mode == entry.key
                              ? Icons.check_circle_outline
                              : Icons.circle_outlined,
                        ),
                        title: Row(
                          children: [
                            Text(entry.value),
                            const SizedBox(width: 8),
                            Tooltip(
                              message: accessHints[entry.key]!,
                              triggerMode: TooltipTriggerMode.manual,
                              child: Icon(
                                Icons.help_outline,
                                size: 16,
                                semanticLabel: accessHints[entry.key],
                              ),
                            ),
                          ],
                        ),
                        selected: !custom && mode == entry.key,
                        enabled:
                            !pending &&
                            report != null &&
                            (!widget.chat.busy || entry.key == 'review'),
                        onTap: () => select(entry.key),
                      ),
                    if (widget.chat.busy)
                      const Text('Choose Review to stop automatic access.'),
                    if (pending) const LinearProgressIndicator(),
                  ],
                ),
              ),
            ),
          ),
        ),
      );
    },
  );
}
