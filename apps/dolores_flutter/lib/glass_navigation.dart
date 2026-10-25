import 'package:flutter/physics.dart';

import 'package:flutter/material.dart';

import 'theme.dart';
import 'liquid_glass.dart';

/// A single moving selection surface. Labels and hit targets remain stationary.
class GlassNavigation extends StatelessWidget {
  final int selected;
  final List<Widget> children;
  final double rowHeight;
  final Axis axis;
  final double crossAxisExtent;
  final bool blur;
  const GlassNavigation({
    super.key,
    required this.selected,
    required this.children,
    this.rowHeight = 48,
    this.axis = Axis.vertical,
    this.crossAxisExtent = 56,
    this.blur = true,
  });

  @override
  Widget build(BuildContext context) {
    final media = MediaQuery.of(context);
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
              child: _TravelingLens(
                target: selected * rowHeight,
                extent: rowHeight,
                maximum: (children.length - 1) * rowHeight,
                horizontal: horizontal,
                reduce: reduce,
                child: IgnorePointer(
                  child: LiquidGlass(
                    lens: true,
                    blur: blur,
                    filterKey: const Key('glass-selection-position'),
                    child: const SizedBox.expand(),
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

/// Retarget both position and velocity; only paint changes during travel.
class _TravelingLens extends StatefulWidget {
  final double target, extent, maximum;
  final bool horizontal, reduce;
  final Widget child;
  const _TravelingLens({
    required this.target,
    required this.extent,
    required this.maximum,
    required this.horizontal,
    required this.reduce,
    required this.child,
  });

  @override
  State<_TravelingLens> createState() => _TravelingLensState();
}

class _TravelingLensState extends State<_TravelingLens>
    with SingleTickerProviderStateMixin {
  late final motion = AnimationController.unbounded(
    vsync: this,
    value: widget.target,
  );

  @override
  void didUpdateWidget(_TravelingLens oldWidget) {
    super.didUpdateWidget(oldWidget);
    if (widget.reduce ||
        oldWidget.horizontal != widget.horizontal ||
        oldWidget.extent != widget.extent) {
      motion.value = widget.target;
    } else if (oldWidget.target != widget.target) {
      motion.animateWith(
        SpringSimulation(
          SpringDescription.withDampingRatio(
            mass: 1,
            stiffness: 480,
            ratio: .86,
          ),
          motion.value,
          widget.target,
          motion.velocity,
          snapToEnd: true,
          tolerance: const Tolerance(distance: .1, velocity: 1),
        ),
      );
    }
  }

  @override
  void dispose() {
    motion.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) => AnimatedBuilder(
    animation: motion,
    child: widget.child,
    builder: (context, child) {
      final position = motion.value.clamp(0.0, widget.maximum);
      final stretch = widget.reduce
          ? 0.0
          : (motion.velocity.abs() / widget.extent * .012).clamp(0.0, .065);
      return Transform.translate(
        offset: widget.horizontal ? Offset(position, 0) : Offset(0, position),
        child: Transform.scale(
          scaleX: widget.horizontal ? 1 + stretch : 1 - stretch * .35,
          scaleY: widget.horizontal ? 1 - stretch * .35 : 1 + stretch,
          child: child,
        ),
      );
    },
  );
}

/// Hover is a fine outline; the moving glass is the only selection fill.
class _NavigationFeedback extends StatefulWidget {
  final Widget child;
  const _NavigationFeedback({required this.child});
  @override
  State<_NavigationFeedback> createState() => _NavigationFeedbackState();
}

class _NavigationFeedbackState extends State<_NavigationFeedback> {
  bool hovered = false, focused = false, pressed = false;
  @override
  Widget build(BuildContext context) {
    final p = Palette(Theme.of(context).brightness == Brightness.dark);
    final reduced =
        MediaQuery.disableAnimationsOf(context) ||
        MediaQuery.accessibleNavigationOf(context);
    return MouseRegion(
      onEnter: (_) => setState(() => hovered = true),
      onExit: (_) => setState(() => hovered = false),
      child: Focus(
        canRequestFocus: false,
        skipTraversal: true,
        onFocusChange: (value) => setState(() => focused = value),
        child: DecoratedBox(
          decoration: BoxDecoration(
            borderRadius: BorderRadius.circular(16),
            border: Border.all(
              color: focused
                  ? p.accent
                  : hovered
                  ? p.muted.withValues(alpha: .25)
                  : Colors.transparent,
              width: focused ? 2 : 1,
            ),
          ),
          child: Listener(
            onPointerDown: (_) => setState(() => pressed = true),
            onPointerUp: (_) => setState(() => pressed = false),
            onPointerCancel: (_) => setState(() => pressed = false),
            child: AnimatedScale(
              scale: pressed && !reduced ? .97 : 1,
              duration: reduced
                  ? Duration.zero
                  : const Duration(milliseconds: 140),
              curve: Curves.easeOutCubic,
              child: widget.child,
            ),
          ),
        ),
      ),
    );
  }
}
