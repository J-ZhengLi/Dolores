import 'notification_scope.dart';

import 'dart:async';
import 'dart:math' as math;
import 'dart:ui' show lerpDouble;

import 'package:flutter/material.dart';
import 'package:flutter/services.dart';

import 'attention_host.dart';
import 'liquid_glass.dart';

/// Presentation belongs to the app overlay; notification history belongs to its host.
class NotificationOverlay extends StatelessWidget {
  final AttentionHost? host;
  final Widget child;
  final double top;
  const NotificationOverlay({
    super.key,
    required this.host,
    required this.child,
    this.top = 42,
  });
  @override
  Widget build(BuildContext context) => host == null
      ? child
      : NotificationScope(
          host: host!,
          // This layer also sits above Navigator in MaterialApp.builder.
          // Its tooltips need a full-window overlay ancestor of their own.
          child: Overlay.wrap(
            child: Stack(
              children: [
                Positioned.fill(child: child),
                Positioned(
                  top: top,
                  left: 16,
                  right: 16,
                  child: Align(
                    alignment: Alignment.topCenter,
                    child: NotificationToast(host: host!),
                  ),
                ),
              ],
            ),
          ),
        );
}

class NotificationToast extends StatefulWidget {
  final AttentionHost host;
  const NotificationToast({super.key, required this.host});
  @override
  State<NotificationToast> createState() => _NotificationToastState();
}

class _NotificationToastState extends State<NotificationToast>
    with TickerProviderStateMixin {
  late final entry = AnimationController(
    vsync: this,
    duration: const Duration(milliseconds: 340),
  );
  late final detail = AnimationController(
    vsync: this,
    duration: const Duration(milliseconds: 360),
  );
  Timer? timer;
  String? identity;
  Map<String, dynamic>? renderedNotice;
  bool hovered = false, focused = false, dismissing = false;
  double drag = 0;
  final focus = FocusNode();
  FocusNode? returnFocus;
  bool detailsWereOpen = false;
  AttentionHost get host => widget.host;
  bool get reduce =>
      MediaQuery.disableAnimationsOf(context) ||
      MediaQuery.accessibleNavigationOf(context);
  @override
  void initState() {
    super.initState();
    host.addListener(changed);
  }

  @override
  void didChangeDependencies() {
    super.didChangeDependencies();
    changed();
  }

  @override
  void didUpdateWidget(NotificationToast old) {
    super.didUpdateWidget(old);
    if (old.host != host) {
      old.host.removeListener(changed);
      host.addListener(changed);
      identity = null;
      changed();
    }
  }

  void changed() {
    if (!mounted) return;
    final next = host.toast?['id'] as String?;
    if (!identical(renderedNotice, host.toast)) {
      renderedNotice = host.toast;
      if (dismissing && host.toast != null) {
        dismissing = false;
        unawaited(entry.forward());
      }
    }
    if (host.detailsOpen && !detailsWereOpen) {
      returnFocus = FocusManager.instance.primaryFocus;
      WidgetsBinding.instance.addPostFrameCallback((_) {
        if (mounted && host.detailsOpen) focus.requestFocus();
      });
    }
    detailsWereOpen = host.detailsOpen;
    if (next != identity) {
      identity = next;
      dismissing = false;
      timer?.cancel();
      if (next != null) {
        entry.value = reduce || host.detailsOpen ? 1 : 0;
        if (!reduce) unawaited(entry.forward());
        detail.value = 0;
      }
    }
    if (dismissing) {
      setState(() {});
      return;
    }
    if (next != null) {
      if (reduce) {
        entry.value = 1;
        detail.value = host.detailsOpen ? 1 : 0;
      } else {
        unawaited(
          detail.animateTo(host.detailsOpen ? 1 : 0, curve: Curves.linear),
        );
      }
      arm();
    }
    setState(() {});
  }

  void arm() {
    timer?.cancel();
    if (identity != null &&
        !host.detailsOpen &&
        !hovered &&
        !focused &&
        !dismissing) {
      timer = Timer(const Duration(seconds: 5), dismiss);
    }
  }

  Future<void> dismiss() async {
    if (dismissing || identity == null) return;
    final id = identity;
    final notice = host.toast;
    dismissing = true;
    timer?.cancel();
    try {
      if (!reduce) {
        await detail.reverse().orCancel;
        if (mounted) await entry.reverse().orCancel;
      }
    } on TickerCanceled {
      return;
    }
    if (mounted && identity == id && identical(host.toast, notice)) {
      host.dismissToast();
      if (returnFocus?.context != null) returnFocus!.requestFocus();
    }
  }

  @override
  void dispose() {
    host.removeListener(changed);
    timer?.cancel();
    entry.dispose();
    detail.dispose();
    focus.dispose();
    super.dispose();
  }

  IconData icon(Map n) => switch (n['severity'] ?? n['kind']) {
    'success' || 'completed' => Icons.check_circle_outline,
    'error' || 'failed' => Icons.error_outline,
    'warning' || 'approval' => Icons.pending_actions,
    'question' => Icons.help_outline,
    _ => Icons.info_outline,
  };
  String title(Map n) =>
      n['title'] as String? ?? attentionLabel(n['kind'] as String? ?? 'app');
  @override
  Widget build(BuildContext context) {
    final notice = host.toast;
    if (notice == null) return const SizedBox.shrink();
    final viewport = MediaQuery.sizeOf(context);
    final compactWidth = math.min(320.0, viewport.width - 32);
    final fullWidth = math.min(420.0, viewport.width - 32);
    final compactHeight = MediaQuery.textScalerOf(context)
        .scale(56)
        .clamp(56.0, 88.0);
    final fullHeight = math.min(280.0, viewport.height - 96);
    final expanded = host.detailsOpen;
    return AnimatedBuilder(
      animation: Listenable.merge([entry, detail]),
      builder: (context, _) {
        double stage(double value, double begin, double end) => Curves
            .easeOutCubic
            .transform(((value - begin) / (end - begin)).clamp(0.0, 1.0));
        final width =
            lerpDouble(8, compactWidth, stage(entry.value, 0, .48))! +
            (fullWidth - compactWidth) * stage(detail.value, 0, .48);
        final height =
            lerpDouble(8, compactHeight, stage(entry.value, .48, 1))! +
            (fullHeight - compactHeight) * stage(detail.value, .48, 1);
        final targetWidth = expanded ? fullWidth : compactWidth;
        final targetHeight = expanded ? fullHeight : compactHeight;
        return MouseRegion(
          onEnter: (_) {
            hovered = true;
            arm();
          },
          onExit: (_) {
            hovered = false;
            arm();
          },
          child: Focus(
            focusNode: focus,
            onFocusChange: (v) {
              focused = v;
              arm();
            },
            onKeyEvent: (_, e) {
              if (e is KeyDownEvent &&
                  e.logicalKey == LogicalKeyboardKey.escape) {
                unawaited(dismiss());
                return KeyEventResult.handled;
              }
              return KeyEventResult.ignored;
            },
            child: GestureDetector(
              onVerticalDragStart: (_) => drag = 0,
              onVerticalDragUpdate: (d) => drag += d.primaryDelta ?? 0,
              onVerticalDragEnd: (d) {
                if (drag < -24 || (d.primaryVelocity ?? 0) < -100) {
                  unawaited(dismiss());
                }
              },
              child: SizedBox(
                key: const Key('notification-toast-surface'),
                width: width,
                height: height,
                child: ClipRRect(
                  borderRadius: BorderRadius.circular(expanded ? 22 : 28),
                  child: LiquidGlass(
                    radius: expanded ? 22 : 28,
                    child: ClipRect(
                      child: OverflowBox(
                        alignment: Alignment.topCenter,
                        minWidth: targetWidth,
                        maxWidth: targetWidth,
                        minHeight: targetHeight,
                        maxHeight: targetHeight,
                        child: Opacity(
                          opacity:
                              stage(entry.value, .78, 1) *
                              (expanded ? stage(detail.value, .55, 1) : 1),
                          child: Material(
                            color: Colors.transparent,
                            child: expanded
                                ? Padding(
                                    padding: const EdgeInsets.all(16),
                                    child: Column(
                                      crossAxisAlignment:
                                          CrossAxisAlignment.start,
                                      children: [
                                        Row(
                                          children: [
                                            Icon(icon(notice), size: 20),
                                            const SizedBox(width: 10),
                                            Expanded(
                                              child: Text(
                                                title(notice),
                                                maxLines: 2,
                                                overflow: TextOverflow.ellipsis,
                                                style: const TextStyle(
                                                  fontWeight: FontWeight.w600,
                                                ),
                                              ),
                                            ),
                                            IconButton(
                                              tooltip: 'Dismiss notification',
                                              onPressed: dismiss,
                                              icon: const Icon(
                                                Icons.close,
                                                size: 18,
                                              ),
                                            ),
                                          ],
                                        ),
                                        const SizedBox(height: 8),
                                        Expanded(
                                          child: SingleChildScrollView(
                                            child: SelectableText(
                                              notice['details'] as String? ??
                                                  '${attentionLabel(notice['kind'] as String)}. Open the task to inspect its current result or decision.',
                                            ),
                                          ),
                                        ),
                                        if (host.actions[notice['id']] != null)
                                          Align(
                                            alignment: Alignment.centerRight,
                                            child: TextButton(
                                              onPressed:
                                                  host.actions[notice['id']],
                                              child: const Text('Retry'),
                                            ),
                                          ),
                                        if (notice['kind'] != 'app')
                                          Align(
                                            alignment: Alignment.centerRight,
                                            child: TextButton(
                                              onPressed: () async {
                                                final id =
                                                    notice['id'] as String;
                                                await dismiss();
                                                await host.open(id);
                                              },
                                              child: const Text('Open task'),
                                            ),
                                          ),
                                      ],
                                    ),
                                  )
                                : InkWell(
                                    key: const Key('notification-toast-open'),
                                    borderRadius: BorderRadius.circular(28),
                                    onTap: () => host.showDetails(notice),
                                    child: Padding(
                                      padding: const EdgeInsets.only(
                                        left: 16,
                                        right: 4,
                                      ),
                                      child: Row(
                                        children: [
                                          Icon(icon(notice), size: 20),
                                          const SizedBox(width: 12),
                                          Expanded(
                                            child: Text(
                                              title(notice),
                                              maxLines: 1,
                                              overflow: TextOverflow.ellipsis,
                                              style: const TextStyle(
                                                fontWeight: FontWeight.w600,
                                              ),
                                            ),
                                          ),
                                          IconButton(
                                            tooltip: 'Dismiss notification',
                                            onPressed: dismiss,
                                            icon: const Icon(
                                              Icons.close,
                                              size: 16,
                                            ),
                                          ),
                                        ],
                                      ),
                                    ),
                                  ),
                          ),
                        ),
                      ),
                    ),
                  ),
                ),
              ),
            ),
          ),
        );
      },
    );
  }
}
