import 'package:flutter/material.dart';

/// Retains mounted pages and drafts while transitioning only their paint layers.
class PageTransitionStack extends StatefulWidget {
  final String selected;
  final Map<String, Widget> pages;
  const PageTransitionStack({
    super.key,
    required this.selected,
    required this.pages,
  });
  @override
  State<PageTransitionStack> createState() => _PageTransitionStackState();
}

class _PageTransitionStackState extends State<PageTransitionStack>
    with SingleTickerProviderStateMixin {
  late final controller = AnimationController(
    vsync: this,
    duration: const Duration(milliseconds: 240),
    value: 1,
  );
  bool get reduced =>
      MediaQuery.disableAnimationsOf(context) ||
      MediaQuery.accessibleNavigationOf(context);
  @override
  void didUpdateWidget(PageTransitionStack oldWidget) {
    super.didUpdateWidget(oldWidget);
    if (widget.selected != oldWidget.selected) {
      if (reduced) {
        controller.value = 1;
      } else {
        controller.forward(from: 0);
      }
    }
  }

  @override
  void didChangeDependencies() {
    super.didChangeDependencies();
    if (reduced) {
      controller.value = 1;
    }
  }

  @override
  void dispose() {
    controller.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) => ClipRect(
    child: Stack(
      fit: StackFit.expand,
      children: [
        for (final entry in widget.pages.entries)
          Offstage(
            key: ValueKey(entry.key),
            offstage: entry.key != widget.selected,
            child: TickerMode(
              enabled: entry.key == widget.selected,
              child: IgnorePointer(
                ignoring: entry.key != widget.selected,
                child: ExcludeFocus(
                  excluding: entry.key != widget.selected,
                  child: ExcludeSemantics(
                    excluding: entry.key != widget.selected,
                    child: AnimatedBuilder(
                      animation: controller,
                      child: Semantics(
                        container: true,
                        explicitChildNodes: true,
                        child: RepaintBoundary(child: entry.value),
                      ),
                      builder: (context, child) {
                        final active = entry.key == widget.selected;
                        final progress = Curves.easeOutCubic.transform(
                          controller.value,
                        );
                        return Opacity(
                          alwaysIncludeSemantics: true,
                          opacity: active ? progress : 1 - progress,
                          child: Transform.translate(
                            offset: Offset(active ? 8 * (1 - progress) : 0, 0),
                            child: child,
                          ),
                        );
                      },
                    ),
                  ),
                ),
              ),
            ),
          ),
      ],
    ),
  );
}
