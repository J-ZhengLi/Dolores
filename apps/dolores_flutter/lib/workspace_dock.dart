import 'dart:math' as math;
import 'dart:async';

import 'package:flutter/material.dart';
import 'package:flutter/services.dart';

import 'glass_navigation.dart';
import 'theme.dart';

class DockItem {
  final Key key;
  final String label;
  final IconData icon;
  final VoidCallback onPressed;
  const DockItem({
    required this.key,
    required this.label,
    required this.icon,
    required this.onPressed,
  });
}

/// Floating navigation reveals at the bottom edge without resizing the workspace.
class WorkspaceDock extends StatefulWidget {
  final int selected;
  final List<DockItem> items;
  const WorkspaceDock({super.key, required this.selected, required this.items});

  @override
  State<WorkspaceDock> createState() => WorkspaceDockState();
}

class WorkspaceDockState extends State<WorkspaceDock> {
  final focus = FocusNode();
  final firstItemFocus = FocusNode();
  FocusNode? previousFocus;
  Timer? timer;
  bool visible = true, hovered = false, keyboard = false;

  @override
  void initState() {
    super.initState();
    scheduleHide(const Duration(seconds: 2));
  }

  void scheduleHide([Duration delay = const Duration(milliseconds: 650)]) {
    timer?.cancel();
    timer = Timer(delay, () {
      if (mounted && !hovered && !keyboard) setState(() => visible = false);
    });
  }

  void reveal({bool fromKeyboard = false}) {
    timer?.cancel();
    if (fromKeyboard && !focus.hasFocus) {
      previousFocus = FocusManager.instance.primaryFocus;
    }
    setState(() {
      visible = true;
      keyboard = fromKeyboard;
    });
    if (fromKeyboard) {
      WidgetsBinding.instance.addPostFrameCallback((_) {
        if (mounted) firstItemFocus.requestFocus();
      });
    } else {
      scheduleHide(const Duration(seconds: 2));
    }
  }

  @override
  void dispose() {
    timer?.cancel();
    focus.dispose();
    firstItemFocus.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    final reduced =
        MediaQuery.disableAnimationsOf(context) ||
        MediaQuery.accessibleNavigationOf(context);
    final duration = reduced
        ? Duration.zero
        : const Duration(milliseconds: 220);
    final p = Palette(Theme.of(context).brightness == Brightness.dark);
    return SizedBox(
      height: 88 + MediaQuery.paddingOf(context).bottom,
      child: Stack(
        alignment: Alignment.bottomCenter,
        children: [
          AnimatedSlide(
            offset: visible ? Offset.zero : const Offset(0, 1.15),
            duration: duration,
            curve: Curves.easeOutCubic,
            child: AnimatedOpacity(
              opacity: visible ? 1 : 0,
              duration: duration,
              child: IgnorePointer(
                ignoring: !visible,
                child: ExcludeSemantics(
                  excluding: !visible,
                  child: ExcludeFocus(
                    excluding: !visible,
                    child: FocusTraversalGroup(
                      child: Focus(
                        focusNode: focus,
                        skipTraversal: true,
                        onFocusChange: (value) {
                          if (value && !hovered) {
                            setState(() {
                              visible = true;
                              keyboard = true;
                            });
                            timer?.cancel();
                          } else if (!value) {
                            keyboard = false;
                            scheduleHide();
                          }
                        },
                        onKeyEvent: (node, event) {
                          if (event is! KeyDownEvent) {
                            return KeyEventResult.ignored;
                          }
                          if (event.logicalKey == LogicalKeyboardKey.escape) {
                            focus.unfocus();
                            if (previousFocus?.context != null) {
                              previousFocus!.requestFocus();
                            }
                            setState(() {
                              keyboard = false;
                              visible = false;
                            });
                            return KeyEventResult.handled;
                          }
                          keyboard = true;
                          if (event.logicalKey ==
                              LogicalKeyboardKey.arrowRight) {
                            FocusScope.of(context).nextFocus();
                            return KeyEventResult.handled;
                          }
                          if (event.logicalKey ==
                              LogicalKeyboardKey.arrowLeft) {
                            FocusScope.of(context).previousFocus();
                            return KeyEventResult.handled;
                          }
                          return KeyEventResult.ignored;
                        },
                        child: Align(
                          alignment: Alignment.bottomCenter,
                          heightFactor: 1,
                          child: ConstrainedBox(
                            constraints: const BoxConstraints(maxWidth: 760),
                            child: MouseRegion(
                              onEnter: (_) {
                                hovered = true;
                                timer?.cancel();
                              },
                              onExit: (_) {
                                hovered = false;
                                scheduleHide();
                              },
                              child: Listener(
                                onPointerDown: (_) => keyboard = false,
                                child: _DockSurface(
                                  selected: widget.selected,
                                  items: widget.items,
                                  firstItemFocus: firstItemFocus,
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
          if (!visible)
            Align(
              alignment: Alignment.bottomCenter,
              child: ConstrainedBox(
                constraints: const BoxConstraints(maxWidth: 760),
                child: MouseRegion(
                  onEnter: (_) => reveal(),
                  child: Semantics(
                    button: true,
                    label:
                        'Show navigation dock: ${widget.items.map((item) => item.label).join(', ')}. Control Alt D.',
                    child: GestureDetector(
                      key: const Key('dock-reveal'),
                      behavior: HitTestBehavior.opaque,
                      onTap: () => reveal(),
                      child: SizedBox(
                        height: 10 + MediaQuery.paddingOf(context).bottom,
                        width: double.infinity,
                        child: Center(
                          child: Container(
                            width: 40,
                            height: 3,
                            decoration: BoxDecoration(
                              color: p.muted.withValues(alpha: .45),
                              borderRadius: BorderRadius.circular(2),
                            ),
                          ),
                        ),
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
}

class _DockSurface extends StatelessWidget {
  final int selected;
  final List<DockItem> items;
  final FocusNode firstItemFocus;
  const _DockSurface({
    required this.selected,
    required this.items,
    required this.firstItemFocus,
  });

  @override
  Widget build(BuildContext context) {
    final p = Palette(Theme.of(context).brightness == Brightness.dark);
    return SafeArea(
      top: false,
      child: LayoutBuilder(
        builder: (context, constraints) {
          final labels =
              constraints.maxWidth >= 720 &&
              MediaQuery.textScalerOf(context).scale(12) <= 16;
          final extent = math.min(
            labels ? 104.0 : 52.0,
            (constraints.maxWidth - 36) / items.length,
          );
          return Padding(
            padding: const EdgeInsets.fromLTRB(12, 8, 12, 10),
            child: Center(
              heightFactor: 1,
              child: Container(
                key: const Key('workspace-dock'),
                padding: const EdgeInsets.fromLTRB(6, 6, 2, 2),
                decoration: p.sidebarDecoration.copyWith(
                  borderRadius: BorderRadius.circular(20),
                  border: Border.all(color: p.border),
                  boxShadow: [
                    BoxShadow(
                      color: Colors.black.withValues(alpha: p.dark ? .16 : .05),
                      blurRadius: 16,
                      offset: const Offset(0, 3),
                    ),
                  ],
                ),
                child: GlassNavigation(
                  axis: Axis.horizontal,
                  selected: selected,
                  rowHeight: extent,
                  crossAxisExtent: labels ? 56 : 48,
                  children: [
                    for (final item in items)
                      Tooltip(
                        message: item.label,
                        child: TextButton(
                          key: item.key,
                          focusNode: identical(item, items.first)
                              ? firstItemFocus
                              : null,
                          onPressed: item.onPressed,
                          style: TextButton.styleFrom(
                            padding: const EdgeInsets.symmetric(horizontal: 2),
                            minimumSize: Size.zero,
                          ),
                          child: Column(
                            mainAxisAlignment: MainAxisAlignment.center,
                            children: [
                              Icon(item.icon, size: 20),
                              if (labels) ...[
                                const SizedBox(height: 4),
                                Text(
                                  item.label,
                                  maxLines: 1,
                                  style: Theme.of(context).textTheme.labelSmall,
                                ),
                              ],
                            ],
                          ),
                        ),
                      ),
                  ],
                ),
              ),
            ),
          );
        },
      ),
    );
  }
}
