import 'close_behavior.dart';
import 'notification_toast.dart';
import 'attention_panel.dart';
import 'access_selector.dart';
import 'memory.dart';
import 'chat_details.dart';

import 'dart:async';
import 'dart:convert';

import 'package:flutter/material.dart';
import 'package:flutter/rendering.dart' show ScrollDirection;
import 'package:flutter/services.dart';
import 'package:file_selector/file_selector.dart';

import 'bridge.dart';
import 'chat.dart';
import 'history_search.dart';
import 'composer_controller.dart';
import 'reply_content.dart';
import 'message_frame.dart';
import 'rich_composer.dart';
import 'composer_commands.dart';
import 'instruction_queue.dart';
import 'goal_panel.dart';
import 'planning_panel.dart';
import 'usage_details.dart';
import 'task_feedback.dart';
import 'inspector.dart';
import 'tool_activity.dart';
import 'user_question.dart';
import 'subagents.dart';
import 'workspace_picker.dart';
import 'worktrees.dart';
import 'instructions.dart';
import 'chat_sidebar.dart';
import 'agent_timeline.dart';
import 'thread_fork.dart';
import 'side_conversations.dart';
import 'prompt_branches.dart';
import 'attachments.dart';

import 'theme.dart';
import 'home_landing.dart';
import 'settings.dart';
import 'desktop_frame.dart';
import 'desktop_share.dart';
import 'sidebar_resize.dart';
import 'app_host.dart';
import 'workspace_shell.dart';

import 'package:window_manager/window_manager.dart';

export 'model_settings.dart' show ConnectionDialog;
export 'theme.dart' show Palette;

Future<void> main() async {
  WidgetsFlutterBinding.ensureInitialized();
  final desktopFrame = await initializeDesktopFrame();
  final chat = ChatController(NativeBridge());
  final host = AppHost(chat, binding: WidgetsBinding.instance);
  if (desktopFrame) host.destroyWindow = windowManager.destroy;
  runApp(DoloresApp(chat: chat, host: host, desktopFrame: desktopFrame));
  unawaited(chat.initialize());
}

class DoloresApp extends StatelessWidget {
  final ChatController chat;
  final ThemeMode? themeMode;
  final GlobalKey? captureKey;
  final bool desktopFrame;
  final AppHost? host;
  const DoloresApp({
    super.key,
    required this.chat,
    this.themeMode,
    this.captureKey,
    this.desktopFrame = false,
    this.host,
  });
  @override
  Widget build(BuildContext context) => AnimatedBuilder(
    animation: Listenable.merge([
      host?.appearanceChanges ?? chat.appearanceChanges,
      chat.reducedMotion,
    ]),
    builder: (context, _) => MaterialApp(
      navigatorKey: GlobalObjectKey<NavigatorState>(host ?? chat),
      title: 'Dolores',
      debugShowCheckedModeBanner: false,
      theme: doloresTheme(false),
      darkTheme: doloresTheme(true),
      themeMode:
          themeMode ??
          switch ((host?.visible ?? chat).appearance) {
            'light' => ThemeMode.light,
            'dark' => ThemeMode.dark,
            _ => ThemeMode.system,
          },
      builder: (context, child) => CallbackShortcuts(
        bindings: {
          const SingleActivator(LogicalKeyboardKey.keyK, control: true): () {
            final navigator = GlobalObjectKey<NavigatorState>(host ?? chat)
                .currentState;
            final owner = host?.visible ?? chat;
            if (navigator != null &&
                !navigator.canPop() &&
                owner.historySearchAvailable) {
              final overlay = navigator.overlay?.context;
              if (overlay != null) unawaited(showHistorySearch(overlay, owner));
            }
          },
        },
        child: MediaQuery(
          data: MediaQuery.of(context).copyWith(
            disableAnimations:
                MediaQuery.of(context).disableAnimations ||
                (host?.visible ?? chat).reducedMotion.value,
          ),
          child: RepaintBoundary(
            key: captureKey,
            child: NotificationOverlay(
              host: host?.attention,
              top: desktopFrame ? 42 : 10,
              child: desktopFrame
                  ? DesktopFrame(
                      search: AnimatedBuilder(
                        animation: host ?? chat,
                        builder: (_, _) =>
                            (host?.visible ?? chat).historySearchAvailable
                            ? HistorySearchButton(
                                chat: host?.visible ?? chat,
                                onPressed: () {
                                  final navigator =
                                      GlobalObjectKey<NavigatorState>(
                                        host ?? chat,
                                      ).currentState;
                                  if (navigator == null || navigator.canPop()) {
                                    return;
                                  }
                                  final searchContext =
                                      navigator.overlay?.context;
                                  if (searchContext != null) {
                                    unawaited(
                                      showHistorySearch(
                                        searchContext,
                                        host?.visible ?? chat,
                                      ),
                                    );
                                  }
                                },
                              )
                            : const SizedBox.shrink(),
                      ),
                      attention: host == null
                          ? null
                          : AnimatedBuilder(
                              animation: host!,
                              builder: (_, _) =>
                                  host!.visible.attentionAvailable
                                  ? AttentionButton(host: host!.attention)
                                  : const SizedBox.shrink(),
                            ),
                      onTogglePanel: host?.togglePanel,
                      onRequestClose: host == null
                          ? null
                          : () async {
                              final navigator = GlobalObjectKey<NavigatorState>(
                                host ?? chat,
                              ).currentState;
                              final context = navigator?.overlay?.context;
                              if (context != null) {
                                await closeDolores(context, host!);
                              }
                            },
                      onOpenSettings: () async {
                        final navigator = GlobalObjectKey<NavigatorState>(
                          host ?? chat,
                        ).currentState;
                        // Keep the current modal/review and its unsaved work in place.
                        if (navigator == null || navigator.canPop()) return;
                        final settingsContext = navigator.overlay?.context;
                        if (settingsContext != null) {
                          await showSettings(
                            settingsContext,
                            host?.visible ?? chat,
                          );
                        }
                      },
                      child: child!,
                    )
                  : child!,
            ),
          ),
        ),
      ),
      home: host == null
          ? ChatPage(chat: chat)
          : WorkspaceShell(host: host!, nativeTitleBar: desktopFrame),
    ),
  );
}

class ChatPage extends StatefulWidget {
  final ChatController chat;
  final bool embedded;
  final Future<void> Function(String)? onSelectChat, onSelectProject;
  final Future<void> Function()? onOpenProject;
  const ChatPage({
    super.key,
    required this.chat,
    this.embedded = false,
    this.onSelectChat,
    this.onSelectProject,
    this.onOpenProject,
  });
  @override
  State<ChatPage> createState() => _ChatPageState();
}

class _ChatPageState extends State<ChatPage> {
  final composerKey = GlobalKey();
  final modelSettingsAction = Object();
  final input = ComposerController();
  final commands = GlobalKey<ComposerCommandsState>();
  final focus = FocusNode();
  final scroll = ScrollController();
  final shell = GlobalKey<ScaffoldState>();
  ChatController get chat => widget.chat;
  bool following = true;
  bool _positioning = false;
  double sidebarWidth = UiTokens.sidebarWidth;
  int seenRevision = -1;
  @override
  void initState() {
    super.initState();
    chat.addListener(_changed);
    input.text = chat.draft;
    scroll.addListener(() {
      if (!scroll.hasClients) return;
      // Layout corrections after a jump are not a request to stop following.
      if (!_positioning) {
        if (scroll.position.extentAfter <= 1) {
          following = true;
        } else if (scroll.position.extentAfter >= 80) {
          following = false;
        }
      }
      chat.rememberScroll(scroll.offset);
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
          _jumpTo(offset.clamp(0, scroll.position.maxScrollExtent));
          following = offset.isInfinite || scroll.position.extentAfter < 80;
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
          _jumpTo(scroll.position.maxScrollExtent);
        }
      });
    }
  }

  void _jumpTo(double offset) {
    _positioning = true;
    try {
      scroll.jumpTo(offset);
    } finally {
      _positioning = false;
    }
  }

  Widget _historyViewport({
    required EdgeInsets padding,
    required List<Widget> children,
  }) => NotificationListener<ScrollMetricsNotification>(
    onNotification: (notification) {
      if (notification.depth == 0) _followReply();
      return false;
    },
    child: NotificationListener<UserScrollNotification>(
      onNotification: (notification) {
        if (notification.depth == 0) {
          if (notification.direction == ScrollDirection.forward) {
            following = false;
          } else if (notification.direction == ScrollDirection.idle &&
              scroll.hasClients &&
              scroll.position.extentAfter <= 1) {
            following = true;
          }
        }
        return false;
      },
      child: ListView(controller: scroll, padding: padding, children: children),
    ),
  );

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
        showBrand:
            context.findAncestorWidgetOfExactType<DesktopFrame>() == null,
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

  Widget _thinking(List steps) {
    if (steps.isEmpty) return const SizedBox.shrink();
    final p = Palette(Theme.of(context).brightness == Brightness.dark);
    return Theme(
      data: Theme.of(context).copyWith(dividerColor: Colors.transparent),
      child: ExpansionTile(
        key: ValueKey('thinking-${steps.first['number']}'),
        tilePadding: EdgeInsets.zero,
        childrenPadding: const EdgeInsets.only(bottom: 12),
        title: Text('Thinking', style: TextStyle(color: p.muted, fontSize: 12)),
        children: [
          ConstrainedBox(
            constraints: const BoxConstraints(maxHeight: 200),
            child: SingleChildScrollView(
              child: Column(
                crossAxisAlignment: CrossAxisAlignment.start,
                children: [
                  for (final step in steps)
                    SelectableText(
                      step['text'] as String,
                      style: TextStyle(
                        color: p.muted,
                        fontSize: 12,
                        height: 1.5,
                      ),
                    ),
                  if (steps.any((s) => (s['text'] as String).length >= 16000))
                    Text(
                      'Thinking preview shortened.',
                      style: TextStyle(color: p.muted, fontSize: 11),
                    ),
                ],
              ),
            ),
          ),
        ],
      ),
    );
  }

  List<Map<String, dynamic>> _savedMessageSkills(int index) {
    if (chat.messages[index]['role'] != 'user' ||
        index + 1 >= chat.messages.length) {
      return const [];
    }
    final reply = chat.messages[index + 1];
    if (reply['role'] != 'assistant') return const [];
    return ((reply['metadata']?['context']?['skills'] as List?) ?? [])
        .cast<Map<String, dynamic>>();
  }

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
    List<Map<String, dynamic>> skills = const [],
  }) {
    final user = role == 'user';
    final callUsage = metadata?['agent']?['usageByCall'] as List?;
    final reasoningTokens =
        metadata?['usage']?['reasoningTokens'] ??
        callUsage?.lastOrNull?['reasoningTokens'];
    final savedTools = metadata?['agent']?['tools'] as List? ?? [];
    final pausedReceipts = metadata?['paused']?['receipts'] as List? ?? [];
    final savedReceiptKeys = pausedReceipts.isEmpty
        ? const <String>{}
        : savedTools.map((tool) => jsonEncode(tool)).toSet();
    final inheritedTools = pausedReceipts
        .where(
          // Providers can reuse call IDs in later runs. Deduplicate only the
          // full receipt, so a repair never hides earlier failure evidence.
          (receipt) => !savedReceiptKeys.contains(jsonEncode(receipt)),
        )
        .toList();
    return MessageFrame(
      key: key,
      user: user,
      text: text,
      streaming: streaming,
      savedAt: savedAt,
      onEditBranch:
          user &&
              !streaming &&
              messageId != null &&
              chat.promptBranchesAvailable
          ? () => showPromptBranch(context, chat, messageId)
          : null,
      onDiscuss:
          !streaming && messageId != null && chat.sideConversationsAvailable
          ? () => showSideConversation(context, chat, messageId, text)
          : null,
      onBringBack:
          !streaming &&
              !user &&
              messageId != null &&
              chat.sideConversation?['parentAvailable'] == true
          ? () => showSideConversation(
              context,
              chat,
              messageId,
              text,
              bringBack: true,
            )
          : null,
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
          if (user && skills.isNotEmpty)
            Padding(
              padding: const EdgeInsets.only(bottom: 8),
              child: Text(
                'Skills: ${skills.map((s) => s['name']).join(', ')}',
                key: const Key('message-skills'),
                style: TextStyle(fontSize: 12, color: p.muted),
              ),
            ),
          if (streaming)
            Tooltip(
              message: chat.modelTimeoutSeconds > 0
                  ? 'Stall timeout: ${chat.modelTimeoutSeconds}s without new model data. Review has no timeout. Stop remains available.'
                  : '',
              child: Text(
                chat.workspaceRoot == null
                    ? 'Writing…'
                    : chat.modelActivityLabel,
                style: TextStyle(fontSize: 12, color: p.muted),
              ),
            ),
          if (!user && streaming && chat.modelThinking.isNotEmpty)
            _thinking(chat.modelThinking),
          if (!user && !streaming && metadata?['agent']?['thinking'] is List)
            _thinking(metadata!['agent']['thinking'] as List),
          if (!user && !streaming && inheritedTools.isNotEmpty) ...[
            Text(
              'Earlier tool results',
              style: TextStyle(fontSize: 12, color: p.muted),
            ),
            AgentTimeline(records: inheritedTools, bridge: chat.bridge),
          ],
          if (!user)
            AgentTimeline(
              steps: streaming
                  ? [
                      ...chat.modelTexts,
                      if (text.isNotEmpty)
                        {'number': chat.modelStep, 'text': text},
                    ]
                  : metadata?['agent']?['steps'] as List? ?? [],
              records: streaming ? chat.toolRecords : savedTools,
              toolSteps: streaming
                  ? [for (final record in chat.toolRecords) record['modelStep']]
                  : metadata?['agent']?['toolSteps'] as List? ?? [],
              bridge: chat.bridge,
              unsaved: streaming,
              streamingStep: streaming ? chat.modelStep : null,
              onRendered: _followReply,
            ),
          if (parts.isNotEmpty) AttachmentChips(chat: chat, parts: parts),
          if (text.isEmpty && !streaming)
            SelectableText(
              text,
              style: TextStyle(fontSize: 14, height: 1.65, color: p.text),
            )
          else if (text.isNotEmpty && (!streaming || user))
            ReplyContent(
              text: text,
              streaming: streaming,
              onRendered: _followReply,
              onAskAbout:
                  !streaming &&
                      messageId != null &&
                      chat.sideConversationsAvailable
                  ? (selected) =>
                        showSideConversation(context, chat, messageId, selected)
                  : null,
              onBringBack:
                  !streaming &&
                      !user &&
                      messageId != null &&
                      chat.sideConversation?['parentAvailable'] == true
                  ? (selected) => showSideConversation(
                      context,
                      chat,
                      messageId,
                      selected,
                      bringBack: true,
                    )
                  : null,
            ),
          if (!user && !streaming) ...[
            if (metadata?['paused'] is Map) ...[
              const SizedBox(height: 12),
              Text(
                metadata!['paused']['reason'] == 'outputLimit'
                    ? 'Paused at ${metadata['requestSettings']?['maxOutputTokens'] == null ? 'the provider’s response limit' : 'the output limit (${metadata['requestSettings']['maxOutputTokens']} tokens)'}. Progress saved.${reasoningTokens != null ? ' Reasoning used $reasoningTokens tokens.' : ''}'
                    : metadata['paused']['reason'] == 'commandReview'
                    ? 'A command failed or verification was incomplete. Review its output before repair.'
                    : metadata['paused']['reason'] == 'desktopAccess'
                    ? 'Choose a window to continue. Nothing has been shared yet.'
                    : metadata['paused']['reason'] == 'desktopReview'
                    ? 'Computer use paused. Inspect the window and fresh screenshot before reconciling saved progress.'
                    : metadata['paused']['reason'] == 'subagentReview'
                    ? 'A subagent needs review. Inspect its report, Run history and Changes before continuing.'
                    : metadata['paused']['reason'] == 'responseTimeout'
                    ? 'Model connection stalled. Progress saved. Continue when ready.'
                    : metadata['paused']['reason'] == 'taskTimeout'
                    ? 'Paused at your task time limit. Progress saved. Inspect Changes before continuing.'
                    : metadata['paused']['reason'] == 'noProgress'
                    ? 'Paused after repeated tool failures. Progress saved.'
                    : metadata['paused']['reason'] == 'checkpoint'
                    ? 'Task checkpoint saved. Review progress before continuing.'
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
                        (r) => r['name'] == 'computer',
                      )
                    ? 'Continue starts another bounded run with automatic computer actions. Inspect saved results first; capture fresh state before repeating input.'
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
                      'noProgress',
                      'checkpoint',
                    ].contains(metadata['paused']['reason']))
                      TextButton(
                        onPressed: chat.busy || chat.changing
                            ? null
                            : () =>
                                  [
                                    'outputLimit',
                                    'responseTimeout',
                                  ].contains(metadata['paused']['reason'])
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

  Widget welcome(Palette p) => HomeLanding(
    composer: composer(p),
    title: chat.workspaceKind == 'side'
        ? 'A little space to think.'
        : 'Where shall we begin?',
    subtitle: chat.workspaceKind == 'project'
        ? 'Work with files in ${chat.workspaceLabel}.'
        : chat.workspaceKind == 'side'
        ? 'A conversation without file access.'
        : 'Bring a thought. We’ll take it from here.',
    projects: chat.projects,
    sessions: chat.sessions,
    openProject: chat.changing || chat.loading
        ? null
        : widget.onOpenProject ?? openProject,
    connectModel: !chat.configured && !chat.loading ? settings : null,
    selectProject: widget.onSelectProject ?? (root) => chat.openProject(root),
    selectChat: widget.onSelectChat ?? chat.select,
    canSuggest: chat.draft.isEmpty && !chat.changing && !chat.loading,
    suggest: (value) {
      input.text = value;
      chat.draft = value;
      focus.requestFocus();
      setState(() {});
    },
  );
  Widget composer(Palette p) => Column(
    key: composerKey,
    mainAxisSize: MainAxisSize.min,
    children: [
      if (!chat.busy &&
          (chat.modelTexts.isNotEmpty || chat.toolRecords.isNotEmpty))
        ConstrainedBox(
          constraints: const BoxConstraints(maxHeight: 160),
          child: SingleChildScrollView(
            child: AgentTimeline(
              steps: chat.modelTexts,
              records: chat.toolRecords,
              toolSteps: [
                for (final record in chat.toolRecords) record['modelStep'],
              ],
              bridge: chat.bridge,
              unsaved: true,
            ),
          ),
        ),
      if (chat.subagents.isNotEmpty)
        ConstrainedBox(
          constraints: const BoxConstraints(maxHeight: 200),
          child: SingleChildScrollView(
            key: const PageStorageKey('active-child-supervision'),
            child: SubagentCards(
              children: chat.subagents,
              onSupervise: chat.busy && !chat.stopping
                  ? chat.superviseChild
                  : null,
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
      if (chat.userQuestion != null)
        UserQuestionCard(
          key: ValueKey(chat.userQuestion!['callId']),
          chat: chat,
        ),
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
          borderRadius: BorderRadius.circular(20),
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
              if (chat.instructionQueueAvailable) InstructionQueue(chat: chat),
              if (chat.goalsAvailable) GoalPanel(chat: chat),
              if (chat.planningAvailable) PlanningPanel(chat: chat),
              if (chat.commandPickerAvailable)
                ComposerCommands(
                  key: commands,
                  chat: chat,
                  input: input,
                  focus: focus,
                ),
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
                onCommandKey: (event) =>
                    commands.currentState?.handleKey(event) ??
                    KeyEventResult.ignored,
                onPasteImage: chat.attachmentsAvailable
                    ? chat.pasteImage
                    : null,
                readOnly:
                    (chat.busy && !chat.instructionQueueAvailable) ||
                    chat.changing ||
                    chat.loading,
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
                    unawaited(
                      chat.busy && chat.instructionQueueAvailable
                          ? chat.queueDraft()
                          : chat.send(),
                    );
                  }
                },
                trailing: const SizedBox.shrink(),
              ),
              if (chat.acceptsLiveInputs && chat.busy)
                Align(
                  alignment: Alignment.centerRight,
                  child: Wrap(
                    spacing: 8,
                    children: [
                      TextButton(
                        key: const Key('queue-instruction'),
                        onPressed:
                            chat.inputSaving ||
                                chat.skillSaving ||
                                chat.stopping ||
                                chat.draft.trim().isEmpty ||
                                !input.value.composing.isCollapsed
                            ? null
                            : () => chat.queueDraft(),
                        child: Text(
                          'Queue',
                          style: TextStyle(
                            fontFamily: Theme.of(context)
                                .textTheme
                                .bodyMedium
                                ?.fontFamily,
                          ),
                        ),
                      ),
                      TextButton(
                        key: const Key('steer-instruction'),
                        onPressed:
                            chat.inputSaving ||
                                chat.skillSaving ||
                                chat.stopping ||
                                chat.draft.trim().isEmpty ||
                                !input.value.composing.isCollapsed
                            ? null
                            : () => chat.queueDraft(steer: true),
                        child: Text(
                          'Steer now',
                          style: TextStyle(
                            fontFamily: Theme.of(context)
                                .textTheme
                                .bodyMedium
                                ?.fontFamily,
                          ),
                        ),
                      ),
                    ],
                  ),
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
                        popUpAnimationStyle: surfaceAnimation(context),
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
                  if (chat.configured && chat.enabledModels.isNotEmpty)
                    Expanded(
                      child: Align(
                        alignment: Alignment.centerLeft,
                        child: PopupMenuButton<Object>(
                          key: const Key('chat-model-picker'),
                          popUpAnimationStyle: surfaceAnimation(context),
                          tooltip: 'Choose model',
                          enabled:
                              !chat.busy && !chat.changing && !chat.loading,
                          initialValue: chat.model,
                          onSelected: (value) =>
                              identical(value, modelSettingsAction)
                              ? requestSettings()
                              : chat.selectModel(value as String),
                          itemBuilder: (_) => [
                            for (final model in chat.enabledModels)
                              PopupMenuItem(
                                value: model,
                                child: SizedBox(
                                  width: 240,
                                  child: Row(
                                    children: [
                                      Expanded(
                                        child: Text(
                                          model,
                                          maxLines: 2,
                                          overflow: TextOverflow.ellipsis,
                                        ),
                                      ),
                                      if (model == chat.model)
                                        const Icon(Icons.check, size: 16),
                                    ],
                                  ),
                                ),
                              ),
                            const PopupMenuDivider(),
                            PopupMenuItem(
                              value: modelSettingsAction,
                              child: const Text('Model settings'),
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
                  if (chat.workspaceKind != 'side') AccessSelector(chat: chat),
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
                                    chat.skillSaving ||
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
                          borderRadius: BorderRadius.circular(20),
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
                  if (narrow && !widget.embedded)
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
                      child: widget.embedded
                          ? Text(chat.workspaceLabel)
                          : WorkspacePicker(
                              chat: chat,
                              onOpenProject: openProject,
                              onSelectChat: widget.onSelectChat,
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
                        } else if (value == 'worktrees') {
                          showWorktrees(
                            context,
                            chat,
                            onSelect: widget.onSelectChat,
                          );
                        } else if (value == 'export') {
                          chooseExport();
                        } else {
                          showChatDetails(context, chat);
                        }
                      },
                      icon: const Icon(Icons.more_horiz, size: 20),
                      itemBuilder: (_) => [
                        if (chat.worktreesAvailable)
                          PopupMenuItem(
                            value: 'worktrees',
                            enabled: !chat.busy,
                            child: const Text('Worktrees…'),
                          ),
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
                      if (chat.sideConversation != null ||
                          chat.sideConversationError != null)
                        SideConversationBanner(chat: chat),
                      if (chat.promptBranch != null ||
                          chat.promptBranchError != null)
                        PromptBranchBanner(chat: chat),
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
                              child: _historyViewport(
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
                                      skills: _savedMessageSkills(index),
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
                                      skills: chat.pendingSkills,
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
            if (chat.messages.isNotEmpty || chat.busy || chat.loading)
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
        if (widget.embedded) return Scaffold(body: body);
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
