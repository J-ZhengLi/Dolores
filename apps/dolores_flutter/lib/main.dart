import 'access_selector.dart';
import 'memory.dart';
import 'chat_details.dart';

import 'dart:async';

import 'package:flutter/material.dart';
import 'package:file_selector/file_selector.dart';

import 'bridge.dart';
import 'chat.dart';
import 'composer_controller.dart';
import 'reply_content.dart';
import 'message_frame.dart';
import 'rich_composer.dart';
import 'usage_details.dart';
import 'task_feedback.dart';
import 'inspector.dart';
import 'tool_activity.dart';
import 'subagents.dart';
import 'workspace_picker.dart';
import 'instructions.dart';
import 'chat_sidebar.dart';
import 'model_steps.dart';
import 'thread_fork.dart';
import 'attachments.dart';

import 'theme.dart';
import 'settings.dart';
import 'desktop_frame.dart';
import 'desktop_share.dart';
import 'sidebar_resize.dart';
export 'model_settings.dart' show ConnectionDialog;
export 'theme.dart' show Palette;

Future<void> main() async {
  WidgetsFlutterBinding.ensureInitialized();
  final desktopFrame = await initializeDesktopFrame();
  final chat = ChatController(NativeBridge());
  runApp(DoloresApp(chat: chat, desktopFrame: desktopFrame));
  unawaited(chat.initialize());
}

class DoloresApp extends StatelessWidget {
  final ChatController chat;
  final ThemeMode? themeMode;
  final GlobalKey? captureKey;
  final bool desktopFrame;
  const DoloresApp({
    super.key,
    required this.chat,
    this.themeMode,
    this.captureKey,
    this.desktopFrame = false,
  });
  @override
  Widget build(BuildContext context) => AnimatedBuilder(
    animation: chat.appearanceChanges,
    builder: (context, _) => MaterialApp(
      title: 'Dolores',
      debugShowCheckedModeBanner: false,
      theme: doloresTheme(false),
      darkTheme: doloresTheme(true),
      themeMode:
          themeMode ??
          switch (chat.appearance) {
            'light' => ThemeMode.light,
            'dark' => ThemeMode.dark,
            _ => ThemeMode.system,
          },
      builder: (context, child) => RepaintBoundary(
        key: captureKey,
        child: desktopFrame ? DesktopFrame(child: child!) : child!,
      ),
      home: ChatPage(chat: chat),
    ),
  );
}

class ChatPage extends StatefulWidget {
  final ChatController chat;
  const ChatPage({super.key, required this.chat});
  @override
  State<ChatPage> createState() => _ChatPageState();
}

class _ChatPageState extends State<ChatPage> {
  final input = ComposerController();
  final focus = FocusNode();
  final scroll = ScrollController();
  final shell = GlobalKey<ScaffoldState>();
  ChatController get chat => widget.chat;
  bool following = true;
  double sidebarWidth = UiTokens.sidebarWidth;
  int seenRevision = -1;
  @override
  void initState() {
    super.initState();
    chat.addListener(_changed);
    input.text = chat.draft;
    scroll.addListener(() {
      if (scroll.hasClients) following = scroll.position.extentAfter < 80;
      if (scroll.hasClients) chat.rememberScroll(scroll.offset);
    });
    WidgetsBinding.instance.addPostFrameCallback((_) => _changed());
  }

  void _changed() {
    if (!mounted) return;
    if (input.text != chat.draft) {
      input.clearHistory();
      input.value = TextEditingValue(
        text: chat.draft,
        selection: TextSelection.collapsed(offset: chat.draft.length),
      );
    }
    setState(() {});
    if (seenRevision != chat.viewRevision) {
      seenRevision = chat.viewRevision;
      final offset = chat.scrollOffset;
      WidgetsBinding.instance.addPostFrameCallback((_) {
        if (mounted && scroll.hasClients) {
          scroll.jumpTo(offset.clamp(0, scroll.position.maxScrollExtent));
          following = scroll.position.extentAfter < 80;
        }
      });
      return;
    }
    _followReply();
  }

  void _followReply() {
    if (following) {
      WidgetsBinding.instance.addPostFrameCallback((_) {
        if (mounted && following && scroll.hasClients) {
          scroll.jumpTo(scroll.position.maxScrollExtent);
        }
      });
    }
  }

  @override
  void dispose() {
    chat.removeListener(_changed);
    input.dispose();
    focus.dispose();
    scroll.dispose();
    super.dispose();
  }

  Future<void> settings() async {
    shell.currentState?.closeDrawer();
    await showSettings(context, chat, initial: SettingsCategory.models);
    if (mounted) focus.requestFocus();
  }

  Future<void> requestSettings() async {
    shell.currentState?.closeDrawer();
    await showSettings(
      context,
      chat,
      initial: SettingsCategory.models,
      modelsPage: ModelsPage.responses,
    );
    if (mounted) focus.requestFocus();
  }

  Future<void> openProject() async {
    await chat.chooseToolFolder(() => getDirectoryPath());
    if (mounted && chat.error == null) {
      shell.currentState?.closeDrawer();
      focus.requestFocus();
    }
  }

  Future<void> exportChat(String format) async {
    final count = await chat.exportConversation(format, () async {
      final extension = format == 'json' ? 'json' : 'md';
      final result = await getSaveLocation(
        suggestedName: 'dolores-${chat.session}.$extension',
        acceptedTypeGroups: [
          XTypeGroup(
            label: format == 'json' ? 'JSON' : 'Markdown',
            extensions: [extension],
            uniformTypeIdentifiers: ['public.text'],
          ),
        ],
      );
      return result?.path;
    });
    if (mounted && count != null) {
      ScaffoldMessenger.of(context)
          .showSnackBar(SnackBar(content: Text('Exported $count messages.')));
    }
  }

  Future<void> chooseExport() async {
    final choice = await showDialog<String>(
      context: context,
      builder: (context) => SimpleDialog(
        title: const Text('Export chat'),
        children: [
          for (final option in [
            ('markdown', 'Markdown'),
            ('json', 'JSON with run details'),
            ('attachments', 'Attachments'),
          ])
            SimpleDialogOption(
              onPressed: () => Navigator.pop(context, option.$1),
              child: Text(option.$2),
            ),
        ],
      ),
    );
    if (!mounted || choice == null) return;
    if (choice == 'attachments') {
      await exportAttachments(context, chat);
    } else {
      await exportChat(choice);
    }
  }

  Widget sidebar(Palette p, {double width = UiTokens.sidebarWidth}) =>
      ChatSidebar(
        width: width,
        chat: chat,
        onNewTemporary: () {
          chat.newChat(kind: 'temporary');
          shell.currentState?.closeDrawer();
          focus.requestFocus();
        },
        onNewSide: () {
          chat.newChat(kind: 'side');
          shell.currentState?.closeDrawer();
          focus.requestFocus();
        },
        onOpenProject: openProject,
        onProject: (root) async {
          await chat.openProject(root);
          if (mounted && chat.error == null) {
            shell.currentState?.closeDrawer();
            focus.requestFocus();
          }
        },
        onSelect: (id) async {
          await chat.select(id);
          if (mounted && chat.error == null) {
            shell.currentState?.closeDrawer();
          }
        },
        onSettings: () {
          shell.currentState?.closeDrawer();
          showSettings(context, chat);
        },
      );

  Widget message(
    Palette p,
    String role,
    String text, {
    bool streaming = false,
    Key? key,
    Map<String, dynamic>? metadata,
    int? messageId,
    int? savedAt,
    Map<String, dynamic>? feedback,
    List<Map<String, dynamic>> parts = const [],
  }) {
    final user = role == 'user';
    return MessageFrame(
      key: key,
      user: user,
      text: text,
      streaming: streaming,
      savedAt: savedAt,
      onRemember:
          user && !streaming && text.isNotEmpty && !chat.busy && !chat.changing
          ? () async {
              await showDialog<void>(
                context: context,
                barrierDismissible: false,
                builder: (_) => MemoryInspector(
                  bridge: chat.bridge,
                  session: chat.session,
                  initialText: text,
                ),
              );
              chat.invalidateContext();
            }
          : null,
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          if (streaming)
            Text(
              chat.workspaceRoot == null ? 'Writing…' : 'Working…',
              style: TextStyle(fontSize: 11, color: p.muted),
            ),
          if (!user && streaming && chat.modelTexts.isNotEmpty)
            ModelSteps(steps: chat.modelTexts, saved: false),
          if (!user && !streaming && metadata?['agent']?['steps'] is List)
            ModelSteps(steps: metadata!['agent']['steps'] as List),
          if (parts.isNotEmpty) AttachmentChips(chat: chat, parts: parts),
          if (user || text.isEmpty)
            SelectableText(
              text.isEmpty && streaming ? 'Thinking…' : text,
              style: TextStyle(fontSize: 14, height: 1.65, color: p.text),
            )
          else
            ReplyContent(
              text: text,
              streaming: streaming,
              onRendered: _followReply,
            ),
          if (!user && !streaming) ...[
            if (metadata?['paused'] is Map) ...[
              const SizedBox(height: 12),
              Text(
                metadata!['paused']['reason'] == 'outputLimit'
                    ? 'Paused at the output limit (${metadata['requestSettings']?['maxOutputTokens'] ?? 'configured'} tokens). Progress saved.${metadata['usage']?['reasoningTokens'] != null ? ' Reasoning used ${metadata['usage']['reasoningTokens']} tokens.' : ''}'
                    : metadata['paused']['reason'] == 'commandReview'
                    ? 'A command failed or verification was incomplete. Review its output before repair.'
                    : metadata['paused']['reason'] == 'desktopAccess'
                    ? 'Choose a window to continue. Nothing has been shared yet.'
                    : metadata['paused']['reason'] == 'desktopReview'
                    ? 'Computer use paused. Inspect the window and fresh screenshot before reconciling saved progress.'
                    : metadata['paused']['reason'] == 'subagentReview'
                    ? 'A subagent needs review. Inspect its report, Run history and Changes before continuing.'
                    : 'Paused at this run’s step limit. Progress and tool results saved.',
                style: TextStyle(color: p.muted, fontSize: 12),
              ),
              if (messageId == chat.latestPausedId)
                Align(
                  alignment: Alignment.centerLeft,
                  child: TextButton.icon(
                    key: const Key('continue-task'),
                    onPressed:
                        chat.busy ||
                            chat.changing ||
                            chat.loading ||
                            (chat.draft.isNotEmpty &&
                                metadata['paused']['reason'] !=
                                    'desktopAccess') ||
                            (chat.attachments.isNotEmpty &&
                                metadata['paused']['reason'] != 'desktopAccess')
                        ? null
                        : () => metadata['paused']['reason'] == 'desktopAccess'
                              ? shareWindow(context, chat, handoff: true)
                              : ((metadata['agent']?['tools'] as List? ?? [])
                                        .any(
                                          (r) => r['name'] == 'desktop_control',
                                        ) ||
                                    metadata['paused']['reason'] ==
                                        'desktopReview')
                              ? showSettings(
                                  context,
                                  chat,
                                  initial: SettingsCategory.desktop,
                                )
                              : chat.continueTask(messageId!),
                    icon: const Icon(Icons.play_arrow_outlined, size: 18),
                    label: Text(
                      metadata['paused']['reason'] == 'desktopAccess'
                          ? 'Share window'
                          : metadata['paused']['reason'] == 'desktopReview' ||
                                (metadata['agent']?['tools'] as List? ?? [])
                                    .any((r) => r['name'] == 'desktop_control')
                          ? 'Inspect computer use'
                          : metadata['paused']['reason'] == 'commandReview'
                          ? 'Repair and verify'
                          : 'Continue',
                    ),
                  ),
                ),
              Text(
                chat.draft.isNotEmpty || chat.attachments.isNotEmpty
                    ? 'Send or clear your draft to continue. Each continuation uses your current model and limits.'
                    : (metadata['agent']?['tools'] as List? ?? []).any(
                        (r) => r['name'] == 'desktop_control',
                      )
                    ? 'Input may already have occurred. Inspect saved receipts and capture the original window again; approvals are never replayed.'
                    : 'Continue starts another bounded run. New tool calls need fresh approval; incomplete calls have not run.',
                style: TextStyle(color: p.muted, fontSize: 12),
              ),
              if (messageId == chat.latestPausedId)
                Wrap(
                  spacing: 8,
                  children: [
                    TextButton(
                      onPressed: chat.busy || chat.changing
                          ? null
                          : () => showChatDetails(
                              context,
                              chat,
                              initial: ChatDetail.activity,
                            ),
                      child: const Text('View progress'),
                    ),
                    if (![
                      'desktopAccess',
                      'desktopReview',
                      'commandReview',
                      'subagentReview',
                    ].contains(metadata['paused']['reason']))
                      TextButton(
                        onPressed: chat.busy || chat.changing
                            ? null
                            : () =>
                                  metadata['paused']['reason'] == 'outputLimit'
                                  ? requestSettings()
                                  : showSettings(
                                      context,
                                      chat,
                                      initial: SettingsCategory.limits,
                                    ),
                        child: const Text('Adjust limits'),
                      ),
                  ],
                ),
            ],
            if (metadata?['paused']?['reason'] == 'commandReview')
              ToolRecords(
                bridge: chat.bridge,
                records: metadata!['paused']['receipts'] as List,
                maxRecords: 16,
              )
            else if (metadata?['agent']?['tools'] is List)
              ToolRecords(
                records: metadata!['agent']['tools'] as List,
                bridge: chat.bridge,
              ),
            UsageDetails(metadata: metadata),
            if (messageId != null)
              TaskFeedbackButton(
                chat: chat,
                message: {
                  'id': messageId,
                  'content': text,
                  'metadata': metadata,
                  'feedback': feedback,
                },
              ),
          ],
        ],
      ),
    );
  }

  Widget welcome(Palette p) => Center(
    child: SingleChildScrollView(
      child: Padding(
        padding: const EdgeInsets.all(32),
        child: Column(
          mainAxisSize: MainAxisSize.min,
          children: [
            Container(
              width: 60,
              height: 60,
              decoration: BoxDecoration(
                color: p.soft,
                borderRadius: BorderRadius.circular(18),
              ),
              child: Icon(
                Icons.all_inclusive_rounded,
                size: 35,
                color: p.accent,
              ),
            ),
            const SizedBox(height: 24),
            Text(
              chat.workspaceKind == 'side'
                  ? 'A little space to think.'
                  : 'What are we working on?',
              textAlign: TextAlign.center,
              style: TextStyle(
                fontFamily: 'Georgia',
                fontSize: 30,
                color: p.text,
              ),
            ),
            const SizedBox(height: 14),
            Text(
              chat.workspaceKind == 'project'
                  ? 'Work with files in ${chat.workspaceLabel}.'
                  : chat.workspaceKind == 'side'
                  ? 'A conversation without file access.'
                  : 'Start here, or open a project. Your working folder is ready when you send.',
              textAlign: TextAlign.center,
              style: TextStyle(fontSize: 15, color: p.muted),
            ),
            const SizedBox(height: 28),
            if (chat.workspaceKind != 'project')
              TextButton.icon(
                key: const Key('welcome-open-project'),
                onPressed: chat.busy || chat.changing || chat.loading
                    ? null
                    : openProject,
                icon: const Icon(Icons.folder_open_outlined, size: 18),
                label: const Text('Open project…'),
              ),
            if (!chat.configured)
              OutlinedButton.icon(
                onPressed: chat.loading ? null : settings,
                icon: const Icon(Icons.tune, size: 17),
                label: const Text('Connect a model'),
              ),
          ],
        ),
      ),
    ),
  );
  Widget composer(Palette p) => Column(
    mainAxisSize: MainAxisSize.min,
    children: [
      if (!chat.busy && chat.modelTexts.isNotEmpty)
        ConstrainedBox(
          constraints: const BoxConstraints(maxHeight: 160),
          child: SingleChildScrollView(
            child: ModelSteps(steps: chat.modelTexts, saved: false),
          ),
        ),
      if (chat.toolRecords.isNotEmpty || chat.subagents.isNotEmpty)
        ConstrainedBox(
          constraints: const BoxConstraints(maxHeight: 160),
          child: SingleChildScrollView(
            child: Column(
              mainAxisSize: MainAxisSize.min,
              children: [
                if (chat.subagents.isNotEmpty)
                  SubagentCards(children: chat.subagents),
                ToolRecords(records: chat.toolRecords, bridge: chat.bridge),
              ],
            ),
          ),
        ),
      if (chat.busy && chat.activeDesktopTarget != null)
        Container(
          key: const Key('desktop-active-target'),
          width: double.infinity,
          padding: const EdgeInsets.all(10),
          margin: const EdgeInsets.only(bottom: 8),
          decoration: BoxDecoration(
            color: p.soft,
            borderRadius: BorderRadius.circular(10),
          ),
          child: Wrap(
            spacing: 8,
            crossAxisAlignment: WrapCrossAlignment.center,
            children: [
              Text(
                'Computer use · ${chat.activeDesktopTarget!['title']}',
                maxLines: 2,
                overflow: TextOverflow.ellipsis,
              ),
              TextButton(
                onPressed: chat.stopping ? null : chat.revokeDesktop,
                child: const Text('Revoke desktop access'),
              ),
            ],
          ),
        ),
      if (chat.toolApproval != null) ToolApprovalCard(chat: chat),
      if (chat.connectionWarning != null)
        Container(
          key: const Key('connection-warning'),
          width: double.infinity,
          margin: const EdgeInsets.only(bottom: 12),
          padding: const EdgeInsets.all(14),
          decoration: BoxDecoration(
            color: p.surface,
            border: Border.all(color: p.border),
            borderRadius: BorderRadius.circular(10),
          ),
          child: Column(
            crossAxisAlignment: CrossAxisAlignment.start,
            children: [
              Text(
                chat.connectionWarning!,
                style: TextStyle(color: p.muted, fontSize: 13),
              ),
              if (chat.rememberConnection && !chat.configured)
                TextButton(
                  key: const Key('retry-connection'),
                  onPressed: chat.busy || chat.changing
                      ? null
                      : chat.recoverConnection,
                  child: const Text('Retry saved connection'),
                ),
            ],
          ),
        ),
      if (chat.error != null)
        Container(
          key: const Key('error'),
          width: double.infinity,
          margin: const EdgeInsets.only(bottom: 12),
          padding: const EdgeInsets.all(14),
          decoration: BoxDecoration(
            color: p.errorSurface,
            borderRadius: BorderRadius.circular(10),
          ),
          child: Column(
            crossAxisAlignment: CrossAxisAlignment.start,
            mainAxisSize: MainAxisSize.min,
            children: [
              Text(
                chat.error!,
                style: TextStyle(color: p.errorText, fontSize: 13),
              ),
              if (chat.attachmentsAvailable &&
                  (chat.attachments.any(
                        (part) => (part['mime'] as String? ?? '').startsWith(
                          'image/',
                        ),
                      ) ||
                      chat.messages.any(
                        (message) => ((message['parts'] as List?) ?? []).any(
                          (part) => (part['mime'] as String? ?? '').startsWith(
                            'image/',
                          ),
                        ),
                      )))
                TextButton(
                  key: const Key('image-model-settings'),
                  onPressed: chat.busy || chat.changing || chat.loading
                      ? null
                      : requestSettings,
                  child: const Text('Model settings'),
                ),
              if (chat.activeRecovery != null) ...[
                const SizedBox(height: 6),
                Text(
                  chat.activeRecovery!['guidance'] as String,
                  style: TextStyle(color: p.errorText, fontSize: 12),
                ),
                if (chat.activeRecovery!['kind'] != 'stopped')
                  Wrap(
                    spacing: 8,
                    children: [
                      if (chat.activeRecovery!['kind'] == 'desktop')
                        TextButton(
                          onPressed: chat.busy || chat.changing
                              ? null
                              : () => showSettings(
                                  context,
                                  chat,
                                  initial: SettingsCategory.desktop,
                                ),
                          child: const Text('Computer use'),
                        ),
                      if (chat.activeRecovery!['kind'] == 'instructions')
                        TextButton(
                          key: const Key('failure-instructions'),
                          onPressed: chat.busy || chat.changing || chat.loading
                              ? null
                              : () => showInstructions(context, chat),
                          child: const Text('Instructions'),
                        ),
                      if (chat.activeRecovery!['kind'] == 'memory')
                        TextButton(
                          key: const Key('failure-memory'),
                          onPressed: chat.busy || chat.changing || chat.loading
                              ? null
                              : () => showSettings(
                                  context,
                                  chat,
                                  initial: SettingsCategory.memory,
                                ),
                          child: const Text('Memory'),
                        ),
                      if (chat.activeRecovery!['retryable'] == true)
                        TextButton(
                          key: const Key('retry-message'),
                          onPressed:
                              chat.busy ||
                                  chat.changing ||
                                  chat.loading ||
                                  !chat.configured ||
                                  chat.draft.trim().isEmpty
                              ? null
                              : chat.send,
                          child: const Text('Retry message'),
                        ),
                      if ([
                        'timeout',
                        'outputLimit',
                        'generationSettings',
                        'contextLimit',
                      ].contains(chat.activeRecovery!['kind']))
                        TextButton(
                          key: const Key('failure-request-settings'),
                          onPressed: chat.busy || chat.changing
                              ? null
                              : requestSettings,
                          child: const Text('Model response settings'),
                        ),
                      if ([
                        'network',
                        'access',
                        'configuration',
                        'malformedTools',
                        'contextLimit',
                      ].contains(chat.activeRecovery!['kind']))
                        TextButton(
                          key: const Key('failure-connection'),
                          onPressed: chat.busy || chat.changing
                              ? null
                              : settings,
                          child: const Text('Model settings'),
                        ),
                    ],
                  ),
              ],
            ],
          ),
        ),
      Container(
        key: const Key('composer-frame'),
        decoration: BoxDecoration(
          color: p.surface,
          border: Border.all(color: p.border),
          borderRadius: BorderRadius.circular(14),
          boxShadow: [
            BoxShadow(
              color: Colors.black.withValues(alpha: p.dark ? 0.12 : 0.025),
              blurRadius: 24,
              offset: const Offset(0, 8),
            ),
          ],
        ),
        child: Padding(
          padding: const EdgeInsets.fromLTRB(16, 6, 8, 8),
          child: Column(
            mainAxisSize: MainAxisSize.min,
            children: [
              if (!chat.busy && chat.attachments.isNotEmpty)
                AttachmentChips(
                  chat: chat,
                  parts: chat.attachments,
                  removable: true,
                ),
              RichComposer(
                key: const Key('composer'),
                controller: input,
                focusNode: focus,
                onPasteImage: chat.attachmentsAvailable
                    ? chat.pasteImage
                    : null,
                readOnly: chat.busy || chat.changing || chat.loading,
                hint: chat.configured
                    ? 'Message Dolores…'
                    : 'Connect a model to begin…',
                onChanged: (value) {
                  chat.draft = value;
                  chat.invalidateContextPreview();
                  setState(() {});
                },
                onSend: () {
                  if (input.value.composing.isCollapsed) {
                    unawaited(chat.send());
                  }
                },
                trailing: const SizedBox.shrink(),
              ),
              Row(
                key: const Key('composer-actions'),
                mainAxisAlignment: MainAxisAlignment.end,
                children: [
                  if (chat.attachmentsAvailable)
                    SizedBox(
                      width: 36,
                      height: 36,
                      child: PopupMenuButton<String>(
                        key: const Key('attach-file'),
                        tooltip: 'Add to chat',
                        enabled: !chat.busy && !chat.changing && !chat.loading,
                        onSelected: (value) => value == 'window'
                            ? shareWindow(context, chat)
                            : chooseAttachment(chat),
                        itemBuilder: (_) => [
                          const PopupMenuItem(
                            value: 'file',
                            child: Text('Attach file or image'),
                          ),
                          if (chat.workspaceKind != 'side')
                            const PopupMenuItem(
                              value: 'window',
                              child: Text('Share window'),
                            ),
                        ],
                        icon: const Icon(Icons.add, size: 20),
                      ),
                    ),
                  if (chat.workspaceKind != 'side') AccessSelector(chat: chat),
                  if (chat.configured && chat.enabledModels.isNotEmpty)
                    Expanded(
                      child: Align(
                        alignment: Alignment.centerRight,
                        child: PopupMenuButton<String>(
                          key: const Key('chat-model-picker'),
                          tooltip: 'Choose model',
                          enabled:
                              !chat.busy && !chat.changing && !chat.loading,
                          initialValue: chat.model,
                          onSelected: chat.selectModel,
                          itemBuilder: (_) => [
                            for (final model in chat.enabledModels)
                              PopupMenuItem(
                                value: model,
                                child: SizedBox(
                                  width: 240,
                                  child: Text(
                                    model,
                                    maxLines: 2,
                                    overflow: TextOverflow.ellipsis,
                                  ),
                                ),
                              ),
                          ],
                          child: Padding(
                            padding: const EdgeInsets.symmetric(
                              vertical: 10,
                              horizontal: 4,
                            ),
                            child: Row(
                              mainAxisSize: MainAxisSize.min,
                              children: [
                                Icon(
                                  Icons.auto_awesome_outlined,
                                  size: 14,
                                  color: p.muted,
                                ),
                                const SizedBox(width: 6),
                                Flexible(
                                  child: Text(
                                    chat.model,
                                    maxLines: 1,
                                    overflow: TextOverflow.ellipsis,
                                    style: TextStyle(
                                      color: p.muted,
                                      fontSize: 12,
                                    ),
                                  ),
                                ),
                                Icon(
                                  Icons.expand_more,
                                  size: 16,
                                  color: p.muted,
                                ),
                              ],
                            ),
                          ),
                        ),
                      ),
                    )
                  else
                    const Spacer(),
                  ContextIndicator(
                    summary: chat.contextSummary,
                    basis: chat.contextBasis,
                    onPressed: chat.busy || chat.changing || chat.loading
                        ? null
                        : () async {
                            await showChatDetails(context, chat);
                          },
                  ),
                  const SizedBox(width: 4),
                  SizedBox(
                    width: 36,
                    height: 36,
                    child: IconButton.filled(
                      key: const Key('send'),
                      tooltip: chat.busy ? 'Stop response' : 'Send message',
                      onPressed: chat.busy
                          ? (chat.stopping ? null : chat.stop)
                          : (chat.loading ||
                                    chat.changing ||
                                    !chat.configured ||
                                    (chat.draft.trim().isEmpty &&
                                        chat.attachments.isEmpty) ||
                                    !input.value.composing.isCollapsed
                                ? null
                                : chat.send),
                      style: IconButton.styleFrom(
                        backgroundColor: p.accent,
                        foregroundColor: p.bg,
                        disabledBackgroundColor: p.soft,
                        shape: RoundedRectangleBorder(
                          borderRadius: BorderRadius.circular(9),
                        ),
                      ),
                      iconSize: 18,
                      icon: Icon(
                        chat.busy
                            ? Icons.stop_rounded
                            : Icons.arrow_upward_rounded,
                      ),
                    ),
                  ),
                ],
              ),
            ],
          ),
        ),
      ),
    ],
  );
  @override
  Widget build(BuildContext context) {
    final p = Palette(Theme.of(context).brightness == Brightness.dark);
    return LayoutBuilder(
      builder: (_, constraints) {
        final narrow = constraints.maxWidth < UiTokens.drawerBreakpoint;
        final maxSidebarWidth =
            (constraints.maxWidth - UiTokens.conversationMinWidth - 1).clamp(
              UiTokens.sidebarMinWidth,
              UiTokens.sidebarMaxWidth,
            );
        final effectiveSidebarWidth = sidebarWidth.clamp(
          UiTokens.sidebarMinWidth,
          maxSidebarWidth,
        );
        final body = Column(
          children: [
            Container(
              height: 68,
              decoration: BoxDecoration(
                border: Border(bottom: BorderSide(color: p.border)),
              ),
              padding: EdgeInsets.symmetric(horizontal: narrow ? 20 : 32),
              child: Row(
                children: [
                  if (narrow)
                    Builder(
                      builder: (context) => IconButton(
                        tooltip: 'Conversations',
                        onPressed: () => Scaffold.of(context).openDrawer(),
                        icon: const Icon(Icons.menu, size: 20),
                      ),
                    ),
                  Expanded(
                    child: Align(
                      alignment: Alignment.centerLeft,
                      child: WorkspacePicker(
                        chat: chat,
                        onOpenProject: openProject,
                      ),
                    ),
                  ),
                  if (chat.session != null && chat.workspaceRoot != null)
                    IconButton(
                      key: const Key('workspace-changes'),
                      tooltip: 'Changes',
                      onPressed: chat.busy || chat.loading || chat.changing
                          ? null
                          : () => showChatDetails(
                              context,
                              chat,
                              initial: ChatDetail.changes,
                            ),
                      icon: const Icon(Icons.difference_outlined, size: 20),
                    ),
                  IconButton(
                    key: const Key('chat-trajectory'),
                    tooltip: 'Activity',
                    onPressed: chat.loading || chat.changing
                        ? null
                        : () => showChatDetails(
                            context,
                            chat,
                            initial: ChatDetail.activity,
                          ),
                    icon: const Icon(Icons.timeline, size: 20),
                  ),
                  if (chat.session != null)
                    PopupMenuButton<String>(
                      key: const Key('export-chat'),
                      tooltip: 'Chat actions',
                      enabled: !chat.changing && !chat.loading,
                      onSelected: (value) {
                        if (value == 'fork') {
                          showThreadFork(context, chat);
                        } else if (value == 'export') {
                          chooseExport();
                        } else {
                          showChatDetails(context, chat);
                        }
                      },
                      icon: const Icon(Icons.more_horiz, size: 20),
                      itemBuilder: (_) => [
                        PopupMenuItem(
                          value: 'fork',
                          enabled: !chat.busy,
                          child: const Text('Branch chat'),
                        ),
                        PopupMenuItem(
                          value: 'export',
                          enabled: !chat.busy,
                          child: const Text('Export…'),
                        ),
                        PopupMenuItem(
                          value: 'details',
                          enabled: !chat.busy,
                          child: const Text('Chat details'),
                        ),
                      ],
                    ),
                ],
              ),
            ),
            Align(
              alignment: Alignment.center,
              child: ConstrainedBox(
                constraints: const BoxConstraints(
                  maxWidth: UiTokens.contentWidth,
                ),
                child: Padding(
                  padding: EdgeInsets.symmetric(horizontal: narrow ? 24 : 40),
                  child: Column(
                    mainAxisSize: MainAxisSize.min,
                    children: [
                      if (chat.messagesOlder || chat.messagesNewer)
                        Padding(
                          padding: const EdgeInsets.only(bottom: 20),
                          child: Wrap(
                            spacing: 8,
                            crossAxisAlignment: WrapCrossAlignment.center,
                            children: [
                              Text(
                                chat.messagesNewer
                                    ? 'Earlier messages'
                                    : 'Latest messages',
                                style: TextStyle(color: p.muted, fontSize: 12),
                              ),
                              TextButton(
                                key: const Key('older-messages'),
                                onPressed:
                                    !chat.messagesOlder ||
                                        chat.busy ||
                                        chat.changing
                                    ? null
                                    : () => chat.browseMessages(newer: false),
                                child: const Text('Older'),
                              ),
                              TextButton(
                                key: const Key('newer-messages'),
                                onPressed:
                                    !chat.messagesNewer ||
                                        chat.busy ||
                                        chat.changing
                                    ? null
                                    : () => chat.browseMessages(newer: true),
                                child: const Text('Newer'),
                              ),
                              if (chat.messagesNewer)
                                TextButton(
                                  key: const Key('latest-messages'),
                                  onPressed: chat.busy || chat.changing
                                      ? null
                                      : () => chat.browseMessages(
                                          newer: false,
                                          latest: true,
                                        ),
                                  child: const Text('Latest'),
                                ),
                            ],
                          ),
                        ),
                    ],
                  ),
                ),
              ),
            ),
            Expanded(
              child: chat.loading
                  ? const Center(child: CircularProgressIndicator())
                  : (chat.messages.isEmpty && !chat.busy
                        ? welcome(p)
                        : Align(
                            alignment: Alignment.topCenter,
                            child: ConstrainedBox(
                              constraints: const BoxConstraints(
                                maxWidth: UiTokens.contentWidth,
                              ),
                              child: ListView(
                                controller: scroll,
                                padding: EdgeInsets.fromLTRB(
                                  narrow ? 24 : 40,
                                  36,
                                  narrow ? 24 : 40,
                                  0,
                                ),
                                children: [
                                  for (final (index, item)
                                      in chat.messages.indexed)
                                    message(
                                      p,
                                      item['role'] as String,
                                      item['content'] as String,
                                      metadata: (item['metadata'] as Map?)
                                          ?.cast<String, dynamic>(),
                                      parts: ((item['parts'] as List?) ?? [])
                                          .cast<Map<String, dynamic>>(),
                                      messageId: item['id'] as int?,
                                      savedAt: item['savedAt'] as int?,
                                      feedback: (item['feedback'] as Map?)
                                          ?.cast<String, dynamic>(),
                                      key: ValueKey(
                                        '${chat.session}:${item['id'] ?? index}',
                                      ),
                                    ),
                                  if (chat.busy) ...[
                                    message(
                                      p,
                                      'user',
                                      chat.pendingInput,
                                      parts: chat.pendingParts,
                                      key: const ValueKey('pending-user'),
                                    ),
                                    message(
                                      p,
                                      'assistant',
                                      chat.partial,
                                      streaming: true,
                                      key: const ValueKey('pending-assistant'),
                                    ),
                                  ],
                                ],
                              ),
                            ),
                          )),
            ),
            Align(
              alignment: Alignment.bottomCenter,
              child: ConstrainedBox(
                constraints: const BoxConstraints(
                  maxWidth: UiTokens.contentWidth,
                ),
                child: Padding(
                  padding: EdgeInsets.fromLTRB(
                    narrow ? 24 : 40,
                    12,
                    narrow ? 24 : 40,
                    24,
                  ),
                  child: composer(p),
                ),
              ),
            ),
          ],
        );
        return Scaffold(
          key: shell,
          drawer: narrow
              ? Drawer(width: UiTokens.sidebarWidth, child: sidebar(p))
              : null,
          body: Stack(
            fit: StackFit.expand,
            children: [
              Row(
                children: [
                  if (!narrow) ...[
                    sidebar(p, width: effectiveSidebarWidth),
                    Container(width: 1, color: p.border),
                  ],
                  Expanded(child: body),
                ],
              ),
              if (!narrow)
                Positioned(
                  left: effectiveSidebarWidth - 4,
                  top: 0,
                  bottom: 0,
                  width: 9,
                  child: SidebarResizeHandle(
                    width: effectiveSidebarWidth,
                    maxWidth: maxSidebarWidth,
                    onDelta: (delta) => setState(() {
                      sidebarWidth = (effectiveSidebarWidth + delta).clamp(
                        UiTokens.sidebarMinWidth,
                        maxSidebarWidth,
                      );
                    }),
                    onReset: () => setState(() {
                      sidebarWidth = UiTokens.sidebarWidth;
                    }),
                  ),
                ),
            ],
          ),
        );
      },
    );
  }
}
