import 'dart:convert';

import 'package:flutter/material.dart';

import 'chat.dart';
import 'form_field_layout.dart';
import 'inspector.dart';

int _sequence = 0;
String _identity() =>
    'side-${DateTime.now().microsecondsSinceEpoch}-${_sequence++}';

Future<void> showSideConversation(
  BuildContext context,
  ChatController chat,
  int message,
  String excerpt, {
  bool bringBack = false,
}) => showDialog<void>(
  context: context,
  barrierDismissible: false,
  builder: (_) => SideConversationSelection(
    chat: chat,
    message: message,
    excerpt: excerpt,
    bringBack: bringBack,
  ),
);

class SideConversationSelection extends StatefulWidget {
  final ChatController chat;
  final int message;
  final String excerpt;
  final bool bringBack;
  const SideConversationSelection({
    super.key,
    required this.chat,
    required this.message,
    required this.excerpt,
    this.bringBack = false,
  });
  @override
  State<SideConversationSelection> createState() =>
      _SideConversationSelectionState();
}

class _SideConversationSelectionState extends State<SideConversationSelection> {
  late final selected = TextEditingController(text: widget.excerpt);
  final question = TextEditingController();
  late final owner = widget.chat.session!;
  final id = _identity();
  Map<String, dynamic>? quote, attempt;
  bool busy = false;
  String? error;
  @override
  void initState() {
    super.initState();
    WidgetsBinding.instance.addPostFrameCallback((_) {
      if (mounted) preview();
    });
  }

  @override
  void dispose() {
    selected.dispose();
    question.dispose();
    super.dispose();
  }

  Future<void> preview() async {
    if (busy) return;
    setState(() {
      busy = true;
      error = null;
      quote = null;
    });
    try {
      final result = await widget.chat.sideCall(owner, {
        'action': widget.bringBack ? 'previewReturn' : 'preview',
        'message': widget.message,
        'excerpt': selected.text,
      });
      if (mounted) setState(() => quote = Map<String, dynamic>.from(result));
    } catch (e) {
      if (mounted) setState(() => error = '$e');
    }
    if (mounted) setState(() => busy = false);
  }

  Future<void> apply([String? mode]) async {
    if (busy || quote == null) return;
    if (!widget.bringBack &&
        (utf8.encode(question.text).length > 4096 ||
            question.text.contains('\u0000'))) {
      setState(
        () => error = 'Shorten the question to 4 KiB and remove NUL characters. Your question and excerpt remain editable.',
      );
      return;
    }
    setState(() {
      busy = true;
      error = null;
    });
    try {
      if (widget.bringBack) {
        attempt ??= {'id': id, 'quote': quote, 'mode': mode};
        await widget.chat.returnSide(owner, attempt!);
      } else {
        attempt ??= {'id': id, 'quote': quote, 'question': question.text};
        await widget.chat.createSide(owner, attempt!);
      }
      if (mounted) Navigator.pop(context);
    } catch (e) {
      if (mounted) setState(() => error = '$e');
    }
    if (mounted) setState(() => busy = false);
  }

  @override
  Widget build(BuildContext context) => PopScope(
    canPop: !busy,
    child: InspectorFrame(
      title: widget.bringBack ? 'Bring conclusion to parent' : 'Ask about this',
      subtitle: widget.bringBack
          ? 'Share selected text through the parent task’s queue'
          : 'Open an independent discussion with a frozen excerpt',
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
                    Text(
                      widget.bringBack
                          ? 'Only this selected conclusion will be sent. The parent’s task and draft stay in place.'
                          : 'Only selected text is shared. Other messages, attachments and folder access stay with the parent chat.',
                    ),
                    const SizedBox(height: 12),
                    FieldLabel(
                      label: 'Selected excerpt',
                      child: TextField(
                        key: const Key('side-excerpt'),
                        controller: selected,
                        minLines: 3,
                        maxLines: 8,
                        readOnly: busy || attempt != null,
                        decoration: const InputDecoration(),
                        onChanged: (_) => setState(() => quote = null),
                      ),
                    ),
                    const SizedBox(height: 8),
                    Text(
                      '${utf8.encode(selected.text).length} / 8192 bytes',
                      style: Theme.of(context).textTheme.bodySmall,
                    ),
                    if (!widget.bringBack) ...[
                      const SizedBox(height: 12),
                      FieldLabel(
                        label: 'Question draft (optional)',
                        child: TextField(
                          key: const Key('side-question'),
                          controller: question,
                          minLines: 2,
                          maxLines: 4,
                          readOnly: busy || attempt != null,
                          decoration: const InputDecoration(
                            hintText: 'Ask in the new chat when you are ready',
                          ),
                        ),
                      ),
                    ],
                    if (quote != null)
                      Padding(
                        padding: const EdgeInsets.symmetric(vertical: 8),
                        child: Text(
                          'Saved message ${quote!['message']} · this excerpt stays available if the original is changed or removed.',
                          style: Theme.of(context).textTheme.bodySmall,
                        ),
                      ),
                    if (error != null)
                      Padding(
                        padding: const EdgeInsets.symmetric(vertical: 12),
                        child: SelectableText(error!),
                      ),
                  ],
                ),
              ),
            ),
            const SizedBox(height: 12),
            if (busy) const LinearProgressIndicator(),
            Wrap(
              spacing: 8,
              runSpacing: 8,
              children: [
                if (quote == null)
                  OutlinedButton(
                    onPressed: busy ? null : preview,
                    child: const Text('Preview selection'),
                  ),
                if (!widget.bringBack)
                  FilledButton(
                    onPressed: busy || quote == null ? null : () => apply(),
                    child: Text(
                      attempt == null
                          ? 'Open side chat'
                          : 'Retry opening saved discussion',
                    ),
                  ),
                if (widget.bringBack) ...[
                  FilledButton(
                    onPressed:
                        busy ||
                            quote == null ||
                            attempt != null && attempt!['mode'] != 'queue'
                        ? null
                        : () => apply('queue'),
                    child: Text(
                      attempt == null
                          ? 'Queue in parent'
                          : 'Retry queued conclusion',
                    ),
                  ),
                  OutlinedButton(
                    onPressed:
                        busy ||
                            quote == null ||
                            attempt != null && attempt!['mode'] != 'steer'
                        ? null
                        : () => apply('steer'),
                    child: Text(
                      attempt == null
                          ? 'Steer parent'
                          : 'Retry steering conclusion',
                    ),
                  ),
                ],
              ],
            ),
          ],
        ),
      ),
    ),
  );
}

class SideConversationBanner extends StatelessWidget {
  final ChatController chat;
  const SideConversationBanner({super.key, required this.chat});
  @override
  Widget build(BuildContext context) {
    final info = chat.sideConversation;
    if (info == null && chat.sideConversationError == null) {
      return const SizedBox.shrink();
    }
    if (info == null) {
      return Row(
        children: [
          Expanded(child: Text(chat.sideConversationError!)),
          TextButton(
            onPressed: chat.refreshSideConversation,
            child: const Text('Retry side link'),
          ),
        ],
      );
    }
    final origin = info['origin'] as Map;
    final receipts = origin['returns'] as List;
    return Padding(
      padding: const EdgeInsets.only(bottom: 12),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          if (chat.sideConversationError != null)
            Wrap(
              children: [
                Text(chat.sideConversationError!),
                TextButton(
                  onPressed: chat.refreshSideConversation,
                  child: const Text('Retry side link'),
                ),
              ],
            ),
          Wrap(
            spacing: 12,
            crossAxisAlignment: WrapCrossAlignment.center,
            children: [
              Text(
                'Side discussion · ${origin['parentTitle']}',
                style: Theme.of(context).textTheme.bodySmall,
              ),
              TextButton.icon(
                key: const Key('side-return-parent'),
                onPressed: info['parentAvailable'] != true
                    ? null
                    : () async {
                        try {
                          await chat.navigateConversation(
                            origin['parent'] as String,
                          );
                        } catch (e) {
                          if (context.mounted) {
                            ScaffoldMessenger.of(context)
                                .showSnackBar(SnackBar(content: Text('$e')));
                          }
                        }
                      },
                icon: const Icon(Icons.arrow_back, size: 16),
                label: const Text('Return to parent'),
              ),
            ],
          ),
          if (info['parentAvailable'] != true)
            const Text(
              'Parent chat is unavailable. This discussion and its frozen excerpt are retained.',
            ),
          ExpansionTile(
            tilePadding: EdgeInsets.zero,
            title: const Text('Shared excerpt', style: TextStyle(fontSize: 13)),
            children: [
              ConstrainedBox(
                constraints: const BoxConstraints(maxHeight: 160),
                child: SingleChildScrollView(
                  child: SelectableText(origin['quote']['excerpt'] as String),
                ),
              ),
              if (info['sourceAvailable'] != true)
                const Text(
                  'The original message is unavailable; this frozen selection is retained.',
                ),
            ],
          ),
          if (receipts.isNotEmpty)
            ExpansionTile(
              tilePadding: EdgeInsets.zero,
              title: Text(
                '${receipts.length} returned conclusions',
                style: const TextStyle(fontSize: 13),
              ),
              children: [
                ConstrainedBox(
                  constraints: const BoxConstraints(maxHeight: 120),
                  child: ListView(
                    shrinkWrap: true,
                    children: [
                      for (final r in receipts)
                        SelectableText(
                          '${r['mode']} input · ${r['id']}',
                          style: const TextStyle(fontSize: 12),
                        ),
                    ],
                  ),
                ),
              ],
            ),
        ],
      ),
    );
  }
}
