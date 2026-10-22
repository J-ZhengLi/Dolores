import 'dart:ui' show ImageFilter;

import 'package:flutter/material.dart';

import 'theme.dart';

/// A single moving selection surface. Labels and hit targets remain stationary.
class GlassNavigation extends StatelessWidget {
  final int selected;
  final List<Widget> children;
  final double rowHeight;
  const GlassNavigation({
    super.key,
    required this.selected,
    required this.children,
    this.rowHeight = 48,
  });

  @override
  Widget build(BuildContext context) {
    final media = MediaQuery.of(context);
    final p = Palette(Theme.of(context).brightness == Brightness.dark);
    final reduce = media.disableAnimations || media.accessibleNavigation;
    return Stack(
      children: [
        if (selected >= 0 && selected < children.length)
          AnimatedPositioned(
            key: const Key('glass-selection-position'),
            duration: reduce
                ? Duration.zero
                : const Duration(milliseconds: 420),
            curve: const Cubic(.2, .8, .25, 1.05),
            top: selected * rowHeight,
            left: 0,
            right: 0,
            height: rowHeight - 4,
            child: IgnorePointer(
              child: ClipRRect(
                borderRadius: BorderRadius.circular(14),
                child: BackdropFilter(
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
        Column(
          mainAxisSize: MainAxisSize.min,
          children: [
            for (var i = 0; i < children.length; i++)
              Semantics(
                selected: selected == i,
                child: SizedBox(
                  height: rowHeight,
                  width: double.infinity,
                  child: Padding(
                    padding: const EdgeInsets.only(bottom: 4),
                    child: children[i],
                  ),
                ),
              ),
          ],
        ),
      ],
    );
  }
}
