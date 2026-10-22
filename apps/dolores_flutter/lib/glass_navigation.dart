import 'dart:ui' show ImageFilter;

import 'package:flutter/material.dart';

import 'theme.dart';

/// A single moving selection surface. Labels and hit targets remain stationary.
class GlassNavigation extends StatelessWidget {
  final int selected;
  final List<Widget> children;
  final double rowHeight;
  final Axis axis;
  final double crossAxisExtent;
  const GlassNavigation({
    super.key,
    required this.selected,
    required this.children,
    this.rowHeight = 48,
    this.axis = Axis.vertical,
    this.crossAxisExtent = 56,
  });

  @override
  Widget build(BuildContext context) {
    final media = MediaQuery.of(context);
    final p = Palette(Theme.of(context).brightness == Brightness.dark);
    final reduce = media.disableAnimations || media.accessibleNavigation;
    final theme = Theme.of(context);
    final horizontal = axis == Axis.horizontal;
    const feedback = ButtonStyle(
      overlayColor: WidgetStatePropertyAll(Colors.transparent),
      splashFactory: NoSplash.splashFactory,
    );
    return Theme(
      data: theme.copyWith(
        textButtonTheme: TextButtonThemeData(
          style:
              theme.textButtonTheme.style?.copyWith(
                overlayColor: feedback.overlayColor,
                splashFactory: feedback.splashFactory,
              ) ??
              feedback,
        ),
        iconButtonTheme: IconButtonThemeData(
          style:
              theme.iconButtonTheme.style?.copyWith(
                overlayColor: feedback.overlayColor,
                splashFactory: feedback.splashFactory,
              ) ??
              feedback,
        ),
      ),
      child: Stack(
        children: [
          if (selected >= 0 && selected < children.length)
            Positioned(
              top: 0,
              left: 0,
              right: horizontal ? null : 0,
              width: horizontal ? rowHeight - 4 : null,
              height: horizontal ? crossAxisExtent - 4 : rowHeight - 4,
              child: TweenAnimationBuilder<double>(
                tween: Tween(end: selected * rowHeight),
                duration: reduce
                    ? Duration.zero
                    : const Duration(milliseconds: 360),
                curve: const Cubic(.2, .8, .25, 1.05),
                builder: (context, offset, child) => Transform.translate(
                  offset: horizontal ? Offset(offset, 0) : Offset(0, offset),
                  child: child,
                ),
                child: IgnorePointer(
                  child: ClipRRect(
                    borderRadius: BorderRadius.circular(14),
                    child: BackdropFilter(
                      key: const Key('glass-selection-position'),
                      enabled: !media.highContrast && !reduce,
                      filter: ImageFilter.blur(sigmaX: 14, sigmaY: 14),
                      child: DecoratedBox(
                        decoration: BoxDecoration(
                          color: media.highContrast || reduce ? p.soft : null,
                          gradient: media.highContrast || reduce
                              ? null
                              : LinearGradient(
                                  begin: Alignment.topLeft,
                                  end: Alignment.bottomRight,
                                  colors: p.dark
                                      ? const [
                                          Color(0x25ffffff),
                                          Color(0x0affffff),
                                          Color(0x18ffffff),
                                        ]
                                      : const [
                                          Color(0xcfffffff),
                                          Color(0x55edf6ff),
                                          Color(0xaaffffff),
                                        ],
                                ),
                          borderRadius: BorderRadius.circular(14),
                          border: Border.all(
                            color: media.highContrast
                                ? p.text
                                : Colors.white.withValues(
                                    alpha: p.dark ? .18 : .85,
                                  ),
                          ),
                          boxShadow: [
                            BoxShadow(
                              color: p.accent.withValues(alpha: .07),
                              blurRadius: 10,
                              offset: const Offset(0, 3),
                            ),
                          ],
                        ),
                      ),
                    ),
                  ),
                ),
              ),
            ),
          Flex(
            direction: axis,
            mainAxisSize: MainAxisSize.min,
            children: [
              for (var i = 0; i < children.length; i++)
                Semantics(
                  selected: selected == i,
                  child: SizedBox(
                    height: horizontal ? crossAxisExtent : rowHeight,
                    width: horizontal ? rowHeight : double.infinity,
                    child: Padding(
                      padding: horizontal
                          ? const EdgeInsets.only(right: 4, bottom: 4)
                          : const EdgeInsets.only(bottom: 4),
                      child: _NavigationFeedback(child: children[i]),
                    ),
                  ),
                ),
            ],
          ),
        ],
      ),
    );
  }
}

/// Hover is a fine outline; the moving glass is the only selection fill.
class _NavigationFeedback extends StatefulWidget {
  final Widget child;
  const _NavigationFeedback({required this.child});
  @override
  State<_NavigationFeedback> createState() => _NavigationFeedbackState();
}

class _NavigationFeedbackState extends State<_NavigationFeedback> {
  bool hovered = false, focused = false;
  @override
  Widget build(BuildContext context) {
    final p = Palette(Theme.of(context).brightness == Brightness.dark);
    return MouseRegion(
      onEnter: (_) => setState(() => hovered = true),
      onExit: (_) => setState(() => hovered = false),
      child: Focus(
        canRequestFocus: false,
        skipTraversal: true,
        onFocusChange: (value) => setState(() => focused = value),
        child: DecoratedBox(
          decoration: BoxDecoration(
            borderRadius: BorderRadius.circular(14),
            border: Border.all(
              color: focused
                  ? p.accent
                  : hovered
                  ? p.muted.withValues(alpha: .25)
                  : Colors.transparent,
              width: focused ? 2 : 1,
            ),
          ),
          child: widget.child,
        ),
      ),
    );
  }
}
