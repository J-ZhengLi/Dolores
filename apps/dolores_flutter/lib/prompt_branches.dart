import 'dart:convert';

import 'package:flutter/material.dart';

import 'chat.dart';
import 'form_field_layout.dart';
import 'inspector.dart';
import 'worktrees.dart';

int _sequence = 0;

Future<void> showPromptBranch(
  BuildContext context,
  ChatController chat,
  int message,
) => showDialog<void>(
  context: context,
  barrierDismissible: false,
  builder: (_) => PromptBranchEditor(chat: chat, message: message),
);

class PromptBranchEditor extends StatefulWidget {
  final ChatController chat;
  final int message;
  const PromptBranchEditor({
    super.key,
    required this.chat,
    required this.message,
  });
  @override
  State<PromptBranchEditor> createState() => _PromptBranchEditorState();
}

class _PromptBranchEditorState extends State<PromptBranchEditor> {
  final edited = TextEditingController();
  late final parent = widget.chat.session!;
  final id = 'edit-${DateTime.now().microsecondsSinceEpoch}-${_sequence++}';
  Map<String, dynamic>? preview, attempt;
  bool busy = false,
      initialized = false,
      includeAttachments = false,
      includeContext = false;
  String? error, folder;
  Map<String, dynamic>? managed;

  Future<void> chooseManaged() async {
    if (busy || attempt != null) return;
    try {
      await showWorktrees(
        context,
        widget.chat,
        forPromptBranch: true,
        onSelect: (target) async {
          final record = await widget.chat.bridge.call({
            'command': 'worktrees',
            'session': target,
            'request': {'action': 'inspect'},
          }) as Map;
          final checked = Map<String, dynamic>.from(
            await widget.chat.branchCall(parent, {
              'action': 'target',
              'target': target,
              'revision': record['revision'],
            }),
          );
          if (mounted) {
            setState(() {
              managed = checked;
              error = null;
            });
          }
        },
      );
    } catch (e) {
      if (mounted) setState(() => error = '$e');
    }
  }

  @override
  void initState() {
    super.initState();
    WidgetsBinding.instance.addPostFrameCallback((_) {
      if (mounted) load();
    });
  }

  @override
  void dispose() {
    edited.dispose();
    super.dispose();
  }

  Future<void> load() async {
    if (busy) return;
    setState(() {
      busy = true;
      error = null;
    });
    try {
      final value = Map<String, dynamic>.from(
        await widget.chat.branchCall(parent, {
          'action': 'preview',
          'message': widget.message,
        }),
      );
      if (mounted) {
        setState(() {
          preview = value;
          attempt = null;
          if (!initialized) {
            edited.text = value['originalPrompt'] as String;
            initialized = true;
          }
          if (value['workspace']['root'] == null) folder = 'side';
        });
      }
    } catch (e) {
      if (mounted) setState(() => error = '$e');
    }
    if (mounted) setState(() => busy = false);
  }

  Future<void> open() async {
    if (busy ||
        preview == null ||
        folder == null ||
        folder == 'managed' && managed == null) {
      return;
    }
    if (utf8.encode(edited.text).length > 16384 ||
        edited.text.contains('\u0000') ||
        (edited.text.trim().isEmpty &&
            (!includeAttachments ||
                (preview!['attachments'] as List).isEmpty))) {
      setState(
        () => error = 'Keep the edited prompt within 16 KiB and include text or a prompt attachment. Your edit remains editable.',
      );
      return;
    }
    setState(() {
      busy = true;
      error = null;
    });
    attempt ??= {
      'id': folder == 'managed' ? managed!['session'] : id,
      'basis': preview!['basis'],
      'edited': edited.text,
      'includeAttachments': includeAttachments,
      'includeContext': includeContext,
      'folder': folder,
      if (folder == 'managed') 'targetRevision': managed!['revision'],
    };
    try {
      await widget.chat.createPromptBranch(parent, attempt!);
      if (mounted) Navigator.pop(context);
    } catch (e) {
      if (mounted) setState(() => error = '$e');
    }
    if (mounted) setState(() => busy = false);
  }

  @override
  Widget build(BuildContext context) {
    final p = preview;
    final locked = busy || attempt != null;
    return PopScope(
      canPop: !busy,
      child: InspectorFrame(
        title: 'Edit in a new branch',
        subtitle: 'Keep the original conversation and current files',
        canClose: !busy,
        child: Padding(
          padding: const EdgeInsets.all(16),
          child: Column(
            crossAxisAlignment: CrossAxisAlignment.start,
            children: [
              Expanded(
                child: SingleChildScrollView(
                  child: Column(
                    crossAxisAlignment: CrossAxisAlignment.start,
                    children: [
                      const Text(
                        'Opening creates a separate chat and draft. Nothing is sent. Files stay as they are; earlier actions are not repeated.',
                      ),
                      const SizedBox(height: 16),
                      if (initialized)
                        FieldLabel(
                          label: 'Edited prompt',
                          child: TextField(
                            key: const Key('branch-edit'),
                            controller: edited,
                            readOnly: locked,
                            onChanged: (_) => setState(() {}),
                            minLines: 4,
                            maxLines: 10,
                            decoration: const InputDecoration(),
                          ),
                        ),
                      if (initialized)
                        Text(
                          '${utf8.encode(edited.text).length} / 16384 bytes',
                          style: Theme.of(context).textTheme.bodySmall,
                        ),
                      if (p != null) ...[
                        const SizedBox(height: 12),
                        Text(
                          '${p['priorTurns']} earlier exchanges enter this chat. The original ${p['archivedTurns'] ?? (p['priorTurns'] as int) + (p['laterTurns'] as int)} exchanges and receipts stay in its local snapshot.',
                        ),
                        Text(
                          '${p['effectTurns']} exchanges contain tool receipts. Review current files before rerunning.',
                        ),
                        if ((p['attachments'] as List).isNotEmpty)
                          CheckboxListTile(
                            contentPadding: EdgeInsets.zero,
                            title: const Text(
                              'Include original prompt attachments',
                            ),
                            subtitle: Text(
                              (p['attachments'] as List)
                                  .map((a) => a['name'])
                                  .join(', '),
                            ),
                            value: includeAttachments,
                            onChanged: locked
                                ? null
                                : (value) => setState(
                                    () => includeAttachments = value!,
                                  ),
                          ),
                        if ((p['context'] as List).isNotEmpty) ...[
                          CheckboxListTile(
                            contentPadding: EdgeInsets.zero,
                            title: const Text(
                              'Include current context snapshots',
                            ),
                            subtitle: Text(
                              '${(p['context'] as List).length} retained excerpts; plans and goals stay with the original',
                            ),
                            value: includeContext,
                            onChanged: locked
                                ? null
                                : (value) =>
                                      setState(() => includeContext = value!),
                          ),
                          ExpansionTile(
                            tilePadding: EdgeInsets.zero,
                            title: const Text('Review context snapshots'),
                            children: [
                              ConstrainedBox(
                                constraints: const BoxConstraints(
                                  maxHeight: 180,
                                ),
                                child: SingleChildScrollView(
                                  child: SelectableText(
                                    (p['context'] as List)
                                        .map(
                                          (v) =>
                                              '${v['kind']} · ${v['source']}\n${v['text']}\nOmitted: ${v['omissions']}',
                                        )
                                        .join('\n\n'),
                                  ),
                                ),
                              ),
                            ],
                          ),
                        ],
                        const SizedBox(height: 12),
                        FieldLabel(
                          label: 'Working folder',
                          child: DropdownButtonFormField<String>(
                            key: const Key('branch-folder'),
                            initialValue: folder,
                            hint: const Text(
                              'Choose where this branch will work',
                            ),
                            isExpanded: true,
                            items: [
                              if (p['workspace']['root'] != null)
                                const DropdownMenuItem(
                                  value: 'shared',
                                  child: Text('Share current folder'),
                                ),
                              const DropdownMenuItem(
                                value: 'side',
                                child: Text('No working folder'),
                              ),
                              if (p['workspace']['root'] != null &&
                                  widget.chat.worktreesAvailable)
                                const DropdownMenuItem(
                                  value: 'managed',
                                  child: Text('Managed worktree'),
                                ),
                            ],
                            onChanged: locked
                                ? null
                                : (value) => setState(() => folder = value),
                          ),
                        ),
                        if (folder == 'shared')
                          const Padding(
                            padding: EdgeInsets.only(top: 8),
                            child: Text(
                              'Both chats use the same current files. This does not restore an older version.',
                            ),
                          ),
                        if (folder == 'managed') ...[
                          const SizedBox(height: 8),
                          const Text(
                            'Starts at a reviewed commit in a separate folder. Source dirty files and staging stay in the original checkout.',
                          ),
                          TextButton(
                            onPressed: locked ? null : chooseManaged,
                            child: Text(
                              managed == null
                                  ? 'Choose or create worktree'
                                  : 'Choose another worktree',
                            ),
                          ),
                          if (managed != null) ...[
                            SelectableText(managed!['root'] as String),
                            Text(
                              '${managed!['baseRef']} · ${managed!['baseCommit']}',
                              style: Theme.of(context).textTheme.bodySmall,
                            ),
                            const Text(
                              'The checkout is retained even if attaching this prompt fails. Retry this target or inspect it in Worktrees; opening the edited branch sends nothing.',
                            ),
                          ],
                        ],
                      ],
                      if (error != null)
                        Padding(
                          padding: const EdgeInsets.symmetric(vertical: 12),
                          child: SelectableText(error!),
                        ),
                    ],
                  ),
                ),
              ),
              if (busy) const LinearProgressIndicator(),
              const SizedBox(height: 12),
              Wrap(
                spacing: 8,
                runSpacing: 8,
                children: [
                  OutlinedButton(
                    onPressed: busy ? null : load,
                    child: const Text('Refresh source preview'),
                  ),
                  FilledButton(
                    onPressed:
                        busy ||
                            p == null ||
                            folder == null ||
                            folder == 'managed' && managed == null
                        ? null
                        : open,
                    child: Text(
                      attempt == null
                          ? 'Open edited branch'
                          : 'Retry opening saved branch',
                    ),
                  ),
                ],
              ),
            ],
          ),
        ),
      ),
    );
  }
}

class PromptBranchBanner extends StatelessWidget {
  final ChatController chat;
  const PromptBranchBanner({super.key, required this.chat});
  @override
  Widget build(BuildContext context) {
    final info = chat.promptBranch;
    if (info == null && chat.promptBranchError == null) {
      return const SizedBox.shrink();
    }
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        if (chat.promptBranchError != null)
          Wrap(
            children: [
              Text(chat.promptBranchError!),
              TextButton(
                onPressed: chat.refreshPromptBranch,
                child: const Text('Retry branch link'),
              ),
            ],
          ),
        if (info != null) ...[
          Text(
            'Edited branch · ${info['origin']['parentTitle']}',
            style: Theme.of(context).textTheme.bodySmall,
          ),
          Text(switch (info['origin']['folder']) {
            'shared' => 'Current folder is shared. Files were not rolled back.',
            'managed' => 'Separate managed worktree. Source files were kept; historical actions were not repeated.',
            _ => 'No working folder. Historical actions were not repeated.',
          }, style: Theme.of(context).textTheme.bodySmall),
          Wrap(
            spacing: 8,
            children: [
              TextButton.icon(
                key: const Key('branch-parent'),
                icon: const Icon(Icons.arrow_back, size: 16),
                label: const Text('Original chat'),
                onPressed: info['parentAvailable'] != true
                    ? null
                    : () async {
                        try {
                          await chat.navigateConversation(
                            info['origin']['parent'] as String,
                          );
                        } catch (e) {
                          if (context.mounted) {
                            ScaffoldMessenger.of(context)
                                .showSnackBar(SnackBar(content: Text('$e')));
                          }
                        }
                      },
              ),
              TextButton(
                onPressed: () => showDialog<void>(
                  context: context,
                  builder: (_) =>
                      BranchArchive(chat: chat, owner: chat.session!),
                ),
                child: const Text('Original snapshot and receipts'),
              ),
            ],
          ),
          if (info['parentAvailable'] != true)
            const Text(
              'Original chat is unavailable. Its retained snapshot remains readable here.',
            ),
          if (info['parentAvailable'] == true &&
              info['sourceAvailable'] != true)
            const Text(
              'The original chat has changed. This branch keeps the source as it was when created.',
            ),
        ],
      ],
    );
  }
}

class BranchArchive extends StatefulWidget {
  final ChatController chat;
  final String owner;
  const BranchArchive({super.key, required this.chat, required this.owner});
  @override
  State<BranchArchive> createState() => _BranchArchiveState();
}

class _BranchArchiveState extends State<BranchArchive> {
  Map<String, dynamic>? page;
  bool busy = false;
  String? error;
  final previous = <int>[];
  @override
  void initState() {
    super.initState();
    WidgetsBinding.instance.addPostFrameCallback((_) {
      if (mounted) load(0);
    });
  }

  Future<void> load(int offset, {bool back = false}) async {
    if (busy) return;
    setState(() {
      busy = true;
      error = null;
    });
    try {
      final value = Map<String, dynamic>.from(
        await widget.chat.branchCall(widget.owner, {
          'action': 'archive',
          'offset': offset,
        }),
      );
      if (mounted) {
        setState(() {
          if (page != null && offset != page!['offset']) {
            if (back) {
              previous.removeLast();
            } else {
              previous.add(page!['offset'] as int);
            }
          }
          page = value;
        });
      }
    } catch (e) {
      if (mounted) setState(() => error = '$e');
    }
    if (mounted) setState(() => busy = false);
  }

  @override
  Widget build(BuildContext context) => InspectorFrame(
    title: 'Original snapshot and receipts',
    subtitle: 'Historical text and evidence; opening runs no action',
    child: Padding(
      padding: const EdgeInsets.all(16),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          const Text(
            'This local snapshot is not added wholesale to model context. Earlier effects remain historical; inspect current files before rerunning.',
          ),
          if (page != null)
            Text(
              'Bytes ${page!['offset']}–${(page!['offset'] as int) + utf8.encode(page!['text'] as String).length} of ${page!['totalBytes']}',
            ),
          if (error != null) SelectableText(error!),
          if (busy) const LinearProgressIndicator(),
          Expanded(
            child: SingleChildScrollView(
              key: ValueKey(page?['offset']),
              child: SelectableText(
                page?['text'] as String? ?? 'Loading retained snapshot…',
              ),
            ),
          ),
          Wrap(
            spacing: 8,
            children: [
              TextButton(
                onPressed: busy || previous.isEmpty
                    ? null
                    : () => load(previous.last, back: true),
                child: const Text('Previous page'),
              ),
              TextButton(
                onPressed: busy || page?['nextOffset'] == null
                    ? null
                    : () => load(page!['nextOffset'] as int),
                child: const Text('Next page'),
              ),
              if (error != null)
                TextButton(
                  onPressed: busy
                      ? null
                      : () => load(page?['offset'] as int? ?? 0),
                  child: const Text('Retry snapshot'),
                ),
            ],
          ),
        ],
      ),
    ),
  );
}
