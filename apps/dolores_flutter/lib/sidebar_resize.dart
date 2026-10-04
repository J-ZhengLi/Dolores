import 'package:flutter/material.dart';
import 'package:flutter/services.dart';

import 'theme.dart';

/// A generous pointer target around the quiet one-pixel sidebar divider.
class SidebarResizeHandle extends StatefulWidget {
  final double width;
  final double maxWidth;
  final ValueChanged<double> onDelta;
  final VoidCallback onReset;
  const SidebarResizeHandle({
    super.key,
    required this.width,
    required this.maxWidth,
    required this.onDelta,
    required this.onReset,
  });

  @override
  State<SidebarResizeHandle> createState() => _SidebarResizeHandleState();
}

class _SidebarResizeHandleState extends State<SidebarResizeHandle> {
  final focus = FocusNode();
  bool hovered = false, focused = false, dragging = false;

  @override
  void dispose() {
    focus.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    final p = Palette(Theme.of(context).brightness == Brightness.dark);
    return Semantics(
      label: 'Resize sidebar',
      value: '${widget.width.round()} pixels',
      increasedValue:
          '${(widget.width + 16).clamp(UiTokens.sidebarMinWidth, widget.maxWidth).round()} pixels',
      decreasedValue:
          '${(widget.width - 16).clamp(UiTokens.sidebarMinWidth, widget.maxWidth).round()} pixels',
      hint: 'Drag or use Left and Right arrows. Double-click or Home to reset.',
      onIncrease: () => widget.onDelta(16),
      onDecrease: () => widget.onDelta(-16),
      child: CallbackShortcuts(
        bindings: {
          const SingleActivator(LogicalKeyboardKey.arrowLeft): () =>
              widget.onDelta(-16),
          const SingleActivator(LogicalKeyboardKey.arrowRight): () =>
              widget.onDelta(16),
          const SingleActivator(LogicalKeyboardKey.home): widget.onReset,
        },
        child: Focus(
          focusNode: focus,
          onFocusChange: (value) => setState(() => focused = value),
          child: MouseRegion(
            cursor: SystemMouseCursors.resizeLeftRight,
            onEnter: (_) => setState(() => hovered = true),
            onExit: (_) => setState(() => hovered = false),
            child: GestureDetector(
              key: const Key('sidebar-resize'),
              behavior: HitTestBehavior.opaque,
              onTap: focus.requestFocus,
              onHorizontalDragStart: (_) {
                focus.requestFocus();
                setState(() => dragging = true);
              },
              onHorizontalDragUpdate: (details) =>
                  widget.onDelta(details.primaryDelta!),
              onHorizontalDragEnd: (_) => setState(() => dragging = false),
              onHorizontalDragCancel: () => setState(() => dragging = false),
              onDoubleTap: widget.onReset,
              child: Center(
                child: Container(
                  key: const Key('sidebar-divider'),
                  width: dragging || hovered || focused ? 2 : 1,
                  color: dragging ? p.accent : p.border,
                ),
              ),
            ),
          ),
        ),
      ),
    );
  }
}
