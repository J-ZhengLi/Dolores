import 'dart:async';
import 'dart:convert';

import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:file_selector/file_selector.dart';

import 'bridge.dart';
import 'chat.dart';
import 'composer_controller.dart';
import 'reply_content.dart';
import 'rich_composer.dart';
import 'theme.dart';
export 'theme.dart' show Palette;

void main() {
  WidgetsFlutterBinding.ensureInitialized();
  final chat = ChatController(NativeBridge());
  runApp(DoloresApp(chat: chat));
  unawaited(chat.initialize());
}

class DoloresApp extends StatelessWidget {
  final ChatController chat;
  final ThemeMode themeMode;
  final GlobalKey? captureKey;
  const DoloresApp({
    super.key,
    required this.chat,
    this.themeMode = ThemeMode.system,
    this.captureKey,
  });
  @override
  Widget build(BuildContext context) => MaterialApp(
    title: 'Dolores',
    debugShowCheckedModeBanner: false,
    theme: doloresTheme(false),
    darkTheme: doloresTheme(true),
    themeMode: themeMode,
    builder: (context, child) =>
        RepaintBoundary(key: captureKey, child: child!),
    home: ChatPage(chat: chat),
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
  ChatController get chat => widget.chat;
  bool following = true;
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
    await showDialog<void>(
      context: context,
      builder: (_) => ConnectionDialog(chat: chat),
    );
    if (mounted) focus.requestFocus();
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

  Widget sidebar(Palette p) => Container(
    width: UiTokens.sidebarWidth,
    color: p.sidebar,
    child: SafeArea(
      child: Padding(
        padding: const EdgeInsets.fromLTRB(18, 28, 18, 18),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.stretch,
          children: [
            Padding(
              padding: const EdgeInsets.fromLTRB(10, 0, 0, 32),
              child: Row(
                children: [
                  Icon(Icons.all_inclusive_rounded, size: 27, color: p.accent),
                  const SizedBox(width: 12),
                  Expanded(
                    child: Text(
                      'Dolores',
                      maxLines: 1,
                      overflow: TextOverflow.ellipsis,
                      style: TextStyle(
                        fontFamily: 'Georgia',
                        fontSize: 26,
                        letterSpacing: -0.8,
                        color: p.text,
                      ),
                    ),
                  ),
                ],
              ),
            ),
            OutlinedButton.icon(
              key: const Key('new-chat'),
              onPressed: chat.busy || chat.changing || chat.loading
                  ? null
                  : () {
                      chat.newChat();
                      focus.requestFocus();
                    },
              icon: const Icon(Icons.add, size: 18),
              label: const Text('New conversation'),
              style: OutlinedButton.styleFrom(
                foregroundColor: p.text,
                side: BorderSide(color: p.border),
                padding: const EdgeInsets.symmetric(vertical: 15),
                shape: RoundedRectangleBorder(
                  borderRadius: BorderRadius.circular(9),
                ),
              ),
            ),
            Padding(
              padding: const EdgeInsets.fromLTRB(10, 28, 0, 12),
              child: Text(
                'CONVERSATIONS',
                style: TextStyle(
                  color: p.muted,
                  fontSize: 11,
                  letterSpacing: 1.2,
                ),
              ),
            ),
            Expanded(
              child: ListView.builder(
                itemCount: chat.sessions.length,
                itemBuilder: (_, index) {
                  final item = chat.sessions[index], id = item['id'] as String;
                  return Padding(
                    padding: const EdgeInsets.only(bottom: 4),
                    child: Material(
                      color: chat.session == id ? p.soft : Colors.transparent,
                      borderRadius: BorderRadius.circular(8),
                      child: InkWell(
                        borderRadius: BorderRadius.circular(8),
                        onTap: chat.busy || chat.changing
                            ? null
                            : () => chat.select(id),
                        child: Padding(
                          padding: const EdgeInsets.only(left: 12),
                          child: Row(
                            children: [
                              Icon(
                                Icons.chat_bubble_outline_rounded,
                                size: 16,
                                color: p.muted,
                              ),
                              const SizedBox(width: 10),
                              Expanded(
                                child: Text(
                                  item['title'] as String,
                                  maxLines: 1,
                                  overflow: TextOverflow.ellipsis,
                                  style: TextStyle(fontSize: 13, color: p.text),
                                ),
                              ),
                              IconButton(
                                tooltip: 'Delete conversation',
                                iconSize: 16,
                                onPressed: chat.busy || chat.changing
                                    ? null
                                    : () async {
                                        final confirmed =
                                            await showDialog<bool>(
                                              context: context,
                                              builder: (_) => AlertDialog(
                                                title: const Text(
                                                  'Delete conversation?',
                                                ),
                                                content: const Text(
                                                  'This removes its saved messages.',
                                                ),
                                                actions: [
                                                  TextButton(
                                                    onPressed: () =>
                                                        Navigator.pop(
                                                          context,
                                                          false,
                                                        ),
                                                    child: const Text('Keep'),
                                                  ),
                                                  TextButton(
                                                    onPressed: () =>
                                                        Navigator.pop(
                                                          context,
                                                          true,
                                                        ),
                                                    child: const Text('Delete'),
                                                  ),
                                                ],
                                              ),
                                            );
                                        if (confirmed == true) {
                                          await chat.delete(id);
                                        }
                                      },
                                icon: Icon(
                                  Icons.delete_outline,
                                  color: p.muted,
                                ),
                              ),
                            ],
                          ),
                        ),
                      ),
                    ),
                  );
                },
              ),
            ),
            if (chat.sessionsOlder || chat.sessionsNewer)
              Row(
                children: [
                  Expanded(
                    child: TextButton(
                      key: const Key('newer-chats'),
                      onPressed:
                          !chat.sessionsNewer || chat.busy || chat.changing
                          ? null
                          : () => chat.browseSessions(newer: true),
                      child: const Text('Newer'),
                    ),
                  ),
                  Expanded(
                    child: TextButton(
                      key: const Key('older-chats'),
                      onPressed:
                          !chat.sessionsOlder || chat.busy || chat.changing
                          ? null
                          : () => chat.browseSessions(newer: false),
                      child: const Text('Older'),
                    ),
                  ),
                ],
              ),
            Divider(color: p.border),
            const SizedBox(height: 8),
            TextButton.icon(
              key: const Key('connection'),
              onPressed: chat.busy || chat.changing || chat.loading
                  ? null
                  : settings,
              icon: const Icon(Icons.tune, size: 18),
              label: const Text('Model connection'),
              style: TextButton.styleFrom(
                alignment: Alignment.centerLeft,
                padding: const EdgeInsets.all(12),
              ),
            ),
            Padding(
              padding: const EdgeInsets.fromLTRB(12, 8, 0, 0),
              child: Text(
                'A little more awake, every day.',
                style: TextStyle(fontSize: 11, color: p.muted),
              ),
            ),
          ],
        ),
      ),
    ),
  );

  Widget message(
    Palette p,
    String role,
    String text, {
    bool streaming = false,
    Key? key,
  }) {
    final user = role == 'user';
    return Padding(
      key: key,
      padding: const EdgeInsets.only(bottom: 32),
      child: Row(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Container(
            width: 30,
            height: 30,
            decoration: BoxDecoration(
              color: user ? p.border : p.soft,
              borderRadius: BorderRadius.circular(9),
            ),
            alignment: Alignment.center,
            child: Icon(
              user ? Icons.person_outline : Icons.all_inclusive,
              size: 20,
              color: user ? p.text : p.accent,
            ),
          ),
          const SizedBox(width: 16),
          Expanded(
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                Row(
                  children: [
                    Text(
                      user ? 'You' : 'Dolores',
                      style: TextStyle(
                        fontSize: 13,
                        fontWeight: FontWeight.w600,
                        color: p.text,
                      ),
                    ),
                    if (streaming) ...[
                      const SizedBox(width: 10),
                      Text(
                        'Writing…',
                        style: TextStyle(fontSize: 11, color: p.muted),
                      ),
                    ],
                    const Spacer(),
                    if (!streaming && text.isNotEmpty)
                      IconButton(
                        tooltip: 'Copy message',
                        iconSize: 15,
                        visualDensity: VisualDensity.compact,
                        onPressed: () =>
                            Clipboard.setData(ClipboardData(text: text)),
                        icon: Icon(Icons.copy_outlined, color: p.muted),
                      ),
                  ],
                ),
                const SizedBox(height: 6),
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
              ],
            ),
          ),
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
              'Hello. I’m Dolores.',
              textAlign: TextAlign.center,
              style: TextStyle(
                fontFamily: 'Georgia',
                fontSize: 30,
                color: p.text,
              ),
            ),
            const SizedBox(height: 14),
            Text(
              'A quiet space to think things through.',
              textAlign: TextAlign.center,
              style: TextStyle(fontSize: 15, color: p.muted),
            ),
            const SizedBox(height: 28),
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
          child: Text(
            chat.error!,
            style: TextStyle(color: p.errorText, fontSize: 13),
          ),
        ),
      Container(
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
          child: RichComposer(
            key: const Key('composer'),
            controller: input,
            focusNode: focus,
            readOnly: chat.busy || chat.changing || chat.loading,
            hint: chat.configured
                ? 'Message Dolores…'
                : 'Connect a model to begin…',
            onChanged: (value) {
              chat.draft = value;
              setState(() {});
            },
            onSend: () => unawaited(chat.send()),
            trailing: Padding(
              padding: const EdgeInsets.only(bottom: 3),
              child: SizedBox(
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
                                chat.draft.trim().isEmpty
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
                    chat.busy ? Icons.stop_rounded : Icons.arrow_upward_rounded,
                  ),
                ),
              ),
            ),
          ),
        ),
      ),
      if (chat.configured && chat.enabledModels.isNotEmpty)
        Align(
          alignment: Alignment.centerLeft,
          child: PopupMenuButton<String>(
            key: const Key('chat-model-picker'),
            tooltip: 'Choose model',
            enabled: !chat.busy && !chat.changing && !chat.loading,
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
              padding: const EdgeInsets.symmetric(vertical: 10, horizontal: 4),
              child: Row(
                mainAxisSize: MainAxisSize.min,
                children: [
                  Icon(Icons.auto_awesome_outlined, size: 14, color: p.muted),
                  const SizedBox(width: 6),
                  Flexible(
                    child: Text(
                      chat.model,
                      maxLines: 1,
                      overflow: TextOverflow.ellipsis,
                      style: TextStyle(color: p.muted, fontSize: 12),
                    ),
                  ),
                  Icon(Icons.expand_more, size: 16, color: p.muted),
                ],
              ),
            ),
          ),
        ),
      const SizedBox(height: 12),
      Text(
        'Enter to send · Shift+Enter for a new line · Ctrl/⌘+Enter from code',
        style: TextStyle(fontSize: 11, color: p.muted),
      ),
    ],
  );
  @override
  Widget build(BuildContext context) {
    final p = Palette(Theme.of(context).brightness == Brightness.dark);
    return LayoutBuilder(
      builder: (_, constraints) {
        final narrow = constraints.maxWidth < UiTokens.drawerBreakpoint;
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
                    child: Text(
                      'Conversation',
                      maxLines: 1,
                      overflow: TextOverflow.ellipsis,
                      style: TextStyle(
                        fontWeight: FontWeight.w600,
                        fontSize: 14,
                        color: p.text,
                      ),
                    ),
                  ),
                  ConstrainedBox(
                    constraints: const BoxConstraints(maxWidth: 200),
                    child: Container(
                      padding: const EdgeInsets.symmetric(
                        horizontal: 10,
                        vertical: 5,
                      ),
                      decoration: BoxDecoration(
                        border: Border.all(color: p.border),
                        borderRadius: BorderRadius.circular(6),
                      ),
                      child: Text(
                        chat.configured ? chat.model : 'No model connected',
                        maxLines: 1,
                        overflow: TextOverflow.ellipsis,
                        style: TextStyle(fontSize: 11, color: p.muted),
                      ),
                    ),
                  ),
                  if (chat.session != null)
                    PopupMenuButton<String>(
                      key: const Key('export-chat'),
                      tooltip: 'Export complete conversation',
                      enabled: !chat.busy && !chat.changing && !chat.loading,
                      onSelected: exportChat,
                      icon: const Icon(Icons.file_download_outlined, size: 20),
                      itemBuilder: (_) => const [
                        PopupMenuItem(
                          value: 'markdown',
                          child: Text('Export Markdown'),
                        ),
                        PopupMenuItem(
                          value: 'json',
                          child: Text('Export JSON'),
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
                                      key: ValueKey(
                                        '${chat.session}:${item['id'] ?? index}',
                                      ),
                                    ),
                                  if (chat.busy) ...[
                                    message(
                                      p,
                                      'user',
                                      chat.pendingInput,
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
          drawer: narrow
              ? Drawer(width: UiTokens.sidebarWidth, child: sidebar(p))
              : null,
          body: Row(
            children: [
              if (!narrow) ...[
                sidebar(p),
                Container(width: 1, color: p.border),
              ],
              Expanded(child: body),
            ],
          ),
        );
      },
    );
  }
}

class ConnectionDialog extends StatefulWidget {
  final ChatController chat;
  const ConnectionDialog({super.key, required this.chat});
  @override
  State<ConnectionDialog> createState() => _ConnectionDialogState();
}

class _ConnectionDialogState extends State<ConnectionDialog> {
  late final url = TextEditingController(text: widget.chat.baseUrl);
  final manualModel = TextEditingController();
  final search = TextEditingController();
  late List<String> available = [...widget.chat.enabledModels];
  late final Set<String> selected = {
    ...widget.chat.enabledModels,
    if (widget.chat.model.isNotEmpty) widget.chat.model,
  };
  final keyInput = TextEditingController();
  late bool remember =
      widget.chat.rememberConnection || widget.chat.model.isEmpty;
  bool saving = false;
  String? error;
  bool fetching = false, manual = false;
  bool clearKey = false;
  bool get working => saving || fetching;
  String? get requestKey =>
      !clearKey &&
          keyInput.text.isEmpty &&
          (widget.chat.hasSavedKey || widget.chat.configured)
      ? null
      : keyInput.text;
  void endpointChanged(String _) {
    setState(() {
      available = [];
      selected.clear();
      search.clear();
      error = null;
    });
  }

  void addManual() {
    final id = manualModel.text.trim();
    if (id.isEmpty) return;
    setState(() {
      if (utf8.encode(id).length > 200 || selected.length >= 32) {
        error = 'Choose up to 32 models, with IDs up to 200 bytes.';
        return;
      }
      if (!available.contains(id)) available.add(id);
      selected.add(id);
      manualModel.clear();
      error = null;
    });
  }

  Future<void> fetch() async {
    setState(() {
      fetching = true;
      error = null;
    });
    try {
      final models = await widget.chat.listModels(url.text, requestKey);
      if (mounted) {
        setState(() {
          available = {...models, ...selected}.toList()..sort();
          if (selected.isEmpty) selected.add(models.first);
        });
      }
    } catch (failure) {
      if (mounted) {
        setState(() {
          error = failure.toString();
          manual = true;
        });
      }
    } finally {
      if (mounted) setState(() => fetching = false);
    }
  }

  @override
  void dispose() {
    url.dispose();
    manualModel.dispose();
    search.dispose();
    keyInput.dispose();
    super.dispose();
  }

  Future<void> save() async {
    setState(() {
      saving = true;
      error = null;
    });
    try {
      await widget.chat.configure(
        url.text,
        selected.contains(widget.chat.model)
            ? widget.chat.model
            : selected.first,
        requestKey,
        remember: remember,
        models: selected.toList(),
      );
      if (mounted) Navigator.pop(context);
    } catch (failure) {
      if (mounted) {
        setState(() {
          error = failure.toString();
          saving = false;
        });
      }
    }
  }

  Future<void> forget() async {
    setState(() {
      saving = true;
      error = null;
    });
    try {
      await widget.chat.forgetConnection();
      if (mounted) Navigator.pop(context);
    } catch (failure) {
      if (mounted) {
        setState(() {
          saving = false;
          error = failure.toString();
        });
      }
    }
  }

  @override
  Widget build(BuildContext context) => PopScope(
    canPop: !working,
    child: AlertDialog(
      shape: RoundedRectangleBorder(borderRadius: BorderRadius.circular(16)),
      title: const Text(
        'Model connection',
        style: TextStyle(fontSize: 20, fontWeight: FontWeight.w600),
      ),
      content: SizedBox(
        width: 420,
        child: SingleChildScrollView(
          child: Column(
            mainAxisSize: MainAxisSize.min,
            crossAxisAlignment: CrossAxisAlignment.stretch,
            children: [
              const Text(
                'Use an OpenAI-compatible local or hosted API.',
                style: TextStyle(fontSize: 13),
              ),
              const SizedBox(height: 24),
              TextField(
                key: const Key('base-url'),
                controller: url,
                enabled: !working,
                onChanged: endpointChanged,
                decoration: const InputDecoration(
                  labelText: 'Base URL',
                  hintText: 'http://localhost:11434/v1',
                ),
              ),
              const SizedBox(height: 18),
              TextField(
                key: const Key('api-key'),
                controller: keyInput,
                enabled: !working,
                obscureText: true,
                enableSuggestions: false,
                autocorrect: false,
                onChanged: (_) => setState(() => clearKey = false),
                decoration: InputDecoration(
                  labelText: 'API key (optional for local servers)',
                  helperText: clearKey
                      ? 'This connection will use no key.'
                      : widget.chat.hasSavedKey || widget.chat.configured
                      ? 'Leave blank to keep the current key.'
                      : null,
                ),
              ),
              if (widget.chat.hasSavedKey || widget.chat.configured)
                Align(
                  alignment: Alignment.centerLeft,
                  child: TextButton(
                    onPressed: working
                        ? null
                        : () => setState(() {
                            clearKey = !clearKey;
                            keyInput.clear();
                          }),
                    child: Text(
                      clearKey ? 'Keep existing key' : 'Use without a key',
                    ),
                  ),
                ),
              const SizedBox(height: 18),
              OutlinedButton.icon(
                key: const Key('fetch-models'),
                onPressed: working ? null : fetch,
                icon: fetching
                    ? const SizedBox(
                        width: 14,
                        height: 14,
                        child: CircularProgressIndicator(strokeWidth: 2),
                      )
                    : const Icon(Icons.refresh, size: 16),
                label: Text(fetching ? 'Fetching models…' : 'Fetch models'),
              ),
              if (available.isNotEmpty) ...[
                const SizedBox(height: 12),
                Text(
                  '${selected.length} selected · Available in the chat model picker',
                  style: const TextStyle(fontSize: 12),
                ),
                const SizedBox(height: 8),
                TextField(
                  key: const Key('model-search'),
                  controller: search,
                  onChanged: (_) => setState(() {}),
                  enabled: !working,
                  decoration: const InputDecoration(
                    hintText: 'Search models',
                    prefixIcon: Icon(Icons.search, size: 18),
                  ),
                ),
                const SizedBox(height: 6),
                SizedBox(
                  height: (available.length * 44.0).clamp(44, 176),
                  child: Builder(
                    builder: (_) {
                      final filtered = available
                          .where(
                            (id) => id.toLowerCase().contains(
                              search.text.toLowerCase(),
                            ),
                          )
                          .toList();
                      return ListView.builder(
                        itemCount: filtered.length,
                        itemBuilder: (_, index) {
                          final id = filtered[index];
                          return CheckboxListTile(
                            key: ValueKey('enable-model-$id'),
                            dense: true,
                            contentPadding: EdgeInsets.zero,
                            title: Text(
                              id,
                              maxLines: 2,
                              overflow: TextOverflow.ellipsis,
                              style: const TextStyle(fontSize: 13),
                            ),
                            value: selected.contains(id),
                            onChanged: working
                                ? null
                                : (value) => setState(() {
                                    if (value! && selected.length >= 32) {
                                      error = 'Choose up to 32 models.';
                                    } else {
                                      value
                                          ? selected.add(id)
                                          : selected.remove(id);
                                      error = null;
                                    }
                                  }),
                          );
                        },
                      );
                    },
                  ),
                ),
              ],
              TextButton(
                key: const Key('manual-model-toggle'),
                onPressed: working
                    ? null
                    : () => setState(() => manual = !manual),
                child: const Text('Add a model manually'),
              ),
              if (manual)
                Row(
                  children: [
                    Expanded(
                      child: TextField(
                        key: const Key('manual-model'),
                        controller: manualModel,
                        enabled: !working,
                        onSubmitted: (_) => addManual(),
                        decoration: const InputDecoration(
                          labelText: 'Model ID',
                        ),
                      ),
                    ),
                    IconButton(
                      key: const Key('add-manual-model'),
                      tooltip: 'Add model',
                      onPressed: working ? null : addManual,
                      icon: const Icon(Icons.add),
                    ),
                  ],
                ),
              CheckboxListTile(
                key: const Key('remember-connection'),
                contentPadding: EdgeInsets.zero,
                title: const Text(
                  'Remember connection',
                  style: TextStyle(fontSize: 14),
                ),
                subtitle: Text(
                  remember
                      ? 'Store your key in the OS credential store.'
                      : 'Use this connection until Dolores closes.',
                  style: const TextStyle(fontSize: 12),
                ),
                value: remember,
                onChanged: working
                    ? null
                    : (value) => setState(() => remember = value!),
              ),
              if (widget.chat.rememberConnection)
                TextButton(
                  key: const Key('forget-connection'),
                  onPressed: working ? null : forget,
                  child: const Text('Forget saved connection'),
                ),
              if (error != null)
                Padding(
                  padding: const EdgeInsets.only(top: 16),
                  child: Text(
                    error!,
                    style: TextStyle(
                      color: Theme.of(context).colorScheme.error,
                      fontSize: 13,
                    ),
                  ),
                ),
            ],
          ),
        ),
      ),
      actions: [
        TextButton(
          onPressed: working ? null : () => Navigator.pop(context),
          child: const Text('Cancel'),
        ),
        FilledButton(
          key: const Key('save-connection'),
          onPressed: working || selected.isEmpty ? null : save,
          child: Text(saving ? 'Saving…' : 'Save connection'),
        ),
      ],
    ),
  );
}
