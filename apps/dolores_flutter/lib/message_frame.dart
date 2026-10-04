import 'package:flutter/material.dart';
import 'package:flutter/services.dart';

import 'theme.dart';

/// Hover chrome has a reserved footprint so reading/selection never jumps.
class MessageFrame extends StatefulWidget {
  final bool user;
  final String text;
  final int? savedAt;
  final bool streaming;
  final Widget child;

  const MessageFrame({
    super.key,
    required this.user,
    required this.text,
    required this.child,
    this.savedAt,
    this.streaming = false,
  });

  @override
  State<MessageFrame> createState() => _MessageFrameState();
}

class _MessageFrameState extends State<MessageFrame> {
  bool hovered = false;
  bool focused = false;

  String? get timestamp {
    final milliseconds = widget.savedAt;
    if (milliseconds == null || milliseconds <= 0) return null;
    // Malformed/unsupported old data must never break message rendering.
    try {
      final date = DateTime.fromMillisecondsSinceEpoch(milliseconds);
      if (date.year < 1970 || date.year > 9999) return null;
      String two(int value) => value.toString().padLeft(2, '0');
      return '${date.year}-${two(date.month)}-${two(date.day)} '
          '${two(date.hour)}:${two(date.minute)}';
    } on ArgumentError {
      return null;
    }
  }

  @override
  Widget build(BuildContext context) {
    final p = Palette(Theme.of(context).brightness == Brightness.dark);
    final time = timestamp;
    return Padding(
      padding: const EdgeInsets.only(bottom: 16),
      child: LayoutBuilder(
        builder: (context, constraints) => Align(
          alignment: widget.user ? Alignment.centerRight : Alignment.centerLeft,
          child: ConstrainedBox(
            constraints: BoxConstraints(
              maxWidth: constraints.maxWidth * (widget.user ? .88 : 1),
            ),
            child: MouseRegion(
              onEnter: (_) => setState(() => hovered = true),
              onExit: (_) => setState(() => hovered = false),
              child: Focus(
                onFocusChange: (value) => setState(() => focused = value),
                child: Column(
                  mainAxisSize: MainAxisSize.min,
                  crossAxisAlignment: widget.user
                      ? CrossAxisAlignment.end
                      : CrossAxisAlignment.start,
                  children: [
                    Container(
                      key: const Key('message-body'),
                      width: widget.user ? null : double.infinity,
                      padding: widget.user
                          ? const EdgeInsets.symmetric(
                              horizontal: 16,
                              vertical: 12,
                            )
                          : EdgeInsets.zero,
                      decoration: widget.user
                          ? BoxDecoration(
                              color: Theme.of(context).colorScheme.primary,
                              borderRadius: BorderRadius.circular(18),
                            )
                          : null,
                      child: widget.child,
                    ),
                    if (!widget.streaming)
                      SizedBox(
                        height: 32,
                        child: Visibility(
                          visible: hovered || focused,
                          maintainState: true,
                          maintainAnimation: true,
                          maintainSize: true,
                          child: Row(
                            mainAxisSize: MainAxisSize.min,
                            children: [
                              if (time != null)
                                Flexible(
                                  child: Tooltip(
                                    message: 'Saved $time (local time)',
                                    child: Text(
                                      time,
                                      maxLines: 1,
                                      overflow: TextOverflow.ellipsis,
                                      style: TextStyle(
                                        fontSize: 11,
                                        color: p.muted,
                                      ),
                                    ),
                                  ),
                                ),
                              if (widget.text.isNotEmpty)
                                IconButton(
                                  tooltip: 'Copy message',
                                  iconSize: 15,
                                  visualDensity: VisualDensity.compact,
                                  onPressed: () => Clipboard.setData(
                                    ClipboardData(text: widget.text),
                                  ),
                                  icon: Icon(
                                    Icons.copy_outlined,
                                    color: p.muted,
                                  ),
                                ),
                            ],
                          ),
                        ),
                      ),
                  ],
                ),
              ),
            ),
          ),
        ),
      ),
    );
  }
}
