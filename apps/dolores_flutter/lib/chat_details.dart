import 'package:flutter/material.dart';

import 'changes.dart';
import 'chat.dart';
import 'inspector.dart';
import 'run_history.dart';
import 'session_summary.dart';
import 'settings_frame.dart';
import 'theme.dart';
import 'usage_details.dart';
import 'instructions.dart';

enum ChatDetail { context, activity, changes }

Future<void> showChatDetails(
  BuildContext context,
  ChatController chat, {
  ChatDetail initial = ChatDetail.context,
}) async {
  await showDialog<void>(
    context: context,
    barrierDismissible: false,
    builder: (_) => ChatDetails(chat: chat, initial: initial),
  );
  chat.invalidateContext();
}

class ChatDetails extends StatefulWidget {
  final ChatController chat;
  final ChatDetail initial;
  const ChatDetails({
    super.key,
    required this.chat,
    this.initial = ChatDetail.context,
  });
  @override
  State<ChatDetails> createState() => _ChatDetailsState();
}

class _ChatDetailsState extends State<ChatDetails> {
  late ChatDetail selected = widget.initial;
  final panels = <ChatDetail, Widget>{};
  final locks = <ChatDetail, ValueNotifier<bool>>{};
  final drafts = <ChatDetail, SettingsDraft>{};
  bool closing = false;
  bool get pending => closing || locks.values.any((v) => v.value);
  void changed() {
    if (mounted) setState(() {});
  }

  Future<void> close() async {
    if (pending) return;
    final unsaved = drafts.entries.where((e) => e.value.dirty).toList();
    if (unsaved.isNotEmpty &&
        !await resolveSettingsDraft(
          context,
          save: () async {
            setState(() => closing = true);
            try {
              for (final entry in unsaved) {
                if (!await entry.value.save!()) {
                  if (mounted) setState(() => selected = entry.key);
                  return false;
                }
              }
              return true;
            } finally {
              if (mounted) setState(() => closing = false);
            }
          },
        )) {
      return;
    }
    if (mounted) Navigator.pop(context);
  }

  @override
  void dispose() {
    for (final lock in locks.values) {
      lock.dispose();
    }
    super.dispose();
  }

  Widget create(ChatDetail page) => switch (page) {
    ChatDetail.context => ContextHome(chat: widget.chat),
    ChatDetail.activity => Column(
      children: [
        if (widget.chat.session != null)
          Align(
            alignment: Alignment.centerRight,
            child: TextButton.icon(
              onPressed: () => showRunHistory(context, widget.chat),
              icon: const Icon(Icons.history, size: 18),
              label: const Text('Earlier tasks'),
            ),
          ),
        Expanded(child: TrajectoryInspector(chat: widget.chat)),
      ],
    ),
    ChatDetail.changes =>
      widget.chat.session != null && widget.chat.workspaceRoot != null
          ? ChangesInspector(
              bridge: widget.chat.bridge,
              session: widget.chat.session!,
            )
          : const Center(
              child: Padding(
                padding: EdgeInsets.all(24),
                child: Text(
                  'File changes appear in project and temporary working chats.',
                ),
              ),
            ),
  };
  @override
  Widget build(BuildContext context) {
    final p = Palette(Theme.of(context).brightness == Brightness.dark);
    panels.putIfAbsent(selected, () {
      final lock = ValueNotifier(false)..addListener(changed);
      locks[selected] = lock;
      final draft = SettingsDraft();
      drafts[selected] = draft;
      return SettingsEmbedding(
        route: ModalRoute.of(context),
        pending: lock,
        draft: draft,
        child: create(selected),
      );
    });
    return CloseOnEscape(
      onClose: close,
      child: PopScope(
        canPop: false,
        onPopInvokedWithResult: (didPop, _) {
          if (!didPop) close();
        },
        child: Dialog(
          key: const Key('chat-details'),
          alignment: Alignment.centerRight,
          insetPadding: EdgeInsets.zero,
          backgroundColor: p.bg,
          shape: const RoundedRectangleBorder(),
          child: SizedBox(
            width: 520,
            height: double.infinity,
            child: Column(
              children: [
                Padding(
                  padding: const EdgeInsets.fromLTRB(16, 12, 8, 8),
                  child: Row(
                    children: [
                      const Expanded(
                        child: Text(
                          'Chat details',
                          style: TextStyle(
                            fontSize: 18,
                            fontWeight: FontWeight.w600,
                          ),
                        ),
                      ),
                      IconButton(
                        key: const Key('close-chat-details'),
                        tooltip: 'Close details',
                        onPressed: pending ? null : close,
                        icon: const Icon(Icons.close),
                      ),
                    ],
                  ),
                ),
                Padding(
                  padding: const EdgeInsets.symmetric(horizontal: 12),
                  child: Row(
                    children: [
                      for (final page in ChatDetail.values)
                        Expanded(
                          child: Padding(
                            padding: const EdgeInsets.symmetric(horizontal: 2),
                            child: ChoiceChip(
                              label: Text(switch (page) {
                                ChatDetail.context => 'Context',
                                ChatDetail.activity => 'Activity',
                                ChatDetail.changes => 'Changes',
                              }),
                              selected: selected == page,
                              onSelected: pending
                                  ? null
                                  : (_) => setState(() => selected = page),
                            ),
                          ),
                        ),
                    ],
                  ),
                ),
                const Divider(height: 20),
                if (widget.chat.workspaceRoot != null &&
                    widget.chat.session != null)
                  Align(
                    alignment: Alignment.centerRight,
                    child: TextButton.icon(
                      onPressed: pending || widget.chat.busy
                          ? null
                          : () => showInstructions(context, widget.chat),
                      icon: const Icon(Icons.rule_folder_outlined, size: 18),
                      label: const Text('Project instructions'),
                    ),
                  ),
                Expanded(
                  child: IndexedStack(
                    index: selected.index,
                    children: [
                      for (final page in ChatDetail.values)
                        panels[page] ?? const SizedBox.shrink(),
                    ],
                  ),
                ),
              ],
            ),
          ),
        ),
      ),
    );
  }
}

class ContextHome extends StatefulWidget {
  final ChatController chat;
  const ContextHome({super.key, required this.chat});
  @override
  State<ContextHome> createState() => _ContextHomeState();
}

class _ContextHomeState extends State<ContextHome> {
  Map<String, dynamic>? report;
  bool pending = false;
  String? error;
  @override
  void initState() {
    super.initState();
    WidgetsBinding.instance.addPostFrameCallback((_) {
      if (mounted) load();
    });
  }

  Future<void> load() async {
    if (pending) return;
    setState(() {
      pending = true;
      error = null;
    });
    try {
      final value = await widget.chat.previewContext();
      if (mounted) {
        setState(() {
          if (value != null) {
            report = value;
          } else {
            error = widget.chat.error ?? 'Context is unavailable. Try Refresh.';
          }
        });
      }
    } finally {
      if (mounted) setState(() => pending = false);
    }
  }

  @override
  Widget build(BuildContext context) {
    final tokens = report?['tokens'] as Map?;
    final used = tokens?['inputTokens'] as int?,
        limit = tokens?['contextWindowTokens'] as int?;
    return Column(
      children: [
        Padding(
          padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 8),
          child: Column(
            crossAxisAlignment: CrossAxisAlignment.stretch,
            children: [
              if (pending) const LinearProgressIndicator(),
              Row(
                children: [
                  Expanded(
                    child: Text(
                      used == null
                          ? 'Context usage unavailable'
                          : '${formatTokens(used)} / ${formatTokens(limit)} tokens',
                      style: Theme.of(context).textTheme.titleMedium,
                    ),
                  ),
                  IconButton(
                    tooltip: 'Refresh context',
                    onPressed: pending ? null : load,
                    icon: const Icon(Icons.refresh, size: 18),
                  ),
                  IconButton(
                    tooltip: 'Usage details',
                    onPressed: report == null
                        ? null
                        : () => showContextPreview(context, report!),
                    icon: const Icon(Icons.info_outline, size: 18),
                  ),
                ],
              ),
              if (used != null && limit != null && limit > 0)
                Padding(
                  padding: const EdgeInsets.only(top: 10),
                  child: LinearProgressIndicator(
                    value: (used / limit).clamp(0, 1).toDouble(),
                  ),
                ),
              const SizedBox(height: 6),
              if (MediaQuery.sizeOf(context).height >= 560)
                const Text(
                  'Estimated for your next message. Full history stays saved.',
                ),
              if (error != null)
                ConstrainedBox(
                  constraints: const BoxConstraints(maxHeight: 60),
                  child: SingleChildScrollView(child: Text(error!)),
                ),
            ],
          ),
        ),
        Expanded(
          child: widget.chat.session == null
              ? const Center(
                  child: Text('A summary is available after starting a chat.'),
                )
              : SessionSummaryInspector(
                  bridge: widget.chat.bridge,
                  session: widget.chat.session!,
                ),
        ),
      ],
    );
  }
}
