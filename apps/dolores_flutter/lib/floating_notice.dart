import 'dart:async';

import 'attention_host.dart';
import 'notification_scope.dart';

import 'package:flutter/material.dart';

import 'help_tip.dart';
import 'theme.dart';

/// A persistent, dismissible notice that overlays content without moving it.
class FloatingNotice extends StatefulWidget {
  final String message;
  final String title, severity;
  final VoidCallback onDismiss;
  final VoidCallback? onRetry;
  const FloatingNotice({
    super.key,
    required this.message,
    required this.onDismiss,
    this.onRetry,
    this.title = "Dolores update",
    this.severity = "info",
  });
  @override
  State<FloatingNotice> createState() => _FloatingNoticeState();
}

class _FloatingNoticeState extends State<FloatingNotice> {
  AttentionHost? host;
  final identities = <String>{};
  String get message => widget.message;
  VoidCallback get onDismiss => widget.onDismiss;
  VoidCallback? get onRetry => widget.onRetry;
  @override
  void didChangeDependencies() {
    super.didChangeDependencies();
    final next = NotificationScope.of(context);
    if (next != host) {
      host = next;
      schedule();
    }
  }

  @override
  void didUpdateWidget(FloatingNotice old) {
    super.didUpdateWidget(old);
    if (old.message != message) schedule();
  }

  void schedule() {
    WidgetsBinding.instance.addPostFrameCallback((_) {
      if (mounted && host != null) unawaited(publish());
    });
  }

  Future<void> publish() async {
    final owner = host!;
    final text = message;
    final id = await owner.publish(
      'surface-${identityHashCode(this)}',
      widget.title,
      text,
      severity: widget.severity,
      onRetry: onRetry == null
          ? null
          : () {
              if (mounted) onRetry?.call();
            },
      onDismiss: () {
        if (mounted && message == text) onDismiss();
      },
    );
    if (id != null) {
      if (mounted) {
        identities.add(id);
      } else {
        owner.discardCallbacks(id);
      }
    }
  }

  @override
  void dispose() {
    for (final id in identities) {
      host?.discardCallbacks(id);
    }
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    if (host != null) return const SizedBox.shrink();
    final p = Palette(Theme.of(context).brightness == Brightness.dark);
    return Material(
      elevation: 6,
      color: p.surface,
      surfaceTintColor: Colors.transparent,
      shape: RoundedRectangleBorder(
        borderRadius: BorderRadius.circular(14),
        side: BorderSide(color: p.border),
      ),
      child: Padding(
        padding: const EdgeInsets.fromLTRB(14, 10, 4, 10),
        child: Row(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Padding(
              padding: const EdgeInsets.only(top: 3),
              child: Icon(Icons.info_outline, size: 18, color: p.muted),
            ),
            const SizedBox(width: 10),
            Expanded(
              child: Column(
                crossAxisAlignment: CrossAxisAlignment.start,
                mainAxisSize: MainAxisSize.min,
                children: [
                  Text(
                    message,
                    maxLines: 4,
                    overflow: TextOverflow.ellipsis,
                    style: const TextStyle(fontSize: 13),
                  ),
                  if (onRetry != null)
                    TextButton(onPressed: onRetry, child: const Text('Retry')),
                ],
              ),
            ),
            if (message.length > 180) HelpTip(message: message),
            IconButton(
              tooltip: 'Dismiss',
              onPressed: onDismiss,
              icon: const Icon(Icons.close, size: 16),
              visualDensity: VisualDensity.compact,
            ),
          ],
        ),
      ),
    );
  }
}
