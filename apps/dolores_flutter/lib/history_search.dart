import 'dart:async';

import 'package:flutter/material.dart';
import 'package:flutter/services.dart';

import 'chat.dart';
import 'help_tip.dart';
import 'theme.dart';

Future<void> showHistorySearch(BuildContext context, ChatController chat) =>
    showDialog<void>(
      context: context,
      barrierColor: Colors.black.withValues(alpha: .08),
      builder: (_) => Align(
        alignment: Alignment.topCenter,
        child: Padding(
          padding: const EdgeInsets.fromLTRB(16, 48, 16, 16),
          child: Material(
            elevation: 8,
            color: Theme.of(context).colorScheme.surface,
            borderRadius: BorderRadius.circular(16),
            clipBehavior: Clip.antiAlias,
            child: SizedBox(width: 520, child: HistorySearch(chat: chat)),
          ),
        ),
      ),
    );

class HistorySearchButton extends StatelessWidget {
  final ChatController chat;
  final VoidCallback? onPressed;
  const HistorySearchButton({super.key, required this.chat, this.onPressed});
  @override
  Widget build(BuildContext context) => IconButton(
    key: const Key('search-conversations'),
    tooltip: 'Search conversations (Ctrl+K)',
    constraints: const BoxConstraints.tightFor(width: 46, height: 32),
    padding: EdgeInsets.zero,
    onPressed: onPressed ?? () => showHistorySearch(context, chat),
    icon: const Icon(Icons.search, size: 17),
  );
}

class HistorySearch extends StatefulWidget {
  final ChatController chat;
  const HistorySearch({super.key, required this.chat});
  @override
  State<HistorySearch> createState() => _HistorySearchState();
}

class _HistorySearchState extends State<HistorySearch> {
  final text = TextEditingController();
  final focus = FocusNode();
  Timer? debounce;
  int revision = 0, selected = -1;
  bool busy = false, opening = false;
  List<Map<String, dynamic>> items = [];
  Map<String, dynamic>? cursor;
  String? error;
  Future<dynamic> call(Map<String, dynamic> request) =>
      widget.chat.bridge.call({'command': 'history', 'request': request});
  @override
  void dispose() {
    debounce?.cancel();
    text.dispose();
    focus.dispose();
    super.dispose();
  }

  void changed(String value) {
    debounce?.cancel();
    revision++;
    setState(() {
      items = [];
      cursor = null;
      error = null;
      busy = value.trim().isNotEmpty;
      selected = -1;
    });
    if (value.trim().isNotEmpty) {
      debounce = Timer(const Duration(milliseconds: 250), search);
    }
  }

  Future<void> search({bool more = false}) async {
    debounce?.cancel();
    final query = text.text.trim();
    if (query.isEmpty) return;
    final request = ++revision;
    setState(() {
      busy = true;
      error = null;
    });
    try {
      final page = await call({
        'action': 'search',
        'query': {
          'filter': {
            'text': query,
            'archive': 'all',
            'pinned': null,
            'kind': null,
            'root': null,
          },
          'cursor': more ? cursor : null,
        },
      }) as Map;
      if (!mounted || revision != request) return;
      setState(() {
        items = (page['items'] as List)
            .map((v) => Map<String, dynamic>.from(v as Map))
            .toList();
        cursor = page['nextCursor'] is Map
            ? Map<String, dynamic>.from(page['nextCursor'] as Map)
            : null;
        selected = -1;
      });
    } catch (e) {
      if (mounted && revision == request) setState(() => error = '$e');
    } finally {
      if (mounted && revision == request) setState(() => busy = false);
    }
  }

  Future<void> open(Map<String, dynamic> item) async {
    if (opening) return;
    setState(() => opening = true);
    try {
      await widget.chat.checkpointDraft();
      if (!mounted) return;
      await widget.chat.navigateConversation(item['session']['id'] as String);
      if (mounted) Navigator.pop(context);
    } catch (e) {
      if (mounted) setState(() => error = '$e');
    } finally {
      if (mounted) setState(() => opening = false);
    }
  }

  @override
  Widget build(BuildContext context) {
    final p = Palette(Theme.of(context).brightness == Brightness.dark);
    return CallbackShortcuts(
      bindings: {
        const SingleActivator(LogicalKeyboardKey.escape): () =>
            Navigator.pop(context),
        const SingleActivator(LogicalKeyboardKey.arrowDown): () {
          if (items.isNotEmpty) {
            setState(() => selected = (selected + 1) % items.length);
          }
        },
        const SingleActivator(LogicalKeyboardKey.arrowUp): () {
          if (items.isNotEmpty) {
            setState(() => selected = (selected - 1) % items.length);
          }
        },
      },
      child: ConstrainedBox(
        constraints: BoxConstraints(
          maxHeight: (MediaQuery.sizeOf(context).height - 100).clamp(100, 440),
        ),
        child: Column(
          mainAxisSize: MainAxisSize.min,
          children: [
            Padding(
              padding: const EdgeInsets.all(8),
              child: TextField(
                key: const Key('history-search-text'),
                controller: text,
                focusNode: focus,
                autofocus: true,
                onChanged: changed,
                onSubmitted: (_) =>
                    selected >= 0 ? open(items[selected]) : search(),
                decoration: InputDecoration(
                  hintText: 'Search conversations',
                  prefixIcon: const Icon(Icons.search, size: 18),
                  border: InputBorder.none,
                  enabledBorder: InputBorder.none,
                  focusedBorder: InputBorder.none,
                  fillColor: Colors.transparent,
                  suffixIcon: IconButton(
                    tooltip: 'Close search',
                    onPressed: () => Navigator.pop(context),
                    icon: const Icon(Icons.close, size: 18),
                  ),
                ),
              ),
            ),
            if (busy || opening) const LinearProgressIndicator(minHeight: 2),
            if (items.isNotEmpty)
              Flexible(
                child: ListView.builder(
                  primary: false,
                  shrinkWrap: true,
                  padding: const EdgeInsets.fromLTRB(8, 0, 8, 8),
                  itemCount: items.length + (cursor == null ? 0 : 1),
                  itemBuilder: (context, i) {
                    if (i == items.length) {
                      return TextButton(
                        onPressed: busy ? null : () => search(more: true),
                        child: const Text('More results'),
                      );
                    }
                    final item = items[i];
                    return ListTile(
                      key: Key('history-result-${item['session']['id']}'),
                      shape: RoundedRectangleBorder(
                        borderRadius: BorderRadius.circular(10),
                      ),
                      selected: selected == i,
                      selectedTileColor: p.soft,
                      contentPadding: const EdgeInsets.symmetric(
                        horizontal: 12,
                        vertical: 2,
                      ),
                      title: Text(
                        item['session']['title'] as String,
                        maxLines: 1,
                        overflow: TextOverflow.ellipsis,
                      ),
                      subtitle: item['excerpt'] == null
                          ? null
                          : Text(
                              item['excerpt']['text'] as String,
                              maxLines: 2,
                              overflow: TextOverflow.ellipsis,
                              style: TextStyle(color: p.muted, fontSize: 12),
                            ),
                      onTap: opening ? null : () => open(item),
                    );
                  },
                ),
              ),
            if (!busy &&
                error == null &&
                text.text.trim().isNotEmpty &&
                items.isEmpty)
              const Padding(
                padding: EdgeInsets.all(16),
                child: Text('No conversations found.'),
              ),
            if (error != null)
              Padding(
                padding: const EdgeInsets.fromLTRB(16, 0, 16, 12),
                child: HelpLabel(
                  help: error!,
                  child: Text(
                    'Search unavailable. Press Enter to retry.',
                    style: TextStyle(color: p.errorText),
                  ),
                ),
              ),
          ],
        ),
      ),
    );
  }
}
